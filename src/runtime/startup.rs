//! W2: the daemon's single shared startup pass, its window, and the
//! FloorState cache (`docs/proposals/journal-archival-rule-c.md`, the Q1–Q10
//! rulings record).
//!
//! Today `start_with` walks the whole journal four times (`next_seq`, the
//! Work registry, the analytical projection, the claude capability
//! watermark). This module collapses that to **one** shared pass over one
//! `Replay` iterator, fed to every consumer through a small [`ReplaySink`]
//! trait, and makes that one pass *windowed*: it reads only the newest
//! [`STARTUP_WINDOW_SEGMENTS`] segments / [`STARTUP_WINDOW_BYTES`] by seeding
//! the Work registry from a persisted, purely-derived summary of everything
//! older — the [`FloorState`] cache.
//!
//! The cache is a cache in the strict sense: absent, stale, or
//! hash-mismatched, it is discarded and a full replay happens instead, never
//! an error. Nothing is deleted, no floor moves above segment 1 in any
//! production path — the floor-awareness and the cache format exist here so
//! that W3's first deletion code path lands into a repo whose build already
//! fails when the cache misses registry state or the blob mark scan misses a
//! ref.
//!
//! ## Forward compatibility (`sergeant.floor-state.v1`)
//!
//! 1. No `deny_unknown_fields`, anywhere in this file's types: a reader
//!    ignores keys it does not know, at every level (§20's stance applied to
//!    the cache).
//! 2. Additive-only within a version: new fields are `Option<T>` or carry
//!    `#[serde(default)]`. A field is never removed, renamed, or repurposed
//!    inside `v1`.
//! 3. The schema string bumps only when a v1 reader would be *wrong*, not
//!    merely incomplete — an unrecognized schema rebuilds rather than errors,
//!    so a version bump is always safe in both directions.
//! 4. W3's residue slots exist in v1 and are written honestly by W2:
//!    [`FloorBinding::floor_seq`] is the real first seq of the oldest
//!    surviving segment (1 today, never hardcoded); [`FloorWorkRow::pruned_at`]
//!    is defined and always `None` in W2.
//! 5. The cache never carries anything the journal cannot reproduce — every
//!    field is a fold of journal events.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::backend::claude::{AskWithdrawal, note_ask_withdrawal, note_carried_ask_withdrawal};
use crate::domain::event::Event;
use crate::domain::work::{
    KIND_COMMAND_ACCEPTED, KIND_COMMAND_REJECTED, KIND_WORK_SUBMITTED, WorkState,
};
use crate::runtime::analytics::{self, AnalyticsError, AnalyticsFold};
use crate::runtime::fsutil::{create_dir_all_durable, write_atomic};
use crate::runtime::integrity::IntegrityDisposition;
use crate::runtime::journal::{Journal, JournalError, Replay, SegmentBound};
use crate::runtime::projection::{
    Projection, ProjectionError, WorkIndexRow, WorkRegistry, work_registry_projection,
    work_registry_reducer,
};

/// File name of the startup cache inside `projections/`.
pub const FLOOR_STATE_FILE: &str = "floor-state.json";
/// Schema identifier for the startup cache.
///
/// W3 bumps this to v2 (§9.1, ratify-at-review item 5): `FloorWorkRow` gains
/// `first_seq` (without which pruning is inert on a cache-hit start — see
/// its own doc) and drops `pruned_at` (a declared-but-permanently-`None`
/// slot on a *retained* Work's row once W3's actual pruned Works get their
/// own `FloorState::pruned_works` section). A v1 reader would be *wrong* to
/// prune from a v1 cache, not merely incomplete, which is what licenses the
/// bump rather than an additive-only extension within v1.
pub const FLOOR_STATE_SCHEMA: &str = "sergeant.floor-state.v2";

/// Q4: a **fixed** live window on every host — 16 segments or 128 MiB,
/// whichever bounds first. Not configurable, not adaptive: the
/// defaults-argued-for-platforms rule, and `sgt doctor` reports the measured
/// rebuild time so the constant is re-argued from evidence rather than
/// silently tuned per machine.
pub const STARTUP_WINDOW_SEGMENTS: usize = 16;
/// See [`STARTUP_WINDOW_SEGMENTS`].
pub const STARTUP_WINDOW_BYTES: u64 = 128 * 1024 * 1024;

/// Chunk size the per-segment BLAKE3 binding check streams at a time —
/// never `fs::read` a whole segment into memory.
const HASH_CHUNK: usize = 64 * 1024;

/// Errors from the shared startup pass and the FloorState cache's write side.
///
/// Deliberately **no cache-read error variant**: every read fault (§2.4) is a
/// [`CacheMiss`], never an error. A cache-write fault is an error, because by
/// then the state is already correct and a silently unwritten cache would
/// make the next start slow for a reason nobody can see.
#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    /// The journal failed during the shared pass.
    #[error(transparent)]
    Journal(#[from] JournalError),
    /// The Work registry sink refused an event.
    #[error(transparent)]
    Projection(#[from] ProjectionError),
    /// The analytics sink failed to fold or write.
    #[error(transparent)]
    Analytics(#[from] AnalyticsError),
    /// The startup cache could not be written (or removed).
    #[error("startup cache write failed: {0}")]
    CacheWrite(#[from] std::io::Error),
}

/// One consumer of the single shared startup replay.
///
/// `push` is called once per event, in strict ascending seq order, for every
/// event the pass yields. A sink that only wants part of the range filters
/// inside `push` (see [`AnalyticsSink`]).
pub trait ReplaySink {
    /// Fold one event.
    fn push(&mut self, event: &Event) -> Result<(), StartupError>;
    /// Called once after the last event, only if every `push` succeeded.
    fn finish(&mut self) -> Result<(), StartupError> {
        Ok(())
    }
}

/// What the shared pass actually did.
#[derive(Debug, Clone, Default)]
pub struct PassReport {
    /// Events the pass actually yielded (the number `sgt doctor` reads off
    /// `daemon.started`).
    pub replayed_events: u64,
    /// Highest seq the pass saw, 0 if it saw none.
    pub max_seq: u64,
    /// Seq the pass started expecting (the floor, or `H + 1` on a cache hit).
    ///
    /// Best-effort from the pass alone: the first event's own seq, when the
    /// pass saw at least one. A pass that sees **zero** events (a cache-hit
    /// restart with nothing appended since the cache was written) cannot
    /// recover this from the event stream at all — a caller that knows the
    /// intended start (`Plan::from_seq`) is expected to set this field
    /// explicitly, which is what [`crate::daemon::start_with`] does.
    pub from_seq: u64,
}

/// Drive `events` through every sink exactly once, in order.
///
/// Fails closed on the first error from the replay or any sink: a startup
/// that cannot rebuild its state must not serve.
pub fn drive(
    events: impl IntoIterator<Item = Result<Event, JournalError>>,
    sinks: &mut [&mut dyn ReplaySink],
) -> Result<PassReport, StartupError> {
    let mut report = PassReport::default();
    for event in events {
        let event = event?;
        if report.replayed_events == 0 {
            report.from_seq = event.seq;
        }
        report.replayed_events += 1;
        report.max_seq = event.seq;
        for sink in sinks.iter_mut() {
            sink.push(&event)?;
        }
    }
    for sink in sinks.iter_mut() {
        sink.finish()?;
    }
    Ok(report)
}

// ---------------------------------------------------------------------
// The sinks
// ---------------------------------------------------------------------

/// Feeds the shared pass into the live Work registry projection.
pub struct RegistrySink<'a>(pub &'a mut Projection<WorkRegistry>);

impl ReplaySink for RegistrySink<'_> {
    fn push(&mut self, event: &Event) -> Result<(), StartupError> {
        self.0.apply(event)?;
        Ok(())
    }
}

/// Feeds the shared pass into the analytical (DuckDB) projection — but only
/// the window, never the whole pass: `from_seq` here is the **window
/// boundary `W_seq`**, not the cache horizon `H` and not the pass's own
/// start. DuckDB's contents are therefore a pure function of `(W_seq, tail]`
/// whether this start hit the cache or did a full replay — a cache-miss
/// start must not populate more history than a cache-hit start, or
/// `tests/m5_projections.rs`'s "delete the file, restart, identical
/// results" acceptance would be flaky-by-design.
pub struct AnalyticsSink<'a> {
    fold: Option<AnalyticsFold<'a>>,
    from_seq: u64,
}

