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

use crate::domain::event::Event;
use crate::domain::execution::{ExecutionRecord, KIND_EXECUTION_STARTED, KIND_EXECUTION_STOPPED};
use crate::domain::profile::Profile;
use crate::domain::work::{
    KIND_COMMAND_ACCEPTED, KIND_COMMAND_REJECTED, KIND_WORK_SUBMITTED, Work, WorkState,
};
use crate::domain::workflow::{
    KIND_STAGE_BLOCKED, KIND_STAGE_CANCELED, KIND_STAGE_COMPLETED, KIND_STAGE_ENTERED,
    KIND_STAGE_FAILED, KIND_STAGE_NEEDS_INPUT, KIND_STAGE_WAITING, KIND_WORKFLOW_BOUND,
    StageRecord, StageStatus, WorkflowDefinition,
};
use crate::runtime::fsutil::{create_dir_all_durable, write_atomic};
use crate::runtime::journal::JournalError;
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
    /// Every stage attempt, in the order they were entered.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stages: Vec<StageRecord>,
    /// The execution last started for this work.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionRecord>,
    /// Backend the run routed to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    /// Which §13 tier decided that backend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_source: Option<String>,
    /// Launch profile pinned at bind time (§14 — launch configuration only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<Profile>,
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
pub fn work_registry_reducer(state: &mut WorkRegistry, event: &Event) {
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
            }
        }
        KIND_SURFACE_TORN_DOWN => {
            if let (Some(work_id), Ok(report)) = (
                event.work_id.as_ref(),
                serde_json::from_value::<TeardownReport>(event.payload["report"].clone()),
            ) {
                state.runs.entry(work_id.clone()).or_default().teardown = Some(report);
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
        KIND_EXECUTION_STARTED => {
            if let (Some(work_id), Ok(execution)) = (
                event.work_id.as_ref(),
                serde_json::from_value::<ExecutionRecord>(event.payload["execution"].clone()),
            ) {
                state.runs.entry(work_id.clone()).or_default().execution = Some(execution);
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
            if let Some(execution) = event
                .work_id
                .as_ref()
                .and_then(|id| state.runs.get_mut(id))
                .and_then(|run| run.execution.as_mut())
            {
                execution.stop_requested = true;
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
}

/// An empty [`WorkRegistry`] projection ready to fold the journal.
pub fn work_registry_projection() -> Projection<WorkRegistry> {
    Projection::new(WorkRegistry::default(), work_registry_reducer)
}
