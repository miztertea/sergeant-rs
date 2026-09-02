//! S4 Y6a acceptance: the scan trigger is **estate-scoped**, closing the gap
//! the owner correction named
//! (`estate-intelligence-is-the-feature-2026-08-28.md`): a registered
//! `[[repo]]` repository is scanned through X3a's Git path — never the
//! folder walker — alongside `[[knowledge]]` sources, in the same
//! `POST /v1/intelligence/scan` call.
//!
//! [`a_registered_repository_is_scanned_through_the_git_path_and_map_symbol_resolves_a_real_function`]
//! is the wave's own most-valuable-deliverable test: it declares a real
//! `[[repo]]` mount holding this build's own `src/` tree (copied from
//! `CARGO_MANIFEST_DIR` and committed once — a real git repository with real
//! function names, not a synthetic fixture; `tests/fixtures/`'s Office/mail/
//! zip corpora are deliberately left out of the mount so this test proves
//! the estate_git path itself rather than exercising every adapter kind),
//! scans it through the real daemon trigger, and proves three things a
//! folder-walker route could not:
//!
//! 1. the coverage lands under `kind: "estate_git"`, not `local_knowledge`;
//! 2. `content_key` in the trigger's own report is **exactly** the tree OID
//!    `git rev-parse HEAD^{tree}` answers on the mount — X3a's own identity
//!    (`git.rs`'s `list_tree`), which a directory walk's BLAKE3-over-paths
//!    key could never produce by coincidence;
//! 3. `sgt map symbol` resolves a real function this mount's `src/` actually
//!    defines (`scan_estate_git`, present since S3 — long committed, so this
//!    proof does not depend on this session's own uncommitted edits),
//!    sourced from `sergeant-rs` at that same generation.

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

fn run_git(dir: &Path, args: &[&str]) {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("spawn git {args:?} in {}: {e}", dir.display()));
    assert!(
        output.status.success(),
        "git {args:?} in {}: {}",
        dir.display(),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_output(dir: &Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("spawn git {args:?} in {}: {e}", dir.display()));
    assert!(
        output.status.success(),
        "git {args:?} in {}: {}",
        dir.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn copy_dir_recursive(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("mkdir");
    for entry in std::fs::read_dir(from).expect("read_dir") {
        let entry = entry.expect("entry");
        let dest = to.join(entry.file_name());
        let file_type = entry.file_type().expect("file_type");
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest);
        } else if file_type.is_file() {
            std::fs::copy(entry.path(), &dest).expect("copy file");
        }
        // Symlinks: none exist under this build's own `src/`; skipped rather
        // than followed if that ever changes.
    }
}

