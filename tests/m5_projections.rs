//! M5 acceptance tests (docs/gauntlet/contracts/M5.md).
//!
//! 1. Rebuild determinism: real work through the daemon → DuckDB populated →
//!    delete the `.duckdb` file → restart → the §22 canned queries return
//!    identical results, row for row.
//! 2. One owner: nothing outside `daemon`/`runtime` opens the DuckDB file —
//!    the `duckdb` crate is reachable from exactly one module, no `Connection`
//!    escapes it, and clients have only the API.
//! 3. Graph provenance: every edge of `/v1/graph/work/{id}` carries a
//!    `source_seq` resolving to a real journal event whose *content* justifies
//!    that edge.
//! 4. Projection disposability: deleting `projections/` with the daemon
//!    stopped loses nothing — replay restores it and work state is untouched.
//! 5. OTel: with the exporter enabled the §28 span tree (work → stage →
//!    execution.turn) and the implemented metrics are emitted; with it
//!    disabled — the default — no code on the daemon's start path can build a
//!    pipeline, and a running daemon puts nothing into a globally installed
//!    one.
//! 6. Analytics smoke: three of §22's example questions answered through the
//!    daemon (API and CLI), plus the tables behind them.
//!
//! Beyond the numbered list: the §22 tables that only a real adapter can
//! fill (`messages`, `tool_calls`, `usage`) are populated by driving the real
//! Claude adapter over a stand-in binary replaying a recorded turn — and,
//! because a fake-backend run compares them as 0 == 0, acceptance 1 is run a
//! second time over that adapter journal. The rebuild latency of a realistic
//! journal is measured rather than asserted about.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tempfile::TempDir;

use opentelemetry_sdk::metrics::{InMemoryMetricExporter, PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::{InMemorySpanExporter, SdkTracerProvider};

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::claude::{CLAUDE_BACKEND_NAME, ClaudeConfig};
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig, DaemonHandle};
use sergeant_rs::domain::event::Event;
use sergeant_rs::runtime::analytics::{
    Analytics, AnalyticsError, CANNED_QUERIES, DUCKDB_FILE, PROJECTIONS_DIR, duckdb_path,
};
use sergeant_rs::runtime::graph::{GraphContext, GraphEdge};
use sergeant_rs::runtime::journal::{Journal, JournalError};
use sergeant_rs::telemetry::{DEFAULT_OTLP_ENDPOINT, Telemetry, TelemetryConfig};

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

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "sergeant tests")
        .env("GIT_AUTHOR_EMAIL", "tests@example.invalid")
        .env("GIT_COMMITTER_NAME", "sergeant tests")
        .env("GIT_COMMITTER_EMAIL", "tests@example.invalid")
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn init_repo(path: &Path) -> String {
    std::fs::create_dir_all(path).expect("repo dir");
    git(path, &["init", "-b", "main"]);
    std::fs::write(path.join("README.md"), "# fixture\n").expect("write file");
    git(path, &["add", "."]);
    git(path, &["commit", "-m", "initial"]);
    git(path, &["rev-parse", "HEAD"])
}

/// A two-stage workflow, so stage progression and `preceded` edges exist.
fn write_two_stage_workflow(root: &Path) {
    let dir = root.join(".sergeant/workflows/tiny");
    std::fs::create_dir_all(&dir).expect("workflow dir");
    std::fs::write(
        dir.join("workflow.toml"),
        "[workflow]\nname = \"tiny\"\nversion = \"1\"\nstages = [\"00-first\", \"10-second\"]\n",
    )
    .expect("workflow.toml");
    for stage in ["00-first", "10-second"] {
        std::fs::create_dir_all(dir.join(stage)).expect("stage dir");
        std::fs::write(dir.join(stage).join("CONTEXT.md"), "context").expect("CONTEXT.md");
    }
}

