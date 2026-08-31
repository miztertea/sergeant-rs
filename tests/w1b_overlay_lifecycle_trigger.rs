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
//! [`the_admissibility_filter_cannot_write_and_neither_can_anything_it_calls`]
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

/// **S6 D1 — A2 §2 stage 1's estate coordinate.** This suite is
/// single-estate: every generation it records is bound to this one root and
/// every filter it builds is admitted from it. The cross-estate case — two
/// estates on one host daemon, which is where the axis actually earns its
/// keep — is `tests/d1_estate_isolation.rs`, deliberately not folded in
/// here, because a suite that never crosses estates cannot notice an estate
/// filter that does nothing (that is exactly how the leak survived: this
/// file's ancestors all passed).
#[allow(dead_code)]
const D1_ESTATE: &str = "/estates/w1b_overlay_lifecycle_trigger";

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

    // 1. The estate indexes something. The hook never writes Atlas evidence
    //    on an installation that has none
    //    (see `a_work_on_an_unindexed_estate_gains_no_atlas_evidence` below),
    //    so this is the step that makes an installation one that indexes.
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
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::WorkBase {
            work_id: work_id.clone(),
            repository: "product".to_string(),
        },
        ..Admissibility::within_estate(D1_ESTATE)
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
/// estate that indexes nothing gains neither Atlas evidence nor a repository
/// walk (R1). `--work` there is honestly base-only. See the note at the end
/// of the body for what S5 W1c changed about how that is measured.
#[tokio::test]
async fn a_work_on_an_unindexed_estate_gains_no_atlas_evidence() {
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
        "running a Work must not give an installation that indexes nothing any Atlas \
         evidence: {status}"
    );
    handle.shutdown().await;

    // **What this no longer asserts, and why.** Until S5 W1c it also asserted
    // that no Atlas database file existed on disk, because the hook creating
    // one would have been the R1 violation the limit names. A1 §5 declares
    // ONE database and puts the journal-derived `ops` schema in it, so the
    // daemon's own start creates that file on every host whether or not
    // anything is ever indexed — the file stopped being able to carry the
    // claim. The claim itself is unchanged and is asserted one line up, off
    // the evidence: no source has a confirmed generation, so `--work` here is
    // honestly base-only and the hook wrote nothing. Asserting file absence
    // now would be asserting that `ops` had nowhere to live.
    let path = sergeant_rs::runtime::atlas::db::atlas_db_path(data.path());
    assert!(
        path.exists(),
        "A1 §5's one database carries `ops`, which every daemon start folds: {}",
        path.display()
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
        estate: sergeant_rs::domain::source::EstateAdmission::Estate(D1_ESTATE.to_string()),
        source: SourceSelector::WorkBase {
            work_id: work_id.to_string(),
            repository: "product".to_string(),
        },
        ..Admissibility::within_estate(D1_ESTATE)
    };
    assert_eq!(
        db.admissible_generations(&filter, 500)
            .expect("admissible")
            .scope,
        WorkScope::BaseOnly,
        "a Work whose overlay could not be read gets a declared base-only answer"
    );
}

