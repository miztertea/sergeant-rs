//! Small filesystem helpers shared by the event core.

use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// How long [`take_exclusive_lock`] waits out a lock this process could be
/// holding a leaked duplicate of. Orders of magnitude above a fork→exec
/// window, and only ever spent when this same process has held that lock
/// before.
const LOCK_BUDGET: Duration = Duration::from_secs(1);
/// Longest pause between attempts. Backoff starts far below it, so the
/// ordinary transient case clears on the first retry.
const LOCK_MAX_DELAY: Duration = Duration::from_millis(25);

/// Lock files this process has itself taken an exclusive lock on at least
/// once. See [`take_exclusive_lock`] for why that history is what decides
/// whether a refusal is trustworthy.
static SELF_LOCKED: Mutex<BTreeSet<PathBuf>> = Mutex::new(BTreeSet::new());

/// Take the exclusive advisory lock on `path`'s open `file`. `Ok(true)` means
/// this process now holds it; `Ok(false)` means someone else really does and
/// the caller must fail closed.
///
/// A bare `try_lock` is not a sound test for "someone else owns this", but
/// only for locks *this process has held before*. `flock` belongs to the open
/// file description, and `fork` duplicates every descriptor in the process:
/// between a child's fork and its exec — the window before `O_CLOEXEC` fires
/// — that child holds a duplicate of every lock the parent had open,
/// including one another thread dropped a microsecond ago. Sergeant shells
/// out to git constantly (§11), so this is how a daemon that has just
/// released a data dir gets told the data dir is still owned, by its own git
/// subprocess. Such a duplicate cannot outlive the child's exec, so waiting
/// briefly resolves it.
///
/// A lock file this process has never opened cannot have been duplicated into
/// any child of ours — descriptors are inherited at fork, not conjured — so a
/// refusal there is real and is returned immediately. That is what keeps a
/// second daemon failing closed *now* rather than a second later: it never
/// held the data dir's lock, so it has nothing to wait out.
pub(crate) fn take_exclusive_lock(path: &Path, file: &File) -> std::io::Result<bool> {
    let may_be_our_own_leak = self_locked().contains(path);
    let deadline = Instant::now() + LOCK_BUDGET;
    let mut delay = Duration::from_millis(1);
    loop {
        match file.try_lock() {
            Ok(()) => {
                self_locked().insert(path.to_path_buf());
                return Ok(true);
            }
            Err(TryLockError::WouldBlock) => {}
            Err(TryLockError::Error(e)) => return Err(e),
        }
        if !may_be_our_own_leak || Instant::now() >= deadline {
            return Ok(false);
        }
        std::thread::sleep(delay);
        delay = (delay * 2).min(LOCK_MAX_DELAY);
    }
}

fn self_locked() -> std::sync::MutexGuard<'static, BTreeSet<PathBuf>> {
    // A poisoned set only means some thread panicked while inserting; the
    // set's contents are still exactly what they were, and refusing to lock
    // the journal over it would be a worse outcome than carrying on.
    SELF_LOCKED.lock().unwrap_or_else(|e| e.into_inner())
}

/// Create `dir` and any missing ancestors, then fsync the parent of every
/// directory actually created, so the new dirents survive a crash — the same
/// crash-consistency recipe [`write_atomic`] applies to files, carried up the
/// ancestor chain. Without this, the first fsync-acknowledged write inside a
/// freshly created directory could vanish with the directory itself. When the
/// whole path already exists this issues no syncs at all.
pub(crate) fn create_dir_all_durable(dir: &Path) -> std::io::Result<()> {
    let mut created = Vec::new();
    let mut cur = dir;
    while !cur.exists() {
        created.push(cur.to_path_buf());
        match cur.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => cur = parent,
            _ => break,
        }
    }
    fs::create_dir_all(dir)?;
    for new_dir in &created {
        if let Some(parent) = new_dir.parent()
            && !parent.as_os_str().is_empty()
        {
            File::open(parent)?.sync_all()?;
        }
    }
    Ok(())
}

/// Write `bytes` to `path` atomically and durably: a fresh uniquely-named
/// tmp file next to `path` is written and fsynced, renamed over `path`, and
/// the containing directory is fsynced. A reader never observes a partial
/// file, and a crash leaves either the old content or the new — never a torn
/// mix. Replacement arrives as a new inode (rename), never as an in-place
/// truncate of the destination.
pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    write_atomic_mode(path, bytes, None)
}

/// [`write_atomic`], but the file is born owner-only (mode `0600` on Unix):
/// the tmp file is created with the restrictive mode *before* any bytes are
/// written, so no reader ever observes the content under wider permissions —
/// not even between write and a later chmod. Used for the runtime descriptor,
/// which carries the bearer token.
pub(crate) fn write_atomic_secret(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    write_atomic_mode(path, bytes, Some(0o600))
}

