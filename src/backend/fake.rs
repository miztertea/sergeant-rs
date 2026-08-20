//! The deterministic in-process backend (proposal §37).
//!
//! §37's first testing layer is a fake backend that exercises state
//! transitions, workflow progression, routing, recovery and multi-repo
//! surfaces with "no model tokens required". This is that instrument.
//!
//! It is deterministic by construction: a [`FakeBackend`] is handed a script —
//! an ordered list of [`FakeStep`]s — and hands out one step per execution it
//! starts. Nothing is timed, nothing is polled, nothing races. The same script
//! produces the same journal every run.
//!
//! Two properties make it a *contract* instrument rather than a stub:
//!
//! - a step names its native evidence and its explicit signal independently,
//!   so a test can script the §25 pathologies directly — a live process that
//!   has already completed its stage, an exited process that has signalled
//!   nothing;
//! - [`FakeStep::hang`] models a native context that ignores STOP, so
//!   "terminal work still has live process" (§37's Sergeant regression list)
//!   is reproducible without a real process.
//!
//! ## R-H0-7: the fidelity register (what this backend can and cannot express)
//!
//! MVP-2's H0 review asked this module to state, honestly, what it can now
//! script beyond MVP-1's flat settle delay, and what real-adapter behaviour
//! it still cannot stand in for. This is that register — kept here, not in
//! a separate note, so it stays next to the code it describes and does not
//! go stale the way a standalone doc would.
//!
//! **Expressible today:**
//! - every §15 verb's terminal outcome, scripted per execution
//!   ([`FakeStep::complete`]/`complete_with`/`needs_input`/`ask`/`waiting`/
//!   `blocked`/`fail`), plus §25's native-vs-signal split
//!   ([`FakeStep::with_native`]) for the pathologies that split produces;
//! - a native context that ignores STOP/INTERRUPT entirely
//!   ([`FakeStep::hang`]);
//! - R-MVP1-8's settle window — some number of OBSERVEs that report an
//!   interim shape before the step's real outcome becomes visible
//!   ([`FakeStep::settle`]), generalised by R-H0-7's
//!   [`FakeStep::settle_as`] to script *what* that interim shape is rather
//!   than a fixed `(Running, Running)` — including a native context already
//!   `Exited` while the signal is still unresolved, the "deferred terminal
//!   signal" window a real adapter's read-then-parse lag can produce;
//! - R-H0-7's interrupt fidelity: a `StageCompleted`/`Failed` outcome an
//!   execution had not yet reached when INTERRUPT landed (still inside the
//!   settle window) is discarded, never delivered late — matching the
//!   measured fact that a SIGKILLed turn produces no terminal `type:"result"`
//!   envelope
//!   (`docs/environments/cerberus.md`) — while an already-reached signal, or
//!   a park signal (`NeedsInput`/`Waiting`/`Blocked`) at any settle depth,
//!   survives, matching INTERRUPT's own "no turn in flight is a no-op"
//!   contract and a real `post_turn_summary` line's independent stream
//!   timing;
//! - the three external-effect gates (LAUNCH/SEND/OBSERVE) plus the
//!   STOP/INTERRUPT archive-join gate ([`FakeBackend::hold_archives`]),
//!   for proving the core lock is not held across an adapter call and that
//!   an unawaited [`Completion`] is a bug the caller must not commit;
//! - capability withdrawal ([`FakeBackend::set_available`]/
//!   `set_available_after_launches`) and restart shapes that differ per
//!   adapter memory model ([`FakeBackend::forget_executions`] — the Claude
//!   case, contexts durable but unremembered — versus
//!   [`FakeBackend::complete_live_executions`] — the "finished while the
//!   daemon was down" case).
//!
//! **Not expressible, by design or by unmeasured gap — named so nobody
//! assumes silence means "covered":**
//! - **no wall-clock timing.** [`FakeStep::settle`]/`settle_as` count
//!   OBSERVEs, not elapsed duration; the per-turn wall-clock ceiling
//!   (R-MVP1-7) is tested against the *engine's* own `Instant`-based
//!   deadline with a real-but-tiny `Duration` (`Engine::with_turn_ceiling`),
//!   never against backend-reported timing. This is a deliberate design
//!   choice, not a hole: the ceiling has to fire correctly however long a
//!   real adapter's turn takes, and that property does not need this
//!   backend to simulate a clock at all.
//! - **no usage/cost reporting.** [`Capabilities::usage`] is not modelled;
//!   OBSERVE carries no token/dollar figures for any step, scripted or not.
//!   L16's guard-granularity argument and the Claude adapter's own measured
//!   partial-usage-on-interrupt facts (MVP-2 lane D2) both live on the real
//!   adapter's side; nothing here stands in for them, and a test needing
//!   usage evidence must go through `ClaudeBackend` or assert on the
//!   engine's handling of an adapter that *cannot* report it.
//! - **no streamed/partial evidence within one turn.** A real interrupted
//!   turn can leave "whatever `assistant` chunks streamed before the kill"
//!   in its raw archive (`docs/environments/cerberus.md`); this backend has
//!   no chunk or streaming concept at all — one execution carries exactly
//!   one scripted outcome, delivered whole or not at all, never a partial
//!   fragment of it. A test needing partial-evidence fidelity is a
//!   `ClaudeBackend` contract test, not a fake-backend one.
//! - **`kind = "execute"` stages never reach this backend.**
//!   `Engine::bind_stages` routes an execute stage to the fixed `"docker"`
//!   backend before any Work exists (N4); `FakeBackend` has no execute-stage
//!   concept and is never asked for one in a real run. Naming this so it
//!   reads as a scope boundary, not an oversight.
//! - **one flat script queue, not a per-execution or per-stage-kind one.**
//!   `SGT_FAKE_SCRIPT`'s steps are drawn from one FIFO shared by every
//!   execution this backend starts or sends to
//!   (`parsed_steps_are_one_global_fifo_shared_across_executions_and_sends`);
//!   there is no way to script "this specific execution behaves this way
//!   regardless of draw order" except by controlling submission order
//!   yourself. Unchanged by this review — named because R-H0-7 asked what
//!   the fake still cannot do, and this is a real, standing limit on it.

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use serde_json::json;

use super::{
    Backend, BackendError, Capabilities, Completion, ExecutionHandle, NativeEvent, NativeState,
    Observation, PreparedExecution, ProbeReport, ResumeRequest, RuntimeScope, StartRequest,
};
use crate::backend::BackendSignal;

/// Name the default registry registers the fake under.
pub const FAKE_BACKEND_NAME: &str = "fake";

/// Environment variable holding a script for the compiled-in fake backend.
/// See [`FakeBackend::from_env`] for the grammar.
pub const FAKE_SCRIPT_ENV: &str = "SGT_FAKE_SCRIPT";

