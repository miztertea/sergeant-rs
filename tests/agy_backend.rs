//! W1 contract tests for `src/backend/agy.rs`.
//!
//! Two tiers, the A3 pattern from the codex sprint carried through opencode:
//!
//! - **Stub-driven** (this file's bulk): [`StubAgy`] is a shell script modelled
//!   directly on `tests/opencode_backend.rs`'s `StubOpencode`. It answers the
//!   capability probe (`--version` on **stdout**, `--help` on **stderr**, which
//!   is where the real CLI measurably puts it at 1.1.19 — W1 P0.1), answers the
//!   zero-quota `-p "/config" --output-format json` read, records every
//!   argv/env/cwd it is launched with, replays a recorded fixture on stdout, and
//!   can hang / write stderr / exit non-zero / fork a grandchild on
//!   demand. **No `agy` binary is required for any test in this tier.**
//! - **Live** (gated): every test is `#[ignore]`d *and* gated on
//!   `SERGEANT_AGY_TESTS=1` *and* an available probe *and* a quota precheck, and
//!   every live turn pins `--model gemini-3.7-flash-low` (owner ruling K1) with
//!   a bounded, one-word-answer prompt.
//!
//! No test here spawns a sergeant daemon, so `tests/support::DataDir`'s reaper
//! does not apply; only `support::wait_until_executable` is borrowed, for the
//! ETXTBSY window a freshly-chmod'ed stub has.
//!
//! **Fixture provenance.** Every file under `tests/fixtures/agy-1.1.19-*` is a
//! real capture at agy 1.1.19 **except** the three whose own first line says
//! otherwise — `dropped-stream-empty-success` (authored from 1.1.18's changelog
//! description), `soft-deny-success` (authored from the documented soft-deny
//! shape, which W1 P2 measured 1.1.19 *not* producing) and
//! `permission-denied-error-terminal` (authored from probe packet 2's prose
//! description of the 1.1.17 hard deny, which no longer reproduces). That label
//! line is itself an unknown event kind — counted, never interpreted — which is
//! why it can sit in an NDJSON fixture without changing what the decoder does.
//!
//! **The quota arm this suite has and no sibling could** (W1 P0.2): agy answers
//! `-p "/usage" --output-format json` with `usage.total_tokens: 0` and
//! `num_turns: 0`, so the live gate can check the remaining weekly fraction
//! *for free* before spending a turn, and skip loudly with the bucket's own
//! reset time rather than failing mysteriously at the end of a week.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;

use sergeant_rs::backend::agy::{AGY_BACKEND_NAME, AgyBackend, AgyConfig, TransportChoice};
use sergeant_rs::backend::{
    Backend, BackendError, BackendSignal, BindingSummary, ExecutionHandle, NativeState,
    Observation, ProbeReport, ResumeRequest, StartRequest,
};
use sergeant_rs::daemon::{self, DaemonConfig};
use sergeant_rs::domain::estate::InstructionPolicy;
use sergeant_rs::domain::event::EventDraft;
use sergeant_rs::domain::profile::Profile;
use sergeant_rs::runtime::journal::Journal;

mod support;

// ------------------------------------------------------- recorded fixtures

const MINIMAL_TURN: &str = include_str!("fixtures/agy-1.1.19-minimal-turn.jsonl");
const TOOL_USE: &str = include_str!("fixtures/agy-1.1.19-tool-use.jsonl");
const DENIED_CANCELED: &str = include_str!("fixtures/agy-1.1.19-permission-denied-canceled.jsonl");
const EMPTY_SUCCESS: &str = include_str!("fixtures/agy-1.1.19-dropped-stream-empty-success.jsonl");
const SOFT_DENY_SUCCESS: &str = include_str!("fixtures/agy-1.1.19-soft-deny-success.jsonl");
const INVALID_MODEL: &str = include_str!("fixtures/agy-1.1.19-invalid-model-refusal.jsonl");
const SLASH_COMMAND: &str = include_str!("fixtures/agy-1.1.19-slash-command-result.jsonl");
const SIGKILL_TRUNCATED: &str = include_str!("fixtures/agy-1.1.19-sigkill-truncated.jsonl");
const JSON_SCHEMA: &str = include_str!("fixtures/agy-1.1.19-json-schema.jsonl");

// ------------------------------------------------- W3 loop-transport captures

const LOOP_TWO_TURNS: &str = include_str!("fixtures/agy-1.1.19-loop-two-turns.jsonl");
const LOOP_SUBAGENT: &str = include_str!("fixtures/agy-1.1.19-loop-subagent-info.jsonl");
const LOOP_DENIED_TOOL: &str =
    include_str!("fixtures/agy-1.1.19-loop-denied-tool-kills-child.jsonl");
const LOOP_RESUME_INIT_ECHO: &str = include_str!("fixtures/agy-1.1.19-loop-resume-init-echo.jsonl");
const LOOP_CONTROL_REFUSAL: &str =
    include_str!("fixtures/agy-1.1.19-loop-control-request-refusal.jsonl");

/// The single `init` line a loop child emits at **child start**, before any
/// stdin line is read — the real capture of W3 P1 row I's empty-stdin child.
const LOOP_INIT_LINE: &str = include_str!("fixtures/agy-1.1.19-loop-init-only-empty-stdin.jsonl");

/// The conversation id every event of `agy-1.1.19-minimal-turn.jsonl` carries —
/// the id the adapter is expected to learn from that stream's `init` line.
const MINIMAL_TURN_CONVERSATION: &str = "8bfcc611-f2b9-4eb1-b17d-22b4caec46df";
/// The same, for the tool-use capture.
const TOOL_USE_CONVERSATION: &str = "f0e10575-b9d6-4263-bd21-665bf2841bf2";
/// The model `init` echoes in every real capture (they were all pinned to it).
const FIXTURE_MODEL: &str = "gemini-3.7-flash-low";

/// The auto-denial notice agy actually writes to stderr — the **captured bytes**
/// of W1 P2's control turn, byte-identical to P3 turn 1's capture, trailing
/// newline and em-dash included. At 1.1.19 this is the **only** machine-readable
/// evidence that a tool was denied.
///
/// It is a file, not a string literal, for the reason the JSONL captures are:
/// two hand-typed copies of the same "verbatim" text drift (this one did — a
/// hyphen for the em-dash and escaped quotes that were never in the capture),
/// and a fixture that claims provenance it does not have is exactly the
/// narration-vs-reality gap the wave's evidence rules exist to catch.
/// `src/backend/agy.rs`'s own `#[cfg(test)]` block includes the same file.
const DENIAL_NOTICE: &str = include_str!("fixtures/agy-1.1.19-denial-notice.txt");

/// The warning agy writes when `--conversation <id>` names something it does
/// not know — and then starts a *fresh* conversation anyway. Shape verbatim
/// from W1 P0.6 (which captured the zero-UUID form: plain double quotes, no
/// escapes); the id here is substituted for the stub's scenario.
const RESUME_FORK_WARNING: &str = "warning: conversation \"missing-id\" not found";

/// K1: every live turn in this file pins the cheap flash model, and nothing in
/// `src/backend/agy.rs` hardcodes a model — the pin travels on
/// `StartRequest::model` like any other (a unit test scans the source for one).
const LIVE_MODEL: &str = "gemini-3.7-flash-low";

// ------------------------------------------------------------ the stub CLI

/// `agy --help` text carrying every flag the probe requires. The real CLI writes
/// this to **stderr** (measured, W1 P0.1) and so does the stub.
const ALL_HELP: &str = "Usage of agy: --print -p --output-format --model --conversation \
                        --disable-slash-commands --json-schema --input-format --print-timeout";

/// The zero-quota `/config` answer, in the measured shape (W1 P0.2).
const DEFAULT_CONFIG_ANSWER: &str = r#"{"conversation_id":"","status":"SUCCESS","response":"","duration_seconds":0,"num_turns":0,"usage":{"input_tokens":0,"output_tokens":0,"thinking_tokens":0,"cache_read_tokens":0,"total_tokens":0},"command":{"name":"config","data":{"config":{"toolPermission":"request-review","permissions":null,"allowNonWorkspaceAccess":false,"trustedWorkspaces":[]}}}}"#;

const PASSING_VERSION: &str = "1.1.19";

/// A stub `agy` that answers the capability probe, **records every launch**, and
/// replays a recorded fixture on stdout.
struct StubAgy {
    path: PathBuf,
    record: PathBuf,
    replay: PathBuf,
    config_answer: PathBuf,
    config_hang: PathBuf,
    hang: PathBuf,
    stderr: PathBuf,
    exit_code: PathBuf,
    grandchild: PathBuf,
    grandchild_pid: PathBuf,
    /// The single `init` line the loop arm emits at child start, before any
    /// stdin line is read — the shape W3 P1 row I measured.
    loop_init: PathBuf,
    /// Prefix for the loop arm's per-turn marker files (`-stderr-<n>`,
    /// `-stderr-pre-<n>`, `-die-after-<n>`, `-no-init`, `-init-hang`,
    /// `-preinit-err`, `-preinit-out`, `-exit-now`).
    loop_prefix: PathBuf,
}

impl StubAgy {
    fn new(dir: &Path, name: &str, version: &str, help: &str) -> Self {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join(format!("{name}-stub"));
        let record = dir.join(format!("{name}-launches.txt"));
        let replay = dir.join(format!("{name}-replay.jsonl"));
        let loop_init = dir.join(format!("{name}-loop-init.jsonl"));
        let loop_prefix = dir.join(format!("{name}-loop"));
        let config_answer = dir.join(format!("{name}-config.json"));
        let config_hang = dir.join(format!("{name}-config-hang"));
        let hang = dir.join(format!("{name}-hang"));
        let stderr = dir.join(format!("{name}-stderr"));
        let exit_code = dir.join(format!("{name}-exit-code"));
        let grandchild = dir.join(format!("{name}-grandchild"));
        let grandchild_pid = dir.join(format!("{name}-grandchild-pid"));
        std::fs::write(&config_answer, DEFAULT_CONFIG_ANSWER).expect("write config answer");
        std::fs::write(&loop_init, LOOP_INIT_LINE).expect("write loop init");
        // The arms are ordered exactly as the real CLI resolves them: the
        // zero-quota slash-command read is `-p /config`, so it must be
        // recognised before the generic `-p` arm records a turn that never
        // happened.
        let script = format!(
            "#!/bin/sh\n\
             if [ \"$1\" = \"--version\" ]; then echo \"{version}\"; exit 0; fi\n\
             if [ \"$1\" = \"--help\" ]; then printf '%s\\n' \"{help}\" >&2; exit 0; fi\n\
             if [ \"$1\" = \"-p\" ] && [ \"$2\" = \"/config\" ]; then\n  \
               if [ -f \"{config_hang}\" ]; then exec sleep 60; fi\n  \
               cat \"{config_answer}\"; exit 0\n\
             fi\n\
             if [ \"$1\" = \"--print=\" ]; then\n                 {{ for arg in \"$@\"; do printf 'arg %s\\n' \"$arg\"; done;\n                   env | grep -E '^(HOME|SGT_[A-Z_]*|AGY_[A-Z_]*|PROBE_[A-Z_]*)=' | sed 's/^/env /' | tr -d '\\r';\n                   printf 'cwd %s\\n' \"$(pwd)\"; printf 'end\\n'; }} >> \"{record}\"\n                 if [ ! -f \"{loop_prefix}-no-init\" ]; then cat \"{loop_init}\"; fi\n                 if [ -f \"{loop_prefix}-preinit-err\" ]; then cat \"{loop_prefix}-preinit-err\" >&2; fi\n                 if [ -f \"{loop_prefix}-preinit-out\" ]; then cat \"{loop_prefix}-preinit-out\"; fi\n                 if [ -f \"{loop_prefix}-exit-now\" ]; then exit \"$(cat \"{loop_prefix}-exit-now\")\"; fi\n                 if [ -f \"{grandchild}\" ]; then\n                   ( i=0; while [ \"$i\" -lt 600 ]; do sleep 0.1; i=$((i+1)); done ) &\n                   echo $! > \"{grandchild_pid}\"\n                 fi\n                 if [ -f \"{loop_prefix}-init-hang\" ]; then exec sleep 60; fi\n                 n=0\n                 while IFS= read -r line; do\n                     n=$((n+1))\n                     printf 'stdin %s\\n' \"$line\" >> \"{record}\"\n                     if [ -f \"{loop_prefix}-stderr-pre-$n\" ]; then cat \"{loop_prefix}-stderr-pre-$n\" >&2; fi\n                     if [ -f \"{replay}\" ]; then awk -v n=$n 'BEGIN{{s=1}} /^---turn---$/{{s=s+1;next}} s==n{{print}}' \"{replay}\"; fi\n                     if [ -f \"{loop_prefix}-stderr-$n\" ]; then cat \"{loop_prefix}-stderr-$n\" >&2; fi\n                     if [ -f \"{loop_prefix}-die-after-$n\" ]; then exit \"$(cat \"{loop_prefix}-die-after-$n\")\"; fi\n                 done\n                 if [ -f \"{exit_code}\" ]; then exit \"$(cat \"{exit_code}\")\"; fi\n                 exit 0\n             fi\n\
             {{ for arg in \"$@\"; do printf 'arg %s\\n' \"$(printf '%s' \"$arg\" | tr '\\n' '|')\"; done;\n\
             env | grep -E '^(HOME|SGT_[A-Z_]*|AGY_[A-Z_]*|PROBE_[A-Z_]*)=' | sed 's/^/env /' | tr -d '\\r';\n\
             printf 'cwd %s\\n' \"$(pwd)\";\n\
             printf 'end\\n'; }} >> \"{record}\"\n\
             if [ -f \"{replay}\" ]; then cat \"{replay}\"; fi\n\
             if [ -f \"{stderr}\" ]; then cat \"{stderr}\" >&2; fi\n\
             if [ -f \"{grandchild}\" ]; then\n  \
               ( i=0; while [ \"$i\" -lt 600 ]; do sleep 0.1; i=$((i+1)); done ) &\n  \
               echo $! > \"{grandchild_pid}\"\n\
             fi\n\
             if [ -f \"{hang}\" ]; then exec sleep 60; fi\n\
             if [ -f \"{exit_code}\" ]; then exit \"$(cat \"{exit_code}\")\"; fi\n\
             exit 0\n",
            version = version,
            help = help,
            config_answer = config_answer.display(),
            config_hang = config_hang.display(),
            loop_init = loop_init.display(),
            loop_prefix = loop_prefix.display(),
            record = record.display(),
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
            config_answer,
            config_hang,
            hang,
            stderr,
            exit_code,
            grandchild,
            grandchild_pid,
            loop_init,
            loop_prefix,
        }
    }

    /// A stub that passes every probe gate.
    fn passing(dir: &Path) -> Self {
        Self::new(dir, "agy", PASSING_VERSION, ALL_HELP)
    }

    fn replays(&self, transcript: &str) -> &Self {
        std::fs::write(&self.replay, transcript).expect("write replay");
        self
    }

    fn answers_config(&self, json: &str) -> &Self {
        std::fs::write(&self.config_answer, json).expect("write config answer");
        self
    }

    /// Replay, then stay alive — the shape a streaming assertion needs: the
    /// events must be observable *while the process is still running*.
    fn hangs_after_replay(&self) -> &Self {
        std::fs::write(&self.hang, b"hang\n").expect("write hang marker");
        self
    }

    /// Makes the zero-quota `-p "/config"` read specifically hang — the
    /// unauthenticated-CLI-blocks-on-an-interactive-login case
    /// `CONFIG_PROBE_BUDGET`'s doc describes. Deliberately distinct from
    /// `hangs_after_replay`: that marker is checked *after* the script's
    /// recording arm, which the `/config` call never reaches (it returns from
    /// its own arm first), so it cannot exercise `read_config_probe`'s
    /// kill-on-timeout path at all.
    fn hangs_on_config(&self) -> &Self {
        std::fs::write(&self.config_hang, b"hang\n").expect("write config-hang marker");
        self
    }

    /// Replace the loop child's `init` line (identity, resolved model,
    /// permission mode).
    fn loop_init(&self, line: &str) -> &Self {
        std::fs::write(&self.loop_init, line).expect("write loop init");
        self
    }

