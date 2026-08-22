//! W2 §2 (A1 routing verification): proof that `daemon::start_with` **itself**
//! — with no test-supplied stand-in for the name "codex" — now puts a real
//! `CodexBackend` in its registry, so the four-tier `RouteSource` chain
//! (already unit-tested in `src/runtime/router.rs` and already exercised
//! end-to-end with a *test-substituted* `FakeBackend::scripted("codex", ...)`
//! in `tests/m3_execution.rs`'s `t7_routing_precedence_and_structured_failure`)
//! reaches codex as the daemon's own default registration, not a fixture
//! wearing its name.
//!
//! Every test here deliberately configures codex with a nonexistent
//! executable, so the outcome is deterministic on any host (a host with a
//! real, logged-in `codex` on `PATH` must not make these flaky or spend
//! tokens): `resolve_tiers` calls `backend.probe()` before returning success,
//! so an unavailable codex makes every submission below fail with
//! `RouteError::Unavailable` — the *stronger*, host-independent proof that
//! each tier reached the registered adapter and stopped there (named,
//! `backend_unavailable`), rather than falling through to a populated global
//! default the way an unregistered "codex" silently did before this wave.
//!
//! In-process `daemon::start_with` + `reqwest`, no subprocess — the same rig
//! `tests/estate_routes.rs`/`tests/m3_execution.rs`/`tests/codex_backend.rs`
//! use, so `tests/support::DataDir`'s reaper does not apply here either.
//!
//! No new `RouteSource` variant, no new routing logic — `src/runtime/
//! router.rs` is untouched by this wave. No CLI-subprocess test of `sgt run
//! --backend codex` or `sgt codex` actually exec'ing either: `harness::
//! prepare`/`exec` are already unit-tested and exec-boundary functions are
//! not round-trippable through an integration test by design.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::codex::CodexConfig;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend};
use sergeant_rs::daemon::{self, DaemonConfig, DaemonHandle};

mod support;

// ---------------------------------------------------------------- helpers

fn ulid() -> String {
    ulid::Ulid::generate().to_string()
}

fn http() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .expect("client")
}

/// A `CodexConfig` pointed at a definitely-nonexistent executable, so the
/// probe always reports `available: false` regardless of what host runs
/// this suite.
fn deterministic_codex_config(data_dir: &Path) -> CodexConfig {
    CodexConfig {
        executable: PathBuf::from("/nonexistent/definitely-not-codex"),
        ..CodexConfig::new(data_dir)
    }
}

/// mirrors `tests/m3_execution.rs`'s `estate_with_manifest`: an estate at
/// `root` from a verbatim `sergeant.toml`, with a real git checkout at
/// `root/repos/<name>` for each of `repos`.
fn estate_with_manifest(root: &Path, manifest: &str, repos: &[&str]) {
    std::fs::create_dir_all(root).expect("estate root");
    for repo in repos {
        support::init_repo(&root.join("repos").join(repo));
    }
    std::fs::write(root.join("sergeant.toml"), manifest).expect("sergeant.toml");
}

