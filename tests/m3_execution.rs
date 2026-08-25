//! M3 acceptance tests (sergeant-rs-workspace's knowledge/evidence/gauntlet/contracts/M3.md).
//!
//! 1. Single-repository estate: submit against the estate the daemon is bound
//!    to → a real worktree on a work branch cut from the mount's HEAD, with a
//!    complete binding record and the right base SHA.
//! 2. Multi-repo: an estate manifest declaring two repositories → one surface,
//!    two worktree bindings.
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
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::backend::{
    Backend, BackendError, BackendRegistry, BackendSignal, Capabilities, ExecutionHandle,
    NativeEvent, NativeState, Observation, ProbeReport, StartRequest,
};
use sergeant_rs::daemon::{self, DaemonConfig, DaemonHandle};
use sergeant_rs::domain::estate::InstructionPolicy;
use sergeant_rs::domain::event::{Event, EventDraft, EventSource};
use sergeant_rs::domain::execution::KIND_EXECUTION_RECONCILED;
use sergeant_rs::domain::work::{
    KIND_WORK_BLOCKED, KIND_WORK_COMPLETED, KIND_WORK_FAILED, KIND_WORK_SUBMITTED,
};
use sergeant_rs::domain::workflow::{
    KIND_STAGE_BLOCKED, KIND_STAGE_COMPLETED, KIND_STAGE_ENTERED, KIND_STAGE_FAILED,
    KIND_STAGE_INPUT_RECEIVED, KIND_STAGE_OUTPUT_MISSING, KIND_WORKFLOW_BOUND,
    REASON_STAGE_OUTPUT_MISSING, WorkflowDefinition,
};
use sergeant_rs::runtime::engine::{Engine, KIND_TURN_CEILING_INTERRUPTED, SubmitContext};
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

/// Scaffold an estate at `root` from a verbatim `sergeant.toml`, creating the
/// derived mount `<root>/repos/<name>` for every entry of `repos`. Returns
/// each mount's HEAD SHA, in `repos` order.
///
/// [`support::scaffold_estate`] covers the plain shape; this exists for the
/// manifests these tests need extra keys in (`default_backend`,
/// `surfaces_dir`, per-repo `instructions`, `[[profile]]`, `[group.*]`).
/// estate-root §6.1: no entry may carry a `path` — a mount is derived from
/// the estate root and the repository's own name, never declared.
fn estate_with_manifest(root: &Path, manifest: &str, repos: &[&str]) -> Vec<String> {
    std::fs::create_dir_all(root).expect("estate root");
    let heads = repos
        .iter()
        .map(|name| init_repo(&root.join("repos").join(name)))
        .collect();
    std::fs::write(root.join("sergeant.toml"), manifest).expect("sergeant.toml");
    heads
}

/// Write a workflow into an **estate root**: `.sergeant/workflows/<name>/…`.
///
/// estate-root §4.1/§5.2: workflows resolve against `estate.root` — the
/// directory the manifest lives in — so a fixture written inside a mount is
/// never found. Every caller here passes the estate root, not a checkout.
fn write_workflow(root: &Path, name: &str, stages: &[(&str, &str)]) {
    write_workflow_with_tables(root, name, stages, "")
}

