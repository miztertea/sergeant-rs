//! S6 — the semantic half **at the crossing**: through the daemon, over a
//! real corpus, with the committed model installed.
//!
//! # The axis no other suite covers
//!
//! Every semantic test before this one ran either with no model installed,
//! or in-process against `AtlasDb::semantic_search` on a fixture small
//! enough to finish inside any budget. `w3b_semantic_retrieval` has no HTTP
//! boundary at all; `w3_semantic_degradation` removes the model directory on
//! purpose. So the crossing — *semantic x real-repository text x the daemon
//! boundary* — was covered on each axis alone and never at the crossing, and
//! that is where the defect lived: `semantic_search` re-embedded the whole
//! admissible corpus on every query, which on the real estate (19,025 units)
//! spent the CLI's entire 10 s `REQUEST_TIMEOUT` (`src/cli.rs:47`) and
//! returned `error sending request for url (.../v1/search?...)`, exit 1,
//! while the daemon carried on embedding.
//!
//! # What this suite holds
//!
//! * [`the_operator_can_ask_whether_the_semantic_model_is_loaded`] and
//!   [`a_model_directory_that_will_not_load_is_not_reported_as_no_assets`] —
//!   A2 §15's *"reports that coverage/capability honestly"*, on the surface
//!   an operator actually has. Before S6 `GET /v1/intelligence/status`
//!   returned exactly `atlas` and `sources`: the only thing that said
//!   anything about the model was a search answer's disclosure line, so an
//!   operator had to run a query to learn whether a query would work, and a
//!   directory that failed to load was indistinguishable from no assets —
//!   precisely the distinction `SemanticEngine::load`'s own doc says is
//!   there on purpose.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig};
use sergeant_rs::runtime::atlas::semantic::{MODEL_DIR_ENV, MODEL_FILES};

mod support;
use support::DataDir;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

/// The committed assets, through the documented operator override. Safe
/// because cargo-nextest runs every test in its own process — the same
/// reason `tests/w3b_semantic_retrieval.rs` gives.
fn install_model() {
    support::install_model(MODEL_DIR_ENV);
}

fn uninstall_model() {
    support::uninstall_model(MODEL_DIR_ENV);
}

