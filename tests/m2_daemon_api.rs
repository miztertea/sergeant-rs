//! M2 acceptance tests (docs/gauntlet/contracts/M2.md).
//!
//! 1. Descriptor lifecycle: written on start (owner-only, valid JSON,
//!    loopback), `/healthz` answers, removed on clean shutdown.
//! 2. Bearer auth: missing/wrong token → 401; correct token → 200.
//! 3. Submit → pending in list/show; `work.submitted` journaled; daemon
//!    restart replays it (not cached).
//! 4. Idempotency: duplicate `command_id` → one work, byte-identical result,
//!    also across restart.
//! 5. Cancel: canceled; duplicate cancel idempotent; unknown id structured
//!    error; `canceled → pending` impossible.
//! 6. SSE: live tail after connect; resume from seq N yields exactly the
//!    events after N.
//! 7. CLI end-to-end (spawned binary): auto-spawn, submit, list --json;
//!    second daemon fails closed.
//! 8. Spawn race: two concurrent clients, exactly one surviving daemon,
//!    both commands complete.
//!
//! Plus, beyond the numbered list: stale/ambiguous descriptor handling per
//! the contract's auto-spawn clause (including two clients replacing the same
//! stale descriptor), the crash image between a mutation append and its
//! command record, journaled rejections, the daemon's own use of the
//! transition table, and structured errors on malformed query strings.

use std::path::Path;
use std::process::Output;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::daemon::{self, DaemonHandle, RuntimeDescriptor};
use sergeant_rs::domain::event::{EVENT_SCHEMA, Event};
use sergeant_rs::domain::work::{
    KIND_COMMAND_ACCEPTED, KIND_COMMAND_REJECTED, KIND_WORK_CANCELED, KIND_WORK_SUBMITTED,
    WorkState,
};
use sergeant_rs::runtime::journal::Journal;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

fn ulid() -> String {
    ulid::Ulid::generate().to_string()
}

async fn start(dir: &Path) -> DaemonHandle {
    daemon::start(dir).await.expect("daemon start")
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("client")
}

/// Submit work; returns (status, exact response bytes, parsed body).
async fn submit(
    http: &reqwest::Client,
    handle: &DaemonHandle,
    command_id: &str,
    intent: &str,
) -> (reqwest::StatusCode, Vec<u8>, Value) {
    let resp = http
        .post(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&json!({"command_id": command_id, "intent": intent}))
        .send()
        .await
        .expect("submit request");
    let status = resp.status();
    let bytes = resp.bytes().await.expect("submit body").to_vec();
    let value: Value = serde_json::from_slice(&bytes).expect("submit json");
    (status, bytes, value)
}

/// Every event in a data dir's journal, in seq order.
fn journal_events(dir: &Path) -> Vec<Event> {
    Journal::replay_data_dir(dir)
        .expect("replay")
        .map(|e| e.expect("event"))
        .collect()
}

/// Cut a data dir's journal back to its first `keep` events — the image a
/// crash leaves behind at that point in the append sequence.
fn truncate_journal(dir: &Path, keep: usize) {
    let journal = dir.join("journal");
    let mut segments: Vec<_> = std::fs::read_dir(&journal)
        .expect("journal dir")
        .filter_map(|entry| {
            let path = entry.expect("journal entry").path();
            (path.extension().is_some_and(|ext| ext == "ndjson")).then_some(path)
        })
        .collect();
    segments.sort();
    let mut remaining = keep;
    for segment in segments {
        let text = std::fs::read_to_string(&segment).expect("read segment");
        let lines: Vec<&str> = text.lines().collect();
        let take = remaining.min(lines.len());
        remaining -= take;
        let mut kept = lines[..take].join("\n");
        if !kept.is_empty() {
            kept.push('\n');
        }
        std::fs::write(&segment, kept).expect("truncate segment");
    }
}

