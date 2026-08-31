//! S6 — the semantic half **at the crossing**: through the daemon, over a
//! real corpus, with the committed model installed.
//!
//! # The axis no other suite covers
//!
//! Every semantic test before this one ran either with no model installed,
//! or in-process against `AtlasDb::semantic_search` on a fixture small
//! enough to finish inside any budget. `w3b_semantic_retrieval` has no HTTP
//! boundary at all; `w3_semantic_degradation` removes the model directory on
//! purpose. So the crossing — *semantic x real-repository text x the daemon
//! boundary* — was covered on each axis alone and never at the crossing, and
//! that is where the defect lived: `semantic_search` re-embedded the whole
//! admissible corpus on every query, which on the real estate (19,025 units)
//! spent the CLI's entire 10 s `REQUEST_TIMEOUT` (`src/cli.rs:47`) and
//! returned `error sending request for url (.../v1/search?...)`, exit 1,
//! while the daemon carried on embedding.
//!
//! # What this suite holds
//!
//! * [`the_operator_can_ask_whether_the_semantic_model_is_loaded`] and
//!   [`a_model_directory_that_will_not_load_is_not_reported_as_no_assets`] —
//!   A2 §15's *"reports that coverage/capability honestly"*, on the surface
//!   an operator actually has. Before S6 `GET /v1/intelligence/status`
//!   returned exactly `atlas` and `sources`: the only thing that said
//!   anything about the model was a search answer's disclosure line, so an
//!   operator had to run a query to learn whether a query would work, and a
//!   directory that failed to load was indistinguishable from no assets —
//!   precisely the distinction `SemanticEngine::load`'s own doc says is
//!   there on purpose.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig};
use sergeant_rs::runtime::atlas::semantic::{MODEL_DIR_ENV, MODEL_FILES};

mod support;
use support::DataDir;

/// The committed assets, through the documented operator override. Safe
/// because cargo-nextest runs every test in its own process — the same
/// reason `tests/w3b_semantic_retrieval.rs` gives.
fn install_model() {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/semantic-model");
    assert!(assets.join("model.safetensors").is_file());
    unsafe { std::env::set_var(MODEL_DIR_ENV, &assets) };
}

fn uninstall_model() {
    unsafe { std::env::remove_var(MODEL_DIR_ENV) };
}

/// A directory that **is** a complete asset directory by
/// `semantic::model_dir`'s test — all three of [`MODEL_FILES`] present — and
/// whose weights are garbage, so the loader fails.
///
/// The case A2-13 does not cover: not "this host has no model" but "this
/// host has assets that will not load", which
/// `SemanticEngine::load`'s doc calls *"a fault an operator has to hear
/// about"*.
fn install_broken_model() -> tempfile::TempDir {
    let broken = tempfile::tempdir().expect("broken model dir");
    for name in MODEL_FILES {
        std::fs::write(broken.path().join(name), b"not a model").expect("write broken asset");
    }
    unsafe { std::env::set_var(MODEL_DIR_ENV, broken.path()) };
    broken
}

async fn start_daemon(data: &DataDir) -> daemon::DaemonHandle {
    let fake = FakeBackend::scripted(FAKE_BACKEND_NAME, [FakeStep::hang()]);
    daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(BackendRegistry::new().with(Arc::new(fake))),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            claude: None,
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start")
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .expect("client")
}

async fn get(http: &reqwest::Client, handle: &daemon::DaemonHandle, path: &str) -> Value {
    http.get(format!("{}{path}", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("request")
        .json()
        .await
        .expect("json")
}

/// **A2 §15's capability, reported on the surface an operator has.**
///
/// `sgt intelligence status --json` is where an operator asks what this
/// host's intelligence can do. Before S6 it could not answer the one
/// question that decides whether search will rank semantically at all.
#[tokio::test]
async fn the_operator_can_ask_whether_the_semantic_model_is_loaded() {
    install_model();
    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let status = get(&http, &handle, "/v1/intelligence/status").await;
    assert_eq!(
        status["semantic"]["state"], "installed",
        "the committed assets are installed; status said: {status}"
    );
    assert!(
        status["semantic"]["identity"]
            .as_str()
            .is_some_and(|id| id.contains('@')),
        "an installed model names the assets it is: {status}"
    );
    assert!(
        status["semantic"]["content_hash"]
            .as_str()
            .is_some_and(|hash| hash.starts_with("blake3:")),
        "A2 §15 pins assets by content as well as version: {status}"
    );

    handle.shutdown().await;
}

/// **A directory that exists and will not load is a fault, not an absence.**
///
/// `AtlasDb::semantic_engine` reported the load error with `log::warn!` and
/// then yielded `None` — and the daemon installs a `tracing` subscriber with
/// no `log`/`tracing` bridge, so the message reached a facade with no logger
/// and was dropped. Broken assets and no assets became the same answer
/// everywhere an operator could look.
#[tokio::test]
async fn a_model_directory_that_will_not_load_is_not_reported_as_no_assets() {
    let _broken = install_broken_model();
    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let status = get(&http, &handle, "/v1/intelligence/status").await;
    assert_eq!(
        status["semantic"]["state"], "failed",
        "broken assets must not read as an uninstalled host: {status}"
    );
    assert!(
        status["semantic"]["detail"]
            .as_str()
            .is_some_and(|d| !d.is_empty()),
        "a fault an operator has to hear about must say what it was: {status}"
    );

    handle.shutdown().await;
}

/// The third state, so the two above cannot pass by the surface reporting a
/// constant: a host with no assets says so, and says nothing about a fault.
#[tokio::test]
async fn a_host_with_no_assets_reports_not_installed_rather_than_a_fault() {
    uninstall_model();
    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let status = get(&http, &handle, "/v1/intelligence/status").await;
    assert_eq!(status["semantic"]["state"], "not_installed", "{status}");
    assert!(status["semantic"]["detail"].is_null(), "{status}");

    handle.shutdown().await;
}
