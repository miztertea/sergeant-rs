//! The installed Git CLI, driven as a subprocess (proposal §11).
//!
//! §11 is explicit: shell out to the installed Git rather than embedding
//! libgit2 — "Git already defines the behavior we want; Depot's
//! responsibility is deterministic orchestration around it". Everything the
//! surface and estate layers need from Git goes through this one module so
//! there is exactly one place that knows how a Git invocation is spelled,
//! sandboxed, and read.
//!
//! Invocations are hermetic on purpose: no pager, no terminal prompting, no
//! interactive editor. A daemon has no TTY, and a Git subcommand that stops to
//! ask a question would hang a request forever.

use std::path::Path;
use std::process::{Command, Stdio};

/// Overrides the `git` executable [`command`] invokes, mirroring
/// `backend::docker::DOCKER_BIN_ENV` (C11: Slice 1's "scripted Git binary"
/// admission tests had no infrastructure to point at). Without this, every
/// check this module's callers need to prove — no network, no branch
/// switch, no fetch/pull/reset on any admission path — could only be
/// asserted against whatever real `git` happens to be on `PATH`, the one
/// thing a test can neither control nor safely fake.
///
/// Read fresh from the environment inside [`command`] on every invocation
/// rather than cached in a config struct or a `OnceLock`: a cached value (or
/// a `OnceLock`, which would stay poisoned for the rest of a parallel test
/// run) would let one test's override leak into, or block, another's. Tests
/// point it at a scripted binary to observe — or deny — git invocations
/// without touching `PATH`.
///
/// Setting it is nonetheless a *process-global* act, not a thread-scoped
/// one, so a test that sets it must own its whole process rather than share
/// the lib test binary with the many call sites that reach real Git through
/// this module. `tests/c11_injectable_git.rs` is that process, and carries
/// the reasoning.
pub const GIT_BIN_ENV: &str = "SGT_GIT_BIN";

/// Failure running the Git CLI.
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    /// The `git` binary could not be executed at all (not installed, not on
    /// `PATH`, no permission).
    #[error("cannot run git {args:?} in {dir}: {source}")]
    Spawn {
        /// Arguments of the attempted invocation.
        args: Vec<String>,
        /// Working directory of the attempted invocation.
        dir: String,
        /// Underlying spawn failure.
        source: std::io::Error,
    },
    /// Git ran and refused. The exit status and stderr are carried verbatim:
    /// Git's own diagnostic is the most useful evidence we can record.
    #[error("git {args:?} failed in {dir} ({status}): {stderr}")]
    Failed {
        /// Arguments of the failed invocation.
        args: Vec<String>,
        /// Working directory of the failed invocation.
        dir: String,
        /// Exit status text (`exit status: 128`, or a signal).
        status: String,
        /// Trimmed stderr from Git.
        stderr: String,
    },
}

