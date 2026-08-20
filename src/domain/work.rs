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

/// R-MVP1-6's structured-intent schema slot: five optional fields, all of
/// them progressive elaboration of the free-text `intent` a Work already
/// carries — never a replacement for it. Any subset may be present; a client
/// that sends none of them submits exactly as it did before this existed.
///
/// **Additive-only, already safe.** `Work` derives no `deny_unknown_fields`
/// and event payloads round-trip through `serde_json::Value` losslessly, so
/// a field this type gains tomorrow deserializes as absent from every event
/// already journaled, and a binary built before that field existed drops
/// nothing when it re-serializes a newer one. The discipline this type must
/// keep going forward: new fields are `Option`, nothing here is ever removed
/// or retyped.
///
/// **Not reserved (U-R5): no dedup identity, no promotion-provenance field.**
/// Those are the corpus's highest-correction-cost item — seven upstream
/// collision issues — and belong to the post-MVP backlog type that decides
/// them with data, not to a slot added here on spec.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentDetail {
    /// What this Work is trying to achieve, in the elaborator's own words —
    /// distinct from `intent` itself, which stays the primary, required
    /// free text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objective: Option<String>,
    /// Repositories the elaboration names. Purely descriptive data here —
    /// nothing in M-series routes on it — and it is checked for agreement
    /// against the submission's actual `--repo` flags (`Work::repositories`)
    /// at submit time: a `Work` cannot carry two conflicting answers to
    /// "which repositories is this about" (§13's one-source-of-truth rule).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repos: Option<Vec<String>>,
    /// A named `[group.<name>]` this elaboration targets, in place of an
    /// explicit repository list — R-MVP1-5(b)'s group-expansion slot. No
    /// engine surface reads this in MVP-1 (groups are MVP-3's CLI-side
    /// expansion); it is journaled and displayed like the other four.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// What "done" looks like for this Work.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance: Option<String>,
    /// What is explicitly out of scope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclusions: Option<String>,
    /// The workflow the elaboration names, checked for agreement against
    /// the submission's actual `--workflow` flag (`Work::workflow`) at
    /// submit time, the same way `repos` is checked against `repositories`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
}

impl IntentDetail {
    /// Whether every field is absent — a client that sent an empty object,
    /// which is the same fact as sending nothing at all.
    pub fn is_empty(&self) -> bool {
        self.objective.is_none()
            && self.repos.is_none()
            && self.group.is_none()
            && self.acceptance.is_none()
            && self.exclusions.is_none()
            && self.workflow.is_none()
    }
}

/// MVP-3's submit-time envelope override (checkpoint-friction item, the
/// bucketing doc's "turn-cap/ceiling settable at submission"): per-Work
/// overrides of R-MVP1-7's turn cap and per-turn wall-clock ceiling, both of
/// which were daemon-wide-only defaults before this (`Engine::turn_cap`/
/// `turn_ceiling`, `SGT_TURN_CAP`/config, contract Unknown #2: "mechanism
/// contracted, values measured at build time ... not yet a per-Work or
/// submit-time surface"). This is that surface, plumbed as CLI/API onto the
/// existing envelope mechanics — R-MVP1-10's `extend_turn_envelope` already
/// proved a Work-specific cap is legal engine state; this is the same idea
/// at submission time instead of after a block.
///
/// **Additive-only, same discipline as [`IntentDetail`]**: `Work` derives no
/// `deny_unknown_fields`, so an event journaled before this field existed
/// deserializes it as `None`, which is exactly the fact it recorded — no
/// override, the daemon-wide default applied. Absent is not zero: a `None`
/// `turn_cap` means "use `Engine::turn_cap`", never "cap at 0".
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvelopeRequest {
    /// Per-Work turn cap, overriding `Engine::turn_cap` for this Work's
    /// whole life. Still additive with R-MVP1-10's `turn_cap_bonus`: a
    /// `retry`-time `sgt extend` raises *this* base, not the daemon default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_cap: Option<u32>,
    /// Per-Work wall-clock ceiling in whole seconds, overriding
    /// `Engine::turn_ceiling` for this Work's whole life. Seconds (not a
    /// `Duration`) because that is what journals losslessly through
    /// `serde_json::Value` and what a CLI flag naturally is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ceiling_secs: Option<u64>,
}

impl EnvelopeRequest {
    /// Whether both fields are absent — the same fact as sending nothing,
    /// mirroring [`IntentDetail::is_empty`].
    pub fn is_empty(&self) -> bool {
        self.turn_cap.is_none() && self.ceiling_secs.is_none()
    }
}

