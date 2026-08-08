//! Startup recovery (proposal §25).
//!
//! §25's daemon-restart sequence is:
//!
//! ```text
//! replay trajectory
//!         ↓
//! discover executions believed active
//!         ↓
//! ask matching adapter for native identity
//!         ↓
//! reattach / resume / classify
//! ```
//!
//! and its constraint is "no new worker is created until prior ownership is
//! reconciled". That is why this runs between journal replay and the first
//! served request: a work believed active is either resumed from what its
//! adapter now reports, or lands in `blocked` with the evidence. Nothing is
//! restarted speculatively, and nothing is assumed dead.
//!
//! Only `active` work is in flight. A work parked in `waiting`, `needs_input`
//! or `blocked` is exactly where its last explicit signal left it — those are
//! decisions, not uncertainty, and re-deciding them at every restart would be
//! the daemon inventing state the journal never recorded.
//!
//! `pending` is the one other state this looks at, and only in one shape.
//! Submitting is several appends — `work.submitted`, then the engine's
//! materialization, binding and `work.started` records — so a crash inside
//! that window leaves a `pending` work that already has a run record. Nothing
//! else will ever pick it up (`retry` refuses `pending`, and a retried
//! `command_id` replays the recorded outcome without re-planning), and the
//! surviving prefix may name git state created in the user's repository. That
//! is uncertainty, not a decision, so it fails closed to `blocked` with the
//! evidence. A `pending` work with *no* run record is untouched: it is an
//! intent the daemon never started, which is exactly what it looks like.

use serde::{Deserialize, Serialize};

use crate::api::Core;
use crate::domain::execution::ReconcileDisposition;
use crate::domain::work::WorkState;
use crate::runtime::engine::{Engine, EngineError};

/// What one restart's reconciliation did.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconcileReport {
    /// Works that were in flight and resumed from unambiguous evidence.
    pub resumed: Vec<String>,
    /// Works whose executions could not be classified; now `blocked`.
    pub blocked: Vec<String>,
}

/// Reconcile every in-flight work against its backend (§25).
///
/// Each work's reconciliation is isolated: an `EngineError` from one work
/// (e.g. a journal I/O failure appending its `execution.reconciled`) fails
/// *that work* closed to `blocked` with the error as evidence, but does not
/// stop the works after it from reconciling, and does not stop the daemon
/// from starting. §25's fail-closed contract is stated per-work — "ambiguous
/// states fail closed to blocked with evidence" — not "one bad journal entry
/// makes the whole daemon unavailable".
pub fn reconcile(engine: &Engine, core: &mut Core) -> Result<ReconcileReport, EngineError> {
    let in_flight: Vec<String> = core
        .registry
        .state()
        .works
        .values()
        .filter(|work| work.state == WorkState::Active)
        .map(|work| work.id.clone())
        .collect();

    let crashed_starts: Vec<String> = core
        .registry
        .state()
        .works
        .values()
        .filter(|work| work.state == WorkState::Pending)
        .filter(|work| {
            core.registry
                .state()
                .runs
                .get(&work.id)
                .is_some_and(|run| run.is_started())
        })
        .map(|work| work.id.clone())
        .collect();

    let mut report = ReconcileReport::default();
    for work_id in in_flight {
        match engine.reconcile_work(core, &work_id) {
            Ok(ReconcileDisposition::Resumed) => report.resumed.push(work_id),
            Ok(ReconcileDisposition::Ambiguous) => report.blocked.push(work_id),
            Err(e) => {
                block_on_reconcile_error(engine, core, &work_id, &e);
                report.blocked.push(work_id);
            }
        }
    }
    for work_id in crashed_starts {
        if let Err(e) = engine.reconcile_crashed_start(core, &work_id) {
            block_on_reconcile_error(engine, core, &work_id, &e);
        }
        report.blocked.push(work_id);
    }
    Ok(report)
}

