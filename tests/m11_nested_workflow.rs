//! M11: nested workflow packages end to end (S2 W1 — hierarchical
//! execution; decisions E1/E3/E4/E10/E14).
//!
//! The loader half (recursion, composed ids, the boundary side table) is
//! unit-tested next to its own code in `src/domain/workflow.rs`. This suite
//! is the other half: a *running* Work whose workflow nests, driven through
//! the real daemon and the real engine on the fake backend, proving the
//! claims that only a run can prove —
//!
//! 1. two-level recursion executes its leaves through the existing engine,
//!    with no new execution path, and the Work completes (W1 §13 items 1-3);
//! 2. a nested leaf's own output contract is enforced *identically* to a
//!    non-nested leaf's (W1 §13 item 9 — split-hardening contracts are not
//!    bypassed by nesting);
//! 3. a container that declared its own output contract is checked when its
//!    last leaf completes, and an unmet one parks the Work naming the
//!    CONTAINER (E4).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig, DaemonHandle};
use sergeant_rs::domain::event::Event;
use sergeant_rs::domain::workflow::{
    KIND_STAGE_COMPLETED, KIND_STAGE_ENTERED, KIND_STAGE_OUTPUT_MISSING,
    REASON_STAGE_OUTPUT_MISSING,
};
use sergeant_rs::runtime::journal::Journal;

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

/// Write one workflow package's own `workflow.toml`, creating its directory.
/// A nested package is an ordinary package, so every level uses this.
fn write_package(dir: &Path, name: &str, stages: &[&str]) {
    std::fs::create_dir_all(dir).expect("package dir");
    let declared: Vec<String> = stages.iter().map(|id| format!("{id:?}")).collect();
    std::fs::write(
        dir.join("workflow.toml"),
        format!(
            "[workflow]\nname = {name:?}\nversion = \"1\"\nstages = [{}]\n",
            declared.join(", ")
        ),
    )
    .expect("workflow.toml");
}

/// An actor stage: a directory with a `CONTEXT.md`, which is its procedure.
fn write_stage(package: &Path, id: &str, context: &str) {
    let dir = package.join(id);
    std::fs::create_dir_all(&dir).expect("stage dir");
    std::fs::write(dir.join("CONTEXT.md"), context).expect("CONTEXT.md");
}

/// Declare an `output/README.md` contract (the ICM convention's Rule 4
/// shape) for `id` — a leaf id or a container id alike, since a container's
/// own `output/` sits beside its `workflow.toml` exactly as a leaf's sits
/// beside its `CONTEXT.md`.
fn declare_output(package: &Path, id: &str, artifact: &str) {
    let dir = package.join(id).join("output");
    std::fs::create_dir_all(&dir).expect("output dir");
    std::fs::write(
        dir.join("README.md"),
        format!(
            "# Output — `{id}`\n\n**Expected artifact:** `{artifact}` — proof of work.\n\n\
             **Disposition:** `evidence`\n"
        ),
    )
    .expect("output/README.md");
}

/// The two-level fixture every test below starts from:
///
/// ```text
/// nested/
///   00-orient/CONTEXT.md
///   10-investigate/workflow.toml      <- container marker
///     00-lead/CONTEXT.md
///     10-code/CONTEXT.md
///   20-implement/CONTEXT.md
/// ```
///
/// Flattens to `00-orient`, `10-investigate/00-lead`,
/// `10-investigate/10-code`, `20-implement`.
fn write_nested_workflow(root: &Path) -> PathBuf {
    let package = root.join(".sergeant/workflows/nested");
    write_package(
        &package,
        "nested",
        &["00-orient", "10-investigate", "20-implement"],
    );
    write_stage(&package, "00-orient", "orient the work");
    write_stage(&package, "20-implement", "implement it");
    let investigate = package.join("10-investigate");
    write_package(&investigate, "10-investigate", &["00-lead", "10-code"]);
    write_stage(&investigate, "00-lead", "lead the investigation");
    write_stage(&investigate, "10-code", "read the code");
    package
}

