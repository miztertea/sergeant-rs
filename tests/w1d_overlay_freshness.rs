//! S5 W1d — **`--work` reflects what the Work has actually changed.**
//!
//! W1b wired the Work-overlay scan to the surface lifecycle alone
//! (materialize / rematerialize / teardown) and shipped an honest
//! `WorkScope::BaseAndOverlaySnapshot` marker beside it. That was correct
//! and it was not enough: a linked worktree is cut **byte-identical to its
//! base**, so the only overlay a bind can ever record is one that describes
//! nothing the Work did. A2 §2 names `--work` as the *"current Work's
//! world, **including overlay**"*; an overlay recorded before the Work has
//! touched anything does not satisfy that sentence. W1b's own brief said
//! rescanning was out of scope — a narrowing of the parent contract that no
//! brief may make (AGENTS.md, authority inheritance; **J5**).
//!
//! So W1d added one moment: **a turn boundary** — a settled observation
//! whose backend signal is anything but `Running`, i.e. the actor has
//! stopped producing and its surface has stopped moving
//! (`api::is_turn_boundary`). This suite is the test that moment exists.
//!
//! # What this asks that W1b's suite does not
//!
//! `tests/w1b_overlay_lifecycle_trigger.rs` asks "is there a production
//! caller, and does it run on the real daemon?". This one asks the question
//! the empty overlay made unanswerable: **is a file the Work modified
//! findable through `--work` while the Work is still running?** Every step
//! below goes through a real daemon, a real materialized worktree, a real
//! `POST /v1/work/{id}/input` turn, and the real admissibility filter. The
//! edit is made *after* the surface is bound and its bind scan has landed,
//! so nothing here could pass on W1b's lifecycle-only trigger.
//!
//! # Timing discipline (#258 flake class)
//!
//! Same rule as W1b's suite, and for the same reason: every wall-clock
//! bound below is a **deadline on a polling wait**, never a fixed sleep
//! followed by an assertion. Nothing here asserts a rate or a duration as
//! the thing under test — the overlay scan's measured cost lives in
//! `tests/w1d_overlay_scan_measurement.rs`, which is `#[ignore]`d and never
//! part of a pass/fail claim.

// S6 D1: this suite's one filter is admitted from its own real estate root
// (see the `Admissibility::within_estate` call below), because the daemon
// recorded these generations under that exact root. The cross-estate case is
// `tests/d1_estate_isolation.rs`.

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig};
use sergeant_rs::runtime::atlas::db::{Admissibility, AtlasDb, SourceSelector, WorkScope};
use sergeant_rs::runtime::atlas::overlay::overlay_source_name;

mod support;
use support::DataDir;

/// How long a daemon-driven overlay change gets to appear. A **deadline on
/// a polling wait**, deliberately generous — the hook runs a real
/// repository extraction on the intelligence lane on a host that may be
/// running the rest of this suite at the same time. A slower target eats
/// headroom here; it does not change what is asserted.
const FRESHNESS_DEADLINE: Duration = Duration::from_secs(120);
/// Gap between polls. Not a measurement — only how often the question is
/// asked while waiting for the deadline above.
const POLL_GAP: Duration = Duration::from_millis(50);

/// The text the Work writes into its surface. Distinctive enough that
/// finding it anywhere in an admissible unit is proof of provenance, not a
/// coincidence.
const WORK_EDIT: &str = "the widget retry executor backs off exponentially";

