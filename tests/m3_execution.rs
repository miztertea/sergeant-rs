//! M3 acceptance tests (docs/gauntlet/contracts/M3.md).
//!
//! 1. Zero-config: submit in a temp git repo → a real worktree on a work
//!    branch cut from HEAD, with a complete binding record and the right base
//!    SHA.
//! 2. Multi-repo: `sergeant.toml` with two repositories → one surface, two
//!    worktree bindings.
//! 3. Full run: scripted to complete every stage → `completed`, stage
//!    entry/completion journaled in order, worktree removed, branch retained.
//! 4. needs_input: → `needs_input`; API input resumes the run to completion;
//!    input to a work that is not waiting is a structured error.
//! 5. Failure: → `failed` with the reason recorded; retry re-enters the failed
//!    stage and succeeds on the scripted second attempt.
//! 6. Cancellation mid-stage → `canceled`, teardown recorded, and the fake
//!    still running its hang script — work state is not process state.
//! 7. Routing: the whole §13 precedence chain end to end, and a structured
//!    failure listing the options when nothing resolves.
//! 8. Daemon restart mid-run: in-flight work is re-observed through the
//!    backend contract; unambiguous evidence resumes, ambiguity fails closed
//!    to `blocked` with the evidence recorded.
//!
//! Beyond the numbered list: §25's two other pathologies (a dead process that
//! signalled nothing must not fail the work; a live process that signalled
//! completion must complete the stage), the submit crash window (a daemon
//! that dies part-way through starting a run must not leave a silently
//! pending work), stage/work orthogonality, CONTEXT.md delivery to the
//! backend, fail-closed teardown of a dirty worktree, profiles as launch
//! configuration, and the CLI's `respond`/`retry` through the spawned
//! binary.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::backend::{
    Backend, BackendError, BackendRegistry, BackendSignal, Capabilities, ExecutionHandle,
    NativeState, Observation, ProbeReport, StartRequest,
};
use sergeant_rs::daemon::{self, DaemonConfig, DaemonHandle};
use sergeant_rs::domain::event::{Event, EventDraft, EventSource};
use sergeant_rs::domain::execution::KIND_EXECUTION_RECONCILED;
use sergeant_rs::domain::work::{
    KIND_WORK_BLOCKED, KIND_WORK_COMPLETED, KIND_WORK_FAILED, KIND_WORK_SUBMITTED,
};
use sergeant_rs::domain::workflow::{
    KIND_STAGE_COMPLETED, KIND_STAGE_ENTERED, KIND_STAGE_FAILED, KIND_WORKFLOW_BOUND,
};
use sergeant_rs::runtime::journal::Journal;
use sergeant_rs::runtime::surface::{
    KIND_SURFACE_MATERIALIZED, KIND_SURFACE_MATERIALIZING, KIND_SURFACE_TORN_DOWN,
};

mod support;
use support::DataDir;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

// ---------------------------------------------------------------- helpers

fn ulid() -> String {
    ulid::Ulid::generate().to_string()
}

fn http() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .expect("client")
}

/// Run git in `dir` with a fixed identity, panicking with git's diagnostic.
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
        "git {args:?} failed in {}: {}",
        dir.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// A temp git repository with one commit. Returns its HEAD SHA.
fn init_repo(path: &Path) -> String {
    std::fs::create_dir_all(path).expect("repo dir");
    git(path, &["init", "-b", "main"]);
    std::fs::write(path.join("README.md"), "# fixture\n").expect("write file");
    git(path, &["add", "."]);
    git(path, &["commit", "-m", "initial"]);
    git(path, &["rev-parse", "HEAD"])
}

/// Write a workflow into a repository: `.sergeant/workflows/<name>/…`.
fn write_workflow(root: &Path, name: &str, stages: &[(&str, &str)]) {
    let dir = root.join(".sergeant/workflows").join(name);
    let ids: Vec<String> = stages.iter().map(|(id, _)| format!("{id:?}")).collect();
    std::fs::create_dir_all(&dir).expect("workflow dir");
    std::fs::write(
        dir.join("workflow.toml"),
        format!(
            "[workflow]\nname = \"{name}\"\nversion = \"1\"\nstages = [{}]\n",
            ids.join(", ")
        ),
    )
    .expect("workflow.toml");
    for (id, context) in stages {
        std::fs::create_dir_all(dir.join(id)).expect("stage dir");
        std::fs::write(dir.join(id).join("CONTEXT.md"), context).expect("CONTEXT.md");
    }
}

/// A two-stage workflow, so ordering and progression are observable without
/// the built-in workflow's four stages in the way.
fn write_two_stage_workflow(root: &Path) {
    write_workflow(
        root,
        "tiny",
        &[
            ("00-first", "first stage context"),
            ("10-second", "second stage context"),
        ],
    );
}

