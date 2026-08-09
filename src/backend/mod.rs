//! The backend contract (proposal §15): sergeant's one extension boundary.
//!
//! §15 is deliberately not a plugin framework — it is a small native-runtime
//! contract. M3 shipped `probe`, `start`, `send`, `observe`, `stop`; M4's
//! real adapters demanded `interrupt`, `resume` and `history` and they
//! arrived here with their first implementations. `subscribe` is still
//! absent (R1): the Claude adapter ingests each turn's stdout as it streams
//! and pushes normalized events through an [`EventSink`], so nothing needs a
//! pull-based push surface on the trait — measured, not assumed (the M4
//! spike drove real turns through per-turn stdout ingestion alone).
//!
//! The contract's load-bearing shape is [`Observation`]: a backend reports
//! *native evidence* ([`NativeState`]) and, separately, any *explicit signal*
//! ([`BackendSignal`]) about the stage. The engine acts only on the signal.
//! That separation is §25's "work versus execution" rendered as a type — a
//! backend cannot complete a stage by exiting, and cannot fail one by dying.

pub mod claude;
/// Descoped per deviation D6 — see the module's own doc comment.
pub mod codex;
pub mod fake;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::event::EventDraft;
use crate::domain::profile::Profile;

/// One normalized native event (§20/§27): an adapter's translation of a raw
/// vendor record into sergeant's `conversation.*`/`tool.*`/`usage.*`
/// vocabulary. The raw record itself is archived separately (blob store) so
/// vendor fidelity is never lost to this normalization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NativeEvent {
    /// Dotted normalized kind (`conversation.assistant.completed`, ...).
    pub kind: String,
    /// Kind-specific payload, straight from the adapter's normalization.
    pub payload: Value,
}

/// Where an adapter pushes normalized events as they stream (§27).
///
/// Adapters cannot journal directly — the journal is single-owner behind the
/// daemon's core lock — so the daemon hands them a sink that commits on their
/// behalf. Delivery is asynchronous and a sink must never block or fail its
/// caller: adapters emit from wherever their work happens, including the
/// request path that already holds that lock (see `daemon::journaling_sink`).
/// A backend with no sink installed still accumulates its events for
/// [`Backend::history`]; the sink is delivery, not the source of truth.
pub type EventSink = Arc<dyn Fn(EventDraft) + Send + Sync>;

/// What a backend can do (§15's capability list). Absent means `unsupported`,
/// never emulated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Capabilities {
    /// Sessions outlive a single turn.
    pub persistent_sessions: bool,
    /// The harness can run work in its own background.
    pub native_background: bool,
    /// Incremental event streaming.
    pub streaming: bool,
    /// Durable native history retrieval.
    pub history: bool,
    /// Resuming an existing context.
    pub resume: bool,
    /// Interrupting the current turn.
    pub interrupt: bool,
    /// Selecting a model per execution.
    pub model_selection: bool,
    /// Named launch profiles (§14).
    pub profiles: bool,
    /// A human approval flow.
    pub approval_flow: bool,
    /// A human can attach to the live context.
    pub human_attach: bool,
    /// Token/cost usage reporting.
    pub usage: bool,
    /// The harness spawns its own subagents.
    pub native_subagents: bool,
}

/// Answer to §15's PROBE: can this backend operate here, and why not?
///
/// §15's PROBE also reports the harness version. It is not here, for the same
/// reason `subscribe`/`history`/`interrupt`/`resume` are not on the trait:
/// nothing in M3 reads a version, so its shape would be chosen without
/// evidence. `sgt doctor` and the real backends arrive together in M4 and M6;
/// the field arrives with its first reader.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbeReport {
    /// Whether the backend can operate in this environment right now.
    pub available: bool,
    /// Human-readable detail — especially why an unavailable backend is
    /// unavailable, which §13 has to put in front of the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Everything a backend needs to START one execution for one stage attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartRequest {
    /// Work this execution serves.
    pub work_id: String,
    /// Sergeant's id for the execution being started.
    pub execution_id: String,
    /// Stage being executed.
    pub stage_id: String,
    /// 1-based attempt number for that stage.
    pub attempt: u32,
    /// Directory the execution runs in: the work surface (a git worktree for
    /// single-repo work, the surface root for multi-repo).
    pub cwd: PathBuf,
    /// The human intent the Work carries.
    pub intent: String,
    /// The stage's `CONTEXT.md`, verbatim (§12: procedure is data — sergeant
    /// carries it to the actor and does not interpret it).
    pub context: String,
    /// Model to use, when the profile or request selected one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Launch profile to apply (§14: launch configuration, never credentials).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<Profile>,
}