    /// Emit no `init` line at all, then block: the launch-fails-closed case.
    fn loop_never_initializes(&self) -> &Self {
        std::fs::write(format!("{}-no-init", self.loop_prefix.display()), b"x")
            .expect("write no-init marker");
        std::fs::write(format!("{}-init-hang", self.loop_prefix.display()), b"x")
            .expect("write init-hang marker");
        self
    }

    /// Emit no `init` line and then **exit promptly** with `code`, saying
    /// nothing at all. Deliberately distinct from [`Self::loop_never_initializes`],
    /// which hangs: that shape can only ever exercise the LAUNCH-side
    /// `recv_timeout` expiry, and this one is the only way to reach
    /// `LoopReader`'s own `Terminal::None => ExitedWithoutInit` classification.
    fn loop_exits_before_init(&self, code: i32, stderr: &str) -> &Self {
        std::fs::write(format!("{}-no-init", self.loop_prefix.display()), b"x")
            .expect("write no-init marker");
        std::fs::write(
            format!("{}-preinit-err", self.loop_prefix.display()),
            stderr,
        )
        .expect("write pre-init stderr");
        std::fs::write(
            format!("{}-exit-now", self.loop_prefix.display()),
            code.to_string(),
        )
        .expect("write exit-now marker");
        self
    }

    /// Emit no `init` line but a typed terminal `result` — a harness that
    /// refuses before minting a conversation — then exit with `code`. Reaches
    /// `LoopReader`'s `Terminal::Status => RefusedBeforeIdentity` arm, the one
    /// whose refusal quotes agy's own `error` verbatim.
    fn loop_refuses_before_init(&self, result_line: &str, code: i32) -> &Self {
        std::fs::write(format!("{}-no-init", self.loop_prefix.display()), b"x")
            .expect("write no-init marker");
        std::fs::write(
            format!("{}-preinit-out", self.loop_prefix.display()),
            format!("{}\n", result_line.trim_end()),
        )
        .expect("write pre-init output");
        std::fs::write(
            format!("{}-exit-now", self.loop_prefix.display()),
            code.to_string(),
        )
        .expect("write exit-now marker");
        self
    }

    /// Emit the `init` line and then block for ever without reading stdin —
    /// used where a test needs a live child it can kill.
    fn loop_hangs_after_init(&self) -> &Self {
        std::fs::write(format!("{}-init-hang", self.loop_prefix.display()), b"x")
            .expect("write init-hang marker");
        self
    }

    /// stderr written *after* turn `n`'s replay segment (inside the turn).
    fn loop_stderr_after_turn(&self, n: usize, text: &str) -> &Self {
        std::fs::write(format!("{}-stderr-{n}", self.loop_prefix.display()), text)
            .expect("write loop stderr");
        self
    }

    /// stderr written *before* turn `n`'s replay segment — which, for `n >= 2`,
    /// lands in the window between two turns and is the `adjacent` attribution
    /// case §2.6 exists for.
    fn loop_stderr_before_turn(&self, n: usize, text: &str) -> &Self {
        std::fs::write(
            format!("{}-stderr-pre-{n}", self.loop_prefix.display()),
            text,
        )
        .expect("write loop stderr");
        self
    }

    /// Exit with `code` immediately after turn `n`'s segment — a child that
    /// dies between turns (or mid-turn, when segment `n` carries no terminal).
    fn loop_dies_after_turn(&self, n: usize, code: i32) -> &Self {
        std::fs::write(
            format!("{}-die-after-{n}", self.loop_prefix.display()),
            code.to_string(),
        )
        .expect("write loop die marker");
        self
    }

    /// Every stdin line this loop child was handed, in order.
    fn loop_stdin_lines(&self) -> Vec<String> {
        let Ok(text) = std::fs::read_to_string(&self.record) else {
            return Vec::new();
        };
        text.lines()
            .filter_map(|line| line.strip_prefix("stdin ").map(str::to_string))
            .collect()
    }

    fn wait_for_loop_stdin_lines(&self, count: usize) -> Vec<String> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let lines = self.loop_stdin_lines();
            if lines.len() >= count {
                return lines;
            }
            assert!(
                Instant::now() < deadline,
                "only {} of {count} stdin lines recorded",
                lines.len()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// Only the launches that are loop children (argv[0] is `--print=`).
    fn loop_launches(&self) -> Vec<Launch> {
        self.launches()
            .into_iter()
            .filter(|launch| launch.argv.first().map(String::as_str) == Some("--print="))
            .collect()
    }

    fn wait_for_loop_launches(&self, count: usize) -> Vec<Launch> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let launches = self.loop_launches();
            if launches.len() >= count {
                return launches;
            }
            assert!(
                Instant::now() < deadline,
                "only {} of {count} loop launches recorded",
                launches.len()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn writes_stderr(&self, text: &str) -> &Self {
        std::fs::write(&self.stderr, text).expect("write stderr fixture");
        self
    }

    fn exits_with(&self, code: i32) -> &Self {
        std::fs::write(&self.exit_code, code.to_string()).expect("write exit code");
        self
    }

    /// Fork a background grandchild (a detached loop, not a `setsid`) right
    /// after replay and record its own pid — the process a single
    /// `child.kill()` would never reach, and the whole reason INTERRUPT kills
    /// the turn's process *group* (opencode probe 11, carried without
    /// re-deriving it).
    fn spawns_a_grandchild(&self) -> &Self {
        std::fs::write(&self.grandchild, b"go\n").expect("write grandchild marker");
        self
    }

    fn wait_for_grandchild_pid(&self) -> u32 {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(pid) = std::fs::read_to_string(&self.grandchild_pid)
                .ok()
                .and_then(|text| text.trim().parse::<u32>().ok())
            {
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
                _ if line == "end" => launches.push(std::mem::take(&mut current)),
                _ => {}
            }
        }
        launches
    }

    /// Only the launches that are actually *turns*. Nothing else reaches the
    /// recording arm today (the probe's `--version`/`--help`/`-p /config` all
    /// return before it), but saying which children a test means keeps the
    /// assertion honest if that ever changes.
    fn turn_launches(&self) -> Vec<Launch> {
        self.launches()
            .into_iter()
            .filter(|launch| launch.argv.first().map(String::as_str) == Some("-p"))
            .collect()
    }

    fn wait_for_turn_launches(&self, count: usize) -> Vec<Launch> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let launches = self.turn_launches();
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
}

#[derive(Debug, Default)]
struct Launch {
    argv: Vec<String>,
    env: BTreeMap<String, String>,
    cwd: String,
}

impl Launch {
    fn has(&self, flag: &str) -> bool {
        self.argv.iter().any(|arg| arg == flag)
    }

    fn value_after(&self, flag: &str) -> Option<&str> {
        let index = self.argv.iter().position(|arg| arg == flag)?;
        self.argv.get(index + 1).map(String::as_str)
    }

    /// The prompt: the value of `-p`, which is argv[1] by construction
    /// (W1 P0.3 — the prompt rides argv and only argv on this transport).
    fn prompt(&self) -> &str {
        self.value_after("-p").unwrap_or_default()
    }
}

// ----------------------------------------------------------------- helpers

/// Whether a pid is still alive — genuinely running, not merely present in the
/// process table. A bare `kill -0` reports "alive" for an unreaped zombie too,
/// and the grandchild these tests track is never this process's own child (it is
/// forked deep inside the stub's shell and reparented to init the instant that
/// shell dies), so nothing in this process tree can `wait()` it away.
fn pid_alive(pid: u32) -> bool {
    match std::process::Command::new("ps")
        .args(["-o", "state=", "-p", &pid.to_string()])
        .output()
    {
        Ok(out) if out.status.success() => {
            !String::from_utf8_lossy(&out.stdout).trim().starts_with('Z')
        }
        Ok(_) => false,
        Err(_) => std::process::Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false),
    }
}

/// A **print-transport** config. Pinned explicitly rather than left on `Auto`,
/// and that is the honest wiring: `ALL_HELP` offers `--input-format`, so `Auto`
/// against this stub resolves to the loop — which would have silently
/// re-pointed every W1 test at a transport it was never written for. W1's suite
/// is the print transport's suite and now says so.
fn config_for(stub: &StubAgy, data_dir: &Path) -> AgyConfig {
    let mut config = AgyConfig::new(data_dir);
    config.executable = stub.path.clone();
    config.transport = TransportChoice::PrintOnly;
    // Every stub test shrinks the init-line budget: a stub that never emits a
    // line should fail a test in seconds, not in the production thirty.
    // Per-instance, never an environment variable, so one test's budget can
    // never leak into another's `launch()`.
    config.init_line_budget = Some(Duration::from_secs(5));
    config
}

/// A **loop-transport** config, left on `Auto` on purpose: the resolution
/// itself is part of what W3's tests exercise, and pinning `LoopOnly`
/// everywhere would leave the default path untested.
fn loop_config_for(stub: &StubAgy, data_dir: &Path) -> AgyConfig {
    let mut config = config_for(stub, data_dir);
    config.transport = TransportChoice::Auto;
    config
}

/// Split a real loop capture into the child's single `init` line and a replay
/// file whose segments — one per turn, separated by the stub's sentinel — the
/// loop arm emits one at a time as stdin lines arrive.
///
/// Driven off the captures themselves rather than hand-written NDJSON, for the
/// reason every fixture here is a file: a hand-typed "verbatim" shape drifts.
fn loop_capture(capture: &str) -> (String, String) {
    let mut init = String::new();
    let mut segments: Vec<String> = vec![String::new()];
    for line in capture.lines().filter(|line| !line.trim().is_empty()) {
        if init.is_empty() && line.contains("\"event\":\"init\"") {
            init = format!("{line}\n");
            continue;
        }
        segments
            .last_mut()
            .expect("a current segment")
            .push_str(&format!("{line}\n"));
        if line.contains("\"event\":\"result\"") {
            segments.push(String::new());
        }
    }
    while segments.last().is_some_and(|last| last.is_empty()) {
        segments.pop();
    }
    (init, segments.join("---turn---\n"))
}

/// The conversation id a capture's own `init` line mints. Read out of the
/// fixture rather than hardcoded per test: a capture swapped for another must
/// not leave an assertion quietly checking the wrong conversation.
fn conversation_of(capture: &str) -> String {
    let init = capture
        .lines()
        .find(|line| line.contains(r#""event":"init""#))
        .expect("a capture with an init line");
    serde_json::from_str::<Value>(init).expect("init parses")["conversation_id"]
        .as_str()
        .expect("a conversation id")
        .to_string()
}

/// Wait until at least `count` `conversation.turn.ended` events have landed.
fn wait_for_turns_ended(events: &Arc<Mutex<Vec<EventDraft>>>, count: usize) -> Vec<EventDraft> {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let ended = events_of_kind(events, "conversation.turn.ended");
        if ended.len() >= count {
            return ended;
        }
        assert!(
            Instant::now() < deadline,
            "only {} of {count} turns ended",
            ended.len()
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// LAUNCH one loop execution against a stub replaying `capture`'s turns.
fn loop_launched(
    dir: &TempDir,
    capture: &str,
) -> (
    StubAgy,
    AgyBackend,
    ExecutionHandle,
    Arc<Mutex<Vec<EventDraft>>>,
) {
    let stub = StubAgy::passing(dir.path());
    let (init, replay) = loop_capture(capture);
    stub.loop_init(&init);
    stub.replays(&replay);
    let backend = AgyBackend::new(loop_config_for(&stub, dir.path()));
    let (sink, events) = sink();
    backend.set_event_sink(sink);
    let handle = launch_with(&backend, &loop_pinned_request(dir.path())).expect("launch");
    (stub, backend, handle, events)
}

/// The pin every loop capture's `init` line echoes.
fn loop_pinned_request(cwd: &Path) -> StartRequest {
    let mut request = start_request(cwd);
    request.model = Some(FIXTURE_MODEL.to_string());
    request
}

fn start_request(cwd: &Path) -> StartRequest {
    StartRequest {
        work_id: "w-agy".to_string(),
        execution_id: format!("exec-{}", ulid::Ulid::generate()),
        stage_id: "s1".to_string(),
        attempt: 1,
        cwd: cwd.to_path_buf(),
        intent: "do the agy thing".to_string(),
        context: "context body".to_string(),
        model: None,
        profile: None,
        execute: None,
        instruction_policy: InstructionPolicy::default(),
        bindings: Vec::<BindingSummary>::new(),
    }
}

/// A request whose pin matches what every real fixture's `init` line echoes, so
/// a stub-driven launch exercises the *honored* path rather than refusing.
fn pinned_request(cwd: &Path) -> StartRequest {
    let mut request = start_request(cwd);
    request.model = Some(FIXTURE_MODEL.to_string());
    request
}

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

/// Poll OBSERVE until the turn is no longer running. Keyed on `native`, which on
/// this transport means "the per-turn process has exited" — the only thing that
/// can end a print-mode turn.
fn wait_for_settled(backend: &AgyBackend, handle: &ExecutionHandle) -> Observation {
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

fn launch_with(
    backend: &AgyBackend,
    request: &StartRequest,
) -> Result<ExecutionHandle, BackendError> {
    let prepared = backend.prepare(request)?;
    backend.launch(&prepared)
}

/// LAUNCH one execution against a passing stub replaying `fixture`, with the
/// sink installed — the shape most tests below start from.
fn launched(
    dir: &TempDir,
    fixture: &str,
) -> (
    StubAgy,
    AgyBackend,
    ExecutionHandle,
    Arc<Mutex<Vec<EventDraft>>>,
) {
    let stub = StubAgy::passing(dir.path());
    stub.replays(fixture);
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let (sink, events) = sink();
    backend.set_event_sink(sink);
    let handle = launch_with(&backend, &pinned_request(dir.path())).expect("launch");
    (stub, backend, handle, events)
}

// ------------------------------------------------------------------- probe

#[test]
fn the_probe_reports_available_with_measured_provenance_at_or_above_the_floor() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let report: ProbeReport = backend.probe();
    assert!(report.available);
    let detail = report.detail.expect("detail");
    // 1.1.19 is above the 1.1.17 floor, and both numbers are visible: the
    // installed one here, the floor in the ledger's own header.
    assert!(detail.contains("agy 1.1.19"), "{detail}");
    assert!(
        !detail.contains("BELOW"),
        "at/above the floor is measured: {detail}"
    );
    assert!(detail.contains("transport: print-stream-json"));
}

/// R1, verbatim: a build below the measured floor is **usable**, and the probe
/// says plainly that nothing here was re-measured against it.
#[test]
fn a_build_below_the_floor_is_available_with_unmeasured_provenance() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::new(dir.path(), "agy", "1.1.2", ALL_HELP);
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let report = backend.probe();
    assert!(
        report.available,
        "R1: below-floor is available, never refused"
    );
    let detail = report.detail.expect("detail");
    assert!(
        detail.contains("BELOW the measured floor 1.1.17"),
        "{detail}"
    );
    assert!(detail.to_lowercase().contains("unmeasured"));
}

/// The A2 split's lower half: an unparseable version is a refusal, and that is
/// not a version-policy decision — it is "this CLI cannot be measured at all".
#[test]
fn an_unparseable_version_is_refused() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::new(dir.path(), "agy", "nightly", ALL_HELP);
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let report = backend.probe();
    assert!(!report.available);
    let detail = report.detail.expect("detail");
    assert!(detail.contains("cannot parse a version"), "{detail}");
    assert!(
        detail.contains("nightly"),
        "the full string travels: {detail}"
    );
}

/// The A2 split's grammar half: a `--help` missing a flag this adapter composes
/// is a grammar this adapter has never measured, and the refusal names *which*.
#[test]
fn a_help_missing_a_required_flag_is_refused_naming_it() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::new(
        dir.path(),
        "agy",
        PASSING_VERSION,
        "Usage of agy: --print -p --output-format --model --json-schema",
    );
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let report = backend.probe();
    assert!(!report.available);
    let detail = report.detail.expect("detail");
    assert!(
        detail.contains("--conversation"),
        "must name the flag: {detail}"
    );
    assert!(detail.contains("--disable-slash-commands"), "{detail}");
    assert!(detail.contains("never measured against it"), "{detail}");
}

/// W1 P0.1: the real CLI writes its help to **stderr and only stderr**, which
/// is the opposite of what the wave's own spec recorded. The probe reads and
/// concatenates both streams, so this is a measured test rather than a
/// spurious refusal — and a stub that only wrote stdout would let a broken
/// probe pass.
#[test]
fn the_probe_reads_help_from_stderr_where_the_cli_actually_writes_it() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let out = std::process::Command::new(&stub.path)
        .arg("--help")
        .output()
        .expect("run the stub's help");
    assert!(
        out.stdout.is_empty(),
        "the stub must reproduce the measured stream split"
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("--conversation"));
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    assert!(backend.probe().available);
}