async fn start_fake(data_dir: &Path, script: impl IntoIterator<Item = FakeStep>) -> DaemonHandle {
    let fake = FakeBackend::scripted(FAKE_BACKEND_NAME, script);
    let registry = BackendRegistry::new().with(Arc::new(fake));
    daemon::start_with(
        data_dir,
        DaemonConfig {
            backends: Arc::new(registry),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            claude: None,
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start")
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

async fn submit(
    client: &reqwest::Client,
    handle: &DaemonHandle,
    estate: &Path,
    workflow: &str,
) -> Value {
    let (status, body) = post(
        client,
        handle,
        "/v1/work",
        json!({
            "command_id": ulid(),
            "intent": "run a nested workflow",
            "workflow": workflow,
            "estate_root": estate,
            "origin": {"client": "cli", "cwd": estate},
        }),
    )
    .await;
    assert_eq!(status, 201, "submit must be accepted: {body}");
    body
}

fn events_of(data_dir: &Path, work_id: &str, kind: &str) -> Vec<Event> {
    Journal::replay_data_dir(data_dir)
        .expect("replay")
        .map(|e| e.expect("event"))
        .filter(|e| e.work_id.as_deref() == Some(work_id) && e.kind == kind)
        .collect()
}

fn stage_ids(data_dir: &Path, work_id: &str, kind: &str) -> Vec<String> {
    events_of(data_dir, work_id, kind)
        .into_iter()
        .filter_map(|e| e.payload["stage_id"].as_str().map(str::to_string))
        .collect()
}

// ------------------------------------------------------------------ tests

/// W1 §13 items 1-3: a stage directory that is itself a workflow package
/// runs its leaves through the existing engine — same backend, same
/// surface, one flat stage order — and the Work completes. Nothing about
/// nesting reaches the executor: the leaves are ordinary stages whose ids
/// happen to contain `/`.
#[tokio::test]
async fn a_two_level_nested_workflow_runs_its_leaves_in_order_and_completes() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_nested_workflow(&estate);

    // One completion per leaf: four leaves, four steps.
    let handle = start_fake(
        data.path(),
        [
            FakeStep::complete_with("oriented"),
            FakeStep::complete_with("led"),
            FakeStep::complete_with("read"),
            FakeStep::complete_with("implemented"),
        ],
    )
    .await;
    let client = http();
    let body = submit(&client, &handle, &estate, "nested").await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();

    assert_eq!(
        body["work"]["state"], "completed",
        "a nested workflow must run to completion on the ordinary path: {body}"
    );
    assert_eq!(
        stage_ids(data.path(), &work_id, KIND_STAGE_ENTERED),
        [
            "00-orient",
            "10-investigate/00-lead",
            "10-investigate/10-code",
            "20-implement",
        ],
        "leaves enter in flattened document order, with composed hierarchical ids"
    );
    assert_eq!(
        stage_ids(data.path(), &work_id, KIND_STAGE_COMPLETED).len(),
        4,
        "every leaf completes; the container itself never enters or completes"
    );
    // E3 / W1-02: a container is never a stage. No event anywhere names it.
    let entered = stage_ids(data.path(), &work_id, KIND_STAGE_ENTERED);
    assert!(
        !entered.iter().any(|id| id == "10-investigate"),
        "the container must never be entered as a stage: {entered:?}"
    );
    // E10: the wire shape stays the flat leaf list — hierarchical ids as
    // opaque strings, with the container appearing nowhere in it.
    let workflow = &body["workflow"];
    assert_eq!(
        workflow["stages"]
            .as_array()
            .expect("stages")
            .iter()
            .filter_map(|s| s.as_str())
            .collect::<Vec<_>>(),
        [
            "00-orient",
            "10-investigate/00-lead",
            "10-investigate/10-code",
            "20-implement",
        ],
        "the wire shape stays the flat leaf list (E10): {workflow}"
    );

    handle.shutdown().await;
}

/// W1 §13 item 9: a *leaf's* own declared output contract behaves
/// identically nested and not. The same fixture as `m3_execution.rs`'s
/// `t10`, one level down: the artifact is missing, the engine spends its one
/// bounded re-prompt on that leaf, and the second miss parks the Work naming
/// the leaf's own hierarchical id — not the container's.
#[tokio::test]
async fn a_nested_leafs_own_output_contract_is_enforced_exactly_as_a_flat_ones_is() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    let package = write_nested_workflow(&estate);
    // The contract is declared on the nested leaf itself, at
    // `10-investigate/00-lead/output/README.md`.
    declare_output(&package.join("10-investigate"), "00-lead", "lead.md");

    let handle = start_fake(
        data.path(),
        [
            FakeStep::complete(), // 00-orient
            FakeStep::complete(), // 00-lead: "done", lead.md absent
            FakeStep::complete(), // the one bounded re-prompt: still absent
        ],
    )
    .await;
    let client = http();
    let body = submit(&client, &handle, &estate, "nested").await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();

    assert_eq!(
        body["work"]["state"], "needs_input",
        "the nested leaf's unmet contract must park the Work: {body}"
    );
    assert_eq!(
        body["stage"]["stage_id"], "10-investigate/00-lead",
        "the parked stage is the nested leaf, named by its composed id: {body}"
    );
    let reprompts = events_of(data.path(), &work_id, KIND_STAGE_OUTPUT_MISSING);
    assert_eq!(reprompts.len(), 1, "exactly one bounded re-prompt");
    assert_eq!(reprompts[0].payload["stage_id"], "10-investigate/00-lead");
    assert_eq!(reprompts[0].payload["path"], "lead.md");
    assert!(
        reprompts[0].payload["container_id"].is_null(),
        "a leaf's own contract is not a container's: {:?}",
        reprompts[0].payload
    );
    let parked = events_of(data.path(), &work_id, "work.needs_input");
    assert_eq!(
        parked[0].payload["reason_code"],
        REASON_STAGE_OUTPUT_MISSING
    );
    assert_eq!(parked[0].payload["stage_id"], "10-investigate/00-lead");
    // Only the leaf that failed its contract is unfinished: 00-orient
    // completed normally before it.
    assert_eq!(
        stage_ids(data.path(), &work_id, KIND_STAGE_COMPLETED),
        ["00-orient"]
    );

    handle.shutdown().await;
}

/// E4: a container may declare its own output contract, checked at the
/// moment its last leaf completes — the leaf itself having finished cleanly.
/// The gate is the existing one verbatim (one bounded re-prompt of that
/// leaf, then a `stage_output_missing`-class park), and what the operator is
/// told names the **container**, so "the aggregate was never written" does
/// not read as a complaint about a leaf that did its job.
#[tokio::test]
async fn a_container_that_never_produced_its_declared_output_parks_naming_the_container() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    let package = write_nested_workflow(&estate);
    // Declared on the CONTAINER: `10-investigate/output/README.md`, sibling
    // to `10-investigate/workflow.toml`, distinct from either leaf's own.
    declare_output(&package, "10-investigate", "findings.md");

    let handle = start_fake(
        data.path(),
        [
            FakeStep::complete(), // 00-orient
            FakeStep::complete(), // 10-investigate/00-lead
            FakeStep::complete(), // 10-investigate/10-code — closes the container
            FakeStep::complete(), // the container gate's one bounded re-prompt
        ],
    )
    .await;
    let client = http();
    let body = submit(&client, &handle, &estate, "nested").await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();

    assert_eq!(
        body["work"]["state"], "needs_input",
        "a closed container with an unmet contract must park the Work: {body}"
    );
    // The parked stage is still the leaf — a container has no StageRecord
    // (E3) — but everything the operator reads names the container.
    assert_eq!(body["stage"]["stage_id"], "10-investigate/10-code");
    let reprompts = events_of(data.path(), &work_id, KIND_STAGE_OUTPUT_MISSING);
    assert_eq!(
        reprompts.len(),
        1,
        "the container gate reuses the same bounded re-prompt, not a new mechanism"
    );
    assert_eq!(reprompts[0].payload["container_id"], "10-investigate");
    assert_eq!(reprompts[0].payload["stage_id"], "10-investigate/10-code");
    assert_eq!(reprompts[0].payload["path"], "findings.md");
    let parked = events_of(data.path(), &work_id, "work.needs_input");
    assert_eq!(parked.len(), 1);
    assert_eq!(
        parked[0].payload["reason_code"],
        REASON_STAGE_OUTPUT_MISSING
    );
    assert_eq!(parked[0].payload["container_id"], "10-investigate");
    let prompt = parked[0].payload["prompt"].as_str().expect("prompt");
    assert!(
        prompt.contains("container 10-investigate") && prompt.contains("findings.md"),
        "the operator must be told which container never produced what: {prompt}"
    );
    // The run never walked past the container: 20-implement never entered.
    let entered = stage_ids(data.path(), &work_id, KIND_STAGE_ENTERED);
    assert!(
        !entered.iter().any(|id| id == "20-implement"),
        "an unmet container contract must hold the run at its boundary: {entered:?}"
    );

    handle.shutdown().await;
}