impl<'a> AnalyticsSink<'a> {
    /// Wrap an open analytics fold, filtering to events past `from_seq`
    /// (the window boundary).
    pub fn new(fold: AnalyticsFold<'a>, from_seq: u64) -> Self {
        Self {
            fold: Some(fold),
            from_seq,
        }
    }
}

impl ReplaySink for AnalyticsSink<'_> {
    fn push(&mut self, event: &Event) -> Result<(), StartupError> {
        if event.seq > self.from_seq
            && let Some(fold) = self.fold.as_mut()
        {
            fold.push(event)?;
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<(), StartupError> {
        if let Some(fold) = self.fold.take() {
            fold.finish()?;
        }
        Ok(())
    }
}

/// Folds the claude ask-grammar-withdrawal watermark (recon correction 3):
/// the live instance of the fourth startup walk this wave collapses.
#[derive(Debug, Default)]
pub struct CapabilitySink {
    /// The highest-seq withdrawal this sink has seen. Seeded from the prior
    /// cache's `capability_provenance` and then folded forward in place by
    /// every in-window event — `note_ask_withdrawal`'s "highest seq always
    /// wins" rule means a newer in-window withdrawal overwrites the seed
    /// here even when that new seq lands *above* this same start's horizon.
    pub latest: Option<AskWithdrawal>,
    /// What `latest` was seeded with, before this pass touched it. Kept
    /// separately — never folded — so `build_floor_state` can fall back to
    /// it when `latest` has been carried past the horizon: the seed is
    /// always `seq <= H_old <= H` (§2.5), so it is always a legal answer,
    /// while `latest` folding past `H` is exactly the case the fallback
    /// exists for.
    seed: Option<AskWithdrawal>,
}

impl CapabilitySink {
    /// Seed from a prior cache's `capability_provenance`, mirroring
    /// `LedgerSink::seeded`.
    pub fn seeded(seed: Option<AskWithdrawal>) -> Self {
        Self {
            latest: seed.clone(),
            seed,
        }
    }
}

impl ReplaySink for CapabilitySink {
    fn push(&mut self, event: &Event) -> Result<(), StartupError> {
        note_ask_withdrawal(&mut self.latest, event);
        // W3 §8.3: the same watermark can also survive as residue carried
        // on a `prune.intent`, once the event `note_ask_withdrawal` itself
        // would have matched has been pruned away. Both folds apply the
        // same higher-seq-wins rule to the same `latest`, so whichever
        // source is still in the replay window (or both, agreeing) wins
        // correctly.
        note_carried_ask_withdrawal(&mut self.latest, event);
        Ok(())
    }
}

/// Records, per `command_id`, enough to answer §26 by name below the
/// replay window (Q8) and enough to rewrite the FloorState cache's
/// `commands` list at the end of every start without a second walk.
#[derive(Debug, Default)]
pub struct LedgerSink {
    /// The latest known outcome class + Work id per `command_id`.
    pub rows: BTreeMap<String, FloorCommandRow>,
    /// The seq that (re)established each row this pass — absent for a row
    /// that came from the seed and was never touched by this pass, which is
    /// exactly what makes it always survive into the next cache (a seeded
    /// row's original seq is always `<= H_old <= H`).
    pub seqs: BTreeMap<String, u64>,
    /// `command_id -> work_id`, folded from `work.submitted`'s own
    /// `correlation_id` — never from a command event's own (conditional)
    /// `work_id` field. Private bookkeeping this sink needs to answer a
    /// later `command.accepted`/`command.rejected` for the same id.
    command_works: BTreeMap<String, String>,
}

impl LedgerSink {
    /// Seed from a prior cache's `commands` list, keyed by `command_id`.
    pub fn seeded(seed: BTreeMap<String, FloorCommandRow>) -> Self {
        Self {
            rows: seed,
            seqs: BTreeMap::new(),
            command_works: BTreeMap::new(),
        }
    }
}

impl ReplaySink for LedgerSink {
    fn push(&mut self, event: &Event) -> Result<(), StartupError> {
        match event.kind.as_str() {
            KIND_WORK_SUBMITTED => {
                let Some(correlation_id) = event.correlation_id.as_deref() else {
                    return Ok(());
                };
                let Some(work_id) = event.payload["work"]["id"].as_str() else {
                    return Ok(());
                };
                self.command_works
                    .insert(correlation_id.to_string(), work_id.to_string());
                // Tentative "submitted" row for the two-append crash window —
                // overwritten below if the matching command.accepted (the
                // ordinary case) or command.rejected later arrives in this
                // same pass.
                self.rows
                    .entry(correlation_id.to_string())
                    .or_insert_with(|| FloorCommandRow {
                        command_id: correlation_id.to_string(),
                        class: FloorCommandClass::Submitted,
                        work_id: Some(work_id.to_string()),
                    });
                self.seqs.insert(correlation_id.to_string(), event.seq);
            }
            KIND_COMMAND_ACCEPTED | KIND_COMMAND_REJECTED => {
                let Some(command_id) = event.payload["command_id"].as_str() else {
                    return Ok(());
                };
                let class = if event.kind == KIND_COMMAND_ACCEPTED {
                    FloorCommandClass::Accepted
                } else {
                    FloorCommandClass::Rejected
                };
                let work_id = self.command_works.get(command_id).cloned();
                self.rows.insert(
                    command_id.to_string(),
                    FloorCommandRow {
                        command_id: command_id.to_string(),
                        class,
                        work_id,
                    },
                );
                self.seqs.insert(command_id.to_string(), event.seq);
            }
            _ => {}
        }
        Ok(())
    }
}

/// Records the first seq this pass saw for every `work_id` it touches — the
/// `first_seq(id)` half of [`horizon`]'s no-straddle predicate. An id seeded
/// from the cache (never named by this pass at all) has no entry here, which
/// [`horizon`] reads as `first_seq = 0` — exactly right, since a seeded id's
/// `last_seq` is already `<= H_old`, so it can never straddle a candidate
/// `>= H_old`.
#[derive(Debug, Default)]
pub struct HorizonSink {
    first_seq_by_work: BTreeMap<String, u64>,
}

impl HorizonSink {
    /// W3 §2.5: the raw fold this pass produced, for a caller (`daemon::
    /// start_with`) that merges it into the process-lifetime
    /// `Core::first_seq_by_work` index the prune horizon reads. Read-only —
    /// this sink's own fold is otherwise exactly W2's.
    pub fn first_seq_by_work(&self) -> &BTreeMap<String, u64> {
        &self.first_seq_by_work
    }
}

impl ReplaySink for HorizonSink {
    fn push(&mut self, event: &Event) -> Result<(), StartupError> {
        if let Some(id) = &event.work_id {
            self.first_seq_by_work
                .entry(id.clone())
                .or_insert(event.seq);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------
// The startup window
// ---------------------------------------------------------------------

/// The seq at which the live window boundary sits: everything with
/// `seq > window_seq` is inside the window, everything at or below it is
/// not. Walks segments newest-first, accumulating count and bytes, and stops
/// before the segment that would exceed either [`STARTUP_WINDOW_SEGMENTS`]
/// or [`STARTUP_WINDOW_BYTES`] — always keeping at least the newest segment.
/// `0` when the whole journal fits (every existing test, and the live
/// dogfood estate).
pub fn window_seq(bounds: &[SegmentBound]) -> u64 {
    let mut count = 0usize;
    let mut bytes = 0u64;
    for (index, segment) in bounds.iter().enumerate().rev() {
        let is_newest = index == bounds.len() - 1;
        let next_count = count + 1;
        let next_bytes = bytes + segment.bytes;
        if !is_newest && (next_count > STARTUP_WINDOW_SEGMENTS || next_bytes > STARTUP_WINDOW_BYTES)
        {
            return segment.last_seq;
        }
        count = next_count;
        bytes = next_bytes;
    }
    0
}

/// A2's horizon predicate — the correctness heart of the wave, applied to
/// the cache boundary: the highest seq `b` (bounded by the window and by any
/// segment's `last_seq`) such that no Work straddles `b` and every Work
/// settled at or below `b` is out of `works`/`runs`. `H = 0` (nothing
/// summarizable) is a normal outcome, not a failure.
///
/// This is the same predicate W3's prune horizon needs, further narrowed by
/// the retention cap and the per-event allowlist — land the shape here so
/// W3 cites it rather than writing a second one.
pub fn horizon(
    bounds: &[SegmentBound],
    window_seq: u64,
    registry: &WorkRegistry,
    horizon_sink: &HorizonSink,
) -> u64 {
    let mut candidates: Vec<u64> = bounds
        .iter()
        .map(|s| s.last_seq)
        .filter(|&last_seq| last_seq <= window_seq)
        .collect();
    candidates.push(0);
    candidates.sort_unstable();
    candidates.dedup();

    // nostraddle: f(b) = max{ last_seq(id) : first_seq(id) <= b } is
    // monotone in b, so sort by first_seq ascending and sweep a running max.
    let mut by_first_seq: Vec<(u64, u64)> = registry
        .work_index
        .values()
        .map(|row| {
            let first_seq = horizon_sink
                .first_seq_by_work
                .get(&row.id)
                .copied()
                .unwrap_or(0);
            (first_seq, row.last_seq)
        })
        .collect();
    by_first_seq.sort_unstable_by_key(|&(first_seq, _)| first_seq);

    // settled: blocker = min{ last_seq(id) : id in works ∪ runs }.
    let blocker = registry
        .work_index
        .values()
        .filter(|row| registry.works.contains_key(&row.id) || registry.runs.contains_key(&row.id))
        .map(|row| row.last_seq)
        .min()
        .unwrap_or(u64::MAX);

    let mut h = 0u64;
    let mut idx = 0usize;
    let mut running_max = 0u64;
    for &b in &candidates {
        while idx < by_first_seq.len() && by_first_seq[idx].0 <= b {
            running_max = running_max.max(by_first_seq[idx].1);
            idx += 1;
        }
        let nostraddle = running_max <= b;
        let settled = b < blocker;
        if nostraddle && settled {
            h = b;
        }
    }
    h
}

// ---------------------------------------------------------------------
// The FloorState cache
// ---------------------------------------------------------------------

/// One segment's extent as the cache binds to it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FloorSegment {
    /// Rotation-order index.
    pub index: u64,
    /// First seq in this segment.
    pub first_seq: u64,
    /// Last seq in this segment.
    pub last_seq: u64,
    /// Segment file length in bytes, at write time.
    pub bytes: u64,
    /// Lowercase hex BLAKE3 of the segment file's exact bytes, at write time.
    pub blake3: String,
}

/// The prefix of the journal this cache summarizes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FloorBinding {
    /// First seq of the oldest segment this cache summarizes — A1's floor.
    /// Written honestly from [`Journal::floor_seq`], never hardcoded to 1.
    pub floor_seq: u64,
    /// Last seq of the newest segment this cache summarizes: the horizon
    /// `H`.
    pub summarized_through_seq: u64,
    /// Extents of every summarized segment, oldest first.
    pub segments: Vec<FloorSegment>,
}

/// One Work's slim index row, as the cache carries it — [`WorkIndexRow`]'s
/// fields plus `pruned_at` (W3's slot, always `None` in W2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FloorWorkRow {
    /// Same as [`WorkIndexRow::id`].
    pub id: String,
    /// Same as [`WorkIndexRow::intent`].
    pub intent: String,
    /// Same as [`WorkIndexRow::state`].
    pub state: WorkState,
    /// Same as [`WorkIndexRow::integrity`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity: Option<IntegrityDisposition>,
    /// Same as [`WorkIndexRow::created_at`].
    pub created_at: String,
    /// Same as [`WorkIndexRow::updated_at`].
    pub updated_at: String,
    /// Same as [`WorkIndexRow::last_seq`].
    pub last_seq: u64,
    /// W3 §3.1: first seq this process has ever seen for this Work — the
    /// field without which a cache-hit start cannot compute the prune
    /// horizon's no-straddle predicate at all (see
    /// [`crate::runtime::prune::FirstSeqIndex`]'s own doc for why a missing
    /// value would make pruning permanently inert rather than merely
    /// conservative). New in schema v2; a v1 cache never had it, which is
    /// exactly why v1 is a named miss now rather than read as `0`.
    #[serde(default)]
    pub first_seq: u64,
}

/// Recorded outcome class of one below-window command — keys only, never the
/// full response body (Q8; a full `CommandOutcome` archive was measured at
/// 250-500 MB and rejected).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FloorCommandClass {
    /// The command was accepted.
    Accepted,
    /// The command was rejected.
    Rejected,
    /// Only a `work.submitted` was seen for this id — the two-append crash
    /// window, never followed by its `command.accepted`.
    Submitted,
}