/// Parse [`FAKE_SCRIPT_ENV`]'s grammar into steps.
pub fn parse_script(script: &str) -> Vec<FakeStep> {
    script
        .split(';')
        .map(str::trim)
        .filter(|step| !step.is_empty())
        .filter_map(|step| {
            let (verb, detail) = match step.split_once(':') {
                Some((verb, detail)) => (verb.trim(), detail.trim()),
                None => (step, ""),
            };
            match verb {
                "complete" if detail.is_empty() => Some(FakeStep::complete()),
                "complete" => Some(FakeStep::complete_with(detail)),
                "needs_input" => Some(FakeStep::needs_input(detail)),
                "ask" => Some(FakeStep::ask(detail)),
                "waiting" => Some(FakeStep::waiting(detail)),
                "blocked" => Some(FakeStep::blocked(detail)),
                "fail" => Some(FakeStep::fail(detail)),
                "hang" => Some(FakeStep::hang()),
                _ => None,
            }
        })
        .collect()
}

/// One scripted execution's programmed behaviour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeStep {
    /// Native evidence this execution reports.
    pub native: NativeState,
    /// Explicit signal this execution reports.
    pub signal: BackendSignal,
    /// Whether the native context ignores STOP (a hang).
    pub ignores_stop: bool,
    /// R-MVP1-8's settle delay: the number of OBSERVEs, counted from the
    /// launch or send that spawned this step, that report `interim_native`/
    /// `interim_signal` before this step's real native/signal become
    /// visible. `0` (the default) is today's behaviour — the scripted
    /// signal is visible on the very next OBSERVE — so no existing script
    /// changes meaning.
    pub settle: u32,
    /// R-H0-7: what an OBSERVE inside the settle window reports, in place of
    /// the flat `(Running, Running)` every script before this used. Defaults
    /// to exactly that pair — [`FakeStep::settle`] alone is still the same
    /// uniform delay it always was — but [`FakeStep::settle_as`] can script
    /// a *different* interim shape, e.g. a turn whose native context has
    /// already exited while its result is still unresolved: the "deferred
    /// terminal signal" window a real adapter's read-loop-then-parse lag can
    /// produce (§25's ambiguity class), which a bare settle count could not
    /// distinguish from "still running toward the outcome".
    pub interim_native: NativeState,
    /// See [`FakeStep::interim_native`].
    pub interim_signal: BackendSignal,
}

impl FakeStep {
    /// A running execution that has explicitly completed its stage.
    pub fn complete() -> Self {
        Self::running(BackendSignal::StageCompleted { summary: None })
    }

    /// Completes the stage with a summary.
    pub fn complete_with(summary: &str) -> Self {
        Self::running(BackendSignal::StageCompleted {
            summary: Some(summary.to_string()),
        })
    }

    /// Asks for human input — the *adapter* asking (a gate, a policy stop).
    pub fn needs_input(prompt: &str) -> Self {
        Self::running(BackendSignal::needs_input(prompt))
    }

    /// GP-2's ask: the **actor** authored this question and is waiting for a
    /// human answer on the same execution. Distinct from
    /// [`FakeStep::needs_input`] in exactly the way the two are distinct in a
    /// real harness — same park, different author — so a test can prove the
    /// engine carries the authorship instead of flattening it.
    pub fn ask(question: &str) -> Self {
        Self::running(BackendSignal::ask(question))
    }

    /// Waits on an external condition.
    pub fn waiting(reason: &str) -> Self {
        Self::running(BackendSignal::Waiting {
            reason: reason.to_string(),
        })
    }

    /// Reports the stage blocked.
    pub fn blocked(reason: &str) -> Self {
        Self::running(BackendSignal::Blocked {
            reason: reason.to_string(),
        })
    }

    /// Reports the stage failed.
    pub fn fail(reason: &str) -> Self {
        Self::running(BackendSignal::Failed {
            reason: reason.to_string(),
        })
    }

    /// Runs forever and ignores STOP: the native context that will not die.
    pub fn hang() -> Self {
        Self {
            native: NativeState::Running,
            signal: BackendSignal::Running,
            ignores_stop: true,
            settle: 0,
            interim_native: NativeState::Running,
            interim_signal: BackendSignal::Running,
        }
    }

    /// Override the native evidence, leaving the signal alone. This is how a
    /// test scripts §25's separation: `FakeStep::complete().with_native(
    /// NativeState::Exited)` and `FakeStep::hang().with_native(
    /// NativeState::Exited)` are both legal and mean different things.
    pub fn with_native(mut self, native: NativeState) -> Self {
        self.native = native;
        self
    }

    /// R-MVP1-8: report `Running` for the first `k` OBSERVEs after the
    /// launch or send that spawns this step, before this step's own
    /// native/signal become visible. `k = 0` is the no-op default — this
    /// method exists so a test can stand between "a turn was spawned" and
    /// "the turn finished" for a turn that *does* finish, the interval the
    /// fake had no way to model before.
    pub fn settle(mut self, k: u32) -> Self {
        self.settle = k;
        self
    }

    /// R-H0-7: like [`FakeStep::settle`], but the settle window reports
    /// `(interim_native, interim_signal)` instead of the flat
    /// `(Running, Running)` `settle` alone produces. Names the shape
    /// directly rather than adding a family of single-purpose constructors:
    /// a "native already exited, signal not yet resolved" window is
    /// `settle_as(k, NativeState::Exited, BackendSignal::Running)`; any
    /// other scriptable interim observation is the same call with different
    /// arguments.
    pub fn settle_as(
        mut self,
        k: u32,
        interim_native: NativeState,
        interim_signal: BackendSignal,
    ) -> Self {
        self.settle = k;
        self.interim_native = interim_native;
        self.interim_signal = interim_signal;
        self
    }

    fn running(signal: BackendSignal) -> Self {
        Self {
            native: NativeState::Running,
            signal,
            ignores_stop: false,
            settle: 0,
            interim_native: NativeState::Running,
            interim_signal: BackendSignal::Running,
        }
    }
}

/// A place a scripted execution can be made to stall on purpose.
///
/// §22.6 asks for instrumentation proving the core lock is not held across an
/// external effect, and the only way to prove a negative about a lock is to
/// make the effect take arbitrarily long and watch an *independent* request go
/// through anyway. A sleep would make the test a race against a clock; a gate
/// makes it a rendezvous: the test waits until the executor is provably parked
/// inside the effect, does its independent work, and only then releases.
#[derive(Debug, Default)]
struct Gate {
    inner: Mutex<GateState>,
    changed: Condvar,
}

#[derive(Debug, Default)]
struct GateState {
    held: bool,
    /// How many callers are parked inside the gate right now.
    waiting: usize,
}

impl Gate {
    /// Close the gate: every later [`Gate::pass`] parks until released.
    fn hold(&self) {
        self.inner.lock().expect("gate lock").held = true;
    }

    /// Open the gate and wake everyone parked in it.
    fn release(&self) {
        self.inner.lock().expect("gate lock").held = false;
        self.changed.notify_all();
    }

    /// Go through, parking while the gate is closed.
    fn pass(&self) {
        let mut state = self.inner.lock().expect("gate lock");
        state.waiting += 1;
        self.changed.notify_all();
        while state.held {
            state = self.changed.wait(state).expect("gate wait");
        }
        state.waiting -= 1;
        self.changed.notify_all();
    }

    /// Block until at least `n` callers are parked inside, or the deadline
    /// passes. Returns whether the rendezvous happened.
    fn wait_for_waiting(&self, n: usize, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        let mut state = self.inner.lock().expect("gate lock");
        while state.waiting < n {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                return false;
            };
            let (next, timed_out) = self
                .changed
                .wait_timeout(state, remaining)
                .expect("gate wait");
            state = next;
            if timed_out.timed_out() && state.waiting < n {
                return false;
            }
        }
        true
    }
}