/// Start a daemon with a scripted registry.
async fn start_with(
    data_dir: &Path,
    registry: BackendRegistry,
    default: Option<&str>,
) -> DaemonHandle {
    daemon::start_with(
        data_dir,
        DaemonConfig {
            backends: Arc::new(registry),
            default_backend: default.map(str::to_string),
            claude: None,
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start")
}

/// A registry holding one fake under [`FAKE_BACKEND_NAME`].
fn one_fake(script: impl IntoIterator<Item = FakeStep>) -> (BackendRegistry, FakeBackend) {
    let fake = FakeBackend::scripted(FAKE_BACKEND_NAME, script);
    (BackendRegistry::new().with(Arc::new(fake.clone())), fake)
}

/// POST a JSON body to the daemon; returns status and parsed body.
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

/// GET a path from the daemon.
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

/// Submit work whose origin is `cwd`, merging any extra request fields.
async fn submit(
    client: &reqwest::Client,
    handle: &DaemonHandle,
    cwd: &Path,
    intent: &str,
    extra: Value,
) -> (reqwest::StatusCode, Value) {
    let mut body = json!({
        "command_id": ulid(),
        "intent": intent,
        "origin": {"client": "cli", "cwd": cwd},
    });
    if let Some(fields) = extra.as_object() {
        for (key, value) in fields {
            body[key] = value.clone();
        }
    }
    post(client, handle, "/v1/work", body).await
}

fn journal_events(data_dir: &Path) -> Vec<Event> {
    Journal::replay_data_dir(data_dir)
        .expect("replay")
        .map(|e| e.expect("event"))
        .collect()
}

/// Event kinds for one work, in journal order.
fn kinds_for(data_dir: &Path, work_id: &str) -> Vec<String> {
    journal_events(data_dir)
        .into_iter()
        .filter(|e| e.work_id.as_deref() == Some(work_id))
        .map(|e| e.kind)
        .collect()
}

/// Events of one kind for one work.
fn events_of(data_dir: &Path, work_id: &str, kind: &str) -> Vec<Event> {
    journal_events(data_dir)
        .into_iter()
        .filter(|e| e.work_id.as_deref() == Some(work_id) && e.kind == kind)
        .collect()
}

/// Whether a branch exists in a repository.
fn branch_exists(repo: &Path, branch: &str) -> bool {
    !git(repo, &["branch", "--list", branch]).is_empty()
}

/// Name [`OpaqueBackend`] registers under.
const OPAQUE_BACKEND: &str = "opaque";

/// A backend that starts an execution and then cannot usefully say what it is
/// doing. §25 has two shapes of that, and this is an adapter for both.
///
/// The scriptable fake always has a usable answer, which is what makes it a
/// good instrument everywhere else — but "the adapter cannot classify its own
/// context" is a case in its own right, and it needs an adapter that really
/// cannot, not a script reporting that it cannot.
#[derive(Debug, Clone, Copy)]
enum OpaqueBackend {
    /// OBSERVE fails outright: the harness stopped answering.
    ObserveFails,
    /// OBSERVE answers, but with a native state the adapter cannot classify —
    /// while *also* signalling that the stage completed. The signal is the
    /// point: §15 says native evidence and explicit signal are separate, and
    /// §25 says ambiguity fails closed, so unknown liveness must win over a
    /// signal that would otherwise have advanced the run.
    ObserveUnknown,
}

impl OpaqueBackend {
    /// The evidence [`OpaqueBackend::ObserveUnknown`] reports.
    const UNKNOWN_EVIDENCE: &'static str = "the pid is gone but the socket is still open";
}

impl Backend for OpaqueBackend {
    fn name(&self) -> &str {
        OPAQUE_BACKEND
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }

    /// §17: an opaque in-process context per execution, and no service
    /// anyone could start on its behalf.
    fn runtime_scope(&self) -> sergeant_rs::backend::RuntimeScope {
        sergeant_rs::backend::RuntimeScope::PerExecution
    }

    fn probe(&self) -> ProbeReport {
        ProbeReport {
            available: true,
            detail: Some("always available, never informative".to_string()),
        }
    }

    fn start(&self, request: &StartRequest) -> Result<ExecutionHandle, BackendError> {
        Ok(ExecutionHandle {
            execution_id: request.execution_id.clone(),
            native_id: Some(format!("opaque-{}", request.execution_id)),
        })
    }

    fn send(&self, _handle: &ExecutionHandle, _input: &str) -> Result<(), BackendError> {
        Ok(())
    }

    fn observe(&self, _handle: &ExecutionHandle) -> Result<Observation, BackendError> {
        match self {
            Self::ObserveFails => Err(BackendError::Failed {
                backend: OPAQUE_BACKEND.to_string(),
                detail: "the harness stopped answering".to_string(),
            }),
            Self::ObserveUnknown => Ok(Observation {
                native: NativeState::Unknown,
                signal: BackendSignal::StageCompleted {
                    summary: Some("all done".to_string()),
                },
                evidence: Some(Self::UNKNOWN_EVIDENCE.to_string()),
            }),
        }
    }

    fn interrupt(&self, _handle: &ExecutionHandle) -> Result<(), BackendError> {
        Ok(())
    }

    fn resume(
        &self,
        _handle: &ExecutionHandle,
        _request: &sergeant_rs::backend::ResumeRequest,
    ) -> Result<(), BackendError> {
        Ok(())
    }

    /// `Capabilities::default()` advertises no history, so the honest answer
    /// is a refusal — an empty `Ok` from a backend that cannot look is the
    /// shape §15 forbids (see `Backend::history`).
    fn history(
        &self,
        _handle: &ExecutionHandle,
    ) -> Result<Vec<sergeant_rs::backend::NativeEvent>, BackendError> {
        Err(BackendError::Unsupported {
            backend: OPAQUE_BACKEND.to_string(),
            verb: "history".to_string(),
            detail: "this backend records nothing".to_string(),
        })
    }

    fn stop(&self, _handle: &ExecutionHandle) -> Result<(), BackendError> {
        Ok(())
    }
}

// ------------------------------------------------------------------ tests

/// 1. Zero-config discovery materializes a real worktree, on a work branch,
///    cut from the repository's current HEAD, with a complete binding record.
#[tokio::test]
async fn t1_zero_config_submit_materializes_a_real_worktree() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    let head = init_repo(&repo);

    // A hang keeps the run in flight, so the surface can be inspected while
    // it exists rather than after teardown.
    let (registry, _fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();

    let (status, body) = submit(&client, &handle, &repo, "zero config", json!({})).await;
    assert_eq!(status, 201, "submit failed: {body}");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "active");
    assert_eq!(body["work"]["workspace"], "solo");

    // The binding records everything §11 asks for.
    let bindings = body["surface"]["bindings"]
        .as_array()
        .expect("bindings")
        .clone();
    assert_eq!(bindings.len(), 1, "one repository, one binding");
    let binding = &bindings[0];
    assert_eq!(binding["repository"], "solo");
    assert_eq!(binding["base_branch"], "main");
    assert_eq!(binding["base_sha"], head.as_str());
    assert_eq!(binding["head_sha"], head.as_str());
    assert_eq!(binding["work_branch"], format!("sergeant/{work_id}"));
    assert_eq!(
        binding["source_path"].as_str().map(PathBuf::from),
        Some(PathBuf::from(git(&repo, &["rev-parse", "--show-toplevel"])))
    );

    // It is a real worktree, on the work branch, at the base commit — and it
    // lives outside the source checkout (§11).
    let worktree = PathBuf::from(binding["worktree_path"].as_str().expect("worktree path"));
    assert!(worktree.is_dir(), "{} must exist", worktree.display());
    assert!(
        worktree.join("README.md").is_file(),
        "HEAD content checked out"
    );
    assert!(
        !worktree.starts_with(&repo),
        "the surface must not live inside the source checkout: {}",
        worktree.display()
    );
    assert!(
        worktree.starts_with(data.path()),
        "surfaces live under the daemon data dir"
    );
    assert_eq!(
        git(&worktree, &["rev-parse", "--abbrev-ref", "HEAD"]),
        format!("sergeant/{work_id}")
    );
    assert_eq!(git(&worktree, &["rev-parse", "HEAD"]), head);
    // Git itself agrees it is a registered worktree of the source repo.
    assert!(
        git(&repo, &["worktree", "list"]).contains(&worktree.display().to_string()),
        "git must know about the worktree"
    );

    // Creating that branch and that worktree wrote to a repository sergeant
    // does not own, so the journal declares it *first*: the intent to
    // materialize precedes the record of having done it, and names every path
    // and branch a crash in between could leave behind.
    let kinds = kinds_for(data.path(), &work_id);
    let surface_flow: Vec<&String> = kinds.iter().filter(|k| k.starts_with("surface.")).collect();
    assert_eq!(
        surface_flow,
        vec![KIND_SURFACE_MATERIALIZING, KIND_SURFACE_MATERIALIZED],
        "the intent to materialize must be journaled before the result, got {kinds:?}"
    );
    let plan = &events_of(data.path(), &work_id, KIND_SURFACE_MATERIALIZING)[0].payload["plan"];
    assert_eq!(plan["work_branch"], format!("sergeant/{work_id}"));
    assert_eq!(
        plan["repositories"][0]["path"].as_str().map(PathBuf::from),
        Some(PathBuf::from(git(&repo, &["rev-parse", "--show-toplevel"]))),
        "the plan must name the repository a crash could leave a branch in"
    );

    handle.shutdown().await;
}

/// 2. A `sergeant.toml` declaring two repositories produces one surface with
///    two worktree bindings (§9's multi-repo topology, §11's multi-repo
///    surface).
#[tokio::test]
async fn t2_multi_repo_workspace_binds_one_worktree_per_repository() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let api = repos.path().join("payments-api");
    let web = repos.path().join("payments-web");
    let api_head = init_repo(&api);
    let web_head = init_repo(&web);
    std::fs::write(
        api.join("sergeant.toml"),
        r#"
[workspace]
name = "payments"

[[repository]]
name = "api"
path = "."

[[repository]]
name = "web"
path = "../payments-web"
"#,
    )
    .expect("sergeant.toml");

    let (registry, _fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();

    let (status, body) = submit(&client, &handle, &api, "multi repo", json!({})).await;
    assert_eq!(status, 201, "submit failed: {body}");
    assert_eq!(body["work"]["workspace"], "payments");

    let bindings = body["surface"]["bindings"].as_array().expect("bindings");
    assert_eq!(bindings.len(), 2, "one binding per declared repository");
    let names: Vec<&str> = bindings
        .iter()
        .map(|b| b["repository"].as_str().expect("name"))
        .collect();
    assert_eq!(names, ["api", "web"], "declaration order is preserved");
    assert_eq!(bindings[0]["base_sha"], api_head.as_str());
    assert_eq!(bindings[1]["base_sha"], web_head.as_str());

    // Two distinct worktrees under one surface root, each on its own repo.
    let root = PathBuf::from(body["surface"]["root"].as_str().expect("surface root"));
    for binding in bindings {
        let worktree = PathBuf::from(binding["worktree_path"].as_str().expect("path"));
        assert!(
            worktree.starts_with(&root),
            "worktrees share a surface root"
        );
        assert!(worktree.join("README.md").is_file());
        assert_eq!(
            git(&worktree, &["rev-parse", "--abbrev-ref", "HEAD"]),
            binding["work_branch"].as_str().expect("branch")
        );
    }
    // The execution runs at the surface root when several repos are bound.
    let starts = _fake.starts();
    assert_eq!(starts.len(), 1);
    assert_eq!(starts[0].cwd, root);

    handle.shutdown().await;
}

/// 3. A run whose backend completes every stage reaches `completed`, with the
///    stage entry/completion events journaled in order, the worktree removed
///    and the branch retained.
#[tokio::test]
async fn t3_full_run_completes_every_stage_and_retires_the_surface() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    init_repo(&repo);
    write_two_stage_workflow(&repo);

    let (registry, fake) = one_fake([
        FakeStep::complete_with("first done"),
        FakeStep::complete_with("second done"),
    ]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();

    let (status, body) = submit(
        &client,
        &handle,
        &repo,
        "run to completion",
        json!({"workflow": "tiny"}),
    )
    .await;
    assert_eq!(status, 201, "submit failed: {body}");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "completed");

    // Both stages ran, in order, and the CONTEXT.md of each reached the
    // backend verbatim (§12: procedure is data, carried not interpreted).
    let starts = fake.starts();
    assert_eq!(starts.len(), 2, "one execution per stage");
    assert_eq!(starts[0].stage_id, "00-first");
    assert_eq!(starts[0].context, "first stage context");
    assert_eq!(starts[0].intent, "run to completion");
    assert_eq!(starts[1].stage_id, "10-second");
    assert_eq!(starts[1].context, "second stage context");

    // The journal tells the same story, in order.
    let kinds = kinds_for(data.path(), &work_id);
    let stage_flow: Vec<&String> = kinds
        .iter()
        .filter(|k| k.starts_with("stage.") || *k == KIND_WORK_COMPLETED)
        .collect();
    assert_eq!(
        stage_flow,
        vec![
            KIND_STAGE_ENTERED,
            KIND_STAGE_COMPLETED,
            KIND_STAGE_ENTERED,
            KIND_STAGE_COMPLETED,
            KIND_WORK_COMPLETED,
        ],
        "stage entry/completion must be journaled in order, got {kinds:?}"
    );
    let entered = events_of(data.path(), &work_id, KIND_STAGE_ENTERED);
    assert_eq!(entered[0].payload["stage_id"], "00-first");
    assert_eq!(entered[0].payload["index"], 0);
    assert_eq!(entered[1].payload["stage_id"], "10-second");
    assert_eq!(entered[1].payload["index"], 1);
    let completed = events_of(data.path(), &work_id, KIND_STAGE_COMPLETED);
    assert_eq!(completed[0].payload["detail"], "first done");

    // The run pinned its workflow: the definition is in the journal, so a
    // later edit to the files cannot rewrite what executed.
    let bound = events_of(data.path(), &work_id, KIND_WORKFLOW_BOUND);
    assert_eq!(bound.len(), 1);
    assert_eq!(bound[0].payload["workflow"]["name"], "tiny");
    assert_eq!(
        bound[0].payload["workflow"]["stages"][0]["context"],
        "first stage context"
    );

    // Surface retired: worktree removed, branch retained.
    let surface = events_of(data.path(), &work_id, KIND_SURFACE_MATERIALIZED);
    let worktree = PathBuf::from(
        surface[0].payload["surface"]["bindings"][0]["worktree_path"]
            .as_str()
            .expect("worktree path"),
    );
    assert!(
        !worktree.exists(),
        "a clean worktree must be removed at teardown"
    );
    // Including the scaffolding one level up. `git worktree remove` deletes
    // the worktree, not the per-work root that materialize created around it,
    // so for the whole P0 every completed work left an empty
    // `surfaces/<work-id>/` behind — minor once, unbounded over a data dir's
    // life (P1-PERF measured it in all seven scenarios).
    let surface_root = data.path().join("surfaces").join(&work_id);
    assert_eq!(surface_root, worktree.parent().expect("root").to_path_buf());
    assert!(
        !surface_root.exists(),
        "the emptied surface root must go with the worktree: {}",
        surface_root.display()
    );
    assert!(
        data.path().join("surfaces").is_dir(),
        "only the per-work root is removed, not the surfaces directory"
    );
    let branch = format!("sergeant/{work_id}");
    assert!(
        branch_exists(&repo, &branch),
        "teardown keeps the branch; it is the durable output"
    );
    let torn = events_of(data.path(), &work_id, KIND_SURFACE_TORN_DOWN);
    assert_eq!(torn.len(), 1, "teardown is recorded, never silent");
    assert_eq!(torn[0].payload["report"]["clean"], true);
    assert_eq!(
        torn[0].payload["report"]["bindings"][0]["disposition"],
        "removed"
    );

    handle.shutdown().await;
}