/// Write a workflow, with optional raw `[stage."<id>"]` tables (N3 §12.2)
/// appended verbatim after the `[workflow]` section.
fn write_workflow_with_tables(root: &Path, name: &str, stages: &[(&str, &str)], tables: &str) {
    let dir = root.join(".sergeant/workflows").join(name);
    let ids: Vec<String> = stages.iter().map(|(id, _)| format!("{id:?}")).collect();
    std::fs::create_dir_all(&dir).expect("workflow dir");
    std::fs::write(
        dir.join("workflow.toml"),
        format!(
            "[workflow]\nname = \"{name}\"\nversion = \"1\"\nstages = [{}]\n{tables}",
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

/// [`write_two_stage_workflow`], but `00-first` declares an `output/
/// README.md` with both an expected artifact and Amendment 10d's
/// `**Required columns:**` line — the shape `remediate-findings/00-ingest`
/// itself carries — so the gate's structural check has something to
/// enforce for a stage that ships an artifact that exists but is not a
/// typed table.
fn write_two_stage_workflow_with_required_columns(
    root: &Path,
    expected_artifact: &str,
    columns: &[&str],
) {
    write_two_stage_workflow(root);
    let output_dir = root
        .join(".sergeant/workflows/tiny/00-first")
        .join("output");
    std::fs::create_dir_all(&output_dir).expect("output dir");
    let quoted: Vec<String> = columns.iter().map(|c| format!("`{c}`")).collect();
    std::fs::write(
        output_dir.join("README.md"),
        format!(
            "# Output — `00-first`\n\n**Expected artifact:** `{expected_artifact}` — proof of \
             work.\n\n**Required columns:** {} — a typed set, not prose.\n\n**Disposition:** \
             `evidence`\n",
            quoted.join(", ")
        ),
    )
    .expect("output/README.md");
}

/// [`write_two_stage_workflow`], but `00-first` declares an `output/
/// README.md` (#260 Q3's shape, `docs/icm/convention.md` Rule 4) naming
/// `expected_artifact` as its expected artifact — so the hard stage-output
/// gate has something to enforce against a `StageCompleted` signal for that
/// stage.
fn write_two_stage_workflow_with_declared_output(root: &Path, expected_artifact: &str) {
    write_two_stage_workflow(root);
    let output_dir = root
        .join(".sergeant/workflows/tiny/00-first")
        .join("output");
    std::fs::create_dir_all(&output_dir).expect("output dir");
    std::fs::write(
        output_dir.join("README.md"),
        format!(
            "# Output — `00-first`\n\n**Expected artifact:** `{expected_artifact}` — proof of \
             work.\n\n**Disposition:** `evidence`\n"
        ),
    )
    .expect("output/README.md");
}

/// Start a daemon bound to `estate_root`, with a scripted registry.
///
/// estate-root §5.1/§5.2: the estate is a property of the *daemon*, fixed at
/// start and admitted before the data dir is even created. `Engine::plan`
/// reads the manifest at this root and nowhere else; a submission's
/// `origin.cwd` is recorded evidence only. `None` starts a daemon bound to no
/// estate at all — it plans against nothing, so everything it accepts stays
/// `pending`.
async fn start_with(
    data_dir: &Path,
    registry: BackendRegistry,
    default: Option<&str>,
    estate_root: Option<&Path>,
) -> DaemonHandle {
    daemon::start_with(
        data_dir,
        DaemonConfig {
            backends: Arc::new(registry),
            default_backend: default.map(str::to_string),
            claude: None,
            estate_root: estate_root.map(Path::to_path_buf),
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

/// R-MVP1-7's own tight interrupt budget (`tests/m4_backends.rs`'s
/// `interrupt_budget`, same precedent), duplicated here rather than shared:
/// ceiling plus ten driver intervals of jitter on the measured dev
/// container, widened with a loud, named allowance on hosted CI's
/// two-environments split — the *functional* pin (the ceiling fires at all)
/// holds everywhere; the *latency* pin holds only where it was measured.
fn interrupt_budget(ceiling: Duration, poll: Duration) -> Duration {
    let tight = ceiling + poll * 10;
    if std::env::var_os("CI").is_some() {
        eprintln!(
            "WIDENED-ENV: hosted CI runner — interrupt budget {tight:?} + 5s scheduler-jitter \
             allowance (the tight ceiling-latency pin is asserted on the measured dev container)"
        );
        tight + Duration::from_secs(5)
    } else {
        tight
    }
}

/// Make each of `paths` writable or not, for tests that need a git write to
/// fail while every git *read* still succeeds.
fn set_writable(paths: &[PathBuf], writable: bool) {
    for path in paths {
        let mut permissions = std::fs::metadata(path).expect("stat").permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            permissions.set_mode(if writable { 0o755 } else { 0o555 });
        }
        #[cfg(not(unix))]
        permissions.set_readonly(!writable);
        std::fs::set_permissions(path, permissions).expect("chmod");
    }
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

    fn prepare(
        &self,
        request: &StartRequest,
    ) -> Result<sergeant_rs::backend::PreparedExecution, BackendError> {
        Ok(sergeant_rs::backend::PreparedExecution {
            execution_id: request.execution_id.clone(),
            native_id: Some(format!("opaque-{}", request.execution_id)),
            request: request.clone(),
        })
    }

    fn launch(
        &self,
        prepared: &sergeant_rs::backend::PreparedExecution,
    ) -> Result<ExecutionHandle, BackendError> {
        Ok(prepared.handle())
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

    fn interrupt(
        &self,
        _handle: &ExecutionHandle,
    ) -> Result<sergeant_rs::backend::Completion, BackendError> {
        Ok(sergeant_rs::backend::Completion::immediate())
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

    fn stop(
        &self,
        _handle: &ExecutionHandle,
    ) -> Result<sergeant_rs::backend::Completion, BackendError> {
        Ok(sergeant_rs::backend::Completion::immediate())
    }
}

// ------------------------------------------------------------------ tests

/// 1. A submission against the estate the daemon is bound to materializes a
///    real worktree, on a work branch, cut from the mount's current HEAD, with
///    a complete binding record.
///
/// This pinned "zero-config discovery" until estate-root §4.1 deleted it:
/// there is no ancestor walk and no git-toplevel fallback, so the estate is
/// the one the daemon was *started* against and the repository is its derived
/// mount `<estate-root>/repos/solo` (§6.1). What the test measures is
/// unchanged — everything below the first four lines is about the surface,
/// not about how the estate was found.
#[tokio::test]
async fn t1_a_bound_estate_submit_materializes_a_real_worktree() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    let (repo, head) = support::scaffold_solo_estate(&estate, "solo");

    // A hang keeps the run in flight, so the surface can be inspected while
    // it exists rather than after teardown.
    let (registry, _fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (status, body) = submit(&client, &handle, &estate, "one bound estate", json!({})).await;
    assert_eq!(status, 201, "submit failed: {body}");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "active");
    // estate-root §7.1/§7.4: no scope was submitted at all, and a
    // one-repository estate infers its sole repository — `Work.estate`
    // is never written for a new Work (`workflow.bound`'s own `estate`
    // field, journaled separately, is its replacement — see
    // `r_mvp1_4_workflow_bound_carries_repositories_and_instruction_identities`
    // below for that assertion).
    assert_eq!(body["work"]["workspace"], Value::Null);
    assert_eq!(body["work"]["repositories"], json!(["solo"]));

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
    // estate-root §6.1: one estate root, two *derived* mounts. The old shape
    // — the manifest living inside `api` with `path = "."`, and `web` a
    // sibling reached by `path = "../payments-web"` — is refused by name now;
    // a repository is `<estate-root>/repos/<name>` and nothing else.
    let estate = repos.path().join("payments");
    let heads = support::scaffold_estate(&estate, "payments", &["api", "web"]);
    let (api_head, web_head) = (heads[0].clone(), heads[1].clone());

    let (registry, _fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    // estate-root §7.1: "payments" declares two repositories, so this test's
    // whole point (one surface, two bindings) needs an explicit scope now —
    // `--all` is the least-assuming choice that still selects both.
    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "multi repo",
        json!({"scope": {"all": true}}),
    )
    .await;
    assert_eq!(status, 201, "submit failed: {body}");
    // §7.4: `Work.estate` is never written for a new Work; §7.3's
    // resolved-list replacement is asserted via `bindings` below.
    assert_eq!(body["work"]["workspace"], Value::Null);

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

    // §10.1: and the backend is handed the complete binding summary, not
    // only that cwd — which for a multi-repo Work is precisely the case
    // where the cwd is a directory the Work may *not* write to. Every field
    // is the one sergeant journaled for that binding.
    let summary = &starts[0].bindings;
    assert_eq!(
        summary.len(),
        2,
        "one summary entry per bound repository: {summary:?}"
    );
    for (entry, journaled) in summary.iter().zip(bindings) {
        assert_eq!(entry.repository, journaled["repository"].as_str().unwrap());
        assert_eq!(
            entry.worktree_path,
            PathBuf::from(journaled["worktree_path"].as_str().unwrap()),
            "the exact worktree path the Work may modify"
        );
        assert_eq!(
            entry.work_branch,
            journaled["work_branch"].as_str().unwrap()
        );
        assert_eq!(
            entry.base_branch.as_deref(),
            journaled["base_branch"].as_str()
        );
        assert_eq!(
            entry.base_sha,
            journaled["base_sha"].as_str().unwrap(),
            "the exact base SHA, pinned"
        );
        assert_ne!(
            entry.worktree_path, root,
            "no entry names the surface root: it is not part of the mutation surface"
        );
    }
    assert_eq!(summary[0].base_sha, api_head, "declaration order, again");
    assert_eq!(summary[1].base_sha, web_head);

    handle.shutdown().await;
}

/// R-MVP1-4: repositories that disagree on `instructions` policy are
/// refused at submit — before a Work record or a worktree exists — naming
/// both repositories. "One process, one policy" is the ruling, not an
/// omission: there is nowhere for two `--setting-sources` policies to both
/// take effect in a single launched process.
#[tokio::test]
async fn r_mvp1_4_mixed_instructions_policy_refuses_at_submit_naming_both_repos() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("payments");
    estate_with_manifest(
        &estate,
        "[estate]\nname = \"payments\"\n\n\
         [[repo]]\nname = \"api\"\ninstructions = \"suppress\"\n\n\
         [[repo]]\nname = \"web\"\ninstructions = \"local\"\n",
        &["api", "web"],
    );

    let (registry, fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    // estate-root §7.1: an explicit `--all` selects both repositories, so
    // this test still reaches the instruction-policy check it means to pin
    // rather than tripping the (now separate) missing-scope refusal first.
    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "mixed policy",
        json!({"scope": {"all": true}}),
    )
    .await;
    assert_eq!(status, 422, "a mixed policy must refuse at submit: {body}");
    assert_eq!(body["error"]["code"], "instruction_policy_conflict");
    let message = body["error"]["message"].as_str().expect("message");
    assert!(message.contains("api"), "must name api: {message}");
    assert!(message.contains("web"), "must name web: {message}");
    assert!(
        fake.starts().is_empty(),
        "a submit-time refusal must never reach a backend"
    );

    handle.shutdown().await;
}

/// R-MVP1-4 + MVP-2 D2 item 1: `instructions = "local"` parses, pins, and —
/// now that its launch-side translation is measured
/// (`sergeant-rs-workspace's knowledge/evidence/gauntlet/notes/d2-setting-sources-measurement-2026-08-12.md`,
/// `ClaudeBackend::setting_sources_args`) — is accepted at submit and
/// reaches the backend's `StartRequest` intact. This is the plumbing pin the
/// un-refusal needs: MVP-1's sibling test above already proved a *mixed*
/// selection still refuses (unchanged); this proves a *uniform* `local`
/// selection is no longer refused at all, all the way to the backend.
#[tokio::test]
async fn r_mvp1_4_local_instructions_policy_is_accepted_at_submit_and_reaches_the_backend() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    estate_with_manifest(
        &estate,
        "[estate]\nname = \"solo\"\n\n[[repo]]\nname = \"solo\"\ninstructions = \"local\"\n",
        &["solo"],
    );

    let (registry, fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (status, body) = submit(&client, &handle, &estate, "local measured", json!({})).await;
    assert_eq!(status, 201, "local must be accepted at submit now: {body}");
    assert_eq!(body["work"]["state"], "active");

    let starts = fake.starts();
    assert_eq!(
        starts.len(),
        1,
        "an accepted submission must actually reach the backend"
    );
    assert_eq!(
        starts[0].instruction_policy,
        InstructionPolicy::Local,
        "the resolved policy must survive submit -> bind -> StartRequest unchanged"
    );

    handle.shutdown().await;
}

/// TH-11: R-MVP1-11's refusal pinned end to end over a real HTTP submit —
/// its R-MVP1-4 siblings above are; this one previously wasn't (only at the
/// `engine.plan()` call seam), so nothing asserted the client-visible 422
/// body a real submit actually returns.
#[tokio::test]
async fn r_mvp1_11_a_stage_requiring_ask_refuses_at_submit_over_http() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    // §4.1: workflows resolve against the estate root, so the fixture lives
    // there rather than inside the mount.
    let dir = estate.join(".sergeant/workflows/asks");
    std::fs::create_dir_all(dir.join("00-interview")).expect("stage dir");
    std::fs::write(
        dir.join("workflow.toml"),
        "[workflow]\nname = \"asks\"\nversion = \"1\"\nstages = [\"00-interview\"]\n\n\
         [stage.\"00-interview\"]\nrequires_ask = true\n",
    )
    .expect("workflow.toml");
    std::fs::write(dir.join("00-interview/CONTEXT.md"), "ask questions").expect("CONTEXT.md");

    let no_ask = FakeBackend::scripted(FAKE_BACKEND_NAME, []).with_capabilities(Capabilities {
        ask: false,
        ..Capabilities::default()
    });
    let registry = BackendRegistry::new().with(Arc::new(no_ask.clone()));
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "needs a real ask",
        json!({"workflow": "asks"}),
    )
    .await;
    assert_eq!(status, 422, "must refuse at submit: {body}");
    assert_eq!(body["error"]["code"], "ask_capability_unavailable");
    let message = body["error"]["message"].as_str().expect("message");
    assert!(
        message.contains("asks"),
        "must name the workflow: {message}"
    );
    assert!(
        message.contains("00-interview"),
        "must name the stage: {message}"
    );
    assert!(
        message.contains(FAKE_BACKEND_NAME),
        "must name the backend: {message}"
    );
    assert!(
        no_ask.starts().is_empty(),
        "a submit-time refusal must never reach a backend"
    );

    handle.shutdown().await;
}

/// R-MVP1-4's pin, positive case: a uniform (unset, so `suppress`) policy
/// submits, and `workflow.bound` is widened to carry the resolved
/// repository set plus, per repository, the resolved policy and its
/// instruction-file identity — absent recorded as absent, since neither
/// repo here has an `AGENTS.md`.
#[tokio::test]
async fn r_mvp1_4_workflow_bound_carries_repositories_and_instruction_identities() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("payments");
    estate_with_manifest(
        &estate,
        "[estate]\nname = \"payments\"\n\n\
         [[repo]]\nname = \"api\"\n\n\
         [[repo]]\nname = \"web\"\ninstructions = \"suppress\"\n",
        &["api", "web"],
    );

    let (registry, _fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    // estate-root §7.1: "payments" declares two repositories, so an empty
    // scope is refused now — `--all` is this test's explicit request for
    // both, which is what it actually wants to pin (uniform policy across a
    // multi-repository bind), orthogonal to which scope form asked for it.
    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "uniform policy",
        json!({"scope": {"all": true}}),
    )
    .await;
    assert_eq!(status, 201, "submit failed: {body}");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "active");

    let bound = events_of(data.path(), &work_id, KIND_WORKFLOW_BOUND);
    assert_eq!(bound.len(), 1);
    let payload = &bound[0].payload;
    let repositories = payload["repositories"].as_array().expect("repositories");
    assert_eq!(
        repositories
            .iter()
            .map(|r| r["name"].as_str().expect("name"))
            .collect::<Vec<_>>(),
        ["api", "web"],
        "workflow.bound must carry the resolved repository set, not just the estate name"
    );
    let identities = payload["instruction_identities"]
        .as_array()
        .expect("instruction_identities");
    assert_eq!(identities.len(), 2);
    for identity in identities {
        assert_eq!(identity["policy"], "suppress");
        assert!(
            identity["path"].is_null() && identity["content_hash"].is_null(),
            "no AGENTS.md exists in either worktree — absence must be recorded as absent, \
             not omitted: {identity}"
        );
    }

    handle.shutdown().await;
}

/// 3. A run whose backend completes every stage reaches `completed`, with the
///    stage entry/completion events journaled in order, the worktree removed
///    and the branch retained.
#[tokio::test]
async fn t3_full_run_completes_every_stage_and_retires_the_surface() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    let (repo, _head) = support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, fake) = one_fake([
        FakeStep::complete_with("first done"),
        FakeStep::complete_with("second done"),
    ]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (status, body) = submit(
        &client,
        &handle,
        &estate,
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
    // N3 §12.2/§22.3: this legacy `workflow.toml` carries no `[stage.*]`
    // tables, so the pinned definition must show the same actor-default
    // outcome the tagged-stage tests exercise explicitly — the resolved
    // executor spec is journaled whether or not the workflow tagged it.
    assert_eq!(bound[0].payload["workflow"]["stages"][0]["kind"], "actor");
    assert!(bound[0].payload["workflow"]["stages"][0]["harness"].is_null());
    assert!(bound[0].payload["workflow"]["stages"][0]["profile"].is_null());
    let content_hash = bound[0].payload["workflow"]["content_hash"]
        .as_str()
        .expect("content_hash is a string");
    assert_eq!(
        content_hash.len(),
        64,
        "content_hash is a hex-encoded BLAKE3 digest: {content_hash}"
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

/// N3 Outcomes 2+3 (§12.2, §12.5, §22.3): a `[stage."<id>"]` table's
/// `kind`/`harness`/`profile` are pinned into `workflow.bound` verbatim, the
/// named harness is the one that actually executes the stage, and — because
/// the engine only ever consults the projection folded from that journaled
/// event, never re-reading `workflow.toml` — an edit to the file after bind
/// cannot retroactively change what a running Work sees, even mid-run.
#[tokio::test]
async fn t3b_tagged_stage_definitions_are_pinned_and_survive_file_edits_after_bind() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    estate_with_manifest(
        &estate,
        "[estate]\nname = \"solo\"\n\n[[repo]]\nname = \"solo\"\n\n\
         [[profile]]\nname = \"alt-profile\"\nbackend = \"alt-harness\"\n",
        &["solo"],
    );
    write_workflow_with_tables(
        &estate,
        "tagged-tiny",
        &[
            ("00-first", "first stage context"),
            ("10-second", "second stage context"),
        ],
        "\n[stage.\"00-first\"]\nkind = \"actor\"\nharness = \"alt-harness\"\nprofile = \"alt-profile\"\n",
    );

    // needs_input on the first stage gives a window, mid-run, to mutate the
    // workflow's files on disk before the run continues. The two harnesses
    // are separate registered identities, so which one ran each stage is a
    // fact the test can read off the backends rather than infer.
    let alt = FakeBackend::scripted(
        "alt-harness",
        [
            FakeStep::needs_input("continue?"),
            FakeStep::complete_with("first done"),
        ],
    );
    let (registry, fake) = one_fake([FakeStep::complete_with("second done")]);
    let registry = registry.with(Arc::new(alt.clone()));
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
        "tagged run",
        json!({"workflow": "tagged-tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "needs_input");

    // The tagged stage's executor spec, and the untagged stage's
    // actor-default outcome, are both already in the journal.
    let bound = events_of(data.path(), &work_id, KIND_WORKFLOW_BOUND);
    assert_eq!(bound.len(), 1);
    let stages = &bound[0].payload["workflow"]["stages"];
    assert_eq!(stages[0]["id"], "00-first");
    assert_eq!(stages[0]["kind"], "actor");
    assert_eq!(stages[0]["harness"], "alt-harness");
    assert_eq!(stages[0]["profile"], "alt-profile");
    assert_eq!(stages[1]["id"], "10-second");
    assert_eq!(stages[1]["kind"], "actor");
    assert!(stages[1]["harness"].is_null());
    assert!(stages[1]["profile"].is_null());
    let pinned_hash = bound[0].payload["workflow"]["content_hash"]
        .as_str()
        .expect("content_hash")
        .to_string();

    // Mutate the workflow on disk: different harness for the already-pinned
    // stage, and different context for the stage not yet entered. A fresh
    // `resolve()` of the same name now sees different content...
    write_workflow_with_tables(
        &estate,
        "tagged-tiny",
        &[
            ("00-first", "first stage context"),
            ("10-second", "second stage context, EDITED AFTER BIND"),
        ],
        "\n[stage.\"00-first\"]\nkind = \"actor\"\nharness = \"edited-harness\"\nprofile = \"edited-profile\"\n",
    );
    let edited = WorkflowDefinition::resolve(&estate, "tagged-tiny").expect("resolve edited");
    assert_ne!(
        edited.content_hash, pinned_hash,
        "the edit must actually be execution-relevant, or this test proves nothing"
    );

    // ...but delivering input and letting the run continue must still use
    // exactly what was pinned: the second stage's context reaching the
    // backend is the original, not the edited, text.
    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/input"),
        json!({"command_id": ulid(), "input": "yes"}),
    )
    .await;
    assert_eq!(status, 200, "input rejected: {body}");
    assert_eq!(body["work"]["state"], "completed");
    // §12.5: the tagged stage ran on the harness it named, the untagged one
    // on the Work actor default, and each start carries its stage's profile.
    let alt_starts = alt.starts();
    assert_eq!(alt_starts.len(), 1, "stage 00 runs on its named harness");
    assert_eq!(alt_starts[0].stage_id, "00-first");
    assert_eq!(
        alt_starts[0].profile.as_ref().map(|p| p.name.as_str()),
        Some("alt-profile"),
        "the stage's own profile travels with it"
    );
    let starts = fake.starts();
    assert_eq!(
        starts.len(),
        1,
        "the Work default runs only the untagged stage"
    );
    assert_eq!(starts[0].stage_id, "10-second");
    assert_eq!(
        starts[0].context, "second stage context",
        "a mid-run file edit must not reach a stage not yet entered"
    );

    // And the journal itself never rewrote the bound event: it is exactly
    // the one record from before the edit, hash included.
    let bound_after = events_of(data.path(), &work_id, KIND_WORKFLOW_BOUND);
    assert_eq!(bound_after.len(), 1);
    assert_eq!(bound_after[0], bound[0]);
    assert_eq!(
        bound_after[0].payload["workflow"]["content_hash"],
        pinned_hash
    );
    assert_eq!(
        bound_after[0].payload["workflow"]["stages"][0]["harness"],
        "alt-harness"
    );

    handle.shutdown().await;
}

/// 4. needs_input parks the work, the input API resumes it to completion, and
///    input to a work that is not waiting is a structured error.
#[tokio::test]
async fn t4_needs_input_parks_the_run_and_input_resumes_it() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, fake) = one_fake([
        FakeStep::needs_input("which database?"),
        // The answer unblocks the turn: the next scripted step is what the
        // execution reports once input has been delivered.
        FakeStep::complete(),
        FakeStep::complete(),
    ]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
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

/// #260 Q3: a stage whose `output/README.md` declares an expected artifact
/// completes without ever writing it. First `StageCompleted` signal: the
/// engine spends its one bounded re-prompt (a SEND to the same execution,
/// `stage.output_missing` journaled, work stays `active`, no human
/// involved). Second `StageCompleted` still missing: the Work lands
/// `needs_input`, named `stage_output_missing`, naming the stage and the
/// declared path — recoverable through the ordinary `respond` verb exactly
/// like any other `needs_input`, proven here by actually producing the
/// artifact and responding, which lets the run complete normally.
#[tokio::test]
async fn t10_a_stage_completed_without_its_declared_output_is_reprompted_then_needs_input() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow_with_declared_output(&estate, "proof.md");

    let (registry, fake) = one_fake([
        FakeStep::complete(), // 00-first launch: "done", but proof.md absent
        FakeStep::complete(), // the gate's one bounded re-prompt: still absent
        FakeStep::complete(), // the human's answer, delivered after proof.md exists
        FakeStep::complete(), // 10-second launch: completes the work
    ]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
        "finish without the declared output",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    let worktree = PathBuf::from(
        body["surface"]["bindings"][0]["worktree_path"]
            .as_str()
            .expect("worktree"),
    );

    assert_eq!(
        body["work"]["state"], "needs_input",
        "the second StageCompleted with the artifact still missing must park: {body}"
    );
    assert_eq!(body["stage"]["stage_id"], "00-first");
    let reprompts = events_of(data.path(), &work_id, KIND_STAGE_OUTPUT_MISSING);
    assert_eq!(
        reprompts.len(),
        1,
        "exactly one bounded re-prompt, not unlimited: {reprompts:?}"
    );
    assert_eq!(reprompts[0].payload["stage_id"], "00-first");
    assert_eq!(reprompts[0].payload["path"], "proof.md");
    let execution_id = fake.starts()[0].execution_id.clone();
    assert_eq!(
        fake.inputs(&execution_id),
        vec!["declared output proof.md missing — produce it or state what you need".to_string()],
        "the bounded re-prompt must actually reach the same execution"
    );
    let parked = events_of(data.path(), &work_id, "work.needs_input");
    assert_eq!(parked.len(), 1);
    assert_eq!(
        parked[0].payload["reason_code"], REASON_STAGE_OUTPUT_MISSING,
        "the second failure must carry the named reason: {parked:?}"
    );
    assert_eq!(parked[0].payload["stage_id"], "00-first");
    assert_eq!(parked[0].payload["path"], "proof.md");
    // Never advanced past the stage that failed to produce its output.
    assert!(events_of(data.path(), &work_id, KIND_STAGE_COMPLETED).is_empty());

    // Recoverable through the ordinary respond verb: the actor (a human, in
    // this fixture) produces the declared artifact and answers.
    std::fs::create_dir_all(worktree.join("00-first/output")).expect("output dir");
    std::fs::write(worktree.join("00-first/output/proof.md"), "did it\n").expect("write artifact");
    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/input"),
        json!({"command_id": ulid(), "input": "there, done"}),
    )
    .await;
    assert_eq!(status, 200, "input rejected: {body}");
    // This fixture never runs `git commit` (the fake actor has no shell),
    // so the now-present artifact would leave the worktree dirty on its
    // own — but `00-first`'s declared `evidence` disposition means #260
    // Q4's finalize sweep copies `proof.md` out as retained evidence and
    // removes it from the worktree in its own commit before teardown's
    // dirty check ever runs, so the run reaches plain `completed`. The
    // claim this proves is narrower than that plumbing detail: the gate let
    // the run *advance past* 00-first and reach the end at all, which it
    // refused to do twice above.
    assert_eq!(
        body["work"]["state"], "completed",
        "the artifact is now present, so the gate must let the run complete: {body}"
    );
    // Still exactly one re-prompt ever, across the whole run.
    assert_eq!(
        events_of(data.path(), &work_id, KIND_STAGE_OUTPUT_MISSING).len(),
        1
    );

    handle.shutdown().await;
}

/// Amendment 10d (#260's remaining criterion): a stage whose declared
/// artifact exists but is untyped prose — present, but missing the
/// declared `**Required columns:**` as a markdown table row — must be
/// refused the identical `stage_output_missing`-class way an altogether
/// absent artifact is, per mechanism 1's gate, not a bespoke parser. This
/// is the generic engine mechanism `remediate-findings/00-ingest` opts
/// into by declaring `Required columns`, exercised here on a throwaway
/// `tiny` workflow so the test is not tied to that specific package's
/// prose.
#[tokio::test]
async fn t11_a_present_but_untyped_declared_artifact_is_refused_the_same_way_as_a_missing_one() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    let (mount, _head) = support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow_with_required_columns(&estate, "ingest.md", &["id", "axis"]);

    // The artifact must exist *before* the stage ever launches — the fake
    // actor cannot write files itself — so it is seeded into the source
    // repository's own history, the same way any other pre-existing
    // tracked file would be: present in the worktree from the moment it is
    // materialized, exactly as if a prior (buggy) turn had committed
    // untyped prose instead of a typed table.
    std::fs::create_dir_all(mount.join("00-first/output")).expect("output dir");
    std::fs::write(
        mount.join("00-first/output/ingest.md"),
        "Findings: one about id 1, axis correctness. No table here.\n",
    )
    .expect("seed untyped artifact");
    support::git(&mount, &["add", "-A"]);
    support::git(&mount, &["commit", "-m", "seed untyped artifact"]);

    let (registry, fake) = one_fake([
        FakeStep::complete(), // 00-first launch: the untyped artifact is already there
        FakeStep::complete(), // the one bounded re-prompt: still untyped
        FakeStep::complete(), // the human's answer, delivered after it is retyped
        FakeStep::complete(), // 10-second launch: completes the work
    ]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
        "finish with untyped prose instead of a typed table",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    let worktree = PathBuf::from(
        body["surface"]["bindings"][0]["worktree_path"]
            .as_str()
            .expect("worktree"),
    );

    assert_eq!(
        body["work"]["state"], "needs_input",
        "a present-but-untyped artifact must be refused, not accepted: {body}"
    );
    let reprompts = events_of(data.path(), &work_id, KIND_STAGE_OUTPUT_MISSING);
    assert_eq!(
        reprompts.len(),
        1,
        "exactly one bounded re-prompt: {reprompts:?}"
    );
    let execution_id = fake.starts()[0].execution_id.clone();
    assert_eq!(
        fake.inputs(&execution_id),
        vec!["declared output ingest.md missing — produce it or state what you need".to_string()],
        "the bounded re-prompt must actually reach the same execution"
    );
    let parked = events_of(data.path(), &work_id, "work.needs_input");
    assert_eq!(parked.len(), 1);
    assert_eq!(
        parked[0].payload["reason_code"], REASON_STAGE_OUTPUT_MISSING,
        "the same named reason as an altogether-missing artifact: {parked:?}"
    );
    assert_eq!(parked[0].payload["path"], "ingest.md");
    assert!(events_of(data.path(), &work_id, KIND_STAGE_COMPLETED).is_empty());

    // Recoverable the same way: retype the artifact as a real table and
    // answer — the gate then lets the run proceed.
    std::fs::write(
        worktree.join("00-first/output/ingest.md"),
        "| id | axis |\n|---|---|\n| 1 | correctness |\n",
    )
    .expect("retype the artifact");
    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/input"),
        json!({"command_id": ulid(), "input": "retyped it as a table"}),
    )
    .await;
    assert_eq!(status, 200, "input rejected: {body}");
    // The retype above is uncommitted, same as t10's — but `00-first`'s
    // declared `evidence` disposition means #260 Q4's finalize sweep
    // removes it (retained as evidence) in its own commit before teardown's
    // dirty check runs, so this reaches plain `completed` too.
    assert_eq!(
        body["work"]["state"], "completed",
        "a typed table now satisfies the gate: {body}"
    );

    handle.shutdown().await;
}

/// #260 Q4 / Amendment 9's general finalize sweep, end to end: a run with
/// two stage outputs — one silent on disposition (§1a's default: "silence
/// promotes nothing", so `evidence`) and one explicit `promote` — completes
/// clean, with the evidence-class output gone from the branch but retained
/// on disk, and the promote-class output shipped in the branch's own
/// history untouched.
///
/// Both files are seeded committed into the source repository before
/// submit (the fake actor cannot write files itself), so the worktree is
/// clean the moment each stage completes — proving the sweep runs
/// unconditionally, for the ordinary "the actor already committed its own
/// work" case, not only as dirty-worktree recovery.
#[tokio::test]
async fn the_finalize_sweep_removes_evidence_class_output_and_keeps_promote_class() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    let (mount, _head) = support::scaffold_solo_estate(&estate, "solo");
    write_workflow(
        &estate,
        "sweep",
        &[
            ("00-evidence", "first stage context"),
            ("10-promote", "second stage context"),
        ],
    );
    // `00-evidence` declares nothing at all — §1a's default applies.
    // `10-promote` explicitly opts its output out of the sweep.
    let promote_readme = estate.join(".sergeant/workflows/sweep/10-promote/output");
    std::fs::create_dir_all(&promote_readme).expect("output dir");
    std::fs::write(
        promote_readme.join("README.md"),
        "**Expected artifact:** `deliverable.md` — ships with the Work.\n\n\
         **Disposition:** `promote`\n",
    )
    .expect("output/README.md");

    std::fs::create_dir_all(mount.join("00-evidence/output")).expect("output dir");
    std::fs::write(
        mount.join("00-evidence/output/notes.md"),
        "scratch working notes\n",
    )
    .expect("seed evidence artifact");
    std::fs::create_dir_all(mount.join("10-promote/output")).expect("output dir");
    std::fs::write(
        mount.join("10-promote/output/deliverable.md"),
        "the actual deliverable\n",
    )
    .expect("seed promote artifact");
    support::git(&mount, &["add", "-A"]);
    support::git(&mount, &["commit", "-m", "seed both stage outputs"]);

    let (registry, _fake) = one_fake([FakeStep::complete(), FakeStep::complete()]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
        "sweep evidence, keep promote",
        json!({"workflow": "sweep"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    let surface_root = PathBuf::from(body["surface"]["root"].as_str().expect("surface root"));

    let final_body = get(&client, &handle, &format!("/v1/work/{work_id}")).await;
    assert_eq!(
        final_body["work"]["state"], "completed",
        "the sweep's own commit leaves the worktree clean: {final_body}"
    );
    let branch = format!("sergeant/{work_id}");

    // Evidence-class: gone from the branch's own history...
    let evidence_still_present = Command::new("git")
        .args(["show", &format!("{branch}:00-evidence/output/notes.md")])
        .current_dir(&mount)
        .output()
        .expect("run git")
        .status
        .success();
    assert!(
        !evidence_still_present,
        "evidence-class output must not survive in the branch the sweep committed to"
    );
    // ...but retained on disk, byte for byte.
    let retained = surface_root.join("solo.output/00-evidence/notes.md");
    assert_eq!(
        std::fs::read_to_string(&retained).expect("retained evidence"),
        "scratch working notes\n"
    );

    // Promote-class: still exactly what was seeded, still on the branch.
    assert_eq!(
        git(
            &mount,
            &[
                "show",
                &format!("{branch}:10-promote/output/deliverable.md")
            ]
        ),
        "the actual deliverable"
    );
    assert!(
        !surface_root.join("solo.output/10-promote").exists(),
        "promote-class output is never copied out as evidence — it ships as-is"
    );

    handle.shutdown().await;
}

/// F-IN-01: the finalize sweep's commit must be scoped to the `output/`
/// dirs it actually swept, not the whole worktree (`git add -A`) — else
/// stray dirty/untracked content the actor left *elsewhere* in the
/// worktree gets silently folded into the engine's own finalize commit and
/// laundered clean before `teardown`'s dirty-worktree check ever runs,
/// defeating `retained_dirty` detection for that content. This combines a
/// real evidence-class output (which is what makes the sweep actually
/// commit) with unrelated dirty content in the same worktree — the
/// compound path neither `a_dirty_worktree_is_retained_and_recorded_at_teardown`
/// nor `the_finalize_sweep_removes_evidence_class_output_and_keeps_promote_class`
/// exercises on its own.
#[tokio::test]
async fn a_finalize_sweep_never_launders_unrelated_dirty_content_into_its_own_commit() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    let (mount, _head) = support::scaffold_solo_estate(&estate, "solo");
    write_workflow(&estate, "sweep", &[("00-evidence", "first stage context")]);

    // `00-evidence` declares nothing — §1a's default (`evidence`) applies,
    // so the sweep actually has something to commit.
    std::fs::create_dir_all(mount.join("00-evidence/output")).expect("output dir");
    std::fs::write(
        mount.join("00-evidence/output/notes.md"),
        "scratch working notes\n",
    )
    .expect("seed evidence artifact");
    support::git(&mount, &["add", "-A"]);
    support::git(&mount, &["commit", "-m", "seed evidence output"]);

    let (registry, _fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();
    let (_, body) = submit(
        &client,
        &handle,
        &estate,
        "evidence plus a stray mess",
        json!({"workflow": "sweep"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    let worktree = PathBuf::from(
        body["surface"]["bindings"][0]["worktree_path"]
            .as_str()
            .expect("worktree"),
    );
    let surface_root = PathBuf::from(body["surface"]["root"].as_str().expect("surface root"));

    // Content wholly unrelated to any stage's declared output — the sweep
    // must never touch this.
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
    assert_eq!(
        report["clean"], false,
        "the stray file must still be seen as dirty — not absorbed into \
         the finalize commit: {report}"
    );
    assert_eq!(report["bindings"][0]["disposition"], "retained_dirty");
    assert!(
        report["bindings"][0]["changes"]
            .as_str()
            .expect("changes recorded")
            .contains("half-done.rs"),
        "the stray content must be what teardown found dirty: {report}"
    );
    let patch_path = PathBuf::from(
        report["bindings"][0]["patch"]["path"]
            .as_str()
            .expect("a captured patch path"),
    );
    let patch_text = std::fs::read_to_string(&patch_path).expect("read the captured patch");
    assert!(
        patch_text.contains("half-done.rs") && patch_text.contains("fn main()"),
        "the captured patch must hold the stray content, not be silently \
         dropped by an over-broad finalize commit: {patch_text}"
    );

    // The evidence sweep still did its own job: retained on disk...
    let retained = surface_root.join("solo.output/00-evidence/notes.md");
    assert_eq!(
        std::fs::read_to_string(&retained).expect("retained evidence"),
        "scratch working notes\n"
    );

    handle.shutdown().await;
}

/// R-H0-7 shape 1 (deferred turn finish), engine-level: `FakeStep::settle`
/// already models "a turn that is `Running` for some number of polls after
/// LAUNCH returns, then resolves to its real outcome" — the shape the codex
/// research measured directly on both transports (JSONL turns resolving over
/// real seconds; the app-server's own `item/started` arriving strictly
/// before `item/completed`). No new mechanism exists for this test to drive;
/// it exists to pin that the *engine* — not just the fake — actually
/// respects the window: submit's own synchronous answer must not observe the
/// conclusion before the settle window is spent, only the real completion
/// driver, ticking on its own schedule, gets to see it.
#[tokio::test]
async fn deferred_finish_does_not_let_the_engine_observe_a_conclusion_before_launch_settles() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, fake) = one_fake([
        FakeStep::complete_with("ran the migration").settle(3),
        FakeStep::complete(),
    ]);
    let handle = daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(registry),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            completion_poll: Duration::from_millis(20),
            estate_root: Some(estate.clone()),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
        "resolves over real polls",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(
        body["work"]["state"], "active",
        "the settle window must still be open when submit's own synchronous \
         answer is formed — the engine must not have observed a conclusion \
         that has not settled yet: {body}"
    );
    assert_eq!(body["stage"]["status"], "active");
    let execution_id = fake.starts()[0].execution_id.clone();

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let shown = get(&client, &handle, &format!("/v1/work/{work_id}")).await;
        if shown["work"]["state"] == "completed" {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "the settled outcome never surfaced: {shown}"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let completed = events_of(data.path(), &work_id, KIND_STAGE_COMPLETED);
    assert_eq!(completed[0].payload["detail"], "ran the migration");
    assert!(
        fake.observations()
            .iter()
            .filter(|id| **id == execution_id)
            .count()
            >= 4,
        "the settle window (3 interim OBSERVEs) plus the settled one must \
         actually have been asked, not skipped past: {:?}",
        fake.observations()
    );

    handle.shutdown().await;
}

/// R-H0-7 shape 4 (queued input), engine-level: `queue_input()` against an
/// execution parked in `needs_input`, followed by the ordinary API `input`
/// response that actually resumes the stage, must answer both in the one
/// turn the live response drove — not a turn `queue_input` spent of its own.
/// Read back through `Backend::history()` itself (not `FakeBackend::inputs`,
/// a test-only accessor that bypasses the trait) so what is proven is what
/// the real trait surface would report, the shape a real transcript read
/// would eventually see.
#[tokio::test]
async fn a_resumed_stage_journals_a_queued_item_and_the_live_send_as_one_turn() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, fake) = one_fake([
        FakeStep::needs_input("which database?"),
        FakeStep::complete(),
    ]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
        "queue then answer live",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "needs_input");
    let execution_id = fake.starts()[0].execution_id.clone();

    // Enqueued out of band, before any live SEND — the fake's stand-in for
    // `codex queue --thread <id> --message <text>` against an idle thread.
    fake.queue_input(&execution_id, "also answer this");

    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/input"),
        json!({"command_id": ulid(), "input": "postgres"}),
    )
    .await;
    assert_eq!(status, 200, "input rejected: {body}");
    assert_eq!(body["work"]["state"], "completed");

    // The journal's own record of the resume names exactly the live send —
    // what the operator actually typed, not what the queue held.
    let received = events_of(data.path(), &work_id, KIND_STAGE_INPUT_RECEIVED);
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].payload["input"], "postgres");

    // The backend's own trait-level history shows both, in enqueue order,
    // proving the queued item and the live send both landed in the one
    // resumed turn rather than the queued item spending a turn of its own.
    let execution_handle = ExecutionHandle {
        execution_id: execution_id.clone(),
        native_id: Some(format!("fake-session-{execution_id}")),
    };
    let history = fake.history(&execution_handle).expect("history");
    assert_eq!(
        history,
        vec![
            NativeEvent {
                kind: "conversation.user".to_string(),
                payload: json!({"text": "also answer this", "queued": true}),
            },
            NativeEvent {
                kind: "conversation.user".to_string(),
                payload: json!({"text": "postgres"}),
            },
        ],
        "the queued item and the live send land in the same resumed turn"
    );

    handle.shutdown().await;
}

