//! W1 contract tests for `src/backend/opencode.rs`.
//!
//! Two tiers, the split the sprint plan carries over from the codex sprint's
//! A3 pattern:
//!
//! - **Stub-driven** (this file's bulk): [`StubOpencode`] is a shell script
//!   modelled directly on `tests/codex_backend.rs`'s `StubCodex`. It answers
//!   the capability probe (`--version`, `run --help`, `--help` — the last two
//!   on **stderr**, which is where the real CLI measurably puts them),
//!   records every argv/env/cwd/stdin it is launched with, replays a recorded
//!   fixture on stdout, serves `export <sessionID>` out of a table of
//!   recorded documents, and can stall / hang / write stderr / exit non-zero
//!   on demand. No `opencode` binary is required for any test in this tier.
//! - **Live** (gated): every test is `#[ignore]`d *and* gated on
//!   `SERGEANT_OPENCODE_TESTS=1` plus an available probe, and every live turn
//!   pins `-m opencode/big-pickle` (owner ruling K1: the free Zen tier, zero
//!   cost) with a bounded, one-word-answer prompt.
//!
//! No test here spawns a sergeant daemon, so `tests/support::DataDir`'s
//! reaper does not apply (the same reason `tests/codex_backend.rs`'s
//! stub-driven tests construct their backend directly); only
//! `support::wait_until_executable` is borrowed, for the ETXTBSY window a
//! freshly-chmod'ed stub has.
//!
//! Two daemon-level tests (W2) prove registration itself — that
//! `daemon::start_with` puts an "opencode" entry in the registry with no
//! test-supplied stand-in — via in-process `daemon::start_with` +
//! `handle.shutdown().await`, the same rig `tests/m3_execution.rs`/
//! `tests/estate_routes.rs` use, so `tests/support::DataDir`'s reaper does
//! not apply to them either (no subprocess is spawned).
//!
//! Every fixture under `tests/fixtures/opencode-1.18.19-*` is a **real
//! capture** at opencode 1.18.19 — nothing in this file authors an event
//! stream. Where a test needs a shape no probe recorded (an export document
//! for a session the stub minted), it reuses a recorded document rather than
//! inventing one.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;

use sergeant_rs::backend::opencode::{
    OPENCODE_BACKEND_NAME, OPENCODE_CONFIG_CONTENT_ENV, OpencodeBackend, OpencodeConfig,
};
use sergeant_rs::backend::{
    Backend, BackendError, BackendSignal, BindingSummary, ExecutionHandle, NativeState,
    ProbeReport, ResumeRequest, StartRequest,
};
use sergeant_rs::daemon::{self, DaemonConfig};
use sergeant_rs::domain::estate::InstructionPolicy;
use sergeant_rs::domain::event::EventDraft;
use sergeant_rs::domain::profile::Profile;
use sergeant_rs::runtime::journal::Journal;

mod support;

// ------------------------------------------------------- recorded fixtures

const MINIMAL_TURN: &str = include_str!("fixtures/opencode-1.18.19-minimal-turn.jsonl");
const TOOL_USE_TURN: &str = include_str!("fixtures/opencode-1.18.19-tool-use.jsonl");
const AUTOREJECT_TURN: &str = include_str!("fixtures/opencode-1.18.19-permission-autoreject.jsonl");
const SIGKILL_TRUNCATED: &str = include_str!("fixtures/opencode-1.18.19-sigkill-truncated.jsonl");
const UNKNOWN_MODEL_ERROR: &str =
    include_str!("fixtures/opencode-1.18.19-error-unknown-model.jsonl");
const EXPORT_SESSION: &str = include_str!("fixtures/opencode-1.18.19-export-session.json");

/// The session id every event of `opencode-1.18.19-minimal-turn.jsonl`
/// carries — the id the adapter is expected to learn from that stream's first
/// line, and therefore the id a test has to register an export document under.
const MINIMAL_TURN_SESSION: &str = "ses_fd1bbfff3ffek3hxLwVl6yk5FW";
/// The same, for the two-step tool-use capture.
const TOOL_USE_SESSION: &str = "ses_fd1bb9528ffe0kzw9648HPy49e";
/// The same, for the SIGKILL-truncated capture.
const TRUNCATED_SESSION: &str = "ses_fd1a74442ffetD1z7lZiKnUTP5";

/// K1: every live turn in this file pins the free Zen model, and nothing in
/// `src/backend/opencode.rs` hardcodes a model — the pin travels on
/// `StartRequest::model` like any other.
const LIVE_MODEL: &str = "opencode/big-pickle";

// ------------------------------------------------------------ the stub CLI

/// `opencode run --help` text carrying every flag the probe requires. The
/// real CLI writes this to stderr (measured, yargs) and so does the stub.
const ALL_RUN_HELP: &str = "\
opencode run [message..]\\n\
Options: -s, --session  -m, --model  --format  --agent  --auto  --dir  -f, --file";

/// `opencode --help` text listing every subcommand the probe requires.
const ALL_TOP_HELP: &str = "\
Commands: opencode run [message..] | opencode export [sessionID] | opencode serve | opencode models";

const PASSING_VERSION: &str = "1.18.19";

/// A stub `opencode` that answers the capability probe, **records every
/// launch**, replays a recorded fixture on stdout, and serves `export` out of
/// a directory of recorded documents.
struct StubOpencode {
    path: PathBuf,
    record: PathBuf,
    replay: PathBuf,
    exports: PathBuf,
    stall: PathBuf,
    release: PathBuf,
    hang: PathBuf,
    stderr: PathBuf,
    exit_code: PathBuf,
    grandchild: PathBuf,
    grandchild_pid: PathBuf,
}

impl StubOpencode {
    fn new(dir: &Path, name: &str, version: &str, run_help: &str, top_help: &str) -> Self {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join(format!("{name}-stub"));
        let record = dir.join(format!("{name}-launches.txt"));
        let replay = dir.join(format!("{name}-replay.jsonl"));
        let exports = dir.join(format!("{name}-exports"));
        let stall = dir.join(format!("{name}-stall"));
        let release = dir.join(format!("{name}-release"));
        let hang = dir.join(format!("{name}-hang"));
        let stderr = dir.join(format!("{name}-stderr"));
        let exit_code = dir.join(format!("{name}-exit-code"));
        let grandchild = dir.join(format!("{name}-grandchild"));
        let grandchild_pid = dir.join(format!("{name}-grandchild-pid"));
        std::fs::create_dir_all(&exports).expect("exports dir");
        // The subcommand arms are ordered exactly as the real CLI resolves
        // them: `run --help` is help, not a run, so it must be recognised
        // before the generic `run` arm records a launch that never happened.
        let script = format!(
            "#!/bin/sh\n\
             if [ \"$1\" = \"--version\" ]; then echo \"{version}\"; exit 0; fi\n\
             if [ \"$1\" = \"run\" ] && [ \"$2\" = \"--help\" ]; then printf '%s\\n' \"{run_help}\" >&2; exit 0; fi\n\
             if [ \"$1\" = \"--help\" ]; then printf '%s\\n' \"{top_help}\" >&2; exit 0; fi\n\
             {{ for arg in \"$@\"; do printf 'arg %s\\n' \"$arg\"; done;\n\
             env | grep -E '^(OPENCODE_[A-Z_]*|SGT_[A-Z_]*)=' | sed 's/^/env /' | tr -d '\\r';\n\
             printf 'cwd %s\\n' \"$(pwd)\";\n\
             printf 'stdin %s\\n' \"$(cat | tr '\\n' '|')\";\n\
             printf 'end\\n'; }} >> \"{record}\"\n\
             if [ \"$1\" = \"export\" ]; then\n  \
               printf 'Exporting session: %s\\n' \"$2\" >&2\n  \
               if [ -f \"{exports}/$2.json\" ]; then cat \"{exports}/$2.json\"; exit 0; fi\n  \
               printf 'Error: Session not found: %s\\n' \"$2\" >&2\n  \
               exit 1\n\
             fi\n\
             if [ -f \"{stall}\" ]; then\n  \
               i=0\n  \
               while [ ! -f \"{release}\" ] && [ \"$i\" -lt 200 ]; do sleep 0.1; i=$((i+1)); done\n  \
               rm -f \"{release}\"\n\
             fi\n\
             if [ -f \"{replay}\" ]; then cat \"{replay}\"; fi\n\
             if [ -f \"{stderr}\" ]; then cat \"{stderr}\" >&2; fi\n\
             if [ -f \"{grandchild}\" ]; then\n  \
               ( i=0; while [ \"$i\" -lt 300 ]; do sleep 0.1; i=$((i+1)); done ) &\n  \
               echo $! > \"{grandchild_pid}\"\n\
             fi\n\
             if [ -f \"{hang}\" ]; then exec sleep 30; fi\n\
             if [ -f \"{exit_code}\" ]; then exit \"$(cat \"{exit_code}\")\"; fi\n\
             exit 0\n",
            version = version,
            run_help = run_help,
            top_help = top_help,
            exports = exports.display(),
            record = record.display(),
            stall = stall.display(),
            release = release.display(),
            replay = replay.display(),
            stderr = stderr.display(),
            grandchild = grandchild.display(),
            grandchild_pid = grandchild_pid.display(),
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
            exports,
            stall,
            release,
            hang,
            stderr,
            exit_code,
            grandchild,
            grandchild_pid,
        }
    }

