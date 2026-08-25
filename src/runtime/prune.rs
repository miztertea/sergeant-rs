//! W3: the prune engine — bounded retention made real (issue #17's rulings
//! record, `sergeant-rs-workspace's knowledge/evidence/reference/foundation-2026-08-21.md` W3 section).
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
/// is not here pins its segment (proved directly by
/// `an_unknown_non_work_scoped_kind_pins_its_segment`, below). A kind is
/// meant to enter this list only with its own replay-equivalence proof, one
/// test per kind (the wave spec's own `tests/w3_allowlist_equivalence.rs`
/// suite) — this landing does not add that dedicated file; the nine kinds
/// already here are the ones §2 itself argues are safe, each with the
/// one-line equivalence claim in its own comment below, but not yet each
/// pinned by its own named test the way the spec's §13.1 step 6 asks for.
pub const NON_WORK_ALLOWLIST: &[&str] = &[
    crate::daemon::KIND_DAEMON_STARTED, // no registry effect at all
    crate::daemon::KIND_DAEMON_STOPPED, // no registry effect at all
    crate::daemon::KIND_BACKEND_PROBED, // no registry effect at all
    // sets `admission_paused`, force-cleared unconditionally at every
    // startup before the descriptor is published, so the replayed value is
    // never load-bearing.
    crate::daemon::KIND_ADMISSION_PAUSED,
    crate::daemon::KIND_ADMISSION_RESUMED, // same
    KIND_COMMAND_ACCEPTED,                 // ledger key survives in `pruned_commands` (Q8)
    KIND_COMMAND_REJECTED,                 // same
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
    /// Hexes the *previous* cycle quarantined that a surviving or
    /// since-committed event has referenced again — moved back to their
    /// live content address instead of deleted (I-W3-6's other half: a
    /// dedup adoption that lands between one cycle's mark scan and the
    /// *next* cycle's deferred delete, with nobody having called
    /// `BlobStore::get`/`put` in between to trigger the ordinary rescue).
    #[serde(default)]
    pub rescue_quarantined: Vec<String>,
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
    /// Absent from any intent committed before this field existed —
    /// defaults to empty, which is always a safe (if slightly stale)
    /// answer for a record this old (§6.2's "record, apply nothing" applies
    /// symmetrically to a field that is simply missing).
    #[serde(default)]
    rescue_quarantined: Vec<String>,
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
            rescue_quarantined: payload.rescue_quarantined,
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
            rescue_quarantined: self.rescue_quarantined.clone(),
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
    /// Blobs the previous cycle quarantined, to delete now — already
    /// excluded from anything the surviving-side scan found still
    /// referenced (I-W3-6; see [`PrunePlan::rescue_quarantined`]) and from
    /// anything *this* cycle condemns afresh (see
    /// [`PrunePlan::defer_quarantined`]).
    pub delete_quarantined: Vec<String>,
    /// Blobs the previous cycle quarantined that this cycle's own mark scan
    /// condemns **again** — deleted by neither this cycle nor rescued by it,
    /// but re-marked and deferred to the next one (A5's two-phase
    /// quarantine).
    ///
    /// Condemning and deleting the same hex in one cycle collapses the
    /// deferral window to nothing, which is the only thing standing between
    /// an in-flight `BlobStore::put` dedup-hit — one that has already
    /// rescued the content back to its live address but whose referencing
    /// event has not been committed yet, so neither the mark scan nor the
    /// guard-held top-up can see it — and a destroyed live blob with a
    /// dangling `b3:` reference pointing at it.
    ///
    /// Not carried on [`PruneIntentRecord`] and not needed there: every hex
    /// here is by construction a member of `condemn`, which *is* recorded,
    /// and which is what the registry installs as the next cycle's
    /// `quarantined_blobs` — so the deferral survives a crash without a
    /// field of its own.
    pub defer_quarantined: Vec<String>,
    /// Blobs the previous cycle quarantined that the surviving-side scan
    /// found referenced by a retained event — a dedup adoption that landed
    /// between that cycle's mark scan and this one's, with no live
    /// `get`/`put` in between to trigger the ordinary rescue. `run` moves
    /// these back to their live content address instead of deleting them.
    pub rescue_quarantined: Vec<String>,
    /// The highest seq the surviving-side scan actually reached when it ran
    /// (`horizon` when nothing needed that scan at all) — `run`'s top-up
    /// reads events after this before quarantining/deleting, closing the
    /// window between this plan and the guard that commits it. Bounded by
    /// the answer the off-guard scan already reached, not by the whole
    /// retained journal (§5.1).
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
    /// Blobs moved back to their live content address because the
    /// surviving-side scan (or the guard-held top-up) found them
    /// referenced by a retained event — [`PruneIntentRecord::rescue_quarantined`]'s
    /// count, distinct from `blobs_rescued_before_delete` (which counts an
    /// *ordinary* `get`/`put` rescue that already happened by the time this
    /// cycle looked).
    #[serde(default)]
    pub blobs_rescued_by_reference: usize,
    /// The floor after this cycle.
    pub floor_seq_after: u64,
    /// More was eligible than [`PRUNE_MAX_WORKS_PER_CYCLE`] allowed; [`run`]
    /// re-arms `prune_pending` from this so the next tick continues the
    /// backlog drain rather than waiting on an unrelated rotation (§7.4).
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
///
/// **Known simplicity debt, not a correctness gap (spec-fidelity R2):**
/// [`crate::runtime::startup::horizon`]'s own doc comment says this sweep
/// (nostraddle + retired, evaluated over the same sorted-by-first-seq
/// running-max shape) is "the same predicate W3's prune horizon needs...
/// land the shape here so W3 cites it rather than writing a second one."
/// This function narrows that shape further — the retention cap
/// (`cap_seq`) and the batch cap (`PRUNE_MAX_WORKS_PER_CYCLE`) have no
/// counterpart in `startup::horizon` at all — and the two sweeps are
/// maintained today as two independently-evolving copies rather than one
/// shared helper the way the doc comment asks. Not reused directly:
/// `startup::horizon` is exercised by every W2 cache test in production
/// today, and reshaping it to accept the two extra predicates this module
/// needs is real, separate surgery on already-shipped, load-bearing code —
/// risk this landing did not take on. [`sweep_horizon`], directly below, is
/// this module's own sole copy of the shape; a future wave that wants the
/// citation to be literal should factor a shared sweep both call, not
/// merely note the debt again here.
pub fn candidate_horizon(
    bounds: &[SegmentBound],
    registry: &WorkRegistry,
    first_seq: &FirstSeqIndex,
    policy: &PrunePolicy,
) -> (u64, PruneStall) {
    sweep_horizon(bounds, registry, first_seq, policy, u64::MAX)
}

/// The sweep [`candidate_horizon`] runs, factored out so [`plan`]'s
/// allowlist/pair pin (§6.5) can re-run the *same* `nostraddle`/`retired`/
/// `capped`/batch-cap predicates restricted to `b <= ceiling`, rather than
/// merely taking the largest segment boundary below the pin and trusting
/// the residue cross-check to catch a straddling Work the sweep itself
/// would have refused. `ceiling = u64::MAX` (via [`candidate_horizon`])
/// changes nothing versus every candidate being eligible on its own terms.
fn sweep_horizon(
    bounds: &[SegmentBound],
    registry: &WorkRegistry,
    first_seq: &FirstSeqIndex,
    policy: &PrunePolicy,
    ceiling: u64,
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
        .filter(|&last| last <= ceiling)
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

    for event in crate::runtime::journal::Journal::replay_data_dir_from_floor(data_dir)? {
        let event = event?;
        if event.seq > ceiling {
            break;
        }

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

    // Lower the horizon until the range this cycle would actually condemn
    // is pin-free *on its own terms*, re-scanning at each lowered height.
    //
    // One pass is not enough, because the pin predicate is not monotone in
    // the ceiling: `scan_condemned_range`'s `open_intents` arm is
    // ceiling-*dependent* by construction. A `prune.intent` at seq `S`
    // whose matching `prune.completed` sits at seq `C` reads as a closed
    // pair at a ceiling of `candidate` (both in range) and as an unpaired,
    // segment-pinning intent at a lowered ceiling `h` with `S <= h < C`.
    // Consulting only the first pass's answer would therefore unlink the
    // segment holding a still-open intent — exactly what §6.5 forbids, and
    // exactly what leaves `pending_prune` permanently `Some` with nothing
    // left to complete it from.
    //
    // Terminates: an iteration only runs when the current scan found a pin
    // at or below the current horizon, and it sets the next horizon
    // strictly below that pin's own segment — so `horizon` strictly
    // decreases, bounded below by the `0` that returns `None`. Each
    // iteration costs one pass over a range strictly smaller than the last,
    // and iterating at all requires a pin — the uncommon case; the ordinary
    // cycle still pays exactly one condemned-range pass, as before.
    let mut horizon = candidate;
    let mut scan = scan_condemned_range(data_dir, candidate, registry)?;
    // The pin that actually bound the final horizon — the last one
    // consulted, which is the one a stall report should name.
    let mut effective_pin_seq: Option<u64> = None;
    while let Some(pin_seq) = scan.lowest_pin_seq {
        effective_pin_seq = Some(pin_seq);
        let pin_first_seq = bounds
            .iter()
            .find(|b| b.first_seq <= pin_seq && pin_seq <= b.last_seq)
            .map(|b| b.first_seq)
            .unwrap_or(pin_seq);
        // §7.1: "the largest *admissible* candidate below the pin" — not
        // merely the largest segment boundary below it. A lower horizon is
        // not automatically admissible: `nostraddle` is not monotone in
        // `b`, so a Work straddling this new, smaller boundary can make a
        // candidate that looked fine at `candidate`'s height inadmissible
        // down here. Re-running the same sweep restricted to
        // `b < pin_first_seq` is what actually re-checks
        // `nostraddle`/`retired`/`capped` at the lowered height, rather
        // than relying on the residue cross-check below to reject a
        // straddling plan after the fact (that check still runs — belt and
        // braces — but this is what keeps a straddling pin from producing a
        // permanent, misdiagnosed `ResidueMismatch` stall every cycle
        // instead of a correctly-computed, possibly-smaller-but-legal
        // horizon).
        let ceiling = pin_first_seq.saturating_sub(1);
        let (admissible, _) = sweep_horizon(bounds, registry, first_seq, policy, ceiling);
        horizon = admissible.min(horizon);
        if horizon == 0 {
            return Ok(None);
        }
        // Everything the previous pass collected may include events above
        // the new, smaller ceiling — rescan exactly the (smaller) range
        // this cycle would now condemn, and consult *its* pin next.
        scan = scan_condemned_range(data_dir, horizon, registry)?;
    }

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

    // The surviving-side scan: needed whenever there is something a live
    // reference could save — either a fresh condemn candidate (the
    // long-standing check) or a hex the *previous* cycle already
    // quarantined and this cycle would otherwise delete (I-W3-6's other
    // half, §7.1's deferred-delete step). Tracks the highest seq it
    // actually reached, which becomes `scan_through`: bounded by the answer
    // this off-guard pass reached, not by the whole retained journal (§5.1)
    // — `run`'s guard-held top-up only has to re-read from here forward,
    // not from `horizon` forward.
    let mut surviving_refs: BTreeSet<BlobRef> = BTreeSet::new();
    let mut surviving_scan_through = horizon;
    let needs_surviving_scan =
        !scan.condemn_refs.is_empty() || !registry.quarantined_blobs.is_empty();
    if needs_surviving_scan {
        for event in crate::runtime::journal::Journal::replay_data_dir_from_floor(data_dir)? {
            let event = event?;
            if event.seq <= horizon {
                continue;
            }
            surviving_scan_through = surviving_scan_through.max(event.seq);
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

    // §5.2/I-W3-6 and A5's two-phase quarantine: every hex the *previous*
    // cycle quarantined is resolved here to exactly one of three outcomes,
    // because the next cycle's `registry.quarantined_blobs` is about to be
    // replaced wholesale by *this* cycle's `condemn` set — anything left
    // unresolved would never be revisited again.
    //
    // 1. **Rescue** — a surviving event references it. A dedup adoption
    //    landed between that cycle's mark scan and this one's, with no live
    //    `get`/`put` in between to have rescued it the ordinary way; `run`
    //    moves it back to its live content address.
    // 2. **Defer** — *this* cycle's own mark scan condemns it again. It is
    //    being re-marked right now (step 2 finds it already in `.pruned/`
    //    and reports `Ok(false)`), it rides into the next cycle's
    //    `quarantined_blobs` on `condemn`, and *that* cycle decides its
    //    fate. Deleting it here instead would collapse the two-phase
    //    quarantine to a single phase for this hex: mark and sweep in one
    //    guard hold, with no window at all for a concurrent
    //    `BlobStore::put` dedup-hit — whose referencing event is not
    //    committed yet, and so is invisible to both the mark scan and
    //    `run`'s guard-held top-up — to have its content survive. That is a
    //    destroyed live blob and a dangling `b3:` ref, which is precisely
    //    the failure the deferral exists to prevent.
    // 3. **Delete** — neither of the above: untouched since the previous
    //    cycle marked it, which is what a full deferral window having
    //    elapsed with nobody claiming it looks like.
    //
    // (1) and (2) cannot both apply: `condemn` already has every
    // surviving-side reference subtracted from it just above.
    let surviving_hexes: BTreeSet<&str> = surviving_refs.iter().map(BlobRef::hex).collect();
    let condemned_hexes: BTreeSet<&str> = condemn.iter().map(BlobRef::hex).collect();
    let mut rescue_quarantined: Vec<String> = Vec::new();
    let mut defer_quarantined: Vec<String> = Vec::new();
    let mut delete_quarantined: Vec<String> = Vec::new();
    for hex in &registry.quarantined_blobs {
        if surviving_hexes.contains(hex.as_str()) {
            rescue_quarantined.push(hex.clone());
        } else if condemned_hexes.contains(hex.as_str()) {
            defer_quarantined.push(hex.clone());
        } else {
            delete_quarantined.push(hex.clone());
        }
    }

    let (_, mut stall) = candidate_horizon(bounds, registry, first_seq, policy);
    stall.horizon_seq = horizon;
    stall.pinning_seq = effective_pin_seq;
    if let Some(pin_seq) = effective_pin_seq {
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
        delete_quarantined,
        defer_quarantined,
        rescue_quarantined,
        scan_through: surviving_scan_through,
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
    // scan and this hold could have referenced a blob the plan condemned,
    // or re-referenced a hex the *previous* cycle already quarantined
    // (I-W3-6's other half — the same window, on the deferred-delete side).
    // Closing this window is what makes quarantine+rescue a second line of
    // defence rather than the only one. Skipped entirely when there is
    // nothing either list could still be wrong about, so a plan with no
    // blobs in play never pays for a read here at all.
    if !plan.condemn.is_empty() || !plan.delete_quarantined.is_empty() {
        for event in core.events_after(plan.scan_through)? {
            for r in blob::refs_in_event(&event) {
                plan.condemn.remove(&r);
                if let Some(pos) = plan.delete_quarantined.iter().position(|h| h == r.hex()) {
                    plan.rescue_quarantined
                        .push(plan.delete_quarantined.remove(pos));
                }
                // A deferred hex rides into the next cycle's quarantined
                // set only *because* it is in `condemn` — which the line
                // above may have just removed it from. Left in neither
                // list it would sit in `.pruned/` with nothing scheduled to
                // revisit it, so resolve it the way the new reference asks:
                // rescue it back to its live address now.
                if let Some(pos) = plan.defer_quarantined.iter().position(|h| h == r.hex()) {
                    plan.rescue_quarantined
                        .push(plan.defer_quarantined.remove(pos));
                }
            }
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
        rescue_quarantined: plan.rescue_quarantined.clone(),
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

    // Step 2 (T3): quarantine. Includes `plan.defer_quarantined` by
    // construction (it is a subset of `condemn`); for those, the live path
    // is normally already absent and `quarantine` reports its idempotent
    // `Ok(false)` — unless a `put` dedup-hit rescued the content back to
    // its live address since the previous cycle, which is exactly the case
    // the deferral is protecting and exactly the case that re-marks it here
    // for the *next* cycle to sweep.
    let mut blobs_quarantined = 0usize;
    for hex in &record.condemn {
        let blob_ref: BlobRef = format!("b3:{hex}").parse()?;
        if blobs.quarantine(&blob_ref)? {
            blobs_quarantined += 1;
        }
    }

    // Step 3a (T5's live half, I-W3-6): rescue back to the live content
    // address any hex the mark scan or this guard's own top-up found
    // referenced by a surviving event since the previous cycle quarantined
    // it — before the deferred delete below, from the disjoint partition
    // `plan` already computed, so together the two steps account for every
    // hex the previous cycle quarantined exactly once.
    let mut blobs_rescued_by_reference = 0usize;
    for hex in &record.rescue_quarantined {
        if blobs.rescue_quarantined_hex(hex)? {
            blobs_rescued_by_reference += 1;
        }
    }

    // Step 3b (T5): the previous cycle's deferred delete.
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
        blobs_rescued_by_reference,
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
    // §7.4: when the batch cap truncated this cycle's target set, the
    // remainder is left for the next cycle and `prune_pending` is re-armed
    // so the next tick continues rather than waiting for an unrelated
    // rotation — `outcome.truncated_by_cap` is exactly the signal `plan`
    // already computed for this.
    core.prune_pending = outcome.truncated_by_cap;

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
    let mut blobs_rescued_by_reference = 0usize;
    for hex in &pending.rescue_quarantined {
        if blobs.rescue_quarantined_hex(hex)? {
            blobs_rescued_by_reference += 1;
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
        blobs_rescued_by_reference,
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

    /// Re-open `dir`'s journal and fold it from its **current floor** into a
    /// fresh registry — the daemon's own full-replay start, in miniature.
    ///
    /// This is how the crash-window tests get a `Core` whose `pending_prune`
    /// reflects the journal *as it is on disk right now*, rather than the
    /// in-memory state of a process that has already folded its own
    /// completion. `Projection::resumed(.., floor - 1, ..)` rather than
    /// `work_registry_projection()` because a journal a prune has already
    /// cut no longer starts at seq 1 (A1).
    fn refold_core(dir: &std::path::Path) -> crate::api::Core {
        let journal = crate::runtime::journal::Journal::open_with(dir, 1).expect("re-open journal");
        let floor = journal.floor_seq().expect("floor_seq").unwrap_or(1);
        let mut registry = crate::runtime::projection::Projection::resumed(
            crate::runtime::projection::WorkRegistry::default(),
            floor - 1,
            crate::runtime::projection::work_registry_reducer,
        );
        registry
            .catch_up(journal.replay_from_floor().expect("replay_from_floor"))
            .expect("fold the journal from its floor");
        let (events_tx, _rx) = tokio::sync::broadcast::channel(16);
        crate::api::Core::new(journal, registry, events_tx)
    }

    /// Un-write the newest segment — which, in a `tiny_core` journal (one
    /// event per segment), holds exactly one event.
    ///
    /// Used to model "the completion's own event never landed": the crash
    /// window that sits *after* every physical effect of a completion
    /// (quarantine, deferred delete, unlink) and *before* the
    /// `prune.completed` append that acknowledges them. Asserts the kind it
    /// is removing, so a fixture that drifts fails loudly here instead of
    /// quietly testing something else.
    fn unwrite_newest_event(dir: &std::path::Path, expect_kind: &str) {
        let journal_dir = dir.join("journal");
        let mut segments: Vec<std::path::PathBuf> = std::fs::read_dir(&journal_dir)
            .expect("read journal dir")
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|ext| ext == "ndjson"))
            .collect();
        segments.sort();
        let newest = segments.last().expect("at least one segment");
        let text = std::fs::read_to_string(newest).expect("read the newest segment");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(
            lines.len(),
            1,
            "this helper assumes a one-event-per-segment fixture"
        );
        let event: Event = serde_json::from_str(lines[0]).expect("parse the newest event");
        assert_eq!(
            event.kind, expect_kind,
            "the newest event must be the one this crash window un-writes"
        );
        std::fs::remove_file(newest).expect("un-write the newest event");
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
            rescue_quarantined: plan.rescue_quarantined.clone(),
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

    /// Like [`build_and_commit_intent`], but `w1`'s completion references a
    /// real, previously-written blob — so the intent's own `condemn` is
    /// non-empty and F2/F3 (the blob-side crash windows) have something to
    /// crash mid-way through.
    fn build_and_commit_intent_with_blob(
        core: &mut crate::api::Core,
        dir: &std::path::Path,
    ) -> (PrunePlan, BlobRef) {
        let blob_ref = BlobStore::open(dir)
            .expect("open blob store")
            .put(b"crash window fixture blob")
            .expect("put");
        let hex = blob_ref.hex().to_string();

        commit(
            core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("w1"),
            serde_json::json!({"work": {
                "id": "w1", "intent": "crash window blob fixture", "state": "pending",
                "created_by": "test", "created_at": "2026-01-01T00:00:00.000Z",
            }}),
        );
        commit(
            core,
            daemon_source(),
            crate::domain::work::KIND_WORK_COMPLETED,
            Some("w1"),
            serde_json::json!({"result_blob": format!("b3:{hex}")}),
        );
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
        assert!(
            plan.condemn.iter().any(|r| r.hex() == hex),
            "the fixture's blob must actually be a condemn candidate"
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
            rescue_quarantined: plan.rescue_quarantined.clone(),
            started_at_startup: false,
        };
        core.commit(EventDraft::new(
            EventSource::new("daemon", "sergeant"),
            KIND_PRUNE_INTENT,
            record.to_payload(),
        ))
        .expect("commit intent");
        core.flush().expect("flush intent");
        (plan, blob_ref)
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

    /// N8 / F2: a crash after the intent's blob has already been moved into
    /// quarantine, before `prune.completed` is appended, is completed in
    /// full at the next start — `BlobStore::quarantine`'s own idempotent
    /// `Ok(false)` on an already-moved blob is what makes re-running the
    /// whole loop from the intent's record safe regardless of whether the
    /// crashed cycle got to the rename or not, and never leaves two copies
    /// of the same content.
    #[test]
    fn crash_window_f2_mid_quarantine_completes_idempotently() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = tiny_core(dir.path());
        let (plan, blob_ref) = build_and_commit_intent_with_blob(&mut core, dir.path());

        // Simulate the crash landing *after* the rename: the blob is
        // already sitting in quarantine when the next start begins.
        let blobs = BlobStore::open(dir.path()).expect("open blob store");
        assert!(
            blobs
                .quarantine(&blob_ref)
                .expect("simulate the completed quarantine step"),
            "the fixture's blob must actually still be live to quarantine"
        );

        complete_interrupted(&mut core, dir.path()).expect("complete_interrupted");
        assert!(core.registry.state().pending_prune.is_none());
        assert!(core.registry.state().pruned_works.contains_key("w1"));
        let bounds_after = core.journal.segment_bounds().expect("bounds");
        assert!(
            bounds_after
                .iter()
                .all(|b| !plan.segments.iter().any(|s| s.index == b.index)),
            "every planned segment must still be gone"
        );

        let quarantine_path = dir
            .path()
            .join("blobs")
            .join("b3")
            .join(".pruned")
            .join(blob_ref.hex());
        let live_path = dir.path().join("blobs").join("b3").join(blob_ref.hex());
        assert!(
            quarantine_path.exists(),
            "the blob must still be quarantined"
        );
        assert!(!live_path.exists(), "the blob must never also exist live");

        // Idempotency, exercised for real: crash the *completion itself*,
        // after every physical effect it had and before its own
        // `prune.completed` landed. Re-folding from that journal gives a
        // `Core` whose `pending_prune` is still the same unpaired intent —
        // so this second `complete_interrupted` genuinely re-walks the
        // whole cycle over work the first pass already finished, rather
        // than exiting at the `pending_prune == None` guard on its first
        // statement (which is all a plain second call can ever do, and is
        // what `crash_window_f1_...`'s own no-op assertion covers).
        drop(core);
        unwrite_newest_event(dir.path(), KIND_PRUNE_COMPLETED);
        let mut core = refold_core(dir.path());
        assert!(
            core.registry.state().pending_prune.is_some(),
            "the re-folded journal must still hold the intent unpaired — otherwise the second \
             pass below tests nothing"
        );
        let next_seq_before_retry = core.journal.next_seq();

        complete_interrupted(&mut core, dir.path()).expect(
            "re-quarantining an already-quarantined blob and re-unlinking already-gone \
             segments must both be tolerated",
        );
        assert!(core.registry.state().pending_prune.is_none());
        assert!(core.registry.state().pruned_works.contains_key("w1"));
        assert_eq!(
            core.journal.next_seq(),
            next_seq_before_retry + 1,
            "the re-run must append exactly its own completion, nothing else"
        );

        // Still exactly one copy of the content, still in quarantine —
        // never a live copy alongside it, never lost.
        assert_eq!(
            std::fs::read(&quarantine_path).expect("the quarantined copy must have survived"),
            b"crash window fixture blob"
        );
        assert!(!live_path.exists(), "the blob must never also exist live");
    }

    /// N8 continued / F3: a crash after a *second* cycle's intent is
    /// committed, before its deferred delete of the *first* cycle's
    /// quarantined blob runs, is completed in full at the next start —
    /// `BlobStore::drop_quarantined`'s own idempotent `Ok(false)` on an
    /// already-gone quarantined file is what makes re-running the deferred
    /// delete safe regardless of whether the crashed cycle got to it.
    #[test]
    fn crash_window_f3_mid_deferred_delete_completes_idempotently() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = tiny_core(dir.path());

        // Cycle 1, run to completion: quarantines the blob.
        let (plan1, blob_ref) = build_and_commit_intent_with_blob(&mut core, dir.path());
        complete_interrupted(&mut core, dir.path()).expect("complete cycle 1");
        assert_eq!(
            core.registry.state().quarantined_blobs,
            vec![blob_ref.hex().to_string()],
            "cycle 1 must have recorded the blob as quarantined"
        );
        drop(plan1);

        // `anchor` (from `build_and_commit_intent_with_blob`) has done its
        // job and must stop being the horizon's perpetual blocker.
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_COMPLETED,
            Some("anchor"),
            serde_json::json!({}),
        );

        // New activity, prunable on its own, whose cycle's own
        // `delete_quarantined` inherits cycle 1's quarantined hex.
        submit_and_complete(&mut core, "w_mid");
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("anchor2"),
            serde_json::json!({"work": {
                "id": "anchor2", "intent": "keep the writer off w_mid's segments",
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
        let plan2 = plan(
            dir.path(),
            &bounds,
            candidate,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        )
        .expect("plan")
        .expect("w_mid must be prunable");
        assert_eq!(
            plan2.delete_quarantined,
            vec![blob_ref.hex().to_string()],
            "cycle 2 must inherit cycle 1's quarantined hex to delete"
        );

        // Commit only cycle 2's intent (step 1) — simulating a crash
        // before the deferred delete (step 3) ever runs.
        let record = PruneIntentRecord {
            intent_seq: 0,
            policy: plan2.policy,
            horizon_seq: plan2.horizon_seq,
            floor_seq_before: core.journal.floor_seq().expect("floor").unwrap_or(1),
            floor_seq_after: plan2.horizon_seq + 1,
            segments: plan2
                .segments
                .iter()
                .map(|b| PruneSegment {
                    index: b.index,
                    first_seq: b.first_seq,
                    last_seq: b.last_seq,
                    bytes: b.bytes,
                })
                .collect(),
            residue: plan2.residue.clone(),
            carried_forward: plan2.carried_forward.clone(),
            condemn: plan2.condemn.iter().map(|r| r.hex().to_string()).collect(),
            delete_quarantined: plan2.delete_quarantined.clone(),
            rescue_quarantined: plan2.rescue_quarantined.clone(),
            started_at_startup: false,
        };
        core.commit(EventDraft::new(
            EventSource::new("daemon", "sergeant"),
            KIND_PRUNE_INTENT,
            record.to_payload(),
        ))
        .expect("commit cycle 2's intent");
        core.flush().expect("flush cycle 2's intent");

        let quarantine_path = dir
            .path()
            .join("blobs")
            .join("b3")
            .join(".pruned")
            .join(blob_ref.hex());
        assert!(
            quarantine_path.exists(),
            "the blob must still be sitting in quarantine before the crash-recovery completes it"
        );

        complete_interrupted(&mut core, dir.path()).expect("complete cycle 2");
        assert!(core.registry.state().pending_prune.is_none());
        assert!(core.registry.state().pruned_works.contains_key("w_mid"));
        assert!(
            !quarantine_path.exists(),
            "the deferred delete must actually have run"
        );

        // Idempotency, exercised for real (see F2's own note): crash the
        // completion itself — the deferred delete has already run, its
        // `prune.completed` never landed — and complete again from a
        // re-fold of that journal, so `drop_quarantined`'s tolerated
        // already-gone case is actually walked into rather than skipped by
        // an early return.
        drop(core);
        unwrite_newest_event(dir.path(), KIND_PRUNE_COMPLETED);
        let mut core = refold_core(dir.path());
        let pending = core
            .registry
            .state()
            .pending_prune
            .clone()
            .expect("the re-folded journal must still hold cycle 2's intent unpaired");
        assert_eq!(
            pending.delete_quarantined,
            vec![blob_ref.hex().to_string()],
            "the re-run must be re-walking the very deferred delete that already happened"
        );

        complete_interrupted(&mut core, dir.path())
            .expect("re-deleting an already-deleted quarantined blob must be tolerated");
        assert!(core.registry.state().pending_prune.is_none());
        assert!(core.registry.state().pruned_works.contains_key("w_mid"));
        assert!(
            !quarantine_path.exists(),
            "the re-run must leave the deferred delete done, not undone"
        );
        assert!(
            !dir.path()
                .join("blobs")
                .join("b3")
                .join(blob_ref.hex())
                .exists(),
            "and must never resurrect a live copy of what it deleted"
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

    /// §8.3's read side: the residue's whole point is that a *later* replay
    /// — one that never sees `w1`'s own now-deleted
    /// `conversation.turn.grammar_unmeasured` event — must still recover the
    /// withdrawal from the surviving `prune.intent`. Without
    /// `note_carried_ask_withdrawal` wired into `CapabilitySink::push`, a
    /// fresh no-cache replay would silently answer `None`, re-raising
    /// `Capabilities::ask` on an installation that had already proved it
    /// absent — exactly the gap §8.3 exists to close.
    #[test]
    fn a_fresh_floor_replay_recovers_the_ask_withdrawal_from_the_prune_residue_alone() {
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
        run(&mut core, dir.path(), plan, false).expect("run");

        // The scenario §8.3 is written against: a replay from the floor
        // that never sees `withdrawal_event` at all (it was unlinked with
        // its segment) — driven through the same `CapabilitySink` a real
        // no-cache daemon start uses, seeded from nothing.
        let mut capability = startup::CapabilitySink::seeded(None);
        let events = crate::runtime::journal::Journal::replay_data_dir_from_floor(dir.path())
            .expect("replay_data_dir_from_floor");
        startup::drive(events, &mut [&mut capability]).expect("drive");

        let recovered = capability
            .latest
            .as_ref()
            .expect("the withdrawal must survive a fresh floor replay with no cache");
        assert_eq!(
            recovered.seq, withdrawal_event.seq,
            "the carried watermark must keep its *original* seq, not the intent's"
        );
        assert_eq!(recovered.version, "2.1.226");
    }

    /// N5: a blob referenced by both a to-be-pruned Work and a retained one
    /// must never be condemned — the surviving-side scan's whole purpose.
    #[test]
    fn a_blob_shared_with_a_retained_work_is_never_condemned() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = tiny_core(dir.path());
        let blob_ref = BlobStore::open(dir.path())
            .expect("open blob store")
            .put(b"shared content")
            .expect("put");
        let hex = blob_ref.hex().to_string();

        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("w_old"),
            serde_json::json!({"work": {
                "id": "w_old", "intent": "prunable, references the shared blob",
                "state": "pending", "created_by": "test",
                "created_at": "2026-01-01T00:00:00.000Z",
            }}),
        );
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_COMPLETED,
            Some("w_old"),
            serde_json::json!({"result_blob": format!("b3:{hex}")}),
        );
        // `w_keep` stays active forever (never retired_whole), so it is
        // always the horizon's blocker — which guarantees its own
        // referencing event's seq is always strictly *above* whatever
        // horizon this cycle computes, i.e. always in the surviving range.
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("w_keep"),
            serde_json::json!({"work": {
                "id": "w_keep", "intent": "retained, references the same blob",
                "state": "pending", "created_by": "test",
                "created_at": "2026-01-01T00:00:00.000Z",
                "ref": format!("b3:{hex}"),
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
        .expect("w_old must still be prunable even though w_keep pins the horizon below it");

        assert_eq!(
            plan.residue
                .works
                .iter()
                .map(|r| r.id.as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["w_old"]),
            "only w_old — the Work entirely before w_keep's pin — may be pruned"
        );
        assert!(
            !plan.condemn.iter().any(|r| r.hex() == hex),
            "the shared blob must never be condemned while w_keep still references it live"
        );

        let outcome = run(&mut core, dir.path(), plan, false).expect("run");
        assert_eq!(outcome.blobs_quarantined, 0);
        let bytes = BlobStore::open(dir.path())
            .expect("open blob store")
            .get(&blob_ref)
            .expect("the shared blob must still be live and intact");
        assert_eq!(bytes, b"shared content");
    }

    /// N6: a blob one cycle quarantines, that a *live* event adopts again
    /// before the next cycle's deferred delete runs, must be rescued back
    /// to its content address rather than deleted — the two-phase
    /// quarantine's whole reason to exist (A5). Exercises the fix directly:
    /// before it, `delete_quarantined` was `registry.quarantined_blobs`
    /// verbatim, with no check against what the surviving journal now
    /// references.
    #[test]
    fn a_dedup_adoption_between_mark_and_sweep_is_rescued_from_quarantine() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = tiny_core(dir.path());
        let blob_ref = BlobStore::open(dir.path())
            .expect("open blob store")
            .put(b"adopted content")
            .expect("put");
        let hex = blob_ref.hex().to_string();

        // Cycle 1: w_old is the only reference to the blob — it gets
        // quarantined.
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("w_old"),
            serde_json::json!({"work": {
                "id": "w_old", "intent": "prunable, references the blob first",
                "state": "pending", "created_by": "test",
                "created_at": "2026-01-01T00:00:00.000Z",
            }}),
        );
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_COMPLETED,
            Some("w_old"),
            serde_json::json!({"result_blob": format!("b3:{hex}")}),
        );
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("anchor1"),
            serde_json::json!({"work": {
                "id": "anchor1", "intent": "keep the writer off w_old's segments",
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
        let plan1 = plan(
            dir.path(),
            &bounds,
            candidate,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        )
        .expect("plan")
        .expect("w_old must be prunable");
        let outcome1 = run(&mut core, dir.path(), plan1, false).expect("run cycle 1");
        assert_eq!(outcome1.blobs_quarantined, 1);
        assert_eq!(
            core.registry.state().quarantined_blobs,
            vec![hex.clone()],
            "the blob must be quarantined, recorded on the registry"
        );

        // Between cycle 1's completion and cycle 2, a live event adopts the
        // same content again (a dedup hit this test asserts directly,
        // rather than through `BlobStore::put`'s own internal rescue, to
        // isolate `plan`'s mark-scan-side handling of it). `anchor1` is
        // completed so it stops blocking; `anchor2` — carrying the
        // reference — takes over as the perpetual blocker, so its own
        // event's seq is always in the surviving range for cycle 2.
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_COMPLETED,
            Some("anchor1"),
            serde_json::json!({}),
        );
        submit_and_complete(&mut core, "w_mid");
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("anchor2"),
            serde_json::json!({"work": {
                "id": "anchor2", "intent": "adopts the quarantined blob again",
                "state": "pending", "created_by": "test",
                "created_at": "2026-01-01T00:00:00.000Z",
                "ref": format!("b3:{hex}"),
            }}),
        );
        core.flush().expect("flush");

        let bounds = core.journal.segment_bounds().expect("bounds");
        let (candidate, _) = candidate_horizon(
            &bounds,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        );
        let plan2 = plan(
            dir.path(),
            &bounds,
            candidate,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        )
        .expect("plan")
        .expect("w_mid must be prunable in cycle 2");

        assert!(
            plan2.delete_quarantined.is_empty(),
            "the adopted hex must not be scheduled for deletion"
        );
        assert_eq!(
            plan2.rescue_quarantined,
            vec![hex.clone()],
            "the adopted hex must be scheduled for rescue instead"
        );

        let outcome2 = run(&mut core, dir.path(), plan2, false).expect("run cycle 2");
        assert_eq!(outcome2.blobs_deleted, 0);
        assert_eq!(outcome2.blobs_rescued_by_reference, 1);

        let bytes = BlobStore::open(dir.path())
            .expect("open blob store")
            .get(&blob_ref)
            .expect("the adopted blob must be live again, not deleted");
        assert_eq!(bytes, b"adopted content");
    }

    /// N6's other half — the same-cycle collapse. A hex this cycle's own
    /// mark scan condemns **again** must never also be deleted from
    /// quarantine in the same cycle: `condemn` re-marks it now, and the
    /// deferred delete of it belongs to the *next* cycle. Quarantining and
    /// deleting one hex inside a single guard hold leaves a deferral window
    /// of exactly zero — and that window is the only thing standing between
    /// an in-flight `BlobStore::put` dedup-hit (its referencing event not
    /// yet committed, so invisible to both the mark scan and `run`'s
    /// guard-held top-up) and a destroyed live blob with a dangling `b3:`
    /// reference to it.
    ///
    /// `a_dedup_adoption_between_mark_and_sweep_is_rescued_from_quarantine`
    /// deliberately keeps its adoption event in the *surviving* range,
    /// which is the rescue path. This one puts the adoption **inside cycle
    /// 2's own condemned range**, which is what lands the hex in `condemn`
    /// and `delete_quarantined` at the same time — the overlap the two-set
    /// partition used to ignore entirely.
    #[test]
    fn a_hex_condemned_again_this_cycle_is_never_deleted_in_the_same_cycle() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = tiny_core(dir.path());
        let blob_ref = BlobStore::open(dir.path())
            .expect("open blob store")
            .put(b"re-adopted content")
            .expect("put");
        let hex = blob_ref.hex().to_string();
        let quarantine_path = dir
            .path()
            .join("blobs")
            .join("b3")
            .join(".pruned")
            .join(&hex);
        let live_path = dir.path().join("blobs").join("b3").join(&hex);

        let policy = PrunePolicy {
            retention: 0,
            source: PolicySource::Config,
        };
        let run_a_cycle = |core: &mut crate::api::Core| -> PruneOutcome {
            let bounds = core.journal.segment_bounds().expect("bounds");
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
            assert!(
                plan.delete_quarantined
                    .iter()
                    .all(|h| !plan.condemn.iter().any(|r| r.hex() == h)),
                "no hex may be scheduled for both a fresh condemn and this cycle's delete"
            );
            run(core, dir.path(), plan, false).expect("run")
        };

        // Cycle 1: w_old is the only reference — the blob is quarantined.
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("w_old"),
            serde_json::json!({"work": {
                "id": "w_old", "intent": "prunable, references the blob first",
                "state": "pending", "created_by": "test",
                "created_at": "2026-01-01T00:00:00.000Z",
            }}),
        );
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_COMPLETED,
            Some("w_old"),
            serde_json::json!({"result_blob": format!("b3:{hex}")}),
        );
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("anchor1"),
            serde_json::json!({"work": {
                "id": "anchor1", "intent": "keep the writer off w_old's segments",
                "state": "pending", "created_by": "test",
                "created_at": "2026-01-01T00:00:00.000Z",
            }}),
        );
        core.flush().expect("flush");
        assert_eq!(run_a_cycle(&mut core).blobs_quarantined, 1);
        assert_eq!(core.registry.state().quarantined_blobs, vec![hex.clone()]);
        assert!(quarantine_path.exists() && !live_path.exists());

        // A real `put` of the same content between the cycles: its dedup
        // check rescues the quarantined copy back to its live address
        // (`put_rescues_instead_of_writing_a_second_copy`). This is the
        // in-flight writer whose own event has not been committed yet.
        let readopted = BlobStore::open(dir.path())
            .expect("open blob store")
            .put(b"re-adopted content")
            .expect("put the same content again");
        assert_eq!(readopted.hex(), hex);
        assert!(
            live_path.exists() && !quarantine_path.exists(),
            "the dedup-hit put must have rescued the blob back to its live address"
        );

        // `anchor1` stops blocking; `w_mid` carries the new reference and
        // is itself fully retired *below* cycle 2's horizon, so its event —
        // unlike N6's — sits inside the range cycle 2 condemns.
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_COMPLETED,
            Some("anchor1"),
            serde_json::json!({}),
        );
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("w_mid"),
            serde_json::json!({"work": {
                "id": "w_mid", "intent": "re-adopts the same content",
                "state": "pending", "created_by": "test",
                "created_at": "2026-01-01T00:00:00.000Z",
                "ref": format!("b3:{hex}"),
            }}),
        );
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_COMPLETED,
            Some("w_mid"),
            serde_json::json!({}),
        );
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("anchor2"),
            serde_json::json!({"work": {
                "id": "anchor2", "intent": "keep the writer off w_mid's segments",
                "state": "pending", "created_by": "test",
                "created_at": "2026-01-01T00:00:00.000Z",
            }}),
        );
        core.flush().expect("flush");

        // Cycle 2: the hex is condemned afresh *and* is last cycle's
        // quarantined hex. It must be re-marked, not swept.
        let bounds = core.journal.segment_bounds().expect("bounds");
        let (candidate, _) = candidate_horizon(
            &bounds,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        );
        let plan2 = plan(
            dir.path(),
            &bounds,
            candidate,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        )
        .expect("plan")
        .expect("w_mid must be prunable in cycle 2");
        assert!(
            plan2.condemn.iter().any(|r| r.hex() == hex),
            "the fixture must actually put the hex back in this cycle's fresh condemn set"
        );
        assert!(
            plan2.delete_quarantined.is_empty(),
            "a hex this cycle re-condemns must not also be swept this cycle"
        );
        assert!(
            plan2.rescue_quarantined.is_empty(),
            "nor rescued — no *surviving* event references it"
        );
        assert_eq!(
            plan2.defer_quarantined,
            vec![hex.clone()],
            "it belongs to the deferred set: re-marked now, re-evaluated next cycle"
        );

        let outcome2 = run(&mut core, dir.path(), plan2, false).expect("run cycle 2");
        assert_eq!(
            outcome2.blobs_quarantined, 1,
            "the re-adopted (live again) blob must be quarantined by this cycle"
        );
        assert_eq!(
            outcome2.blobs_deleted, 0,
            "and must not be deleted by the same cycle that just marked it"
        );
        assert_eq!(
            std::fs::read(&quarantine_path).expect("the content must still exist, in quarantine"),
            b"re-adopted content"
        );
        assert!(
            core.registry.state().quarantined_blobs.contains(&hex),
            "the deferral must be armed for the next cycle via the intent's own condemn list"
        );

        // The window is real: a writer that dedup-hits this content in the
        // interval still gets it back intact, which is the entire point of
        // deferring the delete by a cycle.
        let bytes = BlobStore::open(dir.path())
            .expect("open blob store")
            .get(&blob_ref)
            .expect("the deferred blob must still be recoverable");
        assert_eq!(bytes, b"re-adopted content");
    }

    /// §3.5's stated invariant, checked directly against a real fixture
    /// rather than only inferred from `nostraddle`'s own definition: every
    /// retained (still-in-`work_index`) Work's `first_seq` is at or above
    /// the journal's floor after a real prune — nothing retained can start
    /// underneath the segments a cycle just unlinked.
    #[test]
    fn every_retained_work_starts_at_or_above_the_floor() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = tiny_core(dir.path());
        let plan = build_and_commit_intent(&mut core, dir.path());
        complete_interrupted(&mut core, dir.path()).expect("complete_interrupted");
        assert!(core.registry.state().pruned_works.contains_key("w1"));
        drop(plan);

        let floor = core.journal.floor_seq().expect("floor_seq").unwrap_or(1);
        for (id, first) in &core.first_seq_by_work {
            if core.registry.state().work_index.contains_key(id) {
                assert!(
                    *first >= floor,
                    "retained Work {id} starts at seq {first}, below the floor {floor}"
                );
            }
        }
        assert!(
            !core.registry.state().work_index.is_empty(),
            "the fixture must actually retain something (anchor) for this to check anything"
        );
    }

    /// §6.4: residue must keep surviving no matter how many *further*
    /// cycles run after the Work that originally recorded it is gone —
    /// carried forward from `prune.intent` to `prune.intent` indefinitely,
    /// not just across the one cycle that first pruned it. Three
    /// generations: cycle 1 prunes `w1` itself; cycles 2 and 3 prune
    /// unrelated later Works, each carrying `w1`'s row forward without ever
    /// touching it again.
    #[test]
    fn a_carried_forward_residue_survives_three_generations_of_prune() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = tiny_core(dir.path());

        let policy = PrunePolicy {
            retention: 0,
            source: PolicySource::Config,
        };
        let run_one_cycle = |core: &mut crate::api::Core, work_id: &str, anchor: &str| {
            submit_and_complete(core, work_id);
            commit(
                core,
                daemon_source(),
                crate::domain::work::KIND_WORK_SUBMITTED,
                Some(anchor),
                serde_json::json!({"work": {
                    "id": anchor, "intent": "keep the writer off this cycle's segments",
                    "state": "pending", "created_by": "test",
                    "created_at": "2026-01-01T00:00:00.000Z",
                }}),
            );
            core.flush().expect("flush");
            let bounds = core.journal.segment_bounds().expect("bounds");
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
            run(core, dir.path(), plan, false).expect("run")
        };

        // Cycle 1: prunes w1 directly.
        run_one_cycle(&mut core, "w1", "anchor1");
        assert!(core.registry.state().pruned_works.contains_key("w1"));
        // `anchor1` must stop blocking before the next cycle.
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_COMPLETED,
            Some("anchor1"),
            serde_json::json!({}),
        );

        // Cycle 2: prunes w2; must carry w1's row forward untouched.
        run_one_cycle(&mut core, "w2", "anchor2");
        assert!(core.registry.state().pruned_works.contains_key("w1"));
        assert!(core.registry.state().pruned_works.contains_key("w2"));
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_COMPLETED,
            Some("anchor2"),
            serde_json::json!({}),
        );

        // Cycle 3: prunes w3; both w1 and w2's rows must still be present,
        // carried across two further generations neither of them was
        // touched in again.
        run_one_cycle(&mut core, "w3", "anchor3");
        let registry = core.registry.state();
        assert!(
            registry.pruned_works.contains_key("w1"),
            "w1's residue must survive two further generations of prune"
        );
        assert!(registry.pruned_works.contains_key("w2"));
        assert!(registry.pruned_works.contains_key("w3"));
    }

    /// The ordering half of §8.3: a withdrawal *carried* forward from a
    /// pruned Work must never outrank a **newer** withdrawal recorded by a
    /// still-live, retained event — "higher seq wins" must keep meaning
    /// what it means regardless of which side (carried residue, or a
    /// surviving event `note_ask_withdrawal` matches directly) a replay
    /// happens to see first.
    #[test]
    fn a_carried_withdrawal_never_outranks_a_newer_retained_one() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = tiny_core(dir.path());
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("w_old"),
            serde_json::json!({"work": {
                "id": "w_old", "intent": "records the older withdrawal", "state": "pending",
                "created_by": "test", "created_at": "2026-01-01T00:00:00.000Z",
            }}),
        );
        let older = commit(
            &mut core,
            EventSource::new("backend", "claude"),
            "conversation.turn.grammar_unmeasured",
            Some("w_old"),
            serde_json::json!({"capability": "ask", "version": "2.1.226"}),
        );
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_COMPLETED,
            Some("w_old"),
            serde_json::json!({}),
        );
        // `w_new` stays active forever (never retired_whole), so it is
        // always the horizon's blocker — guaranteeing its own withdrawal
        // event's seq is always in the surviving range, never condemned.
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("w_new"),
            serde_json::json!({"work": {
                "id": "w_new", "intent": "records the newer withdrawal", "state": "pending",
                "created_by": "test", "created_at": "2026-01-01T00:00:00.000Z",
            }}),
        );
        let newer = commit(
            &mut core,
            EventSource::new("backend", "claude"),
            "conversation.turn.grammar_unmeasured",
            Some("w_new"),
            serde_json::json!({"capability": "ask", "version": "2.1.226"}),
        );
        assert!(
            newer.seq > older.seq,
            "the fixture's whole point is a newer, surviving withdrawal"
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
        .expect("w_old must be prunable even though w_new pins the horizon below it");

        // The carried residue only ever holds `older` — `newer`'s event is
        // in the surviving range, never scanned into the condemned side.
        assert_eq!(
            plan.residue.capability_provenance.as_ref().map(|w| w.seq),
            Some(older.seq),
        );
        run(&mut core, dir.path(), plan, false).expect("run");

        // A fresh floor replay must fold *both* sources and land on the
        // newer, still-live withdrawal — never regress to the carried,
        // older one.
        let mut capability = startup::CapabilitySink::seeded(None);
        let events = crate::runtime::journal::Journal::replay_data_dir_from_floor(dir.path())
            .expect("replay_data_dir_from_floor");
        startup::drive(events, &mut [&mut capability]).expect("drive");
        let recovered = capability
            .latest
            .as_ref()
            .expect("a withdrawal must be recoverable");
        assert_eq!(
            recovered.seq, newer.seq,
            "the newer, still-live withdrawal must win over the older, carried-forward one"
        );
    }

    /// §7.4: when the batch cap truncates a cycle's target set, the
    /// remainder must not wait for an unrelated rotation — `run` must
    /// re-arm `prune_pending` from `PruneOutcome::truncated_by_cap` rather
    /// than unconditionally clearing it on every successful commit.
    #[test]
    fn a_batch_cap_truncated_cycle_re_arms_prune_pending() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = tiny_core(dir.path());
        submit_and_complete(&mut core, "w1");
        // I-W3-4: push the writer's live segment off of w1's own.
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
        let mut plan = plan(
            dir.path(),
            &bounds,
            candidate,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        )
        .expect("plan")
        .expect("something must be prunable");
        // Simulate what `candidate_horizon` would have set had this cycle's
        // eligible set actually exceeded `PRUNE_MAX_WORKS_PER_CYCLE` — the
        // trigger itself belongs to `candidate_horizon`'s own tests; this
        // isolates `run`'s handling of the flag it is handed.
        plan.stall.truncated_by_cap = true;

        let outcome = run(&mut core, dir.path(), plan, false).expect("run");
        assert!(outcome.truncated_by_cap);
        assert!(
            core.prune_pending,
            "a batch-capped cycle must re-arm prune_pending so the next tick \
             continues the drain rather than waiting for an unrelated rotation"
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

    /// N19 (§6.5): an unpaired `prune.intent` — its own `prune.completed`
    /// not (yet) anywhere in the journal, exactly the shape a crash between
    /// steps 1 and 5 leaves behind — must pin its own segment, exactly like
    /// an unknown non-work-scoped kind (N4). Without this, a later cycle's
    /// mark scan could condemn the segment holding a still-open intent,
    /// after which the eventual crash-recovery completion would have
    /// nothing left to finish it from consistently — "a pruned id has a
    /// row in neither... or, worse, an intent whose completion was
    /// deleted, leaving `pending_prune` permanently `Some`" (§6.5).
    #[test]
    fn a_prune_pair_straddling_the_horizon_pins_its_segment() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = tiny_core(dir.path());

        // An ordinary Work, already eligible for its own cycle.
        submit_and_complete(&mut core, "w_prior");
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("anchor0"),
            serde_json::json!({"work": {
                "id": "anchor0", "intent": "keep the writer off w_prior's segments",
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
        let (candidate0, _) = candidate_horizon(
            &bounds,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        );
        let plan0 = plan(
            dir.path(),
            &bounds,
            candidate0,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        )
        .expect("plan")
        .expect("w_prior must be prunable on its own");

        // Commit *only* the intent (step 1) — simulating a crash before
        // `prune.completed` (F1's window) — and deliberately never run
        // `complete_interrupted`, so the intent stays genuinely unpaired in
        // the journal for the rest of this test, exactly the shape §6.5
        // describes.
        let record = PruneIntentRecord {
            intent_seq: 0,
            policy: plan0.policy,
            horizon_seq: plan0.horizon_seq,
            floor_seq_before: 1,
            floor_seq_after: plan0.horizon_seq + 1,
            segments: plan0
                .segments
                .iter()
                .map(|b| PruneSegment {
                    index: b.index,
                    first_seq: b.first_seq,
                    last_seq: b.last_seq,
                    bytes: b.bytes,
                })
                .collect(),
            residue: plan0.residue.clone(),
            carried_forward: plan0.carried_forward.clone(),
            condemn: plan0.condemn.iter().map(|r| r.hex().to_string()).collect(),
            delete_quarantined: plan0.delete_quarantined.clone(),
            rescue_quarantined: plan0.rescue_quarantined.clone(),
            started_at_startup: false,
        };
        let intent_event = core
            .commit(EventDraft::new(
                EventSource::new("daemon", "sergeant"),
                KIND_PRUNE_INTENT,
                record.to_payload(),
            ))
            .expect("commit the stuck intent");
        core.flush().expect("flush the stuck intent");
        let intent_seq = intent_event.seq;
        assert!(
            core.registry.state().pending_prune.is_some(),
            "the intent must be pending — nothing has completed it"
        );

        // `anchor0` has done its job (kept the writer off `w_prior`'s
        // segments) and must stop being the horizon's blocker, or it would
        // mask the very thing this test is checking.
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_COMPLETED,
            Some("anchor0"),
            serde_json::json!({}),
        );

        // New activity after the stuck intent: another Work, fully retired
        // and past the cap entirely on its own.
        submit_and_complete(&mut core, "w_after");
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("anchor1"),
            serde_json::json!({"work": {
                "id": "anchor1", "intent": "keep the writer off w_after's segments",
                "state": "pending", "created_by": "test",
                "created_at": "2026-01-01T00:00:00.000Z",
            }}),
        );
        core.flush().expect("flush");

        let bounds = core.journal.segment_bounds().expect("bounds");
        let (candidate1, _) = candidate_horizon(
            &bounds,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        );
        // Phase A alone does not know about the stuck intent either — both
        // w_prior and w_after look fully eligible to it.
        assert!(candidate1 > intent_seq, "Phase A must not see the pin yet");

        let plan1 = plan(
            dir.path(),
            &bounds,
            candidate1,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        )
        .expect("plan")
        .expect("w_prior must still be prunable even though the stuck intent pins the rest");

        assert_eq!(
            plan1.stall.pinning_seq,
            Some(intent_seq),
            "the stall report must name the stuck intent's own seq as the pin"
        );
        assert!(
            plan1.horizon_seq < intent_seq,
            "the horizon must never reach or pass the segment holding the unpaired intent"
        );
        let pruned_ids: BTreeSet<&str> =
            plan1.residue.works.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(
            pruned_ids,
            BTreeSet::from(["w_prior"]),
            "only w_prior, entirely before the stuck intent, may be pruned this cycle"
        );
    }

    /// §6.5, the pin the *rescan* finds: the pin predicate is not monotone
    /// in the ceiling, so lowering the horizon can **reveal** a pin the
    /// higher pass could not see. `scan_condemned_range`'s `open_intents`
    /// arm is the ceiling-dependent one: a `prune.intent` at `S` whose
    /// `prune.completed` sits at `C` reads as a closed, unremarkable pair
    /// when both are in range, and as an unpaired, segment-pinning intent
    /// the moment an unrelated pin lowers the horizon to somewhere in
    /// `[S, C)`. A plan that consults only the first pass's answer therefore
    /// unlinks the segment holding a still-open intent — leaving
    /// `pending_prune` permanently `Some` with nothing left to complete it
    /// from, which is exactly what §6.5 forbids.
    ///
    /// Fixture (one event per segment, so seq == segment index):
    ///
    /// ```text
    ///   1  w1 submitted        2  w1 completed
    ///   3  prune.intent  <---------------------------- the straddling pair
    ///   4  mystery.event  (the first pass's own pin)
    ///   5  prune.completed(intent_seq = 3)  <---------- ...closes above 3
    ///   6  w2 submitted        7  w2 completed
    ///   8  anchor submitted    (the perpetual blocker)
    /// ```
    ///
    /// Phase A proposes 7. The first pass (ceiling 7) sees the pair closed
    /// and pins only on `mystery.event`, lowering the horizon to 3 — which
    /// is precisely the segment holding the intent. Only the rescan at 3
    /// can see that intent unpaired, and it must lower the horizon again,
    /// to 2.
    #[test]
    fn a_pair_that_only_straddles_the_lowered_horizon_still_pins_its_segment() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = tiny_core(dir.path());
        let policy = PrunePolicy {
            retention: 0,
            source: PolicySource::Config,
        };

        submit_and_complete(&mut core, "w1");

        let empty_intent = PruneIntentRecord {
            intent_seq: 0,
            policy,
            horizon_seq: 0,
            floor_seq_before: 1,
            floor_seq_after: 1,
            segments: Vec::new(),
            residue: PruneResidue::default(),
            carried_forward: PruneResidue::default(),
            condemn: Vec::new(),
            delete_quarantined: Vec::new(),
            rescue_quarantined: Vec::new(),
            started_at_startup: false,
        };
        let intent_event = core
            .commit(EventDraft::new(
                EventSource::new("daemon", "sergeant"),
                KIND_PRUNE_INTENT,
                empty_intent.to_payload(),
            ))
            .expect("commit the intent");
        let intent_seq = intent_event.seq;

        // The pin the *first* pass will find, sitting between the intent
        // and its completion — not in `NON_WORK_ALLOWLIST`, no `work_id`.
        commit(
            &mut core,
            daemon_source(),
            "mystery.event",
            None,
            serde_json::json!({}),
        );
        let completion_seq = commit(
            &mut core,
            daemon_source(),
            KIND_PRUNE_COMPLETED,
            None,
            serde_json::json!({
                "intent_seq": intent_seq,
                "outcome": PruneOutcome::default(),
                "floor_seq_after": 1,
                "completed_at_startup": false,
            }),
        )
        .seq;
        assert!(
            core.registry.state().pending_prune.is_none(),
            "the pair must be closed as far as the registry is concerned"
        );

        submit_and_complete(&mut core, "w2");
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
        let (candidate, _) = candidate_horizon(
            &bounds,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        );
        assert!(
            candidate > completion_seq,
            "Phase A must propose a horizon above the whole pair — otherwise the pair never \
             straddles anything and this test proves nothing"
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
        .expect("w1 must still be prunable below the intent");

        assert!(
            plan.horizon_seq < intent_seq,
            "the horizon must stop below the segment holding the intent that the *lowered* \
             range leaves unpaired (got {}, intent at {intent_seq})",
            plan.horizon_seq
        );
        assert!(
            plan.segments.iter().all(|s| s.last_seq < intent_seq),
            "and no planned segment may contain it"
        );
        assert_eq!(
            plan.stall.pinning_seq,
            Some(intent_seq),
            "the stall report must name the pin that actually bound the final horizon — the \
             intent found by the rescan, not the `mystery.event` the first pass stopped at"
        );
        assert_eq!(plan.stall.pinning_kind.as_deref(), Some(KIND_PRUNE_INTENT));
        let pruned_ids: BTreeSet<&str> = plan.residue.works.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(
            pruned_ids,
            BTreeSet::from(["w1"]),
            "only w1, entirely below the intent, may be pruned this cycle"
        );
    }

    /// §7.1: a pin that lands *inside* the span of a Work straddling the
    /// pin's own segment boundary must not select "the largest segment
    /// boundary below the pin" as the horizon — `nostraddle` is not
    /// monotone in `b`, so a boundary that looked fine to Phase A can be
    /// inadmissible once the pin lowers the ceiling below the straddling
    /// Work's own span. `plan` must re-check `nostraddle`/`retired`/`capped`
    /// at the lowered height and correctly find nothing prunable, rather
    /// than mis-selecting an inadmissible horizon that would either
    /// straddle a live Work or (as this same bug did before the fix) rely
    /// on the residue cross-check to reject it as a `ResidueMismatch` —
    /// which would then recur identically on every subsequent cycle, since
    /// nothing about the pin or the Work ever changes.
    #[test]
    fn a_pin_inside_a_straddling_works_span_never_selects_an_inadmissible_horizon() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = tiny_core(dir.path());

        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("w_straddle"),
            serde_json::json!({"work": {
                "id": "w_straddle", "intent": "straddles the pin's own segment",
                "state": "pending", "created_by": "test",
                "created_at": "2026-01-01T00:00:00.000Z",
            }}),
        );
        // The pin: a non-work-scoped, non-allowlisted event landing
        // *inside* w_straddle's own span (between its submit and its
        // completion), each in its own segment (`tiny_core` rotates every
        // event).
        commit(
            &mut core,
            daemon_source(),
            "mystery.event",
            None,
            serde_json::json!({}),
        );
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_COMPLETED,
            Some("w_straddle"),
            serde_json::json!({}),
        );
        // I-W3-4: push the writer off w_straddle's own segments — and,
        // being active, `anchor` is also what makes w_straddle look fully
        // retired and past-the-cap to Phase A once the writer moves past it.
        commit(
            &mut core,
            daemon_source(),
            crate::domain::work::KIND_WORK_SUBMITTED,
            Some("anchor"),
            serde_json::json!({"work": {
                "id": "anchor", "intent": "keep the writer off w_straddle's segments",
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
        // Phase A alone (registry-only) does not see the pin: w_straddle is
        // fully retired and past the cap by the time the writer has moved
        // on to `anchor`'s own segment.
        assert!(candidate > 0, "Phase A must not see the pin yet");

        let result = plan(
            dir.path(),
            &bounds,
            candidate,
            core.registry.state(),
            &core.first_seq_by_work,
            &policy,
        );
        assert!(
            matches!(result, Ok(None)),
            "a pin inside a straddling Work's span must correctly find \
             nothing prunable rather than mis-select an inadmissible \
             horizon: got {result:?}"
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
