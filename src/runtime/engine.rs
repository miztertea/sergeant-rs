//! The staged workflow engine (proposal §12, §25).
//!
//! The engine is the only thing that moves a run: it materializes a surface,
//! pins a workflow, routes to a backend, enters stages in order, and reacts to
//! what the backend explicitly says. Its whole design is one rule from §25:
//!
//! > `native turn completed ≠ work completed`
//! > `native process alive ≠ work active`
//! > `native process dead ≠ work failed`
//!
//! So [`Observation::native`](crate::backend::Observation::native) — the
//! liveness evidence — is never read by [`Engine::resume`] when deciding a
//! transition. Only [`BackendSignal`] moves state, and the one place native
//! evidence *is* consulted is [`NativeState::Unknown`], where it can produce
//! exactly one outcome: fail closed to `blocked` with the evidence recorded.
//!
//! Execution is driven synchronously on the caller's request, which is
//! deterministic and sufficient for M3's only backend (in-process, §37). A
//! backend that blocks on a real process needs a scheduler; that arrives with
//! the backends that need it, not before (§4's non-goal, and the M3 contract's
//! "no generalized DAG scheduling").

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::api::{Core, CoreError};
use crate::backend::docker::DOCKER_BACKEND_NAME;
use crate::backend::{
    Backend, BackendError, BackendRegistry, BackendSignal, Completion, Deferred, ExecutionHandle,
    NativeState, Observation, PreparedExecution, ResumeRequest, StartRequest,
};
use crate::domain::estate::{
    Estate, EstateError, EstateRootError, InstructionIdentity, InstructionPolicy, RepositorySpec,
};
use crate::domain::event::{EventDraft, EventSource};
use crate::domain::execution::{
    ExecutionRecord, ExecutionReservation, KIND_EXECUTION_ABANDONED, KIND_EXECUTION_RECONCILED,
    KIND_EXECUTION_RESERVED, KIND_EXECUTION_STARTED, KIND_EXECUTION_STOPPED, ReconcileDisposition,
};
use crate::domain::profile::Profile;
use crate::domain::work::{
    KIND_WORK_BLOCKED, KIND_WORK_COMPLETED, KIND_WORK_FAILED, KIND_WORK_NEEDS_INPUT,
    KIND_WORK_RESUMED, KIND_WORK_STARTED, KIND_WORK_WAITING, Work, WorkState,
};
use crate::domain::workflow::{
    DEFAULT_WORKFLOW, KIND_STAGE_BLOCKED, KIND_STAGE_CANCELED, KIND_STAGE_COMPLETED,
    KIND_STAGE_ENTERED, KIND_STAGE_FAILED, KIND_STAGE_INPUT_RECEIVED, KIND_STAGE_NEEDS_INPUT,
    KIND_STAGE_RESUMED, KIND_STAGE_WAITING, KIND_WORKFLOW_BOUND, StageBinding, StageDefinition,
    StageKind, StageStatus, WorkflowDefinition, WorkflowError,
};
use crate::runtime::preflight::{self, GitPreflight, PreflightRefusal};
use crate::runtime::projection::{WorkRegistry, WorkRun, is_absorbing};
use crate::runtime::router::{Route, RouteError, RouteInputs, route, route_stage};
use crate::runtime::surface::{
    KIND_SURFACE_MATERIALIZED, KIND_SURFACE_MATERIALIZING, KIND_SURFACE_TORN_DOWN,
    RepositoryBinding, SURFACES_DIR, SurfaceError, SurfacePlan, TeardownReport, WorkSurface,
    materialize_admitted, rematerialize, teardown,
};

/// Everything a submission says about how it wants to run.
#[derive(Debug, Clone, Copy, Default)]
pub struct SubmitContext<'a> {
    /// Client's working directory; estate discovery starts here (§9).
    /// `None` means the client offered no repository context, so there is
    /// nothing to materialize and the work stays `pending`.
    pub cwd: Option<&'a Path>,
    /// `origin.client` (§13).
    pub origin_client: Option<&'a str>,
    /// Explicitly requested backend (§13's first tier).
    pub backend: Option<&'a str>,
    /// Requested workflow name.
    pub workflow: Option<&'a str>,
    /// Requested profile name (§14).
    pub profile: Option<&'a str>,
    /// §7.1/§13.3's structured scope request, the CLI/API's `scope.repos` —
    /// explicit repository names, as submitted. Combined with `group`
    /// (union, dedup) by [`Engine::resolve_scope`]; never expanded by any
    /// client, per §7.2's "a client surface adds usability, never
    /// functionality".
    pub repos: &'a [String],
    /// `scope.group` — a declared `[group.<name>]` to expand, resolved
    /// against *this* daemon's own bound manifest (§7.2), not by the caller.
    pub group: Option<&'a str>,
    /// `scope.all` — an explicit whole-estate selection (§7.1's third scope
    /// form). Owner ruling (2026-08-20): combined with a nonempty `repos`
    /// and/or a set `group`, this is refused
    /// ([`EngineError::ConflictingScope`]) rather than silently winning over
    /// them — [`Engine::resolve_scope`] only resolves to every declared
    /// repository when `repos`/`group` are both empty/unset.
    pub all: bool,
    /// The id the Work *would* be given, for §8.1's checks 9 and 10 — both
    /// of which are questions about `sergeant/<work-id>` and
    /// `<surfaces_root>/<work-id>/<repo>`, so the id has to exist before the
    /// record does. The caller mints it and reuses it for the `Work` it
    /// creates when planning succeeds; a refusal discards it, which costs
    /// nothing and is the whole point of asking here.
    ///
    /// `None` skips §8.1 entirely: no id, no checks 9/10, nothing admitted.
    /// That is the shape every non-submission caller of
    /// [`Engine::plan`](Self) has (routing probes, the runtime's own
    /// fixtures), and it must not silently become an unadmitted submission —
    /// which is why [`StartPlan::preflight`] is an `Option`, not an empty
    /// `Vec` that would read as "admitted, nothing found".
    pub work_id: Option<&'a str>,
    /// §8.3's `--override-git-preflight`, exactly as the submission carried
    /// it. **Never a default and never read from configuration**: this is a
    /// borrowed request field with no other source, so there is nowhere for a
    /// run template or an estate manifest to set it from.
    pub override_git_preflight: bool,
}

/// A resolved, side-effect-free plan for starting a run.
///
/// Planning is separated from starting so that everything which can be
/// *decided* — estate topology, workflow content, routing, profile — is
/// decided before a Work record exists. A submission that cannot be routed is
/// rejected with §13's available options instead of creating work that
/// immediately dies.
#[derive(Debug, Clone)]
pub struct StartPlan {
    /// The discovered estate.
    pub estate: Estate,
    /// Repositories this run targets.
    pub repositories: Vec<RepositorySpec>,
    /// Where this run's surface will be materialized (R-MVP1-1):
    /// `estate.surfaces_dir` when the manifest declared one, else the
    /// engine's own default (`SGT_SURFACES_DIR`, else `<data_dir>/surfaces`).
    /// Resolved once, here, so a mid-flight manifest edit cannot move a
    /// running Work's surface out from under it.
    pub surfaces_root: PathBuf,
    /// The resolved workflow, pinned for the life of the run.
    pub workflow: WorkflowDefinition,
    /// The routing decision and the tier that made it.
    pub route: Route,
    /// The launch profile, if one was selected.
    pub profile: Option<Profile>,
    /// One resolved executor decision per stage (§12.5, §17.5's preflight).
    /// Pinned into `workflow.bound` and read by every later stage entry, so a
    /// retry and a restart reconstruct the same decision.
    pub stage_bindings: Vec<StageBinding>,
    /// §8.1's Git admission, when this plan was made for a real submission
    /// (`SubmitContext::work_id` was set).
    ///
    /// `None` means no admission was performed — never "admitted and found
    /// nothing". Materialization reads the pinned base out of this rather
    /// than re-asking each mount, so the distinction decides whether a Work
    /// is based on the commit that was *judged* or on whatever HEAD says by
    /// the time `git worktree add` runs.
    pub preflight: Option<GitPreflight>,
}

/// Why `target_work_id`'s branch cannot (yet) be taken over by a gate Work
/// attaching to it (§8.6 investigation, Mechanism A —
/// `docs/gauntlet/runs/foundation-1/8.6-gate-branch-binding.md`). Every
/// variant is a fail-closed refusal named with a reason: dispatch must never
/// guess or race the target's own teardown (ADR 0005 §4.3, "ambiguity fails
/// closed").
#[derive(Debug, thiserror::Error)]
pub enum BranchTakeoverError {
    /// No such Work.
    #[error("work {work_id} does not exist")]
    TargetNotFound {
        /// The Work id that was looked up.
        work_id: String,
    },
    /// The target has not reached an *absorbing* terminal state
    /// ([`is_absorbing`]) — `pending`/`active`/`waiting`/etc. mean it may
    /// still be writing to its worktree, and `failed` is deliberately
    /// excluded even though `WorkState`'s own doc comment calls it
    /// "terminal": `failed` is retryable, and a retry rematerializes the
    /// target's own worktree back onto the very branch a gate surface would
    /// already have attached to — the exact race this precondition exists to
    /// prevent, not a state this check happens to have overlooked.
    #[error(
        "work {work_id} is {state}, not completed or canceled; a gate cannot take over a branch \
         that may still be written to (active/waiting/etc.) or resurrected onto by a retry \
         (failed)"
    )]
    TargetNotTerminal {
        /// The Work id that was checked.
        work_id: String,
        /// Its actual state.
        state: WorkState,
    },
    /// The target never materialized a surface — there is no branch to take
    /// over. (A Work submitted with no repository context, or one that
    /// never got far enough to bind a workflow, both read this way: no `run`
    /// projected, or a `run` with `surface: None`.)
    #[error("work {work_id} never materialized a surface; there is no branch to take over")]
    NoSurface {
        /// The Work id that was checked.
        work_id: String,
    },
    /// The target reached a terminal state but its surface has not been torn
    /// down — yet, or possibly ever. `Engine::retire_run`'s own ordering
    /// (Work state committed, then the stage marked, then the backend asked
    /// to stop, then the surface torn down) makes this a real window, not a
    /// hypothetical one: a terminal Work whose teardown crashed or is still
    /// in flight reads exactly like this.
    #[error("work {work_id} is terminal but its surface has not been torn down yet")]
    SurfaceNotTornDown {
        /// The Work id that was checked.
        work_id: String,
    },
    /// Teardown ran but did not report every binding `Removed` — at least
    /// one worktree is still attached to (or missing/errored on) the branch
    /// a gate surface would need. Never forced past: teardown's own
    /// fail-closed disposition retained it, and takeover fails closed on top
    /// of that rather than overriding it.
    #[error("work {work_id}'s surface teardown was not clean: {report:?}")]
    SurfaceNotClean {
        /// The Work id that was checked.
        work_id: String,
        /// The teardown report explaining what was retained.
        report: TeardownReport,
    },
}

/// Whether `target_work_id`'s branch is safe for a gate Work to take over by
/// [`crate::runtime::surface::attach`] instead of minting its own — §8.6's
/// Mechanism A. Fails closed, naming the reason, unless the target reached
/// an absorbing terminal state and its surface's teardown reported every
/// binding `Removed`. On success, returns the target's own bindings —
/// everything `surface::attach` needs to know which branch(es), in which
/// repositories, to attach to.
///
/// **This is necessary, not sufficient.** It answers "does the journal say
/// this branch should be free", which is exactly what the §8.6 investigation
/// found the engine had no way to ask before now. It cannot answer "is the
/// branch still free at the moment of the attempt" — a target could, in
/// principle, be rematerialized by something else between this check
/// returning `Ok` and `surface::attach` actually running `git worktree add`.
/// Git itself is the final arbiter of that instant, exactly as it is today
/// for `materialize`'s own branch-collision case; `surface::attach`'s own
/// failure paths (branch reattached elsewhere, branch missing, worktree path
/// collision) are exactly the cases this function cannot see from journaled
/// state alone.
pub fn branch_takeover_precondition(
    registry: &WorkRegistry,
    target_work_id: &str,
) -> Result<Vec<RepositoryBinding>, BranchTakeoverError> {
    // #4: the target of a takeover is by definition terminal (the check
    // just below refuses anything else), which is exactly the case #4
    // evicts the full `Work` struct for — `works` alone would 404 a
    // perfectly good takeover target the instant its run settled. Only
    // `.state` is needed here, and `state_view` answers it from the slim
    // index, never evicted, so this never has to fall back to a journal
    // replay.
    let state =
        registry
            .state_view(target_work_id)
            .ok_or_else(|| BranchTakeoverError::TargetNotFound {
                work_id: target_work_id.to_string(),
            })?;
    if !is_absorbing(state) {
        return Err(BranchTakeoverError::TargetNotTerminal {
            work_id: target_work_id.to_string(),
            state,
        });
    }
    let run = registry
        .run_view(target_work_id)
        .filter(|run| run.surface.is_some())
        .ok_or_else(|| BranchTakeoverError::NoSurface {
            work_id: target_work_id.to_string(),
        })?;
    let surface = run.surface.as_ref().expect("filtered above");
    let report = run
        .teardown
        .as_ref()
        .ok_or_else(|| BranchTakeoverError::SurfaceNotTornDown {
            work_id: target_work_id.to_string(),
        })?;
    if !report.clean {
        return Err(BranchTakeoverError::SurfaceNotClean {
            work_id: target_work_id.to_string(),
            report: report.clone(),
        });
    }
    Ok(surface.bindings.clone())
}

/// An external effect the engine has committed to and must now perform
/// **with the core lock released** (§14.2's middle phase).
///
/// It carries everything the launch needs and nothing the journal owns: the
/// reservation is already durable, so this value is pure intent. A caller that
/// drops one without launching leaves an unsettled reservation in the journal,
/// which is the same state a crash leaves and is handled the same way — fail
/// closed at the next restart, never guessed at.
pub struct PendingLaunch {
    work_id: String,
    reservation: ExecutionReservation,
    backend: Arc<dyn Backend>,
    prepared: PreparedExecution,
}

impl std::fmt::Debug for PendingLaunch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingLaunch")
            .field("work_id", &self.work_id)
            .field("execution_id", &self.reservation.execution_id)
            .field("stage_id", &self.reservation.stage_id)
            .field("attempt", &self.reservation.attempt)
            .field("backend", &self.reservation.backend)
            .finish()
    }
}

impl PendingLaunch {
    /// The work this launch serves.
    pub fn work_id(&self) -> &str {
        &self.work_id
    }

    /// The reserved execution id.
    pub fn execution_id(&self) -> &str {
        &self.reservation.execution_id
    }

    /// Perform the external effects this phase owns: LAUNCH, and the first
    /// OBSERVE of what it produced.
    ///
    /// **Never call this while holding the core lock** — it is the process
    /// spawn, the container create, the thing §22.6 exists to keep out from
    /// under the daemon's single writer. The engine cannot enforce that
    /// (it never sees a guard, only `&mut Core`), so it is stated here and
    /// instrumented by the §22.6 tests.
    ///
    /// OBSERVE rides along rather than being left to `settle_launch` because
    /// it is an external effect too — for the Claude adapter, a handle it has
    /// no memory of sends it walking `/proc` — and §22.6 lists "reading a
    /// large output stream" beside the spawn. Taking it here also costs
    /// nothing: `drive` would have asked the same question one line later,
    /// and asking twice is not guaranteed to get the same answer.
    pub fn perform(&self) -> LaunchOutcome {
        let handle = self.backend.launch(&self.prepared);
        let observed = handle
            .as_ref()
            .ok()
            .map(|handle| self.backend.observe(handle));
        LaunchOutcome { handle, observed }
    }

    /// LAUNCH alone, without the observation — the raw external effect, for
    /// callers that want to drive the two phases apart by hand.
    pub fn launch(&self) -> Result<ExecutionHandle, BackendError> {
        self.backend.launch(&self.prepared)
    }
}

/// What [`PendingLaunch::perform`] came back with: the launch's own result,
/// and the observation taken of it outside the lock (absent when there was
/// nothing to observe because the launch failed).
#[derive(Debug)]
pub struct LaunchOutcome {
    handle: Result<ExecutionHandle, BackendError>,
    observed: Option<Result<Observation, BackendError>>,
}

impl From<Result<ExecutionHandle, BackendError>> for LaunchOutcome {
    /// A launch result with no observation attached, for a caller that drove
    /// [`PendingLaunch::launch`] by hand. `settle_launch` records the start
    /// and parks: it does not observe on the caller's behalf, because it runs
    /// with the core lock held and OBSERVE is an external effect (§22.6).
    fn from(handle: Result<ExecutionHandle, BackendError>) -> Self {
        Self {
            handle,
            observed: None,
        }
    }
}

/// Input the engine has committed to delivering, to be handed to the harness
/// **with the core lock released** (§14.2's middle phase, applied to SEND).
///
/// SEND is the other verb that creates external work, and for a print-mode
/// harness it is the same effect START is: a process fork/exec plus the reader
/// threads that ingest its stdout. It is also the resume path of GP-2's ask
/// primitive — the verb a human's answer travels through — so leaving it under
/// the guard would have put the milestone's own feature on the wrong side of
/// the milestone's own boundary.
pub struct PendingSend {
    work_id: String,
    stage_id: String,
    /// The stage attempt this delivery was reserved against (§14.5's
    /// currency check for SEND, mirroring [`PendingObserve`] and
    /// [`ExecutionReservation`]): a delivery whose stage has since moved to
    /// a different index or attempt is stale even if the execution id
    /// happens to still match.
    index: usize,
    attempt: u32,
    execution: ExecutionRecord,
    backend: Arc<dyn Backend>,
    input: String,
    /// What a re-adoption would need, if the adapter turns out to have
    /// forgotten this execution (see [`PendingSend::perform`]). `None` for a
    /// run with no recorded surface, where there is nothing to resume into.
    resume: Option<ResumeRequest>,
}

impl std::fmt::Debug for PendingSend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingSend")
            .field("work_id", &self.work_id)
            .field("execution_id", &self.execution.execution_id)
            .field("stage_id", &self.stage_id)
            .finish()
    }
}

impl PendingSend {
    /// The work this delivery serves.
    pub fn work_id(&self) -> &str {
        &self.work_id
    }

    /// Deliver the input and observe what the turn now says — both outside
    /// the core lock.
    ///
    /// **Reattach first if the adapter has forgotten this execution.** A work
    /// parked in `needs_input` is, by construction, the work most likely to be
    /// sitting there when a daemon is restarted: it is waiting on a human, and
    /// humans take longer than daemons live. Startup reconciliation
    /// deliberately does not touch it — a parked work is a decision, not
    /// uncertainty (`runtime::recovery`'s own rule) — so the first thing that
    /// can discover the adapter no longer holds the context is the answer
    /// itself. Without this, that answer was journaled, refused with
    /// `UnknownExecution`, and left durable but unreachable: `retry` re-enters
    /// the stage as a fresh attempt and never re-delivers it.
    ///
    /// RESUME is the verb §15 provides for exactly this, and it is the
    /// adapter's decision, not the engine's: one that cannot evidence the
    /// context refuses, the refusal is what `settle_send` records, and the
    /// work fails closed as it did before. RESUME never starts a turn, so a
    /// re-adoption costs nothing and creates no second execution.
    pub fn perform(&self) -> SendOutcome {
        let handle = handle_of(&self.execution);
        let mut reattached = false;
        let mut delivered = self.backend.send(&handle, &self.input);
        if let (Err(BackendError::UnknownExecution { .. }), Some(request)) =
            (&delivered, &self.resume)
            && self.backend.capabilities().resume
            && self.backend.resume(&handle, request).is_ok()
        {
            reattached = true;
            delivered = self.backend.send(&handle, &self.input);
        }
        let observed = delivered.is_ok().then(|| self.backend.observe(&handle));
        SendOutcome {
            delivered,
            reattached,
            observed,
        }
    }
}

/// What [`PendingSend::perform`] came back with.
#[derive(Debug)]
pub struct SendOutcome {
    delivered: Result<(), BackendError>,
    reattached: bool,
    observed: Option<Result<Observation, BackendError>>,
}

/// An observation of a live execution the engine has decided to take **with
/// the core lock released** (§14.2's middle phase, applied to OBSERVE alone).
///
/// The other two performers observe as the tail of an effect they themselves
/// caused. This one exists for the case nothing causes: a turn that was still
/// `InFlight` at launch-settle and finishes later, on its own, with no client
/// anywhere near the daemon (issue #46 — Run B sat 45 minutes in `active`
/// after a clean turn end because the only observers were launch-settle,
/// SEND-settle, restart recovery, and client cranks).
///
/// It carries the run coordinate the observation is *about*, and nothing else:
/// no reservation, because deciding to look at something commits sergeant to
/// nothing and appends nothing. What makes the result safe to act on is
/// [`Engine::settle_observe`]'s currency check against this snapshot, not a
/// durable claim staked before the effect.
pub struct PendingObserve {
    work_id: String,
    execution: ExecutionRecord,
    stage_id: String,
    index: usize,
    attempt: u32,
    backend: Arc<dyn Backend>,
}

impl std::fmt::Debug for PendingObserve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingObserve")
            .field("work_id", &self.work_id)
            .field("execution_id", &self.execution.execution_id)
            .field("stage_id", &self.stage_id)
            .field("attempt", &self.attempt)
            .finish()
    }
}

impl PendingObserve {
    /// The work this observation is about.
    pub fn work_id(&self) -> &str {
        &self.work_id
    }

    /// The execution being observed.
    pub fn execution_id(&self) -> &str {
        &self.execution.execution_id
    }

    /// Ask the backend where the execution is. **Never while holding the core
    /// lock**: OBSERVE is an external effect (§22.6, round-2 finding
    /// N3R2-03 — for the Claude adapter a handle it has no memory of sends it
    /// walking `/proc` and reading a transcript off disk).
    pub fn perform(&self) -> ObserveOutcome {
        ObserveOutcome {
            observed: self.backend.observe(&handle_of(&self.execution)),
        }
    }
}

/// What [`PendingObserve::perform`] came back with.
#[derive(Debug)]
pub struct ObserveOutcome {
    observed: Result<Observation, BackendError>,
}

/// A turn's INTERRUPT the engine has decided to issue (R-MVP1-7's per-turn
/// wall-clock ceiling), to be performed **with the core lock released**
/// (§14.2's middle phase applied to INTERRUPT, §22.6).
///
/// Distinct from STOP (`PendingSurface`'s teardown path, and
/// `Engine::stop_execution`'s reviewed under-the-lock exemption): the
/// execution stays known afterward and a later turn may still use it. Only
/// *this* turn is being asked to end, and whether it complied is
/// [`Engine::due_observations`]'s question to answer, never this one's
/// (§25 — an interrupt is a request, not a verdict).
pub struct PendingInterrupt {
    work_id: String,
    execution: ExecutionRecord,
    backend: Arc<dyn Backend>,
    /// The ceiling this crossing was measured against
    /// ([`Engine::effective_turn_ceiling`] at collection time in
    /// [`Engine::due_interrupts`]) — MVP-3's per-Work override when the
    /// client set one, else the daemon-wide default. Carried on the struct
    /// rather than recomputed at settle time so the journaled
    /// `ceiling_secs` always names the bound this specific crossing actually
    /// violated, not just whatever `Engine::turn_ceiling` happens to be by
    /// the time settlement runs.
    ceiling_secs: f64,
}

impl std::fmt::Debug for PendingInterrupt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingInterrupt")
            .field("work_id", &self.work_id)
            .field("execution_id", &self.execution.execution_id)
            .finish()
    }
}

impl PendingInterrupt {
    /// The work whose turn is being interrupted.
    pub fn work_id(&self) -> &str {
        &self.work_id
    }

    /// Issue the INTERRUPT. **Never while holding the core lock** — this is
    /// the external effect (§22.6), symmetric with every other performer
    /// here: it cannot see a `Core` by construction.
    pub fn perform(&self) -> InterruptOutcome {
        InterruptOutcome {
            outcome: self.backend.interrupt(&handle_of(&self.execution)),
        }
    }
}

/// What [`PendingInterrupt::perform`] came back with.
#[derive(Debug)]
pub struct InterruptOutcome {
    outcome: Result<Completion, BackendError>,
}

/// A git operation the engine has committed to and must now perform **with
/// the core lock released** (§14.2's middle phase, applied to §11's surfaces).
///
/// Materializing a surface is three `git` fork/exec/wait cycles per repository
/// on a checkout sergeant does not own and cannot bound; teardown and
/// re-attachment are more of the same. Measured in this container, a single
/// `git worktree add` on a 3.4 MB `.git` takes 86 ms — and a monorepo is not
/// 3.4 MB. Under the guard that is every other request on the daemon, reads
/// included, queueing behind someone else's checkout.
pub struct PendingSurface {
    work_id: String,
    /// The daemon's data dir, carried so [`Self::perform`] can locate the
    /// interprocess repository locks (§9.4). Deliberately separate from
    /// `SurfaceEffect::Materialize`'s `surfaces_root`, which R-MVP1-1 unbound
    /// from the data dir: an estate may put its surfaces anywhere, and the
    /// locks still live in the daemon's own storage.
    data_dir: PathBuf,
    effect: SurfaceEffect,
}

/// Which git operation a [`PendingSurface`] carries, and what the engine will
/// do with the result once it has the lock back.
pub enum SurfaceEffect {
    /// Create the run's worktrees, then bind the workflow and enter stage 0.
    Materialize {
        /// Root directory this run's surface lives under (R-MVP1-1) — the
        /// plan's already-resolved `surfaces_root`, not necessarily under
        /// `data_dir`.
        surfaces_root: PathBuf,
        /// The plan whose repositories are being materialized, carried so the
        /// binding it produces is the one that was admitted.
        plan: Box<StartPlan>,
    },
    /// Re-attach the worktrees a terminal teardown removed, then re-enter the
    /// stage this retry is for.
    Rematerialize {
        /// The recorded surface, whose branches are the durable part.
        surface: WorkSurface,
        /// Stage index the retry re-enters.
        index: usize,
        /// Attempt number it re-enters as.
        attempt: u32,
    },
    /// Remove the worktrees and report what the disk actually held.
    Teardown {
        /// The recorded surface.
        surface: WorkSurface,
        /// Whether a restart, rather than the run itself, is what recorded it.
        recovered: bool,
    },
}

impl std::fmt::Debug for PendingSurface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let effect = match &self.effect {
            SurfaceEffect::Materialize { .. } => "materialize",
            SurfaceEffect::Rematerialize { .. } => "rematerialize",
            SurfaceEffect::Teardown { .. } => "teardown",
        };
        f.debug_struct("PendingSurface")
            .field("work_id", &self.work_id)
            .field("effect", &effect)
            .finish()
    }
}

impl PendingSurface {
    /// The work this operation serves.
    pub fn work_id(&self) -> &str {
        &self.work_id
    }

