//! X3a's read-only claim, via the recording Git binary: **an Atlas scan of a
//! declared mount never fetches, pulls, switches, resets or writes, and never
//! reads the working tree.**
//!
//! The module doc of `runtime::atlas::git` states this; a comment cannot be
//! violated, so this is the test that can be. A shim records every Git
//! invocation — working directory and full argument vector — and then `exec`s
//! the real Git, so a complete, successful scan runs end to end and the
//! recording is of the real thing rather than of a stub's idea of it.
//!
//! **This must stay the only test in this file.** `std::env::set_var` is
//! process-global rather than thread-scoped, and `runtime::git::command` reads
//! the override fresh on every invocation — a sibling test in this process
//! would see the shim too, and would race the `TempDir` holding it. Cargo
//! gives every integration-test file its own process, so a file with one test
//! has no sibling to leak into. `tests/e_admission_uses_no_network_git.rs`
//! carries the same rule for the same reason.
//!
//! # What is asserted, and why in two parts
//!
//! * **Verbs.** Everything that fetches, mutates history, moves a ref, or
//!   changes a checkout is forbidden outright: a derived index has no business
//!   running any of them, in any directory, ever. The allowed set is named
//!   positively as well, so a scan that grew a *new* Git verb has to be
//!   re-decided here rather than sliding in under "not on the forbidden list".
//! * **State.** Verbs alone would not catch a read that happened to have side
//!   effects, so the mount's HEAD, its full ref listing, its `git status`
//!   porcelain and every file byte are compared before and after. A scan must
//!   leave a repository indistinguishable from one that was never scanned.

use std::collections::BTreeSet;

use tempfile::TempDir;

use sergeant_rs::runtime::atlas::git::{EstateGitSource, scan_estate_git};
use sergeant_rs::runtime::git::{GIT_BIN_ENV, git};

mod support;
use support::{parse_log, real_git, write_recording_shim};

/// Verbs that must never appear on a scan path, in any directory.
const FORBIDDEN: &[&str] = &[
    "fetch",
    "pull",
    "push",
    "remote",
    "ls-remote",
    "clone",
    "rebase",
    "merge",
    "cherry-pick",
    "switch",
    "checkout",
    "reset",
    "commit",
    "add",
    "worktree",
    "gc",
    "prune",
    "repack",
    "update-ref",
    "symbolic-ref",
    "branch",
    "tag",
    "stash",
    "clean",
    "restore",
    "apply",
    "update-index",
    "write-tree",
    "hash-object",
];

/// The complete set of verbs a scan is allowed to run. Named positively so a
/// new one is a decision, not an omission.
const ALLOWED: &[&str] = &["rev-parse", "ls-tree", "cat-file"];

