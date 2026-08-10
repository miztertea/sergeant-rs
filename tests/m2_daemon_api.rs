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
//! stale descriptor), an unknown descriptor schema, the crash image between a
//! mutation append and its command record, journaled rejections, the daemon's
//! own use of the transition table, structured errors on malformed queries,
//! bodies, command ids and resume headers, the router's own 404/405, and
//! shutdown with a live SSE tail attached.

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

mod support;
use support::{DataDir, ReapSignal};

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

/// Cancel `work_id`; returns (status, exact response bytes).
async fn cancel(
    http: &reqwest::Client,
    handle: &DaemonHandle,
    work_id: &str,
    command_id: &str,
) -> (reqwest::StatusCode, Vec<u8>) {
    let resp = http
        .post(format!("{}/v1/work/{work_id}/cancel", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&json!({"command_id": command_id}))
        .send()
        .await
        .expect("cancel request");
    let status = resp.status();
    let bytes = resp.bytes().await.expect("cancel body").to_vec();
    (status, bytes)
}

/// Seqs returned by `GET /v1/events`, optionally with `?from=`.
async fn history_seqs(
    http: &reqwest::Client,
    handle: &DaemonHandle,
    from: Option<u64>,
) -> Vec<u64> {
    let url = match from {
        Some(seq) => format!("{}/v1/events?from={seq}", handle.endpoint),
        None => format!("{}/v1/events", handle.endpoint),
    };
    let body: Value = http
        .get(url)
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("history")
        .json()
        .await
        .expect("history json");
    body["events"]
        .as_array()
        .expect("events")
        .iter()
        .map(|e| e["seq"].as_u64().expect("seq"))
        .collect()
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
    assert_eq!(descriptor.token, handle.token);
    assert_token_plausible(&descriptor.token);
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
    let first_token = descriptor.token.clone();
    handle.shutdown().await;
    assert!(
        !path.exists(),
        "descriptor must be removed on clean shutdown"
    );

    // The token is fresh randomness per daemon, not a build-time constant:
    // a successor on the same data dir publishes a different one. This is
    // not only auth strength — the client's stale-descriptor identity check
    // (`is_stale_descriptor`: endpoint + pid + token) is only sound because
    // a successor's descriptor can never compare equal to the stale one.
    let successor = start(dir.path()).await;
    let second_token = successor.token.clone();
    assert_token_plausible(&second_token);
    assert_ne!(
        first_token, second_token,
        "each daemon must publish a fresh random token"
    );
    successor.shutdown().await;
}

/// A bearer token must be long, high-entropy-looking, and safe to put in a
/// header: two Crockford-base32 ULIDs' worth of uppercase alphanumerics.
fn assert_token_plausible(token: &str) {
    assert!(
        token.len() >= 32,
        "token is too short to be random ({} chars): {token}",
        token.len()
    );
    assert!(
        token
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()),
        "token must be uppercase base32 alphanumerics: {token}"
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
    let before = event_count(&http, &handle).await;
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
    // And nothing they sent was journaled: the journal is exactly where the
    // daemon's own startup left it.
    let events = event_count(&http, &handle).await;
    assert_eq!(events, before, "rejected requests must not append events");

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

/// `GET /v1/events` is contracted as "history, from a given seq". Every
/// cut point must yield exactly the tail after it — an implementation that
/// ignored `from` and always replayed everything would still satisfy a test
/// that only ever calls the route bare.
#[tokio::test]
async fn event_history_from_seq_returns_exactly_the_tail() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();
    submit(&http, &handle, &ulid(), "first").await;
    let (_, _, body) = submit(&http, &handle, &ulid(), "second").await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    cancel(&http, &handle, &work_id, &ulid()).await;

    // daemon.started + backend.probed (one per registered backend, M4)
    // + 2×(work.submitted, command.accepted) + work.canceled
    // + command.accepted.
    let all = history_seqs(&http, &handle, None).await;
    assert_eq!(all.len(), 9, "unexpected history: {all:?}");
    assert_eq!(all, (1..=9).collect::<Vec<u64>>(), "seqs are 1..n in order");

    // from=N ⇒ exactly the events after N, for every N in the history.
    for (cut, from) in all.iter().copied().enumerate() {
        assert_eq!(
            history_seqs(&http, &handle, Some(from)).await,
            all[cut + 1..].to_vec(),
            "?from={from} must return exactly the events after it"
        );
    }
    // from=0 is the whole history; past the end is empty, not an error.
    assert_eq!(history_seqs(&http, &handle, Some(0)).await, all);
    assert!(
        history_seqs(&http, &handle, Some(all.len() as u64 + 1))
            .await
            .is_empty(),
        "a from beyond the last seq yields nothing"
    );
    handle.shutdown().await;
}

/// The router's own errors are part of "errors are structured JSON": an
/// unknown route and a known route hit with the wrong method must answer
/// with the same `{"error": {...}}` shape as any handler, not axum's stock
/// empty body.
#[tokio::test]
async fn unknown_route_and_wrong_method_answer_structured_json() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();
    let work_id = "01AN4Z07BY79KA1307SR9X4MV3";

    for path in ["/nope", "/v1/nope", &format!("/v1/work/{work_id}/nope")] {
        let resp = http
            .get(format!("{}{path}", handle.endpoint))
            .bearer_auth(&handle.token)
            .send()
            .await
            .expect("unknown route request");
        assert_eq!(resp.status(), 404, "expected 404 for {path}");
        assert_json_error(resp, "not_found").await;
    }

    // Wrong method on a known route, at both router levels: `/healthz` on the
    // outer router, the cancel and show routes inside the nested `/v1`.
    let resp = http
        .post(format!("{}/healthz", handle.endpoint))
        .send()
        .await
        .expect("wrong method on healthz");
    assert_eq!(resp.status(), 405);
    assert_json_error(resp, "method_not_allowed").await;

    for (method, path) in [
        ("GET", format!("/v1/work/{work_id}/cancel")),
        ("POST", format!("/v1/work/{work_id}")),
    ] {
        let url = format!("{}{path}", handle.endpoint);
        let request = match method {
            "GET" => http.get(url),
            _ => http.post(url),
        };
        let resp = request
            .bearer_auth(&handle.token)
            .send()
            .await
            .expect("wrong method request");
        assert_eq!(resp.status(), 405, "expected 405 for {method} {path}");
        assert_json_error(resp, "method_not_allowed").await;
    }
    handle.shutdown().await;
}