/// 4. needs_input parks the work, the input API resumes it to completion, and
///    input to a work that is not waiting is a structured error.
#[tokio::test]
async fn t4_needs_input_parks_the_run_and_input_resumes_it() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    init_repo(&repo);
    write_two_stage_workflow(&repo);

    let (registry, fake) = one_fake([
        FakeStep::needs_input("which database?"),
        // The answer unblocks the turn: the next scripted step is what the
        // execution reports once input has been delivered.
        FakeStep::complete(),
        FakeStep::complete(),
    ]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &repo,
        "ask me something",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "needs_input");
    assert_eq!(body["stage"]["status"], "needs_input");
    assert_eq!(body["stage"]["detail"], "which database?");
    assert_eq!(
        body["stage"]["stage_id"], "00-first",
        "the work is parked, but the stage coordinate is unchanged"
    );

    // The answer reaches the backend and the run continues to completion.
    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/input"),
        json!({"command_id": ulid(), "input": "postgres"}),
    )
    .await;
    assert_eq!(status, 200, "input rejected: {body}");
    assert_eq!(body["work"]["state"], "completed");
    let execution_id = fake.starts()[0].execution_id.clone();
    assert_eq!(fake.inputs(&execution_id), vec!["postgres".to_string()]);

    // Input to a work that is not waiting is refused, with a structured
    // error and no state change.
    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/input"),
        json!({"command_id": ulid(), "input": "nobody asked"}),
    )
    .await;
    assert_eq!(status, 409, "unsolicited input must be refused: {body}");
    assert_eq!(body["error"]["code"], "not_awaiting_input");
    assert!(
        body["error"]["message"]
            .as_str()
            .expect("message")
            .contains("completed"),
        "the error must say what state the work is actually in: {body}"
    );
    let shown = get(&client, &handle, &format!("/v1/work/{work_id}")).await;
    assert_eq!(shown["work"]["state"], "completed");

    // And input to a work that never existed is a 404, not a 409.
    let (status, body) = post(
        &client,
        &handle,
        "/v1/work/01AN4Z07BY79KA1307SR9X4MV3/input",
        json!({"command_id": ulid(), "input": "hello"}),
    )
    .await;
    assert_eq!(status, 404);
    assert_eq!(body["error"]["code"], "work_not_found");

    handle.shutdown().await;
}

/// 5. A failed stage fails the work with the reason recorded, and the retry
///    verb re-enters that same stage — succeeding on the scripted second
///    attempt.
#[tokio::test]
async fn t5_failure_records_the_reason_and_retry_re_enters_the_stage() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    init_repo(&repo);
    write_two_stage_workflow(&repo);

    let (registry, fake) = one_fake([
        FakeStep::fail("the tests do not compile"),
        FakeStep::complete(),
        FakeStep::complete(),
    ]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &repo,
        "fail then retry",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "failed");
    assert_eq!(body["stage"]["status"], "failed");
    assert_eq!(body["stage"]["detail"], "the tests do not compile");

    let failed = events_of(data.path(), &work_id, KIND_WORK_FAILED);
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].payload["reason"], "the tests do not compile");
    assert_eq!(failed[0].payload["stage_id"], "00-first");
    let stage_failed = events_of(data.path(), &work_id, KIND_STAGE_FAILED);
    assert_eq!(
        stage_failed[0].payload["detail"],
        "the tests do not compile"
    );

    // Retry re-enters the *same* stage as a second attempt, and the run then
    // finishes. The surface was retired on failure, so this also proves the
    // retained branch is what makes a retry possible.
    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/retry"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 200, "retry rejected: {body}");
    assert_eq!(body["work"]["state"], "completed");

    let entered = events_of(data.path(), &work_id, KIND_STAGE_ENTERED);
    let attempts: Vec<(String, u64)> = entered
        .iter()
        .map(|e| {
            (
                e.payload["stage_id"].as_str().expect("stage").to_string(),
                e.payload["attempt"].as_u64().expect("attempt"),
            )
        })
        .collect();
    assert_eq!(
        attempts,
        vec![
            ("00-first".to_string(), 1),
            ("00-first".to_string(), 2),
            ("10-second".to_string(), 1),
        ],
        "retry re-enters the failed stage, it does not skip ahead"
    );
    let second_attempt = &fake.starts()[1];
    assert_eq!(second_attempt.stage_id, "00-first");
    assert_eq!(second_attempt.attempt, 2);
    assert_eq!(
        second_attempt.context, "first stage context",
        "the retry executes the same stage's procedure"
    );

    // Retrying a completed work is refused: there is nothing to re-enter.
    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/retry"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 409);
    assert_eq!(body["error"]["code"], "not_retryable");

    handle.shutdown().await;
}

