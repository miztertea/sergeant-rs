//! W1 contract tests for `src/backend/codex.rs` (spec §7.2/§7.3).
//!
//! Two tiers, exactly the spec's own split:
//!
//! - **Stub-driven** (this file's bulk): [`StubCodex`] is a shell script
//!   modelled directly on `tests/m4_backends.rs`'s `StubClaude` — it answers
//!   the capability probe, records every argv/env/cwd/stdin it is launched
//!   with, replays a chosen fixture on stdout, and can hang/stall/write
//!   stderr/exit non-zero on demand. No `codex` binary is required for any
//!   test in this tier.
//! - **Live** (§7.3, gated): every test is `#[ignore]`d and additionally
//!   gated on `SERGEANT_CODEX_TESTS=1`, an available probe, and a
//!   logged-in `codex`. Every live turn pins `-m gpt-5.6-luna` and a bounded,
//!   one-word-answer prompt.
//!
//! No daemon is spawned by any test here (`CodexBackend::new` is constructed
//! directly, the way the claude stub tests do), so `tests/support::DataDir`'s
//! reaper is not required.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;

use sergeant_rs::backend::codex::{CODEX_BACKEND_NAME, CodexBackend, CodexConfig};
use sergeant_rs::backend::{
    Backend, BackendError, BindingSummary, ExecutionHandle, NativeState, ProbeReport,
    ResumeRequest, StartRequest,
};
use sergeant_rs::domain::estate::InstructionPolicy;
use sergeant_rs::domain::event::EventDraft;

mod support;

// ---------------------------------------------------------------- helpers

/// A fresh, UUIDv7-shaped-enough thread id, unique per call — collisions
/// across a `--test-threads` re-run would otherwise make a liveness
/// assertion flip (the exact hazard `tests/m4_backends.rs::fresh_session_id`
/// documents for Claude's session ids).
fn fresh_thread_id() -> String {
    let hex: String = ulid::Ulid::generate()
        .to_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!(
        "{}-{}-7{}-8{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[13..16],
        &hex[17..20],
        &hex[20..32]
    )
}

/// `exec --help` text carrying every [`REQUIRED_EXEC_FLAGS`] entry and the
/// `resume` subcommand — a passing probe's exec-help surface.
const ALL_EXEC_HELP: &str = "\
Usage: codex exec [OPTIONS] [PROMPT]\n\
       codex exec [OPTIONS] <COMMAND> [ARGS]\n\
Commands: resume | fork | review | help\n\
Options: --json --model -m --cd -C --skip-git-repo-check --profile -p --sandbox -s --ephemeral \
--add-dir --ignore-user-config -o --output-last-message";

/// `exec resume --help` text carrying every [`REQUIRED_RESUME_FLAGS`] entry.
const ALL_RESUME_HELP: &str = "\
Usage: codex exec resume [OPTIONS] [SESSION_ID] [PROMPT]\n\
Options: --json --model -m --skip-git-repo-check --ephemeral -o --output-last-message";

const PASSING_VERSION: &str = "codex-cli 0.149.0";
const LOGGED_IN: &str = "Logged in using ChatGPT";

/// A stub `codex` that answers the capability probe (`--version`,
/// `exec --help`, `exec resume --help`, `login status`), **records every
/// launch**, and replays a recorded fixture on stdout.
struct StubCodex {
    path: PathBuf,
    record: PathBuf,
    replay: PathBuf,
    hang: PathBuf,
    stall: PathBuf,
    release: PathBuf,
    stderr: PathBuf,
    exit_code: PathBuf,
}

#[derive(Debug, Default)]
struct Launch {
    argv: Vec<String>,
    env: BTreeMap<String, String>,
    cwd: String,
    stdin: String,
}

impl Launch {
    fn has(&self, flag: &str) -> bool {
        self.argv.iter().any(|arg| arg == flag)
    }

    fn value_after(&self, flag: &str) -> Option<&str> {
        let index = self.argv.iter().position(|arg| arg == flag)?;
        self.argv.get(index + 1).map(String::as_str)
    }
}

