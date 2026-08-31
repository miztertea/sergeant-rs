//! #334 — the journal's fold is not allowed to fall behind the journal.
//!
//! **The contract this suite defends (J5).** `H1-HOST-RUNTIME.md` §6: *"The
//! journal remains the authoritative evidence stream for what Sergeant can
//! prove about its own execution."* `A1-ATLAS-WORLD-INTELLIGENCE.md` A1-01:
//! *"Atlas is derived evidence; journal/Git/original bytes remain
//! authority."* Every acceptance item of the form "the journal records X" is
//! unprovable while a write can land in the journal and never reach the
//! projections that answer for it.
//!
//! **The defect** (`00-orient`, this wave). [`Core::commit`] is the only path
//! that appends *and* folds. Atlas's `record_scan` cannot use it — F1's
//! crash-window coupling puts the `source.scanned` append strictly between
//! staging and confirming a generation — so it takes `&mut Journal` and the
//! registry is left behind. [`Core::absorb_journaled`] is the catch-up that
//! closes that, and its doc comment says *"every direct-journal writer must
//! call this before releasing the hold"* — but that was **prose**, and
//! `intelligence_add_source` (`POST /v1/intelligence/sources`) did not call
//! it. The next `Core::commit` then fails `Projection::apply`'s contiguity
//! check on whatever command happened to come next, and keeps failing.
//!
//! What this suite pins:
//!
//! * [`a_hold_that_appended_directly_without_absorbing_does_not_wedge_the_next_commit`]
//!   — the invariant is enforced where every hold ends, not where each writer
//!   remembers.
//! * [`a_hold_that_skipped_absorb_journaled_is_counted_as_a_breach`] /
//!   [`a_hold_that_absorbed_records_no_breach`] — the repair does not excuse
//!   the writer. The breach is a number a test can fail on, not a log line
//!   nothing reads. This pair is also the non-vacuity proof for the guard:
//!   the first test *is* a writer that skips `absorb_journaled`, and the
//!   second is the identical fixture that does not.
//! * [`every_direct_journal_writer_in_the_api_absorbs_before_releasing_its_hold`]
//!   — the source-level half. The runtime backstop above keeps a forgetful
//!   writer from wedging the daemon; this one keeps the writer from being
//!   written. `tests/x5_a1a_acceptance.rs`'s
//!   `the_trigger_is_reachable_from_production_code_not_only_a_test_module`
//!   is the in-repo precedent for reading `src/api.rs` as text to pin a
//!   cross-cutting property no single call site can state (R2).
//! * [`the_wedged_cascade_from_the_issue_is_recovered_when_the_hold_releases`]
//!   — #334's *first* observed shape (`expected 146, got 149`). `00-orient`
//!   §3a: that is not three concurrent appenders, it is three prior failed
//!   commits after one un-absorbed write, because `Core::commit` appends
//!   before it folds and so widens the gap by one every time. This pins the
//!   recovery, not the widening: the widening is inside `00-orient` §7's
//!   escalated J0 and is deliberately not made a contract here.

use std::fs;
use std::path::Path;

use serde_json::json;
use tempfile::TempDir;
use tokio::sync::broadcast;

use sergeant_rs::api::Core;
use sergeant_rs::domain::event::{EventDraft, EventSource};
use sergeant_rs::runtime::atlas::db::AtlasDb;
use sergeant_rs::runtime::atlas::record::record_scan;
use sergeant_rs::runtime::atlas::scan::{KnowledgeSource, scan_local_knowledge};
use sergeant_rs::runtime::journal::Journal;
use sergeant_rs::runtime::projection::work_registry_projection;

/// A `Core` over a real journal in `data`, folded exactly as the daemon
/// folds one at start.
fn build_core(data: &Path) -> Core {
    core_and_subscriber(data).0
}

/// The same `Core`, with a live SSE subscriber attached — `00-orient` §3b's
/// distinction is that the failure loses the *fold and the publish*, not the
/// append, so a test about it has to be able to see what was published.
fn core_and_subscriber(
    data: &Path,
) -> (Core, broadcast::Receiver<sergeant_rs::domain::event::Event>) {
    let journal = Journal::open(data).expect("journal");
    let mut registry = work_registry_projection();
    registry
        .catch_up(journal.replay().expect("replay"))
        .expect("catch up");
    let (events_tx, events_rx) = broadcast::channel(64);
    (Core::new(journal, registry, events_tx), events_rx)
}

fn draft(n: u32) -> EventDraft {
    EventDraft::new(
        EventSource::new("test", "f334"),
        "f334.probe",
        json!({"n": n}),
    )
}

