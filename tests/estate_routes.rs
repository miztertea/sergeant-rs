//! T-series T3 acceptance: the seven `/v1/estate/*`/`/v1/doctor` routes
//! (`reference/proposal-tui-t-series.md` §16.2/§16.3) as thin daemon-side
//! wrappers over `crate::domain::manifest`/`crate::cli::doctor` — never a
//! second implementation of repo/group/doctor semantics.
//!
//! Same in-process daemon rig as `tests/m2_daemon_api.rs`/`tests/
//! m6_surfaces.rs` (`daemon::start_with` + a real HTTP client over its
//! loopback endpoint), plus a real git fixture the same way `tests/
//! m8_estate_cli.rs` builds one for `sgt repo add`'s populate-or-verify.
//!
//! Every estate here uses the default `<estate_root>/.sergeant/data` layout
//! and starts its daemon **bound** to the estate root (estate-root §5.1):
//! `src/api.rs`'s `resolve_estate_root` reads that binding off the engine
//! rather than walking up from `data_dir`, so these routes depend on nothing
//! but the root the daemon was started against — and, as before, not at all
//! on the test process's own working directory (`current_dir` is global
//! process state, and nothing here needs to touch it).

use std::path::Path;
use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

use sergeant_rs::api::ApiClient;
use sergeant_rs::daemon::{self, DaemonConfig, DaemonHandle};

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "sergeant tests")
        .env("GIT_AUTHOR_EMAIL", "tests@example.invalid")
        .env("GIT_COMMITTER_NAME", "sergeant tests")
        .env("GIT_COMMITTER_EMAIL", "tests@example.invalid")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
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

/// A bare `[estate]` manifest with no `[[repo]]` entries yet (the state a
/// fresh `sgt init` leaves), with `data_dir` at the default nested path this
/// daemon-rig can start straight off, and returns `(estate_root, data_dir)`.
fn estate() -> (TempDir, std::path::PathBuf) {
    let root = TempDir::new().expect("estate tempdir");
    std::fs::write(
        root.path().join("sergeant.toml"),
        "[estate]\nname = \"t3-estate\"\n",
    )
    .expect("write sergeant.toml");
    let data_dir = root.path().join(".sergeant/data");
    std::fs::create_dir_all(&data_dir).expect("data dir");
    (root, data_dir)
}

