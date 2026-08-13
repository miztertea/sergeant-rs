//! WATCH acceptance suite (`docs/gauntlet/contracts/WATCH.md`;
//! `reference/proposal-sgt-watch-v1.md`): W1–W8 (§16.2, as amended), the
//! live R-WATCH-* pins (R-WATCH-1's waiting tests, R-WATCH-2's two live
//! fingerprint tests, R-WATCH-3's no-spawn refusal, R-WATCH-9's deep-equals
//! and terminal-lag honesty, R-WATCH-10's signal test), and §16.3's
//! structural import checks.
//!
//! Every live-daemon test runs the real `sgt` binary against a
//! [`support::DataDir`] (R-WATCH-10(b)) — the same reaping guard every other
//! stateful CLI suite in this repo uses, so an auto-spawned (or, for `sgt
//! watch` itself, deliberately *not* auto-spawned) daemon never survives a
//! test.
//!
//! **A recurring adaptation, stated once.** The fake backend resolves every
//! LAUNCH/SEND synchronously within the HTTP request that caused it — the
//! `settle` delay `src/backend/fake.rs` documents (an async completion
//! surfaced later, through `drive_completions`) is not reachable through
//! `SGT_FAKE_SCRIPT`'s env grammar, only through in-process script vectors.
//! So once a Work is sitting `active`, the *only* client-triggerable
//! transition available from outside (`WorkState::can_transition`) is
//! `cancel` — `respond` needs `needs_input`, `retry` needs
//! `failed`/`blocked`/`waiting`. Tests that need "a transition arrives while
//! the watcher is merely attached" (W1, W3, R-WATCH-10a) use `cancel` for
//! that transition; it is a full member of R-WATCH-1's watch set and
//! exercises exactly the property being pinned (silence while unwatched,
//! exactly one notice on the real matching transition) — the proposal's own
//! W1 example transitions to `needs_input` instead only because it assumes
//! an async backend.

use std::io::{BufRead, BufReader};
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;

use sergeant_rs::runtime::journal::Journal;
use sergeant_rs::watch::WatchState;

mod support;
use support::DataDir;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
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
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// A real git repository with one commit — a valid workspace root for
/// zero-config discovery from `cwd` (M3's own pattern).
fn init_repo(path: &Path) {
    std::fs::create_dir_all(path).expect("repo dir");
    git(path, &["init", "-b", "main"]);
    std::fs::write(path.join("README.md"), "# fixture\n").expect("write file");
    git(path, &["add", "."]);
    git(path, &["commit", "-m", "initial"]);
}

/// One completed `sgt` invocation.
#[derive(Debug)]
struct Output {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Output {
    fn json(&self) -> Value {
        serde_json::from_str(&self.stdout)
            .unwrap_or_else(|e| panic!("stdout is not JSON ({e}): {}", self.stdout))
    }

