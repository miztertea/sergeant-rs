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
//! **§17 runtime scope, and why ENSURE RUNTIME is not a verb here (R1).**
//! §17 forbids the core from assuming any one daemon model, so every adapter
//! declares its [`RuntimeScope`] ([`Backend::runtime_scope`]) and the daemon
//! journals it with the probe. §15's ENSURE RUNTIME — "start or attach to any
//! backend-level service required" — is deliberately *absent* from this
//! trait, and this is the rung log for that absence rather than a silence:
//! both adapters that exist declare `per_execution`, which is precisely the
//! scope with no backend-level service to start. The Claude adapter's
//! "runtime" is the installed CLI, and the only thing anyone can ensure about
//! it — that it is present, recent enough, and speaks the launch grammar this
//! adapter measured — is what PROBE already does and journals. A verb whose
//! every implementation would be `Ok(())` would be machinery ahead of its
//! evidence; it arrives with the first adapter whose scope is `external`,
//! `per_profile` or `per_workspace` and which therefore has a service to
//! start (§16's OpenCode server and Prime daemon are the shapes that will
//! force it), together with the failure modes only such a backend can show.
//!
//! The contract's load-bearing shape is [`Observation`]: a backend reports
//! *native evidence* ([`NativeState`]) and, separately, any *explicit signal*
//! ([`BackendSignal`]) about the stage. The engine acts only on the signal.
//! That separation is §25's "work versus execution" rendered as a type — a
//! backend cannot complete a stage by exiting, and cannot fail one by dying.

pub mod claude;
/// Descoped per deviation D6 — see the module's own doc comment.
pub mod codex;
pub mod docker;
pub mod fake;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::estate::InstructionPolicy;
use crate::domain::event::EventDraft;
use crate::domain::profile::Profile;
use crate::domain::workflow::ExecuteSpec;

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
///
/// The sink is where sergeant's *durable* record of normalized events comes
/// from: what reaches the journal is what survives a restart. It is not the
/// same surface as [`Backend::history`], which is the backend's own retrieval
/// of native history and exists only where an adapter can honestly serve it
/// (see [`Capabilities::history`]).
pub type EventSink = Arc<dyn Fn(EventDraft) + Send + Sync>;

/// §17: the runtime model an adapter needs, declared rather than assumed.
///
/// §17's rule is that the core "must not assume one backend daemon per worker
/// or one global daemon per backend"; adapters declare which of these four
/// shapes they are, and sergeant records the declaration with the probe. It
/// is evidence about the adapter, not a capability toggle: nothing in the
/// engine branches on it yet, and the value is journaled precisely so that
/// the first thing which *does* need to (a supervisor for backend-level
/// services) is written against recorded declarations instead of guesses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeScope {
    /// The runtime exists outside sergeant's lifecycle entirely; sergeant
    /// attaches to it and never owns it.
    External,
    /// One runtime instance per launch profile (§14).
    PerProfile,
    /// One runtime instance per estate.
    PerWorkspace,
    /// Each execution owns its own native runtime; there is no shared
    /// backend-level service to start or attach to.
    PerExecution,
}

impl RuntimeScope {
    /// The scope's canonical snake_case name.
    pub fn as_str(self) -> &'static str {
        match self {
            RuntimeScope::External => "external",
            RuntimeScope::PerProfile => "per_profile",
            RuntimeScope::PerWorkspace => "per_workspace",
            RuntimeScope::PerExecution => "per_execution",
        }
    }
}

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
    /// Durable native history retrieval: [`Backend::history`] answers with
    /// this execution's whole normalized history, or refuses — never with a
    /// partial list a caller could read as the whole thing. An adapter that
    /// can only report what its own process happened to see advertises
    /// `false` here and refuses (§15: unsupported means unsupported, not
    /// emulation), and [`Backend::history`] enforces exactly that pairing.
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
    /// **The actor can author a question that parks its own stage**
    /// (proposal §11.1's "observe explicit semantic signals", sharpened by
    /// N2's GP-2 finding). `true` claims something narrow and checkable: this
    /// adapter can tell, from the harness's own output, that the *actor* — not
    /// the adapter, not a policy gate — asked the human a question and is
    /// waiting for the answer, and it reports that as
    /// [`BackendSignal::NeedsInput`] with [`AskAuthor::Actor`].
    ///
    /// An adapter whose harness offers no such record advertises `false` and
    /// never guesses one from prose: "the last thing it said ended in a
    /// question mark" is a heuristic, and a heuristic that parks a stage is an
    /// adapter inventing a checkpoint. L8 binds the flag to a contract test
    /// against the installed harness.
    pub ask: bool,
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