/// Requests the daemon cannot even key by command id — an unparseable body,
/// a `command_id` that is not a ULID — and a malformed resume header must
/// answer structured 4xx errors, and must leave nothing in the journal:
/// there is no command identity to record them under.
#[tokio::test]
async fn malformed_bodies_command_ids_and_resume_headers_are_structured_errors() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();
    let work_id = "01AN4Z07BY79KA1307SR9X4MV3";
    let before = event_count(&http, &handle).await;

    // Not a ULID → 400 invalid_command_id, on every mutation route.
    for (path, body) in [
        ("/v1/work", json!({"command_id": "nope", "intent": "hi"})),
        (
            "/v1/work/01AN4Z07BY79KA1307SR9X4MV3/cancel",
            json!({"command_id": "nope"}),
        ),
    ] {
        let resp = http
            .post(format!("{}{path}", handle.endpoint))
            .bearer_auth(&handle.token)
            .json(&body)
            .send()
            .await
            .expect("bad command_id request");
        assert_eq!(resp.status(), 400, "expected 400 for {path}");
        assert_json_error(resp, "invalid_command_id").await;
    }

    // Unparseable and mis-shaped bodies → structured 4xx invalid_request.
    for (path, body) in [
        ("/v1/work", "{ not json"),
        ("/v1/work", r#"{"intent": "no command id"}"#),
        (&format!("/v1/work/{work_id}/cancel"), "]"),
    ] {
        let resp = http
            .post(format!("{}{path}", handle.endpoint))
            .bearer_auth(&handle.token)
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await
            .expect("bad body request");
        assert!(
            resp.status().is_client_error(),
            "expected a 4xx for body {body:?} on {path}, got {}",
            resp.status()
        );
        assert_json_error(resp, "invalid_request").await;
    }

    // A resume header that is not a decimal seq → 400, not a silent restart
    // of the stream from the beginning.
    let resp = http
        .get(format!("{}/v1/events/stream", handle.endpoint))
        .bearer_auth(&handle.token)
        .header("Last-Event-ID", "not-a-seq")
        .send()
        .await
        .expect("bad resume header request");
    assert_eq!(resp.status(), 400);
    assert_json_error(resp, "invalid_request").await;

    // None of it was journaled: these fail before any command identity exists.
    assert_eq!(
        event_count(&http, &handle).await,
        before,
        "unkeyable requests must append no events"
    );
    handle.shutdown().await;
}

