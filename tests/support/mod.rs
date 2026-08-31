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
/// and the daemon materializes a real estate the test never asked for
/// (measured: `t7_cli_end_to_end_auto_spawn_and_second_daemon_fails_closed`
/// and two siblings went from `pending` to `blocked` the moment the base
/// moved under `target/`). `/var/tmp/<name>` is the already-established
/// disk-backed, outside-any-checkout location for exactly this class of
/// rig (`CONTRIBUTING.md`, `sergeant-rs-workspace's knowledge/evidence/host-measurements/cerberus.md`'s #70 row) —
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
/// exists to catch (the workspace knowledge library's Lesson L18; this is the third instance of it in
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
    /// (#70, evidence in `sergeant-rs-workspace's knowledge/evidence/host-measurements/cerberus.md`). An operator
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

/// SIGTERM every daemon on `data_dir`, then SIGKILL whatever is left — and
/// then everything those daemons had descended from them.
///
/// Returns what was signalled and with what, so a test can assert the rig
/// did something rather than trusting it silently — and so the escalation
/// stops being invisible. The SIGKILL fallback used to fire without a word:
/// the run still went green, the daemon still died, and the only trace was a
/// coverage profile that never arrived. A `Kill` in this report is therefore
/// also announced on stderr, because `Drop` throws the value away and the
/// escalation is exactly the thing a discarded return value must not hide.
///
/// **#310: the descendant sweep, and why its order is the whole point.** A
/// daemon's children are its children only while it is alive; the instant it
/// is signalled they reparent to init and no ancestry query can find them
/// again. That is how dozens of ~265 MB `opencode serve` probe children
/// accumulated over a working day while every orphan check reported clean —
/// both doctrinal patterns are `sgt`-shaped and the leaked species is named
/// `opencode`. So the tree is enumerated **before** the daemon is signalled,
/// and the recorded pids are signalled afterwards.
///
/// This is belt to the product's own braces (`backend::child`'s
/// `PR_SET_PDEATHSIG`, which is what actually closes the leak). It is the
/// half that still works on a platform with no parent-death signal, and the
/// half that reaches a child something other than a hardened probe spawned.
pub fn reap_daemons(data_dir: &Path) -> Vec<ReapedDaemon> {
    let pids = daemon_pids(data_dir);
    if pids.is_empty() {
        return Vec::new();
    }
    let descendants: Vec<u32> = pids
        .iter()
        .flat_map(|&pid| sergeant_rs::platform::process::descendants(pid))
        .filter(|pid| !pids.contains(pid))
        .collect();
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
    // The recorded tree, now that the daemons are gone. `kill -KILL -<pid>`
    // rather than `kill -KILL <pid>`: a hardened probe child leads its own
    // process group (`backend::child`), so the negated form reaches whatever
    // *it* spawned, and an already-empty group is `ESRCH` — success, not an
    // error worth reporting. Both forms are sent, because a descendant that
    // is not a group leader is reached only by the plain one.
    let survivors: Vec<u32> = descendants
        .into_iter()
        .filter(|&pid| sergeant_rs::platform::process::process_alive(pid))
        .collect();
    if !survivors.is_empty() {
        for pid in &survivors {
            let _ = std::process::Command::new("/bin/sh")
                .arg("-c")
                .arg(format!(
                    "kill -KILL -{pid} 2>/dev/null; kill -KILL {pid} 2>/dev/null"
                ))
                .status();
        }
        eprintln!(
            "support::reap_daemons: {} process(es) descended from the daemon(s) on {:?} \
             outlived them and were killed by recorded pid: {survivors:?}. #310: a probe \
             child that reaches this line is one the product's own PR_SET_PDEATHSIG should \
             already have taken.",
            survivors.len(),
            data_dir,
        );
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

// ------------------------------------------------- recording git shim (§18)

/// One recorded git invocation: where it ran and what it was asked to do.
///
/// Shared because two suites now assert on the *set of git verbs* a code path
/// runs — `e_admission_uses_no_network_git.rs` (§6.4: admission never touches
/// a network or branch-changing verb) and `e_sweep_uses_only_local_git.rs`
/// (#159: a sweep only ever reads refs and deletes `sergeant/*` branches).
/// Each still owns its own single-test process, for the `std::env::set_var`
/// reason those files document; only the shim mechanics are shared.
#[derive(Debug)]
pub struct Invocation {
    pub cwd: PathBuf,
    pub args: Vec<String>,
}

impl Invocation {
    /// The subcommand: the first argument that is not a global option.
    /// `runtime::git::command` always prefixes `--no-pager`, and several
    /// call sites pass `-c key=value`, so the verb is not simply `args[0]`.
    pub fn verb(&self) -> Option<&str> {
        let mut args = self.args.iter();
        while let Some(arg) = args.next() {
            if arg == "-c" || arg == "-C" {
                let _ = args.next();
                continue;
            }
            if arg.starts_with('-') {
                continue;
            }
            return Some(arg.as_str());
        }
        None
    }
}

pub fn parse_log(text: &str) -> Vec<Invocation> {
    text.lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (cwd, args) = line
                .split_once('\t')
                .unwrap_or_else(|| panic!("malformed recording line: {line:?}"));
            Invocation {
                cwd: PathBuf::from(cwd),
                args: args
                    .split('\u{1}')
                    .filter(|arg| !arg.is_empty())
                    .map(str::to_string)
                    .collect(),
            }
        })
        .collect()
}

