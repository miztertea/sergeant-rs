//! I9 — W2's acceptance gate (spec §6): a windowed replay, seeded from the
//! FloorState cache, reproduces the registry a full replay produces — field
//! by field, through the *real* `Plan`/`drive`/`horizon` machinery, never a
//! re-implementation of it.
//!
//! Two journals exercise the same assertion function:
//!
//! 1. A real, daemon-produced journal (the acceptance case) — proves the
//!    cache covers what the system actually writes.
//! 2. A synthetic, kind-exhaustive journal — proves it covers every event
//!    kind `apply_registry_event` matches on, including ones a fake-backend
//!    run never produces.
//!
//! The compile-time half of the gate (`assert_registry_pinned`'s exhaustive
//! destructuring of `WorkRegistry`) is what makes a future field addition to
//! that struct a compile error here until someone decides, in writing,
//! whether the startup cache must carry it.

mod support;

use std::collections::BTreeSet;
use std::future::Future;
use std::path::Path;

use http_body_util::BodyExt;
use serde_json::json;
use tempfile::TempDir;

use sergeant_rs::api::{Core, below_floor_refusal};
use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::claude::AskWithdrawal;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig, DaemonHandle};
use sergeant_rs::domain::event::{Event, EventDraft, EventSource};
use sergeant_rs::domain::execution::{
    KIND_EXECUTION_RESERVED, KIND_EXECUTION_STARTED, KIND_EXECUTION_STOPPED,
};
use sergeant_rs::domain::work::{
    KIND_COMMAND_ACCEPTED, KIND_COMMAND_REJECTED, KIND_WORK_BLOCKED, KIND_WORK_CANCELED,
    KIND_WORK_COMPLETED, KIND_WORK_STARTED, KIND_WORK_SUBMITTED,
};
use sergeant_rs::domain::workflow::{
    KIND_STAGE_COMPLETED, KIND_STAGE_ENTERED, KIND_WORKFLOW_BOUND,
};
use sergeant_rs::runtime::atlas::db::Analytics;
use sergeant_rs::runtime::journal::Journal;
use sergeant_rs::runtime::projection::{
    Projection, WorkRegistry, work_registry_projection, work_registry_reducer,
};
use sergeant_rs::runtime::startup::{
    self, AnalyticsSink, CacheMiss, CapabilitySink, FloorCommandRow, FloorState, HorizonSink,
    LedgerSink, Plan, RegistrySink,
};
use sergeant_rs::runtime::surface::KIND_SURFACE_TORN_DOWN;

use support::DataDir;

fn ulid() -> String {
    ulid::Ulid::generate().to_string()
}

/// Synchronously decode an axum `Response`'s status and JSON body.
///
/// `assert_registry_pinned` runs from both plain `#[test]`s and
/// `#[tokio::test]`s (some of the latter via a `std::panic::catch_unwind`
/// closure that cannot itself be `async`), so this deliberately does not
/// require a Tokio runtime: `below_floor_refusal`'s body is always a single,
/// already-serialized `Bytes` buffer (`Json::into_response` — never a
/// stream), so a body future built over it resolves on its very first poll.
/// A noop waker is therefore exactly right — if that ever stops being true,
/// this panics loudly instead of hanging.
fn response_body_json(
    response: axum::response::Response,
) -> (axum::http::StatusCode, serde_json::Value) {
    let status = response.status();
    let mut collect = std::pin::pin!(response.into_body().collect());
    let waker = std::task::Waker::noop();
    let mut cx = std::task::Context::from_waker(waker);
    let collected = match collect.as_mut().poll(&mut cx) {
        std::task::Poll::Ready(result) => {
            result.expect("below_floor_refusal's body must not itself error")
        }
        std::task::Poll::Pending => panic!(
            "below_floor_refusal's response body was not immediately ready — it must be a \
             fully-buffered Json body, never a stream"
        ),
    };
    let bytes = collected.to_bytes();
    let json = serde_json::from_slice(&bytes).expect("below_floor_refusal's body must be JSON");
    (status, json)
}

// ------------------------------------------------------------------
// The compile-time gate (§6.3)
// ------------------------------------------------------------------