/// Assert a response is a structured JSON error with `code`.
async fn assert_json_error(resp: reqwest::Response, code: &str) {
    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    assert_eq!(
        content_type
            .as_deref()
            .map(|v| v.starts_with("application/json")),
        Some(true),
        "errors are structured JSON, got content-type {content_type:?} with {status}",
    );
    let body: Value = resp.json().await.expect("error body json");
    assert_eq!(body["error"]["code"], code, "error body was {body}");
    assert!(
        body["error"]["message"]
            .as_str()
            .is_some_and(|m| !m.is_empty()),
        "an error must carry a message: {body}"
    );
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
///
/// The seeded state is `completed`: M2 wrote this test with `active`, which
/// M3's engine makes cancellable (cancelling a running work is the whole
/// point of §12's cancellation verb). `completed` is absorbing in every
/// milestone, so the property this test exists to pin — the daemon consults
/// the table before appending — is unchanged.
#[tokio::test]
async fn t5b_daemon_refuses_an_illegal_transition_and_appends_no_state_event() {
    let dir = TempDir::new().expect("tempdir");
    let work_id = ulid();
    seed_work_in_state(dir.path(), &work_id, "completed");
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
    assert_eq!(show["work"]["state"], "completed");
    assert!(!WorkState::Completed.can_transition(WorkState::Canceled));

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
    assert_eq!(show["work"]["state"], "completed");
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

/// §26 binds *every* mutation, not just submit: a repeated `command_id` on
/// the cancel route must replay the recorded outcome without re-executing.
/// A fresh command_id hits the already-canceled branch instead, so only a
/// resend of the same id exercises the replay guard — and only the journal
/// can tell the two apart.
#[tokio::test]
async fn duplicate_cancel_command_id_replays_the_recorded_outcome() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();
    let (_, _, body) = submit(&http, &handle, &ulid(), "cancel me twice").await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();

    let command_id = ulid();
    let (s1, b1) = cancel(&http, &handle, &work_id, &command_id).await;
    assert_eq!(s1, 200);
    let before = event_count(&http, &handle).await;
    let (s2, b2) = cancel(&http, &handle, &work_id, &command_id).await;
    assert_eq!(s2, s1, "the duplicate must replay the recorded status");
    assert_eq!(b1, b2, "the duplicate must be byte-identical");
    assert_eq!(
        event_count(&http, &handle).await,
        before,
        "a replayed cancel must append no events"
    );

    // The same holds for a *rejected* cancel: one command_id, one recorded
    // 404, no second rejection appended.
    let missing_id = ulid();
    let (s3, b3) = cancel(&http, &handle, "01AN4Z07BY79KA1307SR9X4MV3", &missing_id).await;
    assert_eq!(s3, 404);
    let before = event_count(&http, &handle).await;
    let (s4, b4) = cancel(&http, &handle, "01AN4Z07BY79KA1307SR9X4MV3", &missing_id).await;
    assert_eq!((s4, &b4), (s3, &b3));
    assert_eq!(event_count(&http, &handle).await, before);

    handle.shutdown().await;

    // And across a restart, from the journal rather than from memory.
    let handle = start(dir.path()).await;
    let http = client();
    let (s5, b5) = cancel(&http, &handle, &work_id, &command_id).await;
    assert_eq!(s5, s1);
    assert_eq!(b5, b1, "post-restart duplicate must be byte-identical");
    handle.shutdown().await;

    let events = journal_events(dir.path());
    for id in [&command_id, &missing_id] {
        let recorded: Vec<_> = events
            .iter()
            .filter(|e| {
                (e.kind == KIND_COMMAND_ACCEPTED || e.kind == KIND_COMMAND_REJECTED)
                    && e.payload["command_id"] == id.as_str()
            })
            .collect();
        assert_eq!(recorded.len(), 1, "exactly one command record for {id}");
    }
    assert_eq!(
        events
            .iter()
            .filter(|e| e.kind == KIND_WORK_CANCELED)
            .count(),
        1,
        "three cancels under one command_id must transition once"
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

    // Resume from seq N: exactly the events after N, in order. Reading only
    // as far as the expected frames would not see a duplicate or spurious
    // frame arriving behind them, so append two more events after resuming
    // and assert the *whole* tail — replayed history and live continuation —
    // matches frame for frame.
    let mut resumed = http
        .get(format!("{}/v1/events/stream", handle.endpoint))
        .header("Last-Event-ID", last_seq.to_string())
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("sse resume");
    let (_, _, body) = submit(&http, &handle, &ulid(), "sse resume event").await;
    let second_work_id = body["work"]["id"].as_str().expect("work id").to_string();
    let events = read_sse_events(&mut resumed, 4).await;
    let tail: Vec<(u64, &str)> = events
        .iter()
        .map(|(seq, kind, _)| (*seq, kind.as_str()))
        .collect();
    assert_eq!(
        tail,
        vec![
            (last_seq + 1, "work.submitted"),
            (last_seq + 2, "command.accepted"),
            (last_seq + 3, "work.submitted"),
            (last_seq + 4, "command.accepted"),
        ],
        "resume must yield exactly the events after N — no gaps, repeats, \
         or replays from the start"
    );
    assert_eq!(events[0].2["work_id"], work_id.as_str());
    assert_eq!(events[2].2["work_id"], second_work_id.as_str());
    drop(resumed);

    handle.shutdown().await;
}

/// Graceful shutdown must not wait on a body that never ends. A live SSE tail
/// is the steady state for any monitoring client, and axum's graceful
/// shutdown waits for in-flight responses — so unless the daemon force-closes
/// its streams, one attached tail means `daemon.stopped` is never journaled,
/// the descriptor is never removed, and every later client lands in the
/// ambiguous "PID alive but endpoint unresponsive" fail-closed branch.
#[tokio::test]
async fn shutdown_completes_with_a_live_sse_client_attached() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();
    let path = daemon::descriptor_path(dir.path());

    let mut stream = http
        .get(format!("{}/v1/events/stream", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("sse connect");
    assert_eq!(stream.status(), 200);
    // Read the replayed startup frames (daemon.started, then one
    // backend.probed per registered backend): the pump is genuinely attached
    // and streaming, not merely requested.
    let startup = journal_events(dir.path()).len();
    assert_eq!(
        read_sse_events(&mut stream, startup).await.len(),
        startup,
        "the SSE client must be attached before shutdown"
    );

    tokio::time::timeout(Duration::from_secs(15), handle.shutdown())
        .await
        .expect("shutdown must not block on a live SSE tail");

    assert!(
        !path.exists(),
        "descriptor must be removed even with an SSE client attached"
    );
    let kinds: Vec<String> = journal_events(dir.path())
        .into_iter()
        .map(|e| e.kind)
        .collect();
    assert!(
        kinds.contains(&daemon::KIND_DAEMON_STOPPED.to_string()),
        "daemon.stopped must be journaled even with an SSE client attached, got {kinds:?}"
    );

    // The stream itself ends rather than dangling on a dead daemon.
    let closed = tokio::time::timeout(Duration::from_secs(5), stream.chunk())
        .await
        .expect("the SSE stream must be closed by shutdown, not left open");
    assert!(
        matches!(closed, Ok(None) | Err(_)),
        "the stream must end, not deliver more frames: {closed:?}"
    );

    // And the data dir is reusable immediately: the lock went with the daemon.
    let successor = start(dir.path()).await;
    successor.shutdown().await;
}

/// Run the sgt binary with args against a data dir; capture output.
///
/// The child runs in the data dir, not in whatever directory `cargo test` was
/// invoked from. From M3 on, `sgt run` sends its working directory as §13
/// origin metadata and the daemon discovers a workspace from it — so a test
/// that inherited the crate's own checkout would materialize real git
/// worktrees off this repository. These M2 tests are about the daemon, the
/// API and the CLI transport; the data dir is a plain temp dir with no
/// repository, so `sgt run` submits work that stays `pending`, exactly as it
/// did when no engine existed. Work surfaces get their own tests in
/// `m3_execution.rs`, in temp repositories built for the purpose.
/// The parameter is a [`DataDir`], not a `&Path`, on purpose: running the
/// binary against a data dir may auto-spawn a detached daemon, and the guard
/// is the thing that reaps it. Taking a bare path here is how a future test
/// would leak one without noticing.
fn sgt(data_dir: &DataDir, args: &[&str]) -> Output {
    std::process::Command::new(SGT)
        .current_dir(data_dir.path())
        .arg("--data-dir")
        .arg(data_dir.path())
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

/// The reaper, measured rather than assumed.
///
/// An instrument nobody has tried is a claim: this runs the auto-spawn path
/// the leak came from, checks that a daemon really is there, and then checks
/// that the guard's own reap removes it. If `DataDir` ever stops finding the
/// daemons it is supposed to kill — a changed command line, a `/proc` that is
/// not there — this fails here instead of quietly resuming the accumulation
/// (89 live daemons on one container, measured, before the guard existed).
///
/// It also pins *how* the reaper stops what it finds. The guard has always
/// escalated SIGTERM → SIGKILL, but it used to report only pids, so a daemon
/// that slept through the ten-second grace was reaped identically to one that
/// shut down cleanly — the difference showed up nowhere. It is not cosmetic:
/// a SIGKILLed process runs nothing at exit, so under instrumentation it
/// contributes no coverage profile, and "the numbers are low" would have been
/// the only symptom. The reaper now names the signal each pid needed, and
/// this test asserts the healthy answer: SIGTERM was enough.
#[test]
fn the_data_dir_guard_reaps_the_daemon_a_client_command_spawns() {
    let dir = DataDir::new();
    let output = sgt(&dir, &["run", "auto-spawn a daemon to reap"]);
    assert!(
        output.status.success(),
        "sgt run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let spawned = dir.daemon_pids();
    assert_eq!(
        spawned.len(),
        1,
        "the client command must have auto-spawned exactly one daemon on this data          dir (found {spawned:?}) — if it spawned none, the rest of this test proves          nothing"
    );
    assert!(
        daemon::pid_alive(spawned[0]),
        "the scan must report a live pid, not a leftover /proc entry"
    );

    let reaped = dir.reap();
    assert_eq!(
        reaped.iter().map(|daemon| daemon.pid).collect::<Vec<_>>(),
        spawned,
        "the guard must reap what it found"
    );
    assert_eq!(
        reaped
            .iter()
            .map(|daemon| daemon.signal)
            .collect::<Vec<_>>(),
        vec![ReapSignal::Term],
        "the reaper must report the signal each daemon needed, and a daemon that handles \
         SIGTERM must need only that one: a reported {kill} means either the daemon slept \
         through the {grace}s grace or the reaper stopped asking politely, and both are \
         losses — nothing registered at exit runs after SIGKILL",
        kill = ReapSignal::Kill,
        grace = 10,
    );
    assert!(
        dir.daemon_pids().is_empty(),
        "after the reap nothing may still be running on this data dir"
    );
    // …and the guard's `Drop` then finds nothing to complain about, which is
    // the assertion every other test in this file gets for free.
}

#[test]
fn t7_cli_end_to_end_auto_spawn_and_second_daemon_fails_closed() {
    let dir = DataDir::new();

    // No daemon running: `sgt run` auto-spawns one and submits.
    let output = sgt(&dir, &["run", "ship the M2 milestone"]);
    assert!(
        output.status.success(),
        "sgt run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // `sgt work list --json` shows it.
    let output = sgt(&dir, &["work", "list", "--json"]);
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
    let output = sgt(&dir, &["daemon"]);
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

/// The rest of the contracted CLI surface through the spawned binary:
/// `sgt status`, `sgt work show <id>`, `sgt cancel <id>`, in both human and
/// `--json` form. Everything here talks to a real daemon over the loopback
/// API — no in-process shortcuts.
#[test]
fn t7b_cli_status_show_and_cancel_through_the_binary() {
    let dir = DataDir::new();

    // Auto-spawn + submit, reading the id straight out of --json.
    let output = sgt(&dir, &["--json", "run", "inspect me"]);
    assert!(
        output.status.success(),
        "sgt run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let submitted: Value = serde_json::from_slice(&output.stdout).expect("run --json prints JSON");
    let work_id = submitted["work"]["id"]
        .as_str()
        .expect("submitted work id")
        .to_string();

    // `sgt status`, human: daemon health plus the counts.
    let output = sgt(&dir, &["status"]);
    assert!(
        output.status.success(),
        "sgt status failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("daemon ok"), "status said: {stdout}");
    assert!(stdout.contains("work: 1 total"), "status said: {stdout}");
    assert!(stdout.contains("pending: 1"), "status said: {stdout}");

    // `sgt status --json`: the same facts, machine-shaped.
    let output = sgt(&dir, &["status", "--json"]);
    assert!(output.status.success());
    let status: Value = serde_json::from_slice(&output.stdout).expect("status --json prints JSON");
    assert_eq!(status["work_total"], 1);
    assert_eq!(status["work_by_state"]["pending"], 1);
    assert_eq!(status["system"]["api_revision"], "v1");
    assert_eq!(
        status["system"]["data_dir"].as_str(),
        Some(dir.path().to_string_lossy().as_ref())
    );

    // `sgt work show <id>`: human and --json, both naming the work.
    let output = sgt(&dir, &["work", "show", &work_id]);
    assert!(output.status.success());
    let shown: Value =
        serde_json::from_slice(&output.stdout).expect("work show prints the record as JSON");
    assert_eq!(shown["id"].as_str(), Some(work_id.as_str()));
    assert_eq!(shown["state"], "pending");
    assert_eq!(shown["intent"], "inspect me");

    let output = sgt(&dir, &["work", "show", &work_id, "--json"]);
    assert!(output.status.success());
    let shown: Value = serde_json::from_slice(&output.stdout).expect("show --json prints JSON");
    assert_eq!(shown["work"]["id"].as_str(), Some(work_id.as_str()));

    // `sgt cancel <id>`: reports the new state, and it sticks.
    let output = sgt(&dir, &["cancel", &work_id]);
    assert!(
        output.status.success(),
        "sgt cancel failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("canceled") && stdout.contains(&work_id),
        "cancel said: {stdout}"
    );
    let output = sgt(&dir, &["work", "show", &work_id, "--json"]);
    let shown: Value = serde_json::from_slice(&output.stdout).expect("show --json");
    assert_eq!(shown["work"]["state"], "canceled");

    // A server-side error reaches the user as a nonzero exit and the
    // daemon's own structured message, not a panic or a silent success.
    let output = sgt(&dir, &["work", "show", "01AN4Z07BY79KA1307SR9X4MV3"]);
    assert!(!output.status.success(), "unknown id must exit nonzero");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no work with id"),
        "the daemon's diagnostic must reach the user, got: {stderr}"
    );

    stop_daemon(dir.path());
}

#[test]
fn t8_two_concurrent_auto_spawns_one_survivor_both_commands_complete() {
    let dir = DataDir::new();
    // Scoped threads so both racers share the one guarded data dir rather
    // than a copy of its path: whichever daemon survives the race, the guard
    // that reaps it is the same object.
    let (out_a, out_b) = std::thread::scope(|scope| {
        let a = scope.spawn(|| sgt(&dir, &["run", "racer A"]));
        let b = scope.spawn(|| sgt(&dir, &["run", "racer B"]));
        (a.join().expect("thread A"), b.join().expect("thread B"))
    });
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
    let output = sgt(&dir, &["work", "list", "--json"]);
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
    let dir = DataDir::new();
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
    let output = sgt(&dir, &["run", "after stale descriptor"]);
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
    let dir = DataDir::new();
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
    let output = sgt(&dir, &["run", "must fail closed"]);
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

/// The descriptor's `schema` field is a promise, not a decoration: a
/// descriptor written by a build this one does not understand must stop the
/// client dead — the same fail-closed rule an unknown snapshot schema gets.
/// Half-interpreting it could mean talking to the wrong process, or spawning
/// a second daemon on a data dir that already has an owner.
#[test]
fn descriptor_with_an_unknown_schema_fails_closed() {
    let dir = DataDir::new();
    std::fs::create_dir_all(dir.path()).expect("data dir");
    let from_the_future = json!({
        "schema": "sergeant.runtime/v2",
        "endpoint": "http://127.0.0.1:1",
        "pid": std::process::id(),
        "api_revision": "v2",
        "token": "from-the-future",
    });
    let path = daemon::descriptor_path(dir.path());
    std::fs::write(&path, serde_json::to_vec(&from_the_future).expect("json"))
        .expect("write future descriptor");

    let output = sgt(&dir, &["work", "list"]);
    assert!(
        !output.status.success(),
        "an unknown descriptor schema must fail closed, got: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown schema") && stderr.contains("sergeant.runtime/v2"),
        "the diagnostic must name the schema it refused, got: {stderr}"
    );
    // It refused *before* acting: no daemon was spawned (spawning is what
    // creates daemon.log) and the descriptor is untouched.
    assert!(
        !dir.path().join("daemon.log").exists(),
        "a refused descriptor must not spawn a daemon"
    );
    assert_eq!(
        std::fs::read(&path).expect("descriptor still there"),
        serde_json::to_vec(&from_the_future).expect("json"),
    );
}

/// Two clients replacing the *same* stale descriptor must not leave the
/// winner's daemon undiscoverable. The endpoint here accepts connections and
/// never answers, so each client spends its full health-probe timeout in the
/// window during which the other client's daemon publishes a fresh
/// descriptor — the window in which a client that unlinks the path deletes a
/// successor's record and wedges the data dir for good.
#[test]
fn concurrent_stale_replacement_leaves_the_surviving_daemon_discoverable() {
    let dir = DataDir::new();
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

    let (out_a, out_b) = std::thread::scope(|scope| {
        let a = scope.spawn(|| sgt(&dir, &["run", "stale racer A"]));
        std::thread::sleep(Duration::from_millis(300));
        let b = scope.spawn(|| sgt(&dir, &["run", "stale racer B"]));
        (a.join().expect("thread A"), b.join().expect("thread B"))
    });
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
    let output = sgt(&dir, &["work", "list", "--json"]);
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
