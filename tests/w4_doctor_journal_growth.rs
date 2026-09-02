//! W4 §2: `sgt doctor`'s `journal_growth` row — segments, floor, retained
//! count against the cap, prune stall, and startup rebuild time, all
//! read-only and runnable with **no live daemon** (recon correction 7:
//! doctor's journal reads are lock-free by design).
//!
//! Every case here builds its journal, shuts any in-process daemon down (an
//! in-process daemon never needs `support::DataDir`'s reaper — it is never
//! detached, `tests/m2_daemon_api.rs`'s own convention), and then reads the
//! row through the real `sgt` binary's `--json doctor`, matching
//! `tests/m8_estate_cli.rs`'s own `run()`/`Output` shape exactly — the
//! sanctioned way for an integration test to reach `cli::doctor::Report`,
//! which is `pub(crate)` and not otherwise reachable from outside the crate.

use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig, DaemonHandle};
use sergeant_rs::domain::event::{EventDraft, EventSource};
use sergeant_rs::runtime::journal::Journal;

mod support;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

fn ulid() -> String {
    ulid::Ulid::generate().to_string()
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("client")
}

fn write_one_stage_workflow(root: &Path) {
    let dir = root.join(".sergeant/workflows/solo-stage");
    std::fs::create_dir_all(&dir).expect("workflow dir");
    std::fs::write(
        dir.join("workflow.toml"),
        "[workflow]\nname = \"solo-stage\"\nversion = \"1\"\nstages = [\"00-only\"]\n",
    )
    .expect("workflow.toml");
    std::fs::create_dir_all(dir.join("00-only")).expect("stage dir");
    std::fs::write(dir.join("00-only").join("CONTEXT.md"), "context").expect("CONTEXT.md");
}

async fn start_with_retention(
    data_dir: &Path,
    _estate_root: &Path,
    retention: u32,
    script: impl IntoIterator<Item = FakeStep>,
) -> DaemonHandle {
    let fake = FakeBackend::scripted(FAKE_BACKEND_NAME, script);
    let registry = BackendRegistry::new().with(Arc::new(fake));
    daemon::start_with(
        data_dir,
        DaemonConfig {
            backends: Arc::new(registry),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            segment_max_bytes: Some(256),
            retention: Some(retention),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start")
}

async fn submit(
    http: &reqwest::Client,
    handle: &DaemonHandle,
    root: &Path,
    command_id: &str,
    intent: &str,
) -> Value {
    let resp = http
        .post(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&json!({
            "command_id": command_id,
            "intent": intent,
            // D4: the estate this submission addresses.
            "estate_root": root,
            "origin": {"client": "cli", "cwd": root},
        }))
        .send()
        .await
        .expect("submit");
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::CREATED,
        "{:?}",
        resp.text().await
    );
    resp.json().await.expect("json")
}

async fn wait_until<F: Fn(&Value) -> bool>(http: &reqwest::Client, handle: &DaemonHandle, ok: F) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        let system: Value = http
            .get(format!("{}/v1/work", handle.endpoint))
            .bearer_auth(&handle.token)
            .send()
            .await
            .expect("list")
            .json()
            .await
            .expect("json");
        if ok(&system) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("condition never became true");
}

async fn wait_until_all_settled(http: &reqwest::Client, handle: &DaemonHandle) {
    wait_until(http, handle, |system| {
        system["works"]
            .as_array()
            .map(|rows| {
                rows.iter()
                    .all(|w| w["state"] == "completed" || w["state"] == "failed")
            })
            .unwrap_or(false)
    })
    .await;
}

