//! `sgt <harness>` passthrough acceptance (ADR 0006, D2): `sgt claude` (and
//! `codex`/`opencode`/`goose`, which share the same code path) compose an
//! environment, bind the estate, then **exec** — replacing the `sgt`
//! process image, never forking and supervising it.
//!
//! These tests never spawn a daemon (the stub the harness verb execs into
//! just records what it was handed and exits), so none of them need
//! `support::DataDir`'s reaping guard — there is nothing here for it to
//! reap.

use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use tempfile::TempDir;

mod support;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

/// Write an executable stand-in for a harness (`claude`, `codex`, ...) that
/// records its own pid, argv, and a few named env vars to `record`, one
/// fact per line, then exits 0. The pid line is the crux of the exec test:
/// a shebang script's `$$` is the pid the kernel handed the *original*
/// process image — unchanged by `execve`, whether directly or through the
/// `#!/bin/sh` interpreter indirection — so it equals whatever pid the test
/// process observed for the `sgt` child it spawned if and only if `sgt`
/// really replaced its own image rather than forking a supervised child
/// (which would carry a *different* pid).
fn write_stub(dir: &Path, name: &str, record: &Path) -> String {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(name);
    // `--version` is handled separately, and never recorded as a launch:
    // `support::wait_until_executable` below probes exactly this stub with
    // `--version` to wait out a fresh script's `ETXTBSY` window (#83), and
    // that self-check must not count as a launch this test then asserts
    // against — the same shape `stub_claude` in `tests/m6_surfaces.rs` and
    // `StubClaude` in `tests/m4_backends.rs` already use.
    let script = format!(
        "#!/bin/sh\n\
         case \"$1\" in\n  \
           --version) echo stub;;\n  \
           *)\n    \
             {{\n      \
               printf 'pid %s\\n' \"$$\";\n      \
               for arg in \"$@\"; do printf 'arg %s\\n' \"$arg\"; done;\n      \
               printf 'env PATH=%s\\n' \"$PATH\";\n      \
               printf 'env SGT_DATA_DIR=%s\\n' \"${{SGT_DATA_DIR-<unset>}}\";\n      \
               printf 'end\\n';\n    \
             }} >> \"{record}\";;\n\
         esac\n",
        record = record.display(),
    );
    std::fs::write(&path, script).expect("write stub");
    let mut permissions = std::fs::metadata(&path).expect("stat stub").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).expect("chmod stub");
    support::wait_until_executable(&path);
    path.display().to_string()
}

/// One recorded launch of the stub.
#[derive(Debug, Default)]
struct Launch {
    pid: Option<u32>,
    args: Vec<String>,
    env: std::collections::BTreeMap<String, String>,
}

