//! Native per-user service unit generation and reachability probing (H1
//! §2, #275/#276).
//!
//! Two things live here, kept apart the same way [`super::process`] keeps
//! `running_processes`/`process_alive` apart — different questions, tested
//! differently:
//!
//! - **Generation** ([`systemd_unit`], [`launchd_plist`]) is pure text
//!   assembly over a [`ServiceSpec`] — no service manager, no filesystem, no
//!   subprocess. It is exercised by golden-file tests from any host, always
//!   (ADR 0002 D3): the systemd arm and the macOS arm are both compiled and
//!   both tested unconditionally, the same discipline `platform::data_dir`
//!   already uses for its `FREEDESKTOP`/`MACOS` constants.
//! - **Reachability** ([`ManagerStatus`], [`detect_status`]) shells out to
//!   `systemctl --user`/`launchctl`, following the `docker_check` degradation
//!   pattern: a missing or unreachable manager is a named, warn-not-fail
//!   fact, never a broken build. The classification logic
//!   ([`classify_systemctl_output`]) is a plain function over already-
//!   captured process output, unit-tested directly without a real systemd
//!   user session — the `SGT_GIT_BIN`-style injectable-binary precedent
//!   (`runtime::git::GIT_BIN_ENV`) covers the subprocess-integration half via
//!   [`SYSTEMCTL_BIN_ENV`]/[`LAUNCHCTL_BIN_ENV`], a PATH-shimmed fake binary
//!   a test can point at.
//!
//! **The env-stripping hazard this generator exists to close (#275):**
//! `scripts/gate.sh`'s own measured incident (Cerberus, 2026-08-11) found
//! that a systemd-user-managed unit only bakes `HOME`/`PATH` in from the
//! *manager's* own environment, and silently discards anything set as an
//! inline prefix on the start command. [`ServiceSpec::path`] is therefore
//! computed once, at generation time, by the caller (never read from the
//! environment inside this module) and baked into the unit's own
//! `Environment=`/the plist's own `EnvironmentVariables` — never assumed to
//! flow through from whatever shell ran `sgt daemon install-service`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The generated systemd user unit's file name, unqualified — joined under
/// [`systemd_user_unit_dir`] to get the full install path.
pub const SYSTEMD_UNIT_NAME: &str = "sergeant-daemon.service";

/// The generated LaunchAgent's `Label`, and (with `.plist` appended) its
/// file name under [`launchd_agents_dir`].
pub const LAUNCHD_LABEL: &str = "com.sergeant.host-daemon";

/// Every input the two generators need — deliberately plain data, not a
/// struct that reaches into the environment itself, so [`systemd_unit`] and
/// [`launchd_plist`] stay pure functions of their arguments and golden-file
/// tests can pin their output byte for byte with substituted paths.
#[derive(Debug, Clone)]
pub struct ServiceSpec {
    /// The current `sgt` binary's own path (`std::env::current_exe()`),
    /// exec'd with a fixed `--data-dir <host_runtime_dir> daemon` tail — the
    /// unit does not go through a shell, so no quoting is needed.
    pub binary_path: PathBuf,
    /// The host runtime root ([`crate::cli::resolve_host_runtime_dir`]'s
    /// result — W1b's seam): named explicitly with `--data-dir` rather than
    /// relying on any inherited environment, the same "name it explicitly,
    /// never infer it from an environment a detached child does not
    /// reliably keep" discipline `spawn_daemon` already applies to
    /// `estate_root`.
    pub host_runtime_dir: PathBuf,
    /// PATH, already composed with [`crate::harness::toolchain_path_dirs`]
    /// baked in by the caller at generation time (#275's fix) — never
    /// recomputed or read from the environment inside this module. See the
    /// module doc's env-stripping hazard note.
    pub path: String,
}

