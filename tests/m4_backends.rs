//! M4 acceptance tests (docs/gauntlet/contracts/M4.md), counted against the
//! contract's own list — **six items**: acceptance 5 (Codex fixture tests)
//! was dropped with deviation D6, along with the adapter it tested.
//!
//! 1. Contract test, opt-in, real Claude: start → session_id captured; a
//!    second send resumes the same session (continuity proven by nonce
//!    recall); raw stream-json lines in the blob store; normalized events
//!    journaled with correlation/causation.
//! 2. Model pin: valid alias honored (live half inside test 1; recorded
//!    honored fixture deterministic); provider-qualified rejected
//!    pre-flight (at START *and* at RESUME); substitution detected from an
//!    envelope fixture *derived* from the recorded one by three documented
//!    edits — the recording the contract asks for cannot exist here, and
//!    `tests/fixtures/README.md` says exactly what was changed and why.
//! 3. Interrupt: mid-turn process kill leaves work state untouched and the
//!    conversation resumable — deterministically against a stub that hangs
//!    mid-stream, and live (opt-in) against the installed CLI.
//! 4. Recovery: restart reconciliation from real session evidence
//!    (deterministic via a fabricated `claude_home` and real process
//!    liveness; exercised live inside test 1's re-adoption step). §25's
//!    sequence in order — **reattach** through §15 RESUME, then classify:
//!    existing transcript and no live turn → re-adopted and blocked with
//!    resumable evidence; a turn still running → reported running, never
//!    "exited", and refused for adoption; missing transcript → execution
//!    retired, work blocked with evidence, never silently failed.
//! 6. Regression catalog (§37, from Sergeant): seven named tests below,
//!    each citing its origin, green against the fake backend.
//! 7. Version gate: the adapter refuses (fail closed, structured error
//!    naming the probe) when required CLI capabilities are absent.
//!
//! Everything the deterministic tests assert about the *launch grammar* and
//! about stream-json ingestion is asserted against what the adapter actually
//! ran: [`StubClaude`] records every argv, environment variable and prompt it
//! is launched with, and replays a **recorded** transcript
//! (`tests/fixtures/claude-2.1.226-turn.jsonl`, verbatim lines from a real
//! 2.1.226 turn) on stdout. A stub that answered nothing would leave D2's
//! whole grammar — the thing this milestone exists to prove — unpinned.
//!
//! The two real-Claude tests are `#[ignore]`d *and* env-gated: they cost
//! tokens, so they must never report `ok` for a run that did nothing. Run
//! them with
//! `SERGEANT_CLAUDE_TESTS=1 cargo test --test m4_backends -- --ignored`.
//! Dropping the variable from that command is an error, not a silent skip
//! (see [`claude_live_enabled`]); an unauthenticated or unusable CLI is a
//! skip, announced on the real file descriptor 2 so libtest cannot swallow
//! it. Between them the two live tests exercise every §37 backend-contract
//! verb against the installed binary — probe, start, identity, send,
//! observe, history, interrupt, resume, stop, restart/reconcile.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::api::Core;
use sergeant_rs::backend::claude::{
    CLAUDE_BACKEND_NAME, ClaudeBackend, ClaudeConfig, PinVerdict, preflight_model_pin,
    verify_model_pin,
};
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::backend::{
    Backend, BackendError, BackendRegistry, BackendSignal, ExecutionHandle, NativeState,
    ProbeReport, ResumeRequest, RuntimeScope, StartRequest,
};
use sergeant_rs::daemon::{self, DaemonConfig, journaling_sink};
use sergeant_rs::domain::event::{Event, EventDraft, EventSource};
use sergeant_rs::domain::execution::{
    KIND_EXECUTION_RECONCILED, KIND_EXECUTION_STARTED, KIND_EXECUTION_STOPPED,
};
use sergeant_rs::domain::profile::Profile;
use sergeant_rs::domain::work::{
    KIND_WORK_BLOCKED, KIND_WORK_FAILED, KIND_WORK_NEEDS_INPUT, KIND_WORK_RESUMED,
    KIND_WORK_STARTED, KIND_WORK_SUBMITTED, WorkState,
};
use sergeant_rs::domain::workflow::{
    KIND_STAGE_ENTERED, KIND_STAGE_INPUT_RECEIVED, KIND_STAGE_NEEDS_INPUT, KIND_WORKFLOW_BOUND,
};
use sergeant_rs::runtime::blob::{BlobRef, BlobStore};
use sergeant_rs::runtime::engine::Engine;
use sergeant_rs::runtime::journal::Journal;
use sergeant_rs::runtime::projection::work_registry_projection;
use sergeant_rs::runtime::recovery;
use sergeant_rs::runtime::surface::KIND_SURFACE_MATERIALIZED;

// ---------------------------------------------------------------- helpers

fn ulid() -> String {
    ulid::Ulid::generate().to_string()
}

/// A fresh v4-shaped session id, unique per call.
///
/// Tests that put a session id into a real process's argv, or into a
/// fabricated transcript, must not share one: two runs of this binary
/// overlapping (the gauntlet machine, CI with a second job, a `--test-threads`
/// re-run) would otherwise each see the *other's* stand-in process carrying
/// "their" id, and a liveness assertion would flip. A hard-coded id makes
/// that collision certain rather than unlikely.
fn fresh_session_id() -> String {
    let hex: String = ulid::Ulid::generate()
        .to_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!(
        "{}-{}-4{}-8{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[13..16],
        &hex[17..20],
        &hex[20..32]
    )
}

/// Spawn a stand-in for a turn of `session_id`: a live process whose argv is
/// exactly the shape this adapter launches (`claude … --resume <uuid>`), and
/// which **forks nothing**.
///
/// The no-fork part is load-bearing for any test that asserts a *negative*
/// about liveness. A child between `fork` and `exec` still shows the parent's
/// argv in `/proc`, so a stand-in that runs an external command (`sleep`)
/// transiently presents a *second* process carrying the session id — a
/// property of the stand-in, not of the adapter, and one that turned an
/// assertion about the adapter into a coin flip under load (measured: 3
/// failures in 12 runs of this file with the machine busy). Blocking on a
/// shell builtin reading from a pipe the test holds open keeps one process,
/// with the argv the test cares about, until it is killed.
fn spawn_turn_stand_in(session_id: &str) -> std::process::Child {
    let child = std::process::Command::new("sh")
        .args(["-c", "read line", "claude", "--resume", session_id])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("spawn a stand-in for a turn");
    // `spawn` returns before the child has exec'd, and until it does its
    // /proc entry still shows this test binary's argv. Wait for the real
    // thing, so a test measures the adapter and not the fork.
    let cmdline = PathBuf::from(format!("/proc/{}/cmdline", child.id()));
    let deadline = Instant::now() + Duration::from_secs(30);
    while !String::from_utf8_lossy(&std::fs::read(&cmdline).unwrap_or_default())
        .contains(session_id)
    {
        assert!(Instant::now() < deadline, "the stand-in never exec'd");
        std::thread::sleep(Duration::from_millis(10));
    }
    child
}

/// What the installed CLI says about authentication (`claude auth status
/// --json`, measured on 2.1.226 in this container:
/// `{"loggedIn": true, "authMethod": "oauth_token", "apiProvider":
/// "firstParty"}`). Token-free and offline, so the gate costs nothing.
#[derive(Debug, PartialEq, Eq)]
enum AuthStatus {
    /// The CLI reports a logged-in account.
    LoggedIn,
    /// The CLI reports no account.
    LoggedOut,
    /// The question could not be answered (no binary, unparseable answer).
    Unreadable(String),
}

fn claude_auth_status(executable: &Path) -> AuthStatus {
    let output = match std::process::Command::new(executable)
        .args(["auth", "status", "--json"])
        .output()
    {
        Ok(output) => output,
        Err(e) => return AuthStatus::Unreadable(format!("cannot run {executable:?} auth: {e}")),
    };
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    match serde_json::from_str::<Value>(&text) {
        Ok(status) => match status["loggedIn"].as_bool() {
            Some(true) => AuthStatus::LoggedIn,
            Some(false) => AuthStatus::LoggedOut,
            None => AuthStatus::Unreadable(format!("no loggedIn field in {text:?}")),
        },
        Err(e) => AuthStatus::Unreadable(format!("auth status is not JSON ({e}): {text:?}")),
    }
}

/// Whether the opt-in real-Claude tests may run, and if not, which of the
/// contract's two skip conditions applies.
#[derive(Debug, PartialEq, Eq)]
enum LiveGate {
    /// Opted in, CLI usable, account authenticated.
    Run,
    /// `SERGEANT_CLAUDE_TESTS` is not `1`.
    NotOptedIn,
    /// Opted in, but the harness cannot serve the tests.
    Unusable(String),
}

/// The gate's whole decision, as a pure function of its three inputs, so the
/// branches can be pinned without an environment (see the unit test at the
/// bottom of this file).
fn live_gate(opt_in: Option<&str>, probe: &ProbeReport, auth: AuthStatus) -> LiveGate {
    if opt_in != Some("1") {
        return LiveGate::NotOptedIn;
    }
    if !probe.available {
        return LiveGate::Unusable(format!(
            "the installed claude does not pass the adapter's probe: {}",
            probe.detail.clone().unwrap_or_default()
        ));
    }
    match auth {
        AuthStatus::LoggedIn => LiveGate::Run,
        AuthStatus::LoggedOut => LiveGate::Unusable(
            "the installed claude reports no logged-in account (claude auth status --json: \
             loggedIn=false); these tests need a real conversation"
                .to_string(),
        ),
        AuthStatus::Unreadable(why) => LiveGate::Unusable(format!(
            "cannot establish that the installed claude is authenticated: {why}"
        )),
    }
}

/// Whether the opt-in real-Claude tests may run (budget-conscious: they
/// spend real haiku turns). The M4 contract asks for two skip conditions —
/// the env var unset, *and* an unauthenticated CLI — and they are answered
/// differently on purpose:
///
/// - **unset**: `#[ignore]` already skips these in every ordinary run, and
///   that is the clean skip the contract asks for (the report says
///   `ignored`, not `ok`). Reaching this function with the var unset means
///   someone ran the documented `-- --ignored` command *without* opting in,
///   and returning early there is exactly the false green `#[ignore]` exists
///   to prevent: libtest captures the notice and prints `ok` for a test that
///   did nothing. So that case panics, naming the opt-in. No tokens are ever
///   spent by a panic.
/// - **unauthenticated / unusable CLI**: the contract asks for a clean skip,
///   so the test returns early — and the notice is written straight to file
///   descriptor 2 rather than through `eprintln!`, because libtest's capture
///   only intercepts the print macros. A skip nobody can see is the same
///   false green in a different costume.
fn claude_live_enabled(test: &str) -> bool {
    let config = live_config(Path::new("/nonexistent-probe-only"));
    let probe = ClaudeBackend::new(config.clone()).probe();
    let gate = live_gate(
        std::env::var("SERGEANT_CLAUDE_TESTS").ok().as_deref(),
        &probe,
        claude_auth_status(&config.executable),
    );
    match gate {
        LiveGate::Run => true,
        LiveGate::NotOptedIn => panic!(
            "{test} is opt-in and spends real tokens: run it with \
             SERGEANT_CLAUDE_TESTS=1 cargo test --test m4_backends -- --ignored. \
             (Without the variable these tests are skipped by #[ignore]; asking for \
             --ignored without it must not report a green test that did nothing.)"
        ),
        LiveGate::Unusable(why) => {
            let _ = std::io::stderr()
                .write_all(format!("SKIPPED {test}: {why}\n").as_bytes())
                .and_then(|()| std::io::stderr().flush());
            false
        }
    }
}

/// An empty core over `data_dir` (mirror of the crate's internal test
/// fixture; integration tests cannot import `runtime::testing`).
fn core(data_dir: &Path) -> Core {
    let journal = Journal::open(data_dir).expect("journal");
    let mut registry = work_registry_projection();
    registry
        .catch_up(journal.replay().expect("replay"))
        .expect("catch up");
    let (events_tx, _rx) = tokio::sync::broadcast::channel(16);
    Core {
        journal,
        registry,
        events_tx,
    }
}

fn commit(core: &mut Core, work_id: &str, kind: &str, payload: Value) {
    core.commit(
        EventDraft::new(EventSource::new("daemon", "test"), kind, payload).with_work_id(work_id),
    )
    .expect("commit");
}

fn submit_work(core: &mut Core, work_id: &str, intent: &str) {
    commit(
        core,
        work_id,
        KIND_WORK_SUBMITTED,
        json!({"work": {
            "id": work_id,
            "intent": intent,
            "state": "pending",
            "created_by": "test",
            "created_at": "2026-01-01T00:00:00Z",
        }}),
    );
}

/// Journal a minimal one-stage run bound to `backend` with a recorded
/// execution, leaving the work `active` — the state restart reconciliation
/// acts on.
fn journal_active_run(core: &mut Core, work_id: &str, backend: &str, native_id: &str) {
    submit_work(core, work_id, "in flight across a restart");
    commit(
        core,
        work_id,
        KIND_WORKFLOW_BOUND,
        json!({
            "workflow": {"name": "tiny", "version": "1", "source": "test",
                         "stages": [{"id": "00-only", "context": "c"}]},
            "backend": backend,
        }),
    );
    commit(
        core,
        work_id,
        KIND_STAGE_ENTERED,
        json!({"stage_id": "00-only", "index": 0, "attempt": 1}),
    );
    commit(
        core,
        work_id,
        KIND_EXECUTION_STARTED,
        json!({"execution": {
            "execution_id": format!("exec-{work_id}"),
            "backend": backend,
            "native_id": native_id,
            "stage_id": "00-only",
            "attempt": 1,
            "stop_requested": false,
        }}),
    );
    commit(core, work_id, KIND_WORK_STARTED, json!({}));
}

/// Journal a materialized single-repository surface for a work, so restart
/// reconciliation has the directory later turns would run in.
///
/// Only the record matters here — nothing in reconciliation touches the
/// worktree — but the record is load-bearing: §25's reattach step re-supplies
/// the launch configuration from what sergeant journaled, and the work
/// surface is part of it. `journal_active_run` deliberately leaves it out, so
/// tests can cover both shapes.
fn journal_surface(core: &mut Core, work_id: &str, cwd: &Path) {
    commit(
        core,
        work_id,
        KIND_SURFACE_MATERIALIZED,
        json!({"surface": {
            "work_id": work_id,
            "root": cwd,
            "bindings": [{
                "repository": "solo",
                "source_path": cwd,
                "base_branch": "main",
                "base_sha": "0".repeat(40),
                "worktree_path": cwd,
                "work_branch": format!("sergeant/{work_id}"),
                "head_sha": "0".repeat(40),
            }],
        }}),
    );
}

/// Payloads of one work's events of one kind, in journal order.
fn events_of(core: &Core, work_id: &str, kind: &str) -> Vec<Value> {
    core.journal
        .replay()
        .expect("replay")
        .map(|e| e.expect("event"))
        .filter(|e| e.kind == kind && e.work_id.as_deref() == Some(work_id))
        .map(|e| e.payload)
        .collect()
}

/// All journaled events, for envelope-level assertions.
fn all_events(core: &Core) -> Vec<Event> {
    core.journal
        .replay()
        .expect("replay")
        .map(|e| e.expect("event"))
        .collect()
}