    /// A stub that passes every probe gate.
    fn passing(dir: &Path) -> Self {
        Self::new(dir, "opencode", PASSING_VERSION, ALL_RUN_HELP, ALL_TOP_HELP)
    }

    fn replays(&self, transcript: &str) -> &Self {
        std::fs::write(&self.replay, transcript).expect("write replay");
        self
    }

    /// Register the document this stub's `export <session_id>` answers with.
    /// Anything else gets the measured not-found shape (exit 1, empty stdout,
    /// `Error: Session not found: <id>` on stderr).
    fn exports_session(&self, session_id: &str, document: &str) -> &Self {
        std::fs::write(self.exports.join(format!("{session_id}.json")), document)
            .expect("write export document");
        self
    }

    /// Block before replaying anything, until [`Self::release_turn`].
    fn stalls_until_released(&self) -> &Self {
        std::fs::write(&self.stall, b"gate\n").expect("write stall marker");
        self
    }

    fn release_turn(&self) {
        std::fs::write(&self.release, b"go\n").expect("write release marker");
    }

    /// Replay, then stay alive — the shape a streaming assertion needs: the
    /// events must be observable *while the process is still running*.
    fn hangs_after_replay(&self) -> &Self {
        std::fs::write(&self.hang, b"hang\n").expect("write hang marker");
        self
    }

    fn writes_stderr(&self, text: &str) -> &Self {
        std::fs::write(&self.stderr, text).expect("write stderr fixture");
        self
    }

    fn exits_with(&self, code: i32) -> &Self {
        std::fs::write(&self.exit_code, code.to_string()).expect("write exit code");
        self
    }

    /// Marks the stub to fork a background grandchild (a detached loop, not
    /// a `setsid`) right after replay, and record that grandchild's own
    /// pid — the process a single `child.kill()` would never reach, and the
    /// whole reason INTERRUPT kills the turn's process *group* instead
    /// (probe 11; mirrors `tests/codex_backend.rs::StubCodex::
    /// spawns_a_grandchild`).
    fn spawns_a_grandchild(&self) -> &Self {
        std::fs::write(&self.grandchild, b"go\n").expect("write grandchild marker");
        self
    }

    fn grandchild_pid(&self) -> Option<u32> {
        std::fs::read_to_string(&self.grandchild_pid)
            .ok()
            .and_then(|text| text.trim().parse().ok())
    }

    fn wait_for_grandchild_pid(&self) -> u32 {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(pid) = self.grandchild_pid() {
                return pid;
            }
            assert!(
                Instant::now() < deadline,
                "grandchild pid was never recorded"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
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

    /// Only the launches that are actually *turns*. The post-turn pin check
    /// and HISTORY both spawn `opencode export` against this same stub, so a
    /// test asserting on turn argv has to say which children it means.
    fn run_launches(&self) -> Vec<Launch> {
        self.launches()
            .into_iter()
            .filter(|launch| launch.argv.first().map(String::as_str) == Some("run"))
            .collect()
    }

    fn wait_for_run_launches(&self, count: usize) -> Vec<Launch> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let launches = self.run_launches();
            if launches.len() >= count {
                return launches;
            }
            assert!(
                Instant::now() < deadline,
                "only {} of {count} turn launches recorded",
                launches.len()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
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

// ----------------------------------------------------------------- helpers

/// Whether a pid is still alive — genuinely running, not merely present in
/// the process table. Identical to `tests/codex_backend.rs::pid_alive`
/// (kept as its own copy here rather than a shared `support` fn, matching
/// this suite's existing precedent of a self-contained `StubOpencode` file):
/// a bare `kill -0` reports "alive" for an unreaped zombie too, and the
/// grandchild this test tracks is never this process's own child (it is
/// forked deep inside the stub's shell script and reparented to init the
/// instant that shell dies), so nothing in this process tree can `wait()` it
/// away. `ps -o state=` reports `Z` for a zombie on both procps and BSD
/// `ps`; that means the kernel already delivered the fatal signal, so this
/// reports it as dead even though init has not reaped its entry yet.
fn pid_alive(pid: u32) -> bool {
    match std::process::Command::new("ps")
        .args(["-o", "state=", "-p", &pid.to_string()])
        .output()
    {
        Ok(out) if out.status.success() => {
            !String::from_utf8_lossy(&out.stdout).trim().starts_with('Z')
        }
        // `ps` itself reports no such pid at all (nonzero exit) — gone.
        Ok(_) => false,
        // Couldn't gather the fact (`ps` missing?): fall back to plain
        // existence rather than silently declaring victory.
        Err(_) => std::process::Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false),
    }
}

fn config_for(stub: &StubOpencode, data_dir: &Path) -> OpencodeConfig {
    let mut config = OpencodeConfig::new(data_dir);
    config.executable = stub.path.clone();
    // Every stub test shrinks the first-event budget: a stub that never
    // emits a line should fail a test in seconds, not in the production
    // thirty. Per-instance, never an environment variable, so one test's
    // budget can never leak into another's.
    config.session_id_budget = Some(Duration::from_secs(5));
    config
}

fn start_request(cwd: &Path) -> StartRequest {
    StartRequest {
        work_id: "w-opencode".to_string(),
        execution_id: format!("exec-{}", ulid::Ulid::generate()),
        stage_id: "s1".to_string(),
        attempt: 1,
        cwd: cwd.to_path_buf(),
        intent: "do the opencode thing".to_string(),
        context: "context body".to_string(),
        model: None,
        profile: None,
        execute: None,
        instruction_policy: InstructionPolicy::default(),
        bindings: Vec::<BindingSummary>::new(),
    }
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
            .find(|event| event.kind == kind)
            .cloned()
        {
            return found;
        }
        assert!(Instant::now() < deadline, "no {kind} event arrived");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn events_of_kind(events: &Arc<Mutex<Vec<EventDraft>>>, kind: &str) -> Vec<EventDraft> {
    events
        .lock()
        .expect("lock")
        .iter()
        .filter(|event| event.kind == kind)
        .cloned()
        .collect()
}

/// Poll OBSERVE until the turn is no longer running. Deliberately keyed on
/// `native`, which on this transport means "the per-turn process has exited"
/// — the only thing that can end a `run` turn.
fn wait_for_settled(
    backend: &OpencodeBackend,
    handle: &ExecutionHandle,
) -> sergeant_rs::backend::Observation {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let observation = backend.observe(handle).expect("observe");
        if observation.native != NativeState::Running {
            return observation;
        }
        assert!(Instant::now() < deadline, "turn never settled");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// LAUNCH one execution against a stub, with the sink already installed.
fn launch_with(
    backend: &OpencodeBackend,
    request: &StartRequest,
) -> Result<ExecutionHandle, BackendError> {
    let prepared = backend.prepare(request)?;
    backend.launch(&prepared)
}

// ------------------------------------------------------------------- probe

#[test]
fn the_probe_reports_measured_provenance_at_the_floor() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let report: ProbeReport = backend.probe();
    assert!(report.available);
    let detail = report.detail.expect("detail");
    assert!(detail.contains("opencode 1.18.19"), "{detail}");
    assert!(
        !detail.contains("BELOW"),
        "at the floor is measured, not below it: {detail}"
    );
    assert!(detail.contains("transport: run-json"));
}

/// R1, verbatim: a build below the measured floor is usable, and the probe
/// says plainly that nothing here was re-measured against it.
#[test]
fn the_probe_reports_unmeasured_provenance_below_the_floor_and_stays_available() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::new(dir.path(), "opencode", "1.18.2", ALL_RUN_HELP, ALL_TOP_HELP);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let report = backend.probe();
    assert!(
        report.available,
        "R1: below-floor is available, never refused"
    );
    let detail = report.detail.expect("detail");
    assert!(detail.contains("BELOW the measured floor"), "{detail}");
    assert!(detail.to_lowercase().contains("unmeasured"));
}

/// The A2 split's third condition: an unparseable version is a refusal, and
/// that is not a version-policy decision — it is "this CLI cannot be
/// measured at all".
#[test]
fn the_probe_refuses_an_unparseable_version() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::new(
        dir.path(),
        "opencode",
        "nightly",
        ALL_RUN_HELP,
        ALL_TOP_HELP,
    );
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let report = backend.probe();
    assert!(!report.available);
    assert!(
        report
            .detail
            .expect("detail")
            .contains("cannot parse a version")
    );
}

/// The A2 split's second condition: a `run --help` missing a flag this
/// adapter's launch grammar composes is a grammar never measured here.
#[test]
fn the_probe_refuses_a_missing_run_flag_naming_it() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::new(
        dir.path(),
        "opencode",
        PASSING_VERSION,
        "opencode run [message..]\\nOptions: -m, --model  --format",
        ALL_TOP_HELP,
    );
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let report = backend.probe();
    assert!(!report.available);
    let detail = report.detail.expect("detail");
    assert!(detail.contains("--session"), "must name the flag: {detail}");
}

#[test]
fn the_probe_refuses_a_missing_subcommand_naming_it() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::new(
        dir.path(),
        "opencode",
        PASSING_VERSION,
        ALL_RUN_HELP,
        "Commands: opencode run [message..] | opencode serve",
    );
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let report = backend.probe();
    assert!(!report.available);
    let detail = report.detail.expect("detail");
    assert!(
        detail.contains("export"),
        "history: true rests on `opencode export`; a build without it must say so: {detail}"
    );
}