/// 6. Cancellation mid-stage: the work is `canceled`, teardown is recorded,
///    and the backend is *still running* its hang script. This is the §25
///    invariant in its most tempting-to-violate form — a live native process
///    does not keep the work active, and a canceled work does not require the
///    process to be dead.
#[tokio::test]
async fn t6_cancel_mid_stage_leaves_no_zombie_work_state() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    init_repo(&repo);
    write_two_stage_workflow(&repo);

    // `hang` ignores STOP: the native context refuses to die.
    let (registry, fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &repo,
        "cancel me mid stage",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "active", "the stage is running");
    assert_eq!(body["stage"]["stage_id"], "00-first");
    let execution_id = body["execution"]["execution_id"]
        .as_str()
        .expect("execution id")
        .to_string();
    let worktree = PathBuf::from(
        body["surface"]["bindings"][0]["worktree_path"]
            .as_str()
            .expect("worktree"),
    );
    assert!(worktree.is_dir());

    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/cancel"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 200, "cancel failed: {body}");
    assert_eq!(body["work"]["state"], "canceled");
    assert_eq!(body["stage"]["status"], "canceled");

    // The backend was asked to stop — and did not comply. The work is
    // canceled anyway: work state is not process state.
    assert_eq!(fake.stop_requests(), vec![execution_id.clone()]);
    assert!(
        fake.is_live(&execution_id),
        "the hang script must still be running — that is the point"
    );
    assert_eq!(fake.native_state(&execution_id), Some(NativeState::Running));

    // Teardown happened anyway and is recorded; the branch survives.
    let torn = events_of(data.path(), &work_id, KIND_SURFACE_TORN_DOWN);
    assert_eq!(torn.len(), 1, "teardown must be recorded on cancellation");
    assert_eq!(torn[0].payload["report"]["clean"], true);
    assert!(!worktree.exists(), "the worktree was removed");
    assert!(branch_exists(&repo, &format!("sergeant/{work_id}")));

    // No zombie state: nothing further is journaled for this work, and the
    // still-running backend cannot resurrect it. A second cancel is an
    // idempotent no-op, and the terminal state is absorbing.
    let kinds_before = kinds_for(data.path(), &work_id);
    let (status, _) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/cancel"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 200);
    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/retry"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 409, "a canceled work cannot be retried: {body}");
    let kinds_after = kinds_for(data.path(), &work_id);
    assert_eq!(
        kinds_after
            .iter()
            .filter(|k| k.starts_with("work.") || k.starts_with("stage."))
            .count(),
        kinds_before
            .iter()
            .filter(|k| k.starts_with("work.") || k.starts_with("stage."))
            .count(),
        "a terminal work must not gain new state events"
    );

    // Restarting the daemon does not revive it either: it is terminal, so
    // recovery does not even consider it in flight.
    handle.shutdown().await;
    let (registry, _) = one_fake([FakeStep::complete()]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let shown = get(&http(), &handle, &format!("/v1/work/{work_id}")).await;
    assert_eq!(shown["work"]["state"], "canceled");
    handle.shutdown().await;
}

/// 7. The §13 precedence chain, end to end through the API, plus the failure
///    that lists the options.
#[tokio::test]
async fn t7_routing_precedence_and_structured_failure() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let plain = repos.path().join("plain");
    let configured = repos.path().join("configured");
    init_repo(&plain);
    init_repo(&configured);
    std::fs::write(
        configured.join("sergeant.toml"),
        "[workspace]\nname = \"configured\"\ndefault_backend = \"codex\"\n\n[[repository]]\nname = \"configured\"\npath = \".\"\n",
    )
    .expect("sergeant.toml");

    let registry = BackendRegistry::new()
        .with(Arc::new(FakeBackend::scripted(
            "claude",
            [FakeStep::hang()],
        )))
        .with(Arc::new(FakeBackend::scripted("codex", [FakeStep::hang()])))
        .with(Arc::new(FakeBackend::scripted(
            FAKE_BACKEND_NAME,
            [FakeStep::hang()],
        )));
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();

    // Explicit beats origin affinity beats workspace default beats global.
    let cases = [
        (
            &plain,
            json!({"backend": "claude", "origin": {"client": "codex", "cwd": plain}}),
            "claude",
            "explicit",
        ),
        (
            &plain,
            json!({"origin": {"client": "codex", "cwd": plain}}),
            "codex",
            "origin_affinity",
        ),
        // Affinity and workspace default both populated, and different:
        // the adjacent tiers are only ordered by a case that presents both.
        (
            &configured,
            json!({"origin": {"client": "claude", "cwd": configured}}),
            "claude",
            "origin_affinity",
        ),
        (
            &configured,
            json!({"origin": {"client": "cli", "cwd": configured}}),
            "codex",
            "workspace_default",
        ),
        (
            &plain,
            json!({"origin": {"client": "cli", "cwd": plain}}),
            FAKE_BACKEND_NAME,
            "global_default",
        ),
    ];
    for (cwd, extra, expected_backend, expected_source) in cases {
        let (status, body) = submit(&client, &handle, cwd, "route me", extra).await;
        assert_eq!(status, 201, "submit failed: {body}");
        assert_eq!(
            body["backend"], expected_backend,
            "expected {expected_backend} via {expected_source}, got {body}"
        );
        assert_eq!(body["route_source"], expected_source);
    }

    // A tier that names an unknown backend fails with the options, and never
    // silently substitutes the one that is available.
    let (status, body) = submit(
        &client,
        &handle,
        &plain,
        "unknown backend",
        json!({"backend": "opencode"}),
    )
    .await;
    assert_eq!(status, 422);
    assert_eq!(body["error"]["code"], "backend_not_found");
    assert_eq!(
        body["error"]["available_backends"],
        json!(["claude", "codex", FAKE_BACKEND_NAME])
    );
    handle.shutdown().await;

    // Nothing selected and no global default: a structured failure that lists
    // what could have been asked for.
    let data = TempDir::new().expect("tempdir");
    let registry = BackendRegistry::new().with(Arc::new(FakeBackend::new("claude")));
    let handle = start_with(data.path(), registry, None).await;
    let (status, body) = submit(&http(), &handle, &plain, "nothing selected", json!({})).await;
    assert_eq!(status, 422, "unroutable work must be refused: {body}");
    assert_eq!(body["error"]["code"], "no_backend_selected");
    // The scripted fake occupies the "claude" slot, so the daemon adds
    // nothing: Codex is descoped (D6) and never registered, and a backend
    // that is not registered is not offered.
    assert_eq!(body["error"]["available_backends"], json!(["claude"]));
    assert!(
        body["error"]["message"]
            .as_str()
            .expect("message")
            .contains("claude")
    );
    // And nothing was created: an unroutable submission is not half-accepted.
    let list = get(&http(), &handle, "/v1/work").await;
    assert!(
        list["works"].as_array().expect("works").is_empty(),
        "a routing failure must not create work: {list}"
    );
    handle.shutdown().await;
}

/// 8. A daemon restart re-observes in-flight work through the backend
///    contract: a native session that survived resumes from what it now
///    reports; one the adapter cannot recognise fails closed to `blocked`
///    with the evidence recorded (§25).
#[tokio::test]
async fn t8_restart_resumes_unambiguous_work_and_blocks_ambiguous_work() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    init_repo(&repo);
    write_two_stage_workflow(&repo);

    // `durable` models a native session that outlives the daemon: the same
    // backend instance is registered again after the restart. `volatile`
    // models one that did not: a fresh instance with no memory of it.
    let durable = FakeBackend::scripted("durable", [FakeStep::hang()]);
    let volatile = FakeBackend::scripted("volatile", [FakeStep::hang()]);
    let registry = BackendRegistry::new()
        .with(Arc::new(durable.clone()))
        .with(Arc::new(volatile.clone()));
    let handle = start_with(data.path(), registry, Some("durable")).await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &repo,
        "survives the restart",
        json!({"workflow": "tiny", "backend": "durable"}),
    )
    .await;
    let survivor = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "active");

    let (_, body) = submit(
        &client,
        &handle,
        &repo,
        "loses its session",
        json!({"workflow": "tiny", "backend": "volatile"}),
    )
    .await;
    let orphan = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "active");

    handle.shutdown().await;

    // While the daemon was down, the surviving session finished its stage.
    durable.complete_live_executions();

    // A brand-new instance: this backend has never heard of the orphan. It is
    // the one the restarted daemon holds, so it is the one whose START count
    // can say anything about what recovery did.
    let restarted_volatile = FakeBackend::scripted("volatile", []);
    let registry = BackendRegistry::new()
        .with(Arc::new(durable.clone()))
        .with(Arc::new(restarted_volatile.clone()));
    let handle = start_with(data.path(), registry, Some("durable")).await;
    let client = http();

    // Unambiguous: re-observed, resumed, and driven to completion.
    let shown = get(&client, &handle, &format!("/v1/work/{survivor}")).await;
    assert_eq!(
        shown["work"]["state"], "completed",
        "an unambiguously finished session resumes: {shown}"
    );
    let reconciled = events_of(data.path(), &survivor, KIND_EXECUTION_RECONCILED);
    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].payload["disposition"], "resumed");
    assert_eq!(reconciled[0].payload["backend"], "durable");
    // And it resumed from a process that was *dead*: the session finished its
    // stage and exited while the daemon was down, which is the ordinary
    // restart case and §25's sharpest trap — `native process dead ≠ work
    // failed`. Recovery has to read the signal, not the corpse.
    let evidence = reconciled[0].payload["evidence"]
        .as_str()
        .expect("evidence recorded");
    assert_eq!(
        evidence, "native=exited, signal=stage_completed",
        "the resumed work's own evidence must show the process was dead"
    );

    // Ambiguous: blocked, with the adapter's own evidence recorded.
    let shown = get(&client, &handle, &format!("/v1/work/{orphan}")).await;
    assert_eq!(
        shown["work"]["state"], "blocked",
        "an unrecognised execution must fail closed: {shown}"
    );
    let reconciled = events_of(data.path(), &orphan, KIND_EXECUTION_RECONCILED);
    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].payload["disposition"], "ambiguous");
    let evidence = reconciled[0].payload["evidence"]
        .as_str()
        .expect("evidence recorded");
    assert!(
        evidence.contains("does not recognise"),
        "the evidence must be the adapter's own answer, got {evidence:?}"
    );
    let blocked = events_of(data.path(), &orphan, KIND_WORK_BLOCKED);
    assert_eq!(blocked.len(), 1);
    assert!(
        blocked[0].payload["evidence"]
            .as_str()
            .is_some_and(|e| !e.is_empty()),
        "blocking must carry evidence: {}",
        blocked[0].payload
    );

    // The ambiguity is recorded at *stage* level too, not only against the
    // work: a work-level block that leaves the stage reading `active` leaves
    // a stage coordinate nothing will ever move again.
    let stage_blocked = events_of(data.path(), &orphan, "stage.blocked");
    assert_eq!(
        stage_blocked.len(),
        1,
        "the stage the work was parked in must be marked too"
    );
    assert_eq!(stage_blocked[0].payload["stage_id"], "00-first");
    assert_eq!(
        stage_blocked[0].payload["detail"].as_str(),
        Some(evidence),
        "the stage carries the same adapter evidence as the work"
    );

    // Reconciliation asked the surviving session what it was doing exactly
    // once, and the resumed run acted on *that* answer rather than asking a
    // second time — two OBSERVEs are not guaranteed to agree, and the run
    // would then be driven from an answer no decision was made on. The
    // survivor's first execution was observed once while it was running,
    // before the restart, and once by reconciliation: a third would be the
    // duplicate this pins against.
    let first_execution = durable.starts()[0].execution_id.clone();
    let observes = durable
        .observations()
        .iter()
        .filter(|id| **id == first_execution)
        .count();
    assert_eq!(
        observes,
        2,
        "one OBSERVE before the restart, one to reconcile: got {:?}",
        durable.observations()
    );

    // Recovery never invents a second execution for work it could not
    // reconcile (§25: "no new worker is created until prior ownership is
    // reconciled"). Asserted on the *restarted* instance: the pre-restart
    // handle has its own execution table, so its count froze at shutdown and
    // could not fail however recovery behaved.
    assert_eq!(
        volatile.starts().len(),
        1,
        "the first daemon started it once"
    );
    assert!(
        restarted_volatile.starts().is_empty(),
        "the orphan's backend must not be restarted speculatively, got {:?}",
        restarted_volatile.starts()
    );

    handle.shutdown().await;
}

