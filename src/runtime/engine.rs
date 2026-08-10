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

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::{Value, json};

use crate::api::{Core, CoreError};
use crate::backend::{
    Backend, BackendError, BackendRegistry, BackendSignal, Deferred, ExecutionHandle, NativeState,
    Observation, PreparedExecution, ResumeRequest, StartRequest,
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
    KIND_STAGE_WAITING, KIND_WORKFLOW_BOUND, StageBinding, StageDefinition, StageStatus,
    WorkflowDefinition, WorkflowError,
};
use crate::domain::workspace::{RepositorySpec, Workspace, WorkspaceError};
use crate::runtime::projection::WorkRun;
use crate::runtime::router::{Route, RouteError, RouteInputs, route, route_stage};
use crate::runtime::surface::{
    KIND_SURFACE_MATERIALIZED, KIND_SURFACE_MATERIALIZING, KIND_SURFACE_TORN_DOWN, SurfaceError,
    SurfacePlan, materialize, rematerialize, teardown,
};

/// Everything a submission says about how it wants to run.
#[derive(Debug, Clone, Copy, Default)]
pub struct SubmitContext<'a> {
    /// Client's working directory; workspace discovery starts here (§9).
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
    /// Requested repository subset; empty means the whole workspace.
    pub repositories: &'a [String],
}

/// A resolved, side-effect-free plan for starting a run.
///
/// Planning is separated from starting so that everything which can be
/// *decided* — workspace topology, workflow content, routing, profile — is
/// decided before a Work record exists. A submission that cannot be routed is
/// rejected with §13's available options instead of creating work that
/// immediately dies.
#[derive(Debug, Clone)]
pub struct StartPlan {
    /// The discovered workspace.
    pub workspace: Workspace,
    /// Repositories this run targets.
    pub repositories: Vec<RepositorySpec>,
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
    /// A launch result with no observation attached: `settle_launch` then
    /// takes the observation itself, as it did before the phase existed.
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
    /// Workspace resolution failed.
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
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
    /// The requested repository subset does not match the workspace.
    #[error("{0}")]
    RepositorySelection(String),
    /// The requested profile is not declared by the workspace.
    #[error("no profile named {requested:?} in this workspace (has: {available})")]
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
}