/// Wait for at least `count` journaled events for one execution.
///
/// The journaling sink commits on its own thread — deliberately, so an
/// adapter emitting from the request path can never block or panic there
/// (see `daemon::journaling_sink`) — so a test that read the journal the
/// instant after an emit would be racing the committer, not asserting on it.
fn wait_for_events(
    shared: &Arc<tokio::sync::Mutex<Core>>,
    execution_id: &str,
    count: usize,
) -> Vec<Event> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let mine: Vec<Event> = {
            let core = shared.blocking_lock();
            all_events(&core)
                .into_iter()
                .filter(|e| e.execution_id.as_deref() == Some(execution_id))
                .collect()
        };
        if mine.len() >= count {
            return mine;
        }
        assert!(
            Instant::now() < deadline,
            "only {} of {count} events for {execution_id}: {:?}",
            mine.len(),
            mine.iter().map(|e| &e.kind).collect::<Vec<_>>()
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// A stub `claude` that answers the capability probe, **records every
/// launch**, and replays a recorded transcript on stdout.
///
/// The recording half is what makes the launch grammar testable at all: the
/// adapter's argv is the D2 contract (`-p --verbose --output-format
/// stream-json`, `--setting-sources user`, `--session-id` then `--resume`,
/// the permission mode), and a stub that only exits 0 leaves every one of
/// those mutable with the suite green.
struct StubClaude {
    /// Path to hand the adapter as its executable.
    path: PathBuf,
    /// Where launches are appended.
    record: PathBuf,
    /// File whose contents are written to stdout on a turn invocation.
    replay: PathBuf,
    /// While this file exists, a turn invocation `exec`s a long sleep after
    /// replaying — a turn that is genuinely still in flight.
    hang: PathBuf,
}

/// One recorded launch of [`StubClaude`].
#[derive(Debug)]
struct Launch {
    argv: Vec<String>,
    env: std::collections::BTreeMap<String, String>,
    cwd: String,
    stdin: String,
}

impl Launch {
    /// The value the adapter passed after `flag`, if it passed the flag.
    fn value_of(&self, flag: &str) -> Option<&str> {
        let index = self.argv.iter().position(|arg| arg == flag)?;
        self.argv.get(index + 1).map(String::as_str)
    }

    fn has(&self, flag: &str) -> bool {
        self.argv.iter().any(|arg| arg == flag)
    }
}

/// Environment variables the stub reports back, chosen because the adapter
/// makes a claim about each one.
const RECORDED_ENV: &[&str] = &["CLAUDE_CODE_SESSION_ID", "CLAUDE_CONFIG_DIR", "IS_SANDBOX"];

impl StubClaude {
    fn new(dir: &Path, version: &str, help_flags: &[&str]) -> Self {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join("claude-stub");
        let record = dir.join("claude-launches.txt");
        let replay = dir.join("claude-replay.jsonl");
        let hang = dir.join("claude-hang");
        let help = help_flags.join(" ");
        let env_lines: String = RECORDED_ENV
            .iter()
            .map(|name| format!("      printf 'env {name}=%s\\n' \"${{{name}-<unset>}}\";\n"))
            .collect();
        let script = format!(
            "#!/bin/sh\n\
             case \"$1\" in\n  \
               --version) echo \"{version}\";;\n  \
               --help) echo \"{help}\";;\n  \
               *)\n    \
                 {{ for arg in \"$@\"; do printf 'arg %s\\n' \"$arg\"; done;\n\
             {env_lines}      \
                   printf 'cwd %s\\n' \"$(pwd)\";\n      \
                   printf 'stdin %s\\n' \"$(cat | tr '\\n' '|')\";\n      \
                   printf 'end\\n'; }} >> \"{record}\"\n    \
                 if [ -f \"{replay}\" ]; then cat \"{replay}\"; fi\n    \
                 if [ -f \"{hang}\" ]; then exec sleep 30; fi\n    \
                 exit 0;;\n\
             esac\n",
            record = record.display(),
            replay = replay.display(),
            hang = hang.display(),
        );
        std::fs::write(&path, script).expect("write stub");
        let mut permissions = std::fs::metadata(&path).expect("stat stub").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&path, permissions).expect("chmod stub");
        let stub = Self {
            path,
            record,
            replay,
            hang,
        };
        stub.wait_until_executable();
        stub
    }

    /// Run the stub once, retrying `ETXTBSY`, before handing it to a test.
    ///
    /// A file that some process still holds open for writing cannot be
    /// `exec`'d — and every `Command::spawn` on a sibling test thread forks a
    /// child that inherits this test binary's open descriptors for the
    /// fork-to-exec window. Under load (four test threads, this gauntlet's
    /// machine, CI) that window overlaps the write above often enough to turn
    /// "the adapter refuses an unlaunchable CLI" into a false red in a test
    /// about something else entirely. Absorbing it here, once, keeps the
    /// flake out of every stub-based test without weakening any of them: the
    /// adapter's own refusal path is still pinned, deliberately, by
    /// `a_start_that_cannot_spawn_leaves_no_phantom_execution`.
    fn wait_until_executable(&self) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match std::process::Command::new(&self.path)
                .arg("--version")
                .output()
            {
                Ok(_) => return,
                Err(e) if e.raw_os_error() == Some(26) => {
                    assert!(
                        Instant::now() < deadline,
                        "the stub stayed ETXTBSY for 10s: {e}"
                    );
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(e) => panic!("the stub is not runnable: {e}"),
            }
        }
    }

    /// Make later turn invocations replay their transcript and then stay
    /// alive (`exec sleep`) until they are killed — a turn genuinely in
    /// flight, which is what INTERRUPT and STOP need in order to have
    /// anything to kill.
    fn hangs_after_replay(&self) -> &Self {
        std::fs::write(&self.hang, b"hang\n").expect("write hang marker");
        self
    }

    /// A stub that passes the probe with every required flag.
    fn passing(dir: &Path) -> Self {
        Self::new(dir, "2.1.226 (Claude Code)", ALL_FLAGS)
    }

    /// Make turn invocations replay `transcript` on stdout.
    fn replays(&self, transcript: &str) -> &Self {
        std::fs::write(&self.replay, transcript).expect("write replay");
        self
    }

    /// Every turn launch so far, in order.
    fn launches(&self) -> Vec<Launch> {
        let Ok(text) = std::fs::read_to_string(&self.record) else {
            return Vec::new();
        };
        let mut launches = Vec::new();
        let mut current = Launch {
            argv: Vec::new(),
            env: std::collections::BTreeMap::new(),
            cwd: String::new(),
            stdin: String::new(),
        };
        for line in text.lines() {
            match line.split_once(' ') {
                Some(("arg", value)) => current.argv.push(value.to_string()),
                Some(("env", value)) => {
                    let (name, value) = value.split_once('=').expect("env record shape");
                    current.env.insert(name.to_string(), value.to_string());
                }
                Some(("cwd", value)) => current.cwd = value.to_string(),
                Some(("stdin", value)) => current.stdin = value.to_string(),
                _ if line == "end" => launches.push(std::mem::replace(
                    &mut current,
                    Launch {
                        argv: Vec::new(),
                        env: std::collections::BTreeMap::new(),
                        cwd: String::new(),
                        stdin: String::new(),
                    },
                )),
                _ => {}
            }
        }
        launches
    }

    /// Block until `count` launches have been recorded (the reader thread
    /// finishes asynchronously), then return them.
    fn wait_for_launches(&self, count: usize) -> Vec<Launch> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let launches = self.launches();
            if launches.len() >= count {
                return launches;
            }
            assert!(
                Instant::now() < deadline,
                "only {} of {count} launches recorded",
                launches.len()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

/// Four verbatim stream-json lines from a real 2.1.226 turn (provenance in
/// `tests/fixtures/README.md`): a `tool_use`, its `tool_result`, the
/// assistant's text, and the result envelope with `modelUsage`.
const RECORDED_TURN: &str = include_str!("fixtures/claude-2.1.226-turn.jsonl");

/// The substitution envelope, **derived** from the recorded one by three
/// named edits (`tests/fixtures/README.md` documents each). Not a recording:
/// print-mode substitution cannot be provoked on an entitled account, and the
/// spike's evidence for it is a TUI warning line, not a result envelope.
const SUBSTITUTION_ENVELOPE: &str =
    include_str!("fixtures/claude-2.1.226-substitution-envelope.derived.json");

/// The recorded turn minus its result envelope: what a turn killed
/// mid-generation leaves on stdout (measured — a SIGKILLed turn emits no
/// `type:"result"` line, and the conversation stays resumable).
fn partial_turn() -> String {
    RECORDED_TURN
        .lines()
        .take(3)
        .map(|line| format!("{line}\n"))
        .collect()
}

/// Every flag the adapter's probe requires, for stubs that should pass.
const ALL_FLAGS: &[&str] = &[
    "--print",
    "--verbose",
    "--output-format",
    "--session-id",
    "--resume",
    "--setting-sources",
    "--model",
    "--permission-mode",
    "--dangerously-skip-permissions",
];

fn start_request(
    execution_id: &str,
    cwd: &Path,
    intent: &str,
    model: Option<&str>,
) -> StartRequest {
    StartRequest {
        work_id: format!("work-{execution_id}"),
        execution_id: execution_id.to_string(),
        stage_id: "00-only".to_string(),
        attempt: 1,
        cwd: cwd.to_path_buf(),
        intent: intent.to_string(),
        context: String::new(),
        model: model.map(str::to_string),
        profile: None,
    }
}

/// Poll OBSERVE until the turn is no longer in flight (native != Running),
/// panicking on timeout. Returns the settled observation.
fn wait_settled(
    backend: &ClaudeBackend,
    handle: &ExecutionHandle,
    timeout: Duration,
) -> sergeant_rs::backend::Observation {
    let deadline = Instant::now() + timeout;
    loop {
        let observation = backend.observe(handle).expect("observe");
        if observation.native != NativeState::Running {
            return observation;
        }
        assert!(
            Instant::now() < deadline,
            "turn did not settle within {timeout:?}: {observation:?}"
        );
        std::thread::sleep(Duration::from_millis(500));
    }
}

/// The live-adapter config for this container: real `claude`, scratch data
/// dir, `IS_SANDBOX=1` because the tests run as root (module docs in
/// `backend::claude` record the measured refusal without it).
fn live_config(data_dir: &Path) -> ClaudeConfig {
    let mut config = ClaudeConfig::new(data_dir);
    config.env.insert("IS_SANDBOX".to_string(), "1".to_string());
    config
}

/// The daemon's journaling sink chains causation deterministically: within
/// one execution, each committed event's `causation_id` is the id the
/// journal assigned to the previous one, and `correlation_id` rides
/// through untouched. (The live contract test asserts the same properties
/// end to end; this pins them without tokens.)
#[test]
fn the_journaling_sink_chains_causation_within_an_execution() {
    let data = TempDir::new().expect("tempdir");
    let shared = Arc::new(tokio::sync::Mutex::new(core(data.path())));
    let sink = journaling_sink(shared.clone());

    for (kind, text) in [
        ("conversation.user", "first"),
        ("conversation.assistant.completed", "second"),
        ("usage.updated", "third"),
    ] {
        sink(EventDraft {
            source: EventSource::new("backend", "claude"),
            workspace_id: None,
            work_id: Some("w1".to_string()),
            execution_id: Some("e1".to_string()),
            correlation_id: Some("e1".to_string()),
            causation_id: None,
            kind: kind.to_string(),
            payload: json!({"text": text}),
        });
    }
    // A different execution starts its own chain.
    sink(EventDraft {
        source: EventSource::new("backend", "claude"),
        workspace_id: None,
        work_id: Some("w2".to_string()),
        execution_id: Some("e2".to_string()),
        correlation_id: Some("e2".to_string()),
        causation_id: None,
        kind: "conversation.user".to_string(),
        payload: json!({"text": "other"}),
    });

    let e1 = wait_for_events(&shared, "e1", 3);
    let e1: Vec<&Event> = e1.iter().collect();
    assert_eq!(e1.len(), 3);
    assert_eq!(e1[0].causation_id, None, "a chain starts unprovoked");
    assert_eq!(e1[1].causation_id.as_deref(), Some(e1[0].id.as_str()));
    assert_eq!(e1[2].causation_id.as_deref(), Some(e1[1].id.as_str()));
    assert!(e1.iter().all(|e| e.correlation_id.as_deref() == Some("e1")));
    let e2 = wait_for_events(&shared, "e2", 1);
    assert_eq!(
        e2[0].causation_id, None,
        "chains never leak across executions"
    );
}

/// The committer's causation chain is bounded, so a long-lived daemon does
/// not accumulate one map entry per execution it ever ran.
///
/// Nothing in the event stream says when an execution is finished — a
/// conversation can be sent to again at any time — so the entry cannot be
/// evicted on a signal; it is evicted by age. The consequence is stated
/// rather than hidden: past the bound, the oldest execution's next event
/// starts a fresh chain, exactly as it would after a daemon restart, while
/// correlation (which is never forgotten) still groups the execution's
/// events for all time.
#[test]
fn the_sinks_causation_chain_is_bounded() {
    let data = TempDir::new().expect("tempdir");
    let shared = Arc::new(tokio::sync::Mutex::new(core(data.path())));
    let sink = journaling_sink(shared.clone());
    let emit = |execution: &str, text: &str| {
        sink(EventDraft {
            source: EventSource::new("backend", "claude"),
            workspace_id: None,
            work_id: Some("w".to_string()),
            execution_id: Some(execution.to_string()),
            correlation_id: Some(execution.to_string()),
            causation_id: None,
            kind: "conversation.user".to_string(),
            payload: json!({"text": text}),
        });
    };

    let oldest = "exec-0000";
    emit(oldest, "first");
    wait_for_events(&shared, oldest, 1);
    // One event each for enough later executions to push the first out.
    for index in 1..=daemon::SINK_CHAIN_CAPACITY {
        emit(&format!("exec-{index:04}"), "only");
    }
    let newest = format!("exec-{:04}", daemon::SINK_CHAIN_CAPACITY);
    wait_for_events(&shared, &newest, 1);

    emit(oldest, "second");
    emit(&newest, "second");
    let oldest_events = wait_for_events(&shared, oldest, 2);
    assert_eq!(
        oldest_events[1].causation_id, None,
        "past the bound the oldest chain is forgotten, not remembered forever"
    );
    assert_eq!(
        oldest_events[1].correlation_id.as_deref(),
        Some(oldest),
        "correlation is never forgotten: the execution's events still group"
    );
    let newest_events = wait_for_events(&shared, &newest, 2);
    assert_eq!(
        newest_events[1].causation_id.as_deref(),
        Some(newest_events[0].id.as_str()),
        "and a chain inside the bound still chains"
    );
}

/// The opt-in gate's whole decision, pinned without an environment.
///
/// The contract asks for two skip conditions — the env var unset, *and* an
/// unauthenticated CLI — and the second was simply unimplemented: with the
/// variable set on an unauthenticated container the live tests ran into
/// `panic!("first turn must complete")` instead of skipping. The gate now
/// consults `claude auth status --json` (measured on 2.1.226:
/// `{"loggedIn": true, ...}`), which is offline and token-free.
#[test]
fn the_live_gate_skips_on_both_of_the_contracts_conditions() {
    let available = ProbeReport {
        available: true,
        detail: Some("claude 2.1.226".to_string()),
    };
    let unavailable = ProbeReport {
        available: false,
        detail: Some("capability probe: cannot run \"claude\" --version".to_string()),
    };

    assert_eq!(
        live_gate(Some("1"), &available, AuthStatus::LoggedIn),
        LiveGate::Run
    );
    assert_eq!(
        live_gate(None, &available, AuthStatus::LoggedIn),
        LiveGate::NotOptedIn,
        "no opt-in, no tokens — and the caller turns this into a loud failure \
         rather than a green test that did nothing"
    );
    assert_eq!(
        live_gate(Some("0"), &available, AuthStatus::LoggedIn),
        LiveGate::NotOptedIn
    );
    match live_gate(Some("1"), &available, AuthStatus::LoggedOut) {
        LiveGate::Unusable(why) => assert!(why.contains("no logged-in account"), "{why}"),
        other => panic!("an unauthenticated CLI must skip, got {other:?}"),
    }
    match live_gate(
        Some("1"),
        &available,
        AuthStatus::Unreadable("auth status is not JSON".to_string()),
    ) {
        LiveGate::Unusable(why) => assert!(
            why.contains("cannot establish"),
            "an unanswerable question is not a yes: {why}"
        ),
        other => panic!("expected a skip, got {other:?}"),
    }
    match live_gate(Some("1"), &unavailable, AuthStatus::LoggedIn) {
        LiveGate::Unusable(why) => assert!(why.contains("cannot run"), "{why}"),
        other => panic!("a CLI the probe refuses cannot serve these tests, got {other:?}"),
    }
}

/// The sink is called from the daemon's own request path, and that is the
/// case no fake can stand in for.
///
/// `POST /v1/work` takes the core lock on a tokio worker, and the engine
/// calls `Backend::start` while holding it; the Claude adapter spawns the
/// turn's process and then emits `conversation.user` on that same thread. A
/// sink that locked the core there would panic inside the runtime (or wait
/// on the lock its own caller holds) — *after* the native process exists and
/// *before* `execution.started` is journaled. That is an orphaned,
/// token-burning process with no durable record: an L6 window opened by the
/// event path. This test drives the real adapter (over a stub binary)
/// through the real daemon, which is the only configuration where that
/// window is reachable.
#[tokio::test]
async fn the_real_adapter_journals_from_the_daemon_request_path() {
    let data = TempDir::new().expect("tempdir");
    let repo = TempDir::new().expect("repo");
    init_repo(repo.path());
    let stub = StubClaude::passing(data.path());
    stub.replays(RECORDED_TURN);
    let mut claude = ClaudeConfig::new(data.path());
    claude.executable = stub.path.clone();

    let handle = daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(BackendRegistry::new()),
            default_backend: Some(CLAUDE_BACKEND_NAME.to_string()),
            claude: Some(claude),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon");

    let client = reqwest::Client::new();
    let submitted: Value = client
        .post(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&json!({
            "command_id": ulid(),
            "intent": "drive the real adapter",
            "origin": {"client": "cli", "cwd": repo.path()},
        }))
        .send()
        .await
        .expect("submit")
        .json()
        .await
        .expect("json");
    assert_eq!(
        submitted["backend"], CLAUDE_BACKEND_NAME,
        "the real adapter must be the one that ran: {submitted}"
    );
    let work_id = submitted["work"]["id"]
        .as_str()
        .expect("work id")
        .to_string();
    let execution_id = submitted["execution"]["execution_id"]
        .as_str()
        .expect("execution id")
        .to_string();

    // The turn really launched (so the window was really open)...
    let launches = stub.wait_for_launches(1);
    assert!(launches[0].has("-p"), "{:?}", launches[0].argv);
    // ...the daemon is still serving, and the work is intact.
    let shown: Value = client
        .get(format!("{}/v1/work/{work_id}", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("show")
        .json()
        .await
        .expect("json");
    assert_eq!(shown["work"]["id"], work_id.as_str());

    // And the adapter's normalized events reached the journal — read back
    // through the live daemon's own API, which is also what proves the
    // committer is not wedged behind the core lock.
    let deadline = Instant::now() + Duration::from_secs(15);
    let events = loop {
        let body: Value = client
            .get(format!("{}/v1/events", handle.endpoint))
            .bearer_auth(&handle.token)
            .send()
            .await
            .expect("events")
            .json()
            .await
            .expect("json");
        let events: Vec<Event> = body["events"]
            .as_array()
            .expect("events")
            .iter()
            .map(|e| serde_json::from_value(e.clone()).expect("event"))
            .filter(|e: &Event| e.execution_id.as_deref() == Some(execution_id.as_str()))
            .collect();
        if events.iter().any(|e| e.kind == "usage.updated") {
            break events;
        }
        assert!(
            Instant::now() < deadline,
            "the turn's events never reached the journal: {:?}",
            events.iter().map(|e| &e.kind).collect::<Vec<_>>()
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    };
    handle.shutdown().await;
    let kinds: Vec<&str> = events.iter().map(|e| e.kind.as_str()).collect();
    assert!(
        kinds.contains(&"conversation.user"),
        "the emit on the request path landed: {kinds:?}"
    );
    assert!(kinds.contains(&"usage.updated"), "{kinds:?}");
    let backend_events: Vec<&Event> = events
        .iter()
        .filter(|e| e.source.source_type == "backend")
        .collect();
    assert!(
        backend_events
            .iter()
            .all(|e| e.correlation_id.as_deref() == Some(execution_id.as_str())),
        "every normalized event is correlated to its execution"
    );
    assert!(
        backend_events
            .iter()
            .skip(1)
            .all(|e| e.causation_id.is_some()),
        "the committer chains causation: {backend_events:?}"
    );
}

/// The capability/version probe is *recorded at backend registration* (M4
/// contract), not only surfaced inside a later refusal: the daemon journals
/// one `backend.probed` event per registered backend, carrying what the
/// probe found.
#[tokio::test]
async fn the_capability_probe_is_journaled_at_registration() {
    let data = TempDir::new().expect("tempdir");
    // A CLI below the minimum trusted version: the probe's verdict is
    // "unavailable", and the record must say so with its evidence.
    let stub = StubClaude::new(data.path(), "2.1.220 (Claude Code)", ALL_FLAGS);
    let mut claude = ClaudeConfig::new(data.path());
    claude.executable = stub.path.clone();
    let handle = daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(BackendRegistry::new()),
            default_backend: None,
            claude: Some(claude),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon");
    handle.shutdown().await;

    let probed: Vec<Event> = Journal::replay_data_dir(data.path())
        .expect("replay")
        .map(|e| e.expect("event"))
        .filter(|e| e.kind == daemon::KIND_BACKEND_PROBED)
        .collect();
    assert_eq!(probed.len(), 1, "one record per registered backend");
    let payload = &probed[0].payload;
    assert_eq!(payload["backend"], CLAUDE_BACKEND_NAME);
    assert_eq!(payload["available"], false);
    let detail = payload["detail"].as_str().expect("probe detail recorded");
    assert!(detail.contains("2.1.220"), "{detail}");
    assert!(detail.contains("minimum trusted 2.1.226"), "{detail}");
    assert_eq!(
        payload["capabilities"]["native_subagents"], false,
        "capabilities are recorded as advertised, and nothing unmeasured is advertised"
    );
    assert_eq!(
        payload["capabilities"]["history"], false,
        "the durable-history claim is journaled, so a false one would become a \
         durable record: this adapter does not retrieve durable native history"
    );
    // §17: the adapter's declared runtime scope is part of the record. The
    // core is forbidden from assuming a daemon model, so the declaration has
    // to exist somewhere a later reader can find it.
    assert_eq!(payload["runtime_scope"], "per_execution");
}

// ------------------------------------------- 2. model pin (deterministic)

/// Acceptance 2 (pre-flight layer): a provider-qualified pin is refused
/// before any process is launched, with a structured error naming why.
#[test]
fn a2_provider_qualified_pin_is_rejected_preflight() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubClaude::passing(dir.path());
    let mut config = ClaudeConfig::new(dir.path());
    config.executable = stub.path.clone();
    let backend = ClaudeBackend::new(config);

    let request = start_request(
        "e-pin",
        dir.path(),
        "irrelevant",
        Some("anthropic/claude-sonnet-5"),
    );
    let err = backend.start(&request).expect_err("must refuse pre-flight");
    let text = err.to_string();
    assert!(text.contains("provider-qualified"), "{text}");
    // Pre-flight means pre-flight: no process was launched...
    assert!(
        stub.launches().is_empty(),
        "a pre-flight refusal must not spawn the CLI: {:?}",
        stub.launches()
    );
    // ...and no execution state was left behind. (Asserting that OBSERVE
    // errors would prove nothing: a handle for a refused start carries no
    // native id, so OBSERVE refuses it either way.)
    assert!(
        backend.tracked_executions().is_empty(),
        "a refused start must leave no phantom execution: {:?}",
        backend.tracked_executions()
    );
}

/// The other half of "a refused start leaves nothing behind": a start that
/// passes the gate and the pin check and *then* fails to spawn. The probe is
/// cached from a successful run, so the launch itself is what fails.
#[test]
fn a_start_that_cannot_spawn_leaves_no_phantom_execution() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().expect("tempdir");
    let stub = StubClaude::passing(dir.path());
    let mut config = ClaudeConfig::new(dir.path());
    config.executable = stub.path.clone();
    let backend = ClaudeBackend::new(config);
    assert!(backend.probe().available, "probe runs (and caches) first");

    // Now the executable stops being executable: everything up to the spawn
    // succeeds, the spawn does not.
    std::fs::set_permissions(&stub.path, std::fs::Permissions::from_mode(0o644))
        .expect("chmod stub");
    let err = backend
        .start(&start_request(
            "e-nospawn",
            dir.path(),
            "cannot launch",
            None,
        ))
        .expect_err("spawn must fail");
    assert!(err.to_string().contains("cannot spawn"), "{err}");
    assert!(
        backend.tracked_executions().is_empty(),
        "a failed launch must not leave an execution OBSERVE would misread as \
         interrupted-but-resumable: {:?}",
        backend.tracked_executions()
    );
}

// --------------------------------------------- D2's launch grammar, pinned

/// D2 *is* this argv. Every element below is a measured decision recorded in
/// `backend::claude`'s module docs, and each one is silently reversible
/// without an assertion on what the adapter actually launched:
/// `--setting-sources user` (LESSONS L2's project-memory capture hazard),
/// the `--session-id`-then-`--resume` progression (one durable conversation,
/// not a new one per turn), the removal of an inherited
/// `CLAUDE_CODE_SESSION_ID` (the nested-session hazard), and the permission
/// mode.
#[test]
fn d2_the_launch_grammar_is_session_pinned_then_resumed() {
    let dir = TempDir::new().expect("tempdir");
    let cwd = TempDir::new().expect("tempdir");
    let stub = StubClaude::passing(dir.path());
    let mut config = ClaudeConfig::new(dir.path());
    config.executable = stub.path.clone();
    config.env.insert("IS_SANDBOX".to_string(), "1".to_string());
    let backend = ClaudeBackend::new(config);

    // A nested environment's session id must not leak into the turn. The
    // process environment is the only place this hazard lives (a value in
    // adapter config would be deliberate operator intent, not inheritance),
    // so it is set here and removed the moment the spawn has copied it —
    // the child's environment is fixed at spawn, so the window is `start`.
    let mut request = start_request("e-grammar", cwd.path(), "the intent", Some("haiku"));
    request.context = "the stage context".to_string();
    unsafe { std::env::set_var("CLAUDE_CODE_SESSION_ID", "inherited-parent-session") };
    let started = backend.start(&request);
    unsafe { std::env::remove_var("CLAUDE_CODE_SESSION_ID") };
    let handle = started.expect("start");
    let session_id = handle.native_id.clone().expect("session id");
    wait_settled(&backend, &handle, Duration::from_secs(10));
    backend.send(&handle, "second turn").expect("send");
    wait_settled(&backend, &handle, Duration::from_secs(10));

    let launches = stub.wait_for_launches(2);
    assert_eq!(launches.len(), 2, "one process per turn");

    let first = &launches[0];
    assert!(first.has("-p"), "print mode: {:?}", first.argv);
    assert!(first.has("--verbose"), "{:?}", first.argv);
    assert_eq!(first.value_of("--output-format"), Some("stream-json"));
    assert_eq!(
        first.value_of("--setting-sources"),
        Some("user"),
        "L2: the target repo's project memory must not capture the execution agent"
    );
    assert_eq!(
        first.value_of("--session-id"),
        Some(session_id.as_str()),
        "the first turn pins the identity sergeant chose"
    );
    assert!(
        !first.has("--resume"),
        "the first turn has nothing to resume: {:?}",
        first.argv
    );
    assert_eq!(first.value_of("--model"), Some("haiku"));
    assert!(
        first.has("--dangerously-skip-permissions"),
        "the no-profile default (L2's production default): {:?}",
        first.argv
    );
    assert_eq!(
        first.cwd,
        cwd.path()
            .canonicalize()
            .expect("cwd")
            .display()
            .to_string()
    );
    assert_eq!(
        first.env["CLAUDE_CODE_SESSION_ID"], "<unset>",
        "an inherited session id is removed, not passed through"
    );
    assert_eq!(
        first.env["IS_SANDBOX"], "1",
        "adapter config env is applied"
    );
    assert_eq!(
        first.stdin, "the intent||the stage context",
        "§12: intent plus the stage's CONTEXT.md, verbatim, on stdin"
    );

    let second = &launches[1];
    assert_eq!(
        second.value_of("--resume"),
        Some(session_id.as_str()),
        "every later turn continues the same conversation"
    );
    assert!(
        !second.has("--session-id"),
        "a second --session-id would start a new conversation — the inverse of D2: {:?}",
        second.argv
    );
    assert!(
        !second.has("--fork-session"),
        "D2 forbids forking: {:?}",
        second.argv
    );
    assert_eq!(second.stdin, "second turn");
}

/// §14: a profile is launch configuration, carried to the *real* adapter —
/// executable, environment, config home, and the permission mode that
/// replaces the skip-permissions default.
#[test]
fn a_profile_is_launch_configuration_carried_to_the_claude_adapter() {
    let dir = TempDir::new().expect("tempdir");
    let profile_dir = TempDir::new().expect("tempdir");
    let system = StubClaude::passing(dir.path());
    let profiled = StubClaude::passing(profile_dir.path());
    let mut config = ClaudeConfig::new(dir.path());
    config.executable = system.path.clone();
    let backend = ClaudeBackend::new(config);

    let mut request = start_request("e-profile", dir.path(), "profiled", None);
    request.profile = Some(Profile {
        name: "enterprise".to_string(),
        backend: CLAUDE_BACKEND_NAME.to_string(),
        executable: Some(profiled.path.clone()),
        config_home: Some(PathBuf::from("/tmp/claude-work-home")),
        env: [("GIT_AUTHOR_NAME".to_string(), "sergeant".to_string())]
            .into_iter()
            .collect(),
        default_model: None,
        options: [("permission_mode".to_string(), "plan".to_string())]
            .into_iter()
            .collect(),
    });
    let handle = backend.start(&request).expect("start");
    wait_settled(&backend, &handle, Duration::from_secs(10));

    assert!(
        system.launches().is_empty(),
        "the profile's executable replaces the configured one"
    );
    let launches = profiled.wait_for_launches(1);
    let launch = &launches[0];
    assert_eq!(launch.value_of("--permission-mode"), Some("plan"));
    assert!(
        !launch.has("--dangerously-skip-permissions"),
        "a profile-pinned permission mode is not silently escalated: {:?}",
        launch.argv
    );
    assert_eq!(launch.env["CLAUDE_CONFIG_DIR"], "/tmp/claude-work-home");
}

/// RESUME re-adopts a conversation with the launch configuration the caller
/// re-supplies — and with nothing invented in its place.
///
/// This is the fail-open shape the adapter must not have: after a restart it
/// would be trivially convenient to rebuild the execution from defaults, and
/// the result is an adapter that silently replaces a profile's
/// `--permission-mode` with `--dangerously-skip-permissions` (a security
/// decision belonging to the human), drops the model pin so every later turn
/// verifies as "unpinned" while the journal still records a pin, and
/// journals normalized events under an empty work id.
#[test]
fn resume_launches_later_turns_under_the_re_supplied_configuration() {
    let data = TempDir::new().expect("tempdir");
    let home = TempDir::new().expect("tempdir");
    let cwd = TempDir::new().expect("tempdir");
    let session_id = "5a4b3c2d-1e0f-4a1b-8c2d-3e4f5a6b7c8d";
    let project = home.path().join("projects").join("-work-surface");
    std::fs::create_dir_all(&project).expect("project dir");
    std::fs::write(project.join(format!("{session_id}.jsonl")), "{}\n").expect("transcript");

    let stub = StubClaude::passing(data.path());
    stub.replays(RECORDED_TURN);
    let mut config = ClaudeConfig::new(data.path());
    config.claude_home = Some(home.path().to_path_buf());
    // Not the executable the profile names: the profile's must win here too.
    config.executable = PathBuf::from("/nonexistent/claude");
    let backend = ClaudeBackend::new(config);
    let shared = Arc::new(tokio::sync::Mutex::new(core(data.path())));
    backend.set_event_sink(journaling_sink(shared.clone()));

    let handle = ExecutionHandle {
        execution_id: "exec-readopt".to_string(),
        native_id: Some(session_id.to_string()),
    };
    backend
        .resume(
            &handle,
            &ResumeRequest {
                work_id: "01M4READOPT".to_string(),
                cwd: cwd.path().to_path_buf(),
                model: Some("haiku".to_string()),
                profile: Some(Profile {
                    name: "enterprise".to_string(),
                    backend: CLAUDE_BACKEND_NAME.to_string(),
                    executable: Some(stub.path.clone()),
                    config_home: Some(PathBuf::from("/tmp/claude-work-home")),
                    env: BTreeMap::new(),
                    default_model: None,
                    options: [("permission_mode".to_string(), "plan".to_string())]
                        .into_iter()
                        .collect(),
                }),
            },
        )
        .expect("re-adopt");
    backend.send(&handle, "carry on").expect("send");
    wait_settled(&backend, &handle, Duration::from_secs(10));

    let launches = stub.wait_for_launches(1);
    let launch = &launches[0];
    assert_eq!(
        launch.value_of("--resume"),
        Some(session_id),
        "a re-adopted turn continues the journaled conversation: {:?}",
        launch.argv
    );
    assert_eq!(
        launch.value_of("--model"),
        Some("haiku"),
        "the re-supplied pin is enforced, not dropped: {:?}",
        launch.argv
    );
    assert_eq!(launch.value_of("--permission-mode"), Some("plan"));
    assert!(
        !launch.has("--dangerously-skip-permissions"),
        "resume must not escalate past a profile-pinned permission mode: {:?}",
        launch.argv
    );
    assert_eq!(launch.env["CLAUDE_CONFIG_DIR"], "/tmp/claude-work-home");
    assert_eq!(
        launch.cwd,
        cwd.path()
            .canonicalize()
            .expect("cwd")
            .display()
            .to_string()
    );

    // The pin verifies against the recorded envelope, and the work binding
    // survives the restart.
    let events = wait_for_events(&shared, "exec-readopt", 4);
    let usage = events
        .iter()
        .find(|e| e.kind == "usage.updated")
        .expect("usage.updated");
    assert_eq!(usage.payload["model_pin"]["verdict"], "honored");
    assert!(
        events
            .iter()
            .all(|e| e.work_id.as_deref() == Some("01M4READOPT")),
        "post-restart events carry the work they serve: {events:?}"
    );
}

// -------------------------------- §20 raw archive and §27 normalization

/// Acceptance 1's deterministic half: a recorded turn is normalized into §27
/// events (assistant text, tool requested, tool completed, usage) *and*
/// archived verbatim to the §20 blob store, with the blob ref reaching the
/// journal. Replaying a recording is what makes this assertable without
/// tokens — and without it, ingestion and archiving can both be deleted
/// outright with the suite green.
#[test]
fn a_recorded_turn_is_normalized_and_archived_verbatim() {
    let data = TempDir::new().expect("tempdir");
    let stub = StubClaude::passing(data.path());
    stub.replays(RECORDED_TURN);
    let mut config = ClaudeConfig::new(data.path());
    config.executable = stub.path.clone();
    let backend = ClaudeBackend::new(config);
    let shared = Arc::new(tokio::sync::Mutex::new(core(data.path())));
    backend.set_event_sink(journaling_sink(shared.clone()));

    let handle = backend
        .start(&start_request(
            "e-replay",
            data.path(),
            "replay the recording",
            Some("haiku"),
        ))
        .expect("start");
    let observation = wait_settled(&backend, &handle, Duration::from_secs(10));
    let BackendSignal::StageCompleted { summary } = &observation.signal else {
        panic!("the recorded turn completes: {observation:?}");
    };
    assert_eq!(summary.as_deref(), Some("OK"));
    assert!(
        observation
            .evidence
            .as_deref()
            .unwrap_or("")
            .contains("\"verdict\":\"honored\""),
        "the recorded envelope's modelUsage honors the haiku pin: {observation:?}"
    );

    // §27: the adapter's normalized view of the turn, read where it is
    // durable — the journal the event sink feeds. (Not through HISTORY: this
    // adapter advertises no durable history and refuses the verb; the
    // pairing is pinned by `capabilities_match_behaviour_for_every_backend`.)
    let events = wait_for_events(&shared, "e-replay", 6);
    let kinds: Vec<&str> = events.iter().map(|e| e.kind.as_str()).collect();
    assert_eq!(
        kinds,
        vec![
            "conversation.user",
            "tool.requested",
            "tool.completed",
            "conversation.assistant.completed",
            "conversation.turn.ended",
            "usage.updated",
        ],
        "every recorded line normalizes, in stream order"
    );
    let tool = &events[1].payload;
    assert_eq!(tool["name"], "Bash");
    assert_eq!(tool["input"]["command"], "echo hello-fixture");
    assert_eq!(events[2].payload["tool_use_id"], tool["id"]);
    assert_eq!(events[3].payload["text"], "OK");

    // §20: the raw lines, verbatim, resolvable from the journaled ref.
    let usage = events
        .iter()
        .find(|e| e.kind == "usage.updated")
        .expect("usage.updated journaled");
    assert_eq!(usage.payload["model_pin"]["verdict"], "honored");
    assert!(usage.payload["raw_error"].is_null());
    let raw_ref = usage.payload["raw"].as_str().expect("raw blob ref");
    let store = BlobStore::open(data.path()).expect("blob store");
    let blob = store
        .get(&BlobRef::from_str(raw_ref).expect("valid ref"))
        .expect("raw transcript archived");
    assert_eq!(
        String::from_utf8_lossy(&blob),
        RECORDED_TURN,
        "the archive is the stream, byte for byte"
    );
    // And the turn-ended event carries the same ref, so a turn with no
    // result envelope still has its raw capture reachable.
    let ended = events
        .iter()
        .find(|e| e.kind == "conversation.turn.ended")
        .expect("turn.ended journaled");
    assert_eq!(ended.payload["raw"], raw_ref);
    assert_eq!(ended.payload["interrupted"], false);
    assert_eq!(ended.payload["result_envelope"], true);
}

/// A turn that ends without a result envelope still reports where its raw
/// capture went — the interrupted and crashed cases are exactly when someone
/// needs to read what the turn managed to say, and archiving bytes nothing
/// can name is capture in the store and evidence nowhere.
#[test]
fn an_envelope_less_turn_still_surfaces_its_raw_capture() {
    let data = TempDir::new().expect("tempdir");
    let stub = StubClaude::passing(data.path());
    // A partial stream: assistant text, then the process dies (no result).
    let partial: String = RECORDED_TURN
        .lines()
        .take(3)
        .map(|line| format!("{line}\n"))
        .collect();
    stub.replays(&partial);
    let mut config = ClaudeConfig::new(data.path());
    config.executable = stub.path.clone();
    let backend = ClaudeBackend::new(config);
    let shared = Arc::new(tokio::sync::Mutex::new(core(data.path())));
    backend.set_event_sink(journaling_sink(shared.clone()));

    let handle = backend
        .start(&start_request(
            "e-partial",
            data.path(),
            "dies mid-turn",
            None,
        ))
        .expect("start");
    let observation = wait_settled(&backend, &handle, Duration::from_secs(10));
    assert_eq!(
        observation.native,
        NativeState::Unknown,
        "no envelope, no interrupt request: ambiguity"
    );
    let evidence = observation.evidence.unwrap_or_default();
    let raw_ref = evidence
        .split("raw=")
        .nth(1)
        .and_then(|rest| rest.split(';').next())
        .expect("the evidence names the archive")
        .trim()
        .to_string();
    let store = BlobStore::open(data.path()).expect("blob store");
    let blob = store
        .get(&BlobRef::from_str(&raw_ref).expect("the evidence's ref resolves"))
        .expect("partial transcript archived");
    assert_eq!(String::from_utf8_lossy(&blob), partial);

    let events = wait_for_events(&shared, "e-partial", 4);
    let ended = events
        .iter()
        .find(|e| e.kind == "conversation.turn.ended")
        .expect("an envelope-less turn still journals its end");
    assert_eq!(ended.payload["raw"], raw_ref);
    assert_eq!(ended.payload["result_envelope"], false);
    assert!(
        !events.iter().any(|e| e.kind == "usage.updated"),
        "no envelope, no usage to report"
    );
}

/// A failed §20 archive is reported, not swallowed: the turn's own verdict
/// stands (a storage failure is not execution ambiguity), but nothing is
/// left believing the bytes are on disk.
#[test]
fn a_failed_raw_archive_is_reported_with_its_reason() {
    let data = TempDir::new().expect("tempdir");
    let stub = StubClaude::passing(data.path());
    stub.replays(RECORDED_TURN);
    let mut config = ClaudeConfig::new(data.path());
    config.executable = stub.path.clone();
    let backend = ClaudeBackend::new(config);
    let shared = Arc::new(tokio::sync::Mutex::new(core(data.path())));
    backend.set_event_sink(journaling_sink(shared.clone()));
    // The blob store's root, occupied by a file: `put` cannot write there.
    std::fs::write(data.path().join("blobs"), b"not a directory").expect("occupy blob root");

    let handle = backend
        .start(&start_request(
            "e-noarchive",
            data.path(),
            "archive fails",
            None,
        ))
        .expect("start");
    let observation = wait_settled(&backend, &handle, Duration::from_secs(10));
    let evidence = observation.evidence.unwrap_or_default();
    assert!(
        evidence.contains("raw=unarchived ("),
        "the evidence must name the failure, not read like an absent archive: {evidence}"
    );
    let events = wait_for_events(&shared, "e-noarchive", 6);
    let ended = events
        .iter()
        .find(|e| e.kind == "conversation.turn.ended")
        .expect("turn.ended journaled");
    assert!(ended.payload["raw"].is_null());
    assert!(
        ended.payload["raw_error"]
            .as_str()
            .is_some_and(|e| !e.is_empty()),
        "the archive failure reaches the journal: {}",
        ended.payload
    );
}

/// STOP retires the execution: the latch holds, so a stopped conversation
/// refuses further input even though its transcript is untouched.
#[test]
fn stop_latches_and_a_stopped_execution_refuses_input() {
    let data = TempDir::new().expect("tempdir");
    let stub = StubClaude::passing(data.path());
    stub.replays(RECORDED_TURN);
    let mut config = ClaudeConfig::new(data.path());
    config.executable = stub.path.clone();
    let backend = ClaudeBackend::new(config);

    let handle = backend
        .start(&start_request("e-stop", data.path(), "stop me", None))
        .expect("start");
    wait_settled(&backend, &handle, Duration::from_secs(10));
    backend.send(&handle, "still welcome").expect("send");
    wait_settled(&backend, &handle, Duration::from_secs(10));
    backend.stop(&handle).expect("stop");
    let err = backend
        .send(&handle, "no longer welcome")
        .expect_err("a stopped execution accepts nothing");
    assert!(err.to_string().contains("stopped"), "{err}");
    assert_eq!(
        stub.launches().len(),
        2,
        "the refused SEND launched no process"
    );
}

/// INTERRUPT kills the per-turn process, and the killed turn is reported as
/// interrupted-and-resumable rather than as ambiguity.
///
/// The whole kill mechanism — the shared `Arc<Mutex<Child>>`, the
/// `interrupt_requested` bit the reader thread consumes, the reader waking on
/// EOF — had no deterministic coverage at all: gutting `interrupt` to a no-op,
/// or flipping `interrupt_requested` to `false`, left every test green, with
/// the only exercise of either in an `#[ignore]`d opt-in test. The stub hangs
/// after replaying (`exec sleep`, so the killed pid is the process holding
/// stdout), which is what gives INTERRUPT something real to kill.
#[test]
fn an_in_flight_turn_is_killed_by_interrupt_and_reported_as_resumable() {
    let data = TempDir::new().expect("tempdir");
    let stub = StubClaude::passing(data.path());
    // The measured shape of a turn killed mid-generation: the lines it
    // managed to stream, and no result envelope (module docs, `backend::
    // claude`). The stub then stays alive until it is killed.
    stub.replays(&partial_turn()).hangs_after_replay();
    let mut config = ClaudeConfig::new(data.path());
    config.executable = stub.path.clone();
    let backend = ClaudeBackend::new(config);

    let handle = backend
        .start(&start_request(
            "e-interrupt",
            data.path(),
            "a turn that will not end on its own",
            None,
        ))
        .expect("start");
    // The turn is genuinely in flight: it replayed and is still alive.
    stub.wait_for_launches(1);
    let in_flight = backend.observe(&handle).expect("observe");
    assert_eq!(
        in_flight.native,
        NativeState::Running,
        "the hanging stub keeps the turn in flight: {in_flight:?}"
    );

    backend.interrupt(&handle).expect("interrupt");
    let after = wait_settled(&backend, &handle, Duration::from_secs(10));
    assert_eq!(
        after.native,
        NativeState::Exited,
        "the process was killed: {after:?}"
    );
    assert_eq!(
        after.signal,
        BackendSignal::Running,
        "an interrupted turn draws no conclusion about the stage (§25): {after:?}"
    );
    let evidence = after.evidence.clone().unwrap_or_default();
    assert!(
        evidence.contains("interrupted by request"),
        "the interrupt bit is what separates a killed turn from ambiguity: {evidence}"
    );
    assert!(
        evidence.contains("resumable"),
        "work ≠ process: the conversation survives the kill: {evidence}"
    );
    // The partial stream the turn managed to emit is archived and named.
    let raw_ref = evidence
        .split("raw=")
        .nth(1)
        .expect("the evidence names the archive")
        .trim()
        .to_string();
    let store = BlobStore::open(data.path()).expect("blob store");
    store
        .get(&BlobRef::from_str(&raw_ref).expect("the evidence's ref resolves"))
        .expect("the interrupted turn's partial transcript is archived");

    // And the conversation really is resumable: the next SEND launches a
    // `--resume` turn on the same session.
    let session_id = handle.native_id.clone().expect("session id");
    backend.send(&handle, "carry on").expect("send after kill");
    let launches = stub.wait_for_launches(2);
    assert_eq!(launches[1].value_of("--resume"), Some(session_id.as_str()));
    backend.stop(&handle).expect("stop");
}

/// A print-mode conversation runs one turn at a time, and SEND refuses to
/// start a second one while the first is in flight.
///
/// Only the *stopped* refusal was pinned (and that test always waits for the
/// turn to settle first), so disabling this guard left the suite green — with
/// the consequence that two `--resume` processes would run against one
/// session, which is the one thing `--resume` does not do.
#[test]
fn send_refuses_a_second_turn_while_one_is_in_flight() {
    let data = TempDir::new().expect("tempdir");
    let stub = StubClaude::passing(data.path());
    stub.replays(&partial_turn()).hangs_after_replay();
    let mut config = ClaudeConfig::new(data.path());
    config.executable = stub.path.clone();
    let backend = ClaudeBackend::new(config);

    let handle = backend
        .start(&start_request(
            "e-oneturn",
            data.path(),
            "a turn that stays in flight",
            None,
        ))
        .expect("start");
    stub.wait_for_launches(1);
    assert_eq!(
        backend.observe(&handle).expect("observe").native,
        NativeState::Running
    );

    let err = backend
        .send(&handle, "and another thing")
        .expect_err("a second concurrent turn must be refused");
    assert!(
        err.to_string().contains("one turn at a time"),
        "the refusal says why: {err}"
    );
    assert_eq!(
        stub.launches().len(),
        1,
        "the refused SEND launched no second process against the session: {:?}",
        stub.launches()
    );

    // Once the turn settles, the same SEND is accepted.
    backend.interrupt(&handle).expect("interrupt");
    wait_settled(&backend, &handle, Duration::from_secs(10));
    backend.send(&handle, "and another thing").expect("send");
    assert_eq!(stub.wait_for_launches(2).len(), 2);
    backend.stop(&handle).expect("stop");
}

/// STOP kills the turn in flight, not just the ability to send.
///
/// `stop` delegates its kill to `interrupt`, so the same untested mechanism
/// was carrying it: with `interrupt` gutted, STOP silently stopped killing
/// anything while still latching the refusal, and every test stayed green.
#[test]
fn stop_kills_a_turn_that_is_still_running() {
    let data = TempDir::new().expect("tempdir");
    let stub = StubClaude::passing(data.path());
    stub.replays(&partial_turn()).hangs_after_replay();
    let mut config = ClaudeConfig::new(data.path());
    config.executable = stub.path.clone();
    let backend = ClaudeBackend::new(config);

    let handle = backend
        .start(&start_request(
            "e-stopkill",
            data.path(),
            "stop me mid-turn",
            None,
        ))
        .expect("start");
    stub.wait_for_launches(1);
    assert_eq!(
        backend.observe(&handle).expect("observe").native,
        NativeState::Running
    );

    backend.stop(&handle).expect("stop");
    let after = wait_settled(&backend, &handle, Duration::from_secs(10));
    assert_eq!(
        after.native,
        NativeState::Exited,
        "STOP retires the native context, it does not merely stop listening: {after:?}"
    );
    let err = backend
        .send(&handle, "no longer welcome")
        .expect_err("a stopped execution accepts nothing");
    assert!(err.to_string().contains("stopped"), "{err}");
}

/// §25's native-identity rule, on the adapter this milestone ships.
///
/// The regression catalog's stale-identity entry (`r5`) drives the *fake*;
/// against the Claude adapter, every handle in the suite carried the true
/// session id, so deleting the identity comparison in `check_identity` left
/// the whole suite green. A forged handle must not be able to read, feed,
/// interrupt, re-adopt or kill someone else's conversation.
#[test]
fn a_forged_handle_never_resolves_against_the_claude_adapter() {
    let data = TempDir::new().expect("tempdir");
    let home = TempDir::new().expect("tempdir");
    let stub = StubClaude::passing(data.path());
    stub.replays(RECORDED_TURN);
    let mut config = ClaudeConfig::new(data.path());
    config.executable = stub.path.clone();
    config.claude_home = Some(home.path().to_path_buf());
    let backend = ClaudeBackend::new(config);

    let handle = backend
        .start(&start_request(
            "e-identity",
            data.path(),
            "the real one",
            None,
        ))
        .expect("start");
    wait_settled(&backend, &handle, Duration::from_secs(10));
    let real_session = handle.native_id.clone().expect("session id");

    // The forged identity even has a durable transcript on disk, so RESUME
    // cannot refuse it merely for lack of one: the *mismatch* is the refusal.
    let forged_session = fresh_session_id();
    let project = home.path().join("projects").join("-elsewhere");
    std::fs::create_dir_all(&project).expect("project dir");
    std::fs::write(project.join(format!("{forged_session}.jsonl")), "{}\n").expect("transcript");

    for wrong in [None, Some(forged_session.clone())] {
        let forged = ExecutionHandle {
            execution_id: handle.execution_id.clone(),
            native_id: wrong.clone(),
        };
        for outcome in [
            backend.observe(&forged).map(|_| ()),
            backend.send(&forged, "hello"),
            backend.interrupt(&forged),
            backend.resume(&forged, &ResumeRequest::new("w", data.path())),
            backend.history(&forged).map(|_| ()),
            backend.stop(&forged),
        ] {
            assert!(
                matches!(outcome, Err(BackendError::UnknownExecution { .. })),
                "a handle carrying {wrong:?} instead of the issued session identity \
                 must not resolve, got {outcome:?}"
            );
        }
    }

    // None of those refusals touched the execution they named: it is still
    // known, still unstopped, and still on its own conversation.
    let observed = backend.observe(&handle).expect("still known");
    assert!(
        observed
            .evidence
            .as_deref()
            .unwrap_or("")
            .contains(&real_session),
        "a forged STOP must not have retired the real execution: {observed:?}"
    );
    backend.send(&handle, "still mine").expect("send");
    wait_settled(&backend, &handle, Duration::from_secs(10));
    assert_eq!(stub.wait_for_launches(2).len(), 2);
}

/// §15 RESUME applies pin verification's pre-flight layer to a *re-supplied*
/// pin exactly as START does — before anything is adopted on the strength of
/// it. Only START's check was pinned, so deleting RESUME's left the suite
/// green while the doc comment kept claiming it.
#[test]
fn resume_refuses_a_pin_that_could_never_be_honored() {
    let data = TempDir::new().expect("tempdir");
    let home = TempDir::new().expect("tempdir");
    let session_id = fresh_session_id();
    let project = home.path().join("projects").join("-work-surface");
    std::fs::create_dir_all(&project).expect("project dir");
    std::fs::write(project.join(format!("{session_id}.jsonl")), "{}\n").expect("transcript");

    let mut config = ClaudeConfig::new(data.path());
    config.claude_home = Some(home.path().to_path_buf());
    let backend = ClaudeBackend::new(config);
    let handle = ExecutionHandle {
        execution_id: "exec-pinned-resume".to_string(),
        native_id: Some(session_id.clone()),
    };

    let err = backend
        .resume(
            &handle,
            &ResumeRequest {
                work_id: "01M4RESUMEPIN".to_string(),
                cwd: data.path().to_path_buf(),
                model: Some("anthropic/claude-haiku-4-5".to_string()),
                profile: None,
            },
        )
        .expect_err("a provider-qualified pin is refused pre-flight at RESUME too");
    assert!(err.to_string().contains("provider-qualified"), "{err}");
    assert!(
        backend.tracked_executions().is_empty(),
        "nothing is adopted on the strength of a pin that cannot hold: {:?}",
        backend.tracked_executions()
    );

    // The same request with an honorable pin adopts it.
    backend
        .resume(
            &handle,
            &ResumeRequest {
                work_id: "01M4RESUMEPIN".to_string(),
                cwd: data.path().to_path_buf(),
                model: Some("haiku".to_string()),
                profile: None,
            },
        )
        .expect("re-adopt");
    assert_eq!(backend.tracked_executions(), vec!["exec-pinned-resume"]);
}

/// §15's honesty rule, checked against behaviour rather than against the
/// flag's own word, for every backend this milestone ships.
///
/// The Claude adapter used to advertise `history: true` — the capability
/// whose documented meaning is *durable* native history — while implementing
/// it as process-local memory, so after a RESUME it answered `Ok([])` for a
/// conversation whose transcript was on disk: "nothing was said" and "this
/// daemon was not here", the same value. The claim is now `false` and the
/// verb refuses, which is the pairing this pins in both directions.
#[test]
fn capabilities_match_behaviour_for_every_backend() {
    let data = TempDir::new().expect("tempdir");
    let home = TempDir::new().expect("tempdir");
    let stub = StubClaude::passing(data.path());
    stub.replays(RECORDED_TURN);
    let mut config = ClaudeConfig::new(data.path());
    config.executable = stub.path.clone();
    config.claude_home = Some(home.path().to_path_buf());
    let claude = ClaudeBackend::new(config);
    let fake = FakeBackend::new(FAKE_BACKEND_NAME);

    // Claude: the capability is false, and the verb refuses in a way a
    // caller can tell apart from an empty conversation.
    assert!(!claude.capabilities().history);
    let handle = claude
        .start(&start_request(
            "e-history",
            data.path(),
            "say something",
            None,
        ))
        .expect("start");
    wait_settled(&claude, &handle, Duration::from_secs(10));
    match claude.history(&handle) {
        Err(BackendError::Unsupported { verb, detail, .. }) => {
            assert_eq!(verb, "history");
            assert!(
                detail.contains("<session_id>.jsonl") && detail.contains("event sink"),
                "the refusal names where the record actually is: {detail}"
            );
        }
        other => panic!("an unsupported capability must refuse, got {other:?}"),
    }

    // And after a restart — the case that made the old claim false — the
    // answer is still a refusal, never an empty list.
    let session_id = fresh_session_id();
    let project = home.path().join("projects").join("-work-surface");
    std::fs::create_dir_all(&project).expect("project dir");
    std::fs::write(project.join(format!("{session_id}.jsonl")), "{}\n").expect("transcript");
    let reborn = {
        let mut config = ClaudeConfig::new(data.path());
        config.executable = stub.path.clone();
        config.claude_home = Some(home.path().to_path_buf());
        ClaudeBackend::new(config)
    };
    let adopted = ExecutionHandle {
        execution_id: "exec-readopted-history".to_string(),
        native_id: Some(session_id),
    };
    reborn
        .resume(&adopted, &ResumeRequest::new("01M4HISTORY", data.path()))
        .expect("re-adopt");
    assert!(
        matches!(
            reborn.history(&adopted),
            Err(BackendError::Unsupported { .. })
        ),
        "an adopted conversation must not report an empty history"
    );

    // The fake advertises the capability, and can honor it: a known
    // execution's whole history, an unknown one refused.
    assert!(fake.capabilities().history);
    let fake_handle = fake
        .start(&start_request("e-fake-history", data.path(), "hi", None))
        .expect("start");
    fake.send(&fake_handle, "one").expect("send");
    assert_eq!(fake.history(&fake_handle).expect("history").len(), 1);

    // §17: both adapters declare a runtime scope, and neither is assumed.
    assert_eq!(claude.runtime_scope(), RuntimeScope::PerExecution);
    assert_eq!(fake.runtime_scope(), RuntimeScope::PerExecution);
    assert_eq!(RuntimeScope::PerExecution.as_str(), "per_execution");
}

/// Acceptance 2 (substitution layer): detection runs against recorded
/// envelope fixtures — the honored shape is the measured 2.1.226 envelope,
/// the substitution scenario is the spike's (not forcible live; this
/// account is entitled). Exit codes never appear in the verdict.
///
/// What this does *not* do is close the contract's open Unknown about the
/// substitution surface in print mode (transcript line vs result envelope
/// field): the spike measured the TUI, and with an entitled account no live
/// substitution can be provoked here. The adapter therefore fails closed —
/// any envelope whose model fields do not match the pin is substitution —
/// and this test pins that rule, not a measurement of the warning surface.
///
/// **The substituted fixture is derived, not recorded, and it says so.** The
/// contract asks for "a recorded fixture of the substitution envelope"; that
/// artifact cannot exist here (this account is entitled, and the spike's own
/// substitution evidence is a TUI warning line rather than a print-mode
/// envelope), so the fixture on disk is the *recorded honored envelope* with
/// three named edits, listed in `tests/fixtures/README.md`. Keeping it in a
/// file with its derivation written down — rather than as a `json!` literal
/// in this test — is what makes "recorded" versus "derived" reviewable
/// instead of a claim in a comment.
#[test]
fn a2_substitution_is_detected_from_the_recorded_envelope() {
    // Recorded envelope, byte-verbatim from the fixture's own result line
    // (2026-08-09, claude 2.1.226, --model haiku).
    let honored: Value = serde_json::from_str(
        RECORDED_TURN
            .lines()
            .last()
            .expect("the fixture's result envelope"),
    )
    .expect("the recorded result envelope parses");
    assert_eq!(honored["is_error"], false);
    assert_eq!(
        verify_model_pin(Some("haiku"), &honored),
        PinVerdict::Honored("claude-haiku-4-5-20251001".to_string())
    );

    // Derived substitution envelope: the same measured shape, the spike's
    // scenario (an unentitled "opus" silently served by sonnet, mission
    // green — `is_error:false`, `subtype:"success"`, a real usage block).
    let substituted: Value =
        serde_json::from_str(SUBSTITUTION_ENVELOPE).expect("the derived envelope parses");
    assert_eq!(
        substituted["is_error"], false,
        "the whole point: nothing in the envelope reports a failure"
    );
    assert_eq!(substituted["subtype"], "success");
    assert_eq!(
        verify_model_pin(Some("opus"), &substituted),
        PinVerdict::Substituted("claude-sonnet-5-20260101".to_string())
    );
    // And the same envelope honors the pin the model actually ran under, so
    // the rule is a comparison and not a blanket refusal.
    assert_eq!(
        verify_model_pin(Some("sonnet"), &substituted),
        PinVerdict::Honored("claude-sonnet-5-20260101".to_string())
    );
    // No model evidence: attempted, never honored (positive evidence only).
    assert_eq!(
        verify_model_pin(Some("opus"), &json!({})),
        PinVerdict::Attempted
    );
    // And the pre-flight grammar check refuses qualification outright.
    assert!(preflight_model_pin("anthropic/claude-opus-5").is_err());
}

// ------------------------------------------------ 4. restart reconciliation

/// Acceptance 4a: a recorded Claude execution whose durable transcript
/// still exists is **reattached** (§25's reattach step, through §15 RESUME)
/// and then classified from that evidence — the work fails closed to
/// `blocked` carrying "resumable" evidence (the turn's outcome died with
/// the daemon; sergeant does not guess it), and is *retryable*, never
/// silently failed.
///
/// Reattachment and the block are not in tension: what is resumable is the
/// *conversation*, and what is unknown is the outcome of the turn that was in
/// flight. Reconciliation re-adopts the first — so the execution the journal
/// names is owned again, and a later SEND continues that same conversation
/// rather than starting a second one — and refuses to invent the second.
#[test]
fn a4_restart_reattaches_a_surviving_session_and_blocks_with_resumable_evidence() {
    let data = TempDir::new().expect("tempdir");
    let home = TempDir::new().expect("tempdir");
    let session_id = fresh_session_id();
    let project = home.path().join("projects").join("-work-surface");
    std::fs::create_dir_all(&project).expect("project dir");
    std::fs::write(project.join(format!("{session_id}.jsonl")), "{}\n").expect("transcript");

    let mut config = ClaudeConfig::new(data.path());
    config.claude_home = Some(home.path().to_path_buf());
    let claude = Arc::new(ClaudeBackend::new(config));
    let registry = BackendRegistry::new().with(claude.clone());
    let engine = Engine::new(Arc::new(registry), None, data.path());

    let mut core = core(data.path());
    let work_id = "01M4RECOVERA";
    journal_active_run(&mut core, work_id, CLAUDE_BACKEND_NAME, &session_id);
    journal_surface(&mut core, work_id, data.path());

    let report = recovery::reconcile(&engine, &mut core).expect("reconcile");
    assert_eq!(
        report.resumed,
        vec![work_id.to_string()],
        "definite evidence"
    );
    assert_eq!(
        core.registry.state().works[work_id].state,
        WorkState::Blocked
    );
    let blocked = events_of(&core, work_id, KIND_WORK_BLOCKED);
    assert_eq!(blocked.len(), 1);
    assert!(
        blocked[0]["reason"]
            .as_str()
            .is_some_and(|r| r.contains("resumable")),
        "the operator must see the conversation survived: {}",
        blocked[0]
    );
    assert!(
        events_of(&core, work_id, KIND_WORK_FAILED).is_empty(),
        "never silently failed"
    );

    // Reconciliation reattached: it did not merely look at the conversation,
    // it re-adopted it, and the journal records which of the two happened.
    let reconciled = events_of(&core, work_id, KIND_EXECUTION_RECONCILED);
    assert_eq!(reconciled.len(), 1);
    assert_eq!(
        reconciled[0]["reattached"], true,
        "the surviving conversation must be re-adopted, not just observed: {}",
        reconciled[0]
    );
    assert_eq!(
        claude.tracked_executions(),
        vec![format!("exec-{work_id}")],
        "the adapter owns the journaled execution again"
    );
    // Reattaching created no second execution and spent no tokens: one
    // `execution.started` in the journal, and the same native identity.
    assert_eq!(events_of(&core, work_id, KIND_EXECUTION_STARTED).len(), 1);

    // The blocked work's own reason says it was re-adopted, so an operator
    // reading the journal can tell "sergeant owns this again, the turn's
    // result is unknown" from "sergeant never got it back" — and it claims
    // no interrupt nobody requested.
    let reason = blocked[0]["reason"].as_str().unwrap_or_default();
    assert!(reason.contains("re-adopted"), "{reason}");
    assert!(
        !reason.contains("interrupt"),
        "adoption must not be reported as an interrupted turn: {reason}"
    );

    // §15 RESUME is idempotent — restart reconciliation can re-run after a
    // crash inside its own append window (L6) — and a handle with the
    // journaled identity is still accepted afterwards.
    let handle = ExecutionHandle {
        execution_id: format!("exec-{work_id}"),
        native_id: Some(session_id.to_string()),
    };
    claude
        .resume(
            &handle,
            &ResumeRequest {
                work_id: work_id.to_string(),
                cwd: data.path().to_path_buf(),
                model: Some("haiku".to_string()),
                profile: None,
            },
        )
        .expect("re-adopt is idempotent");
    assert_eq!(claude.tracked_executions().len(), 1, "no second adoption");
    // ...and reconciliation is idempotent: the work is no longer active,
    // so a second restart re-derives nothing (L6: every append window in
    // reconcile re-runs safely).
    let again = recovery::reconcile(&engine, &mut core).expect("second reconcile");
    assert!(again.resumed.is_empty() && again.blocked.is_empty());
}

/// The ruled §25 wiring, on the deterministic backend: when reconciliation
/// evidence is unambiguously resumable, the engine **reattaches and drives**
/// instead of parking the work — and it does so without creating a second
/// execution.
///
/// The fake's `durable` shape is the ordinary restart: the native context
/// finished the stage and exited while the daemon was down. Before the
/// wiring, reconcile only observed, so `Backend::resume` had no production
/// caller at all and a resumable execution was never re-adopted by anything.
#[test]
fn a4_reconcile_reattaches_a_resumable_execution_before_it_classifies() {
    let data = TempDir::new().expect("tempdir");
    let fake = FakeBackend::new(FAKE_BACKEND_NAME);
    let registry = BackendRegistry::new().with(Arc::new(fake.clone()));
    let engine = Engine::new(Arc::new(registry), None, data.path());
    let mut core = core(data.path());

    let work_id = "01M4REATTACH";
    let execution_id = format!("exec-{work_id}");
    let handle = fake
        .start(&start_request(
            &execution_id,
            data.path(),
            "reattach me",
            None,
        ))
        .expect("fake start");
    journal_active_run(
        &mut core,
        work_id,
        FAKE_BACKEND_NAME,
        handle.native_id.as_deref().expect("native id"),
    );
    journal_surface(&mut core, work_id, data.path());
    // While the daemon was down the stage finished and the context exited.
    fake.complete_live_executions();

    let report = recovery::reconcile(&engine, &mut core).expect("reconcile");
    assert_eq!(report.resumed, vec![work_id.to_string()]);
    assert_eq!(
        core.registry.state().works[work_id].state,
        WorkState::Completed,
        "an unambiguously resumable execution is resumed, not parked"
    );

    // Reattached through §15 RESUME, with the launch configuration rebuilt
    // from the journal — the work it serves and the surface later turns run
    // in — and asked exactly once.
    let resumes = fake.resume_requests();
    assert_eq!(resumes.len(), 1, "reattached once: {resumes:?}");
    assert_eq!(resumes[0].0, execution_id);
    assert_eq!(resumes[0].1.work_id, work_id);
    assert_eq!(resumes[0].1.cwd, data.path());
    let reconciled = events_of(&core, work_id, KIND_EXECUTION_RECONCILED);
    assert_eq!(reconciled[0]["reattached"], true);
    assert_eq!(reconciled[0]["disposition"], "resumed");

    // And no duplicate execution: reattaching adopts the recorded context,
    // it does not START another one (§25: "no new worker is created until
    // prior ownership is reconciled").
    assert_eq!(
        fake.starts().len(),
        1,
        "recovery must not start a second execution: {:?}",
        fake.starts()
    );
    assert_eq!(events_of(&core, work_id, KIND_EXECUTION_STARTED).len(), 1);
}

/// The other half of the ruling: evidence that is *not* unambiguously
/// resumable stays fail-closed. An adapter that refuses to re-adopt is the
/// §25 ambiguity, and the work blocks with the refusal as its evidence —
/// before anything is observed, and without the engine second-guessing it.
#[test]
fn a4_reconcile_that_cannot_reattach_stays_fail_closed() {
    let data = TempDir::new().expect("tempdir");
    // A fresh instance: this backend has never heard of the execution.
    let fake = FakeBackend::new(FAKE_BACKEND_NAME);
    let registry = BackendRegistry::new().with(Arc::new(fake.clone()));
    let engine = Engine::new(Arc::new(registry), None, data.path());
    let mut core = core(data.path());

    let work_id = "01M4NOREATTACH";
    journal_active_run(&mut core, work_id, FAKE_BACKEND_NAME, "fake-session-gone");
    journal_surface(&mut core, work_id, data.path());

    let report = recovery::reconcile(&engine, &mut core).expect("reconcile");
    assert_eq!(report.blocked, vec![work_id.to_string()]);
    assert_eq!(
        core.registry.state().works[work_id].state,
        WorkState::Blocked
    );
    let reconciled = events_of(&core, work_id, KIND_EXECUTION_RECONCILED);
    assert_eq!(reconciled[0]["disposition"], "ambiguous");
    assert_eq!(reconciled[0]["reattached"], false);
    let evidence = reconciled[0]["evidence"].as_str().unwrap_or_default();
    assert!(
        evidence.contains("could not reattach") && evidence.contains("does not recognise"),
        "the refusal, in the adapter's own words: {evidence}"
    );
    assert!(
        fake.observations().is_empty(),
        "a context that could not be re-adopted is not classified from an \
         observation of it: {:?}",
        fake.observations()
    );
    assert!(
        fake.starts().is_empty(),
        "and nothing is started speculatively"
    );
}

/// A backend that does not advertise RESUME is not asked for it, and
/// reconciles exactly as it did before the reattach step existed (§15:
/// unsupported means unsupported — the engine does not call a verb a
/// capability denies).
#[test]
fn a4_reconcile_does_not_ask_a_backend_that_cannot_resume() {
    let data = TempDir::new().expect("tempdir");
    let fake = FakeBackend::new(FAKE_BACKEND_NAME);
    assert!(fake.capabilities().resume, "the fake does advertise it");
    let registry = BackendRegistry::new().with(Arc::new(fake.clone()));
    let engine = Engine::new(Arc::new(registry), None, data.path());
    let mut core = core(data.path());

    // No surface journaled: a run whose surface record a crash lost has
    // nowhere to reattach into, so the engine does not invent one — and the
    // classification from OBSERVE alone still drives it.
    let work_id = "01M4NOSURFACE";
    let execution_id = format!("exec-{work_id}");
    let handle = fake
        .start(&start_request(
            &execution_id,
            data.path(),
            "no surface",
            None,
        ))
        .expect("fake start");
    journal_active_run(
        &mut core,
        work_id,
        FAKE_BACKEND_NAME,
        handle.native_id.as_deref().expect("native id"),
    );
    fake.complete_live_executions();

    let report = recovery::reconcile(&engine, &mut core).expect("reconcile");
    assert_eq!(report.resumed, vec![work_id.to_string()]);
    assert!(
        fake.resume_requests().is_empty(),
        "no surface, no ResumeRequest to build honestly: {:?}",
        fake.resume_requests()
    );
    let reconciled = events_of(&core, work_id, KIND_EXECUTION_RECONCILED);
    assert_eq!(reconciled[0]["reattached"], false);
    assert_eq!(
        core.registry.state().works[work_id].state,
        WorkState::Completed
    );
}

/// Acceptance 4b: a session that no longer exists → the execution is
/// retired (`execution.stopped` journaled at reconcile), the work is
/// `blocked` with the adapter's evidence, and nothing invents a failure.
#[test]
fn a4_restart_with_vanished_session_retires_the_execution_and_blocks() {
    let data = TempDir::new().expect("tempdir");
    let home = TempDir::new().expect("tempdir"); // empty: no transcript
    let session_id = "99999999-8888-4777-8666-555555555555";

    let mut config = ClaudeConfig::new(data.path());
    config.claude_home = Some(home.path().to_path_buf());
    let claude = Arc::new(ClaudeBackend::new(config));
    let registry = BackendRegistry::new().with(claude.clone());
    let engine = Engine::new(Arc::new(registry), None, data.path());

    let mut core = core(data.path());
    let work_id = "01M4RECOVERB";
    journal_active_run(&mut core, work_id, CLAUDE_BACKEND_NAME, session_id);

    let report = recovery::reconcile(&engine, &mut core).expect("reconcile");
    assert_eq!(report.blocked, vec![work_id.to_string()]);
    assert_eq!(
        core.registry.state().works[work_id].state,
        WorkState::Blocked
    );

    let reconciled = events_of(&core, work_id, KIND_EXECUTION_RECONCILED);
    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0]["disposition"], "ambiguous");
    assert!(
        reconciled[0]["evidence"]
            .as_str()
            .is_some_and(|e| e.contains("does not recognise")),
        "{}",
        reconciled[0]
    );
    // Retired: the stale execution identity is never reachable again.
    let stopped = events_of(&core, work_id, KIND_EXECUTION_STOPPED);
    assert_eq!(stopped.len(), 1, "execution must be retired at reconcile");
    assert!(
        stopped[0]["reason"]
            .as_str()
            .is_some_and(|r| r.contains("retired at reconcile")),
        "{}",
        stopped[0]
    );
    assert!(
        events_of(&core, work_id, KIND_WORK_FAILED).is_empty(),
        "blocked with evidence, never silently failed"
    );
    // And RESUME refuses to fabricate a context for it.
    let handle = ExecutionHandle {
        execution_id: format!("exec-{work_id}"),
        native_id: Some(session_id.to_string()),
    };
    assert!(matches!(
        claude.resume(&handle, &ResumeRequest::new(work_id, data.path())),
        Err(BackendError::UnknownExecution { .. })
    ));
}