/// 5. A failed stage fails the work with the reason recorded, and the retry
///    verb re-enters that same stage — succeeding on the scripted second
///    attempt.
#[tokio::test]
async fn t5_failure_records_the_reason_and_retry_re_enters_the_stage() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, fake) = one_fake([
        FakeStep::fail("the tests do not compile"),
        FakeStep::complete(),
        FakeStep::complete(),
    ]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
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
    let estate = repos.path().join("solo-estate");
    let (repo, _head) = support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    // `hang` ignores STOP: the native context refuses to die.
    let (registry, fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
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
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let shown = get(&http(), &handle, &format!("/v1/work/{work_id}")).await;
    assert_eq!(shown["work"]["state"], "canceled");
    handle.shutdown().await;
}

/// 7. The §13 precedence chain, end to end through the API, plus the failure
///    that lists the options.
///
/// Tier three — the estate default — is a property of the *daemon* now, not
/// of the request: §5.2 gives `origin.cwd` no authority, so a single daemon
/// can no longer be walked between a defaulted and an undefaulted estate by
/// varying the submission's cwd. The chain is therefore exercised across two
/// daemons, one bound to each estate, and the tier ordering assertion is
/// unchanged — what moved is which process answers, not which rule wins.
#[tokio::test]
async fn t7_routing_precedence_and_structured_failure() {
    let repos = TempDir::new().expect("tempdir");
    let plain = repos.path().join("plain");
    let configured = repos.path().join("configured");
    support::scaffold_estate(&plain, "plain", &["plain"]);
    estate_with_manifest(
        &configured,
        "[estate]\nname = \"configured\"\ndefault_backend = \"codex\"\n\n\
         [[repo]]\nname = \"configured\"\n",
        &["configured"],
    );

    // Three registered identities, rebuilt per daemon: a `BackendRegistry` is
    // consumed by the start it configures.
    let routing_registry = || {
        BackendRegistry::new()
            .with(Arc::new(FakeBackend::scripted(
                "claude",
                [FakeStep::hang()],
            )))
            .with(Arc::new(FakeBackend::scripted("codex", [FakeStep::hang()])))
            .with(Arc::new(FakeBackend::scripted(
                FAKE_BACKEND_NAME,
                [FakeStep::hang()],
            )))
    };
    let plain_data = TempDir::new().expect("tempdir");
    let plain_handle = start_with(
        plain_data.path(),
        routing_registry(),
        Some(FAKE_BACKEND_NAME),
        Some(&plain),
    )
    .await;
    let configured_data = TempDir::new().expect("tempdir");
    let configured_handle = start_with(
        configured_data.path(),
        routing_registry(),
        Some(FAKE_BACKEND_NAME),
        Some(&configured),
    )
    .await;
    let client = http();

    // Explicit beats origin affinity beats estate default beats global.
    let cases = [
        (
            &plain_handle,
            &plain,
            json!({"backend": "claude", "origin": {"client": "codex", "cwd": plain}}),
            "claude",
            "explicit",
        ),
        (
            &plain_handle,
            &plain,
            json!({"origin": {"client": "codex", "cwd": plain}}),
            "codex",
            "origin_affinity",
        ),
        // Affinity and estate default both populated, and different: the
        // adjacent tiers are only ordered by a case that presents both.
        (
            &configured_handle,
            &configured,
            json!({"origin": {"client": "claude", "cwd": configured}}),
            "claude",
            "origin_affinity",
        ),
        (
            &configured_handle,
            &configured,
            json!({"origin": {"client": "cli", "cwd": configured}}),
            "codex",
            // §13.2's rename left this tier's *journaled* name alone — see
            // `RouteSource::as_str`. One dated spelling beats two spellings
            // of one tier in the same journal.
            "workspace_default",
        ),
        (
            &plain_handle,
            &plain,
            json!({"origin": {"client": "cli", "cwd": plain}}),
            FAKE_BACKEND_NAME,
            "global_default",
        ),
    ];
    for (handle, cwd, extra, expected_backend, expected_source) in cases {
        let (status, body) = submit(&client, handle, cwd, "route me", extra).await;
        assert_eq!(status, 201, "submit failed: {body}");
        assert_eq!(
            body["backend"], expected_backend,
            "expected {expected_backend} via {expected_source}, got {body}"
        );
        assert_eq!(body["route_source"], expected_source);
    }
    configured_handle.shutdown().await;

    // A tier that names an unknown backend fails with the options, and never
    // silently substitutes the one that is available.
    let (status, body) = submit(
        &client,
        &plain_handle,
        &plain,
        "unknown backend",
        // Since W2 the daemon registers a real Opencode adapter by default
        // (codex forced the identical swap one wave earlier, for the same
        // reason), so naming "opencode" here would no longer demonstrate an
        // unknown backend — use "goose" instead, still never registered.
        json!({"backend": "goose"}),
    )
    .await;
    assert_eq!(status, 422);
    assert_eq!(body["error"]["code"], "backend_not_found");
    // "docker" is present even though this registry only names three fakes:
    // `start_with` always registers the real docker adapter unless the
    // config already names one (N4), the same way it always adds claude
    // unless the config pre-empts it — none of these three fakes is named
    // "docker", so nothing pre-empts it here. "agy" is present too, since
    // the agy-adapter sprint's own W2, for the identical reason.
    assert_eq!(
        body["error"]["available_backends"],
        json!([
            "agy",
            "claude",
            "codex",
            "docker",
            FAKE_BACKEND_NAME,
            "opencode"
        ])
    );
    plain_handle.shutdown().await;

    // Nothing selected and no global default: a structured failure that lists
    // what could have been asked for. This daemon is bound to an estate too —
    // an *unbound* one plans nothing, and §13's no-selection outcome is the
    // one case a captured intent is allowed to carry (`Engine::plan`), so
    // reaching the refusal at all requires a real estate to plan against.
    let data = TempDir::new().expect("tempdir");
    let registry = BackendRegistry::new().with(Arc::new(FakeBackend::new("claude")));
    let handle = start_with(data.path(), registry, None, Some(&plain)).await;
    let (status, body) = submit(&http(), &handle, &plain, "nothing selected", json!({})).await;
    assert_eq!(status, 422, "unroutable work must be refused: {body}");
    assert_eq!(body["error"]["code"], "no_backend_selected");
    // The scripted fake occupies the "claude" slot, so the daemon adds
    // nothing there — but it still adds the real docker, codex, opencode,
    // and agy adapters (N4/W2/agy-W2, nothing here is named "docker",
    // "codex", "opencode", or "agy"): agy is now the fifth real adapter the
    // daemon registers by default, same as docker, codex, and opencode.
    assert_eq!(
        body["error"]["available_backends"],
        json!(["agy", "claude", "codex", "docker", "opencode"])
    );
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

// ------------------------------- §22.4. the per-stage harness routing matrix
//
// N3 Outcome 3 (§12.5): "stage harness explicit → else Work actor default via
// today's routing chain → else fail with options. No silent substitution,
// stage by stage." The matrix §22.4 requires has eight rows; each is named on
// the assertion that covers it. Two registered fake harness identities —
// `fake` and `alt-harness` — make "which harness ran this stage" a fact read
// off the backends rather than inferred from a payload.

/// An estate at `root` whose `sergeant.toml` declares one profile per
/// harness, plus a three-stage workflow whose stage tables are `tables`. Both
/// live at the estate root (§4.1); the sole repository is its derived mount
/// `repos/solo` (§6.1), which is why no entry names a path.
///
/// Re-callable on a root it already scaffolded: t7d rewrites the manifest and
/// the workflow mid-test to prove a *bound* stage decision cannot see later
/// edits, and re-initializing an already-committed mount would have nothing
/// to commit.
fn mixed_harness_repo(root: &Path, tables: &str) {
    let mount = root.join("repos").join("solo");
    if !mount.join(".git").exists() {
        init_repo(&mount);
    }
    std::fs::write(
        root.join("sergeant.toml"),
        format!(
            "[estate]\nname = \"solo\"\n\n[[repo]]\nname = \"solo\"\n\n\
             [[profile]]\nname = \"alt-profile\"\nbackend = \"alt-harness\"\n\
             default_model = \"alt-model-1\"\n\n\
             [[profile]]\nname = \"default-profile\"\nbackend = \"{FAKE_BACKEND_NAME}\"\n\
             default_model = \"default-model-1\"\n"
        ),
    )
    .expect("sergeant.toml");
    write_workflow_with_tables(
        root,
        "mixed",
        &[
            ("00-a", "stage a context"),
            ("10-b", "stage b context"),
            ("20-a", "stage a again context"),
        ],
        tables,
    );
}

/// §21.4's verbatim gate item and §22.4's rows 1, 2 and 6: one workflow runs
/// **A → B → A** across two registered harness identities, the unqualified
/// stage takes the Work actor default, and the explicit ones override it.
#[tokio::test]
async fn t7b_a_mixed_harness_workflow_runs_a_then_b_then_a() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    // Stage 10 is the only one that names a harness; 00 and 20 inherit the
    // Work actor default. So the run is default → alt → default: A→B→A.
    mixed_harness_repo(
        &estate,
        "\n[stage.\"10-b\"]\nkind = \"actor\"\nharness = \"alt-harness\"\nprofile = \"alt-profile\"\n",
    );

    let alt = FakeBackend::new("alt-harness");
    let (registry, fake) = one_fake([]);
    let registry = registry.with(Arc::new(alt.clone()));
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "A then B then A",
        json!({"workflow": "mixed"}),
    )
    .await;
    assert_eq!(status, 201, "submit rejected: {body}");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "completed", "{body}");

    // Row 6: the interleaving is real, not a payload claim.
    let default_stages: Vec<String> = fake.starts().iter().map(|s| s.stage_id.clone()).collect();
    let alt_stages: Vec<String> = alt.starts().iter().map(|s| s.stage_id.clone()).collect();
    assert_eq!(
        default_stages,
        vec!["00-a".to_string(), "20-a".to_string()],
        "row 1: every unqualified stage runs on the Work actor default"
    );
    assert_eq!(
        alt_stages,
        vec!["10-b".to_string()],
        "row 2: an explicit stage harness overrides the Work default"
    );

    // Row 3: the stage's profile — and therefore its model (§24.8) — belongs
    // to the harness that stage named.
    assert_eq!(
        alt.starts()[0].model.as_deref(),
        Some("alt-model-1"),
        "the stage profile's model reaches the stage's harness"
    );
    assert!(
        fake.starts()[0].profile.is_none(),
        "an unqualified stage with no Work profile carries none"
    );

    // The decision is pinned in `workflow.bound`, which is what a retry and a
    // restart replay from.
    let bound = events_of(data.path(), &work_id, KIND_WORKFLOW_BOUND);
    let bindings = bound[0].payload["stage_bindings"]
        .as_array()
        .expect("stage_bindings")
        .clone();
    assert_eq!(bindings.len(), 3);
    assert_eq!(bindings[0]["harness"], FAKE_BACKEND_NAME);
    assert_eq!(bindings[0]["route_source"], "work_actor_default");
    assert_eq!(bindings[1]["harness"], "alt-harness");
    assert_eq!(bindings[1]["route_source"], "stage_harness");
    assert_eq!(bindings[1]["profile"]["name"], "alt-profile");
    assert_eq!(bindings[2]["harness"], FAKE_BACKEND_NAME);

    handle.shutdown().await;
}

