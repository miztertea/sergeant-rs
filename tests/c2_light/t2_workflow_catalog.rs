//! T-series T2 acceptance tests: `GET /v1/workflows` (proposal
//! `reference/proposal-tui-t-series.md` §11.2, Decisions T2-39/T2-40).
//!
//! Retains and updates the original workflow-catalog contract (§19.4):
//! embedded fallback, an indexed published workflow, drafts excluded, an
//! unindexed directory excluded, a non-absolute `cwd` rejected, and no event
//! append.
//!
//! **Estate-root §5.2 changed what the catalog is *about*.** It used to
//! discover a estate from the client's `cwd`, which made "what could I
//! bind" a question about wherever the caller happened to be standing. The
//! catalog is now the **bound estate's**, the same estate `POST /v1/work`
//! plans against — a client cannot be shown a catalog it could not actually
//! submit into. `cwd` remains in the query grammar and is still validated
//! (a relative one is still a structured 400), but it is evidence only.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::daemon::{self, DaemonHandle};

use crate::support;

fn http() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .expect("client")
}

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "sergeant tests")
        .env("GIT_AUTHOR_EMAIL", "tests@example.invalid")
        .env("GIT_COMMITTER_NAME", "sergeant tests")
        .env("GIT_COMMITTER_EMAIL", "tests@example.invalid")
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn init_repo(path: &Path) {
    std::fs::create_dir_all(path).expect("repo dir");
    git(path, &["init", "-b", "main"]);
    std::fs::write(path.join("README.md"), "# fixture\n").expect("write file");
    git(path, &["add", "."]);
    git(path, &["commit", "-m", "initial"]);
}

/// An estate root at `path` (§4.1) with one derived mount (§6.1), and a
/// daemon started **bound** to it (§5.1) — the catalog's subject.
fn init_estate(path: &Path) {
    init_repo(path);
    init_repo(&path.join("repos").join("solo"));
    std::fs::write(
        path.join("sergeant.toml"),
        "[estate]\nname = \"catalog-estate\"\n\n[[repo]]\nname = \"solo\"\n",
    )
    .expect("write sergeant.toml");
}

