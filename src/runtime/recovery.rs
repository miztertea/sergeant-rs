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
//! `pending` is the other state this looks at for the work's own status, and
//! only in one shape. Submitting is several appends — `work.submitted`, then
//! the engine's materialization, binding and `work.started` records — so a
//! crash inside that window leaves a `pending` work that already has a run
//! record. Nothing else will ever pick it up (`retry` refuses `pending`, and
//! a retried `command_id` replays the recorded outcome without re-planning),
//! and the surviving prefix may name git state created in the user's
//! repository. That is uncertainty, not a decision, so it fails closed to
//! `blocked` with the evidence. A `pending` work with *no* run record is
//! untouched: it is an intent the daemon never started, which is exactly
//! what it looks like.
//!
//! Terminal work (`completed` / `failed` / `canceled`) is never reconsidered
//! for its own status, but its surface teardown is swept if the completion
//! tail's crash window (issue #9) left it unfinished: a run that records a
//! surface but no `surface.torn_down` gets teardown re-run and the missing
//! event appended, evidence from disk rather than a guess. This never
//! changes a work's state and never blocks startup — see
//! [`reconcile_terminal_surface`](crate::runtime::engine::Engine::reconcile_terminal_surface).

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
    /// Terminal works whose surface teardown a crash left unrecorded, and
    /// which this restart finished and journaled. Empty on a clean restart.
    #[serde(default)]
    pub surfaces_retired: Vec<String>,
    /// Terminal works whose *reservation* a crash left outstanding, closed by
    /// this restart (§22.5's cancel-during-launch window).
    #[serde(default)]
    pub reservations_retired: Vec<String>,
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

    // The completion tail's crash window (L6, issue #9). `work.completed` /
    // `work.failed` / `work.canceled` and the `surface.torn_down` that follows
    // are adjacent appends; a kill between them leaves a terminal work whose
    // surface the journal still records as live. Until now nothing looked at
    // terminal work again, so the missing event stayed missing forever and —
    // when the crash landed before `teardown()` itself ran — a worktree and a
    // surface root stayed on disk with no owner. These are the works whose
    // teardown this restart finishes and records.
    //
    // **Scope, stated so the issue can be closed honestly.** This is a
    // forward fix, and only that. It is keyed on the *missing event*, so a
    // work whose `surface.torn_down` is already recorded is never looked at
    // again — including the ones torn down before `teardown()` learned to
    // remove the empty root (issue #5), whose `surfaces/<work-id>/` are still
    // on disk in any data dir that predates it. Reclaiming those means
    // deleting directories no journal entry points at, which is a garbage
    // collector's job and not a recovery pass's: recovery acts on evidence
    // that something was left unfinished, and there is none here. Tracked as
    // issue #17.
    let stranded_surfaces: Vec<String> = core
        .registry
        .state()
        .works
        .values()
        .filter(|work| {
            matches!(
                work.state,
                WorkState::Completed | WorkState::Failed | WorkState::Canceled
            )
        })
        .filter(|work| {
            core.registry
                .state()
                .runs
                .get(&work.id)
                .is_some_and(|run| run.surface.is_some() && run.teardown.is_none())
        })
        .map(|work| work.id.clone())
        .collect();

    // §22.5's cancel-during-launch window. `begin_retire_run` closes an
    // outstanding reservation when a work goes terminal, but a daemon killed
    // between the cancel landing and `settle_launch` running never got there —
    // and nothing else would ever look, because reconciliation reads only
    // `active` work, `pending`-but-started work, and terminal work missing a
    // teardown (which a cancel has already written). The result was a
    // permanently unsettled reservation, naming a possibly-live native context
    // in a worktree cancel had just removed, that `work show` reported as open
    // forever. These are the works whose reservation this restart closes.
    let stranded_reservations: Vec<String> = core
        .registry
        .state()
        .works
        .values()
        .filter(|work| {
            matches!(
                work.state,
                WorkState::Completed | WorkState::Failed | WorkState::Canceled
            )
        })
        .filter(|work| {
            core.registry
                .state()
                .runs
                .get(&work.id)
                .is_some_and(|run| run.unsettled_reservation().is_some())
        })
        .map(|work| work.id.clone())
        .collect();

    let mut report = ReconcileReport::default();
    for work_id in in_flight {
        let outcome = engine.reconcile_work(core, &work_id);
        flush_after_reconciling(core, &work_id);
        match outcome {
            Ok(ReconcileDisposition::Resumed) => report.resumed.push(work_id),
            Ok(ReconcileDisposition::Ambiguous) => report.blocked.push(work_id),
            Err(e) => {
                block_on_reconcile_error(engine, core, &work_id, &e);
                // The block above is itself a commit; flush it too rather
                // than leaving it for the next work's flush to cover.
                flush_after_reconciling(core, &work_id);
                report.blocked.push(work_id);
            }
        }
    }
    for work_id in crashed_starts {
        if let Err(e) = engine.reconcile_crashed_start(core, &work_id) {
            block_on_reconcile_error(engine, core, &work_id, &e);
        }
        flush_after_reconciling(core, &work_id);
        report.blocked.push(work_id);
    }
    // Isolated like every other work's reconciliation, and for a smaller
    // stake: a terminal work's audit trail is worth finishing, never worth
    // refusing to start the daemon over. A failure here (a wedged journal, a
    // git that will not answer) leaves the work exactly as it was — terminal,
    // with the teardown still unrecorded — so the next restart tries again.
    // Nothing is *blocked* on it: the work already reached its conclusion, and
    // rewriting that conclusion because its scaffolding could not be swept
    // would be the daemon inventing state the journal never recorded.
    // Ordered before the surface sweep on purpose: closing the record of a
    // possibly-live external identity is the more urgent of the two, and a
    // teardown that fails must not stop it happening.
    for work_id in stranded_reservations {
        let outcome = engine.reconcile_terminal_reservation(core, &work_id);
        flush_after_reconciling(core, &work_id);
        match outcome {
            Ok(true) => report.reservations_retired.push(work_id),
            Ok(false) => {}
            Err(e) => tracing::warn!(
                work_id,
                error = %e,
                "could not close a reservation a crash left open on a terminal work"
            ),
        }
    }
    for work_id in stranded_surfaces {
        let outcome = engine.reconcile_terminal_surface(core, &work_id);
        flush_after_reconciling(core, &work_id);
        match outcome {
            Ok(true) => report.surfaces_retired.push(work_id),
            Ok(false) => {}
            Err(e) => tracing::warn!(
                work_id,
                error = %e,
                "could not finish the surface teardown a crash interrupted"
            ),
        }
    }
    Ok(report)
}

