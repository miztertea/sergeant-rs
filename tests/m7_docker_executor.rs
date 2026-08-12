//! N4 acceptance: the Docker executor (`kind = "execute"`), against a real
//! local Docker Engine (§22.7-§22.10).
//!
//! **Environment posture (CLAUDE.md's testing rules, `docs/environments/`).**
//! Every test in this file probe-gates on [`docker_unavailable`] and skips
//! loudly (`SKIPPED-ENV`) rather than failing hard when the host cannot
//! express the shape — Cerberus (`docs/environments/cerberus.md`) runs the
//! full matrix; a host with no Docker reachable does not fail these.
//!
//! Every test that creates a container cleans it up itself and additionally
//! asserts, via [`assert_containers_gone`]/[`assert_no_containers_for_work`],
//! that the exact containers it created no longer exist — the harness-side
//! half of §16.10's "no leaked owned containers" acceptance, independent of
//! whatever the adapter under test claims it did. Scoped per test rather
//! than a global sweep, because `cargo test`'s default parallelism runs
//! several of these tests against the same Docker Engine concurrently.

mod support;

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use serde_json::json;
use tempfile::TempDir;

use sergeant_rs::backend::docker::{DockerBackend, DockerConfig};
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::backend::{
    Backend, BackendError, BackendRegistry, PreparedExecution, StartRequest,
};
use sergeant_rs::daemon::{self, DaemonConfig};
use sergeant_rs::domain::workflow::{ExecuteSpec, NetworkPolicy, WorkspaceAccess};

/// A small, always-present image every measured environment can pull or
/// already has (`docs/environments/cerberus.md`'s Docker probes use
/// `alpine`). Kept as one named constant so a future environment that needs
/// a different probe image changes one line.
const PROBE_IMAGE: &str = "alpine:3";

/// Whether the local Docker Engine answers at all. Every test in this file
/// calls this first and returns early (with a loud, named skip) when it does
/// not — the CLAUDE.md/environments convention for a shape a hosted runner
/// may not be able to express, distinct from a locally-fixable precondition.
fn docker_unavailable() -> Option<&'static str> {
    match Command::new("docker").arg("version").output() {
        Ok(out) if out.status.success() => None,
        Ok(_) => Some("SKIPPED-ENV: `docker version` exited nonzero on this host"),
        Err(_) => Some("SKIPPED-ENV: no `docker` binary reachable on this host"),
    }
}

macro_rules! require_docker {
    () => {
        if let Some(reason) = docker_unavailable() {
            eprintln!("{reason}");
            return;
        }
    };
}

/// Assert the exact containers this test created (named by their execution
/// ids) are gone — the harness's own independent check of §16.10/§16.12's
/// "exact, never global" cleanup rule, run beside (never instead of) each
/// test's own assertions.
///
/// Scoped to the names this call is given, not "nothing sergeant-owned
/// exists anywhere" — `cargo test`'s default parallelism runs several of
/// this file's tests against the same Docker Engine at once, so a global
/// sweep would fail on another test's still-live container and prove
/// nothing about the one under test. Checking each named container
/// individually is exactly as strong a claim about *this* test's cleanup
/// and immune to that race.
fn assert_containers_gone(names: &[&str]) {
    for name in names {
        let output = Command::new("docker")
            .args(["inspect", name])
            .output()
            .expect("docker inspect");
        assert!(
            !output.status.success(),
            "container {name} should have been removed by stop()/cleanup but still exists"
        );
    }
}

/// [`assert_containers_gone`], scoped by `io.sergeant.work` label instead of
/// by name — for a test that never learns its execution's engine-minted id
/// directly (the mixed-workflow proof, driven through the real API).
fn assert_no_containers_for_work(work_id: &str) {
    let output = Command::new("docker")
        .args([
            "ps",
            "-a",
            "--filter",
            &format!("label=io.sergeant.work={work_id}"),
            "--format",
            "{{.Names}}",
        ])
        .output()
        .expect("docker ps");
    let names: Vec<&str> = std::str::from_utf8(&output.stdout)
        .expect("utf8")
        .lines()
        .filter(|s| !s.is_empty())
        .collect();
    assert!(
        names.is_empty(),
        "work {work_id}: sergeant-owned containers leaked: {names:?}"
    );
}

