//! Shared test support: a data directory that reaps the daemons it causes.
//!
//! **The measurement this exists for.** A `cargo test` run used to leave live
//! `sgt --data-dir /tmp/.tmpXXXXXX daemon` processes behind — 89 of them had
//! accumulated on one container over a working day, each holding a data dir
//! that no longer existed. Every one came from the auto-spawn path: a test
//! runs the binary, the client starts a detached daemon because none is
//! running, and the daemon outlives the test by design. The tests that
//! remembered to kill it did so at the end of the body, which is exactly the
//! line a failed assertion skips.
//!
//! So the data dir owns the reaping. [`DataDir`] kills every daemon whose
//! command line names it when it goes out of scope — on the panic path too,
//! because `Drop` runs while unwinding — and *verifies* that none is left,
//! which is what makes a future leak a failing test rather than a process
//! nobody counts. The suites take `&DataDir` rather than `&Path` wherever
//! they run the binary, so a new test cannot quietly opt out of it: pointing
//! `sgt` at a bare `TempDir` no longer type-checks.
//!
//! Kept deliberately dependency-free (`kill(1)` and `/proc`, both of which
//! these suites already use) — a test rig is not a place to spend the
//! milestone's dependency budget.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tempfile::TempDir;

/// How long a daemon gets to exit after SIGTERM before SIGKILL.
const TERM_GRACE: Duration = Duration::from_secs(10);
/// How long it then gets to disappear from the process table.
const KILL_GRACE: Duration = Duration::from_secs(5);

/// The signal a daemon actually needed before it went away.
///
/// Reported rather than inferred: the reaper records what it *sent*, so a
/// change to the escalation order shows up in the report instead of hiding
/// behind "it died, didn't it".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReapSignal {
    /// The polite one. The daemon runs its shutdown path and returns from
    /// `main`, which is also the only path that flushes anything registered
    /// to run at exit — coverage profiles among them.
    Term,
    /// The rude one, used only after `TERM_GRACE` elapsed. Nothing at-exit
    /// runs: a SIGKILLed daemon contributes no coverage profile (measured,
    /// see `scripts/coverage/README.md`), so a run where this appears is a
    /// run whose numbers are short by whatever that daemon executed.
    Kill,
}

impl ReapSignal {
    /// The `kill(1)` flag that delivers it.
    fn flag(self) -> &'static str {
        match self {
            Self::Term => "-TERM",
            Self::Kill => "-KILL",
        }
    }
}

impl std::fmt::Display for ReapSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Term => "SIGTERM",
            Self::Kill => "SIGKILL",
        })
    }
}

/// One reaped daemon and the strongest signal the reaper had to send it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReapedDaemon {
    pub pid: u32,
    pub signal: ReapSignal,
}

/// A temporary sergeant data dir that reaps the daemons running on it.
///
/// Construct one wherever a test points the `sgt` binary at a data dir: any
/// client command may auto-spawn a daemon, and that daemon is detached, so
/// nothing else in the process tree will clean it up.
#[derive(Debug)]
pub struct DataDir {
    temp: TempDir,
}

impl DataDir {
    /// A fresh empty data dir.
    pub fn new() -> Self {
        Self {
            temp: TempDir::new().expect("tempdir"),
        }
    }

    /// The path to hand to `--data-dir`.
    pub fn path(&self) -> &Path {
        self.temp.path()
    }

    /// The path as the string a command line wants.
    pub fn display(&self) -> String {
        self.temp.path().display().to_string()
    }

    /// Kill every daemon on this data dir and wait for it to go, returning
    /// the pids that had to be killed and the signal each one needed.
    ///
    /// Idempotent, and safe to call from a test that also stops its daemon
    /// the polite way — a data dir with no daemon on it reaps nothing.
    pub fn reap(&self) -> Vec<ReapedDaemon> {
        reap_daemons(self.temp.path())
    }

    /// The pids of daemons currently running on this data dir.
    pub fn daemon_pids(&self) -> Vec<u32> {
        daemon_pids(self.temp.path())
    }
}