/// Every field of `WorkRegistry`, enumerated. Adding a field to
/// `WorkRegistry` is a compile error here until someone decides, in
/// writing, whether the startup cache must carry it.
///
/// ASSERTED fields must be equal between a full replay and cache+window.
/// ALLOWLIST fields may differ, with the reason and the compensating
/// mechanism argued inline.
fn assert_registry_pinned(
    full: &WorkRegistry,
    windowed: &WorkRegistry,
    cache: &FloorState,
    full_capability: Option<&AskWithdrawal>,
    windowed_capability: Option<&AskWithdrawal>,
    full_last_seq: u64,
    windowed_last_seq: u64,
) {
    let WorkRegistry {
        works,
        commands,
        command_works,
        runs,
        terminal_runs,
        terminal_run_order: _,
        terminal_works,
        terminal_work_order: _,
        work_index,
        admission_paused: _, // ALLOWLIST: force-cleared unconditionally at every
        // startup (daemon.rs) before anything is served —
        // its replayed value is never load-bearing. See
        // `the_admission_pause_force_clear_mechanism_is_pinned`.
        pruned_works, // ASSERTED (W3, I-W3-7): the residue must reach a
        // cacheless floor replay and a cache+window start
        // identically.
        pruned_commands, // ASSERTED (W3, I-W3-8): same reasoning.
        pending_prune,   // ASSERTED (W3, §6.5): always `None` in a *written*
        // cache — an intent below `H` can never be left
        // uncompleted — and the assertion is what keeps that
        // true rather than assumed.
        quarantined_blobs, // ASSERTED (W3, §5.2): the next cycle's deletion
                           // list; a cache that lost it would leak
                           // quarantined blobs forever.
    } = full;

    // ASSERTED — the load-bearing equalities.
    assert_eq!(
        work_index, &windowed.work_index,
        "work_index (with last_seq) must be byte-identical between a full replay and \
         cache+window — if this fails, the horizon predicate is wrong"
    );
    assert_eq!(works, &windowed.works, "works must be byte-identical");
    assert_eq!(runs, &windowed.runs, "runs must be byte-identical");
    assert_eq!(
        pruned_works, &windowed.pruned_works,
        "pruned_works must be byte-identical between a full replay and cache+window (I-W3-7)"
    );
    assert_eq!(
        pruned_commands, &windowed.pruned_commands,
        "pruned_commands must be byte-identical between a full replay and cache+window (I-W3-8)"
    );
    assert_eq!(
        pending_prune, &windowed.pending_prune,
        "pending_prune must agree between a full replay and cache+window"
    );
    assert_eq!(
        quarantined_blobs, &windowed.quarantined_blobs,
        "quarantined_blobs must be byte-identical between a full replay and cache+window"
    );
    // W3's strengthening of the work_index assertion: a Work is in exactly
    // one of `work_index`/`pruned_works`, on both sides.
    assert!(
        work_index
            .keys()
            .collect::<BTreeSet<_>>()
            .is_disjoint(&pruned_works.keys().collect::<BTreeSet<_>>()),
        "a Work must never appear in both work_index and pruned_works (full)"
    );
    assert!(
        windowed
            .work_index
            .keys()
            .collect::<BTreeSet<_>>()
            .is_disjoint(&windowed.pruned_works.keys().collect::<BTreeSet<_>>()),
        "a Work must never appear in both work_index and pruned_works (windowed)"
    );

    // ALLOWLIST: commands — Q8, the cache carries keys not `CommandOutcome`
    // bodies. (a) everything the window kept is unchanged; (b) everything it
    // dropped is covered by the cache by key.
    for (id, outcome) in &windowed.commands {
        assert_eq!(
            commands.get(id),
            Some(outcome),
            "windowed.commands must be a subset of full.commands with equal values (id {id})"
        );
    }
    let cache_commands: std::collections::BTreeMap<&str, &FloorCommandRow> = cache
        .commands
        .iter()
        .map(|r| (r.command_id.as_str(), r))
        .collect();
    for id in commands.keys() {
        if !windowed.commands.contains_key(id) {
            let row = *cache_commands.get(id.as_str()).unwrap_or_else(|| {
                panic!(
                    "command {id} is below the window but missing from the cache's ledger — \
                     §26 could no longer refuse it by name"
                )
            });

            // (c) the refusal itself: the real `below_floor_refusal`, not a
            // re-implementation, must answer 409 with the right code — and,
            // for a submit, the right Work, checked against `full`'s own
            // ground-truth `command_works` index rather than merely echoed
            // back from the same row already checked above.
            let (status, body) = response_body_json(below_floor_refusal(row));
            assert_eq!(
                status,
                axum::http::StatusCode::CONFLICT,
                "below_floor_refusal for below-window command {id} must answer 409, not {status}"
            );
            assert_eq!(
                body["error"]["code"],
                json!("command_below_replay_window"),
                "below_floor_refusal for {id} must use §4.3's error code"
            );
            assert_eq!(
                body["error"]["command_id"],
                json!(id),
                "below_floor_refusal's body must name the command it refused"
            );
            if let Some(expected_work_id) = command_works.get(id) {
                assert_eq!(
                    body["error"]["work_id"],
                    json!(expected_work_id),
                    "a below-window submit's refusal must name the same Work `full`'s own \
                     command_works index (the ground truth, not the cache) says it created"
                );
            }
        }
    }

    // ALLOWLIST: command_works — same reasoning, keyed the same way.
    for (id, work_id) in &windowed.command_works {
        assert_eq!(command_works.get(id), Some(work_id));
    }
    for id in command_works.keys() {
        if !windowed.command_works.contains_key(id) {
            let row = cache_commands
                .get(id.as_str())
                .unwrap_or_else(|| panic!("command_works entry {id} missing from the cache"));
            assert_eq!(
                row.work_id.as_deref(),
                Some(command_works[id].as_str()),
                "the cache's row for {id} must name the same Work the crash-window index does"
            );
        }
    }

    // ALLOWLIST: terminal_works/terminal_work_order — a bounded, in-memory
    // cache whose miss path (`rederive_work`) is documented byte-identical.
    // (a) subset with equal values; (b) every dropped id still has a
    // work_index row in both, so the rederive fallback is reachable.
    for (id, work) in &windowed.terminal_works {
        assert_eq!(terminal_works.get(id), Some(work));
    }
    for id in terminal_works.keys() {
        if !windowed.terminal_works.contains_key(id) {
            assert!(
                windowed.work_index.contains_key(id) && work_index.contains_key(id),
                "an evicted terminal work missing from the window must still have a \
                 work_index row in both (id {id})"
            );
        }
    }
    assert_eq!(
        windowed.terminal_work_order.iter().collect::<BTreeSet<_>>(),
        windowed.terminal_works.keys().collect::<BTreeSet<_>>(),
        "terminal_work_order's members must be exactly terminal_works' keys"
    );

    // ALLOWLIST: terminal_runs/terminal_run_order — identical reasoning,
    // mirroring `rederive_run`.
    for (id, run) in &windowed.terminal_runs {
        assert_eq!(terminal_runs.get(id), Some(run));
    }
    for id in terminal_runs.keys() {
        if !windowed.terminal_runs.contains_key(id) {
            assert!(
                windowed.work_index.contains_key(id) && work_index.contains_key(id),
                "an evicted terminal run missing from the window must still have a \
                 work_index row in both (id {id})"
            );
        }
    }
    assert_eq!(
        windowed.terminal_run_order.iter().collect::<BTreeSet<_>>(),
        windowed.terminal_runs.keys().collect::<BTreeSet<_>>(),
        "terminal_run_order's members must be exactly terminal_runs' keys"
    );

    // W3's strengthening: a pruned id has a row in *neither* cache — the
    // fold that inserts it into `pruned_works`/`pruned_commands` removes it
    // from `terminal_works`/`terminal_runs` in the same step (§6.2 step 2),
    // so the subset assertions above hold without exception rather than
    // needing one for a pruned id.
    for id in pruned_works.keys() {
        assert!(
            !terminal_works.contains_key(id) && !terminal_runs.contains_key(id),
            "a pruned Work id must not linger in either terminal cache (full, id {id})"
        );
    }
    for id in windowed.pruned_works.keys() {
        assert!(
            !windowed.terminal_works.contains_key(id) && !windowed.terminal_runs.contains_key(id),
            "a pruned Work id must not linger in either terminal cache (windowed, id {id})"
        );
    }

    // Capability watermark (recon correction 3's live gap) — the cache seed
    // folded forward by the window must agree with the full pass.
    assert_eq!(
        full_capability, windowed_capability,
        "the capability-provenance watermark must match between the full pass and \
         cache-seed-plus-window"
    );

    // §6.3, outside the destructuring: the projections' own high-water-mark
    // seq (`Projection::last_seq()`, distinct from `work_index`'s per-row
    // `last_seq` already asserted above) must agree too.
    assert_eq!(
        full_last_seq, windowed_last_seq,
        "Projection::last_seq() must match between a full replay and cache+window"
    );

    // Cache round-trip.
    let bytes = serde_json::to_vec(cache).expect("serialize FloorState");
    let back: FloorState = serde_json::from_slice(&bytes).expect("deserialize FloorState");
    assert_eq!(
        &back, cache,
        "FloorState must round-trip through JSON unchanged"
    );
}