/// Force-remove a container by name, ignoring "already gone" — a test's own
/// belt-and-suspenders cleanup, never a substitute for the adapter's own
/// `stop()`, which is what each test actually exercises and asserts on.
fn force_remove(name: &str) {
    let _ = Command::new("docker").args(["rm", "-f", name]).output();
}

fn backend(data_dir: &Path) -> DockerBackend {
    DockerBackend::new(DockerConfig::new(data_dir)).expect("docker backend")
}

fn spec(command: Vec<&str>, access: WorkspaceAccess) -> ExecuteSpec {
    ExecuteSpec {
        image: PROBE_IMAGE.to_string(),
        command: command.into_iter().map(str::to_string).collect(),
        workdir: "/workspace".to_string(),
        workspace_access: access,
        network: NetworkPolicy::None,
        env: BTreeMap::new(),
    }
}

fn request(work_id: &str, execution_id: &str, cwd: &Path, exec: ExecuteSpec) -> StartRequest {
    StartRequest {
        work_id: work_id.to_string(),
        execution_id: execution_id.to_string(),
        stage_id: "10-validate".to_string(),
        attempt: 1,
        cwd: cwd.to_path_buf(),
        intent: "m7 contract test".to_string(),
        context: String::new(),
        model: None,
        profile: None,
        execute: Some(exec),
        instruction_policy: sergeant_rs::domain::workspace::InstructionPolicy::default(),
    }
}

fn launch(
    backend: &DockerBackend,
    prepared: &PreparedExecution,
) -> sergeant_rs::backend::ExecutionHandle {
    backend.launch(prepared).expect("launch")
}

// ---------------------------------------------------------- 1. exit mapping

/// §22.7 test 9 / §11.2/§15.2: exit 0 completes, nonzero fails — and the
/// mapping is mechanical (the container's stdout content never enters the
/// decision), never inferred from anything but the exit code.
#[test]
fn exit_zero_completes_and_nonzero_fails_with_captured_evidence() {
    require_docker!();
    let data = support::DataDir::new();
    let cwd = TempDir::new().expect("cwd");
    let backend = backend(data.path());

    for (execution_id, command, expect_completed) in [
        (
            "m7-exit0",
            vec!["sh", "-c", "echo this text must never be read; exit 0"],
            true,
        ),
        (
            "m7-exit1",
            vec![
                "sh",
                "-c",
                "echo this text must never be read either; exit 7",
            ],
            false,
        ),
    ] {
        let req = request(
            "w1",
            execution_id,
            cwd.path(),
            spec(command, WorkspaceAccess::ReadOnly),
        );
        let prepared = backend.prepare(&req).expect("prepare");
        let handle = launch(&backend, &prepared);

        let observation = wait_for_exit(&backend, &handle);
        use sergeant_rs::backend::BackendSignal;
        match (&observation.signal, expect_completed) {
            (BackendSignal::StageCompleted { summary }, true) => {
                let detail = summary.as_deref().unwrap_or_default();
                assert!(
                    detail.contains("\"exit_code\":0"),
                    "completed summary must carry exit_code 0: {detail}"
                );
            }
            (BackendSignal::Failed { reason }, false) => {
                assert!(
                    reason.contains("\"exit_code\":7"),
                    "failed reason must carry the real exit code: {reason}"
                );
            }
            other => panic!("execution {execution_id}: unexpected signal {other:?}"),
        }
        backend.stop(&handle).expect("stop").wait();
    }
    assert_containers_gone(&["sgt-m7-exit0", "sgt-m7-exit1"]);
}

// -------------------------------------------------------- 2. mount contract