// ------------------------------------------------- beyond the numbered list

/// §25 in both directions, which no single acceptance test covers: a dead
/// native process that signalled nothing must not fail the work, and a live
/// native process that signalled completion must complete the stage.
#[tokio::test]
async fn native_liveness_never_decides_work_state_in_either_direction() {
    let repos = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    init_repo(&repo);
    write_two_stage_workflow(&repo);

    // Dead process, no signal: `native process dead ≠ work failed`.
    let data = TempDir::new().expect("tempdir");
    let (registry, _fake) = one_fake([FakeStep::hang().with_native(NativeState::Exited)]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let (_, body) = submit(
        &http(),
        &handle,
        &repo,
        "exited but silent",
        json!({"workflow": "tiny"}),
    )
    .await;
    assert_eq!(
        body["work"]["state"], "active",
        "a dead process that said nothing must not fail the work: {body}"
    );
    assert_eq!(body["stage"]["status"], "active");
    handle.shutdown().await;

    // Live process, explicit completion: `native process alive ≠ work active`.
    let data = TempDir::new().expect("tempdir");
    let (registry, _fake) = one_fake([FakeStep::complete(), FakeStep::complete()]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let (_, body) = submit(
        &http(),
        &handle,
        &repo,
        "alive but done",
        json!({"workflow": "tiny"}),
    )
    .await;
    assert_eq!(
        body["work"]["state"], "completed",
        "an explicit completion completes the work however alive the process is: {body}"
    );
    handle.shutdown().await;
}

/// Waiting and blocked are §12 verbs too, and both are re-enterable through
/// retry. Stage state stays orthogonal to work state throughout: the stage
/// coordinate never becomes a §10 value and the §10 value never names a
/// stage.
#[tokio::test]
async fn waiting_and_blocked_park_the_work_and_retry_re_enters() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    init_repo(&repo);
    write_two_stage_workflow(&repo);

    let (registry, _fake) = one_fake([
        FakeStep::waiting("CI is still running"),
        FakeStep::blocked("needs an architecture decision"),
        FakeStep::complete(),
        FakeStep::complete(),
    ]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &repo,
        "park me",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "waiting");
    assert_eq!(body["stage"]["status"], "waiting");
    assert_eq!(body["stage"]["detail"], "CI is still running");
    // Orthogonality: the work state is a §10 value, the stage is a coordinate.
    assert_eq!(body["stage"]["stage_id"], "00-first");
    assert_eq!(body["stage"]["index"], 0);
    assert_eq!(body["stage"]["of"], 2);

    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/retry"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 200, "waiting work is re-enterable: {body}");
    assert_eq!(body["work"]["state"], "blocked");
    assert_eq!(body["stage"]["detail"], "needs an architecture decision");

    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/retry"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 200, "blocked work is re-enterable: {body}");
    assert_eq!(body["work"]["state"], "completed");

    handle.shutdown().await;
}

/// Cancelling a work that already failed keeps the failure visible: the
/// cancellation is a fact about the Work, and rewriting the stage's recorded
/// outcome as "canceled" would erase why the run stopped.
#[tokio::test]
async fn cancelling_a_failed_work_does_not_rewrite_the_stage_failure() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    init_repo(&repo);
    write_two_stage_workflow(&repo);

    let (registry, _fake) = one_fake([FakeStep::fail("out of disk")]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();
    let (_, body) = submit(
        &client,
        &handle,
        &repo,
        "fail then cancel",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "failed");

    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/cancel"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(body["work"]["state"], "canceled");
    assert_eq!(
        body["stage"]["status"], "failed",
        "the stage keeps the outcome it actually had: {body}"
    );
    assert_eq!(body["stage"]["detail"], "out of disk");
    // And the run is not retired twice: teardown was already recorded.
    assert_eq!(
        events_of(data.path(), &work_id, KIND_SURFACE_TORN_DOWN).len(),
        1,
        "a run is retired once"
    );

    handle.shutdown().await;
}

/// Teardown fails closed: a worktree with changes in it is retained and
/// recorded, never silently destroyed. Sergeant does not delete work it did
/// not create.
#[tokio::test]
async fn a_dirty_worktree_is_retained_and_recorded_at_teardown() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    init_repo(&repo);
    write_two_stage_workflow(&repo);

    let (registry, _fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();
    let (_, body) = submit(
        &client,
        &handle,
        &repo,
        "leaves a mess",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    let worktree = PathBuf::from(
        body["surface"]["bindings"][0]["worktree_path"]
            .as_str()
            .expect("worktree"),
    );

    // The "execution" leaves uncommitted work behind.
    std::fs::write(worktree.join("half-done.rs"), "fn main() {}\n").expect("dirty the worktree");

    let (status, _) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/cancel"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 200);

    let torn = events_of(data.path(), &work_id, KIND_SURFACE_TORN_DOWN);
    assert_eq!(torn.len(), 1);
    let report = &torn[0].payload["report"];
    assert_eq!(report["clean"], false, "a retained worktree is not clean");
    assert_eq!(report["bindings"][0]["disposition"], "retained_dirty");
    assert!(
        report["bindings"][0]["changes"]
            .as_str()
            .expect("changes recorded")
            .contains("half-done.rs"),
        "the evidence must name what was found: {report}"
    );
    assert!(
        worktree.join("half-done.rs").is_file(),
        "a dirty worktree must survive teardown"
    );

    handle.shutdown().await;
}

/// A multi-repository submission where a later repository cannot be
/// materialized: the earlier ones already have a real branch and worktree in
/// the user's own checkouts. Those are rolled back and the report is
/// journaled, so the failure never leaves git state nobody recorded.
#[tokio::test]
async fn a_repository_that_cannot_be_materialized_rolls_back_the_ones_that_could() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let api = repos.path().join("payments-api");
    init_repo(&api);
    // A repository with no commits at all: a real repository (so the
    // workspace resolves it) with no HEAD to cut a surface from.
    let web = repos.path().join("payments-web");
    std::fs::create_dir_all(&web).expect("repo dir");
    git(&web, &["init", "-b", "main"]);
    std::fs::write(
        api.join("sergeant.toml"),
        "[workspace]\nname = \"payments\"\n\n\
         [[repository]]\nname = \"api\"\npath = \".\"\n\n\
         [[repository]]\nname = \"web\"\npath = \"../payments-web\"\n",
    )
    .expect("sergeant.toml");

    let (registry, fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();

    let (status, body) = submit(&client, &handle, &api, "half a surface", json!({})).await;
    assert_eq!(status, 201, "submit failed: {body}");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(
        body["work"]["state"], "blocked",
        "a surface that cannot be built fails the work closed: {body}"
    );
    assert!(body["surface"].is_null(), "no surface was recorded");
    assert!(
        fake.starts().is_empty(),
        "nothing reaches a backend without a surface"
    );

    // What *was* created is torn down, and the teardown is journaled — this
    // is the only record that a branch and a worktree briefly existed in the
    // user's repository.
    let torn = events_of(data.path(), &work_id, KIND_SURFACE_TORN_DOWN);
    assert_eq!(torn.len(), 1, "the rollback must be recorded, never silent");
    let report = &torn[0].payload["report"];
    assert_eq!(report["bindings"].as_array().expect("bindings").len(), 1);
    assert_eq!(report["bindings"][0]["repository"], "api");
    assert_eq!(report["bindings"][0]["disposition"], "removed");
    let rolled_back = PathBuf::from(
        report["bindings"][0]["worktree_path"]
            .as_str()
            .expect("worktree path"),
    );
    assert!(
        !rolled_back.exists(),
        "the rolled-back worktree must be gone: {}",
        rolled_back.display()
    );

    // The evidence on the block carries the same inventory, so an operator
    // reading the work sees what happened without replaying the journal.
    let blocked = events_of(data.path(), &work_id, KIND_WORK_BLOCKED);
    assert_eq!(blocked.len(), 1);
    assert!(
        blocked[0].payload["reason"]
            .as_str()
            .is_some_and(|r| r.contains("cannot materialize work surface")),
        "the reason must name the failure: {}",
        blocked[0].payload
    );
    assert!(
        blocked[0].payload["evidence"]
            .as_str()
            .is_some_and(|e| e.contains("\"repository\":\"api\"")),
        "the evidence must name what was rolled back: {}",
        blocked[0].payload
    );
    // Teardown retains branches by contract, and the report above names it —
    // recorded, not orphaned. The repository that never got that far has
    // nothing at all.
    assert!(branch_exists(&api, &format!("sergeant/{work_id}")));
    assert!(!branch_exists(&web, &format!("sergeant/{work_id}")));

    handle.shutdown().await;
}

/// Cancelling a work parked in `blocked` retires the stage it was parked in.
/// The stage coordinate is orthogonal to work state (§10), which is exactly
/// why it has to be retired explicitly: a canceled work whose stage still
/// reads `blocked` is a stage nothing will ever move again.
#[tokio::test]
async fn cancelling_a_blocked_work_retires_the_stage_it_was_parked_in() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    init_repo(&repo);
    write_two_stage_workflow(&repo);

    let (registry, _fake) = one_fake([FakeStep::blocked("needs an architecture decision")]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &repo,
        "block then cancel",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "blocked");
    assert_eq!(body["stage"]["status"], "blocked");

    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/cancel"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 200, "cancel failed: {body}");
    assert_eq!(body["work"]["state"], "canceled");
    assert_eq!(
        body["stage"]["status"], "canceled",
        "a parked stage is abandoned with the work, not left parked: {body}"
    );
    let canceled = events_of(data.path(), &work_id, "stage.canceled");
    assert_eq!(canceled.len(), 1);
    assert_eq!(canceled[0].payload["stage_id"], "00-first");
    assert_eq!(canceled[0].payload["detail"], "work canceled");
    // And the surface is retired on the way out, as for any terminal state.
    assert_eq!(
        events_of(data.path(), &work_id, KIND_SURFACE_TORN_DOWN).len(),
        1
    );

    handle.shutdown().await;
}

/// A backend that cannot START the next stage blocks the work with the stage
/// named. The stage-level record matters as much as the work-level one: it
/// says *which* stage could not be entered, which the work state cannot.
#[tokio::test]
async fn a_backend_that_cannot_start_the_next_stage_blocks_with_the_stage_named() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    init_repo(&repo);
    write_two_stage_workflow(&repo);

    let (registry, fake) = one_fake([
        FakeStep::needs_input("which database?"),
        FakeStep::complete(),
    ]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &repo,
        "lose the backend mid run",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "needs_input");

    // The harness goes away while the run is parked. Answering completes the
    // first stage; entering the second cannot start anything.
    fake.set_available(false, "the harness is gone");
    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/input"),
        json!({"command_id": ulid(), "input": "postgres"}),
    )
    .await;
    assert_eq!(status, 200, "input rejected: {body}");
    assert_eq!(
        body["work"]["state"], "blocked",
        "a stage that cannot be started fails closed: {body}"
    );

    let stage_blocked = events_of(data.path(), &work_id, "stage.blocked");
    assert_eq!(stage_blocked.len(), 1);
    assert_eq!(
        stage_blocked[0].payload["stage_id"], "10-second",
        "the record must name the stage that could not be entered"
    );
    assert!(
        stage_blocked[0].payload["detail"]
            .as_str()
            .is_some_and(|d| d.contains("the harness is gone")),
        "the backend's own diagnostic is the evidence: {}",
        stage_blocked[0].payload
    );
    let blocked = events_of(data.path(), &work_id, KIND_WORK_BLOCKED);
    assert_eq!(
        blocked[0].payload["reason"],
        "backend could not start an execution"
    );
    // The first stage still reads as completed: it did complete.
    assert_eq!(
        events_of(data.path(), &work_id, KIND_STAGE_COMPLETED)[0].payload["stage_id"],
        "00-first"
    );

    handle.shutdown().await;
}

