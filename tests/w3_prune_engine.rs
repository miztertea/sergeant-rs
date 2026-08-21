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
    estate_root: &Path,
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
            estate_root: Some(estate_root.to_path_buf()),
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
) -> serde_json::Value {
    let resp = http
        .post(format!("{endpoint}/v1/work"))
        .bearer_auth(token)
        .json(&json!({
            "command_id": command_id,
            "intent": intent,
            "origin": {"client": "cli", "cwd": root},
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
    resp.json().await.expect("json")
}

async fn wait_until_all_settled(http: &reqwest::Client, endpoint: &str, token: &str) {
    let mut last: serde_json::Value = serde_json::Value::Null;
    for _ in 0..200 {
        let system: serde_json::Value = http
            .get(format!("{endpoint}/v1/work"))
            .bearer_auth(token)
            .send()
            .await
            .expect("list")
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
            )
            .await;
            command_ids.push(cmd);
            work_ids.push(body["work"]["id"].as_str().expect("work id").to_string());
        }
        wait_until_all_settled(&http, &handle.endpoint, &handle.token).await;
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
    let resp = http
        .post(format!("{}/v1/work", handle2.endpoint))
        .bearer_auth(&handle2.token)
        .json(&json!({
            "command_id": pruned_command_id,
            "intent": "must be refused by name, not re-executed",
            "origin": {"client": "cli", "cwd": root.path()},
        }))
        .send()
        .await
        .expect("retry a pruned command_id");
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::CONFLICT,
        "a pruned command_id retry must be refused with 409: {:?}",
        resp.text().await
    );
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["error"]["code"], "command_below_replay_window");
    assert_eq!(body["error"]["work_id"], json!(work_ids[0]));

    // N13 / I-W3-7's live half: a pruned Work's own id 404s over HTTP (W3
    // ships no pruned-by-name response shape — W4 owns that surface — but
    // it must be the *same* honest 404 an unknown id gets, never a stale
    // stranded read of evicted-but-still-cached data).
    let resp = http
        .get(format!("{}/v1/work/{}", handle2.endpoint, work_ids[0]))
        .bearer_auth(&handle2.token)
        .send()
        .await
        .expect("show a pruned work");
    assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);

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