/// The probe reads help from **stderr**: the real CLI puts it there
/// (measured, yargs), so a stub that only wrote stdout would let a broken
/// probe pass.
#[test]
fn the_probe_reads_help_from_stderr_where_the_cli_actually_writes_it() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    let out = std::process::Command::new(&stub.path)
        .args(["run", "--help"])
        .output()
        .expect("run the stub's help");
    assert!(
        out.stdout.is_empty(),
        "the stub must reproduce the measured stream split"
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("--session"));
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    assert!(backend.probe().available);
}

#[test]
fn the_probe_detail_carries_the_admission_rows() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let detail = backend.probe().detail.expect("detail");
    assert!(detail.contains("admission rows:"));
    assert!(detail.contains("history | run-json | true"));
    assert!(detail.contains("ask | run-json | false"));
    assert!(detail.contains("stability (all rows)"));
}

#[test]
fn prepare_refuses_when_the_probe_itself_is_unavailable() {
    let dir = TempDir::new().expect("tempdir");
    let mut config = OpencodeConfig::new(dir.path());
    config.executable = dir.path().join("does-not-exist-at-all");
    let backend = OpencodeBackend::new(config);
    let err = backend
        .prepare(&start_request(dir.path()))
        .expect_err("an unavailable probe must refuse at PREPARE, before any spawn");
    assert!(matches!(err, BackendError::Unavailable { .. }));
    assert!(backend.tracked_executions().is_empty());
}

/// The session id is server-minted, so PREPARE has nothing to reserve — and
/// `PreparedExecution`'s own contract says that is honest, not a failure.
#[test]
fn prepare_reserves_no_native_id() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let request = start_request(dir.path());
    let prepared = backend.prepare(&request).expect("prepare");
    assert_eq!(prepared.native_id, None);
    assert_eq!(prepared.execution_id, request.execution_id);
    assert!(
        stub.launches().is_empty(),
        "PREPARE performs no external effect"
    );
}

#[test]
fn prepare_refuses_an_empty_model_pin() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let mut request = start_request(dir.path());
    request.model = Some("   ".to_string());
    assert!(matches!(
        backend.prepare(&request),
        Err(BackendError::Failed { .. })
    ));
}

// ------------------------------------------------------ launch and grammar

/// The admission test behind `persistent_sessions`: the identity this
/// execution is known by is the one the *harness* minted, read off the first
/// event line and nowhere else.
#[test]
fn launch_binds_the_server_minted_session_id() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    assert_eq!(handle.native_id.as_deref(), Some(MINIMAL_TURN_SESSION));
    let observation = wait_for_settled(&backend, &handle);
    assert!(
        observation
            .evidence
            .unwrap_or_default()
            .contains(MINIMAL_TURN_SESSION)
    );
    backend.stop(&handle).expect("stop").wait();
}

#[test]
fn first_turn_argv_and_stdin_carry_the_launch_grammar() {
    let dir = TempDir::new().expect("tempdir");
    let surface = dir.path().join("surface");
    std::fs::create_dir_all(&surface).expect("surface");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let mut request = start_request(&surface);
    request.model = Some(LIVE_MODEL.to_string());
    request.bindings = vec![BindingSummary {
        repository: "solo".to_string(),
        worktree_path: surface.join("solo"),
        work_branch: "sergeant/w-opencode".to_string(),
        base_branch: Some("main".to_string()),
        base_sha: "a".repeat(40),
    }];
    // Without an export document the pin check lands on `Attempted`, which
    // is not a stage failure — exactly the point of this test's neighbour
    // `an_unverifiable_pin_is_attempted_not_a_failure`.
    let handle = launch_with(&backend, &request).expect("launch");
    let launch = &stub.wait_for_run_launches(1)[0];
    assert_eq!(
        launch.argv,
        vec!["run", "--format", "json", "-m", LIVE_MODEL],
        "turn 1 carries no -s: the session does not exist until the harness mints it"
    );
    assert!(!launch.has("--auto"), "never auto-approve permissions");
    assert!(!launch.has("--dir"), "the surface is Command::current_dir");
    assert_eq!(
        launch.cwd,
        surface.canonicalize().expect("canonical").to_string_lossy(),
        "the turn runs in the Work's bound surface"
    );
    // The prompt travels on stdin (probe 8), with all five sections in order.
    let stdin = &launch.stdin;
    assert!(stdin.contains("Execution model: this is a single non-interactive turn"));
    assert!(stdin.contains("auto-rejected by the harness"));
    assert!(stdin.contains("Environment: if this session was reached through"));
    assert!(stdin.contains("Mutation surface: this Work may modify exactly"));
    assert!(stdin.contains("solo: "));
    assert!(stdin.contains("branch sergeant/w-opencode, cut from main at"));
    assert!(stdin.contains("do the opencode thing"));
    assert!(stdin.contains("context body"));
    let model_at = stdin.find("Execution model:").expect("execution model");
    let intent_at = stdin.find("do the opencode thing").expect("intent");
    let context_at = stdin.find("context body").expect("context");
    assert!(
        model_at < intent_at && intent_at < context_at,
        "section order"
    );
    backend.stop(&handle).expect("stop").wait();
}

#[test]
fn a_resume_turn_names_the_session_and_keeps_the_pin() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    stub.exports_session(MINIMAL_TURN_SESSION, EXPORT_SESSION);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let mut request = start_request(dir.path());
    request.model = Some(LIVE_MODEL.to_string());
    let handle = launch_with(&backend, &request).expect("launch");
    wait_for_settled(&backend, &handle);
    backend.send(&handle, "second turn").expect("send");
    let launches = stub.wait_for_run_launches(2);
    let resume = &launches[1];
    assert_eq!(
        resume.argv,
        vec![
            "run",
            "--format",
            "json",
            "-m",
            LIVE_MODEL,
            "-s",
            MINIMAL_TURN_SESSION
        ],
        "the pin is composed on every turn: a pin that lapses on turn 2 is a dropped decision"
    );
    assert_eq!(resume.value_after("-s"), Some(MINIMAL_TURN_SESSION));
    // The shell's `$(cat | tr ...)` drops the trailing separator; what matters is
    // that the whole later-turn prompt travelled on stdin, not on argv.
    assert_eq!(resume.stdin, "second turn");
    backend.stop(&handle).expect("stop").wait();
}