#[derive(Debug)]
struct FakeExecution {
    /// The backend's own name for the native context. §25's restart sequence
    /// asks the adapter for native identity, so a handle that names one must
    /// name *this* one — see [`FakeBackend::resolve`].
    native_id: String,
    step: FakeStep,
    inputs: Vec<String>,
    stopped: bool,
    /// R-MVP1-8: OBSERVEs remaining that must report `Running` before
    /// `step`'s own native/signal become visible. Set from `step.settle`
    /// whenever `step` is assigned by LAUNCH or SEND (the spawn points);
    /// left untouched by the out-of-band restart/ask mutators, which are
    /// not spawn events.
    settle_remaining: u32,
}

#[derive(Debug)]
struct FakeState {
    script: VecDeque<FakeStep>,
    executions: BTreeMap<String, FakeExecution>,
    /// Contexts this adapter no longer holds in memory but which still exist
    /// out in the world — see [`FakeBackend::forget_executions`].
    forgotten: BTreeMap<String, FakeExecution>,
    starts: Vec<StartRequest>,
    stop_requests: Vec<String>,
    interrupt_requests: Vec<String>,
    resume_requests: Vec<(String, ResumeRequest)>,
    observations: Vec<String>,
    probes: usize,
    available: bool,
    detail: Option<String>,
    /// Armed by [`FakeBackend::set_available_after_launches`]: after this
    /// many more successful LAUNCHes, flip `available` off with the carried
    /// detail. `None` when unarmed.
    unavailable_after_launches: Option<(u32, String)>,
}

/// A scriptable, deterministic, in-process backend.
///
/// Cloning shares the script and the execution table, so a test can keep an
/// inspection handle while the daemon's registry holds the same instance —
/// and a registry rebuilt across a simulated restart can either keep the
/// state (a native session that survived) or start fresh (one that did not).
#[derive(Debug, Clone)]
pub struct FakeBackend {
    name: String,
    capabilities: Capabilities,
    state: Arc<Mutex<FakeState>>,
    /// Where LAUNCH can be made to stall (§22.6's "deliberately stalled fake
    /// executor"). Deliberately *not* inside `state`: a caller parked in the
    /// gate must not be holding the backend's own lock, or the instrument
    /// would measure the fake's contention instead of the daemon's.
    launch_gate: Arc<Gate>,
    /// Where SEND can be made to stall. SEND is an external effect too — for
    /// a print-mode harness it is the same fork/exec LAUNCH is — so §22.6's
    /// instrument has to be able to park inside it, or the budget's verdict
    /// is a claim about the paths the instrument happens to reach.
    send_gate: Arc<Gate>,
    /// Where OBSERVE can be made to stall. OBSERVE is the third external
    /// effect and the least obviously one: for the Claude adapter a handle it
    /// has no memory of sends it walking `/proc`, and §22.6 lists "reading a
    /// large output stream" beside the process spawn. It rides home in
    /// [`PendingLaunch::perform`](crate::runtime::engine::PendingLaunch::perform)
    /// and [`PendingSend::perform`](crate::runtime::engine::PendingSend::perform)
    /// for exactly that reason — and an instrument that cannot park inside it
    /// cannot say so, which is how an OBSERVE moved back under the guard
    /// survived wave 1 and wave 2 both.
    observe_gate: Arc<Gate>,
    /// Where a STOP [`Completion`] can be made to stall — the fake's stand-in
    /// for the Claude adapter's transcript-archive join (issue #14/B3).
    archive_gate: Arc<Gate>,
    /// Whether STOP/INTERRUPT hand back a deferred completion at all.
    archive_armed: Arc<Mutex<bool>>,
}

impl FakeBackend {
    /// A fake that completes every stage it is given.
    pub fn new(name: &str) -> Self {
        Self::scripted(name, [])
    }

    /// A fake that plays `script` in order, one step per started execution,
    /// then completes every stage after the script runs out.
    pub fn scripted(name: &str, script: impl IntoIterator<Item = FakeStep>) -> Self {
        Self {
            name: name.to_string(),
            capabilities: Capabilities {
                persistent_sessions: true,
                // Honest: the fake's record of an execution is complete or
                // the execution is unknown to it — there is no partial answer
                // it could return, which is exactly what the capability
                // claims (see `Capabilities::history`).
                history: true,
                resume: true,
                model_selection: true,
                profiles: true,
                // The fake's "actor" is its script, and a scripted
                // `FakeStep::ask` is an unambiguous structured record of the
                // actor authoring a question — which is precisely what the
                // capability claims. Nothing is inferred from prose.
                ask: true,
                ..Capabilities::default()
            },
            state: Arc::new(Mutex::new(FakeState {
                script: script.into_iter().collect(),
                executions: BTreeMap::new(),
                forgotten: BTreeMap::new(),
                starts: Vec::new(),
                stop_requests: Vec::new(),
                interrupt_requests: Vec::new(),
                resume_requests: Vec::new(),
                observations: Vec::new(),
                probes: 0,
                available: true,
                detail: Some("deterministic in-process test backend".to_string()),
                unavailable_after_launches: None,
            })),
            launch_gate: Arc::new(Gate::default()),
            send_gate: Arc::new(Gate::default()),
            observe_gate: Arc::new(Gate::default()),
            archive_gate: Arc::new(Gate::default()),
            archive_armed: Arc::new(Mutex::new(false)),
        }
    }