/// L6 audit for the append sequence this milestone added to reconcile
/// (`execution.reconciled` → `stage.blocked` → `execution.stopped` →
/// `work.blocked`): a daemon that crashes *inside* that window leaves the
/// work still `active`, so the next restart re-runs reconciliation — and
/// the re-run must converge (work blocked, execution not re-retired)
/// instead of erroring or double-appending the retirement.
#[test]
fn a4_a_crash_inside_the_reconcile_append_window_rederives_on_restart() {
    let data = TempDir::new().expect("tempdir");
    let home = TempDir::new().expect("tempdir"); // no transcript: ambiguous
    let session_id = "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee";

    let mut config = ClaudeConfig::new(data.path());
    config.claude_home = Some(home.path().to_path_buf());
    let registry = BackendRegistry::new().with(Arc::new(ClaudeBackend::new(config)));
    let engine = Engine::new(Arc::new(registry), None, data.path());

    let mut core = core(data.path());
    let work_id = "01M4RECOVERC";
    journal_active_run(&mut core, work_id, CLAUDE_BACKEND_NAME, session_id);
    // The surviving prefix of a reconcile that died mid-sequence: the
    // execution was already reconciled and retired, but the work-level
    // block never landed.
    commit(
        &mut core,
        work_id,
        KIND_EXECUTION_RECONCILED,
        json!({
            "execution_id": format!("exec-{work_id}"),
            "backend": CLAUDE_BACKEND_NAME,
            "disposition": "ambiguous",
            "evidence": "first attempt, daemon died before blocking",
        }),
    );
    commit(
        &mut core,
        work_id,
        KIND_EXECUTION_STOPPED,
        json!({
            "execution_id": format!("exec-{work_id}"),
            "backend": CLAUDE_BACKEND_NAME,
            "reason": "retired at reconcile: unrecognized",
            "outcome": {"requested": true},
        }),
    );

    let report = recovery::reconcile(&engine, &mut core).expect("re-run reconcile");
    assert_eq!(report.blocked, vec![work_id.to_string()]);
    assert_eq!(
        core.registry.state().works[work_id].state,
        WorkState::Blocked
    );
    let stopped = events_of(&core, work_id, KIND_EXECUTION_STOPPED);
    assert_eq!(
        stopped.len(),
        1,
        "the retirement is idempotent across the crash window (stop_requested folds)"
    );
    assert!(events_of(&core, work_id, KIND_WORK_FAILED).is_empty());
}

