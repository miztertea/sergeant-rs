//! N4 acceptance: the Docker executor (`kind = "execute"`), against a real
//! local Docker Engine (§22.7-§22.10).
//!
//! **Environment posture (docs/DEVELOPMENT.md's testing rules, `docs/environments/`).**
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

// `docker` (the bare module path) is only used by the RSS/disk-pressure
// measurement in `large_captured_output_does_not_grow_this_process_proportionally`,
// which is itself Linux-only (`/proc/self/status`) — gated separately so a
// non-Linux build (first measured on macOS, 2026-08-15) doesn't trip
// `cargo clippy --all-targets -- -D warnings`'s `unused_imports` lint.
#[cfg(target_os = "linux")]
use sergeant_rs::backend::docker;
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
const PROBE_IMAGE: &str = "alpine:3.24";

/// Whether the local Docker Engine answers at all. Every test in this file
/// calls this first and returns early (with a loud, named skip) when it does
/// not — the docs/DEVELOPMENT.md/environments convention for a shape a hosted runner
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

/// A per-test-*run*-unique execution id built from a human-readable base
/// (#91). `DockerBackend::container_name` derives the container name
/// deterministically from `execution_id` (`sgt-<execution_id>`) — that
/// determinism is itself under contract
/// (`container_name_is_deterministic_from_execution_id` in
/// `src/backend/docker.rs`) and must not change, since resume/retry rely on
/// it (§16.10). So this fixes the name collision on the *test* side instead:
/// mixing a fresh ULID into every fixture's execution id makes the resulting
/// container name unique to this run, so a container leaked by an
/// interrupted prior run — or one a concurrent suite run on the same host is
/// using right now — can never collide with this run's name again.
fn unique_id(base: &str) -> String {
    format!("{base}-{}", ulid::Ulid::generate())
}

/// Failure-proof belt-and-suspenders cleanup for fixture containers (#91).
///
/// Every test also asserts cleanup explicitly via [`assert_containers_gone`]
/// right after its own `stop()`/`wait()` — that assertion is what actually
/// pins the no-leak contract, and stays exactly as strict as before. This
/// guard is the other half: its `Drop` runs during unwinding too (Rust runs
/// `Drop` while a panic unwinds the stack), so a fixture that panics
/// *before* reaching `stop()` — as every test in this file's assertions can
/// — still gets its container force-removed instead of leaking it forever.
/// Construct it as soon as a container's name is known and before anything
/// that can panic; on the happy path its `Drop` only fires after the
/// function's own `assert_containers_gone` call already ran (locals drop at
/// the end of their scope, after the statements that precede them), so it
/// never weakens that check into a no-op.
struct ContainerGuard(Vec<String>);

impl ContainerGuard {
    fn new<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self(names.into_iter().map(Into::into).collect())
    }
}

impl Drop for ContainerGuard {
    fn drop(&mut self) {
        for name in &self.0 {
            force_remove(name);
        }
    }
}

fn backend(data_dir: &Path) -> DockerBackend {
    DockerBackend::new(DockerConfig::new(data_dir)).expect("docker backend")
}