impl StubCodex {
    fn new(dir: &Path, version: &str, exec_help: &str, resume_help: &str, auth_line: &str) -> Self {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join("codex-stub");
        let record = dir.join("codex-launches.txt");
        let replay = dir.join("codex-replay.jsonl");
        let hang = dir.join("codex-hang");
        let stall = dir.join("codex-stall");
        let release = dir.join("codex-release");
        let stderr = dir.join("codex-stderr");
        let exit_code = dir.join("codex-exit-code");
        let script = format!(
            "#!/bin/sh\n\
             if [ \"$1\" = \"--version\" ]; then echo \"{version}\"; exit 0; fi\n\
             if [ \"$1\" = \"login\" ] && [ \"$2\" = \"status\" ]; then echo \"{auth_line}\"; exit 0; fi\n\
             if [ \"$1\" = \"exec\" ] && [ \"$2\" = \"--help\" ]; then printf '%s\\n' \"{exec_help}\"; exit 0; fi\n\
             if [ \"$1\" = \"exec\" ] && [ \"$2\" = \"resume\" ] && [ \"$3\" = \"--help\" ]; then printf '%s\\n' \"{resume_help}\"; exit 0; fi\n\
             {{ for arg in \"$@\"; do printf 'arg %s\\n' \"$arg\"; done;\n\
             printf 'env CODEX_HOME=%s\\n' \"${{CODEX_HOME:-<unset>}}\";\n\
             printf 'cwd %s\\n' \"$(pwd)\";\n\
             printf 'stdin %s\\n' \"$(cat | tr '\\n' '|')\";\n\
             printf 'end\\n'; }} >> \"{record}\"\n\
             if [ -f \"{stall}\" ]; then\n  \
               i=0\n  \
               while [ ! -f \"{release}\" ] && [ \"$i\" -lt 200 ]; do sleep 0.1; i=$((i+1)); done\n  \
               rm -f \"{release}\"\n\
             fi\n\
             if [ -f \"{replay}\" ]; then cat \"{replay}\"; fi\n\
             if [ -f \"{stderr}\" ]; then cat \"{stderr}\" >&2; fi\n\
             if [ -f \"{hang}\" ]; then exec sleep 30; fi\n\
             if [ -f \"{exit_code}\" ]; then exit \"$(cat \"{exit_code}\")\"; fi\n\
             exit 0\n",
            version = version,
            auth_line = auth_line,
            exec_help = exec_help,
            resume_help = resume_help,
            record = record.display(),
            stall = stall.display(),
            release = release.display(),
            replay = replay.display(),
            stderr = stderr.display(),
            hang = hang.display(),
            exit_code = exit_code.display(),
        );
        std::fs::write(&path, script).expect("write stub");
        let mut permissions = std::fs::metadata(&path).expect("stat stub").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&path, permissions).expect("chmod stub");
        support::wait_until_executable(&path);
        Self {
            path,
            record,
            replay,
            hang,
            stall,
            release,
            stderr,
            exit_code,
        }
    }

    /// A stub that passes every probe gate.
    fn passing(dir: &Path) -> Self {
        Self::new(
            dir,
            PASSING_VERSION,
            ALL_EXEC_HELP,
            ALL_RESUME_HELP,
            LOGGED_IN,
        )
    }

    fn replays(&self, transcript: &str) -> &Self {
        std::fs::write(&self.replay, transcript).expect("write replay");
        self
    }

    fn hangs_after_replay(&self) -> &Self {
        std::fs::write(&self.hang, b"hang\n").expect("write hang marker");
        self
    }

    fn stalls_until_released(&self) -> &Self {
        std::fs::write(&self.stall, b"gate\n").expect("write stall marker");
        self
    }

    fn release_turn(&self) {
        std::fs::write(&self.release, b"go\n").expect("write release marker");
    }

    fn writes_stderr(&self, text: &str) -> &Self {
        std::fs::write(&self.stderr, text).expect("write stderr fixture");
        self
    }

    fn exits_with(&self, code: i32) -> &Self {
        std::fs::write(&self.exit_code, code.to_string()).expect("write exit code");
        self
    }

    fn launches(&self) -> Vec<Launch> {
        let Ok(text) = std::fs::read_to_string(&self.record) else {
            return Vec::new();
        };
        let mut launches = Vec::new();
        let mut current = Launch::default();
        for line in text.lines() {
            match line.split_once(' ') {
                Some(("arg", value)) => current.argv.push(value.to_string()),
                Some(("env", value)) => {
                    if let Some((name, value)) = value.split_once('=') {
                        current.env.insert(name.to_string(), value.to_string());
                    }
                }
                Some(("cwd", value)) => current.cwd = value.to_string(),
                Some(("stdin", value)) => current.stdin = value.to_string(),
                _ if line == "end" => launches.push(std::mem::take(&mut current)),
                _ => {}
            }
        }
        launches
    }

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

/// Recorded fixtures, loaded verbatim (spec §7.1 — nothing here is authored).
const AGENT_MESSAGE_TURN: &str = include_str!("fixtures/codex-0.149.0-agent-message-turn.jsonl");
const COMMAND_EXECUTION_TURN: &str =
    include_str!("fixtures/codex-0.149.0-command-execution-turn.jsonl");
const TURN_FAILED: &str = include_str!("fixtures/codex-0.149.0-turn-failed.jsonl");
const NARRATION_TURN: &str =
    include_str!("fixtures/codex-0.149.0-uncorroborated-narration-turn.jsonl");

/// A `thread.started` line naming `thread_id`, for building a synthetic
/// replay transcript in a test that needs a chosen id rather than the
/// fixture's recorded one.
fn plain_turn_naming(thread_id: &str) -> String {
    format!(
        "{{\"type\":\"thread.started\",\"thread_id\":\"{thread_id}\"}}\n\
         {{\"type\":\"turn.started\"}}\n\
         {{\"type\":\"item.completed\",\"item\":{{\"id\":\"item_0\",\"type\":\"agent_message\",\"text\":\"ok\"}}}}\n\
         {{\"type\":\"turn.completed\",\"usage\":{{\"input_tokens\":1,\"cached_input_tokens\":0,\
         \"cache_write_input_tokens\":0,\"output_tokens\":1,\"reasoning_output_tokens\":0}}}}\n"
    )
}

fn start_request(cwd: &Path) -> StartRequest {
    StartRequest {
        work_id: "w-codex".to_string(),
        execution_id: format!("exec-{}", ulid::Ulid::generate()),
        stage_id: "s1".to_string(),
        attempt: 1,
        cwd: cwd.to_path_buf(),
        intent: "do the codex thing".to_string(),
        context: "context body".to_string(),
        model: None,
        profile: None,
        execute: None,
        instruction_policy: InstructionPolicy::default(),
        bindings: Vec::<BindingSummary>::new(),
    }
}

/// A `CodexConfig` pointing at `stub`, with its own scratch `CODEX_HOME` (so
/// no test ever touches an operator's real `~/.codex`).
fn config_for(stub: &StubCodex, data_dir: &Path, codex_home: &Path) -> CodexConfig {
    let mut config = CodexConfig::new(data_dir);
    config.executable = stub.path.clone();
    config.codex_home = Some(codex_home.to_path_buf());
    config
}