/// The other half of E4, which is the one that must not regress: a container
/// whose declared output *is* present closes silently. No new event kind, no
/// container-level record — the next leaf entering is the only evidence the
/// container completed, exactly as it is for a top-level stage today.
#[tokio::test]
async fn a_container_whose_declared_output_is_present_closes_silently() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    let package = write_nested_workflow(&estate);
    declare_output(&package, "10-investigate", "findings.md");

    let handle = start_fake(
        data.path(),
        [
            FakeStep::complete(),
            FakeStep::complete(),
            FakeStep::complete(),
            FakeStep::complete(),
        ],
    )
    .await;
    let client = http();
    let body = submit(&client, &handle, &estate, "nested").await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    let worktree = PathBuf::from(
        body["surface"]["bindings"][0]["worktree_path"]
            .as_str()
            .expect("worktree"),
    );

    // The fake actor writes nothing, so the container's aggregate is
    // missing on the first pass and the Work parks — the same gate the test
    // above pins. Produce it and answer: the container closes, the run
    // walks on to the container's next sibling, and the Work completes.
    assert_eq!(body["work"]["state"], "needs_input", "{body}");
    std::fs::create_dir_all(worktree.join("10-investigate/output")).expect("output dir");
    std::fs::write(
        worktree.join("10-investigate/output/findings.md"),
        "what we found\n",
    )
    .expect("write the container's aggregate");

    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/input"),
        json!({"command_id": ulid(), "input": "there, the aggregate is written"}),
    )
    .await;
    assert_eq!(status, 200, "input rejected: {body}");
    assert_eq!(
        body["work"]["state"], "completed",
        "with the container's artifact present the run must walk past the boundary: {body}"
    );
    // No new vocabulary: the container's completion is evidenced only by
    // the next leaf entering.
    let entered = stage_ids(data.path(), &work_id, KIND_STAGE_ENTERED);
    assert_eq!(
        entered,
        [
            "00-orient",
            "10-investigate/00-lead",
            "10-investigate/10-code",
            "20-implement",
        ]
    );
    assert!(
        Journal::replay_data_dir(data.path())
            .expect("replay")
            .map(|e| e.expect("event"))
            .filter(|e| e.work_id.as_deref() == Some(&work_id))
            .all(|e| !e.kind.starts_with("container.")),
        "W1 adds no new event kind for a container"
    );

    handle.shutdown().await;
}
