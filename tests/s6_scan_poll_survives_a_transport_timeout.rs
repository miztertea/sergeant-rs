//! S6 transport-timeout-is-not-a-verdict: `scan_to_completion`'s status poll
//! (`tests/support/mod.rs`) must not let a *transport*-class failure on the
//! `GET .../scan/{id}` poll — a stalled connection, a client-side
//! `.timeout()` expiring — decide the test's verdict. Release run
//! 33665140069, Gate B: one status poll exceeded the caller's client's 60s
//! timeout on a starved runner (its sibling in the same job took 165s
//! against ~70s solo) and `.expect("scan status request")` turned that into
//! a raw panic indistinguishable from a dead daemon —
//! `knowledge/evidence/resources/host-atlas-s6-series/
//! brief-transport-timeout-is-not-a-verdict.md`.
//!
//! `src/api.rs::send_with_retry`'s own doc comment (~line 7754) is the
//! product's answer to exactly this failure class: a transport error
//! against a daemon provably still alive is retried; a provably dead one
//! fails at once naming it; a real HTTP status is never retried. This suite
//! pins the same shape for the test harness's own poll helper.

use std::sync::atomic::Ordering;
use std::time::Duration;

use serde_json::json;

mod support;

fn stalling_client(client_timeout: Duration) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(client_timeout)
        .build()
        .expect("client")
}

/// The regression itself: a scripted daemon that accepts the scan, then
/// stalls the *first* status poll well past the caller's client timeout —
/// the client gives up on that connection at 150ms and never sees the
/// server's late answer — and answers the *retried* poll (a fresh
/// connection) promptly. Before this wave: `scan_to_completion` panics at
/// the stalled poll's transport error, exactly like the release failure.
/// After: it retries past the stall (the daemon is alive — nothing here
/// claims otherwise), reaches the fresh connection, and returns the
/// completed report.
#[tokio::test]
async fn a_status_poll_that_stalls_past_the_client_timeout_is_retried_not_panicked() {
    let (endpoint, attempts) = support::spawn_scripted_http_server(vec![
        (
            202,
            r#"{"scan_id":"abc123","state":"running","scanned":[]}"#,
            Duration::ZERO,
        ),
        // Stalls 600ms past the client's 150ms timeout: the client aborts
        // this connection before this (late, unreceived) answer is written.
        (
            200,
            r#"{"scan_id":"abc123","state":"completed","scanned":["x"]}"#,
            Duration::from_millis(600),
        ),
        // The retried poll's own, fresh connection: answered immediately.
        (
            200,
            r#"{"scan_id":"abc123","state":"completed","scanned":["x"]}"#,
            Duration::ZERO,
        ),
    ]);
    let http = stalling_client(Duration::from_millis(150));

    let (status, report) =
        support::scan_to_completion(&http, &endpoint, "stub-token", &json!({}), || true).await;

    assert!(
        status.is_success(),
        "expected a success status, got {status}"
    );
    assert_eq!(
        report["state"], "completed",
        "the stalled-then-answered scan must complete: {report}"
    );
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        3,
        "the accept POST, the stalled status GET the client gave up on, and the retried \
         status GET that actually landed — no more"
    );
}

/// The other half of the same rule ("by state, not by a count",
/// `src/api.rs::send_with_retry`'s own precedent): a transport failure on
/// the status poll against a daemon `is_alive` reports as gone must fail at
/// once, naming that, rather than retrying — proven alongside the retry
/// above so the retry this wave adds can never grow into an unconditional
/// loop against a genuinely dead daemon.
#[tokio::test]
#[should_panic(expected = "the daemon is no longer alive")]
async fn a_status_poll_transport_failure_against_a_dead_daemon_fails_at_once() {
    let (endpoint, _attempts) = support::spawn_scripted_http_server(vec![
        (
            202,
            r#"{"scan_id":"abc123","state":"running","scanned":[]}"#,
            Duration::ZERO,
        ),
        (0, "", Duration::ZERO), // hangup: connection reset, not a status
    ]);
    let http = stalling_client(Duration::from_secs(5));

    support::scan_to_completion(&http, &endpoint, "stub-token", &json!({}), || false).await;
}
