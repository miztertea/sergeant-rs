//! Reducer-based projections with snapshot + replay (proposal §24).
//!
//! Current state is a pure fold of the journal: `fn(&mut State, &Event)`.
//! Snapshots are an optimization, never canonical — loading one and replaying
//! the tail must be provably identical to a full replay, and an unknown
//! snapshot schema fails closed so the caller falls back to full replay
//! (which is always available).

use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::daemon::{KIND_ADMISSION_PAUSED, KIND_ADMISSION_RESUMED};
use crate::domain::event::Event;
use crate::domain::execution::{
    ExecutionRecord, ExecutionReservation, KIND_EXECUTION_ABANDONED, KIND_EXECUTION_RESERVED,
    KIND_EXECUTION_STARTED, KIND_EXECUTION_STOPPED,
};
use crate::domain::profile::Profile;
use crate::domain::work::{
    KIND_COMMAND_ACCEPTED, KIND_COMMAND_REJECTED, KIND_WORK_SUBMITTED, Work, WorkState,
};
use crate::domain::workflow::{
    KIND_STAGE_BLOCKED, KIND_STAGE_CANCELED, KIND_STAGE_COMPLETED, KIND_STAGE_ENTERED,
    KIND_STAGE_FAILED, KIND_STAGE_NEEDS_INPUT, KIND_STAGE_RESUMED, KIND_STAGE_WAITING,
    KIND_WORKFLOW_BOUND, StageBinding, StageRecord, StageStatus, WorkflowDefinition,
};
use crate::domain::workspace::InstructionIdentity;
use crate::runtime::fsutil::{create_dir_all_durable, write_atomic};
use crate::runtime::integrity::IntegrityDisposition;
use crate::runtime::journal::{Journal, JournalError};
use crate::runtime::surface::{
    KIND_SURFACE_MATERIALIZED, KIND_SURFACE_MATERIALIZING, KIND_SURFACE_TORN_DOWN, SurfacePlan,
    TeardownReport, WorkSurface,
};

/// Snapshot schema identifier written by this version.
pub const SNAPSHOT_SCHEMA: &str = "sergeant.snapshot/v1";

/// A pure reducer: folds one event into the state. Must not perform I/O.
pub type Reducer<S> = fn(&mut S, &Event);

/// Errors from projections and snapshots.
#[derive(Debug, thiserror::Error)]
pub enum ProjectionError {
    /// The journal failed during a fold.
    #[error(transparent)]
    Journal(#[from] JournalError),
    /// `apply` was given an event that is not exactly the next seq.
    #[error("projection seq mismatch: expected {expected}, got {found}")]
    SeqMismatch {
        /// The seq the projection required next.
        expected: u64,
        /// The seq actually offered.
        found: u64,
    },
    /// Snapshot file I/O failure.
    #[error("snapshot io error: {0}")]
    SnapshotIo(#[from] std::io::Error),
    /// Snapshot (de)serialization failure.
    #[error("snapshot serde error: {0}")]
    SnapshotSerde(#[from] serde_json::Error),
    /// The snapshot declares a schema this version does not understand.
    /// Fail closed: the caller must fall back to full replay.
    #[error("unknown snapshot schema {found:?} (this version reads {SNAPSHOT_SCHEMA:?})")]
    UnknownSnapshotSchema {
        /// The schema string found in the file.
        found: String,
    },
    /// During catch-up, the replay ended below the seq this projection
    /// resumed from: the snapshot claims events the journal does not contain
    /// (a foreign or stale snapshot, or a journal missing its tail). Fail
    /// closed rather than accept state the journal cannot reproduce; full
    /// replay of the real journal is always available.
    #[error(
        "snapshot beyond journal: snapshot at seq {snapshot_seq}, \
         replay ended at seq {journal_last_seq}"
    )]
    SnapshotBeyondJournal {
        /// The seq this projection resumed from (snapshot `last_seq`).
        snapshot_seq: u64,
        /// The last seq the replay actually produced (0 if none).
        journal_last_seq: u64,
    },
}

/// State folded from the journal by a reducer, tracking the last applied seq.
#[derive(Debug, Clone)]
pub struct Projection<S> {
    state: S,
    last_seq: u64,
    reducer: Reducer<S>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SnapshotFile {
    schema: String,
    last_seq: u64,
    state: Value,
}

impl<S> Projection<S> {
    /// A projection at seq 0 (nothing applied yet).
    pub fn new(initial: S, reducer: Reducer<S>) -> Self {
        Self {
            state: initial,
            last_seq: 0,
            reducer,
        }
    }

    /// The current folded state.
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Seq of the last event folded in (0 if none).
    pub fn last_seq(&self) -> u64 {
        self.last_seq
    }

    /// Fold one event. The event must be exactly the next seq — a gap or
    /// duplicate is an error, never a silent skip.
    pub fn apply(&mut self, event: &Event) -> Result<(), ProjectionError> {
        if event.seq != self.last_seq + 1 {
            return Err(ProjectionError::SeqMismatch {
                expected: self.last_seq + 1,
                found: event.seq,
            });
        }
        (self.reducer)(&mut self.state, event);
        self.last_seq = event.seq;
        Ok(())
    }

    /// Fold a full-journal replay into this projection. Events at or below
    /// the current seq (already reflected — the snapshot-covered prefix) are
    /// skipped; seq validation of that prefix belongs to the replay iterator
    /// itself, which fails closed on any gap or duplicate before yielding
    /// (`Replay` in the journal module — the only journal-sourced feed). The
    /// replay must reach at least the seq this projection resumed from: a
    /// journal that ends short of it means the snapshot claims events the
    /// journal does not contain (a foreign or stale snapshot, or a journal
    /// missing its tail) — an error, never a silent `Ok` over state the
    /// journal cannot reproduce. Everything past the current seq must be
    /// contiguous (`apply` enforces it). Returns the number of events
    /// applied.
    pub fn catch_up<I>(&mut self, events: I) -> Result<u64, ProjectionError>
    where
        I: IntoIterator<Item = Result<Event, JournalError>>,
    {
        let snapshot_seq = self.last_seq;
        let mut journal_last_seq = 0u64;
        let mut applied = 0u64;
        for event in events {
            let event = event?;
            journal_last_seq = event.seq;
            if event.seq <= self.last_seq {
                continue;
            }
            self.apply(&event)?;
            applied += 1;
        }
        if journal_last_seq < snapshot_seq {
            return Err(ProjectionError::SnapshotBeyondJournal {
                snapshot_seq,
                journal_last_seq,
            });
        }
        Ok(applied)
    }
}

impl<S: Serialize + DeserializeOwned> Projection<S> {
    /// Write a snapshot of the current state to `path` (atomic tmp + rename,
    /// fsynced, via [`write_atomic`]). The snapshot is an optimization,
    /// never canonical.
    pub fn write_snapshot(&self, path: impl AsRef<Path>) -> Result<(), ProjectionError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            // Durable creation so a fresh snapshot directory's dirent
            // survives a crash along with the snapshot inside it.
            create_dir_all_durable(parent)?;
        }
        let snapshot = SnapshotFile {
            schema: SNAPSHOT_SCHEMA.to_string(),
            last_seq: self.last_seq,
            state: serde_json::to_value(&self.state)?,
        };
        write_atomic(path, &serde_json::to_vec(&snapshot)?)?;
        Ok(())
    }

