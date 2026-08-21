//! W3 §7.2 / §13.1 step 6: one replay-equivalence proof per kind in
//! [`sergeant_rs::runtime::prune::NON_WORK_ALLOWLIST`].
//!
//! `NON_WORK_ALLOWLIST` names nine event kinds a segment may carry with no
//! `work_id` and still be prunable — an unpaired event of any *other* kind
//! pins its segment (`an_unknown_non_work_scoped_kind_pins_its_segment`,
//! N4, in `src/runtime/prune.rs`). Being on the list is a claim: that
//! kind's effect on the registry a replay produces is either absent,
//! force-reset at every startup, or carried forward by some other named
//! mechanism — never silently lost. Each test below proves its one kind's
//! claim directly: build a fixture twice, identical except that one copy
//! also carries one event of the kind under test inside the very range a
//! real prune condemns, run the real `prune::plan`/`prune::run` over both,
//! and assert whatever the allowlist's own comment for that kind promises.

use std::path::Path;

use serde_json::json;
use tempfile::TempDir;

use sergeant_rs::api::Core;
use sergeant_rs::daemon::{
    KIND_ADMISSION_PAUSED, KIND_ADMISSION_RESUMED, KIND_BACKEND_PROBED, KIND_DAEMON_STARTED,
    KIND_DAEMON_STOPPED,
};
use sergeant_rs::domain::event::{Event, EventDraft, EventSource};
use sergeant_rs::domain::work::{
    KIND_COMMAND_ACCEPTED, KIND_COMMAND_REJECTED, KIND_WORK_COMPLETED, KIND_WORK_SUBMITTED,
};
use sergeant_rs::runtime::journal::Journal;
use sergeant_rs::runtime::projection::{WorkRegistry, work_registry_projection};
use sergeant_rs::runtime::prune::{
    self, KIND_PRUNE_COMPLETED, KIND_PRUNE_INTENT, PolicySource, PrunePolicy,
};

fn draft(kind: &str, work_id: Option<&str>, payload: serde_json::Value) -> EventDraft {
    let source = if kind.starts_with("command.") {
        EventSource::new("daemon", "api")
    } else {
        EventSource::new("daemon", "sergeant")
    };
    let mut draft = EventDraft::new(source, kind, payload);
    if let Some(id) = work_id {
        draft = draft.with_work_id(id);
    }
    draft
}

fn submit_payload(work_id: &str, state: &str) -> serde_json::Value {
    json!({"work": {
        "id": work_id,
        "intent": "allowlist-equivalence fixture",
        "state": state,
        "created_by": "test",
        "created_at": "2026-01-01T00:00:00.000Z",
    }})
}

/// Build the fixture: `w_old` (prunable), one optional extra event of a
/// given kind inserted right after `w_old`'s own completion (still inside
/// its condemned span), and `w_keep` (active, pins the horizon just
/// above). Runs a real `prune::plan`/`prune::run` cycle and returns the
/// resulting registry plus whether the cycle actually advanced the floor
/// (i.e. the extra event did not pin its segment).
fn prune_with_optional_extra(dir: &Path, extra: Option<EventDraft>) -> (WorkRegistry, bool) {
    let journal = Journal::open_with(dir, 1).expect("open");
    let registry = work_registry_projection();
    let (events_tx, _rx) = tokio::sync::broadcast::channel::<Event>(16);
    let mut core = Core::new(journal, registry, events_tx);

    core.commit(draft(
        KIND_WORK_SUBMITTED,
        Some("w_old"),
        submit_payload("w_old", "pending"),
    ))
    .expect("commit w_old submitted");
    core.commit(draft(KIND_WORK_COMPLETED, Some("w_old"), json!({})))
        .expect("commit w_old completed");
    if let Some(extra) = extra {
        core.commit(extra).expect("commit the kind under test");
    }
    core.commit(draft(
        KIND_WORK_SUBMITTED,
        Some("w_keep"),
        submit_payload("w_keep", "pending"),
    ))
    .expect("commit w_keep submitted");
    core.flush().expect("flush");

    let bounds = core.journal.segment_bounds().expect("bounds");
    let policy = PrunePolicy {
        retention: 0,
        source: PolicySource::Config,
    };
    let (candidate, _) = prune::candidate_horizon(
        &bounds,
        core.registry.state(),
        &core.first_seq_by_work,
        &policy,
    );
    let Some(plan) = prune::plan(
        dir,
        &bounds,
        candidate,
        core.registry.state(),
        &core.first_seq_by_work,
        &policy,
    )
    .expect("plan") else {
        return (core.registry.state().clone(), false);
    };
    prune::run(&mut core, dir, plan, false).expect("run");
    (core.registry.state().clone(), true)
}