#[test]
fn a_missing_executable_is_refused_naming_it() {
    let dir = TempDir::new().expect("tempdir");
    let mut config = AgyConfig::new(dir.path());
    config.executable = dir.path().join("no-such-agy");
    let report = AgyBackend::new(config).probe();
    assert!(!report.available);
    assert!(report.detail.expect("detail").contains("no-such-agy"));
}

#[test]
fn the_probe_detail_carries_the_admission_rows_and_the_permission_posture() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let detail = backend.probe().detail.expect("detail");
    assert!(detail.contains("admission rows:"));
    assert!(detail.contains("history | print-stream-json | false"));
    assert!(detail.contains("ask | print-stream-json | false"));
    assert!(detail.contains("config_injection | print-stream-json | true"));
    // §11.3: `sgt doctor` (W2's reader) sees the effective posture before any
    // Work runs — read zero-quota, so this costs nothing.
    assert!(
        detail.contains("effective toolPermission=\"request-review\""),
        "{detail}"
    );
    assert!(detail.contains("read zero-quota"), "{detail}");
    assert!(
        detail.contains("no settings home configured"),
        "an unconfigured daemon must be told what it is running on: {detail}"
    );
}

#[test]
fn the_probe_names_the_injection_channel_when_one_is_configured() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let mut config = config_for(&stub, dir.path());
    config.settings_home = Some(dir.path().join("settings-home"));
    let detail = AgyBackend::new(config).probe().detail.expect("detail");
    assert!(detail.contains("injected per launch via HOME="), "{detail}");
    assert!(
        detail.contains("relocates the credential and conversation stores"),
        "the operator footgun is stated, not assumed: {detail}"
    );
}

/// A CLI that cannot answer the zero-quota `/config` read is **not** refused: it
/// is a CLI whose effective configuration this adapter cannot report, and the
/// detail says exactly that.
#[test]
fn an_unreadable_config_probe_is_reported_not_refused() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.answers_config("not json at all");
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let report = backend.probe();
    assert!(report.available, "an unreadable /config is never a refusal");
    assert!(
        report
            .detail
            .expect("detail")
            .contains("cannot report the harness's effective permission configuration")
    );
}

/// The regression `CONFIG_PROBE_BUDGET` exists to prevent: an unauthenticated
/// `agy` blocks the `/config` read on an interactive login prompt. This must
/// never become an unbounded wait inside `daemon::start_with` — the exact
/// class of registration-time hang this project's CI/CD sprint already
/// tracked once for a blocking HTTP client (0.2.2 regression, commit
/// c46152a2), now with a subprocess in place of a transport.
#[test]
fn a_hung_config_probe_is_killed_within_budget_and_falls_back() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.hangs_on_config();
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let started = Instant::now();
    let report = backend.probe();
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(20),
        "read_config_probe must be bounded by CONFIG_PROBE_BUDGET (5s), not the stub's \
         60s hang: probe() took {elapsed:?}"
    );
    assert!(
        report.available,
        "a killed /config read is a best-effort miss, never a refusal"
    );
    assert!(
        report
            .detail
            .expect("detail")
            .contains("cannot report the harness's effective permission configuration"),
        "a timed-out config probe must fall back to the same ConfigProbe::default() detail \
         as an unreadable one"
    );
}

// ----------------------------------------------------------------- prepare

#[test]
fn prepare_reserves_no_native_id() {
    // The conversation id is harness-minted and first appears on the `init`
    // line, so there is nothing to reserve — `PreparedExecution::native_id:
    // None` is exactly the honest answer its own contract blesses.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    assert_eq!(prepared.native_id, None);
    assert_eq!(prepared.request.execution_id, prepared.execution_id);
}

#[test]
fn prepare_refuses_an_unavailable_probe() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::new(dir.path(), "agy", "nightly", ALL_HELP);
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    assert!(matches!(
        backend.prepare(&start_request(dir.path())),
        Err(BackendError::Unavailable { .. })
    ));
}

#[test]
fn prepare_refuses_an_empty_pin_but_not_an_unrecognized_one() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let mut request = start_request(dir.path());
    request.model = Some("   ".to_string());
    assert!(backend.prepare(&request).is_err());
    // An unrecognized model is left to the harness: its own typed refusal
    // enumerates the whole catalog, which is better evidence than a local
    // allowlist this adapter would have to maintain (R1).
    request.model = Some("not-a-real-model".to_string());
    assert!(backend.prepare(&request).is_ok());
}

#[test]
fn a_prompt_larger_than_the_argv_cap_is_refused_at_prepare_not_truncated() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let mut request = start_request(dir.path());
    request.context = "x".repeat(200_000);
    let BackendError::Failed { detail, .. } = backend.prepare(&request).expect_err("refused")
    else {
        panic!("a Failed refusal")
    };
    assert!(detail.contains("argv cap"), "{detail}");
    assert!(
        detail.contains("131072"),
        "names the measured E2BIG boundary: {detail}"
    );
    assert!(detail.contains("Nothing is truncated"), "{detail}");
    // And no process was ever spawned to learn this.
    assert!(stub.turn_launches().is_empty());
}

#[test]
fn a_profile_config_home_is_refused_not_ignored() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let mut request = start_request(dir.path());
    request.profile = Some(Profile {
        name: "p".into(),
        backend: AGY_BACKEND_NAME.into(),
        executable: None,
        config_home: Some(dir.path().join("home")),
        env: BTreeMap::new(),
        default_model: None,
        options: BTreeMap::new(),
    });
    let BackendError::Failed { detail, .. } = backend.prepare(&request).expect_err("refused")
    else {
        panic!("a Failed refusal")
    };
    assert!(detail.contains("config_home is not supported"), "{detail}");
    // The refusal names the measured alternative rather than leaving the
    // operator to guess one.
    assert!(detail.contains("settings_home"), "{detail}");
}

// ------------------------------------------------------------------ launch

#[test]
fn launch_adopts_the_conversation_id_from_the_init_line() {
    let dir = TempDir::new().expect("tempdir");
    let (stub, backend, handle, _) = launched(&dir, MINIMAL_TURN);
    assert_eq!(handle.native_id.as_deref(), Some(MINIMAL_TURN_CONVERSATION));
    let launches = stub.wait_for_turn_launches(1);
    assert_eq!(launches.len(), 1);
    // Turn 1 composes no `--conversation`: the id is harness-minted.
    assert!(!launches[0].has("--conversation"));
    assert_eq!(launches[0].cwd, dir.path().to_string_lossy());
    let _ = backend;
}

#[test]
fn every_turn_composes_disable_slash_commands() {
    // §12: procedure is data. Letting the CLI expand a `/skill` token inside a
    // carried CONTEXT.md would be the harness interpreting sergeant's data —
    // and it closes W1 P0.5's hazard, where a prompt answered as a CLI command
    // returns an empty-SUCCESS terminal with a `command` object.
    let dir = TempDir::new().expect("tempdir");
    let (stub, backend, handle, _) = launched(&dir, MINIMAL_TURN);
    wait_for_settled(&backend, &handle);
    backend.send(&handle, "and again").expect("send");
    let launches = stub.wait_for_turn_launches(2);
    for launch in &launches {
        assert!(
            launch.has("--disable-slash-commands"),
            "every turn, not just the first: {:?}",
            launch.argv
        );
        assert!(
            !launch.has("--dangerously-skip-permissions"),
            "the blanket flag is never a default (claude #47)"
        );
    }
}

#[test]
fn the_launch_prompt_carries_all_five_sections_in_order_as_the_value_of_p() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let mut request = pinned_request(dir.path());
    request.intent = "THE-INTENT".into();
    request.context = "THE-CONTEXT".into();
    request.bindings = vec![BindingSummary {
        repository: "repo".into(),
        worktree_path: dir.path().join("repo"),
        work_branch: "work/x".into(),
        base_branch: Some("main".into()),
        base_sha: "abc123".into(),
    }];
    launch_with(&backend, &request).expect("launch");
    let launches = stub.wait_for_turn_launches(1);
    // The prompt is the VALUE of `-p` — argv[0] is `-p`, argv[1] the prompt.
    assert_eq!(launches[0].argv[0], "-p");
    let prompt = launches[0].prompt();
    let exec = prompt.find("Execution model:").expect("execution model");
    let env = prompt.find("Environment:").expect("environment");
    let surface = prompt.find("Mutation surface:").expect("mutation surface");
    let intent = prompt.find("THE-INTENT").expect("intent");
    let context = prompt.find("THE-CONTEXT").expect("context");
    assert!(exec < env && env < surface && surface < intent && intent < context);
    // agy's own execution model, not a sibling's: a denied tool cancels the
    // whole turn (W1 P2), which is not what opencode's auto-reject does.
    assert!(prompt.contains("agy --print"));
    assert!(prompt.contains("the whole turn is cancelled"));
}

#[test]
fn the_mutation_surface_section_is_omitted_when_there_are_no_bindings() {
    let dir = TempDir::new().expect("tempdir");
    let (stub, _backend, _handle, _) = launched(&dir, MINIMAL_TURN);
    let launches = stub.wait_for_turn_launches(1);
    assert!(!launches[0].prompt().contains("Mutation surface:"));
}

#[test]
fn a_profile_executable_and_env_reach_every_turn() {
    // The `profiles` admission row's test: the generic sergeant axes only.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::new(dir.path(), "profiled", PASSING_VERSION, ALL_HELP);
    stub.replays(MINIMAL_TURN);
    // The backend's own executable is a *different*, non-existent path, so a
    // turn can only run if the profile's executable was the one used.
    let mut config = config_for(&stub, dir.path());
    config.executable = dir.path().join("never-this-one");
    let backend = AgyBackend::new(config);
    let (event_sink, _events) = sink();
    backend.set_event_sink(event_sink);
    let mut request = pinned_request(dir.path());
    request.profile = Some(Profile {
        name: "p".into(),
        backend: AGY_BACKEND_NAME.into(),
        executable: Some(stub.path.clone()),
        config_home: None,
        env: BTreeMap::from([("PROBE_PROFILE_VAR".into(), "reached".into())]),
        default_model: None,
        options: BTreeMap::new(),
    });
    // PREPARE probes with the backend's own (missing) executable, so drive
    // LAUNCH directly off a prepared execution built from the profile's.
    let prepared = sergeant_rs::backend::PreparedExecution {
        execution_id: request.execution_id.clone(),
        native_id: None,
        request: request.clone(),
    };
    let handle = backend.launch(&prepared).expect("launch under the profile");
    wait_for_settled(&backend, &handle);
    backend.send(&handle, "turn two").expect("send");
    let launches = stub.wait_for_turn_launches(2);
    for launch in &launches {
        assert_eq!(
            launch.env.get("PROBE_PROFILE_VAR").map(String::as_str),
            Some("reached"),
            "the profile's env must reach EVERY turn, not only the first"
        );
    }
}

#[test]
fn a_permission_config_reaches_every_turn_without_dirtying_the_work_diff() {
    // The `config_injection` admission row's test (panel ladder rung (a)).
    // The measured channel is a settings HOME: agy reads its permissions from
    // $HOME/.gemini/antigravity-cli/settings.json and $HOME is per-process
    // (W1 P2). W1 wires the mechanism and synthesizes no policy.
    let dir = TempDir::new().expect("tempdir");
    let settings_home = dir.path().join("agy-settings-home");
    std::fs::create_dir_all(settings_home.join(".gemini/antigravity-cli")).expect("mkdir");
    std::fs::write(
        settings_home.join(".gemini/antigravity-cli/settings.json"),
        r#"{"permissions":{"allow":["command(echo)"]}}"#,
    )
    .expect("write settings");
    let stub = StubAgy::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    let mut config = config_for(&stub, dir.path());
    config.settings_home = Some(settings_home.clone());
    let backend = AgyBackend::new(config);
    let (event_sink, _events) = sink();
    backend.set_event_sink(event_sink);
    let work_surface = dir.path().join("work-surface");
    std::fs::create_dir_all(&work_surface).expect("mkdir");
    let handle = launch_with(&backend, &pinned_request(&work_surface)).expect("launch");
    wait_for_settled(&backend, &handle);
    backend.send(&handle, "turn two").expect("send");
    let launches = stub.wait_for_turn_launches(2);
    for launch in &launches {
        assert_eq!(
            launch.env.get("HOME").map(PathBuf::from),
            Some(settings_home.clone()),
            "the settings home must be composed on EVERY turn"
        );
        assert_eq!(launch.cwd, work_surface.to_string_lossy());
    }
    // And nothing was written into the Work's own diff surface.
    let entries: Vec<_> = std::fs::read_dir(&work_surface)
        .expect("read work surface")
        .filter_map(Result::ok)
        .map(|e| e.file_name())
        .collect();
    assert!(
        entries.is_empty(),
        "the Work's surface stays clean: {entries:?}"
    );
}

#[test]
fn the_json_schema_channel_is_composed_only_when_configured() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.replays(JSON_SCHEMA);
    let mut config = config_for(&stub, dir.path());
    config.json_schema = Some(r#"{"type":"object"}"#.into());
    let backend = AgyBackend::new(config);
    let (event_sink, events) = sink();
    backend.set_event_sink(event_sink);
    let handle = launch_with(&backend, &pinned_request(dir.path())).expect("launch");
    let launches = stub.wait_for_turn_launches(1);
    assert_eq!(
        launches[0].value_after("--json-schema"),
        Some(r#"{"type":"object"}"#)
    );
    wait_for_settled(&backend, &handle);
    // W1 wires the CHANNEL and synthesizes no schema; the validated object
    // reaches the journal beside the prose response, never instead of it.
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    assert_eq!(
        ended.payload["structured_output"],
        serde_json::json!({"word": "pong"})
    );
}

#[test]
fn launch_fails_closed_when_no_init_line_arrives() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    // Replays nothing and then hangs: the process is alive, and no `init` line
    // will ever arrive.
    stub.hangs_after_replay();
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let BackendError::Failed { detail, .. } =
        launch_with(&backend, &start_request(dir.path())).expect_err("refused")
    else {
        panic!("a Failed refusal")
    };
    assert!(detail.contains("no `init` line"), "{detail}");
    assert!(detail.contains("the turn was killed"), "{detail}");
    // Every error path removes the execution, so a later OBSERVE of the
    // reserved id is an honest UnknownExecution rather than a context nothing
    // created.
    assert!(backend.tracked_executions().is_empty());
}

#[test]
fn launch_fails_closed_when_the_process_exits_before_any_init() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.writes_stderr("agy: something went wrong before anything was minted\n");
    stub.exits_with(3);
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let BackendError::Failed { detail, .. } =
        launch_with(&backend, &start_request(dir.path())).expect_err("refused")
    else {
        panic!("a Failed refusal")
    };
    assert!(detail.contains("exit_code=Some(3)"), "{detail}");
    assert!(
        detail.contains("something went wrong"),
        "the stderr travels: {detail}"
    );
    assert!(
        detail.contains("no conversation was ever minted"),
        "{detail}"
    );
    assert!(backend.tracked_executions().is_empty());
}

