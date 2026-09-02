//! Reconnect and liveness, preserved from the M6-era `tui.rs`: capped
//! exponential backoff, the durable [`Live`] indicator, and the auth-failure
//! rule that stops automatic retries against a token the daemon will never
//! accept again (issue #16). §17.2 restates these as binding rules rather
//! than changing them.

use std::time::Duration;

use serde_json::Value;

use crate::api::{ApiClient, ClientError};

use super::app::App;

/// The delay before the first reconnect attempt, and the unit the backoff
/// curve doubles from (see [`Backoff`]).
pub const RECONNECT_BASE: Duration = Duration::from_millis(250);

/// Where the backoff curve stops growing: a long-dead tail is retried every
/// 30s forever rather than backing off without bound.
pub const RECONNECT_CAP: Duration = Duration::from_secs(30);

/// Whether the live tail is attached.
///
/// This is *screen state*, not a transient message, and it is rendered in the
/// header next to the seq counter for exactly that reason. The one-line status
/// at the bottom is overwritten by the next command outcome, so a
/// message-only signal would leave a screen that looks live long after a tail
/// died two keystrokes ago — fixed data presented as live truth, which is the
/// failure mode §7 cares about. Recovery from a dead tail is automatic
/// (issue #16: capped backoff, `Reconnecting` below), but the durable
/// indicator stays either way — an attempt still in flight, or one that has
/// stopped for good on a rejected token, must never be reported as `live`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Live {
    /// Attached: the daemon's SSE tail is feeding this screen.
    #[default]
    Attached,
    /// The tail died and the loop is retrying with capped backoff: recovery
    /// no longer waits on a keystroke, but the screen still says so rather
    /// than looking live through the gap. `r` forces an attempt now instead
    /// of waiting on the backoff.
    Reconnecting,
    /// The daemon flatly rejected this client's token (issue #16's other
    /// half): retrying automatically cannot help — the token this process
    /// holds is not one the daemon will ever accept again — so the loop
    /// stopped rather than spend its backoff budget on a door that will
    /// never open. `r` still tries once, honestly.
    AuthFailed,
}

impl Live {
    /// How the header says it, in the one place that decides.
    pub fn label(self) -> &'static str {
        match self {
            Live::Attached => "live",
            Live::Reconnecting => "RECONNECTING… (r retries now)",
            Live::AuthFailed => "AUTH FAILED — token rejected (r retries)",
        }
    }
}

/// Capped exponential backoff between reconnect attempts.
///
/// Pure — computing the next delay never sleeps and never reads a clock —
/// so the growth curve and the cap are asserted directly rather than by
/// waiting one out.
#[derive(Debug, Clone, Copy)]
pub struct Backoff {
    attempt: u32,
}

impl Backoff {
    pub fn new() -> Self {
        Self { attempt: 0 }
    }

    /// The delay before the next attempt, advancing the counter by one.
    ///
    /// The first call already returns [`RECONNECT_BASE`], never zero: a dead
    /// tail retried with no delay at all is the reader-thread spin issue #3
    /// fixed once, reintroduced in a different loop.
    pub fn next_delay(&mut self) -> Duration {
        let shift = self.attempt.min(16); // 250ms << 16 already clears the cap
        self.attempt += 1;
        RECONNECT_BASE
            .saturating_mul(1u32 << shift)
            .min(RECONNECT_CAP)
    }

    /// Back to the first attempt's delay — called once a reconnect succeeds,
    /// so the *next* time the tail dies it is retried quickly again rather
    /// than resuming at whatever this outage's backoff had grown to.
    pub fn reset(&mut self) {
        self.attempt = 0;
    }
}

impl Default for Backoff {
    fn default() -> Self {
        Self::new()
    }
}

/// Whether a failed attach is the daemon flatly rejecting this client's
/// token, rather than being unreachable or mid-restart.
///
/// The distinction is what issue #16 asks for by name: a stale token (the
/// daemon restarted and rotated it) will never start working just because
/// this process asks again, so the auto-reconnect loop must stop instead of
/// spending its backoff budget on a door that will never open.
pub fn is_auth_failure(err: &ClientError) -> bool {
    matches!(err, ClientError::Api { status: 401, .. })
}