/// Collects every [`EventDraft`] a backend emits, for assertions on the
/// normalized event stream.
fn sink() -> (sergeant_rs::backend::EventSink, Arc<Mutex<Vec<EventDraft>>>) {
    let events: Arc<Mutex<Vec<EventDraft>>> = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&events);
    let sink: sergeant_rs::backend::EventSink = Arc::new(move |draft: EventDraft| {
        captured.lock().expect("event capture lock").push(draft);
    });
    (sink, events)
}

fn wait_for_kind(events: &Arc<Mutex<Vec<EventDraft>>>, kind: &str) -> EventDraft {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(found) = events
            .lock()
            .expect("lock")
            .iter()
            .find(|e| e.kind == kind)
            .cloned()
        {
            return found;
        }
        assert!(Instant::now() < deadline, "no {kind} event arrived");
        std::thread::sleep(Duration::from_millis(20));
    }
}

// -------------------------------------------------------------- §2 probe

#[test]
fn the_probe_reports_measured_provenance_at_the_floor() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let report: ProbeReport = backend.probe();
    assert!(report.available);
    let detail = report.detail.expect("detail");
    assert!(detail.contains("codex-cli 0.149.0"));
    assert!(
        !detail.contains("BELOW"),
        "at the floor is measured, not below it: {detail}"
    );
    assert!(detail.contains("auth: logged in using ChatGPT"));
}

#[test]
fn the_probe_reports_unmeasured_provenance_below_the_floor_and_stays_available() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::new(
        dir.path(),
        "codex-cli 0.148.2",
        ALL_EXEC_HELP,
        ALL_RESUME_HELP,
        LOGGED_IN,
    );
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let report = backend.probe();
    assert!(
        report.available,
        "R1: below-floor is available, never refused"
    );
    let detail = report.detail.expect("detail");
    assert!(detail.contains("BELOW the measured floor"));
    assert!(detail.to_lowercase().contains("unmeasured"));
}

#[test]
fn the_probe_refuses_an_unparseable_version() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::new(
        dir.path(),
        "codex-cli nightly",
        ALL_EXEC_HELP,
        ALL_RESUME_HELP,
        LOGGED_IN,
    );
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let report = backend.probe();
    assert!(
        !report.available,
        "A2 condition 3: an unparseable version stays a refusal"
    );
    assert!(
        report
            .detail
            .expect("detail")
            .contains("cannot parse a version")
    );
}

#[test]
fn the_probe_refuses_a_missing_exec_flag_naming_it() {
    let dir = TempDir::new().expect("tempdir");
    let broken_help = "Usage: codex exec\nCommands: resume\nOptions: --model --cd --skip-git-repo-check --profile --sandbox --ephemeral";
    let stub = StubCodex::new(
        dir.path(),
        PASSING_VERSION,
        broken_help,
        ALL_RESUME_HELP,
        LOGGED_IN,
    );
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let report = backend.probe();
    assert!(!report.available);
    let detail = report.detail.expect("detail");
    assert!(
        detail.contains("--json"),
        "must name the missing flag: {detail}"
    );
}

#[test]
fn the_probe_refuses_a_missing_resume_flag_naming_it() {
    let dir = TempDir::new().expect("tempdir");
    let broken_resume_help =
        "Usage: codex exec resume\nOptions: --json --skip-git-repo-check --ephemeral";
    let stub = StubCodex::new(
        dir.path(),
        PASSING_VERSION,
        ALL_EXEC_HELP,
        broken_resume_help,
        LOGGED_IN,
    );
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let report = backend.probe();
    assert!(!report.available);
    let detail = report.detail.expect("detail");
    assert!(
        detail.contains("--model"),
        "must name the missing resume flag: {detail}"
    );
}

#[test]
fn the_probe_refuses_a_missing_resume_subcommand() {
    let dir = TempDir::new().expect("tempdir");
    let no_resume_listed = "Usage: codex exec\nCommands: fork | review | help\nOptions: --json --model --cd --skip-git-repo-check --profile --sandbox --ephemeral";
    let stub = StubCodex::new(
        dir.path(),
        PASSING_VERSION,
        no_resume_listed,
        ALL_RESUME_HELP,
        LOGGED_IN,
    );
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let report = backend.probe();
    assert!(!report.available);
    assert!(report.detail.expect("detail").contains("subcommand"));
}

/// Re-measured while wiring the live suite: `codex login status` writes its
/// answer to **stderr**, not stdout, when not attached to a TTY — never true
/// for a spawned child. `run_auth_probe` must fall back to stderr.
#[test]
fn the_probe_reads_auth_from_stderr_when_stdout_is_empty() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("codex-stub-stderr-auth");
    let script = format!(
        "#!/bin/sh\n\
         if [ \"$1\" = \"--version\" ]; then echo \"{PASSING_VERSION}\"; exit 0; fi\n\
         if [ \"$1\" = \"login\" ] && [ \"$2\" = \"status\" ]; then echo \"{LOGGED_IN}\" >&2; exit 0; fi\n\
         if [ \"$1\" = \"exec\" ] && [ \"$2\" = \"--help\" ]; then printf '%s\\n' \"{ALL_EXEC_HELP}\"; exit 0; fi\n\
         if [ \"$1\" = \"exec\" ] && [ \"$2\" = \"resume\" ] && [ \"$3\" = \"--help\" ]; then printf '%s\\n' \"{ALL_RESUME_HELP}\"; exit 0; fi\n\
         exit 0\n"
    );
    std::fs::write(&path, script).expect("write stub");
    let mut permissions = std::fs::metadata(&path).expect("stat").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).expect("chmod");
    support::wait_until_executable(&path);

    let mut config = CodexConfig::new(dir.path());
    config.executable = path;
    config.codex_home = Some(dir.path().join("codex-home"));
    let backend = CodexBackend::new(config);
    let report = backend.probe();
    assert!(report.available);
    assert!(
        report
            .detail
            .expect("detail")
            .contains("auth: logged in using ChatGPT"),
        "the probe must recover the auth line from stderr"
    );
}