    /// Override the advertised capabilities. R-MVP1-11's preflight test
    /// needs a fake that declares `ask: false` — every other capability the
    /// scripted default sets is deliberately left in place unless the
    /// caller's `capabilities` also changes it, since a test proving one
    /// refusal should not have to restate the whole capability set.
    pub fn with_capabilities(mut self, capabilities: Capabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// A fake scripted from the environment (`SGT_FAKE_SCRIPT`).
    ///
    /// The script mechanism already exists; this is its front door for a fake
    /// running inside a *spawned* daemon, where no test can hand it a
    /// `Vec<FakeStep>` in process. The §39 walkthrough needs exactly that: a
    /// real `sgt daemon`, driven by real CLI commands, that reaches
    /// `needs_input` on cue instead of completing everything instantly.
    ///
    /// Grammar — steps separated by `;`, each `verb` or `verb:detail`:
    ///
    /// ```text
    /// SGT_FAKE_SCRIPT="needs_input:confirm the retry policy;complete:done"
    /// ```
    ///
    /// Verbs: `complete`, `needs_input`, `waiting`, `blocked`, `fail`, `hang`.
    /// An unknown verb is ignored rather than silently mistaken for another —
    /// a typo must not quietly change what the demo demonstrates.
    pub fn from_env(name: &str) -> Self {
        match std::env::var(FAKE_SCRIPT_ENV) {
            Ok(script) => Self::scripted(name, parse_script(&script)),
            Err(_) => Self::new(name),
        }
    }

    /// Stall every later LAUNCH until [`FakeBackend::release_launches`].
    ///
    /// The §22.6 instrument. A launch parked here is an external effect in
    /// flight; anything the daemon can still answer while it is parked is
    /// something the core lock was demonstrably not held across.
    pub fn hold_launches(&self) {
        self.launch_gate.hold();
    }

    /// Let stalled launches through.
    pub fn release_launches(&self) {
        self.launch_gate.release();
    }

    /// Block until `n` launches are parked in the gate, or the timeout
    /// expires. Returns whether the rendezvous happened — a test asserts on
    /// it rather than sleeping and hoping.
    pub fn await_stalled_launches(&self, n: usize, timeout: Duration) -> bool {
        self.launch_gate.wait_for_waiting(n, timeout)
    }

    /// Stall every later SEND until [`FakeBackend::release_sends`].
    ///
    /// The §22.6 instrument for the other verb that creates external work.
    /// `ClaudeBackend::send` spawns a `claude -p --resume` turn plus its
    /// reader threads, so a `respond` is as much a process spawn as a launch
    /// is — and it is the verb GP-2's ask primitive resumes through.
    pub fn hold_sends(&self) {
        self.send_gate.hold();
    }

    /// Let stalled sends through.
    pub fn release_sends(&self) {
        self.send_gate.release();
    }

    /// Block until `n` sends are parked in the gate, or the timeout expires.
    pub fn await_stalled_sends(&self, n: usize, timeout: Duration) -> bool {
        self.send_gate.wait_for_waiting(n, timeout)
    }

    /// Stall every later OBSERVE until [`FakeBackend::release_observes`].
    ///
    /// The §22.6 instrument for the third external effect. Both performers
    /// take their observation before handing the outcome back to the settle
    /// phase; parking inside it is the only way to tell that apart from an
    /// observation taken with the guard already re-acquired.
    pub fn hold_observes(&self) {
        self.observe_gate.hold();
    }

    /// Let stalled observations through.
    pub fn release_observes(&self) {
        self.observe_gate.release();
    }

    /// Block until `n` observations are parked in the gate, or time out.
    pub fn await_stalled_observes(&self, n: usize, timeout: Duration) -> bool {
        self.observe_gate.wait_for_waiting(n, timeout)
    }

    /// Make STOP/INTERRUPT hand back a *deferred* [`Completion`] — the fake's
    /// stand-in for the Claude adapter's transcript archive — and stall it
    /// until [`FakeBackend::release_archives`].
    pub fn hold_archives(&self) {
        *self.archive_armed.lock().expect("archive arm lock") = true;
        self.archive_gate.hold();
    }

    /// Let stalled stop/interrupt completions finish.
    pub fn release_archives(&self) {
        self.archive_gate.release();
    }

    /// Block until `n` stop/interrupt completions are parked, or time out.
    pub fn await_stalled_archives(&self, n: usize, timeout: Duration) -> bool {
        self.archive_gate.wait_for_waiting(n, timeout)
    }

    /// Make PROBE report unavailable (routing must then fail closed).
    pub fn set_available(&self, available: bool, detail: &str) {
        let mut state = self.lock();
        state.available = available;
        state.detail = Some(detail.to_string());
    }

    /// Become unavailable after `n` more successful LAUNCHes — the
    /// deterministic form of "the registry changed under a bound run": armed
    /// *before* submit, so a mid-run refusal needs no test-side race against
    /// the completion driver's ticks (a post-submit `set_available(false)`
    /// call has to beat the next stage's launch, which is a scheduling bet).
    /// The flip lands after LAUNCH `n` returns its handle, so that launch's
    /// own stage runs and the (`n`+1)th stage's PREPARE is the refusal.
    pub fn set_available_after_launches(&self, n: u32, detail: &str) {
        self.lock().unavailable_after_launches = Some((n, detail.to_string()));
    }

    /// Every START request this backend received, in order.
    pub fn starts(&self) -> Vec<StartRequest> {
        self.lock().starts.clone()
    }

    /// Execution ids STOP was requested for, in order.
    pub fn stop_requests(&self) -> Vec<String> {
        self.lock().stop_requests.clone()
    }

    /// Execution ids INTERRUPT was requested for, in order.
    pub fn interrupt_requests(&self) -> Vec<String> {
        self.lock().interrupt_requests.clone()
    }

    /// Every RESUME this backend was asked for, in order, with the request
    /// the caller re-supplied. Restart reconciliation reattaches before it
    /// classifies (§25), so "was this context re-adopted, and with which
    /// launch configuration" is a property tests assert directly.
    pub fn resume_requests(&self) -> Vec<(String, ResumeRequest)> {
        self.lock().resume_requests.clone()
    }

    /// How many times PROBE has been asked of this backend.
    ///
    /// The real adapters cache their probe after the first call, so *when* the
    /// first one happens decides whether a later `prepare` under the core lock
    /// forks `claude --version` or reads a warm cache (§22.6's "probing a slow
    /// external executable"). That ordering is a property tests assert, not an
    /// accident of startup.
    pub fn probe_count(&self) -> usize {
        self.lock().probes
    }

    /// Execution ids OBSERVE was called for, in order.
    ///
    /// OBSERVE is the one backend call the engine could plausibly make twice
    /// for one decision, and the two answers are not guaranteed to agree — so
    /// how many times it was asked is a property tests need to assert, not an
    /// implementation detail.
    pub fn observations(&self) -> Vec<String> {
        self.lock().observations.clone()
    }

    /// Inputs delivered to one execution, in order.
    pub fn inputs(&self, execution_id: &str) -> Vec<String> {
        self.lock()
            .executions
            .get(execution_id)
            .map(|e| e.inputs.clone())
            .unwrap_or_default()
    }

    /// Native evidence this backend would report for an execution, without
    /// going through the trait. Tests use it to assert what the *backend*
    /// believes while asserting separately what the *work* state is.
    pub fn native_state(&self, execution_id: &str) -> Option<NativeState> {
        self.lock()
            .executions
            .get(execution_id)
            .map(|e| e.step.native)
    }

    /// Whether the backend still considers an execution live.
    pub fn is_live(&self, execution_id: &str) -> bool {
        self.native_state(execution_id) == Some(NativeState::Running)
    }

    /// The running actor authors a question, mid-execution (GP-2).
    ///
    /// This is the out-of-band half of the ask primitive: nothing about the
    /// script decided it, and no engine call produced it — a live execution
    /// changed what it is saying while the engine was elsewhere, exactly as a
    /// real actor does when it reaches a decision it cannot make alone. The
    /// next OBSERVE reports it as [`AskAuthor::Actor`](super::AskAuthor::Actor).
    ///
    /// Returns whether the execution was known; an ask against an execution
    /// this backend never had is refused, not invented.
    pub fn actor_asks(&self, execution_id: &str, question: &str) -> bool {
        let mut state = self.lock();
        match state.executions.get_mut(execution_id) {
            Some(execution) => {
                execution.step.signal = BackendSignal::ask(question);
                true
            }
            None => false,
        }
    }

    /// Drop this adapter's *memory* of every execution while leaving the
    /// contexts themselves intact — the shape a daemon restart leaves for an
    /// adapter whose native contexts are durable.
    ///
    /// This is the Claude case made scriptable. A restarted `ClaudeBackend`
    /// has an empty execution table while the conversations it started are
    /// still on disk: SEND and OBSERVE answer `UnknownExecution`, and §15
    /// RESUME is the one verb that can bring a context back into the
    /// adapter's hands. Sharing a `FakeBackend` across a simulated restart
    /// modelled only the *other* case — an adapter that somehow remembered
    /// everything — which is why nothing caught a `respond` that could not
    /// survive one.
    pub fn forget_executions(&self) {
        let mut state = self.lock();
        let remembered = std::mem::take(&mut state.executions);
        state.forgotten.extend(remembered);
    }

    /// Reprogram every live execution to "the stage finished and the native
    /// context then exited" — the session that completed while the daemon was
    /// down, which is the realistic restart case.
    ///
    /// The exited native state is deliberate: it is the §25 pathology a
    /// recovery path is most likely to get wrong, because a dead process
    /// looks like a failure to anyone reading liveness instead of signals.
    pub fn complete_live_executions(&self) {
        let mut state = self.lock();
        for execution in state.executions.values_mut() {
            if !execution.stopped {
                execution.step = FakeStep::complete().with_native(NativeState::Exited);
            }
        }
    }

    /// The [`Completion`] STOP/INTERRUPT hand back: nothing to wait for by
    /// default, and a gated wait once a test has armed the archive stall.
    fn completion(&self) -> Completion {
        if !*self.archive_armed.lock().expect("archive arm lock") {
            return Completion::immediate();
        }
        let gate = Arc::clone(&self.archive_gate);
        Completion::deferred(move || gate.pass())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, FakeState> {
        // A poisoned lock means a test thread panicked mid-mutation; there is
        // no recovery to attempt in a test instrument, so surface it.
        self.state.lock().expect("fake backend state lock")
    }

    /// One step per started execution, in script order; a script that runs
    /// out completes every stage after it.
    fn next_step(state: &mut FakeState) -> FakeStep {
        state.script.pop_front().unwrap_or_else(FakeStep::complete)
    }

    /// Find the execution a handle names, the way §25's restart sequence
    /// says an adapter must: by sergeant's execution id *and* the native
    /// identity sergeant recorded for it. A handle that has lost the native
    /// id, or carries one from another context, is not recognised — that is
    /// the ambiguity §25 requires to fail closed, not a session to guess at.
    fn resolve<'a>(
        &self,
        state: &'a FakeState,
        handle: &ExecutionHandle,
    ) -> Result<&'a FakeExecution, BackendError> {
        let unknown = || BackendError::UnknownExecution {
            backend: self.name.clone(),
            execution_id: handle.execution_id.clone(),
        };
        let execution = state
            .executions
            .get(&handle.execution_id)
            .ok_or_else(unknown)?;
        if handle.native_id.as_deref() != Some(execution.native_id.as_str()) {
            return Err(unknown());
        }
        Ok(execution)
    }
}