/// §22.7 tests 3-4 / §16.6: `read_only` prevents a write inside the mount
/// (the container sees it fail); `read_write` permits it and the file is
/// visible back on the host afterward.
#[test]
fn workspace_access_governs_writes_both_ways() {
    require_docker!();
    let data = support::DataDir::new();
    let cwd = TempDir::new().expect("cwd");
    let backend = backend(data.path());

    // Read-only: the write must fail inside the container.
    let ro_req = request(
        "w2",
        "m7-ro",
        cwd.path(),
        spec(
            vec!["sh", "-c", "echo nope > /workspace/should-not-exist"],
            WorkspaceAccess::ReadOnly,
        ),
    );
    let prepared = backend.prepare(&ro_req).expect("prepare");
    let handle = launch(&backend, &prepared);
    let observation = wait_for_exit(&backend, &handle);
    use sergeant_rs::backend::BackendSignal;
    assert!(
        matches!(observation.signal, BackendSignal::Failed { .. }),
        "a write into a read_only mount must fail the stage: {:?}",
        observation.signal
    );
    backend.stop(&handle).expect("stop").wait();
    assert!(
        !cwd.path().join("should-not-exist").exists(),
        "a read_only mount must never let the container write to the host"
    );

    // Read-write: the write succeeds and lands on the host.
    let rw_req = request(
        "w2",
        "m7-rw",
        cwd.path(),
        spec(
            vec!["sh", "-c", "echo yes > /workspace/should-exist"],
            WorkspaceAccess::ReadWrite,
        ),
    );
    let prepared = backend.prepare(&rw_req).expect("prepare");
    let handle = launch(&backend, &prepared);
    let observation = wait_for_exit(&backend, &handle);
    assert!(
        matches!(observation.signal, BackendSignal::StageCompleted { .. }),
        "a write into a read_write mount must succeed: {:?}",
        observation.signal
    );
    backend.stop(&handle).expect("stop").wait();
    let written = std::fs::read_to_string(cwd.path().join("should-exist")).expect("host file");
    assert_eq!(written.trim(), "yes");

    assert_containers_gone(&["sgt-m7-ro", "sgt-m7-rw"]);
}

// ------------------------------------------------------------- 3. isolation

/// §22.7 test 5 / §16.7: `network = "none"` leaves the container with no
/// usable external network path.
#[test]
fn network_none_has_no_usable_external_path() {
    require_docker!();
    let data = support::DataDir::new();
    let cwd = TempDir::new().expect("cwd");
    let backend = backend(data.path());

    let req = request(
        "w3",
        "m7-net",
        cwd.path(),
        spec(
            vec![
                "sh",
                "-c",
                // A 2s-bounded attempt at outbound; alpine's busybox wget
                // exits nonzero on any connect/resolve failure.
                "wget -T 2 -q -O /dev/null http://1.1.1.1/ || exit 9",
            ],
            WorkspaceAccess::ReadOnly,
        ),
    );
    let prepared = backend.prepare(&req).expect("prepare");
    let handle = launch(&backend, &prepared);
    let observation = wait_for_exit(&backend, &handle);
    use sergeant_rs::backend::BackendSignal;
    assert!(
        matches!(observation.signal, BackendSignal::Failed { .. }),
        "with network=none there must be no usable path to reach anything external: {:?}",
        observation.signal
    );
    backend.stop(&handle).expect("stop").wait();
    assert_containers_gone(&["sgt-m7-net"]);
}

// ------------------------------------------------- 4. identity + collisions