/// One selected repository, as the backend needs to see it (§10.1).
///
/// §10.1: "the backend receives the complete binding summary, not only a cwd,
/// so its prompt/launch grammar can state exact repository paths, expected
/// branches, and base SHAs". A `cwd` alone cannot say any of that — for a
/// multi-repo Work it is the surface *root*, and even for a single-repo Work
/// it names a directory without naming the branch the actor is expected to be
/// on or the commit it started from.
///
/// Every field is a fact sergeant already journaled about the Work's own
/// [`RepositoryBinding`](crate::runtime::surface::RepositoryBinding). Nothing
/// here is policy or instruction: what an adapter *does* with it is the
/// adapter's business, and an adapter that ignores the field entirely still
/// runs (C2 — core reports only what it can prove).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingSummary {
    /// Repository name within the Work's scope.
    pub repository: String,
    /// The linked worktree this Work may modify — the mutation surface.
    pub worktree_path: PathBuf,
    /// Branch the Work executes on in that worktree.
    pub work_branch: String,
    /// Branch the work branch was cut from (`(detached)` if HEAD was).
    pub base_branch: String,
    /// Exact commit it was cut from.
    pub base_sha: String,
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
    /// The pinned container spec, present exactly when the stage this
    /// request serves is `kind = "execute"` (N4, §12.3, §13.1). `None` for
    /// every actor stage. This is how the two-phase executor lifecycle
    /// (§14.2) — unmodified from what the Claude adapter already exercises —
    /// carries Docker's stage-authored contract from `bind_stages`' pinned
    /// [`crate::domain::workflow::StageDefinition`] through PREPARE/LAUNCH
    /// without the engine knowing what an image or a container is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execute: Option<ExecuteSpec>,
    /// R-MVP1-4's resolved instruction-file policy (MVP-2 D2 item 1, the
    /// `--setting-sources` de-leak). `check_instruction_policy` at submit
    /// already refused any selected-repository set that disagrees, so this
    /// is uniform across the Work by construction — one value, not one per
    /// repository. `#[serde(default)]` so a `StartRequest` journaled before
    /// this field existed replays as [`InstructionPolicy::Suppress`], which
    /// is exactly what every such request's actual launch grammar was.
    #[serde(default)]
    pub instruction_policy: InstructionPolicy,
    /// §10.1's complete binding summary: one entry per repository in the
    /// Work's scope, in the Work's own scope order.
    ///
    /// `#[serde(default)]` to an empty vec, so a `StartRequest` journaled
    /// before this field existed replays as "no summary supplied" — which is
    /// exactly what it was. An empty summary is not a claim that the Work has
    /// no repositories; it is the absence of a claim, and an adapter must not
    /// read it as a mutation surface of nothing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<BindingSummary>,
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
    /// R-MVP1-4's resolved instruction-file policy, re-supplied on RESUME for
    /// the same reason the model pin and profile are (MVP-2 D2 item 1): it
    /// is a decision about the Work, pinned at bind, and a restarted
    /// adapter's later turns must launch under the same grammar the run
    /// started with, not the adapter's compiled-in default.
    ///
    /// `Option`, like `model` and `profile` above (INV-R1-13, MVP-2 D3
    /// fixer pass): `None` is the type's own "not re-supplied" state an
    /// adapter must treat as absent, never invented. Before this fix the
    /// field was a bare `InstructionPolicy`, which has no such
    /// representation — every caller silently supplied a real value
    /// (`Suppress`) whether or not it actually had one to re-supply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instruction_policy: Option<InstructionPolicy>,
    /// §10.1's binding summary, re-supplied on RESUME for the same reason the
    /// model pin and the profile are: a restarted adapter has lost whatever it
    /// derived from it, and re-deriving the Work's mutation surface from a
    /// bare `cwd` would be the fabrication this type's contract forbids.
    ///
    /// `#[serde(default)]` to empty, exactly as on
    /// [`StartRequest::bindings`], and read the same way — absence of a
    /// claim, not a claim of absence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<BindingSummary>,
}

