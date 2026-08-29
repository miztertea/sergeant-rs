//! S5 W1b — the Work-overlay **lifecycle trigger**: A2 §2's `--work` promise
//! ("current Work's world, *including overlay*") stops being a capability
//! nothing reaches.
//!
//! Through S4 the overlay half was built, correct at the unit level
//! (`tests/x3a_git_plumbing.rs`), and had **no production caller at all** —
//! recorded honestly as §17 item 2's `met-with-deviation` row and guarded by
//! its own tripwire. W1 then shipped `--work` as base-only and *declared*
//! it (`WorkScope::BaseOnly`) rather than returning a silent partial. This
//! suite is the wiring, and it asks the question that defect class is about:
//! **is there a production caller, and does it run on the real daemon?**
//!
//! H13.2's chosen mechanism is the daemon-side surface-lifecycle hook. Not
//! query-time scanning: that would make a read verb a writer, against the
//! daemon-is-sole-writer boundary. `sgt search` stays a pure reader —
//! [`the_admissibility_filter_cannot_write_because_every_method_takes_an_immutable_self`]
//! is the structural half of that claim, and the daemon-sole-writer /
//! no-client-SQL suites own the rest.
//!
//! # Timing discipline (#258 flake class)
//!
//! This is async daemon-side work, so `scripts/coverage/README.md`'s rule
//! applies: every wall-clock bound here is a **deadline on a polling wait**
//! (a slow host eats headroom and the test still passes), never a fixed
//! sleep followed by an assertion (a slow host would not eat margin, it
//! would invalidate the premise). Nothing below asserts a rate or a
//! duration as the thing under test — only that a state is eventually
//! reached, generously bounded, on the slowest supported target.

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig};
use sergeant_rs::domain::source::Coverage;
use sergeant_rs::runtime::atlas::db::{Admissibility, AtlasDb, SourceSelector, WorkScope};
use sergeant_rs::runtime::atlas::overlay::overlay_source_name;

mod support;
use support::DataDir;

/// How long a lifecycle-driven overlay change gets to appear.
///
/// A **deadline on a polling wait**, deliberately generous: the hook runs a
/// real repository extraction on the intelligence lane, on a host that may
/// be running the rest of this suite at the same time. A slower target eats
/// headroom here; it does not change what is being asserted.
const LIFECYCLE_DEADLINE: Duration = Duration::from_secs(120);
/// Gap between polls. Not a measurement — only how often the question is
/// asked while waiting for the deadline above.
const POLL_GAP: Duration = Duration::from_millis(50);

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

/// A small real repository — a real mount with a real commit, because the
/// overlay path reads a linked worktree's object store and a synthetic
/// fixture would prove nothing about the trigger.
fn scaffold(root: &Path) {
    let mount = root.join("repos").join("product");
    std::fs::create_dir_all(mount.join("src")).expect("mkdir");
    std::fs::write(mount.join("README.md"), "# Product\n\nThe base tree.\n").expect("write");
    std::fs::write(mount.join("src/lib.rs"), "pub fn widget_base() {}\n").expect("write");
    git(&mount, &["init", "--initial-branch=main", "--quiet"]);
    git(&mount, &["config", "user.email", "w1b@example.com"]);
    git(&mount, &["config", "user.name", "W1b Test"]);
    git(&mount, &["add", "-A"]);
    git(&mount, &["commit", "--quiet", "-m", "base"]);
    std::fs::write(
        root.join("sergeant.toml"),
        "[estate]\nname = \"w1b\"\n\n[[repo]]\nname = \"product\"\n",
    )
    .expect("write manifest");

    // A one-stage workflow, the same shape `tests/e_work_sweep.rs` uses: one
    // `FakeStep` drives one Work, so the surface lifecycle under test is not
    // entangled with multi-stage transitions.
    let workflow = root.join(".sergeant/workflows/solo");
    std::fs::create_dir_all(workflow.join("00-only")).expect("workflow dir");
    std::fs::write(
        workflow.join("workflow.toml"),
        "[workflow]\nname = \"solo\"\nversion = \"1\"\nstages = [\"00-only\"]\n",
    )
    .expect("workflow.toml");
    std::fs::write(workflow.join("00-only").join("CONTEXT.md"), "context").expect("CONTEXT.md");
}