/// §22.4's rows 4 and 5, and §17.5: an explicit stage harness that is
/// unavailable, unknown, or mismatched with its profile is refused **before
/// Work or worktree side effects**, and never substituted.
#[tokio::test]
async fn t7c_an_unusable_stage_harness_fails_before_any_side_effect() {
    let client = http();

    // Row 4: registered but probe-unavailable.
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    mixed_harness_repo(
        &estate,
        "\n[stage.\"10-b\"]\nkind = \"actor\"\nharness = \"alt-harness\"\n",
    );
    let alt = FakeBackend::new("alt-harness");
    alt.set_available(false, "alt-harness is not logged in");
    let (registry, fake) = one_fake([]);
    let registry = registry.with(Arc::new(alt.clone()));
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;

    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "unavailable stage harness",
        json!({"workflow": "mixed"}),
    )
    .await;
    assert_eq!(status, 422, "must be refused: {body}");
    assert_eq!(body["error"]["code"], "backend_unavailable");
    assert!(
        body["error"]["message"]
            .as_str()
            .expect("message")
            .contains("alt-harness is not logged in"),
        "the probe's own evidence must reach the user: {body}"
    );
    // Row 5 and §17.5: nothing was created, nothing was launched, and above
    // all nothing ran on the *other* registered harness.
    let list = get(&client, &handle, "/v1/work").await;
    assert!(
        list["works"].as_array().expect("works").is_empty(),
        "a stage-harness failure must not create work: {list}"
    );
    assert!(
        fake.starts().is_empty() && alt.starts().is_empty(),
        "no silent provider substitution"
    );
    assert!(
        !data.path().join("surfaces").exists(),
        "§17.5: rejected before worktree side effects"
    );
    handle.shutdown().await;

    // Row 4, second shape: a harness no daemon here registers. Since W2 (the
    // opencode-adapter sprint) the daemon registers a real Opencode adapter
    // by default too, so naming "opencode" here would no longer demonstrate
    // an unregistered harness (it would be host-dependent: `backend_
    // unavailable` with no opencode on `PATH`, or an actual routing success
    // on a host with a real logged-in one) — codex forced the identical swap
    // one wave earlier; use "goose" instead, still never registered.
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    mixed_harness_repo(
        &estate,
        "\n[stage.\"10-b\"]\nkind = \"actor\"\nharness = \"goose\"\n",
    );
    let (registry, fake) = one_fake([]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "unknown stage harness",
        json!({"workflow": "mixed"}),
    )
    .await;
    assert_eq!(status, 422, "must be refused: {body}");
    assert_eq!(body["error"]["code"], "backend_not_found");
    // The daemon registers the real Claude, Codex, Docker, Opencode, and
    // Agy adapters alongside the scripted fake, so the options list names
    // all six; "goose" is still never registered, which is exactly why
    // naming it fails.
    assert_eq!(
        body["error"]["available_backends"],
        json!([
            "agy",
            "claude",
            "codex",
            "docker",
            FAKE_BACKEND_NAME,
            "opencode"
        ])
    );
    assert!(fake.starts().is_empty(), "no silent provider substitution");
    let list = get(&client, &handle, "/v1/work").await;
    assert!(list["works"].as_array().expect("works").is_empty());
    handle.shutdown().await;

    // Row 3's negative: a stage profile that belongs to a different harness.
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    mixed_harness_repo(
        &estate,
        "\n[stage.\"10-b\"]\nkind = \"actor\"\nharness = \"alt-harness\"\n\
         profile = \"default-profile\"\n",
    );
    let (registry, fake) = one_fake([]);
    let registry = registry.with(Arc::new(FakeBackend::new("alt-harness")));
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "profile belongs elsewhere",
        json!({"workflow": "mixed"}),
    )
    .await;
    assert_eq!(status, 422, "must be refused: {body}");
    assert_eq!(body["error"]["code"], "profile_backend_mismatch");
    assert!(
        body["error"]["message"]
            .as_str()
            .expect("message")
            .contains("10-b"),
        "the offending stage must be named: {body}"
    );
    assert!(fake.starts().is_empty());
    handle.shutdown().await;
}

/// §22.4's rows 7 and 8: **retry uses the same pinned stage decision**, and a
/// **restart reconstructs it from the journal** — not from `workflow.toml`,
/// which by then says something else entirely.
#[tokio::test]
async fn t7d_retry_and_restart_replay_the_pinned_stage_decision() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    mixed_harness_repo(
        &estate,
        "\n[stage.\"10-b\"]\nkind = \"actor\"\nharness = \"alt-harness\"\nprofile = \"alt-profile\"\n",
    );

    // Stage 00 completes on the default harness; stage 10 fails on alt.
    let alt = FakeBackend::scripted(
        "alt-harness",
        [FakeStep::fail("alt could not finish"), FakeStep::complete()],
    );
    let (registry, fake) = one_fake([FakeStep::complete(), FakeStep::complete()]);
    let registry = registry.with(Arc::new(alt.clone()));
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "retry the pinned stage",
        json!({"workflow": "mixed"}),
    )
    .await;
    assert_eq!(status, 201, "submit rejected: {body}");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "failed", "{body}");

    // The workflow on disk now says something different for that stage. A
    // decision re-derived at retry time would pick this up; a pinned one
    // cannot see it.
    mixed_harness_repo(
        &estate,
        format!(
            "\n[stage.\"10-b\"]\nkind = \"actor\"\nharness = \"{FAKE_BACKEND_NAME}\"\n\
             profile = \"default-profile\"\n"
        )
        .as_str(),
    );

    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/retry"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 200, "retry rejected: {body}");
    assert_eq!(body["work"]["state"], "completed", "{body}");

    let alt_attempts: Vec<(String, u32, Option<String>)> = alt
        .starts()
        .iter()
        .map(|s| (s.stage_id.clone(), s.attempt, s.model.clone()))
        .collect();
    assert_eq!(
        alt_attempts,
        vec![
            ("10-b".to_string(), 1, Some("alt-model-1".to_string())),
            ("10-b".to_string(), 2, Some("alt-model-1".to_string())),
        ],
        "row 7: the retry re-entered the stage on the same pinned harness, \
         profile and model"
    );
    assert_eq!(
        fake.starts()
            .iter()
            .filter(|s| s.stage_id == "10-b")
            .count(),
        0,
        "the edited file must not move a bound stage to another harness"
    );
    handle.shutdown().await;

    // Row 8: a fresh daemon over the same journal — which re-reads nothing
    // from the repository — re-derives the same executor for every stage.
    let alt = FakeBackend::new("alt-harness");
    let (registry, _fake) = one_fake([]);
    let registry = registry.with(Arc::new(alt.clone()));
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let view = get(&client, &handle, &format!("/v1/work/{work_id}")).await;
    let bound = events_of(data.path(), &work_id, KIND_WORKFLOW_BOUND);
    let bindings = bound[0].payload["stage_bindings"]
        .as_array()
        .expect("stage_bindings")
        .clone();
    assert_eq!(bindings[1]["harness"], "alt-harness");
    assert_eq!(bindings[1]["profile"]["default_model"], "alt-model-1");
    assert_eq!(
        view["stage"]["executor"]["harness"], FAKE_BACKEND_NAME,
        "the restarted daemon exposes the current stage's pinned executor: {view}"
    );
    handle.shutdown().await;
}

/// §12.5's no-substitution rule at **stage entry** — the one place §17.5's
/// preflight structurally cannot reach.
///
/// The preflight refuses an unusable stage harness before the Work exists
/// (t7c), and every later stage entry replays the pinned decision (t7d). Both
/// assume the registry is the one the run was admitted against. It need not
/// be: a daemon restarted without an adapter — dropped from the config, an
/// harness uninstalled, a build without a feature — meets a `StageBinding`
/// naming a harness it does not have, and the fallback that reads naturally
/// there ("use the Work default") is exactly the silent substitution §12.5
/// forbids. It would also hand that harness `binding.profile`, a profile
/// belonging to a different harness.
///
/// So the run is stopped at the boundary instead, and this is the test that
/// says so. Stage 00 fails on the default harness so the work parks somewhere
/// a restart can pick it up; the daemon comes back with `alt-harness` absent;
/// the retry walks stage 00 to completion and then meets stage 10's pin.
#[tokio::test]
async fn t7f_a_bound_stage_whose_harness_left_the_daemon_blocks_rather_than_substituting() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    mixed_harness_repo(
        &estate,
        "\n[stage.\"10-b\"]\nkind = \"actor\"\nharness = \"alt-harness\"\nprofile = \"alt-profile\"\n",
    );

    // First daemon: both harnesses registered, so the submission is admitted
    // and stage 10's binding is pinned to `alt-harness`. Stage 00 fails, which
    // parks the work in `failed` — a state a restart leaves alone and a retry
    // can walk forward.
    let alt = FakeBackend::new("alt-harness");
    let (registry, fake) = one_fake([FakeStep::fail("stage a could not finish")]);
    let registry = registry.with(Arc::new(alt.clone()));
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "the harness leaves between the bind and the stage",
        json!({"workflow": "mixed"}),
    )
    .await;
    assert_eq!(status, 201, "submit rejected: {body}");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "failed", "{body}");
    let bound = events_of(data.path(), &work_id, KIND_WORKFLOW_BOUND);
    assert_eq!(
        bound[0].payload["stage_bindings"][1]["harness"], "alt-harness",
        "the pin under test must actually be in the journal"
    );
    handle.shutdown().await;

    // Second daemon over the same journal, with `alt-harness` deregistered.
    // Nothing in the estate changed; the *daemon* did.
    let (registry, fake2) = one_fake([FakeStep::complete()]);
    assert!(
        !registry.names().iter().any(|n| n == "alt-harness"),
        "the point of this restart is that the pinned harness is gone"
    );
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;

    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/retry"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 200, "retry rejected: {body}");

    // Stage 00 completed on the Work default, and then the run stopped.
    assert_eq!(
        body["work"]["state"], "blocked",
        "a stage pinned to a missing harness must stop the run: {body}"
    );
    let blocked = events_of(data.path(), &work_id, KIND_WORK_BLOCKED);
    let blocked = blocked.last().expect("work.blocked").payload.clone();
    assert_eq!(
        blocked["reason"], "stage harness is unavailable",
        "the block names why, not just that: {blocked}"
    );
    let evidence = blocked["evidence"].as_str().expect("evidence").to_string();
    for fragment in ["10-b", "alt-harness", "will not substitute"] {
        assert!(
            evidence.contains(fragment),
            "the evidence must name {fragment:?}: {evidence}"
        );
    }
    let stage_blocked = events_of(data.path(), &work_id, KIND_STAGE_BLOCKED);
    assert_eq!(
        stage_blocked.last().expect("stage.blocked").payload["stage_id"],
        "10-b",
        "the block is attributed to the stage that named the missing harness"
    );

    // The substitution that would have been silent, denied: the Work default
    // ran stage 00 and only stage 00, and was never handed stage 10 — nor
    // stage 10's profile, which belongs to another harness entirely.
    assert_eq!(
        fake2
            .starts()
            .iter()
            .map(|s| s.stage_id.clone())
            .collect::<Vec<_>>(),
        vec!["00-a".to_string()],
        "the Work default must not pick up a stage another harness was pinned to"
    );
    assert!(
        !fake.starts().iter().any(|s| s.stage_id == "10-b"),
        "and neither did the first daemon's instance of it"
    );
    handle.shutdown().await;
}

/// §22.6's "probing a slow external executable", pinned rather than assumed.
///
/// N3-05: `ClaudeBackend::prepare` is documented as doing "no process, no
/// adapter state, and no blocking call", and the engine calls it under the core
/// lock on that basis — but on a cold probe cache it forks `claude --version`
/// and blocks on the child. The claim held only by the accident that
/// `daemon::start` pre-warms every backend's probe, and nothing tested that
/// dependency.
///
/// It is no longer an accident. Two mechanisms guarantee a warm cache by the
/// time anything runs under the guard, and both are asserted here: the daemon
/// probes every registered backend before it serves a request, and §17.5's
/// preflight — which runs inside `plan`, with no lock held — probes every
/// harness the workflow's stages name, including one no submission has ever
/// routed to.
#[tokio::test]
async fn t7e_every_harness_a_run_will_use_is_probed_before_the_lock_is_taken() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    mixed_harness_repo(
        &estate,
        "\n[stage.\"10-b\"]\nkind = \"actor\"\nharness = \"alt-harness\"\nprofile = \"alt-profile\"\n",
    );

    // 1. The daemon's pre-warm, before any request exists.
    let alt = FakeBackend::new("alt-harness");
    let (registry, fake) = one_fake([]);
    let registry = registry.with(Arc::new(alt.clone()));
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    assert!(
        alt.probe_count() >= 1 && fake.probe_count() >= 1,
        "daemon startup must probe every registered backend before serving: \
         alt={} fake={}",
        alt.probe_count(),
        fake.probe_count()
    );
    handle.shutdown().await;

    // 2. The preflight, on a registry that was never pre-warmed — which is the
    // case that matters, because it is the one the pre-warm does not cover.
    let alt = FakeBackend::new("alt-harness");
    let fake = FakeBackend::new(FAKE_BACKEND_NAME);
    let engine = Engine::new(
        Arc::new(
            BackendRegistry::new()
                .with(Arc::new(fake.clone()))
                .with(Arc::new(alt.clone())),
        ),
        Some(FAKE_BACKEND_NAME.to_string()),
        data.path(),
    )
    // §5.2: the estate is bound to the engine, not read off the request.
    // `cwd` below is still set because a real submission carries one, but it
    // decides nothing — deleting `with_estate_root` leaves this engine
    // planning against no estate at all and `plan` answering `Ok(None)`.
    .with_estate_root(estate.clone());
    assert_eq!(alt.probe_count(), 0, "cold, as a fresh registry is");

    let plan = engine
        .plan(&SubmitContext {
            cwd: Some(&estate),
            workflow: Some("mixed"),
            ..SubmitContext::default()
        })
        .expect("plan")
        .expect("a estate");
    assert!(
        alt.probe_count() >= 1,
        "§17.5's preflight must probe a harness only a stage names — otherwise \
         the first `prepare` for that stage pays for it under the core lock"
    );
    assert_eq!(
        plan.stage_bindings
            .iter()
            .map(|b| b.harness.as_str())
            .collect::<Vec<_>>(),
        vec![FAKE_BACKEND_NAME, "alt-harness", FAKE_BACKEND_NAME]
    );
}

