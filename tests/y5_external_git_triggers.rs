//! S4 Y5 acceptance: the scan trigger (G8) and external-git acquisition's
//! HTTP surface (G6), end to end against a real in-process daemon —
//! [`sergeant_rs::daemon::start_with`], the same pattern
//! `tests/e_admission_uses_no_network_git.rs` uses, rather than a spawned
//! `sgt` subprocess: lighter, and it is the daemon's own production code
//! path (`api::intelligence_scan`/`intelligence_add_source`) that answers,
//! not a client's idea of it.
//!
//! * [`a_knowledge_scan_indexes_declared_sources_and_reports_from_coverage`]
//!   — the trigger itself: `POST /v1/intelligence/scan` scans a real
//!   `[[knowledge]]` source, records a generation, and its report is built
//!   from the scan's own coverage counts. A second scan of unchanged bytes
//!   answers `unchanged`, never re-recording (ruling §4).
//! * [`a_scan_of_an_estate_with_no_knowledge_sources_reports_so_honestly`]
//!   — the empty case is a real answer, not a silent no-op.
//! * [`an_unallowlisted_locator_is_refused_by_the_api_before_git_runs`] —
//!   G6's primary control, exercised through the real HTTP surface: `ext::`
//!   and `file://` locators are refused with `422` and a named reason,
//!   never reaching a subprocess.
//! * [`the_intelligence_sources_list_is_empty_until_something_is_added`] —
//!   the read side's honest negative.
//! * [`sgt_help_advertises_the_new_verbs`] — the CLI wiring itself: `sgt
//!   knowledge --help` and `sgt intelligence --help` both name the new
//!   verbs this wave adds, the lightweight proof
//!   `tests/x5_a1a_acceptance.rs`'s own retired tripwire used to make the
//!   opposite claim with.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig};

mod support;
use support::DataDir;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

/// A minimal estate with one declared `[[knowledge]]` source pointing at a
/// real directory holding `body`.
fn scaffold_knowledge_estate(root: &Path, source_name: &str, body: &str) -> std::path::PathBuf {
    let knowledge_root = root.join("notes");
    std::fs::create_dir_all(&knowledge_root).expect("knowledge dir");
    std::fs::write(knowledge_root.join("guide.md"), body).expect("write guide.md");
    let manifest = format!(
        "[estate]\nname = \"y5\"\n\n[[knowledge]]\nname = {source_name:?}\npath = \"notes\"\n"
    );
    std::fs::create_dir_all(root).expect("estate root");
    std::fs::write(root.join("sergeant.toml"), manifest).expect("write sergeant.toml");
    knowledge_root
}

/// A daemon started the way every other in-process HTTP test in this suite
/// starts one — a scripted fake backend, since nothing here submits Work.
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
        .timeout(Duration::from_secs(20))
        .build()
        .expect("client")
}