#[test]
fn send_refuses_a_second_turn_while_one_is_in_flight() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    stub.hangs_after_replay();
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    let err = backend
        .send(&handle, "concurrent")
        .expect_err("an opencode session runs one turn at a time");
    assert!(format!("{err}").contains("one turn at a time"));
    backend.stop(&handle).expect("stop").wait();
}

/// LAUNCH refuses when no event line ever arrives, because until one does
/// there is **no session identity** — nothing to hand back, nothing
/// resumable — and it leaves no phantom execution behind.
#[test]
fn launch_that_never_mints_a_session_refuses_and_leaves_no_phantom() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.writes_stderr("opencode: catastrophic startup failure\n");
    stub.exits_with(1);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let err = launch_with(&backend, &start_request(dir.path()))
        .expect_err("no event line means no identity");
    let message = format!("{err}");
    assert!(message.contains("no session was ever minted"), "{message}");
    assert!(
        message.contains("catastrophic startup failure"),
        "{message}"
    );
    assert!(
        backend.tracked_executions().is_empty(),
        "a failed launch leaves no phantom execution"
    );
}

/// The other half of the server-minted-identity problem: a turn that is
/// *alive* but silent still has no session id, so LAUNCH cannot hand back a
/// handle. It waits its bounded budget, kills the turn, and says why.
#[test]
fn launch_times_out_when_no_event_line_arrives_and_kills_the_turn() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    // The stall happens *before* the replay, so the process is alive and has
    // emitted nothing at all when the budget expires.
    stub.stalls_until_released();
    let mut config = config_for(&stub, dir.path());
    config.session_id_budget = Some(Duration::from_millis(400));
    let backend = OpencodeBackend::new(config);
    let err = launch_with(&backend, &start_request(dir.path()))
        .expect_err("a silent turn mints no session, so there is no identity to return");
    let message = format!("{err}");
    assert!(
        message.contains("emitted no event line within"),
        "{message}"
    );
    assert!(message.contains("server-minted"), "{message}");
    assert!(
        backend.tracked_executions().is_empty(),
        "a failed launch leaves no phantom execution"
    );
    stub.release_turn();
}

// --------------------------------------------------------------- streaming

/// The admission test behind `streaming`: normalized events reach the sink
/// **while the turn's process is still alive**, not in one batch at its end.
#[test]
fn events_are_delivered_before_the_turn_process_exits() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    stub.hangs_after_replay();
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    let assistant = wait_for_kind(&events, "conversation.assistant.completed");
    assert_eq!(assistant.payload["text"], "pong");
    assert_eq!(
        backend.observe(&handle).expect("observe").native,
        NativeState::Running,
        "the events above arrived while the turn was still running"
    );
    assert!(
        events_of_kind(&events, "conversation.turn.ended").is_empty(),
        "the turn has not ended yet"
    );
    backend.stop(&handle).expect("stop").wait();
}

/// The admission test behind `usage`: one `usage.updated` per `step_finish`,
/// carrying that step's native token object and cost verbatim.
#[test]
fn step_finish_tokens_and_cost_become_usage_events() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(TOOL_USE_TURN);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    wait_for_settled(&backend, &handle);
    wait_for_kind(&events, "conversation.turn.ended");
    let usage = events_of_kind(&events, "usage.updated");
    assert_eq!(usage.len(), 2, "one per step_finish, not one per turn");
    assert_eq!(usage[0].payload["reason"], "tool-calls");
    assert_eq!(usage[0].payload["step"], 1);
    assert_eq!(usage[0].payload["tokens"]["total"], 7798);
    assert_eq!(usage[1].payload["reason"], "stop");
    assert_eq!(usage[1].payload["step"], 2);
    assert_eq!(usage[1].payload["tokens"]["cache"]["read"], 7744);
    let ended = wait_for_kind(&events, "conversation.turn.ended").payload;
    assert_eq!(
        ended["tokens_final_step"]["total"], 7819,
        "the final step's token object verbatim, never a synthetic sum"
    );
    assert_eq!(ended["cost_total"], 0.0);
    backend.stop(&handle).expect("stop").wait();
}

// ------------------------------------------------------ tools and the rule

#[test]
fn a_tool_use_part_becomes_a_requested_completed_pair_with_its_structured_evidence() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(TOOL_USE_TURN);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    assert_eq!(handle.native_id.as_deref(), Some(TOOL_USE_SESSION));
    wait_for_settled(&backend, &handle);
    let requested = wait_for_kind(&events, "tool.requested").payload;
    assert_eq!(requested["name"], "bash");
    assert_eq!(requested["input"]["command"], "echo sergeant-probe-42");
    let completed = wait_for_kind(&events, "tool.completed").payload;
    assert_eq!(completed["is_error"], false);
    assert_eq!(completed["exit_code"], 0);
    assert_eq!(completed["output_tail"], "sergeant-probe-42\n");
    assert_eq!(
        events_of_kind(&events, "tool.requested").len(),
        1,
        "one call, one request"
    );
    backend.stop(&handle).expect("stop").wait();
}

/// The narration rule, end to end: the SIGKILL capture's text part narrates
/// running a command and the stream carries no `tool_use` line — so the
/// adapter must emit no `tool.*` event at all.
#[test]
fn narration_never_becomes_tool_evidence() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(SIGKILL_TRUNCATED);
    stub.exits_with(137);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    wait_for_settled(&backend, &handle);
    wait_for_kind(&events, "conversation.turn.ended");
    let text = wait_for_kind(&events, "conversation.assistant.completed").payload;
    assert!(
        text["text"].as_str().unwrap().contains("sleep command"),
        "the capture must actually narrate a command for this to be a control"
    );
    assert!(
        events
            .lock()
            .unwrap()
            .iter()
            .all(|event| !event.kind.starts_with("tool.")),
        "prose is transcript content, never tool evidence"
    );
    backend.stop(&handle).expect("stop").wait();
}

/// The admission test behind the `non_blocking_run` row (probe 4): an
/// `ask`-resolving permission auto-rejects — the tool part comes back as an
/// error, the process exits, and nothing hangs.
///
/// The capture is a three-line excerpt whose last line is a
/// `step_finish{reason:"tool-calls"}`, so this turn has no `stop` terminal
/// and settles ambiguous. That is the honest reading of the recorded bytes
/// and it is deliberately not papered over by appending an authored `stop`.
#[test]
fn an_auto_rejected_tool_call_is_a_failed_tool_event_not_a_hang() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(AUTOREJECT_TURN);
    stub.writes_stderr("! permission requested: bash (echo permission-probe); auto-rejecting\n");
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    let observation = wait_for_settled(&backend, &handle);
    let completed = wait_for_kind(&events, "tool.completed").payload;
    assert_eq!(completed["is_error"], true);
    assert_eq!(completed["status"], "error");
    assert!(
        completed["error"]
            .as_str()
            .unwrap()
            .contains("rejected permission")
    );
    assert_eq!(
        observation.native,
        NativeState::Unknown,
        "the excerpt carries no `stop`, so the turn is ambiguous — and it still ended promptly, \
         which is the non-hang claim this row is admitted on"
    );
    let ended = wait_for_kind(&events, "conversation.turn.ended").payload;
    assert_eq!(ended["outcome"], "ambiguous_unknown");
    assert!(
        ended["stderr"].as_str().unwrap().contains("auto-rejecting"),
        "the stderr notice is the whole explanation of the error tool part; issue #46's \
         sync-channel delivery is what keeps it from being lost to the EOF race"
    );
    backend.stop(&handle).expect("stop").wait();
}

// --------------------------------------------------------------- terminals

#[test]
fn a_completed_turn_reports_the_final_steps_text_as_its_summary() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(TOOL_USE_TURN);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    let observation = wait_for_settled(&backend, &handle);
    assert_eq!(observation.native, NativeState::Exited);
    match observation.signal {
        BackendSignal::StageCompleted { summary } => assert_eq!(
            summary.as_deref(),
            Some("sergeant-probe-42"),
            "the final step's text, not the narration that preceded the tool call"
        ),
        other => panic!("expected StageCompleted, got {other:?}"),
    }
    backend.stop(&handle).expect("stop").wait();
}

