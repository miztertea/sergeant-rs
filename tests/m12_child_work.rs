//! M12: child Work — causation transport, validation, independence, and
//! `sgt run --wait` (S2 W1 §5–§8/§13.5–§13.7; decisions E5/E6/E7/E8-amended/E9).
//!
//! The helper that builds the causation triple is unit-tested next to its own
//! code in `src/backend/mod.rs`; each adapter's own env-capture suite proves
//! the triple reaches *that adapter's* spawned process. This suite is the
//! other half — the claim end to end, through the real daemon, the real
//! engine and the real `sgt` binary:
//!
//! 1. the engine hands every managed execution the estate coordinate a child
//!    `sgt -C … run` needs (E5, the plumbing half);
//! 2. an actor process really does inherit all three values and can spend
//!    them on a child submission that the daemon validates (W1 §13.5);
//! 3. a forged or stale claim is **accepted** and journaled as unverified
//!    rather than refused (E8 as amended — the ratification's "rather than by
//!    refusal" clause);
//! 4. the child is independent: parent completion and parent cancellation do
//!    not cascade, scope is not inherited, branches differ (W1 §13.6/§13.7);
//! 5. `--wait` observes a child to terminal without any engine hold state
//!    (E9);
//! 6. the supersession is narrow: a bare `sgt run` from inside a Work surface
//!    still refuses.

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig, DaemonHandle};
use sergeant_rs::domain::event::Event;
use sergeant_rs::runtime::analytics::Analytics;
use sergeant_rs::runtime::journal::Journal;

mod support;

// ---------------------------------------------------------------- helpers

fn ulid() -> String {
    ulid::Ulid::generate().to_string()
}

fn http() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("client")
}