    fn assert_ok(&self, what: &str) -> &Self {
        assert_eq!(
            self.code,
            Some(0),
            "{what} must succeed, got {:?}\nstdout: {}\nstderr: {}",
            self.code,
            self.stdout,
            self.stderr
        );
        self
    }
}

/// Run `sgt` to completion from `cwd` against `data_dir`.
fn sgt(cwd: &Path, data_dir: &DataDir, args: &[&str]) -> Output {
    let output = Command::new(SGT)
        .current_dir(cwd)
        .arg("--data-dir")
        .arg(data_dir.path())
        .args(args)
        .output()
        .expect("run sgt");
    Output {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

/// Submit fake-backend work from inside `repo` (zero-config workspace
/// discovery from `cwd`, M3's own pattern — no `--repo` needed). `script` is
/// `SGT_FAKE_SCRIPT`'s grammar; it only has an effect the first time it
/// reaches a data dir with no daemon yet (the daemon reads the env once, at
/// its own spawn, and its `FakeBackend`'s script is one FIFO shared by every
/// LAUNCH/SEND on that daemon regardless of which Work asks —
/// `parsed_steps_are_one_global_fifo_shared_across_executions_and_sends`
/// pins this at the unit level). An empty script defaults every stage to an
/// immediate `complete` (`FakeBackend::next_step`'s own default).
fn submit(repo: &Path, data_dir: &DataDir, script: &str, intent: &str) -> Value {
    let mut command = Command::new(SGT);
    command
        .current_dir(repo)
        .arg("--data-dir")
        .arg(data_dir.path());
    if !script.is_empty() {
        command.env("SGT_FAKE_SCRIPT", script);
    }
    command.args(["--json", "run", intent, "--backend", "fake"]);
    let output = command.output().expect("run sgt run");
    let out = Output {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    };
    out.assert_ok("sgt run");
    out.json()
}

/// A backgrounded `sgt` process (always `watch` in this suite): piped
/// stdout/stderr, each drained by its own reader thread into a channel, so
/// the test can poll with a bounded timeout a process that — by design — may
/// never produce output on its own.
struct WatchProc {
    child: Child,
    out: mpsc::Receiver<String>,
    err: mpsc::Receiver<String>,
}

impl WatchProc {
    fn spawn(cwd: &Path, data_dir: &DataDir, args: &[&str]) -> Self {
        Self::spawn_env(cwd, data_dir, &[], args)
    }

    fn spawn_env(cwd: &Path, data_dir: &DataDir, env: &[(&str, &str)], args: &[&str]) -> Self {
        let mut command = Command::new(SGT);
        command
            .current_dir(cwd)
            .arg("--data-dir")
            .arg(data_dir.path())
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (key, value) in env {
            command.env(key, value);
        }
        let mut child = command.spawn().expect("spawn sgt watch");
        let stdout = child.stdout.take().expect("stdout piped");
        let stderr = child.stderr.take().expect("stderr piped");

        let (out_tx, out_rx) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if out_tx.send(line).is_err() {
                    break;
                }
            }
        });
        let (err_tx, err_rx) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if err_tx.send(line).is_err() {
                    break;
                }
            }
        });

        Self {
            child,
            out: out_rx,
            err: err_rx,
        }
    }

    fn recv_line(&self, timeout: Duration) -> Option<String> {
        self.out.recv_timeout(timeout).ok()
    }

    /// Every stderr line already buffered, waiting up to `first_wait` for the
    /// first one to arrive.
    fn drain_stderr(&self, first_wait: Duration) -> Vec<String> {
        let mut lines = Vec::new();
        if let Ok(line) = self.err.recv_timeout(first_wait) {
            lines.push(line);
            while let Ok(line) = self.err.try_recv() {
                lines.push(line);
            }
        }
        lines
    }

    fn pid(&self) -> u32 {
        self.child.id()
    }

    /// Poll for exit rather than blocking indefinitely: a bug that leaves
    /// the process running must fail this test loudly, not hang the suite.
    fn wait_timeout(&mut self, timeout: Duration) -> Option<std::process::ExitStatus> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Ok(Some(status)) = self.child.try_wait() {
                return Some(status);
            }
            if Instant::now() >= deadline {
                return None;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn expect_exit(&mut self, timeout: Duration, what: &str) -> std::process::ExitStatus {
        match self.wait_timeout(timeout) {
            Some(status) => status,
            None => {
                self.kill();
                panic!("{what}: process did not exit within {timeout:?}");
            }
        }
    }

    fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for WatchProc {
    /// Never leak a piped child even when a test panics mid-assertion.
    fn drop(&mut self) {
        let _ = self.child.try_wait();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn wait_for_hold_ready(hold: &Path, timeout: Duration) -> bool {
    let ready = format!("{}.ready", hold.display());
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if Path::new(&ready).exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

fn release_hold(hold: &Path) {
    std::fs::write(hold, b"").expect("release the watch test hold");
}

fn journal_len(data_dir: &Path) -> usize {
    Journal::replay_data_dir(data_dir)
        .expect("replay journal")
        .filter_map(Result::ok)
        .count()
}

// ---------------------------------------------------------------------------
// W1 — scoped wait is silent, then delivers
// ---------------------------------------------------------------------------

/// W1 (§16.2, adapted per this file's header note): silent while the Work is
/// merely `active`; exactly one notice, carrying the current snapshot, once
/// it matches; exit 0.
#[test]
fn w1_scoped_wait_is_silent_then_delivers_exactly_one_notice() {
    let data = DataDir::new();
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());

    let submitted = submit(
        repo.path(),
        &data,
        "hang",
        "W1: stays active until canceled",
    );
    let id = submitted["work"]["id"].as_str().expect("id").to_string();
    assert_eq!(submitted["work"]["state"], "active");

    let watch = WatchProc::spawn(repo.path(), &data, &["--json", "watch", &id]);
    assert!(
        watch.recv_line(Duration::from_millis(700)).is_none(),
        "must stay silent while the Work is merely active"
    );

    let canceled = sgt(repo.path(), &data, &["--json", "cancel", &id]);
    canceled.assert_ok("cancel");
    assert_eq!(canceled.json()["work"]["state"], "canceled");

    let line = watch
        .recv_line(Duration::from_secs(10))
        .expect("exactly one notice must follow the matching transition");
    let notice: Value = serde_json::from_str(&line).expect("notice parses as JSON");
    assert_eq!(notice["schema"], "sergeant.watch/v1");
    assert_eq!(notice["reason"], "state_transition");
    assert_eq!(notice["snapshot"]["work"]["state"], "canceled");
    assert_eq!(notice["snapshot"]["work"]["id"], id);

    let mut watch = watch;
    let status = watch.expect_exit(Duration::from_secs(10), "W1 one-shot exit");
    assert!(
        status.success(),
        "must exit 0 on a matching one-shot notice: {status:?}"
    );
    assert!(
        watch.recv_line(Duration::from_millis(200)).is_none(),
        "one-shot must emit exactly one notice, never a second"
    );

    data.reap();
}

// ---------------------------------------------------------------------------
// W2 — already-completed Work returns immediately
// ---------------------------------------------------------------------------

#[test]
fn w2_already_completed_work_returns_immediately() {
    let data = DataDir::new();
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());

    let submitted = submit(repo.path(), &data, "", "W2: completes immediately");
    let id = submitted["work"]["id"].as_str().unwrap().to_string();
    assert_eq!(
        submitted["work"]["state"], "completed",
        "an unscripted fake defaults every stage to an immediate complete"
    );

    let out = sgt(repo.path(), &data, &["--json", "watch", &id]);
    out.assert_ok("watch on an already-completed Work");
    let notice: Value = serde_json::from_str(out.stdout.trim())
        .unwrap_or_else(|e| panic!("stdout is not one JSON object ({e}): {}", out.stdout));
    assert_eq!(notice["reason"], "current_state");
    assert_eq!(notice["trigger"], Value::Null);
    assert_eq!(notice["snapshot"]["work"]["state"], "completed");
    assert!(
        !notice["snapshot"]["output"].is_null(),
        "the output pointer must be present: {notice}"
    );
    assert!(
        !notice["snapshot"]["envelope"].is_null(),
        "the envelope must be present: {notice}"
    );

    data.reap();
}

// ---------------------------------------------------------------------------
// W3 — no snapshot/attach race
// ---------------------------------------------------------------------------

/// W3, via R-WATCH-6's hold seam: a transition forced strictly between the
/// stream attaching and the scoped watch's first current-state Work read is
/// neither lost nor double-reported. The `.ready` handshake is asserted
/// directly — a hold that never actually engaged would make this test
/// vacuous (TH-05's class).
#[test]
fn w3_a_transition_forced_inside_the_held_window_is_reported_exactly_once() {
    let data = DataDir::new();
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());

    let submitted = submit(repo.path(), &data, "hang", "W3: transition inside the hold");
    let id = submitted["work"]["id"].as_str().unwrap().to_string();
    assert_eq!(submitted["work"]["state"], "active");

    let hold = data.path().join("w3.hold");
    let watch = WatchProc::spawn_env(
        repo.path(),
        &data,
        &[("SGT_WATCH_TEST_HOLD", hold.to_str().unwrap())],
        &["--json", "watch", &id, "--follow"],
    );
    assert!(
        wait_for_hold_ready(&hold, Duration::from_secs(10)),
        "the client must touch <path>.ready after attaching — the hold must actually engage"
    );

    // The forcing transition: proven (by .ready above) to land after attach,
    // and it can only reach the client after release below — i.e. strictly
    // inside the race window R-WATCH-6 exists to instrument.
    let canceled = sgt(repo.path(), &data, &["--json", "cancel", &id]);
    canceled.assert_ok("cancel inside the hold");

    release_hold(&hold);

    let line = watch
        .recv_line(Duration::from_secs(10))
        .expect("the transition forced inside the window must not be lost");
    let notice: Value = serde_json::from_str(&line).expect("parses");
    assert_eq!(notice["snapshot"]["work"]["state"], "canceled");

    assert!(
        watch.recv_line(Duration::from_millis(300)).is_none(),
        "the same transition must not be double-reported"
    );

    let mut watch = watch;
    let status = watch.expect_exit(Duration::from_secs(10), "W3 terminal exit");
    assert!(
        status.success(),
        "canceled is terminal — scoped --follow must exit 0: {status:?}"
    );

    data.reap();
}

// ---------------------------------------------------------------------------
// W4 — stale trigger does not produce stale meaning
// ---------------------------------------------------------------------------

/// W4, via the same hold seam: the Work moves `needs_input` → `completed`
/// entirely inside the held window (one respond that itself drives the
/// workflow to completion). The scoped watch's own initial current-state
/// read happens only after release, so it must report the *current*
/// completion — never the `needs_input` that was true when the process
/// started attaching.
#[test]
fn w4_a_stale_trigger_does_not_produce_stale_meaning() {
    let data = DataDir::new();
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());

    let submitted = submit(
        repo.path(),
        &data,
        "needs_input:what backoff?;complete:done",
        "W4: respond-then-complete inside the hold",
    );
    let id = submitted["work"]["id"].as_str().unwrap().to_string();
    assert_eq!(submitted["work"]["state"], "needs_input");

    let hold = data.path().join("w4.hold");
    let watch = WatchProc::spawn_env(
        repo.path(),
        &data,
        &[("SGT_WATCH_TEST_HOLD", hold.to_str().unwrap())],
        &["--json", "watch", &id],
    );
    assert!(
        wait_for_hold_ready(&hold, Duration::from_secs(10)),
        "the hold must actually engage"
    );

    let responded = sgt(
        repo.path(),
        &data,
        &["--json", "respond", &id, "3 attempts, exp backoff"],
    );
    responded.assert_ok("respond");
    assert_eq!(
        responded.json()["work"]["state"],
        "completed",
        "the response must drive this Work all the way to completion inside the hold"
    );

    release_hold(&hold);

    let line = watch
        .recv_line(Duration::from_secs(10))
        .expect("a notice must still follow once the hold releases");
    let notice: Value = serde_json::from_str(&line).expect("parses");
    assert_eq!(
        notice["snapshot"]["work"]["state"], "completed",
        "must report the CURRENT state, not the needs_input that was true when watch started: \
         {notice}"
    );

    let mut watch = watch;
    let status = watch.expect_exit(Duration::from_secs(10), "W4 one-shot exit");
    assert!(status.success());

    data.reap();
}