/// The systemd user unit's full text — `[Unit]`/`[Service]`/`[Install]`,
/// `ExecStart` pointing at [`ServiceSpec::binary_path`] with
/// `--data-dir <host_runtime_dir> daemon`, and `Environment=` carrying
/// [`ServiceSpec::path`] (#275: never an inline prefix on `ExecStart`
/// itself — see the module doc). Deterministic: the same `spec` always
/// produces the same bytes, which is what makes [`crate::runtime::fsutil::
/// write_atomic`]'s idempotent-write pattern ("write only when content
/// differs") a correct fit for installing it.
pub fn systemd_unit(spec: &ServiceSpec) -> String {
    format!(
        "[Unit]\n\
         Description=Sergeant host daemon\n\
         After=network.target\n\
         \n\
         [Service]\n\
         Type=simple\n\
         ExecStart={} --data-dir {} daemon\n\
         Environment=\"PATH={}\"\n\
         Restart=on-failure\n\
         RestartSec=5\n\
         \n\
         [Install]\n\
         WantedBy=default.target\n",
        spec.binary_path.display(),
        spec.host_runtime_dir.display(),
        spec.path,
    )
}

/// The macOS LaunchAgent plist's full text — `ProgramArguments` carrying
/// the binary and its argv as separate array entries (no shell, no
/// quoting-by-string-concatenation), `EnvironmentVariables` carrying
/// [`ServiceSpec::path`] under the `PATH` key. Risk 5 from this seat's own
/// recon (`recon-doctor-init-service.md`): a LaunchAgent's
/// `EnvironmentVariables` has not been measured on real macOS hardware the
/// way `gate.sh`'s systemd finding has — this generator follows the
/// documented Apple-side contract (a plain string-keyed dict, no special
/// expansion), but the "never an inline prefix" fix is applied identically
/// to both arms on the same reasoning until a macOS measurement pass says
/// otherwise.
pub fn launchd_plist(spec: &ServiceSpec) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n\
         <dict>\n\
         \t<key>Label</key>\n\
         \t<string>{label}</string>\n\
         \t<key>ProgramArguments</key>\n\
         \t<array>\n\
         \t\t<string>{binary}</string>\n\
         \t\t<string>--data-dir</string>\n\
         \t\t<string>{host_runtime_dir}</string>\n\
         \t\t<string>daemon</string>\n\
         \t</array>\n\
         \t<key>EnvironmentVariables</key>\n\
         \t<dict>\n\
         \t\t<key>PATH</key>\n\
         \t\t<string>{path}</string>\n\
         \t</dict>\n\
         \t<key>RunAtLoad</key>\n\
         \t<true/>\n\
         \t<key>KeepAlive</key>\n\
         \t<true/>\n\
         \t<key>ProcessType</key>\n\
         \t<string>Background</string>\n\
         </dict>\n\
         </plist>\n",
        label = LAUNCHD_LABEL,
        binary = spec.binary_path.display(),
        host_runtime_dir = spec.host_runtime_dir.display(),
        path = spec.path,
    )
}

/// `~/.config/systemd/user/` — the native per-user unit directory
/// (freedesktop convention; not `$XDG_CONFIG_HOME`-overridable here because
/// `systemctl --user` itself does not honor that override for unit search
/// either, only for its own config).
pub fn systemd_user_unit_dir(home: &Path) -> PathBuf {
    home.join(".config").join("systemd").join("user")
}

/// `~/Library/LaunchAgents/` — the native per-user LaunchAgent directory.
pub fn launchd_agents_dir(home: &Path) -> PathBuf {
    home.join("Library").join("LaunchAgents")
}

/// [`launchd_agents_dir`] joined with `{LAUNCHD_LABEL}.plist` — the exact
/// install path `launchctl bootstrap` expects to find.
pub fn launchd_plist_path(home: &Path) -> PathBuf {
    launchd_agents_dir(home).join(format!("{LAUNCHD_LABEL}.plist"))
}

/// [`systemd_user_unit_dir`] joined with [`SYSTEMD_UNIT_NAME`].
pub fn systemd_unit_path(home: &Path) -> PathBuf {
    systemd_user_unit_dir(home).join(SYSTEMD_UNIT_NAME)
}

