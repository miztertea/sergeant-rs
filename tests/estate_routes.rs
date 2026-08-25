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
//! Every estate here uses the default `<estate_root>/.sergeant/data` layout.
//! Under H1 the daemon is bound to **no** estate: `src/api.rs`'s
//! `resolve_estate_root` reads the estate off the *request* (D4) and admits
//! it, so these routes depend on nothing but the root each request names —
//! and, as before, not at all on the test process's own working directory
//! (`current_dir` is global process state, and nothing here needs to touch
//! it).

use std::path::Path;
use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

use sergeant_rs::api::{ApiClient, ClientError};
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

// ------------------------------------------------- H1: the estate registry

/// `GET /v1/estates` (H1 §4, brief deliverable 6): the registry is
/// **observational**. It is empty on a daemon nobody has addressed, gains a
/// row the moment one is, and never gains a row for a root that failed
/// admission — a refused admission is not an observation.
#[tokio::test]
async fn get_estates_is_empty_until_a_request_addresses_one() {
    let (root, data_dir) = estate();
    let handle = start(&data_dir, root.path()).await;

    let unaddressed = ApiClient::new(&handle.endpoint, &handle.token).expect("client");
    let listed = unaddressed.estates().await.expect("GET /v1/estates");
    assert_eq!(
        listed["estates"].as_array().expect("estates").len(),
        0,
        "a host daemon starts bound to zero estates: {listed}"
    );

    // A root that is not an estate is refused *and leaves no row*.
    let bogus = TempDir::new().expect("tempdir");
    let refused = ApiClient::new(&handle.endpoint, &handle.token)
        .expect("client")
        .with_estate_root(bogus.path());
    refused
        .repos()
        .await
        .expect_err("a non-estate must be refused");
    let listed = unaddressed.estates().await.expect("GET /v1/estates");
    assert_eq!(
        listed["estates"].as_array().expect("estates").len(),
        0,
        "a refused admission must not become a registry row: {listed}"
    );

    // A real one is admitted on first contact, and reported with its
    // canonical root, its display name, and its availability.
    let client = client_for(&handle, root.path());
    client.repos().await.expect("GET /v1/estate/repos");
    let listed = unaddressed.estates().await.expect("GET /v1/estates");
    let rows = listed["estates"].as_array().expect("estates");
    assert_eq!(rows.len(), 1, "{listed}");
    assert_eq!(
        rows[0]["root"],
        Value::String(
            std::fs::canonicalize(root.path())
                .expect("canonical")
                .to_string_lossy()
                .into_owned()
        ),
        "the coordinate is the canonical root (D1), never the display name"
    );
    assert_eq!(rows[0]["name"], "t3-estate");
    assert_eq!(rows[0]["available"], true);
    assert!(rows[0]["admitted_at"].is_string(), "{listed}");

    handle.shutdown().await;
}

/// D4's refusal taxonomy, over the wire, on a route that cannot proceed
/// without an estate:
///
/// - (c) no estate addressed at all → `404 no_estate`, naming what to send;
/// - (a) a root that is not an estate → `422 invalid_estate`, carrying
///   §4.4's own corrective block rather than a summary of it.
///
/// This is the negative the retired "bound to a different estate" refusal
/// used to carry, re-pointed rather than dropped: what must never happen is
/// serving a request for an estate nobody validated, and that is still
/// exactly what happens here.
#[tokio::test]
async fn an_unaddressed_or_unadmitted_estate_is_refused_by_name() {
    let (root, data_dir) = estate();
    let handle = start(&data_dir, root.path()).await;

    let unaddressed = ApiClient::new(&handle.endpoint, &handle.token).expect("client");
    let err = unaddressed
        .repos()
        .await
        .expect_err("an estate-scoped route with no estate must refuse");
    let ClientError::Api {
        status,
        code,
        message,
    } = &err
    else {
        panic!("expected a structured refusal, got {err}");
    };
    assert_eq!((*status, code.as_str()), (404, "no_estate"), "{err}");
    assert!(
        message.contains("estate_root"),
        "the refusal must say what to send: {message}"
    );

    let bogus = TempDir::new().expect("tempdir");
    let err = ApiClient::new(&handle.endpoint, &handle.token)
        .expect("client")
        .with_estate_root(bogus.path())
        .repos()
        .await
        .expect_err("a root that is not an estate must refuse");
    let ClientError::Api {
        status,
        code,
        message,
    } = &err
    else {
        panic!("expected a structured refusal, got {err}");
    };
    assert_eq!((*status, code.as_str()), (422, "invalid_estate"), "{err}");
    assert!(
        message.contains("does not search parent directories"),
        "§4.4's own diagnostic must survive to the client verbatim: {message}"
    );

    handle.shutdown().await;
}
