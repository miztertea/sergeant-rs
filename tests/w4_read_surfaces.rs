//! W4 §1: read-path surfaces (Q10, "show what we have").
//!
//! - §1.1: `floor_seq` on `GET /v1/events` and the SSE stream's floor
//!   control frame, plus the pre-existing silent-close gap's fix (§1.1.3).
//! - §1.2: `work show` on a pruned Work answers by name instead of a bare
//!   404 indistinguishable from a never-existed id.
//! - §1.3: `work transcript`'s pruned answer and its bounded replay.
//!
//! Every daemon here is in-process (`daemon::start`/`start_with`), the same
//! shape `tests/m2_daemon_api.rs` and `tests/w3_prune_engine.rs` already use
//! — nothing here spawns the `sgt` binary, so no `support::DataDir` guard is
//! needed (an in-process daemon dies with the test process; it is never
//! detached).

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::sync::broadcast;

use sergeant_rs::api::Core;
use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig, DaemonHandle};
use sergeant_rs::runtime::journal::Journal;
use sergeant_rs::runtime::projection::work_registry_projection;

mod support;

fn ulid() -> String {
    ulid::Ulid::generate().to_string()
}

/// A one-stage workflow every fixture here binds work to — the same shape
/// `tests/w3_prune_engine.rs::write_one_stage_workflow` uses (a real
/// estate/workflow is needed for a submitted Work to actually reach
/// `completed` through `FakeBackend`; a bare `daemon::start` with no
/// workflow leaves it `pending` forever).
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

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("client")
}

async fn start(dir: &Path) -> DaemonHandle {
    daemon::start(dir).await.expect("daemon start")
}