/// One command-ledger row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FloorCommandRow {
    /// The client-supplied command id.
    pub command_id: String,
    /// What happened to it.
    pub class: FloorCommandClass,
    /// The Work this command created, for a submit. `None` otherwise —
    /// present (not omitted) on the wire only when `Some`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_id: Option<String>,
}

/// The startup cache, persisted at
/// `<data_dir>/projections/floor-state.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FloorState {
    /// Schema identifier — see [`FLOOR_STATE_SCHEMA`].
    pub schema: String,
    /// The journal prefix this cache summarizes.
    pub binding: FloorBinding,
    /// Every summarized Work's slim index row.
    pub works: Vec<FloorWorkRow>,
    /// Every summarized command's ledger row.
    pub commands: Vec<FloorCommandRow>,
    /// The claude ask-grammar-withdrawal watermark at or below the horizon,
    /// if any.
    #[serde(default)]
    pub capability_provenance: Option<AskWithdrawal>,
    /// W3 §9.2: every Work this estate has ever pruned — the whole of
    /// `WorkRegistry::pruned_works`, unconditionally (not filtered by this
    /// start's own horizon: once pruned, a Work's row is a permanent fact
    /// the journal can no longer reproduce on its own below whatever floor
    /// this estate now has, so the cache must carry every one of them, not
    /// just the ones this pass's own horizon happens to cover).
    #[serde(default)]
    pub pruned_works: Vec<crate::runtime::prune::PrunedWorkRow>,
    /// W3 §9.2: the whole of `WorkRegistry::pruned_commands`, same reasoning.
    #[serde(default)]
    pub pruned_commands: Vec<crate::runtime::prune::PrunedCommandRow>,
    /// W3 §6.5: the whole of `WorkRegistry::pending_prune`. Always `None` in
    /// a *written* cache — an intent below the horizon can never be left
    /// uncompleted — carried honestly rather than assumed.
    #[serde(default)]
    ///
    /// Boxed: `PruneIntentRecord` carries two full `PruneResidue`s (each with
    /// its own `Vec`s), which otherwise inlines into `FloorState` and, by
    /// extension, `Plan::Windowed` — large enough to trip
    /// `clippy::large_enum_variant` against `Plan::Full`'s few scalars. This
    /// is always `None` in a written cache (§6.5); boxing costs nothing on
    /// that path and only ever matters for the one CacheMiss-adjacent
    /// diagnostic case that reads it back.
    pub pending_prune: Option<Box<crate::runtime::prune::PruneIntentRecord>>,
    /// W3 §5.2: the whole of `WorkRegistry::quarantined_blobs` — the next
    /// prune cycle's deletion list; losing it would leak quarantined blobs
    /// forever.
    #[serde(default)]
    pub quarantined_blobs: Vec<String>,
}

/// Why a start did not use the cache — logged, and reported on
/// `daemon.started`, never returned as an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheMiss {
    /// `--rebuild-cache` was passed.
    Forced,
    /// No cache file exists.
    Absent,
    /// The file exists but could not be read.
    Unreadable(String),
    /// The file did not parse as JSON in the expected shape.
    Malformed(String),
    /// The file declares a schema this build does not understand.
    UnknownSchema {
        /// The schema string found.
        found: String,
    },
    /// The live journal's oldest surviving segment does not match the
    /// cache's — W3 territory, impossible in W2's production paths.
    FloorMoved {
        /// What the cache believed the floor was.
        cached: u64,
        /// What the live journal's floor actually is.
        live: u64,
    },
    /// The summarized segment set does not otherwise match the live prefix.
    SegmentSetChanged {
        /// What was wrong.
        detail: String,
    },
    /// A summarized segment's length on disk no longer matches the cache.
    SegmentBytesChanged {
        /// The segment index.
        index: u64,
    },
    /// A summarized segment's BLAKE3 no longer matches the cache.
    SegmentHashChanged {
        /// The segment index.
        index: u64,
    },
    /// The cache's summarized prefix does not connect contiguously to the
    /// live tail.
    NotContiguousWithLive {
        /// What the cache says it summarized through.
        summarized_through: u64,
        /// The seq the live journal's next segment actually starts at.
        next_first_seq: u64,
    },
    /// The cache's horizon sits above the current window boundary.
    HorizonAboveWindow {
        /// The cache's horizon.
        horizon: u64,
        /// The current window boundary.
        window: u64,
    },
}

