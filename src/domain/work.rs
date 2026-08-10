//! Work: one durable unit of intent accepted by the daemon (proposal §10).
//!
//! The state set is the full §10 set. M2 could only reach
//! `pending → canceled`, because nothing executed; M3's workflow engine makes
//! the rest of the machine reachable — but only through the same door.
//! Transitions happen only via journal events: a handler or the engine
//! validates the transition against the projected state, appends a `work.*`
//! event, and the registry reducer
//! (`runtime::projection::work_registry_reducer`) folds it back into state.
//!
//! Two §10 properties are load-bearing here and are asserted by tests rather
//! than trusted:
//!
//! - **Workflow stage is orthogonal.** No stage name is ever a `WorkState`,
//!   and the current stage lives in a separate coordinate
//!   (`runtime::projection::WorkRun`).
//! - **Work state is not process state (§25).** Nothing in this module or the
//!   engine derives a `WorkState` from a backend's liveness; every transition
//!   below comes from an explicit signal or an explicit operator command.

use serde::{Deserialize, Serialize};

/// Event kind: a work item was accepted and entered `pending`.
pub const KIND_WORK_SUBMITTED: &str = "work.submitted";
/// Event kind: a work item was canceled.
pub const KIND_WORK_CANCELED: &str = "work.canceled";
/// Event kind: execution of a work item began (`pending → active`).
pub const KIND_WORK_STARTED: &str = "work.started";
/// Event kind: a parked work item was resumed (`→ active`) by input or retry.
pub const KIND_WORK_RESUMED: &str = "work.resumed";
/// Event kind: a work item is waiting on an external condition.
pub const KIND_WORK_WAITING: &str = "work.waiting";
/// Event kind: a work item needs a human answer.
pub const KIND_WORK_NEEDS_INPUT: &str = "work.needs_input";
/// Event kind: a work item is blocked (also the fail-closed landing state for
/// ambiguous recovery, §25).
pub const KIND_WORK_BLOCKED: &str = "work.blocked";
/// Event kind: a work item completed successfully.
pub const KIND_WORK_COMPLETED: &str = "work.completed";
/// Event kind: a work item failed.
pub const KIND_WORK_FAILED: &str = "work.failed";
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
    /// M3's reachable set, with the operation that produces each edge:
    ///
    /// ```text
    /// pending    → active     start (submit resolved a surface and a backend)
    ///            → blocked    start failed; fail closed with evidence
    ///            → canceled   cancel
    /// active     → waiting | needs_input | blocked | completed | failed
    ///                         explicit backend signal for the current stage
    ///            → canceled   cancel
    /// waiting    → active     retry (re-enter the stage)
    /// needs_input→ active     input delivered
    /// blocked    → active     retry
    /// failed     → active     retry
    /// waiting | needs_input | blocked | failed → canceled   cancel
    /// waiting | needs_input | blocked → failed | blocked    later signals
    /// completed  → (nothing)  terminal
    /// canceled   → (nothing)  terminal
    /// ```
    ///
    /// `completed` and `canceled` are absorbing: nothing leaves them, so a
    /// terminal work can never be resurrected by a late signal from a backend
    /// that is still running (§25). `failed` is terminal for the run but
    /// retryable by explicit operator action — §12 lists retry as a verb.
    ///
    /// Everything not listed is illegal and must be rejected *before* any
    /// event is appended. Later milestones widen this table; they never
    /// bypass it.
    pub fn can_transition(self, to: WorkState) -> bool {
        use WorkState::*;
        match self {
            Pending => matches!(to, Active | Blocked | Canceled),
            Active => matches!(
                to,
                Waiting | NeedsInput | Blocked | Completed | Failed | Canceled
            ),
            Waiting | NeedsInput => matches!(to, Active | Blocked | Failed | Canceled),
            Blocked => matches!(to, Active | Failed | Canceled),
            Failed => matches!(to, Active | Canceled),
            Completed | Canceled => false,
        }
    }

    /// The state a `work.*` event kind puts the work into, if the kind is a
    /// state transition at all. This is the single mapping the reducer and
    /// the engine share, so an event kind cannot mean one state when written
    /// and another when replayed.
    pub fn for_event_kind(kind: &str) -> Option<WorkState> {
        match kind {
            KIND_WORK_STARTED | KIND_WORK_RESUMED => Some(WorkState::Active),
            KIND_WORK_WAITING => Some(WorkState::Waiting),
            KIND_WORK_NEEDS_INPUT => Some(WorkState::NeedsInput),
            KIND_WORK_BLOCKED => Some(WorkState::Blocked),
            KIND_WORK_COMPLETED => Some(WorkState::Completed),
            KIND_WORK_FAILED => Some(WorkState::Failed),
            KIND_WORK_CANCELED => Some(WorkState::Canceled),
            _ => None,
        }
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
    /// Requested backend, before routing (§13's explicit tier).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    /// Origin client from the submitting request (§13's `origin.client`).
    /// Kept distinct from `backend`, `profile` and `workflow` on purpose:
    /// §13 forbids collapsing them into one overloaded `agent` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_client: Option<String>,
    /// Requested launch profile (§14).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
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

    /// The whole table, pinned edge by edge. M2 pinned "only
    /// `pending → canceled`"; M3's engine makes the rest of §10 reachable, so
    /// this test states the new table exhaustively rather than loosening into
    /// spot checks — an implementation that allowed one extra edge (say
    /// `completed → active`) must still fail here.
    #[test]
    fn the_transition_table_is_exactly_this() {
        use WorkState::*;
        let all = [
            Pending, Active, Waiting, NeedsInput, Blocked, Completed, Failed, Canceled,
        ];
        let legal: &[(WorkState, WorkState)] = &[
            (Pending, Active),
            (Pending, Blocked),
            (Pending, Canceled),
            (Active, Waiting),
            (Active, NeedsInput),
            (Active, Blocked),
            (Active, Completed),
            (Active, Failed),
            (Active, Canceled),
            (Waiting, Active),
            (Waiting, Blocked),
            (Waiting, Failed),
            (Waiting, Canceled),
            (NeedsInput, Active),
            (NeedsInput, Blocked),
            (NeedsInput, Failed),
            (NeedsInput, Canceled),
            (Blocked, Active),
            (Blocked, Failed),
            (Blocked, Canceled),
            (Failed, Active),
            (Failed, Canceled),
        ];
        for from in all {
            for to in all {
                let expected = legal.contains(&(from, to));
                assert_eq!(
                    from.can_transition(to),
                    expected,
                    "transition {from} -> {to} must be {}",
                    if expected { "legal" } else { "illegal" }
                );
            }
        }
    }

    /// Terminal means terminal: no signal, however late, revives a work that
    /// completed or was canceled (§25 — a backend still running its context
    /// must not be able to un-cancel work).
    #[test]
    fn completed_and_canceled_are_absorbing() {
        use WorkState::*;
        for terminal in [Completed, Canceled] {
            for to in [
                Pending, Active, Waiting, NeedsInput, Blocked, Completed, Failed, Canceled,
            ] {
                assert!(
                    !terminal.can_transition(to),
                    "{terminal} must absorb, but {terminal} -> {to} was allowed"
                );
            }
        }
    }

    /// Every `work.*` transition kind maps to exactly one state, and nothing
    /// else does. The reducer and the engine share this mapping, so a kind
    /// cannot mean one state when written and another when replayed.
    #[test]
    fn event_kinds_map_to_states_one_to_one() {
        use WorkState::*;
        for (kind, state) in [
            (KIND_WORK_STARTED, Active),
            (KIND_WORK_RESUMED, Active),
            (KIND_WORK_WAITING, Waiting),
            (KIND_WORK_NEEDS_INPUT, NeedsInput),
            (KIND_WORK_BLOCKED, Blocked),
            (KIND_WORK_COMPLETED, Completed),
            (KIND_WORK_FAILED, Failed),
            (KIND_WORK_CANCELED, Canceled),
        ] {
            assert_eq!(WorkState::for_event_kind(kind), Some(state), "kind {kind}");
        }
        for kind in [
            KIND_WORK_SUBMITTED,
            KIND_COMMAND_ACCEPTED,
            "stage.completed",
            "execution.started",
        ] {
            assert_eq!(
                WorkState::for_event_kind(kind),
                None,
                "{kind} is not a work-state transition"
            );
        }
    }

    /// `Display` is `as_str` for every variant, not just the ones that
    /// happen to appear inside another assertion's failure message
    /// elsewhere in this file (which only formats on a failing path, so
    /// those calls execute nothing in a green suite). `Pending` and
    /// `Waiting` in particular are never the state on either side of a
    /// failing `can_transition` assertion, so nothing else in this module
    /// exercises their `as_str`/`Display` arms.
    #[test]
    fn display_is_the_canonical_name_for_every_state() {
        use WorkState::*;
        for (state, expected) in [
            (Pending, "pending"),
            (Active, "active"),
            (Waiting, "waiting"),
            (NeedsInput, "needs_input"),
            (Blocked, "blocked"),
            (Completed, "completed"),
            (Failed, "failed"),
            (Canceled, "canceled"),
        ] {
            assert_eq!(state.as_str(), expected);
            assert_eq!(state.to_string(), expected);
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
