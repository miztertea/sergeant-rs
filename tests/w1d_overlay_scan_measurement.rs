//! S5 W1d: what one Work-overlay scan actually costs, against a real
//! repository corpus. **Read-only with respect to the estate, `#[ignore]`d
//! — run explicitly, never in CI.**
//!
//! W1d's brief makes the choice of overlay-scan moment follow a
//! measurement rather than a preference: "an overlay scan is a repository
//! extraction on the intelligence lane. Per-turn on a large repo may be
//! untenable; per-stage may be fine." This is that measurement, and the
//! figures it produced are recorded — dated, with this file named as the
//! method — in the estate's `knowledge/evidence/perf/`.
//!
//! Shape copied from `tests/w3_prune_measurement.rs` and
//! `tests/w2_startup_measurement.rs` (R2): resolve a real corpus, skip
//! loudly rather than fail when it is absent, print the figures, and never
//! write into the estate. The corpus repository is **cloned** into a
//! tempdir before a worktree is cut from it, so the estate's own mount
//! gains no `.git/worktrees/` entry from being measured.
//!
//! Honesty note, in the same spirit as the two files above: this measures
//! one corpus (the estate's own `sergeant-rs` mount, a few hundred
//! source files) on one host. It demonstrates the cost *shape* — a base
//! tree listing, one `git diff`, and a batched blob extraction whose cost
//! is dominated by the unchanged half — and it demonstrates nothing about
//! a repository two orders of magnitude larger.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use sergeant_rs::runtime::atlas::overlay::{WorkOverlay, changed_paths, scan_work_overlay};
use sergeant_rs::runtime::git::git as run_git;

/// The repository the measurement runs against: `$SGT_OVERLAY_CORPUS`, or
/// this estate's own `sergeant-rs` mount.
fn corpus_repo() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("SGT_OVERLAY_CORPUS") {
        return Some(PathBuf::from(dir));
    }
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join("sergeant-rs-workspace/repos/sergeant-rs"))
}

fn require_corpus() -> Option<PathBuf> {
    match corpus_repo() {
        Some(dir) if dir.join(".git").exists() => Some(dir),
        Some(dir) => {
            eprintln!(
                "SKIPPED: corpus repository {dir:?} is not a git checkout — this measurement \
                 needs a real repository and is never a hard requirement"
            );
            None
        }
        None => {
            eprintln!("SKIPPED: could not resolve a corpus repository (no $HOME, no override)");
            None
        }
    }
}

/// The median of `runs` timings of `f`, discarding the first (cold page
/// cache, cold git pack index) — a median rather than a mean because a
/// single scheduler stall should not become the reported number.
fn median_of(runs: usize, mut f: impl FnMut()) -> Duration {
    let mut timings = Vec::with_capacity(runs);
    for i in 0..=runs {
        let t = Instant::now();
        f();
        let elapsed = t.elapsed();
        if i > 0 {
            timings.push(elapsed);
        }
    }
    timings.sort();
    timings[timings.len() / 2]
}

fn tracked_files(repo: &Path, sha: &str) -> Vec<String> {
    run_git(repo, &["ls-tree", "-r", "--name-only", sha])
        .expect("ls-tree")
        .lines()
        .map(str::to_string)
        .collect()
}

/// Cost of one full `scan_work_overlay` over a real repository surface, at
/// zero / one / twenty changed paths, plus the `changed_paths` half on its
/// own — which is what an incremental form would pay if the extraction
/// half could be reused.
#[test]
#[ignore]
fn overlay_scan_cost_on_a_real_repository_surface() {
    let Some(corpus) = require_corpus() else {
        return;
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let clone = dir.path().join("mount");
    // Cloned, never worktree-added in place: the estate's mount is not
    // written to by being measured.
    run_git(
        dir.path(),
        &[
            "clone",
            "--quiet",
            "--no-hardlinks",
            corpus.to_str().expect("utf-8 corpus path"),
            clone.to_str().expect("utf-8 clone path"),
        ],
    )
    .expect("clone the corpus");
    let base = run_git(&clone, &["rev-parse", "HEAD"]).expect("head");
    let surface = dir.path().join("surface");
    run_git(
        &clone,
        &[
            "worktree",
            "add",
            "--quiet",
            "--detach",
            surface.to_str().expect("utf-8 surface path"),
            &base,
        ],
    )
    .expect("worktree add");

    let files = tracked_files(&clone, &base);
    let overlay = || WorkOverlay {
        work_id: "01MEASUREMENT".to_string(),
        repository: "sergeant-rs".to_string(),
        surface: surface.clone(),
        base_sha: base.clone(),
        ignore: Vec::new(),
    };

    const RUNS: usize = 5;
    let clean = median_of(RUNS, || {
        scan_work_overlay(&overlay()).expect("scan a pristine surface");
    });
    let diff_only = median_of(RUNS, || {
        changed_paths(&surface, &base).expect("changed paths");
    });

    let mut edited: Vec<&String> = files
        .iter()
        .filter(|p| p.ends_with(".rs") || p.ends_with(".md"))
        .collect();
    edited.truncate(20);
    assert!(!edited.is_empty(), "the corpus has no .rs or .md files");

    let touch = |n: usize| {
        for path in edited.iter().take(n) {
            let full = surface.join(path);
            let mut body = std::fs::read_to_string(&full).expect("read");
            body.push_str("\n// measurement edit\n");
            std::fs::write(&full, body).expect("write");
        }
    };

    touch(1);
    let one = median_of(RUNS, || {
        scan_work_overlay(&overlay()).expect("scan a surface with one edit");
    });
    let changed_one = changed_paths(&surface, &base).expect("changed paths");

    touch(edited.len());
    let many = median_of(RUNS, || {
        scan_work_overlay(&overlay()).expect("scan a surface with many edits");
    });
    let changed_many: BTreeSet<String> = changed_paths(&surface, &base).expect("changed paths");

    let scanned = scan_work_overlay(&overlay()).expect("scan");
    println!(
        "overlay scan cost, corpus={corpus:?} base={base} tracked_files={} indexed_files={} \
         runs={RUNS} (median, first discarded):\n  \
         changed_paths only               = {diff_only:?}\n  \
         full scan, 0 changed paths       = {clean:?}\n  \
         full scan, {} changed paths      = {one:?}\n  \
         full scan, {} changed paths      = {many:?}",
        files.len(),
        scanned.scan.files.len(),
        changed_one.len(),
        changed_many.len(),
    );
}