impl CacheMiss {
    /// The `daemon.started` vocabulary this variant reports under
    /// `"miss:<tag>"` (`Forced` is reported as `"forced-rebuild"` instead —
    /// see [`Plan::startup_cache_tag`]).
    pub fn tag(&self) -> &'static str {
        match self {
            CacheMiss::Forced => "forced",
            CacheMiss::Absent => "absent",
            CacheMiss::Unreadable(_) => "unreadable",
            CacheMiss::Malformed(_) => "malformed",
            CacheMiss::UnknownSchema { .. } => "unknown_schema",
            CacheMiss::FloorMoved { .. } => "floor_moved",
            CacheMiss::SegmentSetChanged { .. } => "segment_set_changed",
            CacheMiss::SegmentBytesChanged { .. } => "segment_bytes_changed",
            CacheMiss::SegmentHashChanged { .. } => "segment_hash_changed",
            CacheMiss::NotContiguousWithLive { .. } => "not_contiguous_with_live",
            CacheMiss::HorizonAboveWindow { .. } => "horizon_above_window",
        }
    }
}

/// What one startup should do: seed from a verified cache and replay only
/// the window, or fall back to a full floor-aware replay.
#[derive(Debug, Clone, PartialEq)]
pub enum Plan {
    /// Cache accepted. The registry seeds from its rows; the pass starts at
    /// `H + 1`.
    Windowed {
        /// The accepted cache.
        cache: FloorState,
        /// The current window boundary.
        window_seq: u64,
    },
    /// No usable cache. Full floor-aware replay; the registry starts empty.
    Full {
        /// The live journal's floor (1 in every W2 production path).
        floor_seq: u64,
        /// The current window boundary.
        window_seq: u64,
        /// Why the cache was not used.
        miss: CacheMiss,
    },
}

impl Plan {
    /// Decide what this start should do. Reads the cache file at most once;
    /// every failure mode short-circuits to `Plan::Full` with a
    /// [`CacheMiss`] and a `tracing::info!` naming it — never an error.
    pub fn resolve(
        data_dir: &Path,
        journal: &Journal,
        force_rebuild: bool,
    ) -> Result<Plan, StartupError> {
        let live = journal.segment_bounds()?;
        let window_seq = window_seq(&live);
        let floor_seq = live.first().map(|b| b.first_seq).unwrap_or(1);

        macro_rules! miss {
            ($m:expr) => {{
                let miss = $m;
                tracing::info!(reason = ?miss, "startup cache miss");
                return Ok(Plan::Full {
                    floor_seq,
                    window_seq,
                    miss,
                });
            }};
        }

        if force_rebuild {
            miss!(CacheMiss::Forced);
        }

        let path = floor_state_path(data_dir);
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => miss!(CacheMiss::Absent),
            Err(e) => miss!(CacheMiss::Unreadable(e.to_string())),
        };
        let cache: FloorState = match serde_json::from_slice(&bytes) {
            Ok(cache) => cache,
            Err(e) => miss!(CacheMiss::Malformed(e.to_string())),
        };
        if cache.schema != FLOOR_STATE_SCHEMA {
            miss!(CacheMiss::UnknownSchema {
                found: cache.schema.clone(),
            });
        }
        if live.is_empty() && !cache.binding.segments.is_empty() {
            miss!(CacheMiss::SegmentSetChanged {
                detail: "the cache names segments but the live journal has none".to_string(),
            });
        }

        // Prefix check: the binding's segment index list must equal
        // live[0..n], and floor_seq must equal live[0].first_seq.
        let n = cache.binding.segments.len();
        if n > live.len() {
            miss!(CacheMiss::SegmentSetChanged {
                detail: format!(
                    "cache summarizes {n} segments but the live journal has only {}",
                    live.len()
                ),
            });
        }
        for (i, seg) in cache.binding.segments.iter().enumerate() {
            if seg.index != live[i].index {
                if i == 0 {
                    miss!(CacheMiss::FloorMoved {
                        cached: cache.binding.floor_seq,
                        live: live[0].first_seq,
                    });
                }
                miss!(CacheMiss::SegmentSetChanged {
                    detail: format!(
                        "segment index mismatch at position {i}: cached {}, live {}",
                        seg.index, live[i].index
                    ),
                });
            }
            if seg.first_seq != live[i].first_seq || seg.last_seq != live[i].last_seq {
                miss!(CacheMiss::SegmentSetChanged {
                    detail: format!("segment {} bounds mismatch", seg.index),
                });
            }
        }
        if let Some(first_live) = live.first()
            && cache.binding.floor_seq != first_live.first_seq
        {
            miss!(CacheMiss::FloorMoved {
                cached: cache.binding.floor_seq,
                live: first_live.first_seq,
            });
        }

        // Per-segment integrity: length, then a streamed BLAKE3.
        for seg in &cache.binding.segments {
            let live_seg = live
                .iter()
                .find(|s| s.index == seg.index)
                .expect("prefix check above already proved this segment is in `live`");
            let meta = match std::fs::metadata(&live_seg.path) {
                Ok(meta) => meta,
                Err(e) => miss!(CacheMiss::Unreadable(e.to_string())),
            };
            if meta.len() != seg.bytes {
                miss!(CacheMiss::SegmentBytesChanged { index: seg.index });
            }
            let hash = match hash_file(&live_seg.path) {
                Ok(hash) => hash,
                Err(e) => miss!(CacheMiss::Unreadable(e.to_string())),
            };
            if hash != seg.blake3 {
                miss!(CacheMiss::SegmentHashChanged { index: seg.index });
            }
        }

        // Contiguity with the live tail.
        if let Some(last_seg) = cache.binding.segments.last()
            && cache.binding.summarized_through_seq != last_seg.last_seq
        {
            miss!(CacheMiss::NotContiguousWithLive {
                summarized_through: cache.binding.summarized_through_seq,
                next_first_seq: last_seg.last_seq + 1,
            });
        }
        if n < live.len() {
            let next_first_seq = live[n].first_seq;
            if next_first_seq != cache.binding.summarized_through_seq + 1 {
                miss!(CacheMiss::NotContiguousWithLive {
                    summarized_through: cache.binding.summarized_through_seq,
                    next_first_seq,
                });
            }
        }

        // Horizon still inside the window.
        if cache.binding.summarized_through_seq > window_seq {
            miss!(CacheMiss::HorizonAboveWindow {
                horizon: cache.binding.summarized_through_seq,
                window: window_seq,
            });
        }