impl ResumeRequest {
    /// A resume request carrying only the two things every caller has: the
    /// work and its surface. Model, profile and instruction policy all
    /// default to `None` — "not re-supplied" — which adapters must treat as
    /// absent, never invented; an adapter that needs *some* concrete policy
    /// to launch under falls back to its own safe default (today,
    /// `InstructionPolicy::default()` = `Suppress`) only at that point, not
    /// here.
    pub fn new(work_id: impl Into<String>, cwd: impl Into<PathBuf>) -> Self {
        Self {
            work_id: work_id.into(),
            cwd: cwd.into(),
            model: None,
            profile: None,
            instruction_policy: None,
            bindings: Vec::new(),
        }
    }
}

/// Who authored a [`BackendSignal::NeedsInput`] question.
///
/// The distinction is the whole content of GP-2: an adapter asking (a
/// permission gate, a policy stop, a harness that needs a credential) and the
/// *actor* asking (the reasoning agent decided it cannot proceed without a
/// human decision) park the same stage in the same state, but they are
/// different facts about the run, and only the second one is the checkpoint a
/// workflow author is designing for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AskAuthor {
    /// The adapter or the harness itself is asking. The default, because it
    /// is what a journaled payload written before this field existed meant:
    /// nothing had measured actor authorship, so nothing may be claimed.
    #[default]
    Adapter,
    /// The running actor authored the question (§11.1's explicit semantic
    /// signal). Only an adapter advertising [`Capabilities::ask`] may report
    /// this, and only from a structured record of the harness's, never from
    /// prose.
    Actor,
}

impl AskAuthor {
    /// The author's canonical snake_case name.
    pub fn as_str(self) -> &'static str {
        match self {
            AskAuthor::Adapter => "adapter",
            AskAuthor::Actor => "actor",
        }
    }
}

/// Work an adapter must finish *after* a verb returns, handed to the caller
/// so it can be awaited somewhere the caller chooses (§14.3, issue #14/B3).
///
/// STOP has to keep two promises that pull against each other: the execution's
/// evidence is durable before the operation is over, and the daemon's core
/// lock is not held while a thread is joined (§22.6). A verb that joins
/// internally can only keep the first. So the verb does its authoritative part
/// — kill the process, latch the refusal — and hands back the tail as a
/// value; the engine collects those into a [`Deferred`] and the daemon waits
/// on it after releasing the core guard.
///
/// Dropping one detaches the tail rather than waiting for it, which is why the
/// type is `#[must_use]`: an ignored completion is precisely the bug B3
/// records, wearing a different hat.
#[must_use = "an unawaited Completion detaches the adapter's tail work (issue #14/B3): \
              collect it into a Deferred and wait outside the core lock"]
#[derive(Default)]
pub struct Completion {
    wait: Option<Box<dyn FnOnce() + Send>>,
}

impl std::fmt::Debug for Completion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Completion")
            .field("pending", &self.wait.is_some())
            .finish()
    }
}

impl Completion {
    /// Nothing left to wait for: the verb finished everything it promised.
    pub fn immediate() -> Self {
        Self { wait: None }
    }

    /// Tail work the caller must run to make the verb's promise true.
    pub fn deferred(wait: impl FnOnce() + Send + 'static) -> Self {
        Self {
            wait: Some(Box::new(wait)),
        }
    }

    /// Whether anything is still outstanding.
    pub fn is_pending(&self) -> bool {
        self.wait.is_some()
    }

    /// Run the tail work, blocking until it is done. Never call this while
    /// holding the core lock — that is the whole point of the type.
    pub fn wait(self) {
        if let Some(wait) = self.wait {
            wait();
        }
    }
}

