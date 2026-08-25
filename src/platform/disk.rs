//! Free disk space (#81).
//!
//! `df` remains the mechanism — not a `libc`/`statvfs` binding. The module
//! this fact used to live in (`src/backend/docker.rs`) explicitly declined
//! that binding "for one syscall" in favor of the same shell-out posture the
//! rest of this crate already takes for external facts (`kill` for signals,
//! `docker` itself for the container adapter); `df` is present on every
//! measured environment. #81 asks whether that tradeoff still holds now that
//! the shell-out has a *measured* portability cost (GNU's `--output` extension
//! failing outright on BSD/macOS `df`), not just a theoretical one. It still
//! holds: the fix below is a second, POSIX-portable invocation shape
//! (`df -k <path>`, no `--output`) plus positional column parsing, which adds
//! no dependency and keeps every platform on the same "shell out to a
//! coreutil" idiom as the rest of the crate. The cost this trades in is the
//! brittleness `--output` was chosen to avoid — a wrapped or reformatted `df`
//! table would defeat [`parse_bsd_avail_kb`] — accepted here because it is
//! the only POSIX-portable option, it is covered by an injected-probe test
//! that pins the exact BSD column shape, and a parse failure degrades to
//! `None` (`doctor`'s already-honest "could not be measured"), never a wrong
//! number.

use std::path::Path;
use std::process::Command;

/// Bytes of free space at `path`, or `None` if it cannot be measured —
/// `sgt doctor`'s disk-pressure check degrades to "could not be measured" in
/// that case (`src/cli.rs`'s `disk_pressure_check`).
pub fn free_space(path: &Path) -> Option<u64> {
    let kb = raw_avail_kb(path)?;
    Some(kb * 1024)
}

/// GNU coreutils' shape: `df -k --output=avail <path>` prints exactly one
/// column, so the answer is whatever numeric text the last non-empty line
/// holds. Unchanged from this fact's pre-boundary form — Linux behavior does
/// not move.
fn parse_gnu_avail_kb(stdout: &str) -> Option<u64> {
    stdout.lines().last()?.trim().parse().ok()
}

/// BSD/macOS `df` has no `--output`; `df -k <path>` prints a header row and
/// one data row (`Filesystem 1024-blocks Used Available Capacity Mounted on`
/// on macOS's own `df`), so the `Available` column has to be found by
/// position rather than assumed. This is the "decision logic" ADR 0002 D3
/// asks for: pure, unconditionally compiled, exercised by
/// [`bsd_shape_parses_positionally`] below without a macOS host in sight.
fn parse_bsd_avail_kb(stdout: &str) -> Option<u64> {
    let mut lines = stdout.lines();
    let header = lines.next()?;
    let idx = header
        .split_whitespace()
        .position(|col| col.eq_ignore_ascii_case("available"))?;
    let data = lines.next()?;
    data.split_whitespace().nth(idx)?.parse().ok()
}

/// The raw fact: `df`'s stdout for `path`, parsed by whichever shape it
/// turns out to hold. Trying the single-column GNU shape first and falling
/// back to BSD's positional shape is safe in both directions — GNU's output
/// has no header row an "Available" search could latch onto, and BSD's data
/// row is never itself a bare integer — so this does not need a
/// `#[cfg(target_os = ...)]` split at the parsing layer, only at the
/// invocation layer below.
fn parse_avail_kb(stdout: &str) -> Option<u64> {
    parse_gnu_avail_kb(stdout).or_else(|| parse_bsd_avail_kb(stdout))
}

#[cfg(target_os = "linux")]
fn raw_avail_kb(path: &Path) -> Option<u64> {
    let output = Command::new("df")
        .args(["-k", "--output=avail"])
        .arg(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_avail_kb(&String::from_utf8_lossy(&output.stdout))
}

/// **Verified 2026-08-15** on a real macOS host (Apple M3 Pro, macOS 26.6.1,
/// `sergeant-rs-workspace's knowledge/evidence/host-measurements/macbook.md`) — closes #81. `--output` is a GNU
/// extension BSD/macOS `df` does not have, so this drops it and leans on
/// [`parse_bsd_avail_kb`] instead; `tests/m6_surfaces.rs`'s doctor
/// `disk_pressure` checks and `scripts/coverage/common.sh`'s own
/// (separately GNU-only, separately fixed) `df` call both exercised the real
/// `df` binary on this host during the same trip.
#[cfg(target_os = "macos")]
fn raw_avail_kb(path: &Path) -> Option<u64> {
    let output = Command::new("df").arg("-k").arg(path).output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse_avail_kb(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn raw_avail_kb(_path: &Path) -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gnu_shape_parses_the_single_column() {
        assert_eq!(parse_avail_kb("Avail\n1048576\n"), Some(1048576));
        // `--output=avail` alone, no header, is what production actually
        // sends through this path.
        assert_eq!(parse_avail_kb("1048576\n"), Some(1048576));
    }

    /// Pins [`parse_bsd_avail_kb`]'s column lookup — the decision logic a
    /// macOS host would exercise for real, run here on whatever host is
    /// building this crate (ADR 0002 D3). Reverting the `position` search to
    /// a fixed index fails this the moment a column shifts.
    #[test]
    fn bsd_shape_parses_positionally() {
        let stdout = "Filesystem 1024-blocks Used Available Capacity Mounted on\n\
                       /dev/disk3s1 976490568 400000000 500000000 45% /\n";
        assert_eq!(parse_avail_kb(stdout), Some(500_000_000));
    }

    #[test]
    fn missing_available_column_is_none_not_a_wrong_number() {
        let stdout = "Filesystem Used Capacity Mounted on\n/dev/disk3s1 400000000 45% /\n";
        assert_eq!(parse_avail_kb(stdout), None);
    }

    #[test]
    fn empty_output_is_none() {
        assert_eq!(parse_avail_kb(""), None);
    }
}