    /// Load a snapshot written by `write_snapshot`. An unknown schema is an
    /// error (fail closed); the caller falls back to a full replay via
    /// `Projection::new` + `catch_up`.
    pub fn load_snapshot(
        path: impl AsRef<Path>,
        reducer: Reducer<S>,
    ) -> Result<Self, ProjectionError> {
        let bytes = fs::read(path.as_ref())?;
        let snapshot: SnapshotFile = serde_json::from_slice(&bytes)?;
        if snapshot.schema != SNAPSHOT_SCHEMA {
            return Err(ProjectionError::UnknownSnapshotSchema {
                found: snapshot.schema,
            });
        }
        Ok(Self {
            state: serde_json::from_value(snapshot.state)?,
            last_seq: snapshot.last_seq,
            reducer,
        })
    }
}

/// The one concrete M1 projection: per-kind counts, last seq, and the set of
/// distinct `work_id`s seen — enough to prove determinism without inventing
/// domain semantics early.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreStats {
    /// Event count per `kind`.
    pub counts_by_kind: std::collections::BTreeMap<String, u64>,
    /// Seq of the last event folded in.
    pub last_seq: u64,
    /// Every distinct `work_id` seen.
    pub work_ids: std::collections::BTreeSet<String>,
}

/// Reducer for [`CoreStats`].
pub fn core_stats_reducer(state: &mut CoreStats, event: &Event) {
    *state.counts_by_kind.entry(event.kind.clone()).or_insert(0) += 1;
    state.last_seq = event.seq;
    if let Some(work_id) = &event.work_id {
        state.work_ids.insert(work_id.clone());
    }
}

/// An empty [`CoreStats`] projection ready to fold the journal.
pub fn core_stats_projection() -> Projection<CoreStats> {
    Projection::new(CoreStats::default(), core_stats_reducer)
}

/// Recorded outcome of an accepted or rejected mutation command (§26).
///
/// The daemon journals every command's result; a repeated `command_id`
/// replays this record verbatim instead of re-executing — including across a
/// daemon restart, because the registry is rebuilt from the journal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandOutcome {
    /// HTTP status the original execution answered with.
    pub status: u16,
    /// The exact JSON body the original execution answered with. Replaying a
    /// duplicate serializes this same value, so the response is
    /// byte-identical (serde_json maps are order-stable `BTreeMap`s).
    pub result: Value,
}

/// The real Work registry (M2): the M1 `CoreStats` demo projection evolved
/// into domain state, as contracted. Current work records plus the command
/// idempotency ledger, all folded purely from the journal.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WorkRegistry {
    /// Every work record, keyed by work id (ULID keys sort chronologically).
    pub works: std::collections::BTreeMap<String, Work>,
    /// Outcome of every accepted/rejected command, keyed by `command_id`.
    pub commands: std::collections::BTreeMap<String, CommandOutcome>,
    /// Work id created by each submit command, keyed by `command_id` (from
    /// the correlation id the mutation event itself carries).
    ///
    /// This is the crash-window index. A submit is two fsynced appends —
    /// `work.submitted`, then the `command.accepted` record — and a daemon
    /// that dies between them leaves a durable Work record that `commands`
    /// knows nothing about. Without this index a client retry of the same
    /// `command_id` would look brand new and create a *second* Work record,
    /// breaking exact-once for the one case §26 exists to serve: retry after
    /// an uncertain outcome.
    pub command_works: std::collections::BTreeMap<String, String>,
    /// Execution state for every work that has one, keyed by work id. This is
    /// the coordinate §10 calls "orthogonal": `works[id].state` is the §10
    /// work state, `runs[id]` is where the run currently *is*.
    #[serde(default)]
    pub runs: std::collections::BTreeMap<String, WorkRun>,
    /// R-MVP1-9 (Rule A)'s eviction destination: a compacted `WorkRun` for
    /// the most recent [`TERMINAL_RUN_CACHE_CAPACITY`] works `maybe_evict`
    /// has reclaimed from `runs`, keyed by work id.
    ///
    /// This is what makes eviction view-transparent (the ruling's own pin)
    /// for the realistic case — a just-finished work someone is actually
    /// looking at — *without* paying a full journal replay on every read:
    /// `resolve_run` checks here before ever calling [`rederive_run`]. It is
    /// deliberately bounded, not a second unbounded map growing forever
    /// beside `runs`: an unbounded cache here would silently reopen the
    /// exact monotonic-climb defect (#4) R-MVP1-9 exists to close, just
    /// under a different field name. A work aged out of the cache (older
    /// than the most recent `TERMINAL_RUN_CACHE_CAPACITY` evictions) falls
    /// back to a full replay — rare under ordinary browsing, not the
    /// self-defeating "every view, every terminal work" shape this cache
    /// replaces.
    ///
    /// "Compacted", not "identical to the live run": `stages` is truncated
    /// to the one current attempt (`current_stage()` already only ever
    /// reads `.last()`, so nothing a view composes can tell the difference)
    /// — the earlier attempts are exactly the "heavy state" eviction exists
    /// to reclaim, and every other field (`surface`, `teardown`, `backend`,
    /// `workflow`, …) is kept whole, since those are what the output
    /// pointer and `work show` actually render.
    ///
    /// In-memory only, like `runs` itself: a restart's rebuild-on-start
    /// re-folds the whole journal (`work_registry_reducer`), which
    /// re-populates this exactly as it did the first time (capped the same
    /// way) — there is no separate persistence path to keep in sync.
    #[serde(default)]
    pub terminal_runs: std::collections::BTreeMap<String, WorkRun>,
    /// Insertion order for `terminal_runs`, oldest first — the only state
    /// that makes it a *bounded* cache rather than a second `runs` map:
    /// `maybe_evict` pops the front and removes it from `terminal_runs`
    /// whenever a new insertion would exceed
    /// [`TERMINAL_RUN_CACHE_CAPACITY`].
    #[serde(default)]
    pub terminal_run_order: std::collections::VecDeque<String>,
    /// MVP-3's admission drain flag (`sgt daemon stop`, scoped exactly to
    /// that use): whether `POST /v1/work` currently refuses new
    /// submissions. Folded from `admission.paused`/`admission.resumed`
    /// (`daemon.rs`) — a plain `bool`, not a counter, so pausing twice in a
    /// row (a `daemon stop` retry against a still-live daemon) and resuming
    /// an already-resumed daemon (startup's own unconditional clear) are
    /// both idempotent by construction rather than by a caller remembering
    /// to check first.
    #[serde(default)]
    pub admission_paused: bool,
}

/// Bound on how many terminal runs [`WorkRegistry::terminal_runs`] holds at
/// once. Sized well above ordinary interactive browsing (an operator or
/// dashboard looking at recently finished work) so the cache is a hit for
/// the realistic case, while still guaranteeing the map cannot grow without
/// bound under sustained churn (R-MVP1-9's own flat-RSS pin) — an unbounded
/// version of this cache would just move #4's monotonic climb from `runs`
/// to here.
const TERMINAL_RUN_CACHE_CAPACITY: usize = 512;

