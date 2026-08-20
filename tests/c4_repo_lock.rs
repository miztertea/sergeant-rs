//! Amendment C4 acceptance: the repository lock is *interprocess*.
//!
//! §9.4 makes the filesystem lock the correctness boundary and the in-process
//! mutex a mere optimization, and §17 asks that registry operations serialize
//! by Git common directory **across processes**. Neither claim can be tested
//! from inside one process: a `Mutex` and a `flock` are indistinguishable to a
//! single-process test, which is exactly how a rekey that quietly dropped the
//! file lock would slip through a green suite.
//!
//! So this file spawns a **real second process** and makes it hold the lock.
//!
//! # The rig
//!
//! Rather than reach for `CARGO_BIN_EXE_sgt` — which would need a CLI verb
//! whose only purpose is to sit on a lock, i.e. product surface invented for a
//! test — the helper is a `#[test]` in this same binary, re-executed through
//! `std::env::current_exe()`. It does nothing unless [`HELPER_COMMON_DIR`] is
//! set in its environment, so it is an ordinary (trivially passing) test in a
//! normal run and a lock-holding daemon-shaped process when this file drives
//! it. Two files carry the handshake, because pipes would need the helper to
//! flush at exactly the right moments and a sleep would make the test a race:
//!
//! - the helper creates its `ready` file *after* it holds the lock;
//! - the helper exits (releasing the lock by dropping it) once its `release`
//!   file appears.
//!
//! # What is asserted
//!
//! 1. A contender **waits** for a foreign holder and then succeeds — the lock
//!    is genuinely shared across processes, and it is a wait rather than a
//!    refusal (the shape `fsutil::take_exclusive_lock` deliberately does not
//!    have).
//! 2. A holder that stays produces the **bounded structured timeout**, with
//!    the lock path and the repository identity in it, instead of hanging.
//! 3. Two different *source paths* that share one Git common directory — a
//!    checkout and a linked worktree of it — contend on **one** lock file
//!    (§18, and the flipped Phase 0 pin's cross-process half).

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use tempfile::TempDir;

use sergeant_rs::runtime::git::canonical_git_common_dir;
use sergeant_rs::runtime::repolock::{self, RepoLockError};

/// Set in the helper's environment: the git checkout whose common directory
/// it should lock. Absent in an ordinary run, which is what makes the helper
/// test inert.
const HELPER_COMMON_DIR: &str = "SGT_TEST_LOCK_SOURCE";
/// Set in the helper's environment: the data dir the lock file lives under.
const HELPER_DATA_DIR: &str = "SGT_TEST_LOCK_DATA_DIR";
/// The helper creates this once it holds the lock.
const HELPER_READY: &str = "SGT_TEST_LOCK_READY";
/// The helper exits once this appears.
const HELPER_RELEASE: &str = "SGT_TEST_LOCK_RELEASE";

/// The lock-holding second process. Inert unless driven by [`hold_lock_in_a_second_process`].
///
/// It deliberately goes through the *same* public entry points the daemon
/// does — `canonical_git_common_dir` then `repolock::acquire` — rather than
/// opening the lock file by a path this test computed. A rig that computed the
/// path itself would still pass if the daemon's own derivation drifted, which
/// is the one thing this file exists to catch.
#[test]
fn repository_lock_helper_process() {
    let Ok(source) = std::env::var(HELPER_COMMON_DIR) else {
        return; // an ordinary run: nothing to do
    };
    let data_dir = PathBuf::from(std::env::var(HELPER_DATA_DIR).expect("helper data dir"));
    let ready = PathBuf::from(std::env::var(HELPER_READY).expect("helper ready path"));
    let release = PathBuf::from(std::env::var(HELPER_RELEASE).expect("helper release path"));

    let common_dir =
        canonical_git_common_dir(Path::new(&source)).expect("helper: git common directory");
    let held = repolock::acquire(&data_dir, &common_dir).expect("helper: acquire the lock");
    std::fs::write(&ready, held.path().display().to_string()).expect("helper: signal ready");

    // Hold it until told otherwise. Bounded so a parent that dies mid-test
    // cannot leave this process sitting on a lock forever.
    let deadline = Instant::now() + Duration::from_secs(120);
    while !release.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    drop(held);
}

/// A live handle on the helper process, with the paths that talk to it.
struct Helper {
    child: Child,
    release: PathBuf,
    /// The lock file the helper reported holding.
    lock_path: PathBuf,
}

impl Helper {
    /// Let the helper go and wait for it to actually exit — the lock is not
    /// released until the process is gone, so a test that asserted
    /// availability without this would be racing the kernel.
    fn release_and_wait(mut self) {
        std::fs::write(&self.release, b"go").expect("signal release");
        let status = self.child.wait().expect("helper exit");
        assert!(status.success(), "helper process failed: {status}");
    }
}

