//! W5 brief deliverable 6 — estate cutover rehearsal, and rehearsal only.
//!
//! The live cutover of any real estate to host mode is Captain's own call,
//! pending owner ruling (sprint plan W5's own open question) — nothing here
//! performs one. What this proves is the *path* a real cutover would walk:
//! `sgt doctor` detecting estate-local runtime state left over from before
//! the host daemon existed, and `sgt daemon install-service --print`
//! showing what native-service installation would do, against the same
//! scratch estate, in one rehearsal — neither step mutates the fabricated
//! legacy state or writes to any real native-service location.
//!
//! `tests/w4c_service_doctor.rs` already pins `legacy_estate_runtime` and
//! `install-service --print` as two separate claims; this file is their
//! combined walkthrough, exactly as an operator (or Captain, later) would
//! actually run it: doctor first, to see what needs reconciling, then
//! `install-service --print`, to see what installing would do — both
//! against the one scratch estate, neither one touching it for real.

use std::path::Path;
use std::process::Command;

use serde_json::Value;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

struct Output {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}

impl Output {
    fn json(&self) -> Value {
        serde_json::from_str(&self.stdout)
            .unwrap_or_else(|e| panic!("stdout is not JSON ({e}): {}", self.stdout))
    }

    fn assert_ok(&self, what: &str) -> &Self {
        assert!(
            self.status.success(),
            "{what} must succeed, got {:?}\nstdout: {}\nstderr: {}",
            self.status.code(),
            self.stdout,
            self.stderr
        );
        self
    }
}

fn run(cwd: &Path, data_dir: &Path, home: &Path, args: &[&str]) -> Output {
    let out = Command::new(SGT)
        .current_dir(cwd)
        .arg("--data-dir")
        .arg(data_dir)
        .args(args)
        .env("HOME", home)
        // No real service manager reachable — the rehearsal proves the
        // detection and the dry-run print, not a live enablement.
        .env("SGT_SYSTEMCTL_BIN", "sgt-rehearsal-nonexistent-systemctl")
        .env("SGT_LAUNCHCTL_BIN", "sgt-rehearsal-nonexistent-launchctl")
        .output()
        .expect("run sgt");
    Output {
        status: out.status,
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

fn scaffold_bare_estate(root: &Path, name: &str) {
    std::fs::write(
        root.join("sergeant.toml"),
        format!("[estate]\nname = \"{name}\"\n"),
    )
    .expect("manifest");
}

/// guard-map: one scratch estate, fabricated legacy runtime markers, walked
/// through both halves of the rehearsal path in the order an operator
/// actually would. Mutation this kills: `legacy_estate_runtime` losing its
/// `reconcile`/`abandon` vocabulary, `install-service --print` writing
/// anything real, or either step disturbing the other's fixture state.
#[test]
fn rehearsal_walks_doctor_detection_then_install_service_print_without_mutating_anything() {
    let root = tempfile::TempDir::new().expect("estate tempdir");
    scaffold_bare_estate(root.path(), "rehearsal-estate");
    let home = tempfile::TempDir::new().expect("fake home tempdir");
    let host_root = root.path().join("host-runtime-root");

    // Fabricate exactly the three markers `legacy_estate_runtime_check`
    // looks for, under the estate-local `.sergeant/data` the pre-cutover
    // daemon used — daemon.lock, a runtime descriptor, and a journal
    // directory, none of them real.
    let legacy_dir = root.path().join(".sergeant/data");
    std::fs::create_dir_all(legacy_dir.join("journal")).expect("legacy journal dir");
    std::fs::write(legacy_dir.join("daemon.lock"), b"").expect("legacy lock");
    std::fs::write(legacy_dir.join("runtime.json"), b"{}").expect("legacy descriptor");

    // Step 1: `sgt doctor` — no --data-dir, so resolution falls to the
    // estate-local default, exactly where the fabricated markers sit.
    let doctor = Command::new(SGT)
        .current_dir(root.path())
        .args(["--json", "doctor"])
        .env("HOME", home.path())
        .env("SGT_SYSTEMCTL_BIN", "sgt-rehearsal-nonexistent-systemctl")
        .env("SGT_LAUNCHCTL_BIN", "sgt-rehearsal-nonexistent-launchctl")
        .output()
        .expect("run sgt doctor");
    let doctor = Output {
        status: doctor.status,
        stdout: String::from_utf8_lossy(&doctor.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&doctor.stderr).into_owned(),
    };
    let doctor_json = doctor.json();
    let row = doctor_json["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .find(|c| c["name"] == "legacy_estate_runtime")
        .expect("legacy_estate_runtime row present");
    assert_eq!(row["status"].as_str(), Some("warn"), "row: {row}");
    let remedy = row["remedy"]
        .as_str()
        .expect("remedy present")
        .to_lowercase();
    assert!(
        remedy.contains("reconcile") && remedy.contains("abandon"),
        "criterion: the remedy must name the reconcile-or-abandon path, got: {remedy}"
    );

    // The fabricated markers survive doctor untouched — detection is a
    // read, never a repair.
    assert!(legacy_dir.join("daemon.lock").exists());
    assert!(legacy_dir.join("runtime.json").exists());
    assert!(legacy_dir.join("journal").is_dir());

    // Step 2: `sgt daemon install-service --print` — the same estate, a
    // fresh host runtime root elsewhere entirely (the cutover's actual
    // destination), print-only.
    let install = run(
        root.path(),
        &host_root,
        home.path(),
        &["daemon", "install-service", "--print"],
    );
    install.assert_ok("install-service --print");
    assert!(
        install.stdout.contains(&host_root.display().to_string()),
        "the printed unit must name the host runtime root the cutover would use: {}",
        install.stdout
    );

    // The rehearsal touches neither the native service location...
    assert!(
        !home.path().join(".config/systemd/user").exists(),
        "--print must never write the native systemd unit dir"
    );
    assert!(
        !home.path().join("Library/LaunchAgents").exists(),
        "--print must never write the native LaunchAgent dir"
    );
    // ...nor the legacy estate state doctor just reported on — a rehearsal
    // proves the path, it does not walk it for real.
    assert!(legacy_dir.join("daemon.lock").exists());
    assert!(legacy_dir.join("runtime.json").exists());
    assert!(legacy_dir.join("journal").is_dir());
}