impl WorkRegistry {
    /// The run for `work_id`, live or evicted-but-cached — the one lookup
    /// every API view should use instead of reading `runs` directly, so a
    /// terminal work's view keeps working after eviction without a caller
    /// having to know eviction happened at all.
    pub fn run_view(&self, work_id: &str) -> Option<&WorkRun> {
        self.runs
            .get(work_id)
            .or_else(|| self.terminal_runs.get(work_id))
    }
}

/// Everything the journal says about one work's run: the workflow it pinned,
/// the stages it has attempted, the surface it materialized, and the
/// execution it last started.
///
/// Deliberately separate from [`Work`]: §10 keeps workflow stage orthogonal to
/// work state so that "in review" can never become a top-level state-machine
/// value. Keeping them in different structs is that rule made structural.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WorkRun {
    /// The workflow definition this run pinned at bind time. Editing the
    /// workflow files afterwards cannot change a run in flight.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow: Option<WorkflowDefinition>,
    /// What the engine was *about* to materialize, journaled before the first
    /// worktree is created. It is the only record of git side effects that a
    /// crash mid-materialization can leave in the user's repositories.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface_plan: Option<SurfacePlan>,
    /// The work surface, if one was materialized.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface: Option<WorkSurface>,
    /// Teardown report, once the surface has been torn down.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teardown: Option<TeardownReport>,
    /// §11.5's integrity axis, as the retirement that recorded the teardown
    /// assessed it. Orthogonal to [`Work::state`](crate::domain::work::Work),
    /// which this never touches (ADR 0007(b)'s pattern: journal and view
    /// metadata, not a state-machine value).
    ///
    /// `None` is **not assessed**, never clean: a `surface.torn_down`
    /// journaled before this phase carries no assessment, and neither does
    /// the rollback teardown `materialize` runs on a partial failure. Both
    /// replay to `None` and every view says so (C3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity: Option<IntegrityDisposition>,
    /// Every stage attempt, in the order they were entered.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stages: Vec<StageRecord>,
    /// The execution last started for this work.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionRecord>,
    /// An execution reserved but not yet observed to have launched (§14.2).
    ///
    /// Set by `execution.reserved` and cleared by the `execution.started`
    /// that settles it. A run holding one outside that window is a run whose
    /// launch phase never reported back — the crash window §22.5 injects
    /// into, and the one thing recovery must never guess about.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reservation: Option<ExecutionReservation>,
    /// Backend the run routed to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    /// Which §13 tier decided that backend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_source: Option<String>,
    /// Launch profile pinned at bind time (§14 — launch configuration only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<Profile>,
    /// The per-stage executor decisions pinned at bind time (§12.5, §17.5).
    ///
    /// Empty for a run bound before N3, and for one whose every stage takes
    /// the Work actor default — the two are the same fact, and `backend` /
    /// `profile` above carry it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stage_bindings: Vec<StageBinding>,
    /// R-MVP1-4's per-repository instruction-file identities, pinned at bind
    /// time (`Engine::resolve_instruction_identities`). Uniform policy
    /// across every entry by construction (`check_instruction_policy`
    /// refuses submission otherwise) — MVP-2 D2 item 1 reads
    /// `.first().policy` off this to resolve the one
    /// [`crate::domain::workspace::InstructionPolicy`] a `StartRequest`/
    /// `ResumeRequest` carries. Empty for a run bound before R-MVP1-4, which
    /// resolves to [`crate::domain::workspace::InstructionPolicy::Suppress`]
    /// the same way an absent manifest entry does.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instruction_identities: Vec<InstructionIdentity>,
    /// R-MVP1-7's turn envelope: how many turns this Work has spawned across
    /// every stage and every attempt, for the life of the run. Incremented
    /// exactly where a turn is actually spawned — `execution.started`
    /// (LAUNCH) and `stage.resumed` (SEND, committed once per delivered
    /// turn regardless of how many times `PendingSend::perform` retried the
    /// backend call internally, engine.rs:258-266's double-send trap) — so
    /// it is "spawned turns" per the journal, never "verb calls". Cumulative
    /// and never reset by a retry: the envelope bounds the whole Work's
    /// spend, not one attempt's.
    #[serde(default)]
    pub turns_spawned: u32,
    /// R-MVP1-10's exit door for R-MVP1-7's envelope-exhausted landing
    /// (`Engine::extend_turn_envelope`): turns added to `Engine::turn_cap`
    /// for this Work specifically, cumulative across every extension. Zero
    /// for every Work that has never had its envelope extended — the
    /// ordinary case, and why this changes no existing test. Never reset by
    /// a retry, exactly like `turns_spawned` itself: extending the envelope
    /// is a durable, explicit operator decision about this Work's whole
    /// life, not a per-attempt allowance.
    #[serde(default)]
    pub turn_cap_bonus: u32,
}

impl WorkRun {
    /// The current (most recent) stage attempt, if any.
    pub fn current_stage(&self) -> Option<&StageRecord> {
        self.stages.last()
    }

    /// Whether the journal shows a start that got part-way and stopped: the
    /// run has a record of *something* the engine did, but the work never
    /// reached `active`. Recovery uses this to find works stranded in the
    /// submit crash window (§25 fails those closed rather than leaving them
    /// pending forever).
    pub fn is_started(&self) -> bool {
        self.surface_plan.is_some()
            || self.surface.is_some()
            || self.workflow.is_some()
            || self.execution.is_some()
            || self.reservation.is_some()
    }

    /// The reservation whose launch never reported back, if there is one.
    ///
    /// A reservation the matching `execution.started` cleared is not one of
    /// these; neither is one whose execution the journal *does* record. What
    /// is left is precisely the two-phase window: sergeant committed to an
    /// execution identity and the journal never learned whether the external
    /// effect happened.
    pub fn unsettled_reservation(&self) -> Option<&ExecutionReservation> {
        let reservation = self.reservation.as_ref()?;
        match &self.execution {
            Some(execution) if reservation.was_launched_as(execution) => None,
            _ => Some(reservation),
        }
    }

    /// The pinned executor decision for one stage of this run (§12.5).
    ///
    /// Matched on the stage id *and* its index: a workflow may legally repeat
    /// a harness across stages, but two stages never share a coordinate, and
    /// resolving by id alone would let a workflow whose author reused an id
    /// silently borrow another stage's harness.
    pub fn stage_binding(&self, stage_id: &str, index: usize) -> Option<&StageBinding> {
        self.stage_bindings
            .iter()
            .find(|b| b.stage_id == stage_id && b.index == index)
    }

    /// The launch profile in force for the stage this run is currently on.
    ///
    /// A stage with a pinned binding answers from it — including when the
    /// answer is "no profile", which for a stage on a harness the Work-level
    /// profile does not belong to is the only correct answer. Only a run with
    /// no bindings at all (bound before N3) falls back to the Work profile.
    pub fn current_stage_profile(&self) -> Option<Profile> {
        match self
            .current_stage()
            .and_then(|s| self.stage_binding(&s.stage_id, s.index))
        {
            Some(binding) => binding.profile.clone(),
            None => self.profile.clone(),
        }
    }