/// Close the open group after one work's reconciliation, best-effort
/// (invariants round 2, INV-R2-01).
///
/// Startup runs with a bare [`Core`] — no [`crate::api::CoreGuard`], because
/// nothing else can see it yet — so nothing durable happens on its own
/// between commits. Each of the loops above performs unbounded, irreversible
/// external effects per work (`pending.perform()`'s `git worktree remove`,
/// [`Engine::run_inline`]'s relaunch) interleaved with the commits that
/// describe them; without a flush after every work, a crash mid-reconcile
/// could leave those effects undone *and* unrecorded, letting a later
/// restart repeat them. Flushing per work, rather than once at the end,
/// keeps each work's crash window no wider than a single reconciliation —
/// exactly the width every other path in this daemon already gets from
/// `CoreGuard`.
///
/// Best-effort and non-propagating like the rest of this per-work loop
/// (§25's contract: one work's trouble must not stop the others). A flush
/// failure means the journal has poisoned itself ([`crate::runtime::journal::Journal::sync`]),
/// so every following commit — for this work or the next — will fail the
/// same way and surface through its own error path; there is nothing this
/// call can do beyond reporting it.
fn flush_after_reconciling(core: &mut Core, work_id: &str) {
    if let Err(e) = core.flush() {
        tracing::error!(
            work_id,
            error = %e,
            "failed to flush the journal after reconciling this work at startup; \
             the journal is likely poisoned and further recovery is unreliable"
        );
    }
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

    /// INV-R2-01. `reconcile` runs with a bare [`Core`] — no [`crate::api::CoreGuard`]
    /// — so nothing outside `reconcile` itself can make its commits durable.
    /// This pins that `reconcile` does not rely on a caller to flush: after
    /// it returns, the open group is empty and at least one fsync actually
    /// ran, for both the work that reconciled cleanly (an ambiguous
    /// classification, `intact`) and the one whose own reconciliation
    /// errored (`broken`, which takes the `block_on_reconcile_error` path).
    /// Reverting the per-work `flush_after_reconciling` calls in `reconcile`
    /// leaves the group open and fails this test without touching anything
    /// else — every other assertion in this file still holds, because an
    /// unflushed group changes nothing about what got folded into the
    /// registry, only what is durable and published.
    #[test]
    fn reconcile_flushes_after_every_work_so_nothing_it_touches_is_left_unsynced() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());
        let engine = Engine::new(Arc::new(BackendRegistry::new()), None, dir.path());

        let broken = "01AAAAAAAAAAAAAAAAAAAAAAAA";
        let intact = "01BBBBBBBBBBBBBBBBBBBBBBBB";

        // `broken`: active with no run record — `reconcile_work` errors and
        // takes the `block_on_reconcile_error` path.
        testing::submit(&mut core, broken, "active but runless");
        testing::commit(&mut core, broken, KIND_WORK_STARTED, json!({}));

        // `intact`: an ordinary in-flight work whose backend is gone —
        // reconciles to `Ambiguous` without erroring.
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

        // `testing::commit` (used above to build the fixture) never flushes
        // — it is `Core::commit` alone, exactly like the daemon's own bare
        // `core.commit(...)` calls before `reconcile` runs. So nothing has
        // synced yet; the baseline is captured for a stronger assertion
        // below (that reconcile's flushing is not a no-op it stumbled into).
        let fsyncs_before = core.journal.fsync_count();

        let report = reconcile(&engine, &mut core).expect("recovery must not abort");
        assert_eq!(report.blocked, vec![broken.to_string(), intact.to_string()]);

        assert_eq!(
            core.open_group_len(),
            0,
            "reconcile must leave nothing open behind it — every commit it \
             made, across every work it touched, must already be durable \
             and published by the time it returns"
        );
        assert!(
            core.journal.fsync_count() > fsyncs_before,
            "reconcile must have actually synced, not merely left the group \
             empty by coincidence"
        );
    }

    /// A temp git repository with one commit, run with a fixture identity so
    /// nothing depends on the machine's own git config.
    fn repo(path: &std::path::Path) -> crate::domain::estate::RepositorySpec {
        std::fs::create_dir_all(path).expect("repo dir");
        for args in [
            vec!["init", "-b", "main"],
            vec!["commit", "--allow-empty", "-m", "initial"],
        ] {
            let output = std::process::Command::new("git")
                .args(&args)
                .current_dir(path)
                .env("GIT_AUTHOR_NAME", "t")
                .env("GIT_AUTHOR_EMAIL", "t@example.invalid")
                .env("GIT_COMMITTER_NAME", "t")
                .env("GIT_COMMITTER_EMAIL", "t@example.invalid")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_CONFIG_SYSTEM", "/dev/null")
                .output()
                .expect("git");
            assert!(output.status.success(), "git {args:?}: {output:?}");
        }
        crate::domain::estate::RepositorySpec {
            name: "solo".to_string(),
            path: path.to_path_buf(),
        }
    }

    /// §11 through the completion tail's crash window (issue #9): a teardown
    /// a crash swallowed is re-run at startup and produces the *same*
    /// integrity assessment it would have produced at retirement, marked
    /// `recovered`.
    ///
    /// This is the property that makes the assessment trustworthy at all.
    /// `reconcile_terminal_surface` reaches the identical `settle_surface`
    /// arm every live terminal path does, so there is one computation point
    /// and not two — a second implementation here is precisely how a
    /// recovered Work would come to be reported clean while the same disk
    /// state assessed at retirement read dirty.
    #[test]
    fn a_crash_recovered_teardown_records_the_same_integrity_assessment() {
        use crate::domain::work::KIND_WORK_COMPLETED;
        use crate::runtime::integrity::IntegrityDisposition;
        use crate::runtime::surface::{
            KIND_SURFACE_MATERIALIZED, KIND_SURFACE_TORN_DOWN, materialize,
        };

        let dir = tempfile::TempDir::new().expect("tempdir");
        let data = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(data.path());
        let engine = Engine::new(Arc::new(BackendRegistry::new()), None, data.path());

        let work_id = "01RECOVEREDTEARDOWN00";
        let spec = repo(&dir.path().join("solo"));
        let surface = materialize(
            data.path(),
            data.path(),
            work_id,
            std::slice::from_ref(&spec),
        )
        .expect("materialize");
        let worktree = surface.bindings[0].worktree_path.clone();

        testing::submit(&mut core, work_id, "crashed before its teardown landed");
        testing::commit(
            &mut core,
            work_id,
            KIND_SURFACE_MATERIALIZED,
            json!({"surface": surface}),
        );
        testing::commit(&mut core, work_id, KIND_WORK_COMPLETED, json!({}));

        // The run ended on another branch, and the crash landed before
        // teardown could say so.
        let output = std::process::Command::new("git")
            .args(["checkout", "-b", "renegade"])
            .current_dir(&worktree)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .output()
            .expect("git");
        assert!(output.status.success(), "checkout: {output:?}");

        let report = reconcile(&engine, &mut core).expect("recovery");
        assert_eq!(report.surfaces_retired, vec![work_id.to_string()]);

        let torn = events(&core, work_id, KIND_SURFACE_TORN_DOWN);
        assert_eq!(torn.len(), 1, "exactly one teardown is recorded");
        assert_eq!(
            torn[0]["recovered"], true,
            "the trail shows when the teardown was recorded, not that it \
             landed with the completion: {}",
            torn[0]
        );
        assert_eq!(
            torn[0]["integrity"], "dirty",
            "a recovered teardown carries the same assessment a live one \
             would have: {}",
            torn[0]
        );
        assert_eq!(
            torn[0]["report"]["bindings"][0]["findings"][0]["finding"], "assigned_branch_mismatch",
            "{}",
            torn[0]
        );
        assert_eq!(
            core.registry
                .state()
                .run_view(work_id)
                .expect("run")
                .integrity,
            Some(IntegrityDisposition::Dirty),
            "and the projection folds it the same way the live path's does"
        );
        assert_eq!(
            core.registry.state().works[work_id].state,
            WorkState::Completed,
            "integrity never moves Work state (§11.5)"
        );
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