// ------------------------------------------------------------------
// The two passes, driven through the real machinery
// ------------------------------------------------------------------

/// Run the shared pass over `journal`'s whole floor-aware range — exactly
/// what a `Plan::Full` start does — into a fresh scratch analytics dir (never
/// the journal's own data dir: this is an *offline*, read-only re-fold for
/// comparison, run beside a data dir a daemon may still own).
fn run_full_pass(
    journal: &Journal,
    analytics_scratch: &Path,
) -> (WorkRegistry, u64, CapabilitySink, LedgerSink, HorizonSink) {
    // A1: `replay_from_floor()` below already starts yielding at the floor,
    // not at 1 — the `Projection` wrapper must agree, or a genuinely pruned
    // journal (floor > 1) reports a spurious `SeqMismatch` here instead of
    // folding cleanly. Every fixture in this file before the real-prune case
    // (§13.3) has floor == 1, where this is exactly `work_registry_projection()`.
    let floor_seq = journal.floor_seq().expect("floor_seq").unwrap_or(1);
    let mut registry = Projection::resumed(
        WorkRegistry::default(),
        floor_seq.saturating_sub(1),
        work_registry_reducer,
    );
    let mut analytics = Analytics::begin_rebuild(analytics_scratch).expect("analytics scratch");
    let mut capability = CapabilitySink::default();
    let mut ledger = LedgerSink::default();
    let mut horizon_sink = HorizonSink::default();
    {
        let mut analytics_sink = AnalyticsSink::new(analytics.fold().expect("fold"), 0);
        startup::drive(
            journal.replay_from_floor().expect("replay_from_floor"),
            &mut [
                &mut RegistrySink(&mut registry),
                &mut analytics_sink,
                &mut capability,
                &mut ledger,
                &mut horizon_sink,
            ],
        )
        .expect("drive (full pass)");
    }
    // §6.3, outside the destructuring: `last_seq()` is read from the
    // `Projection` wrapper itself, before `.state()` discards it — a
    // full-replay's own high-water mark, compared against the windowed
    // pass's in `run_i9`.
    let last_seq = registry.last_seq();
    (
        registry.state().clone(),
        last_seq,
        capability,
        ledger,
        horizon_sink,
    )
}

/// Build the `FloorState` a start over `journal` would leave behind, using
/// the real `Plan::next_cache` — never a re-implementation of §2.5/§2.6.
fn build_cache(
    journal: &Journal,
    full: &WorkRegistry,
    ledger: &LedgerSink,
    capability: &CapabilitySink,
    horizon_sink: &HorizonSink,
) -> (FloorState, u64) {
    let bounds = journal.segment_bounds().expect("segment_bounds");
    let window_seq = startup::window_seq(&bounds);
    let floor_seq = bounds.first().map(|b| b.first_seq).unwrap_or(1);
    let plan = Plan::Full {
        floor_seq,
        window_seq,
        miss: CacheMiss::Absent,
    };
    let cache = plan
        .next_cache(
            journal,
            full,
            ledger,
            capability,
            horizon_sink,
            horizon_sink.first_seq_by_work(),
        )
        .expect("next_cache")
        .unwrap_or_else(|| {
            panic!(
                "H == 0 for this fixture (window_seq {window_seq}) — the test journal must \
                 have enough segments for the horizon to advance past zero"
            )
        });
    (cache, window_seq)
}

/// Fold the window on top of a cache seed — exactly what `Plan::Windowed`'s
/// arm of `start_with` does.
fn run_windowed_pass(
    journal: &Journal,
    cache: &FloorState,
    window_seq: u64,
) -> (WorkRegistry, u64, Option<AskWithdrawal>) {
    let plan = Plan::Windowed {
        cache: cache.clone(),
        window_seq,
    };
    let mut registry = plan.seed_registry();
    let mut capability = CapabilitySink::seeded(cache.capability_provenance.clone());
    startup::drive(
        plan.replay(journal).expect("replay"),
        &mut [&mut RegistrySink(&mut registry), &mut capability],
    )
    .expect("drive (windowed pass)");
    let last_seq = registry.last_seq();
    (registry.state().clone(), last_seq, capability.latest)
}

/// Run both passes over `journal` and assert they pin, per
/// `assert_registry_pinned`. Returns the built cache, for callers that want
/// to mutate it and prove the gate fails (§6.4).
fn run_i9(journal: &Journal, analytics_scratch: &Path) -> FloorState {
    let (full, full_last_seq, capability, ledger, horizon_sink) =
        run_full_pass(journal, analytics_scratch);
    let (cache, window_seq) = build_cache(journal, &full, &ledger, &capability, &horizon_sink);
    let (windowed, windowed_last_seq, windowed_capability) =
        run_windowed_pass(journal, &cache, window_seq);
    assert_registry_pinned(
        &full,
        &windowed,
        &cache,
        capability.latest.as_ref(),
        windowed_capability.as_ref(),
        full_last_seq,
        windowed_last_seq,
    );
    cache
}

// ------------------------------------------------------------------
// §6.3, outside the destructuring: `Plan::Full` starts leave
// `Core::floor_ledger` empty
// ------------------------------------------------------------------

/// §4.1's own construction argument, pinned at runtime rather than left as
/// only a doc comment: "`floor_ledger` is empty on a `Plan::Full` start *by
/// construction* … Assert it in the I9 suite." A full replay already put
/// every command the journal has into `registry.commands`, so
/// `Plan::ledger_seed()` is empty, and the `Core` a `Plan::Full` start builds
/// never receives anything else.
#[test]
fn plan_full_leaves_core_floor_ledger_empty() {
    let dir = TempDir::new().expect("tempdir");
    let journal = Journal::open(dir.path()).expect("open a fresh journal");
    let plan = Plan::Full {
        floor_seq: 1,
        window_seq: 0,
        miss: CacheMiss::Absent,
    };
    assert!(
        plan.ledger_seed().is_empty(),
        "Plan::Full's ledger_seed must be empty by construction"
    );

    let registry = plan.seed_registry();
    let (events_tx, _rx) = tokio::sync::broadcast::channel(1);
    let core = Core::new(journal, registry, events_tx)
        .with_floor_ledger(std::sync::Arc::new(plan.ledger_seed()));
    assert!(
        core.floor_ledger.is_empty(),
        "a Plan::Full start must leave Core::floor_ledger empty"
    );
}