// ---------------------------------------------------------------------------
// W5 — estate-wide watch begins now
// ---------------------------------------------------------------------------

#[test]
fn w5_estate_wide_watch_begins_now_not_from_history() {
    let data = DataDir::new();
    let repo_a = TempDir::new().expect("tempdir");
    init_repo(repo_a.path());
    let repo_b = TempDir::new().expect("tempdir");
    init_repo(repo_b.path());

    let a = submit(repo_a.path(), &data, "", "W5: historical, must not replay");
    assert_eq!(a["work"]["state"], "completed");

    let watch = WatchProc::spawn(repo_a.path(), &data, &["--json", "watch"]);
    assert!(
        watch.recv_line(Duration::from_millis(700)).is_none(),
        "the historical completion must not be replayed"
    );

    let b = submit(repo_b.path(), &data, "", "W5: new, must be emitted");
    let b_id = b["work"]["id"].as_str().unwrap().to_string();

    let line = watch
        .recv_line(Duration::from_secs(10))
        .expect("the new completion must be emitted");
    let notice: Value = serde_json::from_str(&line).expect("parses");
    assert_eq!(notice["snapshot"]["work"]["id"], b_id);
    assert_eq!(notice["snapshot"]["work"]["state"], "completed");

    let mut watch = watch;
    let status = watch.expect_exit(Duration::from_secs(10), "W5 one-shot exit");
    assert!(status.success());

    data.reap();
}

