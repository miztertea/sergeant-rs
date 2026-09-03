//! W2fix (#293): the daemon serves before it has finished probing, and a
//! submission that needs capability evidence waits for it instead of being
//! answered from thin air.
//!
//! ## What went wrong, and what these tests pin
//!
//! Startup used to run the whole backend probe walk — serially, one
//! subprocess set per adapter — *before* binding-and-publishing, so the
//! runtime descriptor appeared only after every installed third-party CLI had
//! printed `--help`. Measured on Cerberus 2026-08-25 with all five backends
//! installed: descriptor at ~6.4s against a 10s client spawn budget, which
//! meant any concurrent load pushed the first `sgt run` on a cold daemon past
//! the budget and failed every real-spawn suite at once (#293's A/B evidence).
//!
//! The fix is an ordering change, so the pins here are **ordering assertions
//! and nothing else** — no wall-clock thresholds, which would only re-encode
//! one host's timings as a contract. Each test parks a probe in
//! [`FakeBackend::hold_probes`]'s gate, which stands in for an arbitrarily
//! slow real adapter, and then acts exactly as a client does — through the
//! **runtime descriptor file**, not through the in-process handle, because
//! the descriptor is the thing whose lateness was the defect and
//! `daemon::start_with` deliberately waits for the walk before handing a
//! handle back (see its doc). What the tests assert while a probe is parked:
//!
//! 1. the descriptor exists and `/healthz` answers;
//! 2. reads are served in that window, a submission arriving in it is *not*
//!    answered from unprobed capabilities, and it completes correctly once
//!    the probe lands;
//! 3. `backend.probed` still precedes the first Work routed to that backend
//!    in the journal — the evidence order the M4 contract requires, which the
//!    new ordering must not have bought its latency with.

use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend};
use sergeant_rs::daemon::{self, DaemonConfig, DaemonHandle, RuntimeDescriptor};
use sergeant_rs::domain::event::Event;
use sergeant_rs::domain::work::KIND_WORK_SUBMITTED;
use sergeant_rs::runtime::journal::Journal;

use crate::support;
use support::DataDir;

/// How long a rendezvous may take before the test calls it a hang. This is a
/// deadlock guard, not a latency pin: nothing here asserts that any operation
/// was *fast*, only that it happened at all before the suite gave up. Wave
/// `timeout-the-function` (S6, item 2): raised from a bespoke 30s onto the
/// crate's one shared hang-only bound, `support::HANG_BUDGET` — this is
/// exactly the "ends a hang, decides nothing" shape that bound exists for.
const RENDEZVOUS: Duration = support::HANG_BUDGET;

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("client")
}

/// A registry that pre-empts every real adapter `start_with` would otherwise
/// register, so this suite probes no third-party CLI at all.
///
/// `start_with` only adds an adapter when the configured registry has no
/// backend under that name, which is exactly the substitution seam tests are
/// meant to use. Hermetic matters here beyond speed: a host with opencode
/// installed would otherwise put a ~3s real probe inside every rendezvous
/// below and make the suite's own timings depend on what is on the box —
/// which is the class of coupling this whole wave exists to remove.
fn hermetic_registry(subject: &Arc<FakeBackend>) -> BackendRegistry {
    let mut registry = BackendRegistry::new().with(subject.clone());
    for name in ["claude", "docker", "codex", "opencode", "agy"] {
        registry = registry.with(Arc::new(FakeBackend::new(name)));
    }
    registry
}

/// Releases the subject's probe gate on drop.
///
/// Not politeness: a probe parked in the gate is a live `spawn_blocking`
/// task, and dropping a tokio runtime waits for those. A failing assertion
/// between `hold_probes` and `release_probes` would therefore hang the whole
/// test binary instead of reporting the failure — a suite that cannot fail
/// out loud is worse than no suite. `CONTRIBUTING.md`'s rule for exactly this
/// shape: clean up in an RAII guard, never in the happy-path test body.
struct ReleaseProbes(Arc<FakeBackend>);

impl Drop for ReleaseProbes {
    fn drop(&mut self) {
        self.0.release_probes();
    }
}

/// Begin a daemon start with the subject's probe held, and return once that
/// probe is provably parked — i.e. once the process is inside the window this
/// suite is about. The join handle finishes when the whole startup does.
async fn start_into_the_pending_window(
    dir: &DataDir,
    subject: &Arc<FakeBackend>,
) -> tokio::task::JoinHandle<DaemonHandle> {
    subject.hold_probes();
    let starting = {
        let data_dir = dir.path().to_path_buf();
        let config = DaemonConfig {
            backends: Arc::new(hermetic_registry(subject)),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            ..DaemonConfig::default()
        };
        tokio::spawn(async move {
            daemon::start_with(&data_dir, config)
                .await
                .expect("daemon start")
        })
    };
    let gate = subject.clone();
    let parked = tokio::task::spawn_blocking(move || gate.await_stalled_probes(1, RENDEZVOUS))
        .await
        .expect("gate rendezvous task");
    assert!(parked, "a probe must be parked in the gate");
    starting
}

/// The descriptor a client would read, refusing the "not published yet" case
/// as the failure it is rather than polling for it.
fn published_descriptor(dir: &DataDir) -> RuntimeDescriptor {
    daemon::read_descriptor(dir.path())
        .expect("descriptor readable")
        .expect("the descriptor is published before the probe walk completes")
}