/// Every [`Completion`] one engine call accumulated, awaited together.
///
/// The engine never waits on a completion itself: a single API request can
/// stop several executions (a cancel that retires a run, a stage failure that
/// stops one execution and blocks the work), and each one's tail belongs
/// outside the same guard. So they collect here and the caller drains the bag
/// once, in whatever place it is safe to block.
#[must_use = "an undrained Deferred detaches every completion it holds (issue #14/B3)"]
#[derive(Debug, Default)]
pub struct Deferred {
    completions: Vec<Completion>,
}

impl Deferred {
    /// An empty bag.
    pub fn new() -> Self {
        Self::default()
    }

    /// Take ownership of one adapter completion.
    pub fn push(&mut self, completion: Completion) {
        if completion.is_pending() {
            self.completions.push(completion);
        }
    }

    /// Absorb another bag (an inner engine step's).
    pub fn absorb(&mut self, mut other: Deferred) {
        self.completions
            .extend(std::mem::take(&mut other.completions));
    }

    /// Whether anything is outstanding.
    pub fn is_pending(&self) -> bool {
        !self.completions.is_empty()
    }

    /// How many completions are outstanding.
    pub fn len(&self) -> usize {
        self.completions.len()
    }

    /// Whether the bag holds nothing.
    pub fn is_empty(&self) -> bool {
        self.completions.is_empty()
    }

    /// Wait for every completion, in the order they were collected.
    pub fn wait(mut self) {
        for completion in std::mem::take(&mut self.completions) {
            completion.wait();
        }
    }
}

/// A bag dropped with completions still outstanding — an early return via
/// `?` between a push and the eventual `.wait()`, not a caller that forgot
/// to drain it (the `#[must_use]` catches that case at compile time). The
/// tail work still has to run somewhere, and the core lock this always
/// drops under (§22.6) forbids blocking here, so it is handed to a detached
/// thread rather than silently discarded — the same failure issue #14/B3
/// closed, reintroduced at an internal error-propagation boundary.
impl Drop for Deferred {
    fn drop(&mut self) {
        let completions = std::mem::take(&mut self.completions);
        if completions.is_empty() {
            return;
        }
        std::thread::spawn(move || {
            for completion in completions {
                completion.wait();
            }
        });
    }
}

/// A reserved-but-not-yet-launched execution (§14.2's first phase, §14.3's
/// `prepare(request) → PreparedHarnessExecution`).
///
/// PREPARE allocates whatever identity the adapter can allocate *before* an
/// external effect exists — for the Claude adapter, the session UUID it used
/// to mint one line before spawning — and performs no external effect of its
/// own. That restriction is what lets the engine journal the reservation
/// under the core lock (§22.6 forbids external I/O there) and still have the
/// native identity in the record before anything can create it: a daemon that
/// dies between the reservation and the launch leaves a journal entry naming
/// exactly the identity that might, or might not, exist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedExecution {
    /// Sergeant's id for the execution being prepared.
    pub execution_id: String,
    /// The native identity reserved for it, when the adapter can name one
    /// before launching. `None` is honest, not a failure: an adapter whose
    /// identity is minted by the external system (a container id) has nothing
    /// to reserve here, and §14.4's deterministic-name answer for that case
    /// arrives with the executor that needs it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_id: Option<String>,
    /// The request this reservation pinned, carried to LAUNCH so the adapter
    /// cannot resolve its configuration twice and get two answers.
    pub request: StartRequest,
}

