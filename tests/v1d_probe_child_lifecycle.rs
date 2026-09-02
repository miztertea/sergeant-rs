//! #310: a probe's child never outlives the probe, and never outlives the
//! daemon — not even a daemon that dies by `SIGKILL`.
//!
//! ## What this suite is evidence of
//!
//! Every `daemon::start_with` probes every registered backend. The opencode
//! probe runs a real `opencode serve --port 0` and the codex probe a real
//! `codex app-server --listen stdio://`; the suites in this directory start
//! daemons by the hundred and kill them abruptly. A daemon killed while a
//! probe child was live left that child reparented to init forever —
//! diagnosed on Cerberus 2026-08-26 after four OOM-driven session and host
//! deaths, at ~265-342 MB RSS per orphan, one killed at 74 GB total-vm.
//! Neither doctrinal orphan-check pattern matched them (both are `sgt`-shaped,
//! and the leaked species is named `opencode`), so every wave's hygiene check
//! was honest and blind.
//!
//! Two independent facts are pinned here, because the fix has two halves and
//! either one alone would leave the hole open:
//!
//! 1. **The kernel coupling** — `backend::child::harden_probe_child` arms
//!    `prctl(PR_SET_PDEATHSIG, SIGKILL)`, so a hardened child dies when its
//!    parent process is killed. Evidenced against a real, separately spawned
//!    parent (this binary re-executed in a role), with an unhardened control
//!    beside it so a passing assertion cannot mean "the child was going to
//!    exit anyway". Runs on any Linux host, CI included — no CLI needed.
//! 2. **The whole thing, end to end** — a real daemon with real adapters
//!    registered, `SIGKILL`ed mid-probe-walk, leaving **zero** survivors
//!    among the descendant pids captured before the kill. Asserted by exact
//!    pid, never by name-grep: a name-grep would pass on a host where the
//!    operator's own editor happens not to be running opencode, and fail on
//!    one where it is.
//!
//! ## Environment gate
//!
//! Test 2 needs the real third-party CLIs, because a probe that spawns
//! nothing proves nothing about a probe child. It skips loudly
//! (`SKIPPED-ENV`) where they are absent, per this repo's environment-fixture
//! rule — but on a host that has them (Cerberus does) it runs for real, and a
//! survivor is a failure, never a note.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use sergeant_rs::platform::process::{descendants, process_alive, running_processes};

mod support;
use support::DataDir;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

/// Selects the re-executed role this binary plays for test 1. Unset in an
/// ordinary run, which is why [`v1d_role_helper`] is a no-op then.
const ROLE_ENV: &str = "SGT_V1D_ROLE";
/// Where the role helper writes the pid of the child it spawned.
const ROLE_PIDFILE_ENV: &str = "SGT_V1D_PIDFILE";

// --------------------------------------------------------------- role helper

/// A no-op in an ordinary run; the parent half of test 1 when re-executed
/// with [`ROLE_ENV`] set.
///
/// The re-exec exists because the fact under test is not observable from
/// inside a live process. Measured on this kernel (Cerberus, Linux 7.0.0,
/// 2026-08-26), `PR_SET_PDEATHSIG` fires on the parent **process**'s death,
/// not the spawning thread's — so evidencing it means killing a process, and
/// a unit test cannot do that to itself. Re-executing this test binary under
/// `--exact` gives a real parent to kill that runs the real production
/// `harden_probe_child`, with no stand-in and no second implementation.
// The child is deliberately never `wait()`ed: this process is about to be
// SIGKILLed, and whether the child outlives that is the entire measurement.
#[allow(clippy::zombie_processes)]
#[test]
fn v1d_role_helper() {
    let Ok(role) = std::env::var(ROLE_ENV) else {
        return;
    };
    let harden = match role.as_str() {
        "hardened" => true,
        "bare" => false,
        other => panic!("unknown {ROLE_ENV} role {other:?}"),
    };
    let pidfile = PathBuf::from(std::env::var(ROLE_PIDFILE_ENV).expect("role pidfile"));

    let mut command = Command::new("/bin/sh");
    // `exec` so the surviving process is `sleep` itself and the pid written
    // below is the one that has to die — an intermediate shell would make
    // "the child is gone" ambiguous about which child.
    command
        .arg("-c")
        .arg("exec sleep 300")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if harden {
        sergeant_rs::backend::child::harden_probe_child(&mut command);
    }
    let child = command
        .spawn()
        .expect("role helper could not spawn its child");

    // Written-then-renamed so the parent never reads a half-written pid.
    let staging = pidfile.with_extension("staging");
    std::fs::write(&staging, child.id().to_string()).expect("write role pidfile");
    std::fs::rename(&staging, &pidfile).expect("publish role pidfile");

    // Block forever. The parent kills this process; that is the event under
    // test, so returning normally here would defeat the whole fixture.
    loop {
        std::thread::sleep(Duration::from_secs(3600));
    }
}