/// A daemon over the fake backend, plus the adapter handle itself — the
/// `StartRequest`s it recorded are how this suite reads what the engine
/// actually handed a managed execution.
async fn start_fake(
    data_dir: &Path,
    script: impl IntoIterator<Item = FakeStep>,
) -> (DaemonHandle, Arc<FakeBackend>) {
    let fake = Arc::new(FakeBackend::scripted(FAKE_BACKEND_NAME, script));
    let registry = BackendRegistry::new().with(fake.clone());
    let handle = daemon::start_with(
        data_dir,
        DaemonConfig {
            backends: Arc::new(registry),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            claude: None,
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");
    (handle, fake)
}

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

/// Submit one Work against `estate`, with whatever extra top-level keys the
/// caller wants folded into the body (the claimed-causation fields, for the
/// tests that forge them).
async fn submit_with(
    client: &reqwest::Client,
    handle: &DaemonHandle,
    estate: &Path,
    intent: &str,
    extra: Value,
) -> (reqwest::StatusCode, Value) {
    let mut body = json!({
        "command_id": ulid(),
        "intent": intent,
        "estate_root": estate,
        "origin": {"client": "cli", "cwd": estate},
    });
    if let Some(extra) = extra.as_object() {
        for (key, value) in extra {
            body[key] = value.clone();
        }
    }
    post(client, handle, "/v1/work", body).await
}

// ------------------------------------------------------------------ tests

/// E5, the plumbing half: every managed execution is handed the canonical
/// estate root of the Work it serves — the same coordinate the daemon stamps
/// on that Work's events — so `sgt -C "$SERGEANT_ESTATE_ROOT" run` addresses
/// the estate the daemon will validate the claim against.
///
/// Read off the `StartRequest` the engine actually built, not off a computed
/// value: this is the field the adapters' env merge reads from.
#[tokio::test]
async fn a_managed_execution_is_handed_the_estate_root_of_the_work_it_serves() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");

    let (handle, fake) = start_fake(data.path(), [FakeStep::complete_with("done")]).await;
    let client = http();
    let (status, body) = submit_with(&client, &handle, &estate, "parent work", json!({})).await;
    assert_eq!(status, 201, "submit must be accepted: {body}");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();

    let canonical = estate.canonicalize().expect("canonical estate");
    let starts = fake.starts();
    assert!(!starts.is_empty(), "the Work ran at least one stage");
    for start in &starts {
        assert_eq!(start.work_id, work_id);
        assert_eq!(
            start.estate_root.as_deref(),
            Some(canonical.as_path()),
            "every stage's request carries the Work's own canonical estate \
             root, not just the first: {start:?}"
        );
    }
    let start = &starts[0];

    // And the triple built from it is exactly the contract's three values,
    // carrying this execution's real coordinates — the helper is pure, so
    // this is the whole of what the adapters will merge.
    let env = sergeant_rs::backend::causation_env(start);
    assert_eq!(env["SERGEANT_ESTATE_ROOT"], canonical.to_string_lossy());
    assert_eq!(env["SERGEANT_WORK_ID"], work_id);
    assert_eq!(env["SERGEANT_EXECUTION_ID"], start.execution_id);

    handle.shutdown().await;
}

/// W1 §13.5 / E8: a claim that checks out against the daemon's own journal
/// is recorded as a relation on the child Work — and only then. The parent
/// here is parked mid-run, so the execution the child names is genuinely the
/// one running, which is exactly the shape a real `sgt -C … run` from inside
/// a managed execution produces.
#[tokio::test]
async fn a_validated_claim_becomes_a_recorded_parent_relation() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");

    let (handle, fake) = start_fake(
        data.path(),
        [FakeStep::needs_input("parked, holding its execution open")],
    )
    .await;
    let client = http();
    let (_, parent) = submit_with(&client, &handle, &estate, "the parent", json!({})).await;
    let parent_id = parent["work"]["id"]
        .as_str()
        .expect("parent id")
        .to_string();
    let execution_id = fake.starts()[0].execution_id.clone();

    let (status, child) = submit_with(
        &client,
        &handle,
        &estate,
        "the child",
        json!({
            "claimed_parent_work_id": parent_id,
            "claimed_parent_execution_id": execution_id,
        }),
    )
    .await;
    assert_eq!(status, 201, "the child is accepted: {child}");
    assert_eq!(
        child["work"]["parent_work_id"], parent_id,
        "the validated relation is recorded: {child}"
    );
    assert_eq!(child["work"]["parent_execution_id"], execution_id);
    assert!(
        child["work"]["causation_unverified"].is_null(),
        "a claim that validated leaves no unverified marker: {child}"
    );

    // W1-09: the relation lands in the existing analytical envelope for
    // durable query, rather than in a second agent-tree store. Folded from
    // the same one `work.submitted` payload the API answered from.
    let child_id = child["work"]["id"].as_str().expect("child id").to_string();
    let fleet = get(&client, &handle, "/v1/work").await;
    handle.shutdown().await;

    let events: Vec<Result<Event, _>> = Journal::replay_data_dir(data.path())
        .expect("replay")
        .collect();
    let mut analytics = Analytics::in_memory(events).expect("fold the journal");
    let work = analytics.table_rows("work").expect("work table");
    let column = |name: &str| {
        work.columns
            .iter()
            .position(|c| c == name)
            .unwrap_or_else(|| panic!("no {name} column in {:?}", work.columns))
    };
    let id_column = column("work_id");
    let row = work
        .rows
        .iter()
        .find(|row| row[id_column] == json!(child_id))
        .unwrap_or_else(|| panic!("no analytics row for the child: {:?}", work.rows));
    assert_eq!(row[column("parent_work_id")], json!(parent_id));
    assert_eq!(row[column("parent_execution_id")], json!(execution_id));
    assert_eq!(row[column("causation_unverified")], Value::Null);

    // W1 §10: the fleet body carries the relation on the row itself, so the
    // TUI can group parent and child without a second request per Work.
    let child_row = fleet["works"]
        .as_array()
        .expect("works")
        .iter()
        .find(|row| row["id"] == json!(child_id))
        .unwrap_or_else(|| panic!("no fleet row for the child: {fleet}"));
    assert_eq!(
        child_row["parent_work_id"],
        json!(parent_id),
        "the relation rides the fleet row itself (D6: the endpoint returns \
         everything, the TUI filters client-side): {child_row}"
    );
    // `WorkRow::parent`'s own projection of this key is unit-tested beside
    // it in `src/tui/fleet.rs` — the module is crate-private.
}

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