/// Input for an execution the backend no longer recognises: the answer cannot
/// be delivered, so the work fails closed with the stage named rather than
/// silently swallowing the input.
#[tokio::test]
async fn input_for_a_forgotten_execution_blocks_with_the_stage_named() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    init_repo(&repo);
    write_two_stage_workflow(&repo);

    let (registry, _fake) = one_fake([FakeStep::needs_input("which database?")]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();
    let (_, body) = submit(
        &client,
        &handle,
        &repo,
        "answer a session that did not survive",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "needs_input");
    handle.shutdown().await;

    // A brand-new backend instance: the native context did not survive the
    // restart. `needs_input` is a decision, not uncertainty, so recovery
    // leaves the work exactly where it is — the failure surfaces when the
    // answer is actually delivered.
    let (registry, restarted) = one_fake([]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();
    let shown = get(&client, &handle, &format!("/v1/work/{work_id}")).await;
    assert_eq!(shown["work"]["state"], "needs_input");

    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/input"),
        json!({"command_id": ulid(), "input": "postgres"}),
    )
    .await;
    assert_eq!(status, 200, "input rejected: {body}");
    assert_eq!(
        body["work"]["state"], "blocked",
        "undeliverable input must fail closed, not vanish: {body}"
    );
    assert!(
        restarted.starts().is_empty(),
        "and nothing is speculatively restarted to receive it"
    );

    let stage_blocked = events_of(data.path(), &work_id, "stage.blocked");
    assert_eq!(stage_blocked.len(), 1);
    assert_eq!(stage_blocked[0].payload["stage_id"], "00-first");
    assert!(
        stage_blocked[0].payload["detail"]
            .as_str()
            .is_some_and(|d| d.contains("does not recognise")),
        "the backend's own diagnostic is the evidence: {}",
        stage_blocked[0].payload
    );
    let blocked = events_of(data.path(), &work_id, KIND_WORK_BLOCKED);
    assert!(
        blocked[0].payload["reason"]
            .as_str()
            .is_some_and(|r| r.contains("cannot deliver input")),
        "got {}",
        blocked[0].payload
    );

    handle.shutdown().await;
}

/// A backend that starts an execution and then cannot say anything about it.
/// §25: an adapter that cannot classify its own context is ambiguity, and
/// ambiguity fails closed with the evidence — never a guess in either
/// direction.
#[tokio::test]
async fn a_backend_that_cannot_observe_its_execution_fails_the_work_closed() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    init_repo(&repo);
    write_two_stage_workflow(&repo);

    let registry = BackendRegistry::new().with(Arc::new(OpaqueBackend::ObserveFails));
    let handle = start_with(data.path(), registry, Some(OPAQUE_BACKEND)).await;
    let client = http();

    let (status, body) = submit(
        &client,
        &handle,
        &repo,
        "observe me if you can",
        json!({"workflow": "tiny"}),
    )
    .await;
    assert_eq!(status, 201, "submit failed: {body}");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(
        body["work"]["state"], "blocked",
        "an unobservable execution must not be guessed at: {body}"
    );

    let stage_blocked = events_of(data.path(), &work_id, "stage.blocked");
    assert_eq!(stage_blocked.len(), 1);
    assert_eq!(stage_blocked[0].payload["stage_id"], "00-first");
    assert!(
        stage_blocked[0].payload["detail"]
            .as_str()
            .is_some_and(|d| d.contains("stopped answering")),
        "got {}",
        stage_blocked[0].payload
    );
    let blocked = events_of(data.path(), &work_id, KIND_WORK_BLOCKED);
    assert_eq!(
        blocked[0].payload["reason"],
        "backend could not observe the execution"
    );
    assert!(
        blocked[0].payload["evidence"]
            .as_str()
            .is_some_and(|e| e.contains("stopped answering")),
        "got {}",
        blocked[0].payload
    );

    handle.shutdown().await;
}