/// Start the role helper and return `(the helper process, the pid of the
/// child it spawned)`.
///
/// The `Child` is handed back rather than waited on here — every caller kills
/// it and then reaps it, which is what the lint below cannot see across a
/// return.
#[allow(clippy::zombie_processes)]
fn start_role(role: &str, pidfile: &Path) -> (std::process::Child, u32) {
    let helper = Command::new(std::env::current_exe().expect("current exe"))
        .args([
            "--exact",
            "v1d_role_helper",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(ROLE_ENV, role)
        .env(ROLE_PIDFILE_ENV, pidfile)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn the role helper");
    let mut pid_result = None;
    support::wait_until_sync(
        &format!("the {role:?} role helper never published a child pid"),
        support::HANG_BUDGET,
        || {
            if let Ok(text) = std::fs::read_to_string(pidfile)
                && let Ok(pid) = text.trim().parse::<u32>()
            {
                pid_result = Some(pid);
                true
            } else {
                false
            }
        },
    );
    let pid = pid_result.expect("wait_until_sync only returns after its predicate succeeds");
    (helper, pid)
}

fn wait_until_gone(pid: u32, budget: Duration) -> bool {
    let deadline = Instant::now() + budget;
    loop {
        if !process_alive(pid) {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn hard_kill(pid: u32) {
    let _ = Command::new("/bin/sh")
        .arg("-c")
        .arg(format!(
            "kill -KILL -{pid} 2>/dev/null; kill -KILL {pid} 2>/dev/null"
        ))
        .status();
}

// ------------------------------------------------------ 1. the kernel coupling

/// A hardened probe child dies when the process that spawned it is
/// `SIGKILL`ed. This is the one mechanism that survives a killed parent —
/// nothing else can, because no destructor of any kind runs for one.
#[cfg(target_os = "linux")]
#[test]
fn a_hardened_probe_child_dies_when_its_parent_process_is_sigkilled() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (mut helper, child_pid) = start_role("hardened", &dir.path().join("pid"));
    let helper_pid = helper.id();

    hard_kill(helper_pid);
    let _ = helper.wait();

    let gone = wait_until_gone(child_pid, support::HANG_BUDGET);
    if !gone {
        hard_kill(child_pid);
    }
    assert!(
        gone,
        "hardened child {child_pid} outlived its SIGKILLed parent {helper_pid} — \
         PR_SET_PDEATHSIG is not armed, which is exactly the #310 leak"
    );
}

/// The control that makes the assertion above mean something: the identical
/// child, spawned without the hardening, survives the identical kill. This is
/// the behaviour every adapter probe had before #310 was fixed, and it is
/// what put dozens of `opencode serve` processes on PID 1.
#[cfg(target_os = "linux")]
#[test]
fn an_unhardened_child_survives_the_same_kill() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (mut helper, child_pid) = start_role("bare", &dir.path().join("pid"));

    hard_kill(helper.id());
    let _ = helper.wait();

    // Two seconds is not a latency claim: an orphaned `sleep 300` either dies
    // in the kernel's own reparenting (immediately) or lives for five
    // minutes. There is nothing in between for this budget to be wrong about.
    let survived = !wait_until_gone(child_pid, Duration::from_secs(2));
    hard_kill(child_pid);
    assert!(
        survived,
        "the unhardened control child died on its own, so the hardened assertion beside \
         this one proves nothing about PR_SET_PDEATHSIG"
    );
}

// ------------------------------------------------- 2. the whole thing, for real

/// Why this host cannot run the end-to-end assertion, or `None` if it can.
///
/// Both CLIs are named because they are the two probes that spawn a
/// *persistent* child (`opencode serve`, `codex app-server`); a host with
/// neither has no probe child to leak and would pass this test for the wrong
/// reason.
fn missing_probe_clis() -> Option<String> {
    let missing: Vec<&str> = ["opencode", "codex"]
        .into_iter()
        .filter(|cli| {
            !Command::new("/bin/sh")
                .arg("-c")
                .arg(format!("command -v {cli}"))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|status| status.success())
        })
        .collect();
    (!missing.is_empty()).then(|| missing.join(", "))
}

/// The argv of a live process, or `None` if it is gone.
fn argv_of(pid: u32) -> Option<Vec<String>> {
    running_processes()?
        .into_iter()
        .find(|process| process.pid == pid)
        .map(|process| process.argv)
}

/// Whether an argv is one of the two persistent probe children #310 names.
fn is_persistent_probe_child(argv: &[String]) -> bool {
    let joined = argv.join(" ");
    is_opencode_serve_child(argv) || joined.contains("app-server --listen")
}

/// The `opencode serve --port 0` child specifically — the species that
/// actually accumulated on PID 1.
///
/// Told apart from `codex app-server` deliberately, because the two do not
/// leak alike and a test that stopped at whichever appeared first would pass
/// for the wrong reason. An app-server child reads JSON-RPC from a stdin pipe
/// the dying daemon closes, so it sees EOF and exits on its own; a serve
/// child listens on a socket and writes to nobody, so nothing tells it its
/// parent is gone. That difference is why `PR_SET_PDEATHSIG` — not pipe
/// closure — is the mechanism this suite is really about, and why the capture
/// loop below waits for a serve child rather than settling for the first
/// persistent one it sees.
fn is_opencode_serve_child(argv: &[String]) -> bool {
    argv.join(" ").contains("serve --port")
}

/// Spawn a bare `sgt daemon` on `dir` and hand back its process.
///
/// Not through auto-spawn: this test has to hold the daemon's own pid to kill
/// it at a chosen instant, and auto-spawn deliberately detaches.
#[allow(clippy::zombie_processes)]
fn spawn_daemon(dir: &DataDir) -> std::process::Child {
    Command::new(SGT)
        .current_dir(dir.path())
        .arg("--data-dir")
        .arg(dir.path())
        .arg("daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn sgt daemon")
}

/// **#310's centrepiece.** A real daemon, real adapters, `SIGKILL`ed the
/// moment a persistent probe child appears — and then not one of the pids
/// captured before the kill is still alive.
///
/// Asserted by exact captured pid, never by name-grep. A name-grep would pass
/// on a host where the operator's own editor happens not to be running
/// `opencode` and fail on one where it is, which is the opposite of a
/// regression test. Every descendant seen at any point during the walk is
/// accumulated, not just the persistent ones, so a `--help` child that got
/// stuck would be caught by the same assertion.
#[test]
fn a_hard_killed_daemon_leaves_no_probe_descendant_alive() {
    if let Some(missing) = missing_probe_clis() {
        eprintln!(
            "SKIPPED-ENV: #310's end-to-end probe-child assertion needs the real CLIs whose \
             probes spawn a persistent child; not on this host: {missing}. The kernel-coupling \
             half of the fix is still asserted by the two tests above, which need no CLI."
        );
        return;
    }

    let dir = DataDir::new();
    support::scaffold_estate(dir.path(), "v1d", &["solo"]);
    let mut daemon = spawn_daemon(&dir);
    let daemon_pid = daemon.id();

    // Accumulate every descendant this daemon ever shows, and stop the
    // instant one of them is a persistent probe child — that is the window
    // the leak lived in, and waiting for the walk to finish would close it.
    let mut captured: Vec<u32> = Vec::new();
    let mut caught_persistent: Vec<(u32, String)> = Vec::new();
    // wait_until_sync's own panic on timeout is generic (names only the
    // `what` given to it, per this wave's own established tradeoff) --
    // the rich "persistent children seen / all descendants seen" dump the
    // original hand-rolled loop's `assert!` carried on failure cannot
    // survive the fold. `what` keeps the substance of what was expected.
    support::wait_until_sync(
        &format!(
            "the probe walk never showed an `opencode serve --port 0` child under daemon \
             {daemon_pid}. That child is the species #310 is about, and a run that never saw \
             one cannot evidence the fix — so this fails rather than passing vacuously. Check \
             that the opencode adapter is still registered at daemon start and still resolves \
             its transport through the serve gate."
        ),
        support::HANG_BUDGET,
        || {
            let mut caught_the_leaker = false;
            for pid in descendants(daemon_pid) {
                if !captured.contains(&pid) {
                    captured.push(pid);
                }
                let Some(argv) = argv_of(pid) else { continue };
                if is_persistent_probe_child(&argv)
                    && !caught_persistent.iter().any(|(seen, _)| *seen == pid)
                {
                    caught_persistent.push((pid, argv.join(" ")));
                }
                caught_the_leaker |= is_opencode_serve_child(&argv);
            }
            caught_the_leaker
        },
    );

    hard_kill(daemon_pid);
    let _ = daemon.wait();

    // Poll rather than sleep-then-check: the kernel delivers the death signal
    // asynchronously, and a fixed sleep would either be flaky or slow.
    let gone_by = Instant::now() + support::HANG_BUDGET;
    let survivors: Vec<u32> = loop {
        let alive: Vec<u32> = captured
            .iter()
            .copied()
            .filter(|&pid| process_alive(pid))
            .collect();
        if alive.is_empty() || Instant::now() >= gone_by {
            break alive;
        }
        std::thread::sleep(Duration::from_millis(50));
    };

    let described: Vec<String> = survivors
        .iter()
        .map(|&pid| match argv_of(pid) {
            Some(argv) => format!("{pid}: {}", argv.join(" ")),
            None => format!("{pid}: <exited between the check and this line>"),
        })
        .collect();
    for &pid in &survivors {
        hard_kill(pid);
    }
    assert!(
        survivors.is_empty(),
        "#310 regression: {} process(es) descended from daemon {daemon_pid} survived its \
         SIGKILL. Caught mid-walk: {caught_persistent:?}. Survivors: {described:?}",
        survivors.len(),
    );
}

/// The ordinary path, which is the one that runs hundreds of times a suite: a
/// probe walk that completes normally leaves nothing of its own behind.
///
/// Separate from the assertion above because it fails for a different reason.
/// That one catches "a killed daemon orphans its probe child"; this one
/// catches "a probe forgot to kill and reap its child when it finished", the
/// requirement that holds even when nothing ever kills the daemon.
#[test]
fn a_completed_probe_walk_leaves_no_child_of_its_own_behind() {
    if let Some(missing) = missing_probe_clis() {
        eprintln!("SKIPPED-ENV: no probe on this host spawns a child to leave behind: {missing}");
        return;
    }

    let dir = DataDir::new();
    support::scaffold_estate(dir.path(), "v1d", &["solo"]);
    let mut daemon = spawn_daemon(&dir);
    let daemon_pid = daemon.id();

    // The walk is finished when the daemon has been idle of children for a
    // while — asserted as a settled state rather than read from the journal,
    // because what this test is about is processes, not events.
    // Deliberately NOT support::wait_until_sync: this loop's own exit is
    // not this test's verdict either way (its budget just bounds how long
    // it looks) -- `saw_a_child` and `leftover.is_empty()` below are the
    // actual assertions, checked from state this loop leaves behind
    // regardless of whether it exited via the 3s-quiet break or the
    // deadline. wait_until_sync's panic-on-timeout would add a new failure
    // mode this test never had (failing here even when the post-loop
    // state the real assertions check turns out fine).
    let deadline = Instant::now() + support::HANG_BUDGET;
    let mut quiet_since: Option<Instant> = None;
    let mut saw_a_child = false;
    while Instant::now() < deadline {
        let live = descendants(daemon_pid);
        saw_a_child |= !live.is_empty();
        match (live.is_empty(), quiet_since) {
            (true, None) => quiet_since = Some(Instant::now()),
            (true, Some(since)) if since.elapsed() >= Duration::from_secs(3) => break,
            (true, Some(_)) => {}
            (false, _) => quiet_since = None,
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let leftover: Vec<u32> = descendants(daemon_pid);
    let described: Vec<String> = leftover
        .iter()
        .map(|&pid| match argv_of(pid) {
            Some(argv) => format!("{pid}: {}", argv.join(" ")),
            None => format!("{pid}: <exited>"),
        })
        .collect();

    // The daemon goes the polite way here; the rig's own guard is what
    // asserts nothing survived it.
    hard_kill(daemon_pid);
    let _ = daemon.wait();

    assert!(
        saw_a_child,
        "the daemon never spawned a probe child at all, so this test proves nothing"
    );
    assert!(
        leftover.is_empty(),
        "#310 requirement 2: a completed probe walk left {} child process(es) running \
         under daemon {daemon_pid}: {described:?}",
        leftover.len()
    );
}

/// #310 requirement 3, from the rig's side: after the `DataDir` guard has
/// reaped, nothing that was ever a descendant of its daemons is still alive.
///
/// The guard captures the tree *before* it signals, which is the only order
/// that can work — a signalled daemon's children reparent to init and no
/// ancestry query finds them again. That ordering is exactly what made every
/// wave's orphan check honest and blind for a working day.
#[test]
fn the_data_dir_guard_leaves_no_descendant_of_its_daemons_alive() {
    let dir = DataDir::new();
    support::scaffold_estate(dir.path(), "v1d", &["solo"]);
    let mut daemon = spawn_daemon(&dir);
    let daemon_pid = daemon.id();

    let mut captured: Vec<u32> = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        for pid in descendants(daemon_pid) {
            if !captured.contains(&pid) {
                captured.push(pid);
            }
        }
        if !captured.is_empty() && Instant::now() > deadline - Duration::from_secs(7) {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    dir.reap();
    let _ = daemon.wait();

    let survivors: Vec<u32> = captured
        .iter()
        .copied()
        .filter(|&pid| process_alive(pid))
        .collect();
    for &pid in &survivors {
        hard_kill(pid);
    }
    assert!(
        survivors.is_empty(),
        "the DataDir guard reaped daemon {daemon_pid} but left {} of its descendants \
         alive: {survivors:?}",
        survivors.len()
    );
}

// ------------------------------- 3. the in-process daemon's own descendant reap

/// The kernel's one-letter state for `pid` (`R`, `S`, `Z`, ...), or a
/// description of why it could not be read.
///
/// Read from `/proc/<pid>/stat` by splitting at the **last** `)`: the `comm`
/// field before it is parenthesised and unescaped, so a positional split from
/// the left is wrong for any process whose name contains a bracket.
fn proc_state(pid: u32) -> String {
    #[cfg(target_os = "linux")]
    {
        match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
            Ok(stat) => stat
                .rsplit_once(") ")
                .and_then(|(_, tail)| tail.split_whitespace().next().map(str::to_string))
                .unwrap_or_else(|| "<unparseable /proc stat>".to_string()),
            Err(e) => format!("<{e}>"),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        "<no /proc on this platform>".to_string()
    }
}

/// The probe children spawned **directly by this test process**, which is
/// what an in-process daemon's probe children are.
///
/// Attribution by parent pid, not by a descendant walk: the other tests in
/// this file spawn real `sgt daemon` children of this same process, and
/// *their* serve children are descendants of this process too. Only the
/// in-process daemon's are its direct children, so that is the filter.
fn direct_serve_children() -> Vec<u32> {
    let me = std::process::id();
    running_processes()
        .unwrap_or_default()
        .into_iter()
        .filter(|process| process.ppid == Some(me) && is_opencode_serve_child(&process.argv))
        .map(|process| process.pid)
        .collect()
}

/// #310 requirement 3, from the daemon's own side: `DaemonHandle::kill` — the
/// in-process rig's analogue of `SIGKILL` — reaps the probe children its walk
/// still has live.
///
/// Aborting the serve task does not reach the probe walk, and a probe child is
/// a *process*: a real `opencode serve` at ~265 MB that nothing in this
/// process's memory owns once the handle is gone. `PR_SET_PDEATHSIG` covers
/// the out-of-process daemon because that daemon's death is a process death;
/// an in-process daemon's is not, so the same guarantee has to come from the
/// per-daemon `ProbeChildren` set the walk records into.
///
/// `await_probe_walk: false` is what makes the window observable at all: the
/// default hands the handle back only after the walk has finished, by which
/// time every probe has already killed its own child.
#[tokio::test]
async fn daemon_handle_kill_reaps_a_probe_child_its_walk_still_has_live() {
    if let Some(missing) = missing_probe_clis() {
        eprintln!(
            "SKIPPED-ENV: no probe on this host spawns a persistent child for \
             DaemonHandle::kill to reap: {missing}"
        );
        return;
    }

    let dir = DataDir::new();
    support::scaffold_estate(dir.path(), "v1d", &["solo"]);
    let handle = sergeant_rs::daemon::start_with(
        dir.path(),
        sergeant_rs::daemon::DaemonConfig {
            await_probe_walk: false,
            ..Default::default()
        },
    )
    .await
    .expect("start the in-process daemon");

    let deadline = Instant::now() + support::HANG_BUDGET;
    let live = loop {
        let live = direct_serve_children();
        if !live.is_empty() {
            break live;
        }
        if Instant::now() >= deadline {
            break Vec::new();
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    };

    handle.kill().await;

    let survivors: Vec<u32> = live
        .iter()
        .copied()
        .filter(|&pid| {
            // Poll each one: the group kill is a subprocess, so "gone" is not
            // instantaneous even though it is prompt.
            !wait_until_gone(pid, Duration::from_secs(10))
        })
        .collect();
    for &pid in &survivors {
        hard_kill(pid);
    }

    assert!(
        !live.is_empty(),
        "the in-process daemon's walk never showed a live `opencode serve` child within \
         {:?}, so this test cannot evidence the reap — it fails rather than passing \
         vacuously",
        support::HANG_BUDGET,
    );
    // The kernel's own state letter for each survivor, because the two ways
    // to fail here need different fixes and the pid alone does not say which:
    // `R`/`S` means the kill never reached it, `Z` means it was killed and
    // nobody reaped it — and a zombie is not a harmless bookkeeping detail
    // here, it still answers `kill(pid, 0)` as alive and still matches
    // `pgrep -x opencode`, so an orphan check cannot tell it from the leak.
    let states: Vec<String> = survivors.iter().map(|&pid| proc_state(pid)).collect();
    assert!(
        survivors.is_empty(),
        "DaemonHandle::kill left {} probe child(ren) of its own walk behind: {survivors:?} \
         (states {states:?})",
        survivors.len()
    );
}