/// A stop attempt that never reached a native context must not disable the
/// next one — most of all not a human's cancel.
///
/// Reconciliation retires an unreconcilable execution, and that retirement is
/// idempotent through `stop_requested`. But the latch has to mean *the
/// backend was asked and did not refuse*: an attempt journaled as
/// `{"requested": false, "error": "backend not registered"}` (or one whose
/// backend refused outright) never reached the native context at all, and
/// latching on it turned every later stop into a permanent no-op — including
/// `retire_run`'s, which is the path a cancel takes. With the real Claude
/// adapter the cost is a live conversation nobody can ever kill again while
/// the journal records `requested: true`.
#[test]
fn a_stop_attempt_that_never_reached_the_backend_does_not_disable_the_next_one() {
    let data = TempDir::new().expect("tempdir");
    let fake = FakeBackend::new(FAKE_BACKEND_NAME);
    let work_id = "01M4STOPLATCH";
    let execution_id = format!("exec-{work_id}");

    // The daemon that reconciles has *no* backend registered under the name
    // the journal records, so the stop it journals never asks anything.
    let unaware = Engine::new(Arc::new(BackendRegistry::new()), None, data.path());
    let mut core = core(data.path());
    let handle = fake
        .start(&start_request(
            &execution_id,
            data.path(),
            "cancel me",
            None,
        ))
        .expect("fake start");
    journal_active_run(
        &mut core,
        work_id,
        FAKE_BACKEND_NAME,
        handle.native_id.as_deref().expect("native id"),
    );

    unaware
        .reconcile_work(&mut core, work_id)
        .expect("reconcile");
    let stopped = events_of(&core, work_id, KIND_EXECUTION_STOPPED);
    assert_eq!(stopped.len(), 1);
    assert_eq!(
        stopped[0]["outcome"]["requested"], false,
        "the record must not claim a request that was never made: {}",
        stopped[0]
    );
    assert!(
        fake.stop_requests().is_empty(),
        "nothing was asked to stop: {:?}",
        fake.stop_requests()
    );

    // Now a daemon that *does* have the backend cancels the work. The
    // earlier unanswered attempt must not stand in for this one.
    let aware = Engine::new(
        Arc::new(BackendRegistry::new().with(Arc::new(fake.clone()))),
        None,
        data.path(),
    );
    aware
        .retire_run(&mut core, work_id, "canceled by user")
        .expect("cancel");
    assert_eq!(
        fake.stop_requests(),
        vec![execution_id.clone()],
        "the cancel must actually reach the native context"
    );
    let stopped = events_of(&core, work_id, KIND_EXECUTION_STOPPED);
    assert_eq!(stopped.len(), 2, "and it is journaled in its own right");
    assert_eq!(stopped[1]["outcome"]["requested"], true);
    assert!(stopped[1]["outcome"]["error"].is_null());

    // An acknowledged stop *does* latch: the next one is a no-op, which is
    // what keeps reconcile's retirement idempotent across a crash (L6).
    aware
        .retire_run(&mut core, work_id, "canceled again")
        .expect("second cancel");
    assert_eq!(
        fake.stop_requests().len(),
        1,
        "an acknowledged stop is not repeated: {:?}",
        fake.stop_requests()
    );
    assert_eq!(events_of(&core, work_id, KIND_EXECUTION_STOPPED).len(), 2);
}

