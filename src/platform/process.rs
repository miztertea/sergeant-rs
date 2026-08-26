//! Process liveness (#18).
//!
//! Two distinct facts live here, both platform-dependent, kept separate
//! because they answer different questions and fail closed in opposite
//! directions:
//!
//! - [`running_processes`] backs `backend::claude`'s per-turn liveness scan.
//!   It is identity-verified evidence about a *specific conversation's turn*
//!   (argv carries the session id), and failing to gather it means
//!   `Liveness::Unknowable` — the engine then fails a resume closed rather
//!   than guess (§25).
//! - [`process_alive`] backs `daemon.rs`'s spawn/wait/stop paths. It answers
//!   the coarser "does this pid exist at all", with no identity check, and
//!   failing to gather it means *assume alive* — the fail-closed direction
//!   for "should I spawn a second daemon over this one" is the opposite of
//!   the one above.
//!
//! Both are Linux-only via `/proc` today; both get a macOS arm here.
//! **Verified on a real macOS host 2026-08-15** (Apple M3 Pro, macOS 26.6.1,
//! `sergeant-rs-workspace's knowledge/evidence/host-measurements/macbook.md`, the path-to-mac.md arrival trip): the full
//! suite ran real `ps`/`kill` calls end to end (`tests/m4_backends.rs`'s
//! liveness tests, `tests/support/mod.rs`'s `daemon_pids` reaper, and
//! `tests/m6_surfaces.rs`'s `tui_pid`, all of which reuse
//! [`running_processes`] rather than duplicating a `/proc`-only scan — three
//! duplicate copies of that same scan were found and fixed to reuse this
//! module during the same trip). Closes #18.

/// One running process a liveness scan can see: its pid, its parent's pid,
/// and — wherever the platform can produce it — its argv, already tokenized
/// into separate arguments (never a single joined command line: see
/// `backend::claude::cmdline_names_session`'s doc for why a joined line is
/// exactly the false-positive shape this fact must not produce).
#[derive(Debug, Clone)]
pub struct ProcessArgv {
    pub pid: u32,
    /// The parent this process had **at the moment of the scan**. `None`
    /// where the platform arm could not read it (a process that exited
    /// between being listed and being read, a `ps` line missing the column).
    ///
    /// Deliberately not cached anywhere: a parent that dies reparents its
    /// children to init, so this fact is only ever true of the instant it was
    /// gathered. [`descendants`] exists precisely so a caller captures the
    /// tree *before* it kills the root, rather than looking for it after.
    pub ppid: Option<u32>,
    pub argv: Vec<String>,
}

/// Enumerate running processes' pid + argv, or `None` if this platform has
/// no mechanism to gather it at all (never true on a measured platform
/// today — Linux via `/proc`, macOS via `ps` — but the caller's fail-closed
/// path exists for exactly this case). A transient failure reading one
/// process's own details (it exited between being listed and being read)
/// just drops that one entry, unchanged from this fact's pre-boundary
/// Linux-only form.
pub fn running_processes() -> Option<Vec<ProcessArgv>> {
    raw_running_processes()
}

/// Whether a pid currently names a live process — no identity check, just
/// existence. Elsewhere unknowable without more machinery, this reports
/// alive: the fail-closed direction for the spawn/wait decisions that call
/// it (never conclude a daemon is gone, and double-spawn over it, on a
/// platform this cannot evidence).
pub fn process_alive(pid: u32) -> bool {
    raw_process_alive(pid)
}