// ---------------------------------------------------------------------------
// W6 — follow mode
// ---------------------------------------------------------------------------

#[test]
fn w6_follow_mode_continues_past_needs_input_then_exits_on_completion() {
    let data = DataDir::new();
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());

    let submitted = submit(
        repo.path(),
        &data,
        "needs_input:which retry budget?",
        "W6: follow",
    );
    let id = submitted["work"]["id"].as_str().unwrap().to_string();

    let watch = WatchProc::spawn(repo.path(), &data, &["--json", "watch", &id, "--follow"]);
    let first = watch
        .recv_line(Duration::from_secs(10))
        .expect("current_state notice");
    let first: Value = serde_json::from_str(&first).expect("parses");
    assert_eq!(first["reason"], "current_state");
    assert_eq!(first["snapshot"]["work"]["state"], "needs_input");

    // Nonterminal: must still be attached (proposal §6.4).
    assert!(
        watch.recv_line(Duration::from_millis(400)).is_none(),
        "must stay attached after a nonterminal notice"
    );

    let responded = sgt(
        repo.path(),
        &data,
        &["--json", "respond", &id, "3 attempts, exp backoff"],
    );
    responded.assert_ok("respond");

    let second = watch
        .recv_line(Duration::from_secs(10))
        .expect("completion notice");
    let second: Value = serde_json::from_str(&second).expect("parses");
    assert_eq!(second["reason"], "state_transition");
    assert_eq!(second["snapshot"]["work"]["state"], "completed");

    let mut watch = watch;
    let status = watch.expect_exit(Duration::from_secs(10), "W6 follow exit on terminal");
    assert!(status.success());

    data.reap();
}

// ---------------------------------------------------------------------------
// W7 — stream closure is honest
// ---------------------------------------------------------------------------

#[test]
fn w7_stream_closure_is_honest_and_never_restarts_the_daemon() {
    let data = DataDir::new();
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());

    let status_out = sgt(repo.path(), &data, &["status"]);
    status_out.assert_ok("status (auto-spawn)");
    assert_eq!(data.daemon_pids().len(), 1);

    let watch = WatchProc::spawn(repo.path(), &data, &["--json", "watch"]);
    assert!(watch.recv_line(Duration::from_millis(300)).is_none());

    let stop = sgt(repo.path(), &data, &["--json", "daemon", "stop"]);
    stop.assert_ok("daemon stop");

    let mut watch = watch;
    let status = watch.expect_exit(
        Duration::from_secs(15),
        "watch must exit once the stream closes",
    );
    assert!(
        !status.success(),
        "a stream closed before any match must exit nonzero: {status:?}"
    );

    let stderr = watch.drain_stderr(Duration::from_secs(2));
    assert!(
        stderr.iter().any(|line| {
            line.contains("watch stream closed after journal seq")
                && line.contains("rerun `sgt watch` to resubscribe")
        }),
        "stderr must name the closure and the resubscribe remedy: {stderr:?}"
    );

    assert!(
        data.daemon_pids().is_empty(),
        "sgt watch must never have restarted the daemon whose stream just closed"
    );

    data.reap();
}

// ---------------------------------------------------------------------------
// W8 — stdout is protocol
// ---------------------------------------------------------------------------