/// `git rev-parse --path-format=absolute --git-common-dir` in `dir`,
/// canonicalized: the identity §2.7/§9.4 locks on.
///
/// The *common* directory, not the git dir: a linked worktree's own
/// `.git/worktrees/<name>` is private to it, while the common dir is the
/// shared object store, the ref storage, and the linked-worktree registry —
/// the state two concurrent mutators actually collide in. A primary checkout
/// and every `git worktree add` of it answer with one and the same path,
/// which is precisely what a lock keyed on the checkout *path* could never
/// see (proposal §2.7: "different linked worktree paths may share one Git
/// common directory and therefore one ref/worktree registry while receiving
/// different locks").
///
/// Canonicalized because this is about identity, not spelling: a repository
/// reached through a symlink (`/tmp` → `/private/tmp` on macOS, an estate
/// mount behind a symlinked parent) would otherwise answer differently from
/// the same repository reached directly, and hand two names to one thing.
/// When canonicalization fails — a path that raced away underneath us — the
/// absolute answer Git gave is kept as-is rather than discarded: a slightly
/// less normalized identity still serializes correctly against every other
/// caller that resolves it the same way, and refusing outright would fail an
/// operation over a spelling.
pub fn canonical_git_common_dir(dir: &Path) -> Result<std::path::PathBuf, GitError> {
    let raw = git(
        dir,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?;
    let path = std::path::PathBuf::from(raw);
    Ok(std::fs::canonicalize(&path).unwrap_or(path))
}

/// `git rev-parse --path-format=absolute --show-toplevel` in `dir`,
/// canonicalized the same way [`canonical_git_common_dir`] is.
///
/// §3.5's "canonical Git top level" half of a repository binding: what the
/// declared mount path actually resolves to as a working tree, recorded at
/// admission so later readers compare against what was admitted rather than
/// re-asking a checkout that may have moved.
pub fn canonical_git_top_level(dir: &Path) -> Result<std::path::PathBuf, GitError> {
    let raw = git(
        dir,
        &["rev-parse", "--path-format=absolute", "--show-toplevel"],
    )?;
    let path = std::path::PathBuf::from(raw);
    Ok(std::fs::canonicalize(&path).unwrap_or(path))
}

/// Run `git <args>` in `dir` and return trimmed stdout, or a [`GitError`].
pub fn git(dir: &Path, args: &[&str]) -> Result<String, GitError> {
    let output = command(dir, args)
        .output()
        .map_err(|source| GitError::Spawn {
            args: owned(args),
            dir: dir.display().to_string(),
            source,
        })?;
    if !output.status.success() {
        return Err(GitError::Failed {
            args: owned(args),
            dir: dir.display().to_string(),
            status: output.status.to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// [`git`] without the trim: stdout exactly as Git wrote it.
///
/// Necessary for `status --porcelain`, whose leading whitespace *is* the
/// answer — ` M file` (modified, unstaged) and `M  file` (modified, staged)
/// differ only there, and trimming turns the first into the second. §8.3
/// requires an override to "record full porcelain evidence"; evidence with
/// its first status column removed is not that.
///
/// Only the trailing newline Git terminates its last record with is dropped,
/// so a clean tree still answers with an empty string.
pub fn git_verbatim(dir: &Path, args: &[&str]) -> Result<String, GitError> {
    let output = command(dir, args)
        .output()
        .map_err(|source| GitError::Spawn {
            args: owned(args),
            dir: dir.display().to_string(),
            source,
        })?;
    if !output.status.success() {
        return Err(GitError::Failed {
            args: owned(args),
            dir: dir.display().to_string(),
            status: output.status.to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end_matches('\n')
        .to_string())
}

/// Whether `git <args>` in `dir` exits zero. Used for existence questions
/// (`is this a work tree?`, `does this branch exist?`) where the failing case
/// is an answer, not an error.
pub fn git_succeeds(dir: &Path, args: &[&str]) -> bool {
    matches!(command(dir, args).output(), Ok(output) if output.status.success())
}

/// `git clone` where `origin` is untrusted, human-supplied input (`sgt repo
/// add --origin <url>`, not a value this codebase generated). `--` stops an
/// origin starting with `-` from being read as a clone flag, and
/// `GIT_ALLOW_PROTOCOL` locks the transport to the ones a repository origin
/// should ever need — closing off `ext::`/`fd::`-style transport helpers,
/// classic clone-URL command execution.
pub fn git_clone(dir: &Path, origin: &str, dest_path: &str) -> Result<String, GitError> {
    let args = ["clone", "--", origin, dest_path];
    let output = command(dir, &args)
        .env("GIT_ALLOW_PROTOCOL", "file:http:https:ssh:git")
        .output()
        .map_err(|source| GitError::Spawn {
            args: owned(&args),
            dir: dir.display().to_string(),
            source,
        })?;
    if !output.status.success() {
        return Err(GitError::Failed {
            args: owned(&args),
            dir: dir.display().to_string(),
            status: output.status.to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// `git submodule update --init --recursive` in a freshly materialized
/// worktree (#22: `git worktree add` never populates submodules — the
/// superproject's gitlinks are checked out, the submodule directories are
/// not — so a work surface over a repository with submodules silently
/// carried empty directories where the rest of the codebase expected
/// checked-out content, with nothing about materialization saying so).
///
/// A submodule URL is exactly as untrusted as a `sgt repo add --origin` one
/// — declared inside a repository the estate owner may not fully control —
/// so it gets the identical transport allowlist [`git_clone`] already uses,
/// rather than either the compiled-in default (which refuses `file:` for
/// submodule recursion specifically, breaking every sibling-path submodule
/// a local or CI fixture might use) or an unbounded override (which would
/// widen what a nested submodule chain can reach).
pub fn git_submodule_update(dir: &Path) -> Result<String, GitError> {
    let args = ["submodule", "update", "--init", "--recursive"];
    let output = command(dir, &args)
        .env("GIT_ALLOW_PROTOCOL", "file:http:https:ssh:git")
        .output()
        .map_err(|source| GitError::Spawn {
            args: owned(&args),
            dir: dir.display().to_string(),
            source,
        })?;
    if !output.status.success() {
        return Err(GitError::Failed {
            args: owned(&args),
            dir: dir.display().to_string(),
            status: output.status.to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// The remote name a declared `[[repo]] upstream` is written to (#112).
///
/// Forge-neutral by construction: the URL is opaque here and everywhere
/// below, and nothing in this codebase asks a host what it thinks of it. The
/// name is what makes `gh`, `glab`, `git push -u upstream`, and a human's
/// muscle memory all resolve the same way inside a mount.
pub const UPSTREAM_REMOTE: &str = "upstream";

/// `git remote get-url <name>`, or `None` when the remote is not configured.
///
/// A read, so it is safe anywhere config reading is — which is *not* the Work
/// admission path: `tests/e_admission_uses_no_network_git.rs` forbids the
/// `remote` verb there outright, read or write (§6.4: sergeant never infers a
/// remote default for a Work). Callers are `sgt repo add` and `sgt doctor`,
/// both operator-invoked and neither on that path.
pub fn git_remote_url(dir: &Path, name: &str) -> Option<String> {
    git(dir, &["remote", "get-url", name])
        .ok()
        .filter(|url| !url.is_empty())
}

/// `git remote add <name> <url>` — a repository-local config write, no
/// network contact of any kind.
///
/// `--` for the same reason [`git_clone`] uses it: the URL is human-supplied
/// (`sgt repo add --upstream <url>`) and one starting with `-` would
/// otherwise be read as a flag. Nothing validates the URL's *shape* — #112 is
/// forge-neutral, so an ssh alias, a `file:` path and a hosted HTTPS URL are
/// all equally legitimate here.
pub fn git_remote_add(dir: &Path, name: &str, url: &str) -> Result<String, GitError> {
    git(dir, &["remote", "add", "--", name, url])
}

/// `git remote set-url <name> <url>` — [`git_remote_add`] for a remote that
/// already exists, with the identical `--` guard.
pub fn git_remote_set_url(dir: &Path, name: &str, url: &str) -> Result<String, GitError> {
    git(dir, &["remote", "set-url", "--", name, url])
}

/// One hermetic Git invocation: no pager, no prompts, no editor, stdin closed.
fn command(dir: &Path, args: &[&str]) -> Command {
    let git_bin = std::env::var(GIT_BIN_ENV).unwrap_or_else(|_| "git".to_string());
    let mut command = Command::new(git_bin);
    command
        .arg("--no-pager")
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        // A daemon has no terminal: Git must never stop to ask.
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_EDITOR", "true");
    command
}

fn owned(args: &[&str]) -> Vec<String> {
    args.iter().map(|a| (*a).to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_carries_gits_own_diagnostic() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let err = git(dir.path(), &["rev-parse", "--show-toplevel"]).expect_err("not a repo");
        let text = err.to_string();
        assert!(
            text.contains("not a git repository"),
            "git's diagnostic must survive into the error: {text}"
        );
        assert!(!git_succeeds(dir.path(), &["rev-parse", "--show-toplevel"]));
    }
}