/// Seed a data dir's journal with one `work.submitted` whose work is already
/// in `state`. Nothing in the M2 API can produce a state other than `pending`
/// or `canceled`, so states that exercise the transition table have to be
/// planted in the history the daemon replays.
fn seed_work_in_state(dir: &Path, work_id: &str, state: &str) {
    let journal = dir.join("journal");
    std::fs::create_dir_all(&journal).expect("journal dir");
    let event = json!({
        "schema": EVENT_SCHEMA,
        "seq": 1,
        "id": ulid(),
        "timestamp": "2026-08-08T00:00:00.000Z",
        "source": {"type": "daemon", "name": "api"},
        "work_id": work_id,
        "kind": KIND_WORK_SUBMITTED,
        "payload": {"work": {
            "id": work_id,
            "intent": "seeded work",
            "state": state,
            "created_by": "test",
            "created_at": "2026-08-08T00:00:00.000Z",
        }},
    });
    let mut line = serde_json::to_vec(&event).expect("seed json");
    line.push(b'\n');
    std::fs::write(journal.join("00000001.ndjson"), line).expect("write seed segment");
}

/// How many events the daemon has journaled so far (via the API).
async fn event_count(http: &reqwest::Client, handle: &DaemonHandle) -> usize {
    let history: Value = http
        .get(format!("{}/v1/events", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("history")
        .json()
        .await
        .expect("history json");
    history["events"].as_array().expect("events").len()
}

#[tokio::test]
async fn t1_descriptor_lifecycle_and_healthz() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;

    // Descriptor exists, is valid JSON, loopback endpoint, owner-only.
    let path = daemon::descriptor_path(dir.path());
    let bytes = std::fs::read(&path).expect("descriptor readable");
    let descriptor: RuntimeDescriptor = serde_json::from_slice(&bytes).expect("descriptor json");
    assert_eq!(descriptor.schema, "sergeant.runtime/v1");
    assert!(
        descriptor.endpoint.starts_with("http://127.0.0.1:"),
        "loopback endpoint, got {}",
        descriptor.endpoint
    );
    assert_eq!(descriptor.pid, std::process::id());
    assert_eq!(descriptor.api_revision, "v1");
    assert!(!descriptor.token.is_empty());
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let mode = std::fs::metadata(&path)
            .expect("descriptor metadata")
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "descriptor must be owner-only, got {mode:o}");
    }

    // /healthz answers without auth.
    let resp = client()
        .get(format!("{}/healthz", handle.endpoint))
        .send()
        .await
        .expect("healthz");
    assert_eq!(resp.status(), 200);

    // Clean shutdown removes the descriptor.
    handle.shutdown().await;
    assert!(
        !path.exists(),
        "descriptor must be removed on clean shutdown"
    );
}

#[tokio::test]
async fn t2_bearer_token_gates_v1_routes() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();

    // Missing token → 401 with structured error.
    let resp = http
        .get(format!("{}/v1/work", handle.endpoint))
        .send()
        .await
        .expect("no-token request");
    assert_eq!(resp.status(), 401);
    let body: Value = resp.json().await.expect("401 body json");
    assert_eq!(body["error"]["code"], "unauthorized");

    // Wrong token → 401.
    let resp = http
        .get(format!("{}/v1/work", handle.endpoint))
        .bearer_auth("not-the-token")
        .send()
        .await
        .expect("wrong-token request");
    assert_eq!(resp.status(), 401);

    // The mutation routes are gated too, not just the reads: an unauthenticated
    // POST must never reach a handler that appends to the journal.
    for path in ["/v1/work", "/v1/work/01AN4Z07BY79KA1307SR9X4MV3/cancel"] {
        let resp = http
            .post(format!("{}{path}", handle.endpoint))
            .json(&json!({"command_id": ulid(), "intent": "unauthenticated"}))
            .send()
            .await
            .expect("no-token POST");
        assert_eq!(resp.status(), 401, "expected 401 for POST {path}");
        let body: Value = resp.json().await.expect("401 body json");
        assert_eq!(body["error"]["code"], "unauthorized");

        let resp = http
            .post(format!("{}{path}", handle.endpoint))
            .bearer_auth("not-the-token")
            .json(&json!({"command_id": ulid(), "intent": "unauthenticated"}))
            .send()
            .await
            .expect("wrong-token POST");
        assert_eq!(resp.status(), 401, "expected 401 for POST {path}");
    }
    // And nothing they sent was journaled: only daemon.started exists.
    let events = event_count(&http, &handle).await;
    assert_eq!(events, 1, "rejected requests must not append events");

    // Correct token → 200 on several /v1 routes.
    for path in ["/v1/work", "/v1/events", "/v1/system"] {
        let resp = http
            .get(format!("{}{path}", handle.endpoint))
            .bearer_auth(&handle.token)
            .send()
            .await
            .expect("authed request");
        assert_eq!(resp.status(), 200, "expected 200 for {path}");
    }

    handle.shutdown().await;
}

