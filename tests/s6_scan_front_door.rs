//! S6 scan front door: `sgt intelligence scan` must not report failure on
//! success.
//!
//! **The defect this suite exists for, measured before any of it was
//! written.** On the `sergeant-rs-workspace` estate — five declared sources
//! (two `[[repo]]`, three `[[knowledge]]`) — the merged build answered:
//!
//! ```text
//! $ sgt --data-dir <scratch> -C <estate> intelligence scan
//! sgt: error sending request for url (http://127.0.0.1:36973/v1/intelligence/scan)
//! ```
//!
//! after 10.05 s, exit 1, *every time*, while the daemon carried on
//! indexing (`intelligence status` gained sources minutes later, and the
//! daemon process stayed at ~9% CPU for minutes after the client had given
//! up). The scan was synchronous over one HTTP request whose client
//! timeout is ten seconds; the client always errored and the work always
//! succeeded, and nothing told the operator which.
//!
//! What each test here holds:
//!
//! * [`the_trigger_accepts_and_names_the_scan_instead_of_answering_with_the_whole_report`]
//!   — the shape change: `202` plus a scan id, never a report the caller
//!   has to hold a connection open for.
//! * [`per_source_progress_is_visible_while_the_scan_is_still_running`] —
//!   the acceptance returns in a fraction of the scan's own duration, and
//!   a finished source is readable while later ones are still going.
//! * [`completion_is_reported_by_the_scan_itself_not_counted_from_another_command`]
//!   — requirement 2: the scan says it is done.
//! * [`an_unknown_scan_id_is_refused_by_name_rather_than_answered_with_a_fabricated_completion`]
//!   — the honest negative (A1 §15: missing capability is never
//!   represented as successful empty evidence).
//! * [`the_journal_records_the_scan_that_started_and_the_scan_that_completed`]
//!   — requirement 4 and A1-01: the journal is the authority, and it
//!   recorded nothing about a scan before this wave.
//! * [`the_verb_itself_exits_zero_and_prints_a_row_for_every_source`] — the
//!   front door, through the real binary: the exit code and the printed
//!   rows an operator and a script actually see.

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig};
use sergeant_rs::domain::event::Event;
use sergeant_rs::domain::source::{
    KIND_INTELLIGENCE_SCAN_COMPLETED, KIND_INTELLIGENCE_SCAN_STARTED,
};
use sergeant_rs::runtime::journal::Journal;

mod support;
use support::DataDir;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

/// The three sources every estate in this suite declares, in the order the
/// scan undertakes them.
const SOURCES: [&str; 3] = ["notes", "bulk", "tail"];

/// An estate with **three** `[[knowledge]]` sources, one of which is big.
///
/// Multi-source is the whole point: the defect only appears when a scan
/// takes longer than one request timeout, which one small fixture source
/// never does. `bulk` is a copy of this build's own `src/` tree — real
/// files, real parse work, seconds of it — so the acceptance and the
/// completion are genuinely far apart in time, exactly as they are on a
/// real estate.
fn scaffold_estate(root: &Path) {
    std::fs::create_dir_all(root).expect("estate root");
    for name in ["notes", "tail"] {
        let dir = root.join(name);
        std::fs::create_dir_all(&dir).expect("source dir");
        std::fs::write(
            dir.join("note.md"),
            format!("# {name}\n\nOne small file.\n"),
        )
        .expect("write note");
    }
    let bulk = root.join("bulk");
    copy_dir_recursive(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &bulk);
    let manifest = "[estate]\nname = \"s6-scan\"\n\n\
         [[knowledge]]\nname = \"notes\"\npath = \"notes\"\n\n\
         [[knowledge]]\nname = \"bulk\"\npath = \"bulk\"\n\n\
         [[knowledge]]\nname = \"tail\"\npath = \"tail\"\n";
    std::fs::write(root.join("sergeant.toml"), manifest).expect("write sergeant.toml");
}

