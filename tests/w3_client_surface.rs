//! W3 acceptance evidence (H1 sprint plan; brief deliverable 6): lazy
//! admission's client-side half and the host-scoped verb bucket, pinned
//! against the real `sgt` binary.
//!
//! `t7_cli_end_to_end_a_second_estate_is_admitted_and_a_second_process_fails_closed`
//! (`tests/m2_daemon_api.rs`) already proves H1 §11 criterion 1 ("two exact-
//! root estates submit Work to one daemon") end to end, including the
//! journal/registry evidence — this file is additive, not a duplicate: it
//! pins the specific mechanical claims that test's `--data-dir`-explicit
//! fixtures don't exercise — the spawned child's own argv, host-scoped
//! discovery from a directory that is not any estate, the spawned daemon's
//! process-group independence from the client that started it, and `sgt
//! watch`'s D6 estate filter.

use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::Value;

mod support;
use support::DataDir;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

// ---------------------------------------------------------------- helpers

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

fn run(cwd: &Path, data_dir: &Path, args: &[&str]) -> Output {
    let output = Command::new(SGT)
        .current_dir(cwd)
        .arg("--data-dir")
        .arg(data_dir)
        .args(args)
        .output()
        .expect("run sgt");
    Output {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

fn wait_for<F: FnMut() -> bool>(deadline: Duration, mut cond: F) -> bool {
    let end = Instant::now() + deadline;
    loop {
        if cond() {
            return true;
        }
        if Instant::now() >= end {
            return false;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// This process's own argv[0]-derived PATH, plus the pgid a fresh child of
/// this test process gets by default — used by the process-group test below
/// to prove the spawned daemon's group actually differs.
fn pgid_of(pid: u32) -> Option<u32> {
    let output = Command::new("ps")
        .args(["-o", "pgid=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}

fn ppid_of(pid: u32) -> Option<u32> {
    let output = Command::new("ps")
        .args(["-o", "ppid=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}

// ---------------------------------------------------------- deliverable 1

/// Brief deliverable 1: `spawn_daemon` bakes no `-C <estate_root>` scalar
/// into the child's argv (H1 §2/§4). Before this wave the spawned daemon's
/// own command line named exactly one estate — a hard blocker for a second
/// estate's later admission (recon-daemon-lifecycle's highest-risk finding).
#[test]
fn spawn_daemon_never_puts_a_dash_c_estate_root_in_the_hosts_argv() {
    let data = DataDir::new();
    let estate = tempfile::TempDir::new().expect("estate tempdir");
    support::scaffold_estate(estate.path(), "w3-argv", &["solo"]);

    let submitted = run(estate.path(), data.path(), &["--json", "run", "ship it"]);
    submitted.assert_ok("sgt run");

    let pids = data.daemon_pids();
    assert_eq!(pids.len(), 1, "exactly one daemon must have been spawned");
    let daemon_pid = pids[0];

    let processes =
        sergeant_rs::platform::process::running_processes().expect("enumerate processes");
    let argv = &processes
        .iter()
        .find(|p| p.pid == daemon_pid)
        .expect("the spawned daemon's own argv")
        .argv;

    assert!(
        !argv.iter().any(|a| a == "-C"),
        "the spawned daemon's argv must never carry -C (a single-estate scalar): {argv:?}"
    );
    assert!(
        argv.iter().any(|a| a == "--data-dir"),
        "the spawned daemon must still be addressable by its --data-dir: {argv:?}"
    );
    assert!(
        argv.iter().any(|a| a == "daemon"),
        "the spawned process must actually be `daemon`: {argv:?}"
    );

    data.reap();
}

// ---------------------------------------------------------- deliverable 2

/// Brief deliverable 2 (H1 §5): `status`, `work show`/`list`, and
/// `daemon stop` reach the already-running host daemon from a directory
/// that is not, and has no ancestor that is, an estate root — no `-C`
/// needed, no root-gate refusal collected along the way.
#[test]
fn host_scoped_verbs_reach_the_daemon_from_a_directory_with_no_estate_above_it() {
    let data = DataDir::new();
    let estate = tempfile::TempDir::new().expect("estate tempdir");
    support::scaffold_estate(estate.path(), "w3-host-scoped", &["solo"]);

    let submitted = run(estate.path(), data.path(), &["--json", "run", "ship it"]);
    submitted.assert_ok("sgt run");
    let work_id = submitted.json()["work"]["id"]
        .as_str()
        .expect("work id")
        .to_string();

    // A directory wholly unrelated to the estate — no `sergeant.toml`
    // anywhere above it.
    let elsewhere = tempfile::TempDir::new().expect("bare dir");

    let status = run(elsewhere.path(), data.path(), &["--json", "status"]);
    status.assert_ok("sgt status from a non-estate cwd");
    assert!(
        !status.stderr.contains("does not search parent directories"),
        "status must never hit the root gate: {}",
        status.stderr
    );

    let list = run(elsewhere.path(), data.path(), &["--json", "work", "list"]);
    list.assert_ok("sgt work list from a non-estate cwd");
    let works = list.json()["works"].as_array().cloned().unwrap_or_default();
    assert!(
        works.iter().any(|w| w["id"] == work_id),
        "work list from a non-estate cwd must still see the estate's Work: {}",
        list.stdout
    );

    let show = run(
        elsewhere.path(),
        data.path(),
        &["--json", "work", "show", &work_id],
    );
    show.assert_ok("sgt work show from a non-estate cwd");
    assert_eq!(show.json()["work"]["id"], work_id);

    let stopped = run(elsewhere.path(), data.path(), &["--json", "daemon", "stop"]);
    stopped.assert_ok("sgt daemon stop from a non-estate cwd");
    assert_eq!(stopped.json()["status"], "stopped");

    support::wait_until_sync(
        "daemon stop must actually stop the daemon",
        support::HANG_BUDGET,
        || data.daemon_pids().is_empty(),
    );
}

// ---------------------------------------------------------- deliverable 6

/// Brief deliverable 6: terminal-close independence at the host root — the
/// spawned daemon outlives the short-lived client that spawned it (every
/// `sgt run` invocation above already exits while the daemon keeps
/// serving), and it does so in its own process group and reparented off the
/// client's process tree, not merely by accident of timing.
#[test]
fn the_spawned_daemon_survives_its_client_in_its_own_process_group() {
    let data = DataDir::new();
    let estate = tempfile::TempDir::new().expect("estate tempdir");
    support::scaffold_estate(estate.path(), "w3-detach", &["solo"]);

    let submitted = run(estate.path(), data.path(), &["--json", "run", "ship it"]);
    submitted.assert_ok("sgt run");
    // The client (this `run` call) has already returned — its process is
    // gone. The daemon is still there and still healthy.
    let pids = data.daemon_pids();
    assert_eq!(pids.len(), 1, "exactly one daemon");
    let daemon_pid = pids[0];

    let status = run(estate.path(), data.path(), &["--json", "status"]);
    status.assert_ok("the daemon must answer after its launching client exited");

    // Reparented off the test process's own tree: PPID 1 (init) is the
    // ordinary shape once the immediate parent (the short-lived spawning
    // client) has exited and nothing waited on the orphan.
    if let Some(ppid) = ppid_of(daemon_pid) {
        assert_eq!(
            ppid,
            1,
            "the daemon must be reparented to init once its launching client exits, not still \
             parented to this test process (pid {})",
            std::process::id()
        );
    }

    // Own process group: `command.process_group(0)` (`spawn_daemon`) means
    // the daemon's pgid equals its own pid, not this test process's pgid —
    // proof it never receives a signal sent to this test's group (a SIGINT
    // to a hung `cargo test`, e.g.).
    if let (Some(daemon_pgid), Some(my_pgid)) = (pgid_of(daemon_pid), pgid_of(std::process::id())) {
        assert_ne!(
            daemon_pgid, my_pgid,
            "the daemon must sit in its own process group, not this test's"
        );
        assert_eq!(
            daemon_pgid, daemon_pid,
            "a process group leader's pgid equals its own pid"
        );
    }

    data.reap();
}

// ---------------------------------------------------------- deliverable 3

/// Brief deliverable 3 (D6): `sgt watch` (no work id — fleet-wide) inside an
/// estate defaults to that estate's own events; a second estate's
/// transition on the *same* host daemon is invisible to it. `--all` widens
/// the same watch to see every admitted estate.
#[test]
fn watch_defaults_to_the_addressed_estate_and_all_widens_it_to_the_host() {
    let data = DataDir::new();
    let estate_a = tempfile::TempDir::new().expect("estate a");
    let estate_b = tempfile::TempDir::new().expect("estate b");
    support::scaffold_estate(estate_a.path(), "w3-watch-a", &["solo"]);
    support::scaffold_estate(estate_b.path(), "w3-watch-b", &["solo"]);

    // Start the host daemon via estate A, then admit B too (mirrors t7).
    run(estate_a.path(), data.path(), &["--json", "run", "warm a"]).assert_ok("warm a");
    run(estate_b.path(), data.path(), &["--json", "run", "warm b"]).assert_ok("warm b");

    // A scoped-to-A, one-shot, non-follow watch attached *before* B's next
    // transition — an estate-wide watch is edge-triggered (§6.6), so it
    // only reports events after it attaches.
    let mut watch_a = Command::new(SGT)
        .current_dir(estate_a.path())
        .arg("--data-dir")
        .arg(data.path())
        .args(["--json", "watch"])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn sgt watch (estate A, scoped)");

    // Give the watcher time to attach (read journal head, open the SSE
    // stream) before triggering B's transition — otherwise this is racing
    // an unstarted subscriber, not testing the filter.
    std::thread::sleep(Duration::from_millis(300));

    // Trigger a transition in B only: cancel B's already-completed Work is
    // illegal (terminal), so submit a second Work in B and cancel it
    // in-flight is unavailable too (the fake backend settles synchronously)
    // — submitting a fresh Work is itself a `work.submitted`/`work.completed`
    // pair, which is enough to prove the filter: A's watcher must see
    // nothing from it.
    run(
        estate_b.path(),
        data.path(),
        &["--json", "run", "b transition"],
    )
    .assert_ok("b transition");

    // A's scoped watch must NOT have exited (nothing of its own transitioned)
    // within a bounded wait — it is still blocking, silently, exactly as
    // §6.6 promises for an estate-wide watch with no matching event.
    let quiet = wait_for(Duration::from_millis(800), || {
        matches!(watch_a.try_wait(), Ok(Some(_)))
    });
    assert!(
        !quiet,
        "estate A's watch must stay silent on estate B's transition (D6 default scoping)"
    );
    let _ = watch_a.kill();
    let _ = watch_a.wait();

    // `--all` from inside A sees B's transition too: submit one more Work in
    // B while an --all watcher (from A) is attached.
    let mut watch_all = Command::new(SGT)
        .current_dir(estate_a.path())
        .arg("--data-dir")
        .arg(data.path())
        .args(["--json", "watch", "--all"])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn sgt watch --all");
    std::thread::sleep(Duration::from_millis(300));

    run(
        estate_b.path(),
        data.path(),
        &["--json", "run", "b transition under --all"],
    )
    .assert_ok("b transition under --all");

    support::wait_until_sync(
        "--all must see estate B's transition from inside estate A",
        support::HANG_BUDGET,
        || matches!(watch_all.try_wait(), Ok(Some(_))),
    );
    let status = watch_all.wait().expect("wait for --all watch");
    assert!(
        status.success(),
        "a matched one-shot watch exits 0: {status:?}"
    );

    data.reap();
}
