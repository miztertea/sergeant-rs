//! The installed Git CLI, driven as a subprocess (proposal §11).
//!
//! §11 is explicit: shell out to the installed Git rather than embedding
//! libgit2 — "Git already defines the behavior we want; Depot's
//! responsibility is deterministic orchestration around it". Everything the
//! surface and workspace layers need from Git goes through this one module so
//! there is exactly one place that knows how a Git invocation is spelled,
//! sandboxed, and read.
//!
//! Invocations are hermetic on purpose: no pager, no terminal prompting, no
//! interactive editor. A daemon has no TTY, and a Git subcommand that stops to
//! ask a question would hang a request forever.

use std::path::Path;
use std::process::{Command, Stdio};

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

/// One hermetic Git invocation: no pager, no prompts, no editor, stdin closed.
fn command(dir: &Path, args: &[&str]) -> Command {
    let mut command = Command::new("git");
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