// --------------------------------------------------------------- §3 launch

#[test]
fn launch_binds_the_thread_id_from_thread_started() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.replays(AGENT_MESSAGE_TURN);
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    assert!(
        prepared.native_id.is_none(),
        "codex cannot pre-mint a thread id"
    );
    let handle = backend.launch(&prepared).expect("launch");
    assert_eq!(
        handle.native_id.as_deref(),
        Some("01a02508-5880-7980-95b7-1d8bc22d5139"),
        "the handle's native id is the thread.started id, not a sergeant-minted one"
    );
}

#[test]
fn launch_fails_closed_when_the_process_dies_before_thread_started() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    // No replay: the stub emits nothing and exits 0, exactly the measured
    // shape of the untrusted-dir refusal (empty stdout JSON).
    stub.writes_stderr(
        "Not inside a trusted directory and --skip-git-repo-check was not specified.\n",
    );
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let err = backend.launch(&prepared).expect_err("must fail closed");
    match err {
        BackendError::Failed { detail, .. } => {
            assert!(detail.contains("before thread.started arrived"), "{detail}");
            assert!(
                detail.contains("trusted directory"),
                "stderr evidence must be in the detail: {detail}"
            );
        }
        other => panic!("expected Failed, got {other:?}"),
    }
    assert!(
        backend.tracked_executions().is_empty(),
        "a failed launch must leave no phantom execution"
    );
}

#[test]
fn launch_fails_closed_when_thread_started_never_arrives() {
    // SAFETY: this test's own env var, restored below; no other test reads it.
    unsafe { std::env::set_var("SGT_CODEX_TEST_THREAD_ID_BUDGET_MS", "300") };
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.stalls_until_released(); // parks before emitting anything, never released
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let err = backend.launch(&prepared).expect_err("budget must expire");
    match err {
        BackendError::Failed { detail, .. } => {
            assert!(
                detail.contains("did not announce thread.started"),
                "{detail}"
            );
            assert!(detail.contains("process group was killed"), "{detail}");
        }
        other => panic!("expected Failed, got {other:?}"),
    }
    assert!(backend.tracked_executions().is_empty());
    unsafe { std::env::remove_var("SGT_CODEX_TEST_THREAD_ID_BUDGET_MS") };
}

/// The budget's own complement: a turn that parks before emitting anything
/// and is then released *within* the budget must still succeed — LAUNCH
/// waits on `thread.started`, never on the whole turn finishing (issue #46's
/// shape on Claude: a turn whose process outlives launch-settle and then
/// finishes on its own).
#[test]
fn launch_succeeds_once_a_parked_turn_is_released_within_budget() {
    unsafe { std::env::set_var("SGT_CODEX_TEST_THREAD_ID_BUDGET_MS", "5000") };
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.replays(AGENT_MESSAGE_TURN).stalls_until_released();
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");

    std::thread::scope(|scope| {
        scope.spawn(|| {
            std::thread::sleep(Duration::from_millis(200));
            stub.release_turn();
        });
        let handle = backend
            .launch(&prepared)
            .expect("launch must succeed once released in time");
        assert!(handle.native_id.is_some());
    });
    unsafe { std::env::remove_var("SGT_CODEX_TEST_THREAD_ID_BUDGET_MS") };
}

#[test]
fn the_first_turn_launches_the_recorded_grammar() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.replays(AGENT_MESSAGE_TURN);
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let mut request = start_request(dir.path());
    request.model = Some("gpt-5.6-luna".to_string());
    let prepared = backend.prepare(&request).expect("prepare");
    backend.launch(&prepared).expect("launch");
    let launches = stub.wait_for_launches(1);
    let launch = &launches[0];
    assert_eq!(launch.argv[0], "exec");
    assert!(launch.has("--json"));
    assert!(launch.has("--skip-git-repo-check"));
    assert_eq!(launch.value_after("-C"), Some(dir.path().to_str().unwrap()));
    assert_eq!(launch.value_after("-m"), Some("gpt-5.6-luna"));
    assert!(!launch.has("--add-dir"));
    assert!(!launch.has("-p"));
    assert!(!launch.has("--ephemeral"));
    assert_eq!(launch.cwd, dir.path().to_str().unwrap());
    assert!(launch.stdin.contains("do the codex thing"));
    assert!(launch.stdin.contains("context body"));
}

#[test]
fn the_resume_turn_launches_the_recorded_grammar() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.replays(AGENT_MESSAGE_TURN);
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    let thread_id = handle.native_id.clone().unwrap();
    // LAUNCH only waits for thread.started, not for turn 1 to finish — wait
    // for turn 1 to settle before sending turn 2, or SEND correctly refuses
    // a second concurrent turn (that refusal has its own test).
    wait_for_settled(&backend, &handle);

    // Turn 2 must resume the *same* thread, or the reader's own
    // thread.started-mismatch check would fail it — so the stub must keep
    // announcing the same id on every subsequent turn.
    stub.replays(&plain_turn_naming(&thread_id));
    backend.send(&handle, "turn two").expect("send");
    let launches = stub.wait_for_launches(2);
    let resume = &launches[1];
    assert_eq!(resume.argv[0], "exec");
    assert_eq!(resume.argv[1], "resume");
    assert_eq!(
        resume.argv[2], thread_id,
        "thread id sits immediately after `resume`"
    );
    assert!(!resume.has("-C"), "exec resume has no -C on this build");
    assert!(!resume.has("--cd"));
    assert!(resume.stdin.contains("turn two"));
}