/// Acceptance 4, the liveness half: after a restart, the adapter reports
/// what it can *evidence* about the pre-restart turn's process, and nothing
/// more.
///
/// A daemon's death does not kill the `claude` it spawned, so a surviving
/// transcript is not evidence that the turn ended — that inference is §37's
/// "worker reports done but the native session is alive", committed by the
/// adapter. Liveness comes from the session id the turn carries in its own
/// argv: while a process carries it, `native` is `running`; once none does,
/// `exited` is a fact rather than a guess. Either way the work fails closed.
#[test]
fn a4_restart_reports_a_still_running_turn_as_running_not_exited() {
    let data = TempDir::new().expect("tempdir");
    let home = TempDir::new().expect("tempdir");
    let session_id = fresh_session_id();
    let project = home.path().join("projects").join("-work-surface");
    std::fs::create_dir_all(&project).expect("project dir");
    std::fs::write(project.join(format!("{session_id}.jsonl")), "{}\n").expect("transcript");

    // A process that carries the session id in its argv, exactly as a turn
    // this adapter launched does.
    let mut orphan = spawn_turn_stand_in(&session_id);

    let mut config = ClaudeConfig::new(data.path());
    config.claude_home = Some(home.path().to_path_buf());
    let claude = ClaudeBackend::new(config); // a *fresh* adapter: the restart
    let handle = ExecutionHandle {
        execution_id: "exec-orphan".to_string(),
        native_id: Some(session_id.to_string()),
    };

    let observed = claude.observe(&handle).expect("observe");
    assert_eq!(
        observed.native,
        NativeState::Running,
        "a turn that outlived the daemon is running, transcript or no transcript: {observed:?}"
    );
    let evidence = observed.evidence.clone().unwrap_or_default();
    assert!(
        evidence.contains(&orphan.id().to_string()),
        "the evidence names the live process: {evidence}"
    );
    assert!(
        matches!(observed.signal, BackendSignal::Blocked { .. }),
        "an unowned live turn fails the work closed: {observed:?}"
    );

    // And RESUME refuses to adopt it while it runs: this adapter would hold
    // no child handle to interrupt and no stdout to ingest, and a later SEND
    // would put a second process on a session the first one still holds.
    let refused = claude
        .resume(&handle, &ResumeRequest::new("01M4ORPHANWORK", data.path()))
        .expect_err("an unowned live turn cannot be re-adopted");
    assert!(
        refused.to_string().contains("still running"),
        "the refusal says what could not be evidenced: {refused}"
    );

    orphan.kill().expect("kill the orphan");
    orphan.wait().expect("reap the orphan");
    // `wait` reaps *this* child, but a sibling test thread mid-`fork` can
    // still be carrying an inherited copy of an argv for a moment. Liveness
    // is a property of the machine, so the assertion is on where it settles,
    // with a deadline — not on the first sample after the kill.
    let deadline = Instant::now() + Duration::from_secs(10);
    let settled = loop {
        let settled = claude.observe(&handle).expect("observe");
        if settled.native == NativeState::Exited {
            break settled;
        }
        assert!(
            Instant::now() < deadline,
            "no process carries the session any more, but liveness never \
             settled: {settled:?}"
        );
        std::thread::sleep(Duration::from_millis(50));
    };
    assert!(
        settled
            .evidence
            .as_deref()
            .unwrap_or("")
            .contains("no live process"),
        "{settled:?}"
    );
}