async fn post(
    client: &reqwest::Client,
    handle: &daemon::DaemonHandle,
    path: &str,
    body: &Value,
) -> (reqwest::StatusCode, Value) {
    let response = client
        .post(format!("{}{path}", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(body)
        .send()
        .await
        .expect("request");
    let status = response.status();
    let body: Value = response.json().await.expect("json body");
    (status, body)
}

async fn get(
    client: &reqwest::Client,
    handle: &daemon::DaemonHandle,
    path: &str,
) -> (reqwest::StatusCode, Value) {
    let response = client
        .get(format!("{}{path}", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("request");
    let status = response.status();
    let body: Value = response.json().await.expect("json body");
    (status, body)
}

/// The trigger itself, and ruling §4's unchanged-scan-writes-nothing rule.
#[tokio::test]
async fn a_knowledge_scan_indexes_declared_sources_and_reports_from_coverage() {
    let estate_dir = TempDir::new().expect("estate dir");
    scaffold_knowledge_estate(estate_dir.path(), "guides", "# Guide\n\nbody text\n");
    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let body = json!({
        "command_id": ulid::Ulid::generate().to_string(),
        "estate_root": estate_dir.path(),
    });
    let (status, response) = post(&http, &handle, "/v1/intelligence/scan", &body).await;
    assert_eq!(status, 200, "{response}");
    let scanned = response["scanned"].as_array().expect("scanned array");
    assert_eq!(scanned.len(), 1, "{response}");
    let row = &scanned[0];
    assert_eq!(row["source"], "guides", "{row}");
    assert_eq!(row["outcome"], "recorded", "{row}");
    assert!(
        row["generation"].as_str().is_some_and(|g| !g.is_empty()),
        "{row}"
    );
    assert_eq!(
        row["coverage"]["indexed"].as_u64().unwrap_or(0),
        1,
        "the one markdown file was indexed: {row}"
    );

    // Confirmed via the ordinary status read too — the daemon really wrote
    // the generation, not merely reported one.
    let (status, response) = get(&http, &handle, "/v1/intelligence/status").await;
    assert_eq!(status, 200);
    assert_eq!(response["atlas"]["present"], json!(true));
    assert_eq!(response["sources"][0]["source"], "guides");
    assert_eq!(response["sources"][0]["kind"], "local_knowledge");

    // A second scan of byte-identical content changes nothing (ruling §4):
    // the same generation id, `unchanged`, never re-recorded.
    let first_generation = row["generation"].clone();
    let body = json!({
        "command_id": ulid::Ulid::generate().to_string(),
        "estate_root": estate_dir.path(),
    });
    let (status, response) = post(&http, &handle, "/v1/intelligence/scan", &body).await;
    assert_eq!(status, 200, "{response}");
    let row = &response["scanned"][0];
    assert_eq!(row["outcome"], "unchanged", "{row}");
    assert_eq!(row["generation"], first_generation, "{row}");

    handle.shutdown().await;
}

/// The empty case is a real, honest answer — never a silent no-op that
/// looks identical to a scan that indexed something.
///
/// S4 Y6 widened the trigger to every declared source kind (the owner
/// correction `estate-intelligence-is-the-feature-2026-08-28.md`), so the
/// honest empty answer now names all three kinds an estate could have
/// declared, not `[[knowledge]]` alone — this estate declares none of them.
#[tokio::test]
async fn a_scan_of_an_estate_with_no_knowledge_sources_reports_so_honestly() {
    let estate_dir = TempDir::new().expect("estate dir");
    std::fs::write(
        estate_dir.path().join("sergeant.toml"),
        "[estate]\nname = \"empty\"\n",
    )
    .expect("write manifest");
    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let body = json!({
        "command_id": ulid::Ulid::generate().to_string(),
        "estate_root": estate_dir.path(),
    });
    let (status, response) = post(&http, &handle, "/v1/intelligence/scan", &body).await;
    assert_eq!(status, 200, "{response}");
    assert_eq!(response["scanned"], json!([]), "{response}");
    assert!(
        response["detail"]
            .as_str()
            .is_some_and(|d| d.contains("[[repo]]") && d.contains("[[knowledge]]")),
        "{response}"
    );

    handle.shutdown().await;
}

/// G6's primary control, through the real HTTP surface: an `ext::` remote
/// helper and a `file://` locator are both refused before any subprocess —
/// there is no git invocation to observe here at all, which is the point:
/// the refusal happens in [`sergeant_rs::runtime::atlas::locator::validate`],
/// reached from [`sergeant_rs::api`]'s own handler before
/// `acquire_external_git_on_lane` is ever called.
#[tokio::test]
async fn an_unallowlisted_locator_is_refused_by_the_api_before_git_runs() {
    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    for (url, why) in [
        ("ext::sh -c 'id'", "remote-helper"),
        ("file:///etc/passwd", "file scheme"),
        ("git://example.com/repo.git", "git protocol"),
    ] {
        let body = json!({
            "command_id": ulid::Ulid::generate().to_string(),
            "url": url,
        });
        let (status, response) = post(&http, &handle, "/v1/intelligence/sources", &body).await;
        assert_eq!(status, 422, "{why}: {url} should be refused: {response}");
        assert_eq!(
            response["error"]["code"], "invalid_locator",
            "{why}: {response}"
        );
    }

    // Refused before anything was written: no external source appears.
    let (status, response) = get(&http, &handle, "/v1/intelligence/sources").await;
    assert_eq!(status, 200);
    assert_eq!(response["atlas"]["present"], json!(false), "{response}");

    handle.shutdown().await;
}

/// The read side's honest negative.
#[tokio::test]
async fn the_intelligence_sources_list_is_empty_until_something_is_added() {
    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let (status, response) = get(&http, &handle, "/v1/intelligence/sources").await;
    assert_eq!(status, 200);
    assert_eq!(response["atlas"]["present"], json!(false), "{response}");
    assert_eq!(response["sources"], json!([]), "{response}");

    handle.shutdown().await;
}

/// The CLI wiring itself, help-surface only (no daemon needed) — the
/// lightweight proof `tests/x5_a1a_acceptance.rs`'s retired tripwire used to
/// make the opposite claim with.
#[test]
fn sgt_help_advertises_the_new_verbs() {
    let knowledge_help = std::process::Command::new(SGT)
        .args(["knowledge", "--help"])
        .output()
        .expect("sgt knowledge --help");
    let text = String::from_utf8_lossy(&knowledge_help.stdout);
    assert!(
        text.contains("scan"),
        "sgt knowledge --help must name `scan`: {text}"
    );

    let intelligence_help = std::process::Command::new(SGT)
        .args(["intelligence", "--help"])
        .output()
        .expect("sgt intelligence --help");
    let text = String::from_utf8_lossy(&intelligence_help.stdout);
    assert!(
        text.contains("add"),
        "sgt intelligence --help must name `add`: {text}"
    );
    assert!(
        text.contains("list"),
        "sgt intelligence --help must name `list`: {text}"
    );
    assert!(
        text.contains("status"),
        "sgt intelligence --help must still name `status`: {text}"
    );
}