    /// Run the git operation. **Never while holding the core lock.**
    pub fn perform(&self) -> SurfaceOutcome {
        match &self.effect {
            SurfaceEffect::Materialize {
                surfaces_root,
                plan,
            } => SurfaceOutcome::Materialized(materialize_admitted(
                &self.data_dir,
                surfaces_root,
                &self.work_id,
                &plan.repositories,
                // §8.2: the base each repository was *admitted* on, not
                // whatever its HEAD says now. An empty slice (a plan made
                // without admission) leaves the live read in place, which is
                // the only thing it could do.
                plan.preflight
                    .as_ref()
                    .map_or(&[][..], |p| p.repositories.as_slice()),
            )),
            SurfaceEffect::Rematerialize { surface, .. } => {
                // A retry whose worktrees are all still on disk needs no git
                // at all; asking here rather than under the guard keeps even
                // the `stat` calls out of it.
                if surface.bindings.iter().all(|b| b.worktree_path.exists()) {
                    return SurfaceOutcome::Rematerialized(Ok(None));
                }
                SurfaceOutcome::Rematerialized(rematerialize(&self.data_dir, surface).map(Some))
            }
            SurfaceEffect::Teardown { surface, .. } => {
                SurfaceOutcome::TornDown(teardown(&self.data_dir, surface))
            }
        }
    }
}

/// What [`PendingSurface::perform`] came back with.
#[derive(Debug)]
pub enum SurfaceOutcome {
    /// A fresh surface, or the failure that left none.
    Materialized(Result<WorkSurface, SurfaceError>),
    /// A re-attached surface, `None` when nothing needed re-attaching.
    Rematerialized(Result<Option<WorkSurface>, SurfaceError>),
    /// Teardown never fails: it reports what it found (§11 fails closed).
    TornDown(TeardownReport),
}

/// What the engine needs before it can crank again.
#[derive(Debug)]
pub enum Next {
    /// Nothing further: the run is where its last explicit signal left it.
    Parked,
    /// Perform this outside the lock, then feed the result back through
    /// [`Engine::settle_launch`].
    ///
    /// Boxed because the payload dwarfs `Parked` — and because the common
    /// case *is* `Parked`: every observation that does not enter a stage
    /// returns one, so the enum is moved far more often than it is filled.
    Launch(Box<PendingLaunch>),
    /// Deliver this outside the lock, then feed the result back through
    /// [`Engine::settle_send`].
    Send(Box<PendingSend>),
    /// Run this git operation outside the lock, then feed the result back
    /// through [`Engine::settle_surface`].
    Surface(Box<PendingSurface>),
    /// Take this observation outside the lock, then feed the result back
    /// through [`Engine::settle_observe`].
    ///
    /// Unlike the other three this one is never *returned* by a settle: it is
    /// only ever handed in by the daemon's completion driver, which is the one
    /// caller that has to ask a question nobody's request asked.
    Observe(Box<PendingObserve>),
    /// Issue this INTERRUPT outside the lock, then feed the result back
    /// through [`Engine::settle_interrupt`]. R-MVP1-7's per-turn ceiling —
    /// like `Observe`, only ever handed in by the completion driver.
    Interrupt(Box<PendingInterrupt>),
}

/// One crank of the engine: everything it committed under this lock hold,
/// plus what it needs done outside it.
#[must_use = "a Step's Deferred must be drained and its Next acted on, or the \
              adapter's tail work detaches and the run stalls mid-launch"]
#[derive(Debug)]
pub struct Step {
    /// What to do outside the lock.
    pub next: Next,
    /// Adapter tail work collected during this crank (issue #14/B3).
    pub deferred: Deferred,
}

impl Step {
    /// A crank that committed nothing outstanding and needs nothing done.
    pub fn parked() -> Self {
        Self {
            next: Next::Parked,
            deferred: Deferred::new(),
        }
    }

    /// Whether this crank asks for an external launch.
    pub fn needs_launch(&self) -> bool {
        matches!(self.next, Next::Launch(_))
    }
}

/// Failure from an engine operation.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// Estate resolution failed.
    #[error(transparent)]
    Estate(#[from] EstateError),
    /// §4.1's exact-root admission failed for this daemon's **bound** estate
    /// — the root it was started against no longer resolves. Shares
    /// `workspace_error`'s wire code: to a client, "this daemon's estate
    /// cannot be read" is the same class of answer it always was.
    #[error(transparent)]
    EstateRoot(#[from] EstateRootError),
    /// Workflow resolution failed.
    #[error(transparent)]
    Workflow(#[from] WorkflowError),
    /// Routing failed (§13).
    #[error(transparent)]
    Route(#[from] RouteError),
    /// Surface materialization failed.
    #[error(transparent)]
    Surface(#[from] SurfaceError),
    /// A backend operation failed.
    #[error(transparent)]
    Backend(#[from] BackendError),
    /// Journal append or projection fold failed.
    #[error(transparent)]
    Core(#[from] CoreError),
    /// The requested repository subset does not match the estate.
    #[error("{0}")]
    RepositorySelection(String),
    /// estate-root proposal §7.1: a multi-repository estate was submitted to
    /// with no scope selected at all — Sergeant refuses rather than
    /// silently expanding to every repository. `repo_count`/`repos`/`groups`
    /// are exactly what the API's structured 422 body names as the remedy
    /// (§15: "list declared repos/groups and show `--repo`, `--group`,
    /// `--all`").
    #[error(
        "this estate contains {repo_count} repositories, but no Work scope was selected; \
         select repositories with --repo, a declared group with --group, or the whole estate \
         explicitly with --all (declared repos: {repos:?}; declared groups: {groups:?})"
    )]
    MissingScope {
        /// How many repositories the estate declares.
        repo_count: usize,
        /// Every declared repository's name.
        repos: Vec<String>,
        /// Every declared group's name.
        groups: Vec<String>,
    },
    /// §15: `scope.group`/`--group` named a group this estate has not
    /// declared.
    #[error("no group named {requested:?} in this estate (declared: {available:?})")]
    UnknownGroup {
        /// The group name that was requested.
        requested: String,
        /// Every group name the estate does declare.
        available: Vec<String>,
    },
    /// Owner ruling (2026-08-20): `scope.all`/`--all` combined with
    /// `scope.repos`/`--repo` and/or `scope.group`/`--group` is refused
    /// rather than `all` silently winning over the other two. §7.1's three
    /// scope forms are mutually exclusive at the top level — naming both
    /// "every declared repository" and a specific subset is not a union,
    /// it is two different answers to "what is this Work about" in one
    /// request, and Sergeant does not guess which one the caller meant.
    /// Refused here, in [`Engine::resolve_scope`], before any Work record —
    /// the same "reject before Work or worktree side effects" timing
    /// [`EngineError::MissingScope`] already uses.
    #[error(
        "scope.all was combined with scope.repos={repos:?} and/or scope.group={group:?}; --all \
         selects the whole estate and must not be combined with a repository subset — submit \
         --all alone to select every declared repository, or --repo/--group without --all to \
         select a subset"
    )]
    ConflictingScope {
        /// `scope.repos`, exactly as submitted alongside `scope.all`.
        repos: Vec<String>,
        /// `scope.group`, exactly as submitted alongside `scope.all`, if any.
        group: Option<String>,
    },
    /// R-MVP1-4: the selected repositories disagree on `instructions`
    /// policy. One process, one `--setting-sources` — there is nowhere for
    /// two policies to both take effect, so the submission is refused
    /// naming every repository and its value.
    #[error(
        "repositories disagree on instructions policy ({}); one process runs one policy — \
         align every selected [[repo]]'s instructions value, or select a consistent subset \
         with --repo",
        .repos.join(", ")
    )]
    InstructionPolicyConflict {
        /// `"<repo>=<policy>"` for every selected repository.
        repos: Vec<String>,
    },
    /// The requested profile is not declared by the estate.
    #[error("no profile named {requested:?} in this estate (has: {available})")]
    ProfileNotFound {
        /// Requested profile name.
        requested: String,
        /// Declared profile names.
        available: String,
    },
    /// The selected profile launches a different backend than the route
    /// resolved to. Guessing which one the user meant is exactly the
    /// concept-collapsing §13 forbids.
    #[error(
        "profile {profile:?} launches backend {profile_backend:?}, but this work routed to \
         {routed:?} ({tier}); name them consistently"
    )]
    ProfileBackendMismatch {
        /// Profile name.
        profile: String,
        /// Backend the profile declares.
        profile_backend: String,
        /// Backend routing chose.
        routed: String,
        /// Tier that chose it.
        tier: String,
    },
    /// The engine tried a transition the §10 table forbids. This is a bug in
    /// the engine, not a user error, and it fails before any append.
    #[error("illegal work transition for {work_id}: {from} -> {to}")]
    IllegalTransition {
        /// Work id.
        work_id: String,
        /// Current state.
        from: WorkState,
        /// Attempted state.
        to: WorkState,
    },
    /// Input was delivered to a work that is not waiting for any.
    #[error("work {work_id} is {state}, not needs_input; nothing is waiting for an answer")]
    NotAwaitingInput {
        /// Work id.
        work_id: String,
        /// Its actual state.
        state: WorkState,
    },
    /// Retry was asked for a work that has nothing to retry.
    #[error("work {work_id} is {state}; only failed, blocked or waiting work can be retried")]
    NotRetryable {
        /// Work id.
        work_id: String,
        /// Its actual state.
        state: WorkState,
    },
    /// The work has no run state (never started).
    #[error("work {work_id} has no run to act on")]
    NoRun {
        /// Work id.
        work_id: String,
    },
    /// R-MVP1-10's exit door (`Engine::extend_turn_envelope`) was asked for
    /// a work that is not `blocked` — extending an envelope only means
    /// something for a Work actually stopped at one.
    #[error(
        "work {work_id} is {state}, not blocked; nothing here needs its turn envelope extended"
    )]
    NotBlocked {
        /// Work id.
        work_id: String,
        /// Its actual state.
        state: WorkState,
    },
    /// A run references a backend the daemon does not have registered.
    #[error("work {work_id} routed to backend {backend:?}, which is not registered here")]
    BackendMissing {
        /// Work id.
        work_id: String,
        /// The missing backend name.
        backend: String,
    },
    /// A run's stage index points past the end of its pinned workflow.
    #[error("work {work_id} has no stage at index {index}")]
    NoSuchStage {
        /// Work id.
        work_id: String,
        /// The offending index.
        index: usize,
    },
    /// R-MVP1-11: a stage declares `requires_ask = true` but the harness it
    /// resolved to advertises `Capabilities::ask == false`. Refused at
    /// submit, before a Work exists, naming the workflow, the stage and the
    /// backend — the same "reject before Work or worktree side effects"
    /// timing §17.5's other preflight checks already use.
    #[error(
        "workflow {workflow:?} stage {stage:?} requires an actor that can author its own \
         ask (requires_ask = true), but backend {backend:?} does not declare \
         Capabilities::ask; route this stage to a backend that does, or drop \
         requires_ask from the stage's declaration"
    )]
    AskCapabilityUnavailable {
        /// Workflow name.
        workflow: String,
        /// The declaring stage's id.
        stage: String,
        /// The backend the stage resolved to.
        backend: String,
    },
    /// §8.1: Git admission preflight refused this submission, before a Work
    /// record existed and before any Git mutation. Carries every unresolved
    /// finding, each with its own §15 code and remedy — see
    /// [`crate::runtime::preflight`].
    #[error("{0}")]
    GitPreflight(PreflightRefusal),
    /// §17.5: a workflow declares a `kind = "execute"` stage, but the
    /// `"docker"` backend is not registered or its probe reports
    /// unavailable. Refused before Work or worktree side effects, exactly
    /// like every other §17.5 preflight — "an actor-only workflow may run
    /// when Docker is unavailable; a mixed or execute workflow may not."
    #[error(
        "workflow {workflow:?} stage {stage:?} is a Docker execute stage, but the local Docker \
         executor is unavailable: {detail}"
    )]
    ExecuteBackendUnavailable {
        /// Workflow name.
        workflow: String,
        /// The declaring stage's id.
        stage: String,
        /// Why the Docker backend could not be routed to.
        detail: String,
    },
}

impl EngineError {
    /// Structured error code for the API.
    ///
    /// `"workspace_error"` keeps its pre-rename spelling deliberately.
    /// §13.2 moves the *domain vocabulary* from Workspace to Estate; a wire
    /// code is not vocabulary, it is a string clients branch on, and
    /// renaming it would be a compatibility break wearing a rename's
    /// clothes. The human message beside it already says "estate".
    pub fn code(&self) -> &'static str {
        match self {
            EngineError::Estate(_) | EngineError::EstateRoot(_) => "workspace_error",
            EngineError::Workflow(_) => "workflow_error",
            EngineError::Route(e) => e.code(),
            EngineError::Surface(_) => "surface_error",
            EngineError::Backend(_) => "backend_error",
            EngineError::Core(_) => "internal",
            EngineError::RepositorySelection(_) => "unknown_repository",
            EngineError::MissingScope { .. } => "missing_scope",
            EngineError::UnknownGroup { .. } => "unknown_group",
            EngineError::ConflictingScope { .. } => "conflicting_scope",
            EngineError::InstructionPolicyConflict { .. } => "instruction_policy_conflict",
            EngineError::ProfileNotFound { .. } => "profile_not_found",
            EngineError::ProfileBackendMismatch { .. } => "profile_backend_mismatch",
            EngineError::IllegalTransition { .. } => "illegal_transition",
            EngineError::NotAwaitingInput { .. } => "not_awaiting_input",
            EngineError::NotRetryable { .. } => "not_retryable",
            EngineError::NoRun { .. } => "no_run",
            EngineError::NotBlocked { .. } => "not_blocked",
            EngineError::BackendMissing { .. } => "backend_missing",
            EngineError::NoSuchStage { .. } => "no_such_stage",
            EngineError::AskCapabilityUnavailable { .. } => "ask_capability_unavailable",
            EngineError::ExecuteBackendUnavailable { .. } => "execute_backend_unavailable",
            // §15: the code names the *check*, not the phase — "preflight
            // failed" would tell a client nothing it could act on. Every
            // other finding travels in the body beside it.
            EngineError::GitPreflight(refusal) => refusal.code(),
        }
    }

    /// Backends the caller could have asked for, when the failure was routing.
    pub fn available_backends(&self) -> Option<&[String]> {
        match self {
            EngineError::Route(e) => Some(e.available()),
            _ => None,
        }
    }
}

/// Event kind: R-MVP1-7's per-turn wall-clock ceiling interrupted a turn
/// that ran longer than [`Engine::turn_ceiling`]. Declared here rather than
/// `domain::execution` — nothing outside the engine folds it into a
/// projection yet, and the reducer's forward-compatible "unknown kinds are
/// ignored" rule (`runtime::projection`'s own doc comment) means an
/// unrecognized kind costs no projection change to introduce.
pub const KIND_TURN_CEILING_INTERRUPTED: &str = "execution.turn_ceiling_interrupted";

/// Event kind: R-MVP1-10's exit door for R-MVP1-7's envelope-exhausted
/// landing (`Engine::extend_turn_envelope`) — an explicit, journaled
/// operator decision raising this Work's turn envelope by some number of
/// additional turns, cumulative across every extension. Unlike
/// `KIND_TURN_CEILING_INTERRUPTED` above, this one *is* folded —
/// `runtime::projection`'s reducer applies it to `WorkRun::turn_cap_bonus`,
/// because changing what `check_turn_envelope` compares against is the
/// whole point: `retry` alone re-blocks on the identical, unchangeable
/// condition (the revolving-door defect this closes), so the door has to
/// change the condition itself, not just ask again.
pub const KIND_TURN_ENVELOPE_EXTENDED: &str = "execution.turn_envelope_extended";

/// R-MVP1-7's default turn cap.
///
/// **Not yet the ledger's own measured N-series turn count (contract
/// Unknown #2) — that measurement still has not been run** (invariants:
/// MVP1-R1-I1). What changed from the first shipped value (6, the
/// contract's own L16 worked example) is the evidence it is grounded in:
/// `find .sergeant/workflows -name workflow.toml` shows this repo's own
/// longest admitted workflow (`repo-to-icm`) declares 10 stages (11 as of
/// MVP-2 lane D3's `65-self-check`, a `kind = "execute"` stage — its LAUNCH
/// still goes through the same `execution.started`/`turns_spawned`
/// accounting an actor stage's does, spending no model tokens but counting
/// against this same cap exactly like any other stage), and a cap of 6
/// blocked every admitted workflow over 6 stages on its very first run,
/// unconditionally — `repo-to-icm`, the workflow R-MVP1-2's own finalize
/// helper was built for, included. 12 covers every admitted workflow's
/// stage count (max 11) with margin for at least one retry inside any
/// single stage, without inventing an unrelated round number.
/// L16 arithmetic still holds at the new value: the largest measured
/// single turn on this build's evidence is Run B2's $3.21
/// (`GAUNTLET.md`'s N-series close-out), so a 12-turn cap is a ~$38 bound,
/// never a $2.50 one.
///
/// Structurally grounded, still not live-measured: a real N-series run
/// (or the opt-in Claude suite, `SERGEANT_CLAUDE_TESTS=1`) is what would
/// finally answer "how many turns does `repo-to-icm` actually spend",
/// which is the number this constant should carry once it exists. Until
/// then, override per-installation with [`Engine::with_turn_cap`]
/// (`DaemonConfig::turn_cap` / `SGT_TURN_CAP`, wired the same way
/// [`Self::turn_ceiling`]'s override is) — and R-MVP1-10's own exit door,
/// `Engine::extend_turn_envelope`, means a Work that still outgrows
/// whatever cap is configured is recoverable without a restart, let alone
/// a recompile.
pub const DEFAULT_TURN_CAP: u32 = 12;

/// R-MVP1-7's default per-turn wall-clock ceiling: "the soak's hang bound
/// (CUT 10), not a stall detector" — sized to be well past any turn this
/// build has measured (issue #46's Run B turns finished in minutes) and
/// well short of #46's own 45-minute structurally-invisible stall, so a
/// truly hung turn is bounded without the ceiling firing on ordinary work.
/// Provisional pending the ledger's own measurement (contract Unknown #2);
/// override with [`Engine::with_turn_ceiling`].
pub const DEFAULT_TURN_CEILING: Duration = Duration::from_secs(15 * 60);

/// The workflow engine: backends, defaults, and the data dir surfaces live in.
#[derive(Debug, Clone)]
pub struct Engine {
    /// Backends this daemon can route to.
    pub backends: Arc<BackendRegistry>,
    /// §13's last tier before failure.
    pub default_backend: Option<String>,
    /// Data dir owning `surfaces/` (default; R-MVP1-1 makes this a default,
    /// not the only answer — see [`Self::surfaces_root`]).
    pub data_dir: PathBuf,
    /// Default root for work surfaces (R-MVP1-1): `<data_dir>/surfaces`
    /// unless overridden by [`Self::with_surfaces_root`]. A per-estate
    /// `[estate] surfaces_dir` narrows this further, per-plan, in
    /// [`Self::plan`] — this field is only ever the daemon-wide fallback.
    pub surfaces_root: PathBuf,
    /// R-MVP1-7's turn cap: the total number of turns (LAUNCH's turn 1 plus
    /// every SEND after it) any one Work may ever spawn, for its whole life
    /// across every stage and retry. [`Self::check_turn_envelope`] is the
    /// one gate; [`WorkRun::turns_spawned`] is the one counter.
    pub turn_cap: u32,
    /// R-MVP1-7's per-turn wall-clock ceiling, swept by the daemon's
    /// completion driver alongside [`Self::due_observations`] (see
    /// [`Self::due_interrupts`]).
    pub turn_ceiling: Duration,
    /// §5.1/§5.2: the one estate this daemon is bound to, admitted at
    /// startup ([`crate::domain::estate::Estate::admit`]) and pinned
    /// here for the process's whole life.
    ///
    /// **This is the only topology authority.** [`Self::plan`] reads the
    /// manifest at *this* root; a submission's `origin.cwd` is recorded
    /// evidence only (§13.3) and can never move it. That is what removes the
    /// recursion hazard §5.2 names — a child command launched from a Work
    /// surface rediscovering that linked worktree as a new estate.
    ///
    /// `None` only for a daemon started with no estate at all (test rigs and
    /// the intent-capture path): [`Self::plan`] answers `Ok(None)` there,
    /// exactly as "no repository context" always has.
    pub estate_root: Option<PathBuf>,
    /// When each Work's current turn was last (re-)spawned, for the ceiling
    /// sweep. Deliberately **not** journaled or durable: a restart forgets
    /// it, which is acceptable for a soak-test hang bound (never an
    /// adversarial one) and keeps this out of the projection entirely — the
    /// turn cap above is the envelope's durable half; this is the timing
    /// half, and the two are independent by design. Keyed by work id, one
    /// entry per Work with a turn currently in flight.
    turn_started: Arc<Mutex<BTreeMap<String, Instant>>>,
}