/// Everything a backend needs to RESUME an execution it no longer remembers
/// (§15 RESUME after a daemon restart).
///
/// A restarted adapter has lost the launch configuration it pinned at START
/// along with the process that held it, and nothing durable in the *native*
/// harness records it. So the caller re-supplies it from what sergeant
/// journaled, and an adapter must use exactly this and invent nothing: a
/// fabricated permission mode or a dropped model pin would be the adapter
/// making a decision — about security, about cost — that belongs to the
/// human who configured the work.
///
/// The corollary for callers: a model pin that is not re-supplied here is
/// *not enforced* on later turns, and the adapter will report those turns as
/// unpinned rather than pretend otherwise.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumeRequest {
    /// Work the execution serves — normalized events carry it (§27).
    pub work_id: String,
    /// The work surface later turns run in.
    pub cwd: PathBuf,
    /// The model pin the work requested, re-supplied from the journal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// The launch profile the execution started under (§14).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<Profile>,
}

impl ResumeRequest {
    /// A resume request carrying only the two things every caller has: the
    /// work and its surface. Model and profile default to "not re-supplied",
    /// which adapters must treat as absent, never as a default.
    pub fn new(work_id: impl Into<String>, cwd: impl Into<PathBuf>) -> Self {
        Self {
            work_id: work_id.into(),
            cwd: cwd.into(),
            model: None,
            profile: None,
        }
    }
}

/// Handle to a started execution, as the backend names it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionHandle {
    /// Sergeant's execution id (stable across restarts).
    pub execution_id: String,
    /// The backend's native identity for the context, when it has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_id: Option<String>,
}

/// Native evidence about the execution context itself (§15 OBSERVE).
///
/// This is *evidence*, not state: nothing in the engine may transition Work or
/// stage state from a value of this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeState {
    /// The native context is alive.
    Running,
    /// The native context has exited.
    Exited,
    /// The backend cannot tell. §25: ambiguity fails closed.
    Unknown,
}

impl NativeState {
    /// The state's canonical snake_case name.
    pub fn as_str(self) -> &'static str {
        match self {
            NativeState::Running => "running",
            NativeState::Exited => "exited",
            NativeState::Unknown => "unknown",
        }
    }
}

/// An explicit signal from the backend about the stage it is executing.
///
/// Every variant is something the backend *said*. There is deliberately no
/// variant meaning "the process ended, draw your own conclusion".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "signal", rename_all = "snake_case")]
pub enum BackendSignal {
    /// Still working. No stage or work transition follows.
    Running,
    /// The stage is done (§12's explicit completion).
    StageCompleted {
        /// Optional summary of what the stage produced.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        summary: Option<String>,
    },
    /// The stage needs a human answer before it can continue.
    NeedsInput {
        /// What the actor is asking.
        prompt: String,
    },
    /// The stage is waiting on an external condition.
    Waiting {
        /// What it is waiting for.
        reason: String,
    },
    /// The stage is blocked on a decision or gate.
    Blocked {
        /// Why.
        reason: String,
    },
    /// The stage failed.
    Failed {
        /// Why.
        reason: String,
    },
}

impl BackendSignal {
    /// Short name for journaling and diagnostics.
    pub fn as_str(&self) -> &'static str {
        match self {
            BackendSignal::Running => "running",
            BackendSignal::StageCompleted { .. } => "stage_completed",
            BackendSignal::NeedsInput { .. } => "needs_input",
            BackendSignal::Waiting { .. } => "waiting",
            BackendSignal::Blocked { .. } => "blocked",
            BackendSignal::Failed { .. } => "failed",
        }
    }
}

/// One OBSERVE result: native evidence plus any explicit signal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    /// Native evidence about the execution context.
    pub native: NativeState,
    /// The backend's explicit signal about the stage.
    pub signal: BackendSignal,
    /// Free-form evidence recorded with recovery decisions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
}