#[test]
fn w8_json_stdout_is_protocol_no_banner_no_heartbeat_no_stderr_leak() {
    let data = DataDir::new();
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());

    let submitted = submit(repo.path(), &data, "needs_input:q1?;needs_input:q2?", "W8");
    let id = submitted["work"]["id"].as_str().unwrap().to_string();

    let watch = WatchProc::spawn(repo.path(), &data, &["--json", "watch", &id, "--follow"]);
    let first = watch
        .recv_line(Duration::from_secs(10))
        .expect("first notice");
    let _: Value = serde_json::from_str(&first).expect("independently parseable JSON object");

    let responded = sgt(repo.path(), &data, &["--json", "respond", &id, "answer 1"]);
    responded.assert_ok("respond");

    let second = watch
        .recv_line(Duration::from_secs(10))
        .expect("second notice");
    let _: Value = serde_json::from_str(&second).expect("independently parseable JSON object");
    assert_ne!(
        first, second,
        "two different questions must produce two distinct lines"
    );

    assert!(
        watch.err.try_recv().is_err(),
        "no stderr output on the happy path — a banner or heartbeat would show up here"
    );

    let mut watch = watch;
    watch.kill();
    data.reap();
}

// ---------------------------------------------------------------------------
// R-WATCH-1 — the six-state watch set, live
// ---------------------------------------------------------------------------

/// R-WATCH-1: a Work parked `waiting` is watched — exactly one notice for
/// entering it, and (since it is not terminal) a scoped `--follow` stays
/// attached past it until an explicit transition ends the run.
#[test]
fn r_watch_1_waiting_emits_and_follow_continues_past_it() {
    let data = DataDir::new();
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());

    let submitted = submit(
        repo.path(),
        &data,
        "waiting:external dependency",
        "R-WATCH-1: waiting",
    );
    let id = submitted["work"]["id"].as_str().unwrap().to_string();
    assert_eq!(submitted["work"]["state"], "waiting");

    let watch = WatchProc::spawn(repo.path(), &data, &["--json", "watch", &id, "--follow"]);
    let first = watch
        .recv_line(Duration::from_secs(10))
        .expect("exactly one notice for entering waiting");
    let first: Value = serde_json::from_str(&first).expect("parses");
    assert_eq!(first["reason"], "current_state");
    assert_eq!(first["snapshot"]["work"]["state"], "waiting");

    assert!(
        watch.recv_line(Duration::from_millis(500)).is_none(),
        "waiting is nonterminal — a scoped --follow must stay attached, not exit"
    );

    let canceled = sgt(repo.path(), &data, &["--json", "cancel", &id]);
    canceled.assert_ok("cancel from waiting");

    let second = watch
        .recv_line(Duration::from_secs(10))
        .expect("the eventual canceled notice");
    let second: Value = serde_json::from_str(&second).expect("parses");
    assert_eq!(second["snapshot"]["work"]["state"], "canceled");

    let mut watch = watch;
    let status = watch.expect_exit(Duration::from_secs(10), "canceled is terminal");
    assert!(status.success());

    data.reap();
}

/// R-WATCH-10(d): the amended vocabulary, pinned directly — no-emission is
/// exactly `pending`/`active`; scoped-`--follow` continuation is exactly
/// `failed`/`blocked`/`waiting`/`needs_input`. The proposal's original
/// five-state phrasing (which listed `waiting` as unwatched) must not
/// survive into this suite.
#[test]
fn r_watch_10d_the_watch_set_vocabulary_matches_the_amended_wording() {
    for state in ["pending", "active"] {
        assert_eq!(
            WatchState::classify(state),
            None,
            "{state} must not be in the watch set"
        );
    }
    for state in ["failed", "blocked", "waiting", "needs_input"] {
        let classified = WatchState::classify(state)
            .unwrap_or_else(|| panic!("{state} must classify into the watch set"));
        assert!(
            !classified.is_terminal(),
            "{state} must not be terminal — a scoped --follow continues past it"
        );
    }
    for state in ["completed", "canceled"] {
        let classified = WatchState::classify(state).unwrap();
        assert!(classified.is_terminal(), "{state} must be terminal");
    }
}

// ---------------------------------------------------------------------------
// R-WATCH-2 — the fingerprint, live
// ---------------------------------------------------------------------------

/// R-WATCH-2: two different questions inside the same stage attempt produce
/// two notices — `detail_identity` is what tells them apart, since
/// `state`/`stage_id`/`attempt`/`stage_status` are unchanged between them
/// (the proposal's own §17 falsifier).
#[test]
fn r_watch_2_two_different_questions_in_one_attempt_produce_two_notices() {
    let data = DataDir::new();
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());

    let submitted = submit(
        repo.path(),
        &data,
        "needs_input:question A?;needs_input:question B?",
        "R-WATCH-2: two questions",
    );
    let id = submitted["work"]["id"].as_str().unwrap().to_string();

    let watch = WatchProc::spawn(repo.path(), &data, &["--json", "watch", &id, "--follow"]);
    let first = watch
        .recv_line(Duration::from_secs(10))
        .expect("question A notice");
    let first: Value = serde_json::from_str(&first).expect("parses");
    assert_eq!(first["snapshot"]["stage"]["detail"], "question A?");

    let responded = sgt(repo.path(), &data, &["--json", "respond", &id, "answer A"]);
    responded.assert_ok("respond to A");
    assert_eq!(responded.json()["stage"]["detail"], "question B?");
    assert_eq!(
        responded.json()["stage"]["stage_id"],
        first["snapshot"]["stage"]["stage_id"],
        "must be the same stage"
    );
    assert_eq!(
        responded.json()["stage"]["attempt"],
        first["snapshot"]["stage"]["attempt"],
        "must be the same attempt — R-WATCH-2 exists precisely because this does not change"
    );

    let second = watch
        .recv_line(Duration::from_secs(10))
        .expect("question B notice");
    let second: Value = serde_json::from_str(&second).expect("parses");
    assert_eq!(second["snapshot"]["stage"]["detail"], "question B?");
    assert_ne!(
        first["snapshot"], second["snapshot"],
        "two different questions must produce two different snapshots"
    );

    let mut watch = watch;
    watch.kill();
    data.reap();
}