impl Engine {
    /// Build an engine over a registry and data dir. `surfaces_root`
    /// defaults to `<data_dir>/surfaces` — today's layout, nothing moves —
    /// override with [`Self::with_surfaces_root`] (`SGT_SURFACES_DIR`).
    pub fn new(
        backends: Arc<BackendRegistry>,
        default_backend: Option<String>,
        data_dir: &Path,
    ) -> Self {
        Self {
            backends,
            default_backend,
            surfaces_root: data_dir.join(SURFACES_DIR),
            data_dir: data_dir.to_path_buf(),
            turn_cap: DEFAULT_TURN_CAP,
            turn_ceiling: DEFAULT_TURN_CEILING,
            estate_root: None,
            turn_started: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    /// Bind this engine to one estate root (§5.1). The daemon calls this
    /// once at startup with the canonical root it was started against; every
    /// later [`Self::plan`] reads that estate and no other.
    pub fn with_estate_root(mut self, estate_root: PathBuf) -> Self {
        self.estate_root = Some(estate_root);
        self
    }

    /// Override the daemon-wide turn cap (R-MVP1-7). Wired from
    /// `DaemonConfig::turn_cap` / `SGT_TURN_CAP` (`daemon::run_until_signal`)
    /// the same way [`Self::with_turn_ceiling`] is — this is the fleet-wide
    /// default beneath two per-Work doors: MVP-3's submit-time
    /// `Work::envelope` override ([`Self::effective_turn_cap`]) and
    /// R-MVP1-10's [`Self::extend_turn_envelope`], which raises whichever of
    /// those two bases a Work is currently checked against.
    pub fn with_turn_cap(mut self, turn_cap: u32) -> Self {
        self.turn_cap = turn_cap;
        self
    }

    /// Override the per-turn wall-clock ceiling (R-MVP1-7). Same relationship
    /// to MVP-3's submit-time override as [`Self::with_turn_cap`]: this is
    /// the default [`Self::effective_turn_ceiling`] falls back to when a Work
    /// carries none of its own.
    pub fn with_turn_ceiling(mut self, turn_ceiling: Duration) -> Self {
        self.turn_ceiling = turn_ceiling;
        self
    }

    /// R-MVP1-7's effective turn cap for one specific Work: MVP-3's
    /// submit-time override (`work.envelope.turn_cap`) when the client set
    /// one, else this engine's daemon-wide [`Self::turn_cap`] — plus
    /// R-MVP1-10's cumulative `sgt extend` bonus either way. The **one**
    /// formula [`Self::check_turn_envelope`] gates admission on and
    /// `api::work_view` displays, so a client can never see a budget the
    /// engine itself is not actually enforcing.
    pub fn effective_turn_cap(&self, work: Option<&Work>, run: &WorkRun) -> u32 {
        let base = work
            .and_then(|w| w.envelope.as_ref())
            .and_then(|e| e.turn_cap)
            .unwrap_or(self.turn_cap);
        base.saturating_add(run.turn_cap_bonus)
    }

    /// R-MVP1-7's effective per-turn wall-clock ceiling for one specific
    /// Work: MVP-3's submit-time override (`work.envelope.ceiling_secs`)
    /// when the client set one, else this engine's daemon-wide
    /// [`Self::turn_ceiling`]. The one formula [`Self::due_interrupts`]
    /// sweeps against and `api::work_view` displays — same reasoning as
    /// [`Self::effective_turn_cap`].
    pub fn effective_turn_ceiling(&self, work: Option<&Work>) -> Duration {
        work.and_then(|w| w.envelope.as_ref())
            .and_then(|e| e.ceiling_secs)
            .map(Duration::from_secs)
            .unwrap_or(self.turn_ceiling)
    }

    /// Override the default surfaces root (R-MVP1-1). The daemon calls this
    /// once at startup from `SGT_SURFACES_DIR`; a per-estate `[estate]
    /// surfaces_dir` overrides it again, per-submission, in [`Self::plan`] —
    /// this is only the daemon-wide fallback beneath that.
    pub fn with_surfaces_root(mut self, surfaces_root: PathBuf) -> Self {
        self.surfaces_root = surfaces_root;
        self
    }

    /// Resolve everything a run needs, without touching anything.
    ///
    /// **§5.2: the request's cwd has no authority here.** Topology comes from
    /// [`Self::estate_root`] — the one estate this daemon was started
    /// against — and from nowhere else. `context.cwd` survives only as
    /// `origin.cwd`, recorded evidence for diagnostics (§13.3).
    ///
    /// `Ok(None)` means "this daemon is bound to no estate": the intent is
    /// accepted and stays `pending` rather than being rejected, exactly as
    /// "no repository context" always has. Every *other* failure (a
    /// `sergeant.toml` that no longer resolves, an unroutable backend, a
    /// missing workflow) is a real error and is returned as one.
    ///
    /// "No surface" does not mean "no §13". A submission that *names* a
    /// backend has asked for something, and §13's terminal state for a
    /// selection sergeant cannot honour is "fail with available options" —
    /// not "record the name anyway on a work that will never run it". So the
    /// chain is consulted either way; only its no-selection outcome is
    /// tolerated here, because a captured intent with no repository has
    /// nothing to route yet and no default to disappoint.
    ///
    /// The manifest is re-read from the pinned root on every plan rather
    /// than cached from startup, so `sgt repo add` reaches a running daemon
    /// — the *root* is what startup fixes, and the root is what §5 binds.
    pub fn plan(&self, context: &SubmitContext<'_>) -> Result<Option<StartPlan>, EngineError> {
        let estate = match &self.estate_root {
            None => None,
            Some(root) => Some(Estate::resolve(root)?),
        };
        let Some(estate) = estate else {
            self.check_selection_is_honourable(context)?;
            return Ok(None);
        };
        let repositories = Self::resolve_scope(&estate, context)?;

        // R-MVP1-4: one process, one policy. Resolved and refused here, at
        // submit, before a Work record or a worktree exists — the same
        // "reject the submission" timing §17.5 already uses for an
        // unsatisfiable stage requirement.
        Self::check_instruction_policy(&estate, &repositories)?;

        // R-MVP1-1: `[estate] surfaces_dir` narrows the engine's own default,
        // per-submission — resolved once here and pinned into the plan, so a
        // later manifest edit cannot move a running Work's surface.
        let surfaces_root = estate
            .surfaces_dir
            .clone()
            .unwrap_or_else(|| self.surfaces_root.clone());

        let workflow_name = context
            .workflow
            .or(estate.default_workflow.as_deref())
            .unwrap_or(DEFAULT_WORKFLOW)
            .to_string();
        let workflow = WorkflowDefinition::resolve(&estate.root, &workflow_name)?;

        let route = route(
            &RouteInputs {
                explicit: context.backend,
                origin_client: context.origin_client,
                estate_default: estate.default_backend.as_deref(),
                global_default: self.default_backend.as_deref(),
            },
            &self.backends,
        )?;

        let profile = match context.profile {
            None => None,
            Some(name) => {
                let profile = Self::estate_profile(&estate, name)?;
                if profile.backend != route.backend {
                    return Err(EngineError::ProfileBackendMismatch {
                        profile: profile.name,
                        profile_backend: profile.backend,
                        routed: route.backend,
                        tier: route.source.as_str().to_string(),
                    });
                }
                Some(profile)
            }
        };

        // §17.5's whole-workflow preflight, run here — before a Work record
        // and before a worktree — precisely so an unsatisfiable requirement is
        // "reject the submission" rather than "a work that dies at stage 2".
        let stage_bindings = self.bind_stages(&estate, &workflow, &route, profile.as_ref())?;

        // §8.1/§8.4, last: the Git admission contract, run for every selected
        // repository before `work.submitted` and before any Git mutation.
        //
        // Last, deliberately. Everything above is a question about
        // configuration — is this estate readable, is that group declared, can
        // this workflow route — and every one of those is on §8.3's
        // never-bypass list too. Answering them first means a submission that
        // was going to be refused for an unroutable backend is refused for the
        // unroutable backend, rather than for the dirty mount preflight would
        // also have found; it also keeps the ~6 git spawns per repository off
        // submissions that were never going to run.
        //
        // Still strictly before the Work exists: `plan` has created nothing at
        // this point and its caller mints the `Work` only from an `Ok`.
        let preflight = match context.work_id {
            None => None,
            Some(work_id) => Some(
                preflight::run(
                    &estate,
                    &self.data_dir,
                    &surfaces_root,
                    work_id,
                    &repositories,
                    context.override_git_preflight,
                )
                .map_err(EngineError::GitPreflight)?,
            ),
        };

        Ok(Some(StartPlan {
            estate,
            repositories,
            surfaces_root,
            workflow,
            route,
            profile,
            stage_bindings,
            preflight,
        }))
    }

    /// estate-root proposal §7.2/§7.1: turn a submission's `{repos, group,
    /// all}` scope request into the exact repository list to materialize,
    /// resolved against *this daemon's own* already-discovered `estate`
    /// — never by a client. §7.2's whole point is that the CLI, the TUI, or
    /// a direct API caller submitting the identical scope JSON all reach
    /// this one function and get the identical answer; no client surface
    /// reimplements group-membership or empty-scope semantics.
    ///
    /// Order of decisions, per §7.1 (as amended by the owner's 2026-08-20
    /// ruling):
    ///
    /// 0. `all` combined with a nonempty `repos` and/or a set `group` is
    ///    refused outright ([`EngineError::ConflictingScope`]) — naming both
    ///    "every declared repository" and a specific subset is two answers
    ///    to one question, and `all` no longer silently wins.
    /// 1. `all` alone: every declared repository (§7.3 still journals the
    ///    request form separately, so *why* it resolved to everything is
    ///    never lost even though the resolved list alone couldn't say).
    /// 2. Otherwise, the union of `repos` and — when `group` names a
    ///    declared group — that group's members, `repos`-order first, new
    ///    group members appended, duplicates dropped. An undeclared `group`
    ///    is refused by name, listing the estate's declared groups (§15).
    /// 3. An empty union is not "select everything" (§7.1 replaces that
    ///    reading): a one-repository estate infers its sole repository; a
    ///    multi-repository estate is refused with the full §7.1 remedy body
    ///    (repo count, declared repos, declared groups).
    /// 4. A nonempty selection still goes through [`Estate::select`] for
    ///    its own checks (unknown name, duplicate name) — this function only
    ///    decides *what* the name list is, not whether every name in it is
    ///    valid.
    ///
    /// Historical note (MVP3-C2): before this, `run --group`'s expansion was
    /// pure CLI-side convenience that read group membership through an
    /// on-disk-free structural parser (`Estate::declared_groups_scoped`)
    /// specifically so an unrelated broken repository elsewhere in the
    /// estate could not block a group whose own members were all fine. That
    /// carve-out does not survive moving resolution here: `estate` above
    /// is already the strictly-resolved estate (`Estate::resolve` against
    /// this daemon's bound root — the same call `plan` makes for routing and
    /// instruction-policy regardless of scope), so a `--group` submission
    /// now fails on an unrelated broken repository exactly the way a plain
    /// `--repo` submission always has — resolving the estate, not group
    /// lookup, is what requires every declared mount to be present.
    /// (`declared_groups_scoped`, the on-disk-free parser this note
    /// describes, is itself gone: estate-root Phase D deleted it with the
    /// rest of cwd-based discovery.)
    fn resolve_scope(
        estate: &Estate,
        context: &SubmitContext<'_>,
    ) -> Result<Vec<RepositorySpec>, EngineError> {
        if context.all && (!context.repos.is_empty() || context.group.is_some()) {
            return Err(EngineError::ConflictingScope {
                repos: context.repos.to_vec(),
                group: context.group.map(str::to_string),
            });
        }
        if context.all {
            return Ok(estate.repositories.clone());
        }

        let mut names: Vec<String> = context.repos.to_vec();
        if let Some(group_name) = context.group {
            let group = estate
                .groups
                .get(group_name)
                .ok_or_else(|| EngineError::UnknownGroup {
                    requested: group_name.to_string(),
                    available: estate.groups.keys().cloned().collect(),
                })?;
            for repo in &group.repos {
                if !names.contains(repo) {
                    names.push(repo.clone());
                }
            }
        }

        if names.is_empty() {
            if estate.repositories.len() == 1 {
                return Ok(estate.repositories.clone());
            }
            return Err(EngineError::MissingScope {
                repo_count: estate.repositories.len(),
                repos: estate.repositories.iter().map(|r| r.name.clone()).collect(),
                groups: estate.groups.keys().cloned().collect(),
            });
        }

        estate
            .select(&names)
            .map_err(EngineError::RepositorySelection)
    }

    /// R-MVP1-4's submit-time policy check: every selected repository must
    /// resolve to the same [`InstructionPolicy`] — "one process, one
    /// policy — so no composition happens, and that is the ruling, not an
    /// omission".
    ///
    /// `local` no longer refuses here (MVP-2 D2 item 1): its launch-side
    /// translation is measured (`setting_sources_args`,
    /// `docs/gauntlet/notes/d2-setting-sources-measurement-2026-08-12.md`),
    /// so the L1 gate this refusal existed to enforce is satisfied. The
    /// conflict rule is unchanged — mixing policies within one bind is still
    /// refused, because "one process, one `--setting-sources`" is a fact
    /// about the launch grammar, not about what any single value measures to.
    fn check_instruction_policy(
        estate: &Estate,
        repositories: &[RepositorySpec],
    ) -> Result<(), EngineError> {
        if repositories.is_empty() {
            return Ok(());
        }
        let resolved: Vec<(String, InstructionPolicy)> = repositories
            .iter()
            .map(|r| (r.name.clone(), estate.instruction_policy(&r.name)))
            .collect();
        let first = resolved[0].1;
        if resolved.iter().any(|(_, policy)| *policy != first) {
            return Err(EngineError::InstructionPolicyConflict {
                repos: resolved
                    .iter()
                    .map(|(name, policy)| format!("{name}={policy}"))
                    .collect(),
            });
        }
        // INV-R1-09 (MVP-2 D3 fixer pass): `local`'s adapter-side
        // translation (`--setting-sources user,project,local`) authorizes
        // more than its name suggests — hooks, tool permissions, MCP
        // servers from the repository's own `.claude/settings*.json`, not
        // just "reads a text file" (measured,
        // `docs/gauntlet/notes/d2-setting-sources-measurement-2026-08-12.md`).
        // The submit-time refusal that gated this on an unmeasured claim
        // was correctly removed once the value was measured (D2 item 1),
        // but nothing replaced it as an operator-visible signal — no
        // warning at submit, no doctor row, no manifest vocabulary change.
        // A daemon log line is the cheapest honest partial fix available in
        // this pass without inventing new journal schema or CLI surface for
        // a risk-disclosure feature the contract never asked for; it does
        // not close the finding's full ask (a first-class operator signal),
        // only makes the choice observable to whoever runs the daemon.
        if first == InstructionPolicy::Local {
            tracing::warn!(
                repos = ?resolved.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>(),
                "submitting with instructions = \"local\": every selected repository's own \
                 .claude/settings.json and .claude/settings.local.json take effect for this \
                 run (hooks, tool permissions, MCP servers) -- a materially larger authority \
                 surface than reading an instruction file"
            );
        }
        Ok(())
    }

    /// The uniform [`InstructionPolicy`] a bound run resolved to
    /// (`check_instruction_policy` refuses submission unless every selected
    /// repository agreed, so any entry speaks for all of them), read back off
    /// the identities `workflow.bound` pinned rather than re-derived from the
    /// live manifest — a restarted turn must launch under the policy the run
    /// actually started with, not whatever the manifest says today (MVP-2
    /// D2 item 1). A run bound before R-MVP1-4 pinned no identities at all,
    /// which resolves to [`InstructionPolicy::Suppress`] the same way an
    /// absent manifest entry does.
    fn run_instruction_policy(run: &WorkRun) -> InstructionPolicy {
        run.instruction_identities
            .first()
            .map(|identity| identity.policy)
            .unwrap_or_default()
    }

    /// R-MVP1-4's R7: resolve the instruction-file identity `workflow.bound`
    /// pins for each bound repository. Reads from the *materialized
    /// worktree* (`surface`'s bindings), not the source repository: that is
    /// the copy the actor would read under `local`, and pinning it after
    /// materialization (rather than at plan time, before a worktree exists)
    /// is exactly what makes the hash mean anything *for that policy*. A
    /// missing file is recorded as absent — `path`/`content_hash` both
    /// `None` — never silently skipped.
    ///
    /// **W7, then MVP-2 D2 item 1:** runs unconditionally, for every bound
    /// repository regardless of its resolved `policy`. `INSTRUCTION_FILE`'s
    /// own doc has the measured fact, for *both* policies now: this
    /// adapter's launch grammar (`suppress` or the newly-unrefused `local`)
    /// never reads this file natively, so "the file the actor will read is
    /// the one we recorded" does not hold for either — see
    /// `docs/gauntlet/notes/d2-setting-sources-measurement-2026-08-12.md`.
    /// What ships is a correctly-computed, correctly-pinned identity for a
    /// file this adapter does not currently read under any policy — honest
    /// bookkeeping for a mechanism the manifest schema anticipates but this
    /// backend has not implemented, not proof of consumption.
    fn resolve_instruction_identities(
        estate: &Estate,
        surface: &WorkSurface,
    ) -> Vec<InstructionIdentity> {
        surface
            .bindings
            .iter()
            .map(|binding| {
                let policy = estate.instruction_policy(&binding.repository);
                let file = binding
                    .worktree_path
                    .join(crate::domain::estate::INSTRUCTION_FILE);
                let (path, content_hash) = match std::fs::read(&file) {
                    Ok(bytes) => (Some(file), Some(blake3::hash(&bytes).to_hex().to_string())),
                    Err(_) => (None, None),
                };
                InstructionIdentity {
                    repository: binding.repository.clone(),
                    policy,
                    path,
                    content_hash,
                }
            })
            .collect()
    }

    /// Resolve every stage's executor before anything exists (§12.5, §17.5).
    ///
    /// This is the whole-workflow capability preflight. It walks the pinned
    /// stage order and, for each stage, answers the two questions §12.5 poses:
    /// *which harness runs this checkpoint* (an explicit `stage.harness`, else
    /// the Work actor default, else fail with the available harnesses) and
    /// *under which profile*. Both answers are checked against the registry as
    /// it is now — a named harness must be registered **and** probe available
    /// — so §17.5's "reject before Work or worktree side effects" is a
    /// property of when this runs, not of a later guard.
    ///
    /// The probes it performs are also what makes the reservation phase cheap:
    /// every harness this run will ever touch has been probed here, outside
    /// the core lock, so the `prepare` the engine later calls under the lock
    /// reads a warm cache instead of forking a version check (§22.6's
    /// "probing a slow external executable"). That dependency is pinned by a
    /// test rather than left to luck.
    fn bind_stages(
        &self,
        estate: &Estate,
        workflow: &WorkflowDefinition,
        route: &Route,
        work_profile: Option<&Profile>,
    ) -> Result<Vec<StageBinding>, EngineError> {
        let mut bindings = Vec::with_capacity(workflow.stages.len());
        for (index, stage) in workflow.stages.iter().enumerate() {
            if stage.kind == StageKind::Execute {
                // §17.5: "Docker execute capability when any execute stage
                // exists" is a requirement distinct from — and never
                // substitutable by — the actor harness chain. An execute
                // stage always runs on the fixed `"docker"` backend; there is
                // no author-facing harness choice to make (§20.6: container
                // policy lives in the stage, not in a harness selection).
                // Routing it through `route_stage`'s decisive-tier resolver
                // gets the same "unavailable means fail here, not fall
                // through" guarantee an explicit `stage.harness` gets, for
                // free, and the same probe-availability check that makes
                // this preflight (not a later surprise at stage entry).
                let stage_route = route_stage(Some(DOCKER_BACKEND_NAME), None, &self.backends)
                    .map_err(|e| EngineError::ExecuteBackendUnavailable {
                        workflow: workflow.name.clone(),
                        stage: stage.id.clone(),
                        detail: e.to_string(),
                    })?;
                bindings.push(StageBinding {
                    stage_id: stage.id.clone(),
                    index,
                    kind: stage.kind,
                    harness: stage_route.backend,
                    route_source: stage_route.source.as_str().to_string(),
                    // §20.6: profiles are harness launch configuration.
                    // Execute stages have none — their policy is the pinned
                    // `ExecuteSpec` carried on the `StageDefinition` itself,
                    // journaled whole in `workflow.bound`.
                    profile: None,
                });
                continue;
            }
            let stage_route = route_stage(
                stage.harness.as_deref(),
                Some(route.backend.as_str()),
                &self.backends,
            )?;
            // R-MVP1-11: the smallest declaration that makes the ask
            // preflight real. `route_stage` above already proved this
            // backend is registered, so the lookup here cannot miss.
            if stage.requires_ask
                && let Some(backend) = self.backends.get(&stage_route.backend)
                && !backend.capabilities().ask
            {
                return Err(EngineError::AskCapabilityUnavailable {
                    workflow: workflow.name.clone(),
                    stage: stage.id.clone(),
                    backend: stage_route.backend.clone(),
                });
            }
            let profile = self.stage_profile(estate, stage, &stage_route, work_profile)?;
            bindings.push(StageBinding {
                stage_id: stage.id.clone(),
                index,
                kind: stage.kind,
                harness: stage_route.backend,
                route_source: stage_route.source.as_str().to_string(),
                profile,
            });
        }
        Ok(bindings)
    }

    /// One stage's launch profile: its own `[stage."<id>"] profile`, else the
    /// Work-level one it inherits — and in both cases the profile must belong
    /// to the harness this stage actually runs on (§22.4: "stage profile
    /// belongs to its named harness").
    ///
    /// The inherited case is the one worth stating. A Work submitted
    /// `--profile analysis` (a Claude profile) whose stage 10 declares
    /// `harness = "codex"` has no honest answer: applying the profile would
    /// hand Codex a Claude executable and permission mode, and dropping it
    /// silently would run the stage under configuration the human never chose.
    /// Both are the substitution §12.5 forbids, so it is refused here — before
    /// the Work exists — naming the stage, so the fix (`profile` on that
    /// stage's table) is obvious.
    fn stage_profile(
        &self,
        estate: &Estate,
        stage: &StageDefinition,
        stage_route: &Route,
        work_profile: Option<&Profile>,
    ) -> Result<Option<Profile>, EngineError> {
        let (profile, tier) = match stage.profile.as_deref() {
            Some(name) => (
                Some(Self::estate_profile(estate, name)?),
                format!("stage {:?}", stage.id),
            ),
            None => (
                work_profile.cloned(),
                format!("inherited by stage {:?}", stage.id),
            ),
        };
        let Some(profile) = profile else {
            return Ok(None);
        };
        if profile.backend != stage_route.backend {
            return Err(EngineError::ProfileBackendMismatch {
                profile: profile.name,
                profile_backend: profile.backend,
                routed: stage_route.backend.clone(),
                tier,
            });
        }
        Ok(Some(profile))
    }

    /// A profile the estate declares, or §14's "name them consistently"
    /// error with the names that do exist.
    fn estate_profile(estate: &Estate, name: &str) -> Result<Profile, EngineError> {
        estate
            .profile(name)
            .cloned()
            .ok_or_else(|| EngineError::ProfileNotFound {
                requested: name.to_string(),
                available: estate
                    .profiles
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            })
    }

    /// Run §13's chain for a submission that has no repository surface, and
    /// keep only its verdict on the tiers that named something.
    ///
    /// [`RouteError::NoSelection`] is the one outcome this tolerates: nothing
    /// was asked for and nothing will run, so there is no substitution to
    /// refuse and no option list to put in front of anyone. Everything else —
    /// an unknown name, an unavailable backend — is the failure §13 requires,
    /// raised now, instead of a `pending` work that silently records a
    /// backend selection it can never honour.
    fn check_selection_is_honourable(
        &self,
        context: &SubmitContext<'_>,
    ) -> Result<(), EngineError> {
        match route(
            &RouteInputs {
                explicit: context.backend,
                origin_client: context.origin_client,
                estate_default: None,
                global_default: self.default_backend.as_deref(),
            },
            &self.backends,
        ) {
            Ok(_) | Err(RouteError::NoSelection { .. }) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// Start a planned run: materialize the surface, pin the workflow, move
    /// the work to `active`, and enter the first stage.
    ///
    /// A failure *after* the Work exists cannot un-accept it, so it fails
    /// closed to `blocked` with the reason recorded — never to a silent
    /// `pending` that nothing will ever pick up. A *crash* after the Work
    /// exists cannot be caught here at all, so the sequence is ordered to be
    /// recoverable from the journal alone: the intent to materialize is
    /// appended before `git worktree add` touches the user's repository, and
    /// [`Engine::reconcile_crashed_start`] turns whatever prefix survives
    /// into a `blocked` work with that evidence.
    pub fn start(&self, core: &mut Core, work: &Work, plan: &StartPlan) -> Result<(), EngineError> {
        let step = self.begin_start(core, work, plan)?;
        self.run_inline(core, step)
    }

    /// The [`SurfacePlan`] `surface.materializing` records for `plan`,
    /// carrying §8.1's admission when the plan has one.
    ///
    /// Split out rather than inlined so the "with admission" and "without"
    /// shapes are one decision in one place: a plan made for a real
    /// submission always has a [`GitPreflight`], and one made without a
    /// `work_id` (routing probes, fixtures) never does.
    fn surface_plan(plan: &StartPlan, work_id: &str) -> SurfacePlan {
        let base = SurfacePlan::new(&plan.surfaces_root, work_id, &plan.repositories);
        match &plan.preflight {
            Some(preflight) => base.with_admission(preflight),
            None => base,
        }
    }

    /// [`Engine::start`]'s first phase: journal the intent to materialize, and
    /// hand the git back to the caller.
    ///
    /// This used to run `git worktree add` — three fork/exec/wait cycles per
    /// repository — under the caller's guard. It is now §14.2's first phase and
    /// nothing more: one append, then a [`Next::Surface`] the daemon performs
    /// with the lock released. The binding, the `work.started` transition and
    /// the first stage's reservation all happen in
    /// [`Engine::settle_surface`], back under the lock, exactly as before.
    ///
    /// **L6 audit.** The append order is unchanged and so are its windows: the
    /// intent to materialize is durable before anything can create a branch in
    /// a repository sergeant does not own, and
    /// [`Engine::reconcile_crashed_start`] still turns whatever prefix
    /// survives into a `blocked` work with that inventory as its evidence.
    /// Releasing the guard in the middle adds no window a crash did not
    /// already have — a crash and a released lock leave the journal in exactly
    /// the same state — it adds the *concurrency* case, which `settle_surface`
    /// answers with §14.5's rule.
    pub fn begin_start(
        &self,
        core: &mut Core,
        work: &Work,
        plan: &StartPlan,
    ) -> Result<Step, EngineError> {
        // Recorded first: everything the returned effect does can create a
        // branch and a worktree in a repository sergeant does not own.
        // R-MVP1-1: `plan.surfaces_root`, not `self.data_dir` — the plan
        // already resolved any `[estate] surfaces_dir` override at submit.
        self.commit(
            core,
            &work.id,
            KIND_SURFACE_MATERIALIZING,
            // §8.3: "the override and every waived finding are journaled with
            // the Work" — carried on the plan, so the authorization is durable
            // strictly before the effect it authorized, exactly like the paths
            // and branches beside it.
            json!({"plan": Self::surface_plan(plan, &work.id)}),
        )?;
        Ok(Step {
            next: Next::Surface(Box::new(PendingSurface {
                work_id: work.id.clone(),
                data_dir: self.data_dir.clone(),
                effect: SurfaceEffect::Materialize {
                    surfaces_root: plan.surfaces_root.clone(),
                    plan: Box::new(plan.clone()),
                },
            })),
            deferred: Deferred::new(),
        })
    }

    /// §14.2's third phase for a git operation.
    ///
    /// Takes the boxed pending by value for the same reason the launch phase
    /// does: `Next` carries it boxed, and unboxing at every call site to hand
    /// it back would be churn, not clarity.
    #[allow(clippy::boxed_local)]
    pub fn settle_surface(
        &self,
        core: &mut Core,
        pending: Box<PendingSurface>,
        outcome: SurfaceOutcome,
    ) -> Result<Step, EngineError> {
        let work_id = pending.work_id.clone();
        match (pending.effect, outcome) {
            (SurfaceEffect::Materialize { plan, .. }, SurfaceOutcome::Materialized(result)) => {
                self.settle_materialize(core, &work_id, &plan, result)
            }
            (
                SurfaceEffect::Rematerialize {
                    surface,
                    index,
                    attempt,
                },
                SurfaceOutcome::Rematerialized(result),
            ) => self.settle_rematerialize(core, &work_id, &surface, index, attempt, result),
            (SurfaceEffect::Teardown { recovered, .. }, SurfaceOutcome::TornDown(report)) => {
                // §11.5's orthogonal axis, computed at the one point every
                // terminal path converges on: cancel (`begin_retire_run`),
                // failure and completion (`settle_stage`'s signal arms), and
                // the crash-recovery re-run (`reconcile_terminal_surface`,
                // which reaches here through this same arm and therefore
                // records the same assessment, marked `recovered`).
                //
                // Deliberately *not* computed inside `teardown()`: the same
                // function also runs as `materialize`'s partial-failure
                // rollback, which is not a retirement and journals its report
                // with no `integrity` key at all (see `settle_materialize`).
                // An absent key means not assessed — never clean (C3).
                //
                // A sibling key rather than a field inside `report`: additive
                // to the payload, so an old `surface.torn_down` deserializes
                // into the same `TeardownReport` it always did.
                let integrity = report.integrity();
                let mut payload = json!({"report": report, "integrity": integrity});
                if recovered {
                    payload["recovered"] = Value::Bool(true);
                }
                self.commit(core, &work_id, KIND_SURFACE_TORN_DOWN, payload)?;
                Ok(Step::parked())
            }
            // Unreachable by construction — every producer pairs the effect
            // with its own outcome — and a silent no-op would be the worst
            // possible answer if it ever became reachable, since the git has
            // already run. Fail closed with the mismatch named.
            (_, outcome) => {
                let detail =
                    format!("surface effect settled with a mismatched outcome: {outcome:?}");
                self.block(
                    core,
                    &work_id,
                    "internal surface-phase mismatch",
                    Some(detail),
                )?;
                Ok(Step::parked())
            }
        }
    }

    fn settle_materialize(
        &self,
        core: &mut Core,
        work_id: &str,
        plan: &StartPlan,
        result: Result<WorkSurface, SurfaceError>,
    ) -> Result<Step, EngineError> {
        let surface = match result {
            Ok(surface) => surface,
            Err(SurfaceError::PartialFailure { source, teardown }) => {
                // Earlier repositories in this request got a real worktree
                // and branch before a later one failed; materialize() has
                // already torn those down. Journal the report so it is not a
                // mystery, and put it in the evidence too.
                self.commit(
                    core,
                    work_id,
                    KIND_SURFACE_TORN_DOWN,
                    json!({"report": teardown}),
                )?;
                self.block(
                    core,
                    work_id,
                    &format!("cannot materialize work surface: {source}"),
                    Some(serde_json::to_string(&teardown).unwrap_or_default()),
                )?;
                return Ok(Step::parked());
            }
            Err(e) => {
                self.block(
                    core,
                    work_id,
                    &format!("cannot materialize work surface: {e}"),
                    None,
                )?;
                return Ok(Step::parked());
            }
        };
        // The surface exists in the user's repositories now, so it is recorded
        // before anything else is decided — including when the decision turns
        // out to be "this work is gone". §22.5's "no owned external identity
        // lost" is not only about harness sessions.
        self.commit(
            core,
            work_id,
            KIND_SURFACE_MATERIALIZED,
            json!({"surface": surface}),
        )?;
        // §14.5, for the phase that created worktrees: a cancel may have
        // landed while git ran. A late materialization does not start a work
        // that is no longer pending — it hands the worktrees straight back to
        // teardown instead of leaving them on disk with no owner.
        if let Some(why) = self.start_is_stale(core, work_id) {
            tracing::info!(
                work_id,
                why,
                "tearing down a surface whose start was superseded"
            );
            return Ok(Step {
                next: self.teardown_next(core, work_id, false),
                deferred: Deferred::new(),
            });
        }
        // R-MVP1-4: widened from `estate: <name>` to the resolved
        // repository set plus per-repo instruction-policy identities —
        // additive fields in an immutable event, so a mid-flight manifest
        // edit cannot reach a Work already bound. `check_instruction_policy`
        // already refused disagreement and `local` at submit, so `policy` is
        // uniform here by construction; identities are still resolved and
        // recorded regardless — R7's "the file the actor will read is the
        // one we recorded" does not wait on `local` being enabled.
        let instruction_identities = Self::resolve_instruction_identities(&plan.estate, &surface);
        self.commit(
            core,
            work_id,
            KIND_WORKFLOW_BOUND,
            json!({
                "workflow": plan.workflow,
                "backend": plan.route.backend,
                "route_source": plan.route.source,
                "profile": plan.profile,
                "workspace": plan.estate.name,
                "repositories": plan.repositories,
                "instruction_identities": instruction_identities,
                // §12.5's per-stage decisions, pinned with the procedure they
                // belong to: the whole executor spec for the run, decided once
                // and never re-derived (§22.4's retry and restart rows).
                "stage_bindings": plan.stage_bindings,
            }),
        )?;
        self.transition(
            core,
            work_id,
            KIND_WORK_STARTED,
            json!({"backend": plan.route.backend, "workflow": plan.workflow.name}),
        )?;
        let next = self.reserve_stage(core, work_id, 0, 1)?;
        Ok(Step {
            next,
            deferred: Deferred::new(),
        })
    }

    /// Why a materialization's result may no longer start a run, or `None`.
    ///
    /// The work must still exist and still be `pending`: `work.started` is the
    /// only transition out of `pending` the engine makes, and anything else
    /// that moved it — a cancel — did so deliberately.
    fn start_is_stale(&self, core: &Core, work_id: &str) -> Option<String> {
        match core.registry.state().works.get(work_id) {
            None => Some("the work no longer exists".to_string()),
            Some(work) if work.state != WorkState::Pending => Some(format!(
                "work is {} — a surface that finished afterwards cannot start it",
                work.state
            )),
            Some(_) => None,
        }
    }

    /// Drive a [`Step`] to a park **inline**, performing every launch and
    /// waiting for every completion on this thread.
    ///
    /// This is the single-owner path: correct wherever the caller is not
    /// sharing the core with concurrent requests — startup recovery (nothing
    /// is served yet), the deterministic tests, and the sync wrappers below.
    /// The daemon's request path uses the same primitives with the guard
    /// dropped between phases (`api::crank`), so there is one implementation
    /// of the lifecycle and two lock policies over it, rather than two
    /// lifecycles that can drift.
    fn run_inline(&self, core: &mut Core, step: Step) -> Result<(), EngineError> {
        let mut step = step;
        loop {
            let Step { next, deferred } = step;
            deferred.wait();
            match next {
                Next::Parked => return Ok(()),
                Next::Launch(pending) => {
                    let outcome = pending.perform();
                    step = self.settle_launch(core, pending, outcome)?;
                }
                Next::Send(pending) => {
                    let outcome = pending.perform();
                    step = self.settle_send(core, pending, outcome)?;
                }
                Next::Surface(pending) => {
                    let outcome = pending.perform();
                    step = self.settle_surface(core, pending, outcome)?;
                }
                Next::Observe(pending) => {
                    let outcome = pending.perform();
                    step = self.settle_observe(core, pending, outcome)?;
                }
                Next::Interrupt(pending) => {
                    let outcome = pending.perform();
                    step = self.settle_interrupt(core, *pending, outcome)?;
                }
            }
        }
    }

    /// Deliver an answer to a work that asked for input (§12's needs-input
    /// verb, §26's `work.respond`). Input to a work that is not waiting is a
    /// structured error, not a silent no-op.
    pub fn provide_input(
        &self,
        core: &mut Core,
        work_id: &str,
        input: &str,
    ) -> Result<(), EngineError> {
        let step = self.begin_input(core, work_id, input)?;
        self.run_inline(core, step)
    }

    /// [`Engine::provide_input`]'s first phase (see [`Engine::begin_start`]).
    pub fn begin_input(
        &self,
        core: &mut Core,
        work_id: &str,
        input: &str,
    ) -> Result<Step, EngineError> {
        let state = self.work_state(core, work_id)?;
        if state != WorkState::NeedsInput {
            return Err(EngineError::NotAwaitingInput {
                work_id: work_id.to_string(),
                state,
            });
        }
        let run = self.run(core, work_id)?;
        let current = run.current_stage().ok_or_else(|| EngineError::NoRun {
            work_id: work_id.to_string(),
        })?;
        let stage_id = current.stage_id.clone();
        let index = current.index;
        let attempt = current.attempt;
        let execution = run.execution.clone().ok_or_else(|| EngineError::NoRun {
            work_id: work_id.to_string(),
        })?;
        let backend = self.backend_for(work_id, &execution.backend)?;

        self.commit(
            core,
            work_id,
            KIND_STAGE_INPUT_RECEIVED,
            json!({"stage_id": stage_id, "input": input}),
        )?;
        // R-MVP1-7: SEND is the engine's other turn-spawning seam. The
        // human's answer is journaled above either way — it is never lost —
        // but if the envelope is already spent, it is never delivered: the
        // Work goes to `blocked` (a legal `needs_input -> blocked` edge)
        // instead of `resumed`, and no `PendingSend` is ever built.
        if !self.check_turn_envelope(core, work_id, &stage_id)? {
            return Ok(Step::parked());
        }
        self.transition(
            core,
            work_id,
            KIND_WORK_RESUMED,
            json!({"reason": "input_received"}),
        )?;
        // The delivery itself is the external effect and goes back to the
        // caller (§14.2): `ClaudeBackend::send` forks a `claude -p --resume`
        // turn and three reader threads, which is precisely what §22.6 forbids
        // under the guard. The two appends above are the authoritative half
        // and stay here — the answer is durable before anything is spawned, so
        // a crash in the window loses the turn, never the human's words.
        //
        // The stage record itself is **not** flipped back to `Active` here
        // (INV-4, BS2): that would happen before the delivery has actually
        // reached the backend, and the completion driver's `due_observations`
        // requires only `StageStatus::Active` to poll a run — it would race
        // this very delivery for the same turn. `Engine::settle_send` does
        // the flip instead, after the effect, atomically with applying
        // whatever the delivery's own observation already says (§14.5).
        Ok(Step {
            next: Next::Send(Box::new(PendingSend {
                work_id: work_id.to_string(),
                stage_id,
                index,
                attempt,
                execution,
                backend,
                input: input.to_string(),
                resume: self.resume_request(&run, work_id),
            })),
            deferred: Deferred::new(),
        })
    }

    /// Re-enter the current stage (§12's retry verb).
    ///
    /// Retry is the one door back out of `failed`, `blocked` and `waiting`,
    /// and it is always explicit — nothing retries itself. The surface is
    /// re-attached to the branch teardown retained, so a retry continues on
    /// the work the failed attempt left behind.
    pub fn retry(&self, core: &mut Core, work_id: &str) -> Result<(), EngineError> {
        let step = self.begin_retry(core, work_id)?;
        self.run_inline(core, step)
    }

    /// R-MVP1-10's exit door for R-MVP1-7's envelope-exhausted landing:
    /// raise this Work's turn envelope by `additional_turns`, journaled
    /// (`KIND_TURN_ENVELOPE_EXTENDED`) and cumulative across every call.
    ///
    /// `retry` alone is a revolving door for this landing: `WorkRun::
    /// turns_spawned` is cumulative and never reset by a retry, and
    /// `Engine::turn_cap` has no per-Work setter, so re-entering the stage
    /// re-checks the identical, unchangeable condition and re-blocks
    /// immediately (`check_turn_envelope`'s own doc). This is the door that
    /// actually changes the condition: `check_turn_envelope` compares
    /// against `turn_cap + turn_cap_bonus`, so a `retry` issued after this
    /// call can spawn the next turn instead of re-blocking on the same one.
    ///
    /// Legal only from `blocked` — extending an envelope means nothing for
    /// a Work that is not stopped at one, and every landing this door is
    /// meant to open is one, by construction (`check_turn_envelope` always
    /// blocks before returning `Ok(false)`).
    pub fn extend_turn_envelope(
        &self,
        core: &mut Core,
        work_id: &str,
        additional_turns: u32,
    ) -> Result<(), EngineError> {
        let state = self.work_state(core, work_id)?;
        if state != WorkState::Blocked {
            return Err(EngineError::NotBlocked {
                work_id: work_id.to_string(),
                state,
            });
        }
        // A run must exist to extend: the envelope-exhausted landing this
        // door targets always has one (`check_turn_envelope` only ever runs
        // inside an active stage), so a `blocked` Work with none is some
        // other landing this door was never meant to touch — `self.run`
        // fails closed with `NoRun` rather than journal an extension
        // nothing will ever read.
        self.run(core, work_id)?;
        self.commit(
            core,
            work_id,
            KIND_TURN_ENVELOPE_EXTENDED,
            json!({"additional_turns": additional_turns}),
        )
    }

    /// [`Engine::retry`]'s first phase (see [`Engine::begin_start`]).
    pub fn begin_retry(&self, core: &mut Core, work_id: &str) -> Result<Step, EngineError> {
        let state = self.work_state(core, work_id)?;
        if !matches!(
            state,
            WorkState::Failed | WorkState::Blocked | WorkState::Waiting
        ) {
            return Err(EngineError::NotRetryable {
                work_id: work_id.to_string(),
                state,
            });
        }
        let run = self.run(core, work_id)?;
        let current = run
            .current_stage()
            .cloned()
            .ok_or_else(|| EngineError::NoRun {
                work_id: work_id.to_string(),
            })?;

        // A terminal failure tore the worktrees down and kept the branches;
        // re-attach before re-entering the stage. This needs no
        // `surface.materializing` marker of its own: re-attachment can only
        // recreate paths and a branch the recorded surface already names, so
        // a crash here leaves nothing the journal has not already declared.
        //
        // It is git, so it is §14.2's middle phase like every other git: the
        // effect goes back to the caller and the re-entry happens in
        // `settle_surface`. A run with no recorded surface has nothing to
        // re-attach and re-enters right here, under this lock.
        if let Some(surface) = run.surface.clone() {
            return Ok(Step {
                next: Next::Surface(Box::new(PendingSurface {
                    work_id: work_id.to_string(),
                    data_dir: self.data_dir.clone(),
                    effect: SurfaceEffect::Rematerialize {
                        surface,
                        index: current.index,
                        attempt: current.attempt + 1,
                    },
                })),
                deferred: Deferred::new(),
            });
        }
        self.reenter_stage(
            core,
            work_id,
            &current.stage_id,
            current.index,
            current.attempt + 1,
        )
    }

    fn settle_rematerialize(
        &self,
        core: &mut Core,
        work_id: &str,
        surface: &WorkSurface,
        index: usize,
        attempt: u32,
        result: Result<Option<WorkSurface>, SurfaceError>,
    ) -> Result<Step, EngineError> {
        match result {
            Ok(Some(surface)) => self.commit(
                core,
                work_id,
                KIND_SURFACE_MATERIALIZED,
                json!({"surface": surface, "rematerialized": true}),
            )?,
            // Nothing needed re-attaching: the worktrees were still there.
            Ok(None) => {}
            Err(e) => return Err(e.into()),
        }
        // §14.5 again: the retry may have been overtaken while git ran. The
        // paths this recreated are ones the recorded surface already named, so
        // nothing new is owned — but the run must not be re-entered behind a
        // newer decision's back.
        if !matches!(
            self.work_state(core, work_id)?,
            WorkState::Failed | WorkState::Blocked | WorkState::Waiting
        ) {
            tracing::info!(
                work_id,
                "a retry was superseded while its surface re-attached"
            );
            return Ok(Step::parked());
        }
        let stage_id = surface
            .bindings
            .first()
            .map(|_| ())
            .and_then(|()| self.run(core, work_id).ok())
            .and_then(|run| run.current_stage().map(|s| s.stage_id.clone()))
            .unwrap_or_default();
        self.reenter_stage(core, work_id, &stage_id, index, attempt)
    }

    /// The half of retry that is pure journal: resume the work and reserve the
    /// stage's next attempt.
    fn reenter_stage(
        &self,
        core: &mut Core,
        work_id: &str,
        stage_id: &str,
        index: usize,
        attempt: u32,
    ) -> Result<Step, EngineError> {
        self.transition(
            core,
            work_id,
            KIND_WORK_RESUMED,
            json!({"reason": "retry", "stage_id": stage_id}),
        )?;
        let next = self.reserve_stage(core, work_id, index, attempt)?;
        Ok(Step {
            next,
            deferred: Deferred::new(),
        })
    }

    /// Retire a run whose work has just reached a terminal state.
    ///
    /// Ordering is deliberate and is the answer to the M3 contract's open
    /// question about teardown versus execution retirement: the Work state
    /// change has already been journaled by the caller, *then* the stage is
    /// marked, *then* the backend is asked to stop, *then* the surface is torn
    /// down. Nothing here waits for the execution to actually die, and nothing
    /// here reads its liveness — a native context that ignores STOP leaves a
    /// canceled Work canceled, with a recorded stop request and a recorded
    /// teardown. That is §25's separation in the one place it is most
    /// tempting to violate.
    pub fn retire_run(
        &self,
        core: &mut Core,
        work_id: &str,
        reason: &str,
    ) -> Result<(), EngineError> {
        let step = self.begin_retire_run(core, work_id, reason)?;
        self.run_inline(core, step)
    }

    /// [`Engine::retire_run`]'s first phase (see [`Engine::begin_start`]).
    ///
    /// This is the one that mattered for issue #14: the STOP it issues used
    /// to join the Claude adapter's transcript-archive thread inline, under
    /// the daemon's core lock. Now the join rides home in the returned
    /// [`Step::deferred`] and the caller waits for it after releasing the
    /// guard — same promise, different place.
    pub fn begin_retire_run(
        &self,
        core: &mut Core,
        work_id: &str,
        reason: &str,
    ) -> Result<Step, EngineError> {
        let Ok(run) = self.run(core, work_id) else {
            return Ok(Step::parked()); // nothing ever started
        };
        // A stage that already reached its own conclusion keeps it: cancelling
        // a work that failed must not rewrite the failure as a cancellation.
        // Only a stage still in flight (or parked) is marked canceled.
        if let Some(stage) = run.current_stage()
            && matches!(
                stage.status,
                StageStatus::Active
                    | StageStatus::Waiting
                    | StageStatus::NeedsInput
                    | StageStatus::Blocked
            )
        {
            self.commit(
                core,
                work_id,
                KIND_STAGE_CANCELED,
                json!({"stage_id": stage.stage_id, "detail": reason}),
            )?;
        }
        let mut deferred = Deferred::new();
        self.stop_execution(core, work_id, reason, &mut deferred)?;
        // A reservation whose launch has not reported back is *not* covered by
        // the STOP above: `stop_execution` reads `run.execution`, and a
        // reservation has no execution yet. Left standing, it is an open
        // two-phase window on a work that has already concluded — `work show`
        // keeps reporting an outstanding reservation on a canceled work, and a
        // crash before the in-flight launch settles leaves a record no restart
        // will ever look at, because reconciliation skips terminal work.
        self.abandon_reservation(core, work_id, &run, "work_retired", reason)?;
        Ok(Step {
            next: self.teardown_next(core, work_id, false),
            deferred,
        })
    }

    /// Close an outstanding reservation on a work that has gone terminal.
    ///
    /// Bookkeeping only, and deliberately so. Sergeant committed to an
    /// execution identity and cannot tell whether the external effect ever
    /// happened, so this closes the *record* and nothing else: nothing is
    /// started, nothing is stopped, nothing is removed (§22.5's "no unproven
    /// state deleted"), and the reserved identity travels into the event so an
    /// operator has the thing to go and look for.
    ///
    /// Returns whether an event was appended, so recovery can report it.
    fn abandon_reservation(
        &self,
        core: &mut Core,
        work_id: &str,
        run: &WorkRun,
        reason: &str,
        detail: &str,
    ) -> Result<bool, EngineError> {
        let Some(reservation) = run.unsettled_reservation().cloned() else {
            return Ok(false);
        };
        let identity = match &reservation.native_id {
            Some(native_id) => format!("reserved native identity {native_id}"),
            None => "no native identity had been reserved yet".to_string(),
        };
        self.commit(
            core,
            work_id,
            KIND_EXECUTION_ABANDONED,
            json!({
                "execution_id": reservation.execution_id,
                "backend": reservation.backend,
                "native_id": reservation.native_id,
                "stage_id": reservation.stage_id,
                "attempt": reservation.attempt,
                "reason": reason,
                "detail": format!(
                    "{detail}; the launch of execution {} ({identity}) never reported back, so \
                     the reservation is closed as abandoned — sergeant did not start it a \
                     second time and removed nothing it cannot prove it owns",
                    reservation.execution_id
                ),
                // Unknown, and said so: `false` would claim the effect never
                // happened, which is exactly what this window cannot know.
                "launched": Value::Null,
            }),
        )?;
        Ok(true)
    }

    /// Close a reservation a crash left outstanding on a work that is already
    /// terminal (§22.5's cancel-during-launch window).
    ///
    /// The non-crash path is handled by [`Engine::begin_retire_run`]; this is
    /// the same closure for the daemon that died between the cancel landing
    /// and `settle_launch` running. Restart reconciliation never looked at
    /// terminal work for this, so the reservation stayed open forever and
    /// `work show` kept reporting it.
    pub fn reconcile_terminal_reservation(
        &self,
        core: &mut Core,
        work_id: &str,
    ) -> Result<bool, EngineError> {
        let run = self.run(core, work_id)?;
        let state = self.work_state(core, work_id)?;
        self.abandon_reservation(
            core,
            work_id,
            &run,
            "unsettled_at_restart",
            &format!("this work is {state} and a daemon restart found its reservation open"),
        )
    }

    /// Drive a run forward from whatever its backend now says.
    ///
    /// This is the only place stage progression happens, and the only signals
    /// it acts on are the backend's explicit ones.
    ///
    /// **A single-owner path**, exactly as [`Engine::run_inline`] is: finding
    /// out where the run *is* means an OBSERVE, and this function takes it
    /// with a `&mut Core` in hand. That is only safe where the caller holds
    /// the only reference to the Core there is — recovery and the
    /// deterministic tests. The daemon's request path never calls it: every
    /// observation it acts on rides home in a performer's outcome, taken with
    /// the guard released (§22.6, and m6's t11 enforces it structurally).
    pub fn resume(&self, core: &mut Core, work_id: &str) -> Result<(), EngineError> {
        let run = self.run(core, work_id)?;
        let Some(execution) = run.execution.clone() else {
            return Ok(());
        };
        let backend = self.backend_for(work_id, &execution.backend)?;
        let observed = backend.observe(&handle_of(&execution));
        let mut deferred = Deferred::new();
        let next = self.drive(core, work_id, observed, &mut deferred)?;
        self.run_inline(core, Step { next, deferred })
    }

    /// Act on one observation of the run's current execution.
    ///
    /// The observation is **always supplied**, never taken here. Every caller
    /// is either a settle phase — which was handed one by the performer that
    /// took it outside the guard — or a single-owner path that took its own
    /// before calling. An OBSERVE inside this function would be an external
    /// call under the core lock on the daemon's hottest path, and the version
    /// of it that used to sit behind an `Option::unwrap_or_else` here was
    /// invisible to every instrument in the tree (round-2 finding N3R2-03).
    ///
    /// It does not loop: completing a stage enters the next one, whose
    /// *launch* goes back to the caller so the guard is released between them.
    fn drive(
        &self,
        core: &mut Core,
        work_id: &str,
        initial: Result<Observation, BackendError>,
        deferred: &mut Deferred,
    ) -> Result<Next, EngineError> {
        let run = self.run(core, work_id)?;
        if run.execution.is_none() {
            return Ok(Next::Parked);
        }
        let Some(stage) = run.current_stage().cloned() else {
            return Ok(Next::Parked);
        };
        let workflow = run.workflow.clone().ok_or_else(|| EngineError::NoRun {
            work_id: work_id.to_string(),
        })?;

        let observation = match initial {
            Ok(observation) => observation,
            Err(e) => {
                // §25: the adapter cannot classify the native context.
                // Ambiguity fails closed, with the evidence recorded.
                self.commit(
                    core,
                    work_id,
                    KIND_STAGE_BLOCKED,
                    json!({"stage_id": stage.stage_id, "detail": e.to_string()}),
                )?;
                self.block(
                    core,
                    work_id,
                    "backend could not observe the execution",
                    Some(e.to_string()),
                )?;
                return Ok(Next::Parked);
            }
        };
        if observation.native == NativeState::Unknown {
            self.commit(
                core,
                work_id,
                KIND_STAGE_BLOCKED,
                json!({
                    "stage_id": stage.stage_id,
                    "detail": observation
                        .evidence
                        .clone()
                        .unwrap_or_else(|| "backend reports an unknown native state".to_string()),
                }),
            )?;
            self.block(
                core,
                work_id,
                "backend reports an unknown native state",
                observation.evidence.clone(),
            )?;
            return Ok(Next::Parked);
        }

        // From here on, only `observation.signal` is consulted. Native
        // liveness has already had its one and only say (Unknown above).
        match observation.signal {
            BackendSignal::Running => Ok(Next::Parked),
            BackendSignal::NeedsInput { prompt, asked_by } => {
                // GP-2: *who asked* travels with the question. The engine's
                // handling is identical either way — that is the point, the
                // ask primitive reuses `respond` rather than inventing a
                // second resume verb — but a trajectory that cannot say
                // whether a human was consulted because the actor asked or
                // because a gate fired has lost the fact the workflow author
                // was designing around.
                self.commit(
                    core,
                    work_id,
                    KIND_STAGE_NEEDS_INPUT,
                    json!({
                        "stage_id": stage.stage_id,
                        "detail": prompt,
                        "asked_by": asked_by.as_str(),
                    }),
                )?;
                self.transition(
                    core,
                    work_id,
                    KIND_WORK_NEEDS_INPUT,
                    json!({
                        "prompt": prompt,
                        "stage_id": stage.stage_id,
                        "asked_by": asked_by.as_str(),
                    }),
                )?;
                Ok(Next::Parked)
            }
            BackendSignal::Waiting { reason } => {
                self.commit(
                    core,
                    work_id,
                    KIND_STAGE_WAITING,
                    json!({"stage_id": stage.stage_id, "detail": reason}),
                )?;
                self.transition(
                    core,
                    work_id,
                    KIND_WORK_WAITING,
                    json!({"reason": reason, "stage_id": stage.stage_id}),
                )?;
                Ok(Next::Parked)
            }
            BackendSignal::Blocked { reason } => {
                self.commit(
                    core,
                    work_id,
                    KIND_STAGE_BLOCKED,
                    json!({"stage_id": stage.stage_id, "detail": reason}),
                )?;
                self.transition(
                    core,
                    work_id,
                    KIND_WORK_BLOCKED,
                    json!({"reason": reason, "stage_id": stage.stage_id}),
                )?;
                Ok(Next::Parked)
            }
            BackendSignal::Failed { reason } => {
                self.commit(
                    core,
                    work_id,
                    KIND_STAGE_FAILED,
                    json!({"stage_id": stage.stage_id, "detail": reason}),
                )?;
                self.stop_execution(core, work_id, "stage failed", deferred)?;
                self.transition(
                    core,
                    work_id,
                    KIND_WORK_FAILED,
                    json!({"reason": reason, "stage_id": stage.stage_id}),
                )?;
                Ok(self.teardown_next(core, work_id, false))
            }
            BackendSignal::StageCompleted { summary } => {
                let mut payload = json!({"stage_id": stage.stage_id, "index": stage.index});
                if let Some(summary) = summary {
                    payload["detail"] = Value::String(summary);
                }
                self.commit(core, work_id, KIND_STAGE_COMPLETED, payload)?;
                self.stop_execution(core, work_id, "stage completed", deferred)?;
                let next = stage.index + 1;
                if next < workflow.stages.len() {
                    // The next stage's reservation is committed here, under
                    // the same lock; its *launch* goes back to the caller.
                    // That is why this function no longer loops — the loop is
                    // the caller's, and every turn of it releases the lock.
                    return self.reserve_stage(core, work_id, next, 1);
                }
                self.transition(
                    core,
                    work_id,
                    KIND_WORK_COMPLETED,
                    json!({"stages": workflow.stages.len()}),
                )?;
                Ok(self.teardown_next(core, work_id, false))
            }
        }
    }

    /// Reattach, resume, classify one in-flight execution after a daemon
    /// restart (§25's own sequence, in that order).
    ///
    /// Returns the disposition it recorded. Unambiguous evidence resumes the
    /// run from wherever the backend now is; anything the adapter cannot
    /// classify — an unrecognised execution, an unreachable backend, an
    /// unknown native state — lands the work in `blocked` with the evidence,
    /// because §25's rule is that ambiguity fails closed.
    ///
    /// **Reattachment happens first, through §15 RESUME** ([`Engine::reattach`]).
    /// Observing an execution the restarted adapter has not re-adopted can
    /// only ever produce a classification, never a run that continues: the
    /// adapter has no owned context to SEND to afterwards, so every such work
    /// parks in `blocked` and the durable native context it names goes
    /// unclaimed. RESUME is what turns "the evidence says this is resumable"
    /// into "this daemon owns it again", and it is the adapter — not the
    /// engine — that decides whether the evidence supports the claim. An
    /// adapter that refuses fails the work closed exactly as before; nothing
    /// here softens ambiguity, and RESUME never starts a turn, so a
    /// reattached execution is the same execution, never a second one.
    ///
    /// L6 audit of the step: it opens no new append window. RESUME is called
    /// before the first append of this sequence and its only effect is in
    /// adapter memory — no process is started, nothing on disk changes — so a
    /// crash between reattaching and `execution.reconciled` loses the
    /// adoption along with the daemon that made it, leaves the work `active`,
    /// and the next restart reattaches again (adapters make RESUME
    /// idempotent for exactly this reason).
    pub fn reconcile_work(
        &self,
        core: &mut Core,
        work_id: &str,
    ) -> Result<ReconcileDisposition, EngineError> {
        let run = self.run(core, work_id)?;
        // §14.2's window, read first because it is the most recent fact the
        // journal has: an execution whose reservation is durable and whose
        // launch never reported back. It outranks any *earlier* execution
        // this run still records — that one belongs to a settled attempt,
        // and resuming it would be resuming the wrong thing.
        if let Some(reservation) = run.unsettled_reservation().cloned() {
            return self.reconcile_unsettled_reservation(core, work_id, &run, &reservation);
        }
        let Some(execution) = run.execution.clone() else {
            self.record_reconcile(
                core,
                work_id,
                None,
                ReconcileDisposition::Ambiguous,
                false,
                "work was active with no recorded execution",
            )?;
            if let Some(stage) = run.current_stage() {
                self.commit(
                    core,
                    work_id,
                    KIND_STAGE_BLOCKED,
                    json!({"stage_id": stage.stage_id, "detail": "no execution to reconcile"}),
                )?;
            }
            self.block(core, work_id, "no execution to reconcile", None)?;
            return Ok(ReconcileDisposition::Ambiguous);
        };
        let ambiguous = |detail: String| -> (ReconcileDisposition, String) {
            (ReconcileDisposition::Ambiguous, detail)
        };
        // Kept only on the `Resumed` path, so `drive` can act on the exact
        // Observation this decision was made from instead of asking the
        // backend a second time (which is not guaranteed to answer the same
        // way twice).
        let mut resumed_from: Option<Observation> = None;
        let mut reattached = false;
        let (disposition, evidence) =
            match self.backends.get(&execution.backend) {
                None => ambiguous(format!(
                    "backend {:?} is not registered in this daemon",
                    execution.backend
                )),
                Some(backend) => match self.reattach(backend, &run, work_id, &execution) {
                    Err(detail) => ambiguous(detail),
                    Ok(did) => {
                        reattached = did;
                        match backend.observe(&handle_of(&execution)) {
                            Err(e) => ambiguous(e.to_string()),
                            Ok(Observation {
                                native: NativeState::Unknown,
                                evidence,
                                ..
                            }) => ambiguous(evidence.unwrap_or_else(|| {
                                "backend reports unknown native state".to_string()
                            })),
                            Ok(observation) => {
                                let evidence = format!(
                                    "native={}, signal={}",
                                    observation.native.as_str(),
                                    observation.signal.as_str()
                                );
                                resumed_from = Some(observation);
                                (ReconcileDisposition::Resumed, evidence)
                            }
                        }
                    }
                },
            };
        self.record_reconcile(
            core,
            work_id,
            Some(&execution),
            disposition,
            reattached,
            &evidence,
        )?;
        let mut deferred = Deferred::new();
        match disposition {
            ReconcileDisposition::Resumed => {
                // `Resumed` is only ever produced with the Observation it
                // was classified from, and `drive` acts on that one rather
                // than asking a second time — the two answers are not
                // guaranteed to agree, and this one is the evidence the
                // disposition was journaled against.
                let observed = resumed_from.expect("a resumed disposition carries its observation");
                let next = self.drive(core, work_id, Ok(observed), &mut deferred)?;
                // Recovery is the single-owner path: nothing is served yet,
                // so performing the launch here holds up no request.
                self.run_inline(
                    core,
                    Step {
                        next,
                        deferred: std::mem::take(&mut deferred),
                    },
                )?;
            }
            ReconcileDisposition::Ambiguous => {
                if let Some(stage) = run.current_stage() {
                    self.commit(
                        core,
                        work_id,
                        KIND_STAGE_BLOCKED,
                        json!({"stage_id": stage.stage_id, "detail": evidence.clone()}),
                    )?;
                }
                // Retire the unreconcilable execution explicitly: a stale
                // identity must never be reachable for SEND again (§25;
                // Sergeant's stale-pane-identity class). Ordered before the
                // work-level block, so a crash anywhere in this append
                // sequence re-derives on the next restart (L6: the work is
                // still `active` until the final append, and reconcile
                // re-runs on `active`).
                //
                // A retirement the backend *acknowledged* latches
                // (`stop_requested`) and is not repeated. One it refused, or
                // that reached no registered backend, deliberately does not
                // latch: the native context was never asked, so a later stop
                // — a human's cancel, or the next restart's reconcile — must
                // still be able to ask. Re-running it appends a second
                // `execution.stopped` and asks again, which is convergent
                // (asking a backend that refuses has no effect) and honest
                // (every attempt is journaled as the attempt it was).
                self.stop_execution(
                    core,
                    work_id,
                    "retired at reconcile: unrecognized",
                    &mut deferred,
                )?;
                self.block(
                    core,
                    work_id,
                    "execution could not be reconciled after restart",
                    Some(evidence),
                )?;
            }
        }
        deferred.wait();
        Ok(disposition)
    }

    /// Fail a work closed over a reservation whose launch never reported back
    /// (§14.2's crash window, §14.3's Claude start-window).
    ///
    /// This is the window the reservation exists to make inspectable, and the
    /// honest answer to it is short: *sergeant committed to this execution
    /// identity, and cannot tell whether the external effect happened.* Both
    /// branches are live — the daemon may have died before the spawn, or
    /// after it — and no evidence available here separates them. So it fails
    /// closed with the reserved identity in the evidence, which is the thing
    /// an operator needs to go and look for.
    ///
    /// **It does ask the adapter what the reserved identity looks like now.**
    /// The disposition does not depend on the answer — it is ambiguous either
    /// way, because "a live turn under this session id" and "sergeant owns
    /// this execution" are different claims — but the answer is *evidence*,
    /// and for the only real adapter it is available: the reservation
    /// journaled the session UUID, and `ClaudeBackend::observe` answers
    /// exactly this question for a handle it has no memory of, by finding the
    /// session id in a live turn's argv. Declining to look meant a live orphan
    /// writing into the work's worktree was neither reported nor visible to
    /// the human who then hits `retry` and puts a second harness beside it.
    /// An adapter that cannot answer says so, and "no evidence" is recorded as
    /// no evidence — never as proof of absence.
    ///
    /// Three things this deliberately does **not** do:
    ///
    /// - it does not RESUME. Observing is a question; re-adopting is a claim
    ///   of ownership over a context this daemon never launched.
    /// - it does not stop or delete anything. §22.5's rule for every crash
    ///   window is that recovery must not delete unproven state, and a
    ///   possibly-nonexistent native context is the purest case of it. A turn
    ///   the adapter reports as *live* is named in the block reason instead,
    ///   which is what an operator can act on.
    /// - it does not retry. Re-launching would be the "start the external
    ///   effect twice" failure the same rule forbids.
    ///
    /// The reservation is journaled as abandoned so the record is closed —
    /// the identity survives *in that event*, and a later retry starts from a
    /// clean bookkeeping state rather than tripping this branch forever.
    fn reconcile_unsettled_reservation(
        &self,
        core: &mut Core,
        work_id: &str,
        run: &WorkRun,
        reservation: &ExecutionReservation,
    ) -> Result<ReconcileDisposition, EngineError> {
        let identity = match &reservation.native_id {
            Some(native_id) => format!("reserved native identity {native_id}"),
            None => "no native identity had been reserved yet".to_string(),
        };
        let native = self.reserved_identity_liveness(reservation);
        let liveness = match native {
            Some(NativeState::Running) => format!(
                "the adapter reports a LIVE native context under {identity} — it may still be \
                 writing into this work's surface; stop it before retrying"
            ),
            Some(NativeState::Exited) => {
                "the adapter reports the reserved native context has exited".to_string()
            }
            Some(NativeState::Unknown) | None => format!(
                "the adapter could not say whether anything exists under {identity}, which is \
                 not evidence that nothing does"
            ),
        };
        let evidence = format!(
            "execution {} was reserved on backend {:?} for stage {} attempt {}, and the journal \
             never recorded whether its launch happened ({identity}); {liveness}; sergeant will \
             not guess, start it a second time, or remove anything it cannot prove it owns",
            reservation.execution_id,
            reservation.backend,
            reservation.stage_id,
            reservation.attempt,
        );
        self.record_reconcile(
            core,
            work_id,
            None,
            ReconcileDisposition::Ambiguous,
            false,
            &evidence,
        )?;
        self.commit(
            core,
            work_id,
            KIND_EXECUTION_ABANDONED,
            json!({
                "execution_id": reservation.execution_id,
                "backend": reservation.backend,
                "native_id": reservation.native_id,
                "stage_id": reservation.stage_id,
                "attempt": reservation.attempt,
                "reason": "unsettled_at_restart",
                "detail": evidence,
                "launched": Value::Null,
                // Evidence, not state (§15's own distinction): what the
                // adapter could see of the reserved identity at this restart.
                // `null` is "could not look", which is not "nothing there".
                "native": native.map(NativeState::as_str),
            }),
        )?;
        if let Some(stage) = run.current_stage() {
            self.commit(
                core,
                work_id,
                KIND_STAGE_BLOCKED,
                json!({"stage_id": stage.stage_id, "detail": evidence.clone()}),
            )?;
        }
        let reason = if native == Some(NativeState::Running) {
            "an execution was reserved, its launch was never recorded, and a native context \
             under the reserved identity is still running"
        } else {
            "an execution was reserved but its launch was never recorded"
        };
        self.block(core, work_id, reason, Some(evidence))?;
        Ok(ReconcileDisposition::Ambiguous)
    }

    /// What the adapter can see of a reserved-but-unsettled native identity.
    ///
    /// `None` means the question could not be asked or the adapter refused —
    /// an unregistered backend, a reservation with no native id yet, an
    /// `UnknownExecution` from an adapter with no way to look. All of those
    /// are "no evidence", which the caller must not read as "nothing exists".
    fn reserved_identity_liveness(
        &self,
        reservation: &ExecutionReservation,
    ) -> Option<NativeState> {
        let native_id = reservation.native_id.clone()?;
        let backend = self.backends.get(&reservation.backend)?;
        // INV-R1-08: liveness-only, so a backend whose full OBSERVE pays for
        // expensive evidence capture as a side effect (Docker's log
        // capture) is never charged for evidence this call discards anyway
        // — only `.native` is read below.
        backend
            .observe_liveness(&ExecutionHandle {
                execution_id: reservation.execution_id.clone(),
                native_id: Some(native_id),
            })
            .ok()
    }

    /// §25's reattach step: ask the backend to re-adopt the recorded native
    /// context before anything is classified from it.
    ///
    /// `Ok(true)` — reattached, later SENDs continue the same context.
    /// `Ok(false)` — no reattachment was attempted, and the run is classified
    /// from OBSERVE alone exactly as it was before this step existed.
    /// `Err(detail)` — the adapter refused: the native context could not be
    /// evidenced, which is ambiguity and fails the work closed.
    ///
    /// Two cases decline to attempt it, and both are the absence of a claim
    /// rather than a softened one. A backend that does not advertise `resume`
    /// has no such verb to call (§15: unsupported means unsupported). And a
    /// run whose journal records no work surface has nowhere to reattach
    /// *into*: the [`ResumeRequest`] carries the directory later turns run
    /// in, and inventing one is precisely the fabrication `ResumeRequest`'s
    /// own contract forbids — a surface-less run is a journal prefix, and
    /// OBSERVE's classification (which fails closed) is the honest handling.
    /// Every run this engine actually started has a surface: `Engine::start`
    /// journals `surface.materialized` before the first stage is entered.
    fn reattach(
        &self,
        backend: &Arc<dyn Backend>,
        run: &WorkRun,
        work_id: &str,
        execution: &ExecutionRecord,
    ) -> Result<bool, String> {
        if !backend.capabilities().resume {
            return Ok(false);
        }
        let Some(request) = self.resume_request(run, work_id) else {
            return Ok(false);
        };
        backend
            .resume(&handle_of(execution), &request)
            .map(|()| true)
            .map_err(|e| format!("could not reattach to the native context: {e}"))
    }

    /// The §15 RESUME request for a run, rebuilt from what sergeant journaled.
    ///
    /// `None` for a run with no recorded surface: [`ResumeRequest`] carries the
    /// directory later turns run in, and inventing one is the fabrication its
    /// own contract forbids.
    ///
    /// Everything else here is re-supplied, never defaulted: the pin and the
    /// profile are the human's decisions about cost and about permissions —
    /// and, since N3, the *stage's* decisions (§12.5). Re-adopting a stage 10
    /// execution under stage 00's profile would be the same fabrication, one
    /// field further in.
    fn resume_request(&self, run: &WorkRun, work_id: &str) -> Option<ResumeRequest> {
        let surface = run.surface.as_ref()?;
        let stage_profile = run.current_stage_profile();
        Some(ResumeRequest {
            work_id: work_id.to_string(),
            cwd: surface.execution_cwd(),
            model: stage_profile.as_ref().and_then(|p| p.default_model.clone()),
            profile: stage_profile,
            instruction_policy: Some(Self::run_instruction_policy(run)),
            // §10.1, re-supplied from the journaled surface for the same
            // reason the pin and the profile are: a restarted adapter has
            // lost whatever it derived from it, and the Work's mutation
            // surface is not something it may re-invent from a bare cwd.
            bindings: surface.binding_summary(),
        })
    }

    /// Fail a work whose *start* crashed part-way closed (§25).
    ///
    /// Submitting is several fsynced appends: `work.submitted` from the API,
    /// then the engine's `surface.materializing`, `surface.materialized`,
    /// `workflow.bound` and `work.started`. A daemon that dies inside that
    /// sequence leaves a Work in `pending` that nothing will ever pick up —
    /// `retry` refuses `pending`, restart reconciliation only looks at
    /// `active`, and a client retrying the same `command_id` replays the
    /// recorded outcome without re-planning (its `origin.cwd` is not
    /// persisted, so no later actor could re-plan it anyway).
    ///
    /// Worse, the surviving prefix may name a worktree and a
    /// `sergeant/<work-id>` branch that were created in the user's repository.
    /// So this fails the work closed to `blocked` and puts that inventory in
    /// the evidence, where an operator can act on it. It never removes any of
    /// it: sergeant does not destroy git state it cannot prove it owns.
    pub fn reconcile_crashed_start(
        &self,
        core: &mut Core,
        work_id: &str,
    ) -> Result<(), EngineError> {
        let run = self.run(core, work_id)?;
        let mut evidence = vec![format!(
            "daemon restarted while starting this work: it never reached {KIND_WORK_STARTED}"
        )];
        if let Some(plan) = &run.surface_plan {
            evidence.push(format!(
                "branch {} may exist in: {}",
                plan.work_branch,
                plan.repositories
                    .iter()
                    .map(|r| r.path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            evidence.push(format!("worktrees may exist under {}", plan.root.display()));
        }
        evidence.push(format!(
            "surface recorded: {}; workflow bound: {}",
            run.surface.is_some(),
            run.workflow.is_some()
        ));
        self.block(
            core,
            work_id,
            "work surface was left half-materialized by a daemon restart",
            Some(evidence.join("; ")),
        )
    }

    /// R-MVP1-7's turn envelope, checked at both LAUNCH ([`Self::reserve_stage`])
    /// and SEND ([`Self::begin_input`]) — the engine's two turn-spawning
    /// seams — before either performs its effect.
    ///
    /// Reads [`crate::runtime::projection::WorkRun::turns_spawned`], the one
    /// counter both `execution.started` and `stage.resumed` increment, so
    /// this asks "how many turns has the journal actually recorded", never
    /// "how many times has this function been called" — the send-retry
    /// double-count trap named in the contract (`PendingSend::perform` may
    /// call `backend.send` twice for one delivery; only the delivered one
    /// ever reaches `stage.resumed`).
    ///
    /// Returns `Ok(true)` when the caller may proceed to spend a turn.
    /// `Ok(false)` means the (N+1)th turn was refused: the reason is already
    /// journaled and the Work is already `blocked` — the caller's only job
    /// left is to park rather than perform the effect.
    fn check_turn_envelope(
        &self,
        core: &mut Core,
        work_id: &str,
        stage_id: &str,
    ) -> Result<bool, EngineError> {
        let run = self.run(core, work_id)?;
        // R-MVP1-10's exit door: an explicit `extend_turn_envelope` raises
        // the effective cap for this Work specifically, cumulative across
        // every extension. Zero for a Work that was never extended, so this
        // is exactly `self.turn_cap` (or MVP-3's submit-time override, when
        // the client set one) for the ordinary case.
        let work = core.registry.state().works.get(work_id);
        let effective_cap = self.effective_turn_cap(work, &run);
        if run.turns_spawned < effective_cap {
            return Ok(true);
        }
        let reason = format!("turn envelope exhausted ({effective_cap} turns)");
        self.commit(
            core,
            work_id,
            KIND_STAGE_BLOCKED,
            json!({"stage_id": stage_id, "detail": reason}),
        )?;
        self.block(core, work_id, &reason, None)?;
        Ok(false)
    }

    /// Enter a stage and reserve its execution — §14.2's first phase, whole.
    ///
    /// Everything here is authoritative and cheap: validate the stage exists,
    /// resolve the executor, allocate the execution id, ask the adapter to
    /// PREPARE (identity only — no process, no I/O), and append
    /// `execution.reserved`. What comes back is the launch, for the caller to
    /// perform outside the lock.
    ///
    /// **L6 audit.** This makes `stage.entered` → `execution.reserved` →
    /// `execution.started` a three-append sequence with two crash windows,
    /// where before there were two appends and one. That is a deliberate
    /// trade: the new window is the one that was previously *invisible*. A
    /// crash between `stage.entered` and the reservation leaves a stage with
    /// no execution, which recovery already fails closed on. A crash between
    /// the reservation and `execution.started` leaves an unsettled
    /// reservation — and that is the window in which a native context may
    /// exist that sergeant never recorded. Before, that same window existed
    /// between the adapter's spawn and `execution.started`, with *nothing*
    /// durable naming the session; now the journal names it, and
    /// [`crate::runtime::recovery`] can fail the work closed with the
    /// identity in the evidence instead of with a shrug.
    fn reserve_stage(
        &self,
        core: &mut Core,
        work_id: &str,
        index: usize,
        attempt: u32,
    ) -> Result<Next, EngineError> {
        let run = self.run(core, work_id)?;
        let workflow = run.workflow.clone().ok_or_else(|| EngineError::NoRun {
            work_id: work_id.to_string(),
        })?;
        let stage = workflow
            .stage(index)
            .cloned()
            .ok_or_else(|| EngineError::NoSuchStage {
                work_id: work_id.to_string(),
                index,
            })?;
        let surface = run.surface.clone().ok_or_else(|| EngineError::NoRun {
            work_id: work_id.to_string(),
        })?;
        // §12.5: the executor is the stage's, not the run's. The decision was
        // made and journaled at bind time (`StageBinding`), so entering a
        // stage never re-routes — a retry and a restart replay the same
        // harness, profile and model the run was admitted with.
        //
        // A run bound before N3 has no bindings; its stages are the Work
        // actor default, which is exactly what `run.backend`/`run.profile`
        // say. Resolving the fallback here rather than at replay keeps the
        // projection a pure fold of what the journal actually recorded.
        let binding = run.stage_binding(&stage.id, index).cloned();
        let (backend_name, stage_profile) = match &binding {
            Some(binding) => (binding.harness.clone(), binding.profile.clone()),
            None => (
                run.backend.clone().ok_or_else(|| EngineError::NoRun {
                    work_id: work_id.to_string(),
                })?,
                run.profile.clone(),
            ),
        };
        let intent = core
            .registry
            .state()
            .works
            .get(work_id)
            .map(|w| w.intent.clone())
            .unwrap_or_default();

        self.commit(
            core,
            work_id,
            KIND_STAGE_ENTERED,
            json!({"stage_id": stage.id, "index": index, "attempt": attempt}),
        )?;

        // R-MVP1-7: the turn envelope gates LAUNCH here, before the harness
        // is even looked up — reaching this stage attempt at all does not
        // matter if there is no turn left to spend on it. Checked from the
        // journal-derived counter (`WorkRun::turns_spawned`), never from how
        // many times this function has been called.
        if !self.check_turn_envelope(core, work_id, &stage.id)? {
            return Ok(Next::Parked);
        }

        // A harness the journal pinned but this daemon does not have is
        // ambiguity, not a licence to fall back to the Work default: falling
        // back is the silent substitution §12.5 forbids, and it would run the
        // stage on a harness its author explicitly did not choose. The
        // submission preflight (§17.5, `bind_stages`) already refused this
        // case before the Work existed; reaching it here means the registry
        // changed under a bound run — a daemon restarted without an adapter —
        // and the honest answer is still to stop, not to pick a substitute.
        let backend = match self.backends.get(&backend_name).cloned() {
            Some(backend) => backend,
            None => {
                let detail = format!(
                    "stage {:?} is pinned to harness {backend_name:?}, which is not registered \
                     in this daemon (available: {}); sergeant will not substitute another \
                     harness for a stage that named one",
                    stage.id,
                    self.backends.names().join(", "),
                );
                self.commit(
                    core,
                    work_id,
                    KIND_STAGE_BLOCKED,
                    json!({"stage_id": stage.id, "detail": detail}),
                )?;
                self.block(core, work_id, "stage harness is unavailable", Some(detail))?;
                return Ok(Next::Parked);
            }
        };

        let execution_id = ulid::Ulid::generate().to_string();
        let request = StartRequest {
            work_id: work_id.to_string(),
            execution_id: execution_id.clone(),
            stage_id: stage.id.clone(),
            attempt,
            cwd: surface.execution_cwd(),
            intent,
            // §12: procedure is data. The stage's CONTEXT.md is carried to
            // the actor verbatim; sergeant never interprets it.
            context: stage.context.clone(),
            // §24.8: the profile carries the model, so the stage's profile
            // carries the stage's model. There is no per-stage model field.
            model: stage_profile.as_ref().and_then(|p| p.default_model.clone()),
            profile: stage_profile.clone(),
            // N4: the pinned container spec rides along exactly when this
            // stage is `kind = "execute"` — `bind_stages` already refused
            // the submission if that pin could not resolve to an available
            // Docker backend, so reaching here with `Some` is always
            // actionable by the backend named above.
            execute: stage.execute.clone(),
            // MVP-2 D2 item 1: the policy `workflow.bound` pinned, not
            // re-derived from the live manifest.
            instruction_policy: Self::run_instruction_policy(&run),
            // §10.1: the complete binding summary, not only a cwd. Taken from
            // the surface this stage is actually about to run in, so the paths
            // and refs an adapter states are the ones sergeant journaled —
            // never re-derived from the manifest or from the filesystem.
            bindings: surface.binding_summary(),
        };
        let prepared = match backend.prepare(&request) {
            Ok(prepared) => prepared,
            Err(e) => {
                // Refused before anything was reserved: no identity was
                // allocated, so there is nothing for recovery to wonder about.
                self.commit(
                    core,
                    work_id,
                    KIND_STAGE_BLOCKED,
                    json!({"stage_id": stage.id, "detail": e.to_string()}),
                )?;
                self.block(
                    core,
                    work_id,
                    "backend could not start an execution",
                    Some(e.to_string()),
                )?;
                return Ok(Next::Parked);
            }
        };
        let reservation = ExecutionReservation {
            execution_id,
            backend: backend_name,
            native_id: prepared.native_id.clone(),
            stage_id: stage.id.clone(),
            index,
            attempt,
            stage_kind: stage.kind.as_str().to_string(),
            profile: stage_profile.as_ref().map(|p| p.name.clone()),
            model: request.model.clone(),
        };
        self.commit(
            core,
            work_id,
            KIND_EXECUTION_RESERVED,
            json!({"reservation": reservation}),
        )?;
        Ok(Next::Launch(Box::new(PendingLaunch {
            work_id: work_id.to_string(),
            reservation,
            backend,
            prepared,
        })))
    }

    /// §14.2's third phase: verify the reservation is still current, record
    /// what the launch produced, and drive on.
    ///
    /// The verification is the whole reason the phase exists. Between the
    /// reservation and this call the core lock was *open*, so another request
    /// may have canceled the work, retried the stage, or otherwise moved the
    /// durable state on. §14.5's rule is that the late result is subordinate:
    /// it never revives terminal Work, never advances a superseded attempt,
    /// and never becomes the run's execution behind the newer decision's
    /// back. It is recorded as what it is — an execution that was reserved,
    /// possibly launched, and is now retired — and the native context, if one
    /// exists, is asked to stop.
    pub fn settle_launch(
        &self,
        core: &mut Core,
        pending: Box<PendingLaunch>,
        outcome: impl Into<LaunchOutcome>,
    ) -> Result<Step, EngineError> {
        let LaunchOutcome { handle, observed } = outcome.into();
        let outcome = handle;
        let mut deferred = Deferred::new();
        let work_id = pending.work_id.clone();
        let reservation = &pending.reservation;
        if let Some(why) = self.reservation_is_stale(core, &pending) {
            let stop = match &outcome {
                Ok(handle) => match pending.backend.stop(handle) {
                    Ok(completion) => {
                        deferred.push(completion);
                        json!({"requested": true})
                    }
                    Err(e) => json!({"requested": true, "error": e.to_string()}),
                },
                Err(e) => json!({
                    "requested": false,
                    "error": format!("nothing was launched: {e}"),
                }),
            };
            self.commit(
                core,
                &work_id,
                KIND_EXECUTION_ABANDONED,
                json!({
                    "execution_id": reservation.execution_id,
                    "backend": reservation.backend,
                    "native_id": reservation.native_id,
                    "stage_id": reservation.stage_id,
                    "attempt": reservation.attempt,
                    "reason": "superseded",
                    "detail": why,
                    "launched": outcome.is_ok(),
                    "stop": stop,
                }),
            )?;
            return Ok(Step {
                next: Next::Parked,
                deferred,
            });
        }
        let handle = match outcome {
            Ok(handle) => handle,
            Err(e) => {
                // The reservation named an identity nothing ever created.
                // Saying so explicitly is what keeps the window closed:
                // without this event the journal would show a reservation
                // that never settled, and the next restart would fail a work
                // closed over a native context that provably does not exist.
                self.commit(
                    core,
                    &work_id,
                    KIND_EXECUTION_ABANDONED,
                    json!({
                        "execution_id": reservation.execution_id,
                        "backend": reservation.backend,
                        "native_id": reservation.native_id,
                        "stage_id": reservation.stage_id,
                        "attempt": reservation.attempt,
                        "reason": "launch_failed",
                        "detail": e.to_string(),
                        "launched": false,
                    }),
                )?;
                self.commit(
                    core,
                    &work_id,
                    KIND_STAGE_BLOCKED,
                    json!({"stage_id": reservation.stage_id, "detail": e.to_string()}),
                )?;
                self.block(
                    core,
                    &work_id,
                    "backend could not start an execution",
                    Some(e.to_string()),
                )?;
                return Ok(Step {
                    next: Next::Parked,
                    deferred,
                });
            }
        };
        let record = ExecutionRecord {
            execution_id: reservation.execution_id.clone(),
            backend: reservation.backend.clone(),
            native_id: handle.native_id.clone(),
            stage_id: reservation.stage_id.clone(),
            attempt: reservation.attempt,
            stop_requested: false,
        };
        self.commit(
            core,
            &work_id,
            KIND_EXECUTION_STARTED,
            json!({"execution": record}),
        )?;
        // R-MVP1-7's ceiling half: this turn is now in flight. Recorded
        // in-memory only (see `Engine::turn_started`'s doc comment) — a
        // restart forgets it, which is fine for a soak-test hang bound.
        self.record_turn_start(&work_id);
        // An outcome with no observation attached is a launch reported without
        // one — the `From<Result<ExecutionHandle, _>>` shape, which exists for
        // callers driving the two phases apart by hand. There is nothing to
        // act on and nothing to be gained by asking here: an OBSERVE taken at
        // this point would be an external call with the guard held, which is
        // the whole thing the phase exists to prevent. The run is recorded as
        // started and left where it is.
        let next = match observed {
            Some(observed) => self.drive(core, &work_id, observed, &mut deferred)?,
            None => Next::Parked,
        };
        Ok(Step { next, deferred })
    }

    /// §14.2's third phase for SEND: verify the delivery still belongs to the
    /// run's current execution, then act on what the turn now says.
    ///
    /// The staleness rule is §14.5's, unchanged in substance: between the
    /// `stage.input_received` append and this call the guard was open, so a
    /// cancel may have retired the run underneath the delivery. A late answer
    /// does not revive it. What is recorded then is the delivery as the late
    /// evidence it is — including whether the harness actually took it — so
    /// the trajectory shows a human's words reaching a context whose work had
    /// already moved on, rather than showing nothing at all.
    ///
    /// **INV-1 (issue #46's second seam).** A delivery whose turn is still
    /// running when this settles (the common case — `send` returns at spawn,
    /// long before the turn ends) used to leave the stage `NeedsInput`
    /// forever: the completion driver's `due_observations` only ever polls a
    /// stage whose status is `Active`, and nothing else was going to observe
    /// a turn nobody asked for. Once the delivery is confirmed live (not
    /// stale, not an error), `KIND_STAGE_RESUMED` is committed unconditionally
    /// — before `drive` acts on whatever the delivery's own observation
    /// already says — so the driver picks the run back up on its next tick
    /// exactly as it does after a launch. This is done *here*, under the
    /// lock, after the out-of-lock effect, specifically so it opens no window
    /// for the *driver* to observe a delivery still in flight (see
    /// [`Engine::begin_input`]'s doc comment on why the flip does not happen
    /// there instead).
    ///
    /// **Residual (round-2 finding INV-R2-08).** `KIND_STAGE_INPUT_RECEIVED`
    /// — `begin_input`'s own append, ahead of this settle — has no reducer
    /// arm in [`crate::runtime::projection`], so a crash between that append
    /// and this one leaves an `active` work whose stage still reads
    /// `NeedsInput`: a shape `due_observations` skips forever on its own.
    /// This guarantee does not close that window itself — it never runs at
    /// all if the crash landed first. What closes it is entirely
    /// `runtime::recovery`'s: restart reconciliation reads the work as
    /// `active` with no settled delivery to explain the stage's status and
    /// fails it closed via `reconcile_work`/`classify_restart`, before this
    /// settle ever gets a chance to run stale. The two are complementary,
    /// not redundant — INV-1 covers every send that completes normally
    /// (the common case), recovery covers the one that does not survive to
    /// completion — and are that way on purpose, but it is worth being
    /// explicit that this doc comment's promise depends on it.
    pub fn settle_send(
        &self,
        core: &mut Core,
        pending: Box<PendingSend>,
        outcome: SendOutcome,
    ) -> Result<Step, EngineError> {
        let SendOutcome {
            delivered,
            reattached,
            observed,
        } = outcome;
        let work_id = pending.work_id.clone();
        if let Some(why) = self.delivery_is_stale(core, &pending) {
            self.commit(
                core,
                &work_id,
                KIND_EXECUTION_ABANDONED,
                json!({
                    "execution_id": pending.execution.execution_id,
                    "backend": pending.execution.backend,
                    "native_id": pending.execution.native_id,
                    "stage_id": pending.stage_id,
                    "attempt": pending.execution.attempt,
                    "verb": "send",
                    "reason": "superseded",
                    "detail": why,
                    "delivered": delivered.is_ok(),
                    "reattached": reattached,
                }),
            )?;
            return Ok(Step::parked());
        }
        if let Err(e) = delivered {
            self.commit(
                core,
                &work_id,
                KIND_STAGE_BLOCKED,
                json!({"stage_id": pending.stage_id, "detail": e.to_string()}),
            )?;
            self.block(core, &work_id, &format!("cannot deliver input: {e}"), None)?;
            return Ok(Step::parked());
        }
        // The delivery genuinely reached the backend and is not stale: the
        // stage is live again. Committed unconditionally, before `drive`,
        // regardless of what the accompanying observation turns out to say —
        // see this function's doc comment (INV-1).
        self.commit(
            core,
            &work_id,
            KIND_STAGE_RESUMED,
            json!({
                "stage_id": pending.stage_id,
                "index": pending.index,
                "attempt": pending.attempt,
            }),
        )?;
        // R-MVP1-7's ceiling half: SEND just spawned a new turn — restart
        // its clock exactly as LAUNCH does in `settle_launch`.
        self.record_turn_start(&work_id);
        if reattached {
            // §15 RESUME happened on this path, so the trajectory says so —
            // the same fact `execution.reconciled` records at restart, in the
            // one other place this daemon can come to own a context again.
            self.record_reconcile(
                core,
                &work_id,
                Some(&pending.execution),
                ReconcileDisposition::Resumed,
                true,
                "re-adopted the native context to deliver a human's answer",
            )?;
        }
        let mut deferred = Deferred::new();
        // `PendingSend::perform` observes whenever the delivery succeeded, and
        // a failed delivery returned above — so the `None` arm is a delivery
        // that reported success without an observation, which nothing here
        // constructs. It parks rather than observing under the guard, for the
        // same reason `settle_launch`'s does.
        let next = match observed {
            Some(observed) => self.drive(core, &work_id, observed, &mut deferred)?,
            None => Next::Parked,
        };
        Ok(Step { next, deferred })
    }

    /// Every live execution an observation is due on, as effects to perform
    /// **outside the core lock**.
    ///
    /// This is the daemon completion driver's question, asked under the guard
    /// and answered from durable projection state only: which runs currently
    /// have a native context whose answer nobody is going to come and ask for?
    /// It appends nothing and reserves nothing — a poll that finds a turn
    /// still running must leave the journal exactly as it found it, or a
    /// cadence becomes a write amplifier.
    ///
    /// The filter is the *precondition half* of §14.5's staleness rule, and it
    /// is deliberately the same predicate [`Engine::observation_is_stale`]
    /// re-checks after the effect: a run is due an observation when its work
    /// is `active`, its current stage attempt is `active`, the execution the
    /// journal records belongs to that attempt, no stop has been requested on
    /// it, and no reservation of this run is still unsettled (a launch in
    /// flight has its own settle coming, and observing underneath it would
    /// race that settle for the same turn).
    pub fn due_observations(&self, core: &Core) -> Vec<Box<PendingObserve>> {
        let registry = core.registry.state();
        let mut due = Vec::new();
        for (work_id, work) in &registry.works {
            if work.state != WorkState::Active {
                continue;
            }
            let Some(run) = registry.runs.get(work_id) else {
                continue;
            };
            if run.unsettled_reservation().is_some() {
                continue;
            }
            let Some(execution) = run.execution.as_ref() else {
                continue;
            };
            if execution.stop_requested {
                continue;
            }
            let Some(stage) = run.current_stage() else {
                continue;
            };
            if stage.status != StageStatus::Active
                || stage.stage_id != execution.stage_id
                || stage.attempt != execution.attempt
            {
                continue;
            }
            let Ok(backend) = self.backend_for(work_id, &execution.backend) else {
                // A run routed to a backend this daemon no longer registers is
                // recovery's problem, not the driver's: it is exactly the
                // ambiguity §25 fails closed at restart, and inventing a
                // disposition from a poll would be the guess §25 forbids.
                continue;
            };
            due.push(Box::new(PendingObserve {
                work_id: work_id.clone(),
                execution: execution.clone(),
                stage_id: stage.stage_id.clone(),
                index: stage.index,
                attempt: stage.attempt,
                backend,
            }));
        }
        due
    }

    /// Record that a turn just spawned, for [`Self::due_interrupts`].
    fn record_turn_start(&self, work_id: &str) {
        self.turn_started
            .lock()
            .expect("turn_started lock")
            .insert(work_id.to_string(), Instant::now());
    }

    /// Whether `work_id`'s current turn is still the one [`Self::record_turn_start`]
    /// last timed — the same "is this run actually due" predicate
    /// [`Self::due_observations`] applies, minus the backend lookup (a
    /// missing backend is still overdue; it is `Self::interrupt_turn` that
    /// discovers and reports that, exactly as `due_observations` defers a
    /// missing-backend judgement to its own caller).
    fn turn_still_active(&self, core: &Core, work_id: &str) -> bool {
        let registry = core.registry.state();
        let Some(work) = registry.works.get(work_id) else {
            return false;
        };
        if work.state != WorkState::Active {
            return false;
        }
        let Some(run) = registry.runs.get(work_id) else {
            return false;
        };
        let Some(execution) = run.execution.as_ref() else {
            return false;
        };
        if execution.stop_requested {
            return false;
        }
        let Some(stage) = run.current_stage() else {
            return false;
        };
        stage.status == StageStatus::Active
            && stage.stage_id == execution.stage_id
            && stage.attempt == execution.attempt
    }

    /// R-MVP1-7's per-turn wall-clock ceiling: every turn that has run
    /// longer than [`Self::turn_ceiling`], as an effect to interrupt
    /// **outside the core lock** (§22.6) — the same shape
    /// [`Self::due_observations`] already has, and meant to be swept by the
    /// same completion-driver tick (`api.rs`'s `drive_completions`, riding
    /// the existing 200 ms cadence per the contract).
    ///
    /// One interrupt attempt per turn that crosses the ceiling: an entry is
    /// removed from the internal clock the moment it is collected here,
    /// whether because it is stale (no longer this turn — dropped, nothing
    /// to interrupt) or because it is overdue (interrupted once, then
    /// forgotten). What the interrupt actually produced is
    /// [`Self::due_observations`]'s question, not this one's (§25).
    pub fn due_interrupts(&self, core: &Core, now: Instant) -> Vec<Box<PendingInterrupt>> {
        let mut due = Vec::new();
        self.turn_started
            .lock()
            .expect("turn_started lock")
            .retain(|work_id, at| {
                if !self.turn_still_active(core, work_id) {
                    return false; // stale: settled, moved on, or superseded
                }
                // MVP-3: a Work's own submit-time override, when it set one,
                // stands in for the daemon-wide default here — the same
                // formula `check_turn_envelope`'s cap check uses.
                let ceiling = self.effective_turn_ceiling(core.registry.state().works.get(work_id));
                if now.saturating_duration_since(*at) < ceiling {
                    return true; // keep timing it, not yet due
                }
                // Overdue. A backend this daemon no longer registers is the
                // same ambiguity `due_observations` defers to restart
                // recovery rather than guessing at here — this call simply
                // has nothing to interrupt with, and drops the entry either
                // way (one attempt per crossing, per this method's doc).
                if let Ok(run) = self.run(core, work_id)
                    && let Some(execution) = run.execution.clone()
                    && let Some(backend) = self.backends.get(&execution.backend).cloned()
                {
                    due.push(Box::new(PendingInterrupt {
                        work_id: work_id.clone(),
                        execution,
                        backend,
                        ceiling_secs: ceiling.as_secs_f64(),
                    }));
                }
                false
            });
        due
    }

    /// §14.5 for INTERRUPT's *middle* phase: whether the crossing
    /// [`Self::due_interrupts`] collected still describes the turn that is
    /// live now, checked under the guard immediately before the perform.
    ///
    /// The other settles re-check staleness after their effect because the
    /// effect's evidence must be preserved either way; an interrupt is the
    /// one effect whose staleness must be caught *before* it runs, because
    /// `Backend::interrupt` aims at whatever turn holds the execution handle
    /// at delivery time — and the collection→perform window stretches across
    /// every earlier entry's serially-awaited crank. A turn that ended in
    /// that window (and any fresh turn a SEND spawned on the same handle —
    /// visible as a re-inserted `turn_started` entry, since collection
    /// removed this work's) must not be killed for a crossing it never made.
    ///
    /// `Some(reason)` means skip the perform and journal the consumed
    /// crossing through [`Self::settle_stale_interrupt`] instead — never
    /// drop it silently (`due_interrupts`'s own no-silent-loss rule).
    pub fn interrupt_is_stale(
        &self,
        core: &Core,
        pending: &PendingInterrupt,
    ) -> Option<&'static str> {
        if self
            .turn_started
            .lock()
            .expect("turn_started lock")
            .contains_key(&pending.work_id)
        {
            return Some("a fresh turn started on this work after the crossing was collected");
        }
        if !self.turn_still_active(core, &pending.work_id) {
            return Some("the targeted turn is no longer active");
        }
        let registry = core.registry.state();
        let same_execution = registry
            .runs
            .get(&pending.work_id)
            .and_then(|run| run.execution.as_ref())
            .is_some_and(|e| e.execution_id == pending.execution.execution_id);
        if same_execution {
            None
        } else {
            Some("the execution handle was superseded")
        }
    }

    /// Journal a consumed ceiling crossing whose interrupt was *not*
    /// performed because [`Self::interrupt_is_stale`] said the targeted turn
    /// is gone: `outcome.requested` is `false` and `outcome.stale` carries
    /// the reason, so the trajectory records the crossing without claiming a
    /// delivery that never happened.
    pub fn settle_stale_interrupt(
        &self,
        core: &mut Core,
        pending: &PendingInterrupt,
        reason: &str,
    ) -> Result<(), EngineError> {
        self.commit(
            core,
            &pending.work_id,
            KIND_TURN_CEILING_INTERRUPTED,
            json!({
                "execution_id": pending.execution.execution_id,
                "backend": pending.execution.backend,
                "stage_id": pending.execution.stage_id,
                "attempt": pending.execution.attempt,
                "ceiling_secs": pending.ceiling_secs,
                "outcome": {"requested": false, "stale": reason},
            }),
        )
    }

    /// §14.2's third phase for INTERRUPT: journal what was asked and what it
    /// produced. Unconditional — an interrupt request and its outcome are
    /// journaled regardless of whether the backend actually complied.
    ///
    /// **Issue #90.** This also lands the Work in `blocked`, and that part is
    /// *not* deferred to the ordinary OBSERVE path the way the doc here used
    /// to promise. The reason is §25 itself: a killed turn's own adapter
    /// reports the ambiguity honestly and invents nothing —
    /// `observe_in_memory`'s interrupted-with-no-envelope arm
    /// (`src/backend/claude.rs`) returns `BackendSignal::Running` forever,
    /// on purpose, because "the conversation is resumable" is all a single
    /// OBSERVE can support. `drive`'s `Running` arm parks without journaling
    /// anything, so waiting for OBSERVE to resolve this specific ambiguity
    /// means waiting for an answer the adapter has already declared it will
    /// never give — that was the whole of the wedge: `active` forever, no
    /// operator verb able to reach it, only `cancel` (destructive) reachable.
    ///
    /// So this settle takes the decision itself, from the one fact already
    /// in hand — the ceiling crossing it is about to journal — rather than
    /// waiting on native evidence that cannot arrive. That is
    /// [`Self::check_turn_envelope`]'s own shape (`KIND_STAGE_BLOCKED` then
    /// [`Self::block`]) reused for a different §25 trigger, and it opens the
    /// same door R-MVP1-10 already built and tests: `retry` re-enters the
    /// stage (turns_spawned is almost never at cap from a wall-clock
    /// ceiling alone, so `extend` is available but rarely required), or
    /// `cancel` still works and — per #90's own correction — never discards
    /// the retained worktree.
    ///
    /// Guarded on the Work still being `active`: a concurrent cancel or
    /// failure that retired it in the collection→perform→settle window must
    /// keep whatever verb got there first, not be overridden by a crossing
    /// that is now stale in every sense but the timer.
    ///
    /// **L6 audit.** This makes the settle a three-append sequence
    /// (`execution.turn_ceiling_interrupted`, `stage.blocked`,
    /// `work.blocked`) where before it was one. A crash between any two of
    /// them leaves the Work `active` with a partial trail — but that is
    /// exactly the shape [`crate::runtime::recovery::reconcile`] already
    /// finds and re-derives from scratch on the next restart (`in_flight`
    /// is keyed on `WorkState::Active`, and `reconcile_work` never trusts
    /// what a prior run partially wrote); the restart path's own
    /// `classify_restart` (`src/backend/claude.rs`) fails every ambiguous
    /// case closed to `Blocked` independently of what this method managed
    /// to commit, so no new unrecoverable window is opened here — only the
    /// same "stage event, then work event" idiom [`Self::check_turn_envelope`]
    /// and `drive`'s `Blocked`/`Failed`/`StageCompleted` arms already rely on.
    pub fn settle_interrupt(
        &self,
        core: &mut Core,
        pending: PendingInterrupt,
        outcome: InterruptOutcome,
    ) -> Result<Step, EngineError> {
        let mut deferred = Deferred::new();
        let (recorded, request_error) = match outcome.outcome {
            Ok(completion) => {
                deferred.push(completion);
                (json!({"requested": true}), None)
            }
            Err(e) => {
                let detail = e.to_string();
                (
                    json!({"requested": true, "error": detail.clone()}),
                    Some(detail),
                )
            }
        };
        self.commit(
            core,
            &pending.work_id,
            KIND_TURN_CEILING_INTERRUPTED,
            json!({
                "execution_id": pending.execution.execution_id,
                "backend": pending.execution.backend,
                "stage_id": pending.execution.stage_id,
                "attempt": pending.execution.attempt,
                "ceiling_secs": pending.ceiling_secs,
                "outcome": recorded,
            }),
        )?;
        if self.work_state(core, &pending.work_id)? == WorkState::Active {
            let reason = format!("turn ceiling exceeded ({}s)", pending.ceiling_secs);
            self.commit(
                core,
                &pending.work_id,
                KIND_STAGE_BLOCKED,
                json!({"stage_id": pending.execution.stage_id, "detail": reason}),
            )?;
            self.block(core, &pending.work_id, &reason, request_error)?;
        }
        Ok(Step {
            next: Next::Parked,
            deferred,
        })
    }

    /// §14.2's third phase for a bare OBSERVE: verify the observation still
    /// describes the run's current attempt, then act on it.
    ///
    /// **Why the check cannot be skipped even though nothing was reserved.**
    /// The lock was open across the observation, and an observation is the one
    /// input that can *complete a stage*. Between `due_observations` and here
    /// a cancel may have retired the work, a SEND-settle may have consumed the
    /// very same turn end, an interrupt may have superseded the execution, or
    /// a retry may have moved the attempt on. §14.5's rule applies unchanged:
    /// the late answer is subordinate to durable state. Because the check and
    /// [`Engine::drive`] run under one lock hold, two observations of one
    /// finished turn cannot both complete it — the first moves the run and the
    /// second finds itself stale.
    ///
    /// **Nothing is journaled for a stale one, and that is the point.** The
    /// other two settles record `execution.abandoned` because they performed
    /// an external effect whose evidence would otherwise vanish — a process
    /// was spawned, a human's words were delivered. An observation performs
    /// nothing and leaves nothing behind, so there is no evidence to preserve;
    /// journaling every superseded poll would put the driver's cadence into
    /// the trajectory as noise (R1: the event does not need to exist).
    ///
    /// **L6 audit.** This opens no new adjacent-append window. The begin phase
    /// appends nothing at all, so there is no reserve/settle pair to be caught
    /// between, and every append below is `drive`'s — the same sequence
    /// `settle_launch` has always produced, whose crash windows are §22.5's
    /// 5–8 and are pinned there. What the driver *does* newly expose is the
    /// window before it ever runs: a turn that ended into adapter memory and a
    /// daemon that dies before the poll. That outcome is not durable and must
    /// be re-derived at restart, which recovery already does and
    /// `a_crash_after_the_turn_ended_is_re_derived_at_restart` (m4) pins.
    pub fn settle_observe(
        &self,
        core: &mut Core,
        pending: Box<PendingObserve>,
        outcome: ObserveOutcome,
    ) -> Result<Step, EngineError> {
        let work_id = pending.work_id.clone();
        if let Some(why) = self.observation_is_stale(core, &pending) {
            tracing::debug!(
                work_id = %work_id,
                execution_id = %pending.execution.execution_id,
                reason = %why,
                "discarding an observation the run has moved past"
            );
            return Ok(Step::parked());
        }
        let mut deferred = Deferred::new();
        let next = self.drive(core, &work_id, outcome.observed, &mut deferred)?;
        Ok(Step { next, deferred })
    }

    /// Why an observation may no longer be applied, or `None` if it may.
    ///
    /// [`Engine::reservation_is_stale`]'s checklist for the verb that reserves
    /// nothing, plus the two conditions that only a *poll* can lose a race to:
    /// a stop already requested on the execution (a cancel or a completed
    /// stage got here first) and a reservation opened underneath it (a retry
    /// or the next stage is already launching).
    fn observation_is_stale(&self, core: &Core, pending: &PendingObserve) -> Option<String> {
        let registry = core.registry.state();
        let Some(work) = registry.works.get(&pending.work_id) else {
            return Some("the work no longer exists".to_string());
        };
        if work.state != WorkState::Active {
            return Some(format!(
                "work is {} — an observation taken beforehand cannot move it",
                work.state
            ));
        }
        let Some(run) = registry.runs.get(&pending.work_id) else {
            return Some("the run record is gone".to_string());
        };
        if run.unsettled_reservation().is_some() {
            return Some("a launch of this run is still unsettled".to_string());
        }
        match run.execution.as_ref() {
            Some(current) if current.execution_id == pending.execution.execution_id => {
                if current.stop_requested {
                    return Some(format!(
                        "execution {} was already asked to stop",
                        current.execution_id
                    ));
                }
            }
            Some(current) => {
                return Some(format!(
                    "execution {} was superseded by {}",
                    pending.execution.execution_id, current.execution_id
                ));
            }
            None => return Some("the run no longer records an execution".to_string()),
        }
        match run.current_stage() {
            Some(stage)
                if stage.stage_id == pending.stage_id
                    && stage.index == pending.index
                    && stage.attempt == pending.attempt
                    && stage.status == StageStatus::Active => {}
            Some(stage) => {
                return Some(format!(
                    "the run moved on to stage {} attempt {} ({})",
                    stage.stage_id,
                    stage.attempt,
                    stage.status.as_str()
                ));
            }
            None => return Some("the run has no current stage".to_string()),
        }
        None
    }

    /// Why a delivery's result may no longer be applied, or `None` if it may.
    ///
    /// [`Engine::reservation_is_stale`]'s rule, for the verb that has no
    /// reservation: the Work still exists, it is still `active` (the
    /// `work.resumed` this delivery was committed with put it there), the
    /// run's current execution is still the one the input was handed to, and
    /// (INV-4, BS2) the current stage is still the same coordinate this
    /// delivery was reserved against — matching [`Engine::observation_is_stale`]
    /// and [`Engine::reservation_is_stale`], which both check stage
    /// identity, not execution identity alone. Nothing reachable today can
    /// change the stage coordinate while leaving the execution id unchanged
    /// (a stage advance always reserves a new execution), so this clause is
    /// currently redundant with the execution check above it — kept as the
    /// explicit currency check §14.5 asks every settle to make, and as the
    /// guard against a future change that reuses an execution id across a
    /// stage boundary silently reopening the race this function's sibling
    /// [`Engine::settle_send`] was written to avoid.
    fn delivery_is_stale(&self, core: &Core, pending: &PendingSend) -> Option<String> {
        let registry = core.registry.state();
        let Some(work) = registry.works.get(&pending.work_id) else {
            return Some("the work no longer exists".to_string());
        };
        if work.state != WorkState::Active {
            return Some(format!(
                "work is {} — an answer that arrived afterwards cannot move it",
                work.state
            ));
        }
        let Some(run) = registry.runs.get(&pending.work_id) else {
            return Some("the run record is gone".to_string());
        };
        match run.execution.as_ref() {
            Some(current) if current.execution_id == pending.execution.execution_id => {}
            Some(current) => {
                return Some(format!(
                    "execution {} was superseded by {}",
                    pending.execution.execution_id, current.execution_id
                ));
            }
            None => return Some("the run no longer records an execution".to_string()),
        }
        match run.current_stage() {
            Some(stage)
                if stage.stage_id == pending.stage_id
                    && stage.index == pending.index
                    && stage.attempt == pending.attempt => {}
            Some(stage) => {
                return Some(format!(
                    "the run moved on to stage {} attempt {}",
                    stage.stage_id, stage.attempt
                ));
            }
            None => return Some("the run has no current stage".to_string()),
        }
        None
    }

    /// Why a launch's result may no longer be applied, or `None` if it may.
    ///
    /// §14.5's checklist, in order: the Work still exists; it has not gone
    /// terminal (or otherwise left `active`); the reservation this launch
    /// belongs to is still the run's outstanding one; and the stage attempt
    /// it named is still the current one.
    fn reservation_is_stale(&self, core: &Core, pending: &PendingLaunch) -> Option<String> {
        let reservation = &pending.reservation;
        let registry = core.registry.state();
        let Some(work) = registry.works.get(&pending.work_id) else {
            return Some("the work no longer exists".to_string());
        };
        if work.state != WorkState::Active {
            return Some(format!(
                "work is {} — a launch that finished afterwards cannot move it",
                work.state
            ));
        }
        let Some(run) = registry.runs.get(&pending.work_id) else {
            return Some("the run record is gone".to_string());
        };
        match run.reservation.as_ref() {
            Some(current) if current.execution_id == reservation.execution_id => {}
            Some(current) => {
                return Some(format!(
                    "reservation {} was superseded by {}",
                    reservation.execution_id, current.execution_id
                ));
            }
            None => {
                return Some(format!(
                    "reservation {} was already settled or abandoned",
                    reservation.execution_id
                ));
            }
        }
        match run.current_stage() {
            Some(stage)
                if stage.stage_id == reservation.stage_id
                    && stage.index == reservation.index
                    && stage.attempt == reservation.attempt => {}
            Some(stage) => {
                return Some(format!(
                    "the run moved on to stage {} attempt {}",
                    stage.stage_id, stage.attempt
                ));
            }
            None => return Some("the run has no current stage".to_string()),
        }
        None
    }

    /// Ask the current execution to retire, and journal that we asked.
    ///
    /// A stop *request* is the whole truth available to sergeant: whether the
    /// native context complied is only knowable through OBSERVE, and is never
    /// assumed here.
    ///
    /// The adapter's tail work — a transcript archive still being written —
    /// goes into `deferred` rather than being joined here (issue #14/B3):
    /// this runs under the daemon's core lock, and §22.6 forbids a thread
    /// join under it.
    fn stop_execution(
        &self,
        core: &mut Core,
        work_id: &str,
        reason: &str,
        deferred: &mut Deferred,
    ) -> Result<(), EngineError> {
        let Ok(run) = self.run(core, work_id) else {
            return Ok(());
        };
        let Some(execution) = run.execution.clone() else {
            return Ok(());
        };
        if execution.stop_requested {
            return Ok(());
        }
        let outcome = match self.backends.get(&execution.backend) {
            Some(backend) => match backend.stop(&handle_of(&execution)) {
                Ok(completion) => {
                    deferred.push(completion);
                    json!({"requested": true})
                }
                Err(e) => json!({"requested": true, "error": e.to_string()}),
            },
            None => json!({"requested": false, "error": "backend not registered"}),
        };
        self.commit(
            core,
            work_id,
            KIND_EXECUTION_STOPPED,
            json!({
                "execution_id": execution.execution_id,
                "backend": execution.backend,
                "reason": reason,
                "outcome": outcome,
            }),
        )?;
        Ok(())
    }

    /// The effect that will tear this run's surface down, or [`Next::Parked`]
    /// when there is nothing to tear down.
    ///
    /// Idempotent through the projection: a run that already has a teardown
    /// report yields no effect, so re-running teardown (the crash window, a
    /// second restart) touches no repository and appends nothing.
    ///
    /// The removal itself — `git worktree remove` per repository — is the
    /// caller's to perform outside the guard, which is why this returns an
    /// intent rather than a report.
    fn teardown_next(&self, core: &Core, work_id: &str, recovered: bool) -> Next {
        let Ok(run) = self.run(core, work_id) else {
            return Next::Parked;
        };
        let Some(surface) = run.surface.clone() else {
            return Next::Parked;
        };
        if run.teardown.is_some() {
            return Next::Parked; // already retired
        }
        Next::Surface(Box::new(PendingSurface {
            work_id: work_id.to_string(),
            data_dir: self.data_dir.clone(),
            effect: SurfaceEffect::Teardown { surface, recovered },
        }))
    }

    /// Finish a teardown a crash swallowed, on a work that is already
    /// terminal (§25 applied to the completion tail; issue #9).
    ///
    /// **Rung note (R2).** `work.completed` and its trailing
    /// `surface.torn_down` are two adjacent appends, and a kill in between
    /// leaves a work whose state is legal but whose audit trail is not 1:1 —
    /// and, when the crash landed *before* `teardown()` ran, a worktree and a
    /// surface root nothing will ever remove, because reconciliation only ever
    /// looked at work believed in flight. L6 offers two answers: one compound
    /// event, or a tolerant reader that re-derives. Compounding would put a
    /// filesystem operation's report inside the state transition that must
    /// land first — the completion would then wait on git, and a teardown
    /// failure would take the completion with it. So this is the re-derivation
    /// (R2: the existing teardown, re-run, rather than new machinery).
    ///
    /// It is evidence, not a guess: [`teardown`] *inspects* — a worktree that
    /// is gone is recorded `Missing`, one that is still there is removed only
    /// if git says it is clean, and a dirty or unremovable one is retained and
    /// named in the report exactly as it would have been at completion time.
    /// Nothing is assumed about which side of the window the crash fell on;
    /// the report says what the disk actually holds now. The event carries
    /// `recovered: true` so the trail shows *when* the teardown was recorded,
    /// rather than pretending it landed with the completion.
    ///
    /// Returns whether an event was appended.
    ///
    /// This one performs its own git inline, and that is correct here and
    /// nowhere else: it runs between journal replay and the first served
    /// request, where nothing is queueing behind the guard because nothing is
    /// being served (`runtime::recovery`'s single-owner window).
    pub fn reconcile_terminal_surface(
        &self,
        core: &mut Core,
        work_id: &str,
    ) -> Result<bool, EngineError> {
        let Next::Surface(pending) = self.teardown_next(core, work_id, true) else {
            return Ok(false);
        };
        let outcome = pending.perform();
        // A teardown settles to `Parked` with an empty `Deferred`; running it
        // inline here is the single-owner path, so there is no crank to hand
        // the Step to.
        let step = self.settle_surface(core, pending, outcome)?;
        self.run_inline(core, step)?;
        Ok(true)
    }

    fn record_reconcile(
        &self,
        core: &mut Core,
        work_id: &str,
        execution: Option<&ExecutionRecord>,
        disposition: ReconcileDisposition,
        reattached: bool,
        evidence: &str,
    ) -> Result<(), EngineError> {
        self.commit(
            core,
            work_id,
            KIND_EXECUTION_RECONCILED,
            json!({
                "execution_id": execution.map(|e| e.execution_id.clone()),
                "backend": execution.map(|e| e.backend.clone()),
                "disposition": disposition,
                // Whether §15 RESUME re-adopted the native context before it
                // was classified. Its own field rather than prose in the
                // evidence: "sergeant owns this context again" is the fact a
                // later operator (or M6's UI) needs to read back, and the
                // evidence string is the adapter's answer, not the engine's.
                "reattached": reattached,
                "evidence": evidence,
            }),
        )?;
        Ok(())
    }

    /// Move a work to `blocked` with a reason and, where there is one, the
    /// evidence that produced the decision.
    ///
    /// A work that is already `blocked` still gets this journaled: a caller
    /// can re-block an already-`blocked` work with a new reason and evidence
    /// (e.g. restart reconciliation, after an earlier block left it there),
    /// and `transition`'s ordinary same-state short-circuit would otherwise
    /// discard it.
    pub(crate) fn block(
        &self,
        core: &mut Core,
        work_id: &str,
        reason: &str,
        evidence: Option<String>,
    ) -> Result<(), EngineError> {
        let mut payload = json!({"reason": reason});
        if let Some(evidence) = evidence {
            payload["evidence"] = Value::String(evidence);
        }
        if self.work_state(core, work_id)? == WorkState::Blocked {
            return self.commit(core, work_id, KIND_WORK_BLOCKED, payload);
        }
        self.transition(core, work_id, KIND_WORK_BLOCKED, payload)
    }

    /// Append a work-state event, refusing an illegal transition *before* the
    /// append. The §10 table is consulted here and nowhere else in the engine.
    fn transition(
        &self,
        core: &mut Core,
        work_id: &str,
        kind: &str,
        payload: Value,
    ) -> Result<(), EngineError> {
        let from = self.work_state(core, work_id)?;
        let to = WorkState::for_event_kind(kind).ok_or_else(|| EngineError::IllegalTransition {
            work_id: work_id.to_string(),
            from,
            to: from,
        })?;
        if from == to {
            return Ok(()); // already there; no event, no churn
        }
        if !from.can_transition(to) {
            return Err(EngineError::IllegalTransition {
                work_id: work_id.to_string(),
                from,
                to,
            });
        }
        self.commit(core, work_id, kind, payload)
    }

    fn commit(
        &self,
        core: &mut Core,
        work_id: &str,
        kind: &str,
        payload: Value,
    ) -> Result<(), EngineError> {
        core.commit(
            EventDraft::new(EventSource::new("daemon", "engine"), kind, payload)
                .with_work_id(work_id),
        )?;
        Ok(())
    }

    fn work_state(&self, core: &Core, work_id: &str) -> Result<WorkState, EngineError> {
        // #4: `works` only holds active Works now. Most callers here only
        // ever ask about one (scheduling loops touch non-absorbing works by
        // construction), but this is a shared lookup, not scoped to those
        // callers — `state_view` answers from the slim index for exactly
        // this case, and it is never evicted, so it answers correctly for
        // an already-terminal (possibly evicted) `work_id` too, with no
        // journal replay.
        core.registry
            .state()
            .state_view(work_id)
            .ok_or_else(|| EngineError::NoRun {
                work_id: work_id.to_string(),
            })
    }

    fn run(&self, core: &Core, work_id: &str) -> Result<WorkRun, EngineError> {
        core.registry
            .state()
            .runs
            .get(work_id)
            .cloned()
            .ok_or_else(|| EngineError::NoRun {
                work_id: work_id.to_string(),
            })
    }

    fn backend_for(&self, work_id: &str, name: &str) -> Result<Arc<dyn Backend>, EngineError> {
        self.backends
            .get(name)
            .cloned()
            .ok_or_else(|| EngineError::BackendMissing {
                work_id: work_id.to_string(),
                backend: name.to_string(),
            })
    }
}

/// The backend-facing handle for a recorded execution.
fn handle_of(execution: &ExecutionRecord) -> ExecutionHandle {
    ExecutionHandle {
        execution_id: execution.execution_id.clone(),
        native_id: execution.native_id.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::fake::{FAKE_BACKEND_NAME, FakeBackend};
    use crate::runtime::testing;

    fn engine(data_dir: &Path) -> Engine {
        let backends = BackendRegistry::new().with(Arc::new(FakeBackend::new(FAKE_BACKEND_NAME)));
        Engine::new(
            Arc::new(backends),
            Some(FAKE_BACKEND_NAME.to_string()),
            data_dir,
        )
    }

    /// Blocking is how every fail-closed path records *why* it gave up, and a
    /// work can be blocked twice for different reasons — recovery blocking a
    /// work an earlier failure already left blocked, say. `transition`'s
    /// ordinary same-state short-circuit would drop the second reason on the
    /// floor, which is exactly the evidence an operator needs. So `block`
    /// journals it anyway, without pretending a transition happened.
    #[test]
    fn blocking_an_already_blocked_work_records_the_new_reason() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());
        let engine = engine(dir.path());
        let work_id = "01BLOCKTWICE";
        testing::submit(&mut core, work_id, "block me twice");

        engine
            .block(&mut core, work_id, "first reason", None)
            .expect("first block");
        engine
            .block(
                &mut core,
                work_id,
                "second reason",
                Some("what went wrong the second time".to_string()),
            )
            .expect("second block");

        let blocked: Vec<Value> = core
            .journal
            .replay()
            .expect("replay")
            .map(|e| e.expect("event"))
            .filter(|e| e.kind == KIND_WORK_BLOCKED)
            .map(|e| e.payload)
            .collect();
        assert_eq!(
            blocked.len(),
            2,
            "the second reason must not be silently discarded, got {blocked:?}"
        );
        assert_eq!(blocked[0]["reason"], "first reason");
        assert_eq!(blocked[1]["reason"], "second reason");
        assert_eq!(
            blocked[1]["evidence"], "what went wrong the second time",
            "the evidence travels with the reason"
        );
        assert_eq!(
            core.registry.state().works[work_id].state,
            WorkState::Blocked,
            "and the work is still exactly where it was"
        );
    }

    // ------------------------- estate-root Phase C: Engine::resolve_scope

    /// A estate fixture for exercising [`Engine::resolve_scope`] directly
    /// — resolution never touches the filesystem, so `/nowhere` placeholders
    /// are fine (mirrors `domain::estate::tests`' own fixture pattern).
    fn scope_fixture(repos: &[&str], groups: &[(&str, &[&str])]) -> Estate {
        Estate {
            name: "payments".to_string(),
            root: PathBuf::from("/nowhere"),
            repositories: repos
                .iter()
                .map(|name| RepositorySpec {
                    name: name.to_string(),
                    path: PathBuf::from(format!("/nowhere/{name}")),
                })
                .collect(),
            default_backend: None,
            default_workflow: None,
            profiles: Vec::new(),
            config_path: None,
            surfaces_dir: None,
            data_dir: None,
            repository_policy: BTreeMap::new(),
            groups: groups
                .iter()
                .map(|(name, members)| {
                    (
                        name.to_string(),
                        crate::domain::estate::GroupSpec {
                            repos: members.iter().map(|m| m.to_string()).collect(),
                            brief: None,
                        },
                    )
                })
                .collect(),
            repository_origin: BTreeMap::new(),
            repository_upstream: BTreeMap::new(),
            retention: None,
        }
    }

    /// Owner ruling (2026-08-20): `--all` combined with `--repo` and/or
    /// `--group` is refused, naming what was combined — `all` no longer
    /// silently wins over an explicit repos/group selection.
    #[test]
    fn resolve_scope_all_combined_with_repos_or_group_is_refused() {
        let estate = scope_fixture(&["api", "web"], &[("pair", &["api", "web"])]);
        let err = Engine::resolve_scope(
            &estate,
            &SubmitContext {
                repos: &["api".to_string()],
                group: Some("pair"),
                all: true,
                ..SubmitContext::default()
            },
        )
        .expect_err("--all combined with --repo/--group must be refused");
        match err {
            EngineError::ConflictingScope { repos, group } => {
                assert_eq!(repos, vec!["api".to_string()]);
                assert_eq!(group, Some("pair".to_string()));
            }
            other => panic!("expected ConflictingScope, got {other}"),
        }

        // The refusal fires on `--repo` alone, too, not only in combination
        // with `--group`.
        let err = Engine::resolve_scope(
            &estate,
            &SubmitContext {
                repos: &["api".to_string()],
                all: true,
                ..SubmitContext::default()
            },
        )
        .expect_err("--all combined with --repo alone must be refused");
        assert!(matches!(err, EngineError::ConflictingScope { .. }));

        // And on `--group` alone, with no explicit `--repo`.
        let err = Engine::resolve_scope(
            &estate,
            &SubmitContext {
                group: Some("pair"),
                all: true,
                ..SubmitContext::default()
            },
        )
        .expect_err("--all combined with --group alone must be refused");
        assert!(matches!(err, EngineError::ConflictingScope { .. }));
    }

    /// §7.1: `--all` alone still resolves to every declared repository.
    #[test]
    fn resolve_scope_all_alone_still_selects_every_declared_repository() {
        let estate = scope_fixture(&["api", "web"], &[("pair", &["api", "web"])]);
        let resolved = Engine::resolve_scope(
            &estate,
            &SubmitContext {
                all: true,
                ..SubmitContext::default()
            },
        )
        .expect("all alone must resolve");
        let mut names: Vec<&str> = resolved.iter().map(|r| r.name.as_str()).collect();
        names.sort();
        assert_eq!(names, ["api", "web"]);
    }

    /// §7.1: a one-repository estate infers its sole repository on an empty
    /// scope request.
    #[test]
    fn resolve_scope_infers_the_sole_repository_of_a_one_repository_estate_on_empty_scope() {
        let estate = scope_fixture(&["solo"], &[]);
        let resolved = Engine::resolve_scope(&estate, &SubmitContext::default())
            .expect("a one-repository estate infers its sole repository");
        assert_eq!(
            resolved.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(),
            ["solo"]
        );
    }

    /// §7.1: a multi-repository estate refuses an empty scope, and the
    /// refusal carries the full remedy body — repo count, declared repos,
    /// declared groups.
    #[test]
    fn resolve_scope_refuses_an_empty_scope_on_a_multi_repository_estate() {
        let estate = scope_fixture(&["api", "web"], &[("pair", &["api", "web"])]);
        let err = Engine::resolve_scope(&estate, &SubmitContext::default())
            .expect_err("a multi-repository estate must refuse an empty scope");
        match err {
            EngineError::MissingScope {
                repo_count,
                repos,
                groups,
            } => {
                assert_eq!(repo_count, 2);
                assert_eq!(repos, vec!["api".to_string(), "web".to_string()]);
                assert_eq!(groups, vec!["pair".to_string()]);
            }
            other => panic!("expected MissingScope, got {other}"),
        }
    }

    /// §7.1: `--group` may combine with explicit `--repo` additions — union,
    /// dedup, `repos` first in declaration order, new group members
    /// appended.
    #[test]
    fn resolve_scope_unions_group_members_with_explicit_repos_and_dedups() {
        let estate = scope_fixture(&["api", "web", "docs"], &[("pair", &["api", "web"])]);
        let resolved = Engine::resolve_scope(
            &estate,
            &SubmitContext {
                repos: &["web".to_string(), "docs".to_string()],
                group: Some("pair"),
                ..SubmitContext::default()
            },
        )
        .expect("union must resolve");
        assert_eq!(
            resolved.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(),
            ["web", "docs", "api"],
            "explicit --repo first (declaration order), then new group members appended, with \
             no duplicate for the repo already named by both"
        );
    }

    /// §15: an undeclared `--group` is refused by name, listing the estate's
    /// declared groups.
    #[test]
    fn resolve_scope_refuses_an_undeclared_group_naming_the_declared_ones() {
        let estate = scope_fixture(&["api", "web"], &[("pair", &["api", "web"])]);
        let err = Engine::resolve_scope(
            &estate,
            &SubmitContext {
                group: Some("ghost"),
                ..SubmitContext::default()
            },
        )
        .expect_err("an undeclared group must refuse");
        match err {
            EngineError::UnknownGroup {
                requested,
                available,
            } => {
                assert_eq!(requested, "ghost");
                assert_eq!(available, vec!["pair".to_string()]);
            }
            other => panic!("expected UnknownGroup, got {other}"),
        }
    }

    /// §15: an unknown `--repo` name is refused by [`Estate::select`],
    /// naming the estate's declared repositories.
    #[test]
    fn resolve_scope_refuses_an_unknown_repository_naming_the_declared_ones() {
        let estate = scope_fixture(&["api", "web"], &[]);
        let err = Engine::resolve_scope(
            &estate,
            &SubmitContext {
                repos: &["ghost".to_string()],
                ..SubmitContext::default()
            },
        )
        .expect_err("an unknown repository must refuse");
        match err {
            EngineError::RepositorySelection(message) => {
                assert!(message.contains("ghost"), "got: {message}");
                assert!(
                    message.contains("api") && message.contains("web"),
                    "got: {message}"
                );
            }
            other => panic!("expected RepositorySelection, got {other}"),
        }
    }

    /// Every `EngineError`'s wire code, in one table.
    ///
    /// `code()` is the vocabulary `/v1` answers with and the thing a client
    /// branches on; `api.rs` maps it to a status. The paths that raise most
    /// of these variants assert on the human message instead, so a variant
    /// whose code silently changed — or a new variant folded into a
    /// neighbour's arm — would reach clients unnoticed.
    #[test]
    fn every_engine_error_answers_with_its_wire_code() {
        let cases: Vec<(EngineError, &str)> = vec![
            (
                EstateError::Io {
                    path: "/nowhere/sergeant.toml".to_string(),
                    source: std::io::Error::other("unreadable"),
                }
                .into(),
                "workspace_error",
            ),
            (
                WorkflowError::NotFound {
                    name: "nope".to_string(),
                    searched: "/nowhere".to_string(),
                }
                .into(),
                "workflow_error",
            ),
            (
                RouteError::NoSelection {
                    available: vec!["fake".to_string()],
                }
                .into(),
                "no_backend_selected",
            ),
            // All three routing failures, not one of them: `Route` is the
            // only arm that delegates instead of naming a constant, so a
            // single row would let the delegation be replaced by whichever
            // code that row happens to want.
            (
                RouteError::NotFound {
                    requested: "ghost".to_string(),
                    tier: crate::runtime::router::RouteSource::Explicit,
                    available: vec!["fake".to_string()],
                }
                .into(),
                "backend_not_found",
            ),
            (
                RouteError::Unavailable {
                    requested: "claude".to_string(),
                    tier: crate::runtime::router::RouteSource::GlobalDefault,
                    detail: "no CLI on PATH".to_string(),
                    available: vec!["fake".to_string()],
                }
                .into(),
                "backend_unavailable",
            ),
            (SurfaceError::NoRepositories.into(), "surface_error"),
            (
                BackendError::Unavailable {
                    backend: "fake".to_string(),
                    detail: "not here".to_string(),
                }
                .into(),
                "backend_error",
            ),
            (
                CoreError::Projection(crate::runtime::projection::ProjectionError::SeqMismatch {
                    expected: 2,
                    found: 5,
                })
                .into(),
                "internal",
            ),
            (
                EngineError::RepositorySelection("no such repository".to_string()),
                "unknown_repository",
            ),
            (
                EngineError::ProfileNotFound {
                    requested: "nope".to_string(),
                    available: "enterprise".to_string(),
                },
                "profile_not_found",
            ),
            (
                EngineError::ProfileBackendMismatch {
                    profile: "enterprise".to_string(),
                    profile_backend: "codex".to_string(),
                    routed: "fake".to_string(),
                    tier: "global_default".to_string(),
                },
                "profile_backend_mismatch",
            ),
            (
                EngineError::IllegalTransition {
                    work_id: "01W".to_string(),
                    from: WorkState::Completed,
                    to: WorkState::Active,
                },
                "illegal_transition",
            ),
            (
                EngineError::NotAwaitingInput {
                    work_id: "01W".to_string(),
                    state: WorkState::Active,
                },
                "not_awaiting_input",
            ),
            (
                EngineError::NotRetryable {
                    work_id: "01W".to_string(),
                    state: WorkState::Completed,
                },
                "not_retryable",
            ),
            (
                EngineError::NoRun {
                    work_id: "01W".to_string(),
                },
                "no_run",
            ),
            (
                EngineError::BackendMissing {
                    work_id: "01W".to_string(),
                    backend: "ghost".to_string(),
                },
                "backend_missing",
            ),
            (
                EngineError::NoSuchStage {
                    work_id: "01W".to_string(),
                    index: 9,
                },
                "no_such_stage",
            ),
            (
                EngineError::MissingScope {
                    repo_count: 2,
                    repos: vec!["api".to_string(), "web".to_string()],
                    groups: vec![],
                },
                "missing_scope",
            ),
            (
                EngineError::UnknownGroup {
                    requested: "ghost".to_string(),
                    available: vec!["payments".to_string()],
                },
                "unknown_group",
            ),
            (
                EngineError::ConflictingScope {
                    repos: vec!["api".to_string()],
                    group: Some("pair".to_string()),
                },
                "conflicting_scope",
            ),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for (error, code) in &cases {
            assert_eq!(&error.code(), code, "wrong code for {error}");
            assert!(
                seen.insert(*code),
                "two variants share the code {code:?}, which collapses two \
                 client-visible outcomes into one"
            );
        }

        // §13's option list travels with routing failures and with nothing
        // else: a client can only offer a choice where there was one.
        let routing: EngineError = RouteError::NotFound {
            requested: "ghost".to_string(),
            tier: crate::runtime::router::RouteSource::Explicit,
            available: vec!["fake".to_string()],
        }
        .into();
        assert_eq!(
            routing.available_backends(),
            Some(&["fake".to_string()][..]),
            "a routing failure must name what could have been asked for"
        );
        for (error, _) in &cases {
            if matches!(error, EngineError::Route(_)) {
                continue;
            }
            assert!(
                error.available_backends().is_none(),
                "{error} is not a routing failure and must not offer options"
            );
        }
    }

    /// A surface that cannot be materialized blocks the work; it does not
    /// fail the call.
    ///
    /// `Engine::start` runs after the Work exists, so returning an error here
    /// would leave an accepted work in `pending` that nothing ever picks up.
    /// The partial-failure arm (earlier repositories torn down) is pinned by
    /// the m3 suite; this is its sibling — materialization that fails
    /// outright, before any repository was touched.
    #[test]
    fn a_surface_that_cannot_be_materialized_blocks_the_work_rather_than_failing_the_call() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());
        let engine = engine(dir.path());
        let work_id = "01NOSURFACE";
        testing::submit(&mut core, work_id, "cannot be materialized");
        let work = core.registry.state().works[work_id].clone();

        // No repositories: `materialize` refuses outright — a surface with no
        // worktrees could never execute anything — and nothing was created,
        // so there is no teardown report to record.
        let plan = StartPlan {
            estate: Estate {
                name: "solo".to_string(),
                root: dir.path().to_path_buf(),
                repositories: Vec::new(),
                default_backend: None,
                default_workflow: None,
                profiles: Vec::new(),
                config_path: None,
                surfaces_dir: None,
                data_dir: None,
                repository_policy: std::collections::BTreeMap::new(),
                groups: std::collections::BTreeMap::new(),
                repository_origin: std::collections::BTreeMap::new(),
                repository_upstream: std::collections::BTreeMap::new(),
                retention: None,
            },
            repositories: Vec::new(),
            surfaces_root: dir.path().join("surfaces"),
            workflow: WorkflowDefinition {
                name: "tiny".to_string(),
                version: "1".to_string(),
                source: "test".to_string(),
                stages: Vec::new(),
                content_hash: String::new(),
            },
            route: Route {
                backend: FAKE_BACKEND_NAME.to_string(),
                source: crate::runtime::router::RouteSource::GlobalDefault,
            },
            profile: None,
            // N3 (§12.5): one binding per stage, and this plan has no stages.
            stage_bindings: Vec::new(),
            // No admission: this plan was built by hand, not by `plan`.
            preflight: None,
        };

        engine
            .start(&mut core, &work, &plan)
            .expect("a materialization failure is not the caller's error");

        assert_eq!(
            core.registry.state().works[work_id].state,
            WorkState::Blocked,
            "an accepted work whose surface failed must fail closed, never \
             stay pending"
        );
        let events: Vec<_> = core
            .journal
            .replay()
            .expect("replay")
            .map(|e| e.expect("event"))
            .collect();
        let blocked = events
            .iter()
            .find(|e| e.kind == KIND_WORK_BLOCKED)
            .expect("the work is blocked with a reason");
        assert!(
            blocked.payload["reason"]
                .as_str()
                .unwrap_or_default()
                .starts_with("cannot materialize work surface"),
            "the reason must name the failure: {}",
            blocked.payload
        );
        assert!(
            blocked.payload["evidence"].is_null(),
            "nothing was created, so there is no teardown report to attach: {}",
            blocked.payload
        );
        assert!(
            events.iter().any(|e| e.kind == KIND_SURFACE_MATERIALIZING),
            "the intent to materialize is journaled before git is touched"
        );
        for kind in [
            KIND_SURFACE_TORN_DOWN,
            KIND_WORKFLOW_BOUND,
            KIND_WORK_STARTED,
        ] {
            assert!(
                !events.iter().any(|e| e.kind == kind),
                "a surface that never existed must not produce {kind}"
            );
        }

        // Retry is the one door out of `blocked`, and this work has nothing
        // behind it: no stage was ever entered, so the answer is the
        // structured `no_run` the API turns into a 404 — never a panic on an
        // absent stage, and never a second attempt at a surface nobody has
        // fixed yet.
        let error = engine
            .retry(&mut core, work_id)
            .expect_err("there is no stage to re-enter");
        assert_eq!(error.code(), "no_run");
        // The same answer, from the other side: a verb against a work id this
        // daemon has never seen.
        let error = engine
            .retry(&mut core, "01NEVERSUBMITTED")
            .expect_err("an unknown work cannot be retried");
        assert_eq!(error.code(), "no_run");
    }

    /// A run whose backend this daemon no longer registers is named, not
    /// guessed at.
    ///
    /// The journal outlives any one daemon's registry — a data dir opened by
    /// a build that dropped an adapter, or started without it — and the run
    /// records the backend by name. Driving such a run must fail with the
    /// name it cannot find.
    #[test]
    fn a_run_routed_to_an_unregistered_backend_is_named_not_guessed() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());
        let engine = engine(dir.path());
        let work_id = "01GHOSTBACKEND";
        testing::submit(&mut core, work_id, "routed to a backend that is gone");
        testing::commit(
            &mut core,
            work_id,
            KIND_WORKFLOW_BOUND,
            json!({
                "workflow": {"name": "tiny", "version": "1", "source": "test",
                             "stages": [{"id": "00-only", "context": "c"}]},
                "backend": "ghost",
            }),
        );
        testing::commit(
            &mut core,
            work_id,
            KIND_STAGE_ENTERED,
            json!({"stage_id": "00-only", "index": 0, "attempt": 1}),
        );
        testing::commit(
            &mut core,
            work_id,
            KIND_EXECUTION_STARTED,
            json!({"execution": {
                "execution_id": "exec-ghost",
                "backend": "ghost",
                "stage_id": "00-only",
                "attempt": 1,
                "stop_requested": false,
            }}),
        );
        testing::commit(&mut core, work_id, KIND_WORK_STARTED, json!({}));

        let error = engine
            .resume(&mut core, work_id)
            .expect_err("a missing backend cannot be driven");
        assert_eq!(error.code(), "backend_missing");
        assert!(
            error.to_string().contains("ghost"),
            "the diagnostic must name the backend the run wants: {error}"
        );
    }

    /// The §10 transition table is consulted *before* the append, and its
    /// three refusals are all silent-corruption risks if they stop working.
    #[test]
    fn the_transition_table_is_consulted_before_anything_is_appended() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());
        let engine = engine(dir.path());
        let work_id = "01TRANSITIONS";
        testing::submit(&mut core, work_id, "transition me");
        let after_submit = core.journal.next_seq();

        // A kind that maps to no work state is a bug in the caller, refused
        // rather than appended as a transition to nowhere.
        let error = engine
            .transition(&mut core, work_id, KIND_STAGE_ENTERED, json!({}))
            .expect_err("a stage event is not a work transition");
        assert_eq!(error.code(), "illegal_transition");

        // Same state in, same state out: no event, no churn.
        engine
            .transition(&mut core, work_id, KIND_WORK_STARTED, json!({}))
            .expect("pending -> active");
        engine
            .transition(&mut core, work_id, KIND_WORK_STARTED, json!({}))
            .expect("active -> active is not an error");
        let after_start = core.journal.next_seq();

        // And a transition the table forbids fails before the append.
        engine
            .transition(&mut core, work_id, KIND_WORK_COMPLETED, json!({}))
            .expect("active -> completed");
        let after_completion = core.journal.next_seq();
        let error = engine
            .transition(&mut core, work_id, KIND_WORK_STARTED, json!({}))
            .expect_err("completed is terminal");
        assert_eq!(error.code(), "illegal_transition");
        assert!(
            error.to_string().contains("completed -> active"),
            "the refusal must name both ends: {error}"
        );

        assert_eq!(
            after_start - after_submit,
            1,
            "the second start must append nothing"
        );
        assert_eq!(
            core.journal.next_seq(),
            after_completion,
            "a refused transition must leave the journal exactly as it was"
        );
        assert_eq!(
            core.registry.state().works[work_id].state,
            WorkState::Completed
        );
    }