/// An adapter that *can* answer OBSERVE but reports a native state it cannot
/// classify. §25 makes that ambiguity, and ambiguity fails closed — even when
/// the very same answer also carries an explicit "stage completed" signal.
/// Native liveness gets exactly one say in the engine, and this is it: it can
/// stop a run, and it can never advance one. A completion the engine cannot
/// trust the context of is not a completion.
#[tokio::test]
async fn an_unknown_native_state_blocks_even_when_the_signal_says_completed() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    init_repo(&repo);
    write_two_stage_workflow(&repo);

    let registry = BackendRegistry::new().with(Arc::new(OpaqueBackend::ObserveUnknown));
    let handle = start_with(data.path(), registry, Some(OPAQUE_BACKEND)).await;
    let client = http();

    let (status, body) = submit(
        &client,
        &handle,
        &repo,
        "complete me from an unknown context",
        json!({"workflow": "tiny"}),
    )
    .await;
    assert_eq!(status, 201, "submit failed: {body}");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(
        body["work"]["state"], "blocked",
        "an unknown native state must beat the completion signal beside it: {body}"
    );
    assert_eq!(body["stage"]["status"], "blocked");

    // Nothing advanced: the stage was never completed and the second stage
    // was never entered.
    assert!(
        events_of(data.path(), &work_id, KIND_STAGE_COMPLETED).is_empty(),
        "a completion signal from an unclassifiable context must not complete a stage"
    );
    assert_eq!(
        events_of(data.path(), &work_id, KIND_STAGE_ENTERED).len(),
        1,
        "and must not enter the next stage"
    );

    let stage_blocked = events_of(data.path(), &work_id, "stage.blocked");
    assert_eq!(stage_blocked.len(), 1);
    assert_eq!(stage_blocked[0].payload["stage_id"], "00-first");
    assert!(
        stage_blocked[0].payload["detail"]
            .as_str()
            .is_some_and(|d| d.contains("socket is still open")),
        "the adapter's own evidence is what is recorded: {}",
        stage_blocked[0].payload
    );
    let blocked = events_of(data.path(), &work_id, KIND_WORK_BLOCKED);
    assert_eq!(
        blocked[0].payload["reason"],
        "backend reports an unknown native state"
    );
    assert!(
        blocked[0].payload["evidence"]
            .as_str()
            .is_some_and(|e| e.contains("socket is still open")),
        "got {}",
        blocked[0].payload
    );

    handle.shutdown().await;
}

/// The workflow name is client input (`POST /v1/work`'s `workflow` field) and
/// is joined straight onto `.sergeant/workflows/`. A name that walks out of
/// that directory is refused before anything runs — not resolved against
/// whatever `workflow.toml` happens to sit there.
#[tokio::test]
async fn a_workflow_name_that_escapes_the_workflows_directory_is_refused_at_submit() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    init_repo(&repo);
    // A perfectly loadable workflow that simply is not in the workflows
    // directory, so only the guard can be what refuses the submission.
    write_workflow(
        &repo.join("elsewhere"),
        "outside",
        &[("00-only", "context")],
    );
    std::fs::create_dir_all(repo.join(".sergeant/workflows")).expect("workflows root");

    let (registry, fake) = one_fake([]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();

    let (status, body) = submit(
        &client,
        &handle,
        &repo,
        "read someone else's workflow",
        json!({"workflow": "../../elsewhere/.sergeant/workflows/outside"}),
    )
    .await;
    assert_eq!(
        status, 422,
        "a traversing workflow name must be refused: {body}"
    );
    assert_eq!(body["error"]["code"], "workflow_error");
    assert!(
        body["error"]["message"]
            .as_str()
            .expect("message")
            .contains("not a plain directory name"),
        "the diagnostic must say why: {body}"
    );
    // Refused before anything was created.
    let list = get(&client, &handle, "/v1/work").await;
    assert!(list["works"].as_array().expect("works").is_empty());
    assert!(fake.starts().is_empty());

    handle.shutdown().await;
}

