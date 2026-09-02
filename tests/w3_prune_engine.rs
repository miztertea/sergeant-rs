//! W3 end-to-end: a real `DataDir` daemon, driven past its retention cap,
//! actually prunes — §13.1 step 7's suite plus the wave's live acceptance
//! case (Validation Evidence item 5) and the N21 structural scan.
//!
//! Every test here uses a real daemon (`support::DataDir`), never `TempDir`
//! for anything that spawns (§13.4). The blob/mark-scan and journal-level
//! mechanics have their own focused unit tests in `src/runtime/blob.rs`,
//! `src/runtime/journal.rs` and `src/runtime/prune.rs`; this file proves the
//! whole cycle actually runs and actually deletes.

mod support;

use std::path::Path;

use serde_json::json;
use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig, DaemonHandle};
use sergeant_rs::runtime::journal::Journal;
use sergeant_rs::runtime::projection::{Projection, WorkRegistry, work_registry_reducer};

use support::DataDir;

fn ulid() -> String {
    ulid::Ulid::generate().to_string()
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

async fn start_with_retention(
    data_dir: &Path,
    _estate_root: &Path,
    retention: u32,
    script: impl IntoIterator<Item = FakeStep>,
) -> (DaemonHandle, FakeBackend) {
    let fake = FakeBackend::scripted(FAKE_BACKEND_NAME, script);
    let registry = BackendRegistry::new().with(std::sync::Arc::new(fake.clone()));
    let handle = daemon::start_with(
        data_dir,
        DaemonConfig {
            backends: std::sync::Arc::new(registry),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            // Tiny threshold: reaching many segments from a handful of Works
            // without a production-scale journal.
            segment_max_bytes: Some(256),
            // W3: DaemonConfig::retention deliberately bypasses
            // MIN_RETENTION (§1.2) — a test rig, never a manifest.
            retention: Some(retention),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");
    (handle, fake)
}

async fn submit(
    http: &reqwest::Client,
    endpoint: &str,
    token: &str,
    root: &Path,
    command_id: &str,
    intent: &str,
    is_alive: impl Fn() -> bool,
) -> serde_json::Value {
    // Built once, outside the retry closure below: the endpoint dedupes a
    // submit by `command_id` (proven by this file's own N12 assertions on a
    // pruned command_id retry), so retrying this exact body on a transport
    // failure replays the same submission rather than risking a second Work.
    let body = json!({
        "command_id": command_id,
        "intent": intent,
        // D4: the estate this submission addresses. `cwd` stays §13.3
        // recorded evidence, deciding nothing.
        "estate_root": root,
        "origin": {"client": "cli", "cwd": root},
    });
    let resp = support::send_while_alive(
        "submit",
        || {
            http.post(format!("{endpoint}/v1/work"))
                .bearer_auth(token)
                .json(&body)
        },
        is_alive,
    )
    .await;
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::CREATED,
        "{:?}",
        resp.text().await
    );
    resp.json().await.expect("json")
}

async fn wait_until_all_settled(
    http: &reqwest::Client,
    endpoint: &str,
    token: &str,
    is_alive: impl Fn() -> bool,
) {
    let mut last: serde_json::Value = serde_json::Value::Null;
    for _ in 0..200 {
        let system: serde_json::Value = support::send_while_alive(
            "list",
            || http.get(format!("{endpoint}/v1/work")).bearer_auth(token),
            &is_alive,
        )
        .await
        .json()
        .await
        .expect("json");
        let all_done = system["works"]
            .as_array()
            .map(|rows| {
                rows.iter()
                    .all(|w| w["state"] == "completed" || w["state"] == "failed")
            })
            .unwrap_or(false);
        if all_done {
            return;
        }
        last = system;
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    panic!("works never settled; last list: {last}");
}

/// The wave's live acceptance case (§8's Validation Evidence item 5): a
/// `DataDir` daemon with a tiny segment threshold and `retention: Some(4)`,
/// driven past the cap with real Works, restarted — the restart's own
/// startup-triggered prune (§10.3) must actually shrink the journal, move
/// the floor, and leave the pruned Works answerable only by their residue.
///
/// Also exercises, in one fixture:
/// - N2 (`a_work_inside_the_retention_cap_is_never_pruned`'s live sibling):
///   the newest `retention` Works survive.
/// - N12/N13: a pruned command's `command_id` is refused by name over a
///   real HTTP retry, and the Work it created is a durable residue row, not
///   an unknown id.
#[tokio::test]
async fn a_start_on_an_over_cap_journal_prunes_before_serving() {
    let data = DataDir::new();
    let root = TempDir::new().expect("tempdir");
    let (_mount, _head) = support::scaffold_solo_estate(root.path(), "solo");
    write_one_stage_workflow(root.path());

    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .expect("client");

    // §1.2: `DaemonConfig::retention` deliberately bypasses
    // `estate::MIN_RETENTION` — that floor is a *manifest-schema* refusal
    // protecting an operator from a typo in a file, not a runtime bound. A
    // test rig setting `4` here has not typed anything into a manifest.
    const RETENTION: u32 = 4;
    const TOTAL_WORKS: usize = 12;

    let mut command_ids = Vec::new();
    let mut work_ids = Vec::new();

    // Life 1: submit more Works than the cap, all completing. Retention is
    // deliberately huge here — large enough that the *rotation*-triggered
    // tick (§10.4) never fires mid-life, which would prune before this test
    // gets to observe the "before" state at all. Life 2 is the one whose
    // *startup*-triggered prune (§10.3) this test is about.
    {
        let script: Vec<FakeStep> = (0..TOTAL_WORKS).map(|_| FakeStep::complete()).collect();
        let (handle, _fake) =
            start_with_retention(data.path(), root.path(), 1_000_000, script).await;

        for n in 0..TOTAL_WORKS {
            let cmd = ulid();
            let body = submit(
                &http,
                &handle.endpoint,
                &handle.token,
                root.path(),
                &cmd,
                &format!("prune fixture work {n}"),
                || handle.is_alive(),
            )
            .await;
            command_ids.push(cmd);
            work_ids.push(body["work"]["id"].as_str().expect("work id").to_string());
        }
        wait_until_all_settled(&http, &handle.endpoint, &handle.token, || handle.is_alive()).await;
        handle.shutdown().await;
    }

    let (segments_before, floor_before) = {
        let journal = Journal::open(data.path()).expect("reopen journal");
        let bounds = journal.segment_bounds().expect("bounds");
        (bounds.len(), journal.floor_seq().expect("floor_seq"))
    };
    assert_eq!(
        floor_before,
        Some(1),
        "before any prune, the floor is still 1"
    );
    assert!(
        segments_before > 4,
        "fixture must actually span more than a handful of segments \
         (got {segments_before}) or the prune below proves nothing"
    );

    // Life 2: the restart's own startup-triggered prune (§10.3) runs inside
    // `start_with`, before the listener binds — by the time this call
    // returns, the cycle (if any) has already committed and unlinked.
    let (handle2, _fake2) =
        start_with_retention(data.path(), root.path(), RETENTION, Vec::<FakeStep>::new()).await;

    // N12: a pruned command's id is refused by name over a *real* HTTP
    // retry — never re-executed, never a fabricated 200.
    let pruned_command_id = &command_ids[0];
    let resp = support::send_while_alive(
        "retry a pruned command_id",
        || {
            http.post(format!("{}/v1/work", handle2.endpoint))
                .bearer_auth(&handle2.token)
                .json(&json!({
                    "command_id": pruned_command_id,
                    "intent": "must be refused by name, not re-executed",
                    "origin": {"client": "cli", "cwd": root.path()},
                }))
        },
        || handle2.is_alive(),
    )
    .await;
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::CONFLICT,
        "a pruned command_id retry must be refused with 409: {:?}",
        resp.text().await
    );
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["error"]["code"], "command_below_replay_window");
    assert_eq!(body["error"]["work_id"], json!(work_ids[0]));

    // N13 / I-W3-7's live half, updated for W4 §1.2: a pruned Work's own id
    // no longer 404s — it answers the named pruned shape (`state: "pruned"`,
    // `work: null`), Q10's own "show what we have" — never a stale stranded
    // read of evicted-but-still-cached data, and never conflated with an id
    // that never existed at all (`show_work_on_a_never_existing_id_still_404s`
    // in `tests/w4_read_surfaces.rs` is the test that proves the two stay
    // distinguished).
    let resp = support::send_while_alive(
        "show a pruned work",
        || {
            http.get(format!("{}/v1/work/{}", handle2.endpoint, work_ids[0]))
                .bearer_auth(&handle2.token)
        },
        || handle2.is_alive(),
    )
    .await;
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["state"], "pruned");
    assert!(body["work"].is_null());
    assert_eq!(body["id"], work_ids[0]);

    handle2.shutdown().await;

    // Now that nothing holds the journal's exclusive lock: the journal must
    // have visibly shrunk and the floor must have moved.
    let journal = Journal::open(data.path()).expect("reopen journal after prune");
    let bounds = journal.segment_bounds().expect("bounds");
    let floor_after = journal.floor_seq().expect("floor_seq");
    assert!(
        bounds.len() < segments_before,
        "the journal must have fewer segments after the prune (before {segments_before}, after {})",
        bounds.len()
    );
    assert!(
        floor_after.unwrap_or(1) > 1,
        "the floor must have moved above 1 after a real prune, got {floor_after:?}"
    );

    // Rebuild the registry from the (now-pruned) journal and check the
    // residue directly — the ground truth this whole wave exists to keep
    // honest, independent of any cache. `Projection::resumed` (not a fresh
    // `work_registry_projection()` + `catch_up`, which assumes seq 1) is
    // what makes this floor-aware: `replay_from_floor()` starts yielding at
    // the floor, not at 1.
    let mut registry = Projection::resumed(
        WorkRegistry::default(),
        floor_after.expect("a real prune leaves a floor above 1") - 1,
        work_registry_reducer,
    );
    registry
        .catch_up(journal.replay_from_floor().expect("replay_from_floor"))
        .expect("catch up");
    let state = registry.state();

    assert_eq!(
        state.work_index.len(),
        RETENTION as usize,
        "exactly the newest {RETENTION} Works must remain retained"
    );
    assert_eq!(
        state.pruned_works.len(),
        TOTAL_WORKS - RETENTION as usize,
        "every Work past the cap must appear in pruned_works"
    );
    // Disjoint, per §6.2.
    for id in state.work_index.keys() {
        assert!(
            !state.pruned_works.contains_key(id),
            "a Work must never be in both work_index and pruned_works (id {id})"
        );
    }
    // The oldest Works (by submission order) are exactly the pruned ones,
    // and the newest are exactly the retained ones.
    let pruned_count = TOTAL_WORKS - RETENTION as usize;
    for work_id in &work_ids[..pruned_count] {
        assert!(
            state.pruned_works.contains_key(work_id),
            "an older Work ({work_id}) must be in pruned_works"
        );
    }
    for work_id in &work_ids[pruned_count..] {
        assert!(
            state.work_index.contains_key(work_id),
            "a newer Work ({work_id}) must still be retained"
        );
    }

    // I-W3-10 / N11: a pruned journal replays cleanly from the floor, and
    // only the strict, seq-1 primitive still reports `SeqDiscontinuity` —
    // demonstrated, not merely asserted. Reuses the same handle opened
    // above — a second `Journal::open` on the same data dir would contend
    // with itself for the exclusive lock.
    let clean: Result<Vec<_>, _> = journal
        .replay_from_floor()
        .expect("replay_from_floor")
        .collect();
    assert!(
        clean.is_ok(),
        "replay_from_floor must succeed over a pruned journal: {clean:?}"
    );
    let strict: Result<Vec<_>, _> = journal.replay().expect("replay").collect();
    assert!(
        matches!(
            strict,
            Err(sergeant_rs::runtime::journal::JournalError::SeqDiscontinuity { .. })
        ),
        "the strict seq-1 primitive must still report the gap as SeqDiscontinuity: {strict:?}"
    );
}