/// A daemon over a scripted `FakeBackend`, with a tiny segment threshold and
/// an explicit retention — the same shape
/// `tests/w3_prune_engine.rs::start_with_retention` uses.
async fn start_with_retention(
    data_dir: &Path,
    estate_root: &Path,
    retention: u32,
    script: impl IntoIterator<Item = FakeStep>,
) -> DaemonHandle {
    let fake = FakeBackend::scripted(FAKE_BACKEND_NAME, script);
    let registry = BackendRegistry::new().with(Arc::new(fake));
    daemon::start_with(
        data_dir,
        DaemonConfig {
            backends: Arc::new(registry),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            estate_root: Some(estate_root.to_path_buf()),
            segment_max_bytes: Some(256),
            retention: Some(retention),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start")
}

async fn submit(
    http: &reqwest::Client,
    handle: &DaemonHandle,
    root: &Path,
    command_id: &str,
    intent: &str,
) -> Value {
    let resp = http
        .post(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
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

/// A bare submission with no `origin` — for the plain, no-estate,
/// never-settled fixtures (`daemon::start`) that only need *a* journaled
/// event to exist, never a completed Work.
async fn submit_simple(
    http: &reqwest::Client,
    handle: &DaemonHandle,
    command_id: &str,
    intent: &str,
) -> Value {
    let resp = http
        .post(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&json!({"command_id": command_id, "intent": intent}))
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

async fn wait_until_all_settled(http: &reqwest::Client, handle: &DaemonHandle) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        let system: Value = http
            .get(format!("{}/v1/work", handle.endpoint))
            .bearer_auth(&handle.token)
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
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("works never settled");
}

/// Build a journal over the cap so a restart's startup-triggered prune (W3
/// §10.3) actually runs, and return `(handle, pruned_work_id, retained_work_id, retention)`.
/// Mirrors `tests/w3_prune_engine.rs::a_start_on_an_over_cap_journal_prunes_before_serving`'s
/// fixture shape. `data`/`root` outlive the returned handle.
async fn pruned_fixture(data: &TempDir, root: &TempDir) -> (DaemonHandle, String, String, u32) {
    const RETENTION: u32 = 2;
    const TOTAL_WORKS: usize = 6;
    support::scaffold_solo_estate(root.path(), "solo");
    write_one_stage_workflow(root.path());
    let http = client();
    let mut work_ids = Vec::new();
    {
        // Huge retention for life 1, so nothing prunes before this test can
        // observe the "before" state — the rotation-triggered tick must
        // never fire mid-life either, so retention is deliberately enormous.
        let script: Vec<FakeStep> = (0..TOTAL_WORKS).map(|_| FakeStep::complete()).collect();
        let handle = start_with_retention(data.path(), root.path(), 1_000_000, script).await;
        for n in 0..TOTAL_WORKS {
            let cmd = ulid();
            let body = submit(
                &http,
                &handle,
                root.path(),
                &cmd,
                &format!("prune fixture work {n}"),
            )
            .await;
            work_ids.push(body["work"]["id"].as_str().expect("work id").to_string());
        }
        wait_until_all_settled(&http, &handle).await;
        handle.shutdown().await;
    }
    // Life 2: the restart's own startup-triggered prune runs before the
    // listener binds.
    let handle =
        start_with_retention(data.path(), root.path(), RETENTION, Vec::<FakeStep>::new()).await;
    let pruned = work_ids[0].clone();
    let retained = work_ids[TOTAL_WORKS - 1].clone();
    (handle, pruned, retained, RETENTION)
}

/// One raw SSE frame — captured whether or not it carries an `id:` line, so
/// a control frame (which never does, §1.1.2) is not silently dropped the
/// way `tests/m2_daemon_api.rs`'s own `read_sse_events` (which requires all
/// three of id/event/data) would drop it.
#[derive(Debug, Clone)]
struct RawFrame {
    id: Option<u64>,
    event: String,
    data: Value,
}

async fn read_raw_sse_frames(resp: &mut reqwest::Response, count: usize) -> Vec<RawFrame> {
    let mut buffer = String::new();
    let mut frames = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    while frames.len() < count && Instant::now() < deadline {
        let chunk = match tokio::time::timeout(Duration::from_secs(10), resp.chunk()).await {
            Ok(Ok(Some(c))) => c,
            Ok(Ok(None)) => break,
            Ok(Err(e)) => panic!("sse chunk io: {e}"),
            Err(_) => panic!("sse chunk timeout"),
        };
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(pos) = buffer.find("\n\n") {
            let frame = buffer[..pos].to_string();
            buffer.drain(..pos + 2);
            let mut id = None;
            let mut event = None;
            let mut data = None;
            for line in frame.lines() {
                if let Some(v) = line.strip_prefix("id:") {
                    id = v.trim().parse::<u64>().ok();
                } else if let Some(v) = line.strip_prefix("event:") {
                    event = Some(v.trim().to_string());
                } else if let Some(v) = line.strip_prefix("data:") {
                    data = serde_json::from_str::<Value>(v.trim()).ok();
                }
            }
            if let (Some(event), Some(data)) = (event, data) {
                frames.push(RawFrame { id, event, data });
            }
        }
    }
    frames
}

/// Read every remaining chunk until the stream closes or a deadline passes.
/// Returns whether the stream actually closed (vs timed out still open).
async fn drain_until_closed(resp: &mut reqwest::Response, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if Instant::now() >= deadline {
            return false;
        }
        match tokio::time::timeout(Duration::from_secs(2), resp.chunk()).await {
            Ok(Ok(Some(_))) => continue,
            Ok(Ok(None)) => return true,
            Ok(Err(_)) => return true,
            Err(_) => continue,
        }
    }
}

// --------------------------------------------------------- §1.1: floor_seq

#[tokio::test]
async fn event_history_response_carries_floor_seq_and_is_one_on_an_unpruned_journal() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();
    submit_simple(&http, &handle, &ulid(), "hello").await;

    let body: Value = http
        .get(format!("{}/v1/events", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("events")
        .json()
        .await
        .expect("json");
    assert_eq!(
        body["floor_seq"], 1,
        "an unpruned journal's floor is 1, unconditionally present: {body}"
    );

    handle.shutdown().await;
}

#[tokio::test]
async fn event_history_below_the_floor_is_served_from_the_floor_with_floor_seq_set() {
    let data = TempDir::new().expect("tempdir");
    let root = TempDir::new().expect("tempdir");
    let (handle, _pruned, _retained, _retention) = pruned_fixture(&data, &root).await;
    let http = client();

    let resp = http
        .get(format!("{}/v1/events?from=0", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("events from=0 on a pruned journal");
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::OK,
        "a from= below the floor is not an error (A1 clamp)"
    );
    let body: Value = resp.json().await.expect("json");
    let floor_seq = body["floor_seq"].as_u64().expect("floor_seq present");
    assert!(
        floor_seq > 1,
        "a real prune must have moved the floor above 1"
    );
    let events = body["events"].as_array().expect("events array");
    let min_seq = events
        .iter()
        .map(|e| e["seq"].as_u64().expect("seq"))
        .min()
        .expect("at least one event survives past the floor");
    assert_eq!(
        min_seq, floor_seq,
        "the response is served from exactly the floor, never from 1"
    );

    handle.shutdown().await;
}

#[tokio::test]
async fn sse_stream_sends_a_floor_frame_before_history_on_every_connection() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();
    submit_simple(&http, &handle, &ulid(), "hello").await;

    let mut stream = http
        .get(format!("{}/v1/events/stream?from=0", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("sse connect");
    assert_eq!(stream.status(), 200);

    let frames = read_raw_sse_frames(&mut stream, 1).await;
    assert_eq!(
        frames.len(),
        1,
        "expected at least the floor frame: {frames:?}"
    );
    assert_eq!(frames[0].event, "sergeant.floor");
    assert_eq!(frames[0].data["floor_seq"], 1);

    handle.shutdown().await;
}

#[tokio::test]
async fn sse_floor_frame_never_sets_an_sse_id() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();
    submit_simple(&http, &handle, &ulid(), "hello").await;

    let mut stream = http
        .get(format!("{}/v1/events/stream?from=0", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("sse connect");
    let frames = read_raw_sse_frames(&mut stream, 1).await;
    assert_eq!(
        frames[0].id, None,
        "the floor frame must never carry an id: line — it must not poison Last-Event-ID"
    );

    handle.shutdown().await;
}

#[tokio::test]
async fn sse_floor_frame_carries_the_actual_pruned_floor_on_a_pruned_journal() {
    let data = TempDir::new().expect("tempdir");
    let root = TempDir::new().expect("tempdir");
    let (handle, _pruned, _retained, _retention) = pruned_fixture(&data, &root).await;
    let http = client();

    // Ground truth via the already-independently-tested `GET /v1/events`
    // floor_seq — the daemon still holds the journal's exclusive lock, so a
    // second `Journal::open` here would contend with it (`w3_prune_engine.rs`'s
    // own tests only ever reopen the journal *after* `handle.shutdown()`).
    let expected_floor = http
        .get(format!("{}/v1/events", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("events")
        .json::<Value>()
        .await
        .expect("json")["floor_seq"]
        .as_u64()
        .expect("floor_seq");
    assert!(expected_floor > 1);

    let mut stream = http
        .get(format!("{}/v1/events/stream?from=0", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("sse connect");
    let frames = read_raw_sse_frames(&mut stream, 1).await;
    assert_eq!(frames[0].event, "sergeant.floor");
    assert_eq!(frames[0].data["floor_seq"], expected_floor);

    handle.shutdown().await;
}

/// §1.1.3 (A6): a journal failure mid-replay must close the stream with a
/// named error frame, not silently — the same fault-injection seam
/// `tests/m2_daemon_api.rs::sse_history_replay_error_mid_tail_closes_the_stream_without_daemon_damage`
/// uses (a malformed line appended straight to the segment file).
#[tokio::test]
async fn sse_stream_sends_an_error_frame_before_closing_on_a_journal_failure() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();
    submit_simple(&http, &handle, &ulid(), "before the corruption").await;

    let journal_dir = dir.path().join("journal");
    let mut segments: Vec<_> = std::fs::read_dir(&journal_dir)
        .expect("journal dir")
        .filter_map(|entry| {
            let path = entry.expect("entry").path();
            (path.extension().is_some_and(|ext| ext == "ndjson")).then_some(path)
        })
        .collect();
    segments.sort();
    let segment = segments.last().expect("a segment exists").clone();
    let mut text = std::fs::read_to_string(&segment).expect("read segment");
    text.push_str("{ \"not\": \"an event\" }\n");
    std::fs::write(&segment, text).expect("append malformed line");

    let mut stream = http
        .get(format!("{}/v1/events/stream?from=0", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("sse connect");
    assert_eq!(stream.status(), 200, "the handshake itself still succeeds");

    // The floor frame still arrives (computed from segment listing alone,
    // unaffected by a malformed line deeper in the tail); the error frame
    // follows once history replay actually hits the corruption.
    let frames = read_raw_sse_frames(&mut stream, 2).await;
    assert_eq!(
        frames[0].event, "sergeant.floor",
        "floor frame still arrives first: {frames:?}"
    );
    assert_eq!(
        frames[1].event, "sergeant.stream_error",
        "the journal failure must be named, not silent: {frames:?}"
    );
    assert!(
        frames[1].data["error"]
            .as_str()
            .is_some_and(|s| !s.is_empty()),
        "the error frame must carry a non-empty error description: {frames:?}"
    );
    assert_eq!(frames[1].id, None, "a control frame never sets id:");

    let closed = drain_until_closed(&mut stream, Duration::from_secs(5)).await;
    assert!(
        closed,
        "the stream must close after the error frame, not hang open"
    );

    // The daemon itself is undamaged.
    let health = http
        .get(format!("{}/healthz", handle.endpoint))
        .send()
        .await
        .expect("healthz after corrupted replay");
    assert_eq!(health.status(), 200);

    handle.shutdown().await;
}

// ------------------------------------------------- §1.2: `work show` pruned

#[tokio::test]
async fn show_work_on_a_pruned_id_answers_pruned_with_its_date_and_policy() {
    let data = TempDir::new().expect("tempdir");
    let root = TempDir::new().expect("tempdir");
    let (handle, pruned, _retained, retention) = pruned_fixture(&data, &root).await;
    let http = client();

    let resp = http
        .get(format!("{}/v1/work/{pruned}", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("show pruned work");
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::OK,
        "a pruned id is never a 404"
    );
    let body: Value = resp.json().await.expect("json");
    assert_eq!(body["state"], "pruned");
    assert!(body["work"].is_null());
    assert_eq!(body["id"], pruned);
    assert!(
        body["pruned_at"].as_str().is_some_and(|s| !s.is_empty()),
        "pruned_at must be a real timestamp: {body}"
    );
    assert_eq!(
        body["policy"]["retention"], retention,
        "the reported policy must match this daemon's configured cap: {body}"
    );

    handle.shutdown().await;
}

#[tokio::test]
async fn show_work_on_a_never_existing_id_still_404s() {
    let data = TempDir::new().expect("tempdir");
    let root = TempDir::new().expect("tempdir");
    let (handle, _pruned, _retained, _retention) = pruned_fixture(&data, &root).await;
    let http = client();

    let never_existed = ulid();
    let resp = http
        .get(format!("{}/v1/work/{never_existed}", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("show unknown work");
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::NOT_FOUND,
        "an id in neither work_index nor pruned_works must still 404 — proving \
         the two cases are actually distinguished, not merely that the pruned arm exists"
    );

    handle.shutdown().await;
}

// ------------------------------------------------------- #221: graph pruned

/// #221: the graph surface is a named read surface too. Once pruning has
/// removed the Work's graph events, it must still answer from the same
/// residue row as `work show`, rather than silently treating the id as one
/// that never existed.
#[tokio::test]
async fn work_graph_on_a_pruned_id_answers_pruned_with_its_date_and_policy() {
    let data = TempDir::new().expect("tempdir");
    let root = TempDir::new().expect("tempdir");
    let (handle, pruned, _retained, retention) = pruned_fixture(&data, &root).await;
    let http = client();

    let resp = http
        .get(format!("{}/v1/graph/work/{pruned}", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("graph pruned work");
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::OK,
        "a pruned graph id is never indistinguishable from a 404"
    );
    let body: Value = resp.json().await.expect("json");
    assert_eq!(body["state"], "pruned");
    assert!(body["work"].is_null());
    assert_eq!(body["id"], pruned);
    assert!(
        body["pruned_at"].as_str().is_some_and(|s| !s.is_empty()),
        "pruned_at must be a real timestamp: {body}"
    );
    assert_eq!(
        body["policy"]["retention"], retention,
        "the reported policy must match this daemon's configured cap: {body}"
    );

    handle.shutdown().await;
}

#[tokio::test]
async fn work_graph_on_a_never_existing_id_still_404s() {
    let data = TempDir::new().expect("tempdir");
    let root = TempDir::new().expect("tempdir");
    let (handle, _pruned, _retained, _retention) = pruned_fixture(&data, &root).await;
    let http = client();

    let never_existed = ulid();
    let resp = http
        .get(format!("{}/v1/graph/work/{never_existed}", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("graph unknown work");
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::NOT_FOUND,
        "an id in neither work_index nor pruned_works must remain a graph 404"
    );

    handle.shutdown().await;
}

// -------------------------------------------- §1.3: `work transcript` pruned

#[tokio::test]
async fn work_transcript_on_a_pruned_work_answers_pruned_with_empty_turns() {
    let data = TempDir::new().expect("tempdir");
    let root = TempDir::new().expect("tempdir");
    let (handle, pruned, _retained, _retention) = pruned_fixture(&data, &root).await;
    let http = client();

    let resp = http
        .get(format!("{}/v1/work/{pruned}/transcript", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("transcript of a pruned work");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body: Value = resp.json().await.expect("json");
    assert_eq!(body["state"], "pruned");
    assert_eq!(body["turns"], json!([]));
    assert_eq!(body["work_id"], pruned);

    handle.shutdown().await;
}

#[tokio::test]
async fn work_transcript_on_a_retained_work_survives_a_prune() {
    let data = TempDir::new().expect("tempdir");
    let root = TempDir::new().expect("tempdir");
    let (handle, _pruned, retained, _retention) = pruned_fixture(&data, &root).await;
    let http = client();

    let resp = http
        .get(format!("{}/v1/work/{retained}/transcript", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("transcript of a retained work after a prune");
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::OK,
        "a retained Work's transcript must survive a prune of older Works"
    );
    let body: Value = resp.json().await.expect("json");
    assert_eq!(
        body["work_id"], retained,
        "this must be the ordinary (not pruned) transcript shape: {body}"
    );
    assert!(
        body["turns"].is_array(),
        "the ordinary transcript shape carries a turns array, never the pruned shape: {body}"
    );

    handle.shutdown().await;
}

/// §1.3: the replay `work_transcript` performs is bounded to a Work's own
/// `first_seq_by_work` entry, not to the floor — proven directly against
/// `Core::events_after`, the exact primitive the handler calls, rather than
/// by instrumenting I/O counts (implementation-fragile) or by inference from
/// HTTP content alone (the handler's own `work_id` filter would make the
/// response identical whether or not the bound were actually honored, so the
/// HTTP surface alone cannot distinguish a correct bound from `from=0`).
///
/// Fixture: many old, unrelated Works, real-submitted and completed through
/// a live daemon (so the journal shape is exactly what production produces),
/// followed by one recent Work. After shutdown, a fresh `Core` is assembled
/// over the same journal via the crate's fully public constructors (the
/// same recipe `src/api.rs`'s own `test_state` test helper uses) — proving
/// nothing here needs the private daemon internals, only `sergeant_rs::api`'s
/// public surface.
#[tokio::test]
async fn work_transcript_replay_is_bounded_to_the_works_own_first_seq() {
    let dir = TempDir::new().expect("tempdir");
    let root = TempDir::new().expect("tempdir");
    support::scaffold_solo_estate(root.path(), "solo");
    write_one_stage_workflow(root.path());
    let http = client();
    let mut old_ids = Vec::new();
    let recent_id;
    {
        const OLD_WORKS: usize = 8;
        let script: Vec<FakeStep> = (0..OLD_WORKS + 1).map(|_| FakeStep::complete()).collect();
        let handle = start_with_retention(dir.path(), root.path(), 1_000_000, script).await;
        for n in 0..OLD_WORKS {
            let body = submit(
                &http,
                &handle,
                root.path(),
                &ulid(),
                &format!("old junk {n}"),
            )
            .await;
            old_ids.push(body["work"]["id"].as_str().expect("id").to_string());
        }
        let body = submit(&http, &handle, root.path(), &ulid(), "the recent one").await;
        recent_id = body["work"]["id"].as_str().expect("id").to_string();
        wait_until_all_settled(&http, &handle).await;
        handle.shutdown().await;
    }

    // Ground truth: the recent Work's own first event's seq, read straight
    // off the journal — exactly what a correctly-populated
    // `first_seq_by_work` would hold for it.
    let journal = Journal::open(dir.path()).expect("open journal");
    let recent_first_seq = journal
        .replay()
        .expect("replay")
        .map(|e| e.expect("event"))
        .find(|e| e.work_id.as_deref() == Some(recent_id.as_str()))
        .expect("the recent Work has at least one event")
        .seq;
    assert!(
        recent_first_seq > 1,
        "the recent Work must not be the very first event, or this fixture proves nothing"
    );

    // Reassemble a `Core` the same way `test_state` does in `src/api.rs`'s
    // own unit tests — every constructor used here is `pub`.
    let mut registry = work_registry_projection();
    registry
        .catch_up(journal.replay().expect("replay"))
        .expect("catch up registry");
    let (events_tx, _) = broadcast::channel(16);
    let mut core = Core::new(journal, registry, events_tx);
    // `first_seq_by_work` is populated incrementally by a *live* daemon
    // (`Core::commit`), never by a bare journal replay — set it here to the
    // ground truth above, exactly what the real daemon would have held.
    core.first_seq_by_work
        .insert(recent_id.clone(), recent_first_seq);

    // The exact bound `work_transcript` computes.
    let from = recent_first_seq.saturating_sub(1);
    let bounded = core.events_after(from).expect("events_after(from)");
    let bounded_min = bounded.iter().map(|e| e.seq).min().expect("non-empty");
    assert_eq!(
        bounded_min, recent_first_seq,
        "events_after(first_seq - 1) must start exactly at the recent Work's own first event"
    );

    let unbounded = core.events_after(0).expect("events_after(0)");
    assert!(
        unbounded.len() > bounded.len(),
        "the bound must actually be narrower than a full replay, or this fixture proves nothing \
         (unbounded {}, bounded {})",
        unbounded.len(),
        bounded.len()
    );
    assert_eq!(
        unbounded.iter().map(|e| e.seq).min(),
        Some(1),
        "an unbounded replay's minimum seq is 1 — the contrast that proves the bound matters"
    );

    // And the bounded read still carries every one of the recent Work's own
    // events — the bound narrows the scan, never the Work's own history.
    let recent_events_in_bounded = bounded
        .iter()
        .filter(|e| e.work_id.as_deref() == Some(recent_id.as_str()))
        .count();
    let recent_events_in_unbounded = unbounded
        .iter()
        .filter(|e| e.work_id.as_deref() == Some(recent_id.as_str()))
        .count();
    assert_eq!(
        recent_events_in_bounded, recent_events_in_unbounded,
        "the bound must never drop any of the recent Work's own events"
    );
}

/// §1.3 continued: the test above proves the bound against `Core::events_after`
/// directly, but never calls `work_transcript` itself, so it would still
/// pass even if the handler's own `from` computation (src/api.rs, the
/// `first_seq_by_work` lookup) were reverted to a bare `0` — the finding
/// this test answers. Its own doc comment (and this file's copy above)
/// explains why the HTTP *response body* cannot distinguish a correct bound
/// from `from=0`: `transcript_turns` filters by `work_id` internally, so an
/// unbounded replay's extra events never surface in the JSON either way.
///
/// The *status code* can, though: with a tiny `segment_max_bytes` (as
/// `start_with_retention` sets), enough old Works rotate the journal into
/// several segments before the recent one that a correctly-bounded replay
/// (`Replay::after`'s segment-skip, unchanged by this wave) never opens the
/// oldest segment at all. Corrupting that oldest segment — proven, not
/// merely assumed, to hold none of the recent Work's own events — leaves a
/// correct bound untouched (200) while a replay from the floor or from `0`
/// would hit the corruption immediately and 500. This drives the real
/// `GET /v1/work/{id}/transcript` handler over HTTP, the same daemon the
/// other transcript tests in this file use.
#[tokio::test]
async fn work_transcript_over_http_skips_a_corrupted_segment_before_the_works_own_first_seq() {
    let dir = TempDir::new().expect("tempdir");
    let root = TempDir::new().expect("tempdir");
    support::scaffold_solo_estate(root.path(), "solo");
    write_one_stage_workflow(root.path());
    let http = client();

    const OLD_WORKS: usize = 8;
    let script: Vec<FakeStep> = (0..OLD_WORKS + 1).map(|_| FakeStep::complete()).collect();
    let handle = start_with_retention(dir.path(), root.path(), 1_000_000, script).await;
    for n in 0..OLD_WORKS {
        submit(
            &http,
            &handle,
            root.path(),
            &ulid(),
            &format!("old junk {n}"),
        )
        .await;
    }
    let body = submit(&http, &handle, root.path(), &ulid(), "the recent one").await;
    let recent_id = body["work"]["id"].as_str().expect("id").to_string();
    wait_until_all_settled(&http, &handle).await;

    // The daemon is still running and still holds the journal's exclusive
    // lock, so this mutates the segment file on disk directly, exactly as
    // `sse_stream_sends_an_error_frame_before_closing_on_a_journal_failure`
    // above does — every read of it (`events_after`, unlike the daemon's own
    // writer) opens the file fresh.
    let journal_dir = dir.path().join("journal");
    let mut segments: Vec<_> = std::fs::read_dir(&journal_dir)
        .expect("journal dir")
        .filter_map(|entry| {
            let path = entry.expect("entry").path();
            (path.extension().is_some_and(|ext| ext == "ndjson")).then_some(path)
        })
        .collect();
    segments.sort();
    assert!(
        segments.len() > 1,
        "the tiny segment_max_bytes fixture must actually rotate the journal \
         into more than one segment, or this test proves nothing: {segments:?}"
    );
    let oldest = segments.first().expect("at least one segment").clone();
    let oldest_text = std::fs::read_to_string(&oldest).expect("read oldest segment");
    assert!(
        !oldest_text.contains(&recent_id),
        "the oldest segment must not already hold any of the recent Work's \
         own events, or corrupting it would not prove the bound: {oldest:?}"
    );
    std::fs::write(
        &oldest,
        format!("{oldest_text}{{ \"not\": \"an event\" }}\n"),
    )
    .expect("append malformed line to the oldest segment");

    let resp = http
        .get(format!(
            "{}/v1/work/{recent_id}/transcript",
            handle.endpoint
        ))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("transcript over http");
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::OK,
        "a correctly-bounded replay never opens the corrupted oldest segment; \
         a 500 here means the handler replayed from the floor (or from 0) \
         again, the exact regression this test exists to catch"
    );
    let body: Value = resp.json().await.expect("json");
    assert_eq!(body["work_id"], recent_id);
    assert!(body["turns"].is_array());

    handle.shutdown().await;
}