/// Every process currently descended from `root`, breadth-first, `root`
/// itself excluded.
///
/// **Why this exists (#310).** A daemon's probe children are its *children*
/// only while it is alive: the instant it is SIGKILLed they reparent to init
/// and no ancestry query can find them again. So a reaper that wants to take
/// a whole tree down has to enumerate the tree first and signal the recorded
/// pids afterwards — "kill the daemon, then look for its children" is the
/// shape that let ~265 MB `opencode serve` probe children accumulate for a
/// working day while every orphan check reported clean.
///
/// One scan, then a pure walk over it: re-scanning per level would let a
/// process appear under two different parents across scans and be visited
/// twice (or missed entirely). A cycle is impossible in a real process table,
/// but the walk carries a `seen` set anyway — this reads a live kernel
/// surface, and a torn read must not become an infinite loop in a `Drop`.
///
/// Returns empty where [`running_processes`] cannot answer at all: the
/// fail-closed direction for a *reaper* is to signal nothing rather than
/// guess at pids it cannot evidence.
pub fn descendants(root: u32) -> Vec<u32> {
    let Some(processes) = running_processes() else {
        return Vec::new();
    };
    let mut children: std::collections::BTreeMap<u32, Vec<u32>> = std::collections::BTreeMap::new();
    for process in processes {
        if let Some(ppid) = process.ppid {
            children.entry(ppid).or_default().push(process.pid);
        }
    }
    let mut seen: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    let mut found = Vec::new();
    let mut queue = std::collections::VecDeque::from([root]);
    seen.insert(root);
    while let Some(pid) = queue.pop_front() {
        let Some(kids) = children.get(&pid) else {
            continue;
        };
        for &kid in kids {
            if seen.insert(kid) {
                found.push(kid);
                queue.push_back(kid);
            }
        }
    }
    found
}

#[cfg(target_os = "linux")]
fn raw_running_processes() -> Option<Vec<ProcessArgv>> {
    let entries = std::fs::read_dir("/proc").ok()?;
    let mut processes = Vec::new();
    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        else {
            continue;
        };
        let Ok(cmdline) = std::fs::read(entry.path().join("cmdline")) else {
            continue;
        };
        let argv = cmdline
            .split(|byte| *byte == 0)
            .filter(|arg| !arg.is_empty())
            .map(|arg| String::from_utf8_lossy(arg).into_owned())
            .collect();
        let ppid = std::fs::read_to_string(entry.path().join("status"))
            .ok()
            .as_deref()
            .and_then(parse_proc_status_ppid);
        processes.push(ProcessArgv { pid, ppid, argv });
    }
    Some(processes)
}

/// The `PPid:` line of `/proc/<pid>/status`.
///
/// `status` rather than `stat`: `stat`'s parent is field 4, *after* a `comm`
/// field the kernel writes in parentheses without escaping — a process named
/// `foo) 1 (bar` shifts every positional field after it, and a reaper that
/// mis-parses one line signals the wrong pid. `status` is line-oriented and
/// has no such hazard.
#[cfg(any(test, target_os = "linux"))]
fn parse_proc_status_ppid(status: &str) -> Option<u32> {
    status
        .lines()
        .find_map(|line| line.strip_prefix("PPid:"))
        .and_then(|value| value.trim().parse::<u32>().ok())
}