/// Overrides the `systemctl` binary [`detect_systemd_status`] shells out to
/// — mirrors `runtime::git::GIT_BIN_ENV` (a scripted fake stands in for a
/// real user D-Bus session, which CI does not have). Read fresh on every
/// call for the same reason `GIT_BIN_ENV` is: a cached value would leak one
/// test's override into another's in a parallel run.
pub const SYSTEMCTL_BIN_ENV: &str = "SGT_SYSTEMCTL_BIN";

/// Overrides the `launchctl` binary [`detect_launchd_status`] shells out to.
pub const LAUNCHCTL_BIN_ENV: &str = "SGT_LAUNCHCTL_BIN";

/// The three-way answer `sgt doctor`'s `host_service_manager` row (#276)
/// needs: a supported manager reachable and usable right now, present but
/// unusable from this session, or genuinely absent. Each has its own
/// remedy — collapsing `PresentNoUserSession` into `Absent` would tell an
/// SSH/cron operator to install something that is already installed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagerStatus {
    /// The manager answered a reachability probe successfully.
    Reachable,
    /// The manager binary exists and ran, but reported it cannot reach a
    /// user session (e.g. `systemctl --user` with no D-Bus session — common
    /// in bare SSH/cron contexts per this seat's recon, risk 6).
    PresentNoUserSession {
        /// The manager's own stderr, trimmed — the most useful evidence to
        /// surface verbatim.
        detail: String,
    },
    /// The manager binary could not be found or executed at all.
    Absent,
}

/// The classification logic proper, over already-captured process output —
/// unit-tested directly (no real systemd user session needed) per this
/// seat's recon's stated preference for whichever of the two test shapes
/// fits; this one needs no PATH shim at all. Shared by both
/// [`detect_systemd_status`]'s real-subprocess arm and its tests: they can
/// never silently disagree about what a given `systemctl --user` outcome
/// means.
fn classify_systemctl_output(result: std::io::Result<Output>) -> ManagerStatus {
    match result {
        Ok(output) if output.status.success() => ManagerStatus::Reachable,
        Ok(output) => {
            // The binary exists and ran (this arm is only reached when
            // spawning succeeded) but reported failure — on a
            // `systemctl --user`/`launchctl print gui/$UID` probe that
            // means the manager is present but this session cannot reach
            // it (no user D-Bus/GUI session, the common bare SSH/cron
            // shape per this seat's recon risk 6), not "absent". The
            // stderr — typically a bus-connection failure — is carried
            // verbatim as the most useful evidence.
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            ManagerStatus::PresentNoUserSession { detail: stderr }
        }
        Err(_) => ManagerStatus::Absent,
    }
}

/// `systemctl --user list-units` (a read-only reachability probe — nothing
/// here is enabled or started), via the binary named by
/// [`SYSTEMCTL_BIN_ENV`] (default `systemctl`, the same "read fresh, no
/// caching" discipline `runtime::git`'s injection point uses).
#[cfg(target_os = "linux")]
pub fn detect_systemd_status() -> ManagerStatus {
    let bin = std::env::var(SYSTEMCTL_BIN_ENV).unwrap_or_else(|_| "systemctl".to_string());
    classify_systemctl_output(Command::new(bin).args(["--user", "list-units"]).output())
}

#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
pub fn detect_systemd_status() -> ManagerStatus {
    ManagerStatus::Absent
}

/// `launchctl print gui/$UID` — the recon's own stated macOS equivalent of
/// the systemd probe above: a zero exit means launchd is reachable for this
/// user's GUI session, a nonzero exit with `launchctl` itself runnable means
/// present-but-unreachable (no GUI session — the same shape SSH/cron gives
/// systemd), and a failed spawn means absent.
#[cfg(target_os = "macos")]
pub fn detect_launchd_status(uid: u32) -> ManagerStatus {
    let bin = std::env::var(LAUNCHCTL_BIN_ENV).unwrap_or_else(|_| "launchctl".to_string());
    classify_systemctl_output(
        Command::new(bin)
            .args(["print", &format!("gui/{uid}")])
            .output(),
    )
}