impl EngineError {
    /// Structured error code for the API.
    pub fn code(&self) -> &'static str {
        match self {
            EngineError::Workspace(_) => "workspace_error",
            EngineError::Workflow(_) => "workflow_error",
            EngineError::Route(e) => e.code(),
            EngineError::Surface(_) => "surface_error",
            EngineError::Backend(_) => "backend_error",
            EngineError::Core(_) => "internal",
            EngineError::RepositorySelection(_) => "unknown_repository",
            EngineError::ProfileNotFound { .. } => "profile_not_found",
            EngineError::ProfileBackendMismatch { .. } => "profile_backend_mismatch",
            EngineError::IllegalTransition { .. } => "illegal_transition",
            EngineError::NotAwaitingInput { .. } => "not_awaiting_input",
            EngineError::NotRetryable { .. } => "not_retryable",
            EngineError::NoRun { .. } => "no_run",
            EngineError::BackendMissing { .. } => "backend_missing",
            EngineError::NoSuchStage { .. } => "no_such_stage",
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

/// The workflow engine: backends, defaults, and the data dir surfaces live in.
#[derive(Debug, Clone)]
pub struct Engine {
    /// Backends this daemon can route to.
    pub backends: Arc<BackendRegistry>,
    /// §13's last tier before failure.
    pub default_backend: Option<String>,
    /// Data dir owning `surfaces/`.
    pub data_dir: PathBuf,
}

impl Engine {
    /// Build an engine over a registry and data dir.
    pub fn new(
        backends: Arc<BackendRegistry>,
        default_backend: Option<String>,
        data_dir: &Path,
    ) -> Self {
        Self {
            backends,
            default_backend,
            data_dir: data_dir.to_path_buf(),
        }
    }

    /// Resolve everything a run needs, without touching anything.
    ///
    /// `Ok(None)` means "no workspace here": the client gave no working
    /// directory, or the one it gave is not inside a Git repository. §9's
    /// discovery has a definite answer in that case — there is no repository
    /// surface — so the intent is accepted and stays `pending` rather than
    /// being rejected. Every *other* failure (a malformed `sergeant.toml`, an
    /// unroutable backend, a missing workflow) is a real error and is
    /// returned as one.
    ///
    /// "No surface" does not mean "no §13". A submission that *names* a
    /// backend has asked for something, and §13's terminal state for a
    /// selection sergeant cannot honour is "fail with available options" —
    /// not "record the name anyway on a work that will never run it". So the
    /// chain is consulted either way; only its no-selection outcome is
    /// tolerated here, because a captured intent with no repository has
    /// nothing to route yet and no default to disappoint.
    pub fn plan(&self, context: &SubmitContext<'_>) -> Result<Option<StartPlan>, EngineError> {
        let workspace = match context.cwd {
            None => None,
            Some(cwd) => match Workspace::discover(cwd) {
                Ok(workspace) => Some(workspace),
                Err(WorkspaceError::NotARepository { .. }) => None,
                Err(e) => return Err(e.into()),
            },
        };
        let Some(workspace) = workspace else {
            self.check_selection_is_honourable(context)?;
            return Ok(None);
        };
        let repositories = workspace
            .select(context.repositories)
            .map_err(EngineError::RepositorySelection)?;

        let workflow_name = context
            .workflow
            .or(workspace.default_workflow.as_deref())
            .unwrap_or(DEFAULT_WORKFLOW)
            .to_string();
        let workflow = WorkflowDefinition::resolve(&workspace.root, &workflow_name)?;

        let route = route(
            &RouteInputs {
                explicit: context.backend,
                origin_client: context.origin_client,
                workspace_default: workspace.default_backend.as_deref(),
                global_default: self.default_backend.as_deref(),
            },
            &self.backends,
        )?;

        let profile = match context.profile {
            None => None,
            Some(name) => {
                let profile = Self::workspace_profile(&workspace, name)?;
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
        let stage_bindings = self.bind_stages(&workspace, &workflow, &route, profile.as_ref())?;

        Ok(Some(StartPlan {
            workspace,
            repositories,
            workflow,
            route,
            profile,
            stage_bindings,
        }))
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
        workspace: &Workspace,
        workflow: &WorkflowDefinition,
        route: &Route,
        work_profile: Option<&Profile>,
    ) -> Result<Vec<StageBinding>, EngineError> {
        let mut bindings = Vec::with_capacity(workflow.stages.len());
        for (index, stage) in workflow.stages.iter().enumerate() {
            let stage_route = route_stage(
                stage.harness.as_deref(),
                Some(route.backend.as_str()),
                &self.backends,
            )?;
            let profile = self.stage_profile(workspace, stage, &stage_route, work_profile)?;
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
        workspace: &Workspace,
        stage: &StageDefinition,
        stage_route: &Route,
        work_profile: Option<&Profile>,
    ) -> Result<Option<Profile>, EngineError> {
        let (profile, tier) = match stage.profile.as_deref() {
            Some(name) => (
                Some(Self::workspace_profile(workspace, name)?),
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

    /// A profile the workspace declares, or §14's "name them consistently"
    /// error with the names that do exist.
    fn workspace_profile(workspace: &Workspace, name: &str) -> Result<Profile, EngineError> {
        workspace
            .profile(name)
            .cloned()
            .ok_or_else(|| EngineError::ProfileNotFound {
                requested: name.to_string(),
                available: workspace
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
                workspace_default: None,
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

    /// [`Engine::start`]'s first phase: everything up to and including the
    /// first stage's reservation, all of it under the caller's lock.
    ///
    /// The daemon calls this, releases the core guard, performs the returned
    /// [`Next::Launch`], and comes back through [`Engine::settle_launch`].
    pub fn begin_start(
        &self,
        core: &mut Core,
        work: &Work,
        plan: &StartPlan,
    ) -> Result<Step, EngineError> {
        // Recorded first: everything below this line can create a branch and
        // a worktree in a repository sergeant does not own.
        self.commit(
            core,
            &work.id,
            KIND_SURFACE_MATERIALIZING,
            json!({"plan": SurfacePlan::new(&self.data_dir, &work.id, &plan.repositories)}),
        )?;
        let surface = match materialize(&self.data_dir, &work.id, &plan.repositories) {
            Ok(surface) => surface,
            Err(SurfaceError::PartialFailure { source, teardown }) => {
                // Earlier repositories in this request got a real worktree
                // and branch before a later one failed; materialize() has
                // already torn those down. Journal the report so it is not a
                // mystery, and put it in the evidence too.
                self.commit(
                    core,
                    &work.id,
                    KIND_SURFACE_TORN_DOWN,
                    json!({"report": teardown}),
                )?;
                self.block(
                    core,
                    &work.id,
                    &format!("cannot materialize work surface: {source}"),
                    Some(serde_json::to_string(&teardown).unwrap_or_default()),
                )?;
                return Ok(Step::parked());
            }
            Err(e) => {
                self.block(
                    core,
                    &work.id,
                    &format!("cannot materialize work surface: {e}"),
                    None,
                )?;
                return Ok(Step::parked());
            }
        };
        self.commit(
            core,
            &work.id,
            KIND_SURFACE_MATERIALIZED,
            json!({"surface": surface}),
        )?;
        self.commit(
            core,
            &work.id,
            KIND_WORKFLOW_BOUND,
            json!({
                "workflow": plan.workflow,
                "backend": plan.route.backend,
                "route_source": plan.route.source,
                "profile": plan.profile,
                "workspace": plan.workspace.name,
                // §12.5's per-stage decisions, pinned with the procedure they
                // belong to: the whole executor spec for the run, decided once
                // and never re-derived (§22.4's retry and restart rows).
                "stage_bindings": plan.stage_bindings,
            }),
        )?;
        self.transition(
            core,
            &work.id,
            KIND_WORK_STARTED,
            json!({"backend": plan.route.backend, "workflow": plan.workflow.name}),
        )?;
        let next = self.reserve_stage(core, &work.id, 0, 1)?;
        Ok(Step {
            next,
            deferred: Deferred::new(),
        })
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
        let stage_id = run
            .current_stage()
            .map(|s| s.stage_id.clone())
            .unwrap_or_default();
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
        Ok(Step {
            next: Next::Send(Box::new(PendingSend {
                work_id: work_id.to_string(),
                stage_id,
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
        if let Some(surface) = run.surface.clone() {
            let needs_surface = surface.bindings.iter().any(|b| !b.worktree_path.exists());
            if needs_surface {
                match rematerialize(&surface) {
                    Ok(surface) => self.commit(
                        core,
                        work_id,
                        KIND_SURFACE_MATERIALIZED,
                        json!({"surface": surface, "rematerialized": true}),
                    )?,
                    Err(e) => {
                        return Err(e.into());
                    }
                };
            }
        }

        self.transition(
            core,
            work_id,
            KIND_WORK_RESUMED,
            json!({"reason": "retry", "stage_id": current.stage_id}),
        )?;
        let next = self.reserve_stage(core, work_id, current.index, current.attempt + 1)?;
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
        self.tear_down_surface(core, work_id)?;
        Ok(Step {
            next: Next::Parked,
            deferred,
        })
    }

    /// Drive a run forward from whatever its backend now says.
    ///
    /// This is the only place stage progression happens, and the only signals
    /// it acts on are the backend's explicit ones. It loops because completing
    /// a stage enters the next one, which may itself already have something to
    /// say; every iteration either returns or advances to a later stage, so
    /// the loop is bounded by the workflow's stage count.
    pub fn resume(&self, core: &mut Core, work_id: &str) -> Result<(), EngineError> {
        let step = self.begin_resume(core, work_id)?;
        self.run_inline(core, step)
    }

    /// [`Engine::resume`]'s first phase (see [`Engine::begin_start`]).
    pub fn begin_resume(&self, core: &mut Core, work_id: &str) -> Result<Step, EngineError> {
        let mut deferred = Deferred::new();
        let next = self.drive(core, work_id, None, &mut deferred)?;
        Ok(Step { next, deferred })
    }

    /// [`Engine::resume`]'s implementation, with an optional pre-fetched
    /// `Observation` for the first loop iteration.
    ///
    /// Restart reconciliation ([`Engine::reconcile_work`]) already calls
    /// `backend.observe()` once to decide whether the run resumes at all; a
    /// second OBSERVE right after to actually drive it would double the call
    /// for no reason, and the two calls are not guaranteed to agree. `initial`
    /// lets reconciliation hand the answer it already has to the first
    /// iteration; every later iteration (a fresh stage after this one
    /// completes) always observes fresh, because nothing has asked yet.
    fn drive(
        &self,
        core: &mut Core,
        work_id: &str,
        initial: Option<Result<Observation, BackendError>>,
        deferred: &mut Deferred,
    ) -> Result<Next, EngineError> {
        let run = self.run(core, work_id)?;
        let Some(execution) = run.execution.clone() else {
            return Ok(Next::Parked);
        };
        let Some(stage) = run.current_stage().cloned() else {
            return Ok(Next::Parked);
        };
        let workflow = run.workflow.clone().ok_or_else(|| EngineError::NoRun {
            work_id: work_id.to_string(),
        })?;
        let backend = self.backend_for(work_id, &execution.backend)?;

        let observation = match initial.unwrap_or_else(|| backend.observe(&handle_of(&execution))) {
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
                self.tear_down_surface(core, work_id)?;
                Ok(Next::Parked)
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
                self.tear_down_surface(core, work_id)?;
                Ok(Next::Parked)
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
                let next = self.drive(core, work_id, resumed_from.map(Ok), &mut deferred)?;
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
    /// Three things this deliberately does **not** do:
    ///
    /// - it does not ask the adapter to observe or resume the identity. The
    ///   adapter has no record of it (the reservation predates its state), so
    ///   the question would return `UnknownExecution` and that refusal would
    ///   read like evidence the context does not exist. It is not.
    /// - it does not stop or delete anything. §22.5's rule for every crash
    ///   window is that recovery must not delete unproven state, and a
    ///   possibly-nonexistent native context is the purest case of it.
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
        let evidence = format!(
            "execution {} was reserved on backend {:?} for stage {} attempt {}, and the journal \
             never recorded whether its launch happened ({identity}); sergeant will not guess, \
             start it a second time, or remove anything it cannot prove it owns",
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
        self.block(
            core,
            work_id,
            "an execution was reserved but its launch was never recorded",
            Some(evidence),
        )?;
        Ok(ReconcileDisposition::Ambiguous)
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
        let next = self.drive(core, &work_id, observed, &mut deferred)?;
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
        let next = self.drive(core, &work_id, observed, &mut deferred)?;
        Ok(Step { next, deferred })
    }

    /// Why a delivery's result may no longer be applied, or `None` if it may.
    ///
    /// [`Engine::reservation_is_stale`]'s rule, for the verb that has no
    /// reservation: the Work still exists, it is still `active` (the
    /// `work.resumed` this delivery was committed with put it there), and the
    /// run's current execution is still the one the input was handed to.
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
        match registry
            .runs
            .get(&pending.work_id)
            .and_then(|r| r.execution.as_ref())
        {
            Some(current) if current.execution_id == pending.execution.execution_id => None,
            Some(current) => Some(format!(
                "execution {} was superseded by {}",
                pending.execution.execution_id, current.execution_id
            )),
            None => Some("the run no longer records an execution".to_string()),
        }
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

    /// Tear the surface down and journal the report (never silently).
    fn tear_down_surface(&self, core: &mut Core, work_id: &str) -> Result<(), EngineError> {
        self.tear_down_surface_marked(core, work_id, false)
            .map(|_| ())
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
    pub fn reconcile_terminal_surface(
        &self,
        core: &mut Core,
        work_id: &str,
    ) -> Result<bool, EngineError> {
        self.tear_down_surface_marked(core, work_id, true)
    }

    /// The one teardown path: both callers above are this, differing only in
    /// whether the record says a restart is what wrote it.
    ///
    /// Idempotent through the projection: a run that already has a teardown
    /// report is left alone, so re-running teardown (the crash window, a
    /// second restart) appends nothing and touches no repository.
    fn tear_down_surface_marked(
        &self,
        core: &mut Core,
        work_id: &str,
        recovered: bool,
    ) -> Result<bool, EngineError> {
        let Ok(run) = self.run(core, work_id) else {
            return Ok(false);
        };
        let Some(surface) = run.surface.clone() else {
            return Ok(false);
        };
        if run.teardown.is_some() {
            return Ok(false); // already retired
        }
        let report = teardown(&surface);
        let mut payload = json!({"report": report});
        if recovered {
            payload["recovered"] = Value::Bool(true);
        }
        self.commit(core, work_id, KIND_SURFACE_TORN_DOWN, payload)?;
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
        core.registry
            .state()
            .works
            .get(work_id)
            .map(|w| w.state)
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
}