#[test]
fn an_invalid_model_refusal_becomes_a_typed_launch_error_carrying_the_catalog() {
    // W1 P0.3 row A, zero-quota: agy refuses a bad pin BEFORE minting identity,
    // and its error enumerates the whole model catalog. That is strictly better
    // evidence than a bare exit code, so it is carried verbatim.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.replays(INVALID_MODEL);
    stub.exits_with(1);
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let mut request = start_request(dir.path());
    request.model = Some("not-a-real-model".to_string());
    let BackendError::Failed { detail, .. } = launch_with(&backend, &request).expect_err("refused")
    else {
        panic!("a Failed refusal")
    };
    assert!(detail.contains("before minting a conversation"), "{detail}");
    assert!(detail.contains("status=ERROR"), "{detail}");
    assert!(
        detail.contains("Gemini 3.7 Flash (Low)"),
        "the catalog travels: {detail}"
    );
    assert!(backend.tracked_executions().is_empty());
}

#[test]
fn a_substituted_model_refuses_the_launch() {
    // The R4 delta cashed in: `init` precedes any model output, so this is the
    // earliest possible moment and the fewest possible tokens. The turn that
    // would have succeeded is not the turn the human asked for.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let mut request = start_request(dir.path());
    request.model = Some("gemini-3.1-pro-high".to_string());
    let BackendError::Failed { detail, .. } = launch_with(&backend, &request).expect_err("refused")
    else {
        panic!("a Failed refusal")
    };
    assert!(detail.contains(FIXTURE_MODEL), "names what ran: {detail}");
    assert!(
        detail.contains("gemini-3.1-pro-high"),
        "names what was asked for: {detail}"
    );
    assert!(detail.contains("precedes any model output"), "{detail}");
    assert!(backend.tracked_executions().is_empty());
}

#[test]
fn an_unpinned_launch_records_the_verdict_as_unpinned() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let (event_sink, events) = sink();
    backend.set_event_sink(event_sink);
    let handle = launch_with(&backend, &start_request(dir.path())).expect("launch");
    wait_for_settled(&backend, &handle);
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    assert_eq!(
        ended.payload["model_pin"]["verdict"],
        serde_json::json!("unpinned")
    );
}

// ------------------------------------------------- launch-time honesty notices

#[test]
fn a_denying_permission_mode_is_reported_at_launch_not_mid_turn() {
    // The panel amendment's rung-(b) honesty check, shipped regardless of the
    // rung-(a) outcome. LAUNCH still returns a handle: a read-only stage runs
    // fine under a denying mode, and refusing would break Works that never
    // touch a tool. The amendment asks for *reported honestly at launch*, and
    // an emitted, journaled, probe-visible fact is exactly that.
    let dir = TempDir::new().expect("tempdir");
    let (_stub, backend, handle, events) = launched(&dir, MINIMAL_TURN);
    let notice = events_of_kind(&events, "conversation.turn.harness_error")
        .into_iter()
        .find(|e| e.payload["phase"] == serde_json::json!("permission_mode_denies_tools"))
        .expect("a launch-time permission notice");
    assert_eq!(
        notice.payload["effective_mode"],
        serde_json::json!("request-review")
    );
    assert!(
        notice.payload["detail"]
            .as_str()
            .expect("detail")
            .contains("CANCELS the whole turn"),
        "the notice states the MEASURED consequence, not the packet's stale one"
    );
    assert!(
        notice.payload["injection"]
            .as_str()
            .expect("injection")
            .contains("operator config required")
    );
    // And it is a launch-time fact, not something discovered mid-run: the
    // handle came back and the posture is in the turn's own record.
    assert!(handle.native_id.is_some());
    wait_for_settled(&backend, &handle);
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    assert_eq!(
        ended.payload["permission_posture"]["denies_tools"],
        serde_json::json!(true)
    );
    assert_eq!(
        ended.payload["init"]["permission_mode"],
        serde_json::json!("request-review")
    );
}

#[test]
fn a_cwd_outside_the_trusted_workspaces_is_reported_at_launch() {
    // W1 P3, the hazard the wave's own spec did not anticipate: a write from an
    // untrusted cwd silently landed in the CLI's scratch directory while the
    // turn reported SUCCESS, with nothing on stderr or in the NDJSON saying so.
    // Read from the same zero-quota /config the posture comes from.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    stub.answers_config(
        r#"{"conversation_id":"","status":"SUCCESS","num_turns":0,"usage":{"total_tokens":0},
            "command":{"name":"config","data":{"config":{"toolPermission":"request-review",
            "permissions":null,"allowNonWorkspaceAccess":false,
            "trustedWorkspaces":["/somewhere/else"]}}}}"#,
    );
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let (event_sink, events) = sink();
    backend.set_event_sink(event_sink);
    launch_with(&backend, &pinned_request(dir.path())).expect("launch");
    let notice = events_of_kind(&events, "conversation.turn.harness_error")
        .into_iter()
        .find(|e| e.payload["phase"] == serde_json::json!("cwd_outside_trusted_workspaces"))
        .expect("a trusted-workspace notice");
    assert_eq!(
        notice.payload["trusted_workspaces"],
        serde_json::json!(["/somewhere/else"])
    );
    assert!(
        notice.payload["detail"]
            .as_str()
            .expect("detail")
            .contains("scratch directory")
    );
}

#[test]
fn a_cwd_inside_a_trusted_workspace_raises_no_notice() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    stub.answers_config(&format!(
        r#"{{"command":{{"name":"config","data":{{"config":{{"toolPermission":"request-review",
            "permissions":null,"allowNonWorkspaceAccess":false,
            "trustedWorkspaces":["{}"]}}}}}}}}"#,
        dir.path().display()
    ));
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let (event_sink, events) = sink();
    backend.set_event_sink(event_sink);
    launch_with(&backend, &pinned_request(dir.path())).expect("launch");
    wait_for_kind(&events, "conversation.turn.ended");
    assert!(
        !events_of_kind(&events, "conversation.turn.harness_error")
            .iter()
            .any(|e| e.payload["phase"] == serde_json::json!("cwd_outside_trusted_workspaces")),
        "a trusted surface must not be warned about"
    );
}

// ---------------------------------------------------------------- streaming

#[test]
fn events_are_delivered_before_the_turn_process_exits() {
    // The `streaming` admission row's test: the stub replays and then HANGS, so
    // the assertion is that normalized events already landed while the process
    // is still alive.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    stub.hangs_after_replay();
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let (event_sink, events) = sink();
    backend.set_event_sink(event_sink);
    let handle = launch_with(&backend, &pinned_request(dir.path())).expect("launch");
    let assistant = wait_for_kind(&events, "conversation.assistant.completed");
    assert_eq!(assistant.payload["text"], serde_json::json!("pong\n"));
    assert_eq!(
        backend.observe(&handle).expect("observe").native,
        NativeState::Running,
        "the turn process is still alive while its events have already been delivered"
    );
    backend.stop(&handle).expect("stop").wait();
}

#[test]
fn per_step_and_terminal_usage_become_usage_events() {
    let dir = TempDir::new().expect("tempdir");
    let (_stub, backend, handle, events) = launched(&dir, TOOL_USE);
    wait_for_settled(&backend, &handle);
    let usage = events_of_kind(&events, "usage.updated");
    assert!(
        usage.len() >= 2,
        "per-step AND terminal, never a synthetic sum"
    );
    assert_eq!(usage[0].payload["scope"], serde_json::json!("step"));
    let turn = usage.last().expect("a terminal usage event");
    assert_eq!(turn.payload["scope"], serde_json::json!("turn"));
    assert_eq!(
        turn.payload["usage"]["total_tokens"],
        serde_json::json!(27997)
    );
}

// ---------------------------------------------------------------- terminals

#[test]
fn a_completed_turn_reports_its_summary_and_its_raw_blob_ref() {
    let dir = TempDir::new().expect("tempdir");
    let (_stub, backend, handle, events) = launched(&dir, MINIMAL_TURN);
    let observation = wait_for_settled(&backend, &handle);
    assert_eq!(observation.native, NativeState::Exited);
    let BackendSignal::StageCompleted { summary } = observation.signal else {
        panic!("a completion: {observation:?}")
    };
    assert_eq!(summary.as_deref(), Some("pong\n"));
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    assert_eq!(ended.payload["outcome"], serde_json::json!("completed"));
    assert_eq!(ended.payload["status"], serde_json::json!("SUCCESS"));
    assert_eq!(
        ended.payload["model_pin"]["verdict"],
        serde_json::json!("honored")
    );
    assert!(
        ended.payload["raw"].is_string(),
        "the §20 archive ref reaches the journal"
    );
    assert_eq!(
        ended.payload["init"]["model"],
        serde_json::json!(FIXTURE_MODEL)
    );
    assert!(ended.payload["init"]["tool_count"].as_u64().expect("count") > 50);
}

#[test]
fn an_empty_success_terminal_is_ambiguous_not_completed() {
    // The panel's amendment, end to end through the real adapter.
    let dir = TempDir::new().expect("tempdir");
    let (_stub, backend, handle, events) = launched(&dir, EMPTY_SUCCESS);
    let observation = wait_for_settled(&backend, &handle);
    assert_eq!(
        observation.native,
        NativeState::Unknown,
        "§25's ambiguity blocks the Work rather than completing a stage on a stream that said \
         nothing"
    );
    assert!(matches!(observation.signal, BackendSignal::Running));
    let evidence = observation.evidence.expect("evidence");
    assert!(evidence.contains("agent_response_steps=0"), "{evidence}");
    assert!(
        evidence.contains("1.1.18"),
        "names the class this rule guards: {evidence}"
    );
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    assert_eq!(
        ended.payload["outcome"],
        serde_json::json!("ambiguous_unknown")
    );
}

#[test]
fn a_success_terminal_hiding_a_denied_tool_is_ambiguous_not_completed() {
    // §9.3 through the real adapter, with the detector that actually fires at
    // 1.1.19: the stderr notice. The fixture is the synthesized soft-deny
    // shape, and the *stderr* is the measured one.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.replays(SOFT_DENY_SUCCESS);
    stub.writes_stderr(DENIAL_NOTICE);
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let (event_sink, events) = sink();
    backend.set_event_sink(event_sink);
    let handle = launch_with(&backend, &pinned_request(dir.path())).expect("launch");
    let observation = wait_for_settled(&backend, &handle);
    assert_eq!(observation.native, NativeState::Unknown);
    let evidence = observation.evidence.expect("evidence");
    assert!(
        evidence.contains("auto-denied"),
        "the reason is named: {evidence}"
    );
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    assert_eq!(
        ended.payload["outcome"],
        serde_json::json!("ambiguous_unknown")
    );
    assert_eq!(
        ended.payload["stderr_denial_notice"],
        serde_json::json!(true)
    );
    assert_eq!(ended.payload["status"], serde_json::json!("SUCCESS"));
}

#[test]
fn a_denied_tool_call_is_a_cancelled_turn_not_a_hang() {
    // The `non_blocking_run` admission row's test, on the REAL 1.1.19 capture:
    // it resolves promptly with a CANCELED terminal and exit 0, and fails
    // closed rather than completing.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.replays(DENIED_CANCELED);
    stub.writes_stderr(DENIAL_NOTICE);
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let (event_sink, events) = sink();
    backend.set_event_sink(event_sink);
    let handle = launch_with(&backend, &pinned_request(dir.path())).expect("launch");
    let started = Instant::now();
    let observation = wait_for_settled(&backend, &handle);
    assert!(
        started.elapsed() < Duration::from_secs(20),
        "it never hangs"
    );
    assert_eq!(observation.native, NativeState::Unknown);
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    assert_eq!(ended.payload["status"], serde_json::json!("CANCELED"));
    assert_eq!(
        ended.payload["outcome"],
        serde_json::json!("ambiguous_unknown")
    );
    assert_eq!(ended.payload["exit_code"], serde_json::json!(0));
    assert_eq!(
        ended.payload["stderr_denial_notice"],
        serde_json::json!(true)
    );
    // The tool events are still produced: the step resolved DONE with no
    // output, which is the measured shape.
    let completed = events_of_kind(&events, "tool.completed");
    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].payload["state"], serde_json::json!("DONE"));
    assert_eq!(completed[0].payload["has_output"], serde_json::json!(false));
}

#[test]
fn an_unrequested_cancel_is_ambiguous_not_an_interrupt() {
    // Arm 6: treating an unrequested cancel as our own interrupt would be the
    // adapter claiming authorship of an event it did not cause.
    let dir = TempDir::new().expect("tempdir");
    let (_stub, backend, handle, events) = launched(&dir, DENIED_CANCELED);
    wait_for_settled(&backend, &handle);
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    assert_eq!(ended.payload["interrupted"], serde_json::json!(false));
    assert_eq!(
        ended.payload["outcome"],
        serde_json::json!("ambiguous_unknown")
    );
}

#[test]
fn an_unknown_terminal_status_is_ambiguous_with_the_status_echoed() {
    let dir = TempDir::new().expect("tempdir");
    let stream = format!(
        "{{\"event\":\"init\",\"conversation_id\":\"c-1\",\"init\":{{\"model\":\"{FIXTURE_MODEL}\",\
         \"permission_mode\":\"request-review\",\"tools\":[]}}}}\n\
         {{\"event\":\"result\",\"result\":{{\"conversation_id\":\"c-1\",\"status\":\"WAITING\",\
         \"response\":\"words\",\"num_turns\":1}}}}\n"
    );
    let (_stub, backend, handle, events) = launched(&dir, &stream);
    let observation = wait_for_settled(&backend, &handle);
    assert_eq!(observation.native, NativeState::Unknown);
    assert!(
        observation.evidence.expect("evidence").contains("WAITING"),
        "the literal status is echoed so a human can act on it"
    );
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    assert_eq!(ended.payload["status"], serde_json::json!("WAITING"));
}

#[test]
fn a_slash_command_result_never_reads_as_a_completed_turn() {
    // Defence in depth: `--disable-slash-commands` means this cannot happen,
    // and if it ever did the empty-SUCCESS rule catches it and
    // `saw_command_result` names why.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.replays(SLASH_COMMAND);
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let (event_sink, _events) = sink();
    backend.set_event_sink(event_sink);
    // A `command_result` stream carries `conversation_id: ""`, which is not an
    // identity — so the launch refuses rather than handing back a handle
    // naming nothing.
    let error = launch_with(&backend, &pinned_request(dir.path())).expect_err("refused");
    let BackendError::Failed { detail, .. } = error else {
        panic!("a Failed refusal")
    };
    assert!(detail.contains("before minting a conversation"), "{detail}");
    assert!(backend.tracked_executions().is_empty());
}

#[test]
fn an_ambiguous_turn_still_journals_its_raw_blob_ref() {
    // The only place the §20 blob ref reaches the journal for a turn with no
    // terminal at all is `conversation.turn.ended` — so it must be emitted
    // however the turn ended.
    let dir = TempDir::new().expect("tempdir");
    let (_stub, backend, handle, events) = launched(&dir, SIGKILL_TRUNCATED);
    let observation = wait_for_settled(&backend, &handle);
    assert_eq!(observation.native, NativeState::Unknown);
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    assert_eq!(
        ended.payload["outcome"],
        serde_json::json!("ambiguous_unknown")
    );
    assert_eq!(ended.payload["status"], Value::Null);
    let blob = ended.payload["raw"].as_str().expect("a blob ref");
    assert!(!blob.is_empty());
    let mut refs = std::collections::BTreeSet::new();
    sergeant_rs::runtime::blob::refs_in_payload(&ended.payload, &mut refs);
    assert_eq!(
        refs.len(),
        1,
        "A4: the archived ref must be recoverable from the emitted payload"
    );
}

#[test]
fn pty_carriage_returns_are_normalized_in_events_but_not_in_the_raw_blob() {
    let dir = TempDir::new().expect("tempdir");
    let (_stub, backend, handle, events) = launched(&dir, TOOL_USE);
    wait_for_settled(&backend, &handle);
    let completed = events_of_kind(&events, "tool.completed");
    assert_eq!(
        completed[0].payload["output_tail"],
        serde_json::json!("agy-w1-probe\n"),
        "events carry LF"
    );
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    let blob = ended.payload["raw"].as_str().expect("a blob ref");
    let hex = blob.rsplit(':').next().expect("hex");
    let bytes = std::fs::read(dir.path().join("blobs").join("b3").join(hex)).expect("read blob");
    assert!(
        String::from_utf8_lossy(&bytes).contains("agy-w1-probe\\r\\n"),
        "the archive keeps the bytes the harness wrote — a normalized archive is an archive that \
         has already been interpreted"
    );
}

