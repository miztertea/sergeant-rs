//! W4c acceptance: service packaging + doctor host rows (#275/#276), plus
//! the #282 stretch.
//!
//! `sgt daemon install-service [--print]` (generation, idempotent write,
//! capability-probe-gated enablement) · the `host_service_manager` and
//! `legacy_estate_runtime` doctor rows · `sgt init`'s host-bootstrap branch
//! · `doc_routes`' managed-block scoping of the `sergeant-rs-workspace`
//! rule (#282). Mirrors `m8_estate_cli.rs`'s own shape: every verb here is
//! either non-daemon-spawning plumbing (`install-service`, `doctor`,
//! `init` — no `support::DataDir` guard needed) or reuses `tests/
//! support`'s injectable fake-binary precedent (`runtime::git::
//! GIT_BIN_ENV`'s sibling, `SGT_SYSTEMCTL_BIN`) to exercise the capability
//! probe without a real systemd user session.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

// ---------------------------------------------------------------- helpers

/// Run `sgt` with `cwd`, an optional `--data-dir`, extra env, and `args` —
/// mirrors `m8_estate_cli.rs`'s own `run` helper exactly (no daemon is ever
/// spawned by any verb this suite exercises).
fn run(cwd: &Path, data_dir: Option<&Path>, env: &[(&str, &str)], args: &[&str]) -> Output {
    let mut command = Command::new(SGT);
    command.current_dir(cwd);
    if let Some(data_dir) = data_dir {
        command.arg("--data-dir").arg(data_dir);
    }
    command.args(args);
    for (key, value) in env {
        command.env(key, value);
    }
    let output = command.output().expect("run sgt");
    Output {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

struct Output {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Output {
    fn json(&self) -> Value {
        serde_json::from_str(&self.stdout)
            .unwrap_or_else(|e| panic!("stdout is not JSON ({e}): {}", self.stdout))
    }

    fn assert_ok(&self, what: &str) -> &Self {
        assert_eq!(
            self.code,
            Some(0),
            "{what} must succeed, got {:?}\nstdout: {}\nstderr: {}",
            self.code,
            self.stdout,
            self.stderr
        );
        self
    }
}

/// A minimal valid estate — `Estate::admit`'s own requirement (`[estate]`
/// declared) is all `install-service`/`doctor`/`init` need here; no repo
/// mounts, matching `m6_surfaces.rs`'s own `doctor(...)` fixture shape.
fn scaffold_bare_estate(root: &Path, name: &str) {
    std::fs::create_dir_all(root).expect("estate root");
    std::fs::write(
        root.join("sergeant.toml"),
        format!("[estate]\nname = {name:?}\n"),
    )
    .expect("write sergeant.toml");
}

/// A scripted binary on `PATH` a test can point `SGT_SYSTEMCTL_BIN` at —
/// the `SGT_GIT_BIN`/`runtime::git`-style injection precedent, applied to
/// `platform::service`'s manager probe. `body` is the shell script's own
/// body (already has `#!/bin/sh` prepended).
fn write_fake_binary(path: &Path, body: &str) {
    std::fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("write fake binary");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
            .expect("chmod fake binary");
    }
}

// --------------------------------------------------------- install-service

/// guard-map: `--print` writes nothing to disk and prints a unit whose
/// `ExecStart` names the host runtime root and whose `Environment=` carries
/// a composed PATH — the golden-file content this seat's brief calls for,
/// exercised end to end through the real binary rather than only through
/// `platform::service`'s own inline tests. Mutation this kills: `--print`
/// writing to the native location anyway, or `ExecStart`/`Environment=`
/// losing the host runtime root / PATH.
#[test]
fn install_service_print_is_a_pure_dry_run() {
    let root = tempfile::TempDir::new().expect("tempdir");
    scaffold_bare_estate(root.path(), "print-test");
    let home = tempfile::TempDir::new().expect("fake home");
    let host_root = root.path().join("host-runtime-root");

    let out = run(
        root.path(),
        Some(&host_root),
        &[("HOME", home.path().to_str().expect("utf8"))],
        &["daemon", "install-service", "--print"],
    );
    out.assert_ok("install-service --print");

    assert!(
        out.stdout.contains(&host_root.display().to_string()),
        "printed unit must name the host runtime root, got:\n{}",
        out.stdout
    );
    assert!(
        out.stdout.contains("Environment=") || out.stdout.contains("EnvironmentVariables"),
        "printed unit must carry PATH enrichment in its own Environment= (or plist \
         EnvironmentVariables), never left to an inline prefix (#275): got:\n{}",
        out.stdout
    );

    // Never writes the native per-user location.
    assert!(
        !home.path().join(".config/systemd/user").exists(),
        "--print must never write the native systemd unit dir"
    );
    assert!(
        !home.path().join("Library/LaunchAgents").exists(),
        "--print must never write the native LaunchAgent dir"
    );
}

/// guard-map: without `--print`, the unit/plist is actually written to the
/// native per-user location under the fake `$HOME`, and — with no real
/// service manager reachable in this test's `PATH` — enablement is skipped
/// with a named remedy rather than failing the command (H1-05, the
/// `docker_check` degradation pattern). Mutation this kills: a write
/// failure being swallowed, or a missing manager turning into a nonzero
/// exit instead of a skip.
#[test]
fn install_service_writes_and_skips_enablement_with_no_manager_reachable() {
    let root = tempfile::TempDir::new().expect("tempdir");
    scaffold_bare_estate(root.path(), "write-test");
    let home = tempfile::TempDir::new().expect("fake home");
    let host_root = root.path().join("host-runtime-root");

    // A `systemctl`/`launchctl` that always fails to spawn — the "Absent"
    // arm, exercised end to end (not just `platform::service`'s own
    // `classify_systemctl_output` unit test).
    let out = run(
        root.path(),
        Some(&host_root),
        &[
            ("HOME", home.path().to_str().expect("utf8")),
            ("SGT_SYSTEMCTL_BIN", "sgt-w4c-nonexistent-systemctl-xyz"),
            ("SGT_LAUNCHCTL_BIN", "sgt-w4c-nonexistent-launchctl-xyz"),
        ],
        &["--json", "daemon", "install-service"],
    );
    out.assert_ok("install-service without --print, no manager reachable");
    let json = out.json();
    assert_eq!(
        json["enabled"].as_bool(),
        Some(false),
        "no manager reachable — enabled must be false, got {json}"
    );
    assert!(
        json["remedy"].as_str().is_some(),
        "a skipped enablement must name a remedy, got {json}"
    );

    #[cfg(target_os = "macos")]
    let written = home
        .path()
        .join("Library/LaunchAgents/com.sergeant.host-daemon.plist");
    #[cfg(not(target_os = "macos"))]
    let written = home
        .path()
        .join(".config/systemd/user/sergeant-daemon.service");

    assert!(
        written.is_file(),
        "the unit/plist must still be written even though enablement was skipped, expected {}",
        written.display()
    );
    let content = std::fs::read_to_string(&written).expect("read written unit");
    assert!(content.contains(&host_root.display().to_string()));
}

/// guard-map: a second `install-service` run over unchanged inputs writes
/// the identical bytes — `write_atomic`'s idempotent-write discipline
/// applied here, pinned by mtime/inode not moving on a byte-identical
/// rewrite would be the stronger version, but content equality alone
/// already kills the mutation class this seat's brief calls out
/// (non-deterministic generation, e.g. an embedded timestamp).
#[test]
fn install_service_generation_is_idempotent_across_repeated_runs() {
    let root = tempfile::TempDir::new().expect("tempdir");
    scaffold_bare_estate(root.path(), "idempotent-test");
    let home = tempfile::TempDir::new().expect("fake home");
    let host_root = root.path().join("host-runtime-root");
    let env: &[(&str, &str)] = &[
        ("HOME", home.path().to_str().expect("utf8")),
        ("SGT_SYSTEMCTL_BIN", "sgt-w4c-nonexistent-systemctl-xyz"),
        ("SGT_LAUNCHCTL_BIN", "sgt-w4c-nonexistent-launchctl-xyz"),
    ];

    run(
        root.path(),
        Some(&host_root),
        env,
        &["daemon", "install-service"],
    )
    .assert_ok("first");

    #[cfg(target_os = "macos")]
    let written = home
        .path()
        .join("Library/LaunchAgents/com.sergeant.host-daemon.plist");
    #[cfg(not(target_os = "macos"))]
    let written = home
        .path()
        .join(".config/systemd/user/sergeant-daemon.service");

    let first_content = std::fs::read_to_string(&written).expect("first content");

    run(
        root.path(),
        Some(&host_root),
        env,
        &["daemon", "install-service"],
    )
    .assert_ok("second");
    let second_content = std::fs::read_to_string(&written).expect("second content");

    assert_eq!(
        first_content, second_content,
        "unchanged inputs must generate byte-identical content across runs"
    );
}

/// guard-map: a scripted `systemctl` on `PATH` (the `SGT_SYSTEMCTL_BIN`
/// injection precedent, Linux-only — this exercises `enable_service`'s
/// `Reachable` arm end to end) reports success, and `install-service`
/// reports `enabled: true`. Skipped on non-Linux since the systemd arm is
/// production-dead there.
#[test]
#[cfg(target_os = "linux")]
fn install_service_enables_when_a_scripted_systemctl_reports_reachable() {
    let root = tempfile::TempDir::new().expect("tempdir");
    scaffold_bare_estate(root.path(), "reachable-test");
    let home = tempfile::TempDir::new().expect("fake home");
    let host_root = root.path().join("host-runtime-root");
    let bin_dir = tempfile::TempDir::new().expect("bin dir");
    let fake_systemctl = bin_dir.path().join("fake-systemctl");
    // Every invocation succeeds — `list-units` (the probe), `daemon-reload`,
    // and `enable --now` alike.
    write_fake_binary(&fake_systemctl, "exit 0");

    let out = run(
        root.path(),
        Some(&host_root),
        &[
            ("HOME", home.path().to_str().expect("utf8")),
            ("SGT_SYSTEMCTL_BIN", fake_systemctl.to_str().expect("utf8")),
        ],
        &["--json", "daemon", "install-service"],
    );
    out.assert_ok("install-service with a reachable scripted systemctl");
    let json = out.json();
    assert_eq!(
        json["enabled"].as_bool(),
        Some(true),
        "a reachable manager must be reported as enabled, got {json}"
    );
}

/// guard-map: a scripted `systemctl` that fails the way a bus-less session
/// does (per this seat's recon and `scripts/gate.sh`'s own measured
/// evidence) is reported as skipped, distinctly from "absent" — the remedy
/// text differs (no linger/desktop session vs. install a manager).
#[test]
#[cfg(target_os = "linux")]
fn install_service_skips_with_a_no_session_remedy_when_systemctl_reports_no_bus() {
    let root = tempfile::TempDir::new().expect("tempdir");
    scaffold_bare_estate(root.path(), "no-session-test");
    let home = tempfile::TempDir::new().expect("fake home");
    let host_root = root.path().join("host-runtime-root");
    let bin_dir = tempfile::TempDir::new().expect("bin dir");
    let fake_systemctl = bin_dir.path().join("fake-systemctl");
    write_fake_binary(
        &fake_systemctl,
        "echo 'Failed to connect to bus: No such file or directory' >&2; exit 1",
    );

    let out = run(
        root.path(),
        Some(&host_root),
        &[
            ("HOME", home.path().to_str().expect("utf8")),
            ("SGT_SYSTEMCTL_BIN", fake_systemctl.to_str().expect("utf8")),
        ],
        &["--json", "daemon", "install-service"],
    );
    out.assert_ok("install-service with a present-but-unreachable scripted systemctl");
    let json = out.json();
    assert_eq!(json["enabled"].as_bool(), Some(false));
    let remedy = json["remedy"].as_str().expect("remedy present");
    assert!(
        remedy.contains("session") || remedy.contains("bus"),
        "the no-user-session remedy must name the actual cause, got: {remedy}"
    );
}

// --------------------------------------------------------------- doctor

/// guard-map: `sgt doctor --json` carries both new rows by name, in every
/// run — a caller that only ever sees `host_service_manager`/
/// `legacy_estate_runtime` missing from the checks array has silently lost
/// the row entirely (not merely "reports a different verdict").
#[test]
fn doctor_reports_both_new_rows_by_name() {
    let root = tempfile::TempDir::new().expect("tempdir");
    scaffold_bare_estate(root.path(), "doctor-rows-test");
    let home = tempfile::TempDir::new().expect("fake home");
    let data_dir = root.path().join("data-dir");

    let out = run(
        root.path(),
        Some(&data_dir),
        &[
            ("HOME", home.path().to_str().expect("utf8")),
            ("SGT_SYSTEMCTL_BIN", "sgt-w4c-nonexistent-systemctl-xyz"),
            ("SGT_LAUNCHCTL_BIN", "sgt-w4c-nonexistent-launchctl-xyz"),
        ],
        &["--json", "doctor"],
    );
    let json = out.json();
    let names: Vec<&str> = json["checks"]
        .as_array()
        .expect("checks array")
        .iter()
        .map(|c| c["name"].as_str().expect("name"))
        .collect();
    assert!(
        names.contains(&"host_service_manager"),
        "doctor must report host_service_manager, got {names:?}"
    );
    assert!(
        names.contains(&"legacy_estate_runtime"),
        "doctor must report legacy_estate_runtime, got {names:?}"
    );
}

/// guard-map: with no manager reachable, `host_service_manager` warns
/// (never fails — dev hosts are legal without one) and names a remedy.
/// Mutation this kills: the row reporting `fail` for an absent manager, or
/// reporting `ok` with no remedy.
#[test]
fn host_service_manager_warns_not_fails_when_absent() {
    let root = tempfile::TempDir::new().expect("tempdir");
    scaffold_bare_estate(root.path(), "manager-absent-test");
    let home = tempfile::TempDir::new().expect("fake home");
    let data_dir = root.path().join("data-dir");

    let out = run(
        root.path(),
        Some(&data_dir),
        &[
            ("HOME", home.path().to_str().expect("utf8")),
            ("SGT_SYSTEMCTL_BIN", "sgt-w4c-nonexistent-systemctl-xyz"),
            ("SGT_LAUNCHCTL_BIN", "sgt-w4c-nonexistent-launchctl-xyz"),
        ],
        &["--json", "doctor"],
    );
    let json = out.json();
    let row = json["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .find(|c| c["name"] == "host_service_manager")
        .expect("host_service_manager row present");
    assert_eq!(row["status"].as_str(), Some("warn"));
    assert!(row["remedy"].as_str().is_some());
}

/// guard-map (Linux): a scripted reachable `systemctl` makes the row `ok`.
#[test]
#[cfg(target_os = "linux")]
fn host_service_manager_is_ok_when_reachable() {
    let root = tempfile::TempDir::new().expect("tempdir");
    scaffold_bare_estate(root.path(), "manager-reachable-test");
    let home = tempfile::TempDir::new().expect("fake home");
    let data_dir = root.path().join("data-dir");
    let bin_dir = tempfile::TempDir::new().expect("bin dir");
    let fake_systemctl = bin_dir.path().join("fake-systemctl");
    write_fake_binary(&fake_systemctl, "exit 0");

    let out = run(
        root.path(),
        Some(&data_dir),
        &[
            ("HOME", home.path().to_str().expect("utf8")),
            ("SGT_SYSTEMCTL_BIN", fake_systemctl.to_str().expect("utf8")),
        ],
        &["--json", "doctor"],
    );
    let json = out.json();
    let row = json["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .find(|c| c["name"] == "host_service_manager")
        .expect("host_service_manager row present");
    assert_eq!(row["status"].as_str(), Some("ok"), "row: {row}");
}

/// guard-map: `legacy_estate_runtime` warns (never fails, this wave) and
/// names the H1 §6 reconcile-or-abandon remedy once `daemon.lock`/
/// `runtime.json`/`journal/` exist under the estate's own
/// `.sergeant/data` — the exact pre-cutover shape a not-yet-migrated
/// estate leaves behind. Mutation this kills: the row staying `ok` with
/// legacy state actually present, or firing on an estate that has none.
#[test]
fn legacy_estate_runtime_warns_when_estate_local_daemon_state_is_present() {
    let root = tempfile::TempDir::new().expect("tempdir");
    scaffold_bare_estate(root.path(), "legacy-runtime-test");
    let home = tempfile::TempDir::new().expect("fake home");
    // Estate-local `.sergeant/data` (the pre-cutover default location) with
    // a stale daemon.lock left in it, simulating an un-migrated estate.
    let legacy_dir = root.path().join(".sergeant/data");
    std::fs::create_dir_all(&legacy_dir).expect("legacy dir");
    std::fs::write(legacy_dir.join("daemon.lock"), b"").expect("legacy lock");

    // No --data-dir flag: resolution falls to the estate default
    // (`.sergeant/data`), which is exactly the directory just seeded —
    // matching how a real un-migrated estate would look.
    let out = run(
        root.path(),
        None,
        &[
            ("HOME", home.path().to_str().expect("utf8")),
            ("SGT_SYSTEMCTL_BIN", "sgt-w4c-nonexistent-systemctl-xyz"),
            ("SGT_LAUNCHCTL_BIN", "sgt-w4c-nonexistent-launchctl-xyz"),
        ],
        &["--json", "doctor"],
    );
    let json = out.json();
    let row = json["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .find(|c| c["name"] == "legacy_estate_runtime")
        .expect("legacy_estate_runtime row present");
    assert_eq!(row["status"].as_str(), Some("warn"), "row: {row}");
    let remedy = row["remedy"].as_str().expect("remedy present");
    assert!(
        remedy.to_lowercase().contains("stop") && remedy.to_lowercase().contains("drain"),
        "remedy must name the stop-and-drain reconcile path, got: {remedy}"
    );
}

/// guard-map: a fresh estate with none of the three markers reports `ok`.
#[test]
fn legacy_estate_runtime_is_ok_with_no_markers_present() {
    let root = tempfile::TempDir::new().expect("tempdir");
    scaffold_bare_estate(root.path(), "legacy-runtime-clean-test");
    let home = tempfile::TempDir::new().expect("fake home");
    let data_dir = root.path().join("data-dir");

    let out = run(
        root.path(),
        Some(&data_dir),
        &[("HOME", home.path().to_str().expect("utf8"))],
        &["--json", "doctor"],
    );
    let json = out.json();
    let row = json["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .find(|c| c["name"] == "legacy_estate_runtime")
        .expect("legacy_estate_runtime row present");
    assert_eq!(row["status"].as_str(), Some("ok"), "row: {row}");
}

// ----------------------------------------------------------------- init

/// guard-map: a fresh `sgt init` creates the host runtime root and reports
/// it in `--json` output; a second `init` (same `HOME`) finds it already
/// there and reports `created: false` — `m8_estate_cli.rs`'s own
/// `init_scaffolds_and_is_idempotent` idempotence style, applied to the
/// host-bootstrap branch. Mutation this kills: the host runtime root not
/// being created at all, or a second `init` recreating/re-reporting it as
/// newly created.
#[test]
fn init_creates_the_host_runtime_root_once_and_is_idempotent() {
    let root = tempfile::TempDir::new().expect("tempdir");
    let home = tempfile::TempDir::new().expect("fake home");
    let scratch_data_dir = root.path().join("scratch-data-dir");
    let env: &[(&str, &str)] = &[("HOME", home.path().to_str().expect("utf8"))];

    let first = run(
        root.path(),
        Some(&scratch_data_dir),
        env,
        &["--json", "init", "--name", "host-bootstrap-test"],
    );
    first.assert_ok("first init");
    let first_json = first.json();
    let bootstrap = &first_json["host_bootstrap"];
    assert!(
        !bootstrap.is_null(),
        "a resolvable host runtime root must produce a host_bootstrap object, got {first_json}"
    );
    assert_eq!(
        bootstrap["created"].as_bool(),
        Some(true),
        "first init must create the host runtime root, got {bootstrap}"
    );
    let host_runtime_dir = bootstrap["host_runtime_dir"]
        .as_str()
        .expect("host_runtime_dir string");
    assert!(
        PathBuf::from(host_runtime_dir).is_dir(),
        "the reported host runtime root must actually exist on disk: {host_runtime_dir}"
    );

    let second = run(
        root.path(),
        Some(&scratch_data_dir),
        env,
        &["--json", "init", "--name", "host-bootstrap-test"],
    );
    second.assert_ok("second init");
    let second_json = second.json();
    assert_eq!(
        second_json["host_bootstrap"]["created"].as_bool(),
        Some(false),
        "a second init must find the host runtime root already there, got {second_json}"
    );
}

/// guard-map: `sgt init` never touches a live daemon and never installs a
/// service — the host-bootstrap branch's own stated boundary (recon risk
/// 2). A daemon descriptor dropped into the just-created host runtime root
/// beforehand must survive `sgt init` untouched, and no service unit
/// appears under the fake `$HOME` as a side effect of `init` alone.
#[test]
fn init_host_bootstrap_never_installs_a_service_or_touches_daemon_state() {
    let root = tempfile::TempDir::new().expect("tempdir");
    let home = tempfile::TempDir::new().expect("fake home");
    let scratch_data_dir = root.path().join("scratch-data-dir");

    run(
        root.path(),
        Some(&scratch_data_dir),
        &[("HOME", home.path().to_str().expect("utf8"))],
        &["--json", "init", "--name", "no-side-effects-test"],
    )
    .assert_ok("init");

    assert!(
        !home
            .path()
            .join(".config/systemd/user/sergeant-daemon.service")
            .exists(),
        "`sgt init` must never install the systemd unit on its own"
    );
    assert!(
        !home
            .path()
            .join("Library/LaunchAgents/com.sergeant.host-daemon.plist")
            .exists(),
        "`sgt init` must never install the LaunchAgent on its own"
    );
}

// ------------------------------------------------------- doc_routes (#282)

/// guard-map (#282): a private, unshipped `sergeant-rs-workspace` path
/// cited in `AGENTS.md`'s *user-authored* content (outside the
/// `sgt:managed` block) must not fail `doc_routes` — the exact false
/// positive this issue names, found live on the dev workspace's own
/// estate, whose constitution legitimately cites its own repo name.
/// Mutation this kills: reverting the managed-block scoping back to a
/// whole-file substring check.
#[test]
fn doc_routes_ignores_a_workspace_citation_outside_the_managed_block() {
    let root = tempfile::TempDir::new().expect("tempdir");
    scaffold_bare_estate(root.path(), "doc-routes-user-content-test");
    std::fs::write(
        root.path().join("AGENTS.md"),
        "# my constitution\n\n\
         see sergeant-rs-workspace's development record for context.\n\n\
         <!-- sgt:managed:begin -->\n\
         managed body, no workspace citation here\n\
         <!-- sgt:managed:end -->\n",
    )
    .expect("write AGENTS.md");
    let home = tempfile::TempDir::new().expect("fake home");
    let data_dir = root.path().join("data-dir");

    let out = run(
        root.path(),
        Some(&data_dir),
        &[("HOME", home.path().to_str().expect("utf8"))],
        &["--json", "doctor"],
    );
    let json = out.json();
    let row = json["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .find(|c| c["name"] == "doc_routes")
        .expect("doc_routes row present");
    assert_eq!(
        row["status"].as_str(),
        Some("ok"),
        "a workspace citation outside the managed block must not fail: {row}"
    );
}

/// guard-map (#282): the same substring *inside* the `sgt:managed` block —
/// sgt-owned content — still fails, exactly as before. This is what proves
/// the fix scoped the rule rather than disabling it.
#[test]
fn doc_routes_still_fails_when_the_managed_block_itself_cites_the_workspace() {
    let root = tempfile::TempDir::new().expect("tempdir");
    scaffold_bare_estate(root.path(), "doc-routes-managed-content-test");
    std::fs::write(
        root.path().join("AGENTS.md"),
        "# my constitution\n\n\
         <!-- sgt:managed:begin -->\n\
         this managed body cites sergeant-rs-workspace directly\n\
         <!-- sgt:managed:end -->\n",
    )
    .expect("write AGENTS.md");
    let home = tempfile::TempDir::new().expect("fake home");
    let data_dir = root.path().join("data-dir");

    let out = run(
        root.path(),
        Some(&data_dir),
        &[("HOME", home.path().to_str().expect("utf8"))],
        &["--json", "doctor"],
    );
    let json = out.json();
    let row = json["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .find(|c| c["name"] == "doc_routes")
        .expect("doc_routes row present");
    assert_eq!(
        row["status"].as_str(),
        Some("fail"),
        "a workspace citation inside the managed block must still fail: {row}"
    );
    let detail = row["detail"].as_str().expect("detail present");
    assert!(
        detail.contains("sergeant-rs-workspace"),
        "the detail must still name the offending citation, got: {detail}"
    );
    assert!(row["remedy"].as_str().is_some());
}