impl Backend for FakeBackend {
    fn name(&self) -> &str {
        &self.name
    }

    fn capabilities(&self) -> Capabilities {
        self.capabilities
    }

    /// §17: each fake execution is its own in-process context and there is no
    /// shared service behind them, which is `per_execution` — the same scope
    /// the print-mode Claude adapter declares, deliberately: the deterministic
    /// stand-in must not model a runtime model no real adapter here has.
    fn runtime_scope(&self) -> RuntimeScope {
        RuntimeScope::PerExecution
    }

    fn probe(&self) -> ProbeReport {
        let mut state = self.lock();
        state.probes += 1;
        ProbeReport {
            available: state.available,
            detail: state.detail.clone(),
        }
    }

    /// PREPARE allocates this execution's native identity and nothing else:
    /// no script step is consumed, no execution table entry appears, and the
    /// gate is not touched. That is what makes it safe to call under the core
    /// lock — and what makes the reservation the engine journals a claim
    /// about an identity that does not exist yet.
    fn prepare(&self, request: &StartRequest) -> Result<PreparedExecution, BackendError> {
        let state = self.lock();
        if !state.available {
            return Err(BackendError::Unavailable {
                backend: self.name.clone(),
                detail: state
                    .detail
                    .clone()
                    .unwrap_or_else(|| "scripted unavailable".to_string()),
            });
        }
        Ok(PreparedExecution {
            execution_id: request.execution_id.clone(),
            native_id: Some(format!("fake-session-{}", request.execution_id)),
            request: request.clone(),
        })
    }

    /// LAUNCH is the external effect — and therefore the one place a test can
    /// stall this backend (see [`FakeBackend::hold_launches`]). The gate is
    /// passed *before* the state lock is taken, so a parked launch holds
    /// nothing at all: it is a stand-in for a process spawn, not for
    /// contention inside the adapter.
    fn launch(&self, prepared: &PreparedExecution) -> Result<ExecutionHandle, BackendError> {
        self.launch_gate.pass();
        let mut state = self.lock();
        if !state.available {
            return Err(BackendError::Unavailable {
                backend: self.name.clone(),
                detail: state
                    .detail
                    .clone()
                    .unwrap_or_else(|| "scripted unavailable".to_string()),
            });
        }
        let step = Self::next_step(&mut state);
        state.starts.push(prepared.request.clone());
        let native_id = prepared
            .native_id
            .clone()
            .unwrap_or_else(|| format!("fake-session-{}", prepared.execution_id));
        state.executions.insert(
            prepared.execution_id.clone(),
            FakeExecution {
                native_id: native_id.clone(),
                settle_remaining: step.settle,
                step,
                inputs: Vec::new(),
                stopped: false,
            },
        );
        if let Some((remaining, detail)) = state.unavailable_after_launches.take() {
            if remaining <= 1 {
                state.available = false;
                state.detail = Some(detail);
            } else {
                state.unavailable_after_launches = Some((remaining - 1, detail));
            }
        }
        Ok(ExecutionHandle {
            execution_id: prepared.execution_id.clone(),
            native_id: Some(native_id),
        })
    }

    /// SEND is an external effect, and therefore gated exactly as LAUNCH is —
    /// the gate is passed before the state lock, so a parked send holds
    /// nothing of the backend's own.
    fn send(&self, handle: &ExecutionHandle, input: &str) -> Result<(), BackendError> {
        self.send_gate.pass();
        let mut state = self.lock();
        self.resolve(&state, handle)?;
        // Delivering input advances this execution to the next scripted step:
        // the answer is what unblocks the turn.
        let step = Self::next_step(&mut state);
        let execution = state
            .executions
            .get_mut(&handle.execution_id)
            .expect("presence checked above");
        execution.inputs.push(input.to_string());
        execution.step = step;
        execution.settle_remaining = execution.step.settle;
        Ok(())
    }

    /// OBSERVE is an external effect too, and therefore gated exactly as
    /// LAUNCH and SEND are — the gate is passed before the state lock, so a
    /// parked observation holds nothing of the backend's own.
    fn observe(&self, handle: &ExecutionHandle) -> Result<Observation, BackendError> {
        self.observe_gate.pass();
        let mut state = self.lock();
        state.observations.push(handle.execution_id.clone());
        // Identity check first, exactly as SEND does: a handle that does not
        // resolve must not consume a settle tick either.
        self.resolve(&state, handle)?;
        let execution = state
            .executions
            .get_mut(&handle.execution_id)
            .expect("presence checked above");
        // R-MVP1-8/R-H0-7: the first `settle_remaining` OBSERVEs after the
        // spawning launch/send report the step's scripted interim shape
        // (`(Running, Running)` unless [`FakeStep::settle_as`] scripted
        // something else); the real native/signal become visible only once
        // the window is spent. This is what lets a test stand between "turn
        // spawned" and "turn finished" for a turn that finishes, and — with
        // a non-default interim — script a native context that has already
        // exited while its result is still unresolved.
        let (native, signal) = if execution.settle_remaining > 0 {
            execution.settle_remaining -= 1;
            (
                execution.step.interim_native,
                execution.step.interim_signal.clone(),
            )
        } else {
            (execution.step.native, execution.step.signal.clone())
        };
        Ok(Observation {
            native,
            signal,
            evidence: Some(format!(
                "fake backend: native={}, stopped={}",
                native.as_str(),
                execution.stopped
            )),
        })
    }

