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

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend};
use sergeant_rs::daemon::{self, DaemonConfig, DaemonHandle, RuntimeDescriptor};
use sergeant_rs::domain::event::{EVENT_SCHEMA, Event};
use sergeant_rs::domain::work::{
    KIND_COMMAND_ACCEPTED, KIND_COMMAND_REJECTED, KIND_WORK_BLOCKED, KIND_WORK_CANCELED,
    KIND_WORK_SUBMITTED, WorkState,
};
use sergeant_rs::domain::workflow::{KIND_STAGE_ENTERED, KIND_WORKFLOW_BOUND};
use sergeant_rs::runtime::journal::Journal;
use sergeant_rs::runtime::surface::KIND_SURFACE_MATERIALIZED;

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

/// Seed a data dir's journal with a work in `state` plus a *minimal run* — a
/// `workflow.bound` naming `backend`, a zero-binding `surface.materialized`
/// (`enter_stage` requires `run.surface` to be `Some`, same as
/// `run.workflow`/`run.backend` — `NoRun` covers all three, not only "no run
/// row at all"), and a `stage.entered` for `stage_id` — so `retry`/`input`
/// can reach the engine's run-bearing code paths without a real repository
/// or workflow files. M2 has no repo machinery (see the note by `fn sgt`
/// below); a run's evidence is planted the same way `seed_work_in_state`
/// plants work state. The empty `bindings` is what keeps `retry`'s own
/// surface-rematerialize check a no-op (`bindings.iter().any(..)` over
/// nothing is `false`) and gives `enter_stage`'s `execution_cwd()` a root to
/// fall back to without any worktree needing to exist.
fn seed_run_in_state(dir: &Path, work_id: &str, state: &str, backend: &str, stage_id: &str) {
    let journal = dir.join("journal");
    std::fs::create_dir_all(&journal).expect("journal dir");
    let events = [
        json!({
            "schema": EVENT_SCHEMA,
            "seq": 1,
            "id": ulid(),
            "timestamp": "2026-08-08T00:00:00.000Z",
            "source": {"type": "daemon", "name": "api"},
            "work_id": work_id,
            "kind": KIND_WORK_SUBMITTED,
            "payload": {"work": {
                "id": work_id,
                "intent": "seeded work with a run",
                "state": state,
                "created_by": "test",
                "created_at": "2026-08-08T00:00:00.000Z",
            }},
        }),
        json!({
            "schema": EVENT_SCHEMA,
            "seq": 2,
            "id": ulid(),
            "timestamp": "2026-08-08T00:00:01.000Z",
            "source": {"type": "daemon", "name": "engine"},
            "work_id": work_id,
            "kind": KIND_WORKFLOW_BOUND,
            "payload": {
                "workflow": {
                    "name": "seeded",
                    "version": "1",
                    "source": "embedded",
                    "stages": [{"id": stage_id, "context": "seeded stage"}],
                },
                "backend": backend,
                "route_source": "explicit",
                "profile": Value::Null,
            },
        }),
        json!({
            "schema": EVENT_SCHEMA,
            "seq": 3,
            "id": ulid(),
            "timestamp": "2026-08-08T00:00:02.000Z",
            "source": {"type": "daemon", "name": "engine"},
            "work_id": work_id,
            "kind": KIND_SURFACE_MATERIALIZED,
            "payload": {
                "surface": {"work_id": work_id, "root": dir.join("surfaces").join(work_id), "bindings": []},
            },
        }),
        json!({
            "schema": EVENT_SCHEMA,
            "seq": 4,
            "id": ulid(),
            "timestamp": "2026-08-08T00:00:03.000Z",
            "source": {"type": "daemon", "name": "engine"},
            "work_id": work_id,
            "kind": KIND_STAGE_ENTERED,
            "payload": {"stage_id": stage_id, "index": 0, "attempt": 1},
        }),
    ];
    let mut buf = Vec::new();
    for event in events {
        let mut line = serde_json::to_vec(&event).expect("seed json");
        line.push(b'\n');
        buf.extend(line);
    }
    std::fs::write(journal.join("00000001.ndjson"), buf).expect("write seed segment");
}