#[test]
fn a_typed_error_turn_is_a_failed_observation_naming_the_error() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(UNKNOWN_MODEL_ERROR);
    stub.exits_with(1);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    let observation = wait_for_settled(&backend, &handle);
    match observation.signal {
        BackendSignal::Failed { reason } => {
            assert!(reason.contains("UnknownError"), "{reason}");
            assert!(reason.contains("err_a2c9f0ac"), "{reason}");
        }
        other => panic!("expected Failed, got {other:?}"),
    }
    let harness_error = wait_for_kind(&events, "conversation.turn.harness_error").payload;
    assert_eq!(harness_error["phase"], "typed_error");
    backend.stop(&handle).expect("stop").wait();
}

/// §15's invariant, made a test: a process that dies with a nonzero status
/// and says nothing has not failed the stage — it has left the adapter unable
/// to tell, and ambiguity fails closed (`native: Unknown`, no stage verdict).
#[test]
fn a_turn_that_dies_without_a_terminal_is_ambiguous_and_fails_closed() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(SIGKILL_TRUNCATED);
    stub.exits_with(137);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let backend_handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    let observation = wait_for_settled(&backend, &backend_handle);
    assert_eq!(observation.native, NativeState::Unknown);
    assert_eq!(
        observation.signal,
        BackendSignal::Running,
        "no stage verdict may be drawn from a process that merely stopped talking"
    );
    let evidence = observation.evidence.expect("evidence");
    assert!(evidence.contains("exit_code=Some(137)"), "{evidence}");
    assert!(
        evidence.contains("b3:"),
        "the raw stream is cited: {evidence}"
    );
    backend.stop(&backend_handle).expect("stop").wait();
}

/// `interrupt`'s outcome semantics: the turn ends "interrupted, no verdict",
/// and the session identity survives so a later turn can continue it (probe
/// 10). The process-group *mechanism* itself — the thing that actually keeps
/// a bash-tool grandchild from surviving the kill (probe 11) — is proved
/// separately, below, by `opencode_interrupt_kills_the_process_group` (the
/// admission test the `interrupt` row now names).
#[test]
fn interrupt_kills_the_turn_and_leaves_the_session_resumable() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(SIGKILL_TRUNCATED);
    stub.hangs_after_replay();
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    assert_eq!(handle.native_id.as_deref(), Some(TRUNCATED_SESSION));
    // The turn is genuinely still running before the interrupt.
    assert_eq!(
        backend.observe(&handle).expect("observe").native,
        NativeState::Running
    );
    backend.interrupt(&handle).expect("interrupt").wait();
    let observation = wait_for_settled(&backend, &handle);
    assert_eq!(observation.native, NativeState::Exited);
    assert_eq!(
        observation.signal,
        BackendSignal::Running,
        "an interrupt sergeant asked for is not a stage verdict"
    );
    assert!(
        observation
            .evidence
            .unwrap_or_default()
            .contains("remains resumable")
    );
    let ended = wait_for_kind(&events, "conversation.turn.ended").payload;
    assert_eq!(ended["outcome"], "interrupted_running");
    assert_eq!(ended["interrupted"], true);
    // The session identity is intact: SEND continues the same conversation.
    backend
        .send(&handle, "continue")
        .expect("send after interrupt");
    let launches = stub.wait_for_run_launches(2);
    assert_eq!(launches[1].value_after("-s"), Some(TRUNCATED_SESSION));
    backend.stop(&handle).expect("stop").wait();
}

/// The deterministic half of the `interrupt` row's `ProcessTreeTermination`
/// claim (mirrors `tests/codex_backend.rs::codex_interrupt_kills_the_
/// process_group`, itself the admission test the row now names): a turn's
/// bash-tool commands run as *children of the opencode process*, so a plain
/// `child.kill()` on just the direct leader would leave any grandchild
/// running — probe 11 measured exactly that. INTERRUPT's whole justification
/// after probe 11 is that it kills the turn's process *group* instead
/// (`kill_process_group`/`kill_turn`) — this proves that mechanism, not just
/// that `interrupt()` returns `Ok`. If `interrupt`/`kill_inflight_turn` were
/// ever reverted to a plain `Child::kill()`, this test fails: the grandchild
/// is reparented to init and outlives the ten-second poll below.
#[test]
fn opencode_interrupt_kills_the_process_group() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(SIGKILL_TRUNCATED);
    stub.hangs_after_replay();
    stub.spawns_a_grandchild();
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");

    let grandchild_pid = stub.wait_for_grandchild_pid();
    assert!(
        pid_alive(grandchild_pid),
        "the grandchild must be running before INTERRUPT, or this test proves nothing"
    );
    assert_eq!(
        backend.observe(&handle).expect("observe").native,
        NativeState::Running,
        "the leader must still be running before INTERRUPT, or this test proves nothing"
    );

    backend.interrupt(&handle).expect("interrupt").wait();

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if !pid_alive(grandchild_pid) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "grandchild pid {grandchild_pid} survived INTERRUPT: the whole turn process group \
             should have been killed, not just the direct child (opencode leader) -- if \
             `interrupt`/`kill_inflight_turn` were reverted to a plain `Child::kill()`, this is \
             exactly the failure that would reproduce (probe 11)"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    backend.stop(&handle).expect("stop").wait();
}

#[test]
fn interrupting_an_execution_with_no_turn_in_flight_is_a_no_op() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    wait_for_settled(&backend, &handle);
    backend
        .interrupt(&handle)
        .expect("the goal state already holds")
        .wait();
    backend.stop(&handle).expect("stop").wait();
}

// -------------------------------------------------------------- model pins

/// The admission test behind `model_selection`: unlike codex-exec, a
/// substitution is *detectable* here — export names the served model — so it
/// becomes a failed observation that outranks the completion the turn
/// otherwise reached.
#[test]
fn a_substituted_model_is_a_failed_observation() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    // The recorded export document names opencode/big-pickle as the model
    // that served every assistant message; the request asks for a different
    // free-tier model.
    stub.exports_session(MINIMAL_TURN_SESSION, EXPORT_SESSION);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let mut request = start_request(dir.path());
    request.model = Some("opencode/hy3-free".to_string());
    let handle = launch_with(&backend, &request).expect("launch");
    let observation = wait_for_settled(&backend, &handle);
    match observation.signal {
        BackendSignal::Failed { reason } => {
            assert!(reason.contains("model pin not honored"), "{reason}");
            assert!(reason.contains("opencode/hy3-free"), "{reason}");
            assert!(reason.contains("opencode/big-pickle"), "{reason}");
        }
        other => panic!(
            "a turn served by a model the human did not ask for is not a completed stage, got \
             {other:?}"
        ),
    }
    backend.stop(&handle).expect("stop").wait();
}

#[test]
fn an_honored_model_pin_leaves_the_completion_standing() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    stub.exports_session(MINIMAL_TURN_SESSION, EXPORT_SESSION);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let mut request = start_request(dir.path());
    request.model = Some(LIVE_MODEL.to_string());
    let handle = launch_with(&backend, &request).expect("launch");
    let observation = wait_for_settled(&backend, &handle);
    assert!(matches!(
        observation.signal,
        BackendSignal::StageCompleted { .. }
    ));
    let ended = wait_for_kind(&events, "conversation.turn.ended").payload;
    assert_eq!(ended["model_pin"]["verdict"], "honored");
    assert_eq!(ended["model_pin"]["served"], LIVE_MODEL);
    backend.stop(&handle).expect("stop").wait();
}

/// Missing evidence is not a verdict about the Work: a pin that could not be
/// checked is `attempted`, and the stage still completes.
#[test]
fn an_unverifiable_pin_is_attempted_not_a_failure() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    // No export document registered: `opencode export` answers with the
    // measured not-found shape.
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let mut request = start_request(dir.path());
    request.model = Some(LIVE_MODEL.to_string());
    let handle = launch_with(&backend, &request).expect("launch");
    let observation = wait_for_settled(&backend, &handle);
    assert!(matches!(
        observation.signal,
        BackendSignal::StageCompleted { .. }
    ));
    let ended = wait_for_kind(&events, "conversation.turn.ended").payload;
    assert_eq!(ended["model_pin"]["verdict"], "attempted");
    assert!(
        ended["model_pin"]["detail"]
            .as_str()
            .unwrap()
            .contains("Session not found")
    );
    backend.stop(&handle).expect("stop").wait();
}