/// A directory that **is** a complete asset directory by
/// `semantic::model_dir`'s test — all three of [`MODEL_FILES`] present — and
/// whose weights are garbage, so the loader fails.
///
/// The case A2-13 does not cover: not "this host has no model" but "this
/// host has assets that will not load", which
/// `SemanticEngine::load`'s doc calls *"a fault an operator has to hear
/// about"*.
fn install_broken_model() -> tempfile::TempDir {
    let broken = tempfile::tempdir().expect("broken model dir");
    for name in MODEL_FILES {
        std::fs::write(broken.path().join(name), b"not a model").expect("write broken asset");
    }
    unsafe { std::env::set_var(MODEL_DIR_ENV, broken.path()) };
    broken
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
        .timeout(Duration::from_secs(60))
        .build()
        .expect("client")
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

/// **A2 §15's capability, reported on the surface an operator has.**
///
/// `sgt intelligence status --json` is where an operator asks what this
/// host's intelligence can do. Before S6 it could not answer the one
/// question that decides whether search will rank semantically at all.
#[tokio::test]
async fn the_operator_can_ask_whether_the_semantic_model_is_loaded() {
    install_model();
    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let status = get(&http, &handle, "/v1/intelligence/status").await;
    assert_eq!(
        status["semantic"]["state"], "installed",
        "the committed assets are installed; status said: {status}"
    );
    assert!(
        status["semantic"]["identity"]
            .as_str()
            .is_some_and(|id| id.contains('@')),
        "an installed model names the assets it is: {status}"
    );
    assert!(
        status["semantic"]["content_hash"]
            .as_str()
            .is_some_and(|hash| hash.starts_with("blake3:")),
        "A2 §15 pins assets by content as well as version: {status}"
    );

    handle.shutdown().await;
}

/// **A directory that exists and will not load is a fault, not an absence.**
///
/// `AtlasDb::semantic_engine` reported the load error with `log::warn!` and
/// then yielded `None` — and the daemon installs a `tracing` subscriber with
/// no `log`/`tracing` bridge, so the message reached a facade with no logger
/// and was dropped. Broken assets and no assets became the same answer
/// everywhere an operator could look.
#[tokio::test]
async fn a_model_directory_that_will_not_load_is_not_reported_as_no_assets() {
    let _broken = install_broken_model();
    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let status = get(&http, &handle, "/v1/intelligence/status").await;
    assert_eq!(
        status["semantic"]["state"], "failed",
        "broken assets must not read as an uninstalled host: {status}"
    );
    assert!(
        status["semantic"]["detail"]
            .as_str()
            .is_some_and(|d| !d.is_empty()),
        "a fault an operator has to hear about must say what it was: {status}"
    );

    handle.shutdown().await;
}

/// The third state, so the two above cannot pass by the surface reporting a
/// constant: a host with no assets says so, and says nothing about a fault.
#[tokio::test]
async fn a_host_with_no_assets_reports_not_installed_rather_than_a_fault() {
    uninstall_model();
    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let status = get(&http, &handle, "/v1/intelligence/status").await;
    assert_eq!(status["semantic"]["state"], "not_installed", "{status}");
    assert!(status["semantic"]["detail"].is_null(), "{status}");

    handle.shutdown().await;
}

// ------------------------------------------------------------ the crossing

/// The estate the crossing test scans: one `[[knowledge]]` source that is a
/// copy of this build's own `src/` tree.
///
/// **Real repository text, and that is the whole point.** The only figure
/// this program ever had for the semantic scan
/// (`knowledge/evidence/perf/model2vec-footprint-and-scan-2026-08-30.md`)
/// was taken over *synthetic one-sentence* units and named its own gap:
/// *"real repository text rather than synthetic one-sentence units (real
/// units are longer, and cost scales with tokens as well as with units)"*.
/// That named gap is where the defect lived — a fixture corpus of
/// nine one-line documents re-embeds inside any budget, and this one does
/// not. Same instrument `tests/s6_scan_front_door.rs` uses, for the same
/// reason (R2).
const BULK_SOURCES: [&str; 5] = ["docs", ".sergeant", "skills", "src", "tests"];

fn scaffold_estate(root: &Path) {
    std::fs::create_dir_all(root).expect("estate root");
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut manifest = String::from("[estate]\nname = \"s6-crossing\"\n");
    for name in BULK_SOURCES {
        copy_dir_recursive(&manifest_dir.join(name), &root.join(name));
        manifest.push_str(&format!(
            "\n[[knowledge]]\nname = \"{name}\"\npath = \"{name}\"\n"
        ));
    }
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

/// **The crossing: a real `sgt search` through the daemon, with the model
/// installed, over real repository text.**
///
/// The budget is `sgt`'s own — `src/cli.rs`'s
/// `const REQUEST_TIMEOUT: Duration = Duration::from_secs(10)`, applied to
/// the client this very binary builds. Nothing here invents a duration,
/// polls a clock or asserts a rate: the assertion is the **exit code and
/// the answer** an operator gets, and the ten seconds are the product's,
/// not the test's. That is exactly how the defect presented —
///
/// ```text
/// $ sgt --data-dir $D -C $E search "bounded judgment ladder"
/// sgt: error sending request for url (http://127.0.0.1:PORT/v1/search?q=...)
/// elapsed=10s  exit=1        # daemon still ALIVE
/// ```
///
/// — because `semantic_search` re-embedded every admissible unit on every
/// query and the client gave up first while the daemon carried on
/// embedding.
///
/// Three assertions, and each fails differently:
/// * **exit 0** — the client got an answer at all, inside the product's own
///   budget. This is the one the pre-S6 build could not satisfy.
/// * **`semantic: applied`** — over an index built *by this scan*, with the
///   model installed, so the vectors are there and the fourth honest state
///   is not the reason it answered.
/// * **`truncated: false`** — the semantic half saw the whole admissible
///   set. Before S6 the scan stopped at `MAX_ROWS` units and said
///   `truncated: true` on any real corpus, so "it answered" and "it ranked
///   the corpus" were not the same claim.
#[tokio::test]
async fn a_real_search_through_the_daemon_answers_inside_sgts_own_budget() {
    install_model();
    let estate = tempfile::tempdir().expect("estate");
    scaffold_estate(estate.path());
    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let (status, report) = support::scan_to_completion(
        &http,
        &handle.endpoint,
        &handle.token,
        &serde_json::json!({
            "command_id": ulid::Ulid::generate().to_string(),
            "estate_root": estate.path(),
        }),
    )
    .await;
    assert!(status.is_success(), "scan refused: {report}");

    // The corpus has to be big enough that re-embedding it would blow the
    // budget; asserted rather than assumed, so a future change that quietly
    // shrinks the fixture turns this guard vacuous loudly instead of
    // silently. The pre-S6 build spent the whole 10 s on ~19,000 units of
    // this kind of text.
    let indexed: u64 = get(&http, &handle, "/v1/intelligence/status").await["sources"]
        .as_array()
        .expect("sources")
        .iter()
        .filter_map(|row| row["units"].as_u64())
        .sum();
    assert!(
        indexed > 1_000,
        "the crossing needs a corpus re-embedding could not afford; got {indexed} units"
    );

    let output = tokio::task::spawn_blocking({
        let data_dir = data.path().to_path_buf();
        let estate_root = estate.path().to_path_buf();
        move || {
            std::process::Command::new(SGT)
                .arg("--data-dir")
                .arg(&data_dir)
                .arg("-C")
                .arg(&estate_root)
                .args(["search", "how does the daemon admit an estate", "--json"])
                .output()
                .expect("run sgt search")
        }
    })
    .await
    .expect("join");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        output.status.success(),
        "`sgt search` must answer inside its own REQUEST_TIMEOUT over a real \
         corpus with the model installed: status {:?}\nstdout: {stdout}\nstderr: {stderr}",
        output.status.code()
    );
    let answer: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("parse `sgt search --json`: {e}\nstdout: {stdout}"));
    assert_eq!(
        answer["semantic"], "applied",
        "the model is installed and this index was built by this scan: {answer}"
    );
    assert_eq!(
        answer["truncated"], false,
        "the semantic half must see the whole admissible set: {answer}"
    );
    assert!(
        answer["hits"].as_array().is_some_and(|h| !h.is_empty()),
        "a real corpus and a real query produce hits: {answer}"
    );

    handle.shutdown().await;
}