/// What one attach attempt produced.
pub enum Attach {
    /// The tail is open. The caller still owes a refresh before treating it
    /// as resumed — see [`reconnected`].
    Opened(crate::api::EventStream),
    /// Failed for a reason another attempt might fix.
    Retry(String),
    /// Failed because the token this client holds is not one the daemon
    /// will ever accept — see [`is_auth_failure`].
    AuthFailed(String),
}

/// Try to open the live tail from `from`, without touching [`App`] — every
/// caller (first attach, `r`, and the backoff loop) reacts to the outcome
/// differently, through [`reconnected`].
pub async fn try_attach(client: &ApiClient, from: u64) -> Attach {
    // No estate filter: the TUI's own estate-filter wiring is W4d's
    // deliverable (H1 sprint plan), not this wave's — `sgt watch`'s D6
    // filter (`src/watch.rs`) is the only client this wave routes onto the
    // new `estate_root` parameter.
    match client.stream_events(from, None).await {
        Ok(stream) => Attach::Opened(stream),
        Err(e) if is_auth_failure(&e) => Attach::AuthFailed(e.to_string()),
        Err(e) => Attach::Retry(e.to_string()),
    }
}

/// Fold one attach attempt's outcome into the screen, and hand back the
/// stream to keep tailing (`None` if there is none).
///
/// Used for the first attach and for every reconnect — manual or automatic —
/// so "attached" is decided in one place and cannot drift from what the
/// header says. The success arm is where issue #16's subtle requirement
/// lives: **refresh before resuming the tail**. An SSE gap means events were
/// missed, so handing the stream back before re-reading the API would leave
/// the screen confidently wrong about everything that happened during the
/// gap — a reconnect that skips this is worse than no reconnect, because it
/// looks live while lying.
pub async fn reconnected(
    client: &ApiClient,
    app: &mut App,
    backoff: &mut Backoff,
    outcome: Attach,
) -> Option<crate::api::EventStream> {
    match outcome {
        Attach::Opened(stream) => {
            if let Err(e) = app.refresh(client).await {
                app.live = Live::Reconnecting;
                app.status = format!("reconnected, but refresh failed: {e} — retrying…");
                return None;
            }
            backoff.reset();
            app.live = Live::Attached;
            Some(stream)
        }
        Attach::Retry(detail) => {
            app.live = Live::Reconnecting;
            app.status = format!("live tail unavailable: {detail} — reconnecting…");
            None
        }
        Attach::AuthFailed(detail) => {
            app.live = Live::AuthFailed;
            app.status = format!(
                "live tail auth failed: {detail} — the token this session holds \
                 is stale; restart sgt to pick up a fresh one"
            );
            None
        }
    }
}

/// What one step of the live tail produced, for the caller to act on
/// deliberately (R-WATCH-7: `EventStream::next_event` distinguishes a decoded
/// event, a clean/transport stream end, and a malformed frame — this audits
/// that the TUI does not let the new variant collapse back into "the tail
/// ended" the way a naive `.ok()` would).
pub enum TailStep {
    /// A decoded event, as the JSON the rest of this module already expects.
    Event(Value),
    /// The stream ended — cleanly or via a transport failure; both leave the
    /// screen in the same detached state.
    Ended,
    /// The daemon sent a `data:` frame this client could not decode. Distinct
    /// from `Ended` so the status line says what actually happened instead of
    /// quietly relabeling protocol drift as an ordinary disconnect.
    Malformed(String),
}