/// Liveness is evidence about a *turn of this conversation*, and the argv
/// scan has to be able to tell one from a process that merely quotes the id.
///
/// Both halves run against real processes and the real `/proc`, because the
/// bug this pins was in the scan, not in the matcher: the adapter used to ask
/// whether the joined command line *contained* the id, so a shell wrapper
/// quoting it (an operator's `grep`, or any harness that runs commands as
/// `bash -c '<text>'` — this project's build environment does) was reported
/// as `NativeState::Running` with "pid N carries session id in argv". It also
/// made the suite flaky: a concurrent shell was enough to flip an assertion
/// about a conversation with no turn running at all.
#[test]
fn liveness_reads_turn_argv_and_ignores_processes_that_merely_quote_the_id() {
    use sergeant_rs::backend::claude::{Liveness, session_liveness, session_liveness_excluding};
    let session_id = fresh_session_id();

    // A bystander: the id appears inside one argv element, as any shell
    // wrapper or file-reading tool would carry it.
    let mut bystander = std::process::Command::new("sh")
        .args(["-c", &format!("read line # less {session_id}.jsonl")])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("spawn a bystander");
    let bystander_cmdline = PathBuf::from(format!("/proc/{}/cmdline", bystander.id()));
    let deadline = Instant::now() + Duration::from_secs(30);
    while !String::from_utf8_lossy(&std::fs::read(&bystander_cmdline).unwrap_or_default())
        .contains(&session_id)
    {
        assert!(Instant::now() < deadline, "the bystander never exec'd");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        session_liveness(&session_id),
        Liveness::Dead,
        "a process that quotes the session id is not a turn of it"
    );

    // A real turn's argv shape, next to the bystander that is still running.
    let mut turn = spawn_turn_stand_in(&session_id);
    let turn_pid = turn.id();
    assert_eq!(
        session_liveness(&session_id),
        Liveness::Alive(turn_pid),
        "the live turn is found, and it is the one that was spawned"
    );

    // The own-pid skip: sergeant's own process must never report itself as a
    // live turn (a `sgt` invocation or a test binary can carry a session id
    // in its argv). Excluding the stand-in is the same question asked about
    // a different "self", which is the only way to pin the rule from here.
    assert_eq!(
        session_liveness_excluding(&session_id, turn_pid),
        Liveness::Dead,
        "the excluded pid is not evidence about anyone else"
    );

    turn.kill().expect("kill the stand-in");
    turn.wait().expect("reap it");
    bystander.kill().expect("kill the bystander");
    bystander.wait().expect("reap it");
}