/// E8 **as amended**, the wave's load-bearing negative: a forged or stale
/// claim never refuses the submission. The Work is accepted, carries no
/// relation, and the daemon journals an explicit marker naming the failed
/// claim — visible in `work show`, in the same `work.submitted` payload as
/// the Work itself, so no crash window can separate the two (L6).
///
/// Four failing shapes, one per check the validator makes.
#[tokio::test]
async fn a_forged_or_stale_claim_is_accepted_causation_less_with_a_journaled_marker() {
    let repos = TempDir::new().expect("tempdir");
    let data = TempDir::new().expect("tempdir");
    let estate = repos.path().join("solo-estate");
    let other = repos.path().join("other-estate");
    support::scaffold_solo_estate(&estate, "solo");
    support::scaffold_solo_estate(&other, "other");

    let (handle, fake) = start_fake(
        data.path(),
        [
            FakeStep::needs_input("parked"),
            FakeStep::needs_input("parked"),
        ],
    )
    .await;
    let client = http();
    let (_, parent) = submit_with(&client, &handle, &estate, "the parent", json!({})).await;
    let parent_id = parent["work"]["id"]
        .as_str()
        .expect("parent id")
        .to_string();
    let (_, foreign) = submit_with(&client, &handle, &other, "a Work elsewhere", json!({})).await;
    let foreign_id = foreign["work"]["id"].as_str().expect("id").to_string();
    let live_execution = fake.starts()[0].execution_id.clone();

    for (what, claim, expect_in_reason) in [
        (
            "a parent Work this daemon has never journaled",
            json!({"claimed_parent_work_id": "01NEVERJOURNALEDWORKIDXXXXX"}),
            "not in this daemon's journal",
        ),
        (
            "a real Work belonging to a different estate",
            json!({"claimed_parent_work_id": foreign_id}),
            "not the addressed",
        ),
        (
            "a real parent, but an execution that is not its current one",
            json!({
                "claimed_parent_work_id": parent_id,
                "claimed_parent_execution_id": "01NOTTHECURRENTEXECUTION00",
            }),
            "is not",
        ),
        (
            "an execution claim with no parent Work to check it against",
            json!({"claimed_parent_execution_id": live_execution}),
            "names no parent Work",
        ),
    ] {
        let (status, child) =
            submit_with(&client, &handle, &estate, "the child", claim.clone()).await;
        assert_eq!(
            status, 201,
            "{what}: the submission PROCEEDS — E8 as amended never refuses \
             over causation ({claim}): {child}"
        );
        assert!(
            child["work"]["parent_work_id"].is_null()
                && child["work"]["parent_execution_id"].is_null(),
            "{what}: no relation may be recorded from a claim that failed: {child}"
        );
        let marker = &child["work"]["causation_unverified"];
        assert!(
            !marker.is_null(),
            "{what}: the failed claim must be journaled, never silently dropped: {child}"
        );
        assert_eq!(
            marker["parent_work_id"], claim["claimed_parent_work_id"],
            "{what}: the marker keeps the claim verbatim, so an operator can \
             tell a stale environment from a forgery: {marker}"
        );
        assert_eq!(
            marker["parent_execution_id"], claim["claimed_parent_execution_id"],
            "{what}: {marker}"
        );
        let reason = marker["reason"].as_str().unwrap_or_default();
        assert!(
            reason.contains(expect_in_reason),
            "{what}: the reason names which check failed, got {reason:?}"
        );
    }

    handle.shutdown().await;
}

/// Run the real `sgt run` binary against the in-process daemon at `data_dir`,
/// from `estate`, carrying `env` — the environment a managed execution would
/// have left it.
async fn sgt_run(
    data_dir: &Path,
    estate: &Path,
    intent: &str,
    env: &[(&str, String)],
) -> std::process::Output {
    let (data_dir, estate, intent) = (
        data_dir.to_path_buf(),
        estate.to_path_buf(),
        intent.to_string(),
    );
    let env: Vec<(String, String)> = env
        .iter()
        .map(|(k, v)| ((*k).to_string(), v.clone()))
        .collect();
    tokio::task::spawn_blocking(move || {
        let mut command = Command::new(env!("CARGO_BIN_EXE_sgt"));
        command
            .current_dir(&estate)
            .arg("--data-dir")
            .arg(&data_dir)
            .args(["--json", "run", &intent, "--backend", "fake"])
            .stdin(std::process::Stdio::null());
        for (key, value) in &env {
            command.env(key, value);
        }
        command.output().expect("run sgt run")
    })
    .await
    .expect("join the spawned CLI")
}