#[test]
fn the_raw_stream_is_archived_before_any_conclusion_is_drawn() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.replays(AGENT_MESSAGE_TURN);
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    let raw_ref = ended.payload["raw"].as_str().expect("a raw blob ref");
    assert!(!raw_ref.is_empty());

    let observation = backend.observe(&handle).expect("observe");
    assert_eq!(observation.native, NativeState::Exited);
    assert!(observation.evidence.expect("evidence").contains(raw_ref));
}

#[test]
fn a_second_concurrent_turn_is_refused() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.replays(AGENT_MESSAGE_TURN).hangs_after_replay();
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    let err = backend
        .send(&handle, "second turn")
        .expect_err("must be refused");
    match err {
        BackendError::Failed { detail, .. } => {
            assert!(detail.contains("one turn at a time"), "{detail}");
        }
        other => panic!("expected Failed, got {other:?}"),
    }
    backend.stop(&handle).expect("stop").wait();
}

#[test]
fn stop_waits_for_the_turns_evidence_outside_the_lock() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.replays(AGENT_MESSAGE_TURN).hangs_after_replay();
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    // STOP's Completion, once waited, means the reader has already recorded
    // the outcome — so OBSERVE right after must never see InFlight.
    backend.stop(&handle).expect("stop").wait();
    let observation = backend.observe(&handle).expect("observe");
    assert_ne!(
        observation.evidence.as_deref().unwrap_or(""),
        "",
        "some evidence must already be recorded"
    );
}

#[test]
fn codex_history_refuses_and_the_refusal_names_the_records() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.replays(AGENT_MESSAGE_TURN);
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    let err = backend
        .history(&handle)
        .expect_err("history is unsupported");
    match err {
        BackendError::Unsupported { verb, detail, .. } => {
            assert_eq!(verb, "history");
            assert!(detail.contains("rollout"));
        }
        other => panic!("expected Unsupported, got {other:?}"),
    }
}

#[test]
fn history_checks_identity_before_capability() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let bogus = ExecutionHandle {
        execution_id: "no-such-execution".to_string(),
        native_id: Some("t1".to_string()),
    };
    let err = backend.history(&bogus).expect_err("unknown execution");
    assert!(matches!(err, BackendError::UnknownExecution { .. }));
}

#[test]
fn a_completed_turn_observes_stage_completed_with_the_last_message() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.replays(COMMAND_EXECUTION_TURN);
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    let _ended = wait_for_kind(&events, "conversation.turn.ended");
    let observation = backend.observe(&handle).expect("observe");
    assert_eq!(observation.native, NativeState::Exited);
    match observation.signal {
        sergeant_rs::backend::BackendSignal::StageCompleted { summary } => {
            assert_eq!(summary.as_deref(), Some("unsandboxed-ok"));
        }
        other => panic!("expected StageCompleted, got {other:?}"),
    }
    let kinds: Vec<String> = events
        .lock()
        .unwrap()
        .iter()
        .map(|e| e.kind.clone())
        .collect();
    assert!(kinds.iter().any(|k| k == "tool.requested"));
    assert!(kinds.iter().any(|k| k == "tool.completed"));
}

#[test]
fn a_bad_model_pin_observes_as_failed_with_the_api_message() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.replays(TURN_FAILED).exits_with(1);
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let mut request = start_request(dir.path());
    request.model = Some("gpt-5.6-nonexistent-model".to_string());
    let prepared = backend.prepare(&request).expect("prepare");
    let handle = backend
        .launch(&prepared)
        .expect("launch, since thread.started still arrives first");
    let deadline = Instant::now() + Duration::from_secs(10);
    let observation = loop {
        let observation = backend.observe(&handle).expect("observe");
        if observation.native != NativeState::Running {
            break observation;
        }
        assert!(Instant::now() < deadline, "turn never settled");
        std::thread::sleep(Duration::from_millis(20));
    };
    match observation.signal {
        sergeant_rs::backend::BackendSignal::Failed { reason } => {
            assert!(reason.contains("gpt-5.6-nonexistent-model"), "{reason}");
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

#[test]
fn narration_never_produces_tool_events_through_the_full_backend() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.replays(NARRATION_TURN);
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    backend.launch(&prepared).expect("launch");
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    assert_eq!(ended.payload["tool_items"], 0);
    let kinds: Vec<String> = events
        .lock()
        .unwrap()
        .iter()
        .map(|e| e.kind.clone())
        .collect();
    assert!(
        !kinds
            .iter()
            .any(|k| k == "tool.requested" || k == "tool.completed")
    );
}

#[test]
fn resume_refuses_to_adopt_a_thread_with_no_rollout() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let handle = ExecutionHandle {
        execution_id: "e1".to_string(),
        native_id: Some(fresh_thread_id()),
    };
    let request = ResumeRequest::new("w1", dir.path());
    let err = backend
        .resume(&handle, &request)
        .expect_err("no rollout, no adoption");
    assert!(matches!(err, BackendError::UnknownExecution { .. }));
}

#[test]
fn resume_refuses_an_impossible_pin_before_the_idempotent_check() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.replays(AGENT_MESSAGE_TURN);
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    let mut request = ResumeRequest::new("w-codex", dir.path());
    request.model = Some("   ".to_string());
    let err = backend
        .resume(&handle, &request)
        .expect_err("empty pin refused pre-flight");
    match err {
        BackendError::Failed { detail, .. } => assert!(detail.contains("model pin is empty")),
        other => panic!("expected Failed, got {other:?}"),
    }
}

