//! #159 acceptance: §12.3's deliberate sweep, end to end over a real estate.
//!
//! The classification half (`GET /v1/sweep`) is a pure read; the deletion
//! half (`POST /v1/sweep`) is a separate authorization that re-proves its own
//! criterion per branch and journals what it destroyed. The invariants under
//! test are the ones that make the pair safe to hand an operator:
//!
//! - classification mutates nothing,
//! - a `sergeant/*` ref sergeant never journaled is reported as an orphan and
//!   is never deletable (#172's fold-in),
//! - deletion without `"confirm": true` mutates nothing and says so,
//! - deletion destroys only what is `redundant` at deletion time, and the
//!   `command.accepted` record carries the tip SHA of everything it deleted.
//!
//! Same in-process daemon rig as `tests/m6_surfaces.rs`/`tests/
//! estate_routes.rs`: a real daemon bound to a real estate, driven over its
//! loopback API with the fake backend, so the `sergeant/*` refs under test
//! are the ones a real Work actually left behind.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::api::ApiClient;
use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig, DaemonHandle};
use sergeant_rs::runtime::sweep::SweepTarget;

mod support;
use support::{git, scaffold_solo_estate};

/// A `sergeant/*` ref no Work ever created — #172's orphan, planted by hand
/// exactly the way a mount picks one up (an operator's stray branch, a
/// restored backup, a Work journaled into a data dir that is now gone).
const ORPHAN_BRANCH: &str = "sergeant/01BOGUSBOGUSBOGUSBOGUSBOGUS";

/// A one-stage workflow, so one `FakeStep` drives one Work to a terminal
/// state and its surface is torn down with the branch retained (§11).
fn write_solo_workflow(root: &Path) {
    let dir = root.join(".sergeant/workflows/solo");
    std::fs::create_dir_all(&dir).expect("workflow dir");
    std::fs::write(
        dir.join("workflow.toml"),
        "[workflow]\nname = \"solo\"\nversion = \"1\"\nstages = [\"00-only\"]\n",
    )
    .expect("workflow.toml");
    std::fs::create_dir_all(dir.join("00-only")).expect("stage dir");
    std::fs::write(dir.join("00-only").join("CONTEXT.md"), "context").expect("CONTEXT.md");
}

/// An estate with one mount, a solo workflow, and a running daemon bound to
/// it. Returns the estate root, the mount, the data dir and the handle.
async fn rig() -> (TempDir, PathBuf, TempDir, DaemonHandle) {
    let root = TempDir::new().expect("estate tempdir");
    let (mount, _head) = scaffold_solo_estate(root.path(), "svc");
    write_solo_workflow(root.path());
    let data = TempDir::new().expect("data tempdir");
    let fake = FakeBackend::scripted(FAKE_BACKEND_NAME, [FakeStep::complete()]);
    let handle = daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(BackendRegistry::new().with(Arc::new(fake))),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            estate_root: Some(root.path().to_path_buf()),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");
    (root, mount, data, handle)
}