fn git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("spawn git {args:?}: {e}"));
    assert!(
        out.status.success(),
        "git {args:?} in {}: {}",
        dir.display(),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A real mount with a real commit — the overlay path reads a linked
/// worktree's object store, so a synthetic fixture would prove nothing.
fn scaffold(root: &Path) {
    let mount = root.join("repos").join("product");
    std::fs::create_dir_all(&mount).expect("mkdir");
    std::fs::write(mount.join("README.md"), "# Product\n\nThe base tree.\n").expect("write");
    git(&mount, &["init", "--initial-branch=main", "--quiet"]);
    git(&mount, &["config", "user.email", "w1d@example.com"]);
    git(&mount, &["config", "user.name", "W1d Test"]);
    git(&mount, &["add", "-A"]);
    git(&mount, &["commit", "--quiet", "-m", "base"]);
    std::fs::write(
        root.join("sergeant.toml"),
        "[estate]\nname = \"w1d\"\n\n[[repo]]\nname = \"product\"\n",
    )
    .expect("write manifest");

    let workflow = root.join(".sergeant/workflows/solo");
    std::fs::create_dir_all(workflow.join("00-only")).expect("workflow dir");
    std::fs::write(
        workflow.join("workflow.toml"),
        "[workflow]\nname = \"solo\"\nversion = \"1\"\nstages = [\"00-only\"]\n",
    )
    .expect("workflow.toml");
    std::fs::write(workflow.join("00-only").join("CONTEXT.md"), "context").expect("CONTEXT.md");
}

/// Two turns that each park on a question, so the Work is **still running**
/// across the whole test: turn 1 binds the surface, and turn 2 is what the
/// human's answer starts and what ends at the boundary under test.
async fn start(data: &DataDir) -> daemon::DaemonHandle {
    let fake = FakeBackend::scripted(
        FAKE_BACKEND_NAME,
        [
            FakeStep::needs_input("first question"),
            FakeStep::needs_input("second question"),
        ],
    );
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
        .timeout(Duration::from_secs(60))
        .build()
        .expect("client")
}

async fn post(
    http: &reqwest::Client,
    handle: &daemon::DaemonHandle,
    path: &str,
    body: &Value,
) -> (reqwest::StatusCode, Value) {
    let response = http
        .post(format!("{}{path}", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(body)
        .send()
        .await
        .expect("request");
    let status = response.status();
    (status, response.json().await.expect("json"))
}

async fn get(http: &reqwest::Client, handle: &daemon::DaemonHandle, path: &str) -> Value {
    http.get(format!("{}{path}", handle.endpoint))
        .bearer_auth(&handle.token)
        .send()
        .await
        .expect("request")
        .json()
        .await
        .expect("json")
}

/// The daemon's own report of one source's confirmed generation — the only
/// surface this test reads Atlas through while the daemon owns the store
/// (one-owner duckdb). `(content_key, observed_at)`.
async fn generation_of(
    http: &reqwest::Client,
    handle: &daemon::DaemonHandle,
    source: &str,
) -> Option<(String, String)> {
    let status = get(http, handle, "/v1/intelligence/status").await;
    status["sources"].as_array()?.iter().find_map(|row| {
        (row["source"].as_str()? == source).then(|| {
            (
                row["content_key"].as_str().unwrap_or_default().to_string(),
                row["observed_at"].as_str().unwrap_or_default().to_string(),
            )
        })
    })
}

/// Poll until the daemon reports a generation for `source` satisfying
/// `want`, or fail with what it actually reported.
async fn until_generation(
    http: &reqwest::Client,
    handle: &daemon::DaemonHandle,
    what: &str,
    source: &str,
    want: impl Fn(&(String, String)) -> bool,
) -> (String, String) {
    let started = Instant::now();
    let mut last = None;
    while started.elapsed() < FRESHNESS_DEADLINE {
        last = generation_of(http, handle, source).await;
        if let Some(seen) = last.as_ref()
            && want(seen)
        {
            return seen.clone();
        }
        tokio::time::sleep(POLL_GAP).await;
    }
    panic!("{what}; the daemon's last generation for {source} was {last:?}");
}

/// Poll until `/v1/work/{id}` satisfies `want`.
async fn until_work(
    http: &reqwest::Client,
    handle: &daemon::DaemonHandle,
    work_id: &str,
    what: &str,
    want: impl Fn(&Value) -> bool,
) -> Value {
    let started = Instant::now();
    let mut last = Value::Null;
    while started.elapsed() < FRESHNESS_DEADLINE {
        last = get(http, handle, &format!("/v1/work/{work_id}")).await;
        if want(&last) {
            return last;
        }
        tokio::time::sleep(POLL_GAP).await;
    }
    panic!("{what}; the work last read as {last}");
}

// ------------------------------------------------- the one test of this wave

/// **The wave's decisive test.** A Work that has MODIFIED a file in its
/// surface is findable through `--work` *while it is still running*.
///
/// The sequence is the argument:
///
/// 1. the estate indexes `product`, so a base generation exists;
/// 2. a Work materializes a surface and parks on its first question — the
///    W1b bind hook records an overlay generation here, and it describes
///    nothing, because the worktree is still byte-identical to its base;
/// 3. the test writes [`WORK_EDIT`] into `README.md` **inside the bound
///    worktree**, standing in for the actor's own edit;
/// 4. the human answers, starting a second turn, which ends at the second
///    question — **a turn boundary**;
/// 5. the daemon's own status shows the overlay generation's `content_key`
///    has MOVED off the one recorded at bind. That is the whole claim: a
///    rescan happened, daemon-side, with the Work still active. On W1b's
///    lifecycle-only trigger this step is what would hang until the
///    deadline;
/// 6. the Work is still non-terminal, asserted rather than assumed;
/// 7. and the edited text is admissible through `--work` — under the
///    overlay's own source coordinate, with the answer still carrying
///    [`WorkScope::BaseAndOverlaySnapshot`] rather than a claim of
///    currency.
#[tokio::test]
async fn a_running_works_modified_file_is_findable_through_work_scope() {
    let estate = TempDir::new().expect("estate");
    scaffold(estate.path());
    let data = DataDir::new();
    let handle = start(&data).await;
    let http = client();

    // 1. The estate indexes something.
    let (status, body) = support::scan_to_completion(
        &http,
        &handle.endpoint,
        &handle.token,
        &json!({
            "command_id": ulid::Ulid::generate().to_string(),
            "estate_root": estate.path(),
        }),
    )
    .await;
    assert_eq!(status, 200, "{body}");

    // 2. A real Work, with a real surface, parked on its first question.
    let (status, submitted) = post(
        &http,
        &handle,
        "/v1/work",
        &json!({
            "command_id": ulid::Ulid::generate().to_string(),
            "intent": "modify a file and stay running while --work is asked about it",
            "workflow": "solo",
            "estate_root": estate.path(),
            "origin": {"client": "cli", "cwd": estate.path()},
        }),
    )
    .await;
    assert_eq!(status, 201, "{submitted}");
    let work_id = submitted["work"]["id"]
        .as_str()
        .expect("work id")
        .to_string();
    let overlay_source = overlay_source_name(&work_id, "product");

    let work = until_work(
        &http,
        &handle,
        &work_id,
        "the Work never parked on its first question",
        |w| w["work"]["state"] == "needs_input",
    )
    .await;
    let worktree = work["surface"]["bindings"][0]["worktree_path"]
        .as_str()
        .unwrap_or_else(|| panic!("the Work must have a bound worktree: {work}"))
        .to_string();

    let (at_bind, _) = until_generation(
        &http,
        &handle,
        "the bind-time overlay was never recorded",
        &overlay_source,
        |_| true,
    )
    .await;

    // 3. The Work changes something. Written into the bound worktree, and
    //    deliberately AFTER the bind scan above has landed — this is the
    //    edit no lifecycle-only trigger can ever see.
    std::fs::write(
        Path::new(&worktree).join("README.md"),
        format!("# Product\n\nThe base tree.\n\n{WORK_EDIT}\n"),
    )
    .expect("write into the Work's own surface");

    // 4. The human answers: a second turn starts, and ends at the second
    //    question.
    let (status, answered) = post(
        &http,
        &handle,
        &format!("/v1/work/{work_id}/input"),
        &json!({
            "command_id": ulid::Ulid::generate().to_string(),
            "input": "carry on",
        }),
    )
    .await;
    assert_eq!(status, 200, "{answered}");

    // 5. THE CLAIM: the overlay generation moved, daemon-side, without the
    //    surface ever being rebound or torn down.
    let (after_turn, _) = until_generation(
        &http,
        &handle,
        "the overlay was never rescanned after the Work's turn ended — `--work` would \
         still be answering with the empty overlay recorded at bind",
        &overlay_source,
        |(content_key, _)| content_key != &at_bind,
    )
    .await;
    assert_ne!(
        after_turn, at_bind,
        "an overlay generation key is the base SHA composed with the digest of what the \
         surface changed; it must move when the surface changes"
    );

    // 6. Still running — the freshness is not an artifact of the Work
    //    having finished and been torn down.
    let work = get(&http, &handle, &format!("/v1/work/{work_id}")).await;
    let state = work["work"]["state"].as_str().unwrap_or_default();
    assert!(
        matches!(state, "active" | "needs_input" | "waiting" | "blocked"),
        "the whole claim is that this is visible WHILE the Work runs; it was {state}: {work}"
    );

    // 7. And the edit is admissible through `--work`. Read directly, which
    //    needs the daemon to release the store first (one-owner duckdb).
    //    The Work is never retired: its surface and its overlay both still
    //    stand at this point, which is exactly the state under test.
    handle.shutdown().await;
    let db = AtlasDb::open(data.path()).expect("atlas");
    // S6 D1: the estate coordinate is the one the daemon recorded these
    // generations under — this test's own real estate root, resolved by the
    // same `admit_addressed_estate` gate the request went through. A
    // stand-in constant would admit nothing, which is the point of the axis.
    let filter = Admissibility {
        source: SourceSelector::WorkBase {
            work_id: work_id.clone(),
            repository: "product".to_string(),
        },
        ..Admissibility::within_estate(estate.path().to_string_lossy().into_owned())
    };
    let admitted = db.admissible_units(&filter, 500).expect("admissible units");
    assert!(
        matches!(admitted.scope, WorkScope::BaseAndOverlaySnapshot { .. }),
        "the answer must still say what instant it reflects, never claim currency: {:?}",
        admitted.scope
    );
    let hit = admitted
        .hits
        .iter()
        .find(|u| u.unit.body.contains(WORK_EDIT))
        .unwrap_or_else(|| {
            panic!(
                "the Work's own edit must be findable through --work while it runs; \
                 admissible units were {:?}",
                admitted
                    .hits
                    .iter()
                    .map(|u| (&u.source_name, &u.unit.relative_path))
                    .collect::<Vec<_>>()
            )
        });
    assert_eq!(
        hit.source_name, overlay_source,
        "the edit is carried by the Work's OWN overlay generation, never by the \
         repository's shared base generation"
    );
    assert_eq!(hit.unit.relative_path, "README.md");
}
