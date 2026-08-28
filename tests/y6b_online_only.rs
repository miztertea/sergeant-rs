//! S4 Y6 acceptance: the online-only / cloud-placeholder heuristic (G7,
//! A1-06), proven end to end through the real scan trigger —
//! `POST /v1/intelligence/scan` against a real in-process daemon, the same
//! harness `tests/y5_external_git_triggers.rs` uses, rather than calling
//! [`sergeant_rs::runtime::atlas::scan::scan_local_knowledge`] directly (that
//! pure-function level is already covered by `src/runtime/atlas/scan.rs`'s
//! own unit tests — this file is the acceptance item's "through the real
//! `sgt knowledge scan` path" requirement, item 6 of the wave's own brief).
//!
//! * [`an_online_only_placeholder_is_a_named_gap_row_through_the_real_scan_trigger`]
//!   — a sparse stand-in file (`st_blocks == 0`, `st_size > 0` —
//!   `truncate`'s own documented effect, verified in this wave's sandbox)
//!   scanned through the daemon lands `online_only` in the reported
//!   coverage, never `indexed` with the file counted as if it had been
//!   read.
//! * [`a_genuinely_empty_file_is_not_misreported_as_a_placeholder_through_the_real_scan_trigger`]
//!   — the mirror negative: a real empty file stays `indexed`, because
//!   flagging it as a placeholder would be the opposite dishonesty.
//!
//! Both are probe-gated per `CONTRIBUTING.md`'s two-environment rule: a
//! filesystem that will not leave a `set_len` file unallocated makes the
//! divergence this heuristic reads a fact about the environment, not the
//! code under test, so the test skips loudly (`SKIPPED-ENV`) rather than
//! failing on a precondition nothing in this build controls.

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

/// A minimal estate with one declared `[[knowledge]]` source pointing at
/// `root` — the estate scaffold `y5_external_git_triggers.rs` already
/// established, reused rather than re-derived (R2).
fn scaffold_knowledge_estate(root: &Path, source_name: &str) -> std::path::PathBuf {
    let knowledge_root = root.join("notes");
    std::fs::create_dir_all(&knowledge_root).expect("knowledge dir");
    let manifest = format!(
        "[estate]\nname = \"y6b\"\n\n[[knowledge]]\nname = {source_name:?}\npath = \"notes\"\n"
    );
    std::fs::create_dir_all(root).expect("estate root");
    std::fs::write(root.join("sergeant.toml"), manifest).expect("write sergeant.toml");
    knowledge_root
}

/// A sparse file: `set_len` on a freshly created file, which on ext4 (and
/// most Linux filesystems) leaves the hole unallocated — `st_size` equal to
/// `apparent_len`, `st_blocks == 0`. Returns whether the filesystem actually
/// behaved that way, so the caller can skip honestly rather than assume it.
#[cfg(unix)]
fn make_sparse_file(path: &Path, apparent_len: u64) -> bool {
    use std::os::unix::fs::MetadataExt;
    let file = std::fs::File::create(path).expect("create");
    file.set_len(apparent_len).expect("set_len");
    drop(file);
    let meta = std::fs::symlink_metadata(path).expect("stat");
    meta.size() == apparent_len && meta.blocks() == 0
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

/// Item 4's own decisive check (`tests/x5_a1a_acceptance.rs`'s register row
/// 4): a suspected placeholder, scanned through the real trigger, lands a
/// named `online_only` coverage row — never `indexed` with the file counted
/// as though its zero bytes had actually been read.
#[cfg(unix)]
#[tokio::test]
async fn an_online_only_placeholder_is_a_named_gap_row_through_the_real_scan_trigger() {
    let estate_dir = TempDir::new().expect("estate dir");
    let knowledge_root = scaffold_knowledge_estate(estate_dir.path(), "synced");
    let placeholder = knowledge_root.join("report.md");
    if !make_sparse_file(&placeholder, 1_048_576) {
        eprintln!(
            "SKIPPED-ENV: this filesystem did not leave a sparse file unallocated after \
             set_len — the divergence this test exercises is a property of the filesystem, \
             not of the code under test"
        );
        return;
    }

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
    assert_eq!(row["source"], "synced", "{row}");
    assert_eq!(row["kind"], "local_knowledge", "{row}");
    assert_eq!(row["outcome"], "recorded", "{row}");
    assert_eq!(
        row["coverage"]["online_only"].as_u64().unwrap_or(0),
        1,
        "the placeholder must be its own named coverage state: {row}"
    );
    assert_eq!(
        row["coverage"]["indexed"].as_u64().unwrap_or(0),
        0,
        "a suspected placeholder must never be counted as indexed — that is exactly the \
         'silently indexed as empty' case acceptance item 4 forbids: {row}"
    );

    // Confirmed via the ordinary status read too — the daemon really wrote
    // the coverage, not merely reported it in the trigger's own response.
    let (status, response) = client()
        .get(format!("{}/v1/intelligence/status", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("status request")
        .json::<Value>()
        .await
        .map(|v| (reqwest::StatusCode::OK, v))
        .expect("status body");
    assert_eq!(status, 200);
    assert_eq!(
        response["sources"][0]["coverage"]["online_only"].as_u64(),
        Some(1),
        "{response}"
    );

    handle.shutdown().await;
}

/// The mirror negative, through the same real trigger: a genuinely empty
/// file is `indexed`, not `online_only`. Flagging an ordinary empty file as
/// a suspected placeholder would be the opposite dishonesty from the one
/// this heuristic exists to fix.
#[tokio::test]
async fn a_genuinely_empty_file_is_not_misreported_as_a_placeholder_through_the_real_scan_trigger()
{
    let estate_dir = TempDir::new().expect("estate dir");
    let knowledge_root = scaffold_knowledge_estate(estate_dir.path(), "synced");
    std::fs::write(knowledge_root.join("empty.md"), b"").expect("write empty file");

    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let body = json!({
        "command_id": ulid::Ulid::generate().to_string(),
        "estate_root": estate_dir.path(),
    });
    let (status, response) = post(&http, &handle, "/v1/intelligence/scan", &body).await;
    assert_eq!(status, 200, "{response}");
    let row = &response["scanned"][0];
    assert_eq!(
        row["coverage"]["indexed"].as_u64().unwrap_or(0),
        1,
        "a truly empty file is indexed — that is the true answer, not a placeholder: {row}"
    );
    assert_eq!(
        row["coverage"]["online_only"].as_u64().unwrap_or(0),
        0,
        "{row}"
    );

    handle.shutdown().await;
}

/// `sgt --help`'s own CLI wiring: `sgt knowledge --help` still names `scan`
/// (kept working, unchanged spelling), and `sgt intelligence --help` now
/// names it too (the primary spelling, S4 Y6, G8 correction) — the
/// lightweight structural proof `tests/x5_a1a_acceptance.rs`'s own
/// `the_intelligence_verb_set_now_includes_the_trigger_and_the_acquisition_surface`
/// already pins in full; this is the same fact confirmed from a second
/// angle since it is this file's own subject.
#[test]
fn sgt_intelligence_scan_and_sgt_knowledge_scan_both_advertise_the_estate_scoped_trigger() {
    for args in [["knowledge", "--help"], ["intelligence", "--help"]] {
        let help = std::process::Command::new(SGT)
            .args(args)
            .output()
            .unwrap_or_else(|e| panic!("sgt {args:?}: {e}"));
        let text = String::from_utf8_lossy(&help.stdout);
        assert!(
            text.lines()
                .any(|line| line.trim_start().starts_with("scan")),
            "sgt {args:?} must name the `scan` verb: {text}"
        );
    }
}