/// `sgt search` cannot write — **and since the S5 closeout that is enforced
/// by the compiler, not by this scan.**
///
/// # What changed, and why
///
/// H13.2 rejected query-time scanning because it would make a read verb a
/// writer. Through S5 that decision was guarded two ways, and both were
/// weaker than they read:
///
/// 1. `&self` vs `&mut self` — no proof of anything, because DuckDB's own
///    `Connection::execute`, `execute_batch` and `prepare` all take `&self`.
/// 2. A transitive scan of `admissible_*` bodies for write-capable
///    spellings — which the closeout re-verify **defeated in one hop**. Its
///    `WRITE_CAPABLE` list mixed lowercase call syntax (`.execute_batch(`)
///    with uppercase SQL verbs (`DELETE `) and tested
///    `body.to_uppercase().contains(verb)`, so the three call-syntax entries
///    could never match anything: uppercasing the body turns `.execute_batch(`
///    into `.EXECUTE_BATCH(`. Inserting
///    `let v = format!("{}ETE FROM source.units", "DEL"); self.conn.execute_batch(&v);`
///    into `admissible_datasets` left this test **green**.
///
/// So the mechanism moved. `db.rs` now implements the whole A2 §2 filter on
/// `Admissible`, a struct whose only field is a `ReadOnly` — a handle with no
/// write call on it, which hands out no `Statement`, `Appender`,
/// `Transaction` or `Connection`, and whose two query methods take a
/// `ReadSql` that a `const` item proves begins `SELECT `. **A write inside
/// the admissibility filter is a compile error**, verified by inserting each
/// of these and watching the build fail:
///
/// | inserted into `Admissible::datasets` | result |
/// |---|---|
/// | `self.conn.execute_batch(&v)` | `no field 'conn' on type &Admissible` |
/// | `self.reader.execute_batch(&v)` | `no method named 'execute_batch'` |
/// | `self.reader.rows(&v, …)` | `Sql: From<&String> is not satisfied` |
/// | `self.reader.rows(read_sql!("DELETE FROM source.units"), …)` | `evaluation panicked: … beginning 'SELECT ' and containing no ';'` |
///
/// # The prefix check alone was not enough, and that was proven too
///
/// A third round of this closeout landed a write **through** the
/// `SELECT `-prefix check:
///
/// ```ignore
/// self.reader.rows(read_sql!("SELECT 1; DELETE FROM source.generations"), …)
/// ```
///
/// The prefix sees only the leading `SELECT `, and DuckDB executes **every**
/// statement in a `;`-separated batch — measured directly against a raw
/// connection on duckdb 1.10505.0: `prepare` alone runs nothing, `prepare` +
/// `query` runs the whole batch and returns the *last* statement's result,
/// and a three-row table came back empty. (A batch containing a `?` bind is
/// refused at `prepare` instead, so the parameterised spelling of this
/// exploit errors rather than deleting — the unparameterised one is the real
/// one.) `ReadSql` now refuses any `;` at all, checked in the same `const`
/// evaluation as the prefix.
///
/// That the check is a `const` and not a scan is the point: const evaluation
/// sees the **assembled** string, after `concat!` has resolved, so
/// `read_sql!(concat!("DEL", "ETE FROM t"))` is `DELETE FROM t` by the time
/// the check runs. A text scanner reads the source spelling and never sees
/// it. Attempted, and it fails the build.
///
/// # What THIS test is, now
///
/// The compiler owns the claim. This test owns two things the compiler
/// cannot state for a future reader: that the wiring is still in place (part
/// 1), and a cheap second net over the shapes a type does not see (part 2).
/// Part 1 is the one that must never be relaxed — deleting it would let
/// someone quietly reintroduce a `Connection` field on `Admissible` and lose
/// the guarantee with no test going red.
///
/// # What the second net cannot see — stated, not implied
///
/// The scan reads source text, so it is blind to exactly what source text is
/// blind to. It does **not** see: a write assembled from pieces no single
/// line spells (`concat!`, a `const` defined elsewhere in the file and named
/// here, a byte array); a call reached through a trait object or a function
/// pointer; anything in another file. Those are the hops that defeated its
/// predecessor, and the reason it is no longer the load-bearing check. It is
/// kept because it is nearly free and because it *does* catch the one shape
/// the type system still permits: a literal SQL write verb sitting in a
/// method the filter can reach.
#[test]
fn the_admissibility_filter_cannot_write_and_neither_can_anything_it_calls() {
    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime/atlas/db.rs"),
    )
    .expect("read db.rs");
    let lines: Vec<&str> = source.lines().collect();

    // ---------------------------------------------------------------- part 1
    // The compile-time guarantee is still wired. Each assertion below is a
    // load-bearing piece of it; none is cosmetic.

    // (a) The filter's own type holds a read-only handle and NOTHING else. A
    //     second field of any connection-ish type would restore the hop the
    //     closeout closed, and would not change one character of the scan
    //     below.
    assert!(
        source.contains("struct Admissible<'conn> {\n    reader: ReadOnly<'conn>,\n}"),
        "`Admissible` must hold exactly one field, a `ReadOnly` — that single field is the \
         whole reason a write inside the admissibility filter does not compile"
    );

    // (b) The public methods are delegates onto it, so the public path cannot
    //     do anything the filter's own type forbids.
    for family in ["generations", "units", "occurrences", "datasets"] {
        let delegate = format!("        self.admissible().{family}(filter, limit)\n    }}");
        assert!(
            source.contains(&delegate),
            "`AtlasDb::admissible_{family}` must be a one-line delegate onto `Admissible`; \
             any logic that stays on `AtlasDb` runs with `self.conn` in scope and is \
             therefore outside the guarantee"
        );
    }

    // (c) The read-only handle really is write-free: it exposes exactly two
    //     operations, and neither hands back a value with a write on it. This
    //     is spelled as an allowlist of `pub fn`s in the `ReadOnly` impl
    //     rather than a denylist of forbidden method names, because a
    //     denylist is what the old scan was.
    let read_only_impl = block_after(&source, "    impl ReadOnly<'_> {");
    let exposed: Vec<&str> = read_only_impl
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub fn "))
        .filter_map(|rest| rest.split(['(', '<', ' ']).next())
        .collect();
    assert_eq!(
        exposed,
        vec!["rows", "first"],
        "`ReadOnly` must expose only its two mapping queries; anything else — a `prepare`, a \
         `connection`, a `Statement` accessor — is a write one dot away"
    );
    for handed_out in [
        "-> Statement",
        "-> CachedStatement",
        "-> Appender",
        "-> Connection",
    ] {
        assert!(
            !read_only_impl.contains(handed_out),
            "`ReadOnly` must hand out no {handed_out} — a value with a write on it defeats \
             the whole type"
        );
    }
    assert_eq!(
        read_only_impl.matches("sql: ReadSql,").count(),
        2,
        "both `ReadOnly` queries must take a `ReadSql`, never a `Sql`: a `Sql` may be any \
         statement this crate wrote, DELETE included, and DuckDB runs what it is handed"
    );

    // (d) And `ReadSql` really is checked, at compile time. `read_sql!` builds
    //     an anonymous `const` **item** rather than a `const { .. }` block on
    //     purpose: a const block — and the generic associated const inside
    //     `ReadSql::of`, which covers the non-macro path — is a
    //     post-monomorphization error that `cargo check` walks straight past.
    //     That was observed during the closeout, twice: once before this
    //     shape was chosen, and once again when a hand-written `impl SqlText`
    //     carrying a `DELETE` passed `cargo check` and only failed the moment
    //     a test actually called it.
    assert!(
        source.contains("        const _: () = assert!(\n")
            && source.contains("$crate::runtime::atlas::db::store::is_read_statement("),
        "`read_sql!` must evaluate `is_read_statement` as a non-generic `const` item, so a \
         statement that is not one bare `SELECT` fails `cargo check` rather than reaching \
         DuckDB"
    );

    // (e) And the rule that `const` enforces is BOTH conditions. The `;` half
    //     is not decoration: the prefix half alone was defeated during this
    //     closeout by `"SELECT 1; DELETE FROM source.generations"`, which
    //     DuckDB runs in full. `is_read_statement`'s own behaviour is pinned
    //     by a table in `db.rs`'s unit tests
    //     (`is_read_statement_admits_exactly_one_bare_select`); what is
    //     checked here is that the function the macro calls still tests both.
    let read_rule = block_after(
        &source,
        "    pub const fn is_read_statement(sql: &str) -> bool {",
    );
    assert!(
        read_rule.contains("let want = b\"SELECT \";"),
        "`is_read_statement` must still check the seven-byte `SELECT ` prefix"
    );
    assert!(
        read_rule.contains("if bytes[i] == b';' {"),
        "`is_read_statement` must still refuse a `;` anywhere — DuckDB executes every \
         statement in a `;`-separated batch, which is how the prefix-only check was \
         defeated"
    );

    // ---------------------------------------------------------------- part 2
    // The second net. Cheap, honest, and explicitly NOT the guarantee.

    // Every function defined in db.rs, by name, with the body text a caller
    // of it would actually run. Bodies end at the closing brace on the
    // function's own indentation — brace counting would trip over the `{}`
    // in a `format!`, and this file has many.
    let mut bodies: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];
        let Some(rest) = trimmed
            .strip_prefix("pub fn ")
            .or_else(|| trimmed.strip_prefix("fn "))
            .or_else(|| trimmed.strip_prefix("pub(crate) fn "))
        else {
            continue;
        };
        let Some(name) = rest.split(['(', '<', ' ']).next() else {
            continue;
        };
        let closing = format!("{indent}}}");
        let mut body = String::new();
        for next in lines.iter().skip(index + 1) {
            if *next == closing {
                break;
            }
            // Prose is not code: a comment naming a `DELETE` this function
            // does not perform must not fail the scan.
            if !next.trim_start().starts_with("//") {
                body.push_str(next);
                body.push('\n');
            }
        }
        bodies.insert(name.to_string(), body);
    }

    /// A call into the driver that can write. Matched **case-sensitively
    /// against the body as written**, which is the bug the closeout found:
    /// these were previously compared against `body.to_uppercase()`, where
    /// `.execute_batch(` had become `.EXECUTE_BATCH(` and could never match.
    const WRITE_CALLS: [&str; 6] = [
        ".execute(",
        ".execute_batch(",
        ".appender(",
        ".appender_to_db(",
        // Reaching for a connection of its own is how a read path would get
        // around the read-only handle without ever naming a write.
        "Connection::",
        "Store::new(",
    ];

    /// An SQL verb that writes. Matched against the **uppercased** body, so
    /// `delete from` in a runtime-built string is caught too.
    const WRITE_VERBS: [&str; 9] = [
        "INSERT ",
        "UPDATE ",
        "DELETE ",
        "DROP ",
        "CREATE ",
        "ALTER ",
        "ATTACH ",
        "COPY ",
        "TRUNCATE ",
    ];

    /// Does `body` call `name` — as a free function, or on `self`?
    ///
    /// The receiver matters: `statement.query(...)` is duckdb's `query`, not
    /// this file's `Analytics::query`, and following that name would walk
    /// the closure into every unrelated method that happens to share a
    /// spelling. `self.` and no-receiver calls are the ones that really are
    /// this file's own code.
    fn calls(body: &str, name: &str) -> bool {
        let needle = format!("{name}(");
        if body.contains(&format!("self.{needle}")) {
            return true;
        }
        let bytes = body.as_bytes();
        let mut from = 0;
        while let Some(at) = body[from..].find(&needle) {
            let at = from + at;
            let before = at.checked_sub(1).map(|i| bytes[i] as char);
            match before {
                // A receiver, or a longer identifier ending in this name.
                Some(c) if c == '.' || c == '_' || c.is_alphanumeric() => {}
                _ => return true,
            }
            from = at + needle.len();
        }
        false
    }

    // Seeded from the *implementation*, not from the delegates. Seeding on
    // `pub fn admissible_` would now walk four one-line bodies and prove
    // nothing — a vacuous net is worse than none, because it reads like one
    // that works.
    let admissible_impl = block_after(&source, "impl Admissible<'_> {");
    let mut seeds: Vec<String> = admissible_impl
        .lines()
        .filter_map(|line| line.trim().strip_prefix("fn "))
        .filter_map(|rest| rest.split(['(', '<', ' ']).next())
        .map(str::to_string)
        .collect();
    seeds.sort();
    assert_eq!(
        seeds,
        vec![
            "datasets",
            "generations",
            "newest_overlay_observed_at",
            "occurrences",
            "units",
            "work_scope",
        ],
        "the four A2 §2 content-family methods and their two helpers must all be covered; a \
         new one must be added here"
    );

    for seed in &seeds {
        // Everything this method can reach inside db.rs, transitively.
        let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        let mut queue = vec![seed.clone()];
        while let Some(current) = queue.pop() {
            if !seen.insert(current.clone()) {
                continue;
            }
            let Some(body) = bodies.get(&current) else {
                continue;
            };
            for call in WRITE_CALLS {
                assert!(
                    !body.contains(call),
                    "{seed} reaches {current}, which contains {call:?} — the admissibility \
                     filter is the read side of H13.2's decision"
                );
            }
            for verb in WRITE_VERBS {
                assert!(
                    !body.to_uppercase().contains(verb),
                    "{seed} reaches {current}, which contains the SQL verb {verb:?}"
                );
            }
            for callee in bodies.keys() {
                if calls(body, callee) {
                    queue.push(callee.clone());
                }
            }
        }
    }
}

/// The text of the `{ .. }` block opened by the line `header`, up to the
/// closing brace at that line's own indentation.
fn block_after(source: &str, header: &str) -> String {
    let at = source
        .find(header)
        .unwrap_or_else(|| panic!("db.rs must still contain `{header}`"));
    let indent = header.len() - header.trim_start().len();
    let closing = format!("\n{}}}", " ".repeat(indent));
    let rest = &source[at + header.len()..];
    let end = rest
        .find(&closing)
        .unwrap_or_else(|| panic!("`{header}` must close at its own indentation"));
    rest[..end].to_string()
}