    fn last_stage_mut(&mut self, stage_id: &str) -> Option<&mut StageRecord> {
        self.stages
            .iter_mut()
            .rev()
            .find(|s| s.stage_id == stage_id)
    }
}

/// Reducer for [`WorkRegistry`]. Pure fold; no I/O.
///
/// Events are facts: legality was enforced by the writer before the event was
/// appended (transitions only happen via journal events, and only the daemon
/// writes them), so the reducer applies what the journal says. Kinds and
/// payload shapes this reducer does not understand are ignored, never an
/// error — a newer writer's events must not brick an older reader's replay
/// (§20's forward-compatibility stance).
///
/// Thin wrapper: the live registry (this daemon's in-memory projection, and
/// rebuild-on-start's full replay of it) is the one caller that should ever
/// evict — see [`apply_registry_event`] and [`rederive_run`].
pub fn work_registry_reducer(state: &mut WorkRegistry, event: &Event) {
    apply_registry_event(state, event, true);
}

/// Whether a §10 state is *absorbing* — the two states `WorkState::can_transition`
/// answers `false` for from every `to`, so nothing ever leaves them again
/// (`Failed` is deliberately excluded: it is "terminal for the run but
/// retryable by explicit operator action", per `WorkState`'s own doc comment,
/// and `retry` needs the run this function's caller may evict).
///
/// R-MVP1-9 (Rule A) evicts a run's heavy state — `WorkRun`'s stages,
/// execution history, surface and teardown records — exactly when nothing
/// can ever act on it again, which is precisely this set. Evicting `Failed`
/// too would delete state `retry` still needs to re-enter the stage; this is
/// the whole reason eviction is scoped to `is_absorbing`, not to every state
/// `WorkState`'s own doc comment happens to call "terminal".
pub fn is_absorbing(state: WorkState) -> bool {
    matches!(state, WorkState::Completed | WorkState::Canceled)
}

/// Whether `work_id`'s run is safe to evict *right now*, given the state
/// [`WorkRegistry`] already holds for it.
///
/// Absorbing state alone is not enough: `recovery::reconcile` (issue #9,
/// §22.5) finds work still needing attention after a restart by reading
/// exactly the fields eviction would delete — `run.surface`/`run.teardown`
/// for a stranded teardown, `run.unsettled_reservation()` for a stranded
/// two-phase launch. Evicting before those are settled would not just lose
/// data recovery could have used; it would make recovery's own filters see
/// nothing to reconcile, silently turning "needs attention" into "looks
/// fine". So this re-derives the identical two conditions recovery checks —
/// evicting exactly when they would already find nothing to do — which is
/// what makes eviction safe *for* recovery, not merely orthogonal to it.
///
/// I4: a third, narrower window neither of those two conditions alone
/// covers — `run.surface_plan` recorded (`KIND_SURFACE_MATERIALIZING`
/// committed) but `run.surface` still `None`, meaning `materialize` may
/// still be running outside the core lock (§14.2's own two-phase split).
/// `cancel` commits `work.canceled` — an absorbing, `surface.is_none()`
/// state — *before* that materialize call reports back, so without this
/// clause a run could be evicted mid-materialize; `begin_retire_run` would
/// then misread the now-empty `runs` entry as "nothing ever started", and
/// the materialize call's own eventual `KIND_SURFACE_MATERIALIZED` commit
/// would recreate a stripped run through the reducer's `or_default()` arm
/// (§14.5's stale-start teardown check does still tear it down correctly —
/// this clause closes the window rather than relying on that self-
/// correction alone). A run with no plan at all (no repositories to
/// materialize, `Engine::plan`'s "no surface" case) never sets
/// `surface_plan`, so this never blocks a genuinely surface-less Work.
fn run_is_settled(work: &Work, run: &WorkRun) -> bool {
    is_absorbing(work.state)
        && (run.surface.is_none() || run.teardown.is_some())
        && !(run.surface_plan.is_some() && run.surface.is_none())
        && run.unsettled_reservation().is_none()
}

/// Evict `work_id`'s run if [`run_is_settled`] says it is safe to. A no-op
/// for a work with no run recorded at all, or one not yet settled — called
/// unconditionally after every event, so it fires the instant the *last*
/// settling event lands, whichever of teardown/reservation-closure/state
/// transition that turns out to be for a given run.
fn maybe_evict(state: &mut WorkRegistry, work_id: Option<&str>) {
    let Some(work_id) = work_id else { return };
    let Some(work) = state.works.get(work_id) else {
        return;
    };
    let Some(run) = state.runs.get(work_id) else {
        return;
    };
    if !run_is_settled(work, run) {
        return;
    }
    let Some(mut run) = state.runs.remove(work_id) else {
        return;
    };
    // Keep only the current (last) stage attempt: every view reads stages
    // through `current_stage()`, which already only ever looks at
    // `.last()`, so every earlier attempt is unreachable from any view and
    // is exactly the "heavy state" R-MVP1-9 exists to reclaim.
    if run.stages.len() > 1 {
        let last = run.stages.pop();
        run.stages.clear();
        run.stages.extend(last);
    }
    state.terminal_runs.insert(work_id.to_string(), run);
    state.terminal_run_order.push_back(work_id.to_string());
    while state.terminal_run_order.len() > TERMINAL_RUN_CACHE_CAPACITY {
        if let Some(oldest) = state.terminal_run_order.pop_front() {
            state.terminal_runs.remove(&oldest);
        }
    }
}