/// An unpinned turn pays for no export at all — the pin check is the only
/// thing that would spawn one, and there is nothing to check.
#[test]
fn an_unpinned_turn_never_runs_the_export_probe() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    wait_for_settled(&backend, &handle);
    let ended = wait_for_kind(&events, "conversation.turn.ended").payload;
    assert_eq!(ended["model_pin"]["verdict"], "unpinned");
    assert!(
        stub.launches().iter().all(|launch| !launch.has("export")),
        "the export subcommand is never recorded as a launch, and no pin check ran"
    );
    backend.stop(&handle).expect("stop").wait();
}

// ---------------------------------------------------------------- profiles

fn profile(name: &str) -> Profile {
    Profile {
        name: name.to_string(),
        backend: OPENCODE_BACKEND_NAME.to_string(),
        executable: None,
        config_home: None,
        env: BTreeMap::new(),
        default_model: None,
        options: BTreeMap::new(),
    }
}

/// The admission test behind `profiles`: the two generic sergeant axes W1
/// wires — executable and env — reach every turn of the conversation, not
/// just the first.
#[test]
fn a_profile_executable_and_env_reach_every_turn() {
    let dir = TempDir::new().expect("tempdir");
    let default_stub = StubOpencode::passing(dir.path());
    default_stub.replays(MINIMAL_TURN);
    let profile_stub = StubOpencode::new(
        dir.path(),
        "profiled",
        PASSING_VERSION,
        ALL_RUN_HELP,
        ALL_TOP_HELP,
    );
    profile_stub.replays(MINIMAL_TURN);
    let backend = OpencodeBackend::new(config_for(&default_stub, dir.path()));
    let mut request = start_request(dir.path());
    let mut launch_profile = profile("profiled");
    launch_profile.executable = Some(profile_stub.path.clone());
    launch_profile
        .env
        .insert("SGT_PROFILE_MARKER".to_string(), "w1".to_string());
    request.profile = Some(launch_profile);
    let handle = launch_with(&backend, &request).expect("launch");
    wait_for_settled(&backend, &handle);
    backend.send(&handle, "second turn").expect("send");
    let launches = profile_stub.wait_for_run_launches(2);
    assert!(
        default_stub.launches().is_empty(),
        "the profile's executable replaced the backend default for every turn"
    );
    for launch in &launches {
        assert_eq!(
            launch.env.get("SGT_PROFILE_MARKER").map(String::as_str),
            Some("w1"),
            "profile env reaches turn 1 and every later turn alike"
        );
    }
    backend.stop(&handle).expect("stop").wait();
}

/// Refused, not ignored: silently dropping a launch decision the human made
/// is the failure this refusal exists to prevent, and the message names the
/// measured channel to use instead.
#[test]
fn a_profile_config_home_is_refused_naming_the_measured_channel() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let mut request = start_request(dir.path());
    let mut launch_profile = profile("homed");
    launch_profile.config_home = Some(dir.path().join("oc-home"));
    request.profile = Some(launch_profile);
    let err = backend
        .prepare(&request)
        .expect_err("config_home is refused");
    let message = format!("{err}");
    assert!(
        message.contains("config_home is not supported"),
        "{message}"
    );
    assert!(message.contains(OPENCODE_CONFIG_CONTENT_ENV), "{message}");
    assert!(
        stub.launches().is_empty(),
        "an impossible profile never reaches a spawn"
    );
}

#[test]
fn a_profile_opencode_agent_option_is_refused_naming_the_lapse() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let mut request = start_request(dir.path());
    let mut launch_profile = profile("agented");
    launch_profile
        .options
        .insert("opencode_agent".to_string(), "build".to_string());
    request.profile = Some(launch_profile);
    let err = backend
        .prepare(&request)
        .expect_err("--agent is not wired in W1");
    let message = format!("{err}");
    assert!(
        message.contains("opencode_agent is not supported"),
        "{message}"
    );
    assert!(message.contains("unmeasured"), "{message}");
}

// -------------------------------------------------------- config injection

/// The admission test behind the `config_injection` row (probe 9): the
/// configured document reaches every child this adapter spawns — the turn
/// and the token-free `export` alike — so a turn and its own pin check can
/// never run under two different configurations.
#[test]
fn config_content_reaches_every_child_process() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    stub.exports_session(MINIMAL_TURN_SESSION, EXPORT_SESSION);
    let mut config = config_for(&stub, dir.path());
    config.config_content = Some("{\"permission\":{\"bash\":\"deny\"}}".to_string());
    let backend = OpencodeBackend::new(config);
    let mut request = start_request(dir.path());
    request.model = Some(LIVE_MODEL.to_string());
    let handle = launch_with(&backend, &request).expect("launch");
    wait_for_settled(&backend, &handle);
    // The turn's own launch, and the export the pin check ran, are both
    // recorded by the same stub.
    let launches = stub.wait_for_launches(2);
    for launch in &launches {
        assert_eq!(
            launch
                .env
                .get(OPENCODE_CONFIG_CONTENT_ENV)
                .map(String::as_str),
            Some("{\"permission\":{\"bash\":\"deny\"}}"),
            "argv was {:?}",
            launch.argv
        );
    }
    assert!(
        launches
            .iter()
            .any(|launch| launch.argv.first().map(String::as_str) == Some("export")),
        "the pin check's own export is one of the recorded children"
    );
    assert!(
        backend
            .probe()
            .detail
            .expect("detail")
            .contains("OPENCODE_CONFIG_CONTENT"),
        "an injected config is provenance the probe must state"
    );
    backend.stop(&handle).expect("stop").wait();
}

// ----------------------------------------------------------------- history

/// The deterministic half of `history`'s admission (the live half is
/// `live_opencode_history_exports_the_whole_session`): every message and
/// every part of the recorded export document is decoded, in order, with the
/// `reasoning` parts preserved rather than dropped.
#[test]
fn history_decodes_every_message_and_part_of_the_export_fixture() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    stub.exports_session(MINIMAL_TURN_SESSION, EXPORT_SESSION);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    wait_for_settled(&backend, &handle);
    let history = backend.history(&handle).expect("history");
    let kinds: Vec<&str> = history.iter().map(|event| event.kind.as_str()).collect();
    assert_eq!(
        kinds,
        vec![
            "conversation.user",
            "conversation.assistant.completed",
            "usage.updated",
            "conversation.turn.ended",
            "conversation.user",
            "conversation.assistant.completed",
            "usage.updated",
            "conversation.turn.ended",
        ],
        "the whole session, in order — never a prefix (§15's HISTORY contract)"
    );
    assert_eq!(history[1].payload["text"], "stored.");
    assert_eq!(history[5].payload["text"], "zebra-7134");
    assert_eq!(history[3].payload["model"], "big-pickle");
    assert_eq!(history[3].payload["provider"], "opencode");
    assert_eq!(
        history[3].payload["reasoning"].as_array().unwrap().len(),
        1,
        "opencode's reasoning parts have no §27 kind of their own; they are carried here rather \
         than silently dropped"
    );
    assert!(
        history[7].payload["undecoded_parts"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    backend.stop(&handle).expect("stop").wait();
}

/// `Ok(vec![])` from a backend that simply could not look is
/// indistinguishable from "this conversation said nothing" — the confusion
/// `Capabilities::history` exists to prevent. So an unreadable export is an
/// error naming why.
#[test]
fn history_reports_an_unreadable_export_rather_than_an_empty_list() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    wait_for_settled(&backend, &handle);
    let err = backend
        .history(&handle)
        .expect_err("an export that cannot be read is reported, never answered with an empty list");
    let message = format!("{err}");
    assert!(message.contains("Session not found"), "{message}");
    assert!(message.contains("history: true"), "{message}");
    backend.stop(&handle).expect("stop").wait();
}

#[test]
fn the_history_capability_is_the_one_this_registry_did_not_have() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    assert!(
        backend.capabilities().history,
        "R4: a measured better-than-claude capability is used, not left on the floor"
    );
}

// ------------------------------------------------------------- §20 archive