    /// INTERRUPT stops the current turn but never retires the conversation:
    /// the execution stays known, its signal survives, and — like the real
    /// adapters — a compliant native context reports its turn process gone
    /// while a hang keeps running.
    ///
    /// R-H0-7 fidelity fix: "signal survives" is true for a signal that was
    /// already *reached* (`settle_remaining == 0` — the trait's own "no turn
    /// in flight" no-op case) or that is a park state (`NeedsInput`/
    /// `Waiting`/`Blocked`) at any settle depth — a real `post_turn_summary`
    /// line streams independently of the turn's terminal envelope, so a kill
    /// after it was written can still leave it standing. It is never true
    /// for a `StageCompleted`/`Failed` outcome the execution had not yet
    /// reached (`settle_remaining > 0`): measured against Claude 2.1.227
    /// (`docs/environments/cerberus.md`'s interrupt/SIGTERM entry), a
    /// SIGKILLed turn — `interrupt` and `stop` both use it — never produces
    /// a terminal `type:"result"` envelope, so a not-yet-reached terminal
    /// outcome is lost, not delivered late. Before this fix the fake
    /// delivered it anyway once `settle_remaining` ticked down on later
    /// OBSERVEs, which is exactly the shape a real killed turn cannot
    /// produce.
    fn interrupt(&self, handle: &ExecutionHandle) -> Result<Completion, BackendError> {
        let mut state = self.lock();
        state.interrupt_requests.push(handle.execution_id.clone());
        self.resolve(&state, handle)?;
        let execution = state
            .executions
            .get_mut(&handle.execution_id)
            .expect("presence checked above");
        if !execution.step.ignores_stop {
            execution.step.native = NativeState::Exited;
            if execution.settle_remaining > 0
                && matches!(
                    execution.step.signal,
                    BackendSignal::StageCompleted { .. } | BackendSignal::Failed { .. }
                )
            {
                execution.step.signal = BackendSignal::Running;
                execution.settle_remaining = 0;
            }
        }
        drop(state);
        Ok(self.completion())
    }

    /// RESUME re-adopts an execution; §25's identity rule applies the same as
    /// everywhere else — a handle without this context's native identity is
    /// not recognised, it is refused.
    ///
    /// A context this adapter has *forgotten* (see
    /// [`FakeBackend::forget_executions`]) is exactly what RESUME is for: it
    /// comes back into the execution table and later SENDs continue it, which
    /// is the promise `Ok` makes on the real adapter too.
    fn resume(
        &self,
        handle: &ExecutionHandle,
        request: &ResumeRequest,
    ) -> Result<(), BackendError> {
        let mut state = self.lock();
        state
            .resume_requests
            .push((handle.execution_id.clone(), request.clone()));
        if let Some(forgotten) = state.forgotten.get(&handle.execution_id)
            && handle.native_id.as_deref() == Some(forgotten.native_id.as_str())
        {
            let readopted = state
                .forgotten
                .remove(&handle.execution_id)
                .expect("presence checked above");
            state
                .executions
                .insert(handle.execution_id.clone(), readopted);
            return Ok(());
        }
        self.resolve(&state, handle)?;
        Ok(())
    }

    /// HISTORY reports the inputs this execution received as
    /// `conversation.user` events — the minimal honest §27 surface for a
    /// backend with no native transcript.
    fn history(&self, handle: &ExecutionHandle) -> Result<Vec<NativeEvent>, BackendError> {
        let state = self.lock();
        let execution = self.resolve(&state, handle)?;
        Ok(execution
            .inputs
            .iter()
            .map(|input| NativeEvent {
                kind: "conversation.user".to_string(),
                payload: json!({"text": input}),
            })
            .collect())
    }