/// The comparison every per-kind test wants: the *shape* of the registry
/// (which ids exist, in which map, in which state) must agree between the
/// two fixtures. Never a byte-for-byte `WorkRegistry` equality — `w_keep`'s
/// own `last_seq` legitimately differs by one between the two fixtures
/// (the extra event under test occupies a seq of its own before it), and
/// `pruned_at` is a wall-clock timestamp stamped at prune time, neither of
/// which the allowlist's claim is about.
fn assert_registries_equivalent(without: &WorkRegistry, with: &WorkRegistry, msg: &str) {
    assert_eq!(
        without
            .work_index
            .keys()
            .collect::<std::collections::BTreeSet<_>>(),
        with.work_index
            .keys()
            .collect::<std::collections::BTreeSet<_>>(),
        "{msg} (work_index membership differs)"
    );
    for (id, row) in &without.work_index {
        let other = &with.work_index[id];
        assert_eq!(
            row.state, other.state,
            "{msg} (work_index[{id}].state differs)"
        );
        assert_eq!(
            row.intent, other.intent,
            "{msg} (work_index[{id}].intent differs)"
        );
    }
    assert_eq!(
        without
            .pruned_works
            .keys()
            .collect::<std::collections::BTreeSet<_>>(),
        with.pruned_works
            .keys()
            .collect::<std::collections::BTreeSet<_>>(),
        "{msg} (pruned_works membership differs)"
    );
}

/// Build both fixtures (with and without the kind under test) and assert
/// the equivalence the caller's `check` closure claims. Both copies use
/// the same `w_old`/`w_keep` shape, so any difference between `without`
/// and `with` is attributable only to the extra event.
fn assert_kind_is_replay_equivalent(
    kind_label: &str,
    build_extra: impl Fn() -> EventDraft,
    check: impl Fn(&WorkRegistry, &WorkRegistry),
) {
    let dir_without = TempDir::new().expect("tempdir");
    let (without, pruned_without) = prune_with_optional_extra(dir_without.path(), None);
    assert!(
        pruned_without,
        "the baseline fixture (no {kind_label}) must itself be prunable, or this proves nothing"
    );

    let dir_with = TempDir::new().expect("tempdir");
    let (with, pruned_with) = prune_with_optional_extra(dir_with.path(), Some(build_extra()));
    assert!(
        pruned_with,
        "a bare {kind_label} event (no work_id) must not pin its segment — it is on \
         NON_WORK_ALLOWLIST specifically so it does not"
    );

    assert!(
        with.pruned_works.contains_key("w_old"),
        "{kind_label} must not have prevented w_old from being pruned"
    );
    check(&without, &with);
}

#[test]
fn kind_daemon_started_is_replay_equivalent() {
    assert_kind_is_replay_equivalent(
        KIND_DAEMON_STARTED,
        || draft(KIND_DAEMON_STARTED, None, json!({"pid": 1})),
        |without, with| {
            assert_registries_equivalent(
                without,
                with,
                "daemon.started must have no registry effect at all",
            );
        },
    );
}

#[test]
fn kind_daemon_stopped_is_replay_equivalent() {
    assert_kind_is_replay_equivalent(
        KIND_DAEMON_STOPPED,
        || draft(KIND_DAEMON_STOPPED, None, json!({})),
        |without, with| {
            assert_registries_equivalent(
                without,
                with,
                "daemon.stopped must have no registry effect at all",
            );
        },
    );
}

#[test]
fn kind_backend_probed_is_replay_equivalent() {
    assert_kind_is_replay_equivalent(
        KIND_BACKEND_PROBED,
        || {
            draft(
                KIND_BACKEND_PROBED,
                None,
                json!({"backend": "fake", "available": true}),
            )
        },
        |without, with| {
            assert_registries_equivalent(
                without,
                with,
                "backend.probed must have no registry effect at all",
            );
        },
    );
}

/// §11's own mechanism: `admission_paused` is force-cleared unconditionally
/// at every startup (`daemon.rs`, before the descriptor is published and
/// before any request is served), so its replayed value is never
/// load-bearing — see `the_admission_pause_force_clear_mechanism_is_pinned`
/// in `tests/i9_floor_pinning.rs` for the live proof of *that* mechanism.
/// This test only proves the allowlist half: an `admission.paused` event
/// does not pin its own segment, and everything else about the registry is
/// unaffected by its presence or absence.
#[test]
fn kind_admission_paused_is_replay_equivalent() {
    assert_kind_is_replay_equivalent(
        KIND_ADMISSION_PAUSED,
        || draft(KIND_ADMISSION_PAUSED, None, json!({})),
        |without, with| {
            assert_registries_equivalent(
                without,
                with,
                "admission.paused must have no lasting registry effect",
            );
        },
    );
}

#[test]
fn kind_admission_resumed_is_replay_equivalent() {
    assert_kind_is_replay_equivalent(
        KIND_ADMISSION_RESUMED,
        || draft(KIND_ADMISSION_RESUMED, None, json!({})),
        |without, with| {
            assert_registries_equivalent(
                without,
                with,
                "admission.resumed must have no lasting registry effect",
            );
        },
    );
}