/// The one crash-consistency recipe both public writers use, so a future fix
/// to the durability sequence lands in one place: write a uniquely-named tmp
/// file next to `path` (created with `mode`, when given, before any bytes
/// exist), fsync it, rename it over `path`, fsync the directory.
fn write_atomic_mode(path: &Path, bytes: &[u8], mode: Option<u32>) -> std::io::Result<()> {
    let tmp = path.with_extension(format!("tmp-{}", ulid::Ulid::generate()));
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    if let Some(mode) = mode {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(mode);
    }
    #[cfg(not(unix))]
    let _ = mode; // No file modes to honour off Unix.
    let mut file = options.open(&tmp)?;
    file.write_all(bytes)?;
    file.sync_data()?;
    drop(file);
    fs::rename(&tmp, path)?;
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Waiting out a possibly-leaked duplicate must not soften what the lock
    /// means: a holder that is really there is still refused, and the lock is
    /// available again the moment it is released. (That a *different process*
    /// is refused too is pinned end to end by M2's two-concurrent-daemons
    /// acceptance test, which spawns a real second `sgt daemon` and requires
    /// it to exit non-zero. How *quickly* that refusal comes is a latency
    /// property, not a correctness one, and is deliberately not asserted:
    /// pinning it would need a second process holding the lock, which is
    /// harness enough to be worse than the mutation it would catch.)
    #[test]
    fn a_held_lock_is_still_refused_and_is_free_again_once_released() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().join("some.lock");
        let open = || {
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(false)
                .open(&path)
                .expect("open lock file")
        };

        let held = open();
        assert!(take_exclusive_lock(&path, &held).expect("first lock"));

        let contender = open();
        assert!(
            !take_exclusive_lock(&path, &contender).expect("second attempt"),
            "a lock that is genuinely held must be refused, however long we wait"
        );

        drop(held);
        let successor = open();
        assert!(
            take_exclusive_lock(&path, &successor).expect("third attempt"),
            "release must make it available again"
        );
    }

    /// The other half of the rule, and the reason the wait exists at all: a
    /// lock this process has held can still be held for a moment by a
    /// duplicate of our own descriptor that a forked child carried into its
    /// pre-`exec` window. A bare `try_lock` calls that "someone else owns the
    /// data dir" and fails a daemon start that should have succeeded — which
    /// is exactly the intermittent `Locked` this suite used to produce. The
    /// leak is timing, not identity, so it is reproduced here as timing: a
    /// holder that goes away shortly, on a path this process has locked.
    #[test]
    fn a_transient_holder_of_a_lock_this_process_has_held_is_waited_out() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().join("some.lock");
        let open = || {
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(false)
                .open(&path)
                .expect("open lock file")
        };

        let held = open();
        assert!(take_exclusive_lock(&path, &held).expect("first lock"));

        let started = Instant::now();
        let hold_for = Duration::from_millis(50);
        let releaser = std::thread::spawn(move || {
            std::thread::sleep(hold_for);
            drop(held);
        });

        let contender = open();
        assert!(
            take_exclusive_lock(&path, &contender).expect("second attempt"),
            "a holder that is about to go away must be waited out, not reported \
             as a rival owner of the data dir"
        );
        assert!(
            started.elapsed() >= hold_for,
            "and it really did wait for the release rather than racing it"
        );
        releaser.join().expect("releaser");
    }

    #[test]
    fn create_dir_all_durable_creates_nested_dirs_and_is_idempotent() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let nested = dir.path().join("a").join("b").join("c");
        create_dir_all_durable(&nested).expect("create nested");
        assert!(nested.is_dir());
        // Idempotent on an existing path (the no-sync fast path).
        create_dir_all_durable(&nested).expect("recreate");
        assert!(nested.is_dir());
    }

    #[test]
    fn write_atomic_replaces_via_rename_and_leaves_no_tmp() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().join("state.json");
        write_atomic(&path, b"one").expect("first write");
        assert_eq!(fs::read(&path).expect("read"), b"one");

        #[cfg(unix)]
        let first_inode = {
            use std::os::unix::fs::MetadataExt;
            fs::metadata(&path).expect("metadata").ino()
        };

        write_atomic(&path, b"two").expect("second write");
        assert_eq!(fs::read(&path).expect("read"), b"two");

        // The replacement arrived by renaming a fresh file over the path,
        // not by opening and truncating the destination in place: a plain
        // truncating write would keep the inode.
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            assert_ne!(
                fs::metadata(&path).expect("metadata").ino(),
                first_inode,
                "atomic replace must produce a new inode"
            );
        }

        // No tmp litter remains.
        let names: Vec<String> = fs::read_dir(dir.path())
            .expect("read dir")
            .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["state.json"]);
    }
}