/// §14: a profile is *launch configuration*, pinned at bind time and handed
/// to the backend at START. The boundary "no credentials, ever" is structural
/// — the record has no field to put one in — so what is tested here is that
/// the launch fields a profile does carry actually reach the actor, and that
/// naming a profile that does not exist is a structured error rather than a
/// silent run without it.
#[tokio::test]
async fn a_profile_is_launch_configuration_carried_to_the_backend() {
    let repos = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    init_repo(&repo);

    let data = TempDir::new().expect("tempdir");
    std::fs::write(
        repo.join("sergeant.toml"),
        format!(
            "[workspace]\nname = \"solo\"\n\n[[repository]]\nname = \"solo\"\npath = \".\"\n\n\
             [[profile]]\nname = \"enterprise\"\nbackend = \"{FAKE_BACKEND_NAME}\"\n\
             default_model = \"claude-opus-4-7\"\n\
             env = {{ CLAUDE_CONFIG_DIR = \"/tmp/work\", GIT_AUTHOR_NAME = \"sergeant\" }}\n"
        ),
    )
    .expect("sergeant.toml");
    let (registry, fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();
    let (status, body) = submit(
        &client,
        &handle,
        &repo,
        "with a profile",
        json!({"profile": "enterprise"}),
    )
    .await;
    assert_eq!(status, 201, "profile rejected: {body}");
    let start = &fake.starts()[0];
    assert_eq!(start.model.as_deref(), Some("claude-opus-4-7"));
    let profile = start.profile.as_ref().expect("profile delivered");
    assert_eq!(profile.name, "enterprise");
    assert_eq!(profile.env["CLAUDE_CONFIG_DIR"], "/tmp/work");
    assert_eq!(
        profile.env["GIT_AUTHOR_NAME"], "sergeant",
        "commit identity for the work branch is launch configuration, not a \
         credential, and must not be refused"
    );

    // An unknown profile is a structured error naming what exists.
    let (status, body) = submit(
        &client,
        &handle,
        &repo,
        "unknown profile",
        json!({"profile": "nope"}),
    )
    .await;
    assert_eq!(status, 422);
    assert_eq!(body["error"]["code"], "profile_not_found");
    handle.shutdown().await;
}

/// A submission with no repository context is a *captured intent*: there is
/// no workspace to materialize, which §9 answers definitely rather than
/// failing. But "nothing to materialize" is not "nothing to honour" — a
/// submission that names a backend sergeant cannot route to is refused with
/// §13's options, instead of being recorded as pending work carrying a
/// selection nothing will ever run.
#[tokio::test]
async fn a_submission_with_no_workspace_is_captured_but_still_routed() {
    let data = TempDir::new().expect("tempdir");
    let (registry, fake) = one_fake([]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();

    // No origin at all: accepted, pending, no surface.
    let (status, body) = post(
        &client,
        &handle,
        "/v1/work",
        json!({"command_id": ulid(), "intent": "no repository context"}),
    )
    .await;
    assert_eq!(status, 201);
    assert_eq!(body["work"]["state"], "pending");
    assert!(body["surface"].is_null());

    // A cwd that is not a repository is the same answer, not an error.
    let elsewhere = TempDir::new().expect("tempdir");
    let (status, body) = submit(&client, &handle, elsewhere.path(), "not a repo", json!({})).await;
    assert_eq!(status, 201);
    assert_eq!(body["work"]["state"], "pending");
    assert!(body["surface"].is_null());

    // But an explicit backend that cannot be honoured is §13's terminal
    // state — "fail with available options" — surface or no surface.
    let (status, body) = submit(
        &client,
        &handle,
        elsewhere.path(),
        "route me nowhere",
        json!({"backend": "opencode-does-not-exist"}),
    )
    .await;
    assert_eq!(
        status, 422,
        "an unhonourable selection must not be recorded as pending work: {body}"
    );
    assert_eq!(body["error"]["code"], "backend_not_found");
    // Since M4 the daemon registers the real claude adapter alongside the
    // scripted fake. Codex is descoped (D6): not registered, not offered.
    assert_eq!(
        body["error"]["available_backends"],
        json!(["claude", FAKE_BACKEND_NAME])
    );

    // Only two works exist: the refusal created none.
    let list = get(&client, &handle, "/v1/work").await;
    assert_eq!(list["works"].as_array().expect("works").len(), 2);
    assert!(
        fake.starts().is_empty(),
        "nothing without a surface may reach a backend"
    );

    handle.shutdown().await;
}

/// The submit crash window (§25's fail-closed rule applied to sergeant's own
/// bookkeeping). Submitting is several fsynced appends, and a daemon that
/// dies inside that sequence leaves a `pending` work no verb can move:
/// `retry` refuses `pending`, restart reconciliation looks only at `active`,
/// and a client retrying the same `command_id` replays the accepted outcome
/// without re-planning. Worse, the surviving prefix can name a worktree and a
/// work branch already created in the user's own repository.
///
/// The journal here is hand-built to be exactly what such a crash leaves
/// behind, because that is the only way to produce it deterministically.
#[tokio::test]
async fn a_crash_inside_the_submit_window_fails_closed_with_the_git_evidence() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    init_repo(&repo);

    let crashed = ulid();
    let untouched = ulid();
    {
        let mut journal = Journal::open(data.path()).expect("journal");
        for id in [&crashed, &untouched] {
            journal
                .append(
                    EventDraft::new(
                        EventSource::new("api", "test"),
                        KIND_WORK_SUBMITTED,
                        json!({"work": {
                            "id": id,
                            "intent": "submitted just before the crash",
                            "state": "pending",
                            "created_by": "test",
                            "created_at": "2026-01-01T00:00:00Z",
                        }}),
                    )
                    .with_work_id(id),
                )
                .expect("append work.submitted");
        }
        // ...and for one of them, the engine got as far as declaring what it
        // was about to create in the user's repository, then died.
        journal
            .append(
                EventDraft::new(
                    EventSource::new("daemon", "engine"),
                    KIND_SURFACE_MATERIALIZING,
                    json!({"plan": {
                        "root": data.path().join("surfaces").join(&crashed),
                        "work_branch": format!("sergeant/{crashed}"),
                        "repositories": [{"name": "solo", "path": repo}],
                    }}),
                )
                .with_work_id(&crashed),
            )
            .expect("append surface.materializing");
    }

    let (registry, fake) = one_fake([]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let client = http();

    let shown = get(&client, &handle, &format!("/v1/work/{crashed}")).await;
    assert_eq!(
        shown["work"]["state"], "blocked",
        "a half-started work must not stay silently pending: {shown}"
    );
    let blocked = events_of(data.path(), &crashed, KIND_WORK_BLOCKED);
    assert_eq!(blocked.len(), 1, "blocking is journaled once");
    let evidence = blocked[0].payload["evidence"]
        .as_str()
        .expect("evidence recorded");
    assert!(
        evidence.contains(&format!("sergeant/{crashed}"))
            && evidence.contains(&repo.display().to_string()),
        "the evidence must name the git state the crash may have left: {evidence}"
    );

    // Nothing was restarted or re-planned on its behalf (§25: no new worker
    // until prior ownership is reconciled).
    assert!(fake.starts().is_empty());

    // And a `pending` work with no run record is left exactly where it is: an
    // intent the daemon never began is not uncertainty to fail closed on.
    let shown = get(&client, &handle, &format!("/v1/work/{untouched}")).await;
    assert_eq!(shown["work"]["state"], "pending");
    assert!(events_of(data.path(), &untouched, KIND_WORK_BLOCKED).is_empty());

    handle.shutdown().await;
}

/// A malformed `sergeant.toml` is refused with the line named, rather than
/// half-interpreted: checked-in configuration is an instruction, and a typo
/// that silently means nothing is worse than a refusal.
#[tokio::test]
async fn a_malformed_workspace_file_fails_closed() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    init_repo(&repo);
    std::fs::write(
        repo.join("sergeant.toml"),
        "[workspace]\nname = \"solo\"\ndefault_backends = \"fake\"\n\n[[repository]]\nname = \"solo\"\npath = \".\"\n",
    )
    .expect("sergeant.toml");

    let (registry, _fake) = one_fake([]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME)).await;
    let (status, body) = submit(&http(), &handle, &repo, "typo in config", json!({})).await;
    assert_eq!(status, 422, "a typo'd key must not be ignored: {body}");
    assert_eq!(body["error"]["code"], "workspace_error");
    assert!(
        body["error"]["message"]
            .as_str()
            .expect("message")
            .contains("default_backends"),
        "the diagnostic must name the unknown key: {body}"
    );

    // A declared repository that does not exist is refused too.
    std::fs::write(
        repo.join("sergeant.toml"),
        "[workspace]\nname = \"solo\"\n\n[[repository]]\nname = \"ghost\"\npath = \"../nowhere\"\n",
    )
    .expect("sergeant.toml");
    let (status, body) = submit(&http(), &handle, &repo, "missing repo", json!({})).await;
    assert_eq!(status, 422);
    assert!(
        body["error"]["message"]
            .as_str()
            .expect("message")
            .contains("ghost")
    );

    // Two names for one checkout is refused too, and — this is the point —
    // refused *before* anything is materialized. Both entries would be cut
    // onto the same `sergeant/<work-id>` branch of the same repository, and
    // the failure would otherwise land on the second `git worktree add`,
    // after the first had already put a branch in the user's checkout.
    std::fs::write(
        repo.join("sergeant.toml"),
        "[workspace]\nname = \"solo\"\n\n\
         [[repository]]\nname = \"here\"\npath = \".\"\n\n\
         [[repository]]\nname = \"also-here\"\npath = \"./\"\n",
    )
    .expect("sergeant.toml");
    let (status, body) = submit(&http(), &handle, &repo, "one repo, two names", json!({})).await;
    assert_eq!(status, 422, "a same-path duplicate must be refused: {body}");
    assert_eq!(body["error"]["code"], "workspace_error");
    let message = body["error"]["message"].as_str().expect("message");
    assert!(
        message.contains("here") && message.contains("also-here"),
        "the diagnostic must name both entries: {body}"
    );
    // Nothing was created behind the refusal: no surface, no branch.
    assert_eq!(
        git(&repo, &["branch", "--list", "sergeant/*"]),
        "",
        "a statically rejectable submission must not touch the repository"
    );

    handle.shutdown().await;
}

/// The CLI surface the contract adds, through the spawned binary: a run that
/// asks for input, `sgt respond`, `sgt work show` carrying stage and surface,
/// and `sgt retry`.
#[test]
fn cli_respond_and_retry_through_the_binary() {
    let repos = TempDir::new().expect("tempdir");
    let data = DataDir::new();
    let repo = repos.path().join("solo");
    init_repo(&repo);
    write_two_stage_workflow(&repo);

    // The default daemon registry's fake completes every stage, so this run
    // goes end to end without a scripted backend.
    let output = sgt(&repo, &data, &["--json", "run", "ship it"]);
    assert!(
        output.status.success(),
        "sgt run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let submitted: Value = serde_json::from_slice(&output.stdout).expect("run --json");
    let work_id = submitted["work"]["id"]
        .as_str()
        .expect("work id")
        .to_string();
    assert_eq!(submitted["work"]["state"], "completed");
    assert_eq!(submitted["work"]["origin_client"], "cli");
    assert_eq!(submitted["route_source"], "global_default");

    // `sgt work show` (human form) carries the stage and surface coordinates.
    let output = sgt(&repo, &data, &["work", "show", &work_id]);
    assert!(output.status.success());
    let shown: Value = serde_json::from_slice(&output.stdout).expect("work show");
    assert_eq!(shown["id"].as_str(), Some(work_id.as_str()));
    assert_eq!(shown["state"], "completed");
    assert_eq!(shown["stage"]["stage_id"], "30-close");
    assert_eq!(shown["workflow"]["name"], "software-change");
    assert_eq!(shown["workflow"]["source"], "embedded");
    assert!(shown["surface"]["bindings"][0]["work_branch"].is_string());

    // Responding to work that is not waiting exits nonzero with the daemon's
    // own diagnostic (the CLI never invents success).
    let output = sgt(&repo, &data, &["respond", &work_id, "hello"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not needs_input"), "respond said: {stderr}");

    // Same for retry on a completed work.
    let output = sgt(&repo, &data, &["retry", &work_id]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("only failed, blocked or waiting"),
        "retry said: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    stop_daemon(data.path());
}

/// Run the sgt binary from `cwd` against `data_dir`.
///
/// `data_dir` is a [`DataDir`] rather than a path because any client command
/// may auto-spawn a detached daemon; the guard is what reaps it, including on
/// the path where an assertion above the cleanup fails.
fn sgt(cwd: &Path, data_dir: &DataDir, args: &[&str]) -> std::process::Output {
    Command::new(SGT)
        .current_dir(cwd)
        .arg("--data-dir")
        .arg(data_dir.path())
        .args(args)
        .output()
        .expect("run sgt")
}

/// Stop the daemon owning a data dir and wait for its descriptor to go.
fn stop_daemon(data_dir: &Path) {
    if let Ok(Some(descriptor)) = daemon::read_descriptor(data_dir) {
        let _ = Command::new("kill")
            .arg(descriptor.pid.to_string())
            .status();
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while daemon::descriptor_path(data_dir).exists() && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}
