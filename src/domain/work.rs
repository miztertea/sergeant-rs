//! Work: one durable unit of intent accepted by the daemon (proposal §10).
//!
//! The state set is the full §10 set, but in M2 only the transitions that
//! exist without a workflow engine are reachable: submit creates a work in
//! `pending`, cancel moves `pending → canceled`. Every other transition is
//! illegal and rejected at command time (fail closed); later milestones add
//! legal ones by extending [`WorkState::can_transition`].
//!
//! Transitions happen only via journal events: API handlers validate against
//! the projected state, append `work.*` events, and the registry reducer
//! (`runtime::projection::work_registry_reducer`) folds them back into state.

use serde::{Deserialize, Serialize};

/// Event kind: a work item was accepted and entered `pending`.
pub const KIND_WORK_SUBMITTED: &str = "work.submitted";
/// Event kind: a work item was canceled.
pub const KIND_WORK_CANCELED: &str = "work.canceled";
/// Event kind: a mutation command was accepted; payload records its result.
pub const KIND_COMMAND_ACCEPTED: &str = "command.accepted";
/// Event kind: a mutation command was rejected; payload records the error.
pub const KIND_COMMAND_REJECTED: &str = "command.rejected";

/// The §10 work states. Workflow stage is orthogonal and deliberately not a
/// state here.
// No `PartialOrd`/`Ord`: states are a set with a transition table, not a
// scale, and a derived ordering would silently bless declaration order as
// meaning something.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkState {
    /// Accepted, not yet picked up by an execution.
    Pending,
    /// An execution is running against it.
    Active,
    /// Parked awaiting an external condition.
    Waiting,
    /// Blocked on a human response.
    NeedsInput,
    /// Blocked on a dependency or policy gate.
    Blocked,
    /// Finished successfully. Terminal.
    Completed,
    /// Finished unsuccessfully. Terminal.
    Failed,
    /// Canceled by a client. Terminal.
    Canceled,
}

impl WorkState {
    /// Whether `self → to` is a legal transition in this milestone.
    ///
    /// M2 reachable set: `pending → canceled` only (submit creates `pending`
    /// directly; it is a creation, not a transition). Everything else —
    /// including any transition out of a terminal state such as
    /// `canceled → pending` — is illegal and must be rejected before any
    /// event is appended. Later milestones widen this table; they never
    /// bypass it.
    pub fn can_transition(self, to: WorkState) -> bool {
        matches!((self, to), (WorkState::Pending, WorkState::Canceled))
    }

    /// The state's canonical snake_case name (matches the serde form).
    pub fn as_str(self) -> &'static str {
        match self {
            WorkState::Pending => "pending",
            WorkState::Active => "active",
            WorkState::Waiting => "waiting",
            WorkState::NeedsInput => "needs_input",
            WorkState::Blocked => "blocked",
            WorkState::Completed => "completed",
            WorkState::Failed => "failed",
            WorkState::Canceled => "canceled",
        }
    }
}

impl std::fmt::Display for WorkState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A Work record per §10: id, workspace, intent, targeted repositories,
/// workflow, state, created_by, created_at. `workflow` and `backend` are
/// recorded for M3/M4 but not executed in M2.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Work {
    /// ULID identifying this work item.
    pub id: String,
    /// Workspace this work belongs to, when the client scoped it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    /// The human intent this work exists to satisfy.
    pub intent: String,
    /// Targeted repositories (recorded; nothing acts on them until M3).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repositories: Vec<String>,
    /// Requested workflow (recorded for M3; not executed in M2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
    /// Requested backend (recorded for M4; not executed in M2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    /// Current state (only mutated by folding journal events).
    pub state: WorkState,
    /// Who submitted it.
    pub created_by: String,
    /// RFC3339 UTC creation time (recorded in the `work.submitted` payload,
    /// so replay reconstructs it identically).
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_pending_to_canceled_is_legal_in_m2() {
        use WorkState::*;
        let all = [
            Pending, Active, Waiting, NeedsInput, Blocked, Completed, Failed, Canceled,
        ];
        for from in all {
            for to in all {
                let legal = from == Pending && to == Canceled;
                assert_eq!(
                    from.can_transition(to),
                    legal,
                    "transition {from} -> {to} must be {}",
                    if legal { "legal" } else { "illegal" }
                );
            }
        }
    }

    #[test]
    fn state_serde_is_snake_case() {
        assert_eq!(
            serde_json::to_string(&WorkState::NeedsInput).unwrap(),
            "\"needs_input\""
        );
        assert_eq!(
            serde_json::from_str::<WorkState>("\"canceled\"").unwrap(),
            WorkState::Canceled
        );
    }
}
