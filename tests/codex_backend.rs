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
//! Two daemon-level tests (W2, §1.4) prove registration itself — that
//! `daemon::start_with` puts a "codex" entry in the registry with no
//! test-supplied stand-in — via in-process `daemon::start_with` +
//! `handle.shutdown().await`, the same rig `tests/m3_execution.rs`/
//! `tests/estate_routes.rs` use, so `tests/support::DataDir`'s reaper does
//! not apply to them either (no subprocess is spawned). Every other test in
//! this file constructs `CodexBackend::new` directly, the way the claude
//! stub tests do.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;

use sergeant_rs::backend::codex::{
    Budgets, CODEX_BACKEND_NAME, CodexBackend, CodexConfig, TransportChoice,
};
use sergeant_rs::backend::{
    Backend, BackendError, BindingSummary, ExecutionHandle, NativeState, ProbeReport,
    ResumeRequest, StartRequest,
};
use sergeant_rs::daemon::{self, DaemonConfig};
use sergeant_rs::domain::estate::InstructionPolicy;
use sergeant_rs::domain::event::EventDraft;
use sergeant_rs::runtime::git::canonical_git_common_dir;
use sergeant_rs::runtime::journal::Journal;

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

/// Whether a pid is still alive — genuinely running, not merely present in
/// the process table. A bare `kill -0` reports "alive" for an unreaped
/// ZOMBIE too, which is exactly the codebase's own precedent:
/// `tests/m9_watch.rs`'s `spawn_bare_daemon` comment documents a zombie as
/// "invisible to `kill -0` as gone but not actually gone", the *measured*
/// cause of a real timeout failure in
/// `w7_stream_closure_is_honest_and_never_restarts_the_daemon` before that
/// fix. That fix reaps a *direct* child with a background `wait()` thread;
/// it doesn't apply here, because the grandchild this test tracks is never
/// this process's own child — it is forked deep inside the stub's shell
/// script and reparented to init the instant that shell dies, so nothing
/// in this process tree can `wait()` it away. Only the process's state
/// tells a killed-but-unreaped zombie apart from one still genuinely
/// running: `ps -o state=` (POSIX-portable; the one Linux/macOS-shared
/// keyword for it, so no platform split is needed, unlike
/// `platform::process`'s `/proc`-vs-`ps` liveness split) reports `Z` for a
/// zombie on both procps and BSD `ps`. State `Z` means the kernel has
/// already delivered the fatal signal and the process is doing nothing
/// further; that init hasn't reaped its entry yet is not evidence that
/// INTERRUPT failed to kill it, so this reports it as dead.
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
    grandchild: PathBuf,
    grandchild_pid: PathBuf,
    detach: PathBuf,
    /// W3: a marker file whose presence turns on this stub's `app-server`
    /// subcommand emulation (`--help` offering `stdio://`, a minimal
    /// `generate-json-schema` dump, and a bare `initialize`-only responder
    /// on `--listen stdio://`). Absent by default, so every W1/W2 exec-only
    /// test's stub fails G1 fast and Auto resolves to exec with no spawn of
    /// anything that could hang or pollute this stub's shared launches
    /// record — `supports_appserver()` is the opt-in for the tests that
    /// need the opposite.
    appserver_supported: PathBuf,
    /// W3 §6.2's fourth `StubCodex` mode: every request line the `--listen
    /// stdio://` handler reads is appended here verbatim, so a test can
    /// assert on exactly what this adapter sent (`appserver_requests`).
    appserver_requests: PathBuf,
    /// W3 §6.2's "scripted reply table": a directory of `<method_with_
    /// underscores>.jsonl` files, one per method a test wants answered
    /// beyond the built-in bare `initialize` responder. Each line is emitted
    /// verbatim except `__ID__`, substituted with the incoming request's own
    /// id (`appserver_scripts_reply`); a sibling `<file>.exit_after` marker
    /// makes the stub close its stdout right after replying (deterministic
    /// child death mid-turn, or the aftermath of an RPC-failure fallback —
    /// `appserver_exits_after`).
    appserver_scripts_dir: PathBuf,
}

/// The filename stem the stub's own shell script computes for one JSON-RPC
/// method (`tr '/' '_'`) — kept as one function so the Rust side that writes
/// a scripted reply and the shell side that looks it up can never drift.
fn appserver_script_stem(method: &str) -> String {
    method.replace('/', "_")
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
        let grandchild = dir.join("codex-grandchild");
        let grandchild_pid = dir.join("codex-grandchild-pid");
        let detach = dir.join("codex-grandchild-detach");
        let appserver_supported = dir.join("codex-appserver-supported");
        let appserver_requests = dir.join("codex-appserver-requests.jsonl");
        let appserver_scripts_dir = dir.join("codex-appserver-scripts");
        let script = format!(
            "#!/bin/sh\n\
             if [ \"$1\" = \"--version\" ]; then echo \"{version}\"; exit 0; fi\n\
             if [ \"$1\" = \"login\" ] && [ \"$2\" = \"status\" ]; then echo \"{auth_line}\"; exit 0; fi\n\
             if [ \"$1\" = \"exec\" ] && [ \"$2\" = \"--help\" ]; then printf '%s\\n' \"{exec_help}\"; exit 0; fi\n\
             if [ \"$1\" = \"exec\" ] && [ \"$2\" = \"resume\" ] && [ \"$3\" = \"--help\" ]; then printf '%s\\n' \"{resume_help}\"; exit 0; fi\n\
             \
             if [ \"$1\" = \"app-server\" ] && [ \"$2\" = \"--help\" ]; then\n  \
               if [ -f \"{appserver_supported}\" ]; then\n    \
                 printf 'Usage: codex app-server\\n      --listen <URL>\\n          Supported: stdio:// (default)\\n'\n  \
               else\n    \
                 printf 'Usage: codex app-server\\n      --listen <URL>\\n          Supported: ws://IP:PORT only on this stub build\\n'\n  \
               fi\n  \
               exit 0\n\
             fi\n\
             if [ \"$1\" = \"app-server\" ] && [ \"$2\" = \"generate-json-schema\" ]; then\n  \
               if [ -f \"{appserver_supported}\" ]; then\n    \
                 outdir=\"$4\"\n    \
                 mkdir -p \"$outdir/v1\" \"$outdir/v2\"\n    \
                 for f in v1/InitializeParams.json v1/InitializeResponse.json \
                 v2/ThreadStartParams.json v2/ThreadStartResponse.json v2/TurnStartParams.json \
                 v2/TurnStartResponse.json v2/TurnInterruptParams.json \
                 v2/TurnCompletedNotification.json v2/TurnStartedNotification.json \
                 v2/ItemStartedNotification.json v2/ItemCompletedNotification.json \
                 v2/ThreadTokenUsageUpdatedNotification.json ToolRequestUserInputParams.json \
                 ServerRequest.json; do\n      \
                   echo '{{\"stub\":true}}' > \"$outdir/$f\"\n    \
                 done\n    \
                 exit 0\n  \
               else\n    \
                 exit 1\n  \
               fi\n\
             fi\n\
             if [ \"$1\" = \"app-server\" ] && [ \"$2\" = \"--listen\" ]; then\n  \
               if [ -f \"{appserver_supported}\" ]; then\n    \
                 if [ -f \"{stderr}\" ]; then cat \"{stderr}\" >&2; fi\n    \
                 while IFS= read -r line; do\n      \
                   printf '%s\\n' \"$line\" >> \"{appserver_requests}\"\n      \
                   method=$(printf '%s' \"$line\" | sed -n 's/.*\"method\":\"\\([^\"]*\\)\".*/\\1/p')\n      \
                   id=$(printf '%s' \"$line\" | sed -n 's/.*\"id\":\\([0-9]*\\).*/\\1/p')\n      \
                   safe=$(printf '%s' \"$method\" | tr '/' '_')\n      \
                   script_file=\"{appserver_scripts_dir}/$safe.jsonl\"\n      \
                   if [ -f \"$script_file.exit_before\" ]; then\n        \
                     seen=0\n        \
                     if [ -f \"$script_file.count\" ]; then seen=$(cat \"$script_file.count\"); fi\n        \
                     seen=$((seen+1))\n        \
                     echo \"$seen\" > \"$script_file.count\"\n        \
                     if [ \"$seen\" -ge \"$(cat \"$script_file.exit_before\")\" ]; then exit 0; fi\n      \
                   fi\n      \
                   if [ \"$method\" = \"initialize\" ] && [ ! -f \"$script_file\" ]; then\n        \
                     printf '%s\\n' '{{\"id\":__ID__,\"result\":{{\"userAgent\":\"stub/0.0.0\",\
\"codexHome\":\"/stub\",\"platformFamily\":\"unix\",\"platformOs\":\"linux\"}}}}' \
| sed \"s/__ID__/$id/\"\n      \
                   elif [ -f \"$script_file\" ]; then\n        \
                     while IFS= read -r out; do\n          \
                       printf '%s\\n' \"$out\" | sed \"s/__ID__/$id/g\"\n        \
                     done < \"$script_file\"\n      \
                   fi\n      \
                   if [ -f \"$script_file.exit_after\" ]; then\n        \
                     if [ -f \"$script_file.exit_code\" ]; then \
exit \"$(cat \"$script_file.exit_code\")\"; fi\n        \
                     exit 0\n      \
                   fi\n    \
                 done\n    \
                 exit 0\n  \
               else\n    \
                 exit 1\n  \
               fi\n\
             fi\n\
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
             if [ -f \"{grandchild}\" ]; then\n  \
               if [ -f \"{detach}\" ]; then\n    \
                 ( i=0; while [ \"$i\" -lt 300 ]; do sleep 0.1; i=$((i+1)); done ) \
                   </dev/null >/dev/null 2>&1 &\n  \
               else\n    \
                 ( i=0; while [ \"$i\" -lt 300 ]; do sleep 0.1; i=$((i+1)); done ) &\n  \
               fi\n  \
               echo $! > \"{grandchild_pid}\"\n\
             fi\n\
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
            grandchild = grandchild.display(),
            grandchild_pid = grandchild_pid.display(),
            detach = detach.display(),
            hang = hang.display(),
            exit_code = exit_code.display(),
            appserver_supported = appserver_supported.display(),
            appserver_requests = appserver_requests.display(),
            appserver_scripts_dir = appserver_scripts_dir.display(),
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
            grandchild,
            grandchild_pid,
            detach,
            appserver_supported,
            appserver_requests,
            appserver_scripts_dir,
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

    /// Turn on this stub's `app-server` subcommand emulation (W3 §6.2's
    /// fourth `StubCodex` mode): `app-server --help` offers `stdio://`,
    /// `generate-json-schema` writes the 14 pinned files (stub content —
    /// this is what makes G2's file-presence check pass, never a claim the
    /// bytes are real schemas), and `--listen stdio://` answers a bare
    /// `initialize` request out of the box. Every other method
    /// (`thread/start`/`turn/start`/`turn/interrupt`/…) is answered only if
    /// a test scripts a reply for it (`appserver_scripts_reply`) — the
    /// process-bound half of the protocol suite (§6.2: "spawn, handshake
    /// timing, budgets, interrupt, process-group kill, stderr drain, child
    /// death mid-turn") is this stub's scope; the live suite remains this
    /// wave's proof of the real harness's own behaviour.
    fn supports_appserver(&self) -> &Self {
        std::fs::write(&self.appserver_supported, b"go\n").expect("write appserver marker");
        self
    }

    /// W3 §6.2's "scripted reply table": write the literal output lines this
    /// stub's `--listen stdio://` handler emits the moment it reads a
    /// request whose `"method"` is exactly `method` — one block per method,
    /// a generic table the dispatch loop consults by filename, never a
    /// per-method special case hardcoded into the stub itself. `__ID__` in
    /// any line is replaced with the request's own numeric id at emission
    /// time, so a canned reply can echo whichever id this run's client
    /// happened to mint.
    fn appserver_scripts_reply(&self, method: &str, lines: &[&str]) -> &Self {
        std::fs::create_dir_all(&self.appserver_scripts_dir).expect("scripts dir");
        let path = self
            .appserver_scripts_dir
            .join(format!("{}.jsonl", appserver_script_stem(method)));
        std::fs::write(&path, lines.join("\n") + "\n").expect("write scripted reply");
        self
    }

    /// Mark this stub to close its stdout (the adapter reader thread's own
    /// EOF) immediately after it finishes answering one request for
    /// `method` — deterministic child death mid-turn (test 21) and the
    /// aftermath of an interrupt-RPC-failure fallback (test 22), neither of
    /// which should be a race against a real process's own timing.
    fn appserver_exits_after(&self, method: &str) -> &Self {
        std::fs::create_dir_all(&self.appserver_scripts_dir).expect("scripts dir");
        let marker = self.appserver_scripts_dir.join(format!(
            "{}.jsonl.exit_after",
            appserver_script_stem(method)
        ));
        std::fs::write(&marker, b"go\n").expect("write exit-after marker");
        self
    }

    /// [`Self::appserver_exits_after`], with a chosen exit status rather than
    /// a clean `0` — the evidence an ambiguous terminal must be able to name
    /// (a child that died *with* a status, not merely "a child that is
    /// gone").
    fn appserver_exits_after_with_code(&self, method: &str, code: i32) -> &Self {
        self.appserver_exits_after(method);
        let marker = self
            .appserver_scripts_dir
            .join(format!("{}.jsonl.exit_code", appserver_script_stem(method)));
        std::fs::write(&marker, code.to_string()).expect("write exit-code marker");
        self
    }

    /// Mark this stub to close its stdout on the `nth` request for `method`
    /// (1-based, and from then on) **before** emitting that method's scripted
    /// reply — the sibling of [`Self::appserver_exits_after`] for the case it
    /// cannot express: a child that dies with a request already in flight, so
    /// the adapter's own `call` is left waiting for a response nobody will
    /// ever write. The count is what lets one script serve a turn that must
    /// succeed and a later turn that must die.
    fn appserver_exits_before(&self, method: &str, nth: u32) -> &Self {
        std::fs::create_dir_all(&self.appserver_scripts_dir).expect("scripts dir");
        let marker = self.appserver_scripts_dir.join(format!(
            "{}.jsonl.exit_before",
            appserver_script_stem(method)
        ));
        std::fs::write(&marker, nth.to_string()).expect("write exit-before marker");
        self
    }

    /// Every request line this stub's `--listen stdio://` handler has read
    /// so far, parsed as JSON, in arrival order — the captured wire evidence
    /// a deterministic test asserts on (§3.6 test 4: "the captured
    /// `thread/start` params").
    fn appserver_requests(&self) -> Vec<Value> {
        std::fs::read_to_string(&self.appserver_requests)
            .unwrap_or_default()
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).expect("captured request line is valid JSON"))
            .collect()
    }

    /// Marks the stub to fork a background grandchild (a detached loop, not
    /// a `setsid`) right after replay, and record that grandchild's own pid —
    /// the process a single `child.kill()` would never reach, and the whole
    /// reason INTERRUPT kills the turn's process *group* instead (§5.5).
    fn spawns_a_grandchild(&self) -> &Self {
        std::fs::write(&self.grandchild, b"go\n").expect("write grandchild marker");
        self
    }

    /// Marks the grandchild to be forked with its own `/dev/null` standard
    /// streams instead of the turn's inherited pipes. That single difference
    /// is what makes the leader-exit race deterministic: with the pipes
    /// inherited (the default above) the grandchild holds the turn's stdout
    /// open, so the adapter's reader never sees EOF and the turn stays
    /// `InFlight` no matter when the leader dies. Detached, the leader's exit
    /// *is* EOF, the reader reaps and files the turn's outcome, and INTERRUPT
    /// arrives at an execution whose leader is already gone — while the
    /// grandchild it was supposed to kill is still running.
    fn detaches_its_grandchild(&self) -> &Self {
        std::fs::write(&self.detach, b"go\n").expect("write detach marker");
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

// ---------------------------------------------------- W3 §5.2: transport gates

/// A stub with no `.supports_appserver()` marker fails G1 (its `app-server
/// --help` never offers `stdio://`) — `Auto` must fall back to exec, and
/// `probe()` must still be `available: true` (a gate failure changes which
/// transport, never whether the backend works at all).
#[test]
fn appserver_gate_failure_falls_back_to_exec_under_auto() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let report = backend.probe();
    assert!(report.available);
    let detail = report.detail.expect("detail");
    assert!(
        detail.contains("transport: exec"),
        "Auto must resolve to exec when the app-server gate fails: {detail}"
    );
    assert!(
        detail.contains("app-server gate failed"),
        "the failed gate must be named, not silently absorbed: {detail}"
    );
}

/// `AppServerOnly` + a failed gate is the one place §5.2 refuses outright
/// (rule 2): the operator asked for exactly this transport, and silently
/// handing them exec — a different capability row — is the dishonesty the
/// rule exists to prevent.
#[test]
fn appserver_only_refuses_when_a_gate_fails() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    let mut config = config_for(&stub, dir.path(), &dir.path().join("codex-home"));
    config.transport = TransportChoice::AppServerOnly;
    let backend = CodexBackend::new(config);
    let report = backend.probe();
    assert!(
        !report.available,
        "AppServerOnly with a failed gate must refuse, not fall back"
    );
    let detail = report.detail.expect("detail");
    assert!(detail.contains("AppServerOnly"), "{detail}");
    assert!(
        detail.contains("G1"),
        "the failed gate must be named: {detail}"
    );
}

