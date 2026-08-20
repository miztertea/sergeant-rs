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
//! Kept deliberately dependency-free (`kill(1)` and `src/platform/process.rs`'s
//! `running_processes`, which these suites already use) — a test rig is not a
//! place to spend the milestone's dependency budget.
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

/// Where [`DataDir::new`] roots its `TempDir`.
///
/// Not `<crate root>/target/tmp`: several suites walk *up* from a spawned
/// `sgt`'s working directory looking for a git repository (`fn sgt`'s own
/// doc comment in `tests/m2_daemon_api.rs` explains why the data dir must
/// stay outside one), and a data dir nested under this checkout's own
/// `target/` sits inside that checkout — the walk finds this repo's `.git`
/// and the daemon materializes a real workspace the test never asked for
/// (measured: `t7_cli_end_to_end_auto_spawn_and_second_daemon_fails_closed`
/// and two siblings went from `pending` to `blocked` the moment the base
/// moved under `target/`). `/var/tmp/<name>` is the already-established
/// disk-backed, outside-any-checkout location for exactly this class of
/// rig (`docs/DEVELOPMENT.md`, `docs/environments/cerberus.md`'s #70 row) —
/// real disk on every measured Linux/macOS host, confirmed here via `df -h
/// /var/tmp` matching the ext4 root rather than tmpfs. Its absence (e.g. an
/// untested Windows host) falls back to `TempDir::new()`'s own default
/// rather than failing outright — this fix targets the measured incident,
/// not every platform this crate might someday run on.
pub(crate) fn disk_backed_tmp_base() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("SGT_TEST_TMPDIR") {
        return Some(PathBuf::from(dir));
    }
    let var_tmp = PathBuf::from("/var/tmp");
    var_tmp.is_dir().then(|| var_tmp.join("sgt-rs-tests"))
}

/// Marker `[DataDir::new]` writes into every rig it creates, naming the pid
/// of the process that owns it.
///
/// #113: a suite process that gets `SIGKILL`ed mid-run (the R-MVP1-7
/// ceiling, or the harness killing a backgrounded `cargo test`) never runs
/// `Drop` — no destructor of any kind reaches a rig left behind that way, so
/// this marker plus [`reap_orphaned_rigs`] is the mechanism that does,
/// checked at the start of the *next* run rather than relied on from the
/// dead one.
pub(crate) const RIG_OWNER_PID_FILE: &str = ".owner-pid";

fn write_owner_pid(dir: &Path) {
    let _ = std::fs::write(dir.join(RIG_OWNER_PID_FILE), std::process::id().to_string());
}

/// Whether `pid` is currently a live process, decided the same
/// platform-correct way the product itself decides it — never a bare
/// `/proc` read.
///
/// **W7 (#113 platform-correctness fixer).** This function used to read
/// `Path::new("/proc").join(pid.to_string()).is_dir()` directly, with no
/// `cfg` guard at all. macOS has no `/proc`, so on macOS that check is
/// `false` for *every* pid, including live ones, and
/// [`reap_orphaned_rigs`] would then delete the rig of a run that is still
/// using it — exactly the "reaping a live run's state would be a worse
/// defect than the leak this closes" failure that function's own doc
/// comment calls out.
///
/// Ponytail rung **R1** (reuse existing machinery, not a new one):
/// [`platform::process::process_alive`](sergeant_rs::platform::process::process_alive)
/// (`src/platform/process.rs`) already solves this identical fact behind
/// the ADR 0002 platform boundary — `#[cfg(target_os = "linux")]` reads
/// `/proc` exactly as this function used to, `#[cfg(target_os = "macos")]`
/// shells to `kill -0` (marked **UNVERIFIED** there — never run on a real
/// macOS host; verified by running `src/platform/process.rs`'s suite, and
/// this reaper's own suite, on one), and it is `pub`, so this integration
/// test can reach it directly. Its own fail-closed direction — assume
/// alive, never conclude a pid is gone, on a platform this cannot evidence
/// — is exactly the direction this reaper needs, for free. **R7 (a second,
/// locally `cfg`-gated copy) was considered and rejected**: duplicating the
/// macOS arm here would be a second UNVERIFIED implementation to keep in
/// sync with `process.rs`'s own — the reinvention this sprint's own review
/// exists to catch (`LESSONS.md` L18; this is the third instance of it in
/// this sprint alone). The module doc's "dependency-free" pledge above is
/// about spending this crate's *external* dependency budget; reusing this
/// crate's own already-public platform module is not that.
fn pid_is_alive(pid: u32) -> bool {
    sergeant_rs::platform::process::process_alive(pid)
}