/// §20 evidence that could not be stored is *reported*, never dropped: a turn
/// whose raw capture silently does not exist is the failure this guards.
#[test]
fn an_archive_failure_is_reported_not_swallowed() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    // The blob store wants `<data_dir>/blobs/b3/`; a regular file at `blobs`
    // makes creating it fail, with no effect on anything else the adapter
    // does.
    let data_dir = dir.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("data dir");
    std::fs::write(data_dir.join("blobs"), b"not a directory").expect("block the blob store");
    let mut config = config_for(&stub, &data_dir);
    config.executable = stub.path.clone();
    let backend = OpencodeBackend::new(config);
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    let observation = wait_for_settled(&backend, &handle);
    let ended = wait_for_kind(&events, "conversation.turn.ended").payload;
    assert_eq!(ended["raw"], Value::Null);
    assert!(
        ended["raw_error"].is_string(),
        "the archive failure travels into the journal: {ended}"
    );
    assert!(
        observation
            .evidence
            .unwrap_or_default()
            .contains("unarchived ("),
        "and into OBSERVE's evidence, where a human reads it"
    );
    backend.stop(&handle).expect("stop").wait();
}

// -------------------------------------------------------- resume, identity

#[test]
fn resume_refuses_a_session_export_does_not_know() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let handle = ExecutionHandle {
        execution_id: "exec-restart".to_string(),
        native_id: Some("ses_never_existed".to_string()),
    };
    let err = backend
        .resume(&handle, &ResumeRequest::new("w-opencode", dir.path()))
        .expect_err("no durable session, no re-adoption");
    assert!(matches!(err, BackendError::UnknownExecution { .. }));
    assert!(backend.tracked_executions().is_empty());
}

#[test]
fn resume_without_a_native_id_is_unknown_execution() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let handle = ExecutionHandle {
        execution_id: "exec-restart".to_string(),
        native_id: None,
    };
    assert!(matches!(
        backend.resume(&handle, &ResumeRequest::new("w-opencode", dir.path())),
        Err(BackendError::UnknownExecution { .. })
    ));
}

/// A re-adopted session is resumable, and OBSERVE says exactly that *and*
/// that the turn in flight when the previous daemon died left an outcome this
/// one cannot read — unknown, not absent.
#[test]
fn resume_readopts_a_durable_session_and_observe_calls_the_outcome_unknown() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.exports_session("ses_readopted", EXPORT_SESSION);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let handle = ExecutionHandle {
        execution_id: "exec-restart".to_string(),
        native_id: Some("ses_readopted".to_string()),
    };
    backend
        .resume(&handle, &ResumeRequest::new("w-opencode", dir.path()))
        .expect("resume");
    assert_eq!(
        backend.tracked_executions(),
        vec!["exec-restart".to_string()]
    );
    // RESUME never starts a turn: re-adoption costs no tokens.
    assert!(
        stub.launches()
            .iter()
            .all(|launch| launch.argv.first().map(String::as_str) == Some("export")),
        "the only child RESUME spawns is the token-free export it re-adopts on"
    );
    let observation = backend.observe(&handle).expect("observe");
    assert_eq!(observation.native, NativeState::Exited);
    match observation.signal {
        BackendSignal::Blocked { reason } => {
            assert!(reason.contains("re-adopted"), "{reason}");
            assert!(reason.contains("unknown, not absent"), "{reason}");
        }
        other => panic!("expected Blocked, got {other:?}"),
    }
    // A second RESUME of the same identity is idempotent.
    backend
        .resume(&handle, &ResumeRequest::new("w-opencode", dir.path()))
        .expect("resume is idempotent");
}

#[test]
fn observe_refuses_a_handle_naming_a_different_session() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    wait_for_settled(&backend, &handle);
    let forged = ExecutionHandle {
        execution_id: handle.execution_id.clone(),
        native_id: Some("ses_someone_elses".to_string()),
    };
    assert!(
        matches!(
            backend.observe(&forged),
            Err(BackendError::UnknownExecution { .. })
        ),
        "§25: an execution is resolved by sergeant's id AND the native identity"
    );
    backend.stop(&handle).expect("stop").wait();
}

#[test]
fn stop_refuses_further_input() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    let backend = OpencodeBackend::new(config_for(&stub, dir.path()));
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    wait_for_settled(&backend, &handle);
    backend.stop(&handle).expect("stop").wait();
    let err = backend
        .send(&handle, "after stop")
        .expect_err("a stopped execution accepts no input");
    assert!(format!("{err}").contains("is stopped"));
}

// -------------------------------------------------------------- live suite

/// Gate mirroring `tests/codex_backend.rs`'s `LiveGate` (the A3 pattern).
/// There is no auth arm: the packet measured `opencode auth list` reporting
/// **zero credentials** while the free Zen models still served turns, so an
/// auth check here would gate on a fact that is not the one that matters.
#[derive(Debug, PartialEq, Eq)]
enum LiveGate {
    Run,
    NotOptedIn,
    Unusable(String),
}

fn live_gate(opt_in: Option<&str>, probe: &ProbeReport) -> LiveGate {
    if opt_in != Some("1") {
        return LiveGate::NotOptedIn;
    }
    if !probe.available {
        return LiveGate::Unusable(format!(
            "the installed opencode does not pass the adapter's probe: {}",
            probe.detail.clone().unwrap_or_default()
        ));
    }
    LiveGate::Run
}

/// Live config: the system `opencode` (or `SGT_OPENCODE_BIN`), and a scratch
/// data dir under `/var/tmp` — never `/tmp`, a quota'd tmpfs on this host
/// (issue #70).
fn live_config(data_dir: &Path) -> OpencodeConfig {
    OpencodeConfig::new(data_dir)
}

