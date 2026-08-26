//! `sgt <harness> -- <args>` (ADR 0006, D2): `claude`, `codex`, `opencode`,
//! `goose`, `agy`. Composes an environment, binds the estate, then **exec**s —
//! replacing this process image with the harness's, not forking and
//! supervising it. That boundary is load-bearing (the workspace knowledge
//! library's North Star ruling, "Never" list: "reconstructed tmux-era
//! supervision"): once the harness starts,
//! there is no lifecycle here for `sgt` to own. Nothing in this module runs
//! after [`exec`] succeeds — a successful exec never returns to its caller
//! at all.
//!
//! What "the environment" contains (proposal §8.1, left undecided there) is
//! this module's own judgment call, not an owner ruling: PATH enrichment
//! with the per-user toolchain directories measured to be missing from a
//! non-interactive shell's default PATH (`sergeant-rs-workspace's knowledge/evidence/host-measurements/cerberus.md`:
//! `~/.cargo/bin` houses cargo/rustc and is the literal #60 failure;
//! `~/.local/bin` is where the `claude` CLI itself and `no-mistakes` are
//! installed on that same host, so a harness a user cannot even find on
//! PATH is a sharper version of the same problem), plus an explicit
//! `SGT_DATA_DIR` binding to whatever estate the launch directory resolved
//! to — reusing `cli::resolve_data_dir`'s own ladder rather than
//! re-deriving estate discovery, so the harness, the daemon it later
//! spawns, and every actor beneath that daemon all agree on one data dir
//! without re-walking `cwd` per command.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Env var this module sets to bind the harness (and everything it later
/// spawns) to one data dir, explicit at launch rather than rediscovered per
/// command (ADR 0006's estate-binding half). Same variable
/// `cli::resolve_data_dir` already treats as taking unconditional
/// precedence over estate discovery, so setting it here is consistent with
/// every later `sgt` invocation inside the harness session, not a second
/// competing mechanism.
const DATA_DIR_ENV: &str = "SGT_DATA_DIR";

/// §5.3: the canonical estate root this harness session is bound to.
///
/// Evidence and convenience, never authority: a later `sgt` invocation
/// inside the session reads it only to make a *refusal* more informative
/// (§4.4's descendant diagnostic names both roots). It can never waive the
/// exact-root check — if Captain cds into a mount and runs `sgt run`, the
/// tool still refuses and says how to return.
///
/// **Distinct from [`crate::backend::SERGEANT_ESTATE_ROOT_ENV`]** (S2 E5),
/// whose name differs only in prefix and whose meaning does not overlap at
/// all: that one is set per *managed execution*, carries the estate the
/// daemon recorded for that Work, and is validated against the journal
/// before any causal relation is recorded. This one is set once when `sgt
/// <harness>` execs a session and is read only to decorate a refusal.
/// Neither may be read in place of the other; `crate::backend`'s own unit
/// test pins that the causation triple never emits this name.
pub(crate) const ESTATE_ROOT_ENV: &str = "SGT_ESTATE_ROOT";

/// §5.3: which harness composed this environment, so a submission from
/// inside the session routes by origin affinity (§13) and journals its true
/// origin client rather than a bare `cli`.
const ORIGIN_CLIENT_ENV: &str = "SGT_ORIGIN_CLIENT";

/// Per-user toolchain directories `sgt <harness>` prepends to `PATH` before
/// exec'ing, when they exist on disk. See the module docs for the measured
/// evidence behind each one. Deliberately a short, named list rather than an
/// attempt to replicate a login shell's full startup environment — sourcing
/// `.bashrc`/`.profile` would make sergeant responsible for parsing and
/// running arbitrary shell config it does not own (proposal §8.1's
/// over-specification warning); this instead states each directory by name,
/// with its own measured evidence, and stops.
///
/// - `~/.cargo/bin`, `~/.local/bin` — measured (`sergeant-rs-workspace's knowledge/evidence/host-measurements/
///   cerberus.md`), see the module doc above.
/// - `~/.opencode/bin` — measured (`knowledge/evidence/
///   opencode-adapter-probes-2026-08-23.md`, "Installation facts"):
///   opencode's own binary lives there on Cerberus, absent from a
///   non-interactive shell's default PATH — the identical #60 failure class
///   `~/.cargo/bin`/`~/.local/bin` were added to fix.
pub fn toolchain_path_dirs(home: &Path) -> Vec<PathBuf> {
    vec![
        home.join(".cargo").join("bin"),
        home.join(".local").join("bin"),
        home.join(".opencode").join("bin"),
    ]
}