/// §16.10: a deterministic name that already exists under different labels
/// is a collision — refused, never adopted or overwritten.
#[test]
fn a_name_collision_with_mismatched_labels_fails_closed() {
    require_docker!();
    let data = support::DataDir::new();
    let cwd = TempDir::new().expect("cwd");
    let backend = backend(data.path());
    let execution_id = "m7-collide";
    let name = format!("sgt-{execution_id}");

    // Pre-occupy the deterministic name with an unrelated, unlabeled
    // container.
    force_remove(&name);
    let status = Command::new("docker")
        .args(["create", "--name", &name, PROBE_IMAGE, "true"])
        .status()
        .expect("docker create foreign container");
    assert!(status.success());

    let req = request(
        "w4",
        execution_id,
        cwd.path(),
        spec(vec!["true"], WorkspaceAccess::ReadOnly),
    );
    let prepared = backend.prepare(&req).expect("prepare");
    let err = backend
        .launch(&prepared)
        .expect_err("must refuse a mismatched-label collision");
    assert!(matches!(err, BackendError::Failed { .. }));

    force_remove(&name);
    assert_containers_gone(&["sgt-m7-collide"]);
}

/// §16.11's recovery matrix, the shape this adapter's `resume` can prove
/// without a real daemon restart: a missing container is reported as
/// `UnknownExecution` — never fabricated as success or failure.
#[test]
fn resume_on_a_missing_container_fails_closed_as_unknown_execution() {
    require_docker!();
    let data = support::DataDir::new();
    let cwd = TempDir::new().expect("cwd");
    let backend = backend(data.path());
    let req = request(
        "w5",
        "m7-vanished",
        cwd.path(),
        spec(vec!["true"], WorkspaceAccess::ReadOnly),
    );
    let prepared = backend.prepare(&req).expect("prepare");
    let handle = launch(&backend, &prepared);
    // Wait for it to exit, then remove it entirely — out from under the
    // adapter's own bookkeeping, exactly the "started, missing" recovery row.
    let _ = wait_for_exit(&backend, &handle);
    force_remove(&handle.native_id.clone().unwrap());

    let resume_request = sergeant_rs::backend::ResumeRequest::new("w5", cwd.path());
    let err = backend
        .resume(&handle, &resume_request)
        .expect_err("a missing container must not be adopted or fabricated");
    assert!(matches!(err, BackendError::UnknownExecution { .. }));
    assert_containers_gone(&["sgt-m7-vanished"]);
}

// --------------------------------------------------------- 5. cancellation

/// §15.5/§22.7 test 10: INTERRUPT stops exactly the named container; a
/// second, unrelated container is untouched.
#[test]
fn interrupt_stops_only_the_named_container() {
    require_docker!();
    let data = support::DataDir::new();
    let cwd = TempDir::new().expect("cwd");
    let backend = backend(data.path());

    let long = request(
        "w6",
        "m7-cancel-me",
        cwd.path(),
        spec(vec!["sleep", "300"], WorkspaceAccess::ReadOnly),
    );
    let long_prepared = backend.prepare(&long).expect("prepare");
    let long_handle = launch(&backend, &long_prepared);

    let bystander = request(
        "w6",
        "m7-bystander",
        cwd.path(),
        spec(vec!["sleep", "300"], WorkspaceAccess::ReadOnly),
    );
    let bystander_prepared = backend.prepare(&bystander).expect("prepare");
    let bystander_handle = launch(&backend, &bystander_prepared);

    backend.interrupt(&long_handle).expect("interrupt").wait();
    let observation = wait_for_exit(&backend, &long_handle);
    assert_eq!(
        observation.native,
        sergeant_rs::backend::NativeState::Exited
    );

    let bystander_observation = backend
        .observe(&bystander_handle)
        .expect("observe bystander");
    assert_eq!(
        bystander_observation.native,
        sergeant_rs::backend::NativeState::Running,
        "interrupting one execution must not touch an unrelated one"
    );

    backend.stop(&long_handle).expect("stop").wait();
    backend
        .interrupt(&bystander_handle)
        .expect("interrupt bystander")
        .wait();
    let _ = wait_for_exit(&backend, &bystander_handle);
    backend.stop(&bystander_handle).expect("stop").wait();
    assert_containers_gone(&["sgt-m7-cancel-me", "sgt-m7-bystander"]);
}

// -------------------------------------------------------- 6. image pinning