        Ok(Plan::Windowed { cache, window_seq })
    }

    /// The registry a start with this plan should begin folding into: seeded
    /// from the cache's `work_index` rows on a hit, empty on a miss.
    pub fn seed_registry(&self) -> Projection<WorkRegistry> {
        match self {
            Plan::Windowed { cache, .. } => {
                let mut seed = WorkRegistry::default();
                for row in &cache.works {
                    seed.work_index.insert(
                        row.id.clone(),
                        WorkIndexRow {
                            id: row.id.clone(),
                            intent: row.intent.clone(),
                            state: row.state,
                            integrity: row.integrity,
                            created_at: row.created_at.clone(),
                            updated_at: row.updated_at.clone(),
                            last_seq: row.last_seq,
                        },
                    );
                }
                // W3: the cache's residue seeds the registry exactly like
                // `work_index` does — a windowed start's own replay only
                // covers events above the horizon, so a prune's residue
                // recorded below it would otherwise be lost on a cache hit.
                for row in &cache.pruned_works {
                    seed.pruned_works.insert(row.id.clone(), row.clone());
                }
                for row in &cache.pruned_commands {
                    seed.pruned_commands
                        .insert(row.command_id.clone(), row.clone());
                }
                seed.pending_prune = cache.pending_prune.as_deref().cloned();
                seed.quarantined_blobs = cache.quarantined_blobs.clone();
                Projection::resumed(
                    seed,
                    cache.binding.summarized_through_seq,
                    work_registry_reducer,
                )
            }
            Plan::Full { .. } => work_registry_projection(),
        }
    }

    /// The capability watermark a fresh `ClaudeBackend` should be seeded
    /// with before the shared pass runs.
    pub fn capability_seed(&self) -> Option<AskWithdrawal> {
        match self {
            Plan::Windowed { cache, .. } => cache.capability_provenance.clone(),
            Plan::Full { .. } => None,
        }
    }

    /// The `LedgerSink` seed: the cache's below-window command keys, keyed
    /// by `command_id`. Empty on a full-replay start *by construction* — a
    /// full replay puts every command the journal has into
    /// `registry.commands`, so a second lookup would be redundant.
    pub fn ledger_seed(&self) -> BTreeMap<String, FloorCommandRow> {
        match self {
            Plan::Windowed { cache, .. } => cache
                .commands
                .iter()
                .map(|row| (row.command_id.clone(), row.clone()))
                .collect(),
            Plan::Full { .. } => BTreeMap::new(),
        }
    }

    /// The one `Replay` the shared pass drives: the window on a hit,
    /// A1's floor-aware full replay on a miss.
    pub fn replay(&self, journal: &Journal) -> Result<Replay, JournalError> {
        match self {
            Plan::Windowed { cache, .. } => {
                journal.replay_after(cache.binding.summarized_through_seq)
            }
            Plan::Full { .. } => journal.replay_from_floor(),
        }
    }

    /// The seq the replay above starts expecting — `H + 1` on a hit, the
    /// floor on a miss. [`PassReport::from_seq`]'s doc explains why the
    /// caller, not [`drive`], is the one to set this.
    pub fn from_seq(&self) -> u64 {
        match self {
            Plan::Windowed { cache, .. } => cache.binding.summarized_through_seq + 1,
            Plan::Full { floor_seq, .. } => *floor_seq,
        }
    }

    /// The current window boundary, on either arm.
    pub fn window_seq(&self) -> u64 {
        match self {
            Plan::Windowed { window_seq, .. } | Plan::Full { window_seq, .. } => *window_seq,
        }
    }

    /// `daemon.started`'s `startup_cache` field: `"hit"` | `"miss:<reason>"`
    /// | `"forced-rebuild"`.
    pub fn startup_cache_tag(&self) -> String {
        match self {
            Plan::Windowed { .. } => "hit".to_string(),
            Plan::Full {
                miss: CacheMiss::Forced,
                ..
            } => "forced-rebuild".to_string(),
            Plan::Full { miss, .. } => format!("miss:{}", miss.tag()),
        }
    }

    /// Build the cache this start should leave behind, from everything the
    /// shared pass just folded — hit or miss, extending the cache is always
    /// sound (§2.6): a cache-hit start's window has moved forward since the
    /// cache was written, and everything the new cache needs is already in
    /// hand.
    ///
    /// `first_seq` is W3's addition: the process-lifetime index
    /// (`Core::first_seq_by_work`, merged from the cache seed and every
    /// `HorizonSink` this process has folded) each [`FloorWorkRow`] needs to
    /// carry its own `first_seq` — kept a *separate* parameter from
    /// `horizon_sink` rather than replacing it, so this cache's own horizon
    /// computation (`horizon()`, above) is untouched.
    pub fn next_cache(
        &self,
        journal: &Journal,
        registry: &WorkRegistry,
        ledger: &LedgerSink,
        capability: &CapabilitySink,
        horizon_sink: &HorizonSink,
        first_seq: &BTreeMap<String, u64>,
    ) -> Result<Option<FloorState>, StartupError> {
        let bounds = journal.segment_bounds()?;
        let h = horizon(&bounds, self.window_seq(), registry, horizon_sink);
        build_floor_state(&bounds, h, registry, ledger, capability, first_seq)
    }
}

/// The projections directory's `floor-state.json` path for `data_dir`.
fn floor_state_path(data_dir: &Path) -> PathBuf {
    analytics::projections_dir(data_dir).join(FLOOR_STATE_FILE)
}

/// Stream a file through BLAKE3 in [`HASH_CHUNK`]-sized pieces — never
/// `fs::read` the whole segment into memory.
fn hash_file(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; HASH_CHUNK];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

/// Build the cache this start should leave behind (`H == 0` builds nothing
/// summarizable — a normal outcome, not a failure).
///
///   works    = every `work_index` row with `last_seq <= H`
///   commands = seeded rows ∪ { in-window rows whose recording seq `<= H` }
///   capability_provenance = the watermark if its seq `<= H`, else the
///                            seeded one
///   binding  = every segment with `last_seq <= H`, in order, with
///              first_seq/last_seq/bytes/BLAKE3 read fresh from disk now
fn build_floor_state(
    bounds: &[SegmentBound],
    horizon: u64,
    registry: &WorkRegistry,
    ledger: &LedgerSink,
    capability: &CapabilitySink,
    first_seq: &BTreeMap<String, u64>,
) -> Result<Option<FloorState>, StartupError> {
    if horizon == 0 {
        return Ok(None);
    }

    let mut segments = Vec::new();
    for b in bounds {
        if b.last_seq > horizon {
            break;
        }
        let blake3 = hash_file(&b.path)?;
        segments.push(FloorSegment {
            index: b.index,
            first_seq: b.first_seq,
            last_seq: b.last_seq,
            bytes: b.bytes,
            blake3,
        });
    }
    let floor_seq = bounds.first().map(|b| b.first_seq).unwrap_or(1);

    let mut works: Vec<FloorWorkRow> = registry
        .work_index
        .values()
        .filter(|row| row.last_seq <= horizon)
        .map(|row| FloorWorkRow {
            id: row.id.clone(),
            intent: row.intent.clone(),
            state: row.state,
            integrity: row.integrity,
            created_at: row.created_at.clone(),
            updated_at: row.updated_at.clone(),
            last_seq: row.last_seq,
            first_seq: first_seq.get(&row.id).copied().unwrap_or(0),
        })
        .collect();
    works.sort_unstable_by(|a, b| a.id.cmp(&b.id));

    let mut commands: Vec<FloorCommandRow> = ledger
        .rows
        .iter()
        .filter(|(id, _)| {
            ledger
                .seqs
                .get(id.as_str())
                .is_none_or(|&seq| seq <= horizon)
        })
        .map(|(_, row)| row.clone())
        .collect();
    commands.sort_unstable_by(|a, b| a.command_id.cmp(&b.command_id));

    // capability_provenance = the watermark if its seq <= H, else the
    // seeded one (this fn's own doc comment) — `capability.latest` can be
    // carried, by `note_ask_withdrawal`'s single-scalar "highest seq wins"
    // fold, past this same start's horizon (an open Work can hold H back
    // across restarts while a capability withdrawal keeps recurring
    // in-window); falling back to `capability.seed` is what keeps the
    // written cache's watermark honestly `<= H` instead of dropping to
    // `null` and discarding a still-valid, previously cached value.
    let capability_provenance = capability
        .latest
        .clone()
        .filter(|w| w.seq <= horizon)
        .or_else(|| capability.seed.clone());

    Ok(Some(FloorState {
        schema: FLOOR_STATE_SCHEMA.to_string(),
        binding: FloorBinding {
            floor_seq,
            summarized_through_seq: horizon,
            segments,
        },
        works,
        commands,
        capability_provenance,
        // W3 §9.2: unconditional copies of the registry's own residue — see
        // this struct's own field docs for why these are never filtered by
        // `horizon` the way `works`/`commands` are.
        pruned_works: registry.pruned_works.values().cloned().collect(),
        pruned_commands: registry.pruned_commands.values().cloned().collect(),
        pending_prune: registry.pending_prune.clone().map(Box::new),
        quarantined_blobs: registry.quarantined_blobs.clone(),
    }))
}