/// Whether the opt-in live tests may run. Reaching this with the opt-in
/// variable unset is a misuse of `-- --ignored` and panics, naming the
/// opt-in — the false green `#[ignore]` exists to prevent. An unusable
/// harness is a clean skip, written straight to fd 2 (libtest only captures
/// the print macros).
fn opencode_live_enabled(test: &str, data_dir: &Path) -> bool {
    let config = live_config(data_dir);
    let probe = OpencodeBackend::new(config).probe();
    match live_gate(
        std::env::var("SERGEANT_OPENCODE_TESTS").ok().as_deref(),
        &probe,
    ) {
        LiveGate::Run => true,
        LiveGate::NotOptedIn => panic!(
            "{test} is opt-in and consumes the owner's free-tier quota: run it with \
             SERGEANT_OPENCODE_TESTS=1 cargo test --test opencode_backend -- --ignored. (Without \
             the variable these tests are skipped by #[ignore]; asking for --ignored without it \
             must not report a green test that did nothing.)"
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
        .prefix(&format!("opencode-live-{name}-"))
        .tempdir_in("/var/tmp")
        .expect("scratch dir under /var/tmp, never /tmp (quota'd tmpfs on this host)")
}

/// A live request: bounded, one-word-answer, and pinned to the free model
/// (K1). `context` is empty so the whole prompt is the two contracts plus one
/// sentence.
fn live_request(cwd: &Path, intent: &str) -> StartRequest {
    let mut request = start_request(cwd);
    request.model = Some(LIVE_MODEL.to_string());
    request.intent = intent.to_string();
    request.context = String::new();
    request
}

#[test]
#[ignore = "opt-in, consumes free-tier quota: SERGEANT_OPENCODE_TESTS=1 cargo test --test opencode_backend -- --ignored"]
fn live_opencode_probe_reports_the_installed_version() {
    let data_dir = live_workdir("probe");
    if !opencode_live_enabled(
        "live_opencode_probe_reports_the_installed_version",
        data_dir.path(),
    ) {
        return;
    }
    let backend = OpencodeBackend::new(live_config(data_dir.path()));
    let report = backend.probe();
    assert!(report.available);
    let detail = report.detail.expect("detail");
    assert!(detail.contains("opencode "), "{detail}");
    assert!(detail.contains("transport: run-json"), "{detail}");
    assert!(detail.contains("admission rows:"), "{detail}");
}

#[test]
#[ignore = "opt-in, consumes free-tier quota: SERGEANT_OPENCODE_TESTS=1 cargo test --test opencode_backend -- --ignored"]
fn live_opencode_minimal_turn_completes_with_usage() {
    let data_dir = live_workdir("turn");
    if !opencode_live_enabled(
        "live_opencode_minimal_turn_completes_with_usage",
        data_dir.path(),
    ) {
        return;
    }
    let backend = OpencodeBackend::new(live_config(data_dir.path()));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let request = live_request(
        data_dir.path(),
        "Reply with exactly the word pong and nothing else.",
    );
    let handle = launch_with(&backend, &request).expect("launch");
    assert!(
        handle
            .native_id
            .as_deref()
            .is_some_and(|id| id.starts_with("ses_")),
        "the session id is minted by the server and learned from the first event"
    );
    // Incremental delivery, asserted on the event's arrival, never on prose.
    let _user = wait_for_kind(&events, "conversation.user");
    let observation = wait_for_settled(&backend, &handle);
    assert_eq!(observation.native, NativeState::Exited);
    assert!(
        matches!(observation.signal, BackendSignal::StageCompleted { .. }),
        "a live big-pickle turn ends with a step_finish reason \"stop\": {observation:?}"
    );
    let usage = events_of_kind(&events, "usage.updated");
    assert!(!usage.is_empty(), "usage is reported per step_finish");
    assert!(
        usage
            .iter()
            .all(|event| event.payload["tokens"]["total"].is_number()),
        "every usage event carries opencode's own token object"
    );
    let ended = wait_for_kind(&events, "conversation.turn.ended").payload;
    assert_eq!(ended["outcome"], "completed");
    assert_eq!(
        ended["model_pin"]["verdict"], "honored",
        "probe 7's positive pin evidence, live: export names the model that served this turn"
    );
    backend.stop(&handle).expect("stop").wait();
}

#[test]
#[ignore = "opt-in, consumes free-tier quota: SERGEANT_OPENCODE_TESTS=1 cargo test --test opencode_backend -- --ignored"]
fn live_opencode_resume_recalls_a_nonce_across_processes() {
    let data_dir = live_workdir("resume");
    if !opencode_live_enabled(
        "live_opencode_resume_recalls_a_nonce_across_processes",
        data_dir.path(),
    ) {
        return;
    }
    let backend = OpencodeBackend::new(live_config(data_dir.path()));
    let nonce = format!("zulu-{}", std::process::id());
    let request = live_request(
        data_dir.path(),
        &format!("Remember this nonce: {nonce}. Reply only: stored."),
    );
    let handle = launch_with(&backend, &request).expect("launch");
    wait_for_settled(&backend, &handle);
    backend
        .send(&handle, "What was the nonce? Reply with only the nonce.")
        .expect("send");
    let observation = wait_for_settled(&backend, &handle);
    match observation.signal {
        BackendSignal::StageCompleted { summary } => assert!(
            summary.unwrap_or_default().contains(&nonce),
            "turn 2 ran in a separate OS process and still recalled turn 1's nonce"
        ),
        other => panic!("expected StageCompleted, got {other:?}"),
    }
    backend.stop(&handle).expect("stop").wait();
}

#[test]
#[ignore = "opt-in, consumes free-tier quota: SERGEANT_OPENCODE_TESTS=1 cargo test --test opencode_backend -- --ignored"]
fn live_opencode_history_exports_the_whole_session() {
    let data_dir = live_workdir("history");
    if !opencode_live_enabled(
        "live_opencode_history_exports_the_whole_session",
        data_dir.path(),
    ) {
        return;
    }
    let backend = OpencodeBackend::new(live_config(data_dir.path()));
    let request = live_request(data_dir.path(), "Reply with exactly the word alpha.");
    let handle = launch_with(&backend, &request).expect("launch");
    wait_for_settled(&backend, &handle);
    backend
        .send(&handle, "Reply with exactly the word beta.")
        .expect("send");
    wait_for_settled(&backend, &handle);

    let history = backend.history(&handle).expect("history");
    let users = history
        .iter()
        .filter(|event| event.kind == "conversation.user")
        .count();
    let turns_ended = history
        .iter()
        .filter(|event| event.kind == "conversation.turn.ended")
        .count();
    assert_eq!(
        users, 2,
        "both prompts are in the durable record, not only the ones this process saw"
    );
    assert_eq!(turns_ended, 2, "one terminal per assistant message");
    assert!(
        history
            .iter()
            .any(|event| event.payload["model"] == "big-pickle"),
        "the export names the served model per message (probe 7)"
    );
    backend.stop(&handle).expect("stop").wait();
}

// ------------------------------------------------------- W2 §1.4: registration

/// A probeable opencode registers for real: `daemon::start_with`, with no
/// test-supplied stand-in for the name "opencode", puts a real
/// `OpencodeBackend` in its registry and journals its own probe evidence at
/// registration — the direct proof of W2's registration change, not a repeat
/// of any unit test on `OpencodeBackend` itself.
#[tokio::test]
async fn daemon_start_registers_opencode_and_journals_its_own_probe() {
    let data = TempDir::new().expect("tempdir");
    let stub = StubOpencode::passing(data.path());
    let handle = daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(sergeant_rs::backend::BackendRegistry::new()),
            default_backend: None,
            opencode: Some(config_for(&stub, data.path())),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start with an unmeasured-nothing opencode must still start");
    handle.shutdown().await;

    let probed: Vec<_> = Journal::replay_data_dir(data.path())
        .expect("replay")
        .map(|e| e.expect("event"))
        .filter(|e| e.kind == daemon::KIND_BACKEND_PROBED)
        .collect();
    let opencode_probed = probed
        .iter()
        .find(|e| e.payload["backend"] == OPENCODE_BACKEND_NAME)
        .expect("a backend.probed record for opencode");
    assert_eq!(opencode_probed.payload["available"], true);
    assert_eq!(opencode_probed.payload["runtime_scope"], "per_execution");
    assert_eq!(opencode_probed.payload["capabilities"]["ask"], false);
    assert_eq!(
        opencode_probed.payload["capabilities"]["persistent_sessions"],
        true
    );
    assert_eq!(
        opencode_probed.payload["capabilities"]["native_background"],
        false
    );
    assert_eq!(opencode_probed.payload["capabilities"]["streaming"], true);
    // NOTE: unlike codex (false), opencode's `history` is TRUE — R4, via
    // `export`, per opencode.rs:2690's Capabilities literal.
    assert_eq!(opencode_probed.payload["capabilities"]["history"], true);
    assert_eq!(opencode_probed.payload["capabilities"]["resume"], true);
    assert_eq!(opencode_probed.payload["capabilities"]["interrupt"], true);
    assert_eq!(
        opencode_probed.payload["capabilities"]["model_selection"],
        true
    );
    assert_eq!(opencode_probed.payload["capabilities"]["profiles"], true);
    assert_eq!(
        opencode_probed.payload["capabilities"]["approval_flow"],
        false
    );
    assert_eq!(
        opencode_probed.payload["capabilities"]["human_attach"],
        false
    );
    assert_eq!(opencode_probed.payload["capabilities"]["usage"], true);
    assert_eq!(
        opencode_probed.payload["capabilities"]["native_subagents"],
        false
    );
}

/// Opencode missing must not break daemon startup, and must be
/// distinguishable from "unregistered": it registers anyway and journals
/// honest, `available: false` evidence naming why it cannot run — the same
/// posture Docker already takes for a host with no Docker installed.
#[tokio::test]
async fn daemon_start_with_no_opencode_installed_still_starts_and_says_why() {
    let data = TempDir::new().expect("tempdir");
    let handle = daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(sergeant_rs::backend::BackendRegistry::new()),
            default_backend: None,
            opencode: Some(OpencodeConfig {
                executable: PathBuf::from("/nonexistent/definitely-not-opencode"),
                ..OpencodeConfig::new(data.path())
            }),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("a host with no opencode binary must still start a daemon");
    handle.shutdown().await;

    let opencode_probed = Journal::replay_data_dir(data.path())
        .expect("replay")
        .map(|e| e.expect("event"))
        .find(|e| {
            e.kind == daemon::KIND_BACKEND_PROBED && e.payload["backend"] == OPENCODE_BACKEND_NAME
        })
        .expect("opencode is registered — and probed — even though it cannot run");
    assert_eq!(opencode_probed.payload["available"], false);
    let detail = opencode_probed.payload["detail"]
        .as_str()
        .expect("probe detail recorded");
    assert!(detail.contains("cannot run"), "{detail}");
}