/// A shim that records every invocation and then becomes the real git.
///
/// `$PWD` and the argument vector are written as one tab-separated record
/// with `\1`-separated arguments — a separator no git argument this codebase
/// produces can contain, so an argument with a space in it survives intact.
/// Appended with `>>` under a single `printf`, which is atomic enough for
/// concurrent children writing short records to the same file.
pub fn write_recording_shim(path: &Path, log: &Path, real_git: &Path) {
    // The separator is written into the script as a literal U+0001 byte
    // rather than as the escape `\001`: `sh` does not interpret escapes
    // inside double quotes, so the escape would end up in the log as four
    // characters and every record would be one unsplittable field.
    let unit = '\u{1}';
    std::fs::write(
        path,
        format!(
            "#!/bin/sh\n\
             record=\"$PWD\t\"\n\
             for arg in \"$@\"; do record=\"$record$arg{unit}\"; done\n\
             printf '%s\\n' \"$record\" >> {log:?}\n\
             exec {real_git:?} \"$@\"\n",
            log = log.display().to_string(),
            real_git = real_git.display().to_string(),
        ),
    )
    .expect("write shim");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path).expect("stat").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).expect("chmod");
    }
    // Runs the shim once (`--version`), which records a line of its own.
    // Truncate afterwards so the log holds only what the subject did.
    wait_until_executable(path);
    std::fs::write(log, "").expect("truncate the shim's own warm-up record");
}