#[test]
fn no_tool_event_is_ever_produced_from_narration() {
    let dir = TempDir::new().expect("tempdir");
    let narration = format!(
        "{{\"event\":\"init\",\"conversation_id\":\"c-1\",\"init\":{{\"model\":\"{FIXTURE_MODEL}\",\
         \"permission_mode\":\"request-review\",\"tools\":[]}}}}\n\
         {{\"event\":\"step_update\",\"step_update\":{{\"conversation_id\":\"c-1\",\
         \"step_index\":0,\"state\":\"DONE\",\"step_type\":\"agent_response\",\
         \"text_delta\":\"I ran run_command and its tool_info said DONE with output agy-w1-probe\"}}}}\n\
         {{\"event\":\"result\",\"result\":{{\"conversation_id\":\"c-1\",\"status\":\"SUCCESS\",\
         \"response\":\"done\",\"num_turns\":1,\"usage\":{{\"total_tokens\":1}}}}}}\n"
    );
    let (_stub, backend, handle, events) = launched(&dir, &narration);
    wait_for_settled(&backend, &handle);
    wait_for_kind(&events, "conversation.turn.ended");
    assert!(
        events
            .lock()
            .expect("lock")
            .iter()
            .all(|event| !event.kind.starts_with("tool.")),
        "prose is never tool evidence — the narration rule is structural, not stylistic"
    );
}

// -------------------------------------------------------------- send/resume

#[test]
fn a_resume_turn_names_the_conversation_and_keeps_the_pin() {
    // The `persistent_sessions` admission row's test: the conversation survives
    // past turn 1's process and is reused, unprompted, on turn 2's separately
    // spawned one.
    let dir = TempDir::new().expect("tempdir");
    let (stub, backend, handle, _) = launched(&dir, MINIMAL_TURN);
    wait_for_settled(&backend, &handle);
    backend.send(&handle, "turn two").expect("send");
    let launches = stub.wait_for_turn_launches(2);
    assert!(!launches[0].has("--conversation"));
    assert_eq!(
        launches[1].value_after("--conversation"),
        Some(MINIMAL_TURN_CONVERSATION)
    );
    // A pin the human asked for must not silently lapse after turn 1.
    assert_eq!(launches[1].value_after("--model"), Some(FIXTURE_MODEL));
    assert_eq!(launches[1].prompt(), "turn two");
}

#[test]
fn a_resumed_turn_whose_init_echoes_a_different_conversation_fails_the_turn() {
    // W1 P0.6's silent-resume fork: an unknown `--conversation` id does not
    // refuse — agy warns on stderr and starts a FRESH conversation. So a
    // resumed turn is only a resume if the init line echoes the id we asked for.
    let dir = TempDir::new().expect("tempdir");
    let (stub, backend, handle, events) = launched(&dir, MINIMAL_TURN);
    wait_for_settled(&backend, &handle);
    // Turn 2 replays a stream whose init names a DIFFERENT conversation.
    stub.replays(TOOL_USE);
    stub.writes_stderr(RESUME_FORK_WARNING);
    backend.send(&handle, "turn two").expect("send");
    let deadline = Instant::now() + Duration::from_secs(20);
    let observation = loop {
        let observation = backend.observe(&handle).expect("observe");
        if matches!(observation.signal, BackendSignal::Failed { .. }) {
            break observation;
        }
        assert!(
            Instant::now() < deadline,
            "the forked turn never failed: {observation:?}"
        );
        std::thread::sleep(Duration::from_millis(20));
    };
    let BackendSignal::Failed { reason } = observation.signal else {
        unreachable!()
    };
    assert!(
        reason.contains(MINIMAL_TURN_CONVERSATION),
        "names what we asked for: {reason}"
    );
    assert!(
        reason.contains(TOOL_USE_CONVERSATION),
        "names what came back: {reason}"
    );
    let mismatch = events_of_kind(&events, "conversation.turn.harness_error")
        .into_iter()
        .find(|e| e.payload["phase"] == serde_json::json!("resume_identity_mismatch"))
        .expect("a resume-identity mismatch event");
    assert_eq!(mismatch.payload["stderr_warning"], serde_json::json!(true));
}

#[test]
fn the_stderr_conversation_not_found_warning_reaches_the_turn_evidence() {
    // The second, independent detector of the same fact. It is stderr-only —
    // never in the NDJSON — so the stderr drain is what makes it visible.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    stub.writes_stderr(RESUME_FORK_WARNING);
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let (event_sink, events) = sink();
    backend.set_event_sink(event_sink);
    let handle = launch_with(&backend, &pinned_request(dir.path())).expect("launch");
    wait_for_settled(&backend, &handle);
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    assert!(
        ended.payload["stderr"]
            .as_str()
            .expect("stderr")
            .contains("not found"),
        "{:?}",
        ended.payload["stderr"]
    );
}

#[test]
fn send_refuses_a_second_turn_while_one_is_in_flight() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    stub.hangs_after_replay();
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let (event_sink, _events) = sink();
    backend.set_event_sink(event_sink);
    let handle = launch_with(&backend, &pinned_request(dir.path())).expect("launch");
    let BackendError::Failed { detail, .. } =
        backend.send(&handle, "meanwhile").expect_err("refused")
    else {
        panic!("a Failed refusal")
    };
    assert!(detail.contains("one turn at a time"), "{detail}");
    backend.stop(&handle).expect("stop").wait();
}

#[test]
fn a_stopped_execution_accepts_no_input() {
    let dir = TempDir::new().expect("tempdir");
    let (_stub, backend, handle, _) = launched(&dir, MINIMAL_TURN);
    wait_for_settled(&backend, &handle);
    backend.stop(&handle).expect("stop").wait();
    let BackendError::Failed { detail, .. } = backend.send(&handle, "more").expect_err("refused")
    else {
        panic!("a Failed refusal")
    };
    assert!(detail.contains("is stopped"), "{detail}");
}

#[test]
fn resume_refuses_a_handle_with_no_native_id() {
    // There is nothing to re-adopt, and inventing an id is the fabrication
    // `ResumeRequest`'s own contract forbids.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let handle = ExecutionHandle {
        execution_id: "e-1".into(),
        native_id: None,
    };
    assert!(matches!(
        backend.resume(&handle, &ResumeRequest::new("w", dir.path())),
        Err(BackendError::UnknownExecution { .. })
    ));
}

#[test]
fn resume_readopts_a_conversation_and_starts_no_turn() {
    // §15: re-adoption costs no tokens and creates no second execution. And the
    // observation is honest about how weak the claim is here — agy has no
    // token-free re-adoption check, so the durable-context check is deferred to
    // the first subsequent SEND.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.replays(MINIMAL_TURN);
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let (event_sink, _events) = sink();
    backend.set_event_sink(event_sink);
    let handle = ExecutionHandle {
        execution_id: "e-readopt".into(),
        native_id: Some(MINIMAL_TURN_CONVERSATION.to_string()),
    };
    let mut request = ResumeRequest::new("w", dir.path());
    request.model = Some(FIXTURE_MODEL.to_string());
    backend.resume(&handle, &request).expect("resume");
    assert!(
        stub.turn_launches().is_empty(),
        "RESUME never starts a turn"
    );
    let observation = backend.observe(&handle).expect("observe");
    assert_eq!(observation.native, NativeState::Unknown);
    let evidence = observation.evidence.expect("evidence");
    assert!(
        evidence.contains("re-adopted after a restart"),
        "{evidence}"
    );
    assert!(
        evidence.contains("no token-free re-adoption check"),
        "the weaker claim is stated, not papered over: {evidence}"
    );
    // And the next SEND composes the re-adopted id, keeping the re-supplied pin.
    backend.send(&handle, "after the restart").expect("send");
    let launches = stub.wait_for_turn_launches(1);
    assert_eq!(
        launches[0].value_after("--conversation"),
        Some(MINIMAL_TURN_CONVERSATION)
    );
    assert_eq!(launches[0].value_after("--model"), Some(FIXTURE_MODEL));
}

#[test]
fn resume_is_idempotent_and_refuses_a_different_identity() {
    let dir = TempDir::new().expect("tempdir");
    let (_stub, backend, handle, _) = launched(&dir, MINIMAL_TURN);
    wait_for_settled(&backend, &handle);
    backend
        .resume(&handle, &ResumeRequest::new("w-agy", dir.path()))
        .expect("re-adopting what we already own is a no-op");
    let wrong = ExecutionHandle {
        execution_id: handle.execution_id.clone(),
        native_id: Some("some-other-conversation".into()),
    };
    assert!(matches!(
        backend.resume(&wrong, &ResumeRequest::new("w-agy", dir.path())),
        Err(BackendError::UnknownExecution { .. })
    ));
}

// ------------------------------------------------------------------ history

#[test]
fn history_refuses_rather_than_returning_an_empty_list() {
    // §15's pairing: `history: false` and a refusal a caller can tell apart
    // from "this conversation said nothing".
    let dir = TempDir::new().expect("tempdir");
    let (_stub, backend, handle, _) = launched(&dir, MINIMAL_TURN);
    assert!(!backend.capabilities().history);
    let BackendError::Unsupported { verb, detail, .. } =
        backend.history(&handle).expect_err("refused")
    else {
        panic!("an Unsupported refusal")
    };
    assert_eq!(verb, "history");
    assert!(detail.contains("no export verb"), "{detail}");
    assert!(
        detail.contains("journal"),
        "names sergeant's own record: {detail}"
    );
}

// ---------------------------------------------------------------- interrupt