/// Spawn the helper against `source` and block until it holds the lock.
fn hold_lock_in_a_second_process(data_dir: &Path, source: &Path, scratch: &Path) -> Helper {
    let ready = scratch.join("ready");
    let release = scratch.join("release");
    let child = Command::new(std::env::current_exe().expect("current test binary"))
        .args(["--exact", "repository_lock_helper_process", "--nocapture"])
        .env(HELPER_COMMON_DIR, source)
        .env(HELPER_DATA_DIR, data_dir)
        .env(HELPER_READY, &ready)
        .env(HELPER_RELEASE, &release)
        .spawn()
        .expect("spawn the lock-holding helper");

    let deadline = Instant::now() + Duration::from_secs(60);
    while !ready.exists() {
        assert!(
            Instant::now() < deadline,
            "the helper process never reported holding the lock"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let lock_path = PathBuf::from(std::fs::read_to_string(&ready).expect("read ready"));
    Helper {
        child,
        release,
        lock_path,
    }
}

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "sergeant tests")
        .env("GIT_AUTHOR_EMAIL", "tests@example.invalid")
        .env("GIT_COMMITTER_NAME", "sergeant tests")
        .env("GIT_COMMITTER_EMAIL", "tests@example.invalid")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {args:?} failed in {}: {}",
        dir.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// A checkout with one commit, plus a linked worktree of it. Both paths share
/// one Git common directory; that is the whole point of the fixture.
fn checkout_with_linked_worktree(root: &Path) -> (PathBuf, PathBuf) {
    let primary = root.join("primary");
    std::fs::create_dir_all(&primary).expect("repo dir");
    git(&primary, &["init", "-b", "main"]);
    std::fs::write(primary.join("README.md"), "# fixture\n").expect("write");
    git(&primary, &["add", "."]);
    git(&primary, &["commit", "-m", "initial"]);

    let linked = root.join("linked");
    git(
        &primary,
        &[
            "worktree",
            "add",
            linked.to_str().expect("utf8 path"),
            "-b",
            "linked-branch",
        ],
    );
    (primary, linked)
}

/// (1) and (3): a contender waits out a foreign holder and then succeeds —
/// and the two processes met on one lock despite naming *different* source
/// paths, because those paths share one Git common directory (§18).
#[test]
fn a_second_process_holding_the_repository_lock_is_waited_out_not_refused() {
    let dir = TempDir::new().expect("tempdir");
    let data = dir.path().join("data");
    let (primary, linked) = checkout_with_linked_worktree(dir.path());

    // The helper locks via the *linked worktree*; this process contends via
    // the *primary checkout*. Different paths, one registry, one lock.
    let helper = hold_lock_in_a_second_process(&data, &linked, dir.path());

    let common_from_primary = canonical_git_common_dir(&primary).expect("common dir");
    assert_eq!(
        helper.lock_path,
        repolock::lock_path(&data, &common_from_primary),
        "two source paths sharing one git common directory must resolve to one lock file"
    );
    assert!(
        helper.lock_path.starts_with(&data),
        "the lock lives in the daemon's data dir: {}",
        helper.lock_path.display()
    );
    assert!(
        !helper.lock_path.starts_with(&primary),
        "and never inside the user's checkout"
    );

    // Release the holder shortly, from this process, while the acquisition
    // below is already waiting on it.
    let hold_for = Duration::from_millis(400);
    let releaser = std::thread::spawn(move || {
        std::thread::sleep(hold_for);
        helper.release_and_wait();
    });

    let started = Instant::now();
    let taken = repolock::acquire(&data, &common_from_primary)
        .expect("a foreign holder that goes away must be waited out, not refused");
    let waited = started.elapsed();
    releaser.join().expect("releaser thread");

    assert!(
        waited >= hold_for,
        "it must actually have waited for the other process to release: {waited:?}"
    );
    assert_eq!(
        taken.path(),
        repolock::lock_path(&data, &common_from_primary),
        "and it is the very lock file the other process was holding"
    );
    drop(taken);
}

/// (2): a holder that never lets go produces the bounded, structured refusal
/// rather than a hang.
///
/// This is the property the whole primitive is shaped around: surface
/// operations reach the runtime through `api::blocking`, which runs them
/// inline on the reactor under a current-thread runtime, so an unbounded
/// `File::lock()` would park the daemon instead of failing it. The budget is
/// passed explicitly here so the assertion costs a fraction of a second
/// rather than the production 30.
#[test]
fn a_second_process_that_keeps_the_lock_produces_a_bounded_structured_timeout() {
    let dir = TempDir::new().expect("tempdir");
    let data = dir.path().join("data");
    let (primary, linked) = checkout_with_linked_worktree(dir.path());
    let common_dir = canonical_git_common_dir(&primary).expect("common dir");

    let helper = hold_lock_in_a_second_process(&data, &linked, dir.path());

    let budget = Duration::from_millis(250);
    let started = Instant::now();
    let err = repolock::acquire_within(&data, &common_dir, budget)
        .expect_err("a lock held by another process must not be taken");
    let elapsed = started.elapsed();

    assert!(
        elapsed >= budget,
        "the whole budget is spent waiting: {elapsed:?}"
    );
    assert!(
        elapsed < budget * 20,
        "and then it gives up rather than blocking: {elapsed:?}"
    );

    let RepoLockError::Timeout {
        path,
        common_dir: reported,
        waited,
    } = &err
    else {
        panic!("a contended lock must fail as a timeout, not otherwise: {err:?}");
    };
    assert_eq!(
        Path::new(path),
        repolock::lock_path(&data, &common_dir),
        "the refusal names the lock file"
    );
    assert_eq!(
        Path::new(reported),
        common_dir,
        "and the repository identity it stands for"
    );
    assert!(*waited >= budget);
    let text = err.to_string();
    assert!(
        text.contains("holder unknown"),
        "an advisory lock names no owner, and the error says so rather than guessing: {text}"
    );

    // And once the other process really is gone, the lock is available again
    // — the refusal was about contention, not about a lock left poisoned.
    helper.release_and_wait();
    let taken = repolock::acquire(&data, &common_dir)
        .expect("the lock is free once the holding process exits");
    drop(taken);
}