/// Every JSON document on a stdout stream, in order.
///
/// `sgt run --json --wait` prints the submit record exactly as `sgt run
/// --json` always has — pretty, several lines — and then one compact watch
/// notice per line, so the stream is concatenated JSON documents rather than
/// strictly one per line.
fn json_documents(stdout: &str) -> Vec<Value> {
    serde_json::Deserializer::from_str(stdout)
        .into_iter::<Value>()
        .map(|document| document.expect("stdout is a stream of JSON documents"))
        .collect()
}

/// [`sgt_run`] with `--wait`.
async fn sgt_run_wait(data_dir: &Path, estate: &Path, intent: &str) -> std::process::Output {
    let (data_dir, estate, intent) = (
        data_dir.to_path_buf(),
        estate.to_path_buf(),
        intent.to_string(),
    );
    tokio::task::spawn_blocking(move || {
        Command::new(env!("CARGO_BIN_EXE_sgt"))
            .current_dir(&estate)
            .arg("--data-dir")
            .arg(&data_dir)
            .args(["--json", "run", &intent, "--backend", "fake", "--wait"])
            .stdin(std::process::Stdio::null())
            .output()
            .expect("run sgt run --wait")
    })
    .await
    .expect("join the spawned CLI")
}

/// W1 §13.5, the CLI half: `sgt run` reads the `SERGEANT_*` coordinates out
/// of the environment it inherited and sends them as *claims*, and the daemon
/// turns a good claim into a recorded relation.
///
/// The real binary, with the real environment a managed execution would have
/// left it — `origin()`'s own pattern, one field over.
#[tokio::test]
async fn sgt_run_transports_the_inherited_causation_and_the_daemon_validates_it() {
    let repos = TempDir::new().expect("tempdir");
    let data = support::DataDir::new();
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");

    let (handle, fake) = start_fake(data.path(), [FakeStep::needs_input("parked")]).await;
    let client = http();
    let (_, parent) = submit_with(&client, &handle, &estate, "the parent", json!({})).await;
    let parent_id = parent["work"]["id"]
        .as_str()
        .expect("parent id")
        .to_string();
    let execution_id = fake.starts()[0].execution_id.clone();

    // `spawn_blocking`, not a bare `Command::output()`: this daemon lives on
    // the test's own current-thread runtime, so blocking that thread on a
    // child process trying to reach it would deadlock the server it calls.
    let output = sgt_run(
        data.path(),
        &estate,
        "the child",
        &[
            ("SERGEANT_WORK_ID", parent_id.clone()),
            ("SERGEANT_EXECUTION_ID", execution_id.clone()),
            (
                "SERGEANT_ESTATE_ROOT",
                estate.to_string_lossy().into_owned(),
            ),
        ],
    )
    .await;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(
        output.status.code(),
        Some(0),
        "sgt run must succeed\nstdout: {stdout}\nstderr: {stderr}"
    );
    let child: Value = serde_json::from_str(&stdout).expect("json body");
    assert_eq!(
        child["work"]["parent_work_id"], parent_id,
        "the inherited SERGEANT_WORK_ID became a validated relation: {child}"
    );
    assert_eq!(child["work"]["parent_execution_id"], execution_id);

    // A blank exported variable claims nothing rather than claiming "".
    let output = sgt_run(
        data.path(),
        &estate,
        "a blank claim",
        &[
            ("SERGEANT_WORK_ID", String::new()),
            ("SERGEANT_EXECUTION_ID", "   ".to_string()),
        ],
    )
    .await;
    let blank: Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json body");
    assert!(
        blank["work"]["parent_work_id"].is_null()
            && blank["work"]["causation_unverified"].is_null(),
        "an empty variable is absence, not a claim of \"\": {blank}"
    );

    handle.shutdown().await;
}