/// 8. A daemon restart re-observes in-flight work through the backend
///    contract: a native session that survived resumes from what it now
///    reports; one the adapter cannot recognise fails closed to `blocked`
///    with the evidence recorded (§25).
#[tokio::test]
async fn t8_restart_resumes_unambiguous_work_and_blocks_ambiguous_work() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    // `durable` models a native session that outlives the daemon: the same
    // backend instance is registered again after the restart. `volatile`
    // models one that did not: a fresh instance with no memory of it.
    let durable = FakeBackend::scripted("durable", [FakeStep::hang()]);
    let volatile = FakeBackend::scripted("volatile", [FakeStep::hang()]);
    let registry = BackendRegistry::new()
        .with(Arc::new(durable.clone()))
        .with(Arc::new(volatile.clone()));
    let handle = start_with(data.path(), registry, Some("durable"), Some(&estate)).await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
        "survives the restart",
        json!({"workflow": "tiny", "backend": "durable"}),
    )
    .await;
    let survivor = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "active");

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
        "loses its session",
        json!({"workflow": "tiny", "backend": "volatile"}),
    )
    .await;
    let orphan = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "active");

    handle.shutdown().await;

    // Capture the first execution ID and its OBSERVE count now — after the
    // first daemon's completion driver has stopped, before the second daemon's
    // reconciliation runs.  The 200 ms completion-driver interval can fire any
    // number of times during the first daemon's lifetime (a timing fact that
    // varies by host and scheduler); pinning the *delta* rather than the total
    // makes the assertion below robust against those extra pre-restart polls.
    let first_execution = durable.starts()[0].execution_id.clone();
    let obs_before_reconcile = durable
        .observations()
        .iter()
        .filter(|id| **id == first_execution)
        .count();

    // While the daemon was down, the surviving session finished its stage.
    durable.complete_live_executions();

    // A brand-new instance: this backend has never heard of the orphan. It is
    // the one the restarted daemon holds, so it is the one whose START count
    // can say anything about what recovery did.
    let restarted_volatile = FakeBackend::scripted("volatile", []);
    let registry = BackendRegistry::new()
        .with(Arc::new(durable.clone()))
        .with(Arc::new(restarted_volatile.clone()));
    let handle = start_with(data.path(), registry, Some("durable"), Some(&estate)).await;
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
    // second time — two OBSERVEs are not guaranteed to agree, and driving
    // from a second would mean acting on an answer no recorded decision was
    // made on.  The diff `obs_after_reconcile - obs_before_reconcile` isolates
    // exactly what reconciliation contributed, independent of how many times
    // the first daemon's completion driver polled `first_execution`.
    let obs_after_reconcile = durable
        .observations()
        .iter()
        .filter(|id| **id == first_execution)
        .count();
    assert_eq!(
        obs_after_reconcile - obs_before_reconcile,
        1,
        "reconciliation must observe the survivor's first execution exactly once \
         (not zero, not twice): got {:?}",
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

/// R-H0-7 shape 2 (a never-arriving terminal), engine-level: a known,
/// never-forgotten execution whose own signal is permanently `Running` (no
/// settle window that eventually resolves — see `FakeStep::never_arrives`'s
/// own doc) is never concluded by observation alone drawing a `completed` or
/// `failed` verdict from the ambiguity. It parks, and only the per-turn
/// ceiling sweep (`Engine::due_interrupts`/`settle_interrupt`) concludes it —
/// landing on `blocked`, the same honest "we asked and nothing else arrived"
/// this fake already reports for `InterruptedRunning`.
///
/// Scoped to `.with_native(NativeState::Exited)` — the app-server variant —
/// deliberately: the *bare* constructor's `NativeState::Unknown` is a
/// different, already-covered story. `check_observation`'s own "native
/// liveness has already had its one and only say" rule (§25) fails an
/// `Unknown` native closed to `blocked` on its very first OBSERVE,
/// regardless of what the signal says — a pre-existing invariant this new
/// shape gets to reuse for free, already pinned by
/// `an_unknown_native_state_blocks_even_when_the_signal_says_completed`
/// elsewhere in this suite. Only once native reads `Exited` does "only the
/// signal decides" apply and the execution genuinely sit `active`/parked
/// with nothing but the ceiling left to conclude it — the same shape
/// `native_liveness_never_decides_work_state_in_either_direction`'s first
/// scenario already proves for `hang().with_native(NativeState::Exited)`,
/// extended here to a signal that is not a hang at all, just permanently
/// unresolved. A different ambiguity source than
/// `t8_restart_resumes_unambiguous_work_and_blocks_ambiguous_work`'s: that
/// test's ambiguity is "the backend has forgotten this execution entirely";
/// this one's is a *known*, never-forgotten execution whose own signal is
/// permanently stuck.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_known_never_forgotten_execution_whose_signal_never_arrives_parks_rather_than_concluding()
{
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, fake) = one_fake([FakeStep::never_arrives().with_native(NativeState::Exited)]);
    let poll = Duration::from_millis(50);
    // 2s, not the sibling tests' 150ms: this test's mid-check asserts the
    // Work is still `active` "well within the ceiling" after a ~100ms sleep
    // plus two HTTP round-trips. With a 150ms ceiling that left a ~50ms
    // margin, which release run 17's instrumented Gate D runner ate — the
    // ceiling tripped first and the mid-check read `blocked` ("turn ceiling
    // exceeded (0.15s)"). A 20x margin keeps the mid-check meaningful on the
    // slowest measured runner; the ceiling-fires wait below stays bounded by
    // `interrupt_budget`, which already carries CI's jitter allowance.
    let ceiling = Duration::from_secs(2);
    let handle = daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(registry),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            completion_poll: poll,
            turn_ceiling: ceiling,
            estate_root: Some(estate.clone()),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
        "never resolves on its own",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "active");
    let execution_id = fake.starts()[0].execution_id.clone();

    // Well within the ceiling, the ambiguity alone must not have concluded
    // anything: still active, never completed, never failed.
    tokio::time::sleep(poll * 2).await;
    let mid = get(&client, &handle, &format!("/v1/work/{work_id}")).await;
    assert_eq!(
        mid["work"]["state"], "active",
        "a signal that never arrives must never read as a conclusion on its \
         own — it must still be active this early: {mid}"
    );

    // Only the ceiling, riding the same completion driver R-MVP1-7 proves it
    // on, eventually acts — and it lands on blocked, never on completed or
    // failed, which an inference from the ambiguous shape could have
    // suggested either way.
    let budget = interrupt_budget(ceiling, poll);
    let submitted_at = Instant::now();
    loop {
        if !events_of(data.path(), &work_id, KIND_TURN_CEILING_INTERRUPTED).is_empty() {
            break;
        }
        assert!(
            submitted_at.elapsed() < budget,
            "the ceiling never fired — the only mechanism this shape can be \
             concluded by"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let shown = get(&client, &handle, &format!("/v1/work/{work_id}")).await;
    assert_eq!(
        shown["work"]["state"], "blocked",
        "only the ceiling concludes a never-arriving execution, and it does \
         so by blocking, never by completing or failing: {shown}"
    );
    assert_eq!(
        fake.native_state(&execution_id),
        Some(NativeState::Exited),
        "the ordinary interrupt path leaves native exactly where it already \
         was for this variant — nothing left to flip"
    );

    handle.shutdown().await;
}

/// §25 in both directions, which no single acceptance test covers: a dead
/// native process that signalled nothing must not fail the work, and a live
/// native process that signalled completion must complete the stage.
#[tokio::test]
async fn native_liveness_never_decides_work_state_in_either_direction() {
    let repos = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    // Dead process, no signal: `native process dead ≠ work failed`.
    let data = TempDir::new().expect("tempdir");
    let (registry, _fake) = one_fake([FakeStep::hang().with_native(NativeState::Exited)]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let (_, body) = submit(
        &http(),
        &handle,
        &estate,
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
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let (_, body) = submit(
        &http(),
        &handle,
        &estate,
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

/// R-H0-7 shape 3 (a distinct interrupted terminal), engine-level: the same
/// invariant as the test above, extended to the *new* native shape §1.3
/// adds. `Backend::interrupt` is only ever called from one place in this
/// engine — the per-turn ceiling sweep — so both scenarios below drive it
/// through that one real path rather than calling it directly: an ordinary
/// step (INTERRUPT flips native to `Exited`, the process-tree-kill shape)
/// and a `.interrupts_natively()` step (native stays `Running`, the
/// harness-confirmed shape) must both park the Work identically —
/// `blocked`, from the ceiling, never a state that varied with which native
/// shape followed. `mod.rs`'s own `Observation` doc says it plainly: "the
/// engine acts only on the signal."
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn native_liveness_after_interrupt_never_decides_work_state_either_way() {
    let repos = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);
    let poll = Duration::from_millis(50);
    let ceiling = Duration::from_millis(150);
    let budget = interrupt_budget(ceiling, poll);

    async fn run_to_ceiling_block(
        estate: &Path,
        poll: Duration,
        ceiling: Duration,
        budget: Duration,
        step: FakeStep,
    ) -> (FakeBackend, String, String) {
        let data = TempDir::new().expect("tempdir");
        let (registry, fake) = one_fake([step]);
        let handle = daemon::start_with(
            data.path(),
            DaemonConfig {
                backends: Arc::new(registry),
                default_backend: Some(FAKE_BACKEND_NAME.to_string()),
                completion_poll: poll,
                turn_ceiling: ceiling,
                estate_root: Some(estate.to_path_buf()),
                ..DaemonConfig::default()
            },
        )
        .await
        .expect("daemon start");
        let client = http();
        let (_, body) = submit(
            &client,
            &handle,
            estate,
            "overdue by construction",
            json!({"workflow": "tiny"}),
        )
        .await;
        let work_id = body["work"]["id"].as_str().expect("work id").to_string();
        let execution_id = fake.starts()[0].execution_id.clone();

        let submitted_at = Instant::now();
        loop {
            if !events_of(data.path(), &work_id, KIND_TURN_CEILING_INTERRUPTED).is_empty() {
                break;
            }
            assert!(submitted_at.elapsed() < budget, "the ceiling never fired");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let shown = get(&client, &handle, &format!("/v1/work/{work_id}")).await;
        assert_eq!(shown["work"]["state"], "blocked", "{shown}");
        handle.shutdown().await;
        (fake, work_id, execution_id)
    }

    // The ordinary path: INTERRUPT flips native to Exited. A huge settle
    // keeps the step genuinely "still running" — never resolving on its
    // own — long enough for the ceiling to be what concludes it.
    let (fake, _work_id, execution_id) = run_to_ceiling_block(
        &estate,
        poll,
        ceiling,
        budget,
        FakeStep::complete_with("plain interrupt").settle(1_000),
    )
    .await;
    assert_eq!(
        fake.native_state(&execution_id),
        Some(NativeState::Exited),
        "the ordinary interrupt path kills the native context"
    );

    // The harness-confirmed path: INTERRUPT leaves native alive — and the
    // Work still parks exactly the same way.
    let (fake, _work_id, execution_id) = run_to_ceiling_block(
        &estate,
        poll,
        ceiling,
        budget,
        FakeStep::complete_with("confirmed interrupt")
            .settle(1_000)
            .interrupts_natively(),
    )
    .await;
    assert_eq!(
        fake.native_state(&execution_id),
        Some(NativeState::Running),
        "interrupts_natively leaves the native context alive by construction, \
         and the Work still parked identically above"
    );
}

/// Waiting and blocked are §12 verbs too, and both are re-enterable through
/// retry. Stage state stays orthogonal to work state throughout: the stage
/// coordinate never becomes a §10 value and the §10 value never names a
/// stage.
#[tokio::test]
async fn waiting_and_blocked_park_the_work_and_retry_re_enters() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, _fake) = one_fake([
        FakeStep::waiting("CI is still running"),
        FakeStep::blocked("needs an architecture decision"),
        FakeStep::complete(),
        FakeStep::complete(),
    ]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
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
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, _fake) = one_fake([FakeStep::fail("out of disk")]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();
    let (_, body) = submit(
        &client,
        &handle,
        &estate,
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
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, _fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();
    let (_, body) = submit(
        &client,
        &handle,
        &estate,
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
    // #109: the dirty *state* survives teardown, never the directory it
    // happened to live in — the worktree itself is reclaimed once its
    // content is captured as a patch.
    assert!(
        !worktree.exists(),
        "#109: the reclaimed worktree directory must be gone"
    );
    let patch_path = PathBuf::from(
        report["bindings"][0]["patch"]["path"]
            .as_str()
            .expect("a captured patch path"),
    );
    let patch_text = std::fs::read_to_string(&patch_path).expect("read the captured patch");
    assert!(
        patch_text.contains("half-done.rs") && patch_text.contains("fn main()"),
        "the captured patch must hold what the dirty worktree actually had: {patch_text}"
    );

    handle.shutdown().await;
}

/// #109's inspect and dispose verbs, end to end through the real API: a
/// dirty worktree's teardown shows up in `GET /v1/retained`; `POST
/// /v1/work/{id}/reap` refuses without confirmation and destroys the
/// captured patch once confirmed; and the entry is gone from `/v1/retained`
/// afterward. Reverting either handler (dropping the confirm gate, or
/// `surface::reap` itself) fails a different assertion here.
#[tokio::test]
async fn retained_lists_a_dirty_teardown_and_reap_disposes_of_it_only_when_confirmed() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, _fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();
    let (_, body) = submit(
        &client,
        &handle,
        &estate,
        "leaves a mess for #109",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    let worktree = PathBuf::from(
        body["surface"]["bindings"][0]["worktree_path"]
            .as_str()
            .expect("worktree"),
    );
    std::fs::write(worktree.join("half-done.rs"), "fn main() {}\n").expect("dirty the worktree");

    let (status, _) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/cancel"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 200);

    // The inspect verb finds it.
    let retained = get(&client, &handle, "/v1/retained").await;
    let entries = retained["retained"].as_array().expect("retained array");
    let entry = entries
        .iter()
        .find(|e| e["work_id"] == work_id)
        .expect("the dirty binding is listed");
    assert_eq!(entry["repository"], "solo");
    assert_eq!(entry["disposition"], "retained_dirty");
    let bytes = entry["bytes"].as_u64().expect("bytes");
    assert!(bytes > 0, "a real patch has a nonzero size: {entry}");
    let patch_path = PathBuf::from(entry["path"].as_str().expect("patch path"));
    assert!(
        patch_path.is_file(),
        "the patch the inspect verb names must actually exist"
    );

    // Reap refuses without confirmation, and destroys nothing.
    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/reap"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 400, "{body}");
    assert!(
        patch_path.is_file(),
        "an unconfirmed reap must not touch anything"
    );

    // Confirmed, it actually reaps.
    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/reap"),
        json!({"command_id": ulid(), "confirm": true}),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        body["report"]["bindings"][0]["outcome"]["outcome"], "reaped",
        "{body}"
    );
    assert_eq!(
        body["report"]["bindings"][0]["outcome"]["bytes"], bytes,
        "the freed size must match what was actually there: {body}"
    );
    assert!(!patch_path.exists(), "reap must actually delete the patch");

    // And it is gone from the inspect view.
    let retained_after = get(&client, &handle, "/v1/retained").await;
    assert!(
        retained_after["retained"]
            .as_array()
            .expect("retained array")
            .iter()
            .all(|e| e["work_id"] != work_id),
        "a reaped binding must not still be listed: {retained_after}"
    );

    handle.shutdown().await;
}

/// ADR 0007(b): a closing stage that declares a commit as its durable
/// outcome must not be reported as plain `completed` when the branch never
/// advanced and the worktree was left dirty — the safety net for when an
/// actor guesses wrong about its own runtime model anyway (independent of
/// ADR 0007(a), which is what a headless turn's own first-turn prompt
/// states).
///
/// Reproduces the real shape: a stage parks in `waiting` (so its worktree is
/// never torn down), the test dirties that worktree exactly the way an
/// actor stranding mid-command would, and the run is then driven to a
/// normal `StageCompleted` signal for every remaining stage — the fake
/// backend never runs `git commit`, so the branch never advances past its
/// base SHA either. The engine still learns nothing about what a commit
/// *is*: the check only ever compares a teardown disposition and a SHA the
/// pointer already computed (`ADR 0007`).
#[tokio::test]
async fn a_stranded_completion_is_not_reported_as_plain_completed() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, _fake) = one_fake([
        FakeStep::waiting("needs a second look"),
        FakeStep::complete_with("first done"),
        FakeStep::complete_with("second done"),
    ]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
        "declares a commit, never makes one",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(
        body["work"]["state"], "waiting",
        "the first script step parks the run without tearing anything down: {body}"
    );
    let worktree = PathBuf::from(
        body["surface"]["bindings"][0]["worktree_path"]
            .as_str()
            .expect("worktree"),
    );

    // The stage's own actor would have run `git commit` here; it strands
    // instead, exactly the shape #94 produced twice on 2026-08-14.
    std::fs::write(worktree.join("half-done.rs"), "fn main() {}\n").expect("dirty the worktree");

    // Retry re-enters the stage (waiting -> active is legal), and the
    // scripted run then completes every remaining stage normally: the fake
    // backend signals `StageCompleted` twice in a row and never touches
    // git, so this one HTTP call drives the whole run to `completed` and
    // through teardown.
    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/retry"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 200, "retry rejected: {body}");

    // The pointer's own facts prove the shape: the branch never advanced
    // past its base SHA, and the worktree it left behind is still dirty.
    assert_eq!(body["teardown"]["clean"], false, "{body}");
    assert_eq!(
        body["output"]["repositories"][0]["disposition"], "retained_dirty",
        "{body}"
    );
    // #109: the dirty state survives teardown as a captured patch, not the
    // worktree directory itself.
    assert!(
        !worktree.exists(),
        "#109: the reclaimed worktree directory must be gone: {body}"
    );

    // The one thing an operator reads first must not say plain `completed`.
    assert_ne!(
        body["work"]["state"], "completed",
        "a closing stage that declared a commit but never advanced the \
         branch, leaving the worktree dirty, must not be reported as plain \
         success: {body}"
    );
    assert_eq!(
        body["work"]["state"], "completed_dirty",
        "the honest label the pointer already supports: {body}"
    );

    // The same fact must be visible from `sgt work list` — the place an
    // operator looks first, per the ADR's own framing of the problem — not
    // only from `work show`'s fuller record.
    let list = get(&client, &handle, "/v1/work").await;
    let row = list["works"]
        .as_array()
        .expect("works")
        .iter()
        .find(|w| w["id"] == work_id)
        .expect("the work is listed");
    assert_ne!(row["state"], "completed", "{list}");
    assert_eq!(row["state"], "completed_dirty", "{list}");

    // The persisted lifecycle state is untouched: this run is genuinely
    // terminal (retry refuses it, exactly as any other `completed` work),
    // never blocked or failed. Only the reported label changed.
    let (status, _) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/retry"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(
        status, 409,
        "a stranded completion is still `Completed` underneath — terminal, not retryable"
    );

    handle.shutdown().await;
}

/// #173 / §11.7 end to end: a run whose worktree ends on a *different* branch
/// completes, and says so.
///
/// The defect this closes was silent by construction. `final_sha` is read
/// from `refs/heads/sergeant/<work-id>` and `git status --porcelain` only
/// asks whether anything is uncommitted, so a worktree that checked out
/// another branch and committed there was clean by both measures: teardown
/// reported `removed` at the base SHA, `sgt work show` reported plain
/// `completed`, and `git worktree remove` deleted the only record of where
/// the output had actually gone. §17's acceptance line — a worktree ending on
/// another branch can no longer be reported clean, nor represented as though
/// the expected branch contains its output — is what these assertions are.
///
/// Driven through the real daemon rather than `surface::teardown` directly,
/// because the claim is about what an operator is told: the finding has to
/// survive the journal, the projection fold, and both read paths (`work show`
/// and `work list`) to be worth anything.
#[tokio::test]
async fn a_run_that_ends_on_the_wrong_branch_completes_dirty_and_keeps_both_branches() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    let (repo, base_sha) = support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, _fake) = one_fake([
        FakeStep::waiting("parks so the fixture can move the worktree"),
        FakeStep::complete_with("first done"),
        FakeStep::complete_with("second done"),
    ]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
        "commits somewhere else entirely",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    let work_branch = format!("sergeant/{work_id}");
    let worktree = PathBuf::from(
        body["surface"]["bindings"][0]["worktree_path"]
            .as_str()
            .expect("worktree"),
    );

    // What an actor that mis-drives git does: switch off the assigned branch
    // and commit the run's output there instead.
    git(&worktree, &["checkout", "-b", "renegade"]);
    std::fs::write(worktree.join("output.rs"), "fn main() {}\n").expect("write output");
    git(&worktree, &["add", "."]);
    git(&worktree, &["commit", "-m", "output, on the wrong branch"]);
    let renegade_sha = git(&worktree, &["rev-parse", "HEAD"]);

    // Retry re-enters the parked stage and the scripted run completes every
    // remaining stage, driving the whole run through teardown.
    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/retry"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 200, "retry rejected: {body}");

    // A clean worktree on the wrong branch is still removed — the content is
    // committed and both branches are durable — but the report names where
    // it actually was.
    assert_eq!(
        body["teardown"]["bindings"][0]["disposition"], "removed",
        "{body}"
    );
    assert_eq!(
        body["teardown"]["bindings"][0]["observed_head"],
        json!({"head": "branch", "branch": "renegade", "sha": renegade_sha}),
        "{body}"
    );

    // §11.5's axis, and the §11.3 finding behind it.
    assert_eq!(body["integrity"]["disposition"], "dirty", "{body}");
    let finding = &body["integrity"]["findings"][0];
    assert_eq!(finding["finding"], "assigned_branch_mismatch", "{body}");
    assert_eq!(finding["expected_branch"], work_branch, "{body}");
    assert_eq!(finding["expected_sha"], base_sha, "{body}");
    assert_eq!(finding["observed_branch"], "renegade", "{body}");
    assert_eq!(finding["observed_sha"], renegade_sha, "{body}");

    // Work state is untouched (§11.5: no new transition target); only the
    // reported label carries the axis.
    assert_eq!(body["work"]["state"], "completed_dirty", "{body}");
    let (status, _) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/retry"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(
        status, 409,
        "still `Completed` underneath — terminal, not retryable"
    );

    // C5: distinguishable from `sgt work list`, not only from `work show`.
    let list = get(&client, &handle, "/v1/work").await;
    let row = list["works"]
        .as_array()
        .expect("works")
        .iter()
        .find(|w| w["id"] == work_id)
        .expect("the work is listed");
    assert_eq!(row["state"], "completed_dirty", "{list}");
    assert_eq!(row["integrity"]["disposition"], "dirty", "{list}");

    // §11.7 / §12.1: both named branches survive in the source repository,
    // so the commit the run actually made is still reachable by name.
    assert!(
        branch_exists(&repo, &work_branch),
        "the expected branch is always retained, advanced or not"
    );
    assert!(
        branch_exists(&repo, "renegade"),
        "the branch the run actually used is never deleted either"
    );
    assert_eq!(
        git(&repo, &["rev-parse", "refs/heads/renegade"]),
        renegade_sha,
        "and it still points at the run's output"
    );

    handle.shutdown().await;
}

/// Amendment 9 Q5 / #260 mechanism 3: a stage that declares
/// `receives_branch_status = true` gets the engine's own
/// commits-on-branch-since-base fact appended to its `CONTEXT.md`; a stage
/// that never declares it reaches the backend byte-for-byte unchanged (§12:
/// procedure is data, except for this one named, opt-in exception).
///
/// `00-first` parks on `waiting` so the fixture can commit into the
/// worktree before `10-second` — the one stage that opted in — is entered,
/// making "yes (1)" the only honest fact the engine could report.
#[tokio::test]
async fn a_stage_that_opts_in_receives_the_branch_status_fact_others_do_not() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    let (_repo, _head) = support::scaffold_solo_estate(&estate, "solo");
    write_workflow_with_tables(
        &estate,
        "tiny",
        &[
            ("00-first", "first stage context"),
            ("10-second", "second stage context"),
        ],
        "\n[stage.\"10-second\"]\nreceives_branch_status = true\n",
    );

    let (registry, fake) = one_fake([
        FakeStep::waiting("parks so the fixture can commit into the worktree"),
        FakeStep::complete_with("first done"),
        FakeStep::complete_with("second done"),
    ]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
        "opt in to the branch status fact",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    let worktree = PathBuf::from(
        body["surface"]["bindings"][0]["worktree_path"]
            .as_str()
            .expect("worktree"),
    );

    // One real commit on the run's own branch, before the opted-in stage is
    // ever entered.
    std::fs::write(worktree.join("output.rs"), "fn main() {}\n").expect("write output");
    git(&worktree, &["add", "."]);
    git(
        &worktree,
        &["commit", "-m", "the fact 10-second should see"],
    );

    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/retry"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 200, "retry rejected: {body}");
    assert_eq!(body["work"]["state"], "completed", "{body}");

    // Three executions consumed the three-step script: 00-first's parked
    // attempt, its retried (completing) attempt, then 10-second.
    let starts = fake.starts();
    assert_eq!(starts.len(), 3, "{starts:?}");
    assert_eq!(
        starts[0].context, "first stage context",
        "00-first never declared receives_branch_status: its context is untouched"
    );
    assert_eq!(
        starts[1].context, "first stage context",
        "the retried attempt is still 00-first — untouched either way"
    );
    assert_eq!(
        starts[2].context,
        "second stage context\n\n## Engine-computed branch status\n\n\
         commits_on_branch_since_base: yes (1)\n",
        "10-second opted in: the engine's own commit count is appended, not substituted"
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
    let estate = repos.path().join("payments");
    let api = estate.join("repos").join("api");
    init_repo(&api);
    // A mount §8.1 admits and materialization then cannot write to: a real
    // repository at the derived path, on a clean attached HEAD with a full
    // SHA and no colliding ref or path — every question preflight asks is
    // answered — whose `.git` is read-only, so the `git worktree add` that
    // comes *after* admission is the thing that fails.
    //
    // Before Phase E this was a repository with no commits at all, which
    // materialization discovered by failing `rev-parse HEAD` mid-way. §8.1
    // check 7 now refuses that submission before a Work record exists (see
    // `git_preflight_refuses_an_unresolvable_head_and_says_no_override_applies`),
    // so it can no longer reach the rollback path this test is about.
    let web = estate.join("repos").join("web");
    init_repo(&web);
    // `.git` *and* its ref storage: git creates the branch before it creates
    // the worktree registry entry, so leaving `refs/heads` writable would
    // strand a real `sergeant/<work-id>` branch in `web` — which is not the
    // failure this test is about (the last assertion below pins that `web`
    // got nothing at all).
    let web_readonly = [
        web.join(".git"),
        web.join(".git/refs"),
        web.join(".git/refs/heads"),
    ];
    set_writable(&web_readonly, false);
    std::fs::write(
        estate.join("sergeant.toml"),
        "[estate]\nname = \"payments\"\n\n\
         [[repo]]\nname = \"api\"\n\n\
         [[repo]]\nname = \"web\"\n",
    )
    .expect("sergeant.toml");

    let (registry, fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    // estate-root §7.1: "payments" declares two repositories; `--all`
    // selects both so materialization is actually attempted against `web`
    // and can fail the way this test means to pin.
    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "half a surface",
        json!({"scope": {"all": true}}),
    )
    .await;
    // Restored the moment materialization is over, so the rig's own cleanup
    // can still remove the directory.
    set_writable(&web_readonly, true);

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

/// R-MVP1-1: `surfaces_root` is a path distinct from `data_dir`, not merely
/// `data_dir`'s fixed `surfaces/` child. A `data_dir` *inside* the source
/// checkout materializes fine once `[estate] surfaces_dir` points the
/// surface root outside it — the pin's exact scenario. The guard that
/// refuses a surface inside the checkout moved from watching `data_dir` to
/// watching the resolved `surfaces_root`; it did not weaken: a
/// `surfaces_dir` left unset (falling back to `data_dir/surfaces`, still
/// inside the checkout here) still refuses, exactly as an in-checkout
/// `data_dir` always did before the split.
#[tokio::test]
async fn r_mvp1_1_surfaces_root_is_split_from_data_dir_and_the_checkout_guard_follows_it() {
    let repos = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    // The "source checkout" the guard watches is the *mount* (§6.1), so that
    // is where an in-checkout `data_dir` has to live for either scenario to
    // mean anything — not the estate root, which is not a repository at all.
    let repo = estate.join("repos").join("solo");

    // Scenario 1: `data_dir` lives *inside* the checkout — before R-MVP1-1
    // this alone doomed every submission, since the surface root was always
    // `data_dir/surfaces`. `surfaces_dir` points outside it; materializing
    // must succeed anyway.
    let data_inside = repo.join(".sergeant-data");
    let outside = repos.path().join("surfaces-outside");
    estate_with_manifest(
        &estate,
        &format!(
            "[estate]\nname = \"solo\"\nsurfaces_dir = {:?}\n\n[[repo]]\nname = \"solo\"\n",
            outside.to_string_lossy()
        ),
        &["solo"],
    );
    // A data dir *inside* the mount is an untracked directory in it, which
    // §8.1 check 8 correctly reads as a dirty mount. That is a property of
    // this scenario's unusual layout, not of the split under test, so it is
    // excluded the ordinary way an operator would exclude it — in the
    // repository's own `.git/info/exclude`, which needs no commit and adds
    // nothing to the checkout's tracked content.
    std::fs::write(repo.join(".git/info/exclude"), ".sergeant-data*\n")
        .expect("write .git/info/exclude");

    let (registry, _fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        &data_inside,
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();
    let (status, body) = submit(&client, &handle, &estate, "split surfaces root", json!({})).await;
    assert_eq!(status, 201, "submit failed: {body}");
    assert_eq!(
        body["work"]["state"], "active",
        "an outside surfaces_dir must materialize even with data_dir inside the checkout: {body}"
    );
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    let worktree = PathBuf::from(
        body["surface"]["bindings"][0]["worktree_path"]
            .as_str()
            .expect("worktree path"),
    );
    // Exact, not merely `starts_with`: `surface_root` must join `work_id`
    // directly onto `surfaces_dir` with no implicit `surfaces/` re-nested
    // inside it (that join happens exactly once, at `Engine`'s own default
    // computation — `surface_root(surfaces_root, work_id)` no longer adds
    // `SURFACES_DIR` itself). A `starts_with` check alone cannot catch a
    // regression that re-nests an extra `surfaces/` under an already-custom
    // `surfaces_dir`, since the result would still be a prefix match.
    assert_eq!(
        worktree,
        outside.join(&work_id).join("solo"),
        "the worktree must live directly under the declared surfaces_dir/<work_id>/<repo>, \
         with no extra nesting"
    );
    // TH-07's other half: `surface.planned`'s own `root` — not just the
    // worktree path it later produces — must reflect the split too.
    let planned = events_of(&data_inside, &work_id, KIND_SURFACE_MATERIALIZING);
    assert_eq!(planned.len(), 1, "exactly one plan: {planned:?}");
    assert_eq!(
        planned[0].payload["plan"]["root"],
        outside.join(&work_id).display().to_string(),
        "surface.planned's root must already reflect surfaces_dir's split, before \
         anything is materialized: {:?}",
        planned[0].payload
    );
    handle.shutdown().await;

    // Scenario 2: `surfaces_dir` unset falls back to `data_dir/surfaces` —
    // still inside the checkout here — and the guard still refuses it.
    let data_inside2 = repo.join(".sergeant-data-2");
    std::fs::write(
        estate.join("sergeant.toml"),
        "[estate]\nname = \"solo\"\n\n[[repo]]\nname = \"solo\"\n",
    )
    .expect("sergeant.toml");
    let (registry, _fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        &data_inside2,
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();
    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "unsplit still refuses",
        json!({}),
    )
    .await;
    assert_eq!(status, 201, "submit failed: {body}");
    assert_eq!(
        body["work"]["state"], "blocked",
        "an in-checkout surfaces_root must still be refused — the guard moved, not weakened: {body}"
    );
    // TH-07: a bare `state == "blocked"` cannot distinguish the in-checkout
    // guard from any other unrelated materialize failure. Name the actual
    // diagnostic (`surface.rs`'s own refusal text) so this pin means what
    // its own doc comment claims.
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    let blocked = events_of(&data_inside2, &work_id, KIND_WORK_BLOCKED);
    assert_eq!(blocked.len(), 1, "exactly one block: {blocked:?}");
    let reason = blocked[0].payload["reason"]
        .as_str()
        .expect("reason string");
    assert!(
        reason.contains("refusing to materialize a work surface")
            && reason.contains("inside source repository"),
        "the block reason must be the in-checkout guard's own diagnostic, not some other \
         materialize failure: {reason:?}"
    );
    handle.shutdown().await;
}

/// R-MVP1-1's other override, `SGT_SURFACES_DIR`, end to end through a real
/// spawned daemon process (not the in-process `daemon::start_with` the
/// scenario above uses) — proving the env var this ruling names is actually
/// read, not merely documented. `Engine::with_surfaces_root` had zero
/// callers before this fix; a real `sgt run` against a `data_dir` inside the
/// checkout, with `SGT_SURFACES_DIR` pointed outside it, must materialize
/// exactly as an `[estate] surfaces_dir` override does.
#[test]
fn r_mvp1_1_sgt_surfaces_dir_env_var_reaches_a_real_spawned_daemon() {
    let repos = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let data = DataDir::new();
    let outside = repos.path().join("surfaces-outside-env");
    // §4.1/§4.3: the process cwd *is* the estate root the command is
    // admitted against — there is no walk from a mount up to it.
    let output = Command::new(SGT)
        .current_dir(&estate)
        .arg("--data-dir")
        .arg(data.path())
        .arg("--json")
        .args(["run", "env var surfaces root", "--workflow", "tiny"])
        .env("SGT_SURFACES_DIR", &outside)
        .output()
        .expect("run sgt");
    assert!(
        output.status.success(),
        "sgt run: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let body: Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_ne!(
        body["work"]["state"], "blocked",
        "an outside SGT_SURFACES_DIR must materialize even with data_dir inside the checkout: {body}"
    );
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    let worktree = PathBuf::from(
        body["surface"]["bindings"][0]["worktree_path"]
            .as_str()
            .expect("worktree path"),
    );
    // Exact, not merely `starts_with` — mirrors the `[estate] surfaces_dir`
    // scenario above (materialize_one refuses a worktree inside the source
    // checkout, so a path outside it, actually returned here, proves the
    // env var reached `Engine::with_surfaces_root` and was actually used to
    // materialize, not merely echoed back).
    assert_eq!(
        worktree,
        outside.join(&work_id).join("solo"),
        "SGT_SURFACES_DIR must relocate the worktree exactly as [estate] surfaces_dir does"
    );
    // The fake backend may complete (and tear the surface down) before this
    // process is spawned, so a disk check would be racy; the response body
    // above is captured synchronously at materialize time and is proof
    // enough. The default (data_dir/surfaces, inside the checkout here)
    // must not have been used either way.
    assert!(!data.path().join("surfaces").join(&work_id).exists());

    stop_daemon(data.path());
}

/// Retry's re-attachment falls back to a fresh branch cut from the recorded
/// base SHA when the branch teardown was supposed to have retained is gone —
/// deleted by something outside sergeant between the failed attempt and the
/// retry. §12 promises retry as the one door back out of `failed`, and that
/// promise must hold even when the "durable" half of a torn-down surface
/// (the branch) has itself vanished out from under it.
#[tokio::test]
async fn retry_rebuilds_from_base_sha_when_the_retained_branch_is_gone() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    let (repo, base_sha) = support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, _fake) = one_fake([
        FakeStep::fail("boom"),
        FakeStep::complete(),
        FakeStep::complete(),
    ]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
        "branch vanishes before retry",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "failed");

    let branch = format!("sergeant/{work_id}");
    assert!(
        branch_exists(&repo, &branch),
        "teardown must retain the branch on failure"
    );

    // Something outside sergeant deletes the branch teardown was supposed
    // to have kept — a stray cleanup, an operator, a rebase.
    git(&repo, &["branch", "-D", &branch]);
    assert!(!branch_exists(&repo, &branch));

    // …and the repository moves on, so the two candidate start points for
    // the rebuilt worktree are *different commits*. Without this the
    // fixture's `main` still points at `base_sha` (one commit from
    // `init_repo`, an uncommitted workflow, a failed attempt that committed
    // nothing), and "cut from the recorded base SHA" would be
    // indistinguishable from "cut from whatever the default branch happens
    // to point at now" — the head_sha assertion below would hold either way.
    std::fs::write(repo.join("DRIFT.md"), "main moved on\n").expect("write drift file");
    git(&repo, &["add", "DRIFT.md"]);
    git(
        &repo,
        &["commit", "-m", "main moves on after the failed attempt"],
    );
    let main_tip = git(&repo, &["rev-parse", "HEAD"]);
    assert_ne!(
        main_tip, base_sha,
        "the fixture must leave main ahead of the recorded base SHA"
    );

    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/retry"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(
        status, 200,
        "retry must not fail just because the retained branch is gone: {body}"
    );
    assert_eq!(body["work"]["state"], "completed");

    // The branch is back — recreated by the fallback, not merely reused.
    assert!(
        branch_exists(&repo, &branch),
        "rematerialize must recreate the branch retry needs"
    );

    let materialized = events_of(data.path(), &work_id, KIND_SURFACE_MATERIALIZED);
    let rematerialized: Vec<&Event> = materialized
        .iter()
        .filter(|e| e.payload["rematerialized"] == true)
        .collect();
    assert_eq!(
        rematerialized.len(),
        1,
        "retry must record the rematerialization: {materialized:?}"
    );
    let binding = &rematerialized[0].payload["surface"]["bindings"][0];
    assert_eq!(
        binding["base_sha"], base_sha,
        "provenance is unchanged by the fallback"
    );
    assert_eq!(
        binding["head_sha"], base_sha,
        "with no branch left to preserve, the rebuilt worktree starts \
         exactly at the recorded base SHA — not at {main_tip}, where the \
         repository's default branch has since moved to"
    );

    handle.shutdown().await;
}

/// Clears an `chattr +i` immutable bit however the test that set it leaves.
///
/// The immutable directory lives inside a `TempDir`, and `TempDir::drop`
/// *swallows* removal errors — so a panicking assertion between setting the
/// bit and clearing it would leave a directory behind that not even root can
/// `rm` until someone runs `chattr -i` by hand. A straight-line clear only
/// runs on the happy path; a `Drop` runs on the unwinding one too. Same
/// shape as `m2_daemon_api.rs`'s `ReapOnDrop`.
#[cfg(target_os = "linux")]
struct ClearImmutableOnDrop(PathBuf);

#[cfg(target_os = "linux")]
impl Drop for ClearImmutableOnDrop {
    fn drop(&mut self) {
        // Best-effort by construction: a `Drop` that panics during an
        // unwind aborts the process, which would hide the real failure.
        let _ = Command::new("chattr").arg("-i").arg(&self.0).status();
    }
}

/// Teardown's own `RetainedError` disposition: a worktree Git itself
/// refuses to remove, distinct from a dirty one it declines to touch.
///
/// `chattr +i` marks the worktree directory immutable at the filesystem
/// level rather than merely unwritable by permission bits — this daemon
/// (and this test suite) runs as root, for whom an ordinary permission bit
/// is not an obstacle, but the immutable attribute blocks deletion
/// regardless of privilege. `git status --porcelain` still succeeds (reads
/// are unaffected), so teardown proceeds to the removal itself and only
/// that fails, giving `RetainedError` — not `RetainedDirty` — with Git's
/// own diagnostic, and the same evidence journaled.
///
/// **Environment precondition**, two shapes (same policy as the journal
/// O_DIRECT fixture's probe above; S-series coverage-lane runs
/// 31448824808/31452115043 measured both): `chattr` missing from `PATH`
/// stays a hard failure — locally fixable. An environment where
/// `chattr +i` itself fails — no `CAP_LINUX_IMMUTABLE` (GitHub's non-root
/// hosted runners) or a filesystem blind to `FS_IMMUTABLE_FL` (tmpfs,
/// overlayfs) — cannot arm this fixture at all and no hosted-runner user
/// can change that, so that shape is a LOUD probe-gated skip at the top,
/// before any daemon state exists. This is the only test covering the
/// `RetainedError` disposition; the skip prints itself rather than
/// quietly dropping the coverage, and the fault path still executes fully
/// everywhere the census and mutation probes run (root + ext4).
#[tokio::test]
#[cfg(target_os = "linux")]
async fn a_worktree_git_refuses_to_remove_is_retained_with_the_error_and_journaled() {
    // Probe the environment's immutable-bit support on a scratch dir before
    // building any daemon state. chattr-on-PATH failures stay hard below.
    {
        let probe = TempDir::new().expect("probe tempdir");
        let probe_dir = probe.path().join("immutable-probe");
        std::fs::create_dir(&probe_dir).expect("create probe dir");
        let armed = Command::new("chattr")
            .args(["+i", probe_dir.to_str().expect("utf8 path")])
            .status()
            .expect("chattr must be on PATH for this fixture (see the doc comment)");
        let _ = Command::new("chattr")
            .args(["-i", probe_dir.to_str().expect("utf8 path")])
            .status();
        if !armed.success() {
            eprintln!(
                "skipping: chattr +i failed on this environment (no \
                 CAP_LINUX_IMMUTABLE or an FS_IMMUTABLE_FL-blind filesystem \
                 — hosted-runner shape); the RetainedError fixture cannot be \
                 armed here — see the test's doc comment"
            );
            return;
        }
    }

    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, _fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();
    let (_, body) = submit(
        &client,
        &handle,
        &estate,
        "git cannot delete this worktree",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    let worktree = PathBuf::from(
        body["surface"]["bindings"][0]["worktree_path"]
            .as_str()
            .expect("worktree"),
    );

    // The worktree stays clean; only Git's own removal is blocked. The
    // guard is armed *before* the bit is set, so every path out of this
    // test from here on clears it (see `ClearImmutableOnDrop`); arming it
    // for a `chattr` that then fails costs one no-op `chattr -i`.
    let _immutable = ClearImmutableOnDrop(worktree.clone());
    // The fixture asserts an environment fact: this process may set
    // FS_IMMUTABLE_FL (CAP_LINUX_IMMUTABLE, honoured by the TMPDIR
    // filesystem). GitHub Actions' runner user lacks the capability
    // (measured 2026-08-11, CI run 31448175583: "Operation not permitted").
    // Where the fact does not hold the injection is inexpressible — skip
    // honestly rather than panicking over the host's capability set.
    let chattr = Command::new("chattr")
        .args(["+i", worktree.to_str().expect("utf8 path")])
        .status();
    match chattr {
        Err(_) => {
            eprintln!(
                "skipping: chattr is not on PATH; the immutable-bit fixture is not expressible here"
            );
            return;
        }
        Ok(status) if !status.success() => {
            eprintln!(
                "skipping: chattr +i refused (missing CAP_LINUX_IMMUTABLE or \
                 filesystem support); the immutable-bit fixture is not \
                 expressible here"
            );
            return;
        }
        Ok(_) => {}
    }

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
    assert_eq!(
        report["clean"], false,
        "a removal Git refuses is not a clean teardown"
    );
    assert_eq!(report["bindings"][0]["disposition"], "retained_error");
    assert!(
        report["bindings"][0]["detail"]
            .as_str()
            .is_some_and(|d| !d.is_empty()),
        "Git's own diagnostic must be recorded, not swallowed: {report}"
    );
    assert!(
        worktree.is_dir(),
        "a worktree Git refused to remove must survive teardown"
    );

    handle.shutdown().await;
    // `_immutable` drops here — after `shutdown`, before the `TempDir`s
    // declared above it — clearing the bit so their teardown can reclaim
    // the worktree.
}

/// Cancelling a work parked in `blocked` retires the stage it was parked in.
/// The stage coordinate is orthogonal to work state (§10), which is exactly
/// why it has to be retired explicitly: a canceled work whose stage still
/// reads `blocked` is a stage nothing will ever move again.
#[tokio::test]
async fn cancelling_a_blocked_work_retires_the_stage_it_was_parked_in() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, _fake) = one_fake([FakeStep::blocked("needs an architecture decision")]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
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
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, fake) = one_fake([
        FakeStep::needs_input("which database?"),
        FakeStep::complete(),
    ]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (_, body) = submit(
        &client,
        &handle,
        &estate,
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
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, _fake) = one_fake([FakeStep::needs_input("which database?")]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();
    let (_, body) = submit(
        &client,
        &handle,
        &estate,
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
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
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
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let registry = BackendRegistry::new().with(Arc::new(OpaqueBackend::ObserveFails));
    let handle = start_with(data.path(), registry, Some(OPAQUE_BACKEND), Some(&estate)).await;
    let client = http();

    let (status, body) = submit(
        &client,
        &handle,
        &estate,
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
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let registry = BackendRegistry::new().with(Arc::new(OpaqueBackend::ObserveUnknown));
    let handle = start_with(data.path(), registry, Some(OPAQUE_BACKEND), Some(&estate)).await;
    let client = http();

    let (status, body) = submit(
        &client,
        &handle,
        &estate,
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
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    // A perfectly loadable workflow that simply is not in the workflows
    // directory, so only the guard can be what refuses the submission. Both
    // it and the real workflows root hang off the estate root, since that is
    // the one directory §4.1 resolves workflows against.
    write_workflow(
        &estate.join("elsewhere"),
        "outside",
        &[("00-only", "context")],
    );
    std::fs::create_dir_all(estate.join(".sergeant/workflows")).expect("workflows root");

    let (registry, fake) = one_fake([]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (status, body) = submit(
        &client,
        &handle,
        &estate,
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
    let estate = repos.path().join("solo-estate");

    let data = TempDir::new().expect("tempdir");
    estate_with_manifest(
        &estate,
        &format!(
            "[estate]\nname = \"solo\"\n\n[[repo]]\nname = \"solo\"\n\n\
             [[profile]]\nname = \"enterprise\"\nbackend = \"{FAKE_BACKEND_NAME}\"\n\
             default_model = \"claude-opus-4-7\"\n\
             env = {{ CLAUDE_CONFIG_DIR = \"/tmp/work\", GIT_AUTHOR_NAME = \"sergeant\" }}\n"
        ),
        &["solo"],
    );
    let (registry, fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();
    let (status, body) = submit(
        &client,
        &handle,
        &estate,
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
        &estate,
        "unknown profile",
        json!({"profile": "nope"}),
    )
    .await;
    assert_eq!(status, 422);
    assert_eq!(body["error"]["code"], "profile_not_found");
    handle.shutdown().await;
}

/// §14 against §13: a profile that launches a different backend than routing
/// resolved to is refused, and the refusal names the tier that made the
/// choice.
///
/// The two ways this could quietly go wrong are the two §13 forbids: running
/// the profile's backend (silently overruling the route) or running the
/// routed backend with a profile configured for another (a permission mode
/// and a model pin meant for a different harness). Nothing pinned it: the
/// profile that exists in this suite agrees with its route, so the check
/// could be deleted and every test would still pass. Routing here is the
/// last tier — no explicit backend, no estate default — because that is
/// the tier a user is least likely to have in mind when naming a profile.
#[tokio::test]
async fn a_profile_that_names_another_backend_is_refused_with_the_tier_that_routed() {
    let repos = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");

    let data = TempDir::new().expect("tempdir");
    estate_with_manifest(
        &estate,
        "[estate]\nname = \"solo\"\n\n[[repo]]\nname = \"solo\"\n\n\
         [[profile]]\nname = \"elsewhere\"\nbackend = \"codex\"\n\
         default_model = \"gpt-nonexistent\"\n",
        &["solo"],
    );

    // No explicit backend and no estate default: the daemon's global default
    // routes this, which is tier four.
    let (registry, fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();
    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "a profile for a backend this work is not routed to",
        json!({"profile": "elsewhere"}),
    )
    .await;

    assert_eq!(status, 422, "the mismatch must be refused: {body}");
    assert_eq!(body["error"]["code"], "profile_backend_mismatch");
    let message = body["error"]["message"].as_str().expect("message");
    for named in ["elsewhere", "codex", FAKE_BACKEND_NAME, "global_default"] {
        assert!(
            message.contains(named),
            "the refusal must name {named:?} so the user can fix either side: {message}"
        );
    }

    // Refused before anything ran: no execution, and no work to inspect.
    assert!(
        fake.starts().is_empty(),
        "a refused submission must not launch the routed backend either, got {:?}",
        fake.starts()
    );
    let list = get(&client, &handle, "/v1/work").await;
    assert!(
        list["works"].as_array().expect("works").is_empty(),
        "nothing was accepted: {list}"
    );

    handle.shutdown().await;
}

/// A submission to a daemon bound to no estate is a *captured intent*: there
/// is nothing to materialize, which §5.1 answers definitely rather than
/// failing. But "nothing to materialize" is not "nothing to honour" — a
/// submission that names a backend sergeant cannot route to is refused with
/// §13's options, instead of being recorded as pending work carrying a
/// selection nothing will ever run.
///
/// This daemon deliberately starts with `estate_root: None`. That is now the
/// *only* way to reach the no-topology path: §5.2 gives `origin.cwd` no
/// authority, so a request can no longer talk a bound daemon out of its
/// estate by pointing somewhere else.
#[tokio::test]
async fn a_submission_with_no_workspace_is_captured_but_still_routed() {
    let data = TempDir::new().expect("tempdir");
    let (registry, fake) = one_fake([]);
    let handle = start_with(data.path(), registry, Some(FAKE_BACKEND_NAME), None).await;
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

    // A cwd this daemon's estate knows nothing about is the same answer, not
    // an error — the cwd was never what decided it.
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
        // Deliberately not the bare canonical name "opencode": since the
        // opencode-adapter sprint's W2 that name is registered for real, and
        // this sentinel only needs to stay permanently unregistrable, not to
        // demonstrate the unregistered-harness case (that's rows 1793/2045,
        // above, which do use the bare name and were swapped to "goose").
        json!({"backend": "opencode-does-not-exist"}),
    )
    .await;
    assert_eq!(
        status, 422,
        "an unhonourable selection must not be recorded as pending work: {body}"
    );
    assert_eq!(body["error"]["code"], "backend_not_found");
    // Since M4 the daemon registers the real claude adapter alongside the
    // scripted fake, since N4 the docker adapter too, since W2 the codex
    // adapter as well, since the opencode-adapter sprint's own W2 the
    // opencode adapter too, and since the agy-adapter sprint's own W2 the
    // agy adapter as well (registered names sort: agy, claude, codex,
    // docker, fake, opencode).
    assert_eq!(
        body["error"]["available_backends"],
        json!([
            "agy",
            "claude",
            "codex",
            "docker",
            FAKE_BACKEND_NAME,
            "opencode"
        ])
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

/// estate-root proposal §7.1: a multi-repository estate refuses an empty
/// scope with the full remedy body — repo count, declared repos, declared
/// groups, and the three example invocations — rather than silently
/// expanding to every repository.
#[tokio::test]
async fn a_multi_repo_estate_refuses_an_empty_scope_with_the_7_1_remedy() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("payments");
    estate_with_manifest(
        &estate,
        "[estate]\nname = \"payments\"\n\n\
         [[repo]]\nname = \"api\"\n\n\
         [[repo]]\nname = \"web\"\n\n\
         [group.pair]\nrepos = [\"api\", \"web\"]\n",
        &["api", "web"],
    );

    let (registry, fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (status, body) = submit(&client, &handle, &estate, "no scope at all", json!({})).await;
    assert_eq!(status, 422, "an empty scope must refuse: {body}");
    assert_eq!(body["error"]["code"], "missing_scope");
    assert_eq!(body["error"]["repo_count"], 2);
    assert_eq!(body["error"]["repos"], json!(["api", "web"]));
    assert_eq!(body["error"]["groups"], json!(["pair"]));
    assert!(
        body["error"]["examples"]["repo"].as_str().is_some(),
        "must offer a --repo example: {body}"
    );
    assert!(
        body["error"]["examples"]["group"]
            .as_str()
            .is_some_and(|s| s.contains("pair")),
        "must offer a --group example naming a real declared group: {body}"
    );
    assert_eq!(
        body["error"]["examples"]["all"],
        "sgt run \"<intent>\" --all"
    );
    let message = body["error"]["message"].as_str().expect("message");
    assert!(message.contains("2 repositories"), "got: {message}");
    assert!(
        fake.starts().is_empty(),
        "a missing-scope refusal must never reach a backend"
    );

    handle.shutdown().await;
}

/// estate-root proposal §7.1: `--all` is explicit and journaled as the
/// request form even though it resolves to the same list every declared
/// repository would — `scope_request.all` must be `true`, not merely a
/// side effect inferred from `repositories` matching every declared name.
#[tokio::test]
async fn run_all_resolves_every_repository_and_journals_the_request_form() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("payments");
    estate_with_manifest(
        &estate,
        "[estate]\nname = \"payments\"\n\n\
         [[repo]]\nname = \"api\"\n\n\
         [[repo]]\nname = \"web\"\n",
        &["api", "web"],
    );

    let (registry, _fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();

    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "everything, explicitly",
        json!({"scope": {"all": true}}),
    )
    .await;
    assert_eq!(status, 201, "submit failed: {body}");
    let mut repositories: Vec<String> = body["work"]["repositories"]
        .as_array()
        .expect("repositories")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    repositories.sort();
    assert_eq!(repositories, vec!["api", "web"]);
    assert_eq!(
        body["work"]["scope_request"],
        json!({"all": true}),
        "the request form (all: true) must be journaled, not just its resolution: {body}"
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
    let estate = repos.path().join("solo-estate");
    let (repo, _head) = support::scaffold_solo_estate(&estate, "solo");

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
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
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
///
/// The daemon starts on a *valid* manifest and each case rewrites it
/// underneath: §5.1 admits the estate root before the data dir is even
/// created, so a daemon started against a broken manifest would never come up
/// at all and there would be no submit to refuse. Re-reading the manifest at
/// every `plan` — the root is what startup pins, not the file's contents — is
/// what makes the edit visible here, and it is the same property `sgt repo
/// add` relies on to reach a running daemon.
#[tokio::test]
async fn a_malformed_workspace_file_fails_closed() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    let (repo, _head) = support::scaffold_solo_estate(&estate, "solo");
    let manifest = estate.join("sergeant.toml");

    let (registry, _fake) = one_fake([]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;

    std::fs::write(
        &manifest,
        "[estate]\nname = \"solo\"\ndefault_backends = \"fake\"\n\n[[repo]]\nname = \"solo\"\n",
    )
    .expect("sergeant.toml");
    let (status, body) = submit(&http(), &handle, &estate, "typo in config", json!({})).await;
    assert_eq!(status, 422, "a typo'd key must not be ignored: {body}");
    assert_eq!(body["error"]["code"], "workspace_error");
    assert!(
        body["error"]["message"]
            .as_str()
            .expect("message")
            .contains("default_backends"),
        "the diagnostic must name the unknown key: {body}"
    );

    // A declared repository with no mount at `repos/<name>` is refused too
    // (§6.1: the mount is derived, so "declared but absent" is the only shape
    // a missing repository can now take).
    std::fs::write(
        &manifest,
        "[estate]\nname = \"solo\"\n\n[[repo]]\nname = \"ghost\"\n",
    )
    .expect("sergeant.toml");
    let (status, body) = submit(&http(), &handle, &estate, "missing repo", json!({})).await;
    assert_eq!(status, 422);
    assert!(
        body["error"]["message"]
            .as_str()
            .expect("message")
            .contains("ghost")
    );

    // §6.1's removal notice, in the slot the same-path duplicate used to
    // occupy. Two entries can no longer collide on one checkout — a mount is
    // `<estate-root>/repos/<name>` and names are already unique — so the
    // refusal that still has to happen *before* anything is materialized is
    // the one for a manifest that has not migrated: a `path` key is named and
    // rejected outright, never quietly ignored while the derived mount is
    // used behind the operator's back.
    std::fs::write(
        &manifest,
        "[estate]\nname = \"solo\"\n\n\
         [[repo]]\nname = \"solo\"\npath = \".\"\n",
    )
    .expect("sergeant.toml");
    let (status, body) = submit(&http(), &handle, &estate, "a path key survives", json!({})).await;
    assert_eq!(status, 422, "a declared path must be refused: {body}");
    assert_eq!(body["error"]["code"], "workspace_error");
    let message = body["error"]["message"].as_str().expect("message");
    assert!(
        message.contains("solo") && message.contains("`path`"),
        "the diagnostic must name the entry and the removed key: {body}"
    );
    assert!(
        message.contains("repos/solo"),
        "and the derived mount the operator must move the checkout to: {body}"
    );
    // Nothing was created behind the refusal: no surface, no branch.
    assert_eq!(
        git(&repo, &["branch", "--list", "sergeant/*"]),
        "",
        "a statically rejectable submission must not touch the repository"
    );

    handle.shutdown().await;
}

// ------------------------------------------ #22: repository-topology edges
// beyond R-MVP1-12's own fixtures (`src/domain/estate.rs`'s `#22:`-tagged
// tests). These were the "remaining edges" `sergeant-rs-workspace's knowledge/evidence/gauntlet/contracts/MVP-1.md`
// named and deferred: one table-driven-in-spirit test per shape, through the
// real daemon/API, asserting the issue's own three things — correct binding
// record, work completes, teardown clean.
//
// Two of the four shapes changed sign under estate-root §6.2, which validates
// every derived mount: a linked worktree and a symlinked (aliased) checkout
// are now *refused* as repository mounts rather than bound from, so their
// tests pin the refusal and the surviving legitimate shape respectively. See
// each doc comment.

/// A repository with a submodule: the surface actually carries the
/// submodule's content (not the silent empty directory `git worktree add`
/// alone leaves — `src/runtime/surface.rs`'s own unit tests pin the
/// materialize/teardown mechanics; this is the same shape proven to reach the
/// daemon end to end), the work completes, and teardown removes the worktree
/// cleanly.
#[tokio::test]
async fn t9_a_repository_with_a_submodule_completes_and_tears_down_clean() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let inner = repos.path().join("vendor-inner");
    init_repo(&inner);
    std::fs::write(inner.join("vendored.txt"), "vendored content\n").expect("write");
    git(&inner, &["add", "vendored.txt"]);
    git(&inner, &["commit", "-m", "vendored payload"]);
    let inner_head = git(&inner, &["rev-parse", "HEAD"]);

    // The submodule's superproject is the estate's own mount (§6.1); the
    // vendored inner repository stays outside the estate, as an upstream URL
    // would be.
    let estate = repos.path().join("outer-estate");
    let (outer, _head) = support::scaffold_solo_estate(&estate, "outer");
    write_two_stage_workflow(&estate);
    std::fs::write(
        outer.join(".gitmodules"),
        format!(
            "[submodule \"vendored\"]\n\tpath = vendored\n\turl = {}\n",
            inner.display()
        ),
    )
    .expect(".gitmodules");
    std::fs::create_dir_all(outer.join("vendored")).expect("placeholder");
    git(&outer, &["add", ".gitmodules"]);
    git(
        &outer,
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            &inner_head,
            "vendored",
        ],
    );
    git(&outer, &["commit", "-m", "declare a submodule"]);

    // A hang keeps the surface in place so its content is inspectable, then
    // a cancel drives real teardown — the same shape t1/t6 already use.
    let (registry, fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();
    let (status, body) = submit(&client, &handle, &estate, "check the submodule", json!({})).await;
    assert_eq!(status, 201, "submit failed: {body}");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    let worktree = PathBuf::from(
        body["surface"]["bindings"][0]["worktree_path"]
            .as_str()
            .expect("worktree path"),
    );
    assert_eq!(
        std::fs::read_to_string(worktree.join("vendored").join("vendored.txt"))
            .expect("submodule content must be checked out through the daemon's own path"),
        "vendored content\n"
    );

    let (status, body) = post(
        &client,
        &handle,
        &format!("/v1/work/{work_id}/cancel"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, 200, "cancel failed: {body}");
    assert_eq!(body["work"]["state"], "canceled");
    let torn = events_of(data.path(), &work_id, KIND_SURFACE_TORN_DOWN);
    assert_eq!(torn.len(), 1);
    assert_eq!(
        torn[0].payload["report"]["clean"], true,
        "an untouched submodule worktree tears down clean: {:?}",
        torn[0].payload
    );
    assert!(!worktree.exists());
    assert!(branch_exists(&outer, &format!("sergeant/{work_id}")));
    let _ = fake.stop_requests();

    handle.shutdown().await;
}

/// §6.2/§8.1 check 4: a **linked worktree cannot be a repository mount**, and
/// the refusal names it before anything is materialized.
///
/// This pinned the opposite until estate-root landed: `repos/<name>` being a
/// `git worktree add` of some other repository used to bind and tear down
/// fine, because nothing in §11's materialize path cares whether
/// `repository.path` is a main checkout. §6.2 says that is precisely the
/// problem — a mount must own its own branches, common directory, worktree
/// registry and `sergeant/*` refs, and a linked worktree owns none of them,
/// so cutting `sergeant/<work-id>` there writes into a repository the estate
/// does not own. A Work's own surface *is* a linked worktree, which is the
/// recursion this closes.
///
/// Validation is git's own answer, not a path heuristic: the mount's
/// `--git-common-dir` resolves outside the mount, which is the fact the
/// diagnostic reports.
#[tokio::test]
async fn t9b_a_linked_worktree_as_a_repository_mount_is_refused_at_submit() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let main_repo = repos.path().join("main-repo");
    init_repo(&main_repo);

    // A structurally valid estate whose one mount is, at the derived path,
    // a linked worktree of a repository elsewhere.
    let estate = repos.path().join("solo-estate");
    let mount = estate.join("repos").join("solo");
    std::fs::create_dir_all(mount.parent().expect("repos dir")).expect("repos dir");
    git(
        &main_repo,
        &[
            "worktree",
            "add",
            mount.to_str().expect("utf8 path"),
            "-b",
            "a-worktree-of-its-own",
        ],
    );
    std::fs::write(
        estate.join("sergeant.toml"),
        "[estate]\nname = \"solo\"\n\n[[repo]]\nname = \"solo\"\n",
    )
    .expect("sergeant.toml");
    write_two_stage_workflow(&estate);

    // The daemon comes up: §4.1 admission is structural, and a repository
    // that cannot be resolved is a repository problem, not an estate-identity
    // one. The refusal therefore lands where a Work would have bound.
    let (registry, fake) = one_fake([
        FakeStep::complete_with("first done"),
        FakeStep::complete_with("second done"),
    ]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();
    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "bind from a linked worktree",
        json!({"workflow": "tiny"}),
    )
    .await;
    assert_eq!(status, 422, "a linked-worktree mount must refuse: {body}");
    assert_eq!(body["error"]["code"], "workspace_error");
    let message = body["error"]["message"].as_str().expect("message");
    assert!(
        message.contains("linked worktree"),
        "the diagnostic must name the shape, not just fail: {message}"
    );
    assert!(
        message.contains("git common dir"),
        "and the evidence git itself gave for it: {message}"
    );
    assert!(
        message.contains("repos/solo"),
        "and the mount the operator must clone into instead: {message}"
    );

    // Refused before any side effect: no execution, no work, and above all no
    // `sergeant/*` branch cut into the repository that actually owns the
    // common dir.
    assert!(
        fake.starts().is_empty(),
        "a mount refusal must never reach a backend"
    );
    let list = get(&client, &handle, "/v1/work").await;
    assert!(
        list["works"].as_array().expect("works").is_empty(),
        "a mount refusal must not create work: {list}"
    );
    assert_eq!(
        git(&main_repo, &["branch", "--list", "sergeant/*"]),
        "",
        "nothing may be cut into the repository the linked worktree belongs to"
    );
    assert!(
        !data.path().join("surfaces").exists(),
        "and no surface was started"
    );

    handle.shutdown().await;
}

/// A symlinked **estate root**: the path the daemon is bound to reaches the
/// estate through a symlink rather than its real path — an ordinary shape,
/// since the bound root is whatever the operator's shell says `cwd` is.
///
/// §4.1's admission canonicalizes before pinning, so every mount derived from
/// it, `materialize`'s in-checkout guard (`surface.rs`'s own doc on
/// `materialize`, which canonicalizes both sides for exactly this reason) and
/// teardown all work from the real path, and an unremarkable submission
/// completes exactly as it would from the real root. The binding assertion
/// below is what says so: `source_path` is the canonical mount, not the
/// symlinked spelling that was handed in.
///
/// The *other* symlink — a mount that is itself a symlink to a checkout
/// elsewhere — is the opposite ruling: §6.2 refuses it as an alias
/// (`RepositoryMountAliased`, pinned in `src/domain/estate.rs`), because
/// two estates aliasing one checkout is the shared-mount hazard it exists to
/// stop. Canonicalizing the root is not the same as following a mount.
#[cfg(unix)]
#[tokio::test]
async fn t9c_a_symlinked_estate_root_materializes_and_completes() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let real = repos.path().join("real-estate");
    let (mount, head) = support::scaffold_solo_estate(&real, "solo");
    write_two_stage_workflow(&real);
    let via_symlink = repos.path().join("estate-via-symlink");
    std::os::unix::fs::symlink(&real, &via_symlink).expect("symlink");

    let (registry, fake) = one_fake([
        FakeStep::complete_with("first done"),
        FakeStep::complete_with("second done"),
    ]);
    // Bound through the symlink, not the real path.
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&via_symlink),
    )
    .await;
    let client = http();
    let (status, body) = submit(
        &client,
        &handle,
        &via_symlink,
        "bind through a symlink",
        json!({"workflow": "tiny"}),
    )
    .await;
    assert_eq!(status, 201, "submit failed: {body}");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "completed");
    assert_eq!(fake.starts().len(), 2);

    let materialized = events_of(data.path(), &work_id, KIND_SURFACE_MATERIALIZED);
    let binding = &materialized[0].payload["surface"]["bindings"][0];
    assert_eq!(binding["base_sha"], head.as_str());
    assert_eq!(
        binding["source_path"].as_str().map(PathBuf::from),
        Some(std::fs::canonicalize(&mount).unwrap_or(mount.clone())),
        "the mount is derived from the *canonical* root, never the symlinked \
         spelling the daemon was started with"
    );
    let worktree = PathBuf::from(binding["worktree_path"].as_str().expect("worktree path"));
    assert!(
        !worktree.exists(),
        "teardown after completion removes the worktree"
    );
    let torn = events_of(data.path(), &work_id, KIND_SURFACE_TORN_DOWN);
    assert_eq!(torn[0].payload["report"]["clean"], true);
    assert!(branch_exists(&mount, &format!("sergeant/{work_id}")));

    handle.shutdown().await;
}

/// A path with a space (and a non-ASCII character) in the estate root's own
/// directory name — and therefore in every mount derived beneath it —
/// exercised through the real materialize/complete/teardown flow rather than
/// only admission (`src/domain/estate.rs`'s own `#22:`-tagged path-with-a-
/// space fixture covers `Estate::admit`; this is the same shape one level
/// further, through git worktree creation, a real backend execution `cwd`,
/// and teardown).
#[tokio::test]
async fn t9d_a_path_with_a_space_and_non_ascii_completes_and_tears_down() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("réstate with spaces");
    let (repo, head) = support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    let (registry, fake) = one_fake([
        FakeStep::complete_with("first done"),
        FakeStep::complete_with("second done"),
    ]);
    let handle = start_with(
        data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&estate),
    )
    .await;
    let client = http();
    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "a path with a space",
        json!({"workflow": "tiny"}),
    )
    .await;
    assert_eq!(status, 201, "submit failed: {body}");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "completed");
    assert_eq!(fake.starts()[0].context, "first stage context");

    let materialized = events_of(data.path(), &work_id, KIND_SURFACE_MATERIALIZED);
    let binding = &materialized[0].payload["surface"]["bindings"][0];
    assert_eq!(binding["base_sha"], head.as_str());
    let worktree = PathBuf::from(binding["worktree_path"].as_str().expect("worktree path"));
    assert!(!worktree.exists());
    let torn = events_of(data.path(), &work_id, KIND_SURFACE_TORN_DOWN);
    assert_eq!(torn[0].payload["report"]["clean"], true);
    assert!(branch_exists(&repo, &format!("sergeant/{work_id}")));

    handle.shutdown().await;
}

/// The CLI surface the contract adds, through the spawned binary: a run that
/// asks for input, `sgt respond`, `sgt work show` carrying stage and surface,
/// and `sgt retry`.
#[test]
fn cli_respond_and_retry_through_the_binary() {
    let repos = TempDir::new().expect("tempdir");
    let data = DataDir::new();
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");
    write_two_stage_workflow(&estate);

    // §4.3: every estate-scoped command is admitted against the process cwd
    // itself, so the binary runs from the estate root — a mount beneath it is
    // not an estate and no longer resolves to one.
    //
    // The default daemon registry's fake completes every stage, so this run
    // goes end to end without a scripted backend.
    let output = sgt(&estate, &data, &["--json", "run", "ship it"]);
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
    let output = sgt(&estate, &data, &["work", "show", &work_id]);
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
    let output = sgt(&estate, &data, &["respond", &work_id, "hello"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not needs_input"), "respond said: {stderr}");

    // Same for retry on a completed work.
    let output = sgt(&estate, &data, &["retry", &work_id]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("only failed, blocked or waiting"),
        "retry said: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    stop_daemon(data.path());
}

/// Scaffold an estate at `root` declaring three repositories (`api`, `web`,
/// `docs`) — mounted, per §6.1, at `repos/api`, `repos/web`, `repos/docs` —
/// and one group `pair` = `[api, web]`.
fn write_three_repo_estate_with_a_group(root: &Path) {
    estate_with_manifest(
        root,
        "[estate]\nname = \"payments\"\n\n\
         [[repo]]\nname = \"api\"\n\n\
         [[repo]]\nname = \"web\"\n\n\
         [[repo]]\nname = \"docs\"\n\n\
         [group.pair]\nrepos = [\"api\", \"web\"]\n",
        &["api", "web", "docs"],
    );
}

/// estate-root proposal §7.1/§7.2, through the real spawned binary: `--all`
/// forwards `scope.all` verbatim and the daemon resolves it to every
/// declared repository; `--group` combined with `--repo` forwards both and
/// the daemon unions them (dedup) — the CLI performs no expansion of its
/// own (§7.2's "a client surface adds usability, never functionality").
#[test]
fn cli_run_all_and_run_group_plus_repo_wire_shapes() {
    let repos = TempDir::new().expect("tempdir");
    let data = DataDir::new();
    let estate = repos.path().join("payments");
    write_three_repo_estate_with_a_group(&estate);

    let output = sgt(&estate, &data, &["--json", "run", "--all", "everything"]);
    assert!(
        output.status.success(),
        "sgt run --all failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let body: Value = serde_json::from_slice(&output.stdout).expect("run --json");
    let mut repositories: Vec<String> = body["work"]["repositories"]
        .as_array()
        .expect("repositories")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    repositories.sort();
    assert_eq!(repositories, vec!["api", "docs", "web"]);
    assert_eq!(body["work"]["scope_request"], json!({"all": true}));

    let output = sgt(
        &estate,
        &data,
        &[
            "--json", "run", "--group", "pair", "--repo", "docs", "union",
        ],
    );
    assert!(
        output.status.success(),
        "sgt run --group pair --repo docs failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let body: Value = serde_json::from_slice(&output.stdout).expect("run --json");
    let mut repositories: Vec<String> = body["work"]["repositories"]
        .as_array()
        .expect("repositories")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    repositories.sort();
    assert_eq!(repositories, vec!["api", "docs", "web"]);
    assert_eq!(
        body["work"]["scope_request"],
        json!({"repos": ["docs"], "group": "pair"}),
        "the request form must carry --repo and --group exactly as submitted, not their union"
    );

    stop_daemon(data.path());
}

/// estate-root proposal §7.4: `--estate` is gone from the CLI entirely —
/// clap refuses the flag itself, before any daemon could ever be asked
/// about it.
#[test]
fn cli_run_rejects_the_removed_workspace_flag() {
    let repos = TempDir::new().expect("tempdir");
    let data = DataDir::new();
    // A perfectly valid estate root, so the only thing that can refuse this
    // invocation is clap: §4.3 would otherwise admit the root and go on to
    // spawn a daemon.
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");

    let output = sgt(
        &estate,
        &data,
        &["run", "--estate", "payments", "do the thing"],
    );
    assert!(
        !output.status.success(),
        "--estate must be rejected, not silently accepted"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("workspace") || stderr.to_lowercase().contains("unexpected argument"),
        "clap's own diagnostic must name the flag: {stderr}"
    );
    assert!(
        data.daemon_pids().is_empty(),
        "a clap parse failure must never reach ensure_daemon: {stderr}"
    );
}

/// estate-root proposal §7.2: scope resolution is core-owned, so a
/// `scope.group` submitted straight to the API (bypassing the CLI entirely)
/// and the equivalent `--group` submitted through the real spawned binary
/// resolve to the identical repository list — proof that the CLI reaches
/// the same resolution rather than reimplementing it.
#[tokio::test]
async fn scope_resolution_is_identical_across_the_cli_and_a_direct_api_submission() {
    // Leg 1: a direct API submission, no CLI involved at all.
    let api_repos = TempDir::new().expect("tempdir");
    let api_data = TempDir::new().expect("tempdir");
    let api_root = api_repos.path().join("payments");
    write_three_repo_estate_with_a_group(&api_root);
    let (registry, _fake) = one_fake([FakeStep::hang()]);
    let handle = start_with(
        api_data.path(),
        registry,
        Some(FAKE_BACKEND_NAME),
        Some(&api_root),
    )
    .await;
    let client = http();
    let (status, api_body) = submit(
        &client,
        &handle,
        &api_root,
        "direct api scope.group",
        json!({"scope": {"group": "pair"}}),
    )
    .await;
    assert_eq!(status, 201, "direct API submit failed: {api_body}");
    let mut api_repositories: Vec<String> = api_body["work"]["repositories"]
        .as_array()
        .expect("repositories")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    api_repositories.sort();
    handle.shutdown().await;

    // Leg 2: the identical scope, through the real CLI binary.
    let cli_repos = TempDir::new().expect("tempdir");
    let cli_data = DataDir::new();
    let cli_root = cli_repos.path().join("payments");
    write_three_repo_estate_with_a_group(&cli_root);
    let output = sgt(
        &cli_root,
        &cli_data,
        &["--json", "run", "--group", "pair", "cli --group scope"],
    );
    assert!(
        output.status.success(),
        "sgt run --group pair failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let cli_body: Value = serde_json::from_slice(&output.stdout).expect("run --json");
    let mut cli_repositories: Vec<String> = cli_body["work"]["repositories"]
        .as_array()
        .expect("repositories")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    cli_repositories.sort();
    stop_daemon(cli_data.path());

    assert_eq!(
        api_repositories, cli_repositories,
        "the CLI's --group and a direct API scope.group submission must resolve identically"
    );
    assert_eq!(api_repositories, vec!["api", "web"]);
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
