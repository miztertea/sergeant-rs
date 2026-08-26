//! W5 brief deliverable 2 — H1 §11 acceptance battery, walked literally.
//!
//! `knowledge/evidence/resources/host-atlas-series/H1-HOST-RUNTIME.md` §11
//! lists ten criteria. This file is the checklist: one numbered section per
//! criterion, each either (a) a self-contained test here, or (b) — where an
//! earlier wave already pinned the exact claim by name — a comment pointing
//! at that test rather than a byte-for-byte duplicate of it. Criteria 1 and
//! 5 share one real-CLI, two-estate, end-to-end walk (below): that is also
//! the brief's own "two-estate end-to-end through the real CLI" requirement.
//!
//! Running this file alone proves nothing about the suites it references —
//! that is what the wave's own `cargo test --locked` is for. What this file
//! proves is that every criterion has *a* proof somewhere, named.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

use sergeant_rs::runtime::journal::Journal;

mod support;
use support::DataDir;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Run `sgt` bound to `data_dir` (the host runtime root), from `cwd`, with
/// `extra` args prepended by `-C <cwd>` when `cwd` is not `data_dir` itself —
/// mirrors `m2_daemon_api.rs`'s own `sgt`/`sgt_in` shape, reimplemented here
/// (private to that file) rather than exposed through `support`, since this
/// is the only other file that needs the two-estate shape.
fn sgt(data_dir: &Path, cwd: &Path, args: &[&str]) -> Output {
    let mut command = Command::new(SGT);
    command
        .current_dir(cwd)
        .arg("-C")
        .arg(cwd)
        .arg("--data-dir")
        .arg(data_dir)
        .args(args);
    command.output().expect("run sgt")
}

// ------------------------------------------------------------------------
// 1 & 5. Two exact-root estates submit Work to one daemon; one journal
// reconstructs Work across both.
//
// The deeper mechanics of this pair — the daemon reuse assertion, the
// `sergeant/*` branch/repository shape, per-Work `created_by` — are already
// pinned by `tests/m2_daemon_api.rs`'s
// `t7_cli_end_to_end_a_second_estate_is_admitted_and_a_second_process_fails_closed`.
// This is the acceptance file's own literal walk of the same two criteria,
// self-contained here per the brief's "two-estate end-to-end through the
// real CLI" instruction.
// ------------------------------------------------------------------------

#[test]
fn h1_acceptance_1_and_5_two_estates_one_daemon_one_journal() {
    let data_dir = DataDir::new();
    support::scaffold_estate(data_dir.path(), "acc-a", &["solo"]);
    let estate_b = tempfile::TempDir::new().expect("estate b tempdir");
    support::scaffold_estate(estate_b.path(), "acc-b", &["solo"]);

    // Criterion 1: A submits first — the daemon that answers is spawned for
    // A alone (host-scoped, bound to zero estates until first contact).
    let a = sgt(data_dir.path(), data_dir.path(), &["run", "acceptance A"]);
    assert!(
        a.status.success(),
        "estate A's submission must succeed: {}",
        stderr(&a)
    );
    let before = data_dir.daemon_pids();
    assert_eq!(before.len(), 1, "exactly one daemon: {before:?}");

    // B addresses the *same* host runtime root with its own exact estate
    // root — lazily admitted, never a second daemon.
    let b = sgt(data_dir.path(), estate_b.path(), &["run", "acceptance B"]);
    assert!(
        b.status.success(),
        "estate B must be admitted into the running daemon, not refused: {}",
        stderr(&b)
    );
    assert_eq!(
        data_dir.daemon_pids(),
        before,
        "criterion 1: a second estate must reuse the running daemon, never spawn another"
    );

    // `sgt work list --json` — host-scoped, no `-C` — sees both estates'
    // Work on the one fleet.
    let listed = sgt(
        data_dir.path(),
        data_dir.path(),
        &["work", "list", "--json"],
    );
    assert!(listed.status.success(), "{}", stderr(&listed));
    let listed: Value = serde_json::from_str(&stdout(&listed)).expect("work list JSON");
    let works = listed["works"].as_array().expect("works array");
    assert_eq!(works.len(), 2, "both estates' Work on one daemon: {listed}");
    for intent in ["acceptance A", "acceptance B"] {
        let work = works
            .iter()
            .find(|w| w["intent"] == intent)
            .unwrap_or_else(|| panic!("{intent} missing from {listed}"));
        assert_eq!(work["state"], "completed", "{intent}: {work}");
    }

    // Criterion 5: one journal, no estate UUID, reconstructs both roots from
    // the envelope's own `workspace_id` (D1) alone.
    let mut roots: Vec<String> = Journal::replay_data_dir(data_dir.path())
        .expect("replay")
        .map(|e| e.expect("event"))
        .filter(|e| e.kind == "work.submitted")
        .map(|e| {
            e.workspace_id
                .clone()
                .expect("submitted carries its estate")
        })
        .collect();
    roots.sort();
    roots.dedup();
    let mut expected = vec![
        std::fs::canonicalize(data_dir.path())
            .expect("canonical")
            .to_string_lossy()
            .into_owned(),
        std::fs::canonicalize(estate_b.path())
            .expect("canonical")
            .to_string_lossy()
            .into_owned(),
    ];
    expected.sort();
    assert_eq!(
        roots, expected,
        "criterion 5: a replay of the one journal must name both estates' own roots"
    );

    data_dir.reap();
}