/// E9 / W1-10: `--wait` is submit-then-observe. The parent actor blocks; the
/// daemon does not. This drives a real `sgt run --wait` to a Work's terminal
/// state and reads the terminal notice off its stdout.
#[tokio::test]
async fn sgt_run_wait_observes_a_child_to_terminal_and_prints_the_notice() {
    let repos = TempDir::new().expect("tempdir");
    let data = support::DataDir::new();
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");

    let (handle, _fake) = start_fake(data.path(), []).await;
    let output = sgt_run_wait(data.path(), &estate, "a Work to wait on").await;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(
        output.status.code(),
        Some(0),
        "sgt run --wait must return once the Work is terminal\nstdout: {stdout}\nstderr: {stderr}"
    );

    // Line 1 is the submit record; every later line is a watch notice, the
    // last of which is terminal (§9.1's JSONL, exactly as `sgt watch --json`
    // emits it).
    let lines = json_documents(&stdout);
    assert!(
        lines.len() >= 2,
        "the submit record plus at least one notice: {stdout}"
    );
    let work_id = lines[0]["work"]["id"]
        .as_str()
        .expect("work id")
        .to_string();
    let last = lines.last().expect("a notice");
    assert_eq!(
        last["schema"], "sergeant.watch/v1",
        "the terminal notice is an ordinary watch notice, not a second shape \
         invented for this flag: {last}"
    );
    assert_eq!(last["snapshot"]["work"]["id"], work_id);
    assert_eq!(
        last["snapshot"]["work"]["state"], "completed",
        "the wait ends on the terminal state, not on the submission: {last}"
    );

    handle.shutdown().await;
}

/// E9's negative half, stated as behaviour rather than as a promise: waiting
/// introduces **no engine hold state**. The set of event kinds a `--wait`
/// submission journals must be exactly the set an identical un-waited
/// submission journals — a hold, a lease, a new parked state or a "waiter
/// attached" record would all show up here as an extra kind.
///
/// Plus the structural half: `--wait` must *call* `crate::watch::watch`
/// rather than re-derive its head-before-stream-before-read sequencing, and
/// the engine must know nothing about the watch module at all.
#[tokio::test]
async fn waiting_adds_no_engine_state_and_re_derives_nothing() {
    let repos = TempDir::new().expect("tempdir");
    let data = support::DataDir::new();
    let estate = repos.path().join("solo-estate");
    support::scaffold_solo_estate(&estate, "solo");

    let (handle, _fake) = start_fake(data.path(), []).await;
    let waited = sgt_run_wait(data.path(), &estate, "waited").await;
    assert_eq!(waited.status.code(), Some(0));
    let plain = sgt_run(data.path(), &estate, "not waited", &[]).await;
    assert_eq!(plain.status.code(), Some(0));
    let id_of = |output: &std::process::Output| {
        json_documents(&String::from_utf8_lossy(&output.stdout))[0]["work"]["id"]
            .as_str()
            .expect("work id")
            .to_string()
    };
    let (waited_id, plain_id) = (id_of(&waited), id_of(&plain));

    // Let the un-waited Work finish before reading the journal: without the
    // wait, nothing in this test has synchronised with its completion.
    let client = http();
    for _ in 0..200 {
        let view = get(&client, &handle, &format!("/v1/work/{plain_id}")).await;
        if view["work"]["state"] == json!("completed") {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    handle.shutdown().await;

    let kinds_of = |work_id: &str| -> Vec<String> {
        let mut kinds: Vec<String> = Journal::replay_data_dir(data.path())
            .expect("replay")
            .map(|e| e.expect("event"))
            .filter(|e| e.work_id.as_deref() == Some(work_id))
            .map(|e| e.kind)
            .collect();
        kinds.sort();
        kinds.dedup();
        kinds
    };
    assert_eq!(
        kinds_of(&waited_id),
        kinds_of(&plain_id),
        "a waited submission must journal exactly what an un-waited one \
         journals — waiting is a client behaviour (W1-10), so any extra \
         event kind here is engine hold state that must not exist"
    );

    let cli = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli.rs"))
        .expect("read cli.rs");
    assert!(
        cli.contains("crate::watch::watch(&client, &options"),
        "--wait must call the existing watch loop (R2), not re-derive it"
    );
    assert!(
        !cli.contains("stream_events"),
        "cli.rs must not reach for the event stream itself: that is exactly \
         the head-before-stream-before-read sequencing watch.rs owns, and \
         duplicating it reopens the race W1-10 closed"
    );
    let engine = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/runtime/engine.rs"
    ))
    .expect("read engine.rs");
    assert!(
        !engine.contains("crate::watch"),
        "the engine must not know the watch module exists: --wait is client \
         behaviour over the API, never a hold the engine takes"
    );
}