#[test]
fn launch_refuses_a_codex_native_profile_before_any_reservation() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let mut options = BTreeMap::new();
    options.insert("codex_profile".to_string(), "whatever".to_string());
    let profile = sergeant_rs::domain::profile::Profile {
        name: "p".to_string(),
        backend: CODEX_BACKEND_NAME.to_string(),
        executable: None,
        config_home: None,
        env: BTreeMap::new(),
        default_model: None,
        options,
    };
    let mut request = start_request(dir.path());
    request.profile = Some(profile);
    let err = backend
        .prepare(&request)
        .expect_err("refused at PREPARE, before any reservation");
    match err {
        BackendError::Failed { detail, .. } => assert!(detail.contains("codex_profile")),
        other => panic!("expected Failed, got {other:?}"),
    }
    assert!(backend.tracked_executions().is_empty());
}

#[test]
fn a_profile_config_home_sets_codex_home_on_every_turn() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.replays(AGENT_MESSAGE_TURN);
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("default-home"),
    ));
    let profile_home = dir.path().join("profile-home");
    std::fs::create_dir_all(&profile_home).unwrap();
    let profile = sergeant_rs::domain::profile::Profile {
        name: "p".to_string(),
        backend: CODEX_BACKEND_NAME.to_string(),
        executable: None,
        config_home: Some(profile_home.clone()),
        env: BTreeMap::new(),
        default_model: None,
        options: BTreeMap::new(),
    };
    let mut request = start_request(dir.path());
    request.profile = Some(profile);
    let prepared = backend.prepare(&request).expect("prepare");
    backend.launch(&prepared).expect("launch");
    let launches = stub.wait_for_launches(1);
    assert_eq!(
        launches[0].env.get("CODEX_HOME").map(String::as_str),
        Some(profile_home.to_str().unwrap())
    );
}

#[test]
fn codex_never_reports_an_actor_authored_question_end_to_end() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.replays(COMMAND_EXECUTION_TURN);
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    assert!(!backend.capabilities().ask);
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    backend.launch(&prepared).expect("launch");
    let _ = wait_for_kind(&events, "conversation.turn.ended");
    assert!(
        !events
            .lock()
            .unwrap()
            .iter()
            .any(|e| e.kind == "conversation.ask")
    );
}

// ---------------------------------------------------------- §7.3 live suite

/// Gate mirroring `tests/m4_backends.rs`'s `LiveGate`/`claude_live_enabled`
/// exactly (A3: the precedent is m4, not m10).
#[derive(Debug, PartialEq, Eq)]
enum LiveGate {
    Run,
    NotOptedIn,
    Unusable(String),
}

fn live_gate(opt_in: Option<&str>, probe: &ProbeReport, auth_ok: Result<bool, String>) -> LiveGate {
    if opt_in != Some("1") {
        return LiveGate::NotOptedIn;
    }
    if !probe.available {
        return LiveGate::Unusable(format!(
            "the installed codex does not pass the adapter's probe: {}",
            probe.detail.clone().unwrap_or_default()
        ));
    }
    match auth_ok {
        Ok(true) => LiveGate::Run,
        Ok(false) => LiveGate::Unusable(
            "the installed codex reports no logged-in account (`codex login status`); these \
             tests need a real conversation"
                .to_string(),
        ),
        Err(why) => LiveGate::Unusable(format!(
            "cannot establish that the installed codex is authenticated: {why}"
        )),
    }
}

fn codex_auth_ok(executable: &Path) -> Result<bool, String> {
    let output = std::process::Command::new(executable)
        .args(["login", "status"])
        .output()
        .map_err(|e| format!("cannot run {executable:?} login status: {e}"))?;
    // Measured: `codex login status` writes its answer to stderr, not
    // stdout, when not attached to a TTY (never true for a spawned child) —
    // src/backend/codex.rs's `run_auth_probe` has the same fallback.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let text = if stdout.trim().is_empty() {
        String::from_utf8_lossy(&output.stderr).into_owned()
    } else {
        stdout.into_owned()
    };
    Ok(text.trim_start().starts_with("Logged in using"))
}

/// Live config: the system `codex` (or `SGT_CODEX_BIN`), a scratch data dir
/// and a scratch `CODEX_HOME` under `/var/tmp` (never `/tmp`: a quota'd
/// tmpfs on this host — spec §7.3).
fn live_config(data_dir: &Path) -> CodexConfig {
    // Deliberately NOT overriding codex_home: spec §7.3 accepts the trust-entry
    // side effect (§3.8) in the operator's real ~/.codex (or $CODEX_HOME) --
    // auth.json and the session rollout live there, and only `cwd` is a
    // scratch dir under /var/tmp.
    CodexConfig::new(data_dir)
}

/// Whether the opt-in live-codex tests may run. Reaching this with the
/// opt-in variable unset is a misuse of `-- --ignored` and panics, naming
/// the opt-in — the false green `#[ignore]` exists to prevent. An unusable
/// harness is a clean skip, written straight to fd 2 (libtest only captures
/// the print macros).
fn codex_live_enabled(test: &str, data_dir: &Path) -> bool {
    let config = live_config(data_dir);
    let probe = CodexBackend::new(config.clone()).probe();
    let gate = live_gate(
        std::env::var("SERGEANT_CODEX_TESTS").ok().as_deref(),
        &probe,
        codex_auth_ok(&config.executable),
    );
    match gate {
        LiveGate::Run => true,
        LiveGate::NotOptedIn => panic!(
            "{test} is opt-in and spends real tokens: run it with SERGEANT_CODEX_TESTS=1 cargo \
             test --test codex_backend -- --ignored. (Without the variable these tests are \
             skipped by #[ignore]; asking for --ignored without it must not report a green test \
             that did nothing.)"
        ),
        LiveGate::Unusable(why) => {
            let _ = std::io::stderr()
                .write_all(format!("SKIPPED {test}: {why}\n").as_bytes())
                .and_then(|()| std::io::stderr().flush());
            false
        }
    }
}