/// R-WATCH-2's other half: a real mutation (a respond) that leaves the
/// fingerprint unchanged — because the question text happens to repeat —
/// must not re-emit.
#[test]
fn r_watch_2_a_repeated_identical_question_does_not_re_emit() {
    let data = DataDir::new();
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());

    let submitted = submit(
        repo.path(),
        &data,
        "needs_input:same question?;needs_input:same question?",
        "R-WATCH-2: same question twice",
    );
    let id = submitted["work"]["id"].as_str().unwrap().to_string();

    let watch = WatchProc::spawn(repo.path(), &data, &["--json", "watch", &id, "--follow"]);
    let first = watch
        .recv_line(Duration::from_secs(10))
        .expect("the initial current_state notice");
    let first: Value = serde_json::from_str(&first).expect("parses");
    assert_eq!(first["snapshot"]["stage"]["detail"], "same question?");

    let responded = sgt(repo.path(), &data, &["--json", "respond", &id, "answer"]);
    responded.assert_ok("respond — still needs_input, same detail");
    assert_eq!(responded.json()["work"]["state"], "needs_input");
    assert_eq!(responded.json()["stage"]["detail"], "same question?");

    assert!(
        watch.recv_line(Duration::from_millis(700)).is_none(),
        "an identical re-triggered snapshot must not produce a second notice"
    );

    let mut watch = watch;
    watch.kill();
    data.reap();
}

// ---------------------------------------------------------------------------
// R-WATCH-3 — never auto-spawns
// ---------------------------------------------------------------------------

/// R-WATCH-3: inverse of `m2_daemon_api.rs`'s
/// `t7_cli_end_to_end_auto_spawn_and_second_daemon_fails_closed` — a data
/// dir with zero daemons gets a refusal naming the remedy, exit nonzero, and
/// the process table proves nothing was spawned.
#[test]
fn r_watch_3_watch_against_a_dataless_dir_refuses_and_spawns_nothing() {
    let data = DataDir::new();
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());

    assert!(data.daemon_pids().is_empty(), "must start with no daemon");

    let out = sgt(repo.path(), &data, &["watch", "01SOMENONEXISTENTWORKID000"]);
    assert_ne!(out.code, Some(0), "must exit nonzero: {out:?}");
    assert!(
        out.stderr.contains("no daemon is running for"),
        "must name the state: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("sgt run") && out.stderr.contains("sgt daemon"),
        "must name the remedy: {}",
        out.stderr
    );
    assert!(
        data.daemon_pids().is_empty(),
        "sgt watch must never have spawned a daemon"
    );

    data.reap();
}

// ---------------------------------------------------------------------------
// R-WATCH-9 — deep-equals and terminal-lag honesty
// ---------------------------------------------------------------------------

/// R-WATCH-9: the notice's `snapshot` deep-equals the endpoint body, proven
/// on the two states the ruling names as race-free — a `completed`
/// `current_state` notice, and a `needs_input` `state_transition` notice.
#[test]
fn r_watch_9_snapshot_deep_equals_the_work_endpoint_body() {
    // Two separate data dirs (and so two separate daemons): the fake
    // backend's script is read once, at daemon startup, and shared as one
    // FIFO across every LAUNCH/SEND on that daemon regardless of Work — the
    // two sub-scenarios below need two different scripts, so they cannot
    // share a daemon (measured: the first version of this test shared one
    // `DataDir` and the second submission silently got the first
    // submission's already-empty script, completing immediately instead of
    // landing on `needs_input`).

    // -- current_state / completed --------------------------------------
    let data_a = DataDir::new();
    let repo_a = TempDir::new().expect("tempdir");
    init_repo(repo_a.path());
    let a = submit(
        repo_a.path(),
        &data_a,
        "",
        "R-WATCH-9: completed current_state",
    );
    let a_id = a["work"]["id"].as_str().unwrap().to_string();
    let watch_a = sgt(repo_a.path(), &data_a, &["--json", "watch", &a_id]);
    watch_a.assert_ok("watch a completed Work");
    let notice_a: Value = serde_json::from_str(watch_a.stdout.trim()).expect("parses");
    let endpoint_a = sgt(repo_a.path(), &data_a, &["--json", "work", "show", &a_id]);
    endpoint_a.assert_ok("work show");
    assert_eq!(
        notice_a["snapshot"],
        endpoint_a.json(),
        "a settled completed Work's notice must deep-equal a fresh GET /v1/work/{{id}}"
    );
    data_a.reap();

    // -- state_transition / needs_input -----------------------------------
    let data_b = DataDir::new();
    let repo_b = TempDir::new().expect("tempdir");
    init_repo(repo_b.path());
    let b = submit(
        repo_b.path(),
        &data_b,
        "needs_input:q1?;needs_input:q2?",
        "R-WATCH-9: needs_input state_transition",
    );
    let b_id = b["work"]["id"].as_str().unwrap().to_string();
    assert_eq!(b["work"]["state"], "needs_input");
    let watch_b = WatchProc::spawn(
        repo_b.path(),
        &data_b,
        &["--json", "watch", &b_id, "--follow"],
    );
    let _first = watch_b
        .recv_line(Duration::from_secs(10))
        .expect("current_state for q1");
    let responded = sgt(
        repo_b.path(),
        &data_b,
        &["--json", "respond", &b_id, "answer 1"],
    );
    responded.assert_ok("respond");
    let second = watch_b
        .recv_line(Duration::from_secs(10))
        .expect("state_transition for q2");
    let second: Value = serde_json::from_str(&second).expect("parses");
    assert_eq!(second["reason"], "state_transition");
    let endpoint_b = sgt(repo_b.path(), &data_b, &["--json", "work", "show", &b_id]);
    endpoint_b.assert_ok("work show");
    assert_eq!(
        second["snapshot"],
        endpoint_b.json(),
        "a needs_input state_transition notice must deep-equal a fresh GET /v1/work/{{id}} — no \
         concurrent teardown cascade is in flight for this state"
    );

    let mut watch_b = watch_b;
    watch_b.kill();
    data_b.reap();
}