/// A real `source.scanned` append straight through `&mut Journal` — the
/// production shape (`record_scan`), not a hand-rolled `journal.append`, so
/// the test cannot pass by imitating the writer instead of using it.
fn append_a_scan_directly(core: &mut Core, data: &Path, src_root: &Path) {
    let mut atlas = AtlasDb::open(data).expect("atlas");
    let source = KnowledgeSource {
        name: "k".into(),
        root: src_root.to_path_buf(),
        ignore: Vec::new(),
        context_fields: Default::default(),
    };
    let scan = scan_local_knowledge(&source).expect("scan");
    record_scan(&mut atlas, &mut core.journal, &scan, None).expect("record_scan");
}

fn fixture() -> (TempDir, std::path::PathBuf, std::path::PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let data = dir.path().join("data");
    fs::create_dir_all(&data).expect("data dir");
    let src_root = dir.path().join("knowledge");
    fs::create_dir_all(&src_root).expect("src root");
    fs::write(src_root.join("a.md"), "# hello\n\nbody\n").expect("write");
    (dir, data, src_root)
}

/// **The #334 regression.** A hold appends through `&mut Journal` and does
/// not call `absorb_journaled`. Releasing that hold must leave the registry
/// caught up anyway: the next mutation — any `submit`, `cancel`, `input`,
/// `daemon.stopped` or `backend.probed`, "whatever Work command happened to
/// come next" — succeeds.
///
/// Before the fix this fails with `projection seq mismatch: expected 2, got
/// 3`, and every later commit fails too (`00-orient` §3a: `expected` pins,
/// `got` climbs — the issue's own `expected 146, got 149` shape).
#[test]
fn a_hold_that_appended_directly_without_absorbing_does_not_wedge_the_next_commit() {
    let (_dir, data, src_root) = fixture();
    let mut core = build_core(&data);
    core.commit(draft(1)).expect("first commit");

    append_a_scan_directly(&mut core, &data, &src_root);
    // The hold ends here. `CoreGuard::drop` and `CoreGuard::flush` both go
    // through exactly this call, so this IS the release path.
    core.flush().expect("the hold releases");

    core.commit(draft(2))
        .expect("the next commit must not be wedged by a writer that forgot to absorb");
    core.flush().expect("second hold releases");
}

/// The repair at the release point is a backstop, **not** absolution: a
/// writer that skipped [`Core::absorb_journaled`] is counted, so a guard can
/// fail on the breach instead of on a `tracing::error!` line nothing reads.
///
/// This test is itself "a writer that skips it" — the non-vacuity proof the
/// brief asks for. Delete the counting and this goes red; delete the skip
/// (see the control below) and it goes red the other way.
#[test]
fn a_hold_that_skipped_absorb_journaled_is_counted_as_a_breach() {
    let (_dir, data, src_root) = fixture();
    let mut core = build_core(&data);
    core.commit(draft(1)).expect("first commit");
    assert_eq!(core.unabsorbed_holds(), 0, "a clean hold is not a breach");

    append_a_scan_directly(&mut core, &data, &src_root);
    core.flush().expect("the hold releases");

    assert_eq!(
        core.unabsorbed_holds(),
        1,
        "releasing a hold with the registry behind the journal is a breach of \
         `absorb_journaled`'s contract and must be visible as one"
    );
}

/// The control: the identical fixture, with the one call the writer above
/// omitted. Nothing is counted, because nothing was breached — which is what
/// makes the test above a measurement of the absorb and not of the fixture.
#[test]
fn a_hold_that_absorbed_records_no_breach() {
    let (_dir, data, src_root) = fixture();
    let mut core = build_core(&data);
    core.commit(draft(1)).expect("first commit");

    append_a_scan_directly(&mut core, &data, &src_root);
    core.absorb_journaled()
        .expect("the writer absorbs, as it must");
    core.flush().expect("the hold releases");

    assert_eq!(
        core.unabsorbed_holds(),
        0,
        "a writer that folds what it appended breaches nothing"
    );
    core.commit(draft(2)).expect("and the next commit is fine");
}

// --------------------------------------------- the source-level half