#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
pub fn detect_launchd_status(_uid: u32) -> ManagerStatus {
    ManagerStatus::Absent
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> ServiceSpec {
        ServiceSpec {
            binary_path: PathBuf::from("/home/x/.cargo/bin/sgt"),
            host_runtime_dir: PathBuf::from("/home/x/.local/share/sergeant"),
            path: "/home/x/.cargo/bin:/home/x/.local/bin:/usr/bin:/bin".to_string(),
        }
    }

    /// Golden-file: the exact generated systemd unit, paths substituted.
    /// Reverting `systemd_unit`'s format string in any way that changes
    /// this content fails this test immediately — no service manager
    /// needed.
    #[test]
    fn systemd_unit_golden() {
        let unit = systemd_unit(&spec());
        assert_eq!(
            unit,
            "[Unit]\n\
             Description=Sergeant host daemon\n\
             After=network.target\n\
             \n\
             [Service]\n\
             Type=simple\n\
             ExecStart=/home/x/.cargo/bin/sgt --data-dir /home/x/.local/share/sergeant daemon\n\
             Environment=\"PATH=/home/x/.cargo/bin:/home/x/.local/bin:/usr/bin:/bin\"\n\
             Restart=on-failure\n\
             RestartSec=5\n\
             \n\
             [Install]\n\
             WantedBy=default.target\n"
        );
    }

    /// Golden-file: the exact generated LaunchAgent plist, paths
    /// substituted.
    #[test]
    fn launchd_plist_golden() {
        let plist = launchd_plist(&spec());
        assert_eq!(
            plist,
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\">\n\
             <dict>\n\
             \t<key>Label</key>\n\
             \t<string>com.sergeant.host-daemon</string>\n\
             \t<key>ProgramArguments</key>\n\
             \t<array>\n\
             \t\t<string>/home/x/.cargo/bin/sgt</string>\n\
             \t\t<string>--data-dir</string>\n\
             \t\t<string>/home/x/.local/share/sergeant</string>\n\
             \t\t<string>daemon</string>\n\
             \t</array>\n\
             \t<key>EnvironmentVariables</key>\n\
             \t<dict>\n\
             \t\t<key>PATH</key>\n\
             \t\t<string>/home/x/.cargo/bin:/home/x/.local/bin:/usr/bin:/bin</string>\n\
             \t</dict>\n\
             \t<key>RunAtLoad</key>\n\
             \t<true/>\n\
             \t<key>KeepAlive</key>\n\
             \t<true/>\n\
             \t<key>ProcessType</key>\n\
             \t<string>Background</string>\n\
             </dict>\n\
             </plist>\n"
        );
    }

    /// #275's negative test, named explicitly in the brief: the generated
    /// unit's `Environment=` PATH must contain every entry
    /// `harness::toolchain_path_dirs` names, not just whatever happened to
    /// already be on the composing shell's PATH — this is what makes the
    /// env-stripping hazard structurally unable to recur inside a generated
    /// unit.
    #[test]
    fn systemd_unit_environment_path_contains_every_toolchain_dir() {
        let home = PathBuf::from("/home/x");
        let dirs = crate::harness::toolchain_path_dirs(&home);
        let path = crate::harness::compose_path(
            Some(&std::ffi::OsString::from("/usr/bin:/bin")),
            &dirs,
            |_| true, // pretend every toolchain dir exists on disk
        )
        .to_string_lossy()
        .into_owned();
        let unit = systemd_unit(&ServiceSpec {
            binary_path: PathBuf::from("/home/x/.cargo/bin/sgt"),
            host_runtime_dir: PathBuf::from("/home/x/.local/share/sergeant"),
            path: path.clone(),
        });
        for dir in &dirs {
            assert!(
                unit.contains(&dir.display().to_string()),
                "generated unit's Environment= is missing toolchain dir {} — \
                 got:\n{unit}",
                dir.display()
            );
        }
        // And it must be inside `Environment=`, not merely present anywhere
        // (e.g. leaking into `ExecStart` some other way) — pin the exact
        // line.
        assert!(unit.contains(&format!("Environment=\"PATH={path}\"")));
    }

    /// Same negative test, plist arm.
    #[test]
    fn launchd_plist_environment_variables_path_contains_every_toolchain_dir() {
        let home = PathBuf::from("/home/x");
        let dirs = crate::harness::toolchain_path_dirs(&home);
        let path = crate::harness::compose_path(
            Some(&std::ffi::OsString::from("/usr/bin:/bin")),
            &dirs,
            |_| true,
        )
        .to_string_lossy()
        .into_owned();
        let plist = launchd_plist(&ServiceSpec {
            binary_path: PathBuf::from("/home/x/.cargo/bin/sgt"),
            host_runtime_dir: PathBuf::from("/home/x/.local/share/sergeant"),
            path: path.clone(),
        });
        for dir in &dirs {
            assert!(
                plist.contains(&dir.display().to_string()),
                "generated plist's EnvironmentVariables is missing toolchain dir {} — \
                 got:\n{plist}",
                dir.display()
            );
        }
    }

    #[test]
    fn generation_is_deterministic_across_calls() {
        assert_eq!(systemd_unit(&spec()), systemd_unit(&spec()));
        assert_eq!(launchd_plist(&spec()), launchd_plist(&spec()));
    }

    #[test]
    fn install_paths_are_under_the_native_per_user_locations() {
        let home = Path::new("/home/x");
        assert_eq!(
            systemd_unit_path(home),
            PathBuf::from("/home/x/.config/systemd/user/sergeant-daemon.service")
        );
        let home = Path::new("/Users/x");
        assert_eq!(
            launchd_plist_path(home),
            PathBuf::from("/Users/x/Library/LaunchAgents/com.sergeant.host-daemon.plist")
        );
    }

    #[test]
    fn classify_success_is_reachable() {
        let output = Command::new("true").output();
        assert_eq!(classify_systemctl_output(output), ManagerStatus::Reachable);
    }

    #[test]
    fn classify_spawn_failure_is_absent() {
        let result = Command::new("sgt-w4c-nonexistent-binary-xyz").output();
        assert_eq!(classify_systemctl_output(result), ManagerStatus::Absent);
    }

    #[test]
    fn classify_no_bus_session_is_present_no_user_session() {
        // A scripted `sh -c` stand-in for `systemctl --user list-units`
        // against a stub that fails the way a real bus-less session does —
        // the fake-binary shape this seat's brief flags as the fitting
        // precedent for exercising a real subprocess boundary.
        let output = Command::new("sh")
            .args([
                "-c",
                "echo 'Failed to connect to bus: No such file or directory' >&2; exit 1",
            ])
            .output();
        assert_eq!(
            classify_systemctl_output(output),
            ManagerStatus::PresentNoUserSession {
                detail: "Failed to connect to bus: No such file or directory".to_string()
            }
        );
    }

    /// [`detect_systemd_status`]'s real-subprocess arm, exercised through
    /// [`SYSTEMCTL_BIN_ENV`] pointed at a scripted fake — the PATH-shimmed
    /// injection precedent (`runtime::git::GIT_BIN_ENV`) applied here.
    /// Process-global env var: this test owns its own value (set then
    /// immediately restored) rather than running in a shared parallel
    /// fixture, mirroring `runtime::git`'s own caveat about
    /// `tests/c11_injectable_git.rs` needing its own process.
    #[test]
    #[cfg(target_os = "linux")]
    fn detect_systemd_status_uses_the_injected_binary() {
        // SAFETY: this test only mutates an env var it reads back inside
        // the same synchronous call below, restoring it before returning —
        // no other code observes the mutation in between.
        unsafe {
            std::env::set_var(SYSTEMCTL_BIN_ENV, "true");
        }
        let status = detect_systemd_status();
        unsafe {
            std::env::remove_var(SYSTEMCTL_BIN_ENV);
        }
        assert_eq!(status, ManagerStatus::Reachable);
    }
}
