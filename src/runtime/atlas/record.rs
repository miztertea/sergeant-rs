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

use std::collections::BTreeMap;

use crate::domain::event::{EventDraft, EventSource};
use crate::domain::source::KIND_SOURCE_SCANNED;
use crate::runtime::atlas::db::{AtlasDb, AtlasError, ScanCommit};
use crate::runtime::atlas::deny::BadPattern;
use crate::runtime::atlas::git::{EstateGitSource, GitScanError, scan_estate_git};
use crate::runtime::atlas::overlay::{WorkOverlay, scan_work_overlay};
use crate::runtime::atlas::scan::{KnowledgeSource, SourceScan, scan_local_knowledge};
use crate::runtime::integrity::EstateDriftObservation;
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
    /// The source root could not be read at all, and a confirmed generation
    /// already stands, so nothing was written and — the point of the
    /// variant — **nothing was evicted**. An unplugged drive changed no
    /// source bytes, and ruling §4 evicts only when they changed. The
    /// unavailability is recorded as coverage against the generation that
    /// survived it.
    RootUnavailable {
        /// The generation that still stands, untouched.
        generation_id: String,
        /// Its content identity.
        content_key: String,
        /// What was recorded as coverage.
        detail: String,
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
    /// An estate-git or overlay scan failed before any row was written.
    #[error(transparent)]
    GitScan(#[from] GitScanError),
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
/// The order is the only safe one, and each boundary is worth stating —
/// including the fact that the two crash windows are **not** the same window:
///
/// * **Crash between 1 and 2** leaves rows with no summary anywhere. Nothing
///   reads them (every read filters to confirmed generations) and
///   [`reconcile_sources`] evicts them at startup, leaving a
///   `generation_evicted` coverage row. Neither-reported.
/// * **Crash between 2 and 3** leaves rows *and* a durable summary naming
///   them — the event is in the journal, and was already broadcast. Evicting
///   here would be the mirror-image violation of the same both-present rule:
///   journal-present, database-evicted, with the eviction row claiming a
///   missing summary that plainly exists. So [`reconcile_sources`] promotes
///   instead, finishing step 3 on the next start. Both-present.
///
///   This is why reconciliation reads the journal rather than the `state`
///   column alone: the column cannot tell the two windows apart, and they
///   have opposite correct answers.
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
    record_scan(db, journal, &scan_local_knowledge(source)?, workspace_id)
}

/// [`scan_and_record`] for one estate-git source: the same three steps, over a
/// commit instead of a directory.
///
/// The steps are shared rather than mirrored, which is the point. F1's crash
/// window is a property of the *order* — stage, journal, confirm — and a
/// second copy of that order for a second source kind would be a second place
/// for it to be got subtly wrong. What differs between the two source kinds is
/// how bytes are acquired, and that difference is entirely upstream of here.
///
/// Returns the [`ScanRecord`] alongside the scan's drift observation, when the
/// mount's HEAD has moved off the pinned SHA. The observation rides beside the
/// record for the same reason it rides beside the scan: it is a fact about the
/// mount, not about the generation, and folding it in would make a clean scan
/// look unclean.
pub fn scan_and_record_estate_git(
    db: &mut AtlasDb,
    journal: &mut Journal,
    source: &EstateGitSource,
    workspace_id: Option<&str>,
) -> Result<(ScanRecord, Option<EstateDriftObservation>), ScanRecordError> {
    let scanned = scan_estate_git(source)?;
    let record = record_scan(db, journal, &scanned.scan, workspace_id)?;
    Ok((record, scanned.drift))
}

/// [`scan_and_record`] for one Work overlay — same three steps again.
///
/// The generation this writes is scoped to its Work and removed by
/// [`AtlasDb::evict_work_overlays`], never by ruling §4's byte-change rule;
/// see [`super::overlay`] for why those are two different lifetimes.
pub fn scan_and_record_overlay(
    db: &mut AtlasDb,
    journal: &mut Journal,
    overlay: &WorkOverlay,
    workspace_id: Option<&str>,
) -> Result<ScanRecord, ScanRecordError> {
    let scanned = scan_work_overlay(overlay)?;
    record_scan(db, journal, &scanned.scan, workspace_id)
}

/// **F1's crash-window coupling itself**, over an already-completed scan of
/// any source kind. See [`scan_and_record`] for the ordering argument — this
/// is the code that argument is about.
pub fn record_scan(
    db: &mut AtlasDb,
    journal: &mut Journal,
    scan: &SourceScan,
    workspace_id: Option<&str>,
) -> Result<ScanRecord, ScanRecordError> {
    let staged = match db.stage_scan(scan)? {
        ScanCommit::Unchanged {
            generation_id,
            content_key,
        } => {
            return Ok(ScanRecord::Unchanged {
                generation_id,
                content_key,
            });
        }
        ScanCommit::RootUnavailable {
            generation_id,
            content_key,
            detail,
        } => {
            return Ok(ScanRecord::RootUnavailable {
                generation_id,
                content_key,
                detail,
            });
        }
        ScanCommit::Staged { generation_id } => generation_id,
    };
    let mut draft = EventDraft::new(
        EventSource::new("daemon", "atlas"),
        KIND_SOURCE_SCANNED,
        scan_summary(scan, &staged),
    );
    if let Some(workspace_id) = workspace_id {
        draft = draft.with_workspace_id(workspace_id);
    }
    let event = journal.append(draft)?;
    let evicted = db.confirm_scan(&staged, &event.id)?;
    Ok(ScanRecord::Recorded {
        generation_id: staged,
        content_key: scan.content_key.clone(),
        summary_event_id: event.id,
        evicted,
    })
}

/// What one startup reconciliation resolved.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reconciliation {
    /// Generations finished on this start: their summary was in the journal,
    /// so step 3 ran now instead of before the crash.
    pub promoted: Vec<String>,
    /// Generations evicted: no summary anywhere, so the scan never completed.
    pub evicted: Vec<String>,
}