/// Write the cache atomically, or remove a stale file when there is nothing
/// to say (`state == None`, `H == 0`). Leaving a stale file the next start
/// would reject anyway is worse than no file.
pub fn persist_or_remove(state: Option<&FloorState>, data_dir: &Path) -> Result<(), StartupError> {
    let path = floor_state_path(data_dir);
    match state {
        None => match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StartupError::CacheWrite(e)),
        },
        Some(state) => {
            // `projections/` is already created durably on every production
            // start (before `Analytics::begin_rebuild` runs); ensured here
            // too so this function has no ordering dependency of its own.
            if let Some(parent) = path.parent() {
                create_dir_all_durable(parent)?;
            }
            // Pretty, not compact: this file is meant to be read by a person
            // debugging a slow start.
            let bytes = serde_json::to_vec_pretty(state)
                .map_err(|e| StartupError::CacheWrite(std::io::Error::other(e)))?;
            write_atomic(&path, &bytes)?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use tempfile::TempDir;

    use super::*;
    use crate::domain::event::{EventDraft, EventSource};
    use crate::domain::work::{KIND_WORK_STARTED, Work};

    fn draft(kind: &str, work_id: Option<&str>, payload: Value) -> EventDraft {
        let mut draft = EventDraft::new(EventSource::new("daemon", "test"), kind, payload);
        if let Some(id) = work_id {
            draft = draft.with_work_id(id);
        }
        draft
    }

    fn submit_payload(work_id: &str, state: &str) -> Value {
        json!({"work": {
            "id": work_id,
            "intent": "do it",
            "state": state,
            "created_by": "test",
            "created_at": "2026-01-01T00:00:00.000Z",
        }})
    }

    fn segment(index: u64, first_seq: u64, last_seq: u64, bytes: u64) -> SegmentBound {
        SegmentBound {
            index,
            path: PathBuf::from(format!("/dev/null/{index}")),
            first_seq,
            last_seq,
            bytes,
        }
    }

    // -------------------------------------------------------------
    // window_seq
    // -------------------------------------------------------------

    #[test]
    fn window_seq_covers_the_whole_journal_at_exactly_16_segments() {
        let bounds: Vec<SegmentBound> = (1..=16)
            .map(|i| segment(i, (i - 1) * 100 + 1, i * 100, 1024))
            .collect();
        assert_eq!(
            window_seq(&bounds),
            0,
            "16 segments fit inside the window exactly"
        );
    }

    #[test]
    fn window_seq_excludes_the_oldest_segment_at_17_segments() {
        let bounds: Vec<SegmentBound> = (1..=17)
            .map(|i| segment(i, (i - 1) * 100 + 1, i * 100, 1024))
            .collect();
        // Segment 1 (last_seq 100) must be the one excluded.
        assert_eq!(window_seq(&bounds), 100);
    }

    #[test]
    fn window_seq_is_bounded_by_bytes_even_under_16_segments() {
        // Two segments, each just over half the byte budget — the older one
        // must be excluded even though the segment count alone would fit.
        let bounds = vec![
            segment(1, 1, 100, 70 * 1024 * 1024),
            segment(2, 101, 200, 70 * 1024 * 1024),
        ];
        assert_eq!(
            window_seq(&bounds),
            100,
            "the byte bound must exclude segment 1 even at only 2 segments"
        );
    }

    #[test]
    fn window_seq_always_keeps_at_least_the_newest_segment() {
        // One enormous segment, alone: never excluded, however large.
        let bounds = vec![segment(1, 1, 100, 300 * 1024 * 1024)];
        assert_eq!(window_seq(&bounds), 0);
    }

    // -------------------------------------------------------------
    // horizon
    // -------------------------------------------------------------

    fn work_index_row(id: &str, state: WorkState, last_seq: u64) -> WorkIndexRow {
        WorkIndexRow {
            id: id.to_string(),
            intent: "x".to_string(),
            state,
            integrity: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            last_seq,
        }
    }

    #[test]
    fn horizon_advances_past_a_settled_completed_work() {
        let bounds = vec![segment(1, 1, 50, 1024), segment(2, 51, 100, 1024)];
        let mut registry = WorkRegistry::default();
        registry.work_index.insert(
            "w1".to_string(),
            work_index_row("w1", WorkState::Completed, 10),
        );
        // Not in `works` or `runs` — evicted, i.e. settled. Its whole life
        // (seq 5..10) sits inside segment 1, so it disqualifies nothing —
        // the window's own tail (100) is the answer.
        let mut horizon_sink = HorizonSink::default();
        horizon_sink.first_seq_by_work.insert("w1".to_string(), 5);

        let h = horizon(&bounds, 100, &registry, &horizon_sink);
        assert_eq!(
            h, 100,
            "w1 neither straddles nor blocks any candidate, so H reaches the window's own tail"
        );
    }

    #[test]
    fn horizon_stays_below_an_unsettled_work_still_in_works() {
        let bounds = vec![segment(1, 1, 50, 1024), segment(2, 51, 100, 1024)];
        let mut registry = WorkRegistry::default();
        registry.work_index.insert(
            "w1".to_string(),
            work_index_row("w1", WorkState::Active, 10),
        );
        let work: Work = serde_json::from_value(submit_payload("w1", "active")["work"].clone())
            .expect("deserialize Work fixture");
        registry.works.insert("w1".to_string(), work);
        let mut horizon_sink = HorizonSink::default();
        horizon_sink.first_seq_by_work.insert("w1".to_string(), 5);

        let h = horizon(&bounds, 100, &registry, &horizon_sink);
        assert_eq!(
            h, 0,
            "w1 is unsettled at last_seq 10, so no candidate >= 10 qualifies"
        );
    }

    #[test]
    fn horizon_stays_below_a_straddling_work() {
        // w1's first event is inside the window's own segments, but its last
        // event lands past every candidate the window offers (still live,
        // beyond the tail segment_bounds even knows about) — it straddles
        // every candidate up to and including the window's own tail.
        let bounds = vec![segment(1, 1, 50, 1024), segment(2, 51, 100, 1024)];
        let mut registry = WorkRegistry::default();
        registry.work_index.insert(
            "w1".to_string(),
            work_index_row("w1", WorkState::Active, 150),
        );
        let mut horizon_sink = HorizonSink::default();
        horizon_sink.first_seq_by_work.insert("w1".to_string(), 10);

        let h = horizon(&bounds, 100, &registry, &horizon_sink);
        assert_eq!(
            h, 0,
            "w1 straddles every candidate (first_seq 10 <= every b, but last_seq 150 > every b <= 100)"
        );
    }

    #[test]
    fn horizon_is_zero_for_an_empty_registry() {
        let bounds = vec![segment(1, 1, 50, 1024)];
        let registry = WorkRegistry::default();
        let horizon_sink = HorizonSink::default();
        assert_eq!(horizon(&bounds, 50, &registry, &horizon_sink), 50);
    }

    // -------------------------------------------------------------
    // Plan::resolve / CacheMiss
    // -------------------------------------------------------------

    /// Build a real journal + cache pair in a `TempDir`: 20 segments (one
    /// event each, via a 1-byte rotation threshold — reaching past
    /// [`STARTUP_WINDOW_SEGMENTS`] the way §5.4/§9.1 require, never by
    /// shrinking the window), and a cache that honestly summarizes every
    /// segment at or below the resulting `window_seq` — i.e. exactly the
    /// prefix `Plan::resolve` should accept.
    fn build_journal_and_cache(dir: &Path) -> (Journal, FloorState) {
        let mut journal = Journal::open_with(dir, 1).expect("open");
        for n in 1..=20 {
            journal
                .append(draft(
                    KIND_WORK_SUBMITTED,
                    Some(&format!("w{n}")),
                    submit_payload(&format!("w{n}"), "pending"),
                ))
                .expect("append");
        }
        let bounds = journal.segment_bounds().expect("segment_bounds");
        let w_seq = window_seq(&bounds);
        let summarized: Vec<FloorSegment> = bounds
            .iter()
            .filter(|b| b.last_seq <= w_seq)
            .map(|b| FloorSegment {
                index: b.index,
                first_seq: b.first_seq,
                last_seq: b.last_seq,
                bytes: b.bytes,
                blake3: hash_file(&b.path).expect("hash"),
            })
            .collect();
        let summarized_through_seq = summarized.last().map(|s| s.last_seq).unwrap_or(0);
        let cache = FloorState {
            schema: FLOOR_STATE_SCHEMA.to_string(),
            binding: FloorBinding {
                floor_seq: bounds[0].first_seq,
                summarized_through_seq,
                segments: summarized,
            },
            works: Vec::new(),
            commands: Vec::new(),
            capability_provenance: None,
            pruned_works: Vec::new(),
            pruned_commands: Vec::new(),
            pending_prune: None,
            quarantined_blobs: Vec::new(),
        };
        (journal, cache)
    }

    fn write_cache(data_dir: &Path, cache: &FloorState) {
        std::fs::create_dir_all(analytics::projections_dir(data_dir)).expect("mkdir");
        std::fs::write(
            floor_state_path(data_dir),
            serde_json::to_vec_pretty(cache).expect("serialize"),
        )
        .expect("write cache");
    }

    #[test]
    fn resolve_accepts_a_valid_cache() {
        let dir = TempDir::new().expect("tempdir");
        let (journal, cache) = build_journal_and_cache(dir.path());
        write_cache(dir.path(), &cache);

        let plan = Plan::resolve(dir.path(), &journal, false).expect("resolve");
        assert!(matches!(plan, Plan::Windowed { .. }));
        assert_eq!(plan.startup_cache_tag(), "hit");
    }

    #[test]
    fn resolve_reports_forced_when_rebuild_cache_is_set() {
        let dir = TempDir::new().expect("tempdir");
        let (journal, cache) = build_journal_and_cache(dir.path());
        write_cache(dir.path(), &cache);

        let plan = Plan::resolve(dir.path(), &journal, true).expect("resolve");
        assert!(matches!(
            plan,
            Plan::Full {
                miss: CacheMiss::Forced,
                ..
            }
        ));
        assert_eq!(plan.startup_cache_tag(), "forced-rebuild");
    }

    #[test]
    fn resolve_reports_absent_when_no_file_exists() {
        let dir = TempDir::new().expect("tempdir");
        let (journal, _cache) = build_journal_and_cache(dir.path());

        let plan = Plan::resolve(dir.path(), &journal, false).expect("resolve");
        assert!(matches!(
            plan,
            Plan::Full {
                miss: CacheMiss::Absent,
                ..
            }
        ));
        assert_eq!(plan.startup_cache_tag(), "miss:absent");
    }

    #[test]
    fn resolve_reports_malformed_on_garbage_json() {
        let dir = TempDir::new().expect("tempdir");
        let (journal, _cache) = build_journal_and_cache(dir.path());
        std::fs::create_dir_all(analytics::projections_dir(dir.path())).expect("mkdir");
        std::fs::write(floor_state_path(dir.path()), b"not json").expect("write garbage");

        let plan = Plan::resolve(dir.path(), &journal, false).expect("resolve");
        assert!(matches!(
            plan,
            Plan::Full {
                miss: CacheMiss::Malformed(_),
                ..
            }
        ));
    }

    #[test]
    fn resolve_reports_unknown_schema() {
        let dir = TempDir::new().expect("tempdir");
        let (journal, mut cache) = build_journal_and_cache(dir.path());
        cache.schema = "sergeant.floor-state.v99".to_string();
        write_cache(dir.path(), &cache);

        let plan = Plan::resolve(dir.path(), &journal, false).expect("resolve");
        assert!(matches!(
            plan,
            Plan::Full {
                miss: CacheMiss::UnknownSchema { .. },
                ..
            }
        ));
    }

    #[test]
    fn resolve_reports_segment_bytes_changed_on_a_lengthened_segment() {
        let dir = TempDir::new().expect("tempdir");
        let (journal, cache) = build_journal_and_cache(dir.path());
        write_cache(dir.path(), &cache);
        // Append bytes after a summarized segment's one line: `first_seq`
        // still parses (it only ever reads the first line), so this lands
        // squarely on the length check rather than corrupting the segment
        // in a way an earlier check would already catch.
        let seg_path = &journal.segment_bounds().expect("bounds")[0].path;
        {
            use std::io::Write as _;
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(seg_path)
                .expect("open for append");
            writeln!(
                file,
                "extra bytes that change this segment's length on disk"
            )
            .expect("append");
        }

        let plan = Plan::resolve(dir.path(), &journal, false).expect("resolve");
        assert!(matches!(
            plan,
            Plan::Full {
                miss: CacheMiss::SegmentBytesChanged { .. },
                ..
            }
        ));
    }

    #[test]
    fn resolve_reports_segment_hash_changed_on_same_length_mutation() {
        let dir = TempDir::new().expect("tempdir");
        let (journal, cache) = build_journal_and_cache(dir.path());
        write_cache(dir.path(), &cache);
        let seg_path = &journal.segment_bounds().expect("bounds")[0].path;
        let mut bytes = std::fs::read(seg_path).expect("read");
        // Flip one byte inside the content, keeping the length identical.
        let mutate_at = bytes
            .iter()
            .position(|&b| b == b'w')
            .expect("find a byte to flip");
        bytes[mutate_at] = b'x';
        std::fs::write(seg_path, &bytes).expect("mutate in place");

        let plan = Plan::resolve(dir.path(), &journal, false).expect("resolve");
        assert!(matches!(
            plan,
            Plan::Full {
                miss: CacheMiss::SegmentHashChanged { .. },
                ..
            }
        ));
    }

    #[test]
    fn resolve_reports_horizon_above_window_when_the_cache_outruns_the_window() {
        let dir = TempDir::new().expect("tempdir");
        let (journal, mut cache) = build_journal_and_cache(dir.path());
        // Extend the cache to also summarize the next live segment — a
        // self-consistent (real bytes, real hash, contiguous) claim, but one
        // the live window itself keeps live for this 20-segment fixture.
        let bounds = journal.segment_bounds().expect("bounds");
        let next_index = cache.binding.segments.len() as u64 + 1;
        let next = bounds
            .iter()
            .find(|b| b.index == next_index)
            .expect("a next segment exists past what the base cache summarizes");
        cache.binding.segments.push(FloorSegment {
            index: next.index,
            first_seq: next.first_seq,
            last_seq: next.last_seq,
            bytes: next.bytes,
            blake3: hash_file(&next.path).expect("hash"),
        });
        cache.binding.summarized_through_seq = next.last_seq;
        write_cache(dir.path(), &cache);

        let plan = Plan::resolve(dir.path(), &journal, false).expect("resolve");
        assert!(matches!(
            plan,
            Plan::Full {
                miss: CacheMiss::HorizonAboveWindow { .. },
                ..
            }
        ));
    }

    #[test]
    fn resolve_reports_not_contiguous_with_live_on_a_gap() {
        let dir = TempDir::new().expect("tempdir");
        let (journal, mut cache) = build_journal_and_cache(dir.path());
        // Understate `summarized_through_seq` alone (leaving the segments
        // array, and therefore the earlier prefix/integrity checks, entirely
        // untouched) — the horizon it claims no longer agrees with what its
        // own last summarized segment's bound says it covers.
        assert!(
            cache.binding.summarized_through_seq > 0,
            "fixture must summarize something"
        );
        cache.binding.summarized_through_seq -= 1;
        write_cache(dir.path(), &cache);

        let plan = Plan::resolve(dir.path(), &journal, false).expect("resolve");
        assert!(matches!(
            plan,
            Plan::Full {
                miss: CacheMiss::NotContiguousWithLive { .. },
                ..
            }
        ));
    }

    /// §9.1 step 5's own corruption list includes "rename a segment", and
    /// deleting the leading one is the only way W2 ever reaches a floor > 1
    /// (§2.4 step 6's own comment: "this branch is exercised only by tests
    /// that delete a leading segment in a test DataDir"). Two of the nine
    /// `CacheMiss` variants a corruption can produce had no dedicated test
    /// before this: this one is `FloorMoved`.
    #[test]
    fn resolve_reports_floor_moved_when_the_leading_segment_is_deleted() {
        let dir = TempDir::new().expect("tempdir");
        let (journal, cache) = build_journal_and_cache(dir.path());
        write_cache(dir.path(), &cache);
        let leading = journal.segment_bounds().expect("bounds")[0].path.clone();
        std::fs::remove_file(&leading).expect("remove leading segment");

        let plan = Plan::resolve(dir.path(), &journal, false).expect("resolve");
        assert!(
            matches!(
                plan,
                Plan::Full {
                    miss: CacheMiss::FloorMoved { .. },
                    ..
                }
            ),
            "deleting the leading segment must report the *specific* FloorMoved variant, \
             not merely rebuild: {plan:?}"
        );
    }

    /// The sibling of the test above: deleting a non-leading summarized
    /// segment leaves the prefix's first index untouched (so this must not
    /// report `FloorMoved`) but desynchronizes every summarized bound from
    /// that point on — `SegmentSetChanged`, the other of the two `CacheMiss`
    /// variants §9.1 step 5 required and neither had before this.
    #[test]
    fn resolve_reports_segment_set_changed_when_a_middle_segment_is_deleted() {
        let dir = TempDir::new().expect("tempdir");
        let (journal, cache) = build_journal_and_cache(dir.path());
        write_cache(dir.path(), &cache);
        let bounds = journal.segment_bounds().expect("bounds");
        assert!(
            cache.binding.segments.len() > 2,
            "fixture needs at least 3 summarized segments so the deleted one is neither \
             the leading one nor the whole summarized set"
        );
        // Segment index 2 (position 1): not the leading segment, so this
        // must not trip `FloorMoved`.
        std::fs::remove_file(&bounds[1].path).expect("remove a middle segment");

        let plan = Plan::resolve(dir.path(), &journal, false).expect("resolve");
        assert!(
            matches!(
                plan,
                Plan::Full {
                    miss: CacheMiss::SegmentSetChanged { .. },
                    ..
                }
            ),
            "deleting a middle segment must report the *specific* SegmentSetChanged variant, \
             not merely rebuild and not FloorMoved: {plan:?}"
        );
    }

    // -------------------------------------------------------------
    // build_floor_state — capability_provenance's seed fallback
    // -------------------------------------------------------------

    /// Regression pin (invariants finding): `build_floor_state`'s own doc
    /// comment states "capability_provenance = the watermark if its seq
    /// `<= H`, else the seeded one". `capability.latest` is folded in place
    /// by `note_ask_withdrawal`'s single-scalar "highest seq wins" rule, so
    /// a recurring in-window withdrawal can carry it *past* this same call's
    /// horizon while the seed (always `seq <= H_old <= H`, §2.5) is still a
    /// legal answer. Before this fix the function had no `else` at all and
    /// wrote `null`, silently discarding a still-valid, previously cached
    /// watermark.
    #[test]
    fn build_floor_state_falls_back_to_the_seeded_capability_watermark_past_the_horizon() {
        let dir = TempDir::new().expect("tempdir");
        let (journal, cache) = build_journal_and_cache(dir.path());
        let bounds = journal.segment_bounds().expect("segment_bounds");
        let horizon = cache.binding.summarized_through_seq;
        assert!(horizon > 0, "fixture must summarize something");

        let seed = AskWithdrawal {
            seq: 1,
            version: "2.1.100".to_string(),
        };
        let mut capability = CapabilitySink::seeded(Some(seed.clone()));
        // A later in-window event supersedes the seed in the running fold,
        // landing past this start's own horizon.
        capability.latest = Some(AskWithdrawal {
            seq: horizon + 1,
            version: "2.1.200".to_string(),
        });

        let registry = WorkRegistry::default();
        let ledger = LedgerSink::default();
        let state = build_floor_state(
            &bounds,
            horizon,
            &registry,
            &ledger,
            &capability,
            &BTreeMap::new(),
        )
        .expect("build_floor_state")
        .expect("H > 0 must produce a cache");

        assert_eq!(
            state.capability_provenance,
            Some(seed),
            "capability_provenance must fall back to the seeded watermark rather than \
             dropping to null when the running fold's `latest` has been carried past H"
        );
    }

    /// The ordinary case, alongside the fallback above: when `latest` is
    /// still `<= H`, it is used as-is (the seed is not preferred just for
    /// existing).
    #[test]
    fn build_floor_state_uses_latest_capability_watermark_when_still_within_the_horizon() {
        let dir = TempDir::new().expect("tempdir");
        let (journal, cache) = build_journal_and_cache(dir.path());
        let bounds = journal.segment_bounds().expect("segment_bounds");
        let horizon = cache.binding.summarized_through_seq;
        assert!(horizon > 0, "fixture must summarize something");

        let seed = AskWithdrawal {
            seq: 1,
            version: "2.1.100".to_string(),
        };
        let latest = AskWithdrawal {
            seq: horizon,
            version: "2.1.150".to_string(),
        };
        let mut capability = CapabilitySink::seeded(Some(seed));
        capability.latest = Some(latest.clone());

        let registry = WorkRegistry::default();
        let ledger = LedgerSink::default();
        let state = build_floor_state(
            &bounds,
            horizon,
            &registry,
            &ledger,
            &capability,
            &BTreeMap::new(),
        )
        .expect("build_floor_state")
        .expect("H > 0 must produce a cache");

        assert_eq!(
            state.capability_provenance,
            Some(latest),
            "an in-window watermark still <= H must win over the seed"
        );
    }

    // -------------------------------------------------------------
    // ReplaySink / drive
    // -------------------------------------------------------------

    #[test]
    fn drive_feeds_every_sink_every_event_in_order() {
        struct Recorder(Vec<u64>);
        impl ReplaySink for Recorder {
            fn push(&mut self, event: &Event) -> Result<(), StartupError> {
                self.0.push(event.seq);
                Ok(())
            }
        }
        let events: Vec<Result<Event, JournalError>> = vec![
            Ok(draft(KIND_WORK_STARTED, Some("w1"), json!({})).into_event(1)),
            Ok(draft(KIND_WORK_STARTED, Some("w1"), json!({})).into_event(2)),
        ];
        let mut a = Recorder(Vec::new());
        let mut b = Recorder(Vec::new());
        let report = drive(events, &mut [&mut a, &mut b]).expect("drive");
        assert_eq!(a.0, vec![1, 2]);
        assert_eq!(b.0, vec![1, 2]);
        assert_eq!(report.replayed_events, 2);
        assert_eq!(report.max_seq, 2);
        assert_eq!(report.from_seq, 1);
    }

    #[test]
    fn drive_fails_closed_on_the_first_sink_error() {
        struct AlwaysFails;
        impl ReplaySink for AlwaysFails {
            fn push(&mut self, _event: &Event) -> Result<(), StartupError> {
                Err(StartupError::CacheWrite(std::io::Error::other("boom")))
            }
        }
        let events: Vec<Result<Event, JournalError>> =
            vec![Ok(
                draft(KIND_WORK_STARTED, Some("w1"), json!({})).into_event(1)
            )];
        let mut sink = AlwaysFails;
        let err = drive(events, &mut [&mut sink]).expect_err("must fail closed");
        assert!(matches!(err, StartupError::CacheWrite(_)));
    }

    // -------------------------------------------------------------
    // LedgerSink
    // -------------------------------------------------------------

    /// A `work.submitted` event carrying the submitting command's id as its
    /// own `correlation_id` — mirrors `apply_registry_event`'s own
    /// `command_works` population exactly, since `LedgerSink` must agree
    /// with it on where that mapping comes from.
    fn submitted_draft(work_id: &str, command_id: &str) -> EventDraft {
        let mut draft = draft(
            KIND_WORK_SUBMITTED,
            Some(work_id),
            submit_payload(work_id, "pending"),
        );
        draft.correlation_id = Some(command_id.to_string());
        draft
    }

    #[test]
    fn ledger_sink_classifies_accepted_rejected_and_the_crash_window() {
        let mut sink = LedgerSink::default();
        // A submit whose command.accepted follows normally.
        sink.push(&submitted_draft("wA", "wA").into_event(1))
            .unwrap();
        sink.push(
            &draft(
                KIND_COMMAND_ACCEPTED,
                None,
                json!({"command_id": "wA", "operation": "work.submit", "status": 201u16, "result": {}}),
            )
            .into_event(2),
        )
        .unwrap();
        // A submit whose command.accepted never arrives — the crash window.
        sink.push(&submitted_draft("wB", "wB").into_event(3))
            .unwrap();
        // A non-submit command rejected, carrying no work_id.
        sink.push(
            &draft(
                KIND_COMMAND_REJECTED,
                None,
                json!({"command_id": "cmdC", "operation": "work.cancel", "status": 409u16, "result": {}}),
            )
            .into_event(4),
        )
        .unwrap();

        assert_eq!(sink.rows["wA"].class, FloorCommandClass::Accepted);
        assert_eq!(sink.rows["wA"].work_id.as_deref(), Some("wA"));
        assert_eq!(sink.rows["wB"].class, FloorCommandClass::Submitted);
        assert_eq!(sink.rows["wB"].work_id.as_deref(), Some("wB"));
        assert_eq!(sink.rows["cmdC"].class, FloorCommandClass::Rejected);
        assert_eq!(sink.rows["cmdC"].work_id, None);
    }

    // -------------------------------------------------------------
    // FloorState round trip
    // -------------------------------------------------------------

    #[test]
    fn floor_state_round_trips_through_json() {
        let (_journal, cache) = build_journal_and_cache(TempDir::new().expect("tempdir").path());
        let bytes = serde_json::to_vec(&cache).expect("serialize");
        let back: FloorState = serde_json::from_slice(&bytes).expect("deserialize");
        assert_eq!(back, cache);
    }

    #[test]
    fn persist_or_remove_writes_then_a_none_removes_it() {
        let dir = TempDir::new().expect("tempdir");
        let (_journal, cache) = build_journal_and_cache(dir.path());
        persist_or_remove(Some(&cache), dir.path()).expect("write");
        assert!(floor_state_path(dir.path()).exists());
        persist_or_remove(None, dir.path()).expect("remove");
        assert!(!floor_state_path(dir.path()).exists());
        // Removing again (already absent) must not error.
        persist_or_remove(None, dir.path()).expect("idempotent remove");
    }
}
