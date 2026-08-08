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

/// Event kind: an execution was started for a stage.
pub const KIND_EXECUTION_STARTED: &str = "execution.started";
/// Event kind: an execution was asked to retire.
pub const KIND_EXECUTION_STOPPED: &str = "execution.stopped";
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
