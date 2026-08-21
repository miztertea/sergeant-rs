//! W3 §13.1 step 8: the crash-window recovery proofs (N7–N10) plus the
//! recovery-ordering guarantee `daemon::start_with` depends on.
//!
//! F1 (before quarantine), F2 (mid-quarantine), F3 (mid-deferred-delete),
//! F4 (mid-unlink) and F5 (after unlink, before completion) each have a
//! dedicated, named unit test — `crash_window_f1_before_quarantine_
//! completes_at_next_start` through `crash_window_f5_after_unlink_
//! appends_the_completion_from_the_intents_residue` — in
//! `src/runtime/prune.rs`'s own `#[cfg(test)]` module, beside
//! `complete_interrupted` itself: they need `Core`/`prune`'s private test
//! helpers (`tiny_core`, `commit`, `build_and_commit_intent`) to simulate
//! each crash point precisely, which only that module can reach directly.
//! This file carries the one crash-window proof that is genuinely about
//! `daemon::start_with`'s own call *order*, not about `complete_interrupted`
//! in isolation.

use std::path::PathBuf;

/// The end of the `{ … }` block that starts at or after `from` (same
/// helper `tests/m6_surfaces.rs`/`tests/a1_floor_awareness.rs` use).
fn block_end(source: &str, from: usize) -> usize {
    let open = source[from..].find('{').expect("a block") + from;
    let mut depth = 0usize;
    for (offset, c) in source[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return open + offset;
                }
            }
            _ => {}
        }
    }
    source.len()
}

/// An interrupted prune is completed *before* `recovery::reconcile` runs
/// (§6.6/§10.3): the completion's fold removes every pruned Work from
/// `works`/`runs`, and every pruned Work is retired whole (I-W3-3), so
/// reconcile would have found nothing to do for them either way — but the
/// ordering is what makes that true by construction rather than by luck.
/// Reversed, a Work whose prune intent is still pending could be visited
/// by `recovery::reconcile` while its own registry rows still look
/// unsettled, doing recovery work (a teardown, a surface reconciliation)
/// for a Work that is about to be deleted out from under it.
///
/// Structural, not behavioral: the daemon-level behavioral case (a Work
/// that is simultaneously mid-prune-completion and reconcile-eligible) is
/// vanishingly narrow to construct through the real daemon (it requires a
/// crash frozen between two specific fsyncs), and the ordering itself is
/// simple enough that a source-level proof — `complete_interrupted` is
/// textually called before `recovery::reconcile` inside the same
/// `start_with` function body — is the honest, maintainable version of
/// this proof. A future reordering fails here immediately, with no need
/// to first construct the narrow crash window that would otherwise be the
/// only way to observe it.
#[test]
fn an_interrupted_prune_is_completed_before_recovery_reconciles() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("daemon.rs");
    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));

    let fn_start = source
        .find("pub async fn start_with(")
        .expect("daemon::start_with must exist at this signature");
    let fn_end = block_end(&source, fn_start);
    let body = &source[fn_start..fn_end];

    let complete_interrupted_at = body
        .find("crate::runtime::prune::complete_interrupted(")
        .unwrap_or_else(|| {
            panic!(
                "start_with must call crate::runtime::prune::complete_interrupted — it moved, \
                 was renamed, or this scan's pattern needs updating"
            )
        });
    let reconcile_at = body.find("recovery::reconcile(").unwrap_or_else(|| {
        panic!(
            "start_with must call crate::runtime::recovery::reconcile — it moved, was renamed, \
             or this scan's pattern needs updating"
        )
    });

    assert!(
        complete_interrupted_at < reconcile_at,
        "prune::complete_interrupted must be called before recovery::reconcile inside \
         daemon::start_with — a pending prune must be finished before recovery gets a chance \
         to act on a Work that intent is about to delete"
    );
}
