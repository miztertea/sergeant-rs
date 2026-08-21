//! W3: the prune engine — bounded retention made real (issue #17's rulings
//! record, `docs/proposals/foundation-2026-08-21.md` W3 section).
//!
//! W2 (`runtime::startup`) made the daemon able to *ignore* old history; this
//! module makes it able to *delete* it. A policy in `sergeant.toml` names how
//! many Works this estate keeps ([`crate::domain::estate::Estate::retention`]).
//! At every daemon start, and on segment rotation once the retained-Work
//! count crosses that cap, the daemon computes the highest seq below which
//! every Work is terminal, retired whole, past the cap, and not straddling
//! the cut ([`candidate_horizon`]/[`plan`]) — then journals its intent
//! ([`KIND_PRUNE_INTENT`]), quarantines the blobs only those Works
//! referenced, unlinks whole segments oldest-first
//! ([`crate::runtime::journal::Journal::unlink_segments`]), and journals a
//! completion ([`KIND_PRUNE_COMPLETED`]). The completion's residue — carried
//! on the intent so a crash cannot lose it — is what lets a replay from the
//! new floor still answer "Work … was pruned on … under policy retention=N"
//! and still refuse a retried `command_id` by name, with no cache in the loop
//! at all.
//!
//! One-way *logic* dependency, deliberately: this module calls
//! [`crate::runtime::startup`] and [`crate::runtime::projection`]; neither of
//! those ever calls back into this one. `startup::FloorState` does reference
//! this module's residue types ([`PrunedWorkRow`], [`PrunedCommandRow`],
//! [`PruneIntentRecord`]) directly, since the cache has to be able to carry
//! what the registry carries — a data-shape dependency, not a call — and
//! `projection::WorkRegistry` does the same for the identical reason.
//!
//! # Deliberate deviations from the wave spec (see the PR body / final report)
//!
//! - **No `time` crate.** The wave spec's design sketch uses
//!   `time::OffsetDateTime`; this codebase (`src/domain/event.rs`'s own doc
//!   comment) deliberately hand-rolls RFC3339 rather than add a date-time
//!   crate to the pinned set. [`stall_report`] therefore takes
//!   `std::time::SystemTime`, reusing [`crate::domain::event::unix_millis`]
//!   for age arithmetic instead.
//! - **`prune.intent`'s wire payload is flat, not nested under `"blobs"`.**
//!   The spec's illustrative JSON groups `condemn`/`delete_quarantined`
//!   under a `"blobs"` key; nothing outside this wave's own code reads this
//!   payload yet, so [`PruneIntentRecord`] (minus `intent_seq`, which is
//!   read from the event's own `seq` at fold time, never from the payload)
//!   is serialized directly. The mechanism — residue carried on the intent,
//!   completion cross-checked by `intent_seq` — is unchanged.
//! - **The mark scan is one sequential pass, not a batched/threaded one.**
//!   §5.1/§7.1 describe reading the candidate segments and the surviving
//!   segments as two independently-optimized passes (the surviving scan on
//!   a `blocking_sync` thread, outside the guard, skipped early once every
//!   condemned ref is accounted for). [`plan`] performs both reads
//!   sequentially and to completion. Every invariant this buys (I-W3-6 in
//!   particular) still holds — only the described performance work is
//!   deferred.
//! - **Step 6 (cache rewrite after a live prune) is not wired.** §9.3's
//!   second cache write point needs the live capability-provenance
//!   watermark, which today lives only on the running `ClaudeBackend`
//!   adapter, not on `Core` — threading it into the tick's prune call is
//!   real additional wiring this landing does not attempt. [`run`] instead
//!   removes any existing cache file after a successful cycle
//!   ([`crate::runtime::startup::persist_or_remove`] with `None`), so a
//!   start after a live prune pays one full floor-aware replay (safe, per
//!   Q2's "absent cache ⇒ one full replay, never an error") rather than
//!   risking a stale cache — a named performance regression, not a
//!   correctness one. The **other** write point (end of every successful
//!   startup fold) is untouched, so a clean restart after a prune still
//!   writes a fresh v2 cache normally.
//! - **The batch cap ([`PRUNE_MAX_WORKS_PER_CYCLE`]) is enforced inside
//!   [`candidate_horizon`]**, not as a separate post-hoc truncation of
//!   [`plan`]'s residue — since `work_index` (which `candidate_horizon`
//!   already sweeps) holds exactly the *not-yet-pruned* Works, counting
//!   entries with `last_seq <= b` there already is the "new residue" count
//!   §7.4 bounds.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::backend::claude::{AskWithdrawal, note_ask_withdrawal};
use crate::domain::event::{Event, EventDraft, EventSource, rfc3339_utc_now, unix_millis};
use crate::domain::work::{KIND_COMMAND_ACCEPTED, KIND_COMMAND_REJECTED, WorkState};
use crate::runtime::blob::{self, BlobError, BlobRef, BlobStore};
use crate::runtime::integrity::IntegrityDisposition;
use crate::runtime::journal::{JournalError, SegmentBound};
use crate::runtime::projection::{WorkIndexRow, WorkRegistry, run_is_retired};
use crate::runtime::startup::{self, FloorCommandClass, StartupError};

/// A prune cycle's declaration of what it is about to delete, and everything
/// a later reader — including a reader that has lost the cache and every
/// segment this cycle deletes — needs to answer for it. Journaled and
/// fsynced *before* the first rename or unlink.
pub const KIND_PRUNE_INTENT: &str = "prune.intent";

/// The matching completion. Carries counts and the intent's seq; the residue
/// itself lives on the intent, which the fold has already seen.
pub const KIND_PRUNE_COMPLETED: &str = "prune.completed";

/// Q5's allowlist, as a **mechanism**: an event with no `work_id` whose kind
/// is not here pins its segment, and a kind enters this list only with a
/// replay-equivalence proof (`tests/w3_allowlist_equivalence.rs`).
pub const NON_WORK_ALLOWLIST: &[&str] = &[
    crate::daemon::KIND_DAEMON_STARTED,
    crate::daemon::KIND_DAEMON_STOPPED,
    crate::daemon::KIND_BACKEND_PROBED,
    crate::daemon::KIND_ADMISSION_PAUSED,
    crate::daemon::KIND_ADMISSION_RESUMED,
    KIND_COMMAND_ACCEPTED,
    KIND_COMMAND_REJECTED,
    KIND_PRUNE_INTENT,
    KIND_PRUNE_COMPLETED,
];

/// Cap on how many Works one cycle retires, and therefore on how large one
/// `prune.intent`'s own *new* residue can be (~200 B/Work + ~90 B/command
/// key). The *carried* residue is not bounded by this (§6.4).
pub const PRUNE_MAX_WORKS_PER_CYCLE: usize = 4096;

/// A rotation-triggered cycle runs only when at least this many whole
/// segments are eligible, so the O(retained journal) mark scan is amortized
/// over a batch. A startup-triggered cycle ignores it.
pub const PRUNE_BATCH_MIN_SEGMENTS: usize = 4;

/// First seq this daemon has seen for each retained Work — the `first_seq(id)`
/// half of the no-straddle predicate, maintained across the life of the
/// process. Seeded from the cache's rows ∪ the startup pass, advanced by
/// `Core::commit`, pruned ids removed at `prune.completed`.
pub type FirstSeqIndex = BTreeMap<String, u64>;

/// Where a resolved [`PrunePolicy`]'s number came from — journaled on every
/// prune event so the record names the authorization, not just the number
/// (A1: "every discard flows from declared policy … and leaves a journaled
/// record").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicySource {
    /// `[estate] retention` in this estate's `sergeant.toml`.
    Manifest,
    /// `DaemonConfig::retention` — test rigs only.
    Config,
    /// No declaration anywhere; the built-in default.
    Default,
}

/// The retention policy one daemon runs under, resolved once at start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrunePolicy {
    /// Works retained, newest-first by `last_seq`.
    pub retention: u32,
    /// Where the number came from.
    pub source: PolicySource,
}

/// One pruned Work, as the journal remembers it after its events are gone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrunedWorkRow {
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
    /// RFC3339 UTC — the `prune.intent` event's own timestamp, read once
    /// when the cycle commits and embedded verbatim in the payload, so a
    /// later replay reproduces it identically rather than reading a wall
    /// clock at fold time.
    pub pruned_at: String,
}

/// One pruned command's ledger key — Q8's exemption made durable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrunedCommandRow {
    /// The client-supplied command id.
    pub command_id: String,
    /// What happened to it.
    pub class: FloorCommandClass,
    /// The Work this command created, for a submit. `None` otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_id: Option<String>,
}

impl PrunedCommandRow {
    /// Three-field copy into [`startup::FloorCommandRow`], for
    /// `api::replay_command`'s pruned-command arm (§6.3).
    pub fn as_floor_row(&self) -> startup::FloorCommandRow {
        startup::FloorCommandRow {
            command_id: self.command_id.clone(),
            class: self.class,
            work_id: self.work_id.clone(),
        }
    }
}