/// Start a daemon with one scripted fake (the global default) and codex
/// registered for real, but deterministically unavailable.
async fn start(data_dir: &Path, estate_root: &Path, global_default: Option<&str>) -> DaemonHandle {
    daemon::start_with(
        data_dir,
        DaemonConfig {
            backends: Arc::new(
                BackendRegistry::new().with(Arc::new(FakeBackend::new(FAKE_BACKEND_NAME))),
            ),
            default_backend: global_default.map(str::to_string),
            codex: Some(deterministic_codex_config(data_dir)),
            estate_root: Some(estate_root.to_path_buf()),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start")
}

/// POST a JSON body to the daemon; returns status and parsed body.
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

/// GET a path from the daemon.
async fn get(client: &reqwest::Client, handle: &DaemonHandle, path: &str) -> Value {
    client
        .get(format!("{}{path}", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("request")
        .json()
        .await
        .expect("json body")
}

/// Submit work whose origin is `cwd`, merging any extra request fields.
async fn submit(
    client: &reqwest::Client,
    handle: &DaemonHandle,
    cwd: &Path,
    extra: Value,
) -> (reqwest::StatusCode, Value) {
    let mut body = json!({
        "command_id": ulid(),
        "intent": "route me",
        "origin": {"client": "cli", "cwd": cwd},
    });
    if let Some(fields) = extra.as_object() {
        for (key, value) in fields {
            body[key] = value.clone();
        }
    }
    post(client, handle, "/v1/work", body).await
}

// -------------------------------------------------------------------- tests

/// (a) Explicit `--backend codex` resolves to the registered adapter, not
/// `backend_not_found`: codex is *known and named*, and its own probe
/// evidence is what fails the submission.
#[tokio::test]
async fn explicit_backend_codex_reaches_the_registered_adapter_not_not_found() {
    let root = TempDir::new().expect("tempdir");
    support::scaffold_estate(root.path(), "plain", &["plain"]);
    let data = TempDir::new().expect("tempdir");
    let handle = start(data.path(), root.path(), Some(FAKE_BACKEND_NAME)).await;

    let (status, body) = submit(
        &http(),
        &handle,
        root.path(),
        json!({"backend": "codex", "origin": {"client": "cli", "cwd": root.path()}}),
    )
    .await;

    assert_eq!(status, 422);
    assert_eq!(
        body["error"]["code"], "backend_unavailable",
        "codex must be *known and named*, not `backend_not_found` — {body}"
    );
    let available = body["error"]["available_backends"]
        .as_array()
        .expect("array");
    assert!(
        available.iter().any(|v| v == "codex"),
        "codex must be offered as an option: {available:?}"
    );
    // Never silently ran on the global default fake instead.
    let list = get(&http(), &handle, "/v1/work").await;
    assert!(list["works"].as_array().expect("works").is_empty());
    handle.shutdown().await;
}

/// (b) `SGT_ORIGIN_CLIENT=codex` (as `sgt codex` sets, `src/harness.rs`'s
/// `ORIGIN_CLIENT_ENV`) resolves `OriginAffinity` when no explicit flag is
/// given — and, the case that actually catches the pre-registration bug,
/// must NOT fall through to a populated global default.
#[tokio::test]
async fn origin_affinity_from_sgt_codex_is_decisive_over_the_global_default() {
    let root = TempDir::new().expect("tempdir");
    support::scaffold_estate(root.path(), "plain", &["plain"]);
    let data = TempDir::new().expect("tempdir");
    let handle = start(data.path(), root.path(), Some(FAKE_BACKEND_NAME)).await;

    let (status, body) = submit(
        &http(),
        &handle,
        root.path(),
        json!({"origin": {"client": "codex", "cwd": root.path()}}),
    )
    .await;

    assert_eq!(status, 422);
    assert_eq!(body["error"]["code"], "backend_unavailable");
    let available = body["error"]["available_backends"]
        .as_array()
        .expect("array");
    assert!(available.iter().any(|v| v == "codex"));
    let list = get(&http(), &handle, "/v1/work").await;
    assert!(
        list["works"].as_array().expect("works").is_empty(),
        "must not have silently substituted the global default `fake`"
    );
    handle.shutdown().await;
}

/// (c) An estate `default_backend = "codex"` resolves `WorkspaceDefault`,
/// same decisiveness-over-global-default proof as (b).
#[tokio::test]
async fn estate_default_backend_codex_is_decisive_over_the_global_default() {
    let root = TempDir::new().expect("tempdir");
    estate_with_manifest(
        root.path(),
        "[estate]\nname = \"configured\"\ndefault_backend = \"codex\"\n\n\
         [[repo]]\nname = \"plain\"\n",
        &["plain"],
    );
    let data = TempDir::new().expect("tempdir");
    let handle = start(data.path(), root.path(), Some(FAKE_BACKEND_NAME)).await;

    let (status, body) = submit(
        &http(),
        &handle,
        root.path(),
        json!({"origin": {"client": "cli", "cwd": root.path()}}),
    )
    .await;

    assert_eq!(status, 422);
    assert_eq!(body["error"]["code"], "backend_unavailable");
    let available = body["error"]["available_backends"]
        .as_array()
        .expect("array");
    assert!(available.iter().any(|v| v == "codex"));
    let list = get(&http(), &handle, "/v1/work").await;
    assert!(
        list["works"].as_array().expect("works").is_empty(),
        "must not have silently substituted the global default `fake`"
    );
    handle.shutdown().await;
}