/// Run `sgt --data-dir <data_dir> --json doctor` with `cwd`, and return the
/// parsed `--json` report (the whole `Report::to_json()` shape: `healthy`,
/// `data_dir`, `checks`). Never spawns a daemon — `doctor` is one of the
/// five commands documented not to (CONTRIBUTING.md).
fn doctor_json(cwd: &Path, data_dir: &Path) -> Value {
    let output = Command::new(SGT)
        .current_dir(cwd)
        .arg("--data-dir")
        .arg(data_dir)
        .args(["--json", "doctor"])
        .output()
        .expect("run sgt doctor");
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout is not JSON ({e}): stdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn journal_growth_row(report: &Value) -> &Value {
    report["checks"]
        .as_array()
        .expect("checks array")
        .iter()
        .find(|c| c["name"] == "journal_growth")
        .unwrap_or_else(|| panic!("no journal_growth row in report: {report}"))
}

// ------------------------------------------------------------------ tests

#[tokio::test]
async fn journal_growth_reports_segments_floor_and_retained_count_on_a_healthy_estate() {
    let data = TempDir::new().expect("tempdir");
    let root = TempDir::new().expect("tempdir");
    support::scaffold_solo_estate(root.path(), "solo");
    write_one_stage_workflow(root.path());
    let http = client();

    const TOTAL: usize = 4;
    let script: Vec<FakeStep> = (0..TOTAL).map(|_| FakeStep::complete()).collect();
    // `DaemonConfig::retention` is a test-rig-only, in-memory override
    // (`PolicySource::Config`) — it is never persisted anywhere doctor (run
    // standalone, in a separate process, after this daemon has already
    // exited) could read it back from. Doctor resolves retention from
    // `[estate] retention` in the manifest, or the built-in default; this
    // estate declares neither, so the expected cap is `DEFAULT_RETENTION`
    // (1000), not whatever this now-stopped daemon happened to run under.
    let handle = start_with_retention(data.path(), root.path(), 1_000_000, script).await;
    for n in 0..TOTAL {
        submit(&http, &handle, root.path(), &ulid(), &format!("work {n}")).await;
    }
    wait_until_all_settled(&http, &handle).await;
    handle.shutdown().await;

    let report = doctor_json(root.path(), data.path());
    let row = journal_growth_row(&report);
    assert_eq!(row["status"], "ok", "{row}");
    let detail = row["detail"].as_str().expect("detail string");
    assert!(detail.contains("floor seq 1"), "{detail}");
    assert!(
        detail.contains(&format!("{TOTAL}/1000 works retained")),
        "{detail}"
    );
    assert!(detail.contains("no prune stall"), "{detail}");
    assert!(detail.contains("segments"), "{detail}");
}

#[tokio::test]
async fn journal_growth_on_a_fresh_data_dir_is_ok_with_nothing_grown() {
    let data = TempDir::new().expect("tempdir");
    let cwd = TempDir::new().expect("tempdir");

    let report = doctor_json(cwd.path(), data.path());
    let row = journal_growth_row(&report);
    assert_eq!(row["status"], "ok", "{row}");
    assert_eq!(row["detail"], "no journal yet — nothing has grown", "{row}");
}

#[tokio::test]
async fn journal_growth_warns_when_a_prune_is_stalled_and_names_the_blocking_work() {
    let data = TempDir::new().expect("tempdir");
    let root = TempDir::new().expect("tempdir");
    support::scaffold_solo_estate(root.path(), "solo");
    write_one_stage_workflow(root.path());
    let http = client();

    // Four ordinary Works that complete, one that parks in `needs_input` and
    // never settles — the shape `tests/w3_prune_engine.rs` uses for a
    // never-terminal Work pinning the horizon (§7.3's own blocking-work
    // predicate: the *only* non-retired-whole Work is always the blocker,
    // whatever the retention cap is set to — see `src/runtime/prune.rs`'s
    // `blocking_work`).
    let script = vec![
        FakeStep::complete(),
        FakeStep::complete(),
        FakeStep::needs_input("waiting for a human"),
        FakeStep::complete(),
        FakeStep::complete(),
    ];
    let handle = start_with_retention(data.path(), root.path(), 1_000_000, script).await;
    for n in 0..5 {
        submit(&http, &handle, root.path(), &ulid(), &format!("work {n}")).await;
    }
    wait_until(&http, &handle, |system| {
        system["works"].as_array().is_some_and(|rows| {
            rows.iter().all(|w| {
                w["state"] == "completed" || w["state"] == "failed" || w["state"] == "needs_input"
            }) && rows.iter().any(|w| w["state"] == "needs_input")
        })
    })
    .await;
    // Read the stuck Work's own id back from the daemon rather than assuming
    // script-step order matches submission order.
    let system: Value = http
        .get(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("list")
        .json()
        .await
        .expect("json");
    let stuck_id = system["works"]
        .as_array()
        .expect("works array")
        .iter()
        .find(|w| w["state"] == "needs_input")
        .expect("exactly one Work must be parked in needs_input")["id"]
        .as_str()
        .expect("id")
        .to_string();
    handle.shutdown().await;

    let report = doctor_json(root.path(), data.path());
    let row = journal_growth_row(&report);
    assert_eq!(row["status"], "warn", "{row}");
    let detail = row["detail"].as_str().expect("detail string");
    assert!(
        detail.contains(&format!("prune stalled on work {stuck_id}")),
        "{detail}"
    );
    let remedy = row["remedy"].as_str().expect("remedy present on warn");
    assert!(
        remedy.contains("no override exists"),
        "Q7: the remedy must never offer a way to lower the predicate: {remedy}"
    );
}

/// A pure payload-reading test (per `w4-spec.md` §2.5): no real slow
/// startup, just a hand-constructed `daemon.started` event.
#[tokio::test]
async fn journal_growth_warns_at_ten_seconds_and_fails_at_thirty_on_daemon_started() {
    for (rebuild_ms, expected_status) in [(15_000u64, "warn"), (35_000u64, "fail")] {
        let data = TempDir::new().expect("tempdir");
        {
            let mut journal = Journal::open(data.path()).expect("open journal");
            journal
                .append(EventDraft::new(
                    EventSource::new("daemon", "sergeant"),
                    "daemon.started",
                    json!({"rebuild_duration_ms": rebuild_ms}),
                ))
                .expect("append daemon.started");
        }
        let cwd = TempDir::new().expect("tempdir");
        let report = doctor_json(cwd.path(), data.path());
        let row = journal_growth_row(&report);
        assert_eq!(
            row["status"], expected_status,
            "rebuild_duration_ms={rebuild_ms}: {row}"
        );
        let detail = row["detail"].as_str().expect("detail string");
        assert!(
            detail.contains("last rebuild"),
            "rebuild_duration_ms={rebuild_ms}: {detail}"
        );
    }
}

#[tokio::test]
async fn journal_growth_resolves_retention_from_the_manifest_and_falls_back_to_default() {
    // Deliberately does **not** use `support::scaffold_estate` — that helper
    // clones a real repo onto disk for every declared `[[repo]]`, which
    // would not distinguish `Estate::from_config` (strict, resolves every
    // repo through git) from `Estate::from_config_structural` (schema-only).
    // A hand-written manifest with a declared repo *absent* from disk is
    // exactly the shape `Estate::from_config_structural`'s own doc comment
    // names ("a git clone'd estate ... declares repos ... with no repos/ on
    // disk at all") — the case this file's deviation from the spec's own
    // `Estate::from_config` snippet exists to answer correctly.
    for (manifest, expected_suffix) in [
        (
            "[estate]\nname = \"solo\"\nretention = 200\n\n[[repo]]\nname = \"solo\"\n",
            "/200 works retained",
        ),
        (
            "[estate]\nname = \"solo\"\n\n[[repo]]\nname = \"solo\"\n",
            "/1000 works retained",
        ),
    ] {
        let data = TempDir::new().expect("tempdir");
        let root = TempDir::new().expect("tempdir");
        std::fs::write(root.path().join("sergeant.toml"), manifest).expect("write manifest");
        {
            let mut journal = Journal::open(data.path()).expect("open journal");
            journal
                .append(EventDraft::new(
                    EventSource::new("daemon", "sergeant"),
                    "daemon.started",
                    json!({}),
                ))
                .expect("append seed event");
        }
        let report = doctor_json(root.path(), data.path());
        let row = journal_growth_row(&report);
        let detail = row["detail"].as_str().expect("detail string");
        assert!(
            detail.contains(expected_suffix),
            "manifest={manifest:?}: {detail}"
        );
    }
}

#[tokio::test]
async fn journal_growth_survives_a_real_prune_with_no_cache_at_all() {
    let data = TempDir::new().expect("tempdir");
    let root = TempDir::new().expect("tempdir");
    support::scaffold_solo_estate(root.path(), "solo");
    write_one_stage_workflow(root.path());
    let http = client();

    const RETENTION: u32 = 2;
    const TOTAL: usize = 6;
    {
        let script: Vec<FakeStep> = (0..TOTAL).map(|_| FakeStep::complete()).collect();
        let handle = start_with_retention(data.path(), root.path(), 1_000_000, script).await;
        for n in 0..TOTAL {
            submit(&http, &handle, root.path(), &ulid(), &format!("work {n}")).await;
        }
        wait_until_all_settled(&http, &handle).await;
        handle.shutdown().await;
    }
    // Life 2's own startup-triggered prune (W3 §10.3) actually runs — and,
    // per its own completion (`prune::run`'s step 6, "remove any existing
    // [cache]... the next clean start's own write point rebuilds a fresh v2
    // cache"), *deletes* whatever cache life 2's own start point had just
    // written.
    let handle =
        start_with_retention(data.path(), root.path(), RETENTION, Vec::<FakeStep>::new()).await;
    handle.shutdown().await;

    // I-W3-7: no cache at all survives the prune — `journal_growth` must
    // still be honest, folded straight from a floor replay.
    let cache = data.path().join("projections").join("floor-state.json");
    assert!(
        !cache.exists(),
        "a prune's own completion must have removed the cache already"
    );

    let report = doctor_json(root.path(), data.path());
    let row = journal_growth_row(&report);
    assert_eq!(row["status"], "ok", "{row}");
    let detail = row["detail"].as_str().expect("detail string");
    // Retained count against the *default* cap (1000) — this estate declares
    // no `[estate] retention`, and (as above) doctor cannot see the
    // now-exited daemon's own `DaemonConfig::retention` test override.
    assert!(
        detail.contains(&format!("{RETENTION}/1000 works retained")),
        "even with the cache deleted, the retained count must be honest: {detail}"
    );
    assert!(
        !detail.contains("floor seq 1,"),
        "a real prune must have moved the floor: {detail}"
    );

    // Life 3 — the real sequence this test used to dodge.
    //
    // It originally stopped at life 2 and said so in a comment: a further
    // restart on an already-pruned, cache-less journal hit a `Plan::Full`
    // cache-miss gap in `runtime::startup` (a seq-1-seeded `Projection::new`
    // fed `replay_from_floor`'s floor-seeded events — `SeqMismatch`), which
    // W4's non-goals put out of reach at the time. That gap is now closed
    // (`Plan::seed_registry`'s `Full` arm resumes from `floor_seq - 1`, the
    // same floor-aware seeding `rebuild_registry` below already used), and
    // this is the *ordinary* next start of any estate that has ever pruned,
    // so the fixture takes it rather than documenting the dodge.
    //
    // The live-daemon half of this — that life 3 comes up and serves —
    // belongs to `tests/w3_prune_engine.rs`'s
    // `a_start_after_a_prune_with_no_cache_still_serves`. What is doctor's
    // own to prove is that the row stays honest *across* that restart.
    let handle =
        start_with_retention(data.path(), root.path(), RETENTION, Vec::<FakeStep>::new()).await;
    handle.shutdown().await;

    let report = doctor_json(root.path(), data.path());
    let row = journal_growth_row(&report);
    assert_eq!(row["status"], "ok", "after a post-prune restart: {row}");
    let detail = row["detail"].as_str().expect("detail string");
    assert!(
        detail.contains(&format!("{RETENTION}/1000 works retained")),
        "a restart over the pruned journal must not change the retained count: {detail}"
    );
    assert!(
        !detail.contains("floor seq 1,"),
        "the floor must still be above 1 after the restart: {detail}"
    );
}
