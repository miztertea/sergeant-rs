//! Interprocess repository lock, keyed by canonical Git common directory
//! (proposal §9.4, amendment C4).
//!
//! §9.4: *every* operation mutating a repository's linked-worktree registry or
//! its Work refs takes its lock identity from the canonical result of `git
//! rev-parse --path-format=absolute --git-common-dir`, and that lock is
//! **interprocess** — "the implementation may retain an in-process mutex for
//! efficiency, but the filesystem lock is the correctness boundary".
//!
//! # Why this is a new primitive and not `fsutil::take_exclusive_lock`
//!
//! [`crate::runtime::fsutil::take_exclusive_lock`] is a *self-leak
//! workaround*, not a blocking primitive (C4). It answers "does someone else
//! really own the daemon data dir right now", waits at most one second and
//! only when this process has held that same path before, records every path
//! it ever locked in a process-lifetime set, and is designed to fail closed
//! immediately for a rival daemon. Repository locking needs the opposite
//! shape: a contender is *expected*, must wait for the holder rather than
//! refuse, must be able to take and release the same lock many times over a
//! process's life, and must never consult a process-lifetime memo. Widening
//! `take_exclusive_lock` to cover both would have made the daemon-identity
//! refusal wait, which is the one thing it must not do.
//!
//! # Where the lock file lives, and why that is complete (§6.2)
//!
//! `<data_dir>/locks/repo/<blake3-hex of the canonical common dir>.lock`.
//!
//! **Never inside the user's `.git`.** Surface doctrine is that Sergeant does
//! not write into a source checkout: it creates linked worktrees and branches
//! through Git's own porcelain and otherwise leaves the repository alone. A
//! lock file dropped into `.git/` would be Sergeant state living in a
//! directory the user owns, surviving uninstall, and visible to every other
//! tool that walks that repository.
//!
//! That raises the obvious question: if the file is not next to the thing it
//! protects, do two mutators actually meet on it? Under §6.2 they do. "One
//! checkout belongs to one estate" — separate estates use separate clones,
//! there are no manifest aliases into another estate, no `../repo` paths, no
//! symlinked shared mounts, and a Work's linked worktree may not itself be
//! declared as a repository source. One checkout therefore has exactly one
//! estate, one estate has exactly one data dir, and every legitimate mutator
//! of that checkout resolves that same data dir. Keying the *directory* on the
//! data dir and the *file* on the Git common directory is complete for the
//! ownership model the product actually has, and it keeps Sergeant's state
//! inside Sergeant's own storage where the rest of it already is.
//!
//! The hash follows [`crate::runtime::blob`]'s existing precedent for
//! identity-keyed filenames (`<data_dir>/blobs/b3/<hex>`): a blake3 digest of
//! the canonical path bytes, so an arbitrarily deep or oddly spelled path
//! becomes one fixed-length, filesystem-safe name — and so that two spellings
//! that canonicalize to one directory cannot produce two lock files.
//!
//! # Bounded wait, never an infinite one
//!
//! Surface operations are synchronous and reach the async runtime through
//! `api::blocking`, which uses `block_in_place` on a multi-thread runtime but
//! **runs the closure inline on a current-thread one**. An unbounded
//! `File::lock()` there would park the reactor itself. So acquisition is a
//! bounded retry loop over `try_lock` with backoff and a [`LOCK_BUDGET`]
//! deadline; exhausting it returns [`RepoLockError::Timeout`], which names the
//! lock path and the repository it stands for. Failing closed with evidence is
//! an outcome an operator can act on; a hung daemon is not.
//!
//! The budget is also what absorbs the fork/exec window
//! `fsutil::take_exclusive_lock` documents: Sergeant shells out to Git while
//! holding this lock, and between a child's `fork` and its `exec` that child
//! holds a duplicate of every descriptor the parent had open. Rust opens files
//! `O_CLOEXEC`, so such a duplicate cannot outlive the `exec` — waiting is what
//! resolves it, and the retry loop waits by construction.