/// The real `git` the shim execs into.
pub fn real_git() -> PathBuf {
    let out = std::process::Command::new("sh")
        .args(["-c", "command -v git"])
        .output()
        .expect("locate git");
    assert!(out.status.success(), "git must be on PATH for this test");
    PathBuf::from(String::from_utf8_lossy(&out.stdout).trim())
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
/// estate, and `[[repo]]` entries must resolve to `repos/<name>`.
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

/// A C1a/C1b Atlas-scan fixture unit: one section, deliberately minimal
/// (heading level 1, no coordinate) so callers only ever vary ordinal, title
/// and text (R2 — shared by `tests/c1a_compiled_context.rs` and
/// `tests/c1b_tiers_and_budget.rs`, which built byte-identical copies of this
/// before F-SI-01).
pub fn unit(
    ordinal: u64,
    title: &str,
    text: &str,
) -> sergeant_rs::runtime::atlas::scan::ScannedUnit {
    sergeant_rs::runtime::atlas::scan::ScannedUnit {
        ordinal,
        kind: sergeant_rs::domain::source::UnitKind::Section,
        heading_level: Some(1),
        title: Some(title.to_string()),
        byte_start: 0,
        byte_end: text.len() as u64,
        coordinate: None,
        text: text.to_string(),
    }
}

/// A C1a/C1b Atlas-scan fixture file wrapping [`unit`]s. See [`unit`]'s doc.
pub fn file(
    relative_path: &str,
    units: Vec<sergeant_rs::runtime::atlas::scan::ScannedUnit>,
) -> sergeant_rs::runtime::atlas::scan::ScannedFile {
    let bytes: u64 = units.iter().map(|u| u.text.len() as u64).sum();
    sergeant_rs::runtime::atlas::scan::ScannedFile {
        relative_path: relative_path.to_string(),
        content_hash: format!("hash/{relative_path}"),
        extractor: sergeant_rs::runtime::atlas::text::MARKDOWN_EXTRACTOR.to_string(),
        local_key: format!("key/{relative_path}"),
        byte_len: bytes,
        mtime_millis: None,
        units,
        syntax: None,
        parent: None,
    }
}

/// A C1a/C1b Atlas-scan fixture source wrapping [`file`]s. See [`unit`]'s doc.
pub fn scan(
    source_name: &str,
    kind: sergeant_rs::domain::source::SourceKind,
    authority: sergeant_rs::domain::source::AuthorityClass,
    files: Vec<sergeant_rs::runtime::atlas::scan::ScannedFile>,
) -> sergeant_rs::runtime::atlas::scan::SourceScan {
    let mut extractors = std::collections::BTreeSet::new();
    extractors.insert(sergeant_rs::runtime::atlas::text::MARKDOWN_EXTRACTOR.to_string());
    sergeant_rs::runtime::atlas::scan::SourceScan {
        source_name: source_name.to_string(),
        kind,
        authority,
        content_key: format!("{source_name}@generation-1"),
        revision: None,
        observed_at: sergeant_rs::domain::event::rfc3339_utc_now(),
        files,
        coverage: Vec::new(),
        extractors,
        datasets: Vec::new(),
        root: None,
        context_fields: sergeant_rs::runtime::atlas::tabular::ContextFields::none(),
    }
}

/// A cross-process mutex over a fixed OS resource a test cannot make
/// per-process-unique (#305: `t5_disabled_export_runs_no_exporter_machinery`
/// must bind the daemon's literal `DEFAULT_OTLP_ENDPOINT`, port 0 or a
/// per-process offset would test nothing — the whole point is standing in
/// for the address a regression would really dial).
///
/// Backed by an atomically-created lock file (`create_new`, so the OS
/// resolves the race), not `flock`, to stay dependency-free per this
/// module's own precedent. A holder that panics or is killed leaves the file
/// behind; [`Self::acquire`] treats a lock file older than `STALE_AFTER` as
/// abandoned and steals it rather than hanging forever.
pub struct CrossProcessLock {
    path: PathBuf,
}

const STALE_AFTER: Duration = Duration::from_secs(60);
const POLL_INTERVAL: Duration = Duration::from_millis(25);

impl CrossProcessLock {
    /// Block until the named lock is held exclusively by this call.
    ///
    /// `name` should identify the contended resource (e.g. a port number),
    /// not the test — two different tests contending the same resource must
    /// use the same name to actually serialize against each other.
    pub fn acquire(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("sgt-test-lock-{name}"));
        let deadline = Instant::now() + Duration::from_secs(120);
        loop {
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(_) => return Self { path },
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    let stale = std::fs::metadata(&path)
                        .ok()
                        .and_then(|meta| meta.modified().ok())
                        .and_then(|m| m.elapsed().ok())
                        .is_some_and(|age| age > STALE_AFTER);
                    if stale {
                        // A holder that never dropped this — the process was
                        // killed, not just the test failed. Reclaim rather
                        // than deadlock every future run.
                        let _ = std::fs::remove_file(&path);
                        continue;
                    }
                    if Instant::now() > deadline {
                        panic!(
                            "cross-process lock {path:?} held for over 120s; \
                             a holder is stuck or STALE_AFTER needs raising"
                        );
                    }
                    std::thread::sleep(POLL_INTERVAL);
                }
                Err(e) => panic!("acquire cross-process lock {path:?}: {e}"),
            }
        }
    }
}