impl PreparedExecution {
    /// The handle this reservation will name once it is launched.
    pub fn handle(&self) -> ExecutionHandle {
        ExecutionHandle {
            execution_id: self.execution_id.clone(),
            native_id: self.native_id.clone(),
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
        /// Who authored the question (GP-2). `#[serde(default)]` because
        /// payloads journaled before the field existed carry no claim about
        /// authorship, and [`AskAuthor::Adapter`] is exactly "no claim".
        #[serde(default)]
        asked_by: AskAuthor,
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
    /// The stage needs input, asked for by the adapter or the harness itself.
    pub fn needs_input(prompt: impl Into<String>) -> Self {
        BackendSignal::NeedsInput {
            prompt: prompt.into(),
            asked_by: AskAuthor::Adapter,
        }
    }

    /// GP-2's ask: the *actor* authored this question and is waiting.
    /// Only an adapter advertising [`Capabilities::ask`] may build one.
    pub fn ask(prompt: impl Into<String>) -> Self {
        BackendSignal::NeedsInput {
            prompt: prompt.into(),
            asked_by: AskAuthor::Actor,
        }
    }

    /// Who authored this signal's question, when it is one.
    pub fn asked_by(&self) -> Option<AskAuthor> {
        match self {
            BackendSignal::NeedsInput { asked_by, .. } => Some(*asked_by),
            _ => None,
        }
    }

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
    /// The backend does not support this verb, and says so instead of
    /// emulating it (§15: "missing capability means unsupported ... not
    /// emulation"). A refusal a caller can distinguish from an empty answer
    /// is the whole point: `Ok(vec![])` from an adapter that simply cannot
    /// look is indistinguishable from "this conversation produced nothing".
    #[error("backend {backend:?} does not support {verb}: {detail}")]
    Unsupported {
        /// Backend name.
        backend: String,
        /// The §15 verb that is unsupported (`history`, ...).
        verb: String,
        /// What the caller should use instead, or why it is unsupported.
        detail: String,
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

    /// §17: the runtime model this adapter needs. Declared, never assumed by
    /// the core — see the module docs for why ENSURE RUNTIME is not a verb
    /// on this trait yet.
    fn runtime_scope(&self) -> RuntimeScope;

    /// PROBE: can this backend operate here, and at what version?
    fn probe(&self) -> ProbeReport;

    /// PREPARE: reserve one execution's identity, without creating anything
    /// external (§14.3's `prepare(request) → PreparedHarnessExecution`).
    ///
    /// Two rules bind every implementation, and both are what makes the
    /// engine's two-phase boundary honest rather than decorative:
    ///
    /// - **no external effect.** No process, no container, no network, no
    ///   blocking wait. The engine calls this while holding the core lock,
    ///   which §22.6 forbids doing any of those under. What PREPARE may do is
    ///   resolve configuration and allocate identity in adapter memory.
    /// - **the identity is final.** Whatever [`PreparedExecution::native_id`]
    ///   names is what LAUNCH must use. The engine journals the reservation
    ///   *before* calling LAUNCH, precisely so that a crash in between leaves
    ///   a record naming the identity that may now exist unowned.
    ///
    /// A prepared execution that is never launched leaves adapter memory with
    /// the daemon; the journaled reservation is what recovery reads.
    fn prepare(&self, request: &StartRequest) -> Result<PreparedExecution, BackendError>;

    /// LAUNCH: create the native execution context a [`PreparedExecution`]
    /// reserved. This is the external effect, and the engine runs it with the
    /// core lock released.
    ///
    /// A failed launch must leave no phantom behind: the adapter drops
    /// whatever PREPARE registered, so a later OBSERVE of the reserved id is
    /// an honest "unknown execution" rather than a context in a state nothing
    /// created.
    fn launch(&self, prepared: &PreparedExecution) -> Result<ExecutionHandle, BackendError>;

    /// START: create one native execution context — PREPARE then LAUNCH, with
    /// no journal record in between.
    ///
    /// Kept as a verb (§15 names it) and as the single-owner convenience for
    /// callers holding no shared lock. The daemon deliberately does *not* use
    /// it: it journals `execution.reserved` between the two halves, which is
    /// the whole point of the split.
    fn start(&self, request: &StartRequest) -> Result<ExecutionHandle, BackendError> {
        let prepared = self.prepare(request)?;
        self.launch(&prepared)
    }

    /// SEND: deliver input to an execution context.
    fn send(&self, handle: &ExecutionHandle, input: &str) -> Result<(), BackendError>;

    /// OBSERVE: report current native evidence and any explicit signal.
    fn observe(&self, handle: &ExecutionHandle) -> Result<Observation, BackendError>;

    /// The liveness-only half of OBSERVE: "is anything alive under this
    /// identity", with none of the evidence-capture side effects a full
    /// OBSERVE may pay to answer the fuller question (INV-R1-08).
    ///
    /// Default implementation just asks the full [`Backend::observe`] and
    /// keeps its `native` field — correct for every backend whose OBSERVE is
    /// already cheap (an in-memory state read, for the fake and Claude
    /// adapters today). A backend whose OBSERVE pays for expensive evidence
    /// capture as a side effect of classifying an exited execution (the
    /// Docker adapter's log capture, §16.9) should override this to skip
    /// that work — callers that only need liveness, like restart
    /// reconciliation's [`Engine::reserved_identity_liveness`], must never
    /// pay for evidence they are about to discard.
    fn observe_liveness(&self, handle: &ExecutionHandle) -> Result<NativeState, BackendError> {
        self.observe(handle).map(|observation| observation.native)
    }

    /// INTERRUPT: stop the current turn/action without retiring the
    /// execution. For a print-mode adapter this kills the per-turn process;
    /// the durable conversation survives and RESUME/SEND continue it
    /// (measured against Claude Code 2.1.226: a SIGKILLed turn leaves the
    /// session resumable with full recall). Interrupting an execution with
    /// no turn in flight is a no-op, not an error: the goal state — no turn
    /// running — already holds.
    ///
    /// Returns a [`Completion`] for whatever the adapter must still finish
    /// before the interrupt's promise is true (a killed turn's transcript
    /// archive, say). The engine never waits on it under the core lock.
    fn interrupt(&self, handle: &ExecutionHandle) -> Result<Completion, BackendError>;

    /// RESUME: re-adopt an existing native context, e.g. after a daemon
    /// restart, so later SENDs continue the same conversation. The
    /// [`ResumeRequest`] carries the launch configuration the adapter lost
    /// with the old daemon; an adapter uses that and fabricates nothing.
    /// Fails closed when the native context cannot be evidenced.
    ///
    /// This is §25's "reattach" step, and restart reconciliation calls it
    /// before it classifies anything (`Engine::reconcile_work`), so `Ok` is a
    /// load-bearing claim: *this adapter now owns this context and later
    /// SENDs continue it*. An adapter that cannot evidence that — a turn of
    /// the conversation still running unowned, liveness it cannot read at
    /// all, no durable context to adopt — returns an error and lets the
    /// engine fail the work closed. RESUME never starts a turn: re-adoption
    /// costs no tokens and creates no second execution.
    fn resume(&self, handle: &ExecutionHandle, request: &ResumeRequest)
    -> Result<(), BackendError>;

    /// HISTORY: this execution's normalized native history (§27), in order.
    ///
    /// The answer is the *whole* history or a refusal — never a prefix, and
    /// never a suffix. An adapter whose only record is what its own process
    /// happened to observe cannot honor that after a restart, where the
    /// events exist but this process never saw them; such an adapter
    /// advertises [`Capabilities::history`] `false` and returns
    /// [`BackendError::Unsupported`], which a caller can tell apart from "the
    /// conversation said nothing". `Ok` is therefore only legal from a
    /// backend advertising the capability (pinned across the registry by
    /// `tests/m4_backends.rs`), and sergeant's own durable record of these
    /// events remains the journal, fed by the [`EventSink`].
    fn history(&self, handle: &ExecutionHandle) -> Result<Vec<NativeEvent>, BackendError>;

    /// STOP: retire the execution without corrupting recoverable state. A
    /// backend reports that it *asked*; whether the native context complied is
    /// only knowable through OBSERVE.
    ///
    /// The returned [`Completion`] carries the adapter's tail work — the
    /// evidence promise STOP used to keep by joining internally, which put a
    /// thread join under the daemon's core lock (issue #14/B3). The engine
    /// collects it and the daemon waits after releasing the guard, so the
    /// promise survives and the lock does not.
    fn stop(&self, handle: &ExecutionHandle) -> Result<Completion, BackendError>;
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
        Self::new().with(Arc::new(fake::FakeBackend::from_env(
            fake::FAKE_BACKEND_NAME,
        )))
    }
}