    /// A daemon that died between `stage.entered` and `execution.started`
    /// leaves a run with a stage and no execution (L6's adjacent-append
    /// window, on the one sequence that cannot be compounded — the append
    /// has to precede the process). Driving that prefix is a no-op: there is
    /// nothing to observe, and inventing a conclusion about a stage no
    /// backend ever accepted is exactly what §25 forbids.
    #[test]
    fn driving_a_run_whose_execution_never_landed_is_a_no_op() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());
        let engine = engine(dir.path());
        let work_id = "01NOEXECUTION";
        testing::submit(&mut core, work_id, "crashed before the execution landed");
        testing::commit(
            &mut core,
            work_id,
            KIND_WORKFLOW_BOUND,
            json!({
                "workflow": {"name": "tiny", "version": "1", "source": "test",
                             "stages": [{"id": "00-only", "context": "c"}]},
                "backend": FAKE_BACKEND_NAME,
            }),
        );
        testing::commit(&mut core, work_id, KIND_WORK_STARTED, json!({}));
        testing::commit(
            &mut core,
            work_id,
            KIND_STAGE_ENTERED,
            json!({"stage_id": "00-only", "index": 0, "attempt": 1}),
        );
        let head = core.journal.next_seq();

        engine.resume(&mut core, work_id).expect("nothing to drive");

        assert_eq!(
            core.journal.next_seq(),
            head,
            "a run with no execution must not be driven anywhere"
        );
        assert_eq!(
            core.registry.state().works[work_id].state,
            WorkState::Active,
            "and the work stays where recovery can still find it"
        );
    }

    /// An adapter that does not advertise RESUME is never asked to, and an
    /// adapter that cannot classify its native context fails the work closed.
    ///
    /// Both halves are §15/§25 promises the two shipped adapters cannot
    /// exercise: the fake and the Claude adapter both advertise `resume`, and
    /// neither can be made to answer `unknown` at reconcile time without a
    /// production seam. So this pins them against a backend that is exactly
    /// what §15 allows — capabilities absent, not emulated. If the capability
    /// check disappeared, a restart would call an unsupported verb on every
    /// such adapter and turn its refusal into the work's evidence.
    #[test]
    fn reconcile_never_calls_an_unadvertised_resume_and_fails_closed_on_unknown() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        use crate::backend::{Capabilities, Completion, NativeEvent, ProbeReport, RuntimeScope};

        #[derive(Debug, Default)]
        struct Inscrutable {
            resume_calls: AtomicUsize,
        }

        impl Backend for Inscrutable {
            fn name(&self) -> &str {
                "inscrutable"
            }
            fn capabilities(&self) -> Capabilities {
                // Everything absent — §15's "unsupported means unsupported".
                Capabilities::default()
            }
            fn runtime_scope(&self) -> RuntimeScope {
                RuntimeScope::PerExecution
            }
            fn probe(&self) -> ProbeReport {
                ProbeReport {
                    available: true,
                    detail: None,
                }
            }
            // N3 split START into PREPARE + LAUNCH (§14.3); this test never
            // starts an execution, so both halves stay unreachable.
            fn prepare(&self, _: &StartRequest) -> Result<PreparedExecution, BackendError> {
                unreachable!("this test never prepares an execution")
            }
            fn launch(&self, _: &PreparedExecution) -> Result<ExecutionHandle, BackendError> {
                unreachable!("this test never launches an execution")
            }
            fn send(&self, _: &ExecutionHandle, _: &str) -> Result<(), BackendError> {
                unreachable!("this test never sends")
            }
            fn observe(&self, _: &ExecutionHandle) -> Result<Observation, BackendError> {
                Ok(Observation {
                    native: NativeState::Unknown,
                    signal: BackendSignal::Running,
                    evidence: None,
                })
            }
            fn interrupt(&self, _: &ExecutionHandle) -> Result<Completion, BackendError> {
                Ok(Completion::immediate())
            }
            fn resume(&self, _: &ExecutionHandle, _: &ResumeRequest) -> Result<(), BackendError> {
                self.resume_calls.fetch_add(1, Ordering::SeqCst);
                Err(BackendError::Unsupported {
                    backend: "inscrutable".to_string(),
                    verb: "resume".to_string(),
                    detail: "this adapter never advertised it".to_string(),
                })
            }
            fn history(&self, _: &ExecutionHandle) -> Result<Vec<NativeEvent>, BackendError> {
                unreachable!("this test never reads history")
            }
            fn stop(&self, _: &ExecutionHandle) -> Result<Completion, BackendError> {
                Ok(Completion::immediate())
            }
        }

        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());
        let backend = Arc::new(Inscrutable::default());
        let engine = Engine::new(
            Arc::new(BackendRegistry::new().with(backend.clone())),
            None,
            dir.path(),
        );

        let work_id = "01INSCRUTABLE";
        testing::submit(
            &mut core,
            work_id,
            "reconciled by an adapter that cannot say",
        );
        testing::commit(
            &mut core,
            work_id,
            KIND_WORKFLOW_BOUND,
            json!({
                "workflow": {"name": "tiny", "version": "1", "source": "test",
                             "stages": [{"id": "00-only", "context": "c"}]},
                "backend": "inscrutable",
            }),
        );
        // A surface *is* recorded, so the decline below can only be about the
        // missing capability — never about having nowhere to reattach into.
        testing::commit(
            &mut core,
            work_id,
            KIND_SURFACE_MATERIALIZED,
            json!({"surface": {
                "work_id": work_id,
                "root": dir.path(),
                "bindings": [{
                    "repository": "solo",
                    "source_path": dir.path(),
                    "base_branch": "main",
                    "base_sha": "0".repeat(40),
                    "worktree_path": dir.path(),
                    "work_branch": format!("sergeant/{work_id}"),
                    "head_sha": "0".repeat(40),
                }],
            }}),
        );
        testing::commit(
            &mut core,
            work_id,
            KIND_STAGE_ENTERED,
            json!({"stage_id": "00-only", "index": 0, "attempt": 1}),
        );
        testing::commit(
            &mut core,
            work_id,
            KIND_EXECUTION_STARTED,
            json!({"execution": {
                "execution_id": "exec-inscrutable",
                "backend": "inscrutable",
                "stage_id": "00-only",
                "attempt": 1,
                "stop_requested": false,
            }}),
        );
        testing::commit(&mut core, work_id, KIND_WORK_STARTED, json!({}));

        let disposition = engine
            .reconcile_work(&mut core, work_id)
            .expect("reconciliation itself does not fail");

        assert_eq!(disposition, ReconcileDisposition::Ambiguous);
        assert_eq!(
            backend.resume_calls.load(Ordering::SeqCst),
            0,
            "a backend that does not advertise RESUME must not be asked to"
        );
        let reconciled = core
            .journal
            .replay()
            .expect("replay")
            .map(|e| e.expect("event"))
            .find(|e| e.kind == KIND_EXECUTION_RECONCILED)
            .expect("the decision is journaled");
        assert_eq!(
            reconciled.payload["reattached"], false,
            "and the record says so, rather than implying ownership: {}",
            reconciled.payload
        );
        assert_eq!(
            reconciled.payload["evidence"], "backend reports unknown native state",
            "an adapter that offers no evidence still gets a stated reason: {}",
            reconciled.payload
        );
        assert_eq!(
            core.registry.state().works[work_id].state,
            WorkState::Blocked,
            "§25: ambiguity fails closed"
        );
    }

    /// Issue #90: a ceiling interrupt must land the Work somewhere an
    /// operator verb can reach, not park it in `active` forever.
    ///
    /// Before this fix, `settle_interrupt` only journaled
    /// `execution.turn_ceiling_interrupted` and returned `Next::Parked`,
    /// trusting the next OBSERVE to resolve things. It never can: the real
    /// adapter's own interrupted-with-no-envelope arm
    /// (`observe_in_memory`, `src/backend/claude.rs`) reports
    /// `BackendSignal::Running` forever on purpose — "no conclusion about
    /// the stage is invented" — and `drive`'s `Running` arm parks without
    /// journaling anything. The work never left `active`, and `retry`
    /// refuses anything but `failed`/`blocked`/`waiting`, so the only
    /// reachable verb was the destructive `cancel`.
    ///
    /// This fault-injection test drives the real `due_interrupts` →
    /// `settle_interrupt` sequence (never synthesizing the landing) and
    /// asserts the work reaches `blocked` with the ceiling named as the
    /// reason, the stage record carries the same decision (so `retry` has a
    /// current stage to re-enter), and `retry` — R-MVP1-10's existing exit
    /// door, reused for a different §25 trigger — actually reopens it.
    ///
    /// Mutation target (L7): reverting the `WorkState::Active` block added to
    /// `settle_interrupt` leaves the work `active` after this test's
    /// interrupt settles, failing the first assertion below directly.
    #[test]
    fn a_ceiling_interrupt_lands_the_work_in_blocked_not_wedged_active() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());
        let fake = FakeBackend::new(FAKE_BACKEND_NAME);
        let backends = BackendRegistry::new().with(Arc::new(fake.clone()));
        let engine = Engine::new(
            Arc::new(backends),
            Some(FAKE_BACKEND_NAME.to_string()),
            dir.path(),
        )
        .with_turn_ceiling(Duration::ZERO);

        let work_id = "01CEILINGWEDGE";
        testing::submit(&mut core, work_id, "run past the ceiling");
        testing::commit(
            &mut core,
            work_id,
            KIND_WORKFLOW_BOUND,
            json!({
                "workflow": {"name": "tiny", "version": "1", "source": "test",
                             "stages": [{"id": "00-first", "context": "c"}]},
                "backend": FAKE_BACKEND_NAME,
            }),
        );
        // A surface is recorded so the retry proof at the end has something
        // real to re-enter into — `reserve_stage` refuses `NoRun` without
        // one, on the very first launch as much as on a retry.
        testing::commit(
            &mut core,
            work_id,
            KIND_SURFACE_MATERIALIZED,
            json!({"surface": {
                "work_id": work_id,
                "root": dir.path(),
                "bindings": [{
                    "repository": "solo",
                    "source_path": dir.path(),
                    "base_branch": "main",
                    "base_sha": "0".repeat(40),
                    "worktree_path": dir.path(),
                    "work_branch": format!("sergeant/{work_id}"),
                    "head_sha": "0".repeat(40),
                }],
            }}),
        );
        testing::commit(
            &mut core,
            work_id,
            KIND_STAGE_ENTERED,
            json!({"stage_id": "00-first", "index": 0, "attempt": 1}),
        );

        // A real execution the fake backend actually knows about — INTERRUPT
        // below must exercise the same `resolve`-then-kill path the daemon's
        // own crank loop does, not a synthetic identity nothing can answer
        // for.
        let start_request = StartRequest {
            work_id: work_id.to_string(),
            execution_id: "e1".to_string(),
            stage_id: "00-first".to_string(),
            attempt: 1,
            cwd: dir.path().to_path_buf(),
            intent: "run past the ceiling".to_string(),
            context: "c".to_string(),
            model: None,
            profile: None,
            execute: None,
            instruction_policy: InstructionPolicy::default(),
            bindings: Vec::new(),
        };
        let handle = fake.start(&start_request).expect("fake backend start");
        testing::commit(
            &mut core,
            work_id,
            KIND_EXECUTION_STARTED,
            json!({"execution": {
                "execution_id": "e1",
                "backend": FAKE_BACKEND_NAME,
                "native_id": handle.native_id,
                "stage_id": "00-first",
                "attempt": 1,
                "stop_requested": false,
            }}),
        );
        testing::commit(&mut core, work_id, KIND_WORK_STARTED, json!({}));

        engine.record_turn_start(work_id);
        let mut overdue = engine.due_interrupts(&core, Instant::now());
        assert_eq!(
            overdue.len(),
            1,
            "a zero ceiling makes the turn overdue on the first sweep"
        );
        let pending = overdue.pop().expect("the one collected crossing");
        // Driven through `run_inline` rather than `pending.perform()` called
        // directly here: t11 (`tests/m6_surfaces.rs`) statically scans this
        // file and only permits an external effect's perform inside a
        // reviewed single-owner block. `run_inline` is already on that list
        // (deterministic tests hold the only reference to `Core`, same as
        // every other entry there) — this test adds nothing to it.
        engine
            .run_inline(
                &mut core,
                Step {
                    next: Next::Interrupt(pending),
                    deferred: Deferred::new(),
                },
            )
            .expect("run the interrupt inline");

        assert_eq!(
            core.registry.state().works[work_id].state,
            WorkState::Blocked,
            "the interrupted work must land somewhere `retry`/`cancel` can \
             reach, never wedged in `active` forever"
        );
        assert_eq!(
            core.registry.state().runs[work_id]
                .current_stage()
                .expect("a stage")
                .status,
            StageStatus::Blocked,
            "the stage record must carry the same decision, or `retry` has \
             no current stage to re-enter"
        );
        let blocked: Vec<Value> = core
            .journal
            .replay()
            .expect("replay")
            .map(|e| e.expect("event"))
            .filter(|e| e.kind == KIND_WORK_BLOCKED && e.work_id.as_deref() == Some(work_id))
            .map(|e| e.payload)
            .collect();
        assert_eq!(blocked.len(), 1);
        assert!(
            blocked[0]["reason"]
                .as_str()
                .is_some_and(|r| r.contains("ceiling")),
            "the reason names the actual trigger: {:?}",
            blocked[0]
        );

        // R-MVP1-10's door, reused: `retry` is legal from here and produces a
        // genuine re-entry effect — never `NotRetryable` (409), which is
        // exactly what a still-`active` work would have refused with. The
        // effect itself (`Next::Surface`'s rematerialize) is real git work
        // this synthetic fixture cannot perform end to end — that plumbing
        // is R-MVP1-10's own, proven elsewhere — so this stops at `begin_retry`,
        // the phase that proves the door opened.
        let reopened = engine
            .begin_retry(&mut core, work_id)
            .expect("retry must reopen a ceiling-interrupted block, not refuse it");
        assert!(
            matches!(reopened.next, Next::Surface(_)),
            "retry must produce a real re-entry effect, not a silent no-op: {:?}",
            reopened.next
        );
    }

    // ------------------- §8.6 Mechanism A: branch_takeover_precondition

    mod branch_takeover {
        use super::*;
        use crate::runtime::projection::WorkIndexRow;
        use crate::runtime::surface::{BindingDisposition, BindingOrigin, BindingTeardown};

        fn work(id: &str, state: WorkState) -> Work {
            Work {
                id: id.to_string(),
                workspace: None,
                intent: "review this".to_string(),
                repositories: Vec::new(),
                scope_request: Default::default(),
                workflow: None,
                backend: None,
                origin_client: None,
                profile: None,
                intent_detail: None,
                envelope: None,
                git_preflight_override: false,
                state,
                created_by: "test".to_string(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
            }
        }

        fn binding(work_id: &str) -> RepositoryBinding {
            RepositoryBinding {
                repository: "solo".to_string(),
                source_path: PathBuf::from("/repos/solo"),
                base_branch: Some("main".to_string()),
                base_sha: "0".repeat(40),
                worktree_path: PathBuf::from(format!("/data/surfaces/{work_id}/solo")),
                work_branch: format!("sergeant/{work_id}"),
                head_sha: "1".repeat(40),
                origin: BindingOrigin::Cut,
                canonical_top_level: Some(PathBuf::from("/repos/solo")),
                canonical_common_dir: Some(PathBuf::from("/repos/solo/.git")),
                preflight: None,
            }
        }

        fn surface(work_id: &str) -> WorkSurface {
            WorkSurface {
                work_id: work_id.to_string(),
                root: PathBuf::from(format!("/data/surfaces/{work_id}")),
                bindings: vec![binding(work_id)],
            }
        }

        fn teardown_report(work_id: &str, clean: bool) -> TeardownReport {
            let disposition = if clean {
                BindingDisposition::Removed
            } else {
                BindingDisposition::RetainedDirty {
                    changes: "M solo/wip.rs".to_string(),
                    patch: None,
                    outputs: Vec::new(),
                }
            };
            TeardownReport {
                work_id: work_id.to_string(),
                bindings: vec![BindingTeardown {
                    repository: "solo".to_string(),
                    worktree_path: PathBuf::from(format!("/data/surfaces/{work_id}/solo")),
                    work_branch: format!("sergeant/{work_id}"),
                    final_sha: Some("1".repeat(40)),
                    observed_head: None,
                    findings: Vec::new(),
                    disposition,
                }],
                clean,
                drift: Vec::new(),
            }
        }

        /// A target that does not exist at all is refused, named, rather
        /// than panicking on a missing lookup.
        #[test]
        fn refuses_when_the_target_does_not_exist() {
            let registry = WorkRegistry::default();
            let err = branch_takeover_precondition(&registry, "01GHOST")
                .expect_err("a nonexistent work must be refused");
            assert!(matches!(
                err,
                BranchTakeoverError::TargetNotFound { work_id } if work_id == "01GHOST"
            ));
        }

        /// Every non-absorbing state refuses — including `failed`, which
        /// `WorkState`'s own doc comment calls "terminal" but which
        /// `is_absorbing` deliberately excludes because it is retryable. A
        /// retry would rematerialize the target's own worktree back onto the
        /// branch a gate surface may have already attached to, which is
        /// exactly the race this precondition exists to prevent. Each
        /// state's error names that exact state, not a generic refusal
        /// reused across every non-terminal case.
        #[test]
        fn each_non_terminal_state_is_refused_with_its_own_reason() {
            for state in [
                WorkState::Pending,
                WorkState::Active,
                WorkState::Waiting,
                WorkState::NeedsInput,
                WorkState::Blocked,
                WorkState::Failed,
            ] {
                let mut registry = WorkRegistry::default();
                registry
                    .works
                    .insert("01TARGET".to_string(), work("01TARGET", state));
                let err = branch_takeover_precondition(&registry, "01TARGET")
                    .expect_err("must be refused");
                assert!(
                    matches!(
                        err,
                        BranchTakeoverError::TargetNotTerminal { ref work_id, state: s }
                            if work_id == "01TARGET" && s == state
                    ),
                    "state {state}: expected TargetNotTerminal naming it, got {err:?}"
                );
            }
        }

        /// A Work in an absorbing terminal state that never had a run at
        /// all — no repositories, never bound a workflow — has no surface to
        /// take over.
        #[test]
        fn refuses_when_the_target_has_no_run_at_all() {
            let mut registry = WorkRegistry::default();
            registry
                .works
                .insert("01NORUN".to_string(), work("01NORUN", WorkState::Completed));
            let err =
                branch_takeover_precondition(&registry, "01NORUN").expect_err("must be refused");
            assert!(matches!(
                err,
                BranchTakeoverError::NoSurface { work_id } if work_id == "01NORUN"
            ));
        }

        /// A run exists (e.g. a workflow was bound) but no surface was ever
        /// materialized — also no branch to take over.
        #[test]
        fn refuses_when_the_target_has_a_run_but_no_surface() {
            let mut registry = WorkRegistry::default();
            registry.works.insert(
                "01NOSURFACE".to_string(),
                work("01NOSURFACE", WorkState::Completed),
            );
            registry
                .runs
                .insert("01NOSURFACE".to_string(), WorkRun::default());
            let err = branch_takeover_precondition(&registry, "01NOSURFACE")
                .expect_err("must be refused");
            assert!(matches!(
                err,
                BranchTakeoverError::NoSurface { work_id } if work_id == "01NOSURFACE"
            ));
        }

        /// A surface exists but teardown has not run yet — the real window
        /// `retire_run`'s own ordering (state, then stage, then backend
        /// stop, then teardown) leaves open between a Work going terminal
        /// and its surface actually tearing down.
        #[test]
        fn refuses_when_the_surface_has_not_torn_down_yet() {
            let mut registry = WorkRegistry::default();
            registry.works.insert(
                "01NOTEARDOWN".to_string(),
                work("01NOTEARDOWN", WorkState::Completed),
            );
            registry.runs.insert(
                "01NOTEARDOWN".to_string(),
                WorkRun {
                    surface: Some(surface("01NOTEARDOWN")),
                    ..Default::default()
                },
            );
            let err = branch_takeover_precondition(&registry, "01NOTEARDOWN")
                .expect_err("must be refused");
            assert!(matches!(
                err,
                BranchTakeoverError::SurfaceNotTornDown { work_id } if work_id == "01NOTEARDOWN"
            ));
        }

        /// Teardown ran but was not clean — a worktree is still retained
        /// (dirty, missing, or errored). Takeover must not force past
        /// teardown's own fail-closed disposition.
        #[test]
        fn refuses_when_teardown_was_not_clean() {
            let mut registry = WorkRegistry::default();
            registry
                .works
                .insert("01DIRTY".to_string(), work("01DIRTY", WorkState::Completed));
            registry.runs.insert(
                "01DIRTY".to_string(),
                WorkRun {
                    surface: Some(surface("01DIRTY")),
                    teardown: Some(teardown_report("01DIRTY", false)),
                    ..Default::default()
                },
            );
            let err =
                branch_takeover_precondition(&registry, "01DIRTY").expect_err("must be refused");
            assert!(matches!(
                err,
                BranchTakeoverError::SurfaceNotClean { work_id, .. } if work_id == "01DIRTY"
            ));
        }

        /// The permit side: a terminal target whose surface tore down
        /// clean returns exactly its own bindings — everything
        /// `surface::attach` needs.
        #[test]
        fn permits_a_terminal_cleanly_torn_down_target() {
            for state in [WorkState::Completed, WorkState::Canceled] {
                let mut registry = WorkRegistry::default();
                registry
                    .works
                    .insert("01CLEAN".to_string(), work("01CLEAN", state));
                registry.runs.insert(
                    "01CLEAN".to_string(),
                    WorkRun {
                        surface: Some(surface("01CLEAN")),
                        teardown: Some(teardown_report("01CLEAN", true)),
                        ..Default::default()
                    },
                );
                let bindings = branch_takeover_precondition(&registry, "01CLEAN")
                    .unwrap_or_else(|e| panic!("state {state} must be permitted, got {e:?}"));
                assert_eq!(bindings, vec![binding("01CLEAN")]);
            }
        }

        /// The permit side also holds for a run the terminal-run eviction
        /// cache has already reclaimed (`WorkRegistry::terminal_runs`) —
        /// `run_view` falls back to it transparently, and this function must
        /// not bypass that by reading `registry.runs` directly.
        #[test]
        fn permits_a_target_whose_run_was_already_evicted_to_the_terminal_cache() {
            let mut registry = WorkRegistry::default();
            registry.works.insert(
                "01EVICTED".to_string(),
                work("01EVICTED", WorkState::Completed),
            );
            registry.terminal_runs.insert(
                "01EVICTED".to_string(),
                WorkRun {
                    surface: Some(surface("01EVICTED")),
                    teardown: Some(teardown_report("01EVICTED", true)),
                    ..Default::default()
                },
            );
            let bindings = branch_takeover_precondition(&registry, "01EVICTED")
                .expect("an evicted-but-cached terminal run must still be readable");
            assert_eq!(bindings, vec![binding("01EVICTED")]);
        }

        /// #4's own version of the test above: the *Work* itself, not only
        /// its run, can be evicted from the live map now — aged out of
        /// `terminal_works` entirely, answerable only from `work_index`.
        /// `branch_takeover_precondition` must consult that slim index for
        /// `.state` rather than reading `registry.works` bare, or a
        /// perfectly good ADR 0017 takeover target would refuse as
        /// `TargetNotFound` the instant its Work aged out of the cache.
        #[test]
        fn permits_a_target_whose_work_was_evicted_from_the_live_map_too() {
            let mut registry = WorkRegistry::default();
            // Not in `works`, not even in `terminal_works` — only the slim
            // index remembers this id exists and what state it is in.
            registry.work_index.insert(
                "01WORKEVICTED".to_string(),
                WorkIndexRow {
                    id: "01WORKEVICTED".to_string(),
                    intent: "review this".to_string(),
                    state: WorkState::Completed,
                    integrity: None,
                    created_at: "2026-01-01T00:00:00Z".to_string(),
                    updated_at: "2026-01-01T00:00:00Z".to_string(),
                    last_seq: 1,
                },
            );
            registry.terminal_runs.insert(
                "01WORKEVICTED".to_string(),
                WorkRun {
                    surface: Some(surface("01WORKEVICTED")),
                    teardown: Some(teardown_report("01WORKEVICTED", true)),
                    ..Default::default()
                },
            );
            let bindings = branch_takeover_precondition(&registry, "01WORKEVICTED").expect(
                "a Work evicted past `terminal_works`, with only a slim index row left, \
                 must still be readable for its state",
            );
            assert_eq!(bindings, vec![binding("01WORKEVICTED")]);
        }
    }
}
