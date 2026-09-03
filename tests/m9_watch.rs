//! WATCH acceptance suite (`sergeant-rs-workspace's knowledge/evidence/gauntlet/contracts/WATCH.md`;
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
//! **Every invocation in this suite runs from an estate root** (estate-root
//! §4.1/§4.2), even though H1 (sprint-plan D6, brief deliverable 2) moved
//! `watch` and `work show` into the host-scoped bucket alongside `daemon
//! stop` — none of the three requires one any more
//! (`r_watch_3_watch_against_a_dataless_dir_refuses_and_spawns_nothing`
//! pins that directly). `run`/`respond`/`cancel` stay estate-scoped
//! (H1 §11.3), and §4.3 puts their exact-root check ahead of descriptor
//! lookup — so a test of *those* verbs whose cwd is merely a git repository
//! no longer reaches the behavior it means to pin; it collects §4.4's
//! refusal instead. Running host-scoped verbs from an estate root anyway is
//! not wrong, just no longer required — cwd is still meaningful for
//! `watch`'s own D6 default (an estate-wide watch inside an estate stays
//! scoped to it, `--all` widens it), which is exactly why this suite keeps
//! its estates rather than switching to bare directories. The fixtures are
//! therefore [`support::scaffold_estate`]
//! estates with derived `repos/<name>` mounts (§6.1), never bare
//! `init_repo`'d temp dirs, and the daemon a `run` auto-spawns is bound to
//! that one estate (§5.1) — which is why the scenarios below that need two
//! independent daemons scaffold two estates, not just two data dirs.
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

use sergeant_rs::daemon;
use sergeant_rs::runtime::journal::Journal;
use sergeant_rs::watch::WatchState;