/// The shared fold behind both [`work_registry_reducer`] (`evict: true`) and
/// [`rederive_run`] (`evict: false`).
///
/// Eviction must be optional here, not merely absent from a second copy of
/// this match: `rederive_run` re-derives one work's run by folding *only that
/// work's own events* — including its own terminal transition and teardown —
/// through this exact function, and evicting mid-fold would delete the very
/// run being reconstructed before `rederive_run` ever reads it back out. One
/// fold, one flag, so the live registry and the read-time re-derivation can
/// never drift into two different ideas of what a work's run looked like.
fn apply_registry_event(state: &mut WorkRegistry, event: &Event, evict: bool) {
    // MVP-3's admission drain flag: a plain fold, checked before the
    // per-work dispatch below since neither event carries a `work_id`.
    match event.kind.as_str() {
        KIND_ADMISSION_PAUSED => {
            state.admission_paused = true;
            return;
        }
        KIND_ADMISSION_RESUMED => {
            state.admission_paused = false;
            return;
        }
        _ => {}
    }
    // Work-state transitions: one mapping, shared with the writer
    // (`WorkState::for_event_kind`), so a kind cannot mean one state when
    // appended and another when replayed.
    if let Some(new_state) = WorkState::for_event_kind(&event.kind) {
        if let Some(work) = event
            .work_id
            .as_ref()
            .and_then(|id| state.works.get_mut(id))
        {
            work.state = new_state;
        }
        if evict {
            maybe_evict(state, event.work_id.as_deref());
        }
        return;
    }
    match event.kind.as_str() {
        KIND_WORK_SUBMITTED => {
            if let Ok(work) = serde_json::from_value::<Work>(event.payload["work"].clone()) {
                if let Some(command_id) = &event.correlation_id {
                    state
                        .command_works
                        .insert(command_id.clone(), work.id.clone());
                }
                state.works.insert(work.id.clone(), work);
            }
        }
        KIND_WORKFLOW_BOUND => {
            if let (Some(work_id), Ok(workflow)) = (
                event.work_id.as_ref(),
                serde_json::from_value::<WorkflowDefinition>(event.payload["workflow"].clone()),
            ) {
                let run = state.runs.entry(work_id.clone()).or_default();
                run.workflow = Some(workflow);
                run.backend = event.payload["backend"].as_str().map(str::to_string);
                run.route_source = event.payload["route_source"].as_str().map(str::to_string);
                run.profile = serde_json::from_value(event.payload["profile"].clone()).ok();
                // Additive (§20.5). A `workflow.bound` journaled before N3's
                // per-stage selection carries no `stage_bindings`, and an
                // empty vec is exactly what that meant: every stage runs on
                // the Work-level actor default, which `backend`/`profile`
                // above already record.
                run.stage_bindings =
                    serde_json::from_value(event.payload["stage_bindings"].clone())
                        .unwrap_or_default();
                // Additive (§20.5), same reasoning as `stage_bindings` just
                // above: a `workflow.bound` from before R-MVP1-4 carries no
                // `instruction_identities`, and an empty vec is exactly what
                // that meant — every repository resolves to `Suppress`.
                run.instruction_identities =
                    serde_json::from_value(event.payload["instruction_identities"].clone())
                        .unwrap_or_default();
            }
        }
        KIND_SURFACE_MATERIALIZING => {
            if let (Some(work_id), Ok(plan)) = (
                event.work_id.as_ref(),
                serde_json::from_value::<SurfacePlan>(event.payload["plan"].clone()),
            ) {
                state.runs.entry(work_id.clone()).or_default().surface_plan = Some(plan);
            }
        }
        KIND_SURFACE_MATERIALIZED => {
            if let (Some(work_id), Ok(surface)) = (
                event.work_id.as_ref(),
                serde_json::from_value::<WorkSurface>(event.payload["surface"].clone()),
            ) {
                let run = state.runs.entry(work_id.clone()).or_default();
                run.surface = Some(surface);
                run.teardown = None;
                // A rematerialized surface has not been assessed again yet;
                // carrying the previous retirement's verdict forward would
                // report a fact about a teardown this run has replaced.
                run.integrity = None;
            }
        }
        KIND_SURFACE_TORN_DOWN => {
            if let (Some(work_id), Ok(report)) = (
                event.work_id.as_ref(),
                serde_json::from_value::<TeardownReport>(event.payload["report"].clone()),
            ) {
                let run = state.runs.entry(work_id.clone()).or_default();
                run.teardown = Some(report);
                // Additive (§20.5 / C3): a `surface.torn_down` from before
                // this phase — and the rollback teardown `materialize`
                // journals on a partial failure — carry no `integrity` key,
                // and `from_value(Null)` is an error, so both read back as
                // "not assessed" rather than being defaulted to clean.
                run.integrity = serde_json::from_value(event.payload["integrity"].clone()).ok();
            }
        }
        KIND_STAGE_ENTERED => {
            if let (Some(work_id), Some(stage_id), Some(index)) = (
                event.work_id.as_ref(),
                event.payload["stage_id"].as_str(),
                event.payload["index"].as_u64(),
            ) {
                let run = state.runs.entry(work_id.clone()).or_default();
                run.stages.push(StageRecord {
                    stage_id: stage_id.to_string(),
                    index: index as usize,
                    attempt: event.payload["attempt"].as_u64().unwrap_or(1) as u32,
                    status: StageStatus::Active,
                    detail: None,
                });
            }
        }
        KIND_STAGE_COMPLETED
        | KIND_STAGE_WAITING
        | KIND_STAGE_NEEDS_INPUT
        | KIND_STAGE_BLOCKED
        | KIND_STAGE_FAILED
        | KIND_STAGE_CANCELED => {
            if let (Some(work_id), Some(stage_id)) =
                (event.work_id.as_ref(), event.payload["stage_id"].as_str())
            {
                let status = match event.kind.as_str() {
                    KIND_STAGE_COMPLETED => StageStatus::Completed,
                    KIND_STAGE_WAITING => StageStatus::Waiting,
                    KIND_STAGE_NEEDS_INPUT => StageStatus::NeedsInput,
                    KIND_STAGE_BLOCKED => StageStatus::Blocked,
                    KIND_STAGE_FAILED => StageStatus::Failed,
                    _ => StageStatus::Canceled,
                };
                let detail = event.payload["detail"].as_str().map(str::to_string);
                if let Some(stage) = state
                    .runs
                    .entry(work_id.clone())
                    .or_default()
                    .last_stage_mut(stage_id)
                {
                    stage.status = status;
                    stage.detail = detail;
                }
            }
        }
        KIND_STAGE_RESUMED => {
            // BS2 / issue #46's second seam: a `needs_input` stage whose
            // answer was delivered is live again, under the same attempt —
            // not a new one, so (unlike `KIND_STAGE_ENTERED`) this updates
            // the existing record in place rather than pushing a new one.
            // This is what makes the stage due for the completion driver's
            // `due_observations` again (it requires `StageStatus::Active`).
            if let (Some(work_id), Some(stage_id)) =
                (event.work_id.as_ref(), event.payload["stage_id"].as_str())
                && let Some(stage) = state
                    .runs
                    .entry(work_id.clone())
                    .or_default()
                    .last_stage_mut(stage_id)
            {
                stage.status = StageStatus::Active;
                stage.detail = None;
                // R-MVP1-7: SEND spawns every turn after the first. This is
                // the one commit site for `stage.resumed` (`Engine::settle_send`),
                // so it is exactly "one delivered turn", never the internal
                // resume-and-retry a stale send can take inside
                // `PendingSend::perform`.
                state.runs.entry(work_id.clone()).or_default().turns_spawned += 1;
            }
        }
        KIND_EXECUTION_RESERVED => {
            if let (Some(work_id), Ok(reservation)) = (
                event.work_id.as_ref(),
                serde_json::from_value::<ExecutionReservation>(
                    event.payload["reservation"].clone(),
                ),
            ) {
                state.runs.entry(work_id.clone()).or_default().reservation = Some(reservation);
            }
        }
        KIND_EXECUTION_STARTED => {
            if let (Some(work_id), Ok(execution)) = (
                event.work_id.as_ref(),
                serde_json::from_value::<ExecutionRecord>(event.payload["execution"].clone()),
            ) {
                let run = state.runs.entry(work_id.clone()).or_default();
                // The reservation this start settles stops being outstanding.
                // A start that settles a *different* reservation leaves it
                // standing, because that one is still unaccounted for — the
                // window is about the identity, not about the clock.
                if run
                    .reservation
                    .as_ref()
                    .is_some_and(|r| r.was_launched_as(&execution))
                {
                    run.reservation = None;
                }
                run.execution = Some(execution);
                // R-MVP1-7: LAUNCH spawns turn 1 of a stage attempt. This is
                // the one commit site for a successful `execution.started`
                // (`Engine::settle_launch`), so it counts one spawned turn
                // per settled launch, never per LAUNCH attempt (a stale or
                // failed launch never reaches here — see the abandonment
                // arms above).
                run.turns_spawned += 1;
            }
        }
        crate::runtime::engine::KIND_TURN_ENVELOPE_EXTENDED => {
            // R-MVP1-10's exit door: raises the bonus `check_turn_envelope`
            // adds to `Engine::turn_cap` for this Work, cumulative across
            // every extension (`Engine::extend_turn_envelope` is the one
            // commit site). A run with no `runs` entry yet has nothing to
            // extend — `extend_turn_envelope` never reaches this state
            // (requires `Blocked`, which requires a run), so `or_default()`
            // here is defensive, not a real code path.
            if let (Some(work_id), Some(additional)) = (
                event.work_id.as_ref(),
                event.payload["additional_turns"].as_u64(),
            ) {
                let run = state.runs.entry(work_id.clone()).or_default();
                run.turn_cap_bonus = run.turn_cap_bonus.saturating_add(additional as u32);
            }
        }
        KIND_EXECUTION_STOPPED => {
            // The latch records that the *backend was actually asked and did
            // not refuse* — not merely that sergeant journaled an attempt.
            // `stop_requested` is what makes STOP idempotent (the engine
            // skips an execution that carries it), so latching on an attempt
            // that never reached a native context turns every later stop —
            // including a human's cancel — into a permanent no-op against a
            // context nobody ever asked to die. An attempt that names an
            // error, or that never reached a registered backend, leaves the
            // latch open so the next caller tries again; the attempt itself
            // is journaled either way, which is the evidence.
            let acknowledged = event.payload["outcome"]["requested"] == Value::Bool(true)
                && event.payload["outcome"]["error"].is_null();
            if !acknowledged {
                return;
            }
            let Some(run) = event.work_id.as_ref().and_then(|id| state.runs.get_mut(id)) else {
                return;
            };
            // The latch belongs to the execution the event *names*. Before the
            // two-phase boundary every `execution.stopped` was about the run's
            // current execution and the id went unread; now a superseded
            // launch can be stopped while a different execution is current,
            // and latching that one would make a later cancel a permanent
            // no-op against a live native context nobody ever asked to die.
            let named = event.payload["execution_id"].as_str();
            if let Some(execution) = run.execution.as_mut()
                && named.is_none_or(|id| id == execution.execution_id)
            {
                execution.stop_requested = true;
            }
        }
        KIND_EXECUTION_ABANDONED => {
            // The reservation is accounted for — whatever happened to it, the
            // journal now says so, and it is no longer an open two-phase
            // window for recovery to fail closed on.
            if let Some(run) = event.work_id.as_ref().and_then(|id| state.runs.get_mut(id))
                && let Some(reservation) = run.reservation.as_ref()
                && event.payload["execution_id"].as_str() == Some(reservation.execution_id.as_str())
            {
                run.reservation = None;
            }
        }
        KIND_COMMAND_ACCEPTED | KIND_COMMAND_REJECTED => {
            if let (Some(command_id), Some(status)) = (
                event.payload["command_id"].as_str(),
                event.payload["status"].as_u64(),
            ) {
                state.commands.insert(
                    command_id.to_string(),
                    CommandOutcome {
                        status: u16::try_from(status).unwrap_or(500),
                        result: event.payload["result"].clone(),
                    },
                );
            }
        }
        _ => {}
    }
    if evict {
        maybe_evict(state, event.work_id.as_deref());
    }
}