/// POST a JSON body to an arbitrary path; returns (status, exact response
/// bytes, parsed body) — the same triple `submit`/`cancel` return, for the
/// routes those two don't cover (`/input`, `/retry`).
async fn post_json(
    http: &reqwest::Client,
    handle: &DaemonHandle,
    path: &str,
    body: Value,
) -> (reqwest::StatusCode, Vec<u8>, Value) {
    let resp = http
        .post(format!("{}{path}", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&body)
        .send()
        .await
        .expect("request");
    let status = resp.status();
    let bytes = resp.bytes().await.expect("body").to_vec();
    let value: Value = serde_json::from_slice(&bytes).expect("json body");
    (status, bytes, value)
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
/// this test asserts the healthy answer: SIGTERM was enough — first from the
/// reaper's report, then, because a report about oneself is not evidence,
/// from what the daemon left behind on its way out.
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

    // The assertion above is the reaper's own label — what it *says* it sent.
    // On its own that is self-grading, and measurably so: with
    // `ReapSignal::flag()`'s `Term` arm changed to `-KILL`, the reaper still
    // builds `ReapedDaemon { signal: Term }` and the label assertion still
    // passes while every daemon dies of SIGKILL. So the evidence below is
    // read from the *daemon* instead. Measured on this container, one temp
    // data dir per signal: after SIGTERM the daemon runs its shutdown tail —
    // journals `daemon.stopped`, removes `runtime.json` — and after SIGKILL
    // it journals nothing and leaves the descriptor on disk.
    assert!(
        !daemon::descriptor_path(dir.path()).exists(),
        "the reaped daemon must have removed its own runtime descriptor, which only \
         the SIGTERM shutdown path does. Still present means the daemon was killed \
         rather than asked, whatever the report above says it sent."
    );
    let stopped = Journal::replay_data_dir(dir.path())
        .expect("replay the journal")
        .filter_map(Result::ok)
        .filter(|event| event.kind == daemon::KIND_DAEMON_STOPPED)
        .count();
    assert_eq!(
        stopped,
        1,
        "the reaped daemon must have journaled exactly one {} event; {stopped} means \
         the signal it actually received was not one it could handle",
        daemon::KIND_DAEMON_STOPPED
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

// --------------------------------------------------------- #36: input/retry

/// §26 binds `/input` and `/retry` exactly like `/work` and `/cancel`
/// (`malformed_bodies_command_ids_and_resume_headers_are_structured_errors`
/// covers those two): an unparseable body and a `command_id` that is not a
/// ULID must answer structured 4xx errors on these routes too, and must
/// leave nothing in the journal — there is no command identity to record
/// them under.
#[tokio::test]
async fn input_and_retry_reject_malformed_bodies_and_invalid_command_ids() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();
    let work_id = "01AN4Z07BY79KA1307SR9X4MV3";
    let before = event_count(&http, &handle).await;

    // Not a ULID → 400 invalid_command_id, on both routes.
    for (path, body) in [
        (
            format!("/v1/work/{work_id}/input"),
            json!({"command_id": "nope", "input": "hi"}),
        ),
        (
            format!("/v1/work/{work_id}/retry"),
            json!({"command_id": "nope"}),
        ),
    ] {
        let (status, _, body_json) = post_json(&http, &handle, &path, body).await;
        assert_eq!(status, 400, "expected 400 for {path}");
        assert_eq!(
            body_json["error"]["code"], "invalid_command_id",
            "{path}: {body_json}"
        );
    }

    // Unparseable and mis-shaped bodies → structured 4xx invalid_request.
    for (path, body) in [
        (format!("/v1/work/{work_id}/input"), "{ not json"),
        (
            format!("/v1/work/{work_id}/input"),
            r#"{"command_id": "01AN4Z07BY79KA1307SR9X4MV3"}"#, // no `input`
        ),
        (format!("/v1/work/{work_id}/retry"), "]"),
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

    assert_eq!(
        event_count(&http, &handle).await,
        before,
        "unkeyable requests on /input and /retry must append no events"
    );
    handle.shutdown().await;
}

/// The idempotency guarantee (§26) covers `/input` and `/retry` exactly like
/// `/work` and `/cancel`: a repeated `command_id` — even for a *rejected*
/// outcome — must replay the recorded result byte-identical, appending no
/// further events, rather than being re-validated (mirrors
/// `t4_duplicate_command_id_is_byte_identical_even_across_restart` and
/// `rejected_submit_is_journaled_and_replayed`). A freshly submitted work is
/// `pending`, which is neither `needs_input` nor retryable, so both routes
/// answer a recorded 409 — exactly the kind of outcome §26 says must replay,
/// not just a 2xx.
#[tokio::test]
async fn input_and_retry_duplicate_command_id_replays_the_recorded_outcome() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();
    let (_, _, body) = submit(&http, &handle, &ulid(), "not waiting on anything").await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "pending");

    for path in [
        format!("/v1/work/{work_id}/input"),
        format!("/v1/work/{work_id}/retry"),
    ] {
        let command_id = ulid();
        let request_body = if path.ends_with("/input") {
            json!({"command_id": command_id, "input": "nobody asked"})
        } else {
            json!({"command_id": command_id})
        };
        let (s1, b1, body1) = post_json(&http, &handle, &path, request_body.clone()).await;
        assert_eq!(
            s1, 409,
            "a pending work is neither awaiting input nor retryable: {body1}"
        );

        let before = event_count(&http, &handle).await;
        let (s2, b2, _) = post_json(&http, &handle, &path, request_body).await;
        assert_eq!(
            s2, s1,
            "the duplicate must replay the original status: {path}"
        );
        assert_eq!(b1, b2, "the duplicate must be byte-identical: {path}");
        assert_eq!(
            event_count(&http, &handle).await,
            before,
            "a duplicate must append no events: {path}"
        );
    }
    handle.shutdown().await;
}

/// `EngineError::NoRun` answers 404 through `engine_error_status`: a work
/// whose state permits `retry` (`failed`) but whose run never actually
/// started. Nothing in M2's own API can produce this shape — every retryable
/// work here has a run — so it is planted the same way `seed_work_in_state`
/// plants other states the transition table would otherwise refuse to
/// reach: directly in the journal the daemon replays.
#[tokio::test]
async fn retry_on_a_retryable_state_with_no_run_is_a_404_no_run() {
    let dir = TempDir::new().expect("tempdir");
    let work_id = ulid();
    seed_work_in_state(dir.path(), &work_id, "failed");
    let handle = start(dir.path()).await;
    let http = client();

    let (status, _, body) = post_json(
        &http,
        &handle,
        &format!("/v1/work/{work_id}/retry"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 404, "retry on a run that never started: {body}");
    assert_eq!(body["error"]["code"], "no_run");
    handle.shutdown().await;
}

/// The `/input` half of the same shape: `needs_input` permits `provide_input`
/// past its state check, but with no run seeded the lookup underneath it is
/// still `EngineError::NoRun` → 404, not the `work_not_found` this route also
/// answers (that one is a pre-engine check on the *work*, not the *run*).
#[tokio::test]
async fn input_on_a_needs_input_state_with_no_run_is_a_404_no_run() {
    let dir = TempDir::new().expect("tempdir");
    let work_id = ulid();
    seed_work_in_state(dir.path(), &work_id, "needs_input");
    let handle = start(dir.path()).await;
    let http = client();

    let (status, _, body) = post_json(
        &http,
        &handle,
        &format!("/v1/work/{work_id}/input"),
        json!({"command_id": ulid(), "input": "postgres"}),
    )
    .await;
    assert_eq!(status, 404, "input on a run that never started: {body}");
    assert_eq!(body["error"]["code"], "no_run");
    handle.shutdown().await;
}

/// A backend that cannot START the next attempt fails the retry closed to
/// `blocked` — the same §25 fail-closed shape `state.engine.start()` takes
/// inside `submit_work` (`engine_error_status`'s sibling caller), reached
/// here through `retry` instead because M2's own submissions never carry a
/// repository (see the note by `fn sgt` below), so `submit_work` never
/// reaches `state.engine.start()` at all in this file.
///
/// `retry` is what makes this expressible with the shared `FakeBackend`
/// without extending its script grammar: unlike `submit`'s `plan()`/`route()`
/// (which calls `backend.probe()`), `Engine::retry` never re-probes — it goes
/// straight from the seeded run to `enter_stage`'s `backend.start()`. So
/// `fake.set_available(false, …)` set for the whole test fails START without
/// ever needing PROBE to have answered differently first. Building this on
/// `submit` instead is not possible with the fake as it stands: PROBE and
/// START both gate on the same `available` flag, so toggling it around a
/// submit either fails routing too (never reaching "post-accept") or never
/// fails START at all — there is no `.await` between `plan()` and `start()`
/// for a test to interleave a toggle into, and the fake is deliberately
/// unraced by construction.
#[tokio::test]
async fn retry_reenters_the_stage_and_fails_closed_when_the_backend_cannot_start() {
    let dir = TempDir::new().expect("tempdir");
    let work_id = ulid();
    seed_run_in_state(
        dir.path(),
        &work_id,
        "failed",
        FAKE_BACKEND_NAME,
        "00-first",
    );

    let fake = FakeBackend::scripted(FAKE_BACKEND_NAME, []);
    fake.set_available(false, "the harness is gone");
    let registry = BackendRegistry::new().with(Arc::new(fake.clone()));
    let handle = daemon::start_with(
        dir.path(),
        DaemonConfig {
            backends: Arc::new(registry),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            claude: None,
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");
    let http = client();

    let (status, _, body) = post_json(
        &http,
        &handle,
        &format!("/v1/work/{work_id}/retry"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(
        status, 200,
        "a post-accept start failure fails the work closed, it does not error the \
         request: {body}"
    );
    assert_eq!(body["work"]["state"], "blocked");
    assert!(
        fake.starts().is_empty(),
        "an unavailable backend must never actually record a start"
    );

    let events = journal_events(dir.path());
    let blocked: Vec<_> = events
        .iter()
        .filter(|e| e.work_id.as_deref() == Some(work_id.as_str()) && e.kind == KIND_WORK_BLOCKED)
        .collect();
    assert_eq!(blocked.len(), 1, "the failure must be journaled");
    assert_eq!(
        blocked[0].payload["reason"], "backend could not start an execution",
        "{blocked:?}"
    );

    handle.shutdown().await;
}

// -------------------------------------------------------------- #37: SSE races

/// A history replay that fails partway through the requested tail must close
/// the stream, not deliver a truncated tail and pretend it was complete —
/// `Core::events_after`'s `?` inside its collection loop discards whatever it
/// had already gathered, so `forward_events` sees a bare `Err` and must send
/// nothing at all. `Journal::replay_after` re-lists segments from disk on
/// every call, so a line appended straight to the segment file (bypassing
/// the daemon entirely) is visible on the very next request — no restart
/// needed to reach it, unlike the M1 corruption fixtures this mirrors.
#[tokio::test]
async fn sse_history_replay_error_mid_tail_closes_the_stream_without_daemon_damage() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();

    // A real tail worth replaying.
    submit(&http, &handle, &ulid(), "before the corruption").await;

    // Corrupt the tail in place: a well-formed JSON line that is not a
    // well-formed Event, the way a torn or bit-flipped append would look on
    // replay.
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

    // A subscriber asking for the tail now hits the malformed line mid-way
    // through replay: the handshake still succeeds, but no frames follow.
    let mut stream = http
        .get(format!("{}/v1/events/stream?from=0", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("sse connect");
    assert_eq!(
        stream.status(),
        200,
        "the SSE handshake itself still succeeds"
    );
    let closed = tokio::time::timeout(Duration::from_secs(5), stream.chunk())
        .await
        .expect("a broken history replay must close the stream promptly, not hang");
    assert!(
        matches!(closed, Ok(None) | Err(_)),
        "no frames must be delivered once the replay itself failed: {closed:?}"
    );

    // The daemon itself is undamaged: the core lock is released around the
    // failed replay (it is a scoped block, not held across it), so a fresh
    // request against unaffected state still answers normally.
    let health = http
        .get(format!("{}/healthz", handle.endpoint))
        .send()
        .await
        .expect("healthz after corrupted replay");
    assert_eq!(health.status(), 200);

    handle.shutdown().await;
}

/// A subscriber that disconnects *while history is still being replayed* —
/// not after it, which `t6`'s `drop(stream)` already covers — must not leave
/// the resume contract confused: a fresh connection resuming from exactly
/// the one frame the dead subscriber saw must get exactly the rest, no gap
/// and no repeat of the frame already delivered, and the daemon itself must
/// still answer normally afterward.
#[tokio::test]
async fn sse_resume_after_a_disconnect_mid_history_replay_yields_exactly_the_remaining_tail() {
    let dir = TempDir::new().expect("tempdir");
    let handle = start(dir.path()).await;
    let http = client();

    for i in 0..5 {
        submit(&http, &handle, &ulid(), &format!("backlog {i}")).await;
    }
    let all = history_seqs(&http, &handle, None).await;
    assert!(
        all.len() > 1,
        "there must be more than one event to be mid-replay of"
    );

    // Connect from the very start and read only the first history frame —
    // proof replay is genuinely under way, with more of it still queued —
    // then abandon the connection without reading the rest.
    let mut first = http
        .get(format!("{}/v1/events/stream?from=0", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("sse connect");
    let events = read_sse_events(&mut first, 1).await;
    assert_eq!(events.len(), 1, "the first history frame must arrive");
    let last_seen = events[0].0;
    assert!(
        last_seen < *all.last().expect("history"),
        "the disconnect must land while replay still has more queued, not at the end"
    );
    drop(first); // the subscriber vanishes mid-replay, not after it

    let mut resumed = http
        .get(format!("{}/v1/events/stream", handle.endpoint))
        .header("Last-Event-ID", last_seen.to_string())
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("sse resume");
    let remaining = all.len() - last_seen as usize;
    let events = read_sse_events(&mut resumed, remaining).await;
    let seqs: Vec<u64> = events.iter().map(|(seq, _, _)| *seq).collect();
    assert_eq!(
        seqs,
        all[last_seen as usize..].to_vec(),
        "resuming after a mid-replay disconnect must replay exactly the tail the dead \
         subscriber never got, no more and no less"
    );
    drop(resumed);

    let (status, _, _) = submit(&http, &handle, &ulid(), "after the abandoned replay").await;
    assert_eq!(
        status, 201,
        "the daemon must stay responsive after a subscriber vanishes mid-replay"
    );
    handle.shutdown().await;
}

// ---------------------------------------------------------- #38: cli surfaces

/// `sgt work list`'s human-readable form (`--json`'s shape is covered
/// elsewhere in this file): nothing submitted prints the empty-state line,
/// not a blank table, and each submitted work prints one line naming its id,
/// state and intent.
#[test]
fn work_list_human_form_prints_the_empty_and_populated_branches() {
    let dir = DataDir::new();

    let output = sgt(&dir, &["work", "list"]);
    assert!(
        output.status.success(),
        "sgt work list: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "no work",
        "an empty fleet must print the empty-state line, not a blank table"
    );

    let submitted = sgt(&dir, &["--json", "run", "list me in human form"]);
    assert!(submitted.status.success());
    let body: Value = serde_json::from_slice(&submitted.stdout).expect("run --json");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();

    let output = sgt(&dir, &["work", "list"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&work_id)
            && stdout.contains("pending")
            && stdout.contains("list me in human form"),
        "each work must print one line naming its id, state and intent: {stdout}"
    );

    stop_daemon(dir.path());
}

/// `sgt retry`'s human-readable success print (`print_work_line`): the same
/// `"retried <id> (<state>)"` shape `submit`/`respond` already use, reached
/// through a *real* CLI-spawned daemon (the compiled-in default registry's
/// unscripted `fake` backend completes every stage it is given) so the
/// printed state is the daemon's own answer, not a fixture standing in for
/// it.
#[test]
fn retry_success_prints_the_human_readable_line() {
    let dir = DataDir::new();
    let work_id = ulid();
    seed_run_in_state(
        dir.path(),
        &work_id,
        "failed",
        FAKE_BACKEND_NAME,
        "00-first",
    );

    let output = sgt(&dir, &["retry", &work_id]);
    assert!(
        output.status.success(),
        "sgt retry: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&format!("retried {work_id} (completed)")),
        "a retried single-stage work that completes must print its new state: {stdout}"
    );

    stop_daemon(dir.path());
}

/// Reaps whatever daemon lands on `path` when dropped. `DataDir` ties its
/// reap to its *own* tempdir, which is exactly wrong for a spawn whose data
/// dir is resolved by the client's own env fallback (`SGT_DATA_DIR`,
/// `XDG_DATA_HOME`, `HOME`) rather than named directly — so this is the same
/// safety net, built from `support`'s own exported reaper, pointed at the
/// path the client actually resolved instead.
struct ReapOnDrop(PathBuf);

impl Drop for ReapOnDrop {
    fn drop(&mut self) {
        support::reap_daemons(&self.0);
    }
}

/// `resolve_data_dir`'s fallback chain, one link at a time, with no
/// `--data-dir` flag at all so the client's own env resolution is what is
/// under test rather than an override of it: `SGT_DATA_DIR` wins outright;
/// absent that, `$XDG_DATA_HOME/sergeant`; absent both, `$HOME/.local/share/sergeant`.
#[test]
fn resolve_data_dir_falls_back_through_sgt_data_dir_then_xdg_then_home() {
    // SGT_DATA_DIR wins even with XDG_DATA_HOME and HOME also set.
    let sgt_dir = TempDir::new().expect("tempdir");
    let xdg = TempDir::new().expect("tempdir");
    let home = TempDir::new().expect("tempdir");
    let _reap = ReapOnDrop(sgt_dir.path().to_path_buf());
    let output = Command::new(SGT)
        .args(["work", "list", "--json"])
        .env("SGT_DATA_DIR", sgt_dir.path())
        .env("XDG_DATA_HOME", xdg.path())
        .env("HOME", home.path())
        .output()
        .expect("run sgt with all three set");
    assert!(
        output.status.success(),
        "sgt work list: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        sgt_dir.path().join("journal").is_dir(),
        "SGT_DATA_DIR must win over XDG_DATA_HOME and HOME"
    );
    assert!(!xdg.path().join("sergeant").exists());
    assert!(!home.path().join(".local").exists());
    drop(_reap);

    // Absent SGT_DATA_DIR, `$XDG_DATA_HOME/sergeant` is next.
    let xdg2 = TempDir::new().expect("tempdir");
    let home2 = TempDir::new().expect("tempdir");
    let resolved = xdg2.path().join("sergeant");
    let _reap = ReapOnDrop(resolved.clone());
    let output = Command::new(SGT)
        .args(["work", "list", "--json"])
        .env_remove("SGT_DATA_DIR")
        .env("XDG_DATA_HOME", xdg2.path())
        .env("HOME", home2.path())
        .output()
        .expect("run sgt with XDG_DATA_HOME and HOME");
    assert!(
        output.status.success(),
        "sgt work list: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        resolved.join("journal").is_dir(),
        "XDG_DATA_HOME/sergeant must be used"
    );
    assert!(!home2.path().join(".local").exists());
    drop(_reap);

    // Absent both, `$HOME/.local/share/sergeant` is the last resort.
    let home3 = TempDir::new().expect("tempdir");
    let resolved = home3.path().join(".local/share/sergeant");
    let _reap = ReapOnDrop(resolved.clone());
    let output = Command::new(SGT)
        .args(["work", "list", "--json"])
        .env_remove("SGT_DATA_DIR")
        .env_remove("XDG_DATA_HOME")
        .env("HOME", home3.path())
        .output()
        .expect("run sgt with only HOME");
    assert!(
        output.status.success(),
        "sgt work list: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        resolved.join("journal").is_dir(),
        "HOME/.local/share/sergeant must be used"
    );
}