/// Start a daemon on `data_dir`, **bound** to `estate_root` (§5.1). Every
/// `/v1/estate/*` route resolves the estate through that binding now.
async fn start(data_dir: &Path, _estate_root: &Path) -> DaemonHandle {
    daemon::start_with(
        data_dir,
        DaemonConfig {
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start")
}

/// D4: a client addresses the estate its estate-scoped requests mean. The
/// daemon binds none, so a client that named none would be refused by name
/// on every estate-scoped route — which is the contract, not a rig quirk.
fn client_for(handle: &DaemonHandle, estate_root: &Path) -> ApiClient {
    ApiClient::new(&handle.endpoint, &handle.token)
        .expect("client")
        .with_estate_root(estate_root)
}

// -------------------------------------------------------------- repos

#[tokio::test]
async fn get_estate_repos_reflects_the_manifest_empty_then_populated() {
    let (root, data_dir) = estate();
    let handle = start(&data_dir, root.path()).await;
    let client = client_for(&handle, root.path());

    let empty = client.repos().await.expect("get repos");
    assert_eq!(empty["repos"].as_array().expect("array").len(), 0);

    let source = TempDir::new().expect("source repo tempdir");
    init_repo(source.path());
    let origin = source.path().to_str().expect("utf8 path");
    client
        .add_repo("svc-a", Some(origin), None, None)
        .await
        .expect("add repo");

    let repos = client.repos().await.expect("get repos again");
    let list = repos["repos"].as_array().expect("array");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["name"], "svc-a");
    assert_eq!(list[0]["origin"], origin);
    assert!(
        root.path().join("repos/svc-a/.git").exists(),
        "the real populate-or-verify clone must have actually run"
    );

    handle.shutdown().await;
}

#[tokio::test]
async fn post_estate_repos_honors_the_instructions_choice() {
    let (root, data_dir) = estate();
    let handle = start(&data_dir, root.path()).await;
    let client = client_for(&handle, root.path());

    let source = TempDir::new().expect("source repo tempdir");
    init_repo(source.path());
    let origin = source.path().to_str().expect("utf8 path");
    let result = client
        .add_repo("svc-b", Some(origin), None, Some("local"))
        .await
        .expect("add repo");
    assert_eq!(result["instructions"], "local");

    let repos = client.repos().await.expect("get repos");
    let list = repos["repos"].as_array().expect("array");
    assert_eq!(list[0]["instructions"], "local");

    handle.shutdown().await;
}

#[tokio::test]
async fn post_estate_repos_refuses_a_duplicate_name_with_the_manifest_refusal() {
    let (root, data_dir) = estate();
    let handle = start(&data_dir, root.path()).await;
    let client = client_for(&handle, root.path());

    let source = TempDir::new().expect("source repo tempdir");
    init_repo(source.path());
    let origin = source.path().to_str().expect("utf8 path");
    client
        .add_repo("svc-a", Some(origin), None, None)
        .await
        .expect("first add");

    let err = client
        .add_repo("svc-a", Some(origin), None, None)
        .await
        .expect_err("a second add of the same name must be refused");
    let text = err.to_string();
    assert!(
        text.contains("already declared"),
        "the daemon's refusal must carry `manifest::add_repo`'s own wording, not a reworded \
         one: {text}"
    );

    handle.shutdown().await;
}

#[tokio::test]
async fn delete_estate_repos_removes_the_declaration_but_not_the_clone() {
    let (root, data_dir) = estate();
    let handle = start(&data_dir, root.path()).await;
    let client = client_for(&handle, root.path());

    let source = TempDir::new().expect("source repo tempdir");
    init_repo(source.path());
    let origin = source.path().to_str().expect("utf8 path");
    client
        .add_repo("svc-a", Some(origin), None, None)
        .await
        .expect("add repo");

    client.remove_repo("svc-a").await.expect("remove repo");

    let repos = client.repos().await.expect("get repos");
    assert_eq!(repos["repos"].as_array().expect("array").len(), 0);
    assert!(
        root.path().join("repos/svc-a/.git").exists(),
        "§12.1: removing the estate declaration must never delete repos/<name> from disk"
    );

    handle.shutdown().await;
}

#[tokio::test]
async fn delete_estate_repos_refuses_while_a_group_still_references_it() {
    let (root, data_dir) = estate();
    let handle = start(&data_dir, root.path()).await;
    let client = client_for(&handle, root.path());

    let source = TempDir::new().expect("source repo tempdir");
    init_repo(source.path());
    let origin = source.path().to_str().expect("utf8 path");
    client
        .add_repo("svc-a", Some(origin), None, None)
        .await
        .expect("add repo");
    client
        .add_group("core", &["svc-a".to_string()], None)
        .await
        .expect("add group");

    let err = client
        .remove_repo("svc-a")
        .await
        .expect_err("still referenced by a group");
    let text = err.to_string();
    assert!(
        text.contains("core") && text.contains("still a member"),
        "the group-reference refusal must name the referencing group and stay structured: \
         {text}"
    );

    handle.shutdown().await;
}

// -------------------------------------------------------------- groups

#[tokio::test]
async fn post_estate_groups_creates_then_extends_by_union() {
    let (root, data_dir) = estate();
    let handle = start(&data_dir, root.path()).await;
    let client = client_for(&handle, root.path());

    let source = TempDir::new().expect("source repo tempdir");
    init_repo(source.path());
    let origin = source.path().to_str().expect("utf8 path");
    for name in ["svc-a", "svc-b"] {
        client
            .add_repo(name, Some(origin), None, None)
            .await
            .expect("add repo");
    }

    let created = client
        .add_group("core", &["svc-a".to_string()], Some("core services"))
        .await
        .expect("create group");
    assert_eq!(created["repos"], serde_json::json!(["svc-a"]));

    // mkdir-p semantics: a second `add_group` on the same name unions in the
    // new member and leaves the existing one untouched, rather than erroring
    // or replacing the membership outright.
    let extended = client
        .add_group("core", &["svc-b".to_string()], None)
        .await
        .expect("extend group");
    let members: Vec<&str> = extended["repos"]
        .as_array()
        .expect("array")
        .iter()
        .map(|v| v.as_str().expect("str"))
        .collect();
    assert_eq!(members, vec!["svc-a", "svc-b"]);

    handle.shutdown().await;
}

#[tokio::test]
async fn post_estate_groups_refuses_an_undeclared_member() {
    let (root, data_dir) = estate();
    let handle = start(&data_dir, root.path()).await;
    let client = client_for(&handle, root.path());

    let err = client
        .add_group("core", &["ghost".to_string()], None)
        .await
        .expect_err("ghost was never declared as a repository");
    assert!(
        err.to_string().contains("not a declared repository"),
        "{err}"
    );

    handle.shutdown().await;
}

#[tokio::test]
async fn get_estate_groups_reflects_the_manifest() {
    let (root, data_dir) = estate();
    let handle = start(&data_dir, root.path()).await;
    let client = client_for(&handle, root.path());

    let source = TempDir::new().expect("source repo tempdir");
    init_repo(source.path());
    let origin = source.path().to_str().expect("utf8 path");
    client
        .add_repo("svc-a", Some(origin), None, None)
        .await
        .expect("add repo");
    client
        .add_group("core", &["svc-a".to_string()], Some("core services"))
        .await
        .expect("add group");

    let groups = client.groups().await.expect("get groups");
    let list = groups["groups"].as_array().expect("array");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["name"], "core");
    assert_eq!(list[0]["brief"], "core services");

    handle.shutdown().await;
}