/// Fail one work closed after its own reconciliation errored, without
/// letting that error propagate to the works still waiting to reconcile.
/// Best-effort: if the work's own state cannot be read or written either
/// (the same underlying failure, e.g. a wedged journal), there is nothing
/// further to do for it here — the next restart's reconciliation, or an
/// operator, is the recourse, not aborting every other work's recovery.
fn block_on_reconcile_error(engine: &Engine, core: &mut Core, work_id: &str, error: &EngineError) {
    let _ = engine.block(
        core,
        work_id,
        "reconciliation failed",
        Some(error.to_string()),
    );
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;

    use super::*;
    use crate::backend::BackendRegistry;
    use crate::domain::execution::KIND_EXECUTION_STARTED;
    use crate::domain::work::{KIND_WORK_BLOCKED, KIND_WORK_STARTED};
    use crate::domain::workflow::{
        KIND_STAGE_BLOCKED, KIND_STAGE_ENTERED, KIND_WORKFLOW_BOUND, StageStatus,
    };
    use crate::runtime::testing;

    /// §25's fail-closed rule is stated per work: "ambiguous states fail
    /// closed to `blocked` with evidence". One work whose reconciliation
    /// *errors* — not "is ambiguous", but fails outright — must therefore
    /// land in `blocked` with that error as its evidence and leave the works
    /// behind it in the queue to be reconciled normally. Propagating it
    /// instead would make one bad work stop the daemon from starting at all,
    /// which is a much worse failure than the one being reported.
    #[test]
    fn one_works_reconcile_failure_does_not_stop_the_others() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());
        // No backends at all: whatever survives to OBSERVE is ambiguous.
        let engine = Engine::new(Arc::new(BackendRegistry::new()), None, dir.path());

        // Reconciled in work-id order, so `broken` is reached first.
        let broken = "01AAAAAAAAAAAAAAAAAAAAAAAA";
        let intact = "01BBBBBBBBBBBBBBBBBBBBBBBB";

        // `broken` is active with no run record at all — the engine has
        // nothing to reconcile it against, so `reconcile_work` errors.
        testing::submit(&mut core, broken, "active but runless");
        testing::commit(&mut core, broken, KIND_WORK_STARTED, json!({}));

        // `intact` is an ordinary in-flight work whose backend is gone.
        testing::submit(&mut core, intact, "active with a run");
        testing::commit(
            &mut core,
            intact,
            KIND_WORKFLOW_BOUND,
            json!({
                "workflow": {"name": "tiny", "version": "1", "source": "test",
                             "stages": [{"id": "00-first", "context": "c"}]},
                "backend": "vanished",
            }),
        );
        testing::commit(
            &mut core,
            intact,
            KIND_STAGE_ENTERED,
            json!({"stage_id": "00-first", "index": 0, "attempt": 1}),
        );
        testing::commit(
            &mut core,
            intact,
            KIND_EXECUTION_STARTED,
            json!({"execution": {
                "execution_id": "e1",
                "backend": "vanished",
                "native_id": "n1",
                "stage_id": "00-first",
                "attempt": 1,
                "stop_requested": false,
            }}),
        );
        testing::commit(&mut core, intact, KIND_WORK_STARTED, json!({}));

        let report = reconcile(&engine, &mut core).expect("recovery must not abort");
        assert_eq!(report.resumed, Vec::<String>::new());
        assert_eq!(report.blocked, vec![broken.to_string(), intact.to_string()]);

        let works = &core.registry.state().works;
        assert_eq!(works[broken].state, WorkState::Blocked);
        assert_eq!(
            works[intact].state,
            WorkState::Blocked,
            "the work behind the failing one must still have been reconciled"
        );

        // The failing work's own evidence is the error that failed it.
        let blocked = events(&core, broken, KIND_WORK_BLOCKED);
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0]["reason"], "reconciliation failed");
        assert!(
            blocked[0]["evidence"]
                .as_str()
                .is_some_and(|e| e.contains("no run to act on")),
            "the error itself is the evidence: {}",
            blocked[0]
        );

        // The reconciled-but-ambiguous work carries the decision at *stage*
        // level too: the work is blocked, and so is the stage it was parked
        // in — a work-level block alone leaves the stage reading `active`
        // forever, which is a stage nothing will ever move.
        let stage_blocked = events(&core, intact, KIND_STAGE_BLOCKED);
        assert_eq!(stage_blocked.len(), 1, "the parked stage must be marked");
        assert_eq!(stage_blocked[0]["stage_id"], "00-first");
        assert!(
            stage_blocked[0]["detail"]
                .as_str()
                .is_some_and(|d| d.contains("vanished")),
            "the stage carries the adapter's own evidence: {}",
            stage_blocked[0]
        );
        assert_eq!(
            core.registry.state().runs[intact]
                .current_stage()
                .expect("a stage")
                .status,
            StageStatus::Blocked
        );
    }

    /// An `active` work whose journal records no execution at all is the
    /// other shape of §25 ambiguity: there is nothing to ask the adapter
    /// about, so nothing can be classified. It fails closed with the stage
    /// marked, exactly as an unrecognised execution does.
    #[test]
    fn an_active_work_with_no_execution_fails_closed_with_the_stage_marked() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());
        let engine = Engine::new(Arc::new(BackendRegistry::new()), None, dir.path());
        let work_id = "01NOEXECUTION";

        testing::submit(&mut core, work_id, "entered a stage, started nothing");
        testing::commit(
            &mut core,
            work_id,
            KIND_WORKFLOW_BOUND,
            json!({
                "workflow": {"name": "tiny", "version": "1", "source": "test",
                             "stages": [{"id": "00-first", "context": "c"}]},
                "backend": "fake",
            }),
        );
        testing::commit(
            &mut core,
            work_id,
            KIND_STAGE_ENTERED,
            json!({"stage_id": "00-first", "index": 0, "attempt": 1}),
        );
        testing::commit(&mut core, work_id, KIND_WORK_STARTED, json!({}));

        let report = reconcile(&engine, &mut core).expect("recovery must not abort");
        assert_eq!(report.blocked, vec![work_id.to_string()]);
        assert_eq!(
            core.registry.state().works[work_id].state,
            WorkState::Blocked
        );

        let stage_blocked = events(&core, work_id, KIND_STAGE_BLOCKED);
        assert_eq!(stage_blocked.len(), 1);
        assert_eq!(stage_blocked[0]["stage_id"], "00-first");
        assert_eq!(stage_blocked[0]["detail"], "no execution to reconcile");
        let blocked = events(&core, work_id, KIND_WORK_BLOCKED);
        assert_eq!(blocked[0]["reason"], "no execution to reconcile");
    }

    /// Payloads of one work's events of one kind, in journal order.
    fn events(core: &Core, work_id: &str, kind: &str) -> Vec<serde_json::Value> {
        core.journal
            .replay()
            .expect("replay")
            .map(|e| e.expect("event"))
            .filter(|e| e.kind == kind && e.work_id.as_deref() == Some(work_id))
            .map(|e| e.payload)
            .collect()
    }
}