/// L6 audit of the *other* append window this milestone owns: START spawns a
/// token-burning turn before the engine journals `execution.started`. A
/// daemon that dies in between leaves a work `active` with a stage entered
/// and no execution recorded. The next restart must fail it closed with that
/// stated plainly — never resume a stage whose execution nothing can name.
///
/// (What this test cannot assert, and the adapter's module docs say so: the
/// orphaned process itself is unrecoverable evidence, because its session id
/// exists in no journal. Closing that needs a two-phase START.)
#[test]
fn a4_a_crash_between_spawn_and_execution_started_blocks_the_work() {
    let data = TempDir::new().expect("tempdir");
    let mut config = ClaudeConfig::new(data.path());
    config.claude_home = Some(data.path().to_path_buf());
    let registry = BackendRegistry::new().with(Arc::new(ClaudeBackend::new(config)));
    let engine = Engine::new(Arc::new(registry), None, data.path());
    let mut core = core(data.path());

    let work_id = "01M4STARTWIN";
    submit_work(&mut core, work_id, "crashed between spawn and journal");
    commit(
        &mut core,
        work_id,
        KIND_WORKFLOW_BOUND,
        json!({
            "workflow": {"name": "tiny", "version": "1", "source": "test",
                         "stages": [{"id": "00-only", "context": "c"}]},
            "backend": CLAUDE_BACKEND_NAME,
        }),
    );
    commit(
        &mut core,
        work_id,
        KIND_STAGE_ENTERED,
        json!({"stage_id": "00-only", "index": 0, "attempt": 1}),
    );
    commit(&mut core, work_id, KIND_WORK_STARTED, json!({}));
    // …and the daemon died here: the turn is running, `execution.started`
    // never landed.

    let report = recovery::reconcile(&engine, &mut core).expect("reconcile");
    assert_eq!(report.blocked, vec![work_id.to_string()]);
    assert_eq!(
        core.registry.state().works[work_id].state,
        WorkState::Blocked
    );
    let reconciled = events_of(&core, work_id, KIND_EXECUTION_RECONCILED);
    assert_eq!(reconciled[0]["disposition"], "ambiguous");
    assert!(
        reconciled[0]["evidence"]
            .as_str()
            .is_some_and(|e| e.contains("no recorded execution")),
        "the record says what was missing: {}",
        reconciled[0]
    );
    assert!(
        events_of(&core, work_id, KIND_WORK_FAILED).is_empty(),
        "never silently failed"
    );
}

/// A per-turn process that dies without a result envelope, unrequested, is
/// ambiguity and fails closed (the stub exits instantly with no output —
/// the same shape as an OOM-killed or crashed turn).
#[test]
fn a4_a_turn_that_dies_without_an_envelope_is_ambiguous_not_a_verdict() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubClaude::passing(dir.path());
    let mut config = ClaudeConfig::new(dir.path());
    config.executable = stub.path.clone();
    let backend = ClaudeBackend::new(config);

    let handle = backend
        .start(&start_request(
            "e-dead",
            dir.path(),
            "will die silently",
            None,
        ))
        .expect("start");
    let observation = wait_settled(&backend, &handle, Duration::from_secs(10));
    assert_eq!(
        observation.native,
        NativeState::Unknown,
        "no envelope + no interrupt request = ambiguity: {observation:?}"
    );
    assert_eq!(
        observation.signal,
        BackendSignal::Running,
        "no verdict is invented"
    );
}

// ------------------------------------------------------- 7. version gate

/// Acceptance 7: the adapter refuses to launch on a CLI the contract tests
/// never measured — old version, missing flag, or unparseable version —
/// and the structured error names the probe evidence each time.
#[test]
fn a7_version_gate_fails_closed_naming_the_probe() {
    let dir = TempDir::new().expect("tempdir");
    let request = start_request("e-gate", dir.path(), "irrelevant", None);

    // Version below the measured minimum.
    let old = StubClaude::new(dir.path(), "2.1.220 (Claude Code)", ALL_FLAGS);
    let mut config = ClaudeConfig::new(dir.path());
    config.executable = old.path.clone();
    let backend = ClaudeBackend::new(config);
    assert!(!backend.probe().available);
    match backend.start(&request).expect_err("must refuse") {
        BackendError::Unavailable { detail, .. } => {
            assert!(detail.contains("2.1.220"), "{detail}");
            assert!(detail.contains("minimum trusted 2.1.226"), "{detail}");
        }
        other => panic!("expected Unavailable, got {other}"),
    }

    // A required flag missing from --help.
    let dir2 = TempDir::new().expect("tempdir");
    let flags: Vec<&str> = ALL_FLAGS
        .iter()
        .copied()
        .filter(|f| *f != "--resume")
        .collect();
    let flagless = StubClaude::new(dir2.path(), "2.1.226 (Claude Code)", &flags);
    let mut config = ClaudeConfig::new(dir2.path());
    config.executable = flagless.path.clone();
    let backend = ClaudeBackend::new(config);
    match backend.start(&request).expect_err("must refuse") {
        BackendError::Unavailable { detail, .. } => {
            assert!(
                detail.contains("--resume"),
                "the refusal names the missing flag: {detail}"
            );
        }
        other => panic!("expected Unavailable, got {other}"),
    }

    // A version that does not parse is an unmeasurable CLI: refused.
    let dir3 = TempDir::new().expect("tempdir");
    let weird = StubClaude::new(dir3.path(), "nightly-build", ALL_FLAGS);
    let mut config = ClaudeConfig::new(dir3.path());
    config.executable = weird.path.clone();
    let backend = ClaudeBackend::new(config);
    match backend.start(&request).expect_err("must refuse") {
        BackendError::Unavailable { detail, .. } => {
            assert!(detail.contains("cannot parse"), "{detail}");
        }
        other => panic!("expected Unavailable, got {other}"),
    }

    // And a stub that answers everything the probe requires passes it.
    let dir4 = TempDir::new().expect("tempdir");
    let good = StubClaude::passing(dir4.path());
    let mut config = ClaudeConfig::new(dir4.path());
    config.executable = good.path.clone();
    let backend = ClaudeBackend::new(config);
    let probe = backend.probe();
    assert!(probe.available, "{probe:?}");
    assert!(probe.detail.unwrap_or_default().contains("2.1.226"));
}

// ---------------------------------------- 6. the Sergeant regression catalog
//
// Each test recreates one failure class §37 lists, citing where Sergeant
// bled from it. Provenance pointers are into `reference/sergeant-upstream`.

/// §37: "worker reports completion but native session remains alive".
/// Origin: the background-harness spike's motivating defect — a live, idle
/// `claude` found 4h25m after its task completed, still burning CPU
/// (docs/research/claude-background-harness-spike.md, "The defect this
/// spike is scoped to fix"); also docs/troubleshooting.md "Worker says
/// in_progress but is not moving" ("a live parent process is
/// insufficient"). The work completes on the explicit signal; the deathless
/// native context changes nothing.
#[test]
fn r1_worker_reports_done_but_native_session_stays_alive() {
    let data = TempDir::new().expect("tempdir");
    let step = FakeStep {
        native: NativeState::Running,
        signal: BackendSignal::StageCompleted { summary: None },
        ignores_stop: true, // the session that will not die
    };
    let fake = FakeBackend::scripted(FAKE_BACKEND_NAME, [step]);
    let registry = BackendRegistry::new().with(Arc::new(fake.clone()));
    let engine = Engine::new(Arc::new(registry), None, data.path());
    let mut core = core(data.path());

    let work_id = "01M4CATALOG1";
    let handle = fake
        .start(&start_request(
            "exec-01M4CATALOG1",
            data.path(),
            "finish",
            None,
        ))
        .expect("fake start");
    journal_active_run(
        &mut core,
        work_id,
        FAKE_BACKEND_NAME,
        handle.native_id.as_deref().expect("native id"),
    );

    engine.resume(&mut core, work_id).expect("drive");
    assert_eq!(
        core.registry.state().works[work_id].state,
        WorkState::Completed
    );
    assert!(
        fake.is_live("exec-01M4CATALOG1"),
        "the native session outlived completion — and that must not matter"
    );
    // Sergeant asked it to stop and recorded that it asked; compliance is
    // the backend's problem, observed, never assumed.
    assert_eq!(fake.stop_requests(), vec!["exec-01M4CATALOG1"]);
}

/// §37: "daemon dies during delivery". Origin: Sergeant's response
/// generation/acknowledgement machinery (docs/troubleshooting.md "Response
/// already pending" — a response could be recorded but not yet applied,
/// and recovery must not lose or double it). Here: the daemon journals the
/// received input and dies before the backend SEND; restart finds the work
/// active, cannot classify the execution against a fresh backend, and
/// fails closed to blocked — the recorded input survives in the journal,
/// nothing is silently dropped or re-sent.
#[test]
fn r2_daemon_dies_during_delivery_fails_closed_with_the_input_preserved() {
    let data = TempDir::new().expect("tempdir");
    // The restarted daemon has a *fresh* fake: the old in-process backend
    // died with the old daemon, so the execution is unrecognisable.
    let fake = FakeBackend::new(FAKE_BACKEND_NAME);
    let registry = BackendRegistry::new().with(Arc::new(fake.clone()));
    let engine = Engine::new(Arc::new(registry), None, data.path());
    let mut core = core(data.path());

    let work_id = "01M4CATALOG2";
    journal_active_run(&mut core, work_id, FAKE_BACKEND_NAME, "fake-session-old");
    commit(
        &mut core,
        work_id,
        KIND_STAGE_NEEDS_INPUT,
        json!({"stage_id": "00-only", "detail": "which color?"}),
    );
    commit(
        &mut core,
        work_id,
        KIND_WORK_NEEDS_INPUT,
        json!({"prompt": "which color?"}),
    );
    // The crash window: input journaled, resume journaled, SEND never ran.
    commit(
        &mut core,
        work_id,
        KIND_STAGE_INPUT_RECEIVED,
        json!({"stage_id": "00-only", "input": "blue"}),
    );
    commit(
        &mut core,
        work_id,
        KIND_WORK_RESUMED,
        json!({"reason": "input_received"}),
    );

    let report = recovery::reconcile(&engine, &mut core).expect("reconcile");
    assert_eq!(report.blocked, vec![work_id.to_string()]);
    assert_eq!(
        core.registry.state().works[work_id].state,
        WorkState::Blocked
    );
    let inputs = events_of(&core, work_id, KIND_STAGE_INPUT_RECEIVED);
    assert_eq!(inputs.len(), 1, "the delivered answer is journal-preserved");
    assert_eq!(inputs[0]["input"], "blue");
    assert!(
        fake.inputs("exec-01M4CATALOG2").is_empty(),
        "recovery must not re-deliver input on its own authority"
    );
}

/// §37: "native process dies after work is preserved". Origin:
/// docs/troubleshooting.md "Worker became orphaned after blocking" — "it
/// is not an orphan merely because the process ended". A native context
/// that finished its stage and exited while the daemon was down resumes to
/// completion; a dead process is not a failure.
#[test]
fn r3_native_dies_after_work_preserved_is_not_a_failure() {
    let data = TempDir::new().expect("tempdir");
    let fake = FakeBackend::new(FAKE_BACKEND_NAME);
    let registry = BackendRegistry::new().with(Arc::new(fake.clone()));
    let engine = Engine::new(Arc::new(registry), None, data.path());
    let mut core = core(data.path());

    let work_id = "01M4CATALOG3";
    let handle = fake
        .start(&start_request(
            "exec-01M4CATALOG3",
            data.path(),
            "finish then die",
            None,
        ))
        .expect("fake start");
    journal_active_run(
        &mut core,
        work_id,
        FAKE_BACKEND_NAME,
        handle.native_id.as_deref().expect("native id"),
    );
    // While the daemon was down: the stage finished and the context exited.
    fake.complete_live_executions();

    let report = recovery::reconcile(&engine, &mut core).expect("reconcile");
    assert_eq!(report.resumed, vec![work_id.to_string()]);
    assert_eq!(
        core.registry.state().works[work_id].state,
        WorkState::Completed,
        "dead process + completed signal = completed work, never failed"
    );
    assert!(events_of(&core, work_id, KIND_WORK_FAILED).is_empty());
}