/// The opposite: a stub that *does* emulate the app-server subcommands
/// (`.supports_appserver()`) passes G1/G2/G4, so `Auto` resolves to
/// app-server — and the resolution is memoized (§5.3: never revisited per
/// execution), which this proves by calling `probe()` twice and getting the
/// identical resolved transport both times.
#[test]
fn transport_is_resolved_once_and_journaled() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let first = backend.probe();
    let second = backend.probe();
    assert!(first.available);
    assert_eq!(first.detail, second.detail, "resolution must be memoized");
    let detail = first.detail.expect("detail");
    assert!(
        detail.contains("transport: app-server (stdio) (Auto)"),
        "a stub that passes every gate must resolve Auto to app-server: {detail}"
    );
    assert!(
        detail.contains("protocol: fresh") || detail.contains("protocol: stale"),
        "{detail}"
    );
}

/// `ExecOnly` always resolves to exec, even when a stub would otherwise
/// pass every app-server gate — the operator's own configured choice wins
/// unconditionally (§5.2 rule 1).
#[test]
fn transport_choice_exec_only_never_touches_the_appserver_gates() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    let mut config = config_for(&stub, dir.path(), &dir.path().join("codex-home"));
    config.transport = TransportChoice::ExecOnly;
    let backend = CodexBackend::new(config);
    let report = backend.probe();
    assert!(report.available);
    let detail = report.detail.expect("detail");
    assert!(
        detail.contains("transport: exec (ExecOnly configured)"),
        "{detail}"
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
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.stalls_until_released(); // parks before emitting anything, never released
    let mut config = config_for(&stub, dir.path(), &dir.path().join("codex-home"));
    // This test's own instance shrinks the budget to milliseconds instead of
    // the production 30s; `thread_id_budget` lives on this `CodexConfig`
    // value alone, so no other test's `launch()` (running concurrently on
    // another thread) can ever observe it.
    config.thread_id_budget = Some(Duration::from_millis(300));
    let backend = CodexBackend::new(config);
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
}

/// The budget's own complement: a turn that parks before emitting anything
/// and is then released *within* the budget must still succeed — LAUNCH
/// waits on `thread.started`, never on the whole turn finishing (issue #46's
/// shape on Claude: a turn whose process outlives launch-settle and then
/// finishes on its own).
#[test]
fn launch_succeeds_once_a_parked_turn_is_released_within_budget() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.replays(AGENT_MESSAGE_TURN).stalls_until_released();
    let mut config = config_for(&stub, dir.path(), &dir.path().join("codex-home"));
    config.thread_id_budget = Some(Duration::from_millis(5000));
    let backend = CodexBackend::new(config);
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

/// A real linked worktree at `worktree`, cut from a fresh one-commit repo at
/// `source`, plus the git admin directory git itself recorded for it — read
/// straight from the worktree's own `.git` file rather than importing
/// `src/backend/codex.rs`'s private `worktree_git_admin_dir`, so this
/// fixture and the adapter under test independently agree on where that
/// directory lives.
fn real_worktree(source: &Path, worktree: &Path, branch: &str) -> PathBuf {
    support::init_repo(source);
    support::git(
        source,
        &["worktree", "add", "-b", branch, worktree.to_str().unwrap()],
    );
    let dot_git = std::fs::read_to_string(worktree.join(".git")).expect("worktree .git file");
    PathBuf::from(
        dot_git
            .trim()
            .strip_prefix("gitdir:")
            .expect("a linked worktree's .git file names a gitdir")
            .trim(),
    )
}

/// #259: turn 1's `--add-dir` must carry exactly one grant per binding — the
/// Work's own `.git/worktrees/<name>` admin directory, never
/// `repository.path` (the source checkout) and never the shared `.git`
/// itself. Without this grant a codex actor can edit files in its assigned
/// worktree but `git add`/`git commit` there fails with `Read-only file
/// system` (five observed Terra Works, issue #259's own evidence).
#[test]
fn the_first_turn_grants_the_works_own_git_admin_dir_as_an_add_dir_root() {
    let dir = TempDir::new().expect("tempdir");
    let source = dir.path().join("source");
    let worktree = dir.path().join("worktree");
    let admin_dir = real_worktree(&source, &worktree, "sergeant/w-codex");
    // Exactly the one directory, never the repository or the shared `.git`.
    assert_eq!(
        admin_dir,
        source.join(".git").join("worktrees").join("worktree"),
        "git's own scheme for a linked worktree's admin dir"
    );

    let stub = StubCodex::passing(dir.path());
    stub.replays(AGENT_MESSAGE_TURN);
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let mut request = start_request(&worktree);
    request.bindings = vec![BindingSummary {
        repository: "solo".to_string(),
        worktree_path: worktree.clone(),
        work_branch: "sergeant/w-codex".to_string(),
        base_branch: Some("main".to_string()),
        base_sha: "0".repeat(40),
    }];
    let prepared = backend.prepare(&request).expect("prepare");
    backend.launch(&prepared).expect("launch");
    let launches = stub.wait_for_launches(1);
    let launch = &launches[0];
    let add_dir_values: Vec<&str> = launch
        .argv
        .windows(2)
        .filter(|pair| pair[0] == "--add-dir")
        .map(|pair| pair[1].as_str())
        .collect();
    assert_eq!(
        add_dir_values,
        vec![admin_dir.to_str().unwrap()],
        "argv: {:?}",
        launch.argv
    );
}

/// A fixture-level sanity check, not a regression test for
/// `runtime::surface`'s private `common_dir_finding` itself (that
/// function's own regression test,
/// `common_dir_finding_reports_no_mismatch_for_a_genuine_linked_worktree`,
/// lives in `src/runtime/surface.rs`'s unit tests, the only place with
/// access to it). What this test pins is the underlying git fact #259's
/// grant depends on: a linked worktree's `canonical_git_common_dir` still
/// agrees with its source checkout's after `real_worktree` creates it. #259's
/// grant reads the worktree's `.git` file but writes nothing and calls no
/// git subprocess, so this identity is untouched by the fix — this test
/// would pass identically whether or not #259 exists.
#[test]
fn the_git_admin_dir_grant_does_not_disturb_the_worktrees_own_common_dir_identity() {
    let dir = TempDir::new().expect("tempdir");
    let source = dir.path().join("source");
    let worktree = dir.path().join("worktree");
    let _admin_dir = real_worktree(&source, &worktree, "sergeant/w-codex");

    let expected = canonical_git_common_dir(&source).expect("source common dir");
    let observed = canonical_git_common_dir(&worktree).expect("worktree common dir");
    assert_eq!(
        expected, observed,
        "a linked worktree's common dir must still agree with its source checkout's"
    );
}

/// #262: codex-cli's own documented `-c
/// sandbox_workspace_write.network_access=true` override is composed only
/// when a profile/config explicitly opts in — never by default, and never
/// implied by anything else this adapter composes.
#[test]
fn network_access_is_absent_by_default_and_present_when_configured() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.replays(AGENT_MESSAGE_TURN);
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let request = start_request(dir.path());
    let prepared = backend.prepare(&request).expect("prepare");
    backend.launch(&prepared).expect("launch");
    let launches = stub.wait_for_launches(1);
    assert!(
        !launches[0].has("-c"),
        "the network override must never compose unless explicitly configured"
    );

    let dir2 = TempDir::new().expect("tempdir");
    let stub2 = StubCodex::passing(dir2.path());
    stub2.replays(AGENT_MESSAGE_TURN);
    let mut config = config_for(&stub2, dir2.path(), &dir2.path().join("codex-home"));
    config.workspace_write_network_access = true;
    let backend2 = CodexBackend::new(config);
    let request2 = start_request(dir2.path());
    let prepared2 = backend2.prepare(&request2).expect("prepare");
    backend2.launch(&prepared2).expect("launch");
    let launches2 = stub2.wait_for_launches(1);
    assert_eq!(
        launches2[0].value_after("-c"),
        Some("sandbox_workspace_write.network_access=true")
    );
}

/// #259's fail-closed half: a mutation-shaped request (bindings present)
/// whose git admin dir grant cannot be resolved — here, a `worktree_path`
/// that was never a linked worktree git created, so it has no `.git` file
/// naming a `gitdir` — is refused at PREPARE, before anything is spawned.
/// Admitting the Work anyway would let it edit files and then mechanically
/// fail to commit them, retiring `completed_dirty` with no durable branch
/// commit (issue #259's own acceptance criteria).
#[test]
fn prepare_refuses_a_mutation_shaped_request_whose_git_admin_dir_is_unresolvable() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let not_a_worktree = dir.path().join("not-a-worktree");
    std::fs::create_dir_all(&not_a_worktree).expect("dir");

    let mut request = start_request(&not_a_worktree);
    request.bindings = vec![BindingSummary {
        repository: "solo".to_string(),
        worktree_path: not_a_worktree.clone(),
        work_branch: "b".to_string(),
        base_branch: None,
        base_sha: "0".repeat(40),
    }];
    let err = backend.prepare(&request).expect_err("must refuse");
    match err {
        BackendError::Failed { detail, .. } => {
            assert!(detail.contains("#259"), "{detail}");
        }
        other => panic!("expected Failed, got {other:?}"),
    }
    assert!(
        stub.launches().is_empty(),
        "a refused prepare must spawn nothing"
    );
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

/// §15's fail-closed invariant, end-to-end through the real
/// `CodexBackend`/`TurnReader`/`StubCodex` machinery — not just the pure
/// `classify_terminal` unit test (`a_stream_with_no_terminal_classifies_
/// unknown_and_carries_exit_and_stderr`). The stub's process dies (a
/// non-zero exit, unrequested) after `thread.started`/`turn.started` but
/// before any `turn.completed`/`turn.failed` line ever arrives — the same
/// fail-closed row issue #46 was filed about on Claude. Ambiguity must
/// surface as `NativeState::Unknown` with the raw evidence attached, never
/// an invented verdict.
#[test]
fn codex_a_turn_that_dies_without_a_terminal_is_ambiguous_not_a_verdict() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    let thread_id = fresh_thread_id();
    stub.replays(&format!(
        "{{\"type\":\"thread.started\",\"thread_id\":\"{thread_id}\"}}\n\
         {{\"type\":\"turn.started\"}}\n"
    ))
    .exits_with(1);
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");

    let observation = wait_for_settled(&backend, &handle);
    assert_eq!(
        observation.native,
        NativeState::Unknown,
        "no terminal line + no interrupt requested = ambiguity: {observation:?}"
    );
    assert_eq!(
        observation.signal,
        sergeant_rs::backend::BackendSignal::Running,
        "no verdict is invented"
    );
    let evidence = observation.evidence.expect("evidence must be present");
    assert!(evidence.contains("exit_code"), "{evidence}");
    assert!(
        evidence.contains("raw="),
        "the raw stream must be archived even when no conclusion could be drawn from it: \
         {evidence}"
    );
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

/// The deterministic half of §5.5's `interrupt: true` claim (the live test
/// is `live_codex_interrupt_leaves_the_conversation_resumable`): a turn's
/// shell commands run as *children of the codex process*, so a plain
/// `child.kill()` on just the direct child would leave any grandchild
/// (exactly what `/bin/bash -lc '…'` spawns) running. INTERRUPT's whole
/// justification is that it kills the turn's process *group* instead
/// (`kill_process_group`, §5.5) — this proves that mechanism, not just that
/// `interrupt()` returns `Ok`.
#[test]
fn codex_interrupt_kills_the_process_group() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.replays(AGENT_MESSAGE_TURN)
        .hangs_after_replay()
        .spawns_a_grandchild();
    let backend = CodexBackend::new(config_for(
        &stub,
        dir.path(),
        &dir.path().join("codex-home"),
    ));
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");

    let grandchild_pid = stub.wait_for_grandchild_pid();
    assert!(
        pid_alive(grandchild_pid),
        "the grandchild must be running before INTERRUPT, or this test proves nothing"
    );
    // Recorded, not asserted, and deliberately so. This test owns the
    // leader-*alive* ordering and the one below owns the leader-exited one,
    // but which ordering a given host actually produced is a *fact about the
    // failure*, not a precondition worth failing on: a host that ends the
    // turn earlier than this one does should still see INTERRUPT kill the
    // group, and if it doesn't, this line is what says which of the two
    // orderings was really under test.
    let before = format!(
        "turn state at INTERRUPT: {:?}\n  {}",
        backend.observe(&handle).expect("observe").native,
        interrupt_diagnosis("before", grandchild_pid),
    );

    backend.interrupt(&handle).expect("interrupt").wait();

    assert_group_died(
        grandchild_pid,
        "the leader was still running (`exec sleep 30`)",
        &before,
    );
}

/// The same §5.5 promise, in the ordering the leader-alive test above cannot
/// reach: the turn's **leader exits first**, and INTERRUPT arrives after.
///
/// This is the shape a real codex turn takes every time it finishes normally
/// while a command it started keeps running detached (`… &>/dev/null &`) —
/// and it is a *different code path* through the adapter, not a faster
/// version of the same one. The leader's exit closes the turn's stdout, the
/// reader thread reaps and files the outcome, and the execution's turn state
/// leaves `InFlight` — so an INTERRUPT that reaches the process group only by
/// way of a live direct child reaches nothing at all, and the grandchild the
/// group kill exists to kill outlives the interrupt entirely.
///
/// The `detaches_its_grandchild` stub mode is what makes that ordering
/// deterministic rather than a race; `wait_for_kind` then pins the reap as
/// having *already happened* before INTERRUPT is called. Both halves stay:
/// the leader-alive path and the leader-exited path are each one real
/// ordering of §5.5, and only keeping both keeps either from regressing.
#[test]
fn codex_interrupt_kills_the_process_group_after_the_leader_exited() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.replays(AGENT_MESSAGE_TURN)
        .spawns_a_grandchild()
        .detaches_its_grandchild();
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

    let grandchild_pid = stub.wait_for_grandchild_pid();
    // The turn is over — this is the whole point of this test's ordering.
    // `conversation.turn.ended` is emitted only after the reader has reaped
    // the leader and filed the turn's outcome, so once it has arrived the
    // leader is provably gone *before* INTERRUPT is called.
    let _ended = wait_for_kind(&events, "conversation.turn.ended");
    assert_eq!(
        backend.observe(&handle).expect("observe").native,
        NativeState::Exited,
        "the leader must already have exited before INTERRUPT, or this test proves nothing"
    );
    assert!(
        pid_alive(grandchild_pid),
        "the grandchild must outlive its leader before INTERRUPT, or this test proves nothing"
    );
    let before = format!(
        "turn state at INTERRUPT: {:?}\n  {}",
        NativeState::Exited,
        interrupt_diagnosis("before", grandchild_pid),
    );

    backend.interrupt(&handle).expect("interrupt").wait();

    assert_group_died(
        grandchild_pid,
        "the leader had already exited and been reaped",
        &before,
    );
}