/// Q8: the command ledger key survives in `pruned_commands` even though
/// the event that recorded it is gone — the allowlist entry's whole claim.
#[test]
fn kind_command_accepted_is_replay_equivalent() {
    assert_kind_is_replay_equivalent(
        KIND_COMMAND_ACCEPTED,
        || {
            draft(
                KIND_COMMAND_ACCEPTED,
                None,
                json!({"command_id": "cmd-admin", "operation": "admission.pause",
                       "status": 200u16, "result": {}}),
            )
        },
        |without, with| {
            assert!(
                !without.pruned_commands.contains_key("cmd-admin"),
                "the baseline fixture must not carry the command under test"
            );
            assert!(
                with.pruned_commands.contains_key("cmd-admin"),
                "command.accepted's ledger key must survive in pruned_commands (Q8) once its \
                 own event is gone"
            );
            // Nothing else about the registry differs.
            assert_registries_equivalent(
                without,
                with,
                "command.accepted's presence must not otherwise change the registry",
            );
        },
    );
}

#[test]
fn kind_command_rejected_is_replay_equivalent() {
    assert_kind_is_replay_equivalent(
        KIND_COMMAND_REJECTED,
        || {
            draft(
                KIND_COMMAND_REJECTED,
                None,
                json!({"command_id": "cmd-admin-2", "operation": "admission.pause",
                       "status": 409u16, "result": {"error": {"code": "already_paused"}}}),
            )
        },
        |without, with| {
            assert!(
                !without.pruned_commands.contains_key("cmd-admin-2"),
                "the baseline fixture must not carry the command under test"
            );
            assert!(
                with.pruned_commands.contains_key("cmd-admin-2"),
                "command.rejected's ledger key must survive in pruned_commands (Q8) once its \
                 own event is gone"
            );
            assert_registries_equivalent(
                without,
                with,
                "command.rejected's presence must not otherwise change the registry",
            );
        },
    );
}

/// `prune.intent`/`prune.completed` are on the allowlist for a different
/// reason from the other seven: an *unpaired* one still pins its segment
/// (§6.5, N19 — `a_prune_pair_straddling_the_horizon_pins_its_segment` in
/// `src/runtime/prune.rs`); only a **fully paired**, already-folded-and-
/// carried-forward pair is safe to lose, because §6.4's carry-forward
/// mechanism (proved end-to-end by
/// `a_carried_forward_residue_survives_three_generations_of_prune`) is what
/// keeps its residue alive after both halves are gone. What this test adds
/// beyond that one: proving a *bare*, malformed-payload `prune.intent`/
/// `prune.completed` pair (not a real cycle's own output) still does not
/// pin — §6.2's "a declaration that cannot be understood is not a
/// declaration this process can act on" applies to the allowlist check
/// itself, which runs before any attempt to parse the payload.
#[test]
fn kind_prune_intent_is_replay_equivalent() {
    assert_kind_is_replay_equivalent(
        KIND_PRUNE_INTENT,
        || draft(KIND_PRUNE_INTENT, None, json!({})),
        |without, with| {
            assert_registries_equivalent(
                without,
                with,
                "a bare prune.intent must have no other registry effect",
            );
        },
    );
}

#[test]
fn kind_prune_completed_is_replay_equivalent() {
    assert_kind_is_replay_equivalent(
        KIND_PRUNE_COMPLETED,
        || draft(KIND_PRUNE_COMPLETED, None, json!({})),
        |without, with| {
            assert_registries_equivalent(
                without,
                with,
                "a bare prune.completed must have no other registry effect",
            );
        },
    );
}

/// Reachability sanity check: every kind this file claims to cover really
/// is a member of the allowlist it is testing against — if the allowlist
/// ever grows or shrinks, this fails loudly rather than this file silently
/// meaning less than its own doc comment says.
#[test]
fn every_kind_this_file_tests_is_actually_on_the_allowlist() {
    let tested = [
        KIND_DAEMON_STARTED,
        KIND_DAEMON_STOPPED,
        KIND_BACKEND_PROBED,
        KIND_ADMISSION_PAUSED,
        KIND_ADMISSION_RESUMED,
        KIND_COMMAND_ACCEPTED,
        KIND_COMMAND_REJECTED,
        KIND_PRUNE_INTENT,
        KIND_PRUNE_COMPLETED,
    ];
    assert_eq!(
        tested.len(),
        prune::NON_WORK_ALLOWLIST.len(),
        "this file must test exactly the allowlist's own kinds, one test each"
    );
    for kind in tested {
        assert!(
            prune::NON_WORK_ALLOWLIST.contains(&kind),
            "{kind} is tested here but is not on NON_WORK_ALLOWLIST"
        );
    }
    for kind in prune::NON_WORK_ALLOWLIST {
        assert!(
            tested.contains(kind),
            "{kind} is on NON_WORK_ALLOWLIST but has no test in this file"
        );
    }
}