/// An empty [`WorkRegistry`] projection ready to fold the journal.
pub fn work_registry_projection() -> Projection<WorkRegistry> {
    Projection::new(WorkRegistry::default(), work_registry_reducer)
}

/// R-MVP1-9's read side: rebuild one work's [`WorkRun`] straight from the
/// journal, for a caller that finds `WorkRegistry::runs` no longer has it —
/// [`is_absorbing`] eviction having already reclaimed it.
///
/// Folds *only this work's own events* — every other event in the journal is
/// skipped before it ever reaches [`apply_registry_event`] — through the
/// exact reducer logic the live registry uses (`evict: false`, so the
/// terminal transition this replay necessarily includes does not immediately
/// delete what it just rebuilt). The result is therefore byte-identical to
/// what `WorkRegistry::runs` would still hold had eviction never run: same
/// events, same fold, same order, the only difference is when it happens
/// (lazily, at read time, instead of once and kept).
///
/// Returns `Ok(None)` for a work whose events never touched `runs` at all
/// (never started, or genuinely unknown) — the same answer a live,
/// non-evicted registry would give for that work.
pub fn rederive_run(journal: &Journal, work_id: &str) -> Result<Option<WorkRun>, JournalError> {
    let mut scratch = WorkRegistry::default();
    for event in journal.replay()? {
        let event = event?;
        if event.work_id.as_deref() == Some(work_id) {
            apply_registry_event(&mut scratch, &event, false);
        }
    }
    Ok(scratch.runs.remove(work_id))
}

#[cfg(test)]
mod rule_a_eviction_tests {
    //! R-MVP1-9 (Rule A): eviction, its two-condition safety gate, and
    //! [`rederive_run`]'s read-side reconstruction.

    use serde_json::json;

    use super::*;
    use crate::domain::execution::{KIND_EXECUTION_ABANDONED, KIND_EXECUTION_RESERVED};
    use crate::domain::work::{KIND_WORK_CANCELED, KIND_WORK_COMPLETED, KIND_WORK_FAILED};
    use crate::runtime::journal::Journal;
    use crate::runtime::testing;

    fn a_surface(work_id: &str) -> serde_json::Value {
        json!({"surface": {
            "work_id": work_id,
            "root": "/data/surfaces/x",
            "bindings": [{
                "repository": "solo",
                "source_path": "/repos/solo",
                "base_branch": "main",
                "base_sha": "0".repeat(40),
                "worktree_path": "/data/surfaces/x/solo",
                "work_branch": format!("sergeant/{work_id}"),
                "head_sha": "0".repeat(40),
            }],
        }})
    }

    fn a_plan(work_id: &str) -> serde_json::Value {
        json!({"plan": {
            "root": format!("/data/surfaces/{work_id}"),
            "work_branch": format!("sergeant/{work_id}"),
            "repositories": [{"name": "solo", "path": "/repos/solo"}],
        }})
    }

    fn a_teardown(work_id: &str) -> serde_json::Value {
        json!({"report": {
            "work_id": work_id,
            "clean": true,
            "bindings": [{
                "repository": "solo",
                "worktree_path": "/data/surfaces/x/solo",
                "work_branch": format!("sergeant/{work_id}"),
                "disposition": "removed",
            }],
        }})
    }

