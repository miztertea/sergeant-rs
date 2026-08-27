//! S2 V4 brief deliverable 1 — W1 §13 acceptance battery, walked literally.
//!
//! `knowledge/evidence/resources/host-atlas-series/W1-HIERARCHICAL-EXECUTION.md`
//! §13 lists nine criteria. Following `w5_h1_acceptance.rs`'s precedent: one
//! numbered section per criterion, each either (a) a self-contained test
//! here, or (b) — where an earlier S2 wave already pinned the exact claim by
//! name — a comment pointing at that test rather than a byte-for-byte
//! duplicate of it.
//!
//! Running this file alone proves nothing about the suites it references —
//! that is what the wave's own `cargo test --locked` is for. What this file
//! proves is that every criterion has *a* proof somewhere, named.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

// ------------------------------------------------------------------------
// 1. A direct stage can contain a nested `workflow.toml` and execute its
//    leaf stages through the existing engine.
//
// Pinned: `tests/m11_nested_workflow.rs::a_two_level_nested_workflow_runs_its_leaves_in_order_and_completes`
// (that file's own header names this test as covering W1 §13 items 1-3).
// ------------------------------------------------------------------------

// ------------------------------------------------------------------------
// 2. Nested packages can recurse at least two levels in acceptance tests.
//
// Pinned: `tests/m11_nested_workflow.rs::a_leaf_that_closes_two_containers_at_once_gates_them_innermost_first`
// (a three-level fixture — the multi-boundary case the plan panel named —
// whose deepest leaf is simultaneously the last leaf of two ancestor
// containers; strictly deeper than the two-level floor this item asks for).
// ------------------------------------------------------------------------

// ------------------------------------------------------------------------
// 3. Each leaf actor is a fresh execution over the same parent Work
//    surfaces.
//
// Pinned: `tests/m11_nested_workflow.rs::a_two_level_nested_workflow_runs_its_leaves_in_order_and_completes`
// (same test as item 1 — its own header comment names items 1-3 together:
// four leaves, four distinct `StageEntered`/`StageCompleted` pairs, one
// Work, one surface).
// ------------------------------------------------------------------------

// ------------------------------------------------------------------------
// 4. Hierarchical stage IDs journal/replay/render correctly.
//
// Pinned: `tests/m11_nested_workflow.rs::a_composed_stage_id_round_trips_through_events_analytics_and_work_show`
// (events, the analytics fold, and `sgt work show` all carry the composed
// id verbatim) and
// `tests/m11_nested_workflow.rs::a_restarted_daemon_reconstructs_the_deepest_incomplete_path_and_retry_re_enters_it`
// (replay/recovery half: a daemon killed mid-nest comes back naming the
// exact nested leaf from the journal alone, and `retry` re-enters it).
// ------------------------------------------------------------------------

// ------------------------------------------------------------------------
// 5. `sgt -C <estate> run` from a managed execution can create child Work
//    with validated parent Work/execution causation.
//
// Pinned: `tests/m12_child_work.rs::an_actor_process_submits_child_work_with_validated_causation`
// (a real actor process, real `sgt -C` subprocess, real daemon-side
// journal validation of the claimed parent) and
// `tests/m12_child_work.rs::sgt_run_transports_the_inherited_causation_and_the_daemon_validates_it`
// (the CLI-level half: the three `SERGEANT_*` values really travel end to
// end and the daemon records the relation).
// ------------------------------------------------------------------------

// ------------------------------------------------------------------------
// 6. Child Work has independent scope/surfaces/lifecycle.
//
// Pinned: `tests/m12_child_work.rs::a_child_work_is_independent_of_the_parent_that_caused_it`
// (own branch, own surface, own scope; parent cancellation does not
// cascade — W1-12's cancel half).
// ------------------------------------------------------------------------

// ------------------------------------------------------------------------
// 7. Parent/child completion/cancel/merge do not silently cascade.
//
// Completion and cancellation are pinned by name:
// `tests/m12_child_work.rs::a_parent_completing_does_not_cascade_to_its_child`
// and `tests/m12_child_work.rs::a_child_work_is_independent_of_the_parent_that_caused_it`'s
// own cancellation assertions (W1-12).
//
// Merge has no cascade to pin at the engine level because the engine has no
// merge primitive at all to cascade from: per ADR 0015 (a pull request is a
// request; the merge is the authority boundary, not the artifact) `sgt` has
// no CLI verb and no daemon route that merges a branch — merging is `git`/
// `gh` surface on the branch a Work produces, performed by a human outside
// the engine. The self-contained test below pins that absence directly
// rather than asserting a negative about a feature that doesn't exist.
// ------------------------------------------------------------------------