/// Reap rig directories directly under `base` whose owning process is no
/// longer alive, returning the paths removed.
///
/// This is the start-of-run reaper #113 asks for, not another `Drop` guard —
/// `Drop` is structurally incapable of running for a `SIGKILL`ed process, so
/// the only cleanup that survives the way these processes actually die is
/// one that runs when the *next* run starts. [`DataDir::new`] calls this
/// before creating its own rig, so it sweeps up whatever the last run left.
///
/// A directory with no readable [`RIG_OWNER_PID_FILE`] marker (predates this
/// fix, or is mid-write) is left alone rather than guessed about, and so is
/// one whose marker names a pid still present in `/proc` — that second check
/// is what makes this safe to call from a suite running concurrently with
/// others sharing the same base directory: reaping a live run's state would
/// be a worse defect than the leak this closes.
///
/// Ponytail rung **R7** (new machinery): R2 (reuse `DataDir`'s existing
/// `Drop`-based daemon reaper) fails outright — `Drop` cannot run for a
/// `SIGKILL`ed process, which is the whole premise of #113. R4 (a bare
/// `/proc` liveness check) is necessary but not sufficient alone; it supplies
/// no reap-at-start trigger without the marker file pairing it to a rig. R5
/// doesn't apply — this stays dependency-free per the module doc above. The
/// marker-plus-scan pair is the minimum that works.
pub(crate) fn reap_orphaned_rigs(base: &Path) -> Vec<PathBuf> {
    let mut reaped = Vec::new();
    let Ok(entries) = std::fs::read_dir(base) else {
        return reaped;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Ok(contents) = std::fs::read_to_string(path.join(RIG_OWNER_PID_FILE)) else {
            continue;
        };
        let Ok(pid) = contents.trim().parse::<u32>() else {
            continue;
        };
        if pid_is_alive(pid) {
            continue;
        }
        if std::fs::remove_dir_all(&path).is_ok() {
            reaped.push(path);
        }
    }
    reaped
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
    /// A fresh empty data dir, disk-backed by default where that is safe.
    ///
    /// `TempDir::new()` honors `$TMPDIR`, which on a host like Cerberus is a
    /// 16 GB tmpfs `/tmp` — fine for the small rigs most suites need, but a
    /// gigabyte-scale blob-store capture (`tests/m7_docker_executor.rs`'s
    /// `large_captured_output_does_not_grow_this_process_proportionally`,
    /// contract scale 1 GiB) can fill it, and when it fills, every `Bash`
    /// output capture on the host starts failing `EDQUOT` under a command
    /// that still runs underneath — a broken shell, not an obvious full disk
    /// (#70, evidence in `docs/environments/cerberus.md`). An operator
    /// remembering to export `$TMPDIR` before running tests does not close
    /// that: the incident was an unsafe *default*. So the default here is
    /// real disk, when `disk_backed_tmp_base` finds one available; otherwise
    /// this falls back to `TempDir::new()`'s own default rather than
    /// failing a host this fix was never measured against.
    pub fn new() -> Self {
        let Some(base) = disk_backed_tmp_base() else {
            return Self {
                temp: TempDir::new().expect("tempdir"),
            };
        };
        std::fs::create_dir_all(&base).expect("create disk-backed test tmp base dir");
        // #113: sweep up whatever a prior, SIGKILLed run left behind before
        // adding this run's own rig — the only cleanup point that survives
        // how those processes actually die.
        reap_orphaned_rigs(&base);
        let temp = tempfile::Builder::new().tempdir_in(&base).expect("tempdir");
        write_owner_pid(temp.path());
        Self { temp }
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

/// The pids of `sgt … --data-dir <dir> … daemon` processes.
///
/// Matched on argv, not on a substring of the joined command line: the data
/// dir has to be the actual value of `--data-dir`, and `daemon` an actual
/// argument, so a test *client* command that merely mentions the path (or a
/// grep of this very file) is not mistaken for a daemon.
///
/// **First-contact macOS fix (path-to-mac.md trip, 2026-08-15).** This used
/// to read `/proc` directly with no `cfg` guard at all — on macOS
/// `std::fs::read_dir("/proc")` simply errors, so this silently returned an
/// empty `Vec` for every call, on every host, including ones with a live
/// daemon: `DataDir::Drop`'s survivor assertion never fires (masking a real
/// leak), and any test asserting a specific spawned count (e.g.
/// `the_data_dir_guard_reaps_the_daemon_a_client_command_spawns`) sees `[]`
/// and fails. Same Ponytail **R1** reuse [`pid_is_alive`] above already
/// applies: [`sergeant_rs::platform::process::running_processes`]
/// (`src/platform/process.rs`) is the same pid+argv fact behind the ADR 0002
/// platform boundary, `#[cfg]`-correct on both Linux (`/proc`) and macOS
/// (`ps -axo pid=,command=`) — reusing it here rather than adding a second,
/// locally `cfg`-gated copy is the same R7-rejection reasoning `pid_is_alive`
/// already argues.
pub fn daemon_pids(data_dir: &Path) -> Vec<u32> {
    let wanted = data_dir.to_string_lossy().to_string();
    let Some(processes) = sergeant_rs::platform::process::running_processes() else {
        return Vec::new();
    };
    let mut pids = Vec::new();
    for process in processes {
        let Some(program) = process.argv.first() else {
            continue;
        };
        let is_sgt = PathBuf::from(program)
            .file_name()
            .is_some_and(|name| name == "sgt");
        let names_dir = process
            .argv
            .windows(2)
            .any(|pair| pair[0] == "--data-dir" && pair[1] == wanted);
        if is_sgt && names_dir && process.argv.iter().any(|arg| arg == "daemon") {
            pids.push(process.pid);
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
    // Built from the escalation branch this call actually took, never from
    // "it went away, so TERM must have done it": that inference would report
    // `Term` for a reaper that had been changed to open with SIGKILL.
    //
    // What it is not: proof of the signal the kernel delivered. The label and
    // `ReapSignal::flag()` are two halves of the same claim, so a mutation to
    // `flag()` alone moves both. m2's reaper test therefore checks the
    // *daemon's* evidence — descriptor removed, `daemon.stopped` journaled —
    // beside this label, because only the daemon can testify to what it got.
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

/// #83: a freshly written, freshly `chmod +x`'d stand-in script can
/// transiently fail `execve(2)` with `ETXTBSY` ("text file busy", `os error
/// 26`) while another handle on the same inode is still open for writing —
/// under `cargo test`'s default thread parallelism, a sibling test's
/// fork-to-exec window can overlap this one's write. Retry until the exec
/// stops being refused, or surface any other failure immediately.
pub fn wait_until_executable(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while let Err(e) = std::process::Command::new(path).arg("--version").output() {
        assert!(
            e.raw_os_error() == Some(26) && Instant::now() < deadline,
            "the stand-in at {path:?} is not runnable: {e}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

// ----------------------------------------------------- estate fixtures (§4, §6)

/// Run git in `dir` with a fixed identity and no ambient config, panicking
/// with git's own diagnostic. Shared so every suite's estate fixtures agree
/// on the hermetic environment (`GIT_CONFIG_GLOBAL=/dev/null` and friends)
/// rather than each re-deriving it.
pub fn git(dir: &Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
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
        "git {args:?} failed in {}: {}",
        dir.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// A git repository at `path` with one commit. Returns its HEAD SHA.
pub fn init_repo(path: &Path) -> String {
    std::fs::create_dir_all(path).expect("repo dir");
    git(path, &["init", "-b", "main"]);
    std::fs::write(path.join("README.md"), "# fixture\n").expect("write file");
    git(path, &["add", "."]);
    git(path, &["commit", "-m", "initial"]);
    git(path, &["rev-parse", "HEAD"])
}

/// Scaffold a valid estate at `root` (estate-root §4.1, §6.1).
///
/// Writes a `sergeant.toml` declaring `[estate] name` and one `[[repo]]` per
/// entry of `repos` — **no `path` keys**, because mounts are derived — and
/// creates a real git checkout at `root/repos/<name>` for each. Returns the
/// HEAD SHA of each mount, in `repos` order.
///
/// This is the shape every estate-scoped command now requires: exact-root
/// admission means a bare `TempDir` with a git repo in it is no longer a
/// workspace, and `[[repo]]` entries must resolve to `repos/<name>`.
pub fn scaffold_estate(root: &Path, name: &str, repos: &[&str]) -> Vec<String> {
    std::fs::create_dir_all(root).expect("estate root");
    let mut manifest = format!("[estate]\nname = {name:?}\n");
    let mut heads = Vec::with_capacity(repos.len());
    for repo in repos {
        manifest.push_str(&format!("\n[[repo]]\nname = {repo:?}\n"));
        heads.push(init_repo(&root.join("repos").join(repo)));
    }
    std::fs::write(root.join("sergeant.toml"), manifest).expect("write sergeant.toml");
    heads
}

/// [`scaffold_estate`] for the common single-repository case: an estate
/// named `name` with one mount also named `name`. Returns the mount path and
/// its HEAD SHA.
pub fn scaffold_solo_estate(root: &Path, name: &str) -> (PathBuf, String) {
    let head = scaffold_estate(root, name, &[name]).remove(0);
    (root.join("repos").join(name), head)
}
