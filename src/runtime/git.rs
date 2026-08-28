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
    /// A supervised invocation ([`git_fetch_restricted`]) exceeded its
    /// deadline; its process group was killed and reaped (#310 discipline,
    /// S4 Y5 G2's amendment: a remote is attacker-influenced input).
    #[error("git {args:?} in {dir} exceeded its {deadline_secs}s deadline and was killed")]
    TimedOut {
        /// Arguments of the timed-out invocation.
        args: Vec<String>,
        /// Working directory of the timed-out invocation.
        dir: String,
        /// The deadline that was exceeded, in seconds.
        deadline_secs: u64,
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

/// Run `git <args>` in `dir` and return the raw [`std::process::Output`], or
/// a [`GitError`] built from Git's own diagnostic on a nonzero exit.
///
/// The single spawn/status-check/error-construction block [`git`],
/// [`git_verbatim`], and [`git_bytes`] each need — factored out so those
/// three differ only in how they convert a successful `output.stdout`, not
/// in how they spawn or report failure.
fn run(dir: &Path, args: &[&str]) -> Result<std::process::Output, GitError> {
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
    Ok(output)
}

/// Run `git <args>` in `dir` and return trimmed stdout, or a [`GitError`].
pub fn git(dir: &Path, args: &[&str]) -> Result<String, GitError> {
    let output = run(dir, args)?;
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
    let output = run(dir, args)?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end_matches('\n')
        .to_string())
}