#[test]
fn w1_acceptance_7_the_engine_has_no_merge_primitive_to_cascade_from() {
    let src = repo_root().join("src");
    let mut offenders = Vec::new();
    for entry in walk_rs_files(&src) {
        let text = std::fs::read_to_string(&entry).unwrap_or_default();
        for needle in [
            "fn merge_work",
            "\"/merge\"",
            ".route(\"/v1/work/:id/merge\"",
        ] {
            if text.contains(needle) {
                offenders.push(format!("{}: {needle}", entry.display()));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "criterion 7 (merge half): the engine must define no merge verb/route for a Work to \
         cascade through — ADR 0015 keeps merging entirely outside the engine. Found: {offenders:?}"
    );
}

fn walk_rs_files(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk_rs_files(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    out
}

// ------------------------------------------------------------------------
// 8. Current exact-root admission and Work surface integrity tests remain
//    green.
//
// Not one named test — this criterion is "the standing admission/surface
// suites still pass", which the wave's own `cargo test --locked` proves
// directly: `tests/e_git_admission.rs`, `tests/e_admission_uses_no_network_git.rs`,
// `tests/e_sweep_uses_only_local_git.rs`, `tests/c4_repo_lock.rs`,
// `tests/c11_injectable_git.rs`. Nothing changed in this wave's diff touches
// admission or worktree binding; the loader/engine/causation work above is
// additive at the stage-scheduling and CLI layers, not the admission path.
// This comment is the checklist entry naming where the proof lives.
// ------------------------------------------------------------------------

// ------------------------------------------------------------------------
// 9. Split-hardening expected-output/required-column/finalize contracts
//    behave identically for nested leaf stages and are not bypassed by
//    container completion.
//
// Item 9 names three contract families: expected-output, required-column,
// and finalize. All three are walked below, each cited by name; none is
// merely implied by another.
//
// Expected-output half — verify BOTH the leaf and container cases are
// actually pinned (the brief's own instruction):
//
// Half A — a nested leaf's own output contract is enforced exactly as a
// flat leaf's is (the leaf-level half):
// `tests/m11_nested_workflow.rs::a_nested_leafs_own_output_contract_is_enforced_exactly_as_a_flat_ones_is`.
//
// Half B — a CONTAINER's own declared output contract is checked at its
// boundary-closing leaf's completion, and is not silently satisfied by the
// container merely finishing its leaves (the container-level half, E4):
// `tests/m11_nested_workflow.rs::a_container_that_never_produced_its_declared_output_parks_naming_the_container`
// (an unmet container contract parks the Work naming the CONTAINER, not the
// leaf) and
// `tests/m11_nested_workflow.rs::a_container_whose_declared_output_is_present_closes_silently`
// (a met one closes without a spurious park). Both halves present.
//
// Required-column half — Amendment 10d's `**Required columns:**` line is
// enforced against a nested leaf's *composed* hierarchical id, not only a
// flat one's, because `check_output_contract` (`src/runtime/engine.rs`)
// runs `has_required_table_columns` against the same `contract_id` the
// expected-output check above already exercises for nesting — one gate,
// shared by both column families:
// `tests/m11_nested_workflow.rs::a_nested_leafs_required_column_contract_is_enforced_exactly_as_a_flat_ones_is`
// (a present-but-untyped artifact at a nested id is refused the identical
// `stage_output_missing`-class way `tests/m3_execution.rs`'s flat-case
// precedent,
// `t11_a_present_but_untyped_declared_artifact_is_refused_the_same_way_as_a_missing_one`,
// pins for a flat one).
//
// Finalize half — E11: `finalize_sweep` (`src/runtime/surface.rs`) walks to
// the same nested depth as the two gates above, copying an evidence-class
// nested leaf's output out and removing it from the worktree, sweeping a
// container's own sibling `output/` in the same pass, and leaving a
// promote-class nested leaf to ship:
// `src/runtime/surface.rs::tests::the_finalize_sweep_reaches_a_nested_leafs_output`.
// ------------------------------------------------------------------------