/// Everything a completion applies. Carried on the intent (F5: after the
/// unlink there is nothing left to recompute it from) and folded on the
/// completion.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PruneResidue {
    /// Newly (or previously carried) pruned Works.
    pub works: Vec<PrunedWorkRow>,
    /// Newly (or previously carried) pruned command ledger keys.
    pub commands: Vec<PrunedCommandRow>,
    /// The ask-grammar withdrawal watermark, when this cycle is deleting the
    /// event that carried it (§8.3). Its `seq` is the **original** seq, not
    /// the prune event's — "higher seq wins" must keep meaning what it means.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_provenance: Option<AskWithdrawal>,
}

impl PruneResidue {
    /// Union merge: `works`/`commands` keyed by id (a later entry for the
    /// same id replaces the earlier one — collisions are not expected in
    /// practice, since a Work or command is pruned exactly once, but the
    /// rule is defined rather than left to chance), `capability_provenance`
    /// takes the higher `seq`.
    pub fn merge(&mut self, other: PruneResidue) {
        for row in other.works {
            self.works.retain(|r| r.id != row.id);
            self.works.push(row);
        }
        for row in other.commands {
            self.commands.retain(|r| r.command_id != row.command_id);
            self.commands.push(row);
        }
        if let Some(candidate) = other.capability_provenance {
            let wins = self
                .capability_provenance
                .as_ref()
                .is_none_or(|current| candidate.seq > current.seq);
            if wins {
                self.capability_provenance = Some(candidate);
            }
        }
    }

    /// Owned-value sibling of [`PruneResidue::merge`].
    pub fn merged_with(mut self, other: PruneResidue) -> PruneResidue {
        self.merge(other);
        self
    }
}

/// One segment's extent as a prune record binds to it — [`SegmentBound`]
/// minus the filesystem path, which a durable record must never carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PruneSegment {
    /// Rotation-order index.
    pub index: u64,
    /// First seq in this segment.
    pub first_seq: u64,
    /// Last seq in this segment.
    pub last_seq: u64,
    /// Segment file length in bytes at plan time.
    pub bytes: u64,
}

/// A declared, unacknowledged prune cycle — what
/// [`WorkRegistry::pending_prune`] holds between the intent and its
/// completion, and the only thing Q9's crash completion is allowed to act
/// on. Every field but `intent_seq` is a verbatim copy of the intent's
/// payload; `intent_seq` is read from the intent *event's* own `seq` at fold
/// time, never carried in the payload (nothing needs to know its own seq
/// before it exists).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PruneIntentRecord {
    /// Seq of the `prune.intent` event itself.
    pub intent_seq: u64,
    /// The policy this cycle acted under.
    pub policy: PrunePolicy,
    /// The horizon this cycle computed.
    pub horizon_seq: u64,
    /// The floor before this cycle.
    pub floor_seq_before: u64,
    /// The floor after this cycle (`horizon_seq + 1`).
    pub floor_seq_after: u64,
    /// Segments to unlink, ascending.
    pub segments: Vec<PruneSegment>,
    /// This cycle's own newly-discovered residue.
    pub residue: PruneResidue,
    /// Residue carried forward from an earlier, now-condemned
    /// `prune.intent` (§6.4).
    pub carried_forward: PruneResidue,
    /// Hexes to move into `.pruned/` this cycle.
    pub condemn: Vec<String>,
    /// Hexes the *previous* cycle quarantined, to delete now (§5.2).
    pub delete_quarantined: Vec<String>,
    /// Whether this cycle was triggered at daemon start (§10.3) rather than
    /// by rotation (§10.4).
    pub started_at_startup: bool,
}

/// The wire shape of `prune.intent`'s payload — [`PruneIntentRecord`] minus
/// `intent_seq` (see this module's own deviation note).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IntentPayload {
    policy: PrunePolicy,
    horizon_seq: u64,
    floor_seq_before: u64,
    floor_seq_after: u64,
    segments: Vec<PruneSegment>,
    residue: PruneResidue,
    carried_forward: PruneResidue,
    condemn: Vec<String>,
    delete_quarantined: Vec<String>,
    started_at_startup: bool,
}

impl PruneIntentRecord {
    /// Reconstruct from a committed `prune.intent` event: every field but
    /// `intent_seq` from the payload, `intent_seq` from the event's own
    /// `seq`. `None` on a payload this build cannot parse — the caller logs
    /// and ignores, per §6.2's "record, apply nothing … a declaration that
    /// cannot be understood is not a declaration this process can act on".
    pub fn from_event(event: &Event) -> Option<Self> {
        let payload: IntentPayload = serde_json::from_value(event.payload.clone()).ok()?;
        Some(Self {
            intent_seq: event.seq,
            policy: payload.policy,
            horizon_seq: payload.horizon_seq,
            floor_seq_before: payload.floor_seq_before,
            floor_seq_after: payload.floor_seq_after,
            segments: payload.segments,
            residue: payload.residue,
            carried_forward: payload.carried_forward,
            condemn: payload.condemn,
            delete_quarantined: payload.delete_quarantined,
            started_at_startup: payload.started_at_startup,
        })
    }

    /// Render as the `prune.intent` payload this cycle commits.
    fn to_payload(&self) -> serde_json::Value {
        serde_json::to_value(IntentPayload {
            policy: self.policy,
            horizon_seq: self.horizon_seq,
            floor_seq_before: self.floor_seq_before,
            floor_seq_after: self.floor_seq_after,
            segments: self.segments.clone(),
            residue: self.residue.clone(),
            carried_forward: self.carried_forward.clone(),
            condemn: self.condemn.clone(),
            delete_quarantined: self.delete_quarantined.clone(),
            started_at_startup: self.started_at_startup,
        })
        .expect("PruneIntentRecord's payload shape always serializes")
    }
}

/// What one cycle would do, computed without touching anything.
#[derive(Debug, Clone, PartialEq)]
pub struct PrunePlan {
    /// The policy this plan was computed under.
    pub policy: PrunePolicy,
    /// The final horizon (after any allowlist/pairing pin has lowered it).
    pub horizon_seq: u64,
    /// Oldest-contiguous prefix of segments, all `last_seq <= horizon_seq`.
    pub segments: Vec<SegmentBound>,
    /// This cycle's own newly-discovered residue.
    pub residue: PruneResidue,
    /// Residue carried forward from a condemned `prune.intent` (§6.4).
    pub carried_forward: PruneResidue,
    /// Blobs to quarantine this cycle.
    pub condemn: BTreeSet<BlobRef>,
    /// Blobs the previous cycle quarantined, to delete now.
    pub delete_quarantined: Vec<String>,
    /// The highest seq the surviving-side mark scan actually reached —
    /// `run`'s top-up reads events after this before quarantining, closing
    /// the window between this plan and the guard that commits it.
    pub scan_through: u64,
    /// Why pruning did not advance further than it did.
    pub stall: PruneStall,
}

/// What one cycle actually did — returned by [`run`]/[`run_startup`],
/// journaled as `prune.completed.outcome`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PruneOutcome {
    /// Segments unlinked this cycle.
    pub segments_unlinked: usize,
    /// Bytes reclaimed by the unlink.
    pub bytes_reclaimed: u64,
    /// Works newly pruned this cycle (excludes carried-forward).
    pub works_pruned: usize,
    /// Commands newly pruned this cycle (excludes carried-forward).
    pub commands_pruned: usize,
    /// Blobs actually moved into quarantine this cycle.
    pub blobs_quarantined: usize,
    /// Blobs actually deleted from quarantine this cycle.
    pub blobs_deleted: usize,
    /// Quarantined blobs found already gone for a reason other than rescue.
    pub blobs_missing: usize,
    /// `delete_quarantined.len() - blobs_deleted - blobs_missing`: rescued
    /// by a live read or write before this cycle's deferred delete ran.
    pub blobs_rescued_before_delete: usize,
    /// The floor after this cycle.
    pub floor_seq_after: u64,
    /// More was eligible than [`PRUNE_MAX_WORKS_PER_CYCLE`] allowed; the
    /// caller should re-arm `prune_pending`.
    pub truncated_by_cap: bool,
}