use std::fs::{File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::runtime::fsutil::create_dir_all_durable;

/// Directory under the data dir holding every repository lock file.
pub const LOCKS_DIR: &str = "locks";
/// Subdirectory of [`LOCKS_DIR`] for locks keyed by Git common directory.
pub const REPO_LOCKS_DIR: &str = "repo";

/// How long [`acquire`] waits for a holder before failing closed.
///
/// Generous relative to what the guarded spans actually cost — a `git worktree
/// add --no-checkout` was measured at ~43 ms of lock-held time, so this is
/// roughly 700 such operations queued ahead of us — and still finite, because
/// the caller may be the reactor thread itself. A wait this long means
/// something is genuinely wrong (a crashed holder on a filesystem that leaked
/// the lock, a foreign process sitting on the file), and saying so with the
/// path in hand beats waiting forever.
pub const LOCK_BUDGET: Duration = Duration::from_secs(30);

/// Longest pause between attempts. Backoff starts far below it, so ordinary
/// contention — the common case, one short registry write ahead of us —
/// clears on the first or second retry rather than paying the ceiling.
const LOCK_MAX_DELAY: Duration = Duration::from_millis(50);

/// Failure taking a repository's interprocess lock.
#[derive(Debug, thiserror::Error)]
pub enum RepoLockError {
    /// The lock file (or its directory) could not be created or opened.
    #[error(
        "cannot open the repository lock {path} for git common directory {common_dir}: {source}"
    )]
    Io {
        /// Lock file this was about.
        path: String,
        /// Git common directory the lock stands for.
        common_dir: String,
        /// Underlying filesystem failure.
        source: std::io::Error,
    },
    /// Someone else held the lock for longer than the budget allows. Fail
    /// closed with evidence: which lock, which repository identity, how long
    /// we waited. The holder is deliberately reported as unknown — an
    /// advisory `flock` names no owner, and inventing one (by writing a pid
    /// into the file) would be a fact this primitive cannot keep true across
    /// a crash.
    #[error(
        "the repository lock {path} for git common directory {common_dir} is held by another \
         process (holder unknown: an advisory lock names no owner); waited {waited:?} and gave \
         up rather than block indefinitely"
    )]
    Timeout {
        /// Lock file this was about.
        path: String,
        /// Git common directory the lock stands for.
        common_dir: String,
        /// How long acquisition actually waited before giving up.
        waited: Duration,
    },
}

/// An exclusive hold on one repository's interprocess lock.
///
/// Released on drop — by closing the file, which is what releases the advisory
/// lock. Nothing is deleted: the lock file itself is a durable rendezvous
/// point, and unlinking it would let a later acquirer create a *different*
/// inode at the same path and lock that instead, which is the classic way to
/// end up with two "holders" of one lock.
#[derive(Debug)]
pub struct RepositoryLock {
    /// Kept solely for its side effect: closing it unlocks.
    _file: File,
    path: PathBuf,
}

impl RepositoryLock {
    /// The lock file this hold is on. Evidence for callers that report it.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// The lock file for `common_dir` under `data_dir`.
///
/// Pure: makes no filesystem calls and creates nothing. `common_dir` is
/// expected to be already canonical — [`acquire`]'s callers derive it once per
/// operation through `git::canonical_git_common_dir` — because canonicalizing
/// here would make an identity function that sometimes hits the disk and
/// sometimes does not.
pub fn lock_path(data_dir: &Path, common_dir: &Path) -> PathBuf {
    let digest = blake3::hash(common_dir.as_os_str().as_encoded_bytes()).to_hex();
    data_dir
        .join(LOCKS_DIR)
        .join(REPO_LOCKS_DIR)
        .join(format!("{digest}.lock"))
}

/// Take the exclusive interprocess lock for `common_dir`, waiting up to
/// [`LOCK_BUDGET`] for a current holder.
///
/// Re-acquirable: this keeps no process-lifetime memory of what it has locked,
/// so the same repository may be locked and released as many times as there
/// are guarded spans.
pub fn acquire(data_dir: &Path, common_dir: &Path) -> Result<RepositoryLock, RepoLockError> {
    acquire_within(data_dir, common_dir, LOCK_BUDGET)
}

/// [`acquire`] with an explicit budget.
///
/// Public so the multi-process acceptance test can assert the *timeout* arm
/// without spending [`LOCK_BUDGET`] of wall clock to do it: the budget is the
/// one parameter of this primitive that a test cannot otherwise vary, and
/// pinning the structured refusal is worth more than keeping the knob private.
pub fn acquire_within(
    data_dir: &Path,
    common_dir: &Path,
    budget: Duration,
) -> Result<RepositoryLock, RepoLockError> {
    let path = lock_path(data_dir, common_dir);
    let io = |source: std::io::Error| RepoLockError::Io {
        path: path.display().to_string(),
        common_dir: common_dir.display().to_string(),
        source,
    };
    if let Some(parent) = path.parent() {
        create_dir_all_durable(parent).map_err(io)?;
    }
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .map_err(io)?;

    let started = Instant::now();
    let deadline = started + budget;
    let mut delay = Duration::from_millis(1);
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(RepositoryLock { _file: file, path }),
            Err(TryLockError::WouldBlock) => {}
            Err(TryLockError::Error(e)) => return Err(io(e)),
        }
        let now = Instant::now();
        if now >= deadline {
            return Err(RepoLockError::Timeout {
                path: path.display().to_string(),
                common_dir: common_dir.display().to_string(),
                waited: started.elapsed(),
            });
        }
        std::thread::sleep(delay.min(deadline - now));
        delay = (delay * 2).min(LOCK_MAX_DELAY);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two spellings of one directory must not produce two lock files — the
    /// whole defect §2.7 names, expressed at the level of the filename.
    #[test]
    fn the_lock_file_is_a_pure_function_of_the_common_directory() {
        let data = Path::new("/data");
        let common = Path::new("/repos/api/.git");
        assert_eq!(lock_path(data, common), lock_path(data, common));
        assert_ne!(
            lock_path(data, common),
            lock_path(data, Path::new("/repos/web/.git"))
        );
        // Under the data dir, never under the repository.
        let path = lock_path(data, common);
        assert!(path.starts_with(data.join(LOCKS_DIR).join(REPO_LOCKS_DIR)));
        assert!(!path.starts_with(common));
        assert_eq!(path.extension().and_then(|e| e.to_str()), Some("lock"));
    }

