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
    let journal = Journal::open(data).expect("journal");
    let mut registry = work_registry_projection();
    registry
        .catch_up(journal.replay().expect("replay"))
        .expect("catch up");
    let (events_tx, _) = broadcast::channel(64);
    Core::new(journal, registry, events_tx)
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