/// Why pruning is not advancing further — Q7's whole answer: there is no
/// flag, and this type has no field that could become one.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PruneStall {
    /// The Work whose retirement would let the horizon advance furthest.
    pub blocking_work_id: Option<String>,
    /// That Work's state.
    pub blocking_state: Option<WorkState>,
    /// That Work's `updated_at`.
    pub blocking_since: Option<String>,
    /// Age of `blocking_since`, in seconds — populated only by
    /// [`stall_report`] (it alone takes a wall clock); [`candidate_horizon`]
    /// leaves this `None`.
    pub blocking_age_secs: Option<u64>,
    /// The lowest non-allowlisted non-work-scoped event pinning a segment,
    /// when that (not a Work) is what stops the horizon. Populated only by
    /// [`plan`] (it alone reads segment contents); `candidate_horizon`
    /// leaves this `None`.
    pub pinning_kind: Option<String>,
    /// That event's seq.
    pub pinning_seq: Option<u64>,
    /// Context for a doctor-style report.
    pub retained_works: usize,
    /// The retention this policy declares.
    pub retention: u32,
    /// The floor at the time of this report.
    pub floor_seq: u64,
    /// The horizon this report computed.
    pub horizon_seq: u64,
    /// **Not in the wave spec's own field list** — added so
    /// [`PruneOutcome::truncated_by_cap`] has somewhere to read its answer
    /// from without re-deriving it: whether [`PRUNE_MAX_WORKS_PER_CYCLE`]
    /// is what actually bounded this horizon.
    pub truncated_by_cap: bool,
}