/// estate-root proposal §7.3/§13.3's structured scope request: exactly what
/// the client asked for (`--repo`/`--group`/`--all`, as submitted), kept
/// distinct from [`Work::repositories`] below — the *resolved* list a
/// core-owned lookup against the bound estate's `groups` produced from this
/// request (`runtime::engine::Engine::resolve_scope`). §7.3's whole point is
/// that both survive: a later manifest edit (a group's membership changes, a
/// repository is renamed) cannot retroactively rewrite what an already
/// journaled Work was scoped to, because the resolved list is pinned here
/// too, permanently, beside the request that produced it.
///
/// `#[serde(default)]` on [`Work::scope_request`] is what lets every Work
/// journaled before this field existed replay: it has no `scope_request` key
/// at all, and defaults to this type's all-empty [`Default`] — the honest
/// reading for a Work that predates the concept of a structured scope
/// request.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeRequest {
    /// Explicit `--repo`/`scope.repos` names, as submitted (repeatable,
    /// declaration order, not yet deduplicated against `group`'s members —
    /// that union happens during resolution, not here).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repos: Vec<String>,
    /// `--group`/`scope.group`, as submitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// `--all`/`scope.all`: an explicit whole-estate selection (§7.1's third
    /// scope form). Journaled as the request form even though it resolves to
    /// the same repository list a future `--repo` naming every repository
    /// would — §7.3's "a later manifest edit cannot rewrite the meaning of
    /// an existing Work" needs to know the client asked for *everything*,
    /// not for the specific repositories the estate happened to declare that
    /// day.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub all: bool,
}

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

/// A Work record per §10: id, intent, targeted repositories (`scope_request`
/// and its resolution, §7.3), workflow, state, created_by, created_at.
/// `workflow` and `backend` are recorded for M3/M4 but not executed in M2.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Work {
    /// ULID identifying this work item.
    pub id: String,
    /// **Deprecated, never written (estate-root Phase C, §7.4).** Before
    /// Phase C this carried a free-form client-supplied estate label (or,
    /// later, the discovered estate's name as a submit-time fallback). The
    /// daemon is bound to exactly one estate now, so the field has no role
    /// left to play in submission or Work identity — every Work journaled
    /// from Phase C onward leaves this `None`. It stays in the struct, still
    /// `#[serde(default)]`, purely so a pre-Phase-C `work.submitted` event
    /// (the live dogfood estate journals 150+ of them) still deserializes;
    /// nothing new should ever read or write it. [`Work::scope_request`] and
    /// [`Work::repositories`] are its replacement (§7.3).
    ///
    /// **The name stays `workspace` (§13.2).** The Workspace-to-Estate
    /// rename moves the domain vocabulary; it does not rewrite history. This
    /// field exists only to deserialize a key already written into durable
    /// journals, so renaming it would silently stop reading exactly the
    /// events it was kept for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    /// The human intent this work exists to satisfy.
    pub intent: String,
    /// The *resolved* repositories this Work targets — `scope_request`
    /// (below), as core-owned resolution against the bound estate's
    /// `groups`/declared repositories actually expanded it. `Engine::
    /// resolve_scope` computes this once, at submit; a later manifest edit
    /// (a group's membership changes) cannot retroactively change what an
    /// already-journaled Work meant (§7.3).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repositories: Vec<String>,
    /// estate-root proposal §7.3/§13.3's request form: exactly what the
    /// client asked for (`--repo`/`--group`/`--all`), before resolution.
    /// `#[serde(default)]`: absent on every Work journaled before this field
    /// existed, which replays as [`ScopeRequest::default`] — the honest
    /// reading for a Work that predates the concept.
    #[serde(default)]
    pub scope_request: ScopeRequest,
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
    /// R-MVP1-6's structured-intent schema slot: progressive elaboration of
    /// `intent`, never a replacement for it. `None` (the default for every
    /// event journaled before this field existed) and `Some` carrying every
    /// field absent are the same fact — a client that elaborated nothing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent_detail: Option<IntentDetail>,
    /// MVP-3's per-submission envelope override (see [`EnvelopeRequest`]).
    /// `None` is the ordinary case — the daemon-wide `Engine::turn_cap`/
    /// `turn_ceiling` defaults apply, unchanged from before this field
    /// existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub envelope: Option<EnvelopeRequest>,
    /// estate-root §8.3: whether the operator typed
    /// `--override-git-preflight` for *this* submission.
    ///
    /// Journaled on the Work itself, not only on the surface plan that
    /// records what it waived, because the two are different facts and §8.3
    /// asks for both: an override that turned out to waive nothing is still
    /// an operator authorization that was given, and `work.submitted` is
    /// where the submission's own request form lives (beside
    /// [`Self::scope_request`], for the same §7.3-shaped reason).
    ///
    /// `#[serde(default)]`, skipped when false, on [`ScopeRequest::all`]'s
    /// precedent: absent means not overridden, which is what every Work
    /// journaled before Phase E recorded.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub git_preflight_override: bool,
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