/// The seam W4 found while building a doctor fixture: the **ordinary next
/// start after any prune**.
///
/// W3's own prune completion deletes the startup cache (`prune::run`'s step
/// 6 — "the next clean start's own write point rebuilds a fresh v2 cache"),
/// and `complete_interrupted` does the same. So the life *after* a pruning
/// life always resolves to `Plan::Full` / `CacheMiss::Absent` over a journal
/// whose floor is no longer 1. That is not a recovery path anyone has to go
/// looking for — it is the third life of every estate that has ever pruned,
/// and the one W3 deliberately routed through "one safe full replay".
///
/// It was not safe: `Plan::seed_registry`'s `Full` arm handed back a
/// seq-1-expecting `Projection::new` while `Plan::replay`'s `Full` arm fed
/// it `Journal::replay_from_floor()`'s floor-seeded events, so the first
/// event of the pass failed `ProjectionError::SeqMismatch { expected: 1,
/// found: <floor> }` and the daemon refused to come up at all.
///
/// Three lives over one `DataDir`, which is the whole point — life 3 is the
/// one that used to crash.
#[tokio::test]
async fn a_start_after_a_prune_with_no_cache_still_serves() {
    let data = DataDir::new();
    let root = TempDir::new().expect("tempdir");
    let (_mount, _head) = support::scaffold_solo_estate(root.path(), "solo");
    write_one_stage_workflow(root.path());

    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .expect("client");

    const RETENTION: u32 = 4;
    const TOTAL_WORKS: usize = 12;

    let mut work_ids = Vec::new();

    // Life 1: fill the journal past the cap, with retention high enough that
    // the rotation-triggered tick (§10.4) never fires mid-life.
    {
        let script: Vec<FakeStep> = (0..TOTAL_WORKS).map(|_| FakeStep::complete()).collect();
        let (handle, _fake) =
            start_with_retention(data.path(), root.path(), 1_000_000, script).await;
        for n in 0..TOTAL_WORKS {
            let body = submit(
                &http,
                &handle.endpoint,
                &handle.token,
                root.path(),
                &ulid(),
                &format!("post-prune restart fixture work {n}"),
                || handle.is_alive(),
            )
            .await;
            work_ids.push(body["work"]["id"].as_str().expect("work id").to_string());
        }
        wait_until_all_settled(&http, &handle.endpoint, &handle.token, || handle.is_alive()).await;
        handle.shutdown().await;
    }

    // Life 2: the startup-triggered prune (§10.3) runs and, on completion,
    // removes the startup cache.
    {
        let (handle, _fake) =
            start_with_retention(data.path(), root.path(), RETENTION, Vec::<FakeStep>::new()).await;
        handle.shutdown().await;
    }

    let cache = data.path().join("projections").join("floor-state.json");
    assert!(
        !cache.exists(),
        "the prune's own completion must have removed the startup cache — \
         without that, life 3 below is not the cache-miss path this test is about"
    );
    let floor_after_prune = {
        let journal = Journal::open(data.path()).expect("reopen journal after the prune");
        journal.floor_seq().expect("floor_seq")
    };
    assert!(
        floor_after_prune.unwrap_or(1) > 1,
        "a real prune must have moved the floor above 1, got {floor_after_prune:?} — \
         without that, life 3 below never exercises the seam"
    );

    // Life 3: no cache, floor > 1 — `Plan::Full` over `replay_from_floor()`.
    // This is the start that used to fail closed with `SeqMismatch`.
    let fake = FakeBackend::scripted(FAKE_BACKEND_NAME, Vec::<FakeStep>::new());
    let registry = BackendRegistry::new().with(std::sync::Arc::new(fake));
    let handle3 = daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: std::sync::Arc::new(registry),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            segment_max_bytes: Some(256),
            retention: Some(RETENTION),
            ..DaemonConfig::default()
        },
    )
    .await
    .unwrap_or_else(|e| {
        panic!(
            "a start on an already-pruned journal with no cache must come up \
             (floor {floor_after_prune:?}): {e}"
        )
    });

    // ... and must actually *serve*, folded from the floor with no cache to
    // lean on: the retained Works are all there, and the pruned ones still
    // answer by name from residue the pass re-folded out of the surviving
    // `prune.completed`.
    let system: serde_json::Value = support::send_while_alive(
        "list works",
        || {
            http.get(format!("{}/v1/work", handle3.endpoint))
                .bearer_auth(&handle3.token)
        },
        || handle3.is_alive(),
    )
    .await
    .json()
    .await
    .expect("json");
    let listed = system["works"].as_array().expect("works array");
    assert_eq!(
        listed.len(),
        RETENTION as usize,
        "life 3 must serve exactly the retained Works: {system}"
    );

    let pruned_work_id = &work_ids[0];
    let resp = support::send_while_alive(
        "show a pruned work in life 3",
        || {
            http.get(format!("{}/v1/work/{}", handle3.endpoint, pruned_work_id))
                .bearer_auth(&handle3.token)
        },
        || handle3.is_alive(),
    )
    .await;
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(
        body["state"], "pruned",
        "a cache-less full replay must still re-fold the prune residue: {body}"
    );

    handle3.shutdown().await;
}

