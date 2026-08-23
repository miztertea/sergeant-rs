//! #231(b): a structural guard against a *new* orphaned test suite — one
//! present in `tests/*.rs` but wired into neither coverage stage script, so
//! it contributes nothing to Gate D however green it runs (the #231 lesson,
//! also cited by scripts/coverage/c2-suites.sh's own 2026-08-23 comment).
//! This is a floor-maintenance guard, not an audit of the suites it finds
//! already missing at authorship time — those are named in ALLOWLIST below,
//! each with its own reason, and #231(a) is the tracked follow-up to judge
//! them one by one.
//!
//! Must live in a suite wired into C2 or C3 itself (no self-orphaning) —
//! this file is added to c2-suites.sh in the same commit.

use std::path::PathBuf;

/// Pre-existing orphans at authorship time (2026-08-23, opencode W2), each
/// present in neither `scripts/coverage/c2-suites.sh` nor
/// `c3-spawning-suites.sh`. #231(a) is the tracked follow-up to judge each
/// one; this guard's job is only to stop the set from growing silently.
const ALLOWLIST: &[(&str, &str)] = &[
    ("a1_floor_awareness", "pre-existing, #231(a) audit pending"),
    ("a4_blob_ref_pinning", "pre-existing, #231(a) audit pending"),
    ("c11_injectable_git", "pre-existing, #231(a) audit pending"),
    ("c4_repo_lock", "pre-existing, #231(a) audit pending"),
    (
        "e_admission_uses_no_network_git",
        "pre-existing, #231(a) audit pending",
    ),
    ("e_git_admission", "pre-existing, #231(a) audit pending"),
    (
        "e_sweep_uses_only_local_git",
        "pre-existing, #231(a) audit pending",
    ),
    ("e_work_sweep", "pre-existing, #231(a) audit pending"),
    ("f_doctrine_skew", "pre-existing, #231(a) audit pending"),
    ("i9_floor_pinning", "pre-existing, #231(a) audit pending"),
    ("w2_startup_cache", "pre-existing, #231(a) audit pending"),
    (
        "w2_startup_measurement",
        "pre-existing, #231(a) audit pending",
    ),
    (
        "w3_allowlist_equivalence",
        "pre-existing, #231(a) audit pending",
    ),
    (
        "w3_prune_crash_windows",
        "pre-existing, #231(a) audit pending",
    ),
    ("w3_prune_engine", "pre-existing, #231(a) audit pending"),
    (
        "w3_prune_measurement",
        "pre-existing, #231(a) audit pending",
    ),
    (
        "w4_doctor_journal_growth",
        "pre-existing, #231(a) audit pending",
    ),
    ("w4_read_surfaces", "pre-existing, #231(a) audit pending"),
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every `tests/*.rs` basename (minus extension) — Cargo auto-discovers each
/// as its own test binary (no `[[test]]` table in Cargo.toml narrows this).
fn all_suites() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(repo_root().join("tests"))
        .expect("read tests/")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("rs"))
        .map(|e| e.path().file_stem().unwrap().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// Suite names invoked via `--test <name>` in one coverage stage script.
fn wired_suites(script: &str) -> Vec<String> {
    let text = std::fs::read_to_string(repo_root().join("scripts/coverage").join(script))
        .unwrap_or_else(|e| panic!("read {script}: {e}"));
    text.lines()
        .filter_map(|line| line.split_once("--test "))
        .map(|(_, rest)| rest.split_whitespace().next().unwrap_or("").to_string())
        .filter(|name| !name.is_empty())
        .collect()
}

#[test]
fn every_suite_is_wired_or_explicitly_allowlisted() {
    let wired: std::collections::BTreeSet<String> = wired_suites("c2-suites.sh")
        .into_iter()
        .chain(wired_suites("c3-spawning-suites.sh"))
        .collect();
    let allowlist: std::collections::BTreeSet<&str> =
        ALLOWLIST.iter().map(|(name, _)| *name).collect();

    let mut unexplained = Vec::new();
    for suite in all_suites() {
        if !wired.contains(&suite) && !allowlist.contains(suite.as_str()) {
            unexplained.push(suite);
        }
    }
    assert!(
        unexplained.is_empty(),
        "new orphaned suite(s) wired into neither c2-suites.sh nor \
         c3-spawning-suites.sh, and not named in this test's ALLOWLIST with a \
         reason: {unexplained:?} — wire it into a coverage stage, or add an \
         allowlist entry with a specific reason (#231(b))"
    );
}