/// R-WATCH-9's terminal-lag honesty: a *live* completed transition is
/// reported promptly. Not a deep-equals test — a live terminal notice races
/// the teardown cascade (`surface.torn_down`) against a second `GET`, which
/// is exactly the flake the ruling warns against — only that the notice
/// arrives quickly, proving this watcher never waits out that cascade.
#[test]
fn r_watch_9_a_live_completed_transition_is_reported_without_waiting_for_teardown() {
    let data = DataDir::new();
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());

    let submitted = submit(
        repo.path(),
        &data,
        "needs_input:one more thing?",
        "R-WATCH-9: lag honesty",
    );
    let id = submitted["work"]["id"].as_str().unwrap().to_string();

    let watch = WatchProc::spawn(repo.path(), &data, &["--json", "watch", &id, "--follow"]);
    let _current = watch
        .recv_line(Duration::from_secs(10))
        .expect("current_state for needs_input");

    let start = Instant::now();
    let responded = sgt(repo.path(), &data, &["--json", "respond", &id, "answer"]);
    responded.assert_ok("respond drives it to completion");

    let line = watch
        .recv_line(Duration::from_secs(5))
        .expect("the completed notice must arrive promptly");
    let elapsed = start.elapsed();
    let notice: Value = serde_json::from_str(&line).expect("parses");
    assert_eq!(notice["snapshot"]["work"]["state"], "completed");
    assert!(
        elapsed < Duration::from_secs(5),
        "watch must not be waiting out any teardown cascade before reporting: {elapsed:?}"
    );

    let mut watch = watch;
    let status = watch.expect_exit(Duration::from_secs(10), "terminal exit");
    assert!(status.success());

    data.reap();
}

// ---------------------------------------------------------------------------
// R-WATCH-10(a) — signals
// ---------------------------------------------------------------------------

/// R-WATCH-10(a): SIGINT/SIGTERM to a live watcher is a native signal exit —
/// no journal event, no Work state change.
#[test]
fn r_watch_10a_signals_end_the_watcher_natively_with_no_side_effects() {
    let data = DataDir::new();
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());

    let submitted = submit(repo.path(), &data, "hang", "R-WATCH-10a: signal test");
    let id = submitted["work"]["id"].as_str().unwrap().to_string();

    for (flag, expected_signal) in [("-INT", 2), ("-TERM", 15)] {
        let before = journal_len(data.path());

        let watch = WatchProc::spawn(repo.path(), &data, &["watch", &id]);
        assert!(
            watch.recv_line(Duration::from_millis(500)).is_none(),
            "must be genuinely blocked before the signal"
        );

        let pid = watch.pid();
        let signal_status = Command::new("kill")
            .arg(flag)
            .arg(pid.to_string())
            .status()
            .expect("send signal");
        assert!(signal_status.success(), "kill {flag} {pid} must succeed");

        let mut watch = watch;
        let status = watch.expect_exit(Duration::from_secs(10), "signal exit");
        assert_eq!(
            status.signal(),
            Some(expected_signal),
            "the watcher must die OF the signal, not handle it and exit cleanly: {status:?}"
        );

        let after = journal_len(data.path());
        assert_eq!(
            before, after,
            "a signal to the watcher must journal nothing"
        );
    }

    let show = sgt(repo.path(), &data, &["--json", "work", "show", &id]);
    show.assert_ok("work show");
    assert_eq!(
        show.json()["work"]["state"],
        "active",
        "the Work's own state must be unaffected by a signal to an unrelated watcher process"
    );

    data.reap();
}

// ---------------------------------------------------------------------------
// §16.3 — structural import checks
// ---------------------------------------------------------------------------