#[tokio::test]
async fn delete_estate_groups_with_named_repos_removes_only_those_members() {
    let (root, data_dir) = estate();
    let handle = start(&data_dir, root.path()).await;
    let client = client_for(&handle, root.path());

    let source = TempDir::new().expect("source repo tempdir");
    init_repo(source.path());
    let origin = source.path().to_str().expect("utf8 path");
    for name in ["svc-a", "svc-b"] {
        client
            .add_repo(name, Some(origin), None, None)
            .await
            .expect("add repo");
    }
    client
        .add_group("core", &["svc-a".to_string(), "svc-b".to_string()], None)
        .await
        .expect("add group");

    client
        .remove_group("core", &["svc-a".to_string()])
        .await
        .expect("remove one member");

    let groups = client.groups().await.expect("get groups");
    let list = groups["groups"].as_array().expect("array");
    assert_eq!(
        list.len(),
        1,
        "the group itself must survive a partial removal"
    );
    assert_eq!(list[0]["repos"], serde_json::json!(["svc-b"]));

    handle.shutdown().await;
}

#[tokio::test]
async fn delete_estate_groups_with_no_repos_removes_the_whole_group() {
    let (root, data_dir) = estate();
    let handle = start(&data_dir, root.path()).await;
    let client = client_for(&handle, root.path());

    let source = TempDir::new().expect("source repo tempdir");
    init_repo(source.path());
    let origin = source.path().to_str().expect("utf8 path");
    client
        .add_repo("svc-a", Some(origin), None, None)
        .await
        .expect("add repo");
    client
        .add_group("core", &["svc-a".to_string()], None)
        .await
        .expect("add group");

    client
        .remove_group("core", &[])
        .await
        .expect("remove the whole group");

    let groups = client.groups().await.expect("get groups");
    assert_eq!(groups["groups"].as_array().expect("array").len(), 0);

    handle.shutdown().await;
}

#[tokio::test]
async fn delete_estate_groups_refuses_a_nonmember() {
    let (root, data_dir) = estate();
    let handle = start(&data_dir, root.path()).await;
    let client = client_for(&handle, root.path());

    let source = TempDir::new().expect("source repo tempdir");
    init_repo(source.path());
    let origin = source.path().to_str().expect("utf8 path");
    for name in ["svc-a", "svc-b"] {
        client
            .add_repo(name, Some(origin), None, None)
            .await
            .expect("add repo");
    }
    client
        .add_group("core", &["svc-a".to_string()], None)
        .await
        .expect("add group");

    let err = client
        .remove_group("core", &["svc-b".to_string()])
        .await
        .expect_err("svc-b was never a member of core");
    assert!(err.to_string().contains("not a member"), "{err}");

    handle.shutdown().await;
}

// -------------------------------------------------------------- doctor

#[tokio::test]
async fn get_doctor_matches_the_shape_sgt_doctor_json_already_prints() {
    let (root, data_dir) = estate();
    let handle = start(&data_dir, root.path()).await;
    let client = client_for(&handle, root.path());

    let report = client.doctor().await.expect("get doctor report");
    assert!(report["healthy"].is_boolean());
    assert_eq!(
        report["data_dir"],
        Value::String(data_dir.display().to_string())
    );
    let checks = report["checks"].as_array().expect("checks array");
    assert!(!checks.is_empty(), "a real report names at least one check");
    for check in checks {
        assert!(check["name"].is_string());
        assert!(matches!(
            check["status"].as_str(),
            Some("ok" | "warn" | "fail")
        ));
        assert!(check["detail"].is_string());
    }
    let names: Vec<&str> = checks.iter().filter_map(|c| c["name"].as_str()).collect();
    assert!(names.contains(&"estate"), "{names:?}");
    assert!(names.contains(&"disk_pressure"), "{names:?}");

    handle.shutdown().await;
}