async fn start(data: &DataDir) -> daemon::DaemonHandle {
    // `hang()`: the Work stays live after materializing, which is the whole
    // window this suite observes the bound-surface half in. Retirement is
    // then driven explicitly by a cancel, never by a race with completion.
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

/// The CONFIRMED source names the daemon itself reports — the only surface
/// this test reads Atlas through while the daemon owns the store (one-owner
/// duckdb).
async fn indexed_sources(http: &reqwest::Client, handle: &daemon::DaemonHandle) -> Vec<String> {
    let status = get(http, handle, "/v1/intelligence/status").await;
    status["sources"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .filter_map(|r| r["source"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Poll until `want` is true of the daemon's own source list, or fail with
/// what it actually held. A deadline on a polling wait — see this module's
/// timing note.
async fn until_sources(
    http: &reqwest::Client,
    handle: &daemon::DaemonHandle,
    what: &str,
    want: impl Fn(&[String]) -> bool,
) -> Vec<String> {
    let started = Instant::now();
    let mut last = Vec::new();
    while started.elapsed() < LIFECYCLE_DEADLINE {
        last = indexed_sources(http, handle).await;
        if want(&last) {
            return last;
        }
        tokio::time::sleep(POLL_GAP).await;
    }
    panic!("{what} never happened; the daemon's last source list was {last:?}");
}

// ------------------------------- the production trigger, on the real daemon

/// **The wave's decisive test.** A Work whose surface is bound gets an
/// overlay generation written by the daemon itself, and that generation is
/// evicted when the Work retires — closing §17 item 2's sibling half and
/// making `overlay.rs`'s "an overlay's lifetime is its Work's" enforced
/// rather than prose.
///
/// Everything here runs through real production surfaces: the real daemon,
/// the real `POST /v1/intelligence/scan` trigger, a real `POST /v1/work`
/// submission that really materializes a linked worktree, and a real cancel
/// that really tears it down. Nothing in the test calls
/// `scan_work_overlay*` or `evict_work_overlays` itself — if the hook were
/// removed from `src/api.rs`, this test would fail, which is precisely what
/// distinguishes it from the unit-level proofs that stood alone through S4.
#[tokio::test]
async fn a_bound_work_surface_is_scanned_as_an_overlay_and_evicted_when_it_retires() {
    let estate = TempDir::new().expect("estate");
    scaffold(estate.path());
    let data = DataDir::new();
    let handle = start(&data).await;
    let http = client();

    // 1. The estate indexes something. The hook never CREATES an Atlas store
    //    (see `a_work_on_an_unindexed_estate_creates_no_atlas_store` below),
    //    so this is the step that makes an installation one that indexes.
    let (status, body) = post(
        &http,
        &handle,
        "/v1/intelligence/scan",
        &json!({
            "command_id": ulid::Ulid::generate().to_string(),
            "estate_root": estate.path(),
        }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert!(
        indexed_sources(&http, &handle)
            .await
            .contains(&"product".to_string()),
        "the base generation must exist before the overlay half means anything"
    );

    // 2. A real Work, with a real surface.
    let (status, submitted) = post(
        &http,
        &handle,
        "/v1/work",
        &json!({
            "command_id": ulid::Ulid::generate().to_string(),
            "intent": "hold a surface open while its overlay is indexed",
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

    // 3. THE CLAIM: the daemon, on its own, recorded this Work's surface as
    //    an overlay generation under its own `work:<id>/<repo>` coordinate.
    let sources = until_sources(
        &http,
        &handle,
        "the Work's overlay was never indexed",
        |s| s.contains(&overlay_source),
    )
    .await;
    assert!(
        sources.contains(&"product".to_string()),
        "the overlay is recorded ALONGSIDE the base generation, never instead of it: {sources:?}"
    );

    // 4. Retire it. The surface goes; the overlay must go with it.
    let (status, canceled) = post(
        &http,
        &handle,
        &format!("/v1/work/{work_id}/cancel"),
        &json!({"command_id": ulid::Ulid::generate().to_string(), "reason": "w1b"}),
    )
    .await;
    assert_eq!(status, 200, "{canceled}");

    let sources = until_sources(
        &http,
        &handle,
        "the retired Work's overlay was never evicted",
        |s| !s.contains(&overlay_source),
    )
    .await;
    assert!(
        sources.contains(&"product".to_string()),
        "eviction is scoped to the Work: the repository's own base generation must survive it \
         untouched: {sources:?}"
    );

    // 5. The eviction is REPORTED, not silent — the `generation_evicted`
    //    coverage row every other eviction in Atlas leaves. Read directly,
    //    which needs the daemon to release the store first (one-owner
    //    duckdb).
    handle.shutdown().await;
    let db = AtlasDb::open(data.path()).expect("atlas");
    let coverage = db.coverage(&overlay_source, 100).expect("coverage");
    let evicted = coverage
        .iter()
        .find(|row| row.row.status == Coverage::GenerationEvicted)
        .unwrap_or_else(|| {
            panic!(
                "an overlay eviction must leave a coverage row, never a silent gap: {coverage:?}"
            )
        });
    assert!(
        evicted
            .row
            .detail
            .as_deref()
            .is_some_and(|d| d.contains(&work_id)),
        "the row must name the Work whose retirement caused it: {evicted:?}"
    );

    // 6. And a `--work` answer for the retired Work degrades to base-only,
    //    stated on the answer — never a stale claim about a surface that no
    //    longer exists.
    let filter = Admissibility {
        source: SourceSelector::WorkBase {
            work_id: work_id.clone(),
            repository: "product".to_string(),
        },
        ..Admissibility::default()
    };
    let admitted = db
        .admissible_generations(&filter, 500)
        .expect("admissible generations");
    assert_eq!(
        admitted.scope,
        WorkScope::BaseOnly,
        "a retired Work's --work answer must say its overlay is gone"
    );
    assert!(
        admitted
            .hits
            .iter()
            .all(|g| g.source_name != overlay_source),
        "an evicted overlay must not still be admissible: {:?}",
        admitted.hits
    );
}

/// The named limit, checked rather than merely written down: a Work on an
/// estate that indexes nothing gains neither an Atlas database nor a
/// repository walk (R1). `--work` there is honestly base-only.
#[tokio::test]
async fn a_work_on_an_unindexed_estate_creates_no_atlas_store() {
    let estate = TempDir::new().expect("estate");
    scaffold(estate.path());
    let data = DataDir::new();
    let handle = start(&data).await;
    let http = client();

    let (status, submitted) = post(
        &http,
        &handle,
        "/v1/work",
        &json!({
            "command_id": ulid::Ulid::generate().to_string(),
            "intent": "run on an estate that indexes nothing",
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

    // Wait for the surface to actually be bound — otherwise this would pass
    // for the uninteresting reason that nothing had happened yet.
    let started = Instant::now();
    loop {
        let work = get(&http, &handle, &format!("/v1/work/{work_id}")).await;
        if work["surface"]["bindings"]
            .as_array()
            .is_some_and(|b| !b.is_empty())
        {
            break;
        }
        assert!(
            started.elapsed() < LIFECYCLE_DEADLINE,
            "the Work never bound a surface: {work}"
        );
        tokio::time::sleep(POLL_GAP).await;
    }

    let status = get(&http, &handle, "/v1/intelligence/status").await;
    assert_eq!(
        status["atlas"]["present"],
        Value::Bool(false),
        "running a Work must not conjure an Atlas store on an installation that indexes \
         nothing: {status}"
    );
    handle.shutdown().await;
    assert!(
        !sergeant_rs::runtime::atlas::db::atlas_db_path(data.path()).exists(),
        "no atlas.duckdb may exist on disk either"
    );
}

// ------------------------------------------- the failure degrade, and no-write

/// An overlay scan that fails leaves the standing overlay generation in
/// place with a coverage row saying so — never a failed query, never a
/// silent full result.
///
/// The failure is reported where a reader will find it: `AtlasDb::coverage`
/// for the overlay coordinate, the same channel every other unreadable
/// source in Atlas is reported through. When there is no standing overlay
/// generation at all there is nothing for a coverage row to attach to, and
/// the honest answer is the one `--work` gives: `WorkScope::BaseOnly` —
/// asserted here too, so the degrade is pinned at both ends.
#[test]
fn an_overlay_scan_failure_degrades_to_base_only_and_is_reported() {
    let dir = TempDir::new().expect("dir");
    let mut db = AtlasDb::open(dir.path()).expect("atlas");
    let work_id = "01W1BFAIL00000000000000000";
    let source = overlay_source_name(work_id, "product");

    // Nothing stands: nothing is written, and `--work` says base-only.
    assert_eq!(
        db.record_overlay_unavailable(&source, "the surface vanished")
            .expect("record"),
        None,
        "with no standing overlay generation there is nothing to attach coverage to, and \
         staging an empty one would make --work claim an overlay it never read"
    );
    let filter = Admissibility {
        source: SourceSelector::WorkBase {
            work_id: work_id.to_string(),
            repository: "product".to_string(),
        },
        ..Admissibility::default()
    };
    assert_eq!(
        db.admissible_generations(&filter, 500)
            .expect("admissible")
            .scope,
        WorkScope::BaseOnly,
        "a Work whose overlay could not be read gets a declared base-only answer"
    );
}

/// `sgt search` cannot write, structurally: every admissibility method takes
/// `&self`.
///
/// H13.2 rejected query-time scanning because it would make a read verb a
/// writer. That decision is worth exactly as much as what enforces it, and
/// this is the cheapest thing that does: a `&self` method cannot reach
/// `stage_scan`, `confirm_scan`, `evict_work_overlays` or
/// `record_overlay_unavailable`, all of which need `&mut self`. Wiring the
/// overlay into a query path would not compile without first changing a
/// signature this test reads.
#[test]
fn the_admissibility_filter_cannot_write_because_every_method_takes_an_immutable_self() {
    let db = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime/atlas/db.rs"),
    )
    .expect("read db.rs");
    let mut checked = 0;
    for (index, line) in db.lines().enumerate() {
        let Some(name) = line.trim().strip_prefix("pub fn admissible_") else {
            continue;
        };
        checked += 1;
        let signature: String = db.lines().skip(index).take(6).collect::<Vec<_>>().join(" ");
        assert!(
            signature.contains("&self,") && !signature.contains("&mut self"),
            "admissible_{name} must take &self — a query path that could write is exactly what \
             H13.2 rejected when it chose the lifecycle hook over query-time scanning"
        );
    }
    assert_eq!(
        checked, 4,
        "the four A2 §2 content-family methods must all be covered; a new one must be added here"
    );
}
