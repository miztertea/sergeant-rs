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
use crate::domain::work::{
    KIND_COMMAND_ACCEPTED, KIND_COMMAND_REJECTED, KIND_WORK_CANCELED, KIND_WORK_SUBMITTED, Work,
    WorkState,
};
use crate::runtime::fsutil::{create_dir_all_durable, write_atomic};
use crate::runtime::journal::JournalError;

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
        KIND_WORK_CANCELED => {
            if let Some(work) = event
                .work_id
                .as_ref()
                .and_then(|id| state.works.get_mut(id))
            {
                work.state = WorkState::Canceled;
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