/// Deliverable 4(a). The runtime descriptor — the thing every client and test
/// rig waits for — exists, and the daemon behind it is already accepting,
/// while the backend probe walk is still running.
///
/// Revert-sensitive by construction: restore the probe-walk-before-publish
/// ordering and no descriptor exists at all while the gate is held, so this
/// fails on a missing file rather than on a threshold.
#[tokio::test(flavor = "multi_thread")]
async fn the_descriptor_is_published_while_a_backend_probe_is_still_running() {
    let dir = DataDir::new();
    let subject = Arc::new(FakeBackend::new(FAKE_BACKEND_NAME));
    let starting = start_into_the_pending_window(&dir, &subject).await;
    let _release = ReleaseProbes(subject.clone());

    assert_eq!(
        subject.probe_count(),
        0,
        "the subject's probe has not completed yet"
    );
    let descriptor = published_descriptor(&dir);
    assert_eq!(descriptor.pid, std::process::id());

    // Published *and* live: a descriptor pointing at a socket nobody is
    // accepting on would trade one broken promise for another.
    let health = support::send_while_alive(
        "healthz",
        || client().get(format!("{}/healthz", descriptor.endpoint)),
        || daemon::pid_alive(descriptor.pid),
    )
    .await;
    assert!(
        health.status().is_success(),
        "the daemon is already serving"
    );

    subject.release_probes();
    let handle = tokio::time::timeout(RENDEZVOUS, starting)
        .await
        .expect("startup must complete once the probe lands")
        .expect("start task");
    handle.shutdown().await;
    assert!(dir.reap().is_empty(), "no daemon left on this data dir");
}

/// Deliverables 4(b) and 4(c). During the pending window the daemon serves
/// reads, a submission arriving in it is not answered from capabilities
/// nobody measured, and once the probe lands the submission completes — with
/// `backend.probed` ahead of `work.submitted` in the journal.
#[tokio::test(flavor = "multi_thread")]
async fn a_submission_during_probe_pending_lands_after_the_probe_it_needs() {
    let dir = DataDir::new();
    let subject = Arc::new(FakeBackend::new(FAKE_BACKEND_NAME));
    let starting = start_into_the_pending_window(&dir, &subject).await;
    let _release = ReleaseProbes(subject.clone());
    let descriptor = published_descriptor(&dir);

    let http = client();
    let submit = {
        // This POST is not idempotent (it creates a Work), so it cannot be
        // retried the way the GETs in this file are: `support::
        // send_while_alive` is deliberately not used here. What made this
        // call non-deterministic instead was a second, shorter deadline
        // racing the legitimate one — `client()`'s own 10s transport
        // timeout could fire before the daemon answers (this submission is
        // held open until the probe below releases it), while the
        // `RENDEZVOUS`-bounded `tokio::time::timeout` a few lines down is
        // already the real, single termination bound for this wait. A
        // client with no timeout of its own removes the arbitrary duration
        // that used to be able to decide this call's outcome, leaving
        // `RENDEZVOUS` as the one bound in force.
        let http = reqwest::Client::new();
        let endpoint = descriptor.endpoint.clone();
        let token = descriptor.token.clone();
        tokio::spawn(async move {
            http.post(format!("{endpoint}/v1/work"))
                .bearer_auth(&token)
                .json(&json!({
                    "command_id": ulid::Ulid::generate().to_string(),
                    "intent": "a submission that arrived during the probe-pending window",
                }))
                .send()
                .await
                .expect("submit request")
        })
    };

    // The daemon is serving reads in the pending window — and the submission
    // is genuinely still pending in it. Asserted on the daemon's own state
    // rather than on `is_finished()`, so nothing here depends on scheduling.
    let listed: Value = support::send_while_alive(
        "list",
        || {
            http.get(format!("{}/v1/work", descriptor.endpoint))
                .bearer_auth(&descriptor.token)
        },
        || daemon::pid_alive(descriptor.pid),
    )
    .await
    .json()
    .await
    .expect("list json");
    assert_eq!(
        listed["works"].as_array().map(Vec::len),
        Some(0),
        "no Work may be recorded while the backend it would route to is unprobed"
    );

    subject.release_probes();
    let response = tokio::time::timeout(RENDEZVOUS, submit)
        .await
        .expect("the pending submission must complete once its probe lands")
        .expect("submit task");
    assert_eq!(
        response.status(),
        reqwest::StatusCode::CREATED,
        "the submission completes correctly after the probe lands"
    );
    let body: Value = response.json().await.expect("submit json");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();

    let handle = tokio::time::timeout(RENDEZVOUS, starting)
        .await
        .expect("startup must complete once the probe lands")
        .expect("start task");
    handle.shutdown().await;
    assert!(dir.reap().is_empty(), "no daemon left on this data dir");

    // Deliverable 4(c): the evidence order the M4 contract requires is
    // unchanged — the probe record for a backend is durable before any Work
    // routed to it is.
    let events: Vec<Event> = Journal::replay_data_dir(dir.path())
        .expect("replay")
        .map(|e| e.expect("event"))
        .collect();
    let probed = events
        .iter()
        .find(|e| {
            e.kind == daemon::KIND_BACKEND_PROBED && e.payload["backend"] == FAKE_BACKEND_NAME
        })
        .expect("a backend.probed record for the fake");
    let submitted = events
        .iter()
        .find(|e| e.kind == KIND_WORK_SUBMITTED && e.work_id.as_deref() == Some(work_id.as_str()))
        .expect("a work.submitted record");
    assert!(
        probed.seq < submitted.seq,
        "backend.probed (seq {}) must precede the first Work routed to that backend (seq {})",
        probed.seq,
        submitted.seq
    );
}