fn read_launch(record: &Path) -> Launch {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Ok(text) = std::fs::read_to_string(record)
            && text.lines().any(|l| l == "end")
        {
            let mut launch = Launch::default();
            for line in text.lines() {
                match line.split_once(' ') {
                    Some(("pid", value)) => launch.pid = value.parse().ok(),
                    Some(("arg", value)) => launch.args.push(value.to_string()),
                    Some(("env", value)) => {
                        if let Some((name, value)) = value.split_once('=') {
                            launch.env.insert(name.to_string(), value.to_string());
                        }
                    }
                    _ => {}
                }
            }
            return launch;
        }
        assert!(
            Instant::now() < deadline,
            "no launch recorded at {record:?}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// D2: `sgt claude` **execs**, it does not fork-and-supervise. This is the
/// load-bearing boundary — `NORTH-STAR.md`'s "Never" list forbids
/// "reconstructed tmux-era supervision" — and it is provable rather than
/// assumed: an `execve` replaces the calling process's image without
/// changing its pid, while fork-then-exec hands the child a *new* pid. The
/// pid the test observes for the spawned `sgt` process must equal the pid
/// the stub recorded for itself once `sgt` has execed into it.
///
/// guard-map: replacing `command.exec()` with
/// `command.spawn()` + wait (fork-and-supervise) makes this fail — the
/// stub's recorded pid becomes a *child* of the pid `sgt` reports, not the
/// same number.
#[test]
fn sgt_claude_execs_the_child_pid_equals_the_parent_sgt_pid() {
    let bin = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    // A HOME with no `.cargo/bin`/`.local/bin` of its own: PATH enrichment
    // must have nothing to prepend, so the stub directory below (listed
    // first on PATH) is what actually resolves as `claude` — not whatever
    // real harness happens to live in *this* host's own HOME.
    let home = TempDir::new().expect("tempdir");
    let record = bin.path().join("record.txt");
    write_stub(bin.path(), "claude", &record);

    let mut child = Command::new(SGT)
        .arg("--data-dir")
        .arg(data.path())
        .arg("claude")
        .env("PATH", format!("{}:/usr/bin:/bin", bin.path().display()))
        .env("HOME", home.path())
        .spawn()
        .expect("spawn sgt claude");
    let sgt_pid = child.id();

    let launch = read_launch(&record);
    assert_eq!(
        launch.pid,
        Some(sgt_pid),
        "the stub's own pid must equal the pid the test observed for `sgt claude` — anything \
         else means a child process was forked instead of an exec replacing sgt's image"
    );

    let status = child.wait().expect("wait");
    assert!(status.success(), "the exec'd stub exits 0: {status:?}");
}

/// D2's literal example: `sgt claude -- --model opus` reaches `claude` as
/// `--model opus`, unexamined — not reordered, not filtered, not merged with
/// any of sgt's own flags. `--data-dir` (sgt's own global flag) must not
/// leak into the harness's argv either.
///
/// guard-map: passing `args` through a filter, or forgetting `last = true`
/// so clap tries to parse harness flags as sgt's own, fails this.
#[test]
fn arguments_after_the_separator_reach_the_harness_verbatim() {
    let bin = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let home = TempDir::new().expect("tempdir");
    let record = bin.path().join("record.txt");
    write_stub(bin.path(), "claude", &record);

    let mut child = Command::new(SGT)
        .arg("--data-dir")
        .arg(data.path())
        .arg("claude")
        .arg("--")
        .arg("--model")
        .arg("opus")
        .env("PATH", format!("{}:/usr/bin:/bin", bin.path().display()))
        .env("HOME", home.path())
        .spawn()
        .expect("spawn sgt claude");
    child.wait().expect("wait");

    let launch = read_launch(&record);
    assert_eq!(
        launch.args,
        vec!["--model".to_string(), "opus".to_string()],
        "only the harness-bound args may reach the stub, in order, untouched"
    );
}

/// ADR 0006's environment half: PATH is deliberately enriched with the
/// toolchain directories that exist on disk (`~/.cargo/bin`, `~/.local/bin`)
/// and `SGT_DATA_DIR` is bound explicitly to the data dir this invocation
/// resolved — both reaching the exec'd harness, at the same time, in the
/// composed PATH the child actually receives.
///
/// guard-map: dropping either half of `harness::prepare`'s composition (the
/// `command.env("PATH", ...)` block or the trailing `command.env(DATA_DIR_ENV,
/// data_dir)`) makes exactly one of these two assertions fail — proving the
/// two are composed independently rather than one accidentally covering for
/// the other.
#[test]
fn the_composed_environment_reaches_the_exec_d_harness() {
    let bin = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let home = TempDir::new().expect("tempdir");
    let cargo_bin = home.path().join(".cargo").join("bin");
    std::fs::create_dir_all(&cargo_bin).expect("mkdir cargo bin");
    let record = bin.path().join("record.txt");
    write_stub(bin.path(), "claude", &record);

    let mut child = Command::new(SGT)
        .arg("--data-dir")
        .arg(data.path())
        .arg("claude")
        .env("PATH", format!("{}:/usr/bin:/bin", bin.path().display()))
        .env("HOME", home.path())
        .spawn()
        .expect("spawn sgt claude");
    child.wait().expect("wait");

    let launch = read_launch(&record);
    let path = launch.env.get("PATH").expect("PATH recorded");
    assert!(
        std::env::split_paths(path).any(|p| p == cargo_bin),
        "PATH must be enriched with the toolchain dir that exists on disk: {path}"
    );
    assert_eq!(
        launch.env.get("SGT_DATA_DIR").map(String::as_str),
        Some(data.path().display().to_string()).as_deref(),
        "SGT_DATA_DIR must bind explicitly to the data dir this invocation resolved: {:?}",
        launch.env.get("SGT_DATA_DIR")
    );
}
