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
//! Both are Linux-only via `/proc` today; both get a macOS arm here, and
//! **both macOS arms are UNVERIFIED** — never executed on a real macOS host.
//! They close #18 when someone measures them there, not when this lands.

/// One running process a liveness scan can see: its pid, and — wherever the
/// platform can produce it — its argv, already tokenized into separate
/// arguments (never a single joined command line: see
/// `backend::claude::cmdline_names_session`'s doc for why a joined line is
/// exactly the false-positive shape this fact must not produce).
#[derive(Debug, Clone)]
pub struct ProcessArgv {
    pub pid: u32,
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
        processes.push(ProcessArgv { pid, argv });
    }
    Some(processes)
}

#[cfg(target_os = "linux")]
fn raw_process_alive(pid: u32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

/// **UNVERIFIED** — never executed against a real macOS host. There is no
/// `/proc` to scan; `ps -axo pid=,command=` lists every process with its pid
/// and full command line in one shot, tokenized here on whitespace. That
/// tokenization is the one place this arm is weaker than Linux's byte-exact
/// NUL-split argv: a quoted argument containing a space would defeat it. The
/// launch grammar this fact is actually matched against — `--session-id
/// <uuid>` / `--resume <uuid>`, see `backend::claude::cmdline_names_session`
/// — never quotes, so this is judged sufficient for the fact this function
/// serves, not offered as a general argv parser.
#[cfg(target_os = "macos")]
fn raw_running_processes() -> Option<Vec<ProcessArgv>> {
    let output = std::process::Command::new("ps")
        .args(["-axo", "pid=,command="])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut processes = Vec::new();
    for line in text.lines() {
        let mut tokens = line.split_whitespace();
        let Some(pid) = tokens.next().and_then(|token| token.parse::<u32>().ok()) else {
            continue;
        };
        processes.push(ProcessArgv {
            pid,
            argv: tokens.map(str::to_string).collect(),
        });
    }
    Some(processes)
}

/// **UNVERIFIED** — never executed against a real macOS host. `kill -0`
/// signals nothing; a zero exit means the pid exists and this user may
/// signal it. Shells to the same `kill` binary `cli.rs`'s graceful-stop path
/// already invokes for SIGTERM, so this adds no new external dependency.
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