/// §37: "same command delivered twice". Origin: Sergeant's nonce files and
/// duplicate-delivery protection (proposal §26 names them; upstream
/// docs/troubleshooting.md "Repeated notifications" — "do not create
/// duplicate tasks or send duplicate responses"). A repeated command_id
/// replays the recorded outcome; exactly one work exists.
#[tokio::test]
async fn r4_duplicate_command_delivery_replays_instead_of_re_executing() {
    let data = TempDir::new().expect("tempdir");
    let handle = daemon::start_with(data.path(), DaemonConfig::default())
        .await
        .expect("daemon");
    let client = reqwest::Client::new();
    let command_id = ulid();
    let body = json!({"command_id": command_id, "intent": "once only"});
    let mut responses = Vec::new();
    for _ in 0..2 {
        let response = client
            .post(format!("{}/v1/work", handle.endpoint))
            .bearer_auth(&handle.token)
            .json(&body)
            .send()
            .await
            .expect("submit");
        responses.push((response.status(), response.text().await.expect("body")));
    }
    assert_eq!(responses[0], responses[1], "byte-identical replay");
    let list: Value = client
        .get(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("list")
        .json()
        .await
        .expect("json");
    assert_eq!(list["works"].as_array().expect("works").len(), 1);
    handle.shutdown().await;
}

/// §37: "old execution identity reused". Origin: Sergeant's pane-identity
/// discipline (docs/troubleshooting.md "Worker says in_progress...", which
/// demands "exact recorded tmux pane identity", and "Pane is missing" —
/// identity mismatch is orphan evidence, not a target to guess at). A
/// journaled execution whose native identity does not match what the
/// backend issued reconciles ambiguous and is retired — never adopted.
#[test]
fn r5_stale_execution_identity_is_refused_not_adopted() {
    let data = TempDir::new().expect("tempdir");
    let fake = FakeBackend::new(FAKE_BACKEND_NAME);
    let registry = BackendRegistry::new().with(Arc::new(fake.clone()));
    let engine = Engine::new(Arc::new(registry), None, data.path());
    let mut core = core(data.path());

    let work_id = "01M4CATALOG5";
    // The backend really has this execution — under a different identity.
    fake.start(&start_request(
        "exec-01M4CATALOG5",
        data.path(),
        "real",
        None,
    ))
    .expect("fake start");
    // The journal claims a stale native identity for it.
    journal_active_run(
        &mut core,
        work_id,
        FAKE_BACKEND_NAME,
        "stale-session-identity",
    );

    let report = recovery::reconcile(&engine, &mut core).expect("reconcile");
    assert_eq!(report.blocked, vec![work_id.to_string()]);
    let reconciled = events_of(&core, work_id, KIND_EXECUTION_RECONCILED);
    assert_eq!(reconciled[0]["disposition"], "ambiguous");
    assert!(
        fake.inputs("exec-01M4CATALOG5").is_empty() && fake.is_live("exec-01M4CATALOG5"),
        "the real execution was neither fed nor killed through the stale record"
    );
}

/// §37: "client disconnects mid-run". Origin: §25 "Client failure: no
/// consequence — clients do not own execution"; upstream workers likewise
/// survive their human detaching (the spike measured detach never stops a
/// session). An SSE subscriber that vanishes mid-run changes nothing: the
/// work keeps its state and the daemon keeps serving.
#[tokio::test]
async fn r6_client_disconnect_mid_run_has_no_consequence() {
    let data = TempDir::new().expect("tempdir");
    let fake = FakeBackend::scripted(FAKE_BACKEND_NAME, [FakeStep::hang()]);
    let registry = BackendRegistry::new().with(Arc::new(fake.clone()));
    let handle = daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(registry),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            claude: None,
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon");
    let client = reqwest::Client::new();

    // A live event-stream subscriber...
    let stream = client
        .get(format!("{}/v1/events/stream", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("subscribe");
    assert!(stream.status().is_success());

    // ...a run in flight (the hang keeps it active)...
    let repo = TempDir::new().expect("repo");
    init_repo(repo.path());
    let submitted: Value = client
        .post(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&json!({
            "command_id": ulid(),
            "intent": "outlive your client",
            "origin": {"client": "cli", "cwd": repo.path()},
        }))
        .send()
        .await
        .expect("submit")
        .json()
        .await
        .expect("json");
    let work_id = submitted["work"]["id"].as_str().expect("id").to_string();
    assert_eq!(submitted["work"]["state"], "active");

    // ...and the client vanishes, stream and all.
    drop(stream);
    drop(client);

    let fresh = reqwest::Client::new();
    let shown: Value = fresh
        .get(format!("{}/v1/work/{work_id}", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("show")
        .json()
        .await
        .expect("json");
    assert_eq!(
        shown["work"]["state"], "active",
        "clients do not own execution"
    );
    assert!(
        fake.is_live(
            shown["execution"]["execution_id"]
                .as_str()
                .expect("execution id")
        ),
        "the native context did not notice either"
    );
    handle.shutdown().await;
}

/// §37: "work waits for input for hours". Origin: upstream's own doctrine
/// that waiting is a durable state, not a liveness failure —
/// docs/troubleshooting.md "Worker became orphaned after blocking" ("an
/// expected dependency-blocked exit must remain blocked") and the
/// spike-era monitoring rules that treat `blocked` as re-read-state, not
/// timeout. Sergeant has no timeout that kills state: a parked work passes
/// through restart reconciliation untouched — reconciliation acts on
/// `active` work only, because parked states are decisions, not doubt.
#[test]
fn r7_work_waiting_for_input_is_never_timed_out_or_reclassified() {
    let data = TempDir::new().expect("tempdir");
    let fake = FakeBackend::new(FAKE_BACKEND_NAME);
    let registry = BackendRegistry::new().with(Arc::new(fake));
    let engine = Engine::new(Arc::new(registry), None, data.path());
    let mut core = core(data.path());

    let work_id = "01M4CATALOG7";
    journal_active_run(&mut core, work_id, FAKE_BACKEND_NAME, "fake-session-x");
    commit(
        &mut core,
        work_id,
        KIND_STAGE_NEEDS_INPUT,
        json!({"stage_id": "00-only", "detail": "still here?"}),
    );
    commit(
        &mut core,
        work_id,
        KIND_WORK_NEEDS_INPUT,
        json!({"prompt": "still here?"}),
    );

    // However many restarts later — hours of them — the parked state holds.
    for _ in 0..3 {
        let report = recovery::reconcile(&engine, &mut core).expect("reconcile");
        assert!(report.resumed.is_empty() && report.blocked.is_empty());
    }
    assert_eq!(
        core.registry.state().works[work_id].state,
        WorkState::NeedsInput
    );
    assert!(
        events_of(&core, work_id, KIND_WORK_BLOCKED).is_empty(),
        "no timeout reclassified the wait"
    );
}

/// Minimal git repo for daemon-path tests.
fn init_repo(path: &Path) {
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(path)
            .env("GIT_AUTHOR_NAME", "sergeant tests")
            .env("GIT_AUTHOR_EMAIL", "tests@example.invalid")
            .env("GIT_COMMITTER_NAME", "sergeant tests")
            .env("GIT_COMMITTER_EMAIL", "tests@example.invalid")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .output()
            .expect("run git");
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    std::fs::create_dir_all(path).expect("repo dir");
    git(&["init", "-b", "main"]);
    std::fs::write(path.join("README.md"), "# fixture\n").expect("write file");
    git(&["add", "."]);
    git(&["commit", "-m", "initial"]);
}

// -------------------------------------------- 1 & 3. opt-in, real Claude

/// Acceptance 1 (+ the live halves of 2 and 4): one execution = one durable
/// conversation across per-turn processes and even across adapter
/// instances. Three haiku turns, one-line prompts (budget rules).
#[test]
#[ignore = "opt-in, spends real tokens: SERGEANT_CLAUDE_TESTS=1 cargo test -- --ignored"]
fn a1_real_claude_session_identity_survives_turns_and_restart() {
    if !claude_live_enabled("a1_real_claude_session_identity_survives_turns_and_restart") {
        return;
    }
    let data = TempDir::new().expect("tempdir");
    let cwd = TempDir::new().expect("tempdir");
    let backend = ClaudeBackend::new(live_config(data.path()));

    // Normalized events flow through the daemon's own sink into a real core.
    let shared = Arc::new(tokio::sync::Mutex::new(core(data.path())));
    backend.set_event_sink(journaling_sink(shared.clone()));

    let nonce = "SGT-M4-NONCE-7731";
    let request = start_request(
        "e-live-1",
        cwd.path(),
        &format!("Remember the nonce {nonce}. Reply with exactly: OK"),
        Some("haiku"),
    );
    let handle = backend.start(&request).expect("start");
    let session_id = handle
        .native_id
        .clone()
        .expect("session identity captured at start");

    let first = wait_settled(&backend, &handle, Duration::from_secs(180));
    let BackendSignal::StageCompleted { summary } = &first.signal else {
        panic!("first turn must complete: {first:?}");
    };
    assert!(summary.as_deref().unwrap_or("").contains("OK"), "{first:?}");

    // Turn 2 resumes the same session: continuity proven by the nonce.
    backend
        .send(&handle, "What was the nonce? Reply with the nonce only.")
        .expect("send");
    let second = wait_settled(&backend, &handle, Duration::from_secs(180));
    let BackendSignal::StageCompleted { summary } = &second.signal else {
        panic!("second turn must complete: {second:?}");
    };
    assert!(
        summary.as_deref().unwrap_or("").contains(nonce),
        "the conversation must recall turn 1: {second:?}"
    );

    // Journal: normalized events with correlation and causation.
    // Two turns × (conversation.user, assistant.completed, turn.ended,
    // usage.updated), plus any tool events the turns produced.
    let events = wait_for_events(&shared, "e-live-1", 8);
    let mine: Vec<&Event> = events.iter().collect();
    let kinds: Vec<&str> = mine.iter().map(|e| e.kind.as_str()).collect();
    assert!(kinds.contains(&"conversation.user"), "{kinds:?}");
    assert!(
        kinds.contains(&"conversation.assistant.completed"),
        "{kinds:?}"
    );
    assert!(kinds.contains(&"usage.updated"), "{kinds:?}");
    for event in &mine {
        assert_eq!(event.correlation_id.as_deref(), Some("e-live-1"));
    }
    assert!(
        mine.iter().skip(1).all(|e| e.causation_id.is_some()),
        "causation chains within the execution"
    );
    let ids: Vec<&str> = mine.iter().map(|e| e.id.as_str()).collect();
    assert!(
        mine.iter()
            .filter_map(|e| e.causation_id.as_deref())
            .all(|cause| ids.contains(&cause)),
        "every causation points at a journaled event in this execution"
    );

    // Raw stream-json archived: every usage.updated carries a blob ref that
    // resolves, and the bytes are the real transcript of this session.
    let usage_events: Vec<&&Event> = mine.iter().filter(|e| e.kind == "usage.updated").collect();
    assert_eq!(usage_events.len(), 2, "one per turn");
    let store = BlobStore::open(data.path()).expect("blob store");
    for event in &usage_events {
        let raw_ref = event.payload["raw"]
            .as_str()
            .expect("raw blob ref journaled");
        let blob = store
            .get(&BlobRef::from_str(raw_ref).expect("valid ref"))
            .expect("raw transcript archived");
        let text = String::from_utf8_lossy(&blob);
        assert!(
            text.contains(&session_id),
            "the raw lines carry the session id"
        );
        assert!(
            text.contains("\"type\":\"result\""),
            "the result envelope is archived"
        );
        // Acceptance 2, live half: the pin was honored with positive
        // evidence from the result's model fields.
        assert_eq!(
            event.payload["model_pin"]["verdict"], "honored",
            "{}",
            event.payload
        );
        assert!(
            event.payload["model_pin"]["model"]
                .as_str()
                .expect("resolved model")
                .contains("haiku")
        );
    }

    // Acceptance 4, live half: a fresh adapter instance (the restarted
    // daemon) re-adopts the conversation from durable evidence and
    // continues it — same session, same memory.
    let reborn = ClaudeBackend::new(live_config(data.path()));
    // The same core: the journal is single-owner, and the restarted daemon
    // owns the same data dir.
    reborn.set_event_sink(journaling_sink(shared.clone()));
    reborn
        .resume(
            &handle,
            &ResumeRequest {
                // The launch configuration is re-supplied from what sergeant
                // journaled — the adapter fabricates none of it.
                work_id: request.work_id.clone(),
                cwd: cwd.path().to_path_buf(),
                model: Some("haiku".to_string()),
                profile: None,
            },
        )
        .expect("re-adopt from session evidence");
    reborn
        .send(&handle, "Repeat the nonce one more time, nonce only.")
        .expect("send after restart");
    let third = wait_settled(&reborn, &handle, Duration::from_secs(180));
    let BackendSignal::StageCompleted { summary } = &third.signal else {
        panic!("post-restart turn must complete: {third:?}");
    };
    assert!(
        summary.as_deref().unwrap_or("").contains(nonce),
        "the restarted adapter continues the same conversation: {third:?}"
    );
    // The re-supplied pin is enforced and verified on the post-restart turn,
    // and the work binding survives the restart too — a resumed execution
    // that reported "unpinned" with an empty work id would be a pin claim
    // and an attribution both weaker than the journal's own record.
    let after = wait_for_events(&shared, "e-live-1", 12);
    let usage = after
        .iter()
        .rfind(|e| e.kind == "usage.updated")
        .expect("post-restart usage.updated");
    assert_eq!(usage.payload["model_pin"]["verdict"], "honored");
    assert_eq!(usage.work_id.as_deref(), Some(request.work_id.as_str()));

    // §37's backend-contract list names HISTORY and STOP among the verbs
    // every backend must *prove* against the installed harness, and both are
    // pure adapter bookkeeping for this CLI — which is exactly why they need
    // measuring here rather than against a stub.
    //
    // HISTORY: the capability says unsupported, so the verb must refuse. An
    // adapter that answered `Ok([])` here — for a conversation whose
    // transcript is on disk and which just recalled a nonce from three turns
    // ago — would be reporting "nothing was said".
    assert!(!reborn.capabilities().history);
    match reborn.history(&handle) {
        Err(BackendError::Unsupported { verb, .. }) => assert_eq!(verb, "history"),
        other => panic!("the advertised capability and the verb must agree: {other:?}"),
    }

    // STOP: retires the execution without corrupting recoverable state. The
    // durable transcript is still on disk afterwards, and a fresh adapter
    // (another restart) still classifies the conversation as resumable — so
    // "retired" is sergeant's decision about this execution, never damage to
    // the native context.
    let transcript = live_transcript_path(&session_id).expect("the durable transcript exists");
    reborn.stop(&handle).expect("stop");
    let refused = reborn
        .send(&handle, "anything")
        .expect_err("a stopped execution accepts nothing");
    assert!(refused.to_string().contains("stopped"), "{refused}");
    assert!(
        transcript.is_file(),
        "STOP must not touch the durable transcript: {}",
        transcript.display()
    );
    let third_daemon = ClaudeBackend::new(live_config(data.path()));
    let after_stop = third_daemon.observe(&handle).expect("observe");
    assert_eq!(after_stop.native, NativeState::Exited);
    assert!(
        after_stop
            .evidence
            .as_deref()
            .unwrap_or("")
            .contains("transcript"),
        "the conversation survived STOP: {after_stop:?}"
    );
}

/// Where the installed CLI keeps the durable transcript for a session, if it
/// has one: `<claude_home>/projects/*/<session_id>.jsonl`. Only the filename
/// convention is relied on (the same one the adapter uses); the per-cwd
/// directory munging is not.
fn live_transcript_path(session_id: &str) -> Option<PathBuf> {
    let home = std::env::var("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".to_string())).join(".claude")
        });
    let wanted = format!("{session_id}.jsonl");
    for entry in std::fs::read_dir(home.join("projects")).ok()?.flatten() {
        let candidate = entry.path().join(&wanted);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Acceptance 3: interrupt kills the per-turn process mid-generation; no
/// verdict is invented about the stage (work ≠ process), and the
/// conversation resumes with its memory intact.
#[test]
#[ignore = "opt-in, spends real tokens: SERGEANT_CLAUDE_TESTS=1 cargo test -- --ignored"]
fn a3_real_claude_interrupt_leaves_the_conversation_resumable() {
    if !claude_live_enabled("a3_real_claude_interrupt_leaves_the_conversation_resumable") {
        return;
    }
    let data = TempDir::new().expect("tempdir");
    let cwd = TempDir::new().expect("tempdir");
    let backend = ClaudeBackend::new(live_config(data.path()));

    let nonce = "SGT-M4-INT-4402";
    let request = start_request(
        "e-live-3",
        cwd.path(),
        &format!("Remember the nonce {nonce}. Reply with exactly: OK"),
        Some("haiku"),
    );
    let handle = backend.start(&request).expect("start");
    let first = wait_settled(&backend, &handle, Duration::from_secs(180));
    assert!(
        matches!(first.signal, BackendSignal::StageCompleted { .. }),
        "{first:?}"
    );

    // A turn long enough to be killed mid-generation.
    backend
        .send(
            &handle,
            "Write a 500 word story about a lighthouse. No tools.",
        )
        .expect("send long turn");
    // Give the turn time to be genuinely in flight (measured: ~3.5s to
    // first API activity), then kill it.
    std::thread::sleep(Duration::from_secs(6));
    let in_flight = backend.observe(&handle).expect("observe");
    assert_eq!(
        in_flight.native,
        NativeState::Running,
        "the long turn should still be generating when we kill it: {in_flight:?}"
    );
    backend.interrupt(&handle).expect("interrupt");

    let after = wait_settled(&backend, &handle, Duration::from_secs(30));
    assert_eq!(after.native, NativeState::Exited);
    assert_eq!(
        after.signal,
        BackendSignal::Running,
        "an interrupted turn draws no conclusion about the stage: {after:?}"
    );
    assert!(
        after
            .evidence
            .as_deref()
            .unwrap_or("")
            .contains("interrupted"),
        "{after:?}"
    );

    // Work ≠ process: the conversation survived the kill with its memory.
    backend
        .send(&handle, "What was the nonce? Reply with the nonce only.")
        .expect("send after interrupt");
    let recalled = wait_settled(&backend, &handle, Duration::from_secs(180));
    let BackendSignal::StageCompleted { summary } = &recalled.signal else {
        panic!("the resumed turn must complete: {recalled:?}");
    };
    assert!(
        summary.as_deref().unwrap_or("").contains(nonce),
        "the killed turn did not damage the conversation: {recalled:?}"
    );

    // §37 again, on the interrupt path: STOP after an INTERRUPT retires the
    // execution (no turn is in flight to kill, which is a no-op and not an
    // error) and the latch holds against further input.
    backend.stop(&handle).expect("stop");
    let refused = backend
        .send(&handle, "anything")
        .expect_err("a stopped execution accepts nothing");
    assert!(refused.to_string().contains("stopped"), "{refused}");
}