mod support;
use support::DataDir;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// A single-mount estate root, which is what an estate-scoped verb now
/// requires (§4.1): a `sergeant.toml` declaring `[estate]` plus one `[[repo]]`
/// whose mount is *derived* as `repos/solo` (§6.1 — the `path` key is gone,
/// so a repository can no longer be declared anywhere else). One mount also
/// keeps `sgt run`'s scope unambiguous without a `--repo` flag: §7.1 makes
/// whole-estate selection explicit only where there is more than one
/// repository to choose between.
///
/// Returned as the owning `TempDir` — dropping it removes the estate, so
/// callers must bind it for the life of the test.
fn solo_estate() -> TempDir {
    let root = TempDir::new().expect("tempdir");
    support::scaffold_estate(root.path(), "watch-estate", &["solo"]);
    root
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

/// Spawn a bare `sgt daemon` directly, not through auto-spawn.
///
/// `status`/`work`/`analytics`/`tui` no longer auto-spawn (ADR 0009), so a
/// scenario that wants an already-running, otherwise-untouched daemon (no
/// work submitted, no state transitions to race a freshly attached watch
/// against) has to start it this way instead.
///
/// **`-C <estate>` is now purely informational (H1, `is_host_scoped`).**
/// Before H1 this named the one estate the daemon would ever serve, and a
/// later client's own resolved root was checked against exactly that
/// binding. A v3 descriptor carries no estate at all (D3): every estate a
/// daemon started this way ever serves is admitted per-request, over the
/// wire, the first time a client addresses it — `-C` here does nothing a
/// bare `sgt daemon --data-dir <dir> daemon` would not also do. It is kept
/// only because several call sites below reuse `estate` for the `sgt`
/// helper's own admitted-root arguments elsewhere in this file, not because
/// this spawn needs it.
///
/// `DataDir`'s own Drop reaps this by /proc scan (SIGTERM, then SIGKILL) —
/// never by waiting on this `Child`.
#[allow(clippy::zombie_processes)]
fn spawn_bare_daemon(estate: &Path, data_dir: &DataDir) {
    let mut command = Command::new(SGT);
    command
        .arg("-C")
        .arg(estate)
        .arg("--data-dir")
        .arg(data_dir.path())
        .arg("daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let child = command.spawn().expect("spawn sgt daemon");
    // Reaped by a background thread rather than left as this test's `Child`
    // handle: nothing here ever calls `wait()` otherwise, so the process
    // becomes a zombie the instant it exits — invisible to `kill -0` as
    // "gone" but not actually gone, which is exactly what confused `sgt
    // daemon stop`'s own liveness poll into reporting a clean SIGTERM as
    // "did not exit" (measured: this was the actual cause of a 15s-timeout
    // failure in `w7_stream_closure_is_honest_and_never_restarts_the_daemon`
    // before this fix). Auto-spawn never hits this: its immediate parent is
    // the short-lived CLI client, which exits and reparents the daemon to
    // init, which reaps zombies for free.
    std::thread::spawn(move || {
        let mut child = child;
        let _ = child.wait();
    });
    support::wait_until_sync(
        "the bare daemon never published a descriptor",
        support::HANG_BUDGET,
        || {
            daemon::read_descriptor(data_dir.path())
                .expect("read descriptor")
                .is_some()
        },
    );
}

/// Run `sgt` to completion from `cwd` against `data_dir`. Every caller but
/// R-WATCH-3's refusal arm passes an estate root: `cwd` is the effective
/// root this invocation is admitted (or refused) against (§4.1), since no
/// test here uses `-C`.
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

/// Submit fake-backend work from the estate root at `estate` — the only
/// directory `sgt run` is admitted from (§4.1/§4.2); the mount underneath is
/// selected by the estate's own manifest, or by `scope` where the estate
/// declares more than one. `script` is `SGT_FAKE_SCRIPT`'s grammar; it only
/// has an effect the first time it reaches a data dir with no daemon yet
/// (the daemon reads the env once, at its own spawn, and its `FakeBackend`'s
/// script is one FIFO shared by every LAUNCH/SEND on that daemon regardless
/// of which Work asks —
/// `parsed_steps_are_one_global_fifo_shared_across_executions_and_sends`
/// pins this at the unit level). An empty script defaults every stage to an
/// immediate `complete` (`FakeBackend::next_step`'s own default).
///
/// This is also the call that binds the daemon: the client admits `estate`
/// first and passes it to the daemon it spawns (§5.1), so every later
/// `watch`/`respond`/`cancel` in the same test must name the same root or be
/// refused by the descriptor check.
fn submit_scoped(
    estate: &Path,
    data_dir: &DataDir,
    script: &str,
    intent: &str,
    scope: &[&str],
) -> Value {
    let mut command = Command::new(SGT);
    command
        .current_dir(estate)
        .arg("--data-dir")
        .arg(data_dir.path());
    if !script.is_empty() {
        command.env("SGT_FAKE_SCRIPT", script);
    }
    command.args(["--json", "run", intent, "--backend", "fake"]);
    command.args(scope);
    let output = command.output().expect("run sgt run");
    let out = Output {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    };
    out.assert_ok("sgt run");
    out.json()
}

/// [`submit_scoped`] for the single-mount estate every test but W5 uses.
fn submit(estate: &Path, data_dir: &DataDir, script: &str, intent: &str) -> Value {
    submit_scoped(estate, data_dir, script, intent, &[])
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
    /// the process running must fail this test loudly, not hang the suite
    /// (F-SF-01 fix pass: folded onto support::wait_until_sync -- `Drop`
    /// below already unconditionally kills+reaps the child, including
    /// during this panic's own unwind, so no separate pre-panic cleanup
    /// step is needed here; `timeout` stays caller-supplied, every call
    /// site passes 10s, rather than the shared support::HANG_BUDGET: a
    /// process just told to exit and still alive after a generous
    /// multi-second window is the defect under test, not a hang this
    /// suite should wait 120s to confirm).
    fn expect_exit(&mut self, timeout: Duration, what: &str) -> std::process::ExitStatus {
        let mut status = None;
        support::wait_until_sync(what, timeout, || {
            status = self.child.try_wait().ok().flatten();
            status.is_some()
        });
        status.expect("wait_until_sync only returns after the predicate observed Some(status)")
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

fn wait_for_hold_ready(hold: &Path, timeout: Duration, what: &str) {
    let ready = format!("{}.ready", hold.display());
    support::wait_until_sync(what, timeout, || Path::new(&ready).exists());
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
    let estate = solo_estate();

    let submitted = submit(
        estate.path(),
        &data,
        "hang",
        "W1: stays active until canceled",
    );
    let id = submitted["work"]["id"].as_str().expect("id").to_string();
    assert_eq!(submitted["work"]["state"], "active");

    let watch = WatchProc::spawn(estate.path(), &data, &["--json", "watch", &id]);
    assert!(
        watch.recv_line(Duration::from_millis(700)).is_none(),
        "must stay silent while the Work is merely active"
    );

    let canceled = sgt(estate.path(), &data, &["--json", "cancel", &id]);
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
    let estate = solo_estate();

    let submitted = submit(estate.path(), &data, "", "W2: completes immediately");
    let id = submitted["work"]["id"].as_str().unwrap().to_string();
    assert_eq!(
        submitted["work"]["state"], "completed",
        "an unscripted fake defaults every stage to an immediate complete"
    );

    let out = sgt(estate.path(), &data, &["--json", "watch", &id]);
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
    let estate = solo_estate();

    let submitted = submit(
        estate.path(),
        &data,
        "hang",
        "W3: transition inside the hold",
    );
    let id = submitted["work"]["id"].as_str().unwrap().to_string();
    assert_eq!(submitted["work"]["state"], "active");

    let hold = data.path().join("w3.hold");
    let watch = WatchProc::spawn_env(
        estate.path(),
        &data,
        &[("SGT_WATCH_TEST_HOLD", hold.to_str().unwrap())],
        &["--json", "watch", &id, "--follow"],
    );
    wait_for_hold_ready(
        &hold,
        Duration::from_secs(10),
        "the client must touch <path>.ready after attaching — the hold must actually engage",
    );

    // The forcing transition: proven (by .ready above) to land after attach,
    // and it can only reach the client after release below — i.e. strictly
    // inside the race window R-WATCH-6 exists to instrument.
    let canceled = sgt(estate.path(), &data, &["--json", "cancel", &id]);
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
    let estate = solo_estate();

    let submitted = submit(
        estate.path(),
        &data,
        "needs_input:what backoff?;complete:done",
        "W4: respond-then-complete inside the hold",
    );
    let id = submitted["work"]["id"].as_str().unwrap().to_string();
    assert_eq!(submitted["work"]["state"], "needs_input");

    let hold = data.path().join("w4.hold");
    let watch = WatchProc::spawn_env(
        estate.path(),
        &data,
        &[("SGT_WATCH_TEST_HOLD", hold.to_str().unwrap())],
        &["--json", "watch", &id],
    );
    wait_for_hold_ready(
        &hold,
        Duration::from_secs(10),
        "the hold must actually engage",
    );

    let responded = sgt(
        estate.path(),
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

/// W5: an unscoped `sgt watch` covers the whole estate, and starts at the
/// moment it attaches — a completion that is already history when it
/// subscribes is never replayed to it, and one that lands afterward is.
///
/// The two Works now live in two *mounts of one estate* rather than two
/// unrelated git repositories: §5.1 binds a daemon to exactly one estate and
/// §4.1 admits only an exact root, so "two repositories, one data dir" is no
/// longer expressible — the second `run` would be refused by the descriptor
/// check before it submitted anything. Two mounts under one root is the
/// shape that still makes "estate-wide" mean more than "this repository",
/// which is the half of W5 that matters; §7.1 then requires the mount to be
/// named explicitly, since a multi-repository estate is never inferred.
#[test]
fn w5_estate_wide_watch_begins_now_not_from_history() {
    let data = DataDir::new();
    let estate = TempDir::new().expect("tempdir");
    support::scaffold_estate(estate.path(), "w5-estate", &["alpha", "beta"]);

    let a = submit_scoped(
        estate.path(),
        &data,
        "",
        "W5: historical, must not replay",
        &["--repo", "alpha"],
    );
    assert_eq!(a["work"]["state"], "completed");

    let watch = WatchProc::spawn(estate.path(), &data, &["--json", "watch"]);
    assert!(
        watch.recv_line(Duration::from_millis(700)).is_none(),
        "the historical completion must not be replayed"
    );

    let b = submit_scoped(
        estate.path(),
        &data,
        "",
        "W5: new, must be emitted",
        &["--repo", "beta"],
    );
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
    let estate = solo_estate();

    let submitted = submit(
        estate.path(),
        &data,
        "needs_input:which retry budget?",
        "W6: follow",
    );
    let id = submitted["work"]["id"].as_str().unwrap().to_string();

    let watch = WatchProc::spawn(estate.path(), &data, &["--json", "watch", &id, "--follow"]);
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
        estate.path(),
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
    let estate = solo_estate();

    // `status` no longer auto-spawns (ADR 0009); `run` would, but it would
    // also leave an asynchronous state transition racing the watch attached
    // just below. A bare daemon spawn avoids both.
    spawn_bare_daemon(estate.path(), &data);
    assert_eq!(data.daemon_pids().len(), 1);

    let watch = WatchProc::spawn(estate.path(), &data, &["--json", "watch"]);
    assert!(watch.recv_line(Duration::from_millis(300)).is_none());

    let stop = sgt(estate.path(), &data, &["--json", "daemon", "stop"]);
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
    let estate = solo_estate();

    let submitted = submit(
        estate.path(),
        &data,
        "needs_input:q1?;needs_input:q2?",
        "W8",
    );
    let id = submitted["work"]["id"].as_str().unwrap().to_string();

    let watch = WatchProc::spawn(estate.path(), &data, &["--json", "watch", &id, "--follow"]);
    let first = watch
        .recv_line(Duration::from_secs(10))
        .expect("first notice");
    let _: Value = serde_json::from_str(&first).expect("independently parseable JSON object");

    let responded = sgt(
        estate.path(),
        &data,
        &["--json", "respond", &id, "answer 1"],
    );
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
    let estate = solo_estate();

    let submitted = submit(
        estate.path(),
        &data,
        "waiting:external dependency",
        "R-WATCH-1: waiting",
    );
    let id = submitted["work"]["id"].as_str().unwrap().to_string();
    assert_eq!(submitted["work"]["state"], "waiting");

    let watch = WatchProc::spawn(estate.path(), &data, &["--json", "watch", &id, "--follow"]);
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

    let canceled = sgt(estate.path(), &data, &["--json", "cancel", &id]);
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
    let estate = solo_estate();

    let submitted = submit(
        estate.path(),
        &data,
        "needs_input:question A?;needs_input:question B?",
        "R-WATCH-2: two questions",
    );
    let id = submitted["work"]["id"].as_str().unwrap().to_string();

    let watch = WatchProc::spawn(estate.path(), &data, &["--json", "watch", &id, "--follow"]);
    let first = watch
        .recv_line(Duration::from_secs(10))
        .expect("question A notice");
    let first: Value = serde_json::from_str(&first).expect("parses");
    assert_eq!(first["snapshot"]["stage"]["detail"], "question A?");

    let responded = sgt(
        estate.path(),
        &data,
        &["--json", "respond", &id, "answer A"],
    );
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
    let estate = solo_estate();

    let submitted = submit(
        estate.path(),
        &data,
        "needs_input:same question?;needs_input:same question?",
        "R-WATCH-2: same question twice",
    );
    let id = submitted["work"]["id"].as_str().unwrap().to_string();

    let watch = WatchProc::spawn(estate.path(), &data, &["--json", "watch", &id, "--follow"]);
    let first = watch
        .recv_line(Duration::from_secs(10))
        .expect("the initial current_state notice");
    let first: Value = serde_json::from_str(&first).expect("parses");
    assert_eq!(first["snapshot"]["stage"]["detail"], "same question?");

    let responded = sgt(estate.path(), &data, &["--json", "respond", &id, "answer"]);
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
/// `t7_cli_end_to_end_a_second_estate_is_admitted_and_a_second_process_fails_closed`
/// — a data dir with zero daemons gets a refusal naming the remedy, exit
/// nonzero, and the process table proves nothing was spawned.
///
/// **Re-scoped for H1 (sprint-plan D6, brief deliverable 2): `watch` moved
/// into the host-scoped bucket.** Before H1 this pinned *two different*
/// refusal causes at two different gates — root admission (§4.3, before any
/// descriptor lookup) from a non-estate directory, and `observe_connect`'s
/// own "no daemon" from a valid one — because `watch` required an exact
/// estate root to even attempt daemon discovery. `is_host_scoped` retires
/// that requirement for exactly this verb: `sgt watch` never admits a root
/// at all now, so both directories below reach the *same* gate
/// (`observe_connect`) and the *same* refusal. What survives unchanged is
/// the property this test actually exists to prove — the no-spawn
/// guarantee — and it is stronger evidence of it than before: two cwds that
/// used to fail closed for two unrelated reasons now fail closed for the
/// identical one, which is what "host-scoped, no estate required" means.
#[test]
fn r_watch_3_watch_against_a_dataless_dir_refuses_and_spawns_nothing() {
    let data = DataDir::new();
    let estate = solo_estate();

    assert!(data.daemon_pids().is_empty(), "must start with no daemon");

    let out = sgt(
        estate.path(),
        &data,
        &["watch", "01SOMENONEXISTENTWORKID000"],
    );
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

    // H1 §5: `repos/solo` — a real git checkout *inside* a real estate, not
    // itself an estate root — no longer hits the root gate at all for this
    // host-scoped verb. It reaches exactly the same daemon-discovery
    // refusal the estate root above did, having admitted no root and
    // consulted no `sergeant.toml`.
    let mount = estate.path().join("repos").join("solo");
    let outside = sgt(&mount, &data, &["watch", "01SOMENONEXISTENTWORKID000"]);
    assert_ne!(outside.code, Some(0), "must exit nonzero: {outside:?}");
    assert!(
        outside.stderr.contains("no daemon is running for"),
        "host-scoped: a non-estate cwd must reach the same daemon-discovery refusal as an \
         estate root, never §4.4's root-gate text: {}",
        outside.stderr
    );
    assert!(
        data.daemon_pids().is_empty(),
        "a non-estate cwd must spawn nothing either — `watch` never auto-spawns, root or not"
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
    // landing on `needs_input`). Two daemons now also means two *estates*:
    // §5.1 binds each daemon to the root its spawning client admitted, and a
    // client only ever attaches to a descriptor whose `estate_root` equals
    // its own — so the pairing is one estate per data dir, not one estate
    // shared by both.

    // -- current_state / completed --------------------------------------
    let data_a = DataDir::new();
    let estate_a = solo_estate();
    let a = submit(
        estate_a.path(),
        &data_a,
        "",
        "R-WATCH-9: completed current_state",
    );
    let a_id = a["work"]["id"].as_str().unwrap().to_string();
    let watch_a = sgt(estate_a.path(), &data_a, &["--json", "watch", &a_id]);
    watch_a.assert_ok("watch a completed Work");
    let notice_a: Value = serde_json::from_str(watch_a.stdout.trim()).expect("parses");
    let endpoint_a = sgt(estate_a.path(), &data_a, &["--json", "work", "show", &a_id]);
    endpoint_a.assert_ok("work show");
    assert_eq!(
        notice_a["snapshot"],
        endpoint_a.json(),
        "a settled completed Work's notice must deep-equal a fresh GET /v1/work/{{id}}"
    );
    data_a.reap();

    // -- state_transition / needs_input -----------------------------------
    let data_b = DataDir::new();
    let estate_b = solo_estate();
    let b = submit(
        estate_b.path(),
        &data_b,
        "needs_input:q1?;needs_input:q2?",
        "R-WATCH-9: needs_input state_transition",
    );
    let b_id = b["work"]["id"].as_str().unwrap().to_string();
    assert_eq!(b["work"]["state"], "needs_input");
    let watch_b = WatchProc::spawn(
        estate_b.path(),
        &data_b,
        &["--json", "watch", &b_id, "--follow"],
    );
    let _first = watch_b
        .recv_line(Duration::from_secs(10))
        .expect("current_state for q1");
    let responded = sgt(
        estate_b.path(),
        &data_b,
        &["--json", "respond", &b_id, "answer 1"],
    );
    responded.assert_ok("respond");
    let second = watch_b
        .recv_line(Duration::from_secs(10))
        .expect("state_transition for q2");
    let second: Value = serde_json::from_str(&second).expect("parses");
    assert_eq!(second["reason"], "state_transition");
    let endpoint_b = sgt(estate_b.path(), &data_b, &["--json", "work", "show", &b_id]);
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
    let estate = solo_estate();

    let submitted = submit(
        estate.path(),
        &data,
        "needs_input:one more thing?",
        "R-WATCH-9: lag honesty",
    );
    let id = submitted["work"]["id"].as_str().unwrap().to_string();

    let watch = WatchProc::spawn(estate.path(), &data, &["--json", "watch", &id, "--follow"]);
    let _current = watch
        .recv_line(Duration::from_secs(10))
        .expect("current_state for needs_input");

    let start = Instant::now();
    let responded = sgt(estate.path(), &data, &["--json", "respond", &id, "answer"]);
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
    let estate = solo_estate();

    let submitted = submit(estate.path(), &data, "hang", "R-WATCH-10a: signal test");
    let id = submitted["work"]["id"].as_str().unwrap().to_string();

    // The freshly spawned daemon is not journal-quiescent at submit-response
    // time: since the #293 fix, the backend probe walk runs concurrently
    // AFTER the descriptor is published, journaling one `backend.probed` per
    // registered backend up to seconds later (the slowest installed CLI sets
    // the tail). A signal-side-effect assertion needs a settled baseline, so
    // wait for the journal to stop growing before opening the window —
    // otherwise the probe tail lands inside it and the exact-count check
    // blames the watcher for the daemon's own startup evidence.
    {
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut previous = 0usize;
        let mut stable_for = 0u32;
        loop {
            let len = journal_len(data.path());
            // Three consecutive unchanged 250ms samples: long enough to
            // outlast committer batching, far shorter than any real gap.
            if len == previous {
                stable_for += 1;
                if stable_for >= 3 {
                    break;
                }
            } else {
                stable_for = 0;
            }
            assert!(
                Instant::now() < deadline,
                "the journal never settled after submit ({len} events)"
            );
            previous = len;
            std::thread::sleep(Duration::from_millis(250));
        }
    }

    for (flag, expected_signal) in [("-INT", 2), ("-TERM", 15)] {
        let before = journal_len(data.path());

        let watch = WatchProc::spawn(estate.path(), &data, &["watch", &id]);
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

    let show = sgt(estate.path(), &data, &["--json", "work", "show", &id]);
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
/// notification. Mirrors `tests/m6_surfaces.rs`'s `t5`/`t5b` for the TUI.
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