/// The bounded poll both INTERRUPT tests end with. SIGKILL is sent
/// synchronously by `kill_process_group`, but the kernel reaping it is not
/// instant from this test's vantage point — same shape as every other
/// `wait_for_*` helper in this file.
///
/// On failure it carries the whole diagnosis rather than a verdict: the
/// grandchild's process facts as they stood *before* INTERRUPT (whose `pgid`
/// field is the one the group kill had to hit) and as they stand now. A CI
/// run that fails here is meant to be readable without a second run.
fn assert_group_died(grandchild_pid: u32, ordering: &str, before: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if !pid_alive(grandchild_pid) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "grandchild pid {grandchild_pid} survived INTERRUPT ({ordering}): the whole turn \
             process group should have been killed, not just the direct child.\n  {before}\n  \
             {after}",
            after = interrupt_diagnosis("after", grandchild_pid),
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// The whole picture around a grandchild, for a failure message that
/// diagnoses itself rather than just reporting a verdict: the grandchild, the
/// process that forked it (the turn's leader, while it is still alive), and
/// this test process.
///
/// The one field that settles what went wrong is `pgid`. If the grandchild's
/// `pgid` is the leader's pid, `process_group(0)` did its job and the group
/// was there to be killed — a survivor then means the SIGKILL never reached
/// it. If instead it is *this* process's `pgid`, the turn never got a process
/// group of its own and the negated-pid kill named a group that was never
/// the turn's.
fn interrupt_diagnosis(when: &str, grandchild_pid: u32) -> String {
    let mut out = process_facts(&format!("grandchild {when} INTERRUPT"), grandchild_pid);
    if let Some(parent) = ppid_of(grandchild_pid).filter(|parent| *parent > 1) {
        out.push_str("\n  ");
        out.push_str(&process_facts(
            &format!("its parent {when} INTERRUPT"),
            parent,
        ));
    }
    out.push_str("\n  ");
    out.push_str(&process_facts("this test process", std::process::id()));
    out.push_str("\n  ");
    out.push_str(&external_kill_status());
    out
}

/// Whether this host has `kill` as an **executable on `PATH`**, as distinct
/// from the shell builtin of the same name that every POSIX shell has.
///
/// This is the fact that named the cause here, and it is worth the two lines
/// it costs to keep. A host without the executable answers `Command::new
/// ("kill")` with `ENOENT`, so a group kill spawned that way never happens at
/// all — while every other piece of evidence (the group is right, the members
/// are alive, `ps` works) looks exactly like a group kill that was sent and
/// ignored. `kill_process_group` goes through `/bin/sh -c` for that reason;
/// this line is how a future failure says whether that still holds.
fn external_kill_status() -> String {
    match std::process::Command::new("kill")
        .arg("-0")
        .arg(std::process::id().to_string())
        .output()
    {
        Ok(out) => format!(
            "kill(1) as an executable on PATH: spawned, exit {:?}",
            out.status.code()
        ),
        Err(e) => format!(
            "kill(1) as an executable on PATH: NOT SPAWNABLE ({e}) — only the shell builtin \
             exists on this host"
        ),
    }
}

/// The regression that neither INTERRUPT test above can catch on a host that
/// *has* `kill(1)` installed — which is every host this suite ran green on
/// while CI failed on the one that doesn't.
///
/// `kill_process_group` must reach the signal through a shell, whose `kill`
/// is a POSIX-mandated builtin, and never by spawning `kill` as a program off
/// `PATH`. The failure mode of the latter is the worst kind: `ENOENT` at spawn
/// on a host without the executable, no signal sent, and — because the group
/// itself is perfectly well formed — evidence indistinguishable from a
/// SIGKILL that was delivered and ignored. Asserting on the source is crude,
/// but it is the only way to pin a host property this suite cannot vary
/// safely from inside a parallel test process (`set_var` on `PATH` is
/// process-global and would race every other test here).
#[test]
fn the_group_kill_never_depends_on_a_kill_executable_being_installed() {
    // Comment lines are stripped first: the prose right above
    // `kill_process_group` names the very construct being banned, in order to
    // explain why it is banned, and a check that cannot tell code from the
    // comment documenting it would forbid saying so.
    let source: String = include_str!("../src/backend/codex.rs")
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !source.contains("Command::new(\"kill\")"),
        "the process-group kill must not spawn a bare `kill` executable: it is a package a \
         host need not install, and `Command` reports its absence as an ENOENT that a dropped \
         result silently turns into 'INTERRUPT killed nothing'. Go through a shell builtin."
    );
    assert!(
        source.contains("kill -KILL -{pgid}"),
        "the process-group kill must still SIGKILL the negated group id (§5.5)"
    );
}