/// **The invariant, made unforgettable at the point it is written.**
///
/// `Core::absorb_journaled`'s doc comment says: *"Every direct-journal writer
/// must call this before releasing the hold."* That sentence was prose, and
/// `intelligence_add_source` was written without it — which is #334. This
/// reads `src/api.rs`'s production region and requires that every function
/// which hands `&mut …journal` to an Atlas writer also folds what it
/// appended, so the next writer cannot be added the same way.
///
/// Not a substitute for the runtime backstop in `Core::flush`, and not
/// substituted by it: this one fails in review, that one holds in
/// production. Both are needed because a source guard is evadable (a writer
/// could take the journal by another name) and a runtime backstop is silent
/// (it repairs).
#[test]
fn every_direct_journal_writer_in_the_api_absorbs_before_releasing_its_hold() {
    let api = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/api.rs"))
        .expect("read src/api.rs");
    // The real test-module boundary, the same way x5_a1a_acceptance finds it.
    let test_module = api
        .find("\nmod tests {")
        .expect("src/api.rs must have a test module for this boundary to mean anything");

    let lines: Vec<&str> = api.lines().collect();
    let is_fn_line = |line: &str| {
        let t = line.trim_start();
        t.starts_with("fn ")
            || t.starts_with("async fn ")
            || t.starts_with("pub fn ")
            || t.starts_with("pub async fn ")
            || t.starts_with("pub(crate) fn ")
            || t.starts_with("pub(crate) async fn ")
    };

    // Byte offset of the start of each line, so a hit can be placed relative
    // to the test module.
    let mut offsets = Vec::with_capacity(lines.len());
    let mut at = 0usize;
    for line in &lines {
        offsets.push(at);
        at += line.len() + 1;
    }

    let mut writers = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if offsets[i] >= test_module {
            break;
        }
        // A direct-journal writer: this line hands a `&mut …journal` to
        // something. `self.journal.append(...)` inside `Core::commit` is not
        // one — that is the append-AND-fold path.
        let Some(before) = line.split(".journal").next() else {
            continue;
        };
        if !line.contains(".journal")
            || !before
                .trim_end()
                .ends_with(|c: char| c.is_alphanumeric() || c == '_')
        {
            continue;
        }
        if !before.contains("&mut ") {
            continue;
        }
        // Walk back to the enclosing function, then forward to the next one.
        let start = (0..=i).rev().find(|&j| is_fn_line(lines[j])).unwrap_or(0);
        let end = ((i + 1)..lines.len())
            .find(|&j| is_fn_line(lines[j]))
            .unwrap_or(lines.len());
        let name = lines[start].trim().to_string();
        let body = lines[start..end].join("\n");
        if !body.contains("absorb_journaled") {
            writers.push(format!("{} (line {})", name, i + 1));
        }
    }

    assert!(
        writers.is_empty(),
        "these functions in src/api.rs append straight through `&mut …journal` and never \
         call `Core::absorb_journaled`, so they release the hold with the registry behind \
         the journal and wedge the next commit (#334): {writers:#?}"
    );
}

/// **#334's first observed shape, and its recovery.** The issue reports
/// `projection seq mismatch: expected 146, got 149` — a gap of three. Per
/// `00-orient` §3a that is one un-absorbed direct write followed by *three
/// failed commits*, each of which appended before its fold was refused.
///
/// What this test pins is the recovery, and only the recovery: however wide
/// the gap got, the hold's release leaves the journal and the registry in
/// agreement, every event reaches the live subscribers that were owed it,
/// and the next command succeeds. It deliberately does **not** assert that a
/// failed commit appends — `Core::commit`'s append-before-fold ordering is
/// inside the J0 `00-orient` §7 escalated and unresolved, and pinning it in
/// a test would settle by accident a contract question nobody has ruled.
#[test]
fn the_wedged_cascade_from_the_issue_is_recovered_when_the_hold_releases() {
    let (_dir, data, src_root) = fixture();
    let (mut core, mut events) = core_and_subscriber(&data);
    core.commit(draft(1)).expect("first commit");
    core.flush().expect("first hold releases");

    append_a_scan_directly(&mut core, &data, &src_root);
    // Whatever Work commands happened to come next, while the registry was
    // behind. Each one is refused; the daemon is wedged for every mutation.
    for n in 2..=4u32 {
        core.commit(draft(n))
            .expect_err("a command issued while the registry is behind is refused");
    }

    core.flush().expect("the hold releases");

    assert_eq!(
        core.registry.last_seq(),
        core.journal.next_seq() - 1,
        "after the hold releases, the projections must answer for every event the journal \
         holds — H1 §6: the journal is the authority for what Sergeant can prove"
    );

    // §3b: the loss was the fold and the publish. Every event the journal
    // holds past the first flush must have reached the live subscriber too,
    // not merely the file.
    let mut published = Vec::new();
    while let Ok(event) = events.try_recv() {
        published.push(event.seq);
    }
    let journaled: Vec<u64> = core
        .journal
        .replay()
        .expect("replay")
        .map(|e| e.expect("event").seq)
        .collect();
    assert_eq!(
        published, journaled,
        "every journaled event must also have been published; the journal and the surfaces \
         that read it are not allowed to disagree"
    );

    core.commit(draft(5))
        .expect("and the daemon is no longer wedged");
}
