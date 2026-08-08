//! Runtime: journal, projections, git, surfaces, routing, engine, recovery.

pub mod blob;
pub mod engine;
pub(crate) mod fsutil;
pub mod git;
pub mod journal;
pub mod projection;
pub mod recovery;
pub mod router;
pub mod surface;

/// Shared fixtures for the runtime's own unit tests.
///
/// The engine and recovery both act on a [`Core`](crate::api::Core) — journal
/// plus folded registry — and both need one built over a scratch directory
/// with a few facts already in it. One builder, so a test that needs a
/// differently-shaped journal writes the events, not the plumbing.
#[cfg(test)]
pub(crate) mod testing {
    use serde_json::{Value, json};

    use crate::api::Core;
    use crate::domain::event::{EventDraft, EventSource};
    use crate::domain::work::KIND_WORK_SUBMITTED;
    use crate::runtime::journal::Journal;
    use crate::runtime::projection::work_registry_projection;

    /// An empty `Core` over `data_dir`.
    pub(crate) fn core(data_dir: &std::path::Path) -> Core {
        let journal = Journal::open(data_dir).expect("journal");
        let mut registry = work_registry_projection();
        registry
            .catch_up(journal.replay().expect("replay"))
            .expect("catch up");
        let (events_tx, _rx) = tokio::sync::broadcast::channel(16);
        Core {
            journal,
            registry,
            events_tx,
        }
    }

    /// Append one event for `work_id`, as the daemon's engine would.
    pub(crate) fn commit(core: &mut Core, work_id: &str, kind: &str, payload: Value) {
        core.commit(
            EventDraft::new(EventSource::new("daemon", "test"), kind, payload)
                .with_work_id(work_id),
        )
        .expect("commit");
    }

    /// Record a submitted (`pending`) work.
    pub(crate) fn submit(core: &mut Core, work_id: &str, intent: &str) {
        commit(
            core,
            work_id,
            KIND_WORK_SUBMITTED,
            json!({"work": {
                "id": work_id,
                "intent": intent,
                "state": "pending",
                "created_by": "test",
                "created_at": "2026-01-01T00:00:00Z",
            }}),
        );
    }
}