fn live_workdir(name: &str) -> TempDir {
    tempfile::Builder::new()
        .prefix(&format!("codex-live-{name}-"))
        .tempdir_in("/var/tmp")
        .expect("scratch dir under /var/tmp, never /tmp (quota'd tmpfs on this host)")
}

fn wait_for_settled(
    backend: &CodexBackend,
    handle: &ExecutionHandle,
) -> sergeant_rs::backend::Observation {
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        let observation = backend.observe(handle).expect("observe");
        if observation.native != NativeState::Running {
            return observation;
        }
        assert!(Instant::now() < deadline, "live turn never settled");
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[test]
#[ignore = "opt-in, spends real tokens: SERGEANT_CODEX_TESTS=1 cargo test --test codex_backend -- --ignored"]
fn live_codex_probe_reports_the_installed_version_and_auth() {
    let data_dir = live_workdir("probe");
    if !codex_live_enabled(
        "live_codex_probe_reports_the_installed_version_and_auth",
        data_dir.path(),
    ) {
        return;
    }
    let backend = CodexBackend::new(live_config(data_dir.path()));
    let report = backend.probe();
    assert!(report.available);
    let detail = report.detail.expect("detail");
    assert!(detail.contains("codex-cli"));
    assert!(detail.contains("auth:"));
}

#[test]
#[ignore = "opt-in, spends real tokens: SERGEANT_CODEX_TESTS=1 cargo test --test codex_backend -- --ignored"]
fn live_codex_turn_streams_events_before_it_ends() {
    let data_dir = live_workdir("stream");
    if !codex_live_enabled(
        "live_codex_turn_streams_events_before_it_ends",
        data_dir.path(),
    ) {
        return;
    }
    let backend = CodexBackend::new(live_config(data_dir.path()));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let mut request = start_request(data_dir.path());
    request.model = Some("gpt-5.6-luna".to_string());
    request.intent = "Reply with exactly the word ok and nothing else.".to_string();
    request.context = String::new();
    let prepared = backend.prepare(&request).expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    // A user event lands before the turn settles — proof of incremental
    // delivery, never asserted on prose content.
    let _user = wait_for_kind(&events, "conversation.user");
    let observation = wait_for_settled(&backend, &handle);
    assert_eq!(observation.native, NativeState::Exited);
    backend.stop(&handle).expect("stop").wait();
}

#[test]
#[ignore = "opt-in, spends real tokens: SERGEANT_CODEX_TESTS=1 cargo test --test codex_backend -- --ignored"]
fn live_codex_turn_reports_usage() {
    let data_dir = live_workdir("usage");
    if !codex_live_enabled("live_codex_turn_reports_usage", data_dir.path()) {
        return;
    }
    let backend = CodexBackend::new(live_config(data_dir.path()));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let mut request = start_request(data_dir.path());
    request.model = Some("gpt-5.6-luna".to_string());
    request.intent = "Reply with exactly the word ok and nothing else.".to_string();
    request.context = String::new();
    let prepared = backend.prepare(&request).expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    let usage_event = wait_for_kind(&events, "usage.updated");
    let usage: &Value = &usage_event.payload["usage"];
    for field in [
        "input_tokens",
        "cached_input_tokens",
        "cache_write_input_tokens",
        "output_tokens",
        "reasoning_output_tokens",
    ] {
        assert!(usage.get(field).is_some(), "usage missing {field}: {usage}");
    }
    assert_eq!(usage_event.payload["model_pin"]["verdict"], "attempted");
    backend.stop(&handle).expect("stop").wait();
}

#[test]
#[ignore = "opt-in, spends real tokens: SERGEANT_CODEX_TESTS=1 cargo test --test codex_backend -- --ignored"]
fn live_codex_bad_model_pin_fails_loud_not_silent() {
    let data_dir = live_workdir("badpin");
    if !codex_live_enabled(
        "live_codex_bad_model_pin_fails_loud_not_silent",
        data_dir.path(),
    ) {
        return;
    }
    let backend = CodexBackend::new(live_config(data_dir.path()));
    let mut request = start_request(data_dir.path());
    request.model = Some("gpt-5.6-nonexistent-model".to_string());
    request.intent = "Reply with exactly the word ok and nothing else.".to_string();
    request.context = String::new();
    let prepared = backend.prepare(&request).expect("prepare");
    let handle = backend
        .launch(&prepared)
        .expect("launch (thread.started still arrives)");
    let observation = wait_for_settled(&backend, &handle);
    match observation.signal {
        sergeant_rs::backend::BackendSignal::Failed { reason } => {
            assert!(
                reason.to_lowercase().contains("model") || reason.contains("400"),
                "{reason}"
            );
        }
        other => panic!("expected Failed on a bad model pin, got {other:?}"),
    }
}

#[test]
#[ignore = "opt-in, spends real tokens: SERGEANT_CODEX_TESTS=1 cargo test --test codex_backend -- --ignored"]
fn live_codex_resume_recalls_a_nonce_across_processes() {
    let data_dir = live_workdir("resume");
    if !codex_live_enabled(
        "live_codex_resume_recalls_a_nonce_across_processes",
        data_dir.path(),
    ) {
        return;
    }
    let nonce = format!("nonce-{}", ulid::Ulid::generate());
    let backend = CodexBackend::new(live_config(data_dir.path()));
    let mut request = start_request(data_dir.path());
    request.model = Some("gpt-5.6-luna".to_string());
    request.intent = format!("Remember the word {nonce}. Reply ok.");
    request.context = String::new();
    let prepared = backend.prepare(&request).expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    let _ = wait_for_settled(&backend, &handle);

    // The sink must be installed *before* the send whose events matter:
    // `spawn_turn` snapshots the sink at spawn time, so installing one
    // after `send()` returns would miss this turn's events entirely.
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    backend
        .send(
            &handle,
            "What word did I ask you to remember? Reply with just that word.",
        )
        .expect("send");
    let _ = wait_for_settled(&backend, &handle);
    let assistant = wait_for_kind(&events, "conversation.assistant.completed");
    let text = assistant.payload["text"].as_str().unwrap_or("");
    assert!(
        text.contains(&nonce),
        "expected the recalled nonce in {text:?}"
    );
    backend.stop(&handle).expect("stop").wait();
}

/// Retry SEND across the brief window between INTERRUPT killing a process
/// and the reader thread finishing the turn (INTERRUPT's promise is only
/// that the process is signaled — the turn's own evidence is STOP's promise,
/// per the trait doc — so a SEND issued the instant after INTERRUPT returns
/// can legitimately still see `InFlight`).
fn send_retrying(backend: &CodexBackend, handle: &ExecutionHandle, input: &str) {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        match backend.send(handle, input) {
            Ok(()) => return,
            Err(BackendError::Failed { detail, .. })
                if detail.contains("already has a turn in flight") =>
            {
                assert!(
                    Instant::now() < deadline,
                    "second send never became possible"
                );
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => panic!("send failed: {e:?}"),
        }
    }
}

#[test]
#[ignore = "opt-in, spends real tokens: SERGEANT_CODEX_TESTS=1 cargo test --test codex_backend -- --ignored"]
fn live_codex_interrupt_leaves_the_conversation_resumable() {
    let data_dir = live_workdir("interrupt");
    if !codex_live_enabled(
        "live_codex_interrupt_leaves_the_conversation_resumable",
        data_dir.path(),
    ) {
        return;
    }
    let nonce = format!("nonce-{}", ulid::Ulid::generate());
    let backend = CodexBackend::new(live_config(data_dir.path()));
    let mut request = start_request(data_dir.path());
    request.model = Some("gpt-5.6-luna".to_string());
    request.intent = format!("Remember the word {nonce}. Reply ok.");
    request.context = String::new();
    let prepared = backend.prepare(&request).expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    let _ = wait_for_settled(&backend, &handle);

    backend
        .send(
            &handle,
            "Count slowly from 1 to 50, one number per line, waiting a moment between each.",
        )
        .expect("send");
    std::thread::sleep(Duration::from_millis(500));
    backend.interrupt(&handle).expect("interrupt").wait();
    let observation = backend.observe(&handle).expect("observe after interrupt");
    assert_eq!(
        observation.signal,
        sergeant_rs::backend::BackendSignal::Running
    );

    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    send_retrying(
        &backend,
        &handle,
        "What word did I ask you to remember? Reply with just that word.",
    );
    let _ = wait_for_settled(&backend, &handle);
    let assistant = wait_for_kind(&events, "conversation.assistant.completed");
    let text = assistant.payload["text"].as_str().unwrap_or("");
    assert!(
        text.contains(&nonce),
        "conversation must still recall its nonce after an interrupt: {text:?}"
    );
    backend.stop(&handle).expect("stop").wait();
}

#[test]
#[ignore = "opt-in, spends real tokens: SERGEANT_CODEX_TESTS=1 cargo test --test codex_backend -- --ignored"]
fn live_codex_thread_survives_turns_and_a_restart() {
    let data_dir = live_workdir("restart");
    if !codex_live_enabled(
        "live_codex_thread_survives_turns_and_a_restart",
        data_dir.path(),
    ) {
        return;
    }
    let nonce = format!("nonce-{}", ulid::Ulid::generate());
    let backend = CodexBackend::new(live_config(data_dir.path()));
    let mut request = start_request(data_dir.path());
    request.model = Some("gpt-5.6-luna".to_string());
    request.intent = format!("Remember the word {nonce}. Reply ok.");
    request.context = String::new();
    let prepared = backend.prepare(&request).expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    let _ = wait_for_settled(&backend, &handle);
    let thread_id = handle.native_id.clone().expect("thread id");

    // Drop the adapter (simulating a daemon restart) and build a fresh one.
    drop(backend);
    let fresh = CodexBackend::new(live_config(data_dir.path()));
    let resume_request = ResumeRequest::new("w-codex", data_dir.path());
    fresh
        .resume(&handle, &resume_request)
        .expect("resume after restart");
    let (sink_fn, events) = sink();
    fresh.set_event_sink(sink_fn);
    fresh
        .send(
            &handle,
            "What word did I ask you to remember? Reply with just that word.",
        )
        .expect("send after resume");
    let _ = wait_for_settled(&fresh, &handle);
    let assistant = wait_for_kind(&events, "conversation.assistant.completed");
    let text = assistant.payload["text"].as_str().unwrap_or("");
    assert!(
        text.contains(&nonce),
        "expected the recalled nonce in {text:?}"
    );
    assert_eq!(handle.native_id.as_deref(), Some(thread_id.as_str()));
    fresh.stop(&handle).expect("stop").wait();
}
