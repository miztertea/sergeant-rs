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

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};

use super::{
    Backend, BackendError, Capabilities, ExecutionHandle, NativeState, Observation, ProbeReport,
    StartRequest,
};
use crate::backend::BackendSignal;

/// Name the default registry registers the fake under.
pub const FAKE_BACKEND_NAME: &str = "fake";

/// One scripted execution's programmed behaviour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeStep {
    /// Native evidence this execution reports.
    pub native: NativeState,
    /// Explicit signal this execution reports.
    pub signal: BackendSignal,
    /// Whether the native context ignores STOP (a hang).
    pub ignores_stop: bool,
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

    /// Asks for human input.
    pub fn needs_input(prompt: &str) -> Self {
        Self::running(BackendSignal::NeedsInput {
            prompt: prompt.to_string(),
        })
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

    fn running(signal: BackendSignal) -> Self {
        Self {
            native: NativeState::Running,
            signal,
            ignores_stop: false,
        }
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
}

#[derive(Debug)]
struct FakeState {
    script: VecDeque<FakeStep>,
    executions: BTreeMap<String, FakeExecution>,
    starts: Vec<StartRequest>,
    stop_requests: Vec<String>,
    observations: Vec<String>,
    available: bool,
    detail: Option<String>,
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
                resume: true,
                model_selection: true,
                profiles: true,
                ..Capabilities::default()
            },
            state: Arc::new(Mutex::new(FakeState {
                script: script.into_iter().collect(),
                executions: BTreeMap::new(),
                starts: Vec::new(),
                stop_requests: Vec::new(),
                observations: Vec::new(),
                available: true,
                detail: Some("deterministic in-process test backend".to_string()),
            })),
        }
    }

    /// Make PROBE report unavailable (routing must then fail closed).
    pub fn set_available(&self, available: bool, detail: &str) {
        let mut state = self.lock();
        state.available = available;
        state.detail = Some(detail.to_string());
    }

    /// Every START request this backend received, in order.
    pub fn starts(&self) -> Vec<StartRequest> {
        self.lock().starts.clone()
    }

    /// Execution ids STOP was requested for, in order.
    pub fn stop_requests(&self) -> Vec<String> {
        self.lock().stop_requests.clone()
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

    fn probe(&self) -> ProbeReport {
        let state = self.lock();
        ProbeReport {
            available: state.available,
            detail: state.detail.clone(),
        }
    }

    fn start(&self, request: &StartRequest) -> Result<ExecutionHandle, BackendError> {
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
        state.starts.push(request.clone());
        let native_id = format!("fake-session-{}", request.execution_id);
        state.executions.insert(
            request.execution_id.clone(),
            FakeExecution {
                native_id: native_id.clone(),
                step,
                inputs: Vec::new(),
                stopped: false,
            },
        );
        Ok(ExecutionHandle {
            execution_id: request.execution_id.clone(),
            native_id: Some(native_id),
        })
    }

    fn send(&self, handle: &ExecutionHandle, input: &str) -> Result<(), BackendError> {
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
        Ok(())
    }

    fn observe(&self, handle: &ExecutionHandle) -> Result<Observation, BackendError> {
        let mut state = self.lock();
        state.observations.push(handle.execution_id.clone());
        let execution = self.resolve(&state, handle)?;
        Ok(Observation {
            native: execution.step.native,
            signal: execution.step.signal.clone(),
            evidence: Some(format!(
                "fake backend: native={}, stopped={}",
                execution.step.native.as_str(),
                execution.stopped
            )),
        })
    }

    fn stop(&self, handle: &ExecutionHandle) -> Result<(), BackendError> {
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
        Ok(())
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
            BackendSignal::NeedsInput {
                prompt: "who?".to_string()
            }
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

        fake.stop(&hanging).expect("stop");
        fake.stop(&compliant).expect("stop");
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
                fake.stop(&forged),
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

    /// §15's capability flags are advertised, and an unsupported capability is
    /// advertised as `false` rather than emulated.
    #[test]
    fn capabilities_are_advertised_and_never_emulated() {
        let capabilities = FakeBackend::new("fake").capabilities();
        assert!(capabilities.persistent_sessions);
        assert!(capabilities.resume);
        assert!(!capabilities.history);
        assert!(!capabilities.streaming);
    }
}
