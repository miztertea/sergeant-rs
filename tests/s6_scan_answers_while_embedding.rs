//! S6 seam 4 — the embed path must not starve the runtime.
//!
//! **The defect this suite exists for**, confirmed on a real daemon with a
//! real installed model over real repository text
//! (`/var/tmp/hats7/orient-s4-repro-output.log`, "no-clock-decides"
//! `00-orient`): `run_estate_scan` -> `with_atlas_write` ->
//! `AtlasDb::index_generation` calls `SemanticEngine::embed` **inline**, on
//! whatever task/thread is running the scan — no `spawn_blocking`, unlike
//! `Engine::run_intelligence`, which exists for exactly this class of job
//! ("Run directly in an async task it would hold a runtime worker thread
//! for its whole duration and stall every unrelated future sharing it").
//! The observed consequence: a concurrent `GET
//! /v1/intelligence/scan/{id}` — a request that touches none of the state
//! the embed call holds — could not be answered at all until the embed
//! batch finished.
//!
//! # The proof, and why it is not a stopwatch
//!
//! The owner ruling (`no-nondeterministic-tests-2026-09-02.md`) forbids a
//! duration deciding a verdict, so this test never asks "did the GET
//! answer within N ms". It also cannot simply ask "does the scan's
//! `state` ever read something other than `completed`" — measured
//! directly (see this seam's implementation notes), that is true on
//! *both* builds, because a generous, hang-only client timeout means a
//! slow answer is still an answer, and the field a slow answer reports is
//! indistinguishable from a fast one's.
//!
//! What **does** differ, and differs as a matter of scheduling rather
//! than speed, is whether a concurrent poll can ever be *in flight at
//! all* while a write is running. `ApiState::scans`'s new
//! `writing_source` field (this seam's product signal, seam 1's own
//! fallback applied here: *"where no state exists that proves the
//! property, add the smallest observable signal"*) names the source
//! currently being written, embed included, and is only ever visible to
//! an external poll if the runtime could schedule that poll during the
//! write. Before the fix, the write holds the one thread the poll also
//! needs, so no poll — however it is timed or however long it is given
//! to wait — can ever land inside that window; `writing_source` reads
//! null on every answer that ever arrives. After the fix
//! (`spawn_blocking`, matching `Engine::run_intelligence`'s existing
//! shape), the write runs off that thread and a poll landing during a
//! real, multi-hundred-millisecond-or-longer write (this test's
//! real-repository-text corpus; see its own embed timings) reliably
//! observes it set. So this test's assertion is `seen_writing == false`
//! at the end of a whole scan — a fact about whether a window was ever
//! observable, never about how fast anything answered.
//!
//! # Why the wait ends at observation, not at `completed`
//! (owner ruling `no-nondeterministic-tests-2026-09-02.md`, "a wait
//! budget only ends a hang" / "a duration never decides a verdict")
//!
//! This test's decisive fact is `writing_source` having been seen
//! non-null on some poll — not that the scan finishes. Waiting for
//! `state == "completed"` measured a full, real embed (all of
//! `BULK_SOURCES`) against `support::HANG_BUDGET`, a bound sized for
//! "never finishes"; under host contention that embed can legitimately
//! run past 120s even though nothing has hung
//! (`/var/tmp/hats7/close-starve-final.log:95-96,220-221`: two panics at
//! 154-156s beside a churning release build, one core). So the polling
//! predicate below stops the instant `writing_source` is first observed
//! set — the property under test is proven the moment it is seen, and
//! nothing about *how long the scan then takes to finish* is part of the
//! verdict. A terminal `completed` reached before that stays a real
//! failure (asserted below by the same message either way), so a scan
//! that finishes without ever exposing the field still fails correctly.
//!
//! Stopping the async wait early does not make the test's wall time
//! track only the observation, though: `#[tokio::test]` builds a
//! single-threaded-by-default `Runtime` whose `Drop` blocks the test
//! function's own OS thread until every task it spawned — including the
//! `run_estate_scan` future's detached `spawn_blocking` embed work
//! (`src/api.rs::run_estate_scan`, no retained `JoinHandle`) — finishes,
//! confirmed by reading `src/daemon.rs::DaemonHandle::shutdown`, which
//! awaits only the serve task and two explicitly-bounded joins and never
//! touches that detached task (00-orient's finding: no `src/` defect at
//! this revision — `handle.shutdown()` below does not wait on the write,
//! but the runtime's own teardown still does). So this test's tail is
//! still bounded — by that drain, never by a clock this test asserts
//! against — which is exactly why `BULK_SOURCES` is kept to the smallest
//! real corpus that reliably clears the seam, rather than left at
//! whatever size made the old `completed`-gated wait pass.
use std::path::Path;
use std::sync::Arc;

use serde_json::Value;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig};
use sergeant_rs::runtime::atlas::semantic::MODEL_DIR_ENV;

