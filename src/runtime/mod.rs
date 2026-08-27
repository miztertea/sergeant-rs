//! Runtime: journal, projections, atlas, git, surfaces, integrity, repository
//! locking, git admission preflight, routing, engine, recovery, sweep.

pub mod analytics;
pub mod atlas;
pub mod blob;
pub mod engine;
pub mod estates;
pub(crate) mod fsutil;
pub mod git;
pub mod graph;
pub mod integrity;
pub mod journal;
pub mod preflight;
pub mod projection;
pub mod prune;
pub mod recovery;
pub mod repolock;
pub mod router;
pub mod startup;
pub mod surface;
pub mod sweep;

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
        Core::new(journal, registry, events_tx)
    }

    /// Append one event for `work_id`, as the daemon's engine would.
    pub(crate) fn commit(core: &mut Core, work_id: &str, kind: &str, payload: Value) {
        core.commit(
            EventDraft::new(EventSource::new("daemon", "test"), kind, payload)
                .with_work_id(work_id),
        )
        .expect("commit");
    }

    /// Record a submitted (`pending`) work with no estate coordinate — a
    /// submission that offered no repository context, or a journal line
    /// older than the envelope field.
    pub(crate) fn submit(core: &mut Core, work_id: &str, intent: &str) {
        submit_in(core, work_id, intent, None);
    }

    /// [`submit`], but recording the canonical estate root the submission
    /// resolved against — H1 D1's coordinate, on the envelope where the real
    /// `work.submitted` draft puts it (never in the payload).
    pub(crate) fn submit_in(
        core: &mut Core,
        work_id: &str,
        intent: &str,
        estate_root: Option<&str>,
    ) {
        let mut draft = EventDraft::new(
            EventSource::new("daemon", "test"),
            KIND_WORK_SUBMITTED,
            json!({"work": {
                "id": work_id,
                "intent": intent,
                "state": "pending",
                "created_by": "test",
                "created_at": "2026-01-01T00:00:00Z",
            }}),
        )
        .with_work_id(work_id);
        if let Some(root) = estate_root {
            draft = draft.with_workspace_id(root);
        }
        core.commit(draft).expect("commit");
    }
}