fn copy_dir_recursive(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("mkdir");
    for entry in std::fs::read_dir(from).expect("read_dir") {
        let entry = entry.expect("entry");
        let dest = to.join(entry.file_name());
        let file_type = entry.file_type().expect("file_type");
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest);
        } else if file_type.is_file() {
            std::fs::copy(entry.path(), &dest).expect("copy file");
        }
    }
}

async fn start_daemon(data: &DataDir) -> daemon::DaemonHandle {
    let fake = FakeBackend::scripted(FAKE_BACKEND_NAME, [FakeStep::hang()]);
    daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(BackendRegistry::new().with(Arc::new(fake))),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            claude: None,
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start")
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("client")
}

async fn post_scan(
    http: &reqwest::Client,
    handle: &daemon::DaemonHandle,
    estate_root: &Path,
) -> (reqwest::StatusCode, Value) {
    let response = http
        .post(format!("{}/v1/intelligence/scan", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&json!({
            "command_id": ulid::Ulid::generate().to_string(),
            "estate_root": estate_root,
        }))
        .send()
        .await
        .expect("scan request");
    let status = response.status();
    let body: Value = response.json().await.expect("json body");
    (status, body)
}

async fn poll_scan(
    http: &reqwest::Client,
    handle: &daemon::DaemonHandle,
    scan_id: &str,
) -> (reqwest::StatusCode, Value) {
    let response = http
        .get(format!(
            "{}/v1/intelligence/scan/{scan_id}",
            handle.endpoint
        ))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("scan status request");
    let status = response.status();
    let body: Value = response.json().await.expect("json body");
    (status, body)
}

/// The shape change itself: the trigger accepts, names the scan, and says
/// what it undertook to cover — it does not hold the connection open for
/// the whole scan and answer with the report.
///
/// Every assertion here fails against the synchronous trigger this
/// replaced: that one answered `200` with `scanned` already full and no
/// `scan_id` at all.
#[tokio::test]
async fn the_trigger_accepts_and_names_the_scan_instead_of_answering_with_the_whole_report() {
    let estate = TempDir::new().expect("estate");
    scaffold_estate(estate.path());
    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let (status, accepted) = post_scan(&http, &handle, estate.path()).await;
    assert_eq!(
        status, 202,
        "an accepted scan is accepted, not answered: {accepted}"
    );
    let scan_id = accepted["scan_id"]
        .as_str()
        .unwrap_or_else(|| panic!("the acceptance must name the scan it started: {accepted}"));
    assert!(!scan_id.is_empty(), "{accepted}");
    assert_eq!(accepted["state"], "running", "{accepted}");
    assert_eq!(accepted["total_sources"], 3, "{accepted}");
    assert_eq!(accepted["completed_sources"], 0, "{accepted}");
    assert_eq!(
        accepted["scanned"],
        json!([]),
        "nothing has finished at acceptance time, and claiming otherwise would be the same \
         class of lie as reporting a transport failure on a scan that succeeded: {accepted}"
    );
    let named: Vec<&str> = accepted["sources"]
        .as_array()
        .expect("sources")
        .iter()
        .map(|s| s["source"].as_str().unwrap_or("?"))
        .collect();
    for source in SOURCES {
        assert!(
            named.contains(&source),
            "the acceptance must name every source the scan undertook, so a client can say \
             2 of 3 rather than 2 so far: {accepted}"
        );
    }

    support::scan_to_completion(
        &http,
        &handle.endpoint,
        &handle.token,
        &json!({"command_id": ulid::Ulid::generate().to_string(), "estate_root": estate.path()}),
    )
    .await;
    handle.shutdown().await;
}

/// Requirement 3, and the measurement that makes requirement 1 true: the
/// acceptance comes back in a fraction of the scan's own duration, and a
/// source that has finished is readable while the rest are still going.
///
/// This is the test that would have caught the original defect. The scan
/// of a real `src/` tree takes seconds to minutes; the accept must not.
#[tokio::test]
async fn per_source_progress_is_visible_while_the_scan_is_still_running() {
    let estate = TempDir::new().expect("estate");
    scaffold_estate(estate.path());
    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let accepted_at = Instant::now();
    let (status, accepted) = post_scan(&http, &handle, estate.path()).await;
    let accept_latency = accepted_at.elapsed();
    assert_eq!(status, 202, "{accepted}");
    let scan_id = accepted["scan_id"].as_str().expect("scan id").to_string();

    // Poll fast enough to see the middle of the scan, not only its ends.
    let mut saw_partial_progress = false;
    let mut completed = None;
    let deadline = Instant::now() + support::SCAN_BUDGET;
    while Instant::now() < deadline {
        let (status, progress) = poll_scan(&http, &handle, &scan_id).await;
        assert_eq!(status, 200, "{progress}");
        let done = progress["completed_sources"].as_u64().unwrap_or(0);
        if progress["state"] == "running" && (1..3).contains(&done) {
            saw_partial_progress = true;
        }
        if progress["state"] == "completed" {
            completed = Some(progress);
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let completed = completed.expect("the scan must reach completion within the budget");
    let scan_duration = accepted_at.elapsed();

    assert!(
        saw_partial_progress,
        "a finished source must be readable while later sources are still running — that is \
         what makes minutes-long work watchable rather than opaque; the scan took {scan_duration:?} \
         and no poll ever saw a partial count: {completed}"
    );
    assert!(
        accept_latency * 4 < scan_duration,
        "the acceptance must not wait for the scan: accepted in {accept_latency:?} of a \
         {scan_duration:?} scan"
    );
    handle.shutdown().await;
}

/// Requirement 2: completion is knowable from the scan's own surface, and
/// every source it named has an outcome by then — not counted out of
/// `sgt intelligence status`'s row list.
#[tokio::test]
async fn completion_is_reported_by_the_scan_itself_not_counted_from_another_command() {
    let estate = TempDir::new().expect("estate");
    scaffold_estate(estate.path());
    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let (status, final_state) = support::scan_to_completion(
        &http,
        &handle.endpoint,
        &handle.token,
        &json!({"command_id": ulid::Ulid::generate().to_string(), "estate_root": estate.path()}),
    )
    .await;
    assert_eq!(status, 200, "{final_state}");
    assert_eq!(final_state["state"], "completed", "{final_state}");
    assert_eq!(final_state["total_sources"], 3, "{final_state}");
    assert_eq!(final_state["completed_sources"], 3, "{final_state}");
    assert!(
        final_state["completed"].as_str().is_some(),
        "a completed scan carries the instant it finished: {final_state}"
    );
    let rows = final_state["scanned"].as_array().expect("scanned");
    for source in SOURCES {
        let row = rows
            .iter()
            .find(|r| r["source"] == source)
            .unwrap_or_else(|| panic!("no row for {source}: {final_state}"));
        assert_eq!(row["kind"], "local_knowledge", "{row}");
        assert_eq!(row["outcome"], "recorded", "{row}");
        assert!(
            row["coverage"]["indexed"].as_u64().unwrap_or(0) >= 1,
            "the row reports the source's own coverage counts, never a guess: {row}"
        );
    }
    handle.shutdown().await;
}

/// The honest negative. A daemon that never accepted this scan — or that
/// restarted since — says so, with a named code; it does not answer
/// `completed` for a scan it knows nothing about.
#[tokio::test]
async fn an_unknown_scan_id_is_refused_by_name_rather_than_answered_with_a_fabricated_completion() {
    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let (status, body) = poll_scan(&http, &handle, "01M0000000000000000000000A").await;
    assert_eq!(status, 404, "{body}");
    assert_eq!(body["error"]["code"], "unknown_scan", "{body}");
    assert_ne!(body["state"], "completed", "{body}");
    handle.shutdown().await;
}

/// Requirement 4, and A1-01: the journal is the authority, so what the
/// daemon did must be in it. Before this wave a whole multi-source scan
/// left **nothing** — no start, no completion, no error — and the only
/// trace of it was the per-source `source.scanned` summaries, which say
/// nothing about the scan that asked for them.
#[tokio::test]
async fn the_journal_records_the_scan_that_started_and_the_scan_that_completed() {
    let estate = TempDir::new().expect("estate");
    scaffold_estate(estate.path());
    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let (_, final_state) = support::scan_to_completion(
        &http,
        &handle.endpoint,
        &handle.token,
        &json!({"command_id": ulid::Ulid::generate().to_string(), "estate_root": estate.path()}),
    )
    .await;
    let scan_id = final_state["scan_id"]
        .as_str()
        .expect("scan id")
        .to_string();

    // The journal is single-writer per directory: read it once the daemon
    // that owned it has released the lock.
    handle.shutdown().await;
    let journal = Journal::open(data.path()).expect("journal");
    let events: Vec<Event> = journal
        .replay()
        .expect("replay")
        .map(|e| e.expect("event"))
        .collect();

    let started = events
        .iter()
        .find(|e| e.kind == KIND_INTELLIGENCE_SCAN_STARTED)
        .unwrap_or_else(|| {
            panic!("the journal records nothing about a scan that started — the exact gap this wave closed")
        });
    assert_eq!(started.payload["scan_id"], scan_id, "{:?}", started.payload);
    assert_eq!(
        started.payload["total_sources"], 3,
        "the started event names what the scan undertook, so an interrupted scan is legible \
         as attempted-and-unfinished: {:?}",
        started.payload
    );

    let completed = events
        .iter()
        .find(|e| e.kind == KIND_INTELLIGENCE_SCAN_COMPLETED)
        .unwrap_or_else(|| panic!("no completion event: {events:?}"));
    assert_eq!(
        completed.payload["scan_id"], scan_id,
        "{:?}",
        completed.payload
    );
    assert_eq!(
        completed.payload["sources_completed"], 3,
        "{:?}",
        completed.payload
    );
    assert_eq!(
        completed.payload["outcomes"]["recorded"], 3,
        "the completion tallies per-source outcomes, so the journal alone answers what the \
         scan achieved: {:?}",
        completed.payload
    );
    assert!(
        completed.payload["duration_ms"].as_u64().is_some(),
        "{:?}",
        completed.payload
    );
    assert!(
        started.seq < completed.seq,
        "started before completed, in the journal's own order"
    );
}

/// The front door, through the real binary an operator and a script
/// actually run: `sgt intelligence scan` **exits zero** and prints a row
/// per source.
///
/// This is the claim the wave is named for. Against the synchronous
/// trigger, the same command on a real five-source estate exited 1 with
/// `error sending request for url (…/v1/intelligence/scan)` after ten
/// seconds while the scan itself succeeded.
#[tokio::test]
async fn the_verb_itself_exits_zero_and_prints_a_row_for_every_source() {
    let estate = TempDir::new().expect("estate");
    scaffold_estate(estate.path());
    let data = DataDir::new();
    let handle = start_daemon(&data).await;

    // The in-process daemon published a descriptor in `data`, so the real
    // client finds it exactly as it finds a detached one — no daemon is
    // spawned by this test.
    let output = tokio::task::spawn_blocking({
        let data_dir = data.path().to_path_buf();
        let estate_root = estate.path().to_path_buf();
        move || {
            std::process::Command::new(SGT)
                .arg("--data-dir")
                .arg(&data_dir)
                .arg("-C")
                .arg(&estate_root)
                .args(["intelligence", "scan"])
                .output()
                .expect("run sgt intelligence scan")
        }
    })
    .await
    .expect("join");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        output.status.success(),
        "the verb must not report failure on a scan that succeeded: status {:?}\nstdout: \
         {stdout}\nstderr: {stderr}",
        output.status.code()
    );
    for source in SOURCES {
        assert!(
            stdout.contains(source),
            "every scanned source gets its own printed row: {stdout}"
        );
    }
    assert!(
        stdout.contains("[3/3]"),
        "progress is counted against what the scan undertook: {stdout}"
    );
    handle.shutdown().await;
}