    fn stop(&self, handle: &ExecutionHandle) -> Result<Completion, BackendError> {
        let mut state = self.lock();
        state.stop_requests.push(handle.execution_id.clone());
        // One identity rule, checked in one place: a handle that has lost its
        // native id, or carries one from another context, must not be able to
        // stop an execution it does not actually name.
        self.resolve(&state, handle)?;
        let execution = state
            .executions
            .get_mut(&handle.execution_id)
            .expect("presence checked above");
        execution.stopped = true;
        if !execution.step.ignores_stop {
            // A compliant context exits and stops signalling about the stage.
            execution.step.native = NativeState::Exited;
            execution.step.signal = BackendSignal::Running;
        }
        drop(state);
        Ok(self.completion())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn request(execution_id: &str) -> StartRequest {
        StartRequest {
            work_id: "work".to_string(),
            execution_id: execution_id.to_string(),
            stage_id: "00-prepare".to_string(),
            attempt: 1,
            cwd: PathBuf::from("/tmp"),
            intent: "do it".to_string(),
            context: "context".to_string(),
            model: None,
            profile: None,
            execute: None,
            instruction_policy: crate::domain::workspace::InstructionPolicy::default(),
            bindings: Vec::new(),
        }
    }

    #[test]
    fn one_step_per_execution_in_script_order_then_the_default() {
        let fake = FakeBackend::scripted(
            "fake",
            [FakeStep::fail("first fails"), FakeStep::needs_input("who?")],
        );
        let first = fake.start(&request("e1")).expect("start");
        let second = fake.start(&request("e2")).expect("start");
        let third = fake.start(&request("e3")).expect("start");
        assert_eq!(
            fake.observe(&first).expect("observe").signal,
            BackendSignal::Failed {
                reason: "first fails".to_string()
            }
        );
        assert_eq!(
            fake.observe(&second).expect("observe").signal,
            BackendSignal::needs_input("who?")
        );
        assert_eq!(
            fake.observe(&third).expect("observe").signal,
            BackendSignal::StageCompleted { summary: None }
        );
    }

    #[test]
    fn a_hang_ignores_stop_but_a_compliant_context_exits() {
        let fake = FakeBackend::scripted("fake", [FakeStep::hang(), FakeStep::complete()]);
        let hanging = fake.start(&request("hang")).expect("start");
        let compliant = fake.start(&request("ok")).expect("start");

        fake.stop(&hanging).expect("stop").wait();
        fake.stop(&compliant).expect("stop").wait();
        assert_eq!(fake.stop_requests(), vec!["hang", "ok"]);
        assert_eq!(
            fake.observe(&hanging).expect("observe").native,
            NativeState::Running,
            "a hang keeps running after a stop request"
        );
        assert_eq!(
            fake.observe(&compliant).expect("observe").native,
            NativeState::Exited
        );
    }

    #[test]
    fn an_unknown_execution_is_an_error_not_a_guess() {
        let fake = FakeBackend::new("fake");
        let stranger = ExecutionHandle {
            execution_id: "never-started".to_string(),
            native_id: None,
        };
        assert!(matches!(
            fake.observe(&stranger),
            Err(BackendError::UnknownExecution { .. })
        ));
        assert!(matches!(
            fake.send(&stranger, "hello"),
            Err(BackendError::UnknownExecution { .. })
        ));
    }

    /// §25's restart sequence asks the adapter for *native identity*, not just
    /// for sergeant's own id. A handle that lost the native id, or carries
    /// another context's, is unrecognised — so the id sergeant journals at
    /// START has to be the id it presents at OBSERVE.
    ///
    /// The rule holds for every call that names an execution, not just
    /// OBSERVE: a forged handle must not be able to read one, feed one, or
    /// kill one. STOP is the one where getting it wrong is destructive.
    #[test]
    fn a_handle_must_carry_the_native_identity_the_backend_issued() {
        let fake = FakeBackend::new("fake");
        let handle = fake.start(&request("e1")).expect("start");
        let other = fake.start(&request("e2")).expect("start");
        assert_eq!(handle.native_id.as_deref(), Some("fake-session-e1"));
        fake.observe(&handle).expect("the issued handle is known");

        for wrong in [None, other.native_id.clone()] {
            let forged = ExecutionHandle {
                execution_id: handle.execution_id.clone(),
                native_id: wrong,
            };
            for outcome in [
                fake.observe(&forged).map(|_| ()),
                fake.send(&forged, "hello"),
                fake.interrupt(&forged).map(|c| c.wait()),
                fake.resume(&forged, &ResumeRequest::new("w", PathBuf::from("/tmp"))),
                fake.history(&forged).map(|_| ()),
                fake.stop(&forged).map(|c| c.wait()),
            ] {
                assert!(
                    matches!(outcome, Err(BackendError::UnknownExecution { .. })),
                    "a handle without this context's native identity must not resolve"
                );
            }
        }

        // And none of those refusals touched the execution they named: it is
        // still live, still unstopped, and still holding its own script.
        let observed = fake.observe(&handle).expect("still known");
        assert_eq!(observed.native, NativeState::Running);
        assert_eq!(
            observed.evidence.as_deref(),
            Some("fake backend: native=running, stopped=false"),
            "a forged STOP must not have retired the execution"
        );
        assert!(
            fake.inputs("e1").is_empty(),
            "a forged SEND delivered nothing"
        );
    }

    /// The M4 trait surface on the fake: INTERRUPT stops a compliant turn
    /// (native exits, signal survives) but not a hang; RESUME re-adopts a
    /// known handle; HISTORY reports delivered inputs as
    /// `conversation.user` events.
    #[test]
    fn interrupt_resume_and_history_behave_like_the_real_adapters() {
        let fake = FakeBackend::scripted("fake", [FakeStep::needs_input("who?"), FakeStep::hang()]);
        let compliant = fake.start(&request("c")).expect("start");
        let hanging = fake.start(&request("h")).expect("start");

        fake.interrupt(&compliant).expect("interrupt").wait();
        fake.interrupt(&hanging).expect("interrupt").wait();
        assert_eq!(fake.interrupt_requests(), vec!["c", "h"]);
        let observed = fake.observe(&compliant).expect("observe");
        assert_eq!(observed.native, NativeState::Exited, "the turn died");
        assert_eq!(
            observed.signal,
            BackendSignal::needs_input("who?"),
            "the conversation's signal survives the interrupt"
        );
        assert_eq!(
            fake.observe(&hanging).expect("observe").native,
            NativeState::Running,
            "a hang ignores interrupt like it ignores stop"
        );

        fake.resume(&compliant, &ResumeRequest::new("w", "/anywhere"))
            .expect("a known handle re-adopts");
        fake.send(&compliant, "it was Mallory").expect("send");
        assert_eq!(
            fake.history(&compliant).expect("history"),
            vec![NativeEvent {
                kind: "conversation.user".to_string(),
                payload: serde_json::json!({"text": "it was Mallory"}),
            }]
        );
    }

    /// R-H0-7's headline fix: a `StageCompleted` outcome an execution had
    /// not yet reached (mid-settle) at the moment INTERRUPT lands is
    /// discarded, not delivered late. Every OBSERVE from here on — not just
    /// the very next one — must keep reporting the unresolved shape, proving
    /// this isn't a one-tick artefact of where `settle_remaining` happened
    /// to be.
    ///
    /// Mutation targets this kills (L7): the settle-window check dropped
    /// (a terminal signal would then survive an interrupt taken before it
    /// was reached — the exact regression this fix closes); the guard
    /// inverted to fire only when `settle_remaining == 0` (a terminal signal
    /// already reached would be wiped instead of surviving, breaking the
    /// no-op contract the next test pins); `settle_remaining` left non-zero
    /// after the wipe (the interim shape would resurface once the window
    /// ran out, re-delivering the discarded outcome late).
    #[test]
    fn interrupt_mid_settle_discards_a_not_yet_reached_terminal_signal_forever() {
        let fake = FakeBackend::scripted("fake", [FakeStep::complete_with("done").settle(3)]);
        let handle = fake.start(&request("mid-flight")).expect("start");

        // Interrupt immediately, before any OBSERVE has spent a settle tick.
        fake.interrupt(&handle).expect("interrupt").wait();

        for tick in 0..5 {
            let observed = fake.observe(&handle).expect("observe");
            assert_eq!(
                observed.native,
                NativeState::Exited,
                "tick {tick}: the turn died on interrupt"
            );
            assert_eq!(
                observed.signal,
                BackendSignal::Running,
                "tick {tick}: a StageCompleted this execution never reached must never \
                 surface, on this OBSERVE or any later one — SIGKILL produces no terminal \
                 envelope, ever"
            );
        }
    }

    /// The other half of the same fix: INTERRUPT on an execution whose
    /// signal was already reached (no settle window, or one already spent)
    /// is a true no-op on the signal, matching the trait's own "no turn in
    /// flight is a no-op" contract — this is what stops the fix above from
    /// overreaching into "INTERRUPT always erases the outcome".
    #[test]
    fn interrupt_after_the_signal_is_already_reached_is_a_true_no_op() {
        let fake = FakeBackend::scripted("fake", [FakeStep::complete_with("done")]);
        let handle = fake.start(&request("already-done")).expect("start");

        fake.interrupt(&handle).expect("interrupt").wait();

        let observed = fake.observe(&handle).expect("observe");
        assert_eq!(observed.native, NativeState::Exited);
        assert_eq!(
            observed.signal,
            BackendSignal::StageCompleted {
                summary: Some("done".to_string())
            },
            "settle = 0 means the outcome was already reached — nothing to discard"
        );
    }

    /// The fix is scoped to terminal outcomes only: a park signal
    /// (`NeedsInput`) mid-settle survives an interrupt exactly as it always
    /// has, because a real `post_turn_summary` line streams independently of
    /// the turn's terminal envelope and a kill after it was written can
    /// still leave it standing — unlike `StageCompleted`/`Failed`, which
    /// come only from that envelope.
    #[test]
    fn interrupt_mid_settle_still_lets_a_park_signal_through_once_settled() {
        let fake = FakeBackend::scripted("fake", [FakeStep::needs_input("who?").settle(2)]);
        let handle = fake.start(&request("parked")).expect("start");

        fake.interrupt(&handle).expect("interrupt").wait();

        // The settle window itself is untouched by the interrupt — still two
        // interim ticks before the real signal shows.
        assert_eq!(
            fake.observe(&handle).expect("observe").signal,
            BackendSignal::Running
        );
        assert_eq!(
            fake.observe(&handle).expect("observe").signal,
            BackendSignal::Running
        );
        let observed = fake.observe(&handle).expect("observe");
        assert_eq!(observed.native, NativeState::Exited);
        assert_eq!(
            observed.signal,
            BackendSignal::needs_input("who?"),
            "a park signal is not a terminal envelope — it survives the interrupt \
             even though it had not settled yet either"
        );
    }

    /// R-H0-7's [`FakeStep::settle_as`]: the settle window can script a
    /// native context that has already exited while the signal it will
    /// eventually report is still unresolved — the "deferred terminal
    /// signal" shape a flat [`FakeStep::settle`] (always `(Running,
    /// Running)` mid-window) could not express, and which a naive reader of
    /// `native == Exited` as "the outcome is known" would get wrong.
    #[test]
    fn settle_as_scripts_an_exited_native_context_with_an_unresolved_signal() {
        let fake = FakeBackend::scripted(
            "fake",
            [FakeStep::complete_with("done").settle_as(
                2,
                NativeState::Exited,
                BackendSignal::Running,
            )],
        );
        let handle = fake.start(&request("deferred")).expect("start");

        for tick in 0..2 {
            let observed = fake.observe(&handle).expect("observe");
            assert_eq!(
                observed.native,
                NativeState::Exited,
                "tick {tick}: the native context is already gone"
            );
            assert_eq!(
                observed.signal,
                BackendSignal::Running,
                "tick {tick}: but the result is not resolved yet — native exit and \
                 signal resolution are independently scriptable"
            );
        }
        let settled = fake.observe(&handle).expect("observe");
        assert_eq!(
            settled.native,
            NativeState::Running,
            "the scripted step's own native"
        );
        assert_eq!(
            settled.signal,
            BackendSignal::StageCompleted {
                summary: Some("done".to_string())
            },
            "the real outcome becomes visible once the window is spent"
        );
    }

    /// Every §37 grammar verb parses to its matching [`FakeStep`], both with
    /// a `:detail` and (where the grammar allows it) bare — `parse_script`'s
    /// full dispatch table, not a sample of it.
    #[test]
    fn parse_script_dispatches_every_verb_with_and_without_detail() {
        let steps = parse_script(
            "complete;complete:done;needs_input:who?;waiting:reason;\
             blocked:why;fail:oops;hang",
        );
        assert_eq!(
            steps,
            vec![
                FakeStep::complete(),
                FakeStep::complete_with("done"),
                FakeStep::needs_input("who?"),
                FakeStep::waiting("reason"),
                FakeStep::blocked("why"),
                FakeStep::fail("oops"),
                FakeStep::hang(),
            ]
        );
    }

    /// Whitespace around `;` and `:` is trimmed, empty segments (a leading,
    /// trailing, or doubled `;`) are dropped rather than producing a bogus
    /// step, and an unknown verb — bare or with a `:detail` — is ignored
    /// rather than silently mistaken for a known one (documented at
    /// [`FakeBackend::from_env`]: "a typo must not quietly change what the
    /// demo demonstrates").
    #[test]
    fn parse_script_trims_whitespace_drops_empty_segments_and_ignores_unknown_verbs() {
        let steps =
            parse_script("  complete:done  ; ; unknown:whatever ; needs_input: who? ; typo ;  ");
        assert_eq!(
            steps,
            vec![
                FakeStep::complete_with("done"),
                FakeStep::needs_input("who?")
            ]
        );
    }

    /// The empty script parses to no steps at all — the degenerate case a
    /// `SGT_FAKE_SCRIPT=""` or an unset variable produces.
    #[test]
    fn parse_script_of_an_empty_string_is_no_steps() {
        assert_eq!(parse_script(""), Vec::new());
        assert_eq!(parse_script("   ; ;  "), Vec::new());
    }

    /// scripts/perf/README.md ("What the fake script means"): the parsed
    /// steps are one global FIFO shared by the whole backend, not a queue
    /// per execution — one step consumed per START and one per SEND, in
    /// parse order, regardless of which execution asks. This is the
    /// perf harness's documented, measured behavior spec for the backend
    /// this test module is scoped to, so it is pinned here directly rather
    /// than only indirectly through the shell harness.
    #[test]
    fn parsed_steps_are_one_global_fifo_shared_across_executions_and_sends() {
        let fake = FakeBackend::scripted(
            "fake",
            parse_script("needs_input:q1;needs_input:q2;complete:done"),
        );
        // START for two different executions draws two different steps from
        // the same queue — the second START does not restart at the front.
        let e1 = fake.start(&request("e1")).expect("start pops q1");
        let e2 = fake.start(&request("e2")).expect("start pops q2");
        assert_eq!(
            fake.observe(&e1).expect("observe").signal,
            BackendSignal::needs_input("q1")
        );
        assert_eq!(
            fake.observe(&e2).expect("observe").signal,
            BackendSignal::needs_input("q2")
        );

        // SEND draws from that same shared queue too: the next scripted step
        // goes to whichever execution's SEND asks next, here e1's, even
        // though e2 was the one still holding an unresolved `needs_input`.
        fake.send(&e1, "answer").expect("send pops the last step");
        assert_eq!(
            fake.observe(&e1).expect("observe").signal,
            BackendSignal::StageCompleted {
                summary: Some("done".to_string())
            }
        );

        // The script is now exhausted; e2's own SEND falls through to the
        // documented default (§37: "a script that runs out completes every
        // stage after it") rather than reusing or blocking on e1's step.
        fake.send(&e2, "answer")
            .expect("send falls through to the default");
        assert_eq!(
            fake.observe(&e2).expect("observe").signal,
            BackendSignal::StageCompleted { summary: None }
        );
    }

    /// §15's capability flags are advertised, and an unsupported capability is
    /// advertised as `false` rather than emulated — checked against what the
    /// verbs actually do, which is the only way a flag can be wrong.
    #[test]
    fn capabilities_are_advertised_and_never_emulated() {
        let fake = FakeBackend::new("fake");
        let capabilities = fake.capabilities();
        assert!(capabilities.persistent_sessions);
        assert!(capabilities.resume);
        assert!(!capabilities.streaming);
        // HISTORY is advertised because the fake can honor the claim: it
        // answers with an execution's whole history or refuses to recognise
        // it, and never with a prefix that reads like the whole thing.
        assert!(capabilities.history);
        let handle = fake.start(&request("e-caps")).expect("start");
        fake.send(&handle, "one").expect("send");
        assert_eq!(fake.history(&handle).expect("history").len(), 1);
        let stranger = ExecutionHandle {
            execution_id: "never-started".to_string(),
            native_id: None,
        };
        assert!(
            matches!(
                fake.history(&stranger),
                Err(BackendError::UnknownExecution { .. })
            ),
            "an execution this backend never had is refused, not reported as an empty history"
        );
    }
}