/// The pid that forked `pid`, as this host reports it — `1` (or nothing) once
/// the real parent has exited and init has adopted it.
fn ppid_of(pid: u32) -> Option<u32> {
    let out = std::process::Command::new("ps")
        .args(["-o", "ppid=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}

/// Everything this host will say about a pid. `/proc/<pid>/stat` verbatim
/// where there is a `/proc` (its fields 3, 4 and 5 are state, ppid and
/// **pgid** — the group the kill had to name); the POSIX `ps` columns
/// everywhere, since `matrix.yml` runs this same suite on macOS.
fn process_facts(label: &str, pid: u32) -> String {
    let ps = match std::process::Command::new("ps")
        .args([
            "-o",
            "pid=,ppid=,pgid=,state=,comm=",
            "-p",
            &pid.to_string(),
        ])
        .output()
    {
        Ok(out) if out.status.success() => {
            format!("{:?}", String::from_utf8_lossy(&out.stdout).trim())
        }
        Ok(_) => "<ps reports no such pid>".to_string(),
        Err(e) => format!("<ps failed: {e}>"),
    };
    let stat = match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
        Ok(text) => format!("{:?}", text.trim()),
        Err(e) => format!("<unreadable: {e}>"),
    };
    format!("{label}: ps[pid ppid pgid state comm]={ps}; /proc/{pid}/stat={stat}")
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
    let handle = backend.launch(&prepared).expect("launch");
    let launches = stub.wait_for_launches(1);
    assert_eq!(
        launches[0].env.get("CODEX_HOME").map(String::as_str),
        Some(profile_home.to_str().unwrap())
    );

    // §3.5's whole point: the profile's CODEX_HOME must not lapse on a
    // *resume* turn — the exact axis `-p/--profile` is refused over, since
    // `exec resume` has no such flag to re-apply it. Turn 1 alone (above)
    // cannot prove this; drive a second, resume-grammar turn and check its
    // env too.
    let thread_id = handle.native_id.clone().expect("thread id from turn 1");
    wait_for_settled(&backend, &handle);
    stub.replays(&plain_turn_naming(&thread_id));
    backend.send(&handle, "turn two").expect("send");
    let launches = stub.wait_for_launches(2);
    assert_eq!(
        launches[1].env.get("CODEX_HOME").map(String::as_str),
        Some(profile_home.to_str().unwrap()),
        "the profile's CODEX_HOME must re-apply on the resume turn too, not just turn 1"
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

// ------------------------------------------- W3 §6.2: app-server, stub-driven

/// A `thread/start` + `turn/start` scripted reply pair good enough to make
/// LAUNCH succeed on the app-server transport with no real `codex` binary —
/// the shared setup every test below builds on.
fn script_appserver_launch(stub: &StubCodex) {
    stub.appserver_scripts_reply(
        "thread/start",
        &[r#"{"id":__ID__,"result":{"thread":{"id":"01a02508-5880-7980-95b7-1d8bc22d5139","status":{"type":"idle"}}}}"#],
    );
    stub.appserver_scripts_reply(
        "turn/start",
        &[r#"{"id":__ID__,"result":{"turn":{"id":"turn-1"}}}"#],
    );
}

/// An app-server backend on `stub`, with budgets short enough that a test
/// which *must* out-wait one of them does not out-wait the reader itself.
/// Every fix below turns a budget expiry into an immediate, evidenced
/// failure, so a passing run never spends these; a regressed one spends
/// `turn_start` exactly once.
fn appserver_backend(stub: &StubCodex, dir: &Path) -> CodexBackend {
    let mut config = config_for(stub, dir, &dir.join("codex-home"));
    config.transport = TransportChoice::AppServerOnly;
    config.appserver_budgets = Some(Budgets {
        handshake: Duration::from_secs(10),
        thread_start: Duration::from_secs(10),
        turn_start: Duration::from_secs(5),
        interrupt: Duration::from_secs(5),
        stderr_drain: Duration::from_secs(2),
    });
    CodexBackend::new(config)
}

/// Poll OBSERVE until `ready` accepts what it sees, then return it.
fn wait_for_observation(
    backend: &CodexBackend,
    handle: &ExecutionHandle,
    what: &str,
    ready: impl Fn(&sergeant_rs::backend::Observation) -> bool,
) -> sergeant_rs::backend::Observation {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let observation = backend.observe(handle).expect("observe");
        if ready(&observation) {
            return observation;
        }
        assert!(
            Instant::now() < deadline,
            "{what}: never observed; last evidence was {:?} (native {:?}, signal {:?})",
            observation.evidence,
            observation.native,
            observation.signal,
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Every event of `kind` captured so far.
fn events_of_kind(events: &Arc<Mutex<Vec<EventDraft>>>, kind: &str) -> Vec<EventDraft> {
    events
        .lock()
        .expect("event capture lock")
        .iter()
        .filter(|e| e.kind == kind)
        .cloned()
        .collect()
}

/// Wait for one event of `kind` that `matches` accepts.
fn wait_for_event(
    events: &Arc<Mutex<Vec<EventDraft>>>,
    kind: &str,
    what: &str,
    matches: impl Fn(&EventDraft) -> bool,
) -> EventDraft {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(found) = events_of_kind(events, kind).into_iter().find(&matches) {
            return found;
        }
        assert!(
            Instant::now() < deadline,
            "{what}: no matching {kind} event arrived; saw {:?}",
            events_of_kind(events, kind)
                .iter()
                .map(|e| e.payload.clone())
                .collect::<Vec<_>>()
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// §3.6 test 4: the captured `thread/start` params carry exactly `cwd`,
/// `sandbox`, `approvalPolicy`, and `runtimeWorkspaceRoots` naming cwd plus
/// the one binding outside it — never the inside-cwd binding (already
/// covered by cwd), never anything fabricated.
#[test]
fn appserver_thread_start_names_exactly_the_works_surfaces() {
    let dir = TempDir::new().expect("tempdir");
    let outside = TempDir::new().expect("tempdir outside");
    // #259: `prepare` now resolves every binding's git admin dir and
    // refuses if it cannot, so both bindings here must be real linked
    // worktrees (`real_worktree`, the same fixture the #259 tests use) --
    // not bare paths with no `.git` file.
    let inside_source = TempDir::new().expect("tempdir inside source");
    let outside_source = TempDir::new().expect("tempdir outside source");
    let inside_worktree = dir.path().join("inside-binding");
    let outside_worktree = outside.path().join("outside-binding");
    real_worktree(inside_source.path(), &inside_worktree, "b1");
    real_worktree(outside_source.path(), &outside_worktree, "b2");

    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    script_appserver_launch(&stub);
    let mut config = config_for(&stub, dir.path(), &dir.path().join("codex-home"));
    config.transport = TransportChoice::AppServerOnly;
    let backend = CodexBackend::new(config);
    let mut request = start_request(dir.path());
    request.bindings = vec![
        BindingSummary {
            repository: "inside".to_string(),
            worktree_path: inside_worktree,
            work_branch: "b1".to_string(),
            base_branch: None,
            base_sha: "0".repeat(40),
        },
        BindingSummary {
            repository: "outside".to_string(),
            worktree_path: outside_worktree.clone(),
            work_branch: "b2".to_string(),
            base_branch: None,
            base_sha: "0".repeat(40),
        },
    ];
    let prepared = backend.prepare(&request).expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");

    let requests = stub.appserver_requests();
    let thread_start = requests
        .iter()
        .find(|r| r["method"] == "thread/start")
        .expect("thread/start was sent");
    let params = &thread_start["params"];
    assert_eq!(params["cwd"], dir.path().to_string_lossy().as_ref());
    assert_eq!(params["sandbox"], "workspace-write");
    assert_eq!(params["approvalPolicy"], "never");
    let roots: Vec<String> = params["runtimeWorkspaceRoots"]
        .as_array()
        .expect("runtimeWorkspaceRoots is an array")
        .iter()
        .map(|v| v.as_str().unwrap_or("").to_string())
        .collect();
    assert_eq!(
        roots,
        vec![
            dir.path().to_string_lossy().to_string(),
            outside_worktree.to_string_lossy().to_string(),
            inside_source
                .path()
                .join(".git")
                .join("worktrees")
                .join("inside-binding")
                .to_string_lossy()
                .to_string(),
            outside_source
                .path()
                .join(".git")
                .join("worktrees")
                .join("outside-binding")
                .to_string_lossy()
                .to_string(),
        ],
        "cwd, the one outside binding, then #259's git admin dir grants for every binding \
         in order -- never the estate root, never anything fabricated"
    );

    backend.stop(&handle).expect("stop").wait();
}

/// Test 21: a child that dies mid-turn (closes its stdout with no
/// `turn/completed` ever sent) must resolve to a terminal, fail-closed
/// `AmbiguousUnknown` observation — never leave the turn `InFlight` forever
/// (§3.4 point 3 / §15's invariant).
///
/// `native` is `Exited`, not `Unknown`: this observation reaped the child and
/// holds its status, and `NativeState::Unknown`'s own definition is "the
/// backend cannot tell". What fails closed here is the *signal* — `Running`,
/// never a stage verdict — which is the guarantee §5.2 actually asks for.
#[test]
fn appserver_child_death_mid_turn_is_ambiguous_not_completed() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    script_appserver_launch(&stub);
    stub.appserver_exits_after("turn/start");
    let backend = appserver_backend(&stub, dir.path());
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");

    let observation = wait_for_observation(
        &backend,
        &handle,
        "the turn never resolved out of InFlight after the child's stdout closed",
        |observation| {
            observation
                .evidence
                .as_deref()
                .is_some_and(|evidence| evidence.contains("no turn/completed observed"))
        },
    );
    match observation.signal {
        sergeant_rs::backend::BackendSignal::Running => {}
        other => panic!("expected Running (fail-closed, no stage verdict), got {other:?}"),
    }
    assert_eq!(
        observation.native,
        NativeState::Exited,
        "the child was reaped by this very observation; reporting Unknown over the top of \
         that proof is the adapter lying about what it can see"
    );
}

/// Finding 2: the fail-closed routes owe the journal the same
/// `conversation.turn.ended` the happy path emits — exec's own finalizer
/// states the invariant ("every turn ends with this event, however it
/// ended"), and a turn that died mid-flight used to end with a bare
/// `harness_error` and nothing else, throwing away the item tallies the
/// accumulator was holding at the moment it was settled.
#[test]
fn appserver_child_death_mid_turn_still_journals_turn_ended() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    stub.appserver_scripts_reply(
        "thread/start",
        &[r#"{"id":__ID__,"result":{"thread":{"id":"01a02508-5880-7980-95b7-1d8bc22d5139","status":{"type":"idle"}}}}"#],
    );
    // One completed item, then the child dies with no terminal: the tallies
    // below are exactly what the settlement was holding.
    stub.appserver_scripts_reply(
        "turn/start",
        &[
            r#"{"id":__ID__,"result":{"turn":{"id":"turn-1"}}}"#,
            r#"{"method":"item/completed","params":{"item":{"id":"i1","type":"agentMessage","text":"half a thought"}}}"#,
        ],
    );
    stub.appserver_exits_after("turn/start");
    let backend = appserver_backend(&stub, dir.path());
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");

    let ended = wait_for_event(
        &events,
        "conversation.turn.ended",
        "a turn that died mid-flight must still end with conversation.turn.ended",
        |_| true,
    );
    assert_eq!(
        ended.payload["outcome"], "ambiguous_unknown",
        "the ambiguous outcome is the fact this event exists to carry: {:?}",
        ended.payload
    );
    assert_eq!(ended.payload["interrupted"], false);
    assert_eq!(
        ended.payload["message_items"], 1,
        "the item the accumulator had already decoded travels with the settlement, not \
         into the bin: {:?}",
        ended.payload
    );
    // The harness_error is *additional* evidence, never a substitute.
    wait_for_event(
        &events,
        "conversation.turn.harness_error",
        "the child-death harness_error still lands alongside it",
        |e| e.payload["phase"] == "child_exited_mid_turn",
    );

    backend.stop(&handle).expect("stop").wait();
}

/// Finding 1: `turn/start`'s failure path used to be the one writer of the
/// turn cell that was not guarded on what the cell currently held. A child
/// that dies with `turn/start` in flight settles the turn from the reader
/// thread (fail-closed, `AmbiguousUnknown`); the blind `= Idle` rollback then
/// erased that settlement, and OBSERVE went back to reporting "has not run a
/// turn yet" about a turn that had run, ended, and been journaled.
///
/// Both halves of the fix are pinned here: the failing `turn/start` reports
/// the closed stream (it was drained on EOF, not left to expire its own
/// budget), and the settled cell survives the rollback.
#[test]
fn appserver_a_failed_turn_start_never_clobbers_a_settled_turn() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    stub.appserver_scripts_reply(
        "thread/start",
        &[r#"{"id":__ID__,"result":{"thread":{"id":"01a02508-5880-7980-95b7-1d8bc22d5139","status":{"type":"idle"}}}}"#],
    );
    // Turn 1 answers and completes normally. Turn 2's `turn/start` is never
    // answered at all: the child dies with the request in flight.
    stub.appserver_scripts_reply(
        "turn/start",
        &[
            r#"{"id":__ID__,"result":{"turn":{"id":"turn-1"}}}"#,
            r#"{"method":"turn/completed","params":{"turn":{"id":"turn-1","status":"completed"}}}"#,
        ],
    );
    stub.appserver_exits_before("turn/start", 2);
    let backend = appserver_backend(&stub, dir.path());
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    wait_for_observation(&backend, &handle, "turn 1 never completed", |observation| {
        matches!(
            observation.signal,
            sergeant_rs::backend::BackendSignal::StageCompleted { .. }
        )
    });

    let refusal = backend
        .send(&handle, "second turn")
        .expect_err("the child died before answering turn/start");
    let refusal = refusal.to_string();
    assert!(
        refusal.contains("the app-server child's output stream closed"),
        "the pending request is drained the moment the stream ends, so the caller learns \
         the child is gone instead of outliving its own budget; got: {refusal}"
    );

    // No polling: the EOF handler settles the turn *before* the pending
    // senders are drained, so by the time `send` returned, the settlement is
    // already installed. Anything else here is a clobber.
    let observation = backend.observe(&handle).expect("observe");
    let evidence = observation.evidence.clone().unwrap_or_default();
    assert!(
        evidence.contains("no turn/completed observed"),
        "the fail-closed settlement of turn 2 must survive turn/start's rollback; got: \
         {evidence}"
    );
    assert!(
        !evidence.contains("has not run a turn yet"),
        "the rollback clobbered a settled turn back to Idle: {evidence}"
    );

    backend.stop(&handle).expect("stop").wait();
}

/// Finding 4: the ambiguous terminal is the one outcome with no stream
/// evidence of its own, so it must carry every scrap of process evidence the
/// adapter holds — exit status, the last stream `error`, and the child's own
/// stderr tail. Mirrors exec's own ambiguous arm, which has carried all three
/// since W1.
#[test]
fn appserver_the_ambiguous_terminal_carries_exit_status_stderr_and_last_error() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    stub.writes_stderr("codex: bwrap: No such file or directory\n");
    stub.appserver_scripts_reply(
        "thread/start",
        &[r#"{"id":__ID__,"result":{"thread":{"id":"01a02508-5880-7980-95b7-1d8bc22d5139","status":{"type":"idle"}}}}"#],
    );
    stub.appserver_scripts_reply(
        "turn/start",
        &[
            r#"{"id":__ID__,"result":{"turn":{"id":"turn-1"}}}"#,
            r#"{"method":"error","params":{"message":"sandbox helper exited before the turn began"}}"#,
        ],
    );
    stub.appserver_exits_after_with_code("turn/start", 7);
    let backend = appserver_backend(&stub, dir.path());
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");

    // The stderr drain is a second thread; poll until its tail has arrived
    // rather than assuming it beat the reader to the finish line.
    let observation = wait_for_observation(
        &backend,
        &handle,
        "the ambiguous terminal never carried the child's stderr",
        |observation| {
            observation
                .evidence
                .as_deref()
                .is_some_and(|evidence| evidence.contains("bwrap"))
        },
    );
    let evidence = observation.evidence.unwrap_or_default();
    assert!(
        evidence.contains("code=Some(7)"),
        "the child's own exit status is the first thing to ask for and used to be \
         discarded by the liveness peek; got: {evidence}"
    );
    assert!(
        evidence.contains("sandbox helper exited before the turn began"),
        "the last stream error is often the only statement of *why* no terminal came; \
         got: {evidence}"
    );
}

/// Finding 3, path 2, reached the way production reaches it: STOP. §5.7's own
/// note — on app-server the turn's process and the execution's process are the
/// same child, so STOP is `turn/interrupt` followed by sergeant killing that
/// child itself. The harness accepts the interrupt and is then killed before
/// it can send `turn/completed`, which is not a failed RPC and must not be
/// reported as one.
///
/// No polling: STOP's completion joins the reader thread, so by the time
/// `wait()` returns, the EOF that settles this turn has already been handled.
#[test]
fn appserver_the_routine_stop_settles_as_an_acknowledged_interrupt_cut_short() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    script_appserver_launch(&stub);
    stub.appserver_scripts_reply("turn/interrupt", &[r#"{"id":__ID__,"result":{}}"#]);
    let backend = appserver_backend(&stub, dir.path());
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");

    backend.stop(&handle).expect("stop").wait();

    let observation = backend.observe(&handle).expect("observe");
    match observation.signal {
        sergeant_rs::backend::BackendSignal::Running => {}
        other => panic!("expected Running (resumable, no stage verdict), got {other:?}"),
    }
    let evidence = observation.evidence.unwrap_or_default();
    assert!(
        evidence.contains(
            "turn/interrupt was acknowledged, but the child's stdout closed before \
             turn/completed carried the harness's own verdict"
        ),
        "STOP's own interrupt was accepted by the harness; got: {evidence}"
    );
    assert!(
        !evidence.contains("RPC failed") && !evidence.contains("never answered it"),
        "nothing about the routine STOP path failed or went unanswered; got: {evidence}"
    );

    // Finding 2 on this route too: the turn ended, so it ends with the event
    // that says a turn ended.
    let ended = wait_for_event(
        &events,
        "conversation.turn.ended",
        "the turn STOP settled still ends with conversation.turn.ended",
        |_| true,
    );
    assert_eq!(ended.payload["outcome"], "interrupted_running");
    assert_eq!(ended.payload["interrupted"], true);
}

/// Finding 3, path 2 again, by the other route: INTERRUPT alone, with a child
/// that answers and then dies of its own accord rather than being killed.
/// `turn/interrupt` returned `Ok` — the harness accepted it — and the child's
/// stdout then closed before `turn/completed` could carry the harness's own
/// verdict. Same outcome as path 1, a different fact, and the arm used to
/// claim path 1's sentence here too ("turn/interrupt's own RPC never confirmed
/// it"), which is false of an RPC that returned `Ok`.
#[test]
fn appserver_an_interrupt_acknowledged_then_cut_off_says_so_and_nothing_more() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    script_appserver_launch(&stub);
    stub.appserver_scripts_reply("turn/interrupt", &[r#"{"id":__ID__,"result":{}}"#]);
    stub.appserver_exits_after("turn/interrupt");
    let backend = appserver_backend(&stub, dir.path());
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");

    backend.interrupt(&handle).expect("interrupt").wait();

    let observation = wait_for_observation(
        &backend,
        &handle,
        "the turn stayed InFlight after the acknowledged interrupt's stream closed",
        |observation| {
            observation
                .evidence
                .as_deref()
                .is_some_and(|evidence| !evidence.contains("in flight on thread"))
        },
    );
    let evidence = observation.evidence.unwrap_or_default();
    assert!(
        evidence.contains(
            "turn/interrupt was acknowledged, but the child's stdout closed before \
             turn/completed carried the harness's own verdict"
        ),
        "an acknowledged interrupt must not be reported as a failed RPC; got: {evidence}"
    );
    assert!(
        !evidence.contains("RPC failed"),
        "nothing about this path failed; got: {evidence}"
    );
}

/// Finding 5: notifications that arrive after a turn is already settled used
/// to be dropped where they stood — including a genuine `turn/completed`
/// buffered behind an earlier settlement. The settlement is still final (a
/// turn is never re-settled), but the late lines are journaled: one immediate
/// report for the terminal, one end-of-stream summary for the tally.
#[test]
fn appserver_a_terminal_arriving_after_settlement_is_journaled_never_resettled() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    stub.appserver_scripts_reply(
        "thread/start",
        &[r#"{"id":__ID__,"result":{"thread":{"id":"01a02508-5880-7980-95b7-1d8bc22d5139","status":{"type":"idle"}}}}"#],
    );
    // One burst, read in order by the one reader thread: the turn settles on
    // the first `turn/completed`, and everything after it is late. The late
    // terminal deliberately disagrees with the settled one (`failed` vs
    // `completed`) so that "never re-settled" is a fact this test can see.
    stub.appserver_scripts_reply(
        "turn/start",
        &[
            r#"{"id":__ID__,"result":{"turn":{"id":"turn-1"}}}"#,
            r#"{"method":"item/completed","params":{"item":{"id":"i1","type":"agentMessage","text":"done"}}}"#,
            r#"{"method":"turn/completed","params":{"turn":{"id":"turn-1","status":"completed"}}}"#,
            r#"{"method":"item/completed","params":{"item":{"id":"i2","type":"agentMessage","text":"late"}}}"#,
            r#"{"method":"turn/completed","params":{"turn":{"id":"turn-1","status":"failed","error":{"message":"too late"}}}}"#,
        ],
    );
    let backend = appserver_backend(&stub, dir.path());
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");

    let late_terminal = wait_for_event(
        &events,
        "conversation.turn.harness_error",
        "a real terminal arriving after settlement must never be dropped silently",
        |e| e.payload["phase"] == "post_settlement_terminal",
    );
    assert_eq!(late_terminal.payload["method"], "turn/completed");
    assert_eq!(late_terminal.payload["settled_as"], "completed");

    // STOP ends the child's stream, which is when the tally is final.
    backend.stop(&handle).expect("stop").wait();
    let summary = wait_for_event(
        &events,
        "conversation.turn.harness_error",
        "the post-settlement tally is summarized once, at end of stream",
        |e| e.payload["phase"] == "post_settlement_lines",
    );
    assert_eq!(summary.payload["lines"], 2);
    assert_eq!(summary.payload["terminal_seen"], true);
    assert_eq!(
        summary.payload["methods"],
        serde_json::json!(["item/completed", "turn/completed"])
    );

    // Exactly one settlement, and it is the first terminal's.
    let ended = events_of_kind(&events, "conversation.turn.ended");
    assert_eq!(
        ended.len(),
        1,
        "a settled turn is never re-settled by a line that arrives after it: {:?}",
        ended.iter().map(|e| e.payload.clone()).collect::<Vec<_>>()
    );
    assert_eq!(ended[0].payload["outcome"], "completed");
}

/// Finding 5's other exit. End of stream is not the only way a settled turn's
/// post-settlement tally leaves the building: the next turn on the same thread
/// overwrites the cell that tally lives in, and on a thread that runs more
/// than one turn it gets there first. Reporting it only at EOF would drop it
/// silently on exactly the flow the transport exists for.
#[test]
fn appserver_late_lines_are_summarized_when_the_next_turn_displaces_them() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    stub.appserver_scripts_reply(
        "thread/start",
        &[r#"{"id":__ID__,"result":{"thread":{"id":"01a02508-5880-7980-95b7-1d8bc22d5139","status":{"type":"idle"}}}}"#],
    );
    // The same burst every `turn/start` replays: settle, then two late lines.
    stub.appserver_scripts_reply(
        "turn/start",
        &[
            r#"{"id":__ID__,"result":{"turn":{"id":"turn-1"}}}"#,
            r#"{"method":"turn/completed","params":{"turn":{"id":"turn-1","status":"completed"}}}"#,
            r#"{"method":"item/completed","params":{"item":{"id":"i2","type":"agentMessage","text":"late"}}}"#,
            r#"{"method":"turn/completed","params":{"turn":{"id":"turn-1","status":"failed","error":{"message":"too late"}}}}"#,
        ],
    );
    stub.appserver_scripts_reply("turn/interrupt", &[r#"{"id":__ID__,"result":{}}"#]);
    let backend = appserver_backend(&stub, dir.path());
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");

    // Waiting for the late *terminal* is what makes the tally below exact:
    // it is the last of turn 1's four lines, so both late lines have landed.
    wait_for_event(
        &events,
        "conversation.turn.harness_error",
        "turn 1's late terminal",
        |e| e.payload["phase"] == "post_settlement_terminal",
    );
    assert!(
        events_of_kind(&events, "conversation.turn.harness_error")
            .iter()
            .all(|e| e.payload["phase"] != "post_settlement_lines"),
        "nothing has displaced or ended turn 1 yet, so its tally is not final"
    );

    backend.send(&handle, "second turn").expect("send");
    let summary = wait_for_event(
        &events,
        "conversation.turn.harness_error",
        "the displaced turn's tally must be journaled, not overwritten in silence",
        |e| e.payload["phase"] == "post_settlement_lines",
    );
    assert_eq!(summary.payload["lines"], 2);
    assert_eq!(summary.payload["terminal_seen"], true);

    // Scripted only so this test's own teardown is not a 5-second wait on the
    // interrupt budget: turn 2 may still be in flight when STOP asks.
    backend.stop(&handle).expect("stop").wait();
}

/// A stray notification for a turn that already displaced its own cell must
/// not be folded into the *new* turn's evidence — not even a `turn/completed`
/// naming the superseded turn's own id. Without the guard, this stray
/// `turn/completed{status:"failed"}` for turn 1 would stamp `Terminal::Failed`
/// onto turn 2's still-live accumulator and settle turn 2 on the spot, before
/// turn 2's own genuine completion ever arrives.
#[test]
fn appserver_a_stray_notification_for_the_displaced_turn_never_taints_the_new_one() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    stub.appserver_scripts_reply(
        "thread/start",
        &[r#"{"id":__ID__,"result":{"thread":{"id":"01a02508-5880-7980-95b7-1d8bc22d5139","status":{"type":"idle"}}}}"#],
    );
    // Turn 1: starts as "turn-1" and settles cleanly, completed.
    stub.appserver_scripts_reply(
        "turn/start",
        &[
            r#"{"id":__ID__,"result":{"turn":{"id":"turn-1"}}}"#,
            r#"{"method":"turn/completed","params":{"turn":{"id":"turn-1","status":"completed"}}}"#,
        ],
    );
    let backend = appserver_backend(&stub, dir.path());
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");

    wait_for_event(
        &events,
        "conversation.turn.ended",
        "turn 1 settles before turn 2 is ever sent",
        |e| e.payload["outcome"] == "completed",
    );

    // Turn 2: starts as "turn-2", but before its own real completion, a
    // straggler for turn 1 arrives -- a duplicate `turn/completed` naming
    // turn 1's own id with a *disagreeing* status, exactly the shape a
    // buffered late line from the displaced turn would have. Turn 2's own
    // evidence (an `item/completed` and its own `turn/completed`) follows.
    stub.appserver_scripts_reply(
        "turn/start",
        &[
            r#"{"id":__ID__,"result":{"turn":{"id":"turn-2"}}}"#,
            r#"{"method":"turn/completed","params":{"turn":{"id":"turn-1","status":"failed","error":{"message":"stale turn-1 completion, arrived after turn 2 started"}}}}"#,
            r#"{"method":"item/completed","params":{"item":{"type":"agentMessage","id":"msg-2","text":"turn 2 done","phase":"final_answer"},"turnId":"turn-2"}}"#,
            r#"{"method":"turn/completed","params":{"turn":{"id":"turn-2","status":"completed"}}}"#,
        ],
    );
    backend.send(&handle, "second turn").expect("send");

    let stray = wait_for_event(
        &events,
        "conversation.turn.harness_error",
        "the straggler for turn 1 is journaled, not silently merged into turn 2",
        |e| e.payload["phase"] == "stray_notification_for_superseded_turn",
    );
    assert_eq!(stray.payload["method"], "turn/completed");

    // Turn 2 must still reach its own, genuine settlement -- the straggler
    // must not have already finalized it as `failed` in the meantime.
    let deadline = Instant::now() + Duration::from_secs(10);
    let ended = loop {
        let ended = events_of_kind(&events, "conversation.turn.ended");
        if ended.len() >= 2 {
            break ended;
        }
        assert!(
            Instant::now() < deadline,
            "turn 2 never reached its own settlement; saw {:?}",
            ended.iter().map(|e| e.payload.clone()).collect::<Vec<_>>()
        );
        std::thread::sleep(Duration::from_millis(20));
    };
    // Exactly two turns ever ended, and turn 2's own outcome is `completed`
    // -- the straggler's `failed` status never touched it.
    assert_eq!(
        ended.len(),
        2,
        "turn 1 and turn 2 each end exactly once: {:?}",
        ended.iter().map(|e| e.payload.clone()).collect::<Vec<_>>()
    );
    assert_eq!(ended[1].payload["outcome"], "completed");

    let observation = wait_for_observation(
        &backend,
        &handle,
        "turn 2's own completion, not the stray failure, is what OBSERVE reports",
        |observation| {
            observation
                .evidence
                .as_deref()
                .is_some_and(|evidence| !evidence.contains("in flight on thread"))
        },
    );
    match observation.signal {
        sergeant_rs::backend::BackendSignal::StageCompleted { ref summary } => {
            assert_eq!(summary.as_deref(), Some("turn 2 done"));
        }
        other => panic!("expected turn 2's own completion, got {other:?}"),
    }
}

/// A second, redundant `Backend::interrupt` call on the same still-`InFlight`
/// turn must not clobber a first call the harness genuinely acknowledged.
/// The engine itself can produce exactly this sequence -- a ceiling-triggered
/// auto-interrupt, acknowledged, followed by a later human cancel or restart
/// reconcile hitting the same still-running turn -- and before this fix the
/// second call's unconditional write turned a turn the harness had honestly
/// confirmed into one whose evidence says nothing ever answered it.
#[test]
fn appserver_a_second_interrupt_call_does_not_clobber_the_first_acknowledgment() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    script_appserver_launch(&stub);
    stub.appserver_scripts_reply("turn/interrupt", &[r#"{"id":__ID__,"result":{}}"#]);
    // The child answers the one `turn/interrupt` it ever receives, then exits
    // of its own accord -- the same shape as
    // `appserver_an_interrupt_acknowledged_then_cut_off_says_so_and_nothing_more`,
    // with a second, redundant call added right after the first.
    stub.appserver_exits_after("turn/interrupt");
    let backend = appserver_backend(&stub, dir.path());
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");

    backend.interrupt(&handle).expect("first interrupt").wait();
    // Redundant: the engine can legitimately call interrupt (or stop, which
    // calls interrupt itself) again on a turn that is still `InFlight` --
    // a ceiling-triggered auto-interrupt followed by a later human cancel is
    // one real path. This call must neither re-ask the harness nor clobber
    // whatever the first call's own honest answer already recorded, whether
    // it lands while the cell still says `Acknowledged` or after the reader
    // thread has already settled it on the child's exit.
    backend.interrupt(&handle).expect("second interrupt").wait();

    let observation = wait_for_observation(
        &backend,
        &handle,
        "the turn stayed InFlight after the second, redundant interrupt call",
        |observation| {
            observation
                .evidence
                .as_deref()
                .is_some_and(|evidence| !evidence.contains("in flight on thread"))
        },
    );
    let evidence = observation.evidence.unwrap_or_default();
    assert!(
        evidence.contains(
            "turn/interrupt was acknowledged, but the child's stdout closed before \
             turn/completed carried the harness's own verdict"
        ),
        "the first call's genuine acknowledgment must survive a second, redundant call; \
         got: {evidence}"
    );
    assert!(
        !evidence.contains("never answered it") && !evidence.contains("RPC failed"),
        "a second call must not turn an acknowledged interrupt into an unresolved or \
         failed one; got: {evidence}"
    );

    let interrupt_requests = stub
        .appserver_requests()
        .iter()
        .filter(|r| r["method"] == "turn/interrupt")
        .count();
    assert_eq!(
        interrupt_requests, 1,
        "a redundant interrupt call must not re-ask the harness once the first \
         call already got an honest answer"
    );
}

/// Test 22: when `turn/interrupt` itself fails, the adapter's own documented
/// fallback (§2.2) kills the process group and journals
/// `phase:"interrupt_downgraded"` — and, per §15/§3.4 point 3, must also
/// resolve the turn out of `InFlight` rather than leave OBSERVE reporting a
/// hung turn forever just because the RPC that would have confirmed the
/// interrupt never came back clean.
#[test]
fn appserver_interrupt_kills_the_process_group_when_the_rpc_fails() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    script_appserver_launch(&stub);
    stub.appserver_scripts_reply(
        "turn/interrupt",
        &[r#"{"id":__ID__,"error":{"code":-32000,"message":"stub-forced interrupt failure"}}"#],
    );
    let mut config = config_for(&stub, dir.path(), &dir.path().join("codex-home"));
    config.transport = TransportChoice::AppServerOnly;
    let backend = CodexBackend::new(config);
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");

    backend.interrupt(&handle).expect("interrupt").wait();

    let deadline = Instant::now() + Duration::from_secs(10);
    let downgraded = loop {
        if let Some(found) = events
            .lock()
            .unwrap()
            .iter()
            .find(|e| e.payload["phase"] == "interrupt_downgraded")
            .cloned()
        {
            break found;
        }
        assert!(
            Instant::now() < deadline,
            "interrupt_downgraded was never journaled"
        );
        std::thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(downgraded.kind, "conversation.turn.harness_error");

    let observation = wait_for_observation(
        &backend,
        &handle,
        "the turn stayed InFlight forever after the interrupt downgrade",
        |observation| {
            observation
                .evidence
                .as_deref()
                .is_some_and(|evidence| !evidence.contains("in flight on thread"))
        },
    );
    match observation.signal {
        sergeant_rs::backend::BackendSignal::Running => {}
        other => panic!("expected Running (resumable, no stage verdict), got {other:?}"),
    }
    // Finding 3, path 1: this outcome is reachable two ways, and the evidence
    // names the one that happened. Here the RPC really did fail.
    let evidence = observation.evidence.unwrap_or_default();
    assert!(
        evidence.contains(
            "turn/interrupt's own RPC failed and sergeant fell back to the process-group kill"
        ),
        "the downgrade path must name itself; got: {evidence}"
    );

    // The fail-closed settlement journals a turn.ended here too (finding 2).
    let ended = wait_for_event(
        &events,
        "conversation.turn.ended",
        "a turn settled by the interrupt downgrade still ends with conversation.turn.ended",
        |_| true,
    );
    assert_eq!(ended.payload["outcome"], "interrupted_running");
    assert_eq!(ended.payload["interrupted"], true);
}

// ------------------------------------------------- §2 residual-gap coverage
//
// coverage-spec.md §2's stub-driven wave, closing the residual gap left
// after §1's accounting fix (StubCodex/StubCodex-adjacent infrastructure
// only — no live codex, no new coverage-exclusion attributes, no gate
// change). Grouped exactly as the spec's own subsections.

// --------------------------------------------------------- §2a helpers

/// Write a durable rollout file `thread_rollout`/`find_rollout` will locate
/// for `thread_id`, nested a few directories deep under
/// `codex_home/sessions` — the same `YYYY/MM/DD` shape a real `~/.codex`
/// layout uses, and along the way `find_rollout`'s recursive-directory arm
/// (never reachable from a rollout dropped flat in `sessions/`).
fn write_rollout(codex_home: &Path, thread_id: &str) {
    let dir = codex_home
        .join("sessions")
        .join("2026")
        .join("08")
        .join("22");
    std::fs::create_dir_all(&dir).expect("mkdir rollout dir");
    std::fs::write(
        dir.join(format!("rollout-2026-08-22T00-00-00-{thread_id}.jsonl")),
        "{}\n",
    )
    .expect("write rollout fixture");
}

/// Spawn a decoy process whose own argv is exactly `sh -c "read line"`
/// followed by `extra_args` — the same no-fork, blocked-on-a-builtin shape
/// `tests/m4_backends.rs::spawn_turn_stand_in` uses, and for the identical
/// reason: a forked external command transiently duplicates the matching
/// argv (the parent's, mid-`fork`) and turns a liveness assertion into a
/// coin flip under load. This is what makes `turn_liveness`'s *real*
/// wrapper — the one that calls
/// `crate::platform::process::running_processes()` against the actual
/// process table, as opposed to `turn_liveness_among`'s own unit tests
/// (which inject a synthetic `ProcessArgv` list) — honestly testable at
/// all: `argv_names_thread`/`argv_names_surface` match on argv content
/// only, never on the executable being `codex`, so a harmless decoy
/// process satisfies them exactly as a real turn's argv would.
///
/// `zombie_processes` is silenced deliberately: every caller kills and
/// reaps the returned child via [`kill_decoy`], but clippy cannot see across
/// that function boundary, and the deadline `assert!` below is a path where
/// a failing *test* — not this helper — drops the child without waiting,
/// an acceptable leak on an already-failing test run, not the steady-state
/// zombie the lint exists to catch.
#[allow(clippy::zombie_processes)]
fn spawn_decoy_process(extra_args: &[&str]) -> std::process::Child {
    let mut command = std::process::Command::new("sh");
    command.arg("-c").arg("read line");
    for arg in extra_args {
        command.arg(arg);
    }
    let child = command
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn decoy process");
    let pid = child.id();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let execd = sergeant_rs::platform::process::running_processes()
            .into_iter()
            .flatten()
            .any(|process| {
                process.pid == pid
                    && extra_args
                        .iter()
                        .all(|needle| process.argv.iter().any(|arg| arg == needle))
            });
        if execd {
            return child;
        }
        assert!(Instant::now() < deadline, "decoy process never exec'd");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Kill and reap a decoy spawned by [`spawn_decoy_process`]. Blocked on its
/// own `read` builtin with no forked children, so a single `kill()` ends it
/// outright — no leaked grandchild the way a `sleep`-based decoy would risk.
fn kill_decoy(mut child: std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

// coverage-spec.md §2a: `classify_restart`/`turn_liveness`'s live-wrapper
// methods have zero integration coverage today — only the pure decision
// function `turn_liveness_among` is unit tested. These five drive the real
// wrapper (`turn_liveness`, `classify_restart`, `observe`'s `Adopted`
// branch, `resume`'s `Liveness` match) through a real process scan.

#[test]
fn restart_observation_reports_a_live_owned_turn_via_a_decoy_process() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    let codex_home = dir.path().join("codex-home");
    let backend = CodexBackend::new(config_for(&stub, dir.path(), &codex_home));
    let thread_id = fresh_thread_id();
    write_rollout(&codex_home, &thread_id);
    let decoy = spawn_decoy_process(&["exec", "resume", &thread_id]);
    let pid = decoy.id();
    let handle = ExecutionHandle {
        execution_id: "e-unowned-live".to_string(),
        native_id: Some(thread_id.clone()),
    };
    let observation = backend
        .observe(&handle)
        .expect("observe a live, unowned turn");
    kill_decoy(decoy);
    assert_eq!(observation.native, NativeState::Running);
    match observation.signal {
        sergeant_rs::backend::BackendSignal::Blocked { reason } => {
            assert!(
                reason.contains(&pid.to_string()),
                "must name the pid: {reason}"
            );
            assert!(reason.contains("unowned"), "must say unowned: {reason}");
        }
        other => panic!("expected Blocked, got {other:?}"),
    }
}

#[test]
fn restart_observation_reports_surface_ambiguous_for_a_first_turn_decoy() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    let codex_home = dir.path().join("codex-home");
    let backend = CodexBackend::new(config_for(&stub, dir.path(), &codex_home));
    let thread_id = fresh_thread_id();
    write_rollout(&codex_home, &thread_id);
    let handle = ExecutionHandle {
        execution_id: "e-surface-ambiguous".to_string(),
        native_id: Some(thread_id.clone()),
    };
    // Re-adopt with nothing live yet, so RESUME's own liveness check passes
    // and OBSERVE afterward goes through the `Adopted` branch — the only
    // path that calls `classify_restart` with `Some(cwd)`, since an
    // un-adopted OBSERVE (the previous test) never has a surface to check
    // a first-turn decoy against.
    let request = ResumeRequest::new("w-surface-ambiguous", dir.path());
    backend
        .resume(&handle, &request)
        .expect("re-adopt with no live process");
    let cwd_str = dir.path().to_string_lossy().into_owned();
    let decoy = spawn_decoy_process(&["exec", "-C", &cwd_str]);
    let pid = decoy.id();
    let observation = backend.observe(&handle).expect("observe");
    kill_decoy(decoy);
    assert_eq!(observation.native, NativeState::Unknown);
    match observation.signal {
        sergeant_rs::backend::BackendSignal::Blocked { reason } => {
            assert!(
                reason.contains(&pid.to_string()),
                "must name the pid: {reason}"
            );
            assert!(
                reason.contains("cannot be established from its argv"),
                "{reason}"
            );
        }
        other => panic!("expected Blocked, got {other:?}"),
    }
}

#[test]
fn restart_observation_reports_dead_with_durable_rollout() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    let codex_home = dir.path().join("codex-home");
    let backend = CodexBackend::new(config_for(&stub, dir.path(), &codex_home));

    // Unowned wording: never registered with this daemon at all.
    let thread_unowned = fresh_thread_id();
    write_rollout(&codex_home, &thread_unowned);
    let handle_unowned = ExecutionHandle {
        execution_id: "e-dead-unowned".to_string(),
        native_id: Some(thread_unowned),
    };
    let observation = backend
        .observe(&handle_unowned)
        .expect("observe a dead, unowned thread");
    assert_eq!(observation.native, NativeState::Exited);
    match observation.signal {
        sergeant_rs::backend::BackendSignal::Blocked { reason } => {
            assert!(
                reason.contains("in-flight turn's outcome is unknown"),
                "{reason}"
            );
        }
        other => panic!("expected Blocked, got {other:?}"),
    }

    // Adopted wording: re-adopted, then observed with nothing live.
    let thread_adopted = fresh_thread_id();
    write_rollout(&codex_home, &thread_adopted);
    let handle_adopted = ExecutionHandle {
        execution_id: "e-dead-adopted".to_string(),
        native_id: Some(thread_adopted),
    };
    let request = ResumeRequest::new("w-dead-adopted", dir.path());
    backend.resume(&handle_adopted, &request).expect("re-adopt");
    let observation = backend
        .observe(&handle_adopted)
        .expect("observe a dead, adopted thread");
    assert_eq!(observation.native, NativeState::Exited);
    match observation.signal {
        sergeant_rs::backend::BackendSignal::Blocked { reason } => {
            assert!(
                reason.contains("left no outcome this daemon can read"),
                "{reason}"
            );
        }
        other => panic!("expected Blocked, got {other:?}"),
    }
}

#[test]
fn resume_refuses_to_readopt_a_thread_whose_turn_is_still_running() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    let codex_home = dir.path().join("codex-home");
    let backend = CodexBackend::new(config_for(&stub, dir.path(), &codex_home));
    let thread_id = fresh_thread_id();
    write_rollout(&codex_home, &thread_id);
    let decoy = spawn_decoy_process(&["exec", "resume", &thread_id]);
    let pid = decoy.id();
    let handle = ExecutionHandle {
        execution_id: "e-refuse-running".to_string(),
        native_id: Some(thread_id),
    };
    let request = ResumeRequest::new("w-refuse-running", dir.path());
    let err = backend
        .resume(&handle, &request)
        .expect_err("must refuse while a turn of this thread is still running");
    kill_decoy(decoy);
    match err {
        BackendError::Failed { detail, .. } => {
            assert!(
                detail.contains(&pid.to_string()),
                "must name the pid: {detail}"
            );
            assert!(detail.contains("still running"), "{detail}");
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

#[test]
fn resume_refuses_to_readopt_when_surface_is_ambiguous() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    let codex_home = dir.path().join("codex-home");
    let backend = CodexBackend::new(config_for(&stub, dir.path(), &codex_home));
    let thread_id = fresh_thread_id();
    write_rollout(&codex_home, &thread_id);
    let cwd_str = dir.path().to_string_lossy().into_owned();
    let decoy = spawn_decoy_process(&["exec", "-C", &cwd_str]);
    let pid = decoy.id();
    let handle = ExecutionHandle {
        execution_id: "e-refuse-ambiguous".to_string(),
        native_id: Some(thread_id),
    };
    let request = ResumeRequest::new("w-refuse-ambiguous", dir.path());
    let err = backend
        .resume(&handle, &request)
        .expect_err("must refuse when the surface is ambiguously occupied");
    kill_decoy(decoy);
    match err {
        BackendError::Failed { detail, .. } => {
            assert!(
                detail.contains(&pid.to_string()),
                "must name the pid: {detail}"
            );
            assert!(
                detail.contains("cannot say whether it is this thread's turn"),
                "{detail}"
            );
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

// --------------------------------------------------------- §2b: vanishing
// probes — the `Command::new(exe).output()` spawn-failure arms, never
// reachable against `StubCodex::passing` (which always exists and is
// executable).

/// Write a minimal `codex` stand-in for §2b at `dir.join(name)`, running
/// `body` as its `#!/bin/sh` script. Deliberately bypasses `StubCodex`'s own
/// giant launch-recording template: these tests need nothing past the
/// capability probe's own three sequential spawns, and a stub whose whole
/// point is to stop existing partway through has no use for launch
/// recording, replay, or any of `StubCodex`'s other machinery.
fn write_vanishing_stub(dir: &Path, name: &str, body: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{body}")).expect("write vanishing stub");
    let mut permissions = std::fs::metadata(&path).expect("stat stub").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).expect("chmod stub");
    // Not `support::wait_until_executable`: that helper proves runnability
    // by actually running `--version`, which for a stub whose whole point
    // is "answers `--version` exactly once" would spend the one real
    // invocation these tests need for the probe itself. This warms the
    // executable up with an arg neither vanishing script recognizes (falls
    // through to their trailing `exit 1`, consuming nothing), retrying only
    // on `ETXTBSY` (os error 26) — the same transient "still has the file
    // open for writing" window `wait_until_executable` itself retries on.
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match std::process::Command::new(&path)
            .arg("--warm-up-only-matches-no-branch")
            .output()
        {
            Ok(_) => break,
            Err(e) if e.raw_os_error() == Some(26) && Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => panic!("the vanishing stub at {path:?} is not runnable: {e}"),
        }
    }
    path
}

/// Answers `--version` for real, once, then deletes its own script file —
/// every later invocation (starting with `exec --help`) gets `ENOENT`.
fn vanishes_after_version(dir: &Path) -> PathBuf {
    write_vanishing_stub(
        dir,
        "codex-vanishes-after-version",
        &format!(
            "if [ \"$1\" = \"--version\" ]; then echo \"{v}\"; rm -f \"$0\"; exit 0; fi\nexit 1\n",
            v = PASSING_VERSION,
        ),
    )
}

/// Answers `--version` and `exec --help` for real, then deletes its own
/// script file — the next invocation (`exec resume --help`) gets `ENOENT`.
fn vanishes_after_exec_help(dir: &Path) -> PathBuf {
    write_vanishing_stub(
        dir,
        "codex-vanishes-after-exec-help",
        &format!(
            "if [ \"$1\" = \"--version\" ]; then echo \"{v}\"; exit 0; fi\n\
             if [ \"$1\" = \"exec\" ] && [ \"$2\" = \"--help\" ]; then printf '%s\\n' \"{h}\"; \
             rm -f \"$0\"; exit 0; fi\nexit 1\n",
            v = PASSING_VERSION,
            h = ALL_EXEC_HELP,
        ),
    )
}

#[test]
fn probe_refuses_when_the_executable_itself_cannot_be_run() {
    let dir = TempDir::new().expect("tempdir");
    let mut config = CodexConfig::new(dir.path());
    config.executable = dir.path().join("does-not-exist-at-all");
    config.codex_home = Some(dir.path().join("codex-home"));
    let backend = CodexBackend::new(config);
    let report = backend.probe();
    assert!(!report.available);
    let detail = report.detail.expect("detail");
    assert!(detail.contains("cannot run"), "{detail}");
    assert!(detail.contains("--version"), "{detail}");
    assert!(detail.contains("NotFound"), "{detail}");
}

#[test]
fn probe_refuses_when_exec_help_cannot_even_be_run() {
    let dir = TempDir::new().expect("tempdir");
    let mut config = CodexConfig::new(dir.path());
    config.executable = vanishes_after_version(dir.path());
    config.codex_home = Some(dir.path().join("codex-home"));
    let backend = CodexBackend::new(config);
    let report = backend.probe();
    assert!(!report.available);
    let detail = report.detail.expect("detail");
    assert!(detail.contains("cannot run"), "{detail}");
    assert!(detail.contains("exec --help"), "{detail}");
    assert!(detail.contains("NotFound"), "{detail}");
}

#[test]
fn probe_refuses_when_exec_resume_help_cannot_even_be_run() {
    let dir = TempDir::new().expect("tempdir");
    let mut config = CodexConfig::new(dir.path());
    config.executable = vanishes_after_exec_help(dir.path());
    config.codex_home = Some(dir.path().join("codex-home"));
    let backend = CodexBackend::new(config);
    let report = backend.probe();
    assert!(!report.available);
    let detail = report.detail.expect("detail");
    assert!(detail.contains("cannot run"), "{detail}");
    assert!(detail.contains("exec resume --help"), "{detail}");
    assert!(detail.contains("NotFound"), "{detail}");
}

// --------------------------------------------------------- §2c: app-server
// handshake failure arms.

#[test]
fn launch_appserver_refuses_a_thread_start_reply_with_no_thread_id() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    stub.appserver_scripts_reply("thread/start", &[r#"{"id":__ID__,"result":{}}"#]);
    let backend = appserver_backend(&stub, dir.path());
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let err = backend
        .launch(&prepared)
        .expect_err("a thread/start reply with no thread.id must be refused");
    match err {
        BackendError::Failed { detail, .. } => {
            assert!(detail.contains("carried no thread.id"), "{detail}");
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

/// Extends the existing `exit_before` mechanism (previously special-cased
/// away from `initialize`, per the stub's own dispatch loop fix) to the one
/// method that was exempt from it: `initialize` never answering is what G4
/// and LAUNCH's own handshake both need to be able to force.
#[test]
fn gate_g4_handshake_fails_closed_when_initialize_never_answers() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    stub.appserver_exits_before("initialize", 1);
    let mut config = config_for(&stub, dir.path(), &dir.path().join("codex-home"));
    config.transport = TransportChoice::AppServerOnly;
    let backend = CodexBackend::new(config);
    let report = backend.probe();
    assert!(
        !report.available,
        "G4 must refuse under AppServerOnly when initialize never answers"
    );
    let detail = report.detail.expect("detail");
    assert!(
        detail.contains("G4"),
        "the failed gate must be named: {detail}"
    );
}

#[test]
fn launch_appserver_reports_a_handshake_failure_when_initialize_never_answers() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    let backend = appserver_backend(&stub, dir.path());
    // Let the gates pass for real first (a normal spawn, initialize
    // answered) — only *then* arm the marker, so this pins LAUNCH's own
    // handshake call, not a resolution that fell back to exec already.
    let report = backend.probe();
    assert!(
        report.available,
        "gates must pass before the marker is armed: {:?}",
        report.detail
    );
    stub.appserver_exits_before("initialize", 1);
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let err = backend
        .launch(&prepared)
        .expect_err("initialize never answered on this launch's own child");
    match err {
        BackendError::Failed { detail, .. } => {
            assert!(detail.contains("app-server handshake failed"), "{detail}");
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

// --------------------------------------------------------- §2d:
// `appserver_send_turn`'s in-flight refusal and its `turn/start` rollback.

#[test]
fn appserver_send_refuses_a_second_turn_while_the_first_is_in_flight() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    script_appserver_launch(&stub); // turn/start acks, never completes
    let backend = appserver_backend(&stub, dir.path());
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    let err = backend
        .send(&handle, "second turn")
        .expect_err("must be refused while turn 1 is still in flight");
    match err {
        BackendError::Failed { detail, .. } => {
            assert!(detail.contains("already has a turn in flight"), "{detail}");
        }
        other => panic!("expected Failed, got {other:?}"),
    }
    backend.stop(&handle).expect("stop").wait();
}

#[test]
fn appserver_a_failed_turn_start_rolls_back_to_idle_and_a_later_turn_still_succeeds() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    stub.appserver_scripts_reply(
        "thread/start",
        &[r#"{"id":__ID__,"result":{"thread":{"id":"01a02508-5880-7980-95b7-1d8bc22d5139","status":{"type":"idle"}}}}"#],
    );
    // Turn 1: acks and completes immediately, so the cell is settled
    // (Finished, not InFlight) before turn 2 is even attempted.
    stub.appserver_scripts_reply(
        "turn/start",
        &[
            r#"{"id":__ID__,"result":{"turn":{"id":"turn-1"}}}"#,
            r#"{"method":"turn/completed","params":{"turn":{"id":"turn-1","status":"completed"}}}"#,
        ],
    );
    let backend = appserver_backend(&stub, dir.path());
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    wait_for_event(
        &events,
        "conversation.turn.ended",
        "turn 1 settles before turn 2 is attempted",
        |_| true,
    );

    // Turn 2: the RPC itself fails.
    stub.appserver_scripts_reply(
        "turn/start",
        &[r#"{"id":__ID__,"error":{"code":-1,"message":"boom"}}"#],
    );
    let err = backend
        .send(&handle, "second turn")
        .expect_err("turn/start's own RPC failure must surface");
    match err {
        BackendError::Failed { detail, .. } => {
            assert!(detail.contains("turn/start failed"), "{detail}");
        }
        other => panic!("expected Failed, got {other:?}"),
    }

    // Turn 3: re-scripted to succeed. A test that stopped at "the failing
    // call returned Err" would not distinguish the fixed rollback from the
    // bug it replaced (a blind `= Idle` that could clobber a reader-thread
    // `Finished`) — only a *subsequent* send succeeding proves the cell
    // really returned to `Idle` rather than staying wedged.
    stub.appserver_scripts_reply(
        "turn/start",
        &[
            r#"{"id":__ID__,"result":{"turn":{"id":"turn-3"}}}"#,
            r#"{"method":"turn/completed","params":{"turn":{"id":"turn-3","status":"completed"}}}"#,
        ],
    );
    backend
        .send(&handle, "third turn")
        .expect("the cell must have rolled back to Idle, not stayed wedged");
    backend.stop(&handle).expect("stop").wait();
}

// --------------------------------------------------------- §2e:
// `observe_appserver` terminal arms with no coverage through `observe()`
// itself (as opposed to `classify_terminal`'s own unit tests, which pin the
// classification but never call `observe_appserver` over the result).

#[test]
fn appserver_observe_reports_failed_without_the_auth_note_for_an_ordinary_error() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    stub.appserver_scripts_reply(
        "thread/start",
        &[r#"{"id":__ID__,"result":{"thread":{"id":"01a02508-5880-7980-95b7-1d8bc22d5139","status":{"type":"idle"}}}}"#],
    );
    stub.appserver_scripts_reply(
        "turn/start",
        &[
            r#"{"id":__ID__,"result":{"turn":{"id":"turn-1"}}}"#,
            r#"{"method":"turn/completed","params":{"turn":{"id":"turn-1","status":"failed","error":{"message":"boom, ordinary"}}}}"#,
        ],
    );
    let backend = appserver_backend(&stub, dir.path());
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    let observation = wait_for_observation(&backend, &handle, "an ordinary failed turn", |o| {
        matches!(o.signal, sergeant_rs::backend::BackendSignal::Failed { .. })
    });
    match observation.signal {
        sergeant_rs::backend::BackendSignal::Failed { reason } => {
            assert!(reason.contains("boom, ordinary"), "{reason}");
            assert!(
                !reason.contains("(auth)"),
                "an ordinary error must not be tagged auth: {reason}"
            );
        }
        other => panic!("expected Failed, got {other:?}"),
    }
    backend.stop(&handle).expect("stop").wait();
}

#[test]
fn appserver_observe_reports_a_harness_confirmed_interrupt() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    stub.appserver_scripts_reply(
        "thread/start",
        &[r#"{"id":__ID__,"result":{"thread":{"id":"01a02508-5880-7980-95b7-1d8bc22d5139","status":{"type":"idle"}}}}"#],
    );
    stub.appserver_scripts_reply(
        "turn/start",
        &[
            r#"{"id":__ID__,"result":{"turn":{"id":"turn-1"}}}"#,
            r#"{"method":"turn/completed","params":{"turn":{"id":"turn-1","status":"interrupted"}}}"#,
        ],
    );
    let backend = appserver_backend(&stub, dir.path());
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    let observation = wait_for_observation(
        &backend,
        &handle,
        "a harness-confirmed interrupt (turn/completed{status:interrupted})",
        |o| {
            o.evidence
                .as_deref()
                .unwrap_or("")
                .contains("harness-confirmed")
        },
    );
    match observation.signal {
        sergeant_rs::backend::BackendSignal::Running => {}
        other => panic!("an interrupted-but-resumable turn stays Running, got {other:?}"),
    }
    backend.stop(&handle).expect("stop").wait();
}

#[test]
fn appserver_observe_reads_the_childs_own_exit_after_a_turn_completes() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    stub.appserver_scripts_reply(
        "thread/start",
        &[r#"{"id":__ID__,"result":{"thread":{"id":"01a02508-5880-7980-95b7-1d8bc22d5139","status":{"type":"idle"}}}}"#],
    );
    stub.appserver_scripts_reply(
        "turn/start",
        &[
            r#"{"id":__ID__,"result":{"turn":{"id":"turn-1"}}}"#,
            r#"{"method":"turn/completed","params":{"turn":{"id":"turn-1","status":"completed"}}}"#,
        ],
    );
    stub.appserver_exits_after("turn/start");
    let backend = appserver_backend(&stub, dir.path());
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    let observation = wait_for_observation(
        &backend,
        &handle,
        "a completed turn whose child has since exited",
        |o| {
            matches!(
                o.signal,
                sergeant_rs::backend::BackendSignal::StageCompleted { .. }
            ) && o.native == NativeState::Exited
        },
    );
    assert_eq!(observation.native, NativeState::Exited);
}

/// The third, narrowest `InterruptedVia` arm (coverage-spec.md §2e names all
/// three): the stream ends while `turn/interrupt` is still outstanding, so
/// neither an acknowledgement nor an RPC failure is known to be what
/// happened. `appserver_exits_before("turn/interrupt", 1)` closes the
/// stub's stdout before it would otherwise reply to the very first
/// `turn/interrupt` request -- an RPC that goes unanswered, not one that
/// failed and not one the harness accepted. A short `interrupt` budget keeps
/// `Backend::interrupt`'s own blocking call on the unanswered RPC from
/// making this test slow; the reader thread settles the turn on EOF well
/// before that budget ever expires.
#[test]
fn appserver_observe_reports_the_interrupt_rpc_as_unresolved_when_the_stream_closes_first() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    script_appserver_launch(&stub);
    stub.appserver_exits_before("turn/interrupt", 1);
    let mut config = config_for(&stub, dir.path(), &dir.path().join("codex-home"));
    config.transport = TransportChoice::AppServerOnly;
    config.appserver_budgets = Some(Budgets {
        handshake: Duration::from_secs(10),
        thread_start: Duration::from_secs(10),
        turn_start: Duration::from_secs(5),
        interrupt: Duration::from_millis(300),
        stderr_drain: Duration::from_secs(2),
    });
    let backend = CodexBackend::new(config);
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");

    // Sets `InterruptProgress::Requested` before the RPC is even written, so
    // the reader thread's EOF handling is guaranteed to see it outstanding.
    let _ = backend.interrupt(&handle);

    let observation = wait_for_observation(
        &backend,
        &handle,
        "the interrupt's own RPC never got an answer before the stream closed",
        |o| {
            o.evidence
                .as_deref()
                .is_some_and(|evidence| evidence.contains("still outstanding"))
        },
    );
    match observation.signal {
        sergeant_rs::backend::BackendSignal::Running => {}
        other => panic!("expected Running (resumable, no stage verdict), got {other:?}"),
    }
    let evidence = observation.evidence.unwrap_or_default();
    assert!(
        evidence.contains(
            "the child's stdout closed while turn/interrupt was still outstanding, so nothing \
             ever answered it either way"
        ),
        "the unresolved-RPC arm must name itself, not borrow either sibling's sentence; \
         got: {evidence}"
    );
    assert!(
        !evidence.contains("RPC failed") && !evidence.contains("was acknowledged"),
        "must not be confused with either of the other two InterruptedVia arms; got: {evidence}"
    );
}

/// The first `InterruptedVia` arm (coverage-spec.md §2e names all three):
/// `turn/interrupt`'s own RPC comes back a JSON-RPC error, so
/// `interrupt_appserver` falls back to the process-group kill it always
/// keeps ready. Scripting an error reply resolves `call_reserved` to `Err`
/// synchronously inside `Backend::interrupt` itself -- no EOF race to wait
/// out, since `interrupt_appserver`'s own `Err` branch marks `RpcFailed`,
/// kills the group, and settles the turn all before returning.
#[test]
fn appserver_observe_reports_the_interrupt_rpc_as_failed_falling_back_to_the_kill() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    script_appserver_launch(&stub);
    stub.appserver_scripts_reply(
        "turn/interrupt",
        &[r#"{"id":__ID__,"error":{"code":-32000,"message":"boom"}}"#],
    );
    let backend = appserver_backend(&stub, dir.path());
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");

    let _ = backend.interrupt(&handle);

    let observation = wait_for_observation(
        &backend,
        &handle,
        "the interrupt RPC's own error must fall back to the process-group kill",
        |o| {
            o.evidence
                .as_deref()
                .is_some_and(|evidence| evidence.contains("RPC failed"))
        },
    );
    match observation.signal {
        sergeant_rs::backend::BackendSignal::Running => {}
        other => panic!("expected Running (resumable, no stage verdict), got {other:?}"),
    }
    let evidence = observation.evidence.unwrap_or_default();
    assert!(
        evidence.contains(
            "turn/interrupt's own RPC failed and sergeant fell back to the process-group kill"
        ),
        "the RPC-failure arm must name itself, not borrow either sibling's sentence; \
         got: {evidence}"
    );
    assert!(
        !evidence.contains("was acknowledged") && !evidence.contains("still outstanding"),
        "must not be confused with either of the other two InterruptedVia arms; got: {evidence}"
    );
}

/// The second `InterruptedVia` arm (coverage-spec.md §2e names all three):
/// the harness answers `turn/interrupt` with a result -- `resolve` marks
/// `Acknowledged` on the reader thread the instant it decodes that answer,
/// "announce first, wake second" -- but the child's stdout then closes with
/// no `turn/completed` ever naming a verdict. `appserver_exits_after
/// ("turn/interrupt")` is exactly that shape: answer the one RPC, then EOF.
#[test]
fn appserver_observe_reports_the_interrupt_rpc_acknowledged_then_the_stream_closing() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    script_appserver_launch(&stub);
    stub.appserver_scripts_reply("turn/interrupt", &[r#"{"id":__ID__,"result":{}}"#]);
    stub.appserver_exits_after("turn/interrupt");
    let backend = appserver_backend(&stub, dir.path());
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");

    let _ = backend.interrupt(&handle);

    let observation = wait_for_observation(
        &backend,
        &handle,
        "the harness acknowledged the interrupt before the stream closed with no \
         turn/completed",
        |o| {
            o.evidence
                .as_deref()
                .is_some_and(|evidence| evidence.contains("was acknowledged"))
        },
    );
    match observation.signal {
        sergeant_rs::backend::BackendSignal::Running => {}
        other => panic!("expected Running (resumable, no stage verdict), got {other:?}"),
    }
    let evidence = observation.evidence.unwrap_or_default();
    assert!(
        evidence.contains(
            "turn/interrupt was acknowledged, but the child's stdout closed before \
             turn/completed carried the harness's own verdict"
        ),
        "the acknowledged-then-closed arm must name itself, not borrow either sibling's \
         sentence; got: {evidence}"
    );
    assert!(
        !evidence.contains("RPC failed") && !evidence.contains("still outstanding"),
        "must not be confused with either of the other two InterruptedVia arms; got: {evidence}"
    );
}

// --------------------------------------------------------- §2f: small
// residuals, one existing-infrastructure test each.

#[test]
fn a_malformed_replay_line_counts_as_unparsed_but_the_turn_still_settles() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    let transcript = format!(
        "{{\"type\":\"thread.started\",\"thread_id\":\"{tid}\"}}\n\
         {{\"type\":\"turn.started\"}}\n\
         not valid json at all\n\
         {{\"type\":\"item.completed\",\"item\":{{\"id\":\"item_0\",\"type\":\"agent_message\",\"text\":\"ok\"}}}}\n\
         {{\"type\":\"turn.completed\",\"usage\":{{\"input_tokens\":1,\"cached_input_tokens\":0,\
         \"cache_write_input_tokens\":0,\"output_tokens\":1,\"reasoning_output_tokens\":0}}}}\n",
        tid = fresh_thread_id(),
    );
    stub.replays(&transcript);
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
    let ended = wait_for_event(
        &events,
        "conversation.turn.ended",
        "a malformed line must not stop the turn from reaching a terminal",
        |_| true,
    );
    assert_eq!(ended.payload["unparsed_lines"], 1, "{:?}", ended.payload);
    assert_eq!(
        ended.payload["message_items"], 1,
        "the real item either side of the malformed line still decoded: {:?}",
        ended.payload
    );
    // Exec's `conversation.turn.ended` carries no `outcome` field of its own
    // (that key is app-server-only) — OBSERVE is the terminal's real
    // evidence, and it must be a stage verdict, not the ambiguous arm a
    // reader thread that gave up on the malformed line would produce.
    let observation = backend.observe(&handle).expect("observe");
    match observation.signal {
        sergeant_rs::backend::BackendSignal::StageCompleted { .. } => {}
        other => panic!(
            "a malformed line must not turn a completed turn into anything less than a stage \
             verdict: {other:?}"
        ),
    }
    backend.stop(&handle).expect("stop").wait();
}

#[test]
fn appserver_policy_echo_returns_the_thread_start_result_verbatim() {
    let dir = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(dir.path());
    stub.supports_appserver();
    script_appserver_launch(&stub);
    let backend = appserver_backend(&stub, dir.path());
    let prepared = backend
        .prepare(&start_request(dir.path()))
        .expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    let echo = backend
        .appserver_policy_echo(&handle)
        .expect("an app-server execution must echo its thread/start result");
    assert_eq!(
        echo.pointer("/thread/id").and_then(Value::as_str),
        Some("01a02508-5880-7980-95b7-1d8bc22d5139")
    );
    backend.stop(&handle).expect("stop").wait();
}

#[test]
fn appserver_policy_echo_is_none_for_an_exec_transport_execution() {
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
    assert!(
        backend.appserver_policy_echo(&handle).is_none(),
        "exec-transport executions have no app-server policy echo"
    );
    backend.stop(&handle).expect("stop").wait();
}

#[test]
fn prepare_refuses_when_the_probe_itself_is_unavailable() {
    let dir = TempDir::new().expect("tempdir");
    let mut config = CodexConfig::new(dir.path());
    config.executable = dir.path().join("does-not-exist-at-all");
    config.codex_home = Some(dir.path().join("codex-home"));
    let backend = CodexBackend::new(config);
    let err = backend
        .prepare(&start_request(dir.path()))
        .expect_err("an unavailable probe must refuse at PREPARE, before any spawn");
    assert!(matches!(err, BackendError::Unavailable { .. }));
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

/// W3: `live_config` with `ExecOnly` forced. Every pre-W3 `live_codex_*`
/// test below was written against exec's own semantics (its `wait_for_
/// settled` reads `native`, which only exec's per-turn-process model makes
/// mean "the turn is over") — `live_config`'s default `TransportChoice::
/// Auto` now resolves to app-server on any host whose installed codex
/// passes the gates (this host included, measured live), which would
/// silently point those tests at the wrong transport and the wrong OBSERVE
/// shape. Pinning `ExecOnly` here keeps them testing exactly what they
/// always tested; `Auto`'s new default is `live_appserver_config`'s own
/// suite's job to prove.
fn live_exec_config(data_dir: &Path) -> CodexConfig {
    CodexConfig {
        transport: TransportChoice::ExecOnly,
        ..live_config(data_dir)
    }
}

/// W3: `live_config` with `AppServerOnly` forced, so these tests exercise
/// the wired app-server transport itself rather than whatever `Auto` would
/// have picked on this host (which happens to be app-server too, on
/// Cerberus — but a live test asserting on that transport should not depend
/// on Auto's own resolution logic staying that way).
fn live_appserver_config(data_dir: &Path) -> CodexConfig {
    CodexConfig {
        transport: TransportChoice::AppServerOnly,
        ..live_config(data_dir)
    }
}

/// Whether the opt-in live-codex tests may run. Reaching this with the
/// opt-in variable unset is a misuse of `-- --ignored` and panics, naming
/// the opt-in — the false green `#[ignore]` exists to prevent. An unusable
/// harness is a clean skip, written straight to fd 2 (libtest only captures
/// the print macros).
fn codex_live_enabled(test: &str, data_dir: &Path) -> bool {
    codex_live_enabled_with(test, live_config(data_dir))
}

/// W3: the app-server suite's own gate, `AppServerOnly`-forced so a gate
/// failure on some future host is an honest `SKIPPED`, never a silent
/// fallback to exec producing a green test that tested the wrong transport.
fn codex_appserver_live_enabled(test: &str, data_dir: &Path) -> bool {
    codex_live_enabled_with(test, live_appserver_config(data_dir))
}

fn codex_live_enabled_with(test: &str, config: CodexConfig) -> bool {
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

/// W3: the app-server counterpart of [`wait_for_settled`]. Neither `native`
/// (§1.4: "is my child alive", true for the whole execution) nor `signal`
/// (an *interrupted* turn also reports `Running` — no stage verdict, exactly
/// like exec's `InterruptedRunning` — so "signal != Running" cannot tell
/// "settled" apart from "never even started" either) can drive a polling
/// loop the way exec's own turn-ends-its-process signal can. What actually
/// marks a turn as over on this transport is the event
/// `appserver_on_line` emits exactly once per turn, on `turn/completed`
/// (`conversation.turn.ended`) — so this waits for that event (the sink
/// must already be installed) and returns the OBSERVE snapshot taken right
/// after.
fn wait_for_appserver_settled(
    backend: &CodexBackend,
    handle: &ExecutionHandle,
    events: &Arc<Mutex<Vec<EventDraft>>>,
    already_ended: usize,
) -> sergeant_rs::backend::Observation {
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        let ended_count = events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.kind == "conversation.turn.ended")
            .count();
        if ended_count > already_ended {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "live app-server turn never settled"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    backend.observe(handle).expect("observe")
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
    // W3: the resolved transport is now part of this same detail string
    // (§5.5's journaling requirement) — on this host, Auto resolves to
    // app-server with a fresh (non-stale) protocol fingerprint.
    assert!(detail.contains("transport:"));
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
    let backend = CodexBackend::new(live_exec_config(data_dir.path()));
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
    let backend = CodexBackend::new(live_exec_config(data_dir.path()));
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
    let backend = CodexBackend::new(live_exec_config(data_dir.path()));
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
    let backend = CodexBackend::new(live_exec_config(data_dir.path()));
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
    let backend = CodexBackend::new(live_exec_config(data_dir.path()));
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
    let backend = CodexBackend::new(live_exec_config(data_dir.path()));
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
    let fresh = CodexBackend::new(live_exec_config(data_dir.path()));
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

// --------------------------------------------------- W3 §6.4: app-server live suite

#[test]
#[ignore = "opt-in, spends real tokens: SERGEANT_CODEX_TESTS=1 cargo test --test codex_backend -- --ignored"]
fn live_appserver_handshake_and_thread_start() {
    let data_dir = live_workdir("appserver-handshake");
    if !codex_appserver_live_enabled("live_appserver_handshake_and_thread_start", data_dir.path()) {
        return;
    }
    // Zero tokens (M4): PREPARE/LAUNCH's handshake + thread/start never send
    // a turn/start, so this is free to run every time.
    let backend = CodexBackend::new(live_appserver_config(data_dir.path()));
    let mut request = start_request(data_dir.path());
    request.model = Some("gpt-5.6-luna".to_string());
    // An intent that would need a real model turn to finish — but we STOP
    // immediately after LAUNCH's own turn/start returns, well before the
    // model could plausibly answer, so what this proves is the handshake +
    // thread/start path, not a completed turn (that is the next test).
    let prepared = backend.prepare(&request).expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    assert!(
        handle.native_id.is_some(),
        "the native id must come from thread/start's own synchronous result"
    );
    backend.stop(&handle).expect("stop").wait();
}

#[test]
#[ignore = "opt-in, spends real tokens: SERGEANT_CODEX_TESTS=1 cargo test --test codex_backend -- --ignored"]
fn live_appserver_thread_start_echoes_the_requested_policy() {
    let data_dir = live_workdir("appserver-policy");
    if !codex_appserver_live_enabled(
        "live_appserver_thread_start_echoes_the_requested_policy",
        data_dir.path(),
    ) {
        return;
    }
    // Token-free (§3.6): the probe itself already drove `thread/start`
    // during PROBE (G4's handshake uses `initialize` only, not
    // `thread/start` — so this LAUNCH is the first `thread/start` this test
    // sends).
    let outside = live_workdir("appserver-policy-outside");
    let backend = CodexBackend::new(live_appserver_config(data_dir.path()));
    let mut request = start_request(data_dir.path());
    request.model = Some("gpt-5.6-luna".to_string());
    // A binding outside cwd, so `sandbox.writableRoots` has something to
    // discriminate on beyond cwd alone (§3.6's own test 4 shape).
    request.bindings = vec![BindingSummary {
        repository: "outside".to_string(),
        worktree_path: outside.path().to_path_buf(),
        work_branch: "b".to_string(),
        base_branch: None,
        base_sha: "0".repeat(40),
    }];
    let prepared = backend.prepare(&request).expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    // §3.6's own five assertions, on the wire itself: a successful LAUNCH
    // only proves the handshake *accepted* the policy (M6's `-32600` gate
    // without `experimentalApi`) — it proves nothing about what came back.
    // `appserver_policy_echo` is `thread/start`'s own result, captured at
    // LAUNCH and otherwise unreachable from outside this crate.
    let echo = backend
        .appserver_policy_echo(&handle)
        .expect("an app-server execution must have a captured policy echo");
    assert_eq!(
        echo["sandbox"]["type"], "workspaceWrite",
        "thread/start's result: {echo}"
    );
    let writable_roots: Vec<&str> = echo["sandbox"]["writableRoots"]
        .as_array()
        .unwrap_or_else(|| panic!("sandbox.writableRoots must be an array: {echo}"))
        .iter()
        .map(|v| v.as_str().unwrap_or(""))
        .collect();
    assert!(
        writable_roots.contains(&outside.path().to_string_lossy().as_ref()),
        "sandbox.writableRoots must name the out-of-cwd binding: {writable_roots:?}"
    );
    assert_eq!(echo["cwd"], data_dir.path().to_string_lossy().as_ref());
    assert_eq!(echo["approvalPolicy"], "never");
    assert_eq!(echo["model"], "gpt-5.6-luna");
    backend.stop(&handle).expect("stop").wait();
}

#[test]
#[ignore = "opt-in, spends real tokens: SERGEANT_CODEX_TESTS=1 cargo test --test codex_backend -- --ignored"]
fn live_appserver_turn_completes_and_streams() {
    let data_dir = live_workdir("appserver-stream");
    if !codex_appserver_live_enabled("live_appserver_turn_completes_and_streams", data_dir.path()) {
        return;
    }
    let backend = CodexBackend::new(live_appserver_config(data_dir.path()));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let mut request = start_request(data_dir.path());
    request.model = Some("gpt-5.6-luna".to_string());
    request.intent = "Reply with exactly the word ok and nothing else.".to_string();
    request.context = String::new();
    let prepared = backend.prepare(&request).expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    let _user = wait_for_kind(&events, "conversation.user");
    let observation = wait_for_appserver_settled(&backend, &handle, &events, 0);
    assert_eq!(
        observation.native,
        NativeState::Running,
        "the app-server child persists after a completed turn (§1.4) — unlike exec, native never means the turn's own process exited"
    );
    match observation.signal {
        sergeant_rs::backend::BackendSignal::StageCompleted { .. } => {}
        other => panic!("expected StageCompleted, got {other:?}"),
    }
    backend.stop(&handle).expect("stop").wait();
}

#[test]
#[ignore = "opt-in, spends real tokens: SERGEANT_CODEX_TESTS=1 cargo test --test codex_backend -- --ignored"]
fn live_appserver_reports_usage_during_the_turn() {
    let data_dir = live_workdir("appserver-usage");
    if !codex_appserver_live_enabled(
        "live_appserver_reports_usage_during_the_turn",
        data_dir.path(),
    ) {
        return;
    }
    let backend = CodexBackend::new(live_appserver_config(data_dir.path()));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let mut request = start_request(data_dir.path());
    request.model = Some("gpt-5.6-luna".to_string());
    request.intent = "Reply with exactly the word ok and nothing else.".to_string();
    request.context = String::new();
    let prepared = backend.prepare(&request).expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    // §2.3's discriminating assertion: usage must arrive *before* the
    // terminal, not bundled onto it — `thread/tokenUsage/updated` is a
    // separate notification on this transport.
    let usage_event = wait_for_kind(&events, "usage.updated");
    assert!(usage_event.payload["usage"]["total"]["totalTokens"].is_number());
    let observation = wait_for_appserver_settled(&backend, &handle, &events, 0);
    assert_eq!(observation.native, NativeState::Running);
    backend.stop(&handle).expect("stop").wait();
}

#[test]
#[ignore = "opt-in, spends real tokens: SERGEANT_CODEX_TESTS=1 cargo test --test codex_backend -- --ignored"]
fn live_appserver_interrupt_yields_an_interrupted_terminal() {
    let data_dir = live_workdir("appserver-interrupt");
    if !codex_appserver_live_enabled(
        "live_appserver_interrupt_yields_an_interrupted_terminal",
        data_dir.path(),
    ) {
        return;
    }
    let backend = CodexBackend::new(live_appserver_config(data_dir.path()));
    let (sink_fn, events) = sink();
    backend.set_event_sink(sink_fn);
    let mut request = start_request(data_dir.path());
    request.model = Some("gpt-5.6-luna".to_string());
    request.intent =
        "Count slowly from 1 to 200, one number per line, explaining each in a full sentence."
            .to_string();
    request.context = String::new();
    let prepared = backend.prepare(&request).expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    // Bounded wait for genuine mid-flight proof (item/started or a delta),
    // never a fixed sleep — §2.2 step 2's own requirement.
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if events
            .lock()
            .unwrap()
            .iter()
            .any(|e| e.kind == "conversation.user")
        {
            break;
        }
        assert!(Instant::now() < deadline, "turn never even started");
        std::thread::sleep(Duration::from_millis(50));
    }
    std::thread::sleep(Duration::from_millis(400));
    backend.interrupt(&handle).expect("interrupt").wait();
    let observation = wait_for_appserver_settled(&backend, &handle, &events, 0);
    assert_eq!(
        observation.signal,
        sergeant_rs::backend::BackendSignal::Running,
        "an interrupted turn reports Running (no stage verdict), same shape as exec's \
         InterruptedRunning, but harness-confirmed rather than inferred"
    );
    assert_eq!(
        observation.native,
        NativeState::Running,
        "the process was never killed -- that is §2.2's whole point"
    );
    // Resumability: a second turn on the same thread must still succeed.
    send_retrying(&backend, &handle, "Reply with exactly the word ok.");
    let settled = wait_for_appserver_settled(&backend, &handle, &events, 1);
    match settled.signal {
        sergeant_rs::backend::BackendSignal::StageCompleted { .. } => {}
        other => panic!("expected the conversation to still work after interrupt, got {other:?}"),
    }
    backend.stop(&handle).expect("stop").wait();
}

/// §2.4's five-step admission test — "the highest-value single test in this
/// wave". `ask` stays `false` in this build (`ADMISSION_ROWS`): the adapter
/// always declines `item/tool/requestUserInput` (no `NeedsInput` mapping or
/// answering path exists yet), so steps 4/5 cannot pass here regardless of
/// what the model does — that is §2.4's own "only if admitted" scope, not
/// this test's gap. This test's job is steps 1-3, run for real: does the
/// actor's own `item/tool/requestUserInput` ever arrive, cleanly (no
/// `*/requestApproval` contamination)? Both outcomes in §2.4's own outcome
/// table are a pass here — "a negative here is a likely and perfectly good
/// outcome" — what §2.7 forbids is deleting this test or re-deriving its
/// measurement from prose instead of re-running it.
#[test]
#[ignore = "opt-in, spends real tokens: SERGEANT_CODEX_TESTS=1 cargo test --test codex_backend -- --ignored"]
fn live_appserver_actor_authored_question_is_typed() {
    let data_dir = live_workdir("appserver-ask");
    if !codex_appserver_live_enabled(
        "live_appserver_actor_authored_question_is_typed",
        data_dir.path(),
    ) {
        return;
    }
    // Two prompt formulations, tried in order, each once (§2.4 step 2): a
    // model-behaviour probe, so one formulation failing is not the same as
    // the capability being absent. `approvalPolicy: "never"` is already
    // this adapter's unconditional default (§3.3) on every `thread/start`,
    // so step 1 needs no extra wiring here.
    const FORMULATIONS: [&str; 2] = [
        "I want you to write a short poem for a friend. Before you write it, \
         ask me one clarifying question about what kind of poem they'd like -- \
         do not guess or assume, use the request_user_input tool to ask.",
        "Rename the one file in this directory to a clearer name, but the \
         correct new name is genuinely ambiguous from what's here -- use the \
         request_user_input tool to ask me what name I want before you act.",
    ];

    let mut actor_asked = false;
    let mut contaminated = false;
    let mut questions_seen = Value::Null;
    let mut turn_id_seen = Value::Null;

    for prompt in FORMULATIONS {
        let backend = CodexBackend::new(live_appserver_config(data_dir.path()));
        let (sink_fn, events) = sink();
        backend.set_event_sink(sink_fn);
        let mut request = start_request(data_dir.path());
        request.model = Some("gpt-5.6-luna".to_string());
        request.intent = prompt.to_string();
        request.context = String::new();
        let prepared = backend.prepare(&request).expect("prepare");
        let handle = backend.launch(&prepared).expect("launch");
        wait_for_appserver_settled(&backend, &handle, &events, 0);
        backend.stop(&handle).expect("stop").wait();

        let harness_events: Vec<EventDraft> = events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.kind == "conversation.turn.harness_error")
            .cloned()
            .collect();
        let asked = harness_events
            .iter()
            .find(|e| e.payload["method"] == "item/tool/requestUserInput");
        if let Some(asked) = asked {
            actor_asked = true;
            // Step 3's own contamination check: a policy gate's own request
            // firing alongside the actor's means the record caught is not
            // cleanly attributable to the actor alone.
            contaminated = harness_events
                .iter()
                .any(|e| e.payload["phase"] == "approval_denied_unattended");
            questions_seen = asked.payload["questions"].clone();
            turn_id_seen = asked.payload["turn_id"].clone();
            break;
        }
    }

    if !actor_asked {
        // §2.4's own outcome table, first bullet: "absence of a probe
        // result is not a measured negative" -- a legitimate, recorded
        // outcome on this run, not a test failure. Re-run to re-measure.
        eprintln!(
            "live_appserver_actor_authored_question_is_typed: the actor never invoked \
             item/tool/requestUserInput under either formulation on this run (evidence: \
             Unmeasured) -- consistent with ADMISSION_ROWS' recorded ask/AppServer negative; \
             this is a model-behaviour probe, not a build regression."
        );
        return;
    }

    assert!(
        !contaminated,
        "a */requestApproval also fired alongside item/tool/requestUserInput -- the record is \
         not cleanly attributable to the actor (§2.4 step 3's contamination check)"
    );
    assert!(
        questions_seen.as_array().is_some_and(|a| !a.is_empty()),
        "params.questions must be non-empty on the caught request: {questions_seen}"
    );
    assert!(
        !turn_id_seen.is_null(),
        "params.turnId must be present on the caught request: {turn_id_seen}"
    );
    // Steps 4/5 (mapping to `NeedsInput`, answering the request) are §2.4's
    // "only if admitted" work: `ask` stays `false` in this build (no such
    // mapping or answer path exists), so they are not attempted here — the
    // wave's own recorded, honest outcome (§2.6/§2.7), not a gap this test
    // is responsible for closing.
}

#[test]
#[ignore = "opt-in, spends real tokens: SERGEANT_CODEX_TESTS=1 cargo test --test codex_backend -- --ignored"]
fn live_codex_output_schema_round_trips_on_both_transports() {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {"word": {"type": "string"}},
        "required": ["word"],
        "additionalProperties": false
    });
    let prompt = "Answer with the word ok.";

    for (label, config_builder) in [
        (
            "exec",
            Box::new(live_exec_config) as Box<dyn Fn(&Path) -> CodexConfig>,
        ),
        (
            "appserver",
            Box::new(live_appserver_config) as Box<dyn Fn(&Path) -> CodexConfig>,
        ),
    ] {
        let data_dir = live_workdir(&format!("output-schema-{label}"));
        let test_name = format!("live_codex_output_schema_round_trips_on_both_transports[{label}]");
        if !codex_live_enabled_with(&test_name, config_builder(data_dir.path())) {
            continue;
        }

        // Control run: no schema configured, same prompt. The sink is
        // installed *before* LAUNCH: both transports snapshot the sink at
        // spawn time, so installing it after LAUNCH returns would miss
        // turn 1's events entirely (the same hazard `live_codex_resume_
        // recalls_a_nonce_across_processes` documents for exec).
        let control_backend = CodexBackend::new(config_builder(data_dir.path()));
        let (control_sink, control_events) = sink();
        control_backend.set_event_sink(control_sink);
        let mut control_request = start_request(data_dir.path());
        control_request.model = Some("gpt-5.6-luna".to_string());
        control_request.intent = prompt.to_string();
        control_request.context = String::new();
        let control_prepared = control_backend.prepare(&control_request).expect("prepare");
        let control_handle = control_backend.launch(&control_prepared).expect("launch");
        let control_assistant = wait_for_kind(&control_events, "conversation.assistant.completed");
        let control_text = control_assistant.payload["text"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let control_is_schema_shaped = serde_json::from_str::<serde_json::Value>(&control_text)
            .ok()
            .map(|v| v.get("word").is_some() && v.as_object().map(|o| o.len()) == Some(1))
            .unwrap_or(false);
        control_backend.stop(&control_handle).expect("stop").wait();

        // Schema run.
        let mut config = config_builder(data_dir.path());
        config.output_schema = Some(schema.clone());
        let backend = CodexBackend::new(config);
        let (sink_fn, events) = sink();
        backend.set_event_sink(sink_fn);
        let mut request = start_request(data_dir.path());
        request.model = Some("gpt-5.6-luna".to_string());
        request.intent = prompt.to_string();
        request.context = String::new();
        let prepared = backend.prepare(&request).expect("prepare");
        let handle = backend.launch(&prepared).expect("launch");
        let assistant = wait_for_kind(&events, "conversation.assistant.completed");
        let text = assistant.payload["text"].as_str().unwrap_or("").to_string();
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("[{label}] not JSON: {e}: {text:?}"));
        assert!(
            parsed.get("word").is_some(),
            "[{label}] missing word: {text:?}"
        );
        assert_eq!(
            parsed.as_object().map(|o| o.len()),
            Some(1),
            "[{label}] additionalProperties:false must hold: {text:?}"
        );
        assert!(
            !control_is_schema_shaped,
            "[{label}] the discriminating assertion: an unschema'd control run must not \
             coincidentally produce the same shape, or this test cannot tell native validation \
             from a model that just likes JSON: {control_text:?}"
        );
        backend.stop(&handle).expect("stop").wait();
    }
}

// ------------------------------------------------------- W2 §1.4: registration

/// A probeable codex registers for real: `daemon::start_with`, with no
/// test-supplied stand-in for the name "codex", puts a real `CodexBackend`
/// in its registry and journals its own probe evidence at registration —
/// the direct proof of W2's registration change, not a repeat of any unit
/// test on `CodexBackend` itself.
#[tokio::test]
async fn daemon_start_registers_codex_and_journals_its_own_probe() {
    let data = TempDir::new().expect("tempdir");
    let home = TempDir::new().expect("tempdir");
    let stub = StubCodex::passing(data.path());
    let handle = daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(sergeant_rs::backend::BackendRegistry::new()),
            default_backend: None,
            codex: Some(config_for(&stub, data.path(), home.path())),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start with an unmeasured-nothing codex must still start");
    handle.shutdown().await;

    let probed: Vec<_> = Journal::replay_data_dir(data.path())
        .expect("replay")
        .map(|e| e.expect("event"))
        .filter(|e| e.kind == daemon::KIND_BACKEND_PROBED)
        .collect();
    let codex_probed = probed
        .iter()
        .find(|e| e.payload["backend"] == CODEX_BACKEND_NAME)
        .expect("a backend.probed record for codex");
    assert_eq!(codex_probed.payload["available"], true);
    assert_eq!(codex_probed.payload["runtime_scope"], "per_execution");
    assert_eq!(codex_probed.payload["capabilities"]["ask"], false);
    assert_eq!(
        codex_probed.payload["capabilities"]["persistent_sessions"],
        true
    );
    assert_eq!(
        codex_probed.payload["capabilities"]["native_background"],
        false
    );
    assert_eq!(codex_probed.payload["capabilities"]["streaming"], true);
    assert_eq!(codex_probed.payload["capabilities"]["history"], false);
    assert_eq!(codex_probed.payload["capabilities"]["resume"], true);
    assert_eq!(codex_probed.payload["capabilities"]["interrupt"], true);
    assert_eq!(
        codex_probed.payload["capabilities"]["model_selection"],
        true
    );
    assert_eq!(codex_probed.payload["capabilities"]["profiles"], true);
    assert_eq!(codex_probed.payload["capabilities"]["approval_flow"], false);
    assert_eq!(codex_probed.payload["capabilities"]["human_attach"], false);
    assert_eq!(codex_probed.payload["capabilities"]["usage"], true);
    assert_eq!(
        codex_probed.payload["capabilities"]["native_subagents"],
        false
    );
}

/// Issue #259's own acceptance criterion, driven live: "A real Codex
/// contract test edits, stages, and commits in an assigned linked worktree.
/// The commit advances the assigned `sergeant/<work-id>` branch." A real
/// one-commit repo plus a real `git worktree add`-created linked worktree
/// (`real_worktree`, same fixture the stub-driven `--add-dir` test above
/// uses), a turn instructed to append a line and commit it, then the
/// assigned branch's tip in the *source* checkout is asserted to have moved
/// to a new commit whose parent is the worktree's pre-turn `HEAD` — proving
/// the commit is real, on-branch, and not merely a detached-HEAD or
/// working-tree-only edit.
#[test]
#[ignore = "opt-in, spends real tokens: SERGEANT_CODEX_TESTS=1 cargo test --test codex_backend -- --ignored"]
fn live_codex_actor_commits_to_the_works_own_branch() {
    let data_dir = live_workdir("commit");
    if !codex_live_enabled(
        "live_codex_actor_commits_to_the_works_own_branch",
        data_dir.path(),
    ) {
        return;
    }
    let source = data_dir.path().join("source");
    let worktree = data_dir.path().join("worktree");
    let branch = "sergeant/live-259";
    real_worktree(&source, &worktree, branch);
    let before = support::git(&worktree, &["rev-parse", "HEAD"]);

    let backend = CodexBackend::new(live_exec_config(data_dir.path()));
    let mut request = start_request(&worktree);
    request.model = Some("gpt-5.6-luna".to_string());
    request.intent = "Append the single line \"ok\" to README.md, then run `git add \
                       README.md` and `git commit -m \"live #259 test\"`. Do nothing else, \
                       and report only the word done when finished."
        .to_string();
    request.bindings = vec![BindingSummary {
        repository: "solo".to_string(),
        worktree_path: worktree.clone(),
        work_branch: branch.to_string(),
        base_branch: Some("main".to_string()),
        base_sha: before.clone(),
    }];
    let prepared = backend.prepare(&request).expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    wait_for_settled(&backend, &handle);

    let after = support::git(&worktree, &["rev-parse", "HEAD"]);
    assert_ne!(
        before, after,
        "the actor must have committed in its own worktree"
    );
    let parent = support::git(&worktree, &["rev-parse", "HEAD^"]);
    assert_eq!(
        parent, before,
        "the new commit must be built directly on the Work's own starting point"
    );
    let branch_tip = support::git(&source, &["rev-parse", branch]);
    assert_eq!(
        branch_tip, after,
        "the commit must advance the assigned sergeant/<work-id> branch, not just the \
         worktree's detached view of it"
    );
    let head_is_branch = support::git(&worktree, &["symbolic-ref", "--quiet", "--short", "HEAD"]);
    assert_eq!(
        head_is_branch, branch,
        "HEAD must still be the work branch, not detached"
    );
}

/// #262's own acceptance criterion #1: with the network knob explicitly
/// set, a real codex actor can bind `127.0.0.1:0` inside the sandbox.
/// `network_access_is_absent_by_default_and_present_when_configured` (above)
/// only proves the `-c sandbox_workspace_write.network_access=true` flag is
/// composed correctly against a stub — this is the measured half: a live
/// actor actually attempting the bind under codex's real sandbox, so a
/// codex-cli change to what that flag does (or a mis-wired one here) shows
/// up as a failing bind, not just a missing argv token.
#[test]
#[ignore = "opt-in, spends real tokens: SERGEANT_CODEX_TESTS=1 cargo test --test codex_backend -- --ignored"]
fn live_codex_actor_binds_loopback_when_network_access_is_configured() {
    let data_dir = live_workdir("network");
    if !codex_live_enabled(
        "live_codex_actor_binds_loopback_when_network_access_is_configured",
        data_dir.path(),
    ) {
        return;
    }
    let cwd = data_dir.path().join("cwd");
    std::fs::create_dir_all(&cwd).expect("cwd");
    let result_path = cwd.join("bind_result.txt");

    let mut config = live_exec_config(data_dir.path());
    config.workspace_write_network_access = true;
    let backend = CodexBackend::new(config);
    let mut request = start_request(&cwd);
    request.model = Some("gpt-5.6-luna".to_string());
    request.intent = format!(
        "Run this exact Python one-liner: python3 -c \"import socket; s = socket.socket(socket.\
         AF_INET, socket.SOCK_STREAM); s.bind(('127.0.0.1', 0)); print('BIND_OK')\" then write \
         the exact string BIND_OK to a new file at {} (no other content), then report only the \
         word done when finished.",
        result_path.display()
    );

    let prepared = backend.prepare(&request).expect("prepare");
    let handle = backend.launch(&prepared).expect("launch");
    wait_for_settled(&backend, &handle);

    let result = std::fs::read_to_string(&result_path)
        .unwrap_or_else(|e| panic!("actor never wrote {}: {e}", result_path.display()));
    assert_eq!(
        result.trim(),
        "BIND_OK",
        "the actor must have bound 127.0.0.1:0 under the sandbox with network_access configured"
    );
}

/// Codex missing must not break daemon startup, and must be distinguishable
/// from "unregistered": it registers anyway and journals honest, `available:
/// false` evidence naming why it cannot run — the same posture Docker
/// already takes for a host with no Docker installed.
#[tokio::test]
async fn daemon_start_with_no_codex_installed_still_starts_and_says_why() {
    let data = TempDir::new().expect("tempdir");
    let handle = daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(sergeant_rs::backend::BackendRegistry::new()),
            default_backend: None,
            codex: Some(CodexConfig {
                executable: PathBuf::from("/nonexistent/definitely-not-codex"),
                ..CodexConfig::new(data.path())
            }),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("a host with no codex binary must still start a daemon");
    handle.shutdown().await;

    let codex_probed = Journal::replay_data_dir(data.path())
        .expect("replay")
        .map(|e| e.expect("event"))
        .find(|e| {
            e.kind == daemon::KIND_BACKEND_PROBED && e.payload["backend"] == CODEX_BACKEND_NAME
        })
        .expect("codex is registered — and probed — even though it cannot run");
    assert_eq!(codex_probed.payload["available"], false);
    let detail = codex_probed.payload["detail"]
        .as_str()
        .expect("probe detail recorded");
    assert!(detail.contains("cannot run"), "{detail}");
}