fn spec(command: Vec<&str>, access: WorkspaceAccess) -> ExecuteSpec {
    ExecuteSpec {
        image: PROBE_IMAGE.to_string(),
        command: command.into_iter().map(str::to_string).collect(),
        workdir: "/estate".to_string(),
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
        instruction_policy: sergeant_rs::domain::estate::InstructionPolicy::default(),
        bindings: Vec::new(),
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

    let mut names = Vec::new();
    for (base_id, command, expect_completed, expect_exit) in [
        (
            "m7-exit0",
            vec!["sh", "-c", "echo this text must never be read; exit 0"],
            true,
            0,
        ),
        (
            "m7-exit1",
            vec![
                "sh",
                "-c",
                "echo this text must never be read either; exit 7",
            ],
            false,
            7,
        ),
        // TH-13 (MVP-2 D3 fixer pass): the two cases above pair neutral
        // stdout with a matching exit code, so they cannot distinguish "the
        // mapping is mechanical, from the exit code alone" from "an
        // implementation that also scrapes stdout for a success/failure
        // token and happens to agree with the exit code here". These two
        // adversarially disagree: stdout says the opposite of what the exit
        // code says, so only a pure exit-code mapping gets both right.
        (
            "m7-exit-adversarial-ok",
            vec!["sh", "-c", "echo ERROR FAILED; exit 0"],
            true,
            0,
        ),
        (
            "m7-exit-adversarial-fail",
            vec!["sh", "-c", "echo OK SUCCESS; exit 3"],
            false,
            3,
        ),
    ] {
        let execution_id = unique_id(base_id);
        let name = format!("sgt-{execution_id}");
        let _guard = ContainerGuard::new([name.clone()]);
        names.push(name);

        let req = request(
            "w1",
            &execution_id,
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
                    detail.contains(&format!("\"exit_code\":{expect_exit}")),
                    "completed summary must carry exit_code {expect_exit}: {detail}"
                );
            }
            (BackendSignal::Failed { reason }, false) => {
                assert!(
                    reason.contains(&format!("\"exit_code\":{expect_exit}")),
                    "failed reason must carry the real exit code {expect_exit}: {reason}"
                );
            }
            other => panic!("execution {execution_id}: unexpected signal {other:?}"),
        }
        backend.stop(&handle).expect("stop").wait();
    }
    let name_refs: Vec<&str> = names.iter().map(String::as_str).collect();
    assert_containers_gone(&name_refs);
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
    let ro_execution_id = unique_id("m7-ro");
    let ro_name = format!("sgt-{ro_execution_id}");
    let _ro_guard = ContainerGuard::new([ro_name.clone()]);
    let ro_req = request(
        "w2",
        &ro_execution_id,
        cwd.path(),
        spec(
            vec!["sh", "-c", "echo nope > /estate/should-not-exist"],
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
    let rw_execution_id = unique_id("m7-rw");
    let rw_name = format!("sgt-{rw_execution_id}");
    let _rw_guard = ContainerGuard::new([rw_name.clone()]);
    let rw_req = request(
        "w2",
        &rw_execution_id,
        cwd.path(),
        spec(
            vec!["sh", "-c", "echo yes > /estate/should-exist"],
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

    assert_containers_gone(&[&ro_name, &rw_name]);
}

/// §22.7 test 6 (INV-R1-07's named coverage gap, MVP-2 D3 fixer pass): the
/// negative isolation posture `create_container`'s own comment claims
/// (§16.7 — "no privilege, no host namespaces, no extra capabilities, no
/// host devices, and the Docker socket is never mounted in") had never
/// actually been inspected on a real container; every other test only
/// checks the *positive* behavior (a write does or doesn't land). Inspects
/// the real container Docker created and asserts the negative claims
/// directly: exactly one mount (the estate bind, nothing else), not
/// privileged, no added capabilities, no devices.
#[test]
fn a_launched_container_carries_no_isolation_escape_hatches() {
    require_docker!();
    let data = support::DataDir::new();
    let cwd = TempDir::new().expect("cwd");
    let backend = backend(data.path());

    let execution_id = unique_id("m7-isolation");
    let name = format!("sgt-{execution_id}");
    let _guard = ContainerGuard::new([name.clone()]);
    let req = request(
        "w2c",
        &execution_id,
        cwd.path(),
        spec(vec!["sleep", "60"], WorkspaceAccess::ReadOnly),
    );
    let prepared = backend.prepare(&req).expect("prepare");
    let handle = launch(&backend, &prepared);

    let output = Command::new("docker")
        .args(["inspect", &name])
        .output()
        .expect("docker inspect");
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).expect("inspect json");
    let info = &parsed[0];

    let mounts = info["Mounts"].as_array().expect("Mounts array");
    assert_eq!(
        mounts.len(),
        1,
        "exactly one mount (the estate bind), nothing else — no Docker socket, no extra \
         host paths: {mounts:?}"
    );
    assert_eq!(mounts[0]["Destination"], "/estate");
    assert!(
        mounts.iter().all(|m| m["Source"]
            .as_str()
            .is_some_and(|s| !s.contains("docker.sock"))),
        "the Docker socket must never be mounted into an execute-stage container: {mounts:?}"
    );
    assert_eq!(
        info["HostConfig"]["Privileged"], false,
        "an execute-stage container must never run --privileged"
    );
    let cap_add = info["HostConfig"]["CapAdd"].as_array();
    assert!(
        cap_add.is_none_or(|c| c.is_empty()),
        "no extra Linux capabilities must be added: {cap_add:?}"
    );
    let devices = info["HostConfig"]["Devices"].as_array();
    assert!(
        devices.is_none_or(|d| d.is_empty()),
        "no host devices must be granted: {devices:?}"
    );

    backend.interrupt(&handle).expect("interrupt").wait();
    let _ = wait_for_exit(&backend, &handle);
    backend.stop(&handle).expect("stop").wait();
    assert_containers_gone(&[&name]);
}

/// §22.7 test 2 (INV-R1-07's named coverage gap, MVP-2 D3 fixer pass): a
/// worktree path containing a space must bind-mount and round-trip a write
/// correctly — spaces are safe by construction (`Command`'s args are passed
/// via `exec`, never a shell), unlike `,`/`=` which really do collide with
/// `--mount`'s CSV grammar (INV-R1-12, covered separately by a unit test
/// that does not need this space-specific positive proof).
#[test]
fn a_mount_path_containing_a_space_round_trips_correctly() {
    require_docker!();
    let data = support::DataDir::new();
    let cwd_root = TempDir::new().expect("cwd root");
    let cwd = cwd_root.path().join("has a space in it");
    std::fs::create_dir_all(&cwd).expect("mkdir with a space");
    let backend = backend(data.path());

    let execution_id = unique_id("m7-space");
    let name = format!("sgt-{execution_id}");
    let _guard = ContainerGuard::new([name.clone()]);
    let req = request(
        "w2b",
        &execution_id,
        &cwd,
        spec(
            vec!["sh", "-c", "echo yes > /estate/should-exist"],
            WorkspaceAccess::ReadWrite,
        ),
    );
    let prepared = backend.prepare(&req).expect("prepare");
    let handle = launch(&backend, &prepared);
    let observation = wait_for_exit(&backend, &handle);
    use sergeant_rs::backend::BackendSignal;
    assert!(
        matches!(observation.signal, BackendSignal::StageCompleted { .. }),
        "a bind mount whose host path contains a space must still work: {:?}",
        observation.signal
    );
    backend.stop(&handle).expect("stop").wait();
    let written = std::fs::read_to_string(cwd.join("should-exist")).expect("host file");
    assert_eq!(written.trim(), "yes");
    assert_containers_gone(&[&name]);
}

// ------------------------------------------------------------- 3. isolation

/// §22.7 test 5 / §16.7: `network = "none"` leaves the container with no
/// usable external network path.
#[test]
fn network_none_has_no_usable_external_path() {
    require_docker!();

    // TH-12 (MVP-2 D3 fixer pass): this test only discriminates on a host
    // with working egress — on an egress-blocked host (the cloud
    // container's measured posture, `docs/environments/`) the isolated
    // container's `wget` fails for the same reason a *non*-isolated one
    // would, and the assertion below is a guaranteed false green that
    // proves nothing about `--network none`. A positive control: the exact
    // same command, in a container with the ordinary default network,
    // must actually reach out — if it cannot, this host cannot discriminate
    // this test's claim at all, and the honest response is a loud skip, not
    // a pass that means nothing.
    let control = Command::new("docker")
        .args([
            "run",
            "--rm",
            PROBE_IMAGE,
            "sh",
            "-c",
            "wget -T 2 -q -O /dev/null http://1.1.1.1/ || exit 9",
        ])
        .status()
        .expect("docker run (positive control)");
    if !control.success() {
        eprintln!(
            "SKIPPED-ENV: this host has no outbound egress even without --network none (the \
             positive control itself failed), so network_none_has_no_usable_external_path \
             cannot discriminate its claim here"
        );
        return;
    }

    let data = support::DataDir::new();
    let cwd = TempDir::new().expect("cwd");
    let backend = backend(data.path());

    let execution_id = unique_id("m7-net");
    let name = format!("sgt-{execution_id}");
    let _guard = ContainerGuard::new([name.clone()]);
    let req = request(
        "w3",
        &execution_id,
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
    assert_containers_gone(&[&name]);
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
    let execution_id = unique_id("m7-collide");
    let name = format!("sgt-{execution_id}");
    let _guard = ContainerGuard::new([name.clone()]);

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
        &execution_id,
        cwd.path(),
        spec(vec!["true"], WorkspaceAccess::ReadOnly),
    );
    let prepared = backend.prepare(&req).expect("prepare");
    let err = backend
        .launch(&prepared)
        .expect_err("must refuse a mismatched-label collision");
    assert!(matches!(err, BackendError::Failed { .. }));

    force_remove(&name);
    assert_containers_gone(&[&name]);
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
    let execution_id = unique_id("m7-vanished");
    let name = format!("sgt-{execution_id}");
    let _guard = ContainerGuard::new([name.clone()]);
    let req = request(
        "w5",
        &execution_id,
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
    assert_containers_gone(&[&name]);
}

/// COMPOSITION PROBE C2 survivor (m7 fixer panel): §16.10's identity check
/// exists in `observe()` (`if !Self::labeled_for(...) { return Err(...) }`)
/// but no existing test ever drove OBSERVE against a container that holds
/// the deterministic name under different labels — every collision test
/// exercised LAUNCH's own refusal instead, so deleting OBSERVE's check
/// survived the whole suite. This exercises OBSERVE directly against a
/// hand-built handle for a name a *foreign* (unlabeled) container already
/// occupies — the shape a container recycled/replaced out from under the
/// adapter between LAUNCH and OBSERVE would produce.
#[test]
fn observe_on_a_foreign_container_under_the_deterministic_name_fails_closed() {
    require_docker!();
    let execution_id = unique_id("m7-observe-collide");
    let name = format!("sgt-{execution_id}");
    let _guard = ContainerGuard::new([name.clone()]);
    force_remove(&name);
    let status = Command::new("docker")
        .args(["create", "--name", &name, PROBE_IMAGE, "true"])
        .status()
        .expect("docker create foreign container");
    assert!(status.success());

    let data = support::DataDir::new();
    let backend = backend(data.path());
    let handle = sergeant_rs::backend::ExecutionHandle {
        execution_id: execution_id.to_string(),
        native_id: Some(name.clone()),
    };
    let err = backend
        .observe(&handle)
        .expect_err("observe must not report on a container it does not own");
    assert!(matches!(err, BackendError::UnknownExecution { .. }));

    force_remove(&name);
    assert_containers_gone(&[&name]);
}

/// TH-09 survivor (m7 fixer panel): `resume: true` is an advertised
/// capability, but the only existing resume test exercised its negative row
/// (a missing container). This exercises the two rows that were untested:
/// a live, correctly-labeled container is adopted (`Ok(())`), and a
/// foreign container under the deterministic name is refused rather than
/// adopted — the exact shape L8 is about (an advertised verb whose success
/// path was never driven against the installed harness).
#[test]
fn resume_adopts_a_live_labeled_container_and_refuses_a_foreign_one() {
    require_docker!();
    let data = support::DataDir::new();
    let cwd = TempDir::new().expect("cwd");
    let backend = backend(data.path());

    let live_execution_id = unique_id("m7-resume-live");
    let live_name = format!("sgt-{live_execution_id}");
    let _live_guard = ContainerGuard::new([live_name.clone()]);
    let req = request(
        "w4c",
        &live_execution_id,
        cwd.path(),
        spec(vec!["sleep", "60"], WorkspaceAccess::ReadOnly),
    );
    let prepared = backend.prepare(&req).expect("prepare");
    let handle = launch(&backend, &prepared);

    // Simulate a restart: a freshly built `ResumeRequest`/handle pair, as a
    // new daemon process re-deriving the deterministic name would build,
    // rather than anything carried over in memory (`DockerBackend` holds no
    // per-execution state to lose in the first place).
    let resume_request = sergeant_rs::backend::ResumeRequest::new("w4c", cwd.path());
    backend
        .resume(&handle, &resume_request)
        .expect("a live, correctly-labeled container must be re-adopted");

    backend.interrupt(&handle).expect("interrupt").wait();
    let _ = wait_for_exit(&backend, &handle);
    backend.stop(&handle).expect("stop").wait();
    assert_containers_gone(&[&live_name]);

    // The refusal row: a foreign container under the deterministic name.
    let execution_id = unique_id("m7-resume-foreign");
    let name = format!("sgt-{execution_id}");
    let _guard = ContainerGuard::new([name.clone()]);
    force_remove(&name);
    let status = Command::new("docker")
        .args(["create", "--name", &name, PROBE_IMAGE, "true"])
        .status()
        .expect("docker create foreign container");
    assert!(status.success());
    let foreign_handle = sergeant_rs::backend::ExecutionHandle {
        execution_id: execution_id.to_string(),
        native_id: Some(name.clone()),
    };
    let err = backend
        .resume(&foreign_handle, &resume_request)
        .expect_err("a foreign container must never be adopted");
    assert!(matches!(err, BackendError::Failed { .. }));
    force_remove(&name);
    assert_containers_gone(&[&name]);
}

// ---------------------------------------- 4b. §22.6 STOP's deferred removal

/// INV-R1-01 (MVP-2 D3 fixer pass, §22.6): `DockerBackend::stop` must not
/// make the Docker Engine call synchronously — the removal is the
/// `Completion`'s deferred tail work. Proven against a real container: the
/// container still exists immediately after `stop()` returns, and is gone
/// only once `.wait()` runs. `docker.rs`'s own unit test
/// (`stop_touches_no_docker_engine_call_synchronously_and_defers_the_removal`)
/// covers the "no call at all" half without Docker; this covers the "the
/// call really does happen, and only on wait" half, which needs a real
/// container to observe.
#[test]
fn stop_defers_the_actual_removal_until_the_completion_is_waited_on() {
    require_docker!();
    let data = support::DataDir::new();
    let cwd = TempDir::new().expect("cwd");
    let backend = backend(data.path());

    let execution_id = unique_id("m7-deferred-stop");
    let name = format!("sgt-{execution_id}");
    let _guard = ContainerGuard::new([name.clone()]);
    let req = request(
        "w4b",
        &execution_id,
        cwd.path(),
        spec(vec!["true"], WorkspaceAccess::ReadOnly),
    );
    let prepared = backend.prepare(&req).expect("prepare");
    let handle = launch(&backend, &prepared);
    let _ = wait_for_exit(&backend, &handle);

    let completion = backend.stop(&handle).expect("stop");
    // The container must still be there: `stop()` returning is not the
    // same event as the container being removed.
    let still_there = Command::new("docker")
        .args(["inspect", &name])
        .output()
        .expect("docker inspect");
    assert!(
        still_there.status.success(),
        "the container must still exist immediately after stop() returns — removal is deferred"
    );

    completion.wait();
    assert_containers_gone(&[&name]);
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

    let long_execution_id = unique_id("m7-cancel-me");
    let long_name = format!("sgt-{long_execution_id}");
    let _long_guard = ContainerGuard::new([long_name.clone()]);
    let long = request(
        "w6",
        &long_execution_id,
        cwd.path(),
        spec(vec!["sleep", "300"], WorkspaceAccess::ReadOnly),
    );
    let long_prepared = backend.prepare(&long).expect("prepare");
    let long_handle = launch(&backend, &long_prepared);

    let bystander_execution_id = unique_id("m7-bystander");
    let bystander_name = format!("sgt-{bystander_execution_id}");
    let _bystander_guard = ContainerGuard::new([bystander_name.clone()]);
    let bystander = request(
        "w6",
        &bystander_execution_id,
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
    assert_containers_gone(&[&long_name, &bystander_name]);
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

    let first_execution_id = unique_id("m7-pin-1");
    let first_name = format!("sgt-{first_execution_id}");
    let _first_guard = ContainerGuard::new([first_name.clone()]);
    let first = request(
        "w7",
        &first_execution_id,
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
    let retry_execution_id = unique_id("m7-pin-2");
    let retry_name = format!("sgt-{retry_execution_id}");
    let _retry_guard = ContainerGuard::new([retry_name.clone()]);
    let retry = request("w7", &retry_execution_id, cwd.path(), bogus_spec);
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
    assert_containers_gone(&[&first_name, &retry_name]);
}

// -------------------------------------------- 7. §22.8 large-output budget

/// §22.8: peak RSS does not grow proportionally with a large capture. This
/// test process's own peak RSS (`VmHWM`, TH-01) is measured before and
/// after streaming roughly
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
    }
    #[cfg(target_os = "linux")]
    {
        let data = support::DataDir::new();
        let cwd = TempDir::new().expect("cwd");
        let backend = backend(data.path());

        const BYTES: u64 = 256 * 1024 * 1024;
        let execution_id = unique_id("m7-large-output");
        let name = format!("sgt-{execution_id}");
        let _guard = ContainerGuard::new([name.clone()]);
        let req = request(
            "w8",
            &execution_id,
            cwd.path(),
            spec(
                vec![
                    "sh",
                    "-c",
                    &format!(
                        // Different content on each stream (not just
                        // different fds) — the blob store is content-
                        // addressed (BLAKE3, write-once) and correctly
                        // deduplicates identical bytes into one blob, which
                        // would otherwise make TH-07's "at least 2*BYTES on
                        // disk" measurement below fail on a store working
                        // exactly as designed.
                        "yes 0123456789abcdef | head -c {BYTES} >&1; \
                         yes fedcba9876543210 | head -c {BYTES} >&2"
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
        // TH-07 / [Rule B] (MVP-2 D3 fixer pass): "the 1 GiB-capture test
        // measures blob disk cost beside §22.8's RSS budget" — before this
        // fix, only the RSS half was ever measured here. Blob bytes before
        // capture must be counted from a store this test's own capture has
        // not touched yet, so this reads the same `data_dir`'s disk-pressure
        // report the doctor uses.
        let blob_bytes_before = docker::measure_disk_pressure(data.path()).blob_bytes;
        let observation = backend
            .observe(&handle)
            .expect("observe (captures on exit)");
        let after = vm_rss_kb();
        let blob_bytes_after = docker::measure_disk_pressure(data.path()).blob_bytes;

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
        assert_containers_gone(&[&name]);

        let increment_kb = after.saturating_sub(before);
        const BUDGET_KB: u64 = 64 * 1024; // §22.8's 64 MiB
        assert!(
            increment_kb <= BUDGET_KB,
            "peak RSS increment while capturing {BYTES} bytes was {increment_kb} KiB, over the \
             {BUDGET_KB} KiB (64 MiB) budget — the streaming capture path must not buffer the \
             whole log in memory"
        );

        // [Rule B] the blob disk cost, measured beside the RSS budget: the
        // 2*BYTES combined stdout+stderr must actually land on disk as
        // blobs — a capture that silently dropped data instead of buffering
        // it would also pass the RSS check above, so this closes that gap.
        let blob_growth = blob_bytes_after.saturating_sub(blob_bytes_before);
        assert!(
            blob_growth >= 2 * BYTES,
            "capturing {BYTES} bytes of stdout and {BYTES} of stderr must grow the blob store \
             by at least that much on disk: before={blob_bytes_before} after={blob_bytes_after} \
             growth={blob_growth}"
        );
    }
}

/// Reads peak RSS (`VmHWM`), not instantaneous RSS (`VmRSS`).
///
/// TH-01 (MVP-2 D3 fixer pass, 2026-08-12): a buffering capture
/// implementation (`read_to_end` into a `Vec` instead of streaming into the
/// blob store) passes an instantaneous-RSS check with ~500x headroom,
/// because glibc serves large allocations via `mmap` and returns the pages
/// to the OS (`munmap`) once the buffer is freed — by the time the "after"
/// sample is taken, the buffer is already gone and RSS has settled back
/// near its "before" value. `VmHWM` is the kernel's own high-water mark for
/// the process and cannot be un-set by freeing memory, so it is the field
/// that actually expresses §22.8's "peak RSS" claim. Measured on Cerberus
/// against a deliberately buffering `stream_one`: `VmRSS` delta 1,148 KiB
/// (comfortably under the 64 MiB budget) vs. `VmHWM` delta 523,224 KiB
/// (~511 MiB, 8x over budget) for the identical run.
#[cfg(target_os = "linux")]
fn vm_rss_kb() -> u64 {
    let status = std::fs::read_to_string("/proc/self/status").expect("read /proc/self/status");
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmHWM:") {
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

// --------------------------------- 7b. §16.8 host-usable file ownership

/// §16.8 / INV-R1-04 (MVP-2 D3 fixer pass): a container-written file, and a
/// container-created *directory*, both come out owned by the host user who
/// already owns the mounted worktree — not root — so the worktree stays
/// editable and cleanable by that user afterward. Before this fix, files
/// landed `uid 0` (measured on Cerberus) and a root-owned directory could
/// not even be `rm -rf`'d by the host user, which breaks worktree teardown,
/// not just editing.
#[test]
fn container_written_files_and_directories_are_owned_by_the_host_worktree_owner() {
    require_docker!();
    #[cfg(not(unix))]
    {
        eprintln!("SKIPPED-ENV: uid/gid ownership is a unix-only concept");
        return;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let data = support::DataDir::new();
        let cwd = TempDir::new().expect("cwd");
        let backend = backend(data.path());
        let host_uid = std::fs::metadata(cwd.path()).expect("stat cwd").uid();
        let host_gid = std::fs::metadata(cwd.path()).expect("stat cwd").gid();

        let execution_id = unique_id("m7-ownership");
        let name = format!("sgt-{execution_id}");
        let _guard = ContainerGuard::new([name.clone()]);
        let req = request(
            "w7b",
            &execution_id,
            cwd.path(),
            spec(
                vec![
                    "sh",
                    "-c",
                    "echo hi > /estate/owned-file.txt && mkdir /estate/owned-dir && \
                     echo inner > /estate/owned-dir/inner.txt",
                ],
                WorkspaceAccess::ReadWrite,
            ),
        );
        let prepared = backend.prepare(&req).expect("prepare");
        let handle = launch(&backend, &prepared);
        let observation = wait_for_exit(&backend, &handle);
        use sergeant_rs::backend::BackendSignal;
        assert!(
            matches!(observation.signal, BackendSignal::StageCompleted { .. }),
            "the container's own write/mkdir must succeed: {:?}",
            observation.signal
        );
        backend.stop(&handle).expect("stop").wait();
        assert_containers_gone(&[&name]);

        let file_meta = std::fs::metadata(cwd.path().join("owned-file.txt")).expect("file meta");
        assert_eq!(
            (file_meta.uid(), file_meta.gid()),
            (host_uid, host_gid),
            "a container-written file must be owned by the host worktree owner, not root"
        );
        let dir_meta = std::fs::metadata(cwd.path().join("owned-dir")).expect("dir meta");
        assert_eq!(
            (dir_meta.uid(), dir_meta.gid()),
            (host_uid, host_gid),
            "a container-created directory must be owned by the host worktree owner, not root"
        );

        // The stronger claim INV-R1-04 named: the host user can actually
        // remove what the container created, including a nested file inside
        // a container-created directory — the shape that broke under root
        // ownership (a root-owned directory refuses `rm` from a non-root
        // host user even when the file inside it is otherwise readable).
        std::fs::remove_dir_all(cwd.path().join("owned-dir"))
            .expect("host user must be able to remove a container-created directory");
        std::fs::remove_file(cwd.path().join("owned-file.txt"))
            .expect("host user must be able to remove a container-created file");
    }
}

// --------------------------------- 7c. §17.5 execute-stage submit preflight

/// TH-05 / SURVIVOR (m7 fixer panel): `Engine::bind_stages`' `StageKind::Execute`
/// arm (§17.5) refuses submission when the fixed `"docker"` backend cannot
/// be routed to, before any Work or worktree side effect — but nothing
/// drove that arm. A mutation that swallows the error and falls through to
/// ordinary actor routing survived the entire suite because no test ever
/// submitted a `kind = "execute"` workflow against an unavailable docker
/// backend. Needs no real Docker Engine — it proves the *unavailable* path,
/// using a scripted `docker_bin` that cannot even run `docker version`
/// (mirroring `DaemonConfig::docker`'s doc: "tests point it at a scripted
/// `docker_bin` ... without a real Docker Engine").
///
/// **Estate-root §5.1/§5.2.** The daemon must be started *bound* to the
/// estate root or this pin silently stops pinning anything: `Engine::plan`
/// reads the bound estate, never `origin.cwd`, so an unbound daemon returns
/// `Ok(None)`, parks the intent at `pending`, and never reaches
/// `bind_stages` at all — the submission is accepted with 201 and the
/// `StageKind::Execute` arm this test exists to drive is never entered.
/// `origin.cwd` therefore names the repository mount as recorded evidence
/// only (§13.3), and the workflow package lives under the *estate* root's
/// `.sergeant/workflows/`, because that is the root `plan` hands to
/// `WorkflowDefinition::resolve`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_kind_execute_stage_is_refused_at_submit_when_docker_is_unavailable() {
    let data = support::DataDir::new();
    let estate = TempDir::new().expect("estate");
    let (mount, _head) = support::scaffold_solo_estate(estate.path(), "execute-only-repo");

    let workflow_dir = estate.path().join(".sergeant/workflows/execute-only");
    std::fs::create_dir_all(&workflow_dir).expect("workflow dir");
    std::fs::write(
        workflow_dir.join("workflow.toml"),
        concat!(
            "[workflow]\n",
            "name = \"execute-only\"\n",
            "version = \"1\"\n",
            "stages = [\"10-execute\"]\n",
            "\n",
            "[stage.\"10-execute\"]\n",
            "kind = \"execute\"\n",
            "image = \"alpine:3.24\"\n",
            "command = [\"true\"]\n",
            "workdir = \"/estate\"\n",
            "workspace_access = \"read_only\"\n",
            "network = \"none\"\n",
        ),
    )
    .expect("workflow.toml");

    let fake = Arc::new(FakeBackend::scripted(FAKE_BACKEND_NAME, []));
    let handle = daemon::start_with(
        data.path(),
        DaemonConfig {
            estate_root: Some(estate.path().to_path_buf()),
            backends: Arc::new(BackendRegistry::new().with(fake.clone())),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            docker: Some(DockerConfig {
                data_dir: data.path().to_path_buf(),
                docker_bin: "/nonexistent/docker-binary-that-must-never-run".to_string(),
            }),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");

    let http = reqwest::Client::new();
    let response = http
        .post(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&json!({
            "command_id": ulid::Ulid::generate().to_string(),
            "intent": "must be refused before anything exists",
            "workflow": "execute-only",
            "origin": {"client": "cli", "cwd": mount},
        }))
        .send()
        .await
        .expect("submit");
    let status = response.status();
    let body: serde_json::Value = response.json().await.expect("json");
    assert_eq!(status, 422, "must be refused at submit: {body}");
    assert_eq!(body["error"]["code"], "execute_backend_unavailable");

    let list: serde_json::Value = http
        .get(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("list")
        .json()
        .await
        .expect("json");
    assert!(
        list["works"].as_array().expect("works").is_empty(),
        "an execute-backend-unavailable refusal must not create Work: {list}"
    );
    assert!(
        !data.path().join("surfaces").exists(),
        "§17.5: rejected before any worktree side effect"
    );
    assert!(fake.starts().is_empty(), "no actor stage ever ran either");

    handle.shutdown().await;
}

// --------------------------------- 7d. §16.9 observe_liveness skips capture

/// INV-R1-08 (MVP-2 D3 fixer pass): `observe()` pays for a full log capture
/// (blob writes) as a side effect of classifying *any* exited container,
/// even when the caller only wants liveness — exactly restart
/// reconciliation's shape. `observe_liveness()` must answer the same
/// liveness question (`NativeState`) without writing any blob.
#[test]
fn observe_liveness_answers_without_writing_a_blob_and_observe_still_does() {
    require_docker!();
    let data = support::DataDir::new();
    let cwd = TempDir::new().expect("cwd");
    let backend = backend(data.path());

    let execution_id = unique_id("m7-liveness-only");
    let name = format!("sgt-{execution_id}");
    let _guard = ContainerGuard::new([name.clone()]);
    let req = request(
        "w7d",
        &execution_id,
        cwd.path(),
        spec(
            vec!["sh", "-c", "echo some captured evidence; exit 0"],
            WorkspaceAccess::ReadOnly,
        ),
    );
    let prepared = backend.prepare(&req).expect("prepare");
    let handle = launch(&backend, &prepared);

    // Wait for the container to actually exit (via docker inspect directly,
    // not via `observe()`/`observe_liveness()` — either would already
    // demonstrate the property under test before we can measure it).
    loop {
        let info = Command::new("docker")
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
        std::thread::sleep(std::time::Duration::from_millis(150));
    }

    let blobs_dir = data.path().join("blobs");
    let blob_count = |dir: &Path| -> usize { walkdir_count(dir) };
    let before = blob_count(&blobs_dir);

    let liveness = backend.observe_liveness(&handle).expect("observe_liveness");
    assert_eq!(
        liveness,
        sergeant_rs::backend::NativeState::Exited,
        "observe_liveness must still classify the exit correctly"
    );
    let after_liveness_only = blob_count(&blobs_dir);
    assert_eq!(
        before, after_liveness_only,
        "observe_liveness must not write any blob (no evidence capture) — before={before} \
         after={after_liveness_only}"
    );

    // The full OBSERVE, by contrast, really does capture and really does
    // write blobs — proving the distinction above is real, not just an
    // empty blob dir for some unrelated reason.
    let observation = backend.observe(&handle).expect("observe");
    assert_eq!(
        observation.native,
        sergeant_rs::backend::NativeState::Exited
    );
    let after_full_observe = blob_count(&blobs_dir);
    assert!(
        after_full_observe > after_liveness_only,
        "the full observe() must write blobs (stdout+stderr capture) — \
         after_liveness_only={after_liveness_only} after_full_observe={after_full_observe}"
    );

    backend.stop(&handle).expect("stop").wait();
    assert_containers_gone(&[&name]);
}

fn walkdir_count(dir: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            count += walkdir_count(&path);
        } else {
            count += 1;
        }
    }
    count
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
///
/// **Estate-root §5.1/§5.2.** Driving it through the real daemon means the
/// daemon has to be *bound* to the estate whose repository is being
/// surfaced: `Engine::plan` topology comes from that binding and from
/// nowhere else, so an unbound daemon would leave this submission `pending`
/// forever (`Ok(None)` — no surface, no stages, no container) and the whole
/// N4 proof would time out rather than fail on anything it claims. The
/// estate is the §6.1 shape — one derived mount at `<root>/repos/<name>`,
/// no `[[repo]] path` key — and its workflow package sits under the estate
/// root's `.sergeant/workflows/`, which is where `plan` resolves it from.
/// `origin.cwd` is recorded evidence only now (§13.3).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn mixed_actor_execute_actor_workflow_completes_with_evidence_handed_forward() {
    require_docker!();
    let data = support::DataDir::new();
    let estate = TempDir::new().expect("estate");
    let (mount, _head) = support::scaffold_solo_estate(estate.path(), "mixed-proof-repo");

    let workflow_dir = estate.path().join(".sergeant/workflows/mixed-proof");
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
            "image = \"alpine:3.24\"\n",
            "command = [\"sh\", \"-c\", \"echo container-produced-evidence > /estate/validated.txt\"]\n",
            "workdir = \"/estate\"\n",
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
            estate_root: Some(estate.path().to_path_buf()),
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
            "origin": {"client": "cli", "cwd": mount},
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
        // ADR 0007(b): this test's "20-close" stage is a bare fake actor
        // that signals completion without ever running `git commit` — it
        // exists to prove the execute stage's artifact reaches a following
        // actor through the worktree, not to exercise a real closing
        // stage's commit procedure. So the branch never advances and the
        // container's `validated.txt` is left dirty, and the honest label
        // for that is `completed_dirty`, not plain `completed`.
        if polled["work"]["state"] == "completed"
            || polled["work"]["state"] == "completed_dirty"
            || polled["work"]["state"] == "blocked"
        {
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
        last["work"]["state"], "completed_dirty",
        "the mixed workflow completes end to end, but its stub closing stage \
         never commits the execute stage's artifact, so it must not be \
         reported as plain success (ADR 0007(b)): {last}"
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

/// Looks for `validated.txt` as a plain file (a clean worktree, or a
/// `RetainedError`/submodule-fallback directory still standing whole), or —
/// #109's retention-scope change — inside a captured `*.dirty.patch`: an
/// uncommitted execute-stage artifact is exactly the "dirty state" R4 says
/// teardown retains as a patch rather than the whole directory, so the
/// evidence this test is proving survives teardown now travels in that
/// form. Either way the evidence handed to the following actor was not
/// lost, which is the actual N4/§21.5 claim this test pins.
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
            && content.trim() == "container-produced-evidence"
        {
            return true;
        } else if path.extension().and_then(|e| e.to_str()) == Some("patch")
            && let Ok(content) = std::fs::read_to_string(&path)
            && content.contains("validated.txt")
            && content.contains("container-produced-evidence")
        {
            return true;
        }
    }
    false
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