impl Default for DataDir {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DataDir {
    fn drop(&mut self) {
        reap_daemons(self.temp.path());
        let survivors = daemon_pids(self.temp.path());
        // A survivor here is a daemon that ignored SIGTERM *and* SIGKILL, or
        // one this scan cannot see — either way the leak is real and this is
        // the moment it is cheapest to notice. Not while already unwinding:
        // a panic inside a `Drop` during a panic aborts the process and
        // destroys the failure the test was reporting.
        assert!(
            survivors.is_empty() || std::thread::panicking(),
            "a daemon survived its data dir {:?}: pids {survivors:?}. Every test that \
             can auto-spawn must leave nothing behind — a leaked daemon holds a deleted \
             directory open and accumulates across runs.",
            self.temp.path()
        );
    }
}

/// The pids of `sgt … --data-dir <dir> … daemon` processes, from `/proc`.
///
/// Matched on argv, not on a substring of the joined command line: the data
/// dir has to be the actual value of `--data-dir`, and `daemon` an actual
/// argument, so a test *client* command that merely mentions the path (or a
/// grep of this very file) is not mistaken for a daemon.
pub fn daemon_pids(data_dir: &Path) -> Vec<u32> {
    let wanted = data_dir.to_string_lossy().to_string();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    let mut pids = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let Ok(pid) = name.parse::<u32>() else {
            continue;
        };
        let Ok(raw) = std::fs::read(entry.path().join("cmdline")) else {
            continue;
        };
        let argv: Vec<String> = String::from_utf8_lossy(&raw)
            .split('\0')
            .filter(|arg| !arg.is_empty())
            .map(str::to_string)
            .collect();
        let Some(program) = argv.first() else {
            continue;
        };
        let is_sgt = PathBuf::from(program)
            .file_name()
            .is_some_and(|name| name == "sgt");
        let names_dir = argv
            .windows(2)
            .any(|pair| pair[0] == "--data-dir" && pair[1] == wanted);
        if is_sgt && names_dir && argv.iter().any(|arg| arg == "daemon") {
            pids.push(pid);
        }
    }
    pids.sort_unstable();
    pids
}

/// SIGTERM every daemon on `data_dir`, then SIGKILL whatever is left.
///
/// Returns what was signalled and with what, so a test can assert the rig
/// did something rather than trusting it silently — and so the escalation
/// stops being invisible. The SIGKILL fallback used to fire without a word:
/// the run still went green, the daemon still died, and the only trace was a
/// coverage profile that never arrived. A `Kill` in this report is therefore
/// also announced on stderr, because `Drop` throws the value away and the
/// escalation is exactly the thing a discarded return value must not hide.
pub fn reap_daemons(data_dir: &Path) -> Vec<ReapedDaemon> {
    let pids = daemon_pids(data_dir);
    if pids.is_empty() {
        return Vec::new();
    }
    // Built from the signals actually sent, never from "it went away, so
    // TERM must have done it": that inference would report `Term` for a
    // reaper that had been changed to open with SIGKILL.
    let mut reaped = signal(&pids, ReapSignal::Term);
    if !wait_until_gone(data_dir, TERM_GRACE) {
        for killed in signal(&daemon_pids(data_dir), ReapSignal::Kill) {
            match reaped.iter_mut().find(|seen| seen.pid == killed.pid) {
                Some(seen) => seen.signal = killed.signal,
                None => reaped.push(killed),
            }
        }
        wait_until_gone(data_dir, KILL_GRACE);
    }
    for daemon in &reaped {
        if daemon.signal == ReapSignal::Kill {
            eprintln!(
                "support::reap_daemons: daemon {} on {:?} ignored SIGTERM for {}s and needed \
                 SIGKILL — it flushed nothing at exit (coverage profiles included)",
                daemon.pid,
                data_dir,
                TERM_GRACE.as_secs()
            );
        }
    }
    reaped
}

fn signal(pids: &[u32], signal: ReapSignal) -> Vec<ReapedDaemon> {
    pids.iter()
        .map(|&pid| {
            let _ = std::process::Command::new("kill")
                .arg(signal.flag())
                .arg(pid.to_string())
                .status();
            ReapedDaemon { pid, signal }
        })
        .collect()
}

fn wait_until_gone(data_dir: &Path, budget: Duration) -> bool {
    let deadline = Instant::now() + budget;
    loop {
        if daemon_pids(data_dir).is_empty() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