    /// `is_absorbing` matches exactly the two states `WorkState::can_transition`
    /// answers `false` for every `to` — the set §25's own doc comment calls
    /// absorbing, not the broader set `WorkState`'s per-variant doc comments
    /// individually call "terminal" (which also includes `Failed`, and
    /// `Failed` is retryable — evicting it would break `retry`).
    #[test]
    fn is_absorbing_is_exactly_the_two_states_nothing_ever_leaves() {
        use WorkState::*;
        for state in [
            Pending, Active, Waiting, NeedsInput, Blocked, Completed, Failed, Canceled,
        ] {
            let never_transitions_anywhere = [
                Pending, Active, Waiting, NeedsInput, Blocked, Completed, Failed, Canceled,
            ]
            .iter()
            .all(|&to| !state.can_transition(to));
            assert_eq!(
                is_absorbing(state),
                never_transitions_anywhere,
                "state {state}"
            );
        }
        assert!(
            !is_absorbing(WorkState::Failed),
            "Failed must stay retryable"
        );
    }

    /// Eviction waits for *both* settling conditions, in either order: a
    /// completed work whose surface is still standing is not evicted even
    /// though its state is absorbing, and neither is one whose surface is
    /// down but a reservation is still open. Only once both clear does the
    /// run go — and it goes on whichever event turns out to be the second
    /// one to settle, without caring which that was.
    #[test]
    fn eviction_waits_for_both_the_surface_and_the_reservation_to_settle() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());
        let work_id = "01EVICT1";

        testing::submit(&mut core, work_id, "evict me");
        testing::commit(
            &mut core,
            work_id,
            KIND_SURFACE_MATERIALIZED,
            a_surface(work_id),
        );
        testing::commit(
            &mut core,
            work_id,
            KIND_EXECUTION_RESERVED,
            json!({"reservation": {
                "execution_id": "01EXEC",
                "backend": "fake",
                "native_id": "n1",
                "stage_id": "00-first",
                "index": 0,
                "attempt": 1,
                "stage_kind": "actor",
            }}),
        );
        testing::commit(&mut core, work_id, KIND_WORK_COMPLETED, json!({}));
        assert!(
            core.registry.state().runs.contains_key(work_id),
            "absorbing state alone must not evict a run whose surface is still up \
             and whose reservation is still open"
        );

        testing::commit(
            &mut core,
            work_id,
            KIND_SURFACE_TORN_DOWN,
            a_teardown(work_id),
        );
        assert!(
            core.registry.state().runs.contains_key(work_id),
            "the surface settling is not enough on its own — the reservation \
             is still open, and recovery still needs to see it"
        );

        testing::commit(
            &mut core,
            work_id,
            KIND_EXECUTION_ABANDONED,
            json!({"execution_id": "01EXEC"}),
        );
        assert!(
            !core.registry.state().runs.contains_key(work_id),
            "both conditions are now settled — the run must be gone"
        );
    }

    /// I4: the third window — `surface.planned` recorded but `surface`
    /// never materialized — must not evict either, even though
    /// `run.surface.is_none()` alone would satisfy the first two
    /// conditions. This is exactly the shape a `cancel` landing while
    /// `materialize` is still running outside the lock produces:
    /// `work.canceled` (absorbing) lands before `surface.materialized`
    /// (or a failure) ever does.
    #[test]
    fn eviction_waits_for_a_planned_but_not_yet_materialized_surface_too() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());
        let work_id = "01EVICTPLAN";

        testing::submit(&mut core, work_id, "cancel mid-materialize");
        testing::commit(
            &mut core,
            work_id,
            KIND_SURFACE_MATERIALIZING,
            a_plan(work_id),
        );
        // The race: cancellation lands while materialize is still running
        // outside the lock — an absorbing state with surface_plan set and
        // surface still None.
        testing::commit(&mut core, work_id, KIND_WORK_CANCELED, json!({}));
        assert!(
            core.registry.state().runs.contains_key(work_id),
            "a planned-but-not-yet-materialized surface must not be evicted just \
             because the Work already transitioned to an absorbing state — \
             materialize may still be running outside the lock"
        );

        // materialize eventually reports back: the surface exists now, so
        // the first condition's own gate takes over (surface present,
        // teardown not yet) — still not settled.
        testing::commit(
            &mut core,
            work_id,
            KIND_SURFACE_MATERIALIZED,
            a_surface(work_id),
        );
        assert!(
            core.registry.state().runs.contains_key(work_id),
            "a materialized-but-not-torn-down surface still must not be evicted"
        );

        // The stale-start teardown (§14.5) tears it down; now every
        // condition is settled.
        testing::commit(
            &mut core,
            work_id,
            KIND_SURFACE_TORN_DOWN,
            a_teardown(work_id),
        );
        assert!(
            !core.registry.state().runs.contains_key(work_id),
            "once the surface is torn down, every condition is settled and the \
             run must be evicted"
        );
    }

    /// A run whose terminal transition alone settles both conditions (no
    /// surface was ever materialized, no execution was ever reserved) is
    /// evicted the instant the terminal event lands — there is nothing else
    /// to wait for.
    #[test]
    fn a_run_with_nothing_outstanding_is_evicted_at_the_terminal_transition_itself() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());
        let work_id = "01EVICT2";

        testing::submit(&mut core, work_id, "cancel me early");
        testing::commit(
            &mut core,
            work_id,
            KIND_WORK_CANCELED,
            json!({"from": "pending"}),
        );
        assert!(
            !core.registry.state().runs.contains_key(work_id),
            "nothing was outstanding, so eviction has nothing to wait for"
        );
        // The light `Work` record — never evicted — still answers.
        assert_eq!(
            core.registry.state().works[work_id].state,
            WorkState::Canceled
        );
    }

    /// `Failed` is deliberately excluded from `is_absorbing`: it is
    /// retryable, and `retry` needs exactly the run state eviction would
    /// otherwise delete. A failed work's run must survive its own teardown.
    #[test]
    fn a_failed_work_is_never_evicted_because_retry_needs_its_run() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());
        let work_id = "01NOEVICT";

        testing::submit(&mut core, work_id, "fail me");
        testing::commit(
            &mut core,
            work_id,
            KIND_SURFACE_MATERIALIZED,
            a_surface(work_id),
        );
        testing::commit(&mut core, work_id, KIND_WORK_FAILED, json!({}));
        testing::commit(
            &mut core,
            work_id,
            KIND_SURFACE_TORN_DOWN,
            a_teardown(work_id),
        );

        assert!(
            core.registry.state().runs.contains_key(work_id),
            "a failed work's run must still be there for retry to re-enter, \
             even though its surface has already been torn down"
        );
    }

    /// The pin, stated directly: `rederive_run` reconstructs a run that is
    /// byte-identical to the one a non-evicting fold of the identical events
    /// would produce. Same events, same reducer logic (`apply_registry_event`
    /// with `evict: false` either way), same order — the only variable this
    /// test isolates is *whether eviction ran in between*.
    #[test]
    fn rederive_run_is_byte_identical_to_a_run_that_was_never_evicted() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let journal_dir = dir.path().join("journal");
        let work_id = "01REDERIVE";

        // Build the journal once, through the *evicting* live registry —
        // exactly the daemon's own path.
        let mut core = testing::core(&journal_dir);
        testing::submit(&mut core, work_id, "rederive me");
        testing::commit(
            &mut core,
            work_id,
            KIND_SURFACE_MATERIALIZED,
            a_surface(work_id),
        );
        testing::commit(&mut core, work_id, KIND_WORK_COMPLETED, json!({}));
        testing::commit(
            &mut core,
            work_id,
            KIND_SURFACE_TORN_DOWN,
            a_teardown(work_id),
        );
        assert!(
            !core.registry.state().runs.contains_key(work_id),
            "precondition: this run must actually have been evicted, or the \
             comparison below proves nothing"
        );

        // The independent answer: fold the *same* journal from scratch with
        // eviction switched off, the way the registry would look if Rule A
        // had never been built at all. Reuses `core.journal`'s own handle —
        // the journal takes an exclusive lock on its directory, so a second
        // `Journal::open` here would find it held by `core` itself.
        fn non_evicting(state: &mut WorkRegistry, event: &Event) {
            apply_registry_event(state, event, false);
        }
        let mut never_evicted = Projection::new(WorkRegistry::default(), non_evicting);
        never_evicted
            .catch_up(core.journal.replay().expect("replay"))
            .expect("catch up");
        let expected = never_evicted
            .state()
            .runs
            .get(work_id)
            .cloned()
            .expect("the never-evicted fold must have a run for this work");

        let rederived = rederive_run(&core.journal, work_id)
            .expect("replay")
            .expect("a run to rederive");
        assert_eq!(
            rederived, expected,
            "the evicted work's re-derived run must be byte-identical to the \
             run a non-evicting fold of the same events would have produced"
        );
    }

    /// `rederive_run` answers `None` for a work whose events never touched
    /// `runs` at all — the same answer a live, non-evicted registry gives.
    #[test]
    fn rederive_run_of_a_work_with_no_run_ever_recorded_is_none() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());
        let work_id = "01NORUN";
        testing::submit(&mut core, work_id, "never started");

        let rederived = rederive_run(&core.journal, work_id).expect("replay");
        assert_eq!(rederived, None);
    }

    /// The S2 churn scenario's own claim, proven here at the projection
    /// level: `WorkRegistry::runs` — the structure #4's ~25 kB/work
    /// unbounded climb lived in — stays flat across many completed works,
    /// never one entry per work submitted. TH-08: the daemon-level half of
    /// this claim (real RSS, real fds, real hygiene, not a struct field
    /// count) is `docs/perf/s2-churn-mvp1-fixer-2026-08-12.md` — run against
    /// this same fix, decelerating per-wave slope, not the pre-eviction
    /// monotonic climb `docs/perf/baseline-cerberus-2026-08-11.md` measured.
    ///
    /// Mutation this kills: removing the eviction call (or narrowing
    /// `is_absorbing` to nothing) turns `runs.len()` into `N`, not `0`.
    #[test]
    fn many_completed_works_leave_the_runs_map_flat_not_growing_with_n() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());
        const N: usize = 300;
        for i in 0..N {
            let work_id = format!("01CHURN{i:05}");
            testing::submit(&mut core, &work_id, "churn");
            testing::commit(
                &mut core,
                &work_id,
                KIND_SURFACE_MATERIALIZED,
                a_surface(&work_id),
            );
            testing::commit(&mut core, &work_id, KIND_WORK_COMPLETED, json!({}));
            testing::commit(
                &mut core,
                &work_id,
                KIND_SURFACE_TORN_DOWN,
                a_teardown(&work_id),
            );
        }
        assert_eq!(
            core.registry.state().works.len(),
            N,
            "the light Work record is never evicted — the fleet listing still \
             sees every work"
        );
        assert_eq!(
            core.registry.state().runs.len(),
            0,
            "every one of the {N} runs settled with nothing outstanding and \
             must have been evicted — a non-flat count here is #4's leak back"
        );
    }

    /// W2/TH-08: `terminal_runs` (the read-side cache `resolve_run` checks
    /// before ever replaying the journal) must itself stay flat under
    /// sustained churn beyond its own capacity — an unbounded version of it
    /// would silently reopen #4's monotonic climb under a different field
    /// name, defeating the eviction this whole mechanism exists for.
    #[test]
    fn the_terminal_run_cache_itself_stays_bounded_under_churn_beyond_its_capacity() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());
        let total = TERMINAL_RUN_CACHE_CAPACITY + 200;
        for i in 0..total {
            let work_id = format!("01CACHECHURN{i:06}");
            testing::submit(&mut core, &work_id, "cache churn");
            testing::commit(
                &mut core,
                &work_id,
                KIND_SURFACE_MATERIALIZED,
                a_surface(&work_id),
            );
            testing::commit(&mut core, &work_id, KIND_WORK_COMPLETED, json!({}));
            testing::commit(
                &mut core,
                &work_id,
                KIND_SURFACE_TORN_DOWN,
                a_teardown(&work_id),
            );
        }
        let state = core.registry.state();
        assert_eq!(state.runs.len(), 0, "still fully evicted from the live map");
        assert_eq!(
            state.terminal_runs.len(),
            TERMINAL_RUN_CACHE_CAPACITY,
            "the terminal-run cache must never exceed its own capacity, however \
             many works settle — a growing count here is the leak this cache \
             would otherwise reintroduce"
        );
        assert_eq!(
            state.terminal_run_order.len(),
            TERMINAL_RUN_CACHE_CAPACITY,
            "the order queue and the cache map must shrink together"
        );
        // The earliest works aged out of the cache; the most recent
        // `TERMINAL_RUN_CACHE_CAPACITY` are still there — a real work
        // rederive_run can still answer for the aged-out ones (the fallback
        // path), never a lost record.
        let aged_out = "01CACHECHURN000000";
        let still_cached = format!("01CACHECHURN{:06}", total - 1);
        assert!(!state.terminal_runs.contains_key(aged_out));
        assert!(state.terminal_runs.contains_key(&still_cached));
    }

    /// Restart indifference: rebuild-on-start is a full replay through the
    /// same evicting reducer, so a work that was evicted before a restart is
    /// evicted again afterwards — never resurrected into memory just because
    /// the projection was rebuilt from scratch.
    #[test]
    fn a_restart_rebuild_re_evicts_exactly_what_was_evicted_before_it() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let journal_dir = dir.path().join("journal");
        let work_id = "01RESTART";
        {
            let mut core = testing::core(&journal_dir);
            testing::submit(&mut core, work_id, "restart me");
            testing::commit(
                &mut core,
                work_id,
                KIND_SURFACE_MATERIALIZED,
                a_surface(work_id),
            );
            testing::commit(&mut core, work_id, KIND_WORK_COMPLETED, json!({}));
            testing::commit(
                &mut core,
                work_id,
                KIND_SURFACE_TORN_DOWN,
                a_teardown(work_id),
            );
            assert!(!core.registry.state().runs.contains_key(work_id));
        }
        // A fresh process, rebuilding the same way `daemon::start` does:
        // `Projection::new` + `catch_up` over a full replay, nothing else.
        let journal = Journal::open(&journal_dir).expect("reopen journal");
        let mut rebuilt = work_registry_projection();
        rebuilt
            .catch_up(journal.replay().expect("replay"))
            .expect("catch up");
        assert!(
            !rebuilt.state().runs.contains_key(work_id),
            "rebuild-on-start must not resurrect an evicted run into memory"
        );
        assert_eq!(rebuilt.state().works[work_id].state, WorkState::Completed);
    }
}
