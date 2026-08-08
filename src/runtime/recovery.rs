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
        match engine.reconcile_work(core, &work_id)? {
            ReconcileDisposition::Resumed => report.resumed.push(work_id),
            ReconcileDisposition::Ambiguous => report.blocked.push(work_id),
        }
    }
    for work_id in crashed_starts {
        engine.reconcile_crashed_start(core, &work_id)?;
        report.blocked.push(work_id);
    }
    Ok(report)
}