/// §15.3/§22.7 test 8: the first successful resolution for a `(work, stage)`
/// pair is pinned; a second launch of the same pair uses the pin rather than
/// re-resolving — proven here by pointing the *second* launch's authored
/// image at a reference that cannot possibly resolve, and observing that it
/// still runs (because the pin, not the new bogus reference, decided).
#[test]
fn retry_uses_the_pinned_image_identity_not_a_fresh_resolution() {
    require_docker!();
    let data = support::DataDir::new();
    let cwd = TempDir::new().expect("cwd");
    let backend = backend(data.path());

    let first = request(
        "w7",
        "m7-pin-1",
        cwd.path(),
        spec(vec!["true"], WorkspaceAccess::ReadOnly),
    );
    let prepared = backend.prepare(&first).expect("prepare");
    let handle = launch(&backend, &prepared);
    let observation = wait_for_exit(&backend, &handle);
    use sergeant_rs::backend::BackendSignal;
    assert!(matches!(
        observation.signal,
        BackendSignal::StageCompleted { .. }
    ));
    backend.stop(&handle).expect("stop").wait();

    // Same (work_id, stage_id) — `request()` always uses stage_id
    // "10-validate" — but a different execution id (a fresh attempt) and an
    // authored image that cannot resolve on its own.
    let mut bogus_spec = spec(vec!["true"], WorkspaceAccess::ReadOnly);
    bogus_spec.image = "sergeant-test/this-image-does-not-exist:nope".to_string();
    let retry = request("w7", "m7-pin-2", cwd.path(), bogus_spec);
    let prepared = backend.prepare(&retry).expect("prepare");
    let handle = launch(&backend, &prepared);
    let observation = wait_for_exit(&backend, &handle);
    assert!(
        matches!(observation.signal, BackendSignal::StageCompleted { .. }),
        "a retry at the same (work, stage) must use the pinned identity, not the newly \
         authored (unresolvable) reference: {:?}",
        observation.signal
    );
    backend.stop(&handle).expect("stop").wait();
    assert_containers_gone(&["sgt-m7-pin-1", "sgt-m7-pin-2"]);
}

// -------------------------------------------- 7. §22.8 large-output budget

/// §22.8: peak RSS does not grow proportionally with a large capture. This
/// test process's own RSS is measured before and after streaming roughly
/// 256 MiB combined stdout/stderr through `DockerBackend::observe`'s capture
/// path (scaled down from the contract's 1 GiB to keep this test's own
/// runtime reasonable on every measured host; the mechanism being proven —
/// bounded-memory streaming into the blob store — has no size-dependent
/// branch, so 256 MiB and 1 GiB exercise the identical code path). The
/// budget itself stays the contract's own 64 MiB increment.
#[test]
fn large_captured_output_does_not_grow_this_process_proportionally() {
    require_docker!();
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("SKIPPED-ENV: RSS measurement via /proc/self/status is Linux-only");
        return;
    }
    #[cfg(target_os = "linux")]
    {
        let data = support::DataDir::new();
        let cwd = TempDir::new().expect("cwd");
        let backend = backend(data.path());

        const BYTES: u64 = 256 * 1024 * 1024;
        let req = request(
            "w8",
            "m7-large-output",
            cwd.path(),
            spec(
                vec![
                    "sh",
                    "-c",
                    &format!(
                        "yes 0123456789abcdef | head -c {BYTES} >&1; \
                         yes 0123456789abcdef | head -c {BYTES} >&2"
                    ),
                ],
                WorkspaceAccess::ReadOnly,
            ),
        );
        let prepared = backend.prepare(&req).expect("prepare");
        let handle = launch(&backend, &prepared);

        // Wait for the container to actually exit before measuring, so the
        // budget is charged entirely to the capture call below.
        loop {
            let info = std::process::Command::new("docker")
                .args([
                    "inspect",
                    "--format",
                    "{{.State.Running}}",
                    &handle.native_id.clone().unwrap(),
                ])
                .output()
                .expect("inspect");
            if String::from_utf8_lossy(&info.stdout).trim() == "false" {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }

        let before = vm_rss_kb();
        let observation = backend
            .observe(&handle)
            .expect("observe (captures on exit)");
        let after = vm_rss_kb();

        use sergeant_rs::backend::BackendSignal;
        let detail = match &observation.signal {
            BackendSignal::StageCompleted { summary } => summary.clone().unwrap_or_default(),
            other => panic!("expected completion, got {other:?}"),
        };
        assert!(
            detail.contains(&format!("\"stdout_bytes\":{BYTES}")),
            "the full byte count must be recorded even though memory stayed bounded: {detail}"
        );

        backend.stop(&handle).expect("stop").wait();
        assert_containers_gone(&["sgt-m7-large-output"]);

        let increment_kb = after.saturating_sub(before);
        const BUDGET_KB: u64 = 64 * 1024; // §22.8's 64 MiB
        assert!(
            increment_kb <= BUDGET_KB,
            "peak RSS increment while capturing {BYTES} bytes was {increment_kb} KiB, over the \
             {BUDGET_KB} KiB (64 MiB) budget — the streaming capture path must not buffer the \
             whole log in memory"
        );
    }
}