/// [`git`] without any trimming or UTF-8 conversion: stdout exactly as Git
/// wrote it, byte for byte.
///
/// Use this — never [`git`] or [`git_verbatim`], both of which trim a
/// trailing newline and lossily re-encode non-UTF-8 bytes — whenever the
/// output's trailing bytes are structurally significant, such as a patch
/// something downstream will hand to `git apply`: `git diff` always
/// terminates its output with `\n`, and dropping that byte turns a valid
/// patch into one `git apply` rejects as corrupt at end-of-file (#234).
pub fn git_bytes(dir: &Path, args: &[&str]) -> Result<Vec<u8>, GitError> {
    Ok(run(dir, args)?.stdout)
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

/// `git init --bare` at `dest_path` — the empty, no-working-tree object store
/// [`git_fetch_restricted`] then fills. Idempotent in the sense every caller
/// needs: `git init` on an already-initialized bare directory is a no-op
/// success, never an error, so a host cache directory that already exists
/// from a previous acquisition is reused rather than refused.
pub fn git_init_bare(dest_path: &Path) -> Result<(), GitError> {
    // `Command::current_dir` (this module's own `command()` builder) fails
    // to spawn at all against a directory that does not exist yet — unlike
    // `git clone <dest>`, `git init` does not create a missing leaf
    // directory when run *inside* it rather than given it as an argument.
    // The one caller of this function is a fresh host-cache directory that
    // may not exist yet on a source's first acquisition, so this creates it
    // first; a directory that already exists (a refresh) is left untouched.
    std::fs::create_dir_all(dest_path).map_err(|source| GitError::Spawn {
        args: vec!["init".to_string(), "--bare".to_string()],
        dir: dest_path.display().to_string(),
        source,
    })?;
    git(dest_path, &["init", "--bare", "--initial-branch=main"]).map(|_| ())
}

/// `git fetch` a **locator this codebase does not otherwise trust** into an
/// existing bare repository, restricted to exactly `allow_protocol` — S4 Y5's
/// second control, beside the string-level allowlist
/// ([`crate::runtime::atlas::locator`]) that must already have accepted
/// `locator` before this is ever called.
///
/// **`GIT_ALLOW_PROTOCOL` overrides configuration, not merely defaults it**
/// (`git-config`'s own words for the variable: it "overrid[es] any existing
/// configuration" for exactly the protocols named). That is the property this
/// function leans on: even an operator's own `~/.gitconfig` `insteadOf`
/// rewrite that retargets `locator` to an `ext::` or `file://` address
/// underneath this call is refused by Git itself, not merely by the string
/// this codebase already validated. `allow_protocol` is a caller-supplied
/// colon-separated allowlist rather than a constant here so this plumbing
/// stays independently testable (a test fixture is a `file://` repository,
/// which the real caller's allowlist never permits); the one production
/// caller — [`crate::runtime::atlas::external_git`] — always passes exactly
/// `"https:ssh"`, matching [`crate::runtime::atlas::locator`]'s own
/// allowlist and no wider (deliberately **narrower** than [`git_clone`]'s
/// own default, which includes `file` and `git` for an estate's own trusted
/// `[[repo]] --origin` — an external-git intelligence source is
/// attacker-influenced input, S4 Y5 G2's amendment, and gets no such
/// allowance).
///
/// `--depth 1`: only the tip of `refspec` is fetched. Nothing downstream of
/// this call ([`crate::runtime::atlas::git::list_tree`]/`extract_blobs`)
/// reads history, so a shallow fetch is both cheaper and a real bound on
/// acquisition cost — provisional in the same sense every unmeasured ceiling
/// this sprint ships is provisional (#325's precedent): a real external
/// repository corpus may argue for a different number later.
///
/// `refspec:refs/heads/_external_fetch_` names one fixed local ref so a
/// refetch always lands somewhere `git rev-parse` can find regardless of
/// what `refspec` names on the far end (a branch, a tag, or `HEAD`) — the
/// caller resolves the exact commit from that local ref afterward, which is
/// where "exact-commit resolution" (A1 §9) actually happens, not here.
///
/// **Supervised like a parse worker** (S4 Y5 G2's amendment: "a remote is
/// attacker-influenced input"): own process group plus `PR_SET_PDEATHSIG`
/// (`child::harden_probe_child`, #310's exact mechanism), and `deadline`
/// bounds how long the fetch may run before its whole process group is
/// killed and reaped — the identical kill-the-group-then-reap discipline
/// [`crate::runtime::atlas::worker::run_worker`] applies to a parse worker,
/// applied here to the one other subprocess this wave feeds
/// attacker-influenced bytes to. No address-space cap: `git fetch` is the
/// trusted, memory-safe binary this whole codebase already shells out to
/// (proposal §11) rather than a generated grammar parsing untrusted bytes
/// in-process, which is the risk class the parse-worker memory cap exists
/// for (S4 Y1 G2) — the deadline alone is this call's hang guard, matching
/// what #310 itself requires and no more.
pub fn git_fetch_restricted(
    dest_path: &Path,
    locator: &str,
    refspec: &str,
    allow_protocol: &str,
    deadline: std::time::Duration,
) -> Result<String, GitError> {
    let local_ref = "refs/heads/_external_fetch_";
    let args_owned = [
        "fetch".to_string(),
        "--depth".to_string(),
        "1".to_string(),
        "--no-tags".to_string(),
        "--".to_string(),
        locator.to_string(),
        format!("{refspec}:{local_ref}"),
    ];
    let args_ref: Vec<&str> = args_owned.iter().map(String::as_str).collect();
    run_supervised(dest_path, &args_ref, allow_protocol, deadline)?;
    git(dest_path, &["rev-parse", "--verify", local_ref])
}

/// One supervised Git invocation: hardened (own process group,
/// `PR_SET_PDEATHSIG`), `GIT_ALLOW_PROTOCOL` set to `allow_protocol`, killed
/// and reaped if it outlives `deadline`. The whole of [`git_fetch_restricted`]'s
/// supervision, factored out so the fetch call above stays about *what* it
/// runs rather than *how* it is supervised.
fn run_supervised(
    dir: &Path,
    args: &[&str],
    allow_protocol: &str,
    deadline: std::time::Duration,
) -> Result<std::process::Output, GitError> {
    let mut cmd = command(dir, args);
    cmd.env("GIT_ALLOW_PROTOCOL", allow_protocol)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    crate::backend::child::harden_probe_child(&mut cmd);
    let mut process = cmd.spawn().map_err(|source| GitError::Spawn {
        args: owned(args),
        dir: dir.display().to_string(),
        source,
    })?;
    let pgid = process.id();
    let registration = crate::backend::child::register_probe_child(pgid);

    // Drained concurrently on their own threads, exactly as
    // `worker::spawn_and_collect` drains a parse worker's stdout: a fetch
    // whose stderr exceeds the pipe buffer before it exits (an unexpectedly
    // chatty remote, an askpass retry loop) must not deadlock this poll loop
    // by blocking the child on a write nobody is reading.
    use std::io::Read;
    let (stdout_tx, stdout_rx) = std::sync::mpsc::sync_channel::<Vec<u8>>(1);
    if let Some(mut out) = process.stdout.take() {
        std::thread::spawn(move || {
            let mut buffer = Vec::new();
            let _ = out.read_to_end(&mut buffer);
            let _ = stdout_tx.send(buffer);
        });
    } else {
        let _ = stdout_tx.send(Vec::new());
    }
    let (stderr_tx, stderr_rx) = std::sync::mpsc::sync_channel::<Vec<u8>>(1);
    if let Some(mut err) = process.stderr.take() {
        std::thread::spawn(move || {
            let mut buffer = Vec::new();
            let _ = err.read_to_end(&mut buffer);
            let _ = stderr_tx.send(buffer);
        });
    } else {
        let _ = stderr_tx.send(Vec::new());
    }

    let deadline_at = std::time::Instant::now() + deadline;
    let status = loop {
        match process.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if std::time::Instant::now() >= deadline_at {
                    break None;
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            Err(_) => break None,
        }
    };

    let Some(status) = status else {
        crate::backend::child::kill_process_group(Some(pgid));
        let _ = process.kill();
        let _ = process.wait();
        drop(registration);
        return Err(GitError::TimedOut {
            args: owned(args),
            dir: dir.display().to_string(),
            deadline_secs: deadline.as_secs(),
        });
    };
    // Kill the group on every exit path, not only the timeout one (#310): a
    // fetch that forked a helper (an askpass prompt, a credential helper)
    // before exiting leaves that helper in the same pgid otherwise.
    crate::backend::child::kill_process_group(Some(pgid));
    drop(registration);

    let grace = std::time::Duration::from_secs(5);
    let stdout = stdout_rx.recv_timeout(grace).unwrap_or_default();
    let stderr = stderr_rx.recv_timeout(grace).unwrap_or_default();
    if !status.success() {
        return Err(GitError::Failed {
            args: owned(args),
            dir: dir.display().to_string(),
            status: status.to_string(),
            stderr: String::from_utf8_lossy(&stderr).trim().to_string(),
        });
    }
    Ok(std::process::Output {
        status,
        stdout,
        stderr,
    })
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

/// One object `git cat-file --batch` answered with.
///
/// `oid`/`kind` are Git's own header words, kept rather than re-derived: the
/// header is what proves the bytes below it are the object that was asked
/// for, and a caller keying on the OID (A1's F7 estate-git rule) wants the
/// value Git echoed, not the one it sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatFileObject {
    /// The object's full OID, as Git echoed it.
    pub oid: String,
    /// `blob`, `tree`, `commit` or `tag`.
    pub kind: String,
    /// The object's contents, byte for byte.
    pub bytes: Vec<u8>,
}

/// Read many objects from `dir`'s object store in **one** Git process
/// (`cat-file --batch`), answering positionally: element `i` is `oids[i]`'s
/// object, or `None` when Git reported it missing.
///
/// **Why a batch primitive exists at all (R6/R7).** The obvious spelling —
/// `git cat-file blob <oid>` per file — is one process, one fork, one object
/// database open and one pack index load *per file*. On a repository of a few
/// thousand blobs that is the whole cost of the scan, and it is spent on
/// process startup rather than on reading anything. `--batch` is Git's own
/// answer to exactly this shape: one process, one object database, `n`
/// answers. R2–R5 supply no in-tree batching helper, and R6's one-liner
/// ([`git_bytes`] in a loop) is precisely the thing being replaced.
///
/// **Read-only, and only ever the object store.** `cat-file` reads objects; it
/// does not consult, touch or need a working tree, and nothing here fetches,
/// pulls, switches or writes. That is what lets a caller pin a SHA and keep
/// reading it while the mount's HEAD moves underneath (§8.2's pin, made
/// durable for reads).
///
/// **Deadlock, avoided rather than hoped against.** Git streams answers as it
/// reads requests, so a caller that writes every OID before reading a byte
/// can fill the stdout pipe buffer and block Git forever while Git blocks on
/// the caller. The request is therefore written from its own thread while this
/// one drains stdout. Callers bound each batch's cumulative object size; this
/// function does not, because it cannot know a sensible ceiling for a caller
/// it has never met.
pub fn git_cat_file_batch(
    dir: &Path,
    oids: &[String],
) -> Result<Vec<Option<CatFileObject>>, GitError> {
    if oids.is_empty() {
        return Ok(Vec::new());
    }
    let args = ["cat-file", "--batch"];
    let mut child = command(dir, &args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| GitError::Spawn {
            args: owned(&args),
            dir: dir.display().to_string(),
            source,
        })?;
    let mut request = String::with_capacity(oids.len() * 41);
    for oid in oids {
        request.push_str(oid);
        request.push('\n');
    }
    let mut stdin = child.stdin.take().expect("stdin was piped");
    let writer = std::thread::spawn(move || {
        use std::io::Write;
        // The write's own error is deliberately dropped: a Git that died
        // early makes this a broken pipe, and the *useful* diagnostic is the
        // exit status and stderr collected below, not `EPIPE`.
        let _ = stdin
            .write_all(request.as_bytes())
            .and_then(|()| stdin.flush());
    });
    let output = child.wait_with_output().map_err(|source| GitError::Spawn {
        args: owned(&args),
        dir: dir.display().to_string(),
        source,
    })?;
    let _ = writer.join();
    if !output.status.success() {
        return Err(GitError::Failed {
            args: owned(&args),
            dir: dir.display().to_string(),
            status: output.status.to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    parse_cat_file_batch(&output.stdout, oids.len(), dir, &args)
}

/// Split `cat-file --batch`'s stream into one answer per request.
///
/// The wire shape, from `git cat-file`'s own documentation: an existing object
/// is `<oid> SP <type> SP <size> LF <contents> LF`, and a missing one is
/// `<name> SP missing LF`. Sizes are taken from the header rather than by
/// scanning for a terminator, because blob contents may contain any byte
/// including `LF` — a line-oriented parse of this stream is wrong on the first
/// file that ends without a newline.
fn parse_cat_file_batch(
    stdout: &[u8],
    expected: usize,
    dir: &Path,
    args: &[&str],
) -> Result<Vec<Option<CatFileObject>>, GitError> {
    let malformed = |detail: String| GitError::Failed {
        args: owned(args),
        dir: dir.display().to_string(),
        status: "exit status: 0".to_string(),
        stderr: detail,
    };
    let mut out = Vec::with_capacity(expected);
    let mut at = 0usize;
    while out.len() < expected {
        let Some(offset) = stdout[at..].iter().position(|b| *b == b'\n') else {
            return Err(malformed(format!(
                "cat-file --batch answered {} of {expected} objects before its \
                 stream ended",
                out.len()
            )));
        };
        let header = String::from_utf8_lossy(&stdout[at..at + offset]).into_owned();
        at += offset + 1;
        let mut words = header.split(' ');
        let oid = words.next().unwrap_or_default().to_string();
        let kind = words.next().unwrap_or_default().to_string();
        if kind == "missing" || kind.is_empty() {
            out.push(None);
            continue;
        }
        let size: usize = words.next().and_then(|s| s.parse().ok()).ok_or_else(|| {
            malformed(format!("cat-file --batch header is unreadable: {header:?}"))
        })?;
        if at + size > stdout.len() {
            return Err(malformed(format!(
                "cat-file --batch promised {size} bytes for {oid} and delivered {}",
                stdout.len().saturating_sub(at)
            )));
        }
        let bytes = stdout[at..at + size].to_vec();
        // The record's own trailing LF, which is framing and not content.
        at += size + 1;
        out.push(Some(CatFileObject { oid, kind, bytes }));
    }
    Ok(out)
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

    /// The framing is size-prefixed, not line-oriented: a blob with no
    /// trailing newline, a blob that is entirely newlines, an empty blob and a
    /// missing object all have to survive one stream together.
    #[test]
    fn a_batch_stream_is_split_by_its_declared_sizes() {
        let mut stream = Vec::new();
        stream.extend_from_slice(b"aaa blob 5\nno-nl\n");
        stream.extend_from_slice(b"bbb blob 3\n\n\n\n\n");
        stream.extend_from_slice(b"ccc blob 0\n\n");
        stream.extend_from_slice(b"deadbeef missing\n");
        let parsed =
            parse_cat_file_batch(&stream, 4, Path::new("/tmp"), &["cat-file"]).expect("parse");
        assert_eq!(parsed.len(), 4);
        assert_eq!(parsed[0].as_ref().expect("present").bytes, b"no-nl");
        assert_eq!(parsed[0].as_ref().expect("present").oid, "aaa");
        assert_eq!(parsed[1].as_ref().expect("present").bytes, b"\n\n\n");
        assert!(parsed[2].as_ref().expect("present").bytes.is_empty());
        assert!(
            parsed[3].is_none(),
            "a missing object is None, not an error"
        );
    }

    /// A truncated stream is a named failure, never a short answer silently
    /// treated as "these objects do not exist".
    #[test]
    fn a_truncated_batch_stream_is_an_error_not_a_short_answer() {
        let err = parse_cat_file_batch(b"aaa blob 99\nshort", 1, Path::new("/tmp"), &["cat-file"])
            .expect_err("truncated");
        assert!(err.to_string().contains("promised 99 bytes"), "{err}");
        let err = parse_cat_file_batch(b"", 2, Path::new("/tmp"), &["cat-file"])
            .expect_err("empty stream");
        assert!(err.to_string().contains("0 of 2 objects"), "{err}");
    }

    /// End to end against real Git: one process answers many objects, in
    /// request order, and never needs a working tree to do it.
    #[test]
    fn one_process_reads_many_blobs_in_request_order() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        git(root, &["init", "--initial-branch=main"]).expect("init");
        git(root, &["config", "user.email", "t@example.com"]).expect("email");
        git(root, &["config", "user.name", "T"]).expect("name");
        let mut oids = Vec::new();
        for (name, body) in [("a.txt", "alpha"), ("b.txt", "beta\n\nmore"), ("c.txt", "")] {
            std::fs::write(root.join(name), body).expect("write");
            let _ = name;
            oids.push(
                git(
                    root,
                    &["hash-object", "-w", root.join(name).to_str().expect("utf8")],
                )
                .expect("hash-object"),
            );
        }
        oids.push("0".repeat(40));
        let objects = git_cat_file_batch(root, &oids).expect("batch");
        assert_eq!(objects.len(), 4);
        assert_eq!(objects[0].as_ref().expect("a").bytes, b"alpha");
        assert_eq!(objects[1].as_ref().expect("b").bytes, b"beta\n\nmore");
        assert!(objects[2].as_ref().expect("c").bytes.is_empty());
        assert!(objects[3].is_none());
        assert!(git_cat_file_batch(root, &[]).expect("empty").is_empty());
    }

    /// A bare cache initialized, fetched shallow from a `file://` source
    /// (protocol widened for this test only — see [`git_fetch_restricted`]'s
    /// own doc for why `allow_protocol` is a parameter), lands its tip at the
    /// fixed local ref and resolves.
    #[test]
    fn a_bare_cache_fetches_shallow_and_resolves_the_fixed_local_ref() {
        let origin = tempfile::TempDir::new().expect("origin dir");
        git(origin.path(), &["init", "--initial-branch=main"]).expect("init");
        git(origin.path(), &["config", "user.email", "t@example.com"]).expect("email");
        git(origin.path(), &["config", "user.name", "T"]).expect("name");
        std::fs::write(origin.path().join("a.txt"), "one").expect("write");
        git(origin.path(), &["add", "-A"]).expect("add");
        git(origin.path(), &["commit", "-m", "one"]).expect("commit");
        let tip = git(origin.path(), &["rev-parse", "HEAD"]).expect("rev-parse");

        let cache = tempfile::TempDir::new().expect("cache dir");
        git_init_bare(cache.path()).expect("init bare");
        // Idempotent: a second init on the same directory is not an error.
        git_init_bare(cache.path()).expect("init bare again");

        let origin_url = format!("file://{}", origin.path().display());
        let resolved = git_fetch_restricted(
            cache.path(),
            &origin_url,
            "main",
            "file",
            std::time::Duration::from_secs(10),
        )
        .expect("fetch restricted");
        assert_eq!(
            resolved, tip,
            "the fetched ref resolves to the origin's tip"
        );

        // Bare: no working tree, no index.
        assert!(
            !cache.path().join(".git").exists(),
            "the cache dir IS the git dir (bare)"
        );
        assert!(
            !cache.path().join("a.txt").exists(),
            "a bare fetch never materializes a working tree"
        );
    }

    /// `GIT_ALLOW_PROTOCOL` is the second control (module doc): a `file://`
    /// source is refused when the caller's own allowlist does not name
    /// `file`, exactly the restriction [`crate::runtime::atlas::external_git`]
    /// relies on by always passing `"https:ssh"`.
    #[test]
    fn a_protocol_not_in_the_allowlist_is_refused_by_git_itself() {
        let origin = tempfile::TempDir::new().expect("origin dir");
        git(origin.path(), &["init", "--initial-branch=main"]).expect("init");
        git(origin.path(), &["config", "user.email", "t@example.com"]).expect("email");
        git(origin.path(), &["config", "user.name", "T"]).expect("name");
        std::fs::write(origin.path().join("a.txt"), "one").expect("write");
        git(origin.path(), &["add", "-A"]).expect("add");
        git(origin.path(), &["commit", "-m", "one"]).expect("commit");

        let cache = tempfile::TempDir::new().expect("cache dir");
        git_init_bare(cache.path()).expect("init bare");
        let origin_url = format!("file://{}", origin.path().display());
        let err = git_fetch_restricted(
            cache.path(),
            &origin_url,
            "main",
            "https:ssh",
            std::time::Duration::from_secs(10),
        )
        .expect_err("file must be refused when not in the allowlist");
        assert!(matches!(err, GitError::Failed { .. }), "{err}");
    }

    /// A fetch that outlives its deadline is killed and reaped, not left to
    /// hang — the timeout half of [`run_supervised`]'s discipline. A `git
    /// fetch` that never gets a byte back (an origin that accepts the TCP
    /// connection but never speaks the protocol) is the shape a bounded
    /// deadline exists for.
    #[test]
    fn a_fetch_past_its_deadline_is_killed_and_reported_as_timed_out() {
        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").expect("bind a port that never answers");
        let port = listener.local_addr().expect("addr").port();
        // Accept and hold the connection open without ever writing a byte —
        // enough to make `git fetch` block indefinitely waiting on the
        // remote's first protocol line.
        std::thread::spawn(move || {
            // Held, not dropped: dropping the accepted stream immediately
            // closes the connection, which `git` reads as EOF and fails
            // fast on — the opposite of the stall this test needs.
            if let Ok((stream, _addr)) = listener.accept() {
                std::thread::sleep(std::time::Duration::from_secs(30));
                drop(stream);
            }
        });

        let cache = tempfile::TempDir::new().expect("cache dir");
        git_init_bare(cache.path()).expect("init bare");
        let stalled_url = format!("git://127.0.0.1:{port}/repo.git");
        let started = std::time::Instant::now();
        let err = git_fetch_restricted(
            cache.path(),
            &stalled_url,
            "main",
            "git",
            std::time::Duration::from_millis(500),
        )
        .expect_err("a stalled remote must time out, not hang");
        assert!(matches!(err, GitError::TimedOut { .. }), "{err}");
        assert!(
            started.elapsed() < std::time::Duration::from_secs(10),
            "the deadline, not the test harness, must be what ends this: took {:?}",
            started.elapsed()
        );
    }

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