impl Drop for CrossProcessLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Drive `POST /v1/intelligence/scan` to completion and hand back the
/// finished report (S6 scan front door).
///
/// The trigger accepts and returns a `scan_id`; the scan itself runs on the
/// daemon's own task and is followed through
/// `GET /v1/intelligence/scan/{scan_id}`. Every suite that scans goes
/// through this one helper, so no test can quietly go back to asserting on
/// an acceptance as if it were a report.
///
/// This is also what makes those suites *honest* rather than lucky. The
/// synchronous trigger they used to call was raced against one HTTP
/// request timeout, and `y6a`'s own repository scan had begun timing out in
/// CI — a real scan outrunning a fixed client bound, which is the same
/// defect the product had. Here the wait is over the scan's own reported
/// completion, and it carries **no time bound of its own**: elapsed time
/// never decides whether a scan was correct, because estate size is an
/// input and any duration chosen for it is wrong at some estate. A test
/// that hangs is the runner's to kill (`.config/nextest.toml`), which
/// reports "the harness gave up waiting" — a fact about the harness.
/// A bound asserted here would instead claim the scan failed, which is a
/// fact about the product this helper cannot know.
///
/// Returns the status and body of whatever last answered: the acceptance
/// itself when it carries no `scan_id` (the estate declares nothing to
/// scan — a real, immediate answer), otherwise the terminal poll, whose
/// `scanned` array is exactly the per-source report.
pub async fn scan_to_completion(
    http: &reqwest::Client,
    endpoint: &str,
    token: &str,
    body: &serde_json::Value,
) -> (reqwest::StatusCode, serde_json::Value) {
    let response = http
        .post(format!("{endpoint}/v1/intelligence/scan"))
        .bearer_auth(token)
        .json(body)
        .send()
        .await
        .expect("scan request");
    let status = response.status();
    let accepted: serde_json::Value = response.json().await.expect("json body");
    let Some(scan_id) = accepted["scan_id"].as_str().map(str::to_string) else {
        return (status, accepted);
    };
    let mut last: serde_json::Value;
    let mut last_status: reqwest::StatusCode;
    loop {
        let response = http
            .get(format!("{endpoint}/v1/intelligence/scan/{scan_id}"))
            .bearer_auth(token)
            .send()
            .await
            .expect("scan status request");
        last_status = response.status();
        last = response.json().await.expect("json body");
        // A poll that does not answer `200` is terminal, not slow. The only
        // non-success this endpoint has is `404` — never accepted here,
        // accepted before a restart, or aged past `RETAINED_SCANS`
        // (`intelligence_scan_status`'s own doc) — and no id in that state
        // ever becomes a tracked, completing scan. Waiting on it is waiting
        // forever, which is the one failure this unbounded loop must not
        // have; the status is reported rather than polled through.
        assert!(
            last_status.is_success(),
            "GET {endpoint}/v1/intelligence/scan/{scan_id} answered {last_status}: {last}\nthis              daemon does not track that scan, so no amount of further polling can complete it"
        );
        if last["state"] == "completed" {
            return (last_status, last);
        }
        // Display/poll cadence only: no outcome depends on this value.
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// One cited function, sliced the way an acceptance register's citation
/// guard needs to read it: the attributes written above its signature, and
/// its body.
///
/// Shared by `tests/a2_acceptance.rs`, `tests/c1_acceptance.rs` and
/// `tests/x5_a1a_acceptance.rs` (F-SI-01): this ~30-line line-based parser
/// used to be copy-pasted byte-for-byte into all three instead of living
/// once in this already-idiomatic shared module (Ponytail R2).
///
/// Line-based on purpose. The body ends at the first line that is this
/// signature's own indentation followed by a lone `}` — rustfmt guarantees
/// that line, and brace counting does not survive the `{{` inside a
/// `format!` string that several of the cited tests contain.
///
/// Attributes are collected as whole units, not lines: a multi-line
/// attribute such as `#[cfg_attr(\n  feature = "x",\n  ignore\n)]` is
/// walked backward from its closing `)]`, accumulating lines until the
/// accumulated text's `[`/`]` count balances on a line that itself starts
/// `#[` — the point the attribute actually opens. The old single-line-only
/// scan broke on the `)]` continuation line before ever seeing `ignore`
/// (F-IN-01): a cited test disabled via a multi-line attribute satisfied
/// the citation guard as if it still ran.
pub fn cited_function(text: &str, name: &str) -> Option<(Vec<String>, String)> {
    let needle = format!("fn {name}(");
    let lines: Vec<&str> = text.lines().collect();
    let signature = lines.iter().position(|line| line.contains(&needle))?;

    let mut attributes: Vec<String> = Vec::new();
    let mut buf: Vec<&str> = Vec::new();
    let mut index = signature;
    while index > 0 {
        index -= 1;
        let line = lines[index].trim();
        if buf.is_empty() {
            if line.starts_with("//") {
                continue;
            }
            if !line.starts_with("#[") && !line.ends_with(']') {
                break;
            }
        }
        buf.push(line);
        let joined = buf.iter().rev().copied().collect::<Vec<_>>().join("\n");
        let opens = joined.matches('[').count();
        let closes = joined.matches(']').count();
        if line.starts_with("#[") && opens == closes {
            attributes.push(joined);
            buf.clear();
        }
    }

    let indent = &lines[signature][..lines[signature].len() - lines[signature].trim_start().len()];
    let closing = format!("{indent}}}");
    let mut body = String::new();
    for line in &lines[signature + 1..] {
        if *line == closing {
            break;
        }
        body.push_str(line);
        body.push('\n');
    }
    Some((attributes, body))
}

#[cfg(test)]
mod cross_process_lock_tests {
    use super::CrossProcessLock;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Two contenders for the same name never hold it at once; a fresh name
    /// never blocks. This is the decisive property #305's fix depends on —
    /// without it, the port-0 alternative would be indistinguishable from a
    /// lock that doesn't actually exclude.
    #[test]
    fn excludes_concurrent_holders_of_the_same_name() {
        let name = format!("test-{}", std::process::id());
        let overlap = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();
        for _ in 0..8 {
            let name = name.clone();
            let overlap = Arc::clone(&overlap);
            let peak = Arc::clone(&peak);
            handles.push(std::thread::spawn(move || {
                let _lock = CrossProcessLock::acquire(&name);
                let now = overlap.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(now, Ordering::SeqCst);
                std::thread::sleep(std::time::Duration::from_millis(10));
                overlap.fetch_sub(1, Ordering::SeqCst);
            }));
        }
        for h in handles {
            h.join().expect("thread");
        }
        assert_eq!(
            peak.load(Ordering::SeqCst),
            1,
            "at most one holder of the same lock name at a time"
        );
    }
}