mod support;
use support::DataDir;

fn install_model() {
    support::install_model(MODEL_DIR_ENV);
}

/// Real repository text, sized to the seam by measurement rather than
/// guessed (owner brief `brief-observe-then-stop.md`, "Corpus" section,
/// R1/R6): a fixture corpus of a few one-line documents embeds inside any
/// polling cadence, so the write path never holds the runtime long enough
/// for a concurrent poll to land inside it, but a full 5-source real-repo
/// corpus (the shape this file used before this seam) is far larger than
/// the property needs and is why the old, `completed`-gated wait ran
/// 80-115s. Measured directly on this revision, this single directory's
/// own write span alone — instrumented with `eprintln!`s bracketing every
/// `writing_source` transition, `/var/tmp/hats7/s6-measure-run2.log` —
/// ran from t=44ms to t=1.865s (~1.8s), multiple orders of magnitude past
/// `support::WAIT_POLL_INTERVAL` (20ms) and comfortably inside the
/// "multi-hundred-ms or longer" the brief asks for; that is the corpus.
const BULK_SOURCES: [&str; 1] = ["skills"];

fn scaffold_estate(root: &Path) {
    std::fs::create_dir_all(root).expect("estate root");
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut manifest = String::from("[estate]\nname = \"s6-embed-starve\"\n");
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
        .timeout(support::HANG_BUDGET)
        .build()
        .expect("client")
}

/// **The regression this seam exists to close.**
///
/// Posts a scan over the real-text corpus with the model installed, then
/// polls `GET /v1/intelligence/scan/{scan_id}` at [`support::wait_until`]'s
/// ordinary cadence — but, per this file's module doc ("Why the wait ends
/// at observation, not at `completed`"), the wait stops at the first poll
/// that observes `writing_source` non-null, not at `state == "completed"`:
/// waiting for the scan to finish would measure a real, legitimate embed
/// duration against `support::HANG_BUDGET`, a bound sized only to catch a
/// hang, which is exactly the verdict-by-duration the owner ruling
/// forbids. A terminal `completed` reached before `writing_source` was
/// ever observed set is still the real failure, asserted below by the
/// same message either way.
///
/// **The assertion is not about the clock at all.** Every poll along the
/// way records whether the answer's `writing_source` field was non-null
/// — the daemon's own signal (`ApiState::scans`'s `ScanProgress::
/// writing_source`) that a per-source write, embed included, is in
/// flight right now. That field can only ever be visible to an external
/// poll if the daemon's runtime could schedule the poll *while* the
/// write was running. Before this seam's fix, the write held the one
/// thread a poll also needs, so no poll ever lands inside that window —
/// `writing_source` is always null by the time anything answers, no
/// matter how the polling is timed. After the fix, the write runs off
/// that thread, and a poll landing during a real (multi-hundred-ms or
/// longer, per this test's real-repository-text corpus) write reliably
/// observes it set. So the failing assertion here is `seen_writing`
/// being `false` at the end of a whole scan — never a duration, a
/// timeout, or a rate.
#[tokio::test]
async fn a_status_get_observes_a_write_in_flight_while_a_scan_is_embedding() {
    install_model();
    let estate = tempfile::tempdir().expect("estate");
    scaffold_estate(estate.path());
    let data = DataDir::new();
    let handle = start_daemon(&data).await;
    let http = client();

    let accepted: Value = http
        .post(format!("{}/v1/intelligence/scan", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&serde_json::json!({
            "command_id": ulid::Ulid::generate().to_string(),
            "estate_root": estate.path(),
        }))
        .send()
        .await
        .expect("scan request")
        .json()
        .await
        .expect("json body");
    let scan_id = accepted["scan_id"]
        .as_str()
        .expect("scan accepted with an id")
        .to_string();
    let endpoint = &handle.endpoint;
    let token = &handle.token;
    let seen_writing = std::cell::Cell::new(false);

    support::wait_until(
        "a status GET observes `writing_source` set while the scan is in flight (a terminal \
         `completed` observed first is the real failure, asserted below)",
        support::HANG_BUDGET,
        || async {
            let body: Value = support::send_while_alive(
                "GET the scan's status while it is in flight",
                || {
                    http.get(format!("{endpoint}/v1/intelligence/scan/{scan_id}"))
                        .bearer_auth(token)
                },
                || handle.is_alive(),
            )
            .await
            .json()
            .await
            .expect("json body");
            if !body["writing_source"].is_null() {
                seen_writing.set(true);
            }
            seen_writing.get() || body["state"] == "completed"
        },
    )
    .await;

    assert!(
        seen_writing.get(),
        "no status GET, across this whole scan, ever observed `writing_source` set — the \
         only way that can happen is if the daemon's runtime could never schedule a \
         concurrent poll while a source's write (embed included) was in flight, which is \
         exactly the runtime starvation seam 4 exists to close"
    );

    handle.shutdown().await;
}