async fn start_bound(data_dir: &Path, _estate_root: &Path) -> DaemonHandle {
    daemon::start_with(
        data_dir,
        daemon::DaemonConfig {
            ..daemon::DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start")
}

/// Writes one admitted workflow — `workflow.toml`, one stage's `CONTEXT.md`,
/// and its own `index.md` front matter — under `root`'s
/// `.sergeant/workflows/<name>/`.
fn write_workflow(root: &Path, name: &str) {
    let dir = root.join(".sergeant/workflows").join(name);
    std::fs::create_dir_all(dir.join("00-only")).expect("stage dir");
    std::fs::write(
        dir.join("workflow.toml"),
        format!("[workflow]\nname = \"{name}\"\nversion = \"2\"\nstages = [\"00-only\"]\n"),
    )
    .expect("workflow.toml");
    std::fs::write(dir.join("00-only").join("CONTEXT.md"), "do the thing").expect("CONTEXT.md");
    std::fs::write(
        dir.join("index.md"),
        format!(
            "---\nkind: workflow\nname: {name}\nstatus: published\nversion: 2\n\
             description: A one-line description of {name}.\ntags:\n  - fixture\n---\n\n# {name}\n"
        ),
    )
    .expect("index.md");
}

/// Writes the root catalog (`.sergeant/index.md`) naming exactly `rows` —
/// `(name, status)` pairs, in order.
fn write_root_catalog(root: &Path, rows: &[(&str, &str)]) {
    let mut body = String::from("# catalog\n\n| Workflow | Status | Index |\n|---|---|---|\n");
    for (name, status) in rows {
        body.push_str(&format!(
            "| `{name}` | {status} | [`workflows/{name}/index.md`](workflows/{name}/index.md) |\n"
        ));
    }
    std::fs::create_dir_all(root.join(".sergeant")).expect(".sergeant dir");
    std::fs::write(root.join(".sergeant/index.md"), body).expect("root catalog");
}

async fn get_status(handle: &DaemonHandle, path: &str) -> (reqwest::StatusCode, Value) {
    let resp = support::send_while_alive(
        "get_status",
        || {
            http()
                .get(format!("{}{path}", handle.endpoint))
                .bearer_auth(&handle.token)
        },
        || handle.is_alive(),
    )
    .await;
    let status = resp.status();
    (status, resp.json().await.expect("json body"))
}

async fn events(handle: &DaemonHandle) -> Vec<Value> {
    let body: Value = support::send_while_alive(
        "events",
        || {
            http()
                .get(format!("{}/v1/events", handle.endpoint))
                .bearer_auth(&handle.token)
        },
        || handle.is_alive(),
    )
    .await
    .json()
    .await
    .expect("json body");
    body["events"].as_array().cloned().unwrap_or_default()
}

/// The same minimal percent-encoding `ApiClient::workflows` uses in
/// production (`src/api.rs`'s `urlencode`) — reimplemented here rather than
/// imported, since this suite drives the route over raw HTTP the way a
/// non-Rust client would, not through `ApiClient`.
fn urlencoding_stub(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

/// §11.2's happy path: a root-indexed published workflow comes back fully
/// described — every `CatalogEntry` field, and a `StageEntry` per stage.
#[tokio::test]
async fn a_published_workflow_is_listed_fully_described() {
    let data = TempDir::new().expect("tempdir");
    let repo = TempDir::new().expect("tempdir");
    init_estate(repo.path());
    write_workflow(repo.path(), "implement");
    write_root_catalog(repo.path(), &[("implement", "published")]);

    let handle = start_bound(data.path(), repo.path()).await;
    let (status, body) = get_status(
        &handle,
        &format!(
            // D4: `cwd` is evidence; `estate_root` is what the catalog's
            // estate-local half is read from, once admitted.
            "/v1/workflows?cwd={0}&estate_root={0}",
            // Deliberately the same encoding the production client uses.
            urlencoding_stub(&repo.path().display().to_string())
        ),
    )
    .await;
    assert_eq!(status, reqwest::StatusCode::OK, "{body}");

    let workflows = body["workflows"].as_array().expect("workflows array");
    assert_eq!(workflows.len(), 1, "{body}");
    let entry = &workflows[0];
    assert_eq!(entry["name"], "implement");
    assert_eq!(entry["version"], "2");
    assert_eq!(
        entry["source"],
        repo.path()
            .join(".sergeant/workflows/implement")
            .display()
            .to_string()
    );
    assert!(
        entry["content_hash"]
            .as_str()
            .is_some_and(|h| h.len() == 64 && h.chars().all(|c| c.is_ascii_hexdigit())),
        "content_hash must be 64 lowercase hex chars: {entry}"
    );
    assert_eq!(entry["status"], "published");
    assert_eq!(entry["description"], "A one-line description of implement.");
    assert_eq!(entry["tags"], json!(["fixture"]));

    let stages = entry["stages"].as_array().expect("stages array");
    assert_eq!(stages.len(), 1);
    assert_eq!(stages[0]["id"], "00-only");
    assert_eq!(stages[0]["kind"], "actor");
    assert!(stages[0]["harness"].is_null());
    assert!(stages[0]["profile"].is_null());
    assert_eq!(stages[0]["requires_ask"], false);
    assert!(
        stages[0].get("context").is_none() && stages[0].get("execute").is_none(),
        "context/execute must never be serialized (§11.2): {stages:?}"
    );

    handle.shutdown().await;
}

/// §11.2's embedded-fallback edge shape: the bound estate publishes no
/// catalog of its own (here, a daemon bound to no estate at all — §5.1's
/// `None`), so the built-in `software-change` workflow answers instead, with
/// `status`/`description`/`tags` all absent (not `null`, not `[]`).
#[tokio::test]
async fn no_repository_falls_back_to_the_embedded_workflow() {
    let data = TempDir::new().expect("tempdir");
    let elsewhere = TempDir::new().expect("tempdir"); // deliberately not a git repo

    let handle = daemon::start(data.path()).await.expect("daemon start");
    let (status, body) = get_status(
        &handle,
        &format!(
            "/v1/workflows?cwd={}",
            urlencoding_stub(&elsewhere.path().display().to_string())
        ),
    )
    .await;
    assert_eq!(status, reqwest::StatusCode::OK, "{body}");

    let workflows = body["workflows"].as_array().expect("workflows array");
    assert_eq!(workflows.len(), 1, "{body}");
    let entry = &workflows[0];
    assert_eq!(entry["name"], "software-change");
    assert_eq!(entry["source"], "embedded");
    assert!(
        entry.get("status").is_none(),
        "status must be absent, not null, for the embedded entry: {entry}"
    );
    assert!(entry.get("description").is_none());
    assert!(entry.get("tags").is_none());
    assert!(!entry["stages"].as_array().expect("stages").is_empty());

    handle.shutdown().await;
}

/// A repository whose root catalog exists but names only some on-disk
/// workflow directories: an unindexed directory never appears, and a row
/// the root catalog itself marks `draft` never appears either (§19.4).
#[tokio::test]
async fn unindexed_directories_and_draft_rows_are_excluded() {
    let data = TempDir::new().expect("tempdir");
    let repo = TempDir::new().expect("tempdir");
    init_estate(repo.path());
    write_workflow(repo.path(), "implement");
    write_workflow(repo.path(), "shadow"); // on disk, never indexed
    write_workflow(repo.path(), "half-baked");
    write_root_catalog(
        repo.path(),
        &[("implement", "published"), ("half-baked", "draft")],
    );

    let handle = start_bound(data.path(), repo.path()).await;
    let (status, body) = get_status(
        &handle,
        &format!(
            "/v1/workflows?cwd={0}&estate_root={0}",
            urlencoding_stub(&repo.path().display().to_string())
        ),
    )
    .await;
    assert_eq!(status, reqwest::StatusCode::OK, "{body}");
    let names: Vec<&str> = body["workflows"]
        .as_array()
        .expect("workflows array")
        .iter()
        .map(|w| w["name"].as_str().unwrap_or(""))
        .collect();
    assert_eq!(names, vec!["implement"], "{body}");

    handle.shutdown().await;
}

/// §11.2's `400`: a missing `cwd`, and one that is not an absolute path,
/// both answer with the structured `{"error": {...}}` shape every other
/// route uses — not a silent fallback or a panic.
#[tokio::test]
async fn a_missing_or_relative_cwd_is_a_structured_400() {
    let data = TempDir::new().expect("tempdir");
    let handle = daemon::start(data.path()).await.expect("daemon start");

    let (status, body) = get_status(&handle, "/v1/workflows").await;
    assert_eq!(status, reqwest::StatusCode::BAD_REQUEST, "{body}");
    assert!(body["error"]["code"].is_string(), "{body}");

    let (status, body) = get_status(&handle, "/v1/workflows?cwd=relative/path").await;
    assert_eq!(status, reqwest::StatusCode::BAD_REQUEST, "{body}");
    assert!(body["error"]["code"].is_string(), "{body}");

    handle.shutdown().await;
}

/// §19.4: this is a pure read. It appends no event, whether it resolves a
/// repository catalog or falls back to the embedded workflow.
#[tokio::test]
async fn the_route_appends_no_event() {
    let data = TempDir::new().expect("tempdir");
    let repo = TempDir::new().expect("tempdir");
    init_estate(repo.path());
    write_workflow(repo.path(), "implement");
    write_root_catalog(repo.path(), &[("implement", "published")]);

    let handle = start_bound(data.path(), repo.path()).await;
    let before = events(&handle).await.len();
    let (status, _) = get_status(
        &handle,
        &format!(
            "/v1/workflows?cwd={0}&estate_root={0}",
            urlencoding_stub(&repo.path().display().to_string())
        ),
    )
    .await;
    assert_eq!(status, reqwest::StatusCode::OK);
    let after = events(&handle).await.len();
    assert_eq!(before, after, "GET /v1/workflows must not journal anything");

    handle.shutdown().await;
}