/// §10.4's own path, distinct from the sibling test above (which is
/// deliberately built to avoid it, per its own comment, so it can observe
/// the "before" state undisturbed): a daemon that is never restarted still
/// prunes, live, once a segment rotation crosses the cap and at least
/// `PRUNE_BATCH_MIN_SEGMENTS` whole segments are eligible —
/// `maybe_run_rotation_triggered_prune` (§10.4) is exercised by no other
/// test in this suite. Polls `GET /v1/doctor`'s lock-free journal check
/// (never a second `Journal::open` against a data dir the daemon still
/// holds) so this can observe the floor moving without ever shutting the
/// daemon down.
#[tokio::test]
async fn a_rotation_crossing_the_cap_arms_a_prune_within_one_tick() {
    let data = DataDir::new();
    let root = TempDir::new().expect("tempdir");
    let (_mount, _head) = support::scaffold_solo_estate(root.path(), "solo");
    write_one_stage_workflow(root.path());

    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .expect("client");

    const RETENTION: u32 = 3;
    const TOTAL_WORKS: usize = 12;

    let script: Vec<FakeStep> = (0..TOTAL_WORKS).map(|_| FakeStep::complete()).collect();
    let (handle, _fake) = start_with_retention(data.path(), root.path(), RETENTION, script).await;

    let mut command_ids = Vec::new();
    for n in 0..TOTAL_WORKS {
        let cmd = ulid();
        submit(
            &http,
            &handle.endpoint,
            &handle.token,
            root.path(),
            &cmd,
            &format!("rotation-triggered prune fixture {n}"),
            || handle.is_alive(),
        )
        .await;
        command_ids.push(cmd);
    }
    wait_until_all_settled(&http, &handle.endpoint, &handle.token, || handle.is_alive()).await;

    // No restart anywhere in this test: poll the lock-free doctor journal
    // check until the floor has actually moved above 1 — proof the
    // *rotation*-triggered path pruned live, within a handful of the
    // driver's own 200 ms ticks, never waiting on a restart to do it.
    let mut floor_seq = 1u64;
    for _ in 0..100 {
        let report: serde_json::Value = support::send_while_alive(
            "doctor",
            || {
                http.get(format!("{}/v1/doctor", handle.endpoint))
                    .bearer_auth(&handle.token)
            },
            || handle.is_alive(),
        )
        .await
        .json()
        .await
        .expect("json");
        let detail = report["checks"]
            .as_array()
            .expect("checks array")
            .iter()
            .find(|c| c["name"] == "journal")
            .expect("a journal check")["detail"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        if let Some(idx) = detail.find("from seq ") {
            let digits: String = detail[idx + "from seq ".len()..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(seq) = digits.parse::<u64>() {
                floor_seq = seq;
                if floor_seq > 1 {
                    break;
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    assert!(
        floor_seq > 1,
        "the rotation-triggered prune (§10.4) must have moved the floor within a handful \
         of ticks with no restart — the floor is still {floor_seq}"
    );

    // The oldest command must already be refused by name, live, before any
    // shutdown — the same N12 proof as the startup-triggered sibling test,
    // here demonstrating the rotation-triggered path reached the same
    // observable state without one.
    let resp = support::send_while_alive(
        "retry a pruned command_id",
        || {
            http.post(format!("{}/v1/work", handle.endpoint))
                .bearer_auth(&handle.token)
                .json(&json!({
                    "command_id": command_ids[0],
                    "intent": "must be refused by name, live, no restart",
                    "origin": {"client": "cli", "cwd": root.path()},
                }))
        },
        || handle.is_alive(),
    )
    .await;
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::CONFLICT,
        "a pruned command_id retry must be refused with 409, live: {:?}",
        resp.text().await
    );

    handle.shutdown().await;
}

// ------------------------------------------------------------------
// N21: no configuration or flag can lower the prune predicate (Q7/I-W3-11)
// ------------------------------------------------------------------

/// Structural source scan, the shape `tests/m9_watch.rs` and
/// `tests/a4_blob_ref_pinning.rs` already use: `src/runtime/prune.rs` must
/// name no `clap` argument, no `std::env::var`, and no `DaemonConfig` field
/// other than `retention` and `segment_max_bytes` — the declared policy
/// (`[estate] retention` / `DaemonConfig::retention`, test rigs only) is the
/// whole authorization surface, structurally, not by convention.
#[test]
fn no_configuration_or_flag_can_lower_the_prune_predicate() {
    let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/runtime/prune.rs");
    let text = std::fs::read_to_string(&src).expect("read prune.rs");

    assert!(
        !text.contains("clap"),
        "prune.rs must declare no CLI flag of its own"
    );
    assert!(
        !text.contains("std::env::var") && !text.contains("env::var"),
        "prune.rs must read no environment variable"
    );

    // Every `DaemonConfig` field this module names must be one of the two
    // sanctioned ones.
    let sanctioned = ["retention", "segment_max_bytes"];
    for line in text.lines() {
        let Some(idx) = line.find("config.") else {
            continue;
        };
        let rest = &line[idx + "config.".len()..];
        let field: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if field.is_empty() {
            continue;
        }
        assert!(
            sanctioned.contains(&field.as_str()),
            "prune.rs names DaemonConfig field {field:?}, which is not in the sanctioned set \
             {sanctioned:?} — I-W3-11 requires the declared policy to be the whole \
             authorization surface"
        );
    }
}

// ------------------------------------------------------------------
// N23: a prune runs only under the core guard (§10.1)
// ------------------------------------------------------------------

/// The end of the `{ … }` block that starts at or after `from` (mirrors
/// `tests/m6_surfaces.rs`'s own helper of the same name).
fn block_end(source: &str, from: usize) -> usize {
    let open = source[from..].find('{').expect("a block") + from;
    let mut depth = 0usize;
    for (offset, c) in source[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return open + offset;
                }
            }
            _ => {}
        }
    }
    source.len()
}

/// The smallest `{ … }` block that textually contains `at` — the backward
/// counterpart to `block_end`, used here to find "the block this
/// `CoreGuard::acquire` call's binding is scoped to".
fn enclosing_block(source: &str, at: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut depth: i64 = 0;
    let mut i = at;
    while i > 0 {
        i -= 1;
        match bytes[i] {
            b'}' => depth += 1,
            b'{' => {
                if depth == 0 {
                    return (i, block_end(source, i));
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    (0, source.len())
}

/// t11c (`tests/m6_surfaces.rs`) proves every core-lock hold goes through
/// `CoreGuard`. This is the narrower, prune-specific proof §10.1 and the
/// Negative Test Matrix (N23) separately require: that `prune::run` — which
/// takes `&mut Core`, not a `CoreGuard`, so nothing about its own type stops
/// a future caller from handing it a `Core` reached some other way — is only
/// ever called from a lexical scope that also acquired the guard. Holding
/// the guard for the whole cycle is what makes "intent → deletion →
/// completion" atomic with respect to appends: no event can land between
/// the intent and the completion.
///
/// Structural, in the `t11c`/N21 style: every occurrence of
/// `crate::runtime::prune::run(` (the live-cycle entry point — deliberately
/// not `run_startup`/`complete_interrupted`, which run during daemon start
/// before any listener binds or any `CoreGuard` is needed at all, per §10.3)
/// in `src/api.rs` must fall inside the same `{ … }` block as a preceding
/// `CoreGuard::acquire(` call. A future call site reached without first
/// acquiring the guard in the same block fails here with nothing to catch
/// it structurally otherwise, since `prune::run`'s own signature cannot
/// enforce it.
#[test]
fn prune_runs_only_under_the_core_guard() {
    let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/api.rs");
    let text = std::fs::read_to_string(&src).expect("read api.rs");

    let guarded_ranges: Vec<(usize, usize)> = text
        .match_indices("CoreGuard::acquire(")
        .map(|(index, _)| enclosing_block(&text, index))
        .collect();
    assert!(
        !guarded_ranges.is_empty(),
        "api.rs must declare at least one `CoreGuard::acquire` call for this test to mean anything"
    );

    let mut checked_a_call_site = false;
    for (index, _) in text.match_indices("crate::runtime::prune::run(") {
        checked_a_call_site = true;
        assert!(
            guarded_ranges
                .iter()
                .any(|(start, end)| index > *start && index < *end),
            "a `prune::run` call at byte {index} in api.rs is not lexically inside a block \
             that also acquired `CoreGuard` — N23/§10.1 requires the whole cycle to run under \
             the guard, near: {:?}",
            &text[index.saturating_sub(160)..index]
        );
    }
    assert!(
        checked_a_call_site,
        "api.rs must call `crate::runtime::prune::run` at least once for this test to mean \
         anything — if the call site moved or was renamed, update this scan to find it"
    );
}