#[tokio::test]
async fn t3_submit_lists_pending_journals_and_survives_restart() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();

    let (status, _, body) = submit(&http, &handle, &ulid(), "write the M2 tests").await;
    assert_eq!(status, 201);
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "pending");

    // Visible in list and show as pending.
    let list: Value = http
        .get(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("list")
        .json()
        .await
        .expect("list json");
    assert_eq!(list["works"].as_array().expect("works").len(), 1);
    assert_eq!(list["works"][0]["id"], work_id.as_str());
    let show: Value = http
        .get(format!("{}/v1/work/{work_id}", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("show")
        .json()
        .await
        .expect("show json");
    assert_eq!(show["work"]["state"], "pending");

    handle.shutdown().await;

    // The journal contains work.submitted for this work.
    let submitted: Vec<_> = Journal::replay_data_dir(dir.path())
        .expect("replay")
        .map(|e| e.expect("event"))
        .filter(|e| e.kind == KIND_WORK_SUBMITTED && e.work_id.as_deref() == Some(&work_id))
        .collect();
    assert_eq!(submitted.len(), 1, "exactly one work.submitted journaled");

    // Restart on the same data dir: the work is replayed, not cached.
    let handle = start(dir.path()).await;
    let list: Value = client()
        .get(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("list after restart")
        .json()
        .await
        .expect("list json");
    assert_eq!(list["works"].as_array().expect("works").len(), 1);
    assert_eq!(list["works"][0]["id"], work_id.as_str());
    assert_eq!(list["works"][0]["state"], "pending");
    handle.shutdown().await;
}

#[tokio::test]
async fn t4_duplicate_command_id_is_byte_identical_even_across_restart() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();
    let command_id = ulid();

    let (s1, b1, _) = submit(&http, &handle, &command_id, "idempotent submit").await;
    // The duplicate must be a pure replay: byte-identical *and* silent in the
    // journal. Equal responses alone cannot distinguish "replayed the record"
    // from "executed again and happened to answer the same".
    let before = event_count(&http, &handle).await;
    let (s2, b2, _) = submit(&http, &handle, &command_id, "idempotent submit").await;
    let after = event_count(&http, &handle).await;
    assert_eq!(s1, 201);
    assert_eq!(s1, s2, "duplicate must replay the original status");
    assert_eq!(b1, b2, "duplicate must be byte-identical");
    assert_eq!(before, after, "a duplicate must append no events");

    // Exactly one work record exists.
    let list: Value = http
        .get(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("list")
        .json()
        .await
        .expect("list json");
    assert_eq!(list["works"].as_array().expect("works").len(), 1);

    handle.shutdown().await;

    // The duplicate arriving after a daemon restart still replays.
    let handle = start(dir.path()).await;
    let (s3, b3, _) = submit(&client(), &handle, &command_id, "idempotent submit").await;
    assert_eq!(s3, s1);
    assert_eq!(b3, b1, "post-restart duplicate must be byte-identical");
    let list: Value = client()
        .get(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("list")
        .json()
        .await
        .expect("list json");
    assert_eq!(list["works"].as_array().expect("works").len(), 1);
    handle.shutdown().await;

    // The journal is the proof of "without re-executing": across all three
    // submissions of this command_id there is exactly one work.submitted and
    // exactly one recorded outcome for it.
    let events = journal_events(dir.path());
    let submitted: Vec<_> = events
        .iter()
        .filter(|e| e.kind == KIND_WORK_SUBMITTED)
        .collect();
    assert_eq!(submitted.len(), 1, "exactly one work.submitted for 3 sends");
    assert_eq!(submitted[0].correlation_id.as_deref(), Some(&*command_id));
    let recorded: Vec<_> = events
        .iter()
        .filter(|e| {
            (e.kind == KIND_COMMAND_ACCEPTED || e.kind == KIND_COMMAND_REJECTED)
                && e.payload["command_id"] == command_id.as_str()
        })
        .collect();
    assert_eq!(recorded.len(), 1, "exactly one command record for 3 sends");
    assert_eq!(recorded[0].kind, KIND_COMMAND_ACCEPTED);
}

/// §26's whole purpose is retry-after-uncertain-outcome, and the uncertain
/// outcome is a crash between the mutation append and the command record.
/// The journal image left by that crash must not let the retry execute again.
#[tokio::test]
async fn crash_between_mutation_and_command_record_still_replays() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let command_id = ulid();
    let (status, body_bytes, body) = submit(&client(), &handle, &command_id, "crash window").await;
    assert_eq!(status, 201);
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    handle.shutdown().await;

    // Cut the journal back to the exact crash image: daemon.started and the
    // durable work.submitted, with its command.accepted never written.
    let events = journal_events(dir.path());
    let submitted_at = events
        .iter()
        .position(|e| e.kind == KIND_WORK_SUBMITTED)
        .expect("work.submitted journaled");
    truncate_journal(dir.path(), submitted_at + 1);
    let events = journal_events(dir.path());
    assert_eq!(events.len(), submitted_at + 1);
    assert_eq!(events.last().expect("last").kind, KIND_WORK_SUBMITTED);

    // The retry finds the work already durable and no outcome recorded: it
    // must answer for the command that already ran, not run a second one.
    let handle = start(dir.path()).await;
    let http = client();
    let (retry_status, retry_bytes, retry_body) =
        submit(&http, &handle, &command_id, "crash window").await;
    assert_eq!(retry_status, status);
    assert_eq!(
        retry_body["work"]["id"].as_str(),
        Some(work_id.as_str()),
        "the retry must return the work the first attempt created"
    );
    assert_eq!(
        retry_bytes, body_bytes,
        "the recovered result must match the original response"
    );
    let list: Value = http
        .get(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("list")
        .json()
        .await
        .expect("list json");
    assert_eq!(
        list["works"].as_array().expect("works").len(),
        1,
        "the retry must not create a second Work record"
    );

    // And the recovery is durable: a further retry is a plain replay.
    let before = event_count(&http, &handle).await;
    let (again_status, again_bytes, _) = submit(&http, &handle, &command_id, "crash window").await;
    assert_eq!(again_status, status);
    assert_eq!(again_bytes, body_bytes);
    assert_eq!(event_count(&http, &handle).await, before);
    handle.shutdown().await;

    let events = journal_events(dir.path());
    assert_eq!(
        events
            .iter()
            .filter(|e| e.kind == KIND_WORK_SUBMITTED)
            .count(),
        1,
        "the work was submitted exactly once across the crash"
    );
}

/// A rejected submit is a recorded outcome too: the same `command_id` must
/// keep answering the recorded 400 instead of being re-validated (and, with a
/// different body, succeeding — two results for one command id).
#[tokio::test]
async fn rejected_submit_is_journaled_and_replayed() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();
    let command_id = ulid();

    let (status, bytes, body) = submit(&http, &handle, &command_id, "   ").await;
    assert_eq!(status, 400);
    assert_eq!(body["error"]["code"], "invalid_request");

    // Same command_id, now with a valid intent: still the recorded rejection.
    let (retry_status, retry_bytes, _) = submit(&http, &handle, &command_id, "now valid").await;
    assert_eq!(retry_status, status, "one command_id, one result");
    assert_eq!(retry_bytes, bytes, "the rejection must replay verbatim");
    let list: Value = http
        .get(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("list")
        .json()
        .await
        .expect("list json");
    assert!(
        list["works"].as_array().expect("works").is_empty(),
        "a rejected command must never create work"
    );
    handle.shutdown().await;

    // The rejection is in the journal, so it survives a restart.
    let rejected: Vec<_> = journal_events(dir.path())
        .into_iter()
        .filter(|e| e.kind == KIND_COMMAND_REJECTED && e.payload["command_id"] == command_id)
        .collect();
    assert_eq!(rejected.len(), 1, "the rejection must be journaled");
    let handle = start(dir.path()).await;
    let (after_restart, after_bytes, _) =
        submit(&client(), &handle, &command_id, "now valid").await;
    assert_eq!(after_restart, status);
    assert_eq!(after_bytes, bytes);
    handle.shutdown().await;
}

/// Malformed query strings must answer structured JSON like every other
/// error on the surface — axum's stock `Query` rejection is plain text.
#[tokio::test]
async fn malformed_query_string_is_a_structured_error() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();

    for path in ["/v1/events?from=abc", "/v1/events/stream?from=abc"] {
        let resp = http
            .get(format!("{}{path}", handle.endpoint))
            .bearer_auth(&handle.token)
            .send()
            .await
            .expect("bad query request");
        assert_eq!(resp.status(), 400, "expected 400 for {path}");
        assert_eq!(
            resp.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.starts_with("application/json")),
            Some(true),
            "errors are structured JSON, got {:?} for {path}",
            resp.headers().get("content-type"),
        );
        let body: Value = resp.json().await.expect("error body json");
        assert_eq!(body["error"]["code"], "invalid_request");
        assert!(
            body["error"]["message"]
                .as_str()
                .expect("message")
                .contains("from"),
            "the diagnostic must name the bad parameter: {body}"
        );
    }
    handle.shutdown().await;
}

#[tokio::test]
async fn t5_cancel_semantics_and_state_machine() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();

    let (_, _, body) = submit(&http, &handle, &ulid(), "cancel me").await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();

    // Cancel → canceled.
    let resp = http
        .post(format!("{}/v1/work/{work_id}/cancel", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&json!({"command_id": ulid()}))
        .send()
        .await
        .expect("cancel");
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.expect("cancel json");
    assert_eq!(body["work"]["state"], "canceled");

    // Duplicate cancel (fresh command_id) is an idempotent success and does
    // not journal a second work.canceled.
    let resp = http
        .post(format!("{}/v1/work/{work_id}/cancel", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&json!({"command_id": ulid()}))
        .send()
        .await
        .expect("duplicate cancel");
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.expect("duplicate cancel json");
    assert_eq!(body["work"]["state"], "canceled");

    // Cancel of an unknown id → structured error.
    let resp = http
        .post(format!(
            "{}/v1/work/01AN4Z07BY79KA1307SR9X4MV3/cancel",
            handle.endpoint
        ))
        .bearer_auth(&handle.token)
        .json(&json!({"command_id": ulid()}))
        .send()
        .await
        .expect("unknown cancel");
    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.expect("unknown cancel json");
    assert_eq!(body["error"]["code"], "work_not_found");

    handle.shutdown().await;

    // Exactly one work.canceled in the journal despite two cancel commands.
    let canceled: Vec<_> = Journal::replay_data_dir(dir.path())
        .expect("replay")
        .map(|e| e.expect("event"))
        .filter(|e| e.kind == KIND_WORK_CANCELED)
        .collect();
    assert_eq!(canceled.len(), 1, "duplicate cancel must not re-transition");

    // canceled → pending is impossible in the state machine itself.
    assert!(!WorkState::Canceled.can_transition(WorkState::Pending));
}

/// Fail-closed means the *daemon* consults the transition table before
/// appending, not merely that the table is correct in isolation: a work in a
/// state cancel cannot leave is rejected with 409 and no `work.canceled` is
/// ever written.
#[tokio::test]
async fn t5b_daemon_refuses_an_illegal_transition_and_appends_no_state_event() {
    let dir = TempDir::new().expect("tempdir");
    let work_id = ulid();
    seed_work_in_state(dir.path(), &work_id, "active");
    let handle = start(dir.path()).await;
    let http = client();

    // The replayed work really is in a state cancel cannot leave.
    let show: Value = http
        .get(format!("{}/v1/work/{work_id}", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("show")
        .json()
        .await
        .expect("show json");
    assert_eq!(show["work"]["state"], "active");
    assert!(!WorkState::Active.can_transition(WorkState::Canceled));

    let command_id = ulid();
    let resp = http
        .post(format!("{}/v1/work/{work_id}/cancel", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&json!({"command_id": command_id}))
        .send()
        .await
        .expect("cancel active");
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.expect("cancel json");
    assert_eq!(body["error"]["code"], "illegal_transition");

    // The work is untouched.
    let show: Value = http
        .get(format!("{}/v1/work/{work_id}", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("show after")
        .json()
        .await
        .expect("show json");
    assert_eq!(show["work"]["state"], "active");
    handle.shutdown().await;

    let events = journal_events(dir.path());
    assert_eq!(
        events
            .iter()
            .filter(|e| e.kind == KIND_WORK_CANCELED)
            .count(),
        0,
        "a rejected transition must append no state event"
    );
    assert_eq!(
        events
            .iter()
            .filter(|e| e.kind == KIND_COMMAND_REJECTED && e.payload["command_id"] == command_id)
            .count(),
        1,
        "the rejection itself is recorded under its command_id"
    );
}

/// Read SSE frames from a live response until `count` events (or timeout).
/// Returns (seq, kind, data-json) triples.
async fn read_sse_events(resp: &mut reqwest::Response, count: usize) -> Vec<(u64, String, Value)> {
    let mut buffer = String::new();
    let mut events = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    while events.len() < count && Instant::now() < deadline {
        let chunk = tokio::time::timeout(Duration::from_secs(10), resp.chunk())
            .await
            .expect("sse chunk timeout")
            .expect("sse chunk io");
        let Some(chunk) = chunk else { break };
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(pos) = buffer.find("\n\n") {
            let frame = buffer[..pos].to_string();
            buffer.drain(..pos + 2);
            let mut id = None;
            let mut kind = None;
            let mut data = None;
            for line in frame.lines() {
                if let Some(v) = line.strip_prefix("id:") {
                    id = v.trim().parse::<u64>().ok();
                } else if let Some(v) = line.strip_prefix("event:") {
                    kind = Some(v.trim().to_string());
                } else if let Some(v) = line.strip_prefix("data:") {
                    data = serde_json::from_str::<Value>(v.trim()).ok();
                }
            }
            if let (Some(id), Some(kind), Some(data)) = (id, kind, data) {
                events.push((id, kind, data));
            }
        }
    }
    events
}

#[tokio::test]
async fn t6_sse_live_tail_and_resume_from_seq() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();

    // Current last seq (daemon.started is already journaled).
    let history: Value = http
        .get(format!("{}/v1/events", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("history")
        .json()
        .await
        .expect("history json");
    let last_seq = history["events"]
        .as_array()
        .expect("events")
        .last()
        .and_then(|e| e["seq"].as_u64())
        .expect("last seq");

    // Connect a live client from the current position...
    let mut stream = http
        .get(format!(
            "{}/v1/events/stream?from={last_seq}",
            handle.endpoint
        ))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("sse connect");
    assert_eq!(stream.status(), 200);

    // ...then append events after connect.
    let (_, _, body) = submit(&http, &handle, &ulid(), "sse live event").await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();

    // The connected client receives them: work.submitted then
    // command.accepted, both with seq > last_seq, no gaps or repeats.
    let events = read_sse_events(&mut stream, 2).await;
    assert_eq!(events.len(), 2, "live client must receive appended events");
    assert_eq!(events[0].0, last_seq + 1);
    assert_eq!(events[0].1, "work.submitted");
    assert_eq!(events[0].2["work_id"], work_id.as_str());
    assert_eq!(events[1].0, last_seq + 2);
    assert_eq!(events[1].1, "command.accepted");
    drop(stream);

    // Resume from seq N: exactly the events after N, in order.
    let mut resumed = http
        .get(format!("{}/v1/events/stream", handle.endpoint))
        .header("Last-Event-ID", last_seq.to_string())
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("sse resume");
    let events = read_sse_events(&mut resumed, 2).await;
    let seqs: Vec<u64> = events.iter().map(|(seq, _, _)| *seq).collect();
    assert_eq!(
        seqs,
        vec![last_seq + 1, last_seq + 2],
        "resume must yield exactly the events after N"
    );
    drop(resumed);

    handle.shutdown().await;
}

/// Run the sgt binary with args against a data dir; capture output.
fn sgt(data_dir: &Path, args: &[&str]) -> Output {
    std::process::Command::new(SGT)
        .arg("--data-dir")
        .arg(data_dir)
        .args(args)
        .output()
        .expect("run sgt")
}

fn descriptor_of(dir: &Path) -> Option<RuntimeDescriptor> {
    daemon::read_descriptor(dir).expect("read descriptor")
}

/// Kill a daemon by pid (SIGTERM) and wait for its descriptor to disappear.
fn stop_daemon(dir: &Path) {
    if let Some(descriptor) = descriptor_of(dir) {
        let _ = std::process::Command::new("kill")
            .arg(descriptor.pid.to_string())
            .status();
        let deadline = Instant::now() + Duration::from_secs(10);
        while daemon::descriptor_path(dir).exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

#[test]
fn t7_cli_end_to_end_auto_spawn_and_second_daemon_fails_closed() {
    let dir = TempDir::new().expect("tempdir");

    // No daemon running: `sgt run` auto-spawns one and submits.
    let output = sgt(dir.path(), &["run", "ship the M2 milestone"]);
    assert!(
        output.status.success(),
        "sgt run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // `sgt work list --json` shows it.
    let output = sgt(dir.path(), &["work", "list", "--json"]);
    assert!(
        output.status.success(),
        "sgt work list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let listed: Value =
        serde_json::from_slice(&output.stdout).expect("work list --json must print JSON");
    let works = listed["works"].as_array().expect("works array");
    assert_eq!(works.len(), 1);
    assert_eq!(works[0]["intent"], "ship the M2 milestone");
    assert_eq!(works[0]["state"], "pending");
    assert_eq!(works[0]["created_by"], "cli");

    // A second daemon on the same data dir fails closed.
    let output = sgt(dir.path(), &["daemon"]);
    assert!(
        !output.status.success(),
        "second daemon must fail closed, got: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("another daemon"),
        "second daemon should explain the lock, got: {stderr}"
    );

    stop_daemon(dir.path());
}

#[test]
fn t8_two_concurrent_auto_spawns_one_survivor_both_commands_complete() {
    let dir = TempDir::new().expect("tempdir");
    let path_a = dir.path().to_path_buf();
    let path_b = dir.path().to_path_buf();

    let a = std::thread::spawn(move || sgt(&path_a, &["run", "racer A"]));
    let b = std::thread::spawn(move || sgt(&path_b, &["run", "racer B"]));
    let out_a = a.join().expect("thread A");
    let out_b = b.join().expect("thread B");
    assert!(
        out_a.status.success(),
        "client A failed: {}",
        String::from_utf8_lossy(&out_a.stderr)
    );
    assert!(
        out_b.status.success(),
        "client B failed: {}",
        String::from_utf8_lossy(&out_b.stderr)
    );

    // Both commands completed: two works listed.
    let output = sgt(dir.path(), &["work", "list", "--json"]);
    assert!(output.status.success());
    let listed: Value = serde_json::from_slice(&output.stdout).expect("list json");
    let intents: Vec<&str> = listed["works"]
        .as_array()
        .expect("works")
        .iter()
        .map(|w| w["intent"].as_str().expect("intent"))
        .collect();
    assert_eq!(intents.len(), 2, "both submissions must have landed");
    assert!(intents.contains(&"racer A") && intents.contains(&"racer B"));

    // Exactly one surviving daemon: the descriptor names a live PID, and
    // scanning /proc finds exactly one `sgt ... daemon` on this data dir.
    let descriptor = descriptor_of(dir.path()).expect("descriptor exists");
    assert!(
        daemon::pid_alive(descriptor.pid),
        "descriptor PID must be alive"
    );
    let needle = dir.path().to_string_lossy().to_string();
    let mut daemons = 0;
    for entry in std::fs::read_dir("/proc").expect("read /proc") {
        let entry = entry.expect("proc entry");
        if !entry
            .file_name()
            .to_string_lossy()
            .chars()
            .all(|c| c.is_ascii_digit())
        {
            continue;
        }
        let Ok(cmdline) = std::fs::read(entry.path().join("cmdline")) else {
            continue;
        };
        let cmdline = String::from_utf8_lossy(&cmdline).replace('\0', " ");
        if cmdline.contains(&needle) && cmdline.contains(" daemon") && cmdline.contains("sgt") {
            daemons += 1;
        }
    }
    assert_eq!(daemons, 1, "exactly one daemon may survive the race");

    stop_daemon(dir.path());
}

#[test]
fn stale_descriptor_is_replaced_but_ambiguous_descriptor_fails_closed() {
    // Stale: dead PID, refused endpoint → the client replaces it and spawns.
    let dir = TempDir::new().expect("tempdir");
    std::fs::create_dir_all(dir.path()).expect("data dir");
    let stale = json!({
        "schema": "sergeant.runtime/v1",
        "endpoint": "http://127.0.0.1:1",
        "pid": 999_999_999u32,
        "api_revision": "v1",
        "token": "dead",
    });
    std::fs::write(
        daemon::descriptor_path(dir.path()),
        serde_json::to_vec(&stale).expect("stale json"),
    )
    .expect("write stale descriptor");
    let output = sgt(dir.path(), &["run", "after stale descriptor"]);
    assert!(
        output.status.success(),
        "stale descriptor must be replaced: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let replaced = descriptor_of(dir.path()).expect("new descriptor");
    assert_ne!(replaced.pid, 999_999_999);
    stop_daemon(dir.path());

    // Ambiguous: alive PID (this test process), unresponsive endpoint →
    // fail closed with a diagnostic, never a second daemon.
    let dir = TempDir::new().expect("tempdir");
    std::fs::create_dir_all(dir.path()).expect("data dir");
    let ambiguous = json!({
        "schema": "sergeant.runtime/v1",
        "endpoint": "http://127.0.0.1:1",
        "pid": std::process::id(),
        "api_revision": "v1",
        "token": "ambiguous",
    });
    std::fs::write(
        daemon::descriptor_path(dir.path()),
        serde_json::to_vec(&ambiguous).expect("ambiguous json"),
    )
    .expect("write ambiguous descriptor");
    let output = sgt(dir.path(), &["run", "must fail closed"]);
    assert!(
        !output.status.success(),
        "ambiguous descriptor must fail closed"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("refusing to spawn a second daemon"),
        "diagnostic expected, got: {stderr}"
    );
    // And it must not have spawned anything: descriptor unchanged.
    let descriptor = descriptor_of(dir.path()).expect("descriptor still there");
    assert_eq!(descriptor.token, "ambiguous");
}

/// Two clients replacing the *same* stale descriptor must not leave the
/// winner's daemon undiscoverable. The endpoint here accepts connections and
/// never answers, so each client spends its full health-probe timeout in the
/// window during which the other client's daemon publishes a fresh
/// descriptor — the window in which a client that unlinks the path deletes a
/// successor's record and wedges the data dir for good.
#[test]
fn concurrent_stale_replacement_leaves_the_surviving_daemon_discoverable() {
    let dir = TempDir::new().expect("tempdir");
    std::fs::create_dir_all(dir.path()).expect("data dir");
    let blackhole = std::net::TcpListener::bind("127.0.0.1:0").expect("blackhole listener");
    let port = blackhole.local_addr().expect("blackhole addr").port();
    let stale = json!({
        "schema": "sergeant.runtime/v1",
        "endpoint": format!("http://127.0.0.1:{port}"),
        "pid": 999_999_999u32,
        "api_revision": "v1",
        "token": "dead",
    });
    std::fs::write(
        daemon::descriptor_path(dir.path()),
        serde_json::to_vec(&stale).expect("stale json"),
    )
    .expect("write stale descriptor");

    let path_a = dir.path().to_path_buf();
    let path_b = dir.path().to_path_buf();
    let a = std::thread::spawn(move || sgt(&path_a, &["run", "stale racer A"]));
    std::thread::sleep(Duration::from_millis(300));
    let b = std::thread::spawn(move || sgt(&path_b, &["run", "stale racer B"]));
    let out_a = a.join().expect("thread A");
    let out_b = b.join().expect("thread B");
    assert!(
        out_a.status.success(),
        "client A failed: {}",
        String::from_utf8_lossy(&out_a.stderr)
    );
    assert!(
        out_b.status.success(),
        "client B failed: {}",
        String::from_utf8_lossy(&out_b.stderr)
    );

    // The surviving daemon is still reachable through the descriptor.
    let descriptor = descriptor_of(dir.path()).expect("descriptor must survive the race");
    assert_ne!(descriptor.pid, 999_999_999, "stale descriptor was replaced");
    assert!(
        daemon::pid_alive(descriptor.pid),
        "the descriptor must name the live daemon"
    );
    let output = sgt(dir.path(), &["work", "list", "--json"]);
    assert!(
        output.status.success(),
        "a later client must still find the daemon: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let listed: Value = serde_json::from_slice(&output.stdout).expect("list json");
    assert_eq!(listed["works"].as_array().expect("works").len(), 2);

    drop(blackhole);
    stop_daemon(dir.path());
}