// ------------------------------------------------------------------------
// 2. One TUI renders Work from both estates and filters by estate.
//
// Pinned: `tests/m6_surfaces.rs::t5_fleet_and_estate_screen_span_two_estates`
// (drives the compiled-in TUI over a real two-estate daemon; asserts both
// the unfiltered Fleet view and the estate-filtered view render the right
// rows).
// ------------------------------------------------------------------------

// ------------------------------------------------------------------------
// 3. `sgt run` outside an estate still refuses unless `-C` is explicit.
// ------------------------------------------------------------------------

#[test]
fn h1_acceptance_3_run_outside_an_estate_refuses_unless_dash_c_is_explicit() {
    let bare = tempfile::TempDir::new().expect("bare dir");

    // No `-C`, cwd is not an estate: refuses before touching a daemon.
    let refused = Command::new(SGT)
        .current_dir(bare.path())
        .env_remove("SGT_DATA_DIR")
        .args(["run", "should never be admitted"])
        .output()
        .expect("run sgt");
    assert!(
        !refused.status.success(),
        "run must refuse outside an estate"
    );
    assert!(
        stderr(&refused).contains("does not search parent directories"),
        "the refusal must carry the exact-root diagnostic: {}",
        stderr(&refused)
    );

    // `-C <exact-estate-root>` from that same non-estate cwd succeeds.
    let data_dir = DataDir::new();
    let estate = tempfile::TempDir::new().expect("estate tempdir");
    support::scaffold_estate(estate.path(), "acc-c", &["solo"]);
    let addressed = Command::new(SGT)
        .current_dir(bare.path())
        .arg("-C")
        .arg(estate.path())
        .arg("--data-dir")
        .arg(data_dir.path())
        .args(["run", "explicit -C from elsewhere"])
        .output()
        .expect("run sgt");
    assert!(
        addressed.status.success(),
        "criterion 3: `-C <estate-root>` must explicitly address an estate from anywhere: {}",
        stderr(&addressed)
    );
    data_dir.reap();
}

// ------------------------------------------------------------------------
// 4. Closing the invoking terminal/harness does not kill accepted Work.
//
// Pinned: `tests/w3_client_surface.rs::the_spawned_daemon_survives_its_client_in_its_own_process_group`
// (asserts the spawned daemon's pgid differs from the spawning client's, so
// a hung-up controlling terminal's SIGHUP never reaches it).
// ------------------------------------------------------------------------

// ------------------------------------------------------------------------
// 6. Current Git surface integrity tests remain valid.
//
// Not one named test — this criterion is "the standing Git-surface suites
// still pass", which the wave's own `cargo test --locked` proves directly:
// `tests/c4_repo_lock.rs`, `tests/c11_injectable_git.rs`,
// `tests/e_git_admission.rs`, `tests/e_admission_uses_no_network_git.rs`,
// `tests/e_sweep_uses_only_local_git.rs`. Nothing here re-runs them; this
// comment is the checklist entry naming where the proof lives.
// ------------------------------------------------------------------------

// ------------------------------------------------------------------------
// 7. Daemon-only Atlas ownership remains structural.
//
// Pinned: `tests/m5_projections.rs::t2_the_duckdb_file_has_exactly_one_owner`
// (greps the source tree: nothing outside `daemon`/`runtime` opens the
// DuckDB file).
// ------------------------------------------------------------------------

// ------------------------------------------------------------------------
// 8. Execution remains schedulable while a bounded intelligence/OCR
// backlog exists.
//
// Pinned: `src/runtime/engine.rs`'s
// `tests::execution_lane_exhaustion_never_touches_the_intelligence_lane`
// (a full execution lane never blocks an intelligence-lane acquisition, and
// vice versa — the two H1-15 lanes are independently bounded).
// ------------------------------------------------------------------------

// ------------------------------------------------------------------------
// 9. Supported platform installation uses native per-user service
// management rather than a custom supervisor.
//
// Pinned: `tests/w4c_service_doctor.rs::install_service_enables_when_a_scripted_systemctl_reports_reachable`
// (systemd user-unit generation + enablement) and
// `tests/w4c_service_doctor.rs::legacy_estate_runtime_warns_when_estate_local_daemon_state_is_present`
// (the cutover-gate doctor row).
// ------------------------------------------------------------------------

// ------------------------------------------------------------------------
// 10. Product docs follow merged #263 ownership — no proposal corpus is
// restored to sergeant-rs.
//
// The citation-integrity half is pinned:
// `tests/f_doctrine_skew.rs::no_readme_contributing_src_test_or_workflow_file_cites_a_removed_path`
// (no `docs/adr/`, `docs/proposals/`, etc. citation anywhere live). This
// wave adds one more, W5-specific guard directly: brief deliverable 3
// requires landing host-topology architecture prose *without* recreating a
// `docs/adr/` tree, so this asserts that tree still does not exist.
// ------------------------------------------------------------------------

#[test]
fn h1_acceptance_10_no_docs_adr_tree_exists() {
    assert!(
        !repo_root().join("docs/adr").exists(),
        "criterion 10 / brief deliverable 3: docs/adr was dissolved by the split and must not \
         be recreated — host-topology architecture prose belongs in docs/concepts/"
    );
}