#[cfg(target_os = "linux")]
fn raw_process_alive(pid: u32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

/// `ps -axo pid=,ppid=,command=` output, one process per line: pid, parent
/// pid, then its full command line, tokenized here on whitespace. That
/// tokenization is the one
/// place this arm is weaker than Linux's byte-exact NUL-split argv: a quoted
/// argument containing a space would defeat it. The launch grammar this fact
/// is actually matched against — `--session-id <uuid>` / `--resume <uuid>`,
/// see `backend::claude::cmdline_names_session` — never quotes, so this is
/// judged sufficient for the fact this function serves, not offered as a
/// general argv parser. A line whose first token is not a bare pid (`ps`'s
/// own header, should the invocation ever regain one; a torn read's partial
/// first line) is dropped rather than turned into a bogus entry.
///
/// This is the "decision logic" ADR 0002 D3 asks for: exercised by the tests
/// below from whatever host builds this crate, without a macOS host in
/// sight. Unlike [`disk::parse_avail_kb`](super::disk), the Linux arm above
/// never has a production reason to call this — it reads `/proc` directly
/// and produces no text to tokenize — so there is no unconditionally-live
/// caller to keep it out of `cargo clippy`'s dead-code net on a Linux build.
/// Gating on `cfg(any(test, target_os = "macos"))` (rather than leaving it
/// fully unconditional) resolves that: on Linux it exists only for
/// `cargo test`, where it does have a caller, and on macOS it exists in
/// production too.
#[cfg(any(test, target_os = "macos"))]
fn parse_ps_output(stdout: &str) -> Vec<ProcessArgv> {
    let mut processes = Vec::new();
    for line in stdout.lines() {
        let mut tokens = line.split_whitespace();
        let Some(pid) = tokens.next().and_then(|token| token.parse::<u32>().ok()) else {
            continue;
        };
        // The parent column is required by the invocation above, so a line
        // whose second token is not a bare number is a line this parser does
        // not understand — dropped whole rather than turned into an entry
        // whose argv silently starts one token late.
        let Some(ppid) = tokens.next().and_then(|token| token.parse::<u32>().ok()) else {
            continue;
        };
        processes.push(ProcessArgv {
            pid,
            ppid: Some(ppid),
            argv: tokens.map(str::to_string).collect(),
        });
    }
    processes
}

/// **Verified 2026-08-15** on a real macOS host (Apple M3 Pro, macOS 26.6.1)
/// — closes #18. There is no `/proc` to scan; `ps -axo pid=,command=` lists
/// every process with its pid and full command line in one shot. Parsing is
/// [`parse_ps_output`], pinned by tests below.
#[cfg(target_os = "macos")]
fn raw_running_processes() -> Option<Vec<ProcessArgv>> {
    let output = std::process::Command::new("ps")
        .args(["-axo", "pid=,ppid=,command="])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(parse_ps_output(&String::from_utf8_lossy(&output.stdout)))
}

/// **Verified 2026-08-15** on a real macOS host (Apple M3 Pro, macOS 26.6.1)
/// — closes #18. `kill -0` signals nothing; a zero exit means the pid exists
/// and this user may signal it. Shells to the same `kill` binary `cli.rs`'s
/// graceful-stop path already invokes for SIGTERM, so this adds no new
/// external dependency.
#[cfg(target_os = "macos")]
fn raw_process_alive(pid: u32) -> bool {
    std::process::Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .output()
        .is_ok_and(|output| output.status.success())
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn raw_running_processes() -> Option<Vec<ProcessArgv>> {
    None
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn raw_process_alive(_pid: u32) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_shape_parses_pid_ppid_and_argv() {
        let stdout = "1234 1 /usr/bin/claude --session-id abc-123\n";
        let processes = parse_ps_output(stdout);
        assert_eq!(processes.len(), 1);
        assert_eq!(processes[0].pid, 1234);
        assert_eq!(processes[0].ppid, Some(1));
        assert_eq!(
            processes[0].argv,
            vec!["/usr/bin/claude", "--session-id", "abc-123"]
        );
    }

    /// A header row (should `ps -axo pid=,ppid=,command=` ever regain one) or
    /// a torn read's partial first line both have a non-numeric or missing
    /// leading token — neither may become a bogus pid entry the liveness
    /// scan then reasons about.
    #[test]
    fn line_without_a_leading_pid_is_dropped_not_a_bogus_entry() {
        let stdout = "  PID  PPID COMMAND\n5678 4321 /usr/bin/claude --resume def-456\n";
        let processes = parse_ps_output(stdout);
        assert_eq!(processes.len(), 1);
        assert_eq!(processes[0].pid, 5678);
        assert_eq!(processes[0].ppid, Some(4321));
    }

    /// #310: the parent column is what [`descendants`] walks. A line missing
    /// it must not silently become an entry whose argv begins one token late
    /// — `[/usr/bin/claude, --resume, def]` read from a two-column line would
    /// be a *different process's* command line as far as every argv matcher
    /// in this crate is concerned.
    #[test]
    fn line_without_a_parent_column_is_dropped_not_shifted() {
        assert!(parse_ps_output("5678 /usr/bin/claude --resume def-456\n").is_empty());
    }

    #[test]
    fn proc_status_ppid_is_read_from_its_own_line() {
        let status = "Name:\tsgt\nUmask:\t0022\nState:\tS (sleeping)\nTgid:\t42\nPid:\t42\n\
                      PPid:\t7\nTracerPid:\t0\n";
        assert_eq!(parse_proc_status_ppid(status), Some(7));
    }

    /// `Pid:` precedes `PPid:` in the real file and shares its prefix in the
    /// other direction; a matcher looking for `Pid:` anywhere in the line
    /// would answer with the process's own id. Pinned because getting this
    /// backwards makes every process its own parent, and [`descendants`]
    /// would then return nothing at all — a reaper that silently reaps
    /// nothing is the exact failure #310 is about.
    #[test]
    fn proc_status_ppid_is_not_confused_with_the_pid_line() {
        let status = "Pid:\t42\nPPid:\t7\n";
        assert_eq!(parse_proc_status_ppid(status), Some(7));
        let no_parent = "Name:\tinit\nPid:\t1\n";
        assert_eq!(parse_proc_status_ppid(no_parent), None);
    }

    /// A real tree, walked from a real root: this process's own children are
    /// whatever it spawns, and a grandchild must come back too — the probe
    /// children #310 is about are grandchildren of the test process whenever
    /// an adapter isolates its spawn on a helper thread.
    #[test]
    fn descendants_reaches_a_grandchild_not_just_a_child() {
        let mut child = std::process::Command::new("/bin/sh")
            .arg("-c")
            .arg("/bin/sh -c 'sleep 30' & sleep 30")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("spawn a two-level tree");
        let child_pid = child.id();
        // The grandchild is forked by the child, so it exists only after the
        // child has run — poll rather than sleep on a guessed duration.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let found = loop {
            let found = descendants(std::process::id());
            if found.len() >= 2 && found.contains(&child_pid) {
                break found;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "the spawned tree never appeared under this process: {found:?}"
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        let _ = child.kill();
        let _ = child.wait();
        for &pid in &found {
            if pid != child_pid {
                let _ = std::process::Command::new("kill")
                    .args(["-KILL", &pid.to_string()])
                    .status();
            }
        }
        assert!(
            found.contains(&child_pid),
            "the direct child {child_pid} was not in {found:?}"
        );
        assert!(
            found.len() >= 2,
            "only the direct child came back, so the walk never went a second level: {found:?}"
        );
    }

    #[test]
    fn descendants_never_includes_the_root_itself() {
        assert!(!descendants(std::process::id()).contains(&std::process::id()));
    }

    #[test]
    fn empty_line_is_dropped_not_a_bogus_entry() {
        assert!(parse_ps_output("\n").is_empty());
        assert!(parse_ps_output("").is_empty());
    }

    /// Pins the tokenization weakness the doc comment above concedes: a
    /// quoted argument containing a space is split into two argv entries
    /// instead of surviving as one, unlike Linux's byte-exact NUL-split
    /// `/proc` argv. This is judged sufficient today because the launch
    /// grammar this fact is matched against —
    /// `backend::claude::cmdline_names_session`'s `--session-id <uuid>` /
    /// `--resume <uuid>` — never quotes its value. If that grammar ever
    /// does gain a quoted argument, this assertion is where that surfaces:
    /// it pins CURRENT (weaker) behavior, not desired behavior, so it must
    /// fail the moment someone tries to rely on quoting surviving here.
    #[test]
    fn quoted_argument_with_a_space_is_split_current_known_weakness() {
        let stdout = "1234 1 /usr/bin/claude --session-id \"abc def\"\n";
        let processes = parse_ps_output(stdout);
        assert_eq!(processes.len(), 1);
        assert_eq!(
            processes[0].argv,
            vec!["/usr/bin/claude", "--session-id", "\"abc", "def\""]
        );
        assert_ne!(
            processes[0].argv.get(2),
            Some(&"abc def".to_string()),
            "if this now passes, the tokenizer has been fixed to preserve \
             quoted arguments — update this test and the doc comment above \
             `parse_ps_output` to stop calling it a known weakness"
        );
    }
}