#[cfg(target_os = "linux")]
fn vm_rss_kb() -> u64 {
    let status = std::fs::read_to_string("/proc/self/status").expect("read /proc/self/status");
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            return rest
                .trim()
                .trim_end_matches(" kB")
                .trim()
                .parse()
                .unwrap_or(0);
        }
    }
    0
}

// --------------------------------------- 8. doctor's lifecycle probe (§16.3)

#[test]
fn lifecycle_probe_proves_the_real_bind_mount_round_trip() {
    require_docker!();
    let data = support::DataDir::new();
    let backend = backend(data.path());
    let probe = backend.lifecycle_probe(PROBE_IMAGE);
    assert!(
        probe.available,
        "lifecycle probe must succeed on a working Docker install: {probe:?}"
    );
    assert!(probe.bind_mount);
    // No name-scoped leak check here: `lifecycle_probe` mints its own
    // `sgt-probe-<ulid>` name internally and removes it unconditionally
    // right after `docker run` returns (`DockerBackend::lifecycle_probe`),
    // before this call even gets its result back — there is no id this test
    // could check that a global sweep (unsafe under `cargo test`'s default
    // parallelism, see the module docs) would not also need.
}

// ------------------------------------- 9. the mixed actor→execute→actor proof

/// N4's Outcome and §21.5's gate, verbatim: "the first mixed workflow should
/// contain actor → execute → actor and prove that output evidence is
/// available to the following actor without Sergeant interpreting it."
///
/// Driven through the real daemon/API (fake actor, real Docker), not called
/// on the backends directly, so this also exercises `Engine::bind_stages`'
/// execute-stage routing and the ordinary stage-advance cascade end to end.
/// The execute stage writes an artifact into the worktree; the closing actor
/// stage's completion is scripted independently of that artifact's
/// content — sergeant never reads it to decide anything, exactly as §11.2
/// requires. This test's own assertion on the file *is* the "available to
/// the following actor" half of the proof: a real actor's harness would read
/// the same worktree path this assertion reads.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn mixed_actor_execute_actor_workflow_completes_with_evidence_handed_forward() {
    require_docker!();
    let data = support::DataDir::new();
    let repo = TempDir::new().expect("repo");
    init_repo(repo.path());

    let workflow_dir = repo.path().join(".sergeant/workflows/mixed-proof");
    std::fs::create_dir_all(workflow_dir.join("00-prepare")).expect("stage dir");
    std::fs::create_dir_all(workflow_dir.join("10-validate")).expect("stage dir");
    std::fs::create_dir_all(workflow_dir.join("20-close")).expect("stage dir");
    std::fs::write(workflow_dir.join("00-prepare/CONTEXT.md"), "prepare").expect("context");
    std::fs::write(workflow_dir.join("20-close/CONTEXT.md"), "close").expect("context");
    std::fs::write(
        workflow_dir.join("workflow.toml"),
        concat!(
            "[workflow]\n",
            "name = \"mixed-proof\"\n",
            "version = \"1\"\n",
            "stages = [\"00-prepare\", \"10-validate\", \"20-close\"]\n",
            "\n",
            "[stage.\"10-validate\"]\n",
            "kind = \"execute\"\n",
            "image = \"alpine:3\"\n",
            "command = [\"sh\", \"-c\", \"echo container-produced-evidence > /workspace/validated.txt\"]\n",
            "workdir = \"/workspace\"\n",
            "workspace_access = \"read_write\"\n",
            "network = \"none\"\n",
        ),
    )
    .expect("workflow.toml");

    let fake = Arc::new(FakeBackend::scripted(
        FAKE_BACKEND_NAME,
        [FakeStep::complete(), FakeStep::complete()],
    ));
    let handle = daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(BackendRegistry::new().with(fake)),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            docker: Some(DockerConfig::new(data.path())),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");

    let http = reqwest::Client::new();
    let submitted: serde_json::Value = http
        .post(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&json!({
            "command_id": ulid::Ulid::generate().to_string(),
            "intent": "prove actor -> execute -> actor",
            "workflow": "mixed-proof",
            "origin": {"client": "cli", "cwd": repo.path()},
        }))
        .send()
        .await
        .expect("submit")
        .json()
        .await
        .expect("json");
    let work_id = submitted["work"]["id"]
        .as_str()
        .expect("work id")
        .to_string();

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    let last: serde_json::Value;
    loop {
        let polled: serde_json::Value = http
            .get(format!("{}/v1/work/{work_id}", handle.endpoint))
            .bearer_auth(&handle.token)
            .send()
            .await
            .expect("show")
            .json()
            .await
            .expect("json");
        if polled["work"]["state"] == "completed" || polled["work"]["state"] == "blocked" {
            last = polled;
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "work did not settle within budget: {polled}"
        );
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    }

    assert_eq!(
        last["work"]["state"], "completed",
        "the mixed workflow must complete end to end: {last}"
    );

    // §12: sergeant never interprets the container's output — but the
    // *worktree* is real, and the closing actor (a real one, in production)
    // would read exactly this file. This is the "output evidence available
    // to the following actor" half of the N4 proof.
    let branch_worktrees: Vec<_> = std::fs::read_dir(data.path().join("surfaces"))
        .into_iter()
        .flatten()
        .flatten()
        .collect();
    let found = branch_worktrees
        .iter()
        .any(|entry| walk_for_marker(&entry.path()));
    assert!(
        found,
        "the execute stage's artifact (validated.txt) must exist somewhere under the \
         materialized surface for {work_id}"
    );

    handle.shutdown().await;
    assert_no_containers_for_work(&work_id);
}

fn walk_for_marker(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if walk_for_marker(&path) {
                return true;
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some("validated.txt")
            && let Ok(content) = std::fs::read_to_string(&path)
        {
            return content.trim() == "container-produced-evidence";
        }
    }
    false
}

fn init_repo(path: &Path) {
    let git = |args: &[&str]| {
        let output = Command::new("git")
            .args(args)
            .current_dir(path)
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
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    std::fs::create_dir_all(path).expect("repo dir");
    git(&["init", "-b", "main"]);
    std::fs::write(path.join("README.md"), "# fixture\n").expect("write file");
    git(&["add", "."]);
    git(&["commit", "-m", "initial"]);
}

/// Poll OBSERVE until the container has exited, panicking on timeout.
fn wait_for_exit(
    backend: &DockerBackend,
    handle: &sergeant_rs::backend::ExecutionHandle,
) -> sergeant_rs::backend::Observation {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        let observation = backend.observe(handle).expect("observe");
        if observation.native == sergeant_rs::backend::NativeState::Exited {
            return observation;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "container for execution {:?} did not exit within budget",
            handle.execution_id
        );
        std::thread::sleep(std::time::Duration::from_millis(150));
    }
}