#[test]
fn agy_interrupt_kills_the_process_group() {
    // The `interrupt` admission row's test. opencode probe 11's lesson carried
    // without re-deriving it: a plain `child.kill()` would leave the
    // grandchild running, so INTERRUPT signals the whole group.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    // A stream with an `init` line and no terminal — the real SIGKILL shape
    // (W1 P4). Replaying a *completed* turn and then killing it would test
    // nothing about interruption: a turn that already said SUCCESS is
    // completed, and the classifier is right to say so.
    stub.replays(SIGKILL_TRUNCATED);
    stub.spawns_a_grandchild();
    stub.hangs_after_replay();
    let backend = AgyBackend::new(config_for(&stub, dir.path()));
    let (event_sink, events) = sink();
    backend.set_event_sink(event_sink);
    let handle = launch_with(&backend, &pinned_request(dir.path())).expect("launch");
    let grandchild = stub.wait_for_grandchild_pid();
    assert!(
        pid_alive(grandchild),
        "the grandchild must be running first"
    );
    backend.interrupt(&handle).expect("interrupt").wait();
    let deadline = Instant::now() + Duration::from_secs(10);
    while pid_alive(grandchild) {
        assert!(
            Instant::now() < deadline,
            "the grandchild survived the group kill (pid {grandchild})"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    // A kill we asked for is not a conclusion about the stage, and the
    // conversation stays resumable.
    let observation = backend.observe(&handle).expect("observe");
    assert!(matches!(observation.signal, BackendSignal::Running));
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    assert_eq!(ended.payload["interrupted"], serde_json::json!(true));
    assert_eq!(
        ended.payload["outcome"],
        serde_json::json!("interrupted_running")
    );
}

#[test]
fn interrupting_an_execution_with_no_turn_in_flight_is_a_no_op() {
    // `mod.rs`'s own contract: the goal state — no turn running — already holds.
    let dir = TempDir::new().expect("tempdir");
    let (_stub, backend, handle, _) = launched(&dir, MINIMAL_TURN);
    wait_for_settled(&backend, &handle);
    backend
        .interrupt(&handle)
        .expect("a no-op, not an error")
        .wait();
    backend.interrupt(&handle).expect("still a no-op").wait();
}

#[test]
fn observe_refuses_an_identity_this_adapter_never_minted() {
    let dir = TempDir::new().expect("tempdir");
    let (_stub, backend, handle, _) = launched(&dir, MINIMAL_TURN);
    let wrong = ExecutionHandle {
        execution_id: handle.execution_id.clone(),
        native_id: Some("not-the-conversation".into()),
    };
    assert!(matches!(
        backend.observe(&wrong),
        Err(BackendError::UnknownExecution { .. })
    ));
    assert!(matches!(
        backend.observe(&ExecutionHandle {
            execution_id: "never-existed".into(),
            native_id: None,
        }),
        Err(BackendError::UnknownExecution { .. })
    ));
}

// ------------------------------------------ W3: the input-loop transport

#[test]
fn an_auto_resolution_picks_the_loop_when_help_offers_input_format() {
    // §2.8's resolution, and the whole of what `Auto` costs: a substring test
    // on a `--help` text the probe already read.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let backend = AgyBackend::new(loop_config_for(&stub, dir.path()));
    let report = backend.probe();
    assert!(report.available);
    let detail = report.detail.expect("detail");
    assert!(
        detail.contains("transport: input-loop-stream-json (Auto: --help offers --input-format)"),
        "{detail}"
    );
    // The capability set follows the RESOLVED transport, and the one boolean
    // W3's evidence moved is visible in it.
    assert!(backend.capabilities().native_subagents);
    assert!(!backend.capabilities().ask);
    assert!(!backend.capabilities().approval_flow);
}

#[test]
fn an_auto_resolution_falls_back_to_print_when_input_format_is_absent() {
    // A RESOLUTION, not a downgrade: it happens once, at probe time, before any
    // execution exists — and an older agy stays fully usable rather than being
    // refused for a flag the print grammar never needed.
    let dir = TempDir::new().expect("tempdir");
    let help = ALL_HELP.replace(" --input-format", "");
    let stub = StubAgy::new(dir.path(), "agy", PASSING_VERSION, &help);
    let mut config = config_for(&stub, dir.path());
    config.transport = TransportChoice::Auto;
    let backend = AgyBackend::new(config);
    let report = backend.probe();
    assert!(report.available, "the build is still perfectly usable");
    let detail = report.detail.expect("detail");
    assert!(
        detail.contains("transport: print-stream-json (Auto: --input-format absent from --help)"),
        "{detail}"
    );
    assert!(detail.contains("RESOLUTION, not a downgrade"), "{detail}");
    assert!(
        !backend.capabilities().native_subagents,
        "the print column's honest claim, not the loop's"
    );
}

#[test]
fn a_loop_only_choice_refuses_a_build_that_cannot_serve_it() {
    // codex §5.2 rule 2, opencode's `ServeOnly` verbatim: serving a pinned
    // transport on the other one would serve a different set of measured claims
    // than the one that was asked for.
    let dir = TempDir::new().expect("tempdir");
    let help = ALL_HELP.replace(" --input-format", "");
    let stub = StubAgy::new(dir.path(), "agy", PASSING_VERSION, &help);
    let mut config = config_for(&stub, dir.path());
    config.transport = TransportChoice::LoopOnly;
    let backend = AgyBackend::new(config);
    let report = backend.probe();
    assert!(!report.available);
    let detail = report.detail.expect("detail");
    assert!(detail.contains("--input-format"), "{detail}");
    // And PREPARE refuses too, rather than launching something unmeasured.
    assert!(matches!(
        backend.prepare(&start_request(dir.path())),
        Err(BackendError::Unavailable { .. })
    ));
}

#[test]
fn resolving_capabilities_spawns_no_extra_process() {
    // **The 0.2.2 daemon-panic lesson (c46152a2), applied by construction.**
    // opencode's `Auto` had to spawn a serve child and build a blocking HTTP
    // client to resolve, which is why registration could panic; agy's needs no
    // process, no port and no client. Asserted against the stub's own launch
    // record: after `capabilities()`, the recorded launches are exactly the
    // probe's — and `capabilities()` is called straight from `daemon::start_with`.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let backend = AgyBackend::new(loop_config_for(&stub, dir.path()));
    let before = stub.launches().len();
    for _ in 0..5 {
        let _ = backend.capabilities();
    }
    assert_eq!(
        stub.launches().len(),
        before,
        "capabilities() must perform no I/O the probe was not already doing"
    );
    assert!(
        stub.loop_launches().is_empty(),
        "resolving the transport must not spawn a loop child"
    );
}

#[test]
fn a_loop_launch_composes_print_equals_and_both_stream_formats() {
    let dir = TempDir::new().expect("tempdir");
    let (stub, _backend, _handle, _events) = loop_launched(&dir, LOOP_TWO_TURNS);
    let launches = stub.wait_for_loop_launches(1);
    assert_eq!(launches.len(), 1, "ONE child for the whole execution");
    let argv = &launches[0].argv;
    assert_eq!(argv[0], "--print=");
    assert!(
        !argv.iter().any(|arg| arg == "-p"),
        "a bare -p swallows the next flag and fails rc=2 with no NDJSON at all (W3 P0)"
    );
    assert_eq!(
        launches[0].value_after("--input-format"),
        Some("stream-json")
    );
    assert_eq!(
        launches[0].value_after("--output-format"),
        Some("stream-json"),
        "--input-format REQUIRES --output-format stream-json; composed together or not at all"
    );
    assert!(launches[0].has("--disable-slash-commands"));
    assert_eq!(launches[0].value_after("--model"), Some(FIXTURE_MODEL));
    assert!(!launches[0].has("--sandbox"));
    assert!(!launches[0].has("--add-dir"));
    // The prompt is nowhere on argv — it went down stdin, which is the whole
    // point of this transport.
    assert!(
        !argv.iter().any(|arg| arg.contains("do the agy thing")),
        "the prompt must not ride argv on this transport: {argv:?}"
    );
}

#[test]
fn a_loop_launch_learns_identity_and_posture_before_any_message_is_written() {
    // **The transport's real prize.** `init` arrives at child start, so LAUNCH
    // knows the conversation, the resolved model and the effective permission
    // mode — and emits the posture notice — with ZERO quota spent. The stub's
    // own record proves the ordering: the identity landed before the first
    // stdin line was ever handed over.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let (init, replay) = loop_capture(LOOP_TWO_TURNS);
    stub.loop_init(&init);
    stub.replays(&replay);
    let backend = AgyBackend::new(loop_config_for(&stub, dir.path()));
    let (sink, events) = sink();
    backend.set_event_sink(sink);
    let handle = launch_with(&backend, &loop_pinned_request(dir.path())).expect("launch");

    assert!(
        handle.native_id.is_some(),
        "identity is known at LAUNCH, before any turn"
    );
    // The rung-(b) honesty notice, emitted at launch rather than discovered as
    // a mid-run cancellation — and here, before turn 1 was even written.
    let notice = wait_for_kind(&events, "conversation.turn.harness_error");
    assert_eq!(
        notice.payload["phase"],
        serde_json::json!("permission_mode_denies_tools")
    );
    assert_eq!(
        notice.payload["effective_mode"],
        serde_json::json!("request-review")
    );
    let user = wait_for_kind(&events, "conversation.user");
    assert_eq!(
        user.payload["transport"],
        serde_json::json!("input-loop-stream-json")
    );
    // Exactly one line went down stdin for turn 1.
    assert_eq!(stub.wait_for_loop_stdin_lines(1).len(), 1);
}

#[test]
fn a_substituted_model_refuses_a_loop_launch_before_a_turn_is_spent() {
    // **What no other adapter in the registry can say.** Print mode must burn
    // turn 1 to learn which model served it; here the refusal happens before a
    // single byte of prompt is written.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let (init, replay) = loop_capture(LOOP_TWO_TURNS);
    stub.loop_init(&init);
    stub.replays(&replay);
    let backend = AgyBackend::new(loop_config_for(&stub, dir.path()));
    let mut request = start_request(dir.path());
    request.model = Some("gemini-3.7-pro".to_string());
    let error = launch_with(&backend, &request).expect_err("a substituted pin refuses the launch");
    let BackendError::Failed { detail, .. } = error else {
        panic!("a Failed refusal")
    };
    assert!(detail.contains("ZERO quota"), "{detail}");
    assert!(detail.contains(FIXTURE_MODEL), "{detail}");
    // Nothing was written down the child's stdin: no turn was spent.
    assert!(
        stub.loop_stdin_lines().is_empty(),
        "a refused launch must not have written a turn"
    );
    // And it leaves no phantom execution behind.
    assert!(backend.tracked_executions().is_empty());
}

#[test]
fn a_loop_launch_fails_closed_when_no_init_line_arrives() {
    // W1's rule verbatim: no identity, no handle, and the child is group-killed
    // rather than left running.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.loop_never_initializes();
    let backend = AgyBackend::new(loop_config_for(&stub, dir.path()));
    let error = launch_with(&backend, &loop_pinned_request(dir.path())).expect_err("fails closed");
    let BackendError::Failed { detail, .. } = error else {
        panic!("a Failed refusal")
    };
    assert!(detail.contains("no `init` line"), "{detail}");
    assert!(detail.contains("no turn has been spent"), "{detail}");
    assert!(backend.tracked_executions().is_empty());
}

#[test]
fn a_loop_child_that_exits_before_init_fails_launch_naming_its_exit_code() {
    // The OTHER no-init shape, and a genuinely different code path: the test
    // above never leaves `spawn_loop_child`'s `recv_timeout` expiry, because
    // its child hangs. A child that *exits* promptly having said nothing is
    // classified by `LoopReader` itself — `Terminal::None` => `ExitedWithoutInit`
    // — and the refusal names the exit code rather than a budget.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.loop_exits_before_init(7, "agy: could not reach the model service\n");
    let backend = AgyBackend::new(loop_config_for(&stub, dir.path()));
    let error = launch_with(&backend, &loop_pinned_request(dir.path())).expect_err("fails closed");
    let BackendError::Failed { detail, .. } = error else {
        panic!("a Failed refusal")
    };
    assert!(
        detail.contains("emitted no terminal either"),
        "the reader's own classification, not the LAUNCH-side budget expiry: {detail}"
    );
    assert!(detail.contains("exit_code=Some(7)"), "{detail}");
    assert!(
        detail.contains("could not reach the model service"),
        "the child's stderr is the operator's only clue when it streamed nothing: {detail}"
    );
    assert!(
        !detail.contains("no turn has been spent"),
        "a hang and a fast exit must not be reported as the same thing: {detail}"
    );
    // Same fail-closed contract as the hang: no handle, no phantom execution,
    // and nothing was ever written down the child's stdin.
    assert!(backend.tracked_executions().is_empty());
    assert!(stub.loop_stdin_lines().is_empty());
}

#[test]
fn a_loop_child_that_refuses_before_init_quotes_agys_own_error() {
    // A harness that answers with a typed terminal instead of an identity. The
    // refusal must carry agy's own `error` verbatim — the operator's only clue
    // — rather than the generic said-nothing message, and the pre-`init`
    // `result` must NOT be settled as a turn: nothing was ever written to this
    // child's stdin, so a `conversation.turn.ended` here would invent one.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let refusal = LOOP_CONTROL_REFUSAL
        .lines()
        .find(|line| line.contains(r#""event":"result""#))
        .expect("the capture's terminal result line");
    stub.loop_refuses_before_init(refusal, 1);
    let backend = AgyBackend::new(loop_config_for(&stub, dir.path()));
    let (sink, events) = sink();
    backend.set_event_sink(sink);
    let error = launch_with(&backend, &loop_pinned_request(dir.path())).expect_err("fails closed");
    let BackendError::Failed { detail, .. } = error else {
        panic!("a Failed refusal")
    };
    assert!(detail.contains("before minting a conversation"), "{detail}");
    assert!(detail.contains("status=ERROR"), "{detail}");
    assert!(
        detail.contains("is not supported yet"),
        "agy's own error, verbatim: {detail}"
    );
    assert!(backend.tracked_executions().is_empty());
    assert!(
        events_of_kind(&events, "conversation.turn.ended").is_empty(),
        "a result that precedes `init` is a refusal, not a turn — no turn was ever sent"
    );
}

#[test]
fn a_loop_send_writes_exactly_one_ndjson_user_message_per_turn() {
    let dir = TempDir::new().expect("tempdir");
    let (stub, backend, handle, _events) = loop_launched(&dir, LOOP_TWO_TURNS);
    wait_for_settled(&backend, &handle);
    backend.send(&handle, "turn two please").expect("send");
    let lines = stub.wait_for_loop_stdin_lines(2);
    assert_eq!(lines.len(), 2, "one line per turn, no more");
    // ONE child, not one per turn — that is the transport.
    assert_eq!(stub.loop_launches().len(), 1);
    for line in &lines {
        let value: serde_json::Value = serde_json::from_str(line).expect("valid NDJSON");
        assert_eq!(value["event"], serde_json::json!("user"));
        assert_eq!(value["message"]["role"], serde_json::json!("user"));
        assert!(value["message"]["content"].is_string());
    }
    assert!(
        lines[0].contains("do the agy thing"),
        "turn 1 is the launch prompt"
    );
    assert!(lines[1].contains("turn two please"));
}

#[test]
fn a_loop_child_streams_each_turns_events_before_the_next_is_written() {
    // Backs the loop `streaming` row. Two turns through ONE child, each
    // settling with its own summary and its own raw blob ref — the same decoder
    // as print, driven turn by turn.
    let dir = TempDir::new().expect("tempdir");
    let (_stub, backend, handle, events) = loop_launched(&dir, LOOP_TWO_TURNS);
    let first = wait_for_settled(&backend, &handle);
    let BackendSignal::StageCompleted { summary } = first.signal else {
        panic!("turn 1 completes: {first:?}")
    };
    assert_eq!(summary.as_deref(), Some("alpha\n"));
    backend.send(&handle, "and again").expect("send");
    let deadline = Instant::now() + Duration::from_secs(20);
    let second = loop {
        let observation = backend.observe(&handle).expect("observe");
        if let BackendSignal::StageCompleted {
            summary: Some(text),
        } = &observation.signal
            && text == "bravo\n"
        {
            break observation;
        }
        assert!(
            Instant::now() < deadline,
            "turn 2 never settled: {observation:?}"
        );
        std::thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(second.native, NativeState::Exited);

    let ended = wait_for_turns_ended(&events, 2);
    assert_eq!(
        ended.len(),
        2,
        "one turn.ended per turn, cut at each result"
    );
    for event in &ended {
        assert_eq!(
            event.payload["transport"],
            serde_json::json!("input-loop-stream-json")
        );
        assert_eq!(
            event.payload["conversation_id"],
            serde_json::json!(conversation_of(LOOP_TWO_TURNS)),
            "the conversation survives the turn boundary"
        );
        assert!(
            event.payload["raw"].is_string(),
            "every turn owes its own §20 blob"
        );
    }
    // §2.7: the `init` line is archived exactly ONCE, in turn 1's blob, and
    // every later turn points at it rather than re-archiving it.
    assert!(ended[0].payload["init_blob"].is_null());
    assert_eq!(ended[1].payload["init_blob"], ended[0].payload["raw"]);
    assert_ne!(ended[0].payload["raw"], ended[1].payload["raw"]);
}

#[test]
fn a_subagent_record_reaches_the_journal_with_its_child_conversation() {
    // The `native_subagents` row, end to end through the adapter rather than
    // only through the decoder: the typed child conversation id has to survive
    // all the way into the journal for a human to act on it.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let (init, replay) = loop_capture(LOOP_SUBAGENT);
    stub.loop_init(&init);
    stub.replays(&replay);
    let backend = AgyBackend::new(loop_config_for(&stub, dir.path()));
    let (sink, events) = sink();
    backend.set_event_sink(sink);
    let handle = launch_with(&backend, &loop_pinned_request(dir.path())).expect("launch");
    wait_for_settled(&backend, &handle);
    backend.send(&handle, "now invoke it").expect("send");

    let deadline = Instant::now() + Duration::from_secs(20);
    let completed = loop {
        let found: Vec<_> = events_of_kind(&events, "tool.completed")
            .into_iter()
            .filter(|event| event.payload["name"] == serde_json::json!("subagent:subagent"))
            .collect();
        if let Some(event) = found.into_iter().next() {
            break event;
        }
        assert!(
            Instant::now() < deadline,
            "no subagent tool.completed arrived"
        );
        std::thread::sleep(Duration::from_millis(20));
    };
    let child = completed.payload["subagent"][0].clone();
    let child_id = child["conversation_id"]
        .as_str()
        .expect("a child conversation id");
    assert_ne!(child_id, conversation_of(LOOP_SUBAGENT));
    assert_eq!(child["name"], serde_json::json!("echoer"));
    assert!(
        child["log_uri"]
            .as_str()
            .expect("a log uri")
            .starts_with("file://")
    );

    let ended = wait_for_turns_ended(&events, 2);
    let last = ended.last().expect("turn 2 ended");
    assert_eq!(
        last.payload["subagent_conversations"],
        serde_json::json!([child_id]),
        "the admission's own evidence is journaled, not merely noted in a ledger row"
    );
}

#[test]
fn a_denied_tool_on_the_loop_kills_the_child_and_the_next_send_is_refused() {
    // **W3 A2's shape, and §2.5's dead-transport path — routine on this
    // transport rather than exceptional.** The refusal must be ACTIONABLE: it
    // names the conversation, which a fresh child resumes perfectly.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let (init, replay) = loop_capture(LOOP_DENIED_TOOL);
    stub.loop_init(&init);
    stub.replays(&replay);
    // The measured behaviour: the child exits 1 the moment the denied turn ends.
    stub.loop_dies_after_turn(1, 1);
    let backend = AgyBackend::new(loop_config_for(&stub, dir.path()));
    let (sink, events) = sink();
    backend.set_event_sink(sink);
    let handle = launch_with(&backend, &loop_pinned_request(dir.path())).expect("launch");

    let settled = wait_for_settled(&backend, &handle);
    let BackendSignal::Failed { reason } = &settled.signal else {
        panic!("the harness said exactly what went wrong: {settled:?}")
    };
    assert!(
        reason.contains("user denied permission to run command"),
        "{reason}"
    );
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    assert_eq!(
        ended.payload["denied_tools"],
        serde_json::json!(["run_command"])
    );
    assert!(
        ended.payload["loop_input_rejected"].is_null(),
        "a denied tool is a stage-visible harness failure, never an adapter defect"
    );

    // The transport is gone; the next SEND is refused, and refused usefully.
    let error = backend
        .send(&handle, "are you still there")
        .expect_err("the transport is dead");
    let BackendError::Failed { detail, .. } = error else {
        panic!("a Failed refusal")
    };
    assert!(detail.contains("has exited"), "{detail}");
    assert!(
        detail.contains(&conversation_of(LOOP_DENIED_TOOL)),
        "the refusal must name the conversation that is still resumable: {detail}"
    );
    assert!(detail.contains("does not respawn"), "{detail}");
}

#[test]
fn a_child_that_dies_mid_turn_is_ambiguous_unless_we_asked_for_it() {
    // Arms 9/10, reached through the loop's own death path: the child stops
    // talking with no terminal for the in-flight turn. Failing closed is the
    // point — a process that merely stopped may not complete or fail a stage.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let (init, _replay) = loop_capture(LOOP_TWO_TURNS);
    stub.loop_init(&init);
    // A segment with steps but NO terminal, then the child dies.
    stub.replays(
        "{\"event\":\"step_update\",\"step_update\":{\"step_index\":0,\"state\":\"DONE\",\"step_type\":\"user_input\"}}\n",
    );
    stub.loop_dies_after_turn(1, 137);
    let backend = AgyBackend::new(loop_config_for(&stub, dir.path()));
    let (sink, events) = sink();
    backend.set_event_sink(sink);
    let handle = launch_with(&backend, &loop_pinned_request(dir.path())).expect("launch");

    let settled = wait_for_settled(&backend, &handle);
    assert_eq!(
        settled.native,
        NativeState::Unknown,
        "§25's ambiguity, failing closed"
    );
    assert!(matches!(settled.signal, BackendSignal::Running));
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    assert_eq!(
        ended.payload["outcome"],
        serde_json::json!("ambiguous_unknown")
    );
    assert_eq!(ended.payload["exit_code"], serde_json::json!(137));
    assert_eq!(ended.payload["interrupted"], serde_json::json!(false));
}

#[test]
fn stderr_between_two_turns_is_attributed_adjacent_and_labelled() {
    // §2.6, the transport's one genuinely new hazard. A line that lands in the
    // window BETWEEN turns cannot be placed inside one — so it is attached to a
    // turn anyway and the slice is labelled, because dropping it would make an
    // auto-denied tool invisible.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let (init, replay) = loop_capture(LOOP_TWO_TURNS);
    stub.loop_init(&init);
    stub.replays(&replay);
    // Inside turn 1; then in the gap before turn 2's segment.
    stub.loop_stderr_after_turn(1, "a line that belongs to turn one\n");
    stub.loop_stderr_before_turn(2, &format!("{DENIAL_NOTICE}\n"));
    let backend = AgyBackend::new(loop_config_for(&stub, dir.path()));
    let (sink, events) = sink();
    backend.set_event_sink(sink);
    let handle = launch_with(&backend, &loop_pinned_request(dir.path())).expect("launch");
    wait_for_settled(&backend, &handle);
    backend.send(&handle, "again").expect("send");

    let deadline = Instant::now() + Duration::from_secs(20);
    let ended = loop {
        let ended = events_of_kind(&events, "conversation.turn.ended");
        if ended.len() >= 2 {
            break ended;
        }
        assert!(Instant::now() < deadline, "turn 2 never ended");
        std::thread::sleep(Duration::from_millis(20));
    };
    assert!(
        ended[0].payload["stderr"]
            .as_str()
            .expect("turn 1 stderr")
            .contains("belongs to turn one"),
        "an in-turn line is attributed exactly: {:?}",
        ended[0].payload["stderr"]
    );
    assert_eq!(
        ended[0].payload["stderr_attribution"],
        serde_json::json!("exact")
    );
    // The between-turns line is NOT dropped: it reaches turn 2, labelled.
    assert_eq!(
        ended[1].payload["stderr_attribution"],
        serde_json::json!("adjacent"),
        "the classifier says out loud that it was not certain"
    );
    assert_eq!(
        ended[1].payload["stderr_denial_notice"],
        serde_json::json!(true),
        "the auto-denial notice survives attribution — the whole reason §2.6 fails toward noticing"
    );
}

#[test]
fn closing_stdin_lets_a_queued_turn_finish_then_exits() {
    // W3 P2: closing stdin does not cancel queued work — the turn runs to
    // completion and the child then exits 0 with no further event. STOP leans
    // on that, and only group-kills when the bounded wait expires.
    let dir = TempDir::new().expect("tempdir");
    let (stub, backend, handle, events) = loop_launched(&dir, LOOP_TWO_TURNS);
    wait_for_settled(&backend, &handle);
    backend.stop(&handle).expect("stop").wait();

    let ended = events_of_kind(&events, "conversation.turn.ended");
    assert_eq!(
        ended.len(),
        1,
        "the settled turn kept its own clean outcome"
    );
    assert_eq!(ended[0].payload["outcome"], serde_json::json!("completed"));
    // A stopped execution accepts no further input, and says so rather than
    // writing to a pipe nobody is reading.
    let error = backend.send(&handle, "one more").expect_err("stopped");
    let BackendError::Failed { detail, .. } = error else {
        panic!("a Failed refusal")
    };
    assert!(detail.contains("stopped"), "{detail}");
    assert_eq!(stub.loop_stdin_lines().len(), 1);
}

#[test]
fn loop_interrupt_group_kills_the_child_and_its_grandchild() {
    // The `interrupt` row's test on this transport. The tier is UNCHANGED
    // (ProcessTreeTermination) and deliberately so: W3 P4 refuted the native
    // upgrade — SIGINT is fatal and emits a mislabelled terminal — so a
    // SIGINT-first ladder would trade a measured guarantee for an ambiguity.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let (init, _replay) = loop_capture(LOOP_TWO_TURNS);
    stub.loop_init(&init);
    stub.spawns_a_grandchild();
    stub.loop_hangs_after_init();
    let backend = AgyBackend::new(loop_config_for(&stub, dir.path()));
    let (sink, events) = sink();
    backend.set_event_sink(sink);
    let handle = launch_with(&backend, &loop_pinned_request(dir.path())).expect("launch");
    let grandchild = stub.wait_for_grandchild_pid();
    assert!(
        pid_alive(grandchild),
        "the grandchild must be running first"
    );

    backend.interrupt(&handle).expect("interrupt").wait();
    let deadline = Instant::now() + Duration::from_secs(10);
    while pid_alive(grandchild) {
        assert!(
            Instant::now() < deadline,
            "the grandchild survived the group kill (pid {grandchild})"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    // A kill we asked for is not a conclusion about the stage, and W3 A7
    // measured the conversation staying fully resumable afterwards.
    let observation = backend.observe(&handle).expect("observe");
    assert!(matches!(observation.signal, BackendSignal::Running));
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    assert_eq!(ended.payload["interrupted"], serde_json::json!(true));
    assert_eq!(
        ended.payload["outcome"],
        serde_json::json!("interrupted_running")
    );
}

#[test]
fn a_prompt_over_the_loop_cap_is_refused_at_prepare_not_truncated() {
    // PREPARE refuses on the transport this execution will ACTUALLY launch on:
    // a prompt print mode cannot carry rides the loop fine, and one over the
    // loop's own cap is still refused rather than silently trimmed.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let backend = AgyBackend::new(loop_config_for(&stub, dir.path()));
    let mut request = loop_pinned_request(dir.path());
    // Comfortably past print's measured 120 000-byte argv cap.
    request.context = "x".repeat(200_000);
    backend
        .prepare(&request)
        .expect("the loop carries a prompt argv could never hold");
    // The print transport refuses the very same request, which is the delta.
    let print = AgyBackend::new(config_for(&stub, dir.path()));
    let error = print.prepare(&request).expect_err("argv cannot carry it");
    let BackendError::Failed { detail, .. } = error else {
        panic!("a Failed refusal")
    };
    assert!(detail.contains("argv cap"), "{detail}");
    assert!(detail.contains("Nothing is truncated"), "{detail}");
}

#[test]
fn a_profile_executable_and_env_reach_a_loop_child() {
    // W1's declared divergence, on the loop: the generic axes only, and env now
    // reaches ONE child rather than one per turn.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    let other = StubAgy::new(dir.path(), "other", PASSING_VERSION, ALL_HELP);
    let (init, replay) = loop_capture(LOOP_TWO_TURNS);
    other.loop_init(&init);
    other.replays(&replay);
    let backend = AgyBackend::new(loop_config_for(&stub, dir.path()));
    let mut request = loop_pinned_request(dir.path());
    request.profile = Some(Profile {
        name: "p".into(),
        backend: AGY_BACKEND_NAME.into(),
        executable: Some(other.path.clone()),
        config_home: None,
        env: BTreeMap::from([("PROBE_LOOP".into(), "1".into())]),
        default_model: None,
        options: BTreeMap::new(),
    });
    let handle = launch_with(&backend, &request).expect("launch");
    let launches = other.wait_for_loop_launches(1);
    assert_eq!(
        launches[0].env.get("PROBE_LOOP").map(String::as_str),
        Some("1")
    );
    assert!(
        stub.loop_launches().is_empty(),
        "the profile's executable is the one that ran"
    );
    // config_home stays REFUSED, not ignored, on this transport too.
    let mut refused = loop_pinned_request(dir.path());
    refused.profile = Some(Profile {
        name: "p2".into(),
        backend: AGY_BACKEND_NAME.into(),
        executable: None,
        config_home: Some(dir.path().to_path_buf()),
        env: BTreeMap::new(),
        default_model: None,
        options: BTreeMap::new(),
    });
    let error = backend
        .prepare(&refused)
        .expect_err("config_home is refused");
    let BackendError::Failed { detail, .. } = error else {
        panic!("a Failed refusal")
    };
    assert!(detail.contains("config_home is not supported"), "{detail}");
    let _ = handle;
}

#[test]
fn a_settings_home_reaches_a_loop_child() {
    // The measured permission channel (W1 P2), on this transport: HOME is
    // composed for the child, and every W3 probe that needed a different
    // permission posture got it exactly this way.
    let dir = TempDir::new().expect("tempdir");
    let home = dir.path().join("settings-home");
    std::fs::create_dir_all(home.join(".gemini/antigravity-cli")).expect("mkdir");
    std::fs::write(
        home.join(".gemini/antigravity-cli/settings.json"),
        r#"{"permissions":{"allow":["command(echo)"]}}"#,
    )
    .expect("write settings");
    let stub = StubAgy::passing(dir.path());
    let (init, replay) = loop_capture(LOOP_TWO_TURNS);
    stub.loop_init(&init);
    stub.replays(&replay);
    let mut config = loop_config_for(&stub, dir.path());
    config.settings_home = Some(home.clone());
    let backend = AgyBackend::new(config);
    let handle = launch_with(&backend, &loop_pinned_request(dir.path())).expect("launch");
    let launches = stub.wait_for_loop_launches(1);
    assert_eq!(
        launches[0].env.get("HOME").map(String::as_str),
        Some(home.to_string_lossy().as_ref())
    );
    // And the Work's own surface stays clean: the policy lives in the settings
    // home, never in the diff.
    assert!(!dir.path().join("settings.json").exists());
    let _ = handle;
}

#[test]
fn a_resumed_loop_execution_spawns_a_child_naming_the_conversation() {
    // RESUME never starts a turn; the first subsequent SEND is what needs a
    // child, and spawning one there is RESUME's own contract rather than the
    // auto-respawn §2.5 refuses.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    stub.loop_init(LOOP_RESUME_INIT_ECHO);
    let (_init, replay) = loop_capture(LOOP_TWO_TURNS);
    stub.replays(&replay);
    let backend = AgyBackend::new(loop_config_for(&stub, dir.path()));
    let (sink, _events) = sink();
    backend.set_event_sink(sink);
    let conversation = "b3be71a6-fd10-4525-875a-e7789a9811c3";
    let handle = ExecutionHandle {
        execution_id: "exec-readopted".to_string(),
        native_id: Some(conversation.to_string()),
    };
    let mut resume_request = ResumeRequest::new("w-agy", dir.path());
    resume_request.model = Some(FIXTURE_MODEL.to_string());
    backend.resume(&handle, &resume_request).expect("re-adopt");
    assert!(
        stub.loop_launches().is_empty(),
        "RESUME costs no process and no tokens"
    );
    backend.send(&handle, "still there?").expect("send");
    let launches = stub.wait_for_loop_launches(1);
    assert_eq!(
        launches[0].value_after("--conversation"),
        Some(conversation),
        "the re-adopted child names the conversation it is resuming"
    );
}

#[test]
fn a_loop_child_whose_init_echoes_a_different_conversation_refuses_the_send() {
    // W1 P0.6's silent fork, caught on this transport at CHILD START for zero
    // quota — an unknown id does not refuse, it warns on stderr and starts a
    // FRESH conversation, so the echo is the whole check.
    let dir = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(dir.path());
    // The child mints its own id rather than echoing the requested one.
    stub.loop_init(LOOP_INIT_LINE);
    stub.replays("");
    let backend = AgyBackend::new(loop_config_for(&stub, dir.path()));
    let handle = ExecutionHandle {
        execution_id: "exec-forked".to_string(),
        native_id: Some("a-conversation-agy-never-heard-of".to_string()),
    };
    let mut resume_request = ResumeRequest::new("w-agy", dir.path());
    resume_request.model = Some(FIXTURE_MODEL.to_string());
    backend.resume(&handle, &resume_request).expect("re-adopt");
    let error = backend
        .send(&handle, "hello?")
        .expect_err("a fork is not a resume");
    let BackendError::Failed { detail, .. } = error else {
        panic!("a Failed refusal")
    };
    assert!(detail.contains("silent fork"), "{detail}");
    assert!(
        detail.contains("before a single turn was spent"),
        "the whole point of checking at child start: {detail}"
    );
    assert!(
        stub.loop_stdin_lines().is_empty(),
        "no turn may be written into a forked conversation"
    );
}

// -------------------------------------------------------------- live suite

/// Gate mirroring `tests/opencode_backend.rs`'s `LiveGate` (the A3 pattern),
/// plus a **quota arm no sibling adapter could have**: agy answers
/// `-p "/usage" --output-format json` for free (W1 P0.2), so this can check the
/// remaining weekly fraction before spending a turn.
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
            "the installed agy does not pass the adapter's probe: {}",
            probe.detail.clone().unwrap_or_default()
        ));
    }
    LiveGate::Run
}

/// The minimum remaining weekly fraction this suite will spend a turn against.
/// Small on purpose: the point is to skip *loudly* at the end of a quota week
/// rather than fail mysteriously, not to ration.
const MIN_REMAINING_FRACTION: f64 = 0.05;

/// The zero-quota quota precheck. Returns `Err` with the bucket and its own
/// reset time when there is not enough left to be worth a turn.
fn quota_precheck(executable: &Path) -> Result<(), String> {
    let out = std::process::Command::new(executable)
        .args(["-p", "/usage", "--output-format", "json"])
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("cannot run the zero-quota /usage read: {e}"))?;
    let value: Value = serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("the /usage answer did not parse: {e}"))?;
    let groups = value
        .pointer("/command/data/groups")
        .and_then(Value::as_array)
        .ok_or_else(|| "the /usage answer named no groups".to_string())?;
    for group in groups {
        for bucket in group
            .get("buckets")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let id = bucket.get("id").and_then(Value::as_str).unwrap_or("");
            if !id.starts_with("gemini") {
                continue;
            }
            let remaining = bucket
                .get("remaining_fraction")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            if remaining < MIN_REMAINING_FRACTION {
                return Err(format!(
                    "bucket {id} has {remaining:.4} of its weekly limit left (below \
                     {MIN_REMAINING_FRACTION}); it resets at {}",
                    bucket
                        .get("reset_time")
                        .and_then(Value::as_str)
                        .unwrap_or("<unknown>")
                ));
            }
        }
    }
    Ok(())
}