    #[test]
    fn acquire_creates_the_lock_file_under_the_data_dir_and_releases_on_drop() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = dir.path().join("data");
        let common = dir.path().join("repo/.git");

        let held = acquire(&data, &common).expect("first acquisition");
        let path = held.path().to_path_buf();
        assert!(path.is_file(), "the lock file is created on demand");
        assert_eq!(path, lock_path(&data, &common));
        drop(held);

        // Re-acquirable: nothing here is process-lifetime state.
        let again = acquire(&data, &common).expect("reacquisition after release");
        assert_eq!(again.path(), path);
        drop(again);
        assert!(
            path.is_file(),
            "release closes the file; it does not unlink the rendezvous point"
        );
    }

    /// Different repositories do not contend: the point of keying on identity
    /// rather than serializing all git behind one gate.
    #[test]
    fn two_repositories_hold_their_locks_independently() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = dir.path().join("data");
        let api = acquire(&data, &dir.path().join("api/.git")).expect("api");
        let web = acquire(&data, &dir.path().join("web/.git")).expect("web");
        assert_ne!(api.path(), web.path());
    }

    /// A holder that goes away is waited out rather than refused — the shape
    /// `fsutil::take_exclusive_lock` deliberately does *not* have.
    #[test]
    fn a_holder_that_releases_is_waited_out_rather_than_refused() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = dir.path().join("data");
        let common = dir.path().join("repo/.git");

        let held = acquire(&data, &common).expect("first acquisition");
        let hold_for = Duration::from_millis(60);
        let releaser = std::thread::spawn(move || {
            std::thread::sleep(hold_for);
            drop(held);
        });

        let started = Instant::now();
        let taken = acquire(&data, &common).expect("the contender must wait, not fail");
        assert!(
            started.elapsed() >= hold_for,
            "it really waited for the release rather than racing it"
        );
        drop(taken);
        releaser.join().expect("releaser");
    }

    /// The budget is a real ceiling, and exhausting it produces evidence: the
    /// lock path, the repository identity, and an explicit holder-unknown.
    #[test]
    fn a_holder_that_stays_produces_a_bounded_structured_timeout() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = dir.path().join("data");
        let common = dir.path().join("repo/.git");

        let _held = acquire(&data, &common).expect("first acquisition");
        let budget = Duration::from_millis(120);
        let started = Instant::now();
        let err =
            acquire_within(&data, &common, budget).expect_err("a held lock must not be taken");
        let elapsed = started.elapsed();

        assert!(
            elapsed >= budget,
            "it must spend its whole budget: {elapsed:?}"
        );
        assert!(
            elapsed < budget * 20,
            "and then give up, not block indefinitely: {elapsed:?}"
        );
        let RepoLockError::Timeout {
            path, common_dir, ..
        } = &err
        else {
            panic!("a contended lock must time out, not fail some other way: {err:?}");
        };
        assert_eq!(Path::new(path), lock_path(&data, &common));
        assert_eq!(Path::new(common_dir), common);
        let text = err.to_string();
        assert!(
            text.contains("holder unknown"),
            "the refusal is honest about what it cannot know: {text}"
        );
        assert!(
            text.contains(&common.display().to_string()),
            "and names the repository: {text}"
        );
    }
}