/// Failure from a backend operation.
#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    /// The backend does not recognise this execution — e.g. after a restart
    /// where the native context did not survive. Ambiguous by §25.
    #[error("backend {backend:?} does not recognise execution {execution_id}")]
    UnknownExecution {
        /// Backend name.
        backend: String,
        /// The execution id it did not recognise.
        execution_id: String,
    },
    /// The backend cannot operate here at all.
    #[error("backend {backend:?} is unavailable: {detail}")]
    Unavailable {
        /// Backend name.
        backend: String,
        /// Why.
        detail: String,
    },
    /// Any other backend-specific failure.
    #[error("backend {backend:?} failed: {detail}")]
    Failed {
        /// Backend name.
        backend: String,
        /// Why.
        detail: String,
    },
}

/// The §15 contract, M3 subset.
///
/// Methods are synchronous: the only M3 implementation is in-process, and a
/// `dyn`-compatible async trait would need a boxing dependency for no measured
/// benefit. M4's process-driving backends can block on their own runtime
/// behind this same surface.
pub trait Backend: Send + Sync + std::fmt::Debug {
    /// Backend name, as used in routing and `--backend`.
    fn name(&self) -> &str;

    /// Capabilities this backend advertises (§15).
    fn capabilities(&self) -> Capabilities;

    /// PROBE: can this backend operate here, and at what version?
    fn probe(&self) -> ProbeReport;

    /// START: create one native execution context.
    fn start(&self, request: &StartRequest) -> Result<ExecutionHandle, BackendError>;

    /// SEND: deliver input to an execution context.
    fn send(&self, handle: &ExecutionHandle, input: &str) -> Result<(), BackendError>;

    /// OBSERVE: report current native evidence and any explicit signal.
    fn observe(&self, handle: &ExecutionHandle) -> Result<Observation, BackendError>;

    /// INTERRUPT: stop the current turn/action without retiring the
    /// execution. For a print-mode adapter this kills the per-turn process;
    /// the durable conversation survives and RESUME/SEND continue it
    /// (measured against Claude Code 2.1.226: a SIGKILLed turn leaves the
    /// session resumable with full recall). Interrupting an execution with
    /// no turn in flight is a no-op, not an error: the goal state — no turn
    /// running — already holds.
    fn interrupt(&self, handle: &ExecutionHandle) -> Result<(), BackendError>;

    /// RESUME: re-adopt an existing native context, e.g. after a daemon
    /// restart, so later SENDs continue the same conversation. The
    /// [`ResumeRequest`] carries the launch configuration the adapter lost
    /// with the old daemon; an adapter uses that and fabricates nothing.
    /// Fails closed when the native context cannot be evidenced.
    fn resume(&self, handle: &ExecutionHandle, request: &ResumeRequest)
    -> Result<(), BackendError>;

    /// HISTORY: the normalized native events this adapter has accumulated
    /// for an execution (§27), in order. This is retrieval, not delivery —
    /// events pushed through an [`EventSink`] appear here too.
    fn history(&self, handle: &ExecutionHandle) -> Result<Vec<NativeEvent>, BackendError>;

    /// STOP: retire the execution without corrupting recoverable state. A
    /// backend reports that it *asked*; whether the native context complied is
    /// only knowable through OBSERVE.
    fn stop(&self, handle: &ExecutionHandle) -> Result<(), BackendError>;
}

/// The set of backends this daemon can route to.
///
/// Compiled-in rather than configured: M3 has exactly one backend to register,
/// and a configuration format for a one-element list would be machinery ahead
/// of its need. M4 adds real backends to the same registry.
#[derive(Debug, Default, Clone)]
pub struct BackendRegistry {
    backends: BTreeMap<String, Arc<dyn Backend>>,
}

impl BackendRegistry {
    /// An empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a backend under its own name.
    pub fn with(mut self, backend: Arc<dyn Backend>) -> Self {
        self.backends.insert(backend.name().to_string(), backend);
        self
    }

    /// The backend registered under `name`, if any.
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Backend>> {
        self.backends.get(name)
    }

    /// Registered names, sorted.
    pub fn names(&self) -> Vec<String> {
        self.backends.keys().cloned().collect()
    }

    /// The default registry for a daemon: the deterministic fake. The daemon
    /// adds the real adapters itself at startup (`daemon::start_with`),
    /// because they need the data dir and an event sink that only exist
    /// there.
    pub fn default_registry() -> Self {
        Self::new().with(Arc::new(fake::FakeBackend::new(fake::FAKE_BACKEND_NAME)))
    }
}