/// Live config: the system `agy` (or `SGT_AGY_BIN`), and a scratch data dir
/// under `/var/tmp` — never `/tmp`, a quota'd tmpfs on this host (#70).
fn live_config(data_dir: &Path) -> AgyConfig {
    AgyConfig::new(data_dir)
}

/// Whether the opt-in live tests may run. Reaching this with the opt-in
/// variable unset is a misuse of `-- --ignored` and **panics**, naming the
/// opt-in: an `#[ignore]`d test that reports green while doing nothing is the
/// false green the gate exists to prevent. An unusable harness — a failing
/// probe, or an exhausted quota — is a clean skip written straight to fd 2
/// (libtest only captures the print macros).
fn agy_live_enabled(test: &str, data_dir: &Path) -> bool {
    let config = live_config(data_dir);
    let executable = config.executable.clone();
    let probe = AgyBackend::new(config).probe();
    match live_gate(std::env::var("SERGEANT_AGY_TESTS").ok().as_deref(), &probe) {
        LiveGate::Run => match quota_precheck(&executable) {
            Ok(()) => true,
            Err(why) => {
                let _ = std::io::stderr()
                    .write_all(format!("SKIPPED {test}: {why}\n").as_bytes())
                    .and_then(|()| std::io::stderr().flush());
                false
            }
        },
        LiveGate::NotOptedIn => panic!(
            "{test} is opt-in and consumes the owner's free-tier quota: run it with \
             SERGEANT_AGY_TESTS=1 cargo test --test agy_backend -- --ignored. (Without the \
             variable these tests are skipped by #[ignore]; asking for --ignored without it must \
             not report a green test that did nothing.)"
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
        .prefix(&format!("agy-live-{name}-"))
        .tempdir_in("/var/tmp")
        .expect("scratch dir under /var/tmp, never /tmp (quota'd tmpfs on this host)")
}

/// A live request: bounded, one-word-answer, and pinned to the cheap flash
/// model (K1). `context` is empty so the whole prompt is the two contracts plus
/// one sentence.
fn live_request(cwd: &Path, intent: &str) -> StartRequest {
    let mut request = start_request(cwd);
    request.model = Some(LIVE_MODEL.to_string());
    request.intent = intent.to_string();
    request.context = String::new();
    request
}

/// Backs the `model_selection` admission row. **1 live turn.**
#[test]
#[ignore = "opt-in, consumes free-tier quota: SERGEANT_AGY_TESTS=1 cargo test --test agy_backend -- --ignored"]
fn live_agy_init_line_echoes_the_pinned_model_and_mints_the_conversation() {
    let dir = live_workdir("pin");
    if !agy_live_enabled(
        "live_agy_init_line_echoes_the_pinned_model_and_mints_the_conversation",
        dir.path(),
    ) {
        return;
    }
    let backend = AgyBackend::new(live_config(dir.path()));
    let (event_sink, events) = sink();
    backend.set_event_sink(event_sink);
    let handle = launch_with(
        &backend,
        &live_request(dir.path(), "Reply with exactly one word: pong"),
    )
    .expect("launch");
    // The R4 delta, live: the conversation id and the RESOLVED model are both
    // on line 1, before any model output — so LAUNCH already knows them.
    let conversation = handle.native_id.clone().expect("a minted conversation id");
    assert!(!conversation.is_empty());
    wait_for_settled(&backend, &handle);
    let ended = wait_for_kind(&events, "conversation.turn.ended");
    assert_eq!(
        ended.payload["init"]["model"],
        serde_json::json!(LIVE_MODEL)
    );
    assert_eq!(
        ended.payload["model_pin"]["verdict"],
        serde_json::json!("honored")
    );
    assert_eq!(
        ended.payload["conversation_id"],
        serde_json::json!(conversation)
    );
    assert!(
        ended.payload["usage_turn"]["total_tokens"]
            .as_u64()
            .expect("a token count")
            > 0,
        "a real turn spends real tokens"
    );
}

/// Backs the `resume` admission row. **2 live turns.**
#[test]
#[ignore = "opt-in, consumes free-tier quota: SERGEANT_AGY_TESTS=1 cargo test --test agy_backend -- --ignored"]
fn live_agy_resume_recalls_a_nonce_and_echoes_the_same_conversation_id() {
    let dir = live_workdir("resume");
    if !agy_live_enabled(
        "live_agy_resume_recalls_a_nonce_and_echoes_the_same_conversation_id",
        dir.path(),
    ) {
        return;
    }
    let nonce = format!("agy-w1-{}", ulid::Ulid::generate());
    let backend = AgyBackend::new(live_config(dir.path()));
    let (event_sink, events) = sink();
    backend.set_event_sink(event_sink);
    let handle = launch_with(
        &backend,
        &live_request(
            dir.path(),
            &format!("Remember this token exactly: {nonce}. Reply with only the word ok."),
        ),
    )
    .expect("launch");
    let conversation = handle.native_id.clone().expect("a minted conversation id");
    wait_for_settled(&backend, &handle);

    // Turn 2, from a separately spawned OS process carrying `--conversation`.
    backend
        .send(
            &handle,
            "What was the token I asked you to remember? Reply with only the token.",
        )
        .expect("send");
    let deadline = Instant::now() + Duration::from_secs(180);
    let observation = loop {
        let observation = backend.observe(&handle).expect("observe");
        if observation.native != NativeState::Running {
            break observation;
        }
        assert!(Instant::now() < deadline, "the resumed turn never settled");
        std::thread::sleep(Duration::from_millis(200));
    };
    let BackendSignal::StageCompleted { summary } = observation.signal else {
        panic!("the resumed turn should complete: {observation:?}")
    };
    let summary = summary.expect("a summary");
    assert!(
        summary.contains(&nonce),
        "the conversation must recall the nonce across processes; got {summary:?}"
    );
    // And the identity check that makes the tier `ConversationIdEchoOnNextTurn`
    // rather than a hope: agy warns-and-continues on an unknown id, so the echo
    // is what distinguishes a resume from a silent fork (W1 P0.6).
    let ended = events_of_kind(&events, "conversation.turn.ended");
    let last = ended.last().expect("a second turn.ended");
    assert_eq!(
        last.payload["conversation_id"],
        serde_json::json!(conversation)
    );
    assert_eq!(
        last.payload["model_pin"]["verdict"],
        serde_json::json!("honored")
    );
    assert!(
        !events_of_kind(&events, "conversation.turn.harness_error")
            .iter()
            .any(|e| e.payload["phase"] == serde_json::json!("resume_identity_mismatch")),
        "a real resume must raise no identity mismatch"
    );
}

// ------------------------------------------------- W3 live tier (the loop)

/// A live **loop-transport** config. `LoopOnly` rather than `Auto` on purpose:
/// if the installed build cannot serve this transport, these tests must refuse
/// loudly rather than quietly measure the print transport and label the result
/// with the loop's tiers.
fn live_loop_config(data_dir: &Path) -> AgyConfig {
    let mut config = AgyConfig::new(data_dir);
    config.transport = TransportChoice::LoopOnly;
    config
}

/// Poll OBSERVE until the turn settles. Separate from `wait_for_settled`
/// because on this transport `native` does **not** mean "the process exited" —
/// the child outlives every turn — so the keyed signal is the turn's own
/// settled outcome.
fn wait_for_loop_turn(backend: &AgyBackend, handle: &ExecutionHandle) -> Observation {
    let deadline = Instant::now() + Duration::from_secs(300);
    loop {
        let observation = backend.observe(handle).expect("observe");
        if !matches!(observation.signal, BackendSignal::Running)
            || observation.native == NativeState::Exited
        {
            return observation;
        }
        assert!(Instant::now() < deadline, "the loop turn never settled");
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// Backs the loop's `resume`, `model_selection` and `identity_before_first_turn`
/// rows. **2 live turns.**
///
/// One live test for three rows, and the sharing is the finding rather than a
/// shortcut: on this transport a single `init` line at child start carries the
/// conversation id, the resolved model *and* the resume echo, so one turn's
/// worth of quota evidences all three. The *cost* half of the claim — that the
/// echo is checked before any turn is written — is pinned deterministically by
/// `a_loop_child_whose_init_echoes_a_different_conversation_refuses_the_send`,
/// which asserts no stdin line was written at all.
#[test]
#[ignore = "opt-in, consumes free-tier quota: SERGEANT_AGY_TESTS=1 cargo test --test agy_backend -- --ignored"]
fn live_agy_loop_resume_echoes_the_conversation_before_any_turn() {
    let dir = live_workdir("loop-resume");
    if !agy_live_enabled(
        "live_agy_loop_resume_echoes_the_conversation_before_any_turn",
        dir.path(),
    ) {
        return;
    }
    let nonce = format!("agy-w3-{}", ulid::Ulid::generate());
    let backend = AgyBackend::new(live_loop_config(dir.path()));
    assert!(
        backend.capabilities().native_subagents,
        "the loop transport's capability set, not the print one's"
    );
    let (event_sink, events) = sink();
    backend.set_event_sink(event_sink);
    let handle = launch_with(
        &backend,
        &live_request(
            dir.path(),
            &format!("Remember this token exactly: {nonce}. Reply with only the word ok."),
        ),
    )
    .expect("launch");
    // Identity and the RESOLVED model were both known before this turn was
    // written down the child's stdin.
    let conversation = handle.native_id.clone().expect("a minted conversation id");
    assert!(!conversation.is_empty());
    wait_for_loop_turn(&backend, &handle);
    let first = wait_for_kind(&events, "conversation.turn.ended");
    assert_eq!(
        first.payload["transport"],
        serde_json::json!("input-loop-stream-json")
    );
    assert_eq!(
        first.payload["init"]["model"],
        serde_json::json!(LIVE_MODEL)
    );
    assert_eq!(
        first.payload["model_pin"]["verdict"],
        serde_json::json!("honored")
    );
    assert!(
        first.payload["usage_turn"]["total_tokens"]
            .as_u64()
            .expect("a token count")
            > 0,
        "a real turn spends real tokens"
    );
    backend.stop(&handle).expect("stop").wait();

    // A SEPARATE backend, a separate loop child, `--conversation <id>` — the
    // re-adoption path a restarted daemon takes.
    let resumed = AgyBackend::new(live_loop_config(dir.path()));
    let (resumed_sink, resumed_events) = sink();
    resumed.set_event_sink(resumed_sink);
    let mut request = ResumeRequest::new("w-agy", dir.path());
    request.model = Some(LIVE_MODEL.to_string());
    resumed.resume(&handle, &request).expect("re-adopt");
    resumed
        .send(
            &handle,
            "What was the token I asked you to remember? Reply with only the token.",
        )
        .expect("send");
    let observation = wait_for_loop_turn(&resumed, &handle);
    let BackendSignal::StageCompleted { summary } = observation.signal else {
        panic!("the resumed turn should complete: {observation:?}")
    };
    let summary = summary.expect("a summary");
    assert!(
        summary.contains(&nonce),
        "the conversation must recall the nonce across processes; got {summary:?}"
    );
    let ended = events_of_kind(&resumed_events, "conversation.turn.ended");
    let last = ended.last().expect("the resumed turn ended");
    assert_eq!(
        last.payload["conversation_id"],
        serde_json::json!(conversation),
        "the init line echoed the id we asked for — the whole InitEchoAtChildStart tier"
    );
    assert_eq!(
        last.payload["model_pin"]["verdict"],
        serde_json::json!("honored")
    );
    assert!(
        !events_of_kind(&resumed_events, "conversation.turn.harness_error")
            .iter()
            .any(|e| e.payload["phase"] == serde_json::json!("resume_identity_mismatch")),
        "a real resume must raise no identity mismatch"
    );
    resumed.stop(&handle).expect("stop").wait();
}

/// Backs the loop's `native_subagents` and `turn_serialization` rows.
/// **2 live turns.**
///
/// The registry's first `true` for `native_subagents`, and it is admitted on a
/// **typed record or not at all**: a child `conversation_id` distinct from the
/// parent's, on a settled `subagent_info` step. Assistant prose claiming a
/// delegation is explicitly not evidence and would fail this test.
#[test]
#[ignore = "opt-in, consumes free-tier quota: SERGEANT_AGY_TESTS=1 cargo test --test agy_backend -- --ignored"]
fn live_agy_loop_invokes_a_subagent_and_records_its_typed_conversation_id() {
    let dir = live_workdir("loop-subagent");
    if !agy_live_enabled(
        "live_agy_loop_invokes_a_subagent_and_records_its_typed_conversation_id",
        dir.path(),
    ) {
        return;
    }
    let backend = AgyBackend::new(live_loop_config(dir.path()));
    let (event_sink, events) = sink();
    backend.set_event_sink(event_sink);
    let handle = launch_with(
        &backend,
        &live_request(
            dir.path(),
            "Use the define_subagent tool to define a subagent named \"echoer\" whose only job is \
             to reply with one word. Do not do anything else.",
        ),
    )
    .expect("launch");
    let parent = handle.native_id.clone().expect("a minted conversation id");
    wait_for_loop_turn(&backend, &handle);

    // Turn 2 down the SAME child's stdin — which is also the
    // `turn_serialization` row's own exercise: one conversation, two turns,
    // strictly one at a time.
    backend
        .send(
            &handle,
            "Use the invoke_subagent tool to have \"echoer\" reply with exactly the word: delta.",
        )
        .expect("send");
    wait_for_loop_turn(&backend, &handle);

    let ended = events_of_kind(&events, "conversation.turn.ended");
    assert_eq!(ended.len(), 2, "two turns, one child, one conversation");
    assert_eq!(
        ended[1].payload["conversation_id"],
        serde_json::json!(parent)
    );

    let subagent_ids: Vec<String> = ended
        .iter()
        .flat_map(|event| {
            event.payload["subagent_conversations"]
                .as_array()
                .cloned()
                .unwrap_or_default()
        })
        .filter_map(|value| value.as_str().map(str::to_string))
        .collect();
    assert!(
        !subagent_ids.is_empty(),
        "no typed subagent_info record arrived. The row stays FALSE on anything less than a typed \
         child conversation_id — assistant text saying it delegated is not evidence. Transcript: \
         {ended:?}"
    );
    for child in &subagent_ids {
        assert_ne!(
            child, &parent,
            "a subagent sharing the parent's conversation is not a subagent"
        );
    }
    // And the same record reached the tool vocabulary, with the child's own log.
    let completed: Vec<_> = events_of_kind(&events, "tool.completed")
        .into_iter()
        .filter(|event| event.payload["subagent"].is_array())
        .collect();
    let child = completed
        .iter()
        .flat_map(|event| {
            event.payload["subagent"]
                .as_array()
                .cloned()
                .unwrap_or_default()
        })
        .find(|child| child["conversation_id"].is_string())
        .expect("a typed child record on a tool.completed event");
    assert!(
        child["log_uri"].is_string(),
        "the child's own trajectory log"
    );
    backend.stop(&handle).expect("stop").wait();
}

// ------------------------------------------------------- W2 §1.4: registration

/// A probeable agy registers for real: `daemon::start_with`, with no
/// test-supplied stand-in for the name "agy", puts a real `AgyBackend`
/// in its registry and journals its own probe evidence at registration
/// — the direct proof of W2's registration change, not a repeat of any
/// unit test on `AgyBackend` itself.
#[tokio::test]
async fn daemon_start_registers_agy_and_journals_its_own_probe() {
    let data = TempDir::new().expect("tempdir");
    let stub = StubAgy::passing(data.path());
    let handle = daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(sergeant_rs::backend::BackendRegistry::new()),
            default_backend: None,
            agy: Some(config_for(&stub, data.path())),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start with an unmeasured-nothing agy must still start");
    handle.shutdown().await;

    let probed: Vec<_> = Journal::replay_data_dir(data.path())
        .expect("replay")
        .map(|e| e.expect("event"))
        .filter(|e| e.kind == daemon::KIND_BACKEND_PROBED)
        .collect();
    let agy_probed = probed
        .iter()
        .find(|e| e.payload["backend"] == AGY_BACKEND_NAME)
        .expect("a backend.probed record for agy");
    assert_eq!(agy_probed.payload["available"], true);
    assert_eq!(agy_probed.payload["runtime_scope"], "per_execution");
    assert_eq!(agy_probed.payload["capabilities"]["ask"], false);
    assert_eq!(
        agy_probed.payload["capabilities"]["persistent_sessions"],
        true
    );
    assert_eq!(
        agy_probed.payload["capabilities"]["native_background"],
        false
    );
    assert_eq!(agy_probed.payload["capabilities"]["streaming"], true);
    assert_eq!(agy_probed.payload["capabilities"]["history"], false);
    assert_eq!(agy_probed.payload["capabilities"]["resume"], true);
    assert_eq!(agy_probed.payload["capabilities"]["interrupt"], true);
    assert_eq!(agy_probed.payload["capabilities"]["model_selection"], true);
    assert_eq!(agy_probed.payload["capabilities"]["profiles"], true);
    assert_eq!(agy_probed.payload["capabilities"]["approval_flow"], false);
    assert_eq!(agy_probed.payload["capabilities"]["human_attach"], false);
    assert_eq!(agy_probed.payload["capabilities"]["usage"], true);
    assert_eq!(
        agy_probed.payload["capabilities"]["native_subagents"],
        false
    );
}

/// Agy missing must not break daemon startup, and must be
/// distinguishable from "unregistered": it registers anyway and
/// journals honest, `available: false` evidence naming why it cannot
/// run — the same posture Docker/Codex/Opencode already take for a
/// host with no binary installed.
#[tokio::test]
async fn daemon_start_with_no_agy_installed_still_starts_and_says_why() {
    let data = TempDir::new().expect("tempdir");
    let handle = daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(sergeant_rs::backend::BackendRegistry::new()),
            default_backend: None,
            agy: Some(AgyConfig {
                executable: PathBuf::from("/nonexistent/definitely-not-agy"),
                ..AgyConfig::new(data.path())
            }),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("a host with no agy binary must still start a daemon");
    handle.shutdown().await;

    let agy_probed = Journal::replay_data_dir(data.path())
        .expect("replay")
        .map(|e| e.expect("event"))
        .find(|e| e.kind == daemon::KIND_BACKEND_PROBED && e.payload["backend"] == AGY_BACKEND_NAME)
        .expect("agy is registered — and probed — even though it cannot run");
    assert_eq!(agy_probed.payload["available"], false);
    let detail = agy_probed.payload["detail"]
        .as_str()
        .expect("probe detail recorded");
    assert!(detail.contains("cannot run"), "{detail}");
}