/// A real, self-contained git repository holding this build's own `src/`
/// tree, committed once — the estate's `[[repo]]` mount X3a's path reads.
///
/// Built by copying `CARGO_MANIFEST_DIR/src` (real, current source — not a
/// clone of repository history, so `tests/fixtures/`'s Office/mail/zip
/// corpora, which would otherwise spawn real parse workers this test has no
/// reason to exercise, are never part of the mount) rather than `git clone`,
/// so the tree this test scans is exactly the files copied, no more.
fn scaffold_repo_mount(mount: &Path) -> String {
    std::fs::create_dir_all(mount).expect("mkdir mount");
    let live_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    copy_dir_recursive(&live_src, &mount.join("src"));
    run_git(mount, &["init", "--initial-branch=main", "--quiet"]);
    run_git(mount, &["config", "user.email", "y6a-test@example.com"]);
    run_git(mount, &["config", "user.name", "Y6a Test"]);
    run_git(mount, &["add", "-A"]);
    run_git(
        mount,
        &[
            "commit",
            "--quiet",
            "-m",
            "snapshot of sergeant-rs src/ for the Y6a estate-scoped-scan proof",
        ],
    );
    git_output(mount, &["rev-parse", "HEAD"])
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
        .timeout(Duration::from_secs(30))
        .build()
        .expect("client")
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

/// The wave's own named "most valuable deliverable": a declared `[[repo]]`,
/// scanned end to end through the real daemon trigger and the real Git
/// path, with `sgt map symbol` resolving a real function afterward.
#[tokio::test]
async fn a_registered_repository_is_scanned_through_the_git_path_and_map_symbol_resolves_a_real_function()
 {
    let estate_dir = TempDir::new().expect("estate dir");
    let mount = estate_dir.path().join("repos").join("sergeant-rs");
    let pinned_sha = scaffold_repo_mount(&mount);
    let expected_tree_oid = git_output(&mount, &["rev-parse", &format!("{pinned_sha}^{{tree}}")]);

    std::fs::write(
        estate_dir.path().join("sergeant.toml"),
        "[estate]\nname = \"y6a\"\n\n[[repo]]\nname = \"sergeant-rs\"\n",
    )
    .expect("write sergeant.toml");

    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    // ---- the trigger, estate-scoped
    let body = json!({
        "command_id": ulid::Ulid::generate().to_string(),
        "estate_root": estate_dir.path(),
    });
    let (status, response) =
        support::scan_to_completion(&http, &handle.endpoint, &handle.token, &body, || {
            handle.is_alive()
        })
        .await;
    assert_eq!(status, 200, "{response}");
    let scanned = response["scanned"].as_array().expect("scanned array");
    assert_eq!(
        scanned.len(),
        1,
        "exactly the one declared [[repo]], no [[knowledge]] sources: {response}"
    );
    let row = &scanned[0];
    assert_eq!(row["source"], "sergeant-rs", "{row}");
    assert_eq!(
        row["kind"], "estate_git",
        "a registered repository must be reported under the estate_git kind, never \
         local_knowledge — routing it through the folder walker is the exact bug this wave \
         closes: {row}"
    );
    assert_eq!(row["outcome"], "recorded", "{row}");
    assert!(
        row["coverage"]["indexed"].as_u64().unwrap_or(0) > 0,
        "real .rs files must have been indexed: {row}"
    );
    assert_eq!(
        row["content_key"].as_str(),
        Some(expected_tree_oid.as_str()),
        "the generation must be keyed on the tree OID of the PINNED commit \
         (git rev-parse <sha>^{{tree}}, X3a's own identity — git.rs's list_tree) — never a \
         content hash a working-tree walk would have produced instead: {row}"
    );
    assert!(
        row.get("drift").is_none(),
        "the mount never moved between admission and the scan, so there must be no drift \
         observation: {row}"
    );

    // ---- confirmed via the ordinary status read too
    let (status, response) = get(&http, &handle, "/v1/intelligence/status").await;
    assert_eq!(status, 200);
    assert_eq!(response["sources"][0]["source"], "sergeant-rs");
    assert_eq!(response["sources"][0]["kind"], "estate_git");

    // ---- `sgt map repos` names it as a repository source, not a knowledge one
    let (status, response) = get(&http, &handle, "/v1/map/repos").await;
    assert_eq!(status, 200);
    let repos = response["repos"].as_array().expect("repos array");
    assert_eq!(repos.len(), 1, "{response}");
    assert_eq!(repos[0]["source"], "sergeant-rs");

    // ---- the deliverable itself: a real function resolves, sourced from
    // this repository, at this generation.
    let (status, response) = get(
        &http,
        &handle,
        &format!(
            "/v1/map/symbol?name={}",
            sergeant_rs::api::urlencode("scan_estate_git")
        ),
    )
    .await;
    assert_eq!(status, 200, "{response}");
    let symbols = response["symbols"].as_array().expect("symbols array");
    assert!(
        !symbols.is_empty(),
        "`scan_estate_git` — a real function this mount's own src/runtime/atlas/git.rs \
         defines — must resolve through the map surface once the repository is scanned: \
         {response}"
    );
    assert_eq!(symbols[0]["source"], "sergeant-rs", "{symbols:?}");
    assert_eq!(symbols[0]["language"], "rust", "{symbols:?}");

    // The definition site itself, attributed to the real path — `map
    // references` is where a path lives (`map symbol` reports only a count).
    let (status, response) = get(
        &http,
        &handle,
        &format!(
            "/v1/map/references?name={}",
            sergeant_rs::api::urlencode("scan_estate_git")
        ),
    )
    .await;
    assert_eq!(status, 200, "{response}");
    let references = response["references"].as_array().expect("references array");
    assert!(
        references
            .iter()
            .any(|r| r["path"].as_str() == Some("src/runtime/atlas/git.rs")),
        "the site must be attributed to the real path it was defined at, sourced from the \
         estate_git generation: {references:?}"
    );

    handle.shutdown().await;
}

/// The correction's own negative: a `[[knowledge]]` source pointed at a
/// plain copy of the same `src/` tree — the pre-Y6 workaround the owner
/// ruling names as the concrete loss (a knowledge-source copy has no pinned
/// SHA, no blob-OID keys, no drift observation) — is reported under
/// `local_knowledge`, never `estate_git`, so the two kinds stay honestly
/// distinguishable in the trigger's own report even when their content
/// happens to be identical.
#[tokio::test]
async fn a_knowledge_source_copy_of_the_same_content_is_still_reported_local_knowledge_not_estate_git()
 {
    let estate_dir = TempDir::new().expect("estate dir");
    let knowledge_root = estate_dir.path().join("copy");
    copy_dir_recursive(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime/atlas"),
        &knowledge_root.join("atlas"),
    );
    std::fs::write(
        estate_dir.path().join("sergeant.toml"),
        "[estate]\nname = \"y6a-negative\"\n\n[[knowledge]]\nname = \"copy\"\npath = \"copy\"\n",
    )
    .expect("write sergeant.toml");

    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let body = json!({
        "command_id": ulid::Ulid::generate().to_string(),
        "estate_root": estate_dir.path(),
    });
    let (status, response) =
        support::scan_to_completion(&http, &handle.endpoint, &handle.token, &body, || {
            handle.is_alive()
        })
        .await;
    assert_eq!(status, 200, "{response}");
    let row = &response["scanned"][0];
    assert_eq!(
        row["kind"], "local_knowledge",
        "identical bytes, reached through a knowledge declaration rather than a [[repo]] \
         mount, must still be reported local_knowledge — the kind is a fact about how the \
         bytes were declared and acquired, never inferred from what they contain: {row}"
    );

    handle.shutdown().await;
}