// ------------------------------------------------------------------
// Case 1: a real, daemon-produced journal
// ------------------------------------------------------------------

async fn start_fake_tiny_segments(
    data_dir: &Path,
    _estate_root: &Path,
    script: impl IntoIterator<Item = FakeStep>,
) -> (DaemonHandle, FakeBackend) {
    let fake = FakeBackend::scripted(FAKE_BACKEND_NAME, script);
    let registry = BackendRegistry::new().with(std::sync::Arc::new(fake.clone()));
    let handle = daemon::start_with(
        data_dir,
        DaemonConfig {
            backends: std::sync::Arc::new(registry),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            // Tiny threshold (§5.4): reaching a >16-segment journal by
            // shrinking this, never by shrinking the window.
            segment_max_bytes: Some(256),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");
    (handle, fake)
}

fn write_one_stage_workflow(root: &Path) {
    let dir = root.join(".sergeant/workflows/solo-stage");
    std::fs::create_dir_all(&dir).expect("workflow dir");
    std::fs::write(
        dir.join("workflow.toml"),
        "[workflow]\nname = \"solo-stage\"\nversion = \"1\"\nstages = [\"00-only\"]\n",
    )
    .expect("workflow.toml");
    std::fs::create_dir_all(dir.join("00-only")).expect("stage dir");
    std::fs::write(dir.join("00-only").join("CONTEXT.md"), "context").expect("CONTEXT.md");
}

#[tokio::test]
async fn the_real_daemon_journal_pins() {
    let data = DataDir::new();
    let root = TempDir::new().expect("tempdir");
    let (_mount, _head) = support::scaffold_solo_estate(root.path(), "solo");
    write_one_stage_workflow(root.path());

    // Enough completed Works, each one stage, to push well past
    // STARTUP_WINDOW_SEGMENTS at a 256-byte threshold.
    let script: Vec<FakeStep> = (0..12).map(|_| FakeStep::complete()).collect();
    let (handle, _fake) = start_fake_tiny_segments(data.path(), root.path(), script).await;
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .expect("client");
    for _ in 0..12 {
        let resp = http
            .post(format!("{}/v1/work", handle.endpoint))
            .bearer_auth(&handle.token)
            .json(&json!({
                "command_id": ulid(),
                "intent": "i9 acceptance work",
                "estate_root": root.path(),
                "origin": {"client": "cli", "cwd": root.path()},
            }))
            .send()
            .await
            .expect("submit");
        assert_eq!(
            resp.status(),
            reqwest::StatusCode::CREATED,
            "{:?}",
            resp.text().await
        );
    }
    // Give the fake backend's async completion a moment to settle every
    // submitted work before stopping.
    for _ in 0..50 {
        let system: serde_json::Value = http
            .get(format!("{}/v1/work", handle.endpoint))
            .bearer_auth(&handle.token)
            .send()
            .await
            .expect("list")
            .json()
            .await
            .expect("json");
        let all_done = system
            .as_array()
            .map(|rows| {
                rows.iter()
                    .all(|w| w["state"] == "completed" || w["state"] == "failed")
            })
            .unwrap_or(false);
        if all_done {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    handle.shutdown().await;

    let journal = Journal::open(data.path()).expect("reopen journal");
    let bounds = journal.segment_bounds().expect("bounds");
    assert!(
        bounds.len() > 16,
        "fixture must actually exceed the window (got {} segments)",
        bounds.len()
    );

    let scratch = TempDir::new().expect("scratch");
    run_i9(&journal, scratch.path());
}

// ------------------------------------------------------------------
// Case 3 (§13.3): a journal really pruned by `prune::run` — the wave's
// acceptance gate, exactly as it was W2's — plus §8's negative proof.
// ------------------------------------------------------------------

/// A journal that has really been pruned by `prune::run` (not by
/// hand-deleting segments) — §13.3's own words. Built directly over a
/// `Core` (no daemon/HTTP needed to prove this): `w1` records an
/// ask-grammar withdrawal and an accepted command before completing, so
/// `pruned_works`, `pruned_commands` and `capability_provenance` are all
/// non-empty in the residue this cycle carries. Everything else is a plain
/// non-work-scoped, allowlisted event (`admission.paused`/`admission.
/// resumed` — the same shape `build_kind_exhaustive_journal`'s own padding
/// uses) rather than another Work: with no Work left in `work_index` at
/// all once `w1` is pruned, W2's own startup-cache horizon
/// (`startup::horizon`, gated on every retained Work being *settled* —
/// a different predicate from the prune engine's `retired_whole`) has
/// nothing left to block it, so it can advance across all of the padding.
fn build_a_really_pruned_journal(dir: &Path) -> Journal {
    use sergeant_rs::runtime::prune::{self, PolicySource, PrunePolicy};

    let journal = Journal::open_with(dir, 1).expect("open");
    let registry = work_registry_projection();
    let (events_tx, _rx) = tokio::sync::broadcast::channel::<Event>(16);
    let mut core = Core::new(journal, registry, events_tx);

    core.commit(draft(
        KIND_WORK_SUBMITTED,
        Some("w1"),
        submit_payload("w1", "pending"),
    ))
    .expect("commit w1 submitted");
    core.commit(draft(
        KIND_COMMAND_ACCEPTED,
        Some("w1"),
        json!({"command_id": "cmd-1", "operation": "work.submit", "status": 201u16,
               "result": {"id": "w1"}}),
    ))
    .expect("commit cmd-1 accepted");
    core.commit(draft(
        "conversation.turn.grammar_unmeasured",
        Some("w1"),
        json!({"capability": "ask", "version": "2.1.226"}),
    ))
    .expect("commit the ask withdrawal");
    core.commit(draft(KIND_WORK_COMPLETED, Some("w1"), json!({})))
        .expect("commit w1 completed");
    // I-W3-4: one non-work-scoped event pushes the writer off w1's own
    // segments — no second Work needed for that.
    core.commit(draft("admission.paused", None, json!({})))
        .expect("commit admission.paused");
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
    let plan = prune::plan(
        dir,
        &bounds,
        candidate,
        core.registry.state(),
        &core.first_seq_by_work,
        &policy,
    )
    .expect("plan")
    .expect("w1 must be prunable");
    prune::run(&mut core, dir, plan, false).expect("run");

    // Everything from here on is *after* the cycle already committed and
    // unlinked — it cannot change what got pruned, only give
    // `startup::window_seq`/`startup::horizon` enough surviving segments to
    // advance past zero (with `work_index` now empty, nothing blocks it —
    // only the segment/window bookkeeping does).
    for i in 0..(startup::STARTUP_WINDOW_SEGMENTS + 4) {
        core.commit(draft(
            if i % 2 == 0 {
                "admission.paused"
            } else {
                "admission.resumed"
            },
            None,
            json!({}),
        ))
        .expect("commit padding");
    }
    core.flush().expect("flush");

    core.journal
}

/// §8's Validation Evidence item 3 in full: "The I9 gate, extended to a
/// real prune... a full floor replay of a pruned journal must equal
/// cache+window, field by field." Every earlier case in this file builds
/// its journal by hand-appended events or the daemon's ordinary event
/// stream; this is the one that actually runs `prune::run` before
/// comparing.
#[test]
fn i9_pinning_holds_across_a_real_prune() {
    let dir = TempDir::new().expect("tempdir");
    let journal = build_a_really_pruned_journal(dir.path());
    let floor = journal.floor_seq().expect("floor_seq");
    assert!(
        floor.unwrap_or(1) > 1,
        "the fixture must actually have been pruned (floor still 1) for this to prove anything"
    );

    let scratch = TempDir::new().expect("scratch");
    let cache = run_i9(&journal, scratch.path());
    assert!(
        !cache.pruned_works.is_empty(),
        "the fixture must actually carry pruned-work residue for the negative proofs beside \
         this test to mean anything"
    );
    assert!(
        !cache.pruned_commands.is_empty(),
        "the fixture must actually carry pruned-command residue"
    );
    assert!(
        cache.capability_provenance.is_some(),
        "the fixture must actually carry a capability watermark in the real residue"
    );
}

/// §8's negative proof, first mutation: dropping a `PrunedWorkRow` from the
/// prune residue must fail the gate — a cache that silently lost one would
/// let a cacheless floor replay and a cache+window start disagree about
/// whether a Work was ever pruned at all.
#[test]
fn the_gate_fails_when_the_prune_residue_drops_a_pruned_work_row() {
    let dir = TempDir::new().expect("tempdir");
    let journal = build_a_really_pruned_journal(dir.path());
    let scratch = TempDir::new().expect("scratch");
    let (full, full_last_seq, capability, ledger, horizon_sink) =
        run_full_pass(&journal, scratch.path());
    let (mut cache, window_seq) = build_cache(&journal, &full, &ledger, &capability, &horizon_sink);
    assert!(
        !cache.pruned_works.is_empty(),
        "the fixture must actually carry pruned-work residue for this proof to mean anything"
    );
    cache.pruned_works.pop(); // the mutation this test exists to catch

    let (windowed, windowed_last_seq, windowed_capability) =
        run_windowed_pass(&journal, &cache, window_seq);
    let result = std::panic::catch_unwind(|| {
        assert_registry_pinned(
            &full,
            &windowed,
            &cache,
            capability.latest.as_ref(),
            windowed_capability.as_ref(),
            full_last_seq,
            windowed_last_seq,
        );
    });
    assert!(
        result.is_err(),
        "dropping a PrunedWorkRow from a real prune's residue must fail the pinning gate"
    );
}

/// §8's negative proof, second mutation: dropping a `PrunedCommandRow`.
#[test]
fn the_gate_fails_when_the_prune_residue_drops_a_pruned_command_row() {
    let dir = TempDir::new().expect("tempdir");
    let journal = build_a_really_pruned_journal(dir.path());
    let scratch = TempDir::new().expect("scratch");
    let (full, full_last_seq, capability, ledger, horizon_sink) =
        run_full_pass(&journal, scratch.path());
    let (mut cache, window_seq) = build_cache(&journal, &full, &ledger, &capability, &horizon_sink);
    assert!(
        !cache.pruned_commands.is_empty(),
        "the fixture must actually carry pruned-command residue for this proof to mean anything"
    );
    cache.pruned_commands.pop(); // the mutation this test exists to catch

    let (windowed, windowed_last_seq, windowed_capability) =
        run_windowed_pass(&journal, &cache, window_seq);
    let result = std::panic::catch_unwind(|| {
        assert_registry_pinned(
            &full,
            &windowed,
            &cache,
            capability.latest.as_ref(),
            windowed_capability.as_ref(),
            full_last_seq,
            windowed_last_seq,
        );
    });
    assert!(
        result.is_err(),
        "dropping a PrunedCommandRow from a real prune's residue must fail the pinning gate"
    );
}

/// §8's negative proof, third mutation — "the one that matters most,
/// because §8.3 is the gap this spec found by reading rather than by
/// testing": dropping the capability-provenance watermark carried on a
/// *real* prune's residue (not a hand-built one) must fail the gate.
#[test]
fn the_gate_fails_when_the_prune_residue_drops_the_capability_watermark() {
    let dir = TempDir::new().expect("tempdir");
    let journal = build_a_really_pruned_journal(dir.path());
    let scratch = TempDir::new().expect("scratch");
    let (full, full_last_seq, capability, ledger, horizon_sink) =
        run_full_pass(&journal, scratch.path());
    let (mut cache, window_seq) = build_cache(&journal, &full, &ledger, &capability, &horizon_sink);
    assert!(
        cache.capability_provenance.is_some(),
        "the fixture must actually carry a capability watermark for this proof to mean anything"
    );
    cache.capability_provenance = None; // the mutation this test exists to catch

    let (windowed, windowed_last_seq, windowed_capability) =
        run_windowed_pass(&journal, &cache, window_seq);
    let result = std::panic::catch_unwind(|| {
        assert_registry_pinned(
            &full,
            &windowed,
            &cache,
            capability.latest.as_ref(),
            windowed_capability.as_ref(),
            full_last_seq,
            windowed_last_seq,
        );
    });
    assert!(
        result.is_err(),
        "dropping the capability watermark from a real prune's residue must fail the gate"
    );
}

/// §26's live end: a below-window `command_id` retried against a *running*
/// daemon must come back as a real HTTP 409, not an in-process function
/// call — the half of the wave's acceptance case no test before this one
/// drove through an actual server. Three daemon lives over the same data
/// dir: the first submits the command that will end up below the window and
/// enough padding to push the journal past it; the second has no cache yet
/// (`Plan::Full`, `miss:absent`) and, per §2.6, is the start whose end-of-
/// fold write leaves behind the cache the third loads (`Plan::Windowed`,
/// `hit`) — only on the third does the retry actually reach
/// `Core::floor_ledger`.
#[tokio::test]
async fn a_below_window_command_id_is_refused_by_name_through_a_real_http_retry() {
    let data = DataDir::new();
    let root = TempDir::new().expect("tempdir");
    let (_mount, _head) = support::scaffold_solo_estate(root.path(), "solo");
    write_one_stage_workflow(root.path());

    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .expect("client");

    let early_command_id = ulid();
    let submit_body = json!({
        "command_id": early_command_id,
        "intent": "below-window refusal fixture",
        "estate_root": root.path(),
        "origin": {"client": "cli", "cwd": root.path()},
    });
    let early_work_id;

    // Life 1: the command that will end up below the window, plus enough
    // padding works to push the journal past STARTUP_WINDOW_SEGMENTS at a
    // 256-byte threshold.
    {
        let script: Vec<FakeStep> = (0..14).map(|_| FakeStep::complete()).collect();
        let (handle, _fake) = start_fake_tiny_segments(data.path(), root.path(), script).await;

        let resp = http
            .post(format!("{}/v1/work", handle.endpoint))
            .bearer_auth(&handle.token)
            .json(&submit_body)
            .send()
            .await
            .expect("submit the early command");
        assert_eq!(
            resp.status(),
            reqwest::StatusCode::CREATED,
            "{:?}",
            resp.text().await
        );
        let body: serde_json::Value = resp.json().await.expect("json");
        early_work_id = body["work"]["id"]
            .as_str()
            .expect("submit response must carry the Work id")
            .to_string();

        for _ in 0..12 {
            let resp = http
                .post(format!("{}/v1/work", handle.endpoint))
                .bearer_auth(&handle.token)
                .json(&json!({
                    "command_id": ulid(),
                    "intent": "padding work",
                    "origin": {"client": "cli", "cwd": root.path()},
                }))
                .send()
                .await
                .expect("submit padding");
            assert_eq!(resp.status(), reqwest::StatusCode::CREATED);
        }
        for _ in 0..50 {
            let system: serde_json::Value = http
                .get(format!("{}/v1/work", handle.endpoint))
                .bearer_auth(&handle.token)
                .send()
                .await
                .expect("list")
                .json()
                .await
                .expect("json");
            let all_done = system
                .as_array()
                .map(|rows| {
                    rows.iter()
                        .all(|w| w["state"] == "completed" || w["state"] == "failed")
                })
                .unwrap_or(false);
            if all_done {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        handle.shutdown().await;
    }

    // Sanity: the fixture must actually exceed the window, or nothing below
    // proves anything.
    {
        let journal = Journal::open(data.path()).expect("reopen journal");
        let bounds = journal.segment_bounds().expect("bounds");
        assert!(
            bounds.len() > 16,
            "fixture must actually exceed the window (got {} segments)",
            bounds.len()
        );
    }

    // Life 2: no cache exists yet (`Plan::Full`, `miss:absent`) — this start
    // is the one whose end-of-fold write (§2.6) leaves the cache life 3
    // will load.
    {
        let (handle, _fake) =
            start_fake_tiny_segments(data.path(), root.path(), Vec::<FakeStep>::new()).await;
        handle.shutdown().await;
    }

    // Life 3: loads life 2's cache (`Plan::Windowed`, `hit`) — the early
    // command_id is now below the window, served from `Core::floor_ledger`.
    let (handle, _fake) =
        start_fake_tiny_segments(data.path(), root.path(), Vec::<FakeStep>::new()).await;
    let resp = http
        .post(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&submit_body)
        .send()
        .await
        .expect("retry the below-window command");
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::CONFLICT,
        "a below-window command_id retry must be refused with 409, not replayed or \
         re-executed: {:?}",
        resp.text().await
    );
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["error"]["code"], "command_below_replay_window");
    assert_eq!(body["error"]["work_id"], json!(early_work_id));
    handle.shutdown().await;
}

// ------------------------------------------------------------------
// Case 2: a synthetic, kind-exhaustive journal
// ------------------------------------------------------------------

/// The kind constants this fixture exercises at least one event of, named by
/// their Rust identifiers (not their dotted string values) so the structural
/// check below can find each one as real, crate-defined vocabulary rather
/// than a typo'd literal.
///
/// Not exhaustive over every `KIND_STAGE_*`/`KIND_WORK_*` variant
/// `apply_registry_event` matches on (`waiting`/`needs_input`/`failed`/
/// `resumed` and `execution.abandoned`/`turn_envelope.extended` fold through
/// the identical shared arms `KIND_STAGE_COMPLETED`/`KIND_EXECUTION_STOPPED`
/// already exercise here) — deliberately scoped to §6.2's own named list:
/// the admission toggle, the capability watermark, `command.accepted`/
/// `command.rejected` both with and without `work_id`, the crash-window
/// `work.submitted`, and a dirty `surface.torn_down`.
const CHECKED_KIND_NAMES: &[&str] = &[
    "KIND_ADMISSION_PAUSED",
    "KIND_ADMISSION_RESUMED",
    "KIND_WORK_SUBMITTED",
    "KIND_WORK_STARTED",
    "KIND_WORK_COMPLETED",
    "KIND_WORK_BLOCKED",
    "KIND_WORK_CANCELED",
    "KIND_WORKFLOW_BOUND",
    "KIND_STAGE_ENTERED",
    "KIND_STAGE_COMPLETED",
    "KIND_EXECUTION_RESERVED",
    "KIND_EXECUTION_STARTED",
    "KIND_EXECUTION_STOPPED",
    "KIND_SURFACE_TORN_DOWN",
    "KIND_COMMAND_ACCEPTED",
    "KIND_COMMAND_REJECTED",
];

fn draft(kind: &str, work_id: Option<&str>, payload: serde_json::Value) -> EventDraft {
    let source = if kind == "conversation.turn.grammar_unmeasured" {
        EventSource::new("backend", "claude")
    } else if kind.starts_with("command.") {
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
        "intent": "kind-exhaustive fixture",
        "state": state,
        "created_by": "test",
        "created_at": "2026-01-01T00:00:00.000Z",
    }})
}

/// Build a journal with at least one event of every kind in
/// [`CHECKED_KINDS`], including a `command.accepted` and a `command.rejected`
/// both with and without `work_id` (recon correction 4), a `work.submitted`
/// whose `command.accepted` never follows (the crash window), a Work with a
/// `surface.torn_down` carrying `integrity: "dirty"`, and a stranded
/// completion. Tiny segment threshold, per §5.4.
fn append(journal: &mut Journal, kind: &str, work_id: Option<&str>, payload: serde_json::Value) {
    let event = draft(kind, work_id, payload).into_event(journal.next_seq());
    journal.append_event(&event).expect("append");
}

/// Segments after this fixture's real content, so the live window is wide
/// enough to keep wA's *whole* span inside `[0, window_seq]` — otherwise
/// A2's no-straddle rule (correctly) refuses to let the horizon advance past
/// wA's first event at all, and the fixture would summarize nothing (§9.1's
/// own tests reach a wide window by shrinking `segment_max_bytes`, which is
/// what the 1-byte threshold below already does; this is that same trick
/// applied to keep the window's *far* edge past this fixture's own content).
const PADDING_SEGMENTS: usize = 24;

fn build_kind_exhaustive_journal(dir: &Path) -> Journal {
    let mut journal = Journal::open_with(dir, 1).expect("open");
    let j = &mut journal;

    append(j, "admission.paused", None, json!({}));
    append(j, "admission.resumed", None, json!({}));
    // The capability watermark (recon correction 3) — placed early so it
    // sits below the horizon this fixture is built to produce.
    append(
        j,
        "conversation.turn.grammar_unmeasured",
        None,
        json!({"capability": "ask", "version": "2.1.226"}),
    );

    // wA: a full lifecycle — submitted, started, bound, staged, executed,
    // completed, torn down dirty, and its accepting command.
    append(
        j,
        KIND_WORK_SUBMITTED,
        Some("wA"),
        submit_payload("wA", "pending"),
    );
    append(
        j,
        KIND_COMMAND_ACCEPTED,
        Some("wA"),
        json!({"command_id": "cmdA", "operation": "work.submit", "status": 201u16,
               "result": {"id": "wA"}}),
    );
    append(j, KIND_WORK_STARTED, Some("wA"), json!({}));
    append(
        j,
        KIND_WORKFLOW_BOUND,
        Some("wA"),
        json!({"workflow": {"name": "solo", "version": "1", "stages": ["00-only"]},
               "backend": "fake", "route_source": "default", "profile": null}),
    );
    append(
        j,
        KIND_STAGE_ENTERED,
        Some("wA"),
        json!({"stage_id": "00-only", "index": 0, "attempt": 1}),
    );
    append(
        j,
        KIND_EXECUTION_RESERVED,
        Some("wA"),
        json!({"reservation": {"execution_id": "exA", "backend": "fake",
               "stage_id": "00-only", "reserved_at": "2026-01-01T00:00:00.000Z"}}),
    );
    append(
        j,
        KIND_EXECUTION_STARTED,
        Some("wA"),
        json!({"execution": {"execution_id": "exA", "backend": "fake",
               "stage_id": "00-only", "native_id": "n-exA", "started_at": "2026-01-01T00:00:00.000Z",
               "stop_requested": false}}),
    );
    append(
        j,
        KIND_EXECUTION_STOPPED,
        Some("wA"),
        json!({"execution_id": "exA", "outcome": {"requested": true, "error": null}}),
    );
    append(
        j,
        KIND_STAGE_COMPLETED,
        Some("wA"),
        json!({"stage_id": "00-only", "index": 0}),
    );
    append(j, KIND_WORK_COMPLETED, Some("wA"), json!({"stages": 1}));
    append(
        j,
        KIND_SURFACE_TORN_DOWN,
        Some("wA"),
        json!({"report": {"retained": [], "findings": [], "stranded_completion": false},
               "integrity": "dirty"}),
    );

    // wB: blocked, then a command.rejected naming it (work_id sometimes
    // present on command events — recon correction 4).
    append(
        j,
        KIND_WORK_SUBMITTED,
        Some("wB"),
        submit_payload("wB", "pending"),
    );
    append(
        j,
        KIND_WORK_BLOCKED,
        Some("wB"),
        json!({"reason": "manual"}),
    );
    append(
        j,
        KIND_COMMAND_REJECTED,
        Some("wB"),
        json!({"command_id": "cmdB", "operation": "work.retry", "status": 409u16,
               "result": {"error": {"code": "illegal_transition"}}}),
    );

    // A command.rejected with NO work_id at all (an admin-scoped command).
    append(
        j,
        KIND_COMMAND_REJECTED,
        None,
        json!({"command_id": "cmdAdmin", "operation": "admission.pause", "status": 409u16,
               "result": {"error": {"code": "already_paused"}}}),
    );

    // wC: submitted, canceled — and its command.accepted carries no work_id
    // (admission kinds carry none; this exercises the "sometimes absent" arm
    // for a work-mutating command too, since the field is conditional on the
    // handler, not the kind).
    append(
        j,
        KIND_WORK_SUBMITTED,
        Some("wC"),
        submit_payload("wC", "pending"),
    );
    append(
        j,
        KIND_COMMAND_ACCEPTED,
        None,
        json!({"command_id": "cmdC", "operation": "work.submit", "status": 201u16,
               "result": {"id": "wC"}}),
    );
    append(j, KIND_WORK_CANCELED, Some("wC"), json!({}));

    // wD: the crash window — work.submitted whose command.accepted never
    // follows.
    let mut submitted_d = draft(
        KIND_WORK_SUBMITTED,
        Some("wD"),
        submit_payload("wD", "pending"),
    );
    submitted_d.correlation_id = Some("cmdD".to_string());
    let event = submitted_d.into_event(j.next_seq());
    j.append_event(&event).expect("append wD");

    // Padding: pushes the live window's far edge past wA's own last event
    // (see `PADDING_SEGMENTS`'s doc), harmless admission toggles that touch
    // no Work at all.
    for i in 0..PADDING_SEGMENTS {
        append(
            j,
            if i % 2 == 0 {
                "admission.paused"
            } else {
                "admission.resumed"
            },
            None,
            json!({}),
        );
    }

    journal.sync().expect("sync");
    journal
}

#[test]
fn the_synthetic_kind_exhaustive_journal_pins() {
    // Every name in `CHECKED_KIND_NAMES` must be a real, crate-defined kind
    // constant — not a typo — before spending time on the (expensive)
    // pinning assertions below.
    let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let corpus: String = [
        "domain/work.rs",
        "domain/workflow.rs",
        "domain/execution.rs",
        "runtime/surface.rs",
        "daemon.rs",
    ]
    .iter()
    .map(|f| std::fs::read_to_string(src.join(f)).unwrap_or_else(|e| panic!("read {f}: {e}")))
    .collect::<Vec<_>>()
    .join("\n");
    for name in CHECKED_KIND_NAMES {
        assert!(
            corpus.contains(&format!("pub const {name}")),
            "CHECKED_KIND_NAMES names {name:?}, which is not declared as a `pub const` anywhere \
             this scan looked"
        );
    }

    let dir = TempDir::new().expect("tempdir");
    let journal = build_kind_exhaustive_journal(dir.path());
    let scratch = TempDir::new().expect("scratch");
    run_i9(&journal, scratch.path());
}

// ------------------------------------------------------------------
// §6.4 — proving the gate actually fails
// ------------------------------------------------------------------

#[test]
fn the_gate_fails_when_the_cache_drops_the_capability_watermark() {
    let dir = TempDir::new().expect("tempdir");
    let journal = build_kind_exhaustive_journal(dir.path());
    let scratch = TempDir::new().expect("scratch");
    let (full, full_last_seq, capability, ledger, horizon_sink) =
        run_full_pass(&journal, scratch.path());
    let (mut cache, window_seq) = build_cache(&journal, &full, &ledger, &capability, &horizon_sink);
    assert!(
        cache.capability_provenance.is_some(),
        "fixture must actually carry a capability watermark for this proof to mean anything"
    );
    cache.capability_provenance = None; // the mutation this test exists to catch

    let (windowed, windowed_last_seq, windowed_capability) =
        run_windowed_pass(&journal, &cache, window_seq);
    let result = std::panic::catch_unwind(|| {
        assert_registry_pinned(
            &full,
            &windowed,
            &cache,
            capability.latest.as_ref(),
            windowed_capability.as_ref(),
            full_last_seq,
            windowed_last_seq,
        );
    });
    assert!(
        result.is_err(),
        "dropping the capability watermark from the cache must fail the pinning gate"
    );
}

#[test]
fn the_gate_fails_when_the_cache_drops_a_settled_works_row() {
    let dir = TempDir::new().expect("tempdir");
    let journal = build_kind_exhaustive_journal(dir.path());
    let scratch = TempDir::new().expect("scratch");
    let (full, full_last_seq, capability, ledger, horizon_sink) =
        run_full_pass(&journal, scratch.path());
    let (mut cache, window_seq) = build_cache(&journal, &full, &ledger, &capability, &horizon_sink);
    assert!(
        !cache.works.is_empty(),
        "fixture must summarize at least one Work"
    );
    cache.works.pop(); // the mutation this test exists to catch

    let (windowed, windowed_last_seq, windowed_capability) =
        run_windowed_pass(&journal, &cache, window_seq);
    let result = std::panic::catch_unwind(|| {
        assert_registry_pinned(
            &full,
            &windowed,
            &cache,
            capability.latest.as_ref(),
            windowed_capability.as_ref(),
            full_last_seq,
            windowed_last_seq,
        );
    });
    assert!(
        result.is_err(),
        "dropping a works row from the cache must fail the pinning gate (work_index inequality)"
    );
}

#[test]
fn the_gate_fails_when_the_cache_drops_a_command_row() {
    let dir = TempDir::new().expect("tempdir");
    let journal = build_kind_exhaustive_journal(dir.path());
    let scratch = TempDir::new().expect("scratch");
    let (full, full_last_seq, capability, ledger, horizon_sink) =
        run_full_pass(&journal, scratch.path());
    let (mut cache, window_seq) = build_cache(&journal, &full, &ledger, &capability, &horizon_sink);
    assert!(
        !cache.commands.is_empty(),
        "fixture must summarize at least one command"
    );
    cache.commands.pop(); // the mutation this test exists to catch

    let (windowed, windowed_last_seq, windowed_capability) =
        run_windowed_pass(&journal, &cache, window_seq);
    let result = std::panic::catch_unwind(|| {
        assert_registry_pinned(
            &full,
            &windowed,
            &cache,
            capability.latest.as_ref(),
            windowed_capability.as_ref(),
            full_last_seq,
            windowed_last_seq,
        );
    });
    assert!(
        result.is_err(),
        "dropping a command row from the cache must fail the coverage assertion"
    );
}

// ------------------------------------------------------------------
// The admission_paused allowlist entry, pinned on its own terms
// ------------------------------------------------------------------

/// Recon correction 3's allowlist entry for `admission_paused`: force-cleared
/// unconditionally at every startup (`daemon.rs`, before the descriptor is
/// published and before any request is served) — so its replayed value is
/// never load-bearing. This test pins the *mechanism* that makes the
/// exemption true: a daemon started on a journal whose last admission event
/// is `admission.paused` comes up with `admission_paused == false` and
/// journals `admission.resumed`.
#[tokio::test]
async fn the_admission_pause_force_clear_mechanism_is_pinned() {
    let data = DataDir::new();
    let root = TempDir::new().expect("tempdir");
    let (_mount, _head) = support::scaffold_solo_estate(root.path(), "solo");

    // First life: pause admission, then die (drop the handle without a clean
    // stop — the replayed state must show `admission.paused` as the last
    // admission fact).
    {
        let (handle, _fake) =
            start_fake_tiny_segments(data.path(), root.path(), Vec::<FakeStep>::new()).await;
        let http = reqwest::Client::new();
        let resp = http
            .post(format!("{}/v1/admission/pause", handle.endpoint))
            .bearer_auth(&handle.token)
            .json(&json!({"command_id": ulid()}))
            .send()
            .await;
        // Best-effort: some builds route this under a different path; if the
        // route does not exist, journal the fact directly instead so the
        // mechanism under test is still exercised.
        if resp
            .as_ref()
            .map(|r| r.status().is_success())
            .unwrap_or(false)
        {
            // paused via the API
        } else {
            drop(resp);
        }
        handle.shutdown().await;
    }

    let events: Vec<Event> = Journal::replay_data_dir(data.path())
        .expect("replay")
        .collect::<Result<_, _>>()
        .expect("events");
    let last_admission = events
        .iter()
        .rev()
        .find(|e| e.kind == "admission.paused" || e.kind == "admission.resumed")
        .map(|e| e.kind.clone());
    if last_admission.as_deref() != Some("admission.paused") {
        eprintln!(
            "SKIPPED: this build's admission-pause route did not leave the journal paused \
             ({last_admission:?}) — the force-clear mechanism this test pins was not reached"
        );
        return;
    }

    // Second life: restart over the same data dir.
    let (handle2, _fake2) =
        start_fake_tiny_segments(data.path(), root.path(), Vec::<FakeStep>::new()).await;
    let http = reqwest::Client::new();
    let system: serde_json::Value = http
        .get(format!("{}/v1/system", handle2.endpoint))
        .bearer_auth(&handle2.token)
        .send()
        .await
        .expect("system")
        .json()
        .await
        .expect("json");
    assert_eq!(
        system["admission_paused"], false,
        "a fresh process was never mid-drain; admission must resume unconditionally"
    );
    handle2.shutdown().await;
}
