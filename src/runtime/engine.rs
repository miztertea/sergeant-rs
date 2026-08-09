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
    Backend, BackendError, BackendRegistry, BackendSignal, ExecutionHandle, NativeState,
    Observation, StartRequest,
};
use crate::domain::event::{EventDraft, EventSource};
use crate::domain::execution::{
    ExecutionRecord, KIND_EXECUTION_RECONCILED, KIND_EXECUTION_STARTED, KIND_EXECUTION_STOPPED,
    ReconcileDisposition,
};
use crate::domain::profile::Profile;
use crate::domain::work::{
    KIND_WORK_BLOCKED, KIND_WORK_COMPLETED, KIND_WORK_FAILED, KIND_WORK_NEEDS_INPUT,
    KIND_WORK_RESUMED, KIND_WORK_STARTED, KIND_WORK_WAITING, Work, WorkState,
};
use crate::domain::workflow::{
    DEFAULT_WORKFLOW, KIND_STAGE_BLOCKED, KIND_STAGE_CANCELED, KIND_STAGE_COMPLETED,
    KIND_STAGE_ENTERED, KIND_STAGE_FAILED, KIND_STAGE_INPUT_RECEIVED, KIND_STAGE_NEEDS_INPUT,
    KIND_STAGE_WAITING, KIND_WORKFLOW_BOUND, StageStatus, WorkflowDefinition, WorkflowError,
};
use crate::domain::workspace::{RepositorySpec, Workspace, WorkspaceError};
use crate::runtime::projection::WorkRun;
use crate::runtime::router::{Route, RouteError, RouteInputs, route};
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
                let profile = workspace.profile(name).cloned().ok_or_else(|| {
                    EngineError::ProfileNotFound {
                        requested: name.to_string(),
                        available: workspace
                            .profiles
                            .iter()
                            .map(|p| p.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", "),
                    }
                })?;
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

        Ok(Some(StartPlan {
            workspace,
            repositories,
            workflow,
            route,
            profile,
        }))
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
                return Ok(());
            }
            Err(e) => {
                self.block(
                    core,
                    &work.id,
                    &format!("cannot materialize work surface: {e}"),
                    None,
                )?;
                return Ok(());
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
            }),
        )?;
        self.transition(
            core,
            &work.id,
            KIND_WORK_STARTED,
            json!({"backend": plan.route.backend, "workflow": plan.workflow.name}),
        )?;
        if self.enter_stage(core, &work.id, 0, 1)? {
            self.resume(core, &work.id)?;
        }
        Ok(())
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
        if let Err(e) = backend.send(&handle_of(&execution), input) {
            self.commit(
                core,
                work_id,
                KIND_STAGE_BLOCKED,
                json!({"stage_id": stage_id, "detail": e.to_string()}),
            )?;
            self.block(core, work_id, &format!("cannot deliver input: {e}"), None)?;
            return Ok(());
        }
        self.resume(core, work_id)
    }

    /// Re-enter the current stage (§12's retry verb).
    ///
    /// Retry is the one door back out of `failed`, `blocked` and `waiting`,
    /// and it is always explicit — nothing retries itself. The surface is
    /// re-attached to the branch teardown retained, so a retry continues on
    /// the work the failed attempt left behind.
    pub fn retry(&self, core: &mut Core, work_id: &str) -> Result<(), EngineError> {
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
        if self.enter_stage(core, work_id, current.index, current.attempt + 1)? {
            self.resume(core, work_id)?;
        }
        Ok(())
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
        let Ok(run) = self.run(core, work_id) else {
            return Ok(()); // nothing ever started
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
        self.stop_execution(core, work_id, reason)?;
        self.tear_down_surface(core, work_id)
    }

    /// Drive a run forward from whatever its backend now says.
    ///
    /// This is the only place stage progression happens, and the only signals
    /// it acts on are the backend's explicit ones. It loops because completing
    /// a stage enters the next one, which may itself already have something to
    /// say; every iteration either returns or advances to a later stage, so
    /// the loop is bounded by the workflow's stage count.
    pub fn resume(&self, core: &mut Core, work_id: &str) -> Result<(), EngineError> {
        self.drive(core, work_id, None)
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
        initial: Option<Observation>,
    ) -> Result<(), EngineError> {
        let mut pending = initial;
        loop {
            let run = self.run(core, work_id)?;
            let Some(execution) = run.execution.clone() else {
                return Ok(());
            };
            let Some(stage) = run.current_stage().cloned() else {
                return Ok(());
            };
            let workflow = run.workflow.clone().ok_or_else(|| EngineError::NoRun {
                work_id: work_id.to_string(),
            })?;
            let backend = self.backend_for(work_id, &execution.backend)?;

            let observation = match pending.take() {
                Some(observation) => observation,
                None => match backend.observe(&handle_of(&execution)) {
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
                        return Ok(());
                    }
                },
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
                return Ok(());
            }

            // From here on, only `observation.signal` is consulted. Native
            // liveness has already had its one and only say (Unknown above).
            match observation.signal {
                BackendSignal::Running => return Ok(()),
                BackendSignal::NeedsInput { prompt } => {
                    self.commit(
                        core,
                        work_id,
                        KIND_STAGE_NEEDS_INPUT,
                        json!({"stage_id": stage.stage_id, "detail": prompt}),
                    )?;
                    self.transition(
                        core,
                        work_id,
                        KIND_WORK_NEEDS_INPUT,
                        json!({"prompt": prompt, "stage_id": stage.stage_id}),
                    )?;
                    return Ok(());
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
                    return Ok(());
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
                    return Ok(());
                }
                BackendSignal::Failed { reason } => {
                    self.commit(
                        core,
                        work_id,
                        KIND_STAGE_FAILED,
                        json!({"stage_id": stage.stage_id, "detail": reason}),
                    )?;
                    self.stop_execution(core, work_id, "stage failed")?;
                    self.transition(
                        core,
                        work_id,
                        KIND_WORK_FAILED,
                        json!({"reason": reason, "stage_id": stage.stage_id}),
                    )?;
                    self.tear_down_surface(core, work_id)?;
                    return Ok(());
                }
                BackendSignal::StageCompleted { summary } => {
                    let mut payload = json!({"stage_id": stage.stage_id, "index": stage.index});
                    if let Some(summary) = summary {
                        payload["detail"] = Value::String(summary);
                    }
                    self.commit(core, work_id, KIND_STAGE_COMPLETED, payload)?;
                    self.stop_execution(core, work_id, "stage completed")?;
                    let next = stage.index + 1;
                    if next < workflow.stages.len() {
                        if !self.enter_stage(core, work_id, next, 1)? {
                            return Ok(());
                        }
                        continue;
                    }
                    self.transition(
                        core,
                        work_id,
                        KIND_WORK_COMPLETED,
                        json!({"stages": workflow.stages.len()}),
                    )?;
                    self.tear_down_surface(core, work_id)?;
                    return Ok(());
                }
            }
        }
    }

    /// Re-observe one in-flight execution after a daemon restart (§25).
    ///
    /// Returns the disposition it recorded. Unambiguous evidence resumes the
    /// run from wherever the backend now is; anything the adapter cannot
    /// classify — an unrecognised execution, an unreachable backend, an
    /// unknown native state — lands the work in `blocked` with the evidence,
    /// because §25's rule is that ambiguity fails closed.
    pub fn reconcile_work(
        &self,
        core: &mut Core,
        work_id: &str,
    ) -> Result<ReconcileDisposition, EngineError> {
        let run = self.run(core, work_id)?;
        let Some(execution) = run.execution.clone() else {
            self.record_reconcile(
                core,
                work_id,
                None,
                ReconcileDisposition::Ambiguous,
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
        let (disposition, evidence) = match self.backends.get(&execution.backend) {
            None => ambiguous(format!(
                "backend {:?} is not registered in this daemon",
                execution.backend
            )),
            Some(backend) => match backend.observe(&handle_of(&execution)) {
                Err(e) => ambiguous(e.to_string()),
                Ok(Observation {
                    native: NativeState::Unknown,
                    evidence,
                    ..
                }) => ambiguous(
                    evidence.unwrap_or_else(|| "backend reports unknown native state".to_string()),
                ),
                Ok(observation) => {
                    let evidence = format!(
                        "native={}, signal={}",
                        observation.native.as_str(),
                        observation.signal.as_str()
                    );
                    resumed_from = Some(observation);
                    (ReconcileDisposition::Resumed, evidence)
                }
            },
        };
        self.record_reconcile(core, work_id, Some(&execution), disposition, &evidence)?;
        match disposition {
            ReconcileDisposition::Resumed => {
                self.drive(core, work_id, resumed_from)?;
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
                // work-level block, and idempotent via `stop_requested`, so
                // a crash anywhere in this append sequence re-derives on
                // the next restart (L6: the work is still `active` until
                // the final append, and reconcile re-runs on `active`).
                self.stop_execution(core, work_id, "retired at reconcile: unrecognized")?;
                self.block(
                    core,
                    work_id,
                    "execution could not be reconciled after restart",
                    Some(evidence),
                )?;
            }
        }
        Ok(disposition)
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

    /// Enter a stage: journal the entry, then START an execution for it.
    /// Returns whether the run can be driven (a start failure blocks instead).
    fn enter_stage(
        &self,
        core: &mut Core,
        work_id: &str,
        index: usize,
        attempt: u32,
    ) -> Result<bool, EngineError> {
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
        let backend_name = run.backend.clone().ok_or_else(|| EngineError::NoRun {
            work_id: work_id.to_string(),
        })?;
        let backend = self.backend_for(work_id, &backend_name)?;
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
            model: run.profile.as_ref().and_then(|p| p.default_model.clone()),
            profile: run.profile.clone(),
        };
        let handle = match backend.start(&request) {
            Ok(handle) => handle,
            Err(e) => {
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
                return Ok(false);
            }
        };
        let record = ExecutionRecord {
            execution_id,
            backend: backend_name,
            native_id: handle.native_id.clone(),
            stage_id: stage.id.clone(),
            attempt,
            stop_requested: false,
        };
        self.commit(
            core,
            work_id,
            KIND_EXECUTION_STARTED,
            json!({"execution": record}),
        )?;
        Ok(true)
    }

    /// Ask the current execution to retire, and journal that we asked.
    ///
    /// A stop *request* is the whole truth available to sergeant: whether the
    /// native context complied is only knowable through OBSERVE, and is never
    /// assumed here.
    fn stop_execution(
        &self,
        core: &mut Core,
        work_id: &str,
        reason: &str,
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
                Ok(()) => json!({"requested": true}),
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
        let Ok(run) = self.run(core, work_id) else {
            return Ok(());
        };
        let Some(surface) = run.surface.clone() else {
            return Ok(());
        };
        if run.teardown.is_some() {
            return Ok(()); // already retired
        }
        let report = teardown(&surface);
        self.commit(
            core,
            work_id,
            KIND_SURFACE_TORN_DOWN,
            json!({"report": report}),
        )?;
        Ok(())
    }

    fn record_reconcile(
        &self,
        core: &mut Core,
        work_id: &str,
        execution: Option<&ExecutionRecord>,
        disposition: ReconcileDisposition,
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