/// §16.3: `watch.rs` reaches the crate only through `crate::api` (never
/// journal/projection/engine/backend/daemon internals), no new API mutation
/// route exists, and no new event kind names watch/subscription/
/// notification. Mirrors `tests/m6_surfaces.rs`'s `t5`/`t5b` for the TUI and
/// dashboard.
#[test]
fn structural_watch_reaches_the_crate_only_through_api() {
    let source = code_only(&read_source("watch.rs"));
    let paths = crate_paths(&source);
    assert_eq!(
        paths,
        vec!["api".to_string()],
        "watch.rs may reach the crate only through crate::api, but names: {paths:?}"
    );
    for forbidden in [
        "Journal",
        "Analytics",
        "Engine",
        "ApiState",
        "blocking_lock",
        "DockerBackend",
        "BackendRegistry",
    ] {
        assert!(
            !names_token(&source, forbidden),
            "watch.rs names {forbidden}: it must stay a client, never reach daemon internals"
        );
    }
    assert!(
        names_token(&source, "ApiClient"),
        "watch.rs must actually reach state through ApiClient, or the rule above is \
         satisfied by a module that does nothing"
    );

    // No new API mutation route: `sgt watch` is entirely GET /v1/system + GET
    // /v1/events/stream + GET /v1/work/{id}, all pre-existing (WATCH-07).
    let api = read_source("api.rs");
    for needle in ["/v1/watch", "watch/subscribe", "/v1/subscriptions"] {
        assert!(
            !api.contains(needle),
            "no new mutation route may exist for watch: found {needle:?}"
        );
    }

    // No new event kind names watch/subscription/notification (§16.3).
    for path in [
        "domain/work.rs",
        "domain/workflow.rs",
        "domain/event.rs",
        "daemon.rs",
    ] {
        let file_source = read_source(path);
        for line in file_source.lines() {
            if let Some(rest) = line.trim().strip_prefix("pub const KIND_") {
                let lower = rest.to_lowercase();
                assert!(
                    !lower.contains("watch")
                        && !lower.contains("subscription")
                        && !lower.contains("notification"),
                    "{path} declares an event kind naming watch/subscription/notification: {line}"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// structural scan helpers (mirrors tests/m6_surfaces.rs's own — a separate
// test binary cannot import a sibling's private helpers, so this is a
// deliberate, minimal duplication rather than a shared crate).
// ---------------------------------------------------------------------------

fn read_source(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

/// The same file with its `//`-comments removed, string literals tracked so
/// a `"http://…"` is not mistaken for one.
fn code_only(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    for line in source.lines() {
        let bytes: Vec<char> = line.chars().collect();
        let mut in_string = false;
        let mut escaped = false;
        let mut cut = bytes.len();
        for (index, c) in bytes.iter().enumerate() {
            if escaped {
                escaped = false;
                continue;
            }
            match c {
                '\\' if in_string => escaped = true,
                '"' => in_string = !in_string,
                '/' if !in_string && bytes.get(index + 1) == Some(&'/') => {
                    cut = index;
                    break;
                }
                _ => {}
            }
        }
        out.extend(&bytes[..cut]);
        out.push('\n');
    }
    out
}

/// Every distinct crate-root module a source file reaches into, sorted.
fn crate_paths(source: &str) -> Vec<String> {
    let modules = crate_modules();
    let mut found = Vec::new();
    for (index, _) in source.match_indices("crate::") {
        path_heads(&source[index + "crate::".len()..], &mut found);
    }
    let mut relative = Vec::new();
    for (index, _) in source.match_indices("super::") {
        path_heads(&source[index + "super::".len()..], &mut relative);
    }
    found.extend(relative.into_iter().filter(|head| modules.contains(head)));
    found.sort();
    found.dedup();
    found
}

fn path_heads(rest: &str, out: &mut Vec<String>) {
    let rest = rest.trim_start();
    if let Some(group) = rest.strip_prefix('{') {
        for branch in brace_branches(group) {
            path_heads(&branch, out);
        }
        return;
    }
    let head: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if !head.is_empty() {
        out.push(head);
    }
}

fn brace_branches(group: &str) -> Vec<String> {
    let mut branches = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    for c in group.chars() {
        match c {
            '}' if depth == 0 => break,
            '{' => {
                depth += 1;
                current.push(c);
            }
            '}' => {
                depth -= 1;
                current.push(c);
            }
            ',' if depth == 0 => branches.push(std::mem::take(&mut current)),
            _ => current.push(c),
        }
    }
    branches.push(current);
    branches.retain(|branch| !branch.trim().is_empty());
    branches
}

fn crate_modules() -> Vec<String> {
    let lib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs");
    let source = std::fs::read_to_string(&lib).expect("read src/lib.rs");
    let modules: Vec<String> = source
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("pub mod ")
                .and_then(|rest| rest.strip_suffix(';'))
                .map(str::to_string)
        })
        .collect();
    assert!(
        modules.contains(&"api".to_string()) && modules.contains(&"watch".to_string()),
        "src/lib.rs must still declare its modules as `pub mod …;` for this scan to see them: \
         {modules:?}"
    );
    modules
}

fn names_token(source: &str, token: &str) -> bool {
    source
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .any(|word| word == token)
}
