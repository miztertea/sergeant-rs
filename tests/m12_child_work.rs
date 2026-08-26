//! M12: child Work — causation transport, validation, independence, and
//! `sgt run --wait` (S2 W1 §5–§8/§13.5–§13.7; decisions E5/E6/E7/E8-amended/E9).
//!
//! The helper that builds the causation triple is unit-tested next to its own
//! code in `src/backend/mod.rs`; each adapter's own env-capture suite proves
//! the triple reaches *that adapter's* spawned process. This suite is the
//! other half — the claim end to end, through the real daemon, the real
//! engine and the real `sgt` binary:
//!
//! 1. the engine hands every managed execution the estate coordinate a child
//!    `sgt -C … run` needs (E5, the plumbing half);
//! 2. an actor process really does inherit all three values and can spend
//!    them on a child submission that the daemon validates (W1 §13.5);
//! 3. a forged or stale claim is **accepted** and journaled as unverified
//!    rather than refused (E8 as amended — the ratification's "rather than by
//!    refusal" clause);
//! 4. the child is independent: parent completion and parent cancellation do
//!    not cascade, scope is not inherited, branches differ (W1 §13.6/§13.7);
//! 5. `--wait` observes a child to terminal without any engine hold state
//!    (E9);
//! 6. the supersession is narrow: a bare `sgt run` from inside a Work surface
//!    still refuses.

use std::path::Path;
use std::sync::Arc;

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig, DaemonHandle};

mod support;

// ---------------------------------------------------------------- helpers

fn ulid() -> String {
    ulid::Ulid::generate().to_string()
}

fn http() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("client")
}

/// A daemon over the fake backend, plus the adapter handle itself — the
/// `StartRequest`s it recorded are how this suite reads what the engine
/// actually handed a managed execution.
async fn start_fake(
    data_dir: &Path,
    script: impl IntoIterator<Item = FakeStep>,
) -> (DaemonHandle, Arc<FakeBackend>) {
    let fake = Arc::new(FakeBackend::scripted(FAKE_BACKEND_NAME, script));
    let registry = BackendRegistry::new().with(fake.clone());
    let handle = daemon::start_with(
        data_dir,
        DaemonConfig {
            backends: Arc::new(registry),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            claude: None,
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");
    (handle, fake)
}

async fn post(
    client: &reqwest::Client,
    handle: &DaemonHandle,
    path: &str,
    body: Value,
) -> (reqwest::StatusCode, Value) {
    let resp = client
        .post(format!("{}{path}", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&body)
        .send()
        .await
        .expect("request");
    let status = resp.status();
    let value: Value = resp.json().await.expect("json body");
    (status, value)
}

/// Submit one Work against `estate`, with whatever extra top-level keys the
/// caller wants folded into the body (the claimed-causation fields, for the
/// tests that forge them).
async fn submit_with(
    client: &reqwest::Client,
    handle: &DaemonHandle,
    estate: &Path,
    intent: &str,
    extra: Value,
) -> (reqwest::StatusCode, Value) {
    let mut body = json!({
        "command_id": ulid(),
        "intent": intent,
        "estate_root": estate,
        "origin": {"client": "cli", "cwd": estate},
    });
    if let Some(extra) = extra.as_object() {
        for (key, value) in extra {
            body[key] = value.clone();
        }
    }
    post(client, handle, "/v1/work", body).await
}

// ------------------------------------------------------------------ tests

/// E5, the plumbing half: every managed execution is handed the canonical
/// estate root of the Work it serves — the same coordinate the daemon stamps
/// on that Work's events — so `sgt -C "$SERGEANT_ESTATE_ROOT" run` addresses
/// the estate the daemon will validate the claim against.
///
/// Read off the `StartRequest` the engine actually built, not off a computed
/// value: this is the field the adapters' env merge reads from.
#[tokio::test]
async fn a_managed_execution_is_handed_the_estate_root_of_the_work_it_serves() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");

    let (handle, fake) = start_fake(data.path(), [FakeStep::complete_with("done")]).await;
    let client = http();
    let (status, body) = submit_with(&client, &handle, &estate, "parent work", json!({})).await;
    assert_eq!(status, 201, "submit must be accepted: {body}");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();

    let canonical = estate.canonicalize().expect("canonical estate");
    let starts = fake.starts();
    assert!(!starts.is_empty(), "the Work ran at least one stage");
    for start in &starts {
        assert_eq!(start.work_id, work_id);
        assert_eq!(
            start.estate_root.as_deref(),
            Some(canonical.as_path()),
            "every stage's request carries the Work's own canonical estate \
             root, not just the first: {start:?}"
        );
    }
    let start = &starts[0];

    // And the triple built from it is exactly the contract's three values,
    // carrying this execution's real coordinates — the helper is pure, so
    // this is the whole of what the adapters will merge.
    let env = sergeant_rs::backend::causation_env(start);
    assert_eq!(env["SERGEANT_ESTATE_ROOT"], canonical.to_string_lossy());
    assert_eq!(env["SERGEANT_WORK_ID"], work_id);
    assert_eq!(env["SERGEANT_EXECUTION_ID"], start.execution_id);

    handle.shutdown().await;
}