/// Await the next SSE event, or park forever when there is no stream — so
/// `select!` keeps working on the keyboard arm alone.
pub async fn next_event(stream: &mut Option<crate::api::EventStream>) -> TailStep {
    match stream {
        Some(stream) => match stream.next_event().await {
            Ok(Some(event)) => serde_json::to_value(event)
                .map(TailStep::Event)
                .unwrap_or(TailStep::Ended),
            Ok(None) => TailStep::Ended,
            Err(malformed) => TailStep::Malformed(malformed.to_string()),
        },
        None => std::future::pending().await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Backoff never fires the first retry instantly (that is the reader
    /// thread's spin, issue #3, reintroduced in a different loop) and never
    /// grows past its cap — reverting either half of `Backoff::next_delay`
    /// fails one of these two assertions.
    #[test]
    fn reconnect_backoff_starts_above_zero_and_stops_growing() {
        let mut backoff = Backoff::new();
        let first = backoff.next_delay();
        assert!(
            first > Duration::ZERO,
            "an instant first retry is the reader-thread spin (issue #3) in a new loop"
        );
        assert_eq!(first, RECONNECT_BASE);
        assert_eq!(
            backoff.next_delay(),
            RECONNECT_BASE * 2,
            "the curve doubles"
        );
        assert_eq!(backoff.next_delay(), RECONNECT_BASE * 4);

        // A long outage must not make the delay grow without bound.
        for _ in 0..30 {
            backoff.next_delay();
        }
        assert_eq!(
            backoff.next_delay(),
            RECONNECT_CAP,
            "backoff must stop growing at the cap, not keep doubling forever \
             (that is the spin this test guards, just paced instead of tight)"
        );
    }

    /// Reconnecting resets the curve, so the *next* outage is retried
    /// quickly again rather than picking up where a long-past one left off.
    #[test]
    fn reconnect_backoff_resets_after_a_success() {
        let mut backoff = Backoff::new();
        backoff.next_delay();
        backoff.next_delay();
        backoff.next_delay();
        backoff.reset();
        assert_eq!(backoff.next_delay(), RECONNECT_BASE);
    }

    /// Only the daemon's own 401 reads as "this token will never work" — a
    /// transport failure or any other status is something a later attempt
    /// might still fix, so it must not be mistaken for the terminal case.
    #[test]
    fn only_a_401_reads_as_an_auth_failure() {
        assert!(is_auth_failure(&ClientError::Api {
            status: 401,
            code: "unauthorized".to_string(),
            message: "missing or invalid bearer token".to_string(),
        }));
        assert!(!is_auth_failure(&ClientError::Api {
            status: 500,
            code: "internal".to_string(),
            message: "oops".to_string(),
        }));
        assert!(!is_auth_failure(&ClientError::Transport(
            "connection refused".to_string()
        )));
    }

    /// A retryable attach failure is `Reconnecting`, not a silent retry and
    /// not a stale `Attached` — the header must say so distinctly from both
    /// "live" and the terminal auth-failure state.
    #[tokio::test]
    async fn a_retryable_attach_failure_is_reconnecting_not_silent_or_stale_live() {
        let mut app = App::new();
        app.live = Live::Attached;
        let client = ApiClient::new("http://127.0.0.1:1", "t").expect("client");
        let mut backoff = Backoff::new();

        let stream = reconnected(
            &client,
            &mut app,
            &mut backoff,
            Attach::Retry("connection refused".to_string()),
        )
        .await;

        assert!(stream.is_none(), "a failed attempt hands back no stream");
        assert_eq!(app.live, Live::Reconnecting);
        assert_ne!(
            app.live,
            Live::Attached,
            "must not look attached while retrying"
        );
    }

    /// A daemon that rejects the token (e.g. reissued on restart) stops the
    /// loop instead of looping forever against a door that will never open.
    #[tokio::test]
    async fn an_auth_failure_surfaces_instead_of_retrying_forever() {
        let mut app = App::new();
        let client = ApiClient::new("http://127.0.0.1:1", "t").expect("client");
        let mut backoff = Backoff::new();

        let stream = reconnected(
            &client,
            &mut app,
            &mut backoff,
            Attach::AuthFailed("missing or invalid bearer token".to_string()),
        )
        .await;

        assert!(stream.is_none());
        assert_eq!(
            app.live,
            Live::AuthFailed,
            "an auth failure must not be reported as an ordinary retrying tail"
        );
    }

    /// The subtle half of issue #16: a successful reconnect refreshes state
    /// *before* the caller can resume tailing. An SSE gap means events were
    /// missed — resuming the read alone would leave the screen confidently
    /// wrong about everything that happened during the gap.
    ///
    /// Driven against a real socket, not a mock: `reconnected` awaiting its
    /// own `App::refresh` internally is exactly the ordering a mocked
    /// `ApiClient` could not tell apart from "returns the stream, refreshes
    /// later, if ever."
    #[tokio::test]
    async fn a_successful_reconnect_refreshes_state_before_handing_back_the_tail() {
        use std::io::{Read, Write};

        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind stand-in daemon");
        let addr = listener.local_addr().expect("local addr");
        std::thread::spawn(move || {
            fn respond(mut stream: std::net::TcpStream, content_type: &str, body: &str) {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
            }
            // 1: the reconnect's own attach — the tail itself carries nothing.
            if let Ok((stream, _)) = listener.accept() {
                respond(stream, "text/event-stream", "");
            }
            // 2 & 3: the refresh `reconnected` owes before handing the tail
            // back, in `App::refresh`'s own call order (system, then fleet).
            if let Ok((stream, _)) = listener.accept() {
                respond(
                    stream,
                    "application/json",
                    r#"{"version":"post-gap","api_revision":"v1","data_dir":"/tmp/d","journal_head":9}"#,
                );
            }
            if let Ok((stream, _)) = listener.accept() {
                respond(
                    stream,
                    "application/json",
                    r#"{"works":[{"id":"w1","state":"running","intent":"post-gap work","stage":{"stage_id":"10-implement","status":"running"},"resolved_backend":"fake"}]}"#,
                );
            }
        });

        let client = ApiClient::new(&format!("http://{addr}"), "unused-token").expect("client");
        let mut app = App::new();
        app.live = Live::Reconnecting;
        app.system = serde_json::json!({"version": "pre-gap"});
        let mut backoff = Backoff::new();

        let attempt = try_attach(&client, 0).await;
        let stream = reconnected(&client, &mut app, &mut backoff, attempt).await;

        assert!(
            stream.is_some(),
            "a clean 200 on the stream route must attach"
        );
        assert_eq!(app.live, Live::Attached);
        assert_eq!(
            app.system["version"], "post-gap",
            "the reconnect must refresh state before the caller can resume the \
             tail — a stream handed back with the pre-gap snapshot still in \
             `app.system` is exactly the invariant this guards"
        );
        assert_eq!(app.rows.len(), 1, "the fleet refresh must have run too");
    }

    /// Review found two bugs on this same success path that a mocked client
    /// could not tell apart from correct behavior: a stream that opens but
    /// whose refresh then fails must not be reported `Attached` (that would
    /// show the pre-gap snapshot as live), and the backoff must not be reset
    /// until the refresh actually succeeds — resetting on open alone would
    /// make a reconnect that keeps opening a stream but failing its refresh
    /// retry at `RECONNECT_BASE` forever instead of backing off.
    #[tokio::test]
    async fn a_failed_refresh_after_a_successful_attach_stays_reconnecting_and_keeps_the_backoff() {
        use std::io::{Read, Write};

        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind stand-in daemon");
        let addr = listener.local_addr().expect("local addr");
        std::thread::spawn(move || {
            fn respond(mut stream: std::net::TcpStream, status_line: &str, content_type: &str) {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let response = format!(
                    "HTTP/1.1 {status_line}\r\nContent-Type: {content_type}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
                );
                let _ = stream.write_all(response.as_bytes());
            }
            // 1: the reconnect's own attach succeeds — the tail itself
            // carries nothing.
            if let Ok((stream, _)) = listener.accept() {
                respond(stream, "200 OK", "text/event-stream");
            }
            // 2: the refresh's first call (system) fails.
            if let Ok((stream, _)) = listener.accept() {
                respond(stream, "500 Internal Server Error", "application/json");
            }
        });

        let client = ApiClient::new(&format!("http://{addr}"), "unused-token").expect("client");
        let mut app = App::new();
        app.live = Live::Reconnecting;
        app.system = serde_json::json!({"version": "pre-gap"});
        let mut backoff = Backoff::new();
        // Advance the backoff as a real outage would before this attempt, so
        // a wrongful reset back to the base delay is observable below.
        backoff.next_delay();
        backoff.next_delay();

        let attempt = try_attach(&client, 0).await;
        let stream = reconnected(&client, &mut app, &mut backoff, attempt).await;

        assert!(
            stream.is_none(),
            "a refresh failure must not hand back a stream to keep tailing"
        );
        assert_eq!(
            app.live,
            Live::Reconnecting,
            "a stream that opened but whose refresh failed must not be reported \
             Attached — the screen would show the pre-gap snapshot as live"
        );
        assert_eq!(
            app.system["version"], "pre-gap",
            "a failed refresh must not leave a half-updated system snapshot"
        );
        assert_eq!(
            backoff.next_delay(),
            RECONNECT_BASE * 4,
            "the backoff must not reset on a refresh failure — it continues \
             from where the outage's curve already was"
        );
    }
}
