//! Execution: one native execution context owned by a backend (§15, §25).
//!
//! An Execution is *not* a Work and is *not* a stage. It is sergeant's record
//! of a native context a backend created on our behalf — the thing that can be
//! alive, dead, resumable or ambiguous while the Work it serves is in whatever
//! state the workflow put it in. §25 makes the separation absolute:
//!
//! ```text
//! native turn completed  ≠  work completed
//! native process alive   ≠  work active
//! native process dead    ≠  work failed
//! ```
//!
//! This module holds the record; [`crate::runtime::engine`] holds the rule
//! that only explicit signals — never liveness — move Work or stage state.

use serde::{Deserialize, Serialize};

/// Event kind: an execution id and its executor spec were reserved for a
/// stage attempt, *before* anything external was created (§14.2's first
/// phase).
pub const KIND_EXECUTION_RESERVED: &str = "execution.reserved";
/// Event kind: an execution was started for a stage.
pub const KIND_EXECUTION_STARTED: &str = "execution.started";
/// Event kind: an execution was asked to retire.
pub const KIND_EXECUTION_STOPPED: &str = "execution.stopped";
/// Event kind: a reservation was retired without ever becoming this run's
/// execution — the launch failed, or the durable state moved on underneath it
/// while it was in flight (§14.5).
pub const KIND_EXECUTION_ABANDONED: &str = "execution.abandoned";
/// Event kind: the daemon re-observed an execution after a restart (§25).
pub const KIND_EXECUTION_RECONCILED: &str = "execution.reconciled";

/// What sergeant knows about one native execution context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// Sergeant's id for this execution (ULID).
    pub execution_id: String,
    /// Backend that owns it.
    pub backend: String,
    /// The backend's own identity for the native context, when it has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_id: Option<String>,
    /// Stage this execution was started for.
    pub stage_id: String,
    /// Attempt number of that stage.
    pub attempt: u32,
    /// Whether sergeant has asked the backend to retire it. A stop *request*
    /// is all sergeant can know: whether the native context actually died is
    /// the backend's evidence to report, never our inference.
    pub stop_requested: bool,
}

/// What sergeant decided, and committed to, before any external effect
/// existed (§14.2: "allocate sergeant execution id / resolve and pin executor
/// spec / append `execution.reserved`").
///
/// The reservation is the record that makes the launch window inspectable. A
/// daemon that dies between this event and `execution.started` leaves behind
/// exactly one durable statement — *this execution id, this native identity,
/// this stage attempt, this executor spec, were committed to; whether the
/// external effect happened is unknown* — which is a question recovery can
/// answer honestly instead of a silence it has to guess at.
///
/// It is deliberately a superset of the launch decision rather than a pointer
/// to one: re-deriving "which harness would this stage have used" from the
/// pinned workflow plus today's routing chain is re-deciding, and a decision
/// re-made after a restart is not the decision that was taken.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionReservation {
    /// Sergeant's id for the execution being reserved (ULID).
    pub execution_id: String,
    /// Backend that will own it.
    pub backend: String,
    /// The native identity the adapter reserved before launching, when it can
    /// name one (the Claude adapter's session UUID). `None` means the adapter
    /// has nothing to reserve, not that it failed to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_id: Option<String>,
    /// Stage this execution serves.
    pub stage_id: String,
    /// Its index in the pinned workflow.
    pub index: usize,
    /// Attempt number of that stage.
    pub attempt: u32,
    /// The stage's executor kind, as pinned in `workflow.bound` (§12).
    pub stage_kind: String,
    /// The launch profile pinned for this attempt (§14), by name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    /// The model pin carried into this attempt, when there is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl ExecutionReservation {
    /// Whether `execution` is the execution this reservation reserved.
    pub fn was_launched_as(&self, execution: &ExecutionRecord) -> bool {
        execution.execution_id == self.execution_id
    }
}

/// How a restart reconciliation classified an execution (§25's adapter
/// verdicts: still alive / resumable / recoverable / irrecoverable /
/// ambiguous, collapsed to what the M3 contract surface can distinguish).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconcileDisposition {
    /// The backend answered with a definite signal; the run resumed from it.
    Resumed,
    /// The backend could not give a definite answer. Fails closed to blocked.
    Ambiguous,
}
