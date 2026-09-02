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
//!    CONTAINER (E4);
//! 4. a composed id round-trips through the journal, the analytics fold and
//!    `sgt work show`, and a restarted daemon reconstructs the deepest
//!    incomplete path from the journal alone (W1 §13 item 4);
//! 5. the multi-boundary case the plan panel named — a three-level fixture
//!    whose deepest leaf is simultaneously the last leaf of two ancestor
//!    containers, gated innermost first.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig, DaemonHandle};
use sergeant_rs::domain::event::Event;
use sergeant_rs::domain::workflow::{
    KIND_STAGE_COMPLETED, KIND_STAGE_ENTERED, KIND_STAGE_FAILED, KIND_STAGE_OUTPUT_MISSING,
    REASON_STAGE_OUTPUT_MISSING,
};
use sergeant_rs::runtime::atlas::db::Analytics;
use sergeant_rs::runtime::journal::Journal;

mod support;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

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

/// [`declare_output`], but additionally declares Amendment 10d's
/// `**Required columns:**` line — the required-column half of the
/// split-hardening contract, exercised here against a nested leaf's composed
/// id exactly as [`declare_output`] exercises the expected-artifact half
/// (W1 §13 item 9, required-column side; mirrors
/// `tests/m3_execution.rs::write_two_stage_workflow_with_required_columns`,
/// the flat-case precedent).
fn declare_output_with_required_columns(
    package: &Path,
    id: &str,
    artifact: &str,
    columns: &[&str],
) {
    let dir = package.join(id).join("output");
    std::fs::create_dir_all(&dir).expect("output dir");
    let quoted: Vec<String> = columns.iter().map(|c| format!("`{c}`")).collect();
    std::fs::write(
        dir.join("README.md"),
        format!(
            "# Output — `{id}`\n\n**Expected artifact:** `{artifact}` — proof of \
             work.\n\n**Required columns:** {} — a typed set, not prose.\n\n**Disposition:** \
             `evidence`\n",
            quoted.join(", ")
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

/// W1 §13 item 9, the required-column half: a nested leaf's declared
/// `**Required columns:**` line (Amendment 10d) is enforced against its
/// composed hierarchical id exactly as a flat leaf's is —
/// `has_required_table_columns` is reached through `check_output_contract`'s
/// `contract_id` regardless of nesting depth. Mirrors
/// `tests/m3_execution.rs::t11_a_present_but_untyped_declared_artifact_is_refused_the_same_way_as_a_missing_one`,
/// the flat-case precedent this test carries one level deeper.
#[tokio::test]
async fn a_nested_leafs_required_column_contract_is_enforced_exactly_as_a_flat_ones_is() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    let (mount, _head) = support::scaffold_solo_estate(&estate, "solo");
    let package = write_nested_workflow(&estate);
    // The contract is declared on the nested leaf itself, at
    // `10-investigate/00-lead/output/README.md`, this time with a
    // **Required columns:** line.
    declare_output_with_required_columns(
        &package.join("10-investigate"),
        "00-lead",
        "lead.md",
        &["id", "axis"],
    );

    // The declared artifact must exist *before* the stage launches (the fake
    // actor writes nothing) — present, but untyped prose rather than the
    // required table, so the check must reach past "does the file exist" to
    // "does it carry the required columns" for a nested id.
    std::fs::create_dir_all(mount.join("10-investigate/00-lead/output")).expect("output dir");
    std::fs::write(
        mount.join("10-investigate/00-lead/output/lead.md"),
        "Findings: one about id 1, axis correctness. No table here.\n",
    )
    .expect("seed untyped artifact");
    support::git(&mount, &["add", "-A"]);
    support::git(&mount, &["commit", "-m", "seed untyped nested artifact"]);

    let handle = start_fake(
        data.path(),
        [
            FakeStep::complete(), // 00-orient
            FakeStep::complete(), // 00-lead: "done", lead.md present but untyped
            FakeStep::complete(), // the one bounded re-prompt: still untyped
        ],
    )
    .await;
    let client = http();
    let body = submit(&client, &handle, &estate, "nested").await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();

    assert_eq!(
        body["work"]["state"], "needs_input",
        "a present-but-untyped nested artifact must be refused, not accepted: {body}"
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
        "a leaf's own required-column contract is not a container's: {:?}",
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

/// W1 §13 item 4: a composed hierarchical id is an opaque string everywhere
/// downstream of the loader, and this proves it at each of the four places
/// that could have parsed it and did not have to — the journal's own events,
/// the analytics `stages` rows folded from them, the read surface `sgt work
/// show` prints, and (below) a restarted daemon's reconstruction of the
/// deepest incomplete path.
///
/// The claim is deliberately negative: **nothing new was needed**. The
/// reducer, the fold and the read model never parsed a stage id's structure,
/// so a `/` in one changes nothing for them — which is the whole reason
/// hierarchy was encoded in the existing string id (W1-04) rather than in a
/// parallel tree.
#[tokio::test]
async fn a_composed_stage_id_round_trips_through_events_analytics_and_work_show() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_nested_workflow(&estate);

    let handle = start_fake(
        data.path(),
        [
            FakeStep::complete(),                       // 00-orient
            FakeStep::needs_input("which lead first?"), // 10-investigate/00-lead parks here
        ],
    )
    .await;
    let client = http();
    let body = submit(&client, &handle, &estate, "nested").await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "needs_input", "{body}");

    // 1. Events. Every stage event for the nested leaf carries the composed
    //    id verbatim — no escaping, no splitting, no parent field.
    assert_eq!(
        stage_ids(data.path(), &work_id, KIND_STAGE_ENTERED),
        ["00-orient", "10-investigate/00-lead"]
    );
    assert_eq!(
        stage_ids(data.path(), &work_id, "stage.needs_input"),
        ["10-investigate/00-lead"]
    );

    // 2. Analytics. The `stages` table is folded from those events by a pure
    //    reducer; the composed id lands in its `stage_id` column unchanged,
    //    keyed and indexed like any other.
    let events: Vec<Result<Event, _>> = Journal::replay_data_dir(data.path())
        .expect("replay")
        .collect();
    let mut analytics = Analytics::in_memory(events).expect("fold the journal");
    let stages = analytics.table_rows("stages").expect("stages table");
    let stage_id_column = stages
        .columns
        .iter()
        .position(|c| c == "stage_id")
        .expect("stage_id column");
    let idx_column = stages
        .columns
        .iter()
        .position(|c| c == "idx")
        .expect("idx column");
    let nested_row = stages
        .rows
        .iter()
        .find(|row| row[stage_id_column] == json!("10-investigate/00-lead"))
        .unwrap_or_else(|| panic!("no analytics row for the nested leaf: {:?}", stages.rows));
    assert_eq!(
        nested_row[idx_column],
        json!(1),
        "the analytics row keeps the leaf's flat index: {nested_row:?}"
    );

    // 3. The read surface. `sgt work show` prints the same record the API
    //    serves, so this spawns the real binary rather than asserting the
    //    API body twice.
    //
    //    `spawn_blocking`, not a bare `Command::output()`: this daemon lives
    //    on the test's own current-thread runtime, so blocking that thread
    //    on a child process that is trying to reach it would deadlock the
    //    server it is calling.
    let (data_dir, cwd, id) = (data.path().to_path_buf(), estate.clone(), work_id.clone());
    let output = tokio::task::spawn_blocking(move || {
        Command::new(SGT)
            .arg("--data-dir")
            .arg(&data_dir)
            .args(["work", "show", &id])
            .current_dir(&cwd)
            .stdin(std::process::Stdio::null())
            .output()
            .expect("run sgt work show")
    })
    .await
    .expect("join the spawned CLI");
    assert!(
        output.status.success(),
        "sgt work show failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let shown: Value =
        serde_json::from_slice(&output.stdout).expect("work show prints one JSON record");
    assert_eq!(
        shown["stage"]["stage_id"], "10-investigate/00-lead",
        "work show must name the leaf by its composed id: {shown}"
    );
    assert_eq!(
        shown["workflow"]["stages"][1], "10-investigate/00-lead",
        "and carry the flat leaf list E10 settled: {shown}"
    );

    handle.shutdown().await;
}

/// W1 §9 / §13 item 4's recovery half: a daemon that dies mid-nest comes
/// back knowing exactly which leaf was in flight, *from the journal alone*.
///
/// Nothing reconstructs a tree to do it. The pinned `workflow.bound` carries
/// the already-flattened stage list, the journal carries whatever stage id
/// was current, and "the deepest incomplete path" is simply the last
/// `StageRecord` — whose id *is* that path by construction. This test
/// genuinely kills the daemon (`DaemonHandle::kill`, an abrupt task-abort
/// with no cooperative shutdown — see its doc comment) while a nested leaf
/// is the failed, current stage, and asserts both halves: the restarted
/// daemon names that leaf, and `retry` re-enters that leaf rather than an
/// ancestor or the container's first child.
#[tokio::test]
async fn a_restarted_daemon_reconstructs_the_deepest_incomplete_path_and_retry_re_enters_it() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_nested_workflow(&estate);

    let handle = start_fake(
        data.path(),
        [
            FakeStep::complete(),                      // 00-orient
            FakeStep::complete(),                      // 10-investigate/00-lead
            FakeStep::fail("the nested leaf gave up"), // 10-investigate/10-code
        ],
    )
    .await;
    let client = http();
    let body = submit(&client, &handle, &estate, "nested").await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "failed", "{body}");
    assert_eq!(
        stage_ids(data.path(), &work_id, KIND_STAGE_FAILED),
        ["10-investigate/10-code"],
        "the failure event names the nested leaf, which IS the container-scoped failure report"
    );
    handle.kill().await;

    // A fresh daemon over the same journal, which re-reads nothing from the
    // repository. Its remaining script picks up where the dead one left off.
    let handle = start_fake(
        data.path(),
        [
            FakeStep::complete(), // the retried 10-investigate/10-code
            FakeStep::complete(), // 20-implement
        ],
    )
    .await;
    let view = get(&client, &handle, &format!("/v1/work/{work_id}")).await;
    assert_eq!(
        view["stage"]["stage_id"], "10-investigate/10-code",
        "the restarted daemon's current stage is the deepest incomplete path: {view}"
    );
    assert_eq!(view["stage"]["index"], 2, "at its flat index: {view}");
    assert_eq!(
        view["stage"]["of"], 4,
        "of the flattened leaf count: {view}"
    );

    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/retry"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 200, "retry rejected: {body}");
    assert_eq!(
        body["work"]["state"], "completed",
        "the retried nested leaf and the container's sibling must finish the run: {body}"
    );
    let entered = stage_ids(data.path(), &work_id, KIND_STAGE_ENTERED);
    assert_eq!(
        entered,
        [
            "00-orient",
            "10-investigate/00-lead",
            "10-investigate/10-code",
            "10-investigate/10-code", // the retry: same leaf, attempt 2
            "20-implement",
        ],
        "retry re-entered exactly the leaf that failed — never an ancestor container, \
         never the container's first child"
    );
    let retried: Vec<u64> = events_of(data.path(), &work_id, KIND_STAGE_ENTERED)
        .iter()
        .filter(|e| e.payload["stage_id"] == "10-investigate/10-code")
        .map(|e| e.payload["attempt"].as_u64().unwrap_or(1))
        .collect();
    assert_eq!(retried, [1, 2], "attempts 1 and 2 of the same nested leaf");

    handle.shutdown().await;
}

/// The three-level fixture the plan panel named as the recon's sharpest
/// risk:
///
/// ```text
/// deep/
///   00-orient/CONTEXT.md
///   10-investigate/workflow.toml
///     00-lead/CONTEXT.md
///     10-deep/workflow.toml
///       00-inner/CONTEXT.md
///   20-implement/CONTEXT.md
/// ```
///
/// Its third leaf, `10-investigate/10-deep/00-inner`, is simultaneously the
/// last leaf of `10-investigate/10-deep` **and** of `10-investigate` — one
/// `StageCompleted` closing two container boundaries at once.
fn write_three_level_workflow(root: &Path) -> PathBuf {
    let package = root.join(".sergeant/workflows/deep");
    write_package(
        &package,
        "deep",
        &["00-orient", "10-investigate", "20-implement"],
    );
    write_stage(&package, "00-orient", "orient");
    write_stage(&package, "20-implement", "implement");
    let investigate = package.join("10-investigate");
    write_package(&investigate, "10-investigate", &["00-lead", "10-deep"]);
    write_stage(&investigate, "00-lead", "lead");
    let deep = investigate.join("10-deep");
    write_package(&deep, "10-deep", &["00-inner"]);
    write_stage(&deep, "00-inner", "the innermost procedure");
    package
}

/// E4's multi-boundary case, end to end: when one leaf closes two
/// containers that both declared an output contract, the boundaries are
/// evaluated **innermost first**, one at a time, and the run walks on only
/// once every one of them is satisfied.
///
/// The sequencing is the assertion. The inner container's unmet contract is
/// what the operator hears about first; only after it is satisfied does the
/// outer one become the thing holding the run. Neither is skipped, and
/// neither is reported at the same instant as the other — a leaf that ends
/// three packages must not produce three simultaneous parks.
///
/// It also pins the re-prompt budget's shape: bounded per leaf *attempt*,
/// not per contract. The one SEND this attempt is allowed goes to the first
/// unmet contract; every later one on the same attempt parks straight away
/// rather than re-prompting again.
#[tokio::test]
async fn a_leaf_that_closes_two_containers_at_once_gates_them_innermost_first() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    let package = write_three_level_workflow(&estate);
    declare_output(&package, "10-investigate", "findings.md");
    declare_output(&package.join("10-investigate"), "10-deep", "inner.md");

    let handle = start_fake(
        data.path(),
        [
            FakeStep::complete(), // 00-orient
            FakeStep::complete(), // 10-investigate/00-lead
            FakeStep::complete(), // 00-inner: closes both containers, neither satisfied
            FakeStep::complete(), // the one bounded re-prompt (the INNER contract)
            FakeStep::complete(), // after the answer: inner satisfied, outer is not
            FakeStep::complete(), // after the second answer: both satisfied
            FakeStep::complete(), // 20-implement
        ],
    )
    .await;
    let client = http();
    let body = submit(&client, &handle, &estate, "deep").await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    let worktree = PathBuf::from(
        body["surface"]["bindings"][0]["worktree_path"]
            .as_str()
            .expect("worktree"),
    );

    // Innermost first: the inner container is the one that re-prompted and
    // then parked. The outer one has said nothing yet.
    assert_eq!(body["work"]["state"], "needs_input", "{body}");
    assert_eq!(body["stage"]["stage_id"], "10-investigate/10-deep/00-inner");
    let reprompts = events_of(data.path(), &work_id, KIND_STAGE_OUTPUT_MISSING);
    assert_eq!(
        reprompts.len(),
        1,
        "one bounded re-prompt, not one per boundary"
    );
    assert_eq!(
        reprompts[0].payload["container_id"], "10-investigate/10-deep",
        "the innermost unmet contract is the one that gets the attempt's one SEND"
    );
    let parked = events_of(data.path(), &work_id, "work.needs_input");
    assert_eq!(parked.len(), 1);
    assert_eq!(parked[0].payload["container_id"], "10-investigate/10-deep");
    assert_eq!(parked[0].payload["path"], "inner.md");

    // Satisfy the inner contract and answer. Same leaf, same attempt — so
    // the budget is spent, and the *outer* container now parks the run
    // immediately rather than re-prompting a second time.
    std::fs::create_dir_all(worktree.join("10-investigate/10-deep/output")).expect("output dir");
    std::fs::write(
        worktree.join("10-investigate/10-deep/output/inner.md"),
        "the inner aggregate\n",
    )
    .expect("write inner.md");
    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/input"),
        json!({"command_id": ulid(), "input": "inner aggregate written"}),
    )
    .await;
    assert_eq!(status, 200, "input rejected: {body}");
    assert_eq!(
        body["work"]["state"], "needs_input",
        "the outer container's contract is still unmet, so the run must still be held: {body}"
    );
    let parked = events_of(data.path(), &work_id, "work.needs_input");
    assert_eq!(parked.len(), 2, "the outer boundary parks in its own turn");
    assert_eq!(
        parked[1].payload["container_id"], "10-investigate",
        "and it is the OUTER container that is named now: {:?}",
        parked[1].payload
    );
    assert_eq!(parked[1].payload["path"], "findings.md");
    assert_eq!(
        events_of(data.path(), &work_id, KIND_STAGE_OUTPUT_MISSING).len(),
        1,
        "still exactly one re-prompt across this attempt, whatever it was for"
    );
    // Neither park ever let the run past the boundary.
    assert!(
        !stage_ids(data.path(), &work_id, KIND_STAGE_ENTERED)
            .iter()
            .any(|id| id == "20-implement"),
        "the run must be held at the leaf that closes both containers"
    );

    // Satisfy the outer contract too: both boundaries close and the run
    // walks on to the container's sibling and finishes.
    std::fs::create_dir_all(worktree.join("10-investigate/output")).expect("output dir");
    std::fs::write(
        worktree.join("10-investigate/output/findings.md"),
        "the outer aggregate\n",
    )
    .expect("write findings.md");
    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/input"),
        json!({"command_id": ulid(), "input": "outer aggregate written"}),
    )
    .await;
    assert_eq!(status, 200, "input rejected: {body}");
    assert_eq!(
        body["work"]["state"], "completed",
        "with both contracts satisfied the run must complete: {body}"
    );
    assert_eq!(
        stage_ids(data.path(), &work_id, KIND_STAGE_ENTERED),
        [
            "00-orient",
            "10-investigate/00-lead",
            "10-investigate/10-deep/00-inner",
            "20-implement",
        ],
        "one entry per leaf: closing two containers never re-enters or duplicates a stage"
    );
    assert_eq!(
        stage_ids(data.path(), &work_id, KIND_STAGE_COMPLETED).len(),
        4,
        "and each leaf completes exactly once"
    );

    handle.shutdown().await;
}