/// Errors from planning or running a prune cycle.
#[derive(Debug, thiserror::Error)]
pub enum PruneError {
    /// The journal failed during a plan or a cycle.
    #[error(transparent)]
    Journal(#[from] JournalError),
    /// The blob store failed during quarantine or deferred delete.
    #[error(transparent)]
    Blob(#[from] BlobError),
    /// The core failed to commit or fold a prune event.
    #[error(transparent)]
    Core(#[from] crate::api::CoreError),
    /// The prune cache rewrite failed.
    #[error("prune cache rewrite failed: {0}")]
    Cache(#[from] StartupError),
    /// The Work ids found in the condemned segments do not match the set the
    /// registry says is past the horizon.
    #[error("refusing to prune: residue disagrees with the journal ({detail})")]
    ResidueMismatch {
        /// What disagreed.
        detail: String,
    },
}

/// Whether a Work has been *retired whole* — the prune's terminality test,
/// deliberately not the same test eviction uses.
///
/// `run_is_settled` gates eviction on `is_absorbing(state)` —
/// `Completed | Canceled` only. Retention cannot use that: a `Failed` Work is
/// never evicted, so it stays in `works` forever, and a horizon requiring
/// absence-from-`works` would let one old failure pin the entire journal for
/// the life of the estate — Q7's chronic-stalling escalation arriving on day
/// one. This is that escalation, adopted up front.
///
/// What is kept from `run_is_settled`, verbatim (via
/// [`crate::runtime::projection::run_is_retired`]), is the part that
/// protects `recovery::reconcile`: a run with a surface and no teardown, a
/// `surface_plan` with no surface, or an unsettled reservation is *not*
/// retired, whatever the Work's state says.
///
/// A Work absent from both `works` and `runs` is retired whole by
/// construction: the only way out of those maps is `maybe_evict`, which
/// already required the run predicate.
pub fn retired_whole(state: WorkState, run: Option<&crate::runtime::projection::WorkRun>) -> bool {
    matches!(
        state,
        WorkState::Completed | WorkState::Failed | WorkState::Canceled
    ) && run.is_none_or(run_is_retired)
}

/// Rank retained Works (`work_index` — pruned Works have already left it) by
/// `last_seq` descending; the cap is the seq just below the `N`-th largest.
/// `cap_seq(N) = 0` when there are `N` or fewer retained Works (nothing past
/// the cap). `N == 0` is a degenerate policy nobody should declare (nothing
/// retained at all); it answers `u64::MAX` (every seq is "past the cap")
/// rather than underflowing.
///
/// Seqs are unique per event and `last_seq` is an event's own seq, so no two
/// Works can ever tie — the ranking is total, with no tie-break rule to get
/// wrong.
pub fn cap_seq(work_index: &BTreeMap<String, WorkIndexRow>, n: usize) -> u64 {
    if n == 0 {
        return u64::MAX;
    }
    if work_index.len() <= n {
        return 0;
    }
    let mut last_seqs: Vec<u64> = work_index.values().map(|row| row.last_seq).collect();
    last_seqs.sort_unstable_by(|a, b| b.cmp(a));
    last_seqs[n - 1].saturating_sub(1)
}

/// The Work whose retirement would let the horizon advance furthest: the
/// smallest `last_seq` among retained Works failing [`retired_whole`].
fn blocking_work(registry: &WorkRegistry) -> Option<(&WorkIndexRow, WorkState)> {
    registry
        .work_index
        .values()
        .filter(|row| !retired_whole(row.state, registry.runs.get(&row.id)))
        .map(|row| (row, row.state))
        .min_by_key(|(row, _)| row.last_seq)
}

/// Phase A — in memory, from the registry alone. Cheap enough to run on
/// every rotation tick. See [`plan`] for the segment-content-aware Phase B
/// that can only ever *lower* this answer.
pub fn candidate_horizon(
    bounds: &[SegmentBound],
    registry: &WorkRegistry,
    first_seq: &FirstSeqIndex,
    policy: &PrunePolicy,
) -> (u64, PruneStall) {
    let n = policy.retention as usize;
    let cap = cap_seq(&registry.work_index, n);

    // I-W3-4: the segment the writer currently holds open is never
    // unlinked. `bounds`'s own last (newest) entry is always that segment
    // for whichever live `Journal` computed it (`segment_bounds`'s own doc:
    // its `last_seq` comes from `next_seq() - 1`) — so it is excluded from
    // candidacy here, structurally, rather than relying on
    // `unlink_segments`'s refusal to catch it after a plan has already been
    // built around it. With only one segment (nothing has ever rotated),
    // this correctly leaves no candidate at all beyond the `0` sentinel.
    let mut candidates: Vec<u64> = bounds[..bounds.len().saturating_sub(1)]
        .iter()
        .map(|s| s.last_seq)
        .collect();
    candidates.push(0);
    candidates.sort_unstable();
    candidates.dedup();

    let mut by_first_seq: Vec<(u64, u64)> = registry
        .work_index
        .values()
        .map(|row| {
            let f = first_seq.get(&row.id).copied().unwrap_or(0);
            (f, row.last_seq)
        })
        .collect();
    by_first_seq.sort_unstable_by_key(|&(f, _)| f);

    let blocker = blocking_work(registry);
    let blocker_seq = blocker.map(|(row, _)| row.last_seq).unwrap_or(u64::MAX);

    // §7.4's batch cap: the count of not-yet-pruned Works with
    // `last_seq <= b` is monotone in `b`, computed over the same sorted
    // `work_index` list.
    let mut last_seqs_sorted: Vec<u64> = registry.work_index.values().map(|r| r.last_seq).collect();
    last_seqs_sorted.sort_unstable();
    let count_at = |b: u64| last_seqs_sorted.partition_point(|&s| s <= b);

    let mut h = 0u64;
    let mut idx = 0usize;
    let mut running_max = 0u64;
    let mut truncated_by_cap = false;
    for &b in &candidates {
        while idx < by_first_seq.len() && by_first_seq[idx].0 <= b {
            running_max = running_max.max(by_first_seq[idx].1);
            idx += 1;
        }
        let nostraddle = running_max <= b;
        let retired_ok = b < blocker_seq;
        let capped = b <= cap;
        if !(nostraddle && retired_ok && capped) {
            continue;
        }
        if count_at(b) > PRUNE_MAX_WORKS_PER_CYCLE {
            truncated_by_cap = true;
            continue;
        }
        h = b;
    }

    let stall = PruneStall {
        blocking_work_id: blocker.map(|(row, _)| row.id.clone()),
        blocking_state: blocker.map(|(_, state)| state),
        blocking_since: blocker.map(|(row, _)| row.updated_at.clone()),
        blocking_age_secs: None,
        pinning_kind: None,
        pinning_seq: None,
        retained_works: registry.work_index.len(),
        retention: policy.retention,
        floor_seq: bounds.first().map(|b| b.first_seq).unwrap_or(1),
        horizon_seq: h,
        truncated_by_cap,
    };
    (h, stall)
}

/// The full stall report (§7.3), computable at any time from state the
/// daemon already holds — no journal I/O — for W4's doctor check to read
/// under the guard.
///
/// Deviation from the wave spec's own signature: takes
/// [`std::time::SystemTime`], not `time::OffsetDateTime` — see this module's
/// top doc comment.
pub fn stall_report(
    bounds: &[SegmentBound],
    registry: &WorkRegistry,
    first_seq: &FirstSeqIndex,
    policy: &PrunePolicy,
    now: std::time::SystemTime,
) -> PruneStall {
    let (_, mut stall) = candidate_horizon(bounds, registry, first_seq, policy);
    if let Some(since) = &stall.blocking_since {
        let now_millis = now
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        if let Some(then_millis) = unix_millis(since)
            && now_millis >= then_millis
        {
            stall.blocking_age_secs = Some(((now_millis - then_millis) / 1000) as u64);
        }
    }
    stall
}

/// One pass over the journal from the floor through `ceiling` (inclusive),
/// collecting everything a prune cycle needs from the condemned range: the
/// per-command residue (§7.1 step 2), the carried-forward residue from any
/// `prune.intent`/`prune.completed` pair fully inside the range (§6.4), blob
/// refs (§5.1's pruned side), the ask-grammar withdrawal watermark carried
/// forward (§8.3), every Work id actually named (for the residue
/// cross-check, §7.1's last paragraph), and the lowest seq that pins its
/// segment — an unknown non-work-scoped kind, or a `prune.intent` whose
/// matching `prune.completed` is not also in range (§6.5).
struct CondemnedScan {
    residue: PruneResidue,
    carried_forward: PruneResidue,
    condemn_refs: BTreeSet<BlobRef>,
    work_ids_seen: BTreeSet<String>,
    lowest_pin_seq: Option<u64>,
    max_seq_seen: u64,
}

fn scan_condemned_range(
    data_dir: &Path,
    ceiling: u64,
    registry: &WorkRegistry,
) -> Result<CondemnedScan, PruneError> {
    let mut residue = PruneResidue::default();
    let mut carried_forward = PruneResidue::default();
    let mut condemn_refs = BTreeSet::new();
    let mut work_ids_seen = BTreeSet::new();
    let mut open_intents: BTreeMap<u64, ()> = BTreeMap::new();
    let mut lowest_pin_seq: Option<u64> = None;
    let mut carried_ask_withdrawal: Option<AskWithdrawal> = None;
    let mut max_seq_seen = 0u64;

    for event in crate::runtime::journal::Journal::replay_data_dir_from_floor(data_dir)? {
        let event = event?;
        if event.seq > ceiling {
            break;
        }
        max_seq_seen = event.seq;

        if let Some(id) = &event.work_id {
            work_ids_seen.insert(id.clone());
        }
        for r in blob::refs_in_event(&event) {
            condemn_refs.insert(r);
        }

        match event.kind.as_str() {
            KIND_PRUNE_INTENT => {
                if let Some(record) = PruneIntentRecord::from_event(&event) {
                    carried_forward.merge(record.residue);
                    carried_forward.merge(record.carried_forward);
                }
                open_intents.insert(event.seq, ());
            }
            KIND_PRUNE_COMPLETED => {
                if let Some(intent_seq) = event.payload.get("intent_seq").and_then(|v| v.as_u64()) {
                    open_intents.remove(&intent_seq);
                }
            }
            KIND_COMMAND_ACCEPTED | KIND_COMMAND_REJECTED => {
                if let Some(command_id) = event.payload.get("command_id").and_then(|v| v.as_str()) {
                    let class = if event.kind == KIND_COMMAND_ACCEPTED {
                        FloorCommandClass::Accepted
                    } else {
                        FloorCommandClass::Rejected
                    };
                    let work_id = registry.command_works.get(command_id).cloned();
                    residue.commands.push(PrunedCommandRow {
                        command_id: command_id.to_string(),
                        class,
                        work_id,
                    });
                }
            }
            "conversation.turn.grammar_unmeasured" => {
                note_ask_withdrawal(&mut carried_ask_withdrawal, &event);
            }
            kind => {
                if event.work_id.is_none()
                    && !NON_WORK_ALLOWLIST.contains(&kind)
                    && lowest_pin_seq.is_none()
                {
                    lowest_pin_seq = Some(event.seq);
                }
            }
        }
    }

    for &intent_seq in open_intents.keys() {
        if lowest_pin_seq.is_none_or(|p| intent_seq < p) {
            lowest_pin_seq = Some(intent_seq);
        }
    }

    if let Some(w) = carried_ask_withdrawal {
        let wins = residue
            .capability_provenance
            .as_ref()
            .is_none_or(|current| w.seq > current.seq);
        if wins {
            residue.capability_provenance = Some(w);
        }
    }

    Ok(CondemnedScan {
        residue,
        carried_forward,
        condemn_refs,
        work_ids_seen,
        lowest_pin_seq,
        max_seq_seen,
    })
}

/// Phase B — reads the candidate segments once, and (if anything is a
/// condemn candidate) the surviving segments once. Lock-free; only the data
/// dir path is borrowed, never a live `Journal` handle.
///
/// `candidate` is Phase A's ([`candidate_horizon`]'s) answer, already
/// batch-capped; this can only ever lower it further, via the allowlist/pair
/// pin (§6.5) — never raise it.
///
/// Takes `first_seq` in addition to the wave spec's own listed signature
/// (`data_dir, bounds, candidate, registry, policy`) so the [`PruneStall`]
/// this returns can be built by the same [`candidate_horizon`] the caller
/// already ran, rather than a second, differently-informed copy of it.
pub fn plan(
    data_dir: &Path,
    bounds: &[SegmentBound],
    candidate: u64,
    registry: &WorkRegistry,
    first_seq: &FirstSeqIndex,
    policy: &PrunePolicy,
) -> Result<Option<PrunePlan>, PruneError> {
    if candidate == 0 {
        return Ok(None);
    }

    let first_pass = scan_condemned_range(data_dir, candidate, registry)?;
    let first_pass_pin_seq = first_pass.lowest_pin_seq;

    let mut horizon = candidate;
    if let Some(pin_seq) = first_pass_pin_seq {
        let pin_first_seq = bounds
            .iter()
            .find(|b| b.first_seq <= pin_seq && pin_seq <= b.last_seq)
            .map(|b| b.first_seq)
            .unwrap_or(pin_seq);
        horizon = bounds
            .iter()
            .map(|b| b.last_seq)
            .filter(|&last| last < pin_first_seq)
            .max()
            .unwrap_or(0)
            .min(candidate);
    }
    if horizon == 0 {
        return Ok(None);
    }

    // If the pin lowered the horizon, everything the first pass collected
    // may include events above the new, smaller ceiling — rescan exactly
    // that (smaller, guaranteed pin-free — the first pass already proved no
    // pin exists below it) range.
    let scan = if horizon == candidate {
        first_pass
    } else {
        scan_condemned_range(data_dir, horizon, registry)?
    };

    let segments: Vec<SegmentBound> = bounds
        .iter()
        .filter(|b| b.last_seq <= horizon)
        .cloned()
        .collect();
    if segments.is_empty() {
        return Ok(None);
    }

    let mut residue = scan.residue;
    residue.works = registry
        .work_index
        .values()
        .filter(|row| row.last_seq <= horizon)
        .map(|row| PrunedWorkRow {
            id: row.id.clone(),
            intent: row.intent.clone(),
            state: row.state,
            integrity: row.integrity,
            created_at: row.created_at.clone(),
            updated_at: row.updated_at.clone(),
            last_seq: row.last_seq,
            pruned_at: String::new(), // stamped by `run`/`run_startup` at commit time
        })
        .collect();

    // §7.1's cross-check: the Work ids actually named in the condemned
    // range must equal exactly the set `work_index` says is past the
    // horizon. A mismatch means `FirstSeqIndex` or `work_index` disagrees
    // with the journal — abort, never delete.
    let expected: BTreeSet<String> = residue.works.iter().map(|r| r.id.clone()).collect();
    if scan.work_ids_seen != expected {
        return Err(PruneError::ResidueMismatch {
            detail: format!(
                "condemned range names {} Work ids, work_index expects {} at or below horizon {horizon}",
                scan.work_ids_seen.len(),
                expected.len()
            ),
        });
    }

    let mut surviving_refs: BTreeSet<BlobRef> = BTreeSet::new();
    if !scan.condemn_refs.is_empty() {
        for event in crate::runtime::journal::Journal::replay_data_dir_from_floor(data_dir)? {
            let event = event?;
            if event.seq <= horizon {
                continue;
            }
            for r in blob::refs_in_event(&event) {
                surviving_refs.insert(r);
            }
        }
    }
    let condemn: BTreeSet<BlobRef> = scan
        .condemn_refs
        .difference(&surviving_refs)
        .cloned()
        .collect();

    let (_, mut stall) = candidate_horizon(bounds, registry, first_seq, policy);
    stall.horizon_seq = horizon;
    stall.pinning_seq = first_pass_pin_seq;
    if let Some(pin_seq) = first_pass_pin_seq {
        // The kind is only knowable if the pin was an allowlist violation
        // (an unpaired `prune.intent` has no single "kind" worth naming
        // beyond itself); re-derive it with one more read of that one event
        // rather than threading a third piece of state through the scan.
        stall.pinning_kind = crate::runtime::journal::Journal::replay_data_dir_from_floor(data_dir)
            .ok()
            .and_then(|replay| {
                replay
                    .filter_map(Result::ok)
                    .find(|e| e.seq == pin_seq)
                    .map(|e| e.kind)
            });
    }

    Ok(Some(PrunePlan {
        policy: *policy,
        horizon_seq: horizon,
        segments,
        residue,
        carried_forward: scan.carried_forward,
        condemn,
        delete_quarantined: registry.quarantined_blobs.clone(),
        scan_through: scan.max_seq_seen.max(horizon),
        stall,
    }))
}

/// §10.2: re-validate a plan against the *live* registry, immediately before
/// committing its intent. Between planning (outside the guard) and this call
/// (inside it), an append could have changed a planned Work's eligibility —
/// most sharply via the `failed -> active` retry edge.
fn revalidate(core: &crate::api::Core, plan: &PrunePlan) -> bool {
    let registry = core.registry.state();
    for row in &plan.residue.works {
        let Some(live_row) = registry.work_index.get(&row.id) else {
            return false;
        };
        if live_row.last_seq != row.last_seq {
            return false;
        }
        if !retired_whole(live_row.state, registry.runs.get(&row.id)) {
            return false;
        }
    }
    let Ok(live_bounds) = core.journal.segment_bounds() else {
        return false;
    };
    if live_bounds.len() < plan.segments.len() {
        return false;
    }
    for (planned, live) in plan.segments.iter().zip(live_bounds.iter()) {
        if planned.index != live.index
            || planned.first_seq != live.first_seq
            || planned.last_seq != live.last_seq
        {
            return false;
        }
    }
    let cap = cap_seq(&registry.work_index, plan.policy.retention as usize);
    plan.horizon_seq <= cap
}

/// Execute one already-planned cycle under the caller's `CoreGuard` hold
/// (§10.1) — every step here runs with the daemon's single mutation lock
/// held. Re-validates first (§10.2); a plan the live registry no longer
/// agrees with is a no-op (`Ok(PruneOutcome::default())`) rather than a
/// deletion planned from a disagreement (I-W3-1/2/3's whole point).
///
/// `started_at_startup` names which trigger (§10.3 vs §10.4) produced this
/// cycle, journaled on the intent; the completion this call appends always
/// carries `completed_at_startup: false` — this is a live cycle, never the
/// crash-recovery path ([`complete_interrupted`] is that one, and always
/// sets it `true`).
pub fn run(
    core: &mut crate::api::Core,
    data_dir: &Path,
    mut plan: PrunePlan,
    started_at_startup: bool,
) -> Result<PruneOutcome, PruneError> {
    if !revalidate(core, &plan) {
        core.prune_pending = true;
        return Ok(PruneOutcome::default());
    }

    // §5.1's top-up, under the guard: an append between the plan's mark
    // scan and this hold could have referenced a blob the plan condemned.
    // Closing this window is what makes quarantine+rescue a second line of
    // defence rather than the only one (I-W3-6).
    for event in core.events_after(plan.scan_through)? {
        for r in blob::refs_in_event(&event) {
            plan.condemn.remove(&r);
        }
    }

    let floor_seq_before = core.journal.floor_seq()?.unwrap_or(1);
    let pruned_at = rfc3339_utc_now();
    for row in &mut plan.residue.works {
        row.pruned_at = pruned_at.clone();
    }

    let record = PruneIntentRecord {
        intent_seq: 0, // not part of the payload; filled from the committed event's own seq below
        policy: plan.policy,
        horizon_seq: plan.horizon_seq,
        floor_seq_before,
        floor_seq_after: plan.horizon_seq + 1,
        segments: plan
            .segments
            .iter()
            .map(|b| PruneSegment {
                index: b.index,
                first_seq: b.first_seq,
                last_seq: b.last_seq,
                bytes: b.bytes,
            })
            .collect(),
        residue: plan.residue.clone(),
        carried_forward: plan.carried_forward.clone(),
        condemn: plan.condemn.iter().map(|r| r.hex().to_string()).collect(),
        delete_quarantined: plan.delete_quarantined.clone(),
        started_at_startup,
    };

    // Step 1 (T9, fsynced).
    let intent_event = core.commit(EventDraft::new(
        EventSource::new("daemon", "sergeant"),
        KIND_PRUNE_INTENT,
        record.to_payload(),
    ))?;
    core.flush()?;
    let intent_seq = intent_event.seq;

    let blobs = BlobStore::open(data_dir)?;

    // Step 2 (T3): quarantine.
    let mut blobs_quarantined = 0usize;
    for hex in &record.condemn {
        let blob_ref: BlobRef = format!("b3:{hex}").parse()?;
        if blobs.quarantine(&blob_ref)? {
            blobs_quarantined += 1;
        }
    }

    // Step 3 (T5): the previous cycle's deferred delete.
    let mut blobs_deleted = 0usize;
    for hex in &record.delete_quarantined {
        if blobs.drop_quarantined(hex)? {
            blobs_deleted += 1;
        }
    }
    let blobs_rescued_before_delete = record
        .delete_quarantined
        .len()
        .saturating_sub(blobs_deleted);

    // Step 4 (T1/T2): unlink, oldest-first.
    let indices: Vec<u64> = record.segments.iter().map(|s| s.index).collect();
    let bytes_reclaimed = core.journal.unlink_segments(&indices)?;

    let outcome = PruneOutcome {
        segments_unlinked: indices.len(),
        bytes_reclaimed,
        works_pruned: record.residue.works.len(),
        commands_pruned: record.residue.commands.len(),
        blobs_quarantined,
        blobs_deleted,
        blobs_missing: 0,
        blobs_rescued_before_delete,
        floor_seq_after: record.floor_seq_after,
        truncated_by_cap: plan.stall.truncated_by_cap,
    };

    // Step 5 (T10, fsynced).
    core.commit(EventDraft::new(
        EventSource::new("daemon", "sergeant"),
        KIND_PRUNE_COMPLETED,
        serde_json::json!({
            "intent_seq": intent_seq,
            "outcome": outcome,
            "floor_seq_after": record.floor_seq_after,
            "completed_at_startup": false,
        }),
    ))?;
    core.flush()?;
    core.prune_pending = false;

    // Step 6, deviated (see this module's top doc comment): rather than
    // rewrite the cache from the live capability watermark (not reachable
    // from here today), remove any existing one — safe per Q2, and the next
    // clean start's own write point (unaffected) rebuilds a fresh v2 cache.
    let _ = startup::persist_or_remove(None, data_dir);

    Ok(outcome)
}

/// One call for the daemon-start trigger (§10.3): Phase A, then Phase B,
/// then [`run`] — short-circuiting cheaply (a handful of map iterations)
/// when nothing is eligible, which is the common case for a frequently
/// restarted daemon.
pub fn run_startup(
    core: &mut crate::api::Core,
    data_dir: &Path,
    policy: &PrunePolicy,
    first_seq: &FirstSeqIndex,
) -> Result<PruneOutcome, PruneError> {
    let bounds = core.journal.segment_bounds()?;
    let (candidate, _stall) = candidate_horizon(&bounds, core.registry.state(), first_seq, policy);
    if candidate == 0 {
        return Ok(PruneOutcome::default());
    }
    let Some(plan) = plan(
        data_dir,
        &bounds,
        candidate,
        core.registry.state(),
        first_seq,
        policy,
    )?
    else {
        return Ok(PruneOutcome::default());
    };
    run(core, data_dir, plan, true)
}

/// Q9: finish a prune the predecessor declared and did not acknowledge.
/// Evidence-based, exactly in the sense `recovery::reconcile_terminal_surface`
/// is: the intent is a durable, fsynced record that a specific, enumerated
/// deletion was authorized and begun. Nothing here is inferred, nothing is
/// widened, and no deletion is ever started from suspicion.
///
/// Runs steps 2-5 of [`run`]'s cycle from `pending_prune`'s recorded targets
/// alone, with **no re-planning**: it does not recompute a horizon, does not
/// consult the policy, and does not extend the target set by one segment or
/// one blob. Every step is idempotent (quarantine/drop_quarantined/
/// unlink_segments's own `Ok(false)`/tolerated-`NotFound` shapes), so this is
/// correct across every crash window (F1-F5).
pub fn complete_interrupted(
    core: &mut crate::api::Core,
    data_dir: &Path,
) -> Result<(), PruneError> {
    let Some(pending) = core.registry.state().pending_prune.clone() else {
        return Ok(());
    };

    let blobs = BlobStore::open(data_dir)?;
    let mut blobs_quarantined = 0usize;
    for hex in &pending.condemn {
        let blob_ref: BlobRef = format!("b3:{hex}").parse()?;
        if blobs.quarantine(&blob_ref)? {
            blobs_quarantined += 1;
        }
    }
    let mut blobs_deleted = 0usize;
    for hex in &pending.delete_quarantined {
        if blobs.drop_quarantined(hex)? {
            blobs_deleted += 1;
        }
    }
    let blobs_rescued_before_delete = pending
        .delete_quarantined
        .len()
        .saturating_sub(blobs_deleted);

    let indices: Vec<u64> = pending.segments.iter().map(|s| s.index).collect();
    let bytes_reclaimed = core.journal.unlink_segments(&indices)?;

    let outcome = PruneOutcome {
        segments_unlinked: indices.len(),
        bytes_reclaimed,
        works_pruned: pending.residue.works.len(),
        commands_pruned: pending.residue.commands.len(),
        blobs_quarantined,
        blobs_deleted,
        blobs_missing: 0,
        blobs_rescued_before_delete,
        floor_seq_after: pending.floor_seq_after,
        truncated_by_cap: false,
    };

    core.commit(EventDraft::new(
        EventSource::new("daemon", "sergeant"),
        KIND_PRUNE_COMPLETED,
        serde_json::json!({
            "intent_seq": pending.intent_seq,
            "outcome": outcome,
            "floor_seq_after": pending.floor_seq_after,
            "completed_at_startup": true,
        }),
    ))?;
    core.flush()?;
    core.prune_pending = false;
    let _ = startup::persist_or_remove(None, data_dir);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::work::WorkState;

    fn work_row(id: &str, state: WorkState, last_seq: u64) -> WorkIndexRow {
        WorkIndexRow {
            id: id.to_string(),
            intent: "x".to_string(),
            state,
            integrity: None,
            created_at: "2026-01-01T00:00:00.000Z".to_string(),
            updated_at: "2026-01-01T00:00:00.000Z".to_string(),
            last_seq,
        }
    }

    // -------------------------------------------------------------
    // cap_seq
    // -------------------------------------------------------------

    #[test]
    fn cap_seq_is_zero_when_within_retention() {
        let mut idx = BTreeMap::new();
        for n in 1..=3 {
            idx.insert(
                format!("w{n}"),
                work_row(&format!("w{n}"), WorkState::Completed, n * 10),
            );
        }
        assert_eq!(cap_seq(&idx, 4), 0);
        assert_eq!(cap_seq(&idx, 3), 0);
    }

    /// N2's arithmetic core: the cap sits one below the N-th largest
    /// `last_seq`.
    #[test]
    fn the_cap_ranking_is_total_because_last_seq_is_unique() {
        let mut idx = BTreeMap::new();
        for (id, seq) in [("a", 10u64), ("b", 20), ("c", 30), ("d", 40), ("e", 50)] {
            idx.insert(id.to_string(), work_row(id, WorkState::Completed, seq));
        }
        // Retention 2: the two newest are last_seq 50 and 40; the cap sits
        // just below the 2nd largest (40) => 39.
        assert_eq!(cap_seq(&idx, 2), 39);
        // Retention 5 (== count): nothing past the cap.
        assert_eq!(cap_seq(&idx, 5), 0);
    }

    #[test]
    fn cap_seq_of_zero_retention_never_underflows_and_caps_nothing() {
        let mut idx = BTreeMap::new();
        idx.insert("a".to_string(), work_row("a", WorkState::Completed, 10));
        assert_eq!(cap_seq(&idx, 0), u64::MAX);
    }

    // -------------------------------------------------------------
    // retired_whole
    // -------------------------------------------------------------

    #[test]
    fn a_failed_work_with_no_run_is_retired_whole() {
        assert!(retired_whole(WorkState::Failed, None));
    }

    #[test]
    fn an_active_work_is_never_retired_whole() {
        assert!(!retired_whole(WorkState::Active, None));
    }

    // -------------------------------------------------------------
    // PruneResidue::merge
    // -------------------------------------------------------------

    fn pruned_work(id: &str) -> PrunedWorkRow {
        PrunedWorkRow {
            id: id.to_string(),
            intent: "x".to_string(),
            state: WorkState::Completed,
            integrity: None,
            created_at: "2026-01-01T00:00:00.000Z".to_string(),
            updated_at: "2026-01-01T00:00:00.000Z".to_string(),
            last_seq: 1,
            pruned_at: "2026-01-02T00:00:00.000Z".to_string(),
        }
    }

    #[test]
    fn merge_unions_works_and_commands_by_id() {
        let mut a = PruneResidue {
            works: vec![pruned_work("w1")],
            ..Default::default()
        };
        let b = PruneResidue {
            works: vec![pruned_work("w2")],
            ..Default::default()
        };
        a.merge(b);
        let ids: BTreeSet<String> = a.works.iter().map(|w| w.id.clone()).collect();
        assert_eq!(ids, BTreeSet::from(["w1".to_string(), "w2".to_string()]));
    }

    #[test]
    fn merge_keeps_the_higher_seq_capability_provenance() {
        let mut a = PruneResidue {
            capability_provenance: Some(AskWithdrawal {
                seq: 5,
                version: "1.0".to_string(),
            }),
            ..Default::default()
        };
        let lower = PruneResidue {
            capability_provenance: Some(AskWithdrawal {
                seq: 3,
                version: "0.9".to_string(),
            }),
            ..Default::default()
        };
        a.merge(lower);
        assert_eq!(a.capability_provenance.as_ref().unwrap().seq, 5);

        let higher = PruneResidue {
            capability_provenance: Some(AskWithdrawal {
                seq: 9,
                version: "2.0".to_string(),
            }),
            ..Default::default()
        };
        a.merge(higher);
        assert_eq!(a.capability_provenance.as_ref().unwrap().seq, 9);
    }

    // -------------------------------------------------------------
    // candidate_horizon
    // -------------------------------------------------------------

    fn bound(index: u64, first: u64, last: u64) -> SegmentBound {
        SegmentBound {
            index,
            path: std::path::PathBuf::from(format!("/dev/null/{index}")),
            first_seq: first,
            last_seq: last,
            bytes: 1024,
        }
    }

    fn policy(retention: u32) -> PrunePolicy {
        PrunePolicy {
            retention,
            source: PolicySource::Config,
        }
    }

    #[test]
    fn the_horizon_is_zero_when_nothing_is_eligible() {
        let bounds = vec![bound(1, 1, 100)];
        let registry = WorkRegistry::default();
        let (h, _) = candidate_horizon(&bounds, &registry, &FirstSeqIndex::new(), &policy(4));
        assert_eq!(h, 0);
    }

    /// I-W3-4, discovered exactly here by construction (not merely asserted
    /// from the spec): a Work whose *last* event sits in the currently-open
    /// segment must never make that segment a candidate, even when the Work
    /// is otherwise fully retired, past the cap, and non-straddling —
    /// because unlinking it would unlink the writer's own live segment.
    /// With only one segment ever written, nothing is ever prunable,
    /// regardless of how eligible its lone Work looks.
    #[test]
    fn the_writers_own_live_segment_is_never_a_horizon_candidate() {
        let bounds = vec![bound(1, 1, 10)];
        let mut registry = WorkRegistry::default();
        registry
            .work_index
            .insert("w1".to_string(), work_row("w1", WorkState::Completed, 10));
        let mut first_seq = FirstSeqIndex::new();
        first_seq.insert("w1".to_string(), 1);

        let (h, _) = candidate_horizon(&bounds, &registry, &first_seq, &policy(0));
        assert_eq!(
            h, 0,
            "the only segment there is must never be proposed, since it is always the live one"
        );

        // Once a second (now genuinely historical) segment exists, the
        // first one is fair game and the live (second) one still is not.
        let bounds = vec![bound(1, 1, 10), bound(2, 11, 11)];
        let (h, _) = candidate_horizon(&bounds, &registry, &first_seq, &policy(0));
        assert_eq!(h, 10, "the now-historical first segment must be reachable");
    }

    /// N1: a Work whose `last_seq` is above the horizon pins every segment
    /// containing any of its events.
    #[test]
    fn a_work_whose_last_seq_is_above_the_horizon_pins_its_segments() {
        let bounds = vec![bound(1, 1, 50), bound(2, 51, 100)];
        let mut registry = WorkRegistry::default();
        // w1 spans both segments (first_seq 10, last_seq 60): straddles
        // candidate 50, so nostraddle fails there; must fail at 100 too
        // (b < blocker not the issue here — it is retired), so straddling
        // is what should hold this to 0.
        registry
            .work_index
            .insert("w1".to_string(), work_row("w1", WorkState::Completed, 60));
        let mut first_seq = FirstSeqIndex::new();
        first_seq.insert("w1".to_string(), 10);

        let (h, _) = candidate_horizon(&bounds, &registry, &first_seq, &policy(1));
        assert_eq!(h, 0, "w1 straddles every candidate <= 100");
    }

    /// N2.
    #[test]
    fn a_work_inside_the_retention_cap_is_never_pruned() {
        let bounds = vec![bound(1, 1, 50), bound(2, 51, 100)];
        let mut registry = WorkRegistry::default();
        for (id, seq) in [("w1", 20u64), ("w2", 100)] {
            registry
                .work_index
                .insert(id.to_string(), work_row(id, WorkState::Completed, seq));
        }
        let mut first_seq = FirstSeqIndex::new();
        first_seq.insert("w1".to_string(), 15);
        first_seq.insert("w2".to_string(), 90);
        // Retention 2: both Works are within the cap, so cap_seq == 0.
        let (h, _) = candidate_horizon(&bounds, &registry, &first_seq, &policy(2));
        assert_eq!(h, 0);
    }

    /// N3: an unsettled terminal Work (surface, no teardown) pins the
    /// horizon and names itself as the blocker.
    #[test]
    fn an_unsettled_terminal_work_pins_the_horizon_and_names_itself_as_the_blocker() {
        use crate::runtime::projection::WorkRun;
        use crate::runtime::surface::WorkSurface;

        let bounds = vec![bound(1, 1, 50), bound(2, 51, 100)];
        let mut registry = WorkRegistry::default();
        registry.work_index.insert(
            "stuck".to_string(),
            work_row("stuck", WorkState::Completed, 20),
        );
        registry.runs.insert(
            "stuck".to_string(),
            WorkRun {
                surface: Some(WorkSurface {
                    work_id: "stuck".to_string(),
                    root: std::path::PathBuf::from("/nowhere"),
                    bindings: Vec::new(),
                }),
                ..Default::default()
            },
        );
        registry.work_index.insert(
            "clean".to_string(),
            work_row("clean", WorkState::Completed, 90),
        );
        let mut first_seq = FirstSeqIndex::new();
        first_seq.insert("stuck".to_string(), 10);
        first_seq.insert("clean".to_string(), 80);

        let (h, stall) = candidate_horizon(&bounds, &registry, &first_seq, &policy(1));
        assert_eq!(
            h, 0,
            "the unsettled Work at last_seq 20 blocks every candidate >= 20"
        );
        assert_eq!(stall.blocking_work_id.as_deref(), Some("stuck"));
    }

    #[test]
    fn stall_report_names_the_lowest_blocking_work() {
        let bounds = vec![bound(1, 1, 100)];
        let mut registry = WorkRegistry::default();
        registry
            .work_index
            .insert("a".to_string(), work_row("a", WorkState::Active, 10));
        registry
            .work_index
            .insert("b".to_string(), work_row("b", WorkState::Active, 50));
        let stall = stall_report(
            &bounds,
            &registry,
            &FirstSeqIndex::new(),
            &policy(1),
            std::time::SystemTime::now(),
        );
        assert_eq!(stall.blocking_work_id.as_deref(), Some("a"));
    }

    // -------------------------------------------------------------
    // A real cycle over a real journal, via `Core` directly (no daemon).
    // -------------------------------------------------------------

    /// A minimal `Core` over a fresh journal that rotates every event into
    /// its own segment — small enough that a two-event Work already spans
    /// two segments, without needing a production-scale fixture.
    fn tiny_core(dir: &std::path::Path) -> crate::api::Core {
        let journal = crate::runtime::journal::Journal::open_with(dir, 1).expect("open");
        let registry = crate::runtime::projection::work_registry_projection();
        let (events_tx, _rx) = tokio::sync::broadcast::channel(16);
        crate::api::Core::new(journal, registry, events_tx)
    }

    fn commit(
        core: &mut crate::api::Core,
        source: crate::domain::event::EventSource,
        kind: &str,
        work_id: Option<&str>,
        payload: serde_json::Value,
    ) -> crate::domain::event::Event {
        let mut draft = crate::domain::event::EventDraft::new(source, kind, payload);
        if let Some(id) = work_id {
            draft = draft.with_work_id(id);
        }
        core.commit(draft).expect("commit")
    }

    fn daemon_source() -> crate::domain::event::EventSource {
        crate::domain::event::EventSource::new("daemon", "test")
    }

    fn submit_and_complete(core: &mut crate::api::Core, work_id: &str) {
        commit(
            core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some(work_id),
            serde_json::json!({"work": {
                "id": work_id,
                "intent": "prune unit fixture",
                "state": "pending",
                "created_by": "test",
                "created_at": "2026-01-01T00:00:00.000Z",
            }}),
        );
        commit(
            core,
            daemon_source(),
            crate::domain::work::KIND_WORK_COMPLETED,
            Some(work_id),
            serde_json::json!({}),
        );
    }

    /// Build a two-segment, one-Work fixture and commit its `prune.intent`
    /// (step 1 of `run`'s cycle) — then stop, exactly as a crash would.
    /// Every `crash_window_f*` test starts here and diverges only in what
    /// it does to the on-disk segments *before* calling
    /// `complete_interrupted`.
    fn build_and_commit_intent(core: &mut crate::api::Core, dir: &std::path::Path) -> PrunePlan {
        submit_and_complete(core, "w1");
        // I-W3-4: the writer's own live segment is never a prune target, so
        // a trailing, unrelated event pushes it there — otherwise w1's own
        // completing event would still be sitting in the (structurally
        // excluded) live segment and nothing here would be prunable at all.
        commit(
            core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("anchor"),
            serde_json::json!({"work": {
                "id": "anchor", "intent": "keep the writer off w1's segments",
                "state": "pending", "created_by": "test",
                "created_at": "2026-01-01T00:00:00.000Z",
            }}),
        );
        core.flush().expect("flush");

        let bounds = core.journal.segment_bounds().expect("bounds");
        let policy = PrunePolicy {
            retention: 0,
            source: PolicySource::Config,
        };
        let (candidate, _) = candidate_horizon(
            &bounds,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        );
        assert!(candidate > 0, "the fixture must actually be prunable");
        let plan = plan(
            dir,
            &bounds,
            candidate,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        )
        .expect("plan")
        .expect("something must be prunable");
        assert_eq!(plan.residue.works.len(), 1);
        assert_eq!(
            plan.segments.len(),
            2,
            "the fixture must span exactly two segments for the crash-window tests to mean anything"
        );

        let record = PruneIntentRecord {
            intent_seq: 0,
            policy: plan.policy,
            horizon_seq: plan.horizon_seq,
            floor_seq_before: 1,
            floor_seq_after: plan.horizon_seq + 1,
            segments: plan
                .segments
                .iter()
                .map(|b| PruneSegment {
                    index: b.index,
                    first_seq: b.first_seq,
                    last_seq: b.last_seq,
                    bytes: b.bytes,
                })
                .collect(),
            residue: plan.residue.clone(),
            carried_forward: plan.carried_forward.clone(),
            condemn: plan.condemn.iter().map(|r| r.hex().to_string()).collect(),
            delete_quarantined: plan.delete_quarantined.clone(),
            started_at_startup: false,
        };
        core.commit(EventDraft::new(
            EventSource::new("daemon", "sergeant"),
            KIND_PRUNE_INTENT,
            record.to_payload(),
        ))
        .expect("commit intent");
        core.flush().expect("flush intent");
        assert!(
            core.registry.state().pending_prune.is_some(),
            "the intent must be pending — nothing has completed it yet"
        );
        plan
    }

    /// N7 / F1: a crash after `prune.intent` is fsynced, before any rename or
    /// unlink, is completed in full at the next start from the intent's own
    /// recorded targets — and a second completion attempt over the already-
    /// finished cycle is a true no-op (nothing further appended).
    #[test]
    fn crash_window_f1_before_quarantine_completes_at_next_start() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = tiny_core(dir.path());
        let plan = build_and_commit_intent(&mut core, dir.path());
        let next_seq_after_intent = core.journal.next_seq();

        // Next start: complete it.
        complete_interrupted(&mut core, dir.path()).expect("complete_interrupted");
        assert!(
            core.registry.state().pending_prune.is_none(),
            "the cycle must be acknowledged after completion"
        );
        assert!(
            core.registry.state().pruned_works.contains_key("w1"),
            "w1's residue must have been applied"
        );
        let bounds_after = core.journal.segment_bounds().expect("bounds");
        assert!(
            bounds_after
                .iter()
                .all(|b| !plan.segments.iter().any(|s| s.index == b.index)),
            "every planned segment must actually be gone"
        );
        assert_eq!(
            core.journal.next_seq(),
            next_seq_after_intent + 1,
            "exactly one completion event must have been appended"
        );

        // The honesty check: completing an already-finished cycle again
        // appends nothing further.
        complete_interrupted(&mut core, dir.path()).expect("second complete_interrupted");
        assert_eq!(
            core.journal.next_seq(),
            next_seq_after_intent + 1,
            "a completion with no pending intent must append nothing"
        );
    }

    /// N9 / F4: a crash mid-unlink (only the first of the two target
    /// segments actually removed) leaves a journal `complete_interrupted`
    /// can still finish — `unlink_segments` tolerates the already-gone
    /// member and removes the rest — and the result replays cleanly from
    /// its new floor.
    #[test]
    fn crash_window_f4_mid_unlink_leaves_a_journal_that_replays_from_its_new_floor() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = tiny_core(dir.path());
        let plan = build_and_commit_intent(&mut core, dir.path());

        // Simulate the crash: only the oldest of the two planned segments
        // was actually unlinked before the process died.
        let oldest = plan.segments.iter().map(|s| s.index).min().unwrap();
        let journal_dir = dir.path().join("journal");
        std::fs::remove_file(journal_dir.join(format!("{oldest:08}.ndjson")))
            .expect("simulate the partial unlink");

        complete_interrupted(&mut core, dir.path()).expect("complete_interrupted");
        assert!(core.registry.state().pending_prune.is_none());
        assert!(core.registry.state().pruned_works.contains_key("w1"));
        let bounds_after = core.journal.segment_bounds().expect("bounds");
        assert!(
            bounds_after
                .iter()
                .all(|b| !plan.segments.iter().any(|s| s.index == b.index)),
            "the remaining planned segment must also be gone after the completion finishes it"
        );

        // The journal must still replay cleanly from its new floor.
        let replayed: Result<Vec<_>, _> = core
            .journal
            .replay_from_floor()
            .expect("replay_from_floor")
            .collect();
        assert!(replayed.is_ok(), "got {replayed:?}");
    }

    /// N10 / F5: a crash after the last unlink but before `prune.completed`
    /// is appended leaves no segment `complete_interrupted` could ever
    /// re-derive the residue from — which is exactly why the completion is
    /// appended from the *intent's* own carried residue, not recomputed.
    #[test]
    fn crash_window_f5_after_unlink_appends_the_completion_from_the_intents_residue() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = tiny_core(dir.path());
        let plan = build_and_commit_intent(&mut core, dir.path());

        // Simulate the crash: every planned segment is already gone, but no
        // completion was ever appended.
        let journal_dir = dir.path().join("journal");
        for segment in &plan.segments {
            std::fs::remove_file(journal_dir.join(format!("{:08}.ndjson", segment.index)))
                .expect("simulate the finished unlink");
        }

        complete_interrupted(&mut core, dir.path()).expect("complete_interrupted");
        assert!(
            core.registry.state().pending_prune.is_none(),
            "the completion must have been appended from the intent's own residue"
        );
        assert!(
            core.registry.state().pruned_works.contains_key("w1"),
            "w1's row must come from the intent's carried residue — there is nothing else \
             left to recompute it from"
        );
    }

    /// N20: pruning the Work that recorded the ask-grammar withdrawal must
    /// not silently drop the watermark — the residue must carry it forward,
    /// with its *original* seq (not the prune event's), so "higher seq
    /// wins" keeps meaning what it means.
    #[test]
    fn the_ask_withdrawal_survives_the_prune_of_the_work_that_recorded_it() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = tiny_core(dir.path());
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("w1"),
            serde_json::json!({"work": {
                "id": "w1", "intent": "ask fixture", "state": "pending",
                "created_by": "test", "created_at": "2026-01-01T00:00:00.000Z",
            }}),
        );
        let withdrawal_event = commit(
            &mut core,
            EventSource::new("backend", "claude"),
            "conversation.turn.grammar_unmeasured",
            Some("w1"),
            serde_json::json!({"capability": "ask", "version": "2.1.226"}),
        );
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_COMPLETED,
            Some("w1"),
            serde_json::json!({}),
        );
        // I-W3-4: push the writer's live segment off of w1's own — see
        // `build_and_commit_intent`'s identical comment.
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("anchor"),
            serde_json::json!({"work": {
                "id": "anchor", "intent": "keep the writer off w1's segments",
                "state": "pending", "created_by": "test",
                "created_at": "2026-01-01T00:00:00.000Z",
            }}),
        );
        core.flush().expect("flush");

        let bounds = core.journal.segment_bounds().expect("bounds");
        let policy = PrunePolicy {
            retention: 0,
            source: PolicySource::Config,
        };
        let (candidate, _) = candidate_horizon(
            &bounds,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        );
        let plan = plan(
            dir.path(),
            &bounds,
            candidate,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        )
        .expect("plan")
        .expect("something must be prunable");

        let watermark = plan
            .residue
            .capability_provenance
            .as_ref()
            .expect("the withdrawal must be carried in the residue");
        assert_eq!(watermark.seq, withdrawal_event.seq);
        assert_eq!(watermark.version, "2.1.226");

        let outcome = run(&mut core, dir.path(), plan, false).expect("run");
        assert!(outcome.segments_unlinked > 0);
        assert!(
            core.registry.state().pruned_works.contains_key("w1"),
            "w1 must actually have been pruned"
        );
    }

    /// N4: an unknown non-work-scoped event kind pins its segment — Q5's
    /// allowlist as a mechanism, not a comment. A later, otherwise-eligible
    /// Work stays retained because the pin lowers the horizon back below
    /// it; the Work *before* the pin is unaffected.
    #[test]
    fn an_unknown_non_work_scoped_kind_pins_its_segment() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = tiny_core(dir.path());
        submit_and_complete(&mut core, "w1");
        // Not in `NON_WORK_ALLOWLIST` and carries no `work_id` — exactly
        // the shape a future wave's new event kind could add.
        commit(
            &mut core,
            daemon_source(),
            "mystery.event",
            None,
            serde_json::json!({}),
        );
        submit_and_complete(&mut core, "w2");
        // Push the writer off w2's own segment (I-W3-4).
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("anchor"),
            serde_json::json!({"work": {
                "id": "anchor", "intent": "keep the writer off w2's segments",
                "state": "pending", "created_by": "test",
                "created_at": "2026-01-01T00:00:00.000Z",
            }}),
        );
        core.flush().expect("flush");

        let bounds = core.journal.segment_bounds().expect("bounds");
        let policy = PrunePolicy {
            retention: 0,
            source: PolicySource::Config,
        };
        let (candidate, _) = candidate_horizon(
            &bounds,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        );
        // Phase A alone (registry-only) does not see the pin — both Works
        // look fully eligible to it.
        assert!(
            candidate >= core.registry.state().work_index["w2"].last_seq,
            "Phase A must not know about the pin yet"
        );

        let plan = plan(
            dir.path(),
            &bounds,
            candidate,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        )
        .expect("plan")
        .expect("w1 must still be prunable even though w2 is pinned away");

        let pruned_ids: std::collections::BTreeSet<&str> =
            plan.residue.works.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(
            pruned_ids,
            std::collections::BTreeSet::from(["w1"]),
            "only the Work entirely before the pin may be pruned"
        );
        assert_eq!(
            plan.stall.pinning_kind.as_deref(),
            Some("mystery.event"),
            "the stall report must name the pinning kind"
        );
    }

    /// N17: a retry that lands between planning (outside the guard) and
    /// committing (inside it) must abort the cycle rather than delete a
    /// Work whose eligibility the live registry no longer agrees with —
    /// `run` re-validates immediately before committing the intent (§10.2).
    #[test]
    fn a_retry_landing_between_plan_and_commit_aborts_the_cycle() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = tiny_core(dir.path());
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("w1"),
            serde_json::json!({"work": {
                "id": "w1", "intent": "retry-race fixture", "state": "pending",
                "created_by": "test", "created_at": "2026-01-01T00:00:00.000Z",
            }}),
        );
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_FAILED,
            Some("w1"),
            serde_json::json!({}),
        );
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("anchor"),
            serde_json::json!({"work": {
                "id": "anchor", "intent": "keep the writer off w1's segments",
                "state": "pending", "created_by": "test",
                "created_at": "2026-01-01T00:00:00.000Z",
            }}),
        );
        core.flush().expect("flush");

        let bounds = core.journal.segment_bounds().expect("bounds");
        let policy = PrunePolicy {
            retention: 0,
            source: PolicySource::Config,
        };
        let (candidate, _) = candidate_horizon(
            &bounds,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        );
        let plan = plan(
            dir.path(),
            &bounds,
            candidate,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        )
        .expect("plan")
        .expect("a Failed, retired-whole Work must be plannable");
        assert_eq!(plan.residue.works.len(), 1);

        // The race: a retry lands on the live registry (`failed -> active`)
        // between planning and committing — `WorkState::for_event_kind`'s
        // `KIND_WORK_STARTED` mapping.
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_STARTED,
            Some("w1"),
            serde_json::json!({}),
        );
        core.flush().expect("flush the retry");

        let journal_next_seq_before = core.journal.next_seq();
        let bounds_before: Vec<_> = core
            .journal
            .segment_bounds()
            .expect("bounds")
            .iter()
            .map(|b| b.index)
            .collect();

        let outcome = run(&mut core, dir.path(), plan, false).expect("run must not error");
        assert_eq!(
            outcome,
            PruneOutcome::default(),
            "a stale plan must be a no-op, not a deletion planned from a disagreement"
        );
        assert!(
            core.prune_pending,
            "the cycle must be re-armed so the next tick re-plans"
        );
        assert!(
            core.registry.state().pending_prune.is_none(),
            "no intent may have been committed for a plan that failed re-validation"
        );
        assert!(
            !core.registry.state().pruned_works.contains_key("w1"),
            "the retried Work must not have been pruned"
        );
        assert_eq!(
            core.journal.next_seq(),
            journal_next_seq_before,
            "nothing may have been appended"
        );
        let bounds_after: Vec<_> = core
            .journal
            .segment_bounds()
            .expect("bounds")
            .iter()
            .map(|b| b.index)
            .collect();
        assert_eq!(
            bounds_after, bounds_before,
            "nothing may have been unlinked"
        );
    }
}
