//! The thin DB/journal glue around the scanner (F6).
//!
//! This module is deliberately small and deliberately its own file. F6's
//! adapter-shape mandate is that extraction is a pure function over bytes and
//! that "DB-touching glue stays thin and separately reviewable" — so the glue
//! is a file you can read in one sitting, sitting *above* both halves it
//! joins:
//!
//! ```text
//! deny, text  pure predicates and pure functions over bytes
//!    -> scan  the walk: filesystem in, plain Rust out; no DB, no journal
//!       -> db the one owner of Atlas's database file; plain Rust in and out
//!          -> record  this file: three ordered steps, and nothing else
//! ```
//!
//! The arrows run one way. Nothing below depends on anything above it, so
//! there is no cycle for a shortcut to hide in — the scanner cannot quietly
//! grow a database handle, and the database cannot quietly grow a filesystem
//! walk.

use crate::domain::event::{EventDraft, EventSource};
use crate::domain::source::KIND_SOURCE_SCANNED;
use crate::runtime::atlas::db::{AtlasDb, AtlasError, ScanCommit};
use crate::runtime::atlas::deny::BadPattern;
use crate::runtime::atlas::scan::{KnowledgeSource, SourceScan, scan_local_knowledge};
use crate::runtime::journal::{Journal, JournalError};

/// What one call to [`scan_and_record`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanRecord {
    /// The source's bytes are exactly what the newest confirmed generation
    /// was derived from, so nothing was written and nothing was evicted
    /// (ruling §4: a generation is evicted only when source bytes changed).
    Unchanged {
        /// The generation that still stands.
        generation_id: String,
        /// Its content identity.
        content_key: String,
    },
    /// A new generation was written, journaled and confirmed.
    Recorded {
        /// The new generation.
        generation_id: String,
        /// Its content identity.
        content_key: String,
        /// Id of the `source.scanned` event that completes it.
        summary_event_id: String,
        /// The generation this one superseded, if any — evicted in the same
        /// transaction that confirmed the new one, leaving a
        /// `generation_evicted` coverage row.
        evicted: Option<String>,
    },
}

/// Failures of a scan-and-record.
#[derive(Debug, thiserror::Error)]
pub enum ScanRecordError {
    /// A `[[knowledge]] ignore` glob does not compile.
    #[error(transparent)]
    Pattern(#[from] BadPattern),
    /// Atlas refused a write.
    #[error(transparent)]
    Atlas(#[from] AtlasError),
    /// The journal refused the summary.
    #[error(transparent)]
    Journal(#[from] JournalError),
}

/// Scan one source and record it — **F1's crash-window coupling, in three
/// ordered steps.**
///
/// ```text
/// 1. stage    one transaction: generation (provisional) + files + units + coverage
/// 2. journal  one compact `source.scanned` summary
/// 3. confirm  one transaction: mark confirmed, evict the superseded generation
/// ```
///
/// The order is the only safe one, and each boundary is worth stating:
///
/// * **Crash between 1 and 2, or between 2 and 3** leaves a generation with
///   rows and no confirmation. Nothing reads it — every read filters to
///   confirmed generations — and startup reconciliation
///   ([`AtlasDb::reconcile`]) evicts it, leaving a `generation_evicted`
///   coverage row. Neither-reported, never half-reported.
/// * **The eviction of the superseded generation is in step 3, not step 1.**
///   Putting it in the staging transaction would mean a crash before the
///   summary destroys a perfectly good previous generation and leaves
///   nothing in its place. Deferred to the confirming transaction, a crash
///   anywhere leaves the *old* generation standing, which is the answer that
///   still has evidence behind it.
/// * **Step 3 is one transaction**, so "confirmed" and "predecessor evicted"
///   are never separately observable.
pub fn scan_and_record(
    db: &mut AtlasDb,
    journal: &mut Journal,
    source: &KnowledgeSource,
    workspace_id: Option<&str>,
) -> Result<ScanRecord, ScanRecordError> {
    let scan = scan_local_knowledge(source)?;
    let staged = match db.stage_scan(&scan)? {
        ScanCommit::Unchanged {
            generation_id,
            content_key,
        } => {
            return Ok(ScanRecord::Unchanged {
                generation_id,
                content_key,
            });
        }
        ScanCommit::Staged { generation_id } => generation_id,
    };
    let mut draft = EventDraft::new(
        EventSource::new("daemon", "atlas"),
        KIND_SOURCE_SCANNED,
        summary_payload(&scan, &staged),
    );
    if let Some(workspace_id) = workspace_id {
        draft = draft.with_workspace_id(workspace_id);
    }
    let event = journal.append(draft)?;
    let evicted = db.confirm_scan(&staged, &event.id)?;
    Ok(ScanRecord::Recorded {
        generation_id: staged,
        content_key: scan.content_key,
        summary_event_id: event.id,
        evicted,
    })
}

/// The one compact summary a completed scan journals (F1).
///
/// Source, generation, counts, extractor identities — and deliberately not a
/// path list. Coverage detail belongs in the table that can be queried; the
/// journal carries the authoritative *trail* (that this scan happened, over
/// this world, producing this shape), not a second copy of the data.
fn summary_payload(scan: &SourceScan, generation_id: &str) -> serde_json::Value {
    serde_json::json!({
        "source": scan.source_name,
        "source_kind": scan.kind.as_str(),
        "authority_class": scan.authority.as_str(),
        "generation": generation_id,
        "content_key": scan.content_key,
        "observed_at": scan.observed_at,
        "files": scan.files.len(),
        "units": scan.unit_count(),
        "coverage": scan.counts(),
        "extractors": scan.extractors.iter().collect::<Vec<_>>(),
    })
}