async fn start_fake(
    data_dir: &Path,
    script: impl IntoIterator<Item = FakeStep>,
) -> (DaemonHandle, FakeBackend) {
    let fake = FakeBackend::scripted(FAKE_BACKEND_NAME, script);
    let registry = BackendRegistry::new().with(Arc::new(fake.clone()));
    let handle = daemon::start_with(
        data_dir,
        DaemonConfig {
            backends: Arc::new(registry),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");
    (handle, fake)
}

async fn post(handle: &DaemonHandle, path: &str, body: Value) -> (reqwest::StatusCode, Value) {
    let resp = http()
        .post(format!("{}{path}", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&body)
        .send()
        .await
        .expect("request");
    let status = resp.status();
    (status, resp.json().await.expect("json body"))
}

async fn get(handle: &DaemonHandle, path: &str) -> Value {
    http()
        .get(format!("{}{path}", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("request")
        .json()
        .await
        .expect("json body")
}

async fn get_status(handle: &DaemonHandle, path: &str) -> (reqwest::StatusCode, Value) {
    let resp = http()
        .get(format!("{}{path}", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("request");
    let status = resp.status();
    (status, resp.json().await.expect("json body"))
}

async fn submit(handle: &DaemonHandle, cwd: &Path, intent: &str, extra: Value) -> Value {
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
    let (status, value) = post(handle, "/v1/work", body).await;
    assert_eq!(status, reqwest::StatusCode::CREATED, "submit: {value}");
    value
}

fn journal_events(data_dir: &Path) -> Vec<Event> {
    Journal::replay_data_dir(data_dir)
        .expect("replay")
        .map(|e| e.expect("event"))
        .collect()
}

/// Every canned query plus the table counts, as one comparable blob. This is
/// the "identical results, row for row" of acceptance 1.
async fn analytics_snapshot(handle: &DaemonHandle) -> Value {
    let mut snapshot = serde_json::Map::new();
    let index = get(handle, "/v1/analytics").await;
    // Every daemon start journals its own lifecycle (`daemon.started`, the
    // per-backend `backend.probed`, `daemon.stopped`), so the raw `events`
    // count legitimately differs between two runs over the same work. It is
    // checked separately, against the journal, by `events_row_count_matches`.
    let tables: Vec<Value> = index["tables"]
        .as_array()
        .expect("tables")
        .iter()
        .filter(|t| t["table"] != "events")
        .cloned()
        .collect();
    snapshot.insert("tables".to_string(), json!(tables));
    for canned in CANNED_QUERIES {
        let mut answer = get(handle, &format!("/v1/analytics/{}", canned.name)).await;
        // The projection's own high-water mark is metadata about *when* the
        // answer was computed, not part of the answer.
        answer.as_object_mut().expect("object").remove("projection");
        snapshot.insert(canned.name.to_string(), answer);
    }
    Value::Object(snapshot)
}

/// Run a two-stage workflow to completion in a fresh repo, returning the
/// data dir, the repo dir and the work id. The daemon is stopped on return.
async fn completed_run() -> (TempDir, TempDir, String) {
    let data = TempDir::new().expect("tempdir");
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());
    write_two_stage_workflow(repo.path());

    let (handle, _fake) = start_fake(
        data.path(),
        [FakeStep::complete_with("first done"), FakeStep::complete()],
    )
    .await;
    let created = submit(
        &handle,
        repo.path(),
        "measure the projection",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = created["work"]["id"].as_str().expect("work id").to_string();
    let shown = get(&handle, &format!("/v1/work/{work_id}")).await;
    assert_eq!(shown["work"]["state"], "completed", "run: {shown}");
    handle.shutdown().await;
    (data, repo, work_id)
}

// -------------------------------------------- 1. rebuild determinism

/// Acceptance 1. The DuckDB file is a pure function of the journal: deleting
/// it and letting the daemon rebuild produces byte-identical answers to every
/// canned §22 query, and identical row counts in every table.
///
/// If any code path ever wrote a fact into DuckDB that the journal does not
/// carry, this is what loses it — *for the tables a fake-backend run fills*.
/// The fake backend emits no §27 conversation events, so `messages`,
/// `tool_calls` and `usage` would be compared here as 0 == 0 and a rebuild
/// that dropped that half of the fold entirely would stay green. Those tables
/// (and the `produced`/`requested`/`caused`/`used_model` graph edges, which
/// are likewise only ever derived on the adapter path) are covered by
/// `rebuilding_reproduces_the_conversation_tables_a_real_adapter_fills`.
#[tokio::test]
async fn t1_rebuild_from_scratch_reproduces_every_canned_answer() {
    let data = TempDir::new().expect("tempdir");
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());
    write_two_stage_workflow(repo.path());

    // A run with real variety: a failure, an operator retry, and a
    // completion — so the analytics have something to disagree about.
    let (handle, _fake) = start_fake(
        data.path(),
        [
            FakeStep::fail("first attempt broke"),
            FakeStep::complete(),
            FakeStep::complete(),
        ],
    )
    .await;
    let created = submit(
        &handle,
        repo.path(),
        "rebuild me",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = created["work"]["id"].as_str().expect("work id").to_string();
    let (status, retried) = post(
        &handle,
        &format!("/v1/work/{work_id}/retry"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, reqwest::StatusCode::OK, "retry: {retried}");
    assert_eq!(retried["work"]["state"], "completed", "retried: {retried}");

    let before = analytics_snapshot(&handle).await;
    // Named rather than counted: `populated >= 6` was satisfied by exactly the
    // tables a fake run fills, so it could never notice that the comparison
    // below had quietly shrunk to those same seven.
    let populated: BTreeSet<&str> = before["tables"]
        .as_array()
        .expect("tables")
        .iter()
        .filter(|t| t["rows"].as_i64().unwrap_or(0) > 0)
        .filter_map(|t| t["table"].as_str())
        .collect();
    for table in [
        "work",
        "stages",
        "executions",
        "repositories",
        "graph_nodes",
        "graph_edges",
    ] {
        assert!(
            populated.contains(table),
            "a real run must populate §22's {table}: {}",
            before["tables"]
        );
    }
    handle.shutdown().await;

    // §21's diagram, literally: delete the file, restart, rebuild.
    let db = duckdb_path(data.path());
    assert!(db.exists(), "the daemon must have created {}", db.display());
    std::fs::remove_file(&db).expect("delete the projection");

    let (handle, _fake) = start_fake(data.path(), []).await;
    assert!(db.exists(), "restart must rebuild {}", db.display());
    let after = analytics_snapshot(&handle).await;
    let events_rows = get(&handle, "/v1/analytics").await["tables"]
        .as_array()
        .expect("tables")
        .iter()
        .find(|t| t["table"] == "events")
        .and_then(|t| t["rows"].as_u64())
        .expect("an events row count");
    // Read the journal while that daemon is still up: its own
    // `daemon.stopped` would otherwise land between the two measurements.
    let journal_len = journal_events(data.path()).len();
    handle.shutdown().await;
    assert_eq!(
        events_rows as usize, journal_len,
        "the rebuilt events table holds exactly the journal, no more and no less"
    );

    // Row for row, column for column, including the SQL each answer came
    // from — nothing about the answers may depend on the file surviving.
    assert_eq!(
        serde_json::to_string_pretty(&before).expect("json"),
        serde_json::to_string_pretty(&after).expect("json"),
        "the rebuilt projection answered differently"
    );
}

/// The rebuild is also *complete* the moment the daemon is serving: a fresh
/// daemon over an existing journal answers without anyone touching the work
/// again. (Separated from the identity check above so a rebuild that merely
/// echoed a cached answer could not pass both.)
#[tokio::test]
async fn a_fresh_daemon_answers_from_the_journal_alone() {
    let (data, _repo, work_id) = completed_run().await;
    std::fs::remove_dir_all(data.path().join(PROJECTIONS_DIR)).expect("delete projections");

    let (handle, _fake) = start_fake(data.path(), []).await;
    let answer = get(&handle, "/v1/analytics/execution_touched").await;
    let rows = answer["rows"].as_array().expect("rows");
    assert!(
        rows.iter()
            .any(|row| row[1].as_str() == Some(work_id.as_str())),
        "the rebuilt projection knows the completed work: {answer}"
    );
    let graph = get(&handle, &format!("/v1/graph/work/{work_id}")).await;
    assert!(
        !graph["edges"].as_array().expect("edges").is_empty(),
        "the rebuilt graph is populated: {graph}"
    );
    handle.shutdown().await;
}

/// An ordinary restart — nobody deleted anything — must also rebuild.
///
/// The daemon's only population path is rebuild-from-scratch, so a start that
/// found the previous file and tried to *continue* it would either fail
/// outright or accumulate rows the journal cannot account for. Neither is
/// caught by the deletion tests above, because they always hand the daemon a
/// clean slate.
#[tokio::test]
async fn a_restart_over_the_existing_projection_file_rebuilds_it() {
    let (data, _repo, work_id) = completed_run().await;
    assert!(
        duckdb_path(data.path()).exists(),
        "the first daemon left its projection behind"
    );

    // Restart with the file still there, twice — a projection that appended
    // to itself instead of being rebuilt would double its rows here.
    let (handle, _fake) = start_fake(data.path(), []).await;
    let first = analytics_snapshot(&handle).await;
    handle.shutdown().await;
    let (handle, _fake) = start_fake(data.path(), []).await;
    let second = analytics_snapshot(&handle).await;
    let graph = get(&handle, &format!("/v1/graph/work/{work_id}")).await;
    handle.shutdown().await;

    assert_eq!(first, second, "a restart must not accumulate rows");
    assert!(
        !graph["edges"].as_array().expect("edges").is_empty(),
        "the graph survives an ordinary restart: {graph}"
    );
}

// ------------------------------- 1b. the projection fails closed, or not at all

/// A fold that fails part-way must never leave a table silently short.
///
/// Acceptance 1 says the kept-current projection equals a from-scratch
/// rebuild row for row. The dangerous way to break that is not a crash: it is
/// a fold that dies after `last_seq` has moved past rows that never reached
/// the tables. The skip in `catch_up` is permanent, so those rows would never
/// be folded again — `events`, `messages` and `usage` would answer *200 with
/// missing rows* for the rest of the process's life, while the mutable tables
/// quietly self-healed around them. One 503 is the correct cost; wrong rows
/// are not a cost the contract permits at any price.
#[test]
fn a_fold_that_fails_part_way_never_answers_out_of_a_short_table() {
    let good = provenance_fixture();
    assert!(
        good.len() > 3,
        "fixture must have events either side of the failure"
    );

    // The journal goes unreadable in the middle of a catch-up: real, and the
    // same early return every write failure (a full disk, say) takes.
    let torn = || {
        let mut items: Vec<Result<Event, JournalError>> =
            good.iter().take(2).cloned().map(Ok).collect();
        items.push(Err(JournalError::Io(std::io::Error::other(
            "the journal went away mid-replay",
        ))));
        items.extend(good.iter().skip(2).cloned().map(Ok));
        items
    };

    let mut analytics = Analytics::in_memory(Vec::new()).expect("projection");
    assert!(
        analytics.catch_up(torn()).is_err(),
        "the fold must surface the failure"
    );

    // It must now claim to have folded nothing, and refuse to answer at all.
    assert_eq!(
        analytics.last_seq(),
        0,
        "a projection that dropped rows must not claim the seq it dropped them at"
    );
    assert!(
        matches!(analytics.table_counts(), Err(AnalyticsError::NeedsRebuild)),
        "a projection holding a failed fold must refuse to answer, not answer short"
    );

    // And it heals by re-folding the journal, which is the only repair the
    // §40 disposability story allows.
    let refolded = analytics
        .catch_up(good.iter().cloned().map(Ok))
        .expect("re-fold after the journal came back");
    assert_eq!(
        refolded,
        good.len() as u64,
        "the whole journal must be re-folded"
    );

    let healed = local_projection(&mut analytics);
    let rebuilt =
        local_projection(&mut Analytics::in_memory(good.iter().cloned().map(Ok)).expect("rebuild"));
    assert_eq!(
        healed, rebuilt,
        "after healing, kept-current must equal a from-scratch rebuild row for row"
    );
}

/// Acceptance 1's failure semantics over HTTP: a projection that cannot be
/// caught up answers 503 against the *projection*, and the work it describes
/// is untouched.
#[tokio::test]
async fn a_projection_that_cannot_catch_up_answers_503_and_never_wrong_rows() {
    let data = TempDir::new().expect("tempdir");
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());
    write_two_stage_workflow(repo.path());

    let (handle, _fake) =
        start_fake(data.path(), [FakeStep::complete(), FakeStep::complete()]).await;
    let created = submit(
        &handle,
        repo.path(),
        "still answerable",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = created["work"]["id"].as_str().expect("id").to_string();

    // Corrupt the journal *tail* behind the daemon's back. The projection is
    // folded lazily at read time, so the next analytics request has to read
    // this and cannot.
    let segment = journal_segment(data.path());
    let intact = std::fs::read(&segment).expect("read segment");
    let mut torn = intact.clone();
    torn.extend_from_slice(b"{ not an event }\n");
    std::fs::write(&segment, &torn).expect("tear the journal");

    for path in ["/v1/analytics", "/v1/analytics/blocked_time_per_work"] {
        let (status, body) = get_status(&handle, path).await;
        assert_eq!(
            status,
            reqwest::StatusCode::SERVICE_UNAVAILABLE,
            "{path} must report the projection unavailable, not invent an answer"
        );
        assert_eq!(body["error"]["code"], "projection_unavailable");
    }
    let (status, _) = get_status(&handle, &format!("/v1/graph/work/{work_id}")).await;
    assert_eq!(status, reqwest::StatusCode::SERVICE_UNAVAILABLE);

    // The point of 503 rather than 500: the work itself is fine. Its state
    // lives in the journal-backed registry, not in the derived file.
    let work = get(&handle, &format!("/v1/work/{work_id}")).await;
    assert_eq!(work["work"]["state"], "completed");

    // And the failure is transient by construction — nothing was written, so
    // repairing the journal restores the answer with no restart.
    std::fs::write(&segment, &intact).expect("restore the journal");
    let index = get(&handle, "/v1/analytics").await;
    assert!(
        index["tables"]
            .as_array()
            .expect("tables")
            .iter()
            .any(|t| t["table"] == "work" && t["rows"].as_i64() == Some(1)),
        "the projection must answer again once the journal is readable"
    );

    handle.shutdown().await;
}

/// The analytical projection's read-time catch-up asks the journal for its
/// tail while holding the daemon's mutation lock, so the tail read must cost
/// the answer and not the history.
///
/// The correctness risk in doing that is segment skipping: it is the one read
/// path that does not start at seq 1, so it is the one that could silently
/// return a short or misaligned answer. This pins it across real segment
/// boundaries, at every offset, including the two that matter most — nothing
/// pending, and everything pending.
#[test]
fn the_journal_tail_read_is_exact_at_every_offset_across_segments() {
    use sergeant_rs::domain::event::{EventDraft, EventSource};

    let dir = TempDir::new().expect("tempdir");
    // Small enough to rotate every few events, so the skip logic is exercised
    // rather than trivially satisfied by a single segment.
    let mut journal = Journal::open_with(dir.path(), 512).expect("open");
    for index in 0..40u64 {
        journal
            .append(EventDraft::new(
                EventSource::new("daemon", "test"),
                "work.submitted",
                json!({"work": {"id": format!("w{index}"), "state": "pending"}}),
            ))
            .expect("append");
    }
    let segments = std::fs::read_dir(dir.path().join("journal"))
        .expect("journal dir")
        .filter(|e| {
            e.as_ref()
                .expect("entry")
                .path()
                .extension()
                .is_some_and(|x| x == "ndjson")
        })
        .count();
    assert!(segments > 1, "the fixture must span multiple segments");

    let all: Vec<u64> = journal
        .replay()
        .expect("replay")
        .map(|e| e.expect("event").seq)
        .collect();
    assert_eq!(all, (1..=40).collect::<Vec<u64>>());

    for after in 0..=41u64 {
        let tail: Vec<u64> = journal
            .replay_after(after)
            .expect("replay_after")
            .map(|e| e.expect("event").seq)
            .filter(|seq| *seq > after)
            .collect();
        let expected: Vec<u64> = ((after + 1)..=40).collect();
        assert_eq!(tail, expected, "tail after {after} is wrong");
    }
}

/// The journal's single segment file for a data dir.
fn journal_segment(data_dir: &Path) -> PathBuf {
    let dir = data_dir.join("journal");
    let mut segments: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("journal dir")
        .filter_map(|entry| {
            let path = entry.expect("entry").path();
            (path.extension().is_some_and(|e| e == "ndjson")).then_some(path)
        })
        .collect();
    segments.sort();
    segments.pop().expect("at least one segment")
}

// ------------------------------------------------------ 2. one owner

/// Acceptance 2. Only the daemon's own projection module opens DuckDB.
///
/// Enforced structurally and checked mechanically: the `duckdb` crate is
/// named in exactly one source file, that file hands out no connection, and
/// the dependency is a normal one (a client binary that wanted its own copy
/// would have to change this test first).
#[test]
fn t2_the_duckdb_file_has_exactly_one_owner() {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    // A token scan, not a fixed-pattern grep: however the import is spelled
    // (`use duckdb::...`, `duckdb::Connection`, `extern crate duckdb`, a
    // re-export), the crate's own lowercase name has to appear somewhere in
    // the file. Prose that mentions the product ("DuckDB") is capitalized
    // and does not collide with the identifier.
    let names_the_crate = |text: &str| {
        text.split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .any(|token| token == "duckdb")
    };
    for file in rust_sources(&src) {
        if file.ends_with("runtime/analytics.rs") {
            continue;
        }
        let text = std::fs::read_to_string(&file).expect("read source");
        assert!(
            !names_the_crate(&text),
            "{} names the duckdb crate; it must be reachable from one module only",
            file.strip_prefix(&src).expect("under src").display()
        );
    }

    // The positive half of "exactly one owner". Without it the loop above is
    // satisfied by a build that stopped using the crate altogether — and R5
    // named the embedded Rust client specifically, so a silent swap for some
    // other store is exactly the drift this acceptance test exists to catch.
    let analytics = std::fs::read_to_string(src.join("runtime/analytics.rs")).expect("read");
    assert!(
        names_the_crate(&analytics),
        "runtime/analytics.rs must be the one module that uses the duckdb crate"
    );

    // And that module must not leak the connection: everything crossing its
    // boundary is plain data (`QueryResult`, `GraphView`, counts).
    //
    // Private-by-default does not cover this on its own. `Analytics` is a
    // public struct in a public module, so `pub conn: Connection` compiles
    // and is reachable from anywhere in the workspace — and a consumer
    // written against it (`analytics.conn.execute(..)`) names the lowercase
    // crate token nowhere, so the scan above would not see it either. The
    // field declaration is therefore pinned directly.
    assert!(
        analytics.contains("\n    conn: Connection,"),
        "the connection must stay a private field"
    );
    assert!(
        !analytics.contains("pub fn conn") && !analytics.contains("-> &Connection"),
        "no accessor may hand a live DuckDB connection outside the projection"
    );

    // The CLI reaches analytics only through the daemon's HTTP surface.
    let cli = std::fs::read_to_string(src.join("cli.rs")).expect("read cli");
    assert!(
        cli.contains("/v1/analytics") && !names_the_crate(&cli),
        "clients ask the daemon; they do not open the file"
    );
}

fn rust_sources(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).expect("read dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            out.extend(rust_sources(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    out
}

// ----------------------------------------------- 3. graph provenance

/// Acceptance 3. Every edge of a completed work's neighborhood names a
/// journal seq, that seq resolves to a real event, and that event's *content*
/// implies the edge. §23's inspectability rule is only a rule if a wrong
/// `source_seq` fails something.
#[tokio::test]
async fn t3_every_graph_edge_resolves_to_an_event_that_justifies_it() {
    let (data, _repo, work_id) = completed_run().await;
    let journal = Provenance::new(journal_events(data.path()));

    let (handle, _fake) = start_fake(data.path(), []).await;
    let graph = get(&handle, &format!("/v1/graph/work/{work_id}")).await;
    handle.shutdown().await;

    let edges = graph["edges"].as_array().expect("edges").clone();
    assert!(
        edges.len() >= 6,
        "a completed two-stage run must derive a real graph: {graph}"
    );
    let relations: BTreeSet<&str> = edges
        .iter()
        .filter_map(|e| e["relation"].as_str())
        .collect();
    for expected in [
        "submitted",
        "targets",
        "follows",
        "executes",
        "uses",
        "preceded",
    ] {
        assert!(
            relations.contains(expected),
            "missing the §23 {expected:?} edge: {relations:?}"
        );
    }

    for edge in &edges {
        let seq = edge["source_seq"].as_u64().expect("source_seq");
        let event = journal
            .event(seq)
            .unwrap_or_else(|| panic!("edge {edge} names seq {seq}, which is not in the journal"));
        assert!(
            journal.justifies(edge, event),
            "event seq {seq} ({}) does not justify edge {edge}: {}",
            event.kind,
            event.payload
        );
    }

    // A node that only appears as an edge endpoint still comes back, so the
    // neighborhood is readable without a second request.
    let node_ids: BTreeSet<&str> = graph["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .filter_map(|n| n["node_id"].as_str())
        .collect();
    for edge in &edges {
        for end in ["from_node", "to_node"] {
            let id = edge[end].as_str().expect("endpoint");
            assert!(node_ids.contains(id), "edge endpoint {id} has no node");
        }
    }
}

/// The journal, indexed the two ways provenance checking needs it: by seq, to
/// resolve an edge's `source_seq`, and by event id, to resolve a `causation_id`
/// to the message node the journal itself says caused a tool call.
///
/// Provenance is checked on **both** endpoints of every edge, never on the
/// target alone. An edge that names a real event and lands on a real node but
/// wires the *wrong* entity to it — a `requested` edge hanging off the work
/// instead of the execution, a `uses` edge from a different execution — is
/// exactly the defect acceptance 3 exists for, and a target-only check cannot
/// see it: the wrong source is still a legitimate node, so the endpoint-
/// existence loop in `t3` passes it too.
struct Provenance {
    by_seq: BTreeMap<u64, Event>,
    seq_by_id: BTreeMap<String, u64>,
}

impl Provenance {
    fn new(events: impl IntoIterator<Item = Event>) -> Self {
        let by_seq: BTreeMap<u64, Event> = events.into_iter().map(|e| (e.seq, e)).collect();
        let seq_by_id = by_seq.values().map(|e| (e.id.clone(), e.seq)).collect();
        Self { by_seq, seq_by_id }
    }

    /// The event an edge's `source_seq` names, if the journal has one.
    fn event(&self, seq: u64) -> Option<&Event> {
        self.by_seq.get(&seq)
    }

    /// The stage node the journal recorded as most recently entered for
    /// `work_id` strictly before `seq` — the only node a `preceded` edge may
    /// point *from*. "Which stage preceded which" is that edge's whole
    /// content, so checking only its target checks nothing.
    fn stage_before(&self, work_id: &str, seq: u64) -> Option<String> {
        self.by_seq
            .values()
            .rfind(|e| {
                e.seq < seq && e.kind == "stage.entered" && e.work_id.as_deref() == Some(work_id)
            })
            .map(stage_node_of)
    }

    /// The message node produced by the event with id `id`, when that event is
    /// a conversation event of `execution_id`. This is what makes `caused`
    /// checkable: the derivation's entire claim is that the journal's own
    /// causation link names *this* message, so the edge's source must be the
    /// message that id resolves to — not merely some message that exists.
    fn message_node_of(&self, id: &str, execution_id: &str) -> Option<String> {
        let seq = *self.seq_by_id.get(id)?;
        let event = self.by_seq.get(&seq)?;
        (event.kind.starts_with("conversation.")
            && event.execution_id.as_deref() == Some(execution_id))
        .then(|| format!("message:{seq}"))
    }

    /// Whether `event`'s content implies `edge`. Deliberately checks the
    /// payload and both endpoints, not just the kind: an edge pointed at the
    /// right *kind* of event but the wrong instance, or hung off the wrong
    /// entity, is exactly the defect this acceptance test exists for.
    fn justifies(&self, edge: &Value, event: &Event) -> bool {
        let relation = edge["relation"].as_str().unwrap_or_default();
        let to = edge["to_node"].as_str().unwrap_or_default();
        let from = edge["from_node"].as_str().unwrap_or_default();
        let payload = &event.payload;

        // The nodes this one event can legitimately name, read off its own
        // envelope and payload. Every arm below is a comparison against these
        // — an endpoint that is not one of them is not justified by it.
        let work = format!("work:{}", event.work_id.as_deref().unwrap_or_default());
        let submitted_work = format!(
            "work:{}",
            payload["work"]["id"].as_str().unwrap_or_default()
        );
        let execution = format!(
            "execution:{}",
            payload["execution"]["execution_id"]
                .as_str()
                .unwrap_or_default()
        );
        let turn = format!(
            "execution:{}",
            event.execution_id.as_deref().unwrap_or_default()
        );
        let tool_call = format!(
            "tool_call:{}/{}",
            event.execution_id.as_deref().unwrap_or_default(),
            payload["id"]
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| format!("seq{}", event.seq)),
        );

        match relation {
            "submitted" => {
                event.kind == "work.submitted"
                    && from
                        == format!(
                            "client:{}",
                            payload["work"]["origin_client"]
                                .as_str()
                                .or_else(|| payload["work"]["created_by"].as_str())
                                .unwrap_or_default()
                        )
                    && to == submitted_work
            }
            "scoped_to" => {
                event.kind == "work.submitted"
                    && from == submitted_work
                    && to
                        == format!(
                            "workspace:{}",
                            payload["work"]["workspace"].as_str().unwrap_or_default()
                        )
            }
            "targets" => {
                event.kind == "surface.materialized"
                    && from == work
                    && payload["surface"]["bindings"]
                        .as_array()
                        .is_some_and(|bindings| {
                            bindings.iter().any(|b| {
                                to == format!(
                                    "repository:{}",
                                    b["repository"].as_str().unwrap_or_default()
                                )
                            })
                        })
            }
            "follows" => {
                event.kind == "workflow.bound"
                    && from == work
                    && to
                        == format!(
                            "workflow:{}@{}",
                            payload["workflow"]["name"].as_str().unwrap_or_default(),
                            payload["workflow"]["version"].as_str().unwrap_or("?"),
                        )
            }
            "uses_profile" => {
                event.kind == "workflow.bound"
                    && from == work
                    && to
                        == format!(
                            "profile:{}",
                            payload["profile"]["name"].as_str().unwrap_or_default()
                        )
            }
            "executes" => event.kind == "execution.started" && from == execution && to == work,
            "uses" => {
                event.kind == "execution.started"
                    && from == execution
                    && to
                        == format!(
                            "backend:{}",
                            payload["execution"]["backend"].as_str().unwrap_or_default()
                        )
            }
            "bound_to" => {
                event.kind == "execution.started"
                    && from == execution
                    && to
                        == format!(
                            "native_session:{}",
                            payload["execution"]["native_id"]
                                .as_str()
                                .unwrap_or_default()
                        )
            }
            "ran_in" => {
                event.kind == "execution.started"
                    && from == execution
                    && to
                        == format!(
                            "stage:{}/{}#{}",
                            event.work_id.as_deref().unwrap_or_default(),
                            payload["execution"]["stage_id"]
                                .as_str()
                                .unwrap_or_default(),
                            payload["execution"]["attempt"].as_u64().unwrap_or(1),
                        )
            }
            "preceded" => {
                event.kind == "stage.entered"
                    && to == stage_node_of(event)
                    && self
                        .stage_before(event.work_id.as_deref().unwrap_or_default(), event.seq)
                        .is_some_and(|previous| from == previous)
            }
            "produced" => {
                event.kind.starts_with("conversation.")
                    && from == turn
                    && to == format!("message:{}", event.seq)
            }
            "requested" => event.kind == "tool.requested" && from == turn && to == tool_call,
            "caused" => {
                event.kind == "tool.requested"
                    && to == tool_call
                    && event
                        .causation_id
                        .as_deref()
                        .and_then(|cause| {
                            self.message_node_of(
                                cause,
                                event.execution_id.as_deref().unwrap_or_default(),
                            )
                        })
                        .is_some_and(|message| from == message)
            }
            "used_model" => {
                event.kind == "usage.updated"
                    && from == turn
                    && payload["model_usage"]
                        .as_object()
                        .is_some_and(|m| m.keys().any(|model| to == format!("model:{model}")))
            }
            _ => false,
        }
    }
}

/// The stage node one `stage.entered` event instantiates.
fn stage_node_of(event: &Event) -> String {
    format!(
        "stage:{}/{}#{}",
        event.work_id.as_deref().unwrap_or_default(),
        event.payload["stage_id"].as_str().unwrap_or_default(),
        event.payload["attempt"].as_u64().unwrap_or(1),
    )
}

/// The §23 relations the derivation can instantiate from today's journal, and
/// therefore the ones [`Provenance::justifies`] owes a discriminating case.
const DERIVED_RELATIONS: &[&str] = &[
    "bound_to",
    "caused",
    "executes",
    "follows",
    "preceded",
    "produced",
    "ran_in",
    "requested",
    "scoped_to",
    "submitted",
    "targets",
    "used_model",
    "uses",
    "uses_profile",
];

/// A mislabelled edge must actually fail [`Provenance::justifies`] — otherwise
/// acceptance 3 asserts nothing.
///
/// One negative case on one relation would not do it: the arms whose entire
/// content is *which entity is wired to which* (`requested`, `caused`, `uses`,
/// `preceded`) are precisely the ones a single probe on a strong arm leaves
/// unexercised, and a check without its own kill probe is prose (LESSONS L7).
/// So the predicate is driven over a synthetic journal instantiating every
/// relation the derivation can produce, and for each derived edge:
///
/// - the true edge is accepted;
/// - **either** endpoint replaced by any *other real node of the same graph*
///   is rejected — a legitimate node, the wrong entity, which is the mutation
///   a target-only check waves through;
/// - either endpoint replaced by a plausible impostor is rejected;
/// - every *other* event in the journal, offered as the edge's provenance, is
///   rejected.
#[test]
fn provenance_checking_rejects_every_mislabelled_edge() {
    let events = provenance_fixture();
    let journal = Provenance::new(events.clone());

    // Derive the graph the same way the daemon does, so the edges under test
    // are the real ones rather than shapes this test invented.
    let mut context = GraphContext::default();
    let mut nodes: BTreeSet<String> = BTreeSet::new();
    let mut derived: Vec<(GraphEdge, Event)> = Vec::new();
    for event in &events {
        let delta = context.derive(event);
        nodes.extend(delta.nodes.iter().map(|n| n.id.clone()));
        derived.extend(delta.edges.into_iter().map(|edge| (edge, event.clone())));
    }
    let relations: BTreeSet<&str> = derived.iter().map(|(edge, _)| edge.relation).collect();
    assert_eq!(
        relations,
        DERIVED_RELATIONS
            .iter()
            .copied()
            .collect::<BTreeSet<&str>>(),
        "every relation the derivation can produce must get a probe here"
    );

    for (edge, event) in &derived {
        let good = json!({
            "relation": edge.relation,
            "from_node": edge.from,
            "to_node": edge.to,
            "source_seq": edge.source_seq,
        });
        assert!(
            journal.justifies(&good, event),
            "the derivation's own edge must be accepted: {good}"
        );

        for end in ["from_node", "to_node"] {
            let original = good[end].as_str().expect("endpoint").to_string();
            for other in &nodes {
                if *other == original {
                    continue;
                }
                let mut mutant = good.clone();
                mutant[end] = json!(other);
                assert!(
                    !journal.justifies(&mutant, event),
                    "a real but wrong {end} must be rejected: {mutant}"
                );
            }
            let mut mutant = good.clone();
            mutant[end] = json!(format!("{original}-impostor"));
            assert!(
                !journal.justifies(&mutant, event),
                "an impostor {end} must be rejected: {mutant}"
            );
        }

        for other in &events {
            if other.seq == event.seq {
                continue;
            }
            assert!(
                !journal.justifies(&good, other),
                "seq {} ({}) must not justify {good}",
                other.seq,
                other.kind
            );
        }
    }
}

/// One synthetic run carrying exactly one instance of every §23 relation the
/// derivation can produce, in the shapes the engine and the §27 adapter write.
fn provenance_fixture() -> Vec<Event> {
    use sergeant_rs::domain::event::{EventDraft, EventSource};
    let mut seq = 0;
    let mut event = |kind: &str, payload: Value| {
        seq += 1;
        EventDraft::new(EventSource::new("daemon", "test"), kind, payload)
            .with_work_id("w")
            .into_event(seq)
    };
    let submitted = event(
        "work.submitted",
        json!({"work": {
            "id": "w", "intent": "probe provenance", "state": "pending",
            "created_by": "tests", "origin_client": "cli", "workspace": "ws",
        }}),
    );
    let surface = event(
        "surface.materialized",
        json!({"surface": {"work_id": "w", "bindings": [
            {"repository": "api", "source_path": "/tmp/api"},
        ]}}),
    );
    let bound = event(
        "workflow.bound",
        json!({
            "workflow": {"name": "tiny", "version": "1"},
            "profile": {"name": "careful"},
        }),
    );
    let first_stage = event(
        "stage.entered",
        json!({"stage_id": "00-first", "index": 0, "attempt": 1}),
    );
    let started = event(
        "execution.started",
        json!({"execution": {
            "execution_id": "e1", "backend": "fake", "native_id": "n1",
            "stage_id": "00-first", "attempt": 1,
        }}),
    );
    let mut message = event("conversation.user", json!({"text": "do the thing"}));
    message.execution_id = Some("e1".to_string());
    let mut requested = event("tool.requested", json!({"id": "t1", "name": "Bash"}));
    requested.execution_id = Some("e1".to_string());
    requested.causation_id = Some(message.id.clone());
    let mut usage = event(
        "usage.updated",
        json!({"model_usage": {"claude-sonnet-4-5": {"input_tokens": 17}}}),
    );
    usage.execution_id = Some("e1".to_string());
    let second_stage = event(
        "stage.entered",
        json!({"stage_id": "10-second", "index": 1, "attempt": 1}),
    );
    vec![
        submitted,
        surface,
        bound,
        first_stage,
        started,
        message,
        requested,
        usage,
        second_stage,
    ]
}

// ------------------------------------------- 4. projection disposability

/// Acceptance 4. Deleting `projections/` entirely, with the daemon stopped,
/// loses nothing: work state is untouched, the journal is untouched, and the
/// projection comes back identical.
///
/// Same caveat as `t1`: this drives the fake backend, so the §22 tables only a
/// real adapter fills are compared as 0 == 0 here and are covered instead by
/// `rebuilding_reproduces_the_conversation_tables_a_real_adapter_fills`.
#[tokio::test]
async fn t4_deleting_the_projections_directory_loses_nothing() {
    let (data, _repo, work_id) = completed_run().await;
    let journal_before = journal_events(data.path());

    let (handle, _fake) = start_fake(data.path(), []).await;
    let work_before = get(&handle, &format!("/v1/work/{work_id}")).await;
    let snapshot_before = analytics_snapshot(&handle).await;
    let graph_before = get(&handle, &format!("/v1/graph/work/{work_id}")).await;
    handle.shutdown().await;

    let projections = data.path().join(PROJECTIONS_DIR);
    assert!(projections.join(DUCKDB_FILE).exists());
    std::fs::remove_dir_all(&projections).expect("delete the whole projections dir");

    let (handle, _fake) = start_fake(data.path(), []).await;
    let work_after = get(&handle, &format!("/v1/work/{work_id}")).await;
    let snapshot_after = analytics_snapshot(&handle).await;
    let mut graph_after = get(&handle, &format!("/v1/graph/work/{work_id}")).await;
    handle.shutdown().await;

    // Nothing about the *work* moved: the projection was never load-bearing.
    assert_eq!(work_before["work"], work_after["work"]);
    assert_eq!(work_before["stage"], work_after["stage"]);
    assert_eq!(work_before["surface"], work_after["surface"]);

    // The journal grew only by the second daemon's own lifecycle events, and
    // every event the first daemon wrote is still there, unchanged.
    let journal_after = journal_events(data.path());
    assert!(journal_after.len() > journal_before.len());
    assert_eq!(&journal_after[..journal_before.len()], &journal_before[..]);

    // And the derived views came back the same.
    assert_eq!(snapshot_before, snapshot_after);
    let mut graph_before_edges = graph_before.clone();
    graph_before_edges
        .as_object_mut()
        .expect("obj")
        .remove("projection");
    graph_after
        .as_object_mut()
        .expect("obj")
        .remove("projection");
    assert_eq!(graph_before_edges, graph_after);
}

// --------------------------------------------------------- 5. OTel

/// Acceptance 5, enabled half. A fake-backend run with the exporter switched
/// on emits §28's span tree — `work` → `stage.<id>` → `execution.turn`, with
/// real parent links — and the implemented metrics.
#[tokio::test]
async fn t5_enabled_export_emits_the_span_tree_and_metrics() {
    let data = TempDir::new().expect("tempdir");
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());
    write_two_stage_workflow(repo.path());

    let spans = InMemorySpanExporter::default();
    let metrics = InMemoryMetricExporter::default();
    let telemetry = Arc::new(Telemetry::with_exporters(spans.clone(), metrics.clone()));

    let fake = FakeBackend::scripted(
        FAKE_BACKEND_NAME,
        [FakeStep::complete(), FakeStep::complete()],
    );
    let handle = daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(BackendRegistry::new().with(Arc::new(fake))),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            telemetry: Some(telemetry.clone()),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");
    let created = submit(
        &handle,
        repo.path(),
        "trace me",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = created["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(created["work"]["state"], "completed");
    handle.shutdown().await;

    // The export task consumes the broadcast stream, so wait for the tree
    // rather than assuming the last event has been folded.
    let finished = wait_for_spans(&spans, 4).await;
    telemetry.force_flush();

    let by_name: BTreeMap<String, opentelemetry_sdk::trace::SpanData> = finished
        .iter()
        .map(|s| (s.name.to_string(), s.clone()))
        .collect();
    let work = by_name.get("work").expect("a work span");
    let stage = by_name
        .get("stage.00-first")
        .expect("a span for the first stage");
    let execution = finished
        .iter()
        .find(|s| s.name == "execution.turn")
        .expect("an execution.turn span");

    assert_eq!(
        stage.parent_span_id,
        work.span_context.span_id(),
        "§28 nests stage under work"
    );
    assert_eq!(
        execution.parent_span_id,
        by_name
            .get(&format!(
                "stage.{}",
                stage_of(execution).unwrap_or_else(|| "00-first".to_string())
            ))
            .expect("the stage the execution ran in")
            .span_context
            .span_id(),
        "§28 nests execution.turn under its stage"
    );
    assert_eq!(
        execution.span_context.trace_id(),
        work.span_context.trace_id(),
        "one work is one trace"
    );
    assert!(
        attribute(work, "sergeant.work.id") == Some(work_id.clone()),
        "the work span names its work: {:?}",
        work.attributes
    );
    assert!(
        by_name.contains_key("stage.10-second"),
        "both stages are exported: {:?}",
        by_name.keys().collect::<Vec<_>>()
    );

    // Every span carries the journal's timestamps, not the fold's.
    for span in &finished {
        assert!(
            span.end_time >= span.start_time,
            "span {} has an impossible interval",
            span.name
        );
    }

    // The §28 metrics this build implements are actually exported.
    let names = exported_metric_names(&metrics);
    for expected in [
        "sergeant_work_active",
        "sergeant_execution_active",
        "sergeant_execution_duration_seconds",
        "sergeant_stage_duration_seconds",
        "sergeant_journal_append_seconds",
    ] {
        assert!(
            names.contains(expected),
            "§28 metric {expected} was not exported: {names:?}"
        );
    }
}

/// The stage id an `execution.turn` span was nested under, from its own
/// attributes plus the journal's stage naming.
fn stage_of(span: &opentelemetry_sdk::trace::SpanData) -> Option<String> {
    attribute(span, "sergeant.stage.id")
}

fn attribute(span: &opentelemetry_sdk::trace::SpanData, key: &str) -> Option<String> {
    span.attributes
        .iter()
        .find(|kv| kv.key.as_str() == key)
        .map(|kv| kv.value.to_string())
}

/// Wait for `count` spans to reach the exporter.
///
/// Deliberately `async` with a `tokio::time::sleep`: the export runs as a
/// task on this test's runtime, so a blocking sleep here would starve the
/// very thing being waited for and the wait would always time out.
async fn wait_for_spans(
    exporter: &InMemorySpanExporter,
    count: usize,
) -> Vec<opentelemetry_sdk::trace::SpanData> {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let spans = exporter.get_finished_spans().expect("spans");
        if spans.len() >= count {
            return spans;
        }
        assert!(
            Instant::now() < deadline,
            "only {} of {count} spans were exported: {:?}",
            spans.len(),
            spans.iter().map(|s| s.name.clone()).collect::<Vec<_>>()
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

fn exported_metric_names(exporter: &InMemoryMetricExporter) -> BTreeSet<String> {
    exporter
        .get_finished_metrics()
        .expect("metrics")
        .iter()
        .flat_map(|resource| resource.scope_metrics())
        .flat_map(|scope| scope.metrics())
        .map(|metric| metric.name().to_string())
        .collect()
}

/// Acceptance 5, disabled half. The default is off, and off means *nothing*
/// runs: no pipeline is constructed from the default configuration, no code
/// on the daemon's start path can construct one at all, and a running daemon
/// puts nothing into an ambient pipeline that is sitting right there.
///
/// Each claim is checked by the probe that can actually see it, because this
/// test has twice been rewritten into one that could not fail:
///
/// - *no pipeline from the default config* is a unit assertion, below;
/// - *no code on the start path can construct one* is not observable at
///   runtime at all — `sgt daemon` runs `run_until_signal`, which no test
///   drives — so it is checked structurally by
///   [`assert_telemetry_is_built_in_one_gated_place`];
/// - *nothing reaches a collector* needs a collector at an address a
///   regression would really dial, which is `DEFAULT_OTLP_ENDPOINT`, not an
///   ephemeral port nobody is told about;
/// - *nothing lands in an ambient pipeline* needs that pipeline to be
///   genuinely reachable. An earlier shape of this test built a `Telemetry`
///   with in-memory exporters and asserted they stayed empty — but
///   `Telemetry::with_exporters` registers its providers nowhere, so no
///   daemon behaviour could ever have filled them. The exporters below are
///   installed as OpenTelemetry's *global* providers, and the kill probe at
///   the end proves they are live.
#[tokio::test]
async fn t5_disabled_export_runs_no_exporter_machinery() {
    assert!(
        DaemonConfig::default().telemetry.is_none(),
        "the daemon's default configuration must not export"
    );
    assert!(!TelemetryConfig::default().enabled);
    assert!(
        Telemetry::from_config(&TelemetryConfig::default())
            .expect("config")
            .is_none(),
        "a disabled configuration must not build a pipeline at all"
    );

    // The structural guard, and the only coverage anywhere of the daemon's
    // real entrypoint. `sgt daemon` runs `run_until_signal`, which no test
    // drives (every test above hands `start_with` a `DaemonConfig` directly),
    // so without this a regression that dropped the environment gate — or
    // that built a pipeline inside `start_with` regardless of the config it
    // was handed — would ship green.
    assert_telemetry_is_built_in_one_gated_place();

    // Not the only guard, and not a text scan: a bare TCP listener stands in
    // for a collector nobody told the daemon about. If a regression started
    // building a pipeline from the ambient environment regardless of the
    // config it was handed, the fake-backend run below would have somewhere
    // to send it — this asserts it never dials out at all.
    //
    // The listener has to sit on `DEFAULT_OTLP_ENDPOINT`, because that is the
    // address such a regression would actually dial. An ephemeral port whose
    // number is never handed to anything makes "zero connections" true under
    // every possible implementation, which is not a test at all.
    let collector_hits = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let default_port = DEFAULT_OTLP_ENDPOINT
        .rsplit(':')
        .next()
        .and_then(|port| port.parse::<u16>().ok())
        .expect("the default endpoint names a port");
    let collector_listener = std::net::TcpListener::bind(("127.0.0.1", default_port))
        .unwrap_or_else(|e| panic!("bind a stand-in collector on 127.0.0.1:{default_port}: {e}"));
    collector_listener
        .set_nonblocking(true)
        .expect("nonblocking");
    let collector_stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let collector_thread = {
        use std::sync::atomic::Ordering;
        let hits = Arc::clone(&collector_hits);
        let stop = Arc::clone(&collector_stop);
        std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                match collector_listener.accept() {
                    Ok(_) => {
                        hits.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => std::thread::sleep(Duration::from_millis(20)),
                }
            }
        })
    };

    let data = TempDir::new().expect("tempdir");
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());
    write_two_stage_workflow(repo.path());

    // A pipeline the daemon was *not* given, installed process-wide. Anything
    // reaching for an ambient tracer or meter lands in these exporters.
    let spans = InMemorySpanExporter::default();
    let metrics = InMemoryMetricExporter::default();
    let ambient_traces = SdkTracerProvider::builder()
        .with_simple_exporter(spans.clone())
        .build();
    let ambient_meters = SdkMeterProvider::builder()
        .with_reader(PeriodicReader::builder(metrics.clone()).build())
        .build();
    opentelemetry::global::set_tracer_provider(ambient_traces.clone());
    opentelemetry::global::set_meter_provider(ambient_meters.clone());

    let (handle, _fake) =
        start_fake(data.path(), [FakeStep::complete(), FakeStep::complete()]).await;
    let created = submit(
        &handle,
        repo.path(),
        "no trace",
        json!({"workflow": "tiny"}),
    )
    .await;
    assert_eq!(created["work"]["state"], "completed");
    handle.shutdown().await;

    // Shutdown closes the event channel, which is what makes the export task
    // force-flush; give a regression's flush time to reach the listener
    // before concluding that nothing dialed it.
    tokio::time::sleep(Duration::from_millis(750)).await;
    collector_stop.store(true, std::sync::atomic::Ordering::Relaxed);
    collector_thread.join().expect("collector thread");
    assert_eq!(
        collector_hits.load(std::sync::atomic::Ordering::Relaxed),
        0,
        "a disabled daemon dialed a collector nothing told it about"
    );

    let _ = ambient_traces.force_flush();
    let _ = ambient_meters.force_flush();
    assert!(
        spans.get_finished_spans().expect("spans").is_empty(),
        "a daemon with export off must emit no spans"
    );
    assert!(
        exported_metric_names(&metrics).is_empty(),
        "a daemon with export off must emit no metrics"
    );

    // The kill probe for the two assertions above: the ambient pipeline they
    // inspect is live, so "empty" is a fact about the daemon rather than
    // about an exporter nothing could ever have reached.
    {
        use opentelemetry::trace::{Span as _, Tracer as _};
        opentelemetry::global::tracer("m5-probe")
            .start("probe")
            .end();
    }
    opentelemetry::global::meter("m5-probe")
        .u64_counter("m5_probe_total")
        .build()
        .add(1, &[]);
    let _ = ambient_traces.force_flush();
    let _ = ambient_meters.force_flush();
    assert!(
        !spans.get_finished_spans().expect("spans").is_empty(),
        "the ambient tracer must be reachable, or the emptiness above proves nothing"
    );
    assert!(
        exported_metric_names(&metrics).contains("m5_probe_total"),
        "the ambient meter must be reachable, or the emptiness above proves nothing"
    );
}

/// "Off means nothing runs" as a property of the source, not of one run.
///
/// The runtime probes in `t5_disabled_export_runs_no_exporter_machinery` can
/// only observe the daemon they were handed. This covers the shape that no
/// runtime probe reaches: that a pipeline can be *constructed* in exactly one
/// place, that the place is gated on the environment, and that the function
/// every other test drives cannot build one at all.
fn assert_telemetry_is_built_in_one_gated_place() {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    const CONSTRUCTORS: &[&str] = &[
        "Telemetry::otlp",
        "Telemetry::with_exporters",
        "Telemetry::from_config",
    ];
    for file in rust_sources(&src) {
        let relative = file.strip_prefix(&src).expect("under src").to_path_buf();
        // `telemetry.rs` defines them; `daemon.rs` is the one caller, and is
        // checked far more precisely just below.
        if relative == Path::new("telemetry.rs") || relative == Path::new("daemon.rs") {
            continue;
        }
        let text = std::fs::read_to_string(&file).expect("read source");
        for constructor in CONSTRUCTORS {
            assert!(
                !text.contains(constructor),
                "{} calls {constructor}; the export pipeline has exactly one construction site",
                relative.display()
            );
        }
    }

    let daemon = std::fs::read_to_string(src.join("daemon.rs")).expect("read daemon");

    // Every test in this file drives `start_with` with an explicit config. If
    // it built a pipeline of its own, none of them could tell.
    let start_with = item_source(&daemon, "pub async fn start_with(");
    assert!(
        !start_with.contains("Telemetry::"),
        "start_with must never construct an export pipeline; it exports only what it is handed"
    );

    // And the real entrypoint builds one only behind the environment gate.
    let run_until_signal = item_source(&daemon, "pub async fn run_until_signal(");
    assert!(
        run_until_signal.contains("TelemetryConfig::from_env()")
            && run_until_signal.contains("Telemetry::from_config("),
        "`sgt daemon` must build its pipeline from the environment config, which defaults to off"
    );
}

/// The source text of the top-level item beginning with `signature`.
///
/// Ends at the first line consisting of a single `}`, which rustfmt
/// guarantees is the item's own closing brace and nothing nested.
fn item_source<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("{signature} not found"));
    let rest = &source[start..];
    let end = rest
        .find("\n}\n")
        .unwrap_or_else(|| panic!("no closing brace for {signature}"));
    &rest[..end]
}

// -------------------------------------------------- 6. analytics smoke

/// Acceptance 6. Three of §22's example questions, answered through the
/// daemon — over the API and over the CLI, which is the surface a human has.
#[tokio::test]
async fn t6_the_daemon_answers_three_section_22_questions() {
    let data = DataDir::new();
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());
    write_two_stage_workflow(repo.path());

    // A run that blocks, is retried, and then completes: enough history for
    // "how long does work remain blocked" and "which backend retries most"
    // to have non-trivial answers.
    let (handle, _fake) = start_fake(
        data.path(),
        [
            FakeStep::blocked("waiting on a human"),
            FakeStep::complete(),
            FakeStep::complete(),
        ],
    )
    .await;
    let created = submit(
        &handle,
        repo.path(),
        "answer me",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = created["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(created["work"]["state"], "blocked", "created: {created}");
    let (status, retried) = post(
        &handle,
        &format!("/v1/work/{work_id}/retry"),
        json!({"command_id": ulid()}),
    )
    .await;
    assert_eq!(status, reqwest::StatusCode::OK, "retry: {retried}");

    // §22: "How long does work remain blocked?"
    let blocked = get(&handle, "/v1/analytics/blocked_time_per_work").await;
    let row = find_row(&blocked, 0, &work_id);
    assert_eq!(
        column(&blocked, &row, "blocked_episodes").as_i64(),
        Some(1),
        "one blocked episode: {blocked}"
    );
    assert!(
        column(&blocked, &row, "blocked_ms").as_i64().is_some(),
        "a closed blocked interval is measured: {blocked}"
    );

    // §22: "Which backend produces the most retries?"
    let retries = get(&handle, "/v1/analytics/backend_retries").await;
    let row = find_row(&retries, 0, FAKE_BACKEND_NAME);
    assert_eq!(
        column(&retries, &row, "retries").as_i64(),
        Some(1),
        "the retried stage attempt is attributed to the routed backend: {retries}"
    );

    // §22: "What did this execution touch?"
    let touched = get(&handle, "/v1/analytics/execution_touched").await;
    let rows = touched["rows"].as_array().expect("rows");
    let mine: Vec<&Value> = rows
        .iter()
        .filter(|row| row[1].as_str() == Some(work_id.as_str()))
        .collect();
    assert!(
        mine.len() >= 2,
        "each stage attempt is its own execution: {touched}"
    );
    for row in &mine {
        assert_eq!(
            column(&touched, row, "repositories").as_i64(),
            Some(1),
            "the execution touched the one repository the surface bound: {touched}"
        );
    }

    // An unknown question is a structured 404, not a 500 or an empty table.
    let (status, body) = get_status(&handle, "/v1/analytics/no_such_question").await;
    assert_eq!(status, reqwest::StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "unknown_query");

    let endpoint = handle.endpoint.clone();
    handle.shutdown().await;
    let _ = endpoint;

    // The same answers through the CLI, over a daemon it spawns itself.
    let listed = sgt(&data, &["analytics"]);
    for canned in CANNED_QUERIES {
        assert!(
            listed.contains(canned.name),
            "`sgt analytics` must list {}: {listed}",
            canned.name
        );
    }
    assert!(
        listed.contains("graph_edges"),
        "the projection's tables are reported: {listed}"
    );
    let answered = sgt(&data, &["analytics", "backend_retries"]);
    assert!(
        answered.contains("Which backend produces the most retries?")
            && answered.contains(FAKE_BACKEND_NAME),
        "`sgt analytics backend_retries`: {answered}"
    );

    // And §23's minimal CLI rendering, which must show provenance.
    let rendered = sgt(&data, &["work", "show", &work_id, "--graph"]);
    assert!(
        rendered.contains("--executes-->") && rendered.contains("seq "),
        "`sgt work show --graph` renders edges with their source seq: {rendered}"
    );
    let as_json: Value = serde_json::from_str(&sgt(
        &data,
        &["--json", "work", "show", &work_id, "--graph"],
    ))
    .expect("json graph");
    assert!(
        as_json["edges"]
            .as_array()
            .expect("edges")
            .iter()
            .all(|e| e["source_seq"].is_number()),
        "every edge carries provenance in the JSON form too: {as_json}"
    );

    // W8: `work show`'s human form must render the output pointer too — the
    // API's `output` key names, per repository, the retained branch,
    // worktree path and finalize commit; before this fix cli.rs's key
    // whitelist silently dropped it (and `teardown`) from the printed
    // record.
    let shown = sgt(&data, &["work", "show", &work_id]);
    assert!(
        shown.contains("retained_branch") && shown.contains(&format!("sergeant/{work_id}")),
        "`sgt work show` (human form) must render the output pointer's retained branch: {shown}"
    );
}

/// Run `sgt` against a data dir, returning stdout (the CLI spawns and reuses
/// its own daemon, so this exercises the client path a human has).
fn sgt(data_dir: &DataDir, args: &[&str]) -> String {
    let output = Command::new(SGT)
        .arg("--data-dir")
        .arg(data_dir.path())
        .args(args)
        .output()
        .expect("run sgt");
    assert!(
        output.status.success(),
        "sgt {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// The first row whose column `index` equals `value`.
fn find_row(result: &Value, index: usize, value: &str) -> Value {
    result["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .find(|row| row[index].as_str() == Some(value))
        .unwrap_or_else(|| panic!("no row with {value} at column {index}: {result}"))
        .clone()
}

/// One named column of a row.
fn column(result: &Value, row: &Value, name: &str) -> Value {
    let index = result["columns"]
        .as_array()
        .expect("columns")
        .iter()
        .position(|c| c.as_str() == Some(name))
        .unwrap_or_else(|| panic!("no column {name} in {result}"));
    row[index].clone()
}

// ------------------------- §22 tables only a real adapter can fill

/// Four verbatim stream-json lines from a real 2.1.226 turn (provenance in
/// `tests/fixtures/README.md`): a `tool_use`, its `tool_result`, assistant
/// text, and the result envelope with `modelUsage`.
const RECORDED_TURN: &str = include_str!("fixtures/claude-2.1.226-turn.jsonl");

/// A stand-in `claude` that passes the probe and replays a recorded turn.
/// Leaner than M4's — this milestone measures what the *projection* does with
/// real adapter output, not what the adapter puts on the command line.
fn stub_claude(dir: &Path) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("claude-stub");
    let replay = dir.join("claude-replay.jsonl");
    std::fs::write(&replay, RECORDED_TURN).expect("write replay");
    let flags = "--print --verbose --output-format --session-id --resume --setting-sources \
                 --model --permission-mode --dangerously-skip-permissions";
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\ncase \"$1\" in\n  --version) echo '2.1.226 (Claude Code)';;\n  \
             --help) echo '{flags}';;\n  *) cat >/dev/null; cat '{replay}';;\nesac\n",
            replay = replay.display()
        ),
    )
    .expect("write stub");
    let mut permissions = std::fs::metadata(&path).expect("stat").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).expect("chmod");
    // A file another process still holds open for writing cannot be exec'd;
    // absorb the ETXTBSY window before handing it to the adapter.
    let deadline = Instant::now() + Duration::from_secs(10);
    while let Err(e) = Command::new(&path).arg("--version").output() {
        assert!(
            e.raw_os_error() == Some(26) && Instant::now() < deadline,
            "the stub is not runnable: {e}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    path
}

/// A daemon running the *real* Claude adapter over the stand-in binary.
///
/// The backend registry is deliberately empty: the daemon registers the real
/// adapter itself, so nothing here is a fake wearing the adapter's name.
async fn start_claude(data_dir: &Path, telemetry: Option<Arc<Telemetry>>) -> DaemonHandle {
    let mut claude = ClaudeConfig::new(data_dir);
    claude.executable = stub_claude(data_dir);
    daemon::start_with(
        data_dir,
        DaemonConfig {
            backends: Arc::new(BackendRegistry::new()),
            default_backend: Some(CLAUDE_BACKEND_NAME.to_string()),
            claude: Some(claude),
            telemetry,
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start")
}

/// §22's `messages`, `tool_calls` and `usage` tables are filled from real
/// §27 events, not from a shape this test invented: a recorded turn goes
/// through the actual Claude adapter, launched by the actual daemon for a
/// real submission, and the projection folds whatever lands in the journal.
///
/// The same run is what puts a `tool.*` span under §28's `execution.turn` —
/// the fake backend has no tools, so without this the bottom level of §28's
/// tree would be unmeasured.
#[tokio::test]
async fn real_adapter_events_populate_the_conversation_tables_and_tool_spans() {
    let data = TempDir::new().expect("tempdir");
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());
    write_two_stage_workflow(repo.path());

    let spans = InMemorySpanExporter::default();
    let telemetry = Arc::new(Telemetry::with_exporters(
        spans.clone(),
        InMemoryMetricExporter::default(),
    ));
    let handle = start_claude(data.path(), Some(telemetry.clone())).await;
    let created = submit(
        &handle,
        repo.path(),
        "say hello",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = created["work"]["id"].as_str().expect("work id").to_string();

    let events = wait_for_kinds(
        data.path(),
        &[
            "conversation.user",
            "tool.requested",
            "tool.completed",
            "usage.updated",
        ],
    );
    let execution_id = events
        .iter()
        .find(|e| e.kind == "execution.started")
        .and_then(|e| e.payload["execution"]["execution_id"].as_str())
        .expect("an execution")
        .to_string();

    // Fold the real journal into the projection.
    let mut analytics = Analytics::in_memory(events.iter().cloned().map(Ok)).expect("projection");
    let counts: BTreeMap<String, i64> = analytics
        .table_counts()
        .expect("counts")
        .into_iter()
        .collect();
    assert!(
        counts["messages"] >= 2,
        "user prompt and assistant text: {counts:?}"
    );
    assert_eq!(
        counts["tool_calls"], 1,
        "the recorded turn made one tool call"
    );
    assert_eq!(counts["usage"], 1, "one turn, one usage row");

    // The real turn's numbers reach the §22 tables, not just their row
    // counts: the recorded envelope's own token totals come back out.
    let tokens = analytics.query("token_totals_per_work").expect("query");
    let index = |name: &str| tokens.columns.iter().position(|c| c == name).expect(name);
    let row = tokens
        .rows
        .iter()
        .find(|row| row[0] == json!(work_id))
        .unwrap_or_else(|| panic!("no token totals for the work: {tokens:?}"));
    assert_eq!(row[index("input_tokens")], json!(17), "from the recording");
    assert_eq!(
        row[index("output_tokens")],
        json!(178),
        "from the recording"
    );
    assert_eq!(row[index("cache_read_tokens")], json!(49727));

    // And the tool call is joined to its result, not recorded twice.
    let calls = analytics.table_rows("tool_calls").expect("tool_calls");
    let at = |name: &str| calls.columns.iter().position(|c| c == name).expect(name);
    let call = calls.rows.first().expect("one tool call");
    assert_eq!(call[at("name")], json!("Bash"));
    assert_eq!(
        call[at("is_error")],
        json!(false),
        "the tool_result's own verdict, folded onto the request: {calls:?}"
    );
    assert!(
        call[at("completed_seq")].is_number(),
        "request and result are one row, not two: {calls:?}"
    );
    // The name is *validated*, not merely handed to SQL and allowed to fail
    // there: the error says "unknown *table*" — the one error this call can
    // raise, so it must not borrow the name of a different surface — and a
    // name shaped like an injection is refused by the same check rather than
    // reaching the parser.
    for name in ["not_a_table", "work\"; DROP TABLE work; --"] {
        match analytics.table_rows(name) {
            Err(AnalyticsError::UnknownTable { name: reported }) => {
                assert_eq!(reported, name);
            }
            other => panic!("{name:?} must be refused by name, not by SQL: {other:?}"),
        }
    }

    // §22: "What did this execution touch?" — with real tool and message
    // counts behind it now.
    let touched = analytics.query("execution_touched").expect("query");
    let at = |name: &str| touched.columns.iter().position(|c| c == name).expect(name);
    let row = touched
        .rows
        .iter()
        .find(|row| row[0] == json!(execution_id))
        .unwrap_or_else(|| panic!("no row for the execution: {touched:?}"));
    assert_eq!(row[at("tool_calls")], json!(1));
    assert!(row[at("messages")].as_i64().unwrap_or(0) >= 2);
    assert_eq!(row[at("repositories")], json!(1));

    // The graph reaches the §23 conversation nodes, with provenance.
    let graph = analytics.graph_neighborhood(&work_id).expect("graph");
    let relations: BTreeSet<&str> = graph
        .edges
        .iter()
        .filter_map(|e| e["relation"].as_str())
        .collect();
    assert!(
        relations.contains("produced") && relations.contains("requested"),
        "conversation and tool edges: {relations:?}"
    );
    assert!(
        relations.contains("caused"),
        "§23's Message → caused → ToolCall, from the journal's own causation link: {relations:?}"
    );
    assert!(
        relations.contains("used_model"),
        "the model the turn actually ran on: {relations:?}"
    );
    let journal = Provenance::new(events.iter().cloned());
    for edge in &graph.edges {
        let seq = edge["source_seq"].as_u64().expect("source_seq");
        let event = journal.event(seq).expect("edge names a real event");
        assert!(
            journal.justifies(edge, event),
            "edge {edge} vs {}",
            event.kind
        );
    }

    // §28's tool span, from the same run, nested under its execution.
    // The tool span closes when the tool result lands; the execution span
    // only when the daemon retires the turn, so wait for the first and take
    // the tree after shutdown.
    wait_for_spans(&spans, 1).await;
    handle.shutdown().await;
    telemetry.force_flush();
    let finished = spans.get_finished_spans().expect("spans");
    let tool = finished
        .iter()
        .find(|s| s.name == "tool.bash")
        .unwrap_or_else(|| {
            panic!(
                "the recorded Bash tool call must be a §28 tool span: {:?}",
                finished.iter().map(|s| s.name.clone()).collect::<Vec<_>>()
            )
        });
    let execution = finished
        .iter()
        .find(|s| s.name == "execution.turn")
        .expect("an execution.turn span");
    assert_eq!(
        tool.parent_span_id,
        execution.span_context.span_id(),
        "§28 nests tool spans under the turn that requested them"
    );
    assert_eq!(
        attribute(tool, "sergeant.execution.id").as_deref(),
        Some(execution_id.as_str())
    );
}

/// Acceptance 1 again, over the §22 tables a fake backend cannot fill.
///
/// `t1` and `t4` both drive the fake backend, which emits no §27 conversation
/// events, so `messages`, `tool_calls` and `usage` are compared there as
/// 0 == 0 — as are the `produced`, `requested`, `caused` and `used_model`
/// graph edges. Deleting the whole conversation half of the fold leaves both
/// of those tests green, which makes "rebuild reproduces the projection" an
/// unproven claim for exactly the part of §22 that has the most fold state
/// behind it.
///
/// So the comparison is made here against a real adapter's journal, and
/// between the two populations that can actually disagree: the **live,
/// incrementally folded** projection the daemon answered from (`with_analytics`
/// catches it up at read time) against a from-scratch [`Analytics::rebuild`]
/// — the daemon's own startup path — over the identical journal prefix.
#[tokio::test]
async fn rebuilding_reproduces_the_conversation_tables_a_real_adapter_fills() {
    let data = TempDir::new().expect("tempdir");
    let repo = TempDir::new().expect("tempdir");
    init_repo(repo.path());
    write_two_stage_workflow(repo.path());

    let handle = start_claude(data.path(), None).await;
    let created = submit(
        &handle,
        repo.path(),
        "say hello",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = created["work"]["id"].as_str().expect("work id").to_string();
    wait_for_kinds(
        data.path(),
        &[
            "conversation.user",
            "tool.requested",
            "tool.completed",
            "usage.updated",
        ],
    );
    wait_for_a_quiet_journal(data.path()).await;

    let (live, high_water) = api_projection(&handle).await;
    let live_graph = get(&handle, &format!("/v1/graph/work/{work_id}")).await;
    assert_eq!(
        live_graph["projection"]["last_seq"].as_u64(),
        Some(high_water),
        "the journal moved under the snapshot; it is not comparable to a rebuild"
    );
    let events: Vec<Event> = journal_events(data.path())
        .into_iter()
        .filter(|event| event.seq <= high_water)
        .collect();
    handle.shutdown().await;

    // The tables at issue are genuinely in the comparison, not 0 == 0.
    for table in ["messages", "tool_calls", "usage"] {
        assert!(
            live["tables"][table].as_i64().unwrap_or(0) > 0,
            "the adapter run must fill §22's {table}: {}",
            live["tables"]
        );
    }

    let elsewhere = TempDir::new().expect("tempdir");
    let mut rebuilt =
        Analytics::rebuild(elsewhere.path(), events.into_iter().map(Ok)).expect("rebuild");
    let rebuilt_projection = local_projection(&mut rebuilt);
    assert_eq!(
        serde_json::to_string_pretty(&live).expect("json"),
        serde_json::to_string_pretty(&rebuilt_projection).expect("json"),
        "the rebuilt projection answered differently from the live one"
    );

    // And the graph edges that exist only on this path come back too.
    let rebuilt_graph = rebuilt.graph_neighborhood(&work_id).expect("graph");
    let relations: BTreeSet<&str> = rebuilt_graph
        .edges
        .iter()
        .filter_map(|e| e["relation"].as_str())
        .collect();
    for expected in ["produced", "requested", "caused", "used_model"] {
        assert!(
            relations.contains(expected),
            "the rebuild must derive the §23 {expected:?} edge: {relations:?}"
        );
    }
    assert_eq!(json!(rebuilt_graph.nodes), live_graph["nodes"]);
    assert_eq!(json!(rebuilt_graph.edges), live_graph["edges"]);
}

/// Table counts and every canned answer, in the one shape the daemon's API and
/// a locally built [`Analytics`] both reduce to, with the journal seq the
/// projection had folded up to when it answered.
///
/// Unlike [`analytics_snapshot`] this keeps the `events` count: a rebuild over
/// exactly that prefix must reproduce it too.
async fn api_projection(handle: &DaemonHandle) -> (Value, u64) {
    let index = get(handle, "/v1/analytics").await;
    let high_water = index["projection"]["last_seq"]
        .as_u64()
        .expect("a high-water mark");
    let mut tables = serde_json::Map::new();
    for table in index["tables"].as_array().expect("tables") {
        tables.insert(
            table["table"].as_str().expect("table").to_string(),
            table["rows"].clone(),
        );
    }
    let mut answers = serde_json::Map::new();
    for canned in CANNED_QUERIES {
        let answer = get(handle, &format!("/v1/analytics/{}", canned.name)).await;
        assert_eq!(
            answer["projection"]["last_seq"].as_u64(),
            Some(high_water),
            "the journal moved under the snapshot; it is not comparable to a rebuild"
        );
        answers.insert(
            canned.name.to_string(),
            json!({"columns": answer["columns"], "rows": answer["rows"]}),
        );
    }
    (json!({"tables": tables, "answers": answers}), high_water)
}

/// The same shape as [`api_projection`], from a projection built in-process.
fn local_projection(analytics: &mut Analytics) -> Value {
    let mut tables = serde_json::Map::new();
    for (table, rows) in analytics.table_counts().expect("counts") {
        tables.insert(table, json!(rows));
    }
    let mut answers = serde_json::Map::new();
    for canned in CANNED_QUERIES {
        let result = analytics.query(canned.name).expect(canned.name);
        answers.insert(
            canned.name.to_string(),
            json!({"columns": result.columns, "rows": result.rows}),
        );
    }
    json!({"tables": tables, "answers": answers})
}

/// Wait until the journal stops growing, so a snapshot of the live projection
/// and a rebuild over the same prefix describe the same, settled run.
///
/// `async` with a `tokio::time::sleep` for the same reason [`wait_for_spans`]
/// is: the daemon this waits on runs on this test's runtime.
async fn wait_for_a_quiet_journal(data_dir: &Path) {
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut previous = 0usize;
    loop {
        let len = journal_events(data_dir).len();
        if len > 0 && len == previous {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "the journal never settled ({len} events)"
        );
        previous = len;
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

/// Block until every kind in `kinds` has been journaled (the adapter's reader
/// thread and the sink's committer are both asynchronous).
fn wait_for_kinds(data_dir: &Path, kinds: &[&str]) -> Vec<Event> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let events = journal_events(data_dir);
        let seen: BTreeSet<&str> = events.iter().map(|e| e.kind.as_str()).collect();
        if kinds.iter().all(|kind| seen.contains(kind)) {
            return events;
        }
        assert!(
            Instant::now() < deadline,
            "only saw {seen:?}, waiting for {kinds:?}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// The contract's third Unknown: is the OTLP path smoke-testable offline?
///
/// It is — a bare TCP listener is enough of a collector to prove the wiring.
/// This asserts what the acceptance tests deliberately cannot: that the
/// *configured production exporter*, not just the in-memory one, actually
/// serializes the fold and puts it on the wire at the endpoint the config
/// names. Without it, `Telemetry::otlp` would be an advertised capability
/// with no test (L8).
#[test]
fn otlp_export_reaches_a_collector_listening_on_the_configured_endpoint() {
    use std::io::{Read, Write};
    use std::sync::Mutex;

    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind a stand-in collector");
    let endpoint = format!("http://{}", listener.local_addr().expect("addr"));
    listener.set_nonblocking(true).expect("nonblocking");
    let requests: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let collector = {
        let requests = Arc::clone(&requests);
        std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(20);
            while Instant::now() < deadline {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream
                            .set_read_timeout(Some(Duration::from_secs(2)))
                            .expect("read timeout");
                        let mut buffer = [0u8; 8192];
                        let read = stream.read(&mut buffer).unwrap_or(0);
                        requests
                            .lock()
                            .expect("requests")
                            .push(String::from_utf8_lossy(&buffer[..read]).to_string());
                        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
                    }
                    Err(_) => {
                        if requests
                            .lock()
                            .expect("requests")
                            .iter()
                            .any(|request| is_trace_post(request))
                        {
                            return;
                        }
                        std::thread::sleep(Duration::from_millis(20));
                    }
                }
            }
        })
    };

    let telemetry = Telemetry::otlp(&endpoint).expect("an OTLP pipeline");
    for draft in synthetic_run("01M5OTLP") {
        // The fold needs journal-shaped events; seqs and timestamps are the
        // journal's own stamping, so build them the same way it does.
        telemetry.record(&draft.into_event(next_seq()));
    }
    telemetry.force_flush();
    telemetry.shutdown();
    collector.join().expect("collector thread");

    let requests = requests.lock().expect("requests");
    assert!(
        requests.iter().any(|request| is_trace_post(request)),
        "no OTLP trace export reached the endpoint the config named: {requests:?}"
    );
}

/// Whether a raw HTTP request is an OTLP trace export with a body.
fn is_trace_post(request: &str) -> bool {
    request.starts_with("POST /v1/traces")
        && request.contains("application/x-protobuf")
        && !request.ends_with("\r\n\r\n")
}

/// Monotonic seqs for the synthetic events above.
fn next_seq() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(1);
    SEQ.fetch_add(1, Ordering::Relaxed)
}

// ------------------------------------------------ measured rebuild cost

/// The contract's Unknown: is rebuild-from-scratch fast enough that the
/// incremental optimization is unnecessary? Measured here rather than
/// assumed, and asserted only against a bound so loose that only a genuine
/// regression (an accidental O(n²), a lost transaction) trips it.
#[test]
fn rebuild_latency_on_a_realistic_journal_is_measured() {
    let data = TempDir::new().expect("tempdir");
    let mut journal = Journal::open(data.path()).expect("journal");
    let works = 2_000;
    for index in 0..works {
        let work_id = format!("01M5BENCH{index:016}");
        for draft in synthetic_run(&work_id) {
            journal.append(draft).expect("append");
        }
    }
    let events: Vec<Event> = journal
        .replay()
        .expect("replay")
        .map(|e| e.expect("event"))
        .collect();
    let total = events.len();

    let started = Instant::now();
    let mut analytics =
        Analytics::rebuild(data.path(), events.into_iter().map(Ok)).expect("rebuild");
    let elapsed = started.elapsed();
    let counts: BTreeMap<String, i64> = analytics
        .table_counts()
        .expect("counts")
        .into_iter()
        .collect();
    assert_eq!(counts["events"], total as i64);
    assert_eq!(counts["work"], works as i64);

    let query_started = Instant::now();
    for canned in CANNED_QUERIES {
        analytics.query(canned.name).expect(canned.name);
    }
    let query_elapsed = query_started.elapsed();

    println!(
        "rebuild: {total} events -> {:?} ({:.1} events/s); all {} canned queries: {:?}",
        elapsed,
        total as f64 / elapsed.as_secs_f64().max(f64::EPSILON),
        CANNED_QUERIES.len(),
        query_elapsed,
    );
    assert!(
        elapsed < Duration::from_secs(30),
        "rebuilding {total} events took {elapsed:?}"
    );
}

/// One work's worth of lifecycle events, in the shapes the engine writes.
fn synthetic_run(work_id: &str) -> Vec<sergeant_rs::domain::event::EventDraft> {
    use sergeant_rs::domain::event::{EventDraft, EventSource};
    let source = || EventSource::new("daemon", "bench");
    let draft =
        |kind: &str, payload: Value| EventDraft::new(source(), kind, payload).with_work_id(work_id);
    vec![
        draft(
            "work.submitted",
            json!({"work": {
                "id": work_id, "intent": "bench", "state": "pending",
                "created_by": "bench", "created_at": "2026-01-01T00:00:00.000Z",
                "origin_client": "cli", "workspace": "bench", "repositories": [],
            }}),
        ),
        draft(
            "surface.materialized",
            json!({"surface": {"work_id": work_id, "root": "/tmp/bench", "bindings": [{
                "repository": "api", "source_path": "/tmp/api", "base_branch": "main",
                "base_sha": "abc", "worktree_path": "/tmp/wt", "work_branch": "sergeant/x",
                "head_sha": "abc",
            }]}}),
        ),
        draft(
            "workflow.bound",
            json!({
                "workflow": {"name": "tiny", "version": "1", "source": "builtin", "stages": []},
                "backend": "fake", "route_source": "default", "workspace": "bench",
            }),
        ),
        draft("work.started", json!({"backend": "fake"})),
        draft(
            "stage.entered",
            json!({"stage_id": "00-first", "index": 0, "attempt": 1}),
        ),
        draft(
            "execution.started",
            json!({"execution": {
                "execution_id": format!("{work_id}-e1"), "backend": "fake",
                "native_id": format!("{work_id}-n1"), "stage_id": "00-first",
                "attempt": 1, "stop_requested": false,
            }}),
        ),
        draft("stage.completed", json!({"stage_id": "00-first"})),
        draft("work.completed", json!({})),
    ]
}

// -------------------------------------------------------- R-MVP1-2's output pointer

/// R-MVP1-2's sibling, not itself a ruling: once a work's surface has been
/// torn down, `work show` names — per repository — the source repository,
/// the retained branch, the worktree path, and the finalize commit
/// (`surface::teardown`'s own extension, `BindingTeardown::final_sha`) —
/// without decoding the journal. Proven end to end through a real daemon and
/// a real repository, then again after a restart: rebuild-on-start is the
/// only population path (R-MVP1-9), so the pointer must read back
/// byte-identically from a freshly replayed journal, not just from the
/// process that wrote it — and `Completed` is exactly the absorbing state
/// R-MVP1-9 evicts, so this also exercises eviction's read-side
/// re-derivation, not merely a live in-memory hit.
#[tokio::test]
async fn r_mvp1_2_the_output_pointer_names_source_branch_worktree_and_finalize_commit() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let repo = repos.path().join("solo");
    let base_sha = init_repo(&repo);
    write_two_stage_workflow(&repo);

    let (handle, _fake) =
        start_fake(data.path(), [FakeStep::complete(), FakeStep::complete()]).await;
    let body = submit(
        &handle,
        &repo,
        "point me at the output",
        json!({"workflow": "tiny"}),
    )
    .await;
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    assert_eq!(body["work"]["state"], "completed", "{body}");

    let output = body["output"].clone();
    assert_eq!(output["work_id"], work_id);
    assert_eq!(output["clean"], true);
    let repositories = output["repositories"]
        .as_array()
        .expect("repositories array");
    assert_eq!(repositories.len(), 1);
    let repo_out = &repositories[0];
    assert_eq!(repo_out["repository"], "solo");
    assert_eq!(repo_out["source_repo"], repo.display().to_string());
    assert_eq!(repo_out["retained_branch"], format!("sergeant/{work_id}"));
    assert_eq!(repo_out["disposition"], "removed");
    // Nothing was ever committed inside the worktree, so the finalize commit
    // is exactly the SHA the surface was cut from — the honest answer when a
    // closing stage declares no `promote` output.
    assert_eq!(repo_out["finalize_commit"], base_sha);
    // The branch itself really is there, at that commit, in the source repo
    // — the pointer names a real, checkable fact, not a projection artifact.
    assert_eq!(
        git(
            &repo,
            &["rev-parse", &format!("refs/heads/sergeant/{work_id}")]
        ),
        base_sha
    );

    handle.shutdown().await;

    // Restart: rebuild-on-start replays the journal from scratch. The
    // pointer — like every other view `work show` composes — must read back
    // byte-identical, whether or not R-MVP1-9 eviction reclaimed this work's
    // run in between (it did: `Completed` is absorbing).
    let (handle2, _fake2) = start_fake(data.path(), []).await;
    let after = get(&handle2, &format!("/v1/work/{work_id}")).await;
    assert_eq!(
        after["output"], output,
        "the output pointer must survive a restart byte-identically"
    );
    handle2.shutdown().await;
}