/// Which of `dirs` exist on disk but are not already present in `path`, in
/// `dirs`' own order, never repeating an entry. `exists` is injected so this
/// stays a pure function of its inputs, exercisable without a real
/// filesystem (ADR 0002 D3's precedent for platform-adjacent decision
/// logic) — the same reason it is unit-tested directly rather than only
/// through a subprocess.
///
/// Shared by [`compose_path`] (what to prepend before exec'ing) and `sgt
/// doctor`'s environment check (what to warn about) so the two can never
/// silently drift apart into checking a different list than the one that
/// actually gets composed.
pub fn dirs_missing_from_path(
    path: Option<&OsString>,
    dirs: &[PathBuf],
    exists: impl Fn(&Path) -> bool,
) -> Vec<PathBuf> {
    let existing: Vec<PathBuf> = path
        .map(|p| std::env::split_paths(p).collect())
        .unwrap_or_default();
    missing_from_existing(dirs, &existing, exists)
}

/// The part of [`dirs_missing_from_path`] that does not need to know how
/// `existing` was derived — shared with [`compose_path`], which already has
/// `existing` in hand and would otherwise make [`dirs_missing_from_path`]
/// re-split the same `path` a second time.
fn missing_from_existing(
    dirs: &[PathBuf],
    existing: &[PathBuf],
    exists: impl Fn(&Path) -> bool,
) -> Vec<PathBuf> {
    let mut missing: Vec<PathBuf> = Vec::new();
    for dir in dirs {
        if exists(dir) && !existing.contains(dir) && !missing.contains(dir) {
            missing.push(dir.clone());
        }
    }
    missing
}

/// Prepend whichever `dirs` are missing per [`dirs_missing_from_path`],
/// preserving the existing PATH's order after them.
pub fn compose_path(
    path: Option<&OsString>,
    dirs: &[PathBuf],
    exists: impl Fn(&Path) -> bool,
) -> OsString {
    let existing: Vec<PathBuf> = path
        .map(|p| std::env::split_paths(p).collect())
        .unwrap_or_default();
    let mut prefix = missing_from_existing(dirs, &existing, exists);
    prefix.extend(existing);
    std::env::join_paths(prefix).unwrap_or_else(|_| path.cloned().unwrap_or_default())
}

/// Build the [`Command`] `sgt <harness>` execs into: `binary` with `args`
/// passed straight through (§9's `sgt claude -- --model opus` reaches
/// `claude` as `--model opus`, unexamined and unmodified), PATH enriched per
/// [`toolchain_path_dirs`] when `$HOME` resolves, and §5.3's three bindings:
/// `SGT_ESTATE_ROOT` (the canonical root this invocation already admitted —
/// §4.3 puts that check *before* harness preparation), `SGT_DATA_DIR` (the
/// same value `cli::resolve_data_dir` computed for this invocation, so the
/// binding agrees with whatever ladder rung decided it), and
/// `SGT_ORIGIN_CLIENT` (the harness's own name).
///
/// The harness also starts *in* the estate root, which is what makes the
/// ordinary `sgt run` inside the session work without a `-C`. That is
/// convenience layered on top of the binding, not a substitute for it: the
/// exact-root check still runs on every command, and the environment never
/// waives it (§5.3).
pub fn prepare(binary: &str, args: &[String], estate_root: &Path, data_dir: &Path) -> Command {
    let mut command = Command::new(binary);
    command.args(args);
    if let Some(home) = std::env::var_os("HOME") {
        let dirs = toolchain_path_dirs(Path::new(&home));
        let path = compose_path(std::env::var_os("PATH").as_ref(), &dirs, |dir| dir.exists());
        command.env("PATH", path);
    }
    command.env(ESTATE_ROOT_ENV, estate_root);
    command.env(DATA_DIR_ENV, data_dir);
    command.env(ORIGIN_CLIENT_ENV, binary);
    command.current_dir(estate_root);
    command
}