/// **The decisive half of the crossing: the answer comes from vectors the
/// scan stored, and there is no query-time fallback.**
///
/// Same daemon, same real corpus, same real `sgt search` — but the index is
/// built on a host with no model, and the model appears afterwards. An
/// implementation that embeds the corpus at query time answers this in full
/// and says `applied`; one that reads stored vectors cannot, and A1 §15
/// forbids calling the result `applied` either way.
///
/// **Why this assertion and not a stopwatch.** The obvious guard —
/// "the search finishes inside `REQUEST_TIMEOUT`" — is only decisive on a
/// corpus tuned until re-embedding *this host* takes ten seconds, which is
/// a bound re-derived per corpus size and per machine: the class #278
/// retired, and flaky by construction on a faster or slower CI host. It was
/// measured rather than assumed: with `semantic_search` reverted to
/// embedding each generation at query time, this suite's corpus still
/// answered inside the budget and the exit-code assertion above still
/// passed. So the exit code is the operator's shape, and **this** test is
/// the one that goes red when the corpus is embedded at query time. The
/// latency claim is carried by a measurement on the real estate, where it
/// belongs, not by a clock in CI.
#[tokio::test]
async fn a_search_over_an_index_with_no_stored_vectors_says_so_at_the_crossing() {
    uninstall_model();
    let estate = tempfile::tempdir().expect("estate");
    scaffold_estate(estate.path());
    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let (status, report) = support::scan_to_completion(
        &http,
        &handle.endpoint,
        &handle.token,
        &serde_json::json!({
            "command_id": ulid::Ulid::generate().to_string(),
            "estate_root": estate.path(),
        }),
    )
    .await;
    assert!(status.is_success(), "scan refused: {report}");
    handle.shutdown().await;

    // The model appears, and the daemon restarts around it — the ordinary
    // shape of "the operator installed the assets after indexing".
    install_model();
    let handle = start_daemon(&data).await;

    let output = tokio::task::spawn_blocking({
        let data_dir = data.path().to_path_buf();
        let estate_root = estate.path().to_path_buf();
        move || {
            std::process::Command::new(SGT)
                .arg("--data-dir")
                .arg(&data_dir)
                .arg("-C")
                .arg(&estate_root)
                .args(["search", "how does the daemon admit an estate", "--json"])
                .output()
                .expect("run sgt search")
        }
    })
    .await
    .expect("join");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        output.status.success(),
        "the degraded answer is still an answer: status {:?}\nstdout: {stdout}\nstderr: {stderr}",
        output.status.code()
    );
    let answer: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("parse `sgt search --json`: {e}\nstdout: {stdout}"));
    assert_eq!(
        answer["semantic"], "not_indexed",
        "this index carries no vectors for the loaded model, and the answer \
         must say so rather than claim a ranking it cannot produce: {answer}"
    );
    // The lexical half still answers — A2 §15's degradation is "still
    // useful", not "empty".
    assert!(
        answer["hits"].as_array().is_some_and(|h| !h.is_empty()),
        "the lexical half still ranks: {answer}"
    );

    handle.shutdown().await;
}
