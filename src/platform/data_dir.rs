//! The pre-estate data-dir fallback tail (#82).
//!
//! `cli.rs::resolve_data_dir`'s ladder is `--data-dir`, then
//! `SGT_DATA_DIR`, then estate discovery — all three platform-neutral, and
//! **their precedence is an owner ruling tracked separately in #80**, not
//! something this module touches or re-litigates. Only the ladder's last
//! rung — what to do once no estate is found at all — is a platform fact:
//! freedesktop's `$XDG_DATA_HOME`/`~/.local/share` convention is a Linux
//! thing, not a macOS one, whose own convention is
//! `~/Library/Application Support`.

use std::ffi::OsString;
use std::path::PathBuf;

/// One platform's fallback-tail convention, as data rather than as a second
/// code path — the freedesktop and macOS tails differ only in *which*
/// directories they name, not in the shape of "check an env var, else fall
/// back to a fixed subdirectory of `$HOME`". Keeping that shape as one
/// function over a `Convention` value (below) is what lets both platforms'
/// conventions be exercised, unconditionally compiled, from any host (ADR
/// 0002 D3) — there is nothing here a `#[cfg(target_os = "macos")]` split
/// would gain.
struct Convention {
    /// The env var whose presence short-circuits straight to `$VAR/<suffix>`
    /// — freedesktop's `XDG_DATA_HOME`. macOS has no equivalent env-var
    /// override in its own convention.
    env_override: Option<(&'static str, &'static str)>,
    /// The fixed subdirectory of `$HOME` once no override applies.
    home_suffix: &'static str,
}

// Both conventions are always compiled and always tested (ADR 0002 D3), but
// only one is wired to `CURRENT` on any given host — the other is
// production-dead there and only reachable through the tests below, which
// is exactly the point: it lets a Linux host run macOS's decision logic.
#[allow(dead_code)]
const FREEDESKTOP: Convention = Convention {
    env_override: Some(("XDG_DATA_HOME", "sergeant")),
    home_suffix: ".local/share/sergeant",
};

#[allow(dead_code)]
const MACOS: Convention = Convention {
    env_override: None,
    home_suffix: "Library/Application Support/sergeant",
};

#[cfg(target_os = "macos")]
const CURRENT: &Convention = &MACOS;
#[cfg(not(target_os = "macos"))]
const CURRENT: &Convention = &FREEDESKTOP;

/// Resolve one convention's fallback tail, reading env vars through `probe`
/// (production passes [`std::env::var_os`] itself — same shape as
/// `TtyWatch::watching`'s injected probe in `src/tui.rs`). A test can pass a
/// canned lookup to exercise either convention, on any host, including the
/// `HOME`-unset failure this ladder's last rung has always had to name.
fn resolve(
    convention: &Convention,
    probe: impl Fn(&str) -> Option<OsString>,
) -> Result<PathBuf, String> {
    if let Some((var, suffix)) = convention.env_override
        && let Some(value) = probe(var)
    {
        return Ok(PathBuf::from(value).join(suffix));
    }
    match probe("HOME") {
        Some(home) => Ok(PathBuf::from(home).join(convention.home_suffix)),
        None => Err("cannot resolve data dir: set --data-dir, SGT_DATA_DIR, or HOME".to_string()),
    }
}

/// The fallback tail for whatever platform this binary is built for.
///
/// **The macOS arm is verified 2026-08-15** on a real macOS host (Apple M3
/// Pro, macOS 26.6.1, `docs/environments/macbook.md`) — closes #82.
/// `tests/m2_daemon_api.rs`'s and `tests/m8_estate_cli.rs`'s data-dir
/// fallback integration tests both exercised the real `sgt` binary's
/// `~/Library/Application Support/sergeant` path end to end; both tests had
/// only ever asserted the freedesktop path and needed a platform-conditional
/// fix to actually exercise this arm rather than fail on it.
pub fn fallback_dir(probe: impl Fn(&str) -> Option<OsString>) -> Result<PathBuf, String> {
    resolve(CURRENT, probe)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<OsString> {
        move |name| {
            pairs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| OsString::from(*v))
        }
    }

    #[test]
    fn freedesktop_prefers_xdg_data_home() {
        let probe = env(&[("XDG_DATA_HOME", "/xdg"), ("HOME", "/home/x")]);
        assert_eq!(
            resolve(&FREEDESKTOP, probe).unwrap(),
            PathBuf::from("/xdg/sergeant")
        );
    }

    #[test]
    fn freedesktop_falls_back_to_home_local_share() {
        let probe = env(&[("HOME", "/home/x")]);
        assert_eq!(
            resolve(&FREEDESKTOP, probe).unwrap(),
            PathBuf::from("/home/x/.local/share/sergeant")
        );
    }

    #[test]
    fn freedesktop_with_neither_var_errors() {
        assert!(resolve(&FREEDESKTOP, env(&[])).is_err());
    }

    /// The macOS convention's own decision logic — exercised here, without a
    /// macOS host, per ADR 0002 D3. Reverting `MACOS.home_suffix` to the
    /// freedesktop path fails this immediately.
    #[test]
    fn macos_ignores_xdg_data_home_and_uses_application_support() {
        let probe = env(&[("XDG_DATA_HOME", "/xdg"), ("HOME", "/Users/x")]);
        assert_eq!(
            resolve(&MACOS, probe).unwrap(),
            PathBuf::from("/Users/x/Library/Application Support/sergeant")
        );
    }
}