/// Exec into `command` (unix `execve(2)`), replacing this process image.
/// This is the load-bearing boundary the module docs describe: only returns
/// on failure — a successful exec never returns here at all, which is
/// exactly what makes it impossible for anything in this module to run
/// "after the harness starts."
#[cfg(unix)]
pub fn exec(mut command: Command) -> std::io::Error {
    use std::os::unix::process::CommandExt;
    command.exec()
}

/// The measured target set (ADR 0001: Linux, macOS, Windows-via-WSL2) is all
/// unix, so there is no non-unix path this has ever been asked to take.
/// Rather than silently falling back to fork-and-wait — exactly the
/// supervision shape ADR 0006 forbids — this fails closed with a named
/// remedy.
#[cfg(not(unix))]
pub fn exec(_command: Command) -> std::io::Error {
    std::io::Error::other(
        "sgt <harness> replaces this process via exec(2), measured only on Linux, macOS, and \
         Windows via WSL2 (ADR 0001); there is no supervised fallback by design (ADR 0006). \
         On native Windows, run the harness directly instead of through sgt.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn always(_: &Path) -> bool {
        true
    }

    fn never(_: &Path) -> bool {
        false
    }

    #[test]
    fn missing_dirs_are_prepended_in_order() {
        let path = OsString::from("/usr/bin:/bin");
        let dirs = vec![
            PathBuf::from("/home/x/.cargo/bin"),
            PathBuf::from("/home/x/.local/bin"),
        ];
        let composed = compose_path(Some(&path), &dirs, always);
        assert_eq!(
            composed,
            OsString::from("/home/x/.cargo/bin:/home/x/.local/bin:/usr/bin:/bin")
        );
    }

    #[test]
    fn a_dir_already_on_path_is_not_duplicated() {
        let path = OsString::from("/home/x/.cargo/bin:/usr/bin");
        let dirs = vec![
            PathBuf::from("/home/x/.cargo/bin"),
            PathBuf::from("/home/x/.local/bin"),
        ];
        let composed = compose_path(Some(&path), &dirs, always);
        assert_eq!(
            composed,
            OsString::from("/home/x/.local/bin:/home/x/.cargo/bin:/usr/bin")
        );
    }

    #[test]
    fn a_dir_that_does_not_exist_on_disk_is_never_prepended() {
        let path = OsString::from("/usr/bin");
        let dirs = vec![PathBuf::from("/home/x/.cargo/bin")];
        let composed = compose_path(Some(&path), &dirs, never);
        assert_eq!(
            composed,
            OsString::from("/usr/bin"),
            "nothing to enrich when nothing exists"
        );
    }

    /// The exact list `sgt doctor`'s environment check reports on — pinned
    /// here so the check and the composition it is checking can never drift
    /// apart into disagreeing about which directories matter.
    #[test]
    fn dirs_missing_from_path_reports_only_existing_and_absent_from_path() {
        let path = OsString::from("/home/x/.cargo/bin:/usr/bin");
        let dirs = vec![
            PathBuf::from("/home/x/.cargo/bin"),
            PathBuf::from("/home/x/.local/bin"),
            PathBuf::from("/home/x/.nvm/bin"),
        ];
        let exists = |d: &Path| d != Path::new("/home/x/.nvm/bin");
        let missing = dirs_missing_from_path(Some(&path), &dirs, exists);
        assert_eq!(missing, vec![PathBuf::from("/home/x/.local/bin")]);
    }

    #[test]
    fn no_existing_path_still_composes_from_dirs_alone() {
        let dirs = vec![PathBuf::from("/home/x/.cargo/bin")];
        let composed = compose_path(None, &dirs, always);
        assert_eq!(composed, OsString::from("/home/x/.cargo/bin"));
    }

    #[test]
    fn toolchain_path_dirs_names_cargo_local_and_opencode_bin() {
        let home = Path::new("/home/x");
        assert_eq!(
            toolchain_path_dirs(home),
            vec![
                PathBuf::from("/home/x/.cargo/bin"),
                PathBuf::from("/home/x/.local/bin"),
                PathBuf::from("/home/x/.opencode/bin"),
            ]
        );
    }
}