impl Reconciliation {
    /// Whether anything at all needed resolving.
    pub fn is_empty(&self) -> bool {
        self.promoted.is_empty() && self.evicted.is_empty()
    }
}

/// The detail a crash-window eviction leaves on its coverage row.
const NO_SUMMARY: &str =
    "reconciled at startup: rows exist with no source.scanned summary in the journal";

/// **F1's startup reconciliation** — the whole of it, in the one place that
/// can see both halves of the evidence.
///
/// [`scan_and_record`]'s three steps have two crash windows with opposite
/// correct answers, and the database cannot tell them apart on its own: a
/// generation left `provisional` either never got its `source.scanned`
/// summary (step 1→2 — evict; the scan did not complete) or got it and died
/// before the confirming transaction (step 2→3 — promote; the summary is
/// durable, and was already broadcast, so evicting would leave the journal
/// claiming a scan the database had thrown away). The `state` column records
/// the same value in both cases. The journal is what distinguishes them, so
/// the journal is what this consults.
///
/// Cost: **nothing at all** on a start that did not follow a crash.
/// [`AtlasDb::provisional_generations`] is a single indexed-free read of a
/// small table and answers empty on every ordinary start; the journal is only
/// read when it does not.
///
/// **The read is floor-aware** ([`Journal::replay_from_floor`], I-W3-10): on
/// a pruned journal the strict primitive would report a healthy floor above 1
/// as a sequence discontinuity. A summary old enough to have been pruned away
/// is *itself* the answer, and it is eviction: once the trail is gone, no
/// both-present state is reachable for that generation, and the safe
/// direction — nothing half-reported as coverage, with an explicit eviction
/// row saying so — is the one that stays true. In practice a provisional
/// generation is from the run that just died, so its summary is at the tail.
pub fn reconcile_sources(
    db: &mut AtlasDb,
    journal: &Journal,
) -> Result<Reconciliation, ScanRecordError> {
    let awaiting = db.provisional_generations()?;
    if awaiting.is_empty() {
        return Ok(Reconciliation::default());
    }
    // Which of them the journal can vouch for. One pass, and only ever
    // reached because something was actually left unresolved.
    let mut summaries: BTreeMap<String, String> = BTreeMap::new();
    for event in journal.replay_from_floor()? {
        let event = event?;
        if event.kind != KIND_SOURCE_SCANNED {
            continue;
        }
        if let Some(generation) = event.payload.get("generation").and_then(|g| g.as_str()) {
            summaries.insert(generation.to_string(), event.id.clone());
        }
    }

    let mut out = Reconciliation::default();
    let mut orphaned = Vec::new();
    for (generation_id, _) in awaiting {
        match summaries.get(&generation_id) {
            // Step 3, finished late. `confirm_scan` promotes and evicts the
            // predecessor in one transaction, exactly as the live path would
            // have — a recovered scan is a completed scan, not a special one.
            Some(event_id) => {
                db.confirm_scan(&generation_id, event_id)?;
                out.promoted.push(generation_id);
            }
            None => orphaned.push(generation_id),
        }
    }
    out.evicted = db.evict_provisional(&orphaned, NO_SUMMARY)?;
    Ok(out)
}

/// The one compact summary a completed scan journals (F1).
///
/// Source, generation, counts, extractor identities — and deliberately not a
/// path list. Coverage detail belongs in the table that can be queried; the
/// journal carries the authoritative *trail* (that this scan happened, over
/// this world, producing this shape), not a second copy of the data.
///
/// Public because the payload is a **contract, not an implementation
/// detail**: `generation` is the field [`reconcile_sources`] matches a
/// crash-window generation on, so the writer and the reader of that field
/// have to be pinnable together. A test that hand-rolled its own summary
/// would keep passing on the day this function stopped emitting it.
pub fn scan_summary(scan: &SourceScan, generation_id: &str) -> serde_json::Value {
    let mut summary = serde_json::json!({
        "source": scan.source_name,
        "source_kind": scan.kind.as_str(),
        "authority_class": scan.authority.as_str(),
        "generation": generation_id,
        "content_key": scan.content_key,
        "observed_at": scan.observed_at,
        "files": scan.files.len(),
        "units": scan.unit_count(),
        // Symbol *sites* and edges, matching what was written row for row —
        // not the deduplicated symbol index, which is a rollup of these and
        // would make the trail disagree with the table it describes (X3b).
        "symbols": scan.symbol_count(),
        "edges": scan.edge_count(),
        "coverage": scan.counts(),
        "extractors": scan.extractors.iter().collect::<Vec<_>>(),
    });
    // Added only when the source actually has a revision (a pinned commit
    // SHA). An explicit `null` would read as "this source has a revision and
    // we do not know it", which is a different claim and a false one for a
    // filesystem walk.
    if let Some(revision) = &scan.revision
        && let Some(object) = summary.as_object_mut()
    {
        object.insert("revision".to_string(), revision.as_str().into());
    }
    summary
}