#[test]
fn an_atlas_scan_runs_only_read_only_git_and_changes_nothing() {
    let dir = TempDir::new().expect("tempdir");
    let mount = dir.path().join("mount");
    std::fs::create_dir_all(&mount).expect("mkdir");

    // Built with the real Git, before the shim is installed, so the fixture's
    // own setup is not part of the recording.
    let real = real_git();
    git(&mount, &["init", "--initial-branch=main"]).expect("init");
    git(&mount, &["config", "user.email", "t@example.com"]).expect("email");
    git(&mount, &["config", "user.name", "T"]).expect("name");
    git(
        &mount,
        &["remote", "add", "origin", "https://example.invalid/x.git"],
    )
    .expect("remote add");
    for (path, body) in [
        ("README.md", "# Readme\n\nbody\n"),
        ("docs/guide.md", "# Guide\n\n## Section\n\ntext\n"),
        ("notes.txt", "plain\n"),
        ("src/main.rs", "fn main() {}\n"),
    ] {
        let full = mount.join(path);
        std::fs::create_dir_all(full.parent().expect("parent")).expect("mkdir");
        std::fs::write(&full, body).expect("write");
    }
    git(&mount, &["add", "-A"]).expect("add");
    git(&mount, &["commit", "-m", "base"]).expect("commit");
    let pinned = git(&mount, &["rev-parse", "HEAD"]).expect("head");

    // An uncommitted edit and an untracked file: if the scan read the working
    // tree at any point, this is what would show up in its rows.
    std::fs::write(mount.join("README.md"), "# WORKING TREE ONLY\n").expect("write");
    std::fs::write(mount.join("untracked.md"), "# Untracked\n").expect("write");

    let before = State::of(&mount);

    let shim_dir = TempDir::new().expect("tempdir");
    let log = shim_dir.path().join("git-invocations.log");
    let shim = shim_dir.path().join("recording-git");
    write_recording_shim(&shim, &log, &real);

    // SAFETY (test-only): this binary runs exactly one test, so there is no
    // other thread in this process to observe the mutation — the whole reason
    // this file holds one test. Removed only once every git invocation under
    // test has finished, with the shim's `TempDir` still alive throughout.
    unsafe { std::env::set_var(GIT_BIN_ENV, &shim) };
    let scanned = scan_estate_git(&EstateGitSource {
        name: "product".to_string(),
        mount: mount.clone(),
        pinned_sha: pinned.clone(),
        ignore: Vec::new(),
    });
    unsafe { std::env::remove_var(GIT_BIN_ENV) };

    let scanned = scanned.expect("the scan must succeed for the recording to mean anything");
    // Four since X3b: `src/main.rs` is claimed by a grammar, so it is an
    // extractable resource rather than an unsupported one — and the extra
    // resource is read out of the object store like the other three, which is
    // exactly what this suite exists to keep true.
    assert_eq!(scanned.scan.files.len(), 4, "four extractable resources");
    assert!(scanned.drift.is_none(), "HEAD never moved");

    // The committed bytes, not the working tree's.
    let readme = scanned
        .scan
        .files
        .iter()
        .find(|f| f.relative_path == "README.md")
        .expect("README.md");
    assert!(
        readme.units[0].text.contains("# Readme"),
        "the scan read the working tree instead of the object store: {:?}",
        readme.units[0].text
    );
    assert!(
        !scanned
            .scan
            .coverage
            .iter()
            .any(|r| r.path.as_deref() == Some("untracked.md")),
        "an untracked working-tree file entered a pinned scan"
    );

    let recorded = parse_log(&std::fs::read_to_string(&log).expect("read recording"));
    assert!(
        !recorded.is_empty(),
        "nothing was recorded — the shim was not actually used"
    );
    let verbs: BTreeSet<String> = recorded
        .iter()
        .filter_map(|invocation| invocation.verb().map(str::to_string))
        .collect();
    for forbidden in FORBIDDEN {
        assert!(
            !verbs.contains(*forbidden),
            "a scan ran `git {forbidden}`; recorded verbs: {verbs:?}"
        );
    }
    for verb in &verbs {
        assert!(
            ALLOWED.contains(&verb.as_str()),
            "a scan ran `git {verb}`, which is not in the allowed read-only set \
             {ALLOWED:?} — if it belongs there, add it here deliberately"
        );
    }
    // Batched, not one process per file: three blobs were read, and
    // `cat-file` ran once.
    assert_eq!(
        recorded
            .iter()
            .filter(|i| i.verb() == Some("cat-file"))
            .count(),
        1,
        "blob reads were not batched: {recorded:?}"
    );

    assert_eq!(
        before,
        State::of(&mount),
        "the scan changed the mount it was only supposed to read"
    );
}

/// Everything about a repository that a scan must not move.
#[derive(Debug, PartialEq, Eq)]
struct State {
    head: String,
    refs: String,
    porcelain: String,
    files: Vec<(String, Vec<u8>)>,
}

impl State {
    fn of(mount: &std::path::Path) -> Self {
        let mut files = Vec::new();
        collect(mount, mount, &mut files);
        files.sort();
        Self {
            head: git(mount, &["rev-parse", "HEAD"]).expect("head"),
            refs: git(mount, &["show-ref"]).unwrap_or_default(),
            porcelain: git(mount, &["status", "--porcelain"]).expect("status"),
            files,
        }
    }
}

/// Every working-tree file's bytes, excluding `.git` — whose internals Git
/// itself rewrites for ordinary reasons (an index refresh, a pack), and which
/// `head`/`refs` above already cover at the level that matters.
fn collect(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<(String, Vec<u8>)>) {
    for entry in std::fs::read_dir(dir).expect("read_dir").flatten() {
        let path = entry.path();
        let name = entry.file_name();
        if name == ".git" {
            continue;
        }
        if path.is_dir() {
            collect(root, &path, out);
        } else {
            out.push((
                path.strip_prefix(root)
                    .expect("under root")
                    .display()
                    .to_string(),
                std::fs::read(&path).expect("read"),
            ));
        }
    }
}