/// Submit one Work and wait for it to reach a terminal state, returning its
/// id. The fake completes on its first step, so this settles immediately;
/// the poll exists because the crank is asynchronous, not because anything
/// here is timing-dependent.
async fn completed_work(client: &ApiClient, cwd: &Path) -> String {
    let submitted = client
        .post(
            "/v1/work",
            &json!({
                "command_id": ulid::Ulid::generate().to_string(),
                "intent": "leave a branch behind",
                "workflow": "solo",
                "origin": {"client": "cli", "cwd": cwd},
            }),
        )
        .await
        .expect("submit");
    let id = submitted["work"]["id"]
        .as_str()
        .expect("work id")
        .to_string();
    for _ in 0..200 {
        let work = client
            .get(&format!("/v1/work/{id}"))
            .await
            .expect("read work");
        if matches!(
            work["work"]["state"].as_str(),
            Some("completed" | "completed_dirty" | "failed" | "canceled")
        ) {
            return id;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    panic!("work {id} never reached a terminal state");
}

/// The one repository entry of a sweep report.
fn only_repository(report: &Value) -> &Value {
    let repositories = report["repositories"].as_array().expect("repositories");
    assert_eq!(repositories.len(), 1, "{report}");
    &repositories[0]
}

fn branch<'a>(report: &'a Value, name: &str) -> &'a Value {
    only_repository(report)["branches"]
        .as_array()
        .expect("branches")
        .iter()
        .find(|b| b["branch"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("no {name} in {report}"))
}

fn refs_in(mount: &Path) -> String {
    git(mount, &["for-each-ref", "--format=%(refname:short)"])
}

/// guard-map: the classification pass names a terminal Work's fully-merged
/// branch `redundant` and an unjournaled `sergeant/*` ref `orphan`, and
/// leaves the ref store byte-identical. Mutation this kills: sweep repairing
/// or deleting anything on the read path, or classifying an unknown ref as
/// something it has evidence for.
#[tokio::test]
async fn classification_reports_redundant_and_orphan_and_mutates_nothing() {
    let (root, mount, _data, handle) = rig().await;
    let client = ApiClient::new(&handle.endpoint, &handle.token).expect("client");
    let id = completed_work(&client, root.path()).await;
    git(&mount, &["branch", ORPHAN_BRANCH, "main"]);

    let before = refs_in(&mount);
    let report = client.sweep().await.expect("sweep");

    assert_eq!(
        only_repository(&report)["default_branch"].as_str(),
        Some("main"),
        "the report names what every redundant verdict is relative to: {report}"
    );
    let work_branch = format!("sergeant/{id}");
    let swept = branch(&report, &work_branch);
    assert_eq!(
        swept["classification"], "redundant",
        "a terminal Work whose branch never left the base is contained in main: {report}"
    );
    assert_eq!(swept["work_id"], id, "{report}");
    assert_eq!(
        branch(&report, ORPHAN_BRANCH)["classification"],
        "orphan",
        "#172: a sergeant/* ref with no journaled Work is reported, never guessed at: {report}"
    );
    assert!(
        report["note"]
            .as_str()
            .expect("note")
            .contains("squash- or rebase-merge"),
        "the redundancy criterion carries its own caveat: {report}"
    );
    assert_eq!(
        before,
        refs_in(&mount),
        "a classification pass must leave the ref store byte-identical"
    );

    handle.shutdown().await;
}

/// guard-map: a deletion request without confirmation mutates nothing and
/// refuses by name. Mutation this kills: treating a bare or malformed body as
/// an implicit yes (`AGENTS.md`'s fail-closed rule).
#[tokio::test]
async fn deletion_without_confirmation_refuses_and_deletes_nothing() {
    let (root, mount, _data, handle) = rig().await;
    let client = ApiClient::new(&handle.endpoint, &handle.token).expect("client");
    let id = completed_work(&client, root.path()).await;

    let before = refs_in(&mount);
    let targets = [SweepTarget {
        repository: "svc".to_string(),
        branch: format!("sergeant/{id}"),
    }];
    let refusal = client
        .sweep_delete(&targets, false)
        .await
        .expect_err("an unconfirmed sweep must be refused");
    let text = refusal.to_string();
    assert!(
        text.contains("400") && text.contains("GET /v1/sweep"),
        "the refusal must name what to review first: {text}"
    );
    assert_eq!(
        before,
        refs_in(&mount),
        "an unconfirmed sweep must not have touched a single ref"
    );

    handle.shutdown().await;
}

/// guard-map: a confirmed sweep deletes exactly the redundant branch, refuses
/// the orphan asked for alongside it, and journals the tip SHA of what it
/// destroyed. Mutation this kills: deleting from the caller's list rather
/// than from a freshly re-proved classification, or journaling an acceptance
/// that does not record what is now gone.
#[tokio::test]
async fn a_confirmed_sweep_deletes_only_the_redundant_branch_and_journals_its_tip() {
    let (root, mount, _data, handle) = rig().await;
    let client = ApiClient::new(&handle.endpoint, &handle.token).expect("client");
    let id = completed_work(&client, root.path()).await;
    git(&mount, &["branch", ORPHAN_BRANCH, "main"]);
    let work_branch = format!("sergeant/{id}");
    let tip = git(&mount, &["rev-parse", &work_branch]);

    let targets = [
        SweepTarget {
            repository: "svc".to_string(),
            branch: work_branch.clone(),
        },
        SweepTarget {
            repository: "svc".to_string(),
            branch: ORPHAN_BRANCH.to_string(),
        },
    ];
    let result = client
        .sweep_delete(&targets, true)
        .await
        .expect("confirmed sweep");
    let deleted = result["deleted"].as_array().expect("deleted");
    assert_eq!(deleted[0]["outcome"], "deleted", "{result}");
    assert_eq!(deleted[0]["tip"], tip, "{result}");
    assert_eq!(
        deleted[1]["outcome"], "refused",
        "an orphan is never deletable by sweep: {result}"
    );

    let refs = refs_in(&mount);
    assert!(!refs.contains(&work_branch), "{refs}");
    assert!(refs.contains(ORPHAN_BRANCH), "{refs}");
    assert!(refs.contains("main"), "{refs}");

    // The journal records what was destroyed and where it was recoverable
    // from — the whole reason the tip travels in the accepted command.
    let events = client.get("/v1/events?from=0").await.expect("events");
    let accepted = events["events"]
        .as_array()
        .expect("events")
        .iter()
        .filter(|e| e["kind"] == "command.accepted")
        .find(|e| e["payload"]["operation"] == "work.sweep")
        .unwrap_or_else(|| panic!("no work.sweep acceptance in {events}"));
    let journaled = &accepted["payload"]["result"]["deleted"][0];
    assert_eq!(journaled["branch"], work_branch, "{accepted}");
    assert_eq!(journaled["tip"], tip, "{accepted}");

    handle.shutdown().await;
}
