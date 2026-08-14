//! `sgt watch` (`docs/gauntlet/contracts/WATCH.md`; proposal
//! `reference/proposal-sgt-watch-v1.md`): a read-only client-side
//! subscription over the existing SSE + Work inspection surfaces — the
//! harness's return path after `sgt run`, replacing `sgt work show` polling
//! with one blocking call that reports the next attention/terminal Work
//! transition.
//!
//! **Import boundary (WATCH.md §16.3 / proposal §13.2).** This module reaches
//! the crate only through [`crate::api`] — `ApiClient`, `Event`,
//! `EventStream`, `MalformedFrame` — plus `serde`/`serde_json`. No journal,
//! projection, engine, backend, or daemon internals: a long-running watch
//! loop must stay a client, never grow into a second execution layer.
//! `tests/m9_watch.rs`'s structural test pins this by source scan, mirroring
//! `tests/m6_surfaces.rs`'s `t5`/`t5b` for the TUI.
//!
//! **Event doctrine (proposal §7, Decision WATCH-05).** An event is
//! provenance, not meaning: it identifies that something changed and causes
//! a re-read of `GET /v1/work/{id}`, exactly the TUI's own
//! event-as-invalidation pattern (`src/tui.rs`). The watch loop therefore
//! never classifies a Work off an event's `kind` or payload — only off the
//! freshly-read Work snapshot's own `state` — so a stale queued event can
//! never produce a stale notice (proposal §17's falsifier; W4 pins it).

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;

use crate::api::{ApiClient, ClientError, MalformedFrame};

/// `sergeant.watch/v1` (proposal §9.1/§9.2).
pub const WATCH_SCHEMA: &str = "sergeant.watch/v1";

/// R-WATCH-6's test-only rendezvous env var: `SGT_WATCH_TEST_HOLD=<path>`.
/// Unset (production, always) this is a zero-cost no-op — see
/// [`test_hold_rendezvous`].
pub const WATCH_TEST_HOLD_ENV: &str = "SGT_WATCH_TEST_HOLD";

/// R-WATCH-6's dead-man bound on the test hold: a failed test must never
/// orphan a blocked child process.
const WATCH_TEST_HOLD_DEADMAN: Duration = Duration::from_secs(60);

/// Poll interval while parked on the test hold.
const WATCH_TEST_HOLD_POLL: Duration = Duration::from_millis(20);

// ---------------------------------------------------------------------------
// R-WATCH-1: the six-state watch set
// ---------------------------------------------------------------------------

/// R-WATCH-1's watch set: `needs_input`, `blocked`, `waiting`, `failed`,
/// `completed`, `canceled` — every state Sergeant permits an explicit retry
/// or response from, plus the two terminal states. Excludes `pending` and
/// `active`, the two states a Work sits in while ordinary execution is
/// simply proceeding and nothing has stopped for attention.
///
/// `waiting` joins the set beside the proposal's original five (§6.1) per
/// R-WATCH-1: `begin_retry` gates on `Failed | Blocked | Waiting`
/// (`src/runtime/engine.rs`), `BackendSignal::Waiting` routes to
/// `Next::Parked` exactly as `Blocked` does, and nothing auto-resumes it — a
/// Work parked `waiting` would otherwise sit invisible to this verb, which
/// is the polling `sgt watch` exists to kill.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchState {
    /// Blocked on a human response.
    NeedsInput,
    /// Blocked on a dependency or policy gate.
    Blocked,
    /// Parked on an external condition (R-WATCH-1).
    Waiting,
    /// The stage failed; an explicit retry is possible.
    Failed,
    /// Terminal: the Work finished.
    Completed,
    /// Terminal: a client canceled the Work.
    Canceled,
}

impl WatchState {
    /// Classify a Work's `state` string, as `GET /v1/work/{id}` spells it,
    /// into the watch set — or `None` for `pending`/`active` (the two states
    /// this verb never emits for) and for anything this version does not
    /// recognize. An unrecognized state fails closed into "not watched"
    /// rather than guessing.
    pub fn classify(state: &str) -> Option<Self> {
        match state {
            "needs_input" => Some(Self::NeedsInput),
            "blocked" => Some(Self::Blocked),
            "waiting" => Some(Self::Waiting),
            "failed" => Some(Self::Failed),
            "completed" => Some(Self::Completed),
            "canceled" => Some(Self::Canceled),
            _ => None,
        }
    }

    /// Whether a scoped `--follow` watcher exits on reaching this state
    /// (proposal §6.4): only `completed`/`canceled`. R-WATCH-8's abandoned-
    /// failure row is the direct consequence of this being narrower than the
    /// full watch set — a Work that fails and is never retried or canceled
    /// leaves a `--follow` watcher attached indefinitely *after* it has
    /// already emitted the `failed` notice.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Canceled)
    }
}

// ---------------------------------------------------------------------------
// §9.2: the notice wrapper
// ---------------------------------------------------------------------------

/// `sgt watch`'s two notice reasons (§9.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WatchReason {
    /// An event triggered a re-read that now matches the watch set.
    StateTransition,
    /// A scoped watch's initial check found the Work already matching
    /// (proposal §6.5): no event caused this notice.
    CurrentState,
}

/// §9.4's trigger minimization: stable event-envelope provenance only —
/// never the arbitrary event payload, so a raw backend/tool/transcript
/// fragment never enters a harness's context merely because it happened to
/// cause the read.
#[derive(Debug, Clone, Serialize)]
pub struct WatchTrigger {
    /// Journal sequence number.
    pub seq: u64,
    /// Event id.
    pub id: String,
    /// Event timestamp (RFC3339 UTC).
    pub timestamp: String,
    /// Dotted event kind, e.g. `work.completed`.
    pub kind: String,
}

/// One `sergeant.watch/v1` notice (§9.2). `snapshot` is the complete,
/// verbatim `GET /v1/work/{id}` body (R-WATCH-9) — this module never invents
/// a reduced parallel Work schema; it only decides *when* to hand one back.
#[derive(Debug, Clone, Serialize)]
pub struct WatchNotice {
    /// Always [`WATCH_SCHEMA`].
    pub schema: &'static str,
    /// Why this notice was emitted.
    pub reason: WatchReason,
    /// The event that triggered a re-read, or `null` for `current_state`.
    pub trigger: Option<WatchTrigger>,
    /// The complete current `GET /v1/work/{id}` body, forwarded verbatim.
    pub snapshot: Value,
}

// ---------------------------------------------------------------------------
// R-WATCH-2: the fingerprint
// ---------------------------------------------------------------------------

/// R-WATCH-2's fingerprint: `{state, stage_id, attempt, stage_status,
/// detail_identity}`, where `detail_identity` hashes the snapshot's
/// `stage.detail` field (`src/api.rs`'s `work_view`, sourced from
/// `StageRecord.detail`) — the one field every attention-bearing state
/// populates and the projection overwrites on each status change *within*
/// an attempt.
///
/// Without `detail_identity`, a needs_input → respond → needs_input cycle
/// inside one stage attempt changes none of the other four fields
/// (`begin_input` reuses `stage_id`/index/attempt; attempt increments only
/// on retry) — a second, different question in the same attempt would be
/// silently swallowed as a duplicate. This is the proposal's own §17
/// falsifier, closed by including `detail_identity`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Fingerprint {
    state: String,
    stage_id: Option<String>,
    attempt: Option<u64>,
    stage_status: Option<String>,
    detail_identity: u64,
}

impl Fingerprint {
    fn from_snapshot(snapshot: &Value) -> Self {
        let state = snapshot["work"]["state"].as_str().unwrap_or("").to_string();
        let stage = &snapshot["stage"];
        Self {
            state,
            stage_id: stage["stage_id"].as_str().map(str::to_string),
            attempt: stage["attempt"].as_u64(),
            stage_status: stage["status"].as_str().map(str::to_string),
            detail_identity: hash_detail(&stage["detail"]),
        }
    }
}

/// Hash `stage.detail` for [`Fingerprint`]. `detail` is always either a JSON
/// string or `null` (`StageRecord.detail: Option<String>`), so `null` and
/// the empty string collapse to the same hash — "empty/None (completed,
/// canceled) hashes as empty" (R-WATCH-2).
fn hash_detail(detail: &Value) -> u64 {
    let text = detail.as_str().unwrap_or("");
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

// ---------------------------------------------------------------------------
// §9.5: human rendering
// ---------------------------------------------------------------------------

/// One concise human-readable line for a notice (§9.5): work id, state,
/// stage coordinate, then the stage's own detail (the question or blocking
/// reason) when there is one, falling back to the work's intent — which is
/// what a `completed`/`canceled` notice has, since `stage.detail` is empty by
/// then (R-WATCH-2). Embedded whitespace is normalized and the result is
/// always exactly one physical line: no raw event payload, no progress or
/// heartbeat text (§9.5's own rules; §10 bars anything else from stdout while
/// waiting).
pub fn render_human(notice: &WatchNotice) -> String {
    let work = &notice.snapshot["work"];
    let id = work["id"].as_str().unwrap_or("-");
    let state = work["state"].as_str().unwrap_or("-");
    let stage = &notice.snapshot["stage"];
    let stage_coord = match (stage["stage_id"].as_str(), stage["attempt"].as_u64()) {
        (Some(stage_id), Some(attempt)) => format!("{stage_id}#{attempt}"),
        (Some(stage_id), None) => stage_id.to_string(),
        (None, _) => "-".to_string(),
    };
    let detail = stage["detail"].as_str();
    let intent = work["intent"].as_str();
    let tail = normalize_whitespace(detail.or(intent).unwrap_or(""));
    format!("{id}  {state}  {stage_coord}  {tail}")
}

/// Collapse any run of whitespace (including embedded newlines) to a single
/// space, so a multi-line question or blocking reason still renders as one
/// physical stdout line (§9.5).
fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ---------------------------------------------------------------------------
// The watch loop
// ---------------------------------------------------------------------------

/// `sgt watch [WORK_ID] [--follow]`'s parsed options (proposal §13.1).
#[derive(Debug, Clone)]
pub struct WatchOptions {
    /// The Work to scope to, or `None` for an estate-wide watch.
    pub work_id: Option<String>,
    /// Whether to remain attached after a nonterminal match (§6.4).
    pub follow: bool,
}

/// Why a completed [`watch`] call returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchExit {
    /// One-shot: the single matching notice was emitted and the call
    /// returns without regard to whether that state was terminal.
    Matched,
    /// Scoped `--follow` reached `completed`/`canceled` (§6.4) and exited.
    Terminal,
}

/// Why [`watch`] ended without completing normally.
#[derive(Debug, thiserror::Error)]
pub enum WatchError {
    /// A daemon API call failed (transport or a structured error body —
    /// including the existing `work_not_found` for an unknown scoped id,
    /// proposal §6.5, rendered through this error's `Display`).
    #[error("{0}")]
    Client(#[from] ClientError),
    /// R-WATCH-7: the daemon sent a `data:` frame this client could not
    /// decode. Proposal §11.2: this ends the command with a diagnostic
    /// rather than being silently skipped.
    #[error("malformed event frame from the daemon: {0}")]
    Malformed(#[from] MalformedFrame),
    /// Proposal §11.1: the stream closed (daemon restart/shutdown, a lagged
    /// subscriber dropped, a broken chunk) before a one-shot notice or a
    /// scoped follow's terminal state was reached. Names the last observed
    /// sequence so a caller can judge how much may have been missed; a fresh
    /// `sgt watch` invocation is always safe to re-run (a scoped watch checks
    /// current state first; the journal preserves history).
    #[error("watch stream closed after journal seq {last_seq}; rerun `sgt watch` to resubscribe")]
    StreamClosed {
        /// The last journal sequence actually observed.
        last_seq: u64,
    },
    /// R-WATCH-6: `SGT_WATCH_TEST_HOLD=<path>` was set but `<path>` never
    /// appeared within the 60-second dead-man. This can only happen in a
    /// test process that set the env var and then failed to release the
    /// hold — production never sets it.
    #[error(
        "SGT_WATCH_TEST_HOLD={path:?} never appeared within {secs}s; refusing to hang forever \
         on a test rendezvous that will never arrive",
        secs = WATCH_TEST_HOLD_DEADMAN.as_secs()
    )]
    TestHoldTimedOut {
        /// The path the hold was waiting on.
        path: String,
    },
}

/// Drive the watch loop (proposal §8), calling `on_notice` synchronously for
/// every emitted [`WatchNotice`] — in submission order, one call per notice,
/// before this function's own next await point — so a caller can print each
/// notice the moment it exists rather than buffering the whole run.
///
/// Race-free sequencing (§8.1/§8.2): journal head is read *before* the
/// stream attaches, and the stream attaches *before* any current-state Work
/// read — the existing SSE implementation's subscribe-before-history,
/// sequence-deduplicated design (`src/api.rs`) means a transition landing at
/// any point during that window is neither lost nor double-reported; R-WATCH-
/// 6's test hold instruments exactly this window without changing it.
///
/// A scoped watch (`options.work_id: Some`) checks current state first
/// (§6.5) and emits `current_state` immediately when it already matches. An
/// estate-wide watch (`options.work_id: None`) is edge-triggered (§6.6): it
/// begins at the current journal head and never replays historical
/// completions.
pub async fn watch(
    client: &ApiClient,
    options: &WatchOptions,
    mut on_notice: impl FnMut(&WatchNotice),
) -> Result<WatchExit, WatchError> {
    // 1. Journal head.
    let system = client.system().await?;
    let head = system["journal_head"].as_u64().unwrap_or(0);

    // 2. Attach the stream before any Work read (§8.1 step 2, §8.2 step 2).
    let mut stream = client.stream_events(head).await?;

    // R-WATCH-6's test-only rendezvous — zero-cost no-op unless the test
    // process set SGT_WATCH_TEST_HOLD.
    test_hold_rendezvous().await?;

    let mut last_seq = head;

    // A scoped watch's own fingerprint (one Work only); an estate-wide
    // watch's per-work fingerprints (§8.4: duplicate suppression is scoped
    // per watched Work, not global).
    let mut scoped_fingerprint: Option<Fingerprint> = None;
    let mut fleet_fingerprints: HashMap<String, Fingerprint> = HashMap::new();

    // 3./4. Scoped initial current-state check (§6.5). Unscoped has none —
    // §6.6's edge-triggered rule.
    if let Some(id) = &options.work_id {
        let snapshot = client.work(id).await?;
        if let Some(state) = classify_snapshot(&snapshot) {
            let fingerprint = Fingerprint::from_snapshot(&snapshot);
            emit(&mut on_notice, WatchReason::CurrentState, None, snapshot);
            scoped_fingerprint = Some(fingerprint);
            if !options.follow {
                return Ok(WatchExit::Matched);
            }
            if state.is_terminal() {
                return Ok(WatchExit::Terminal);
            }
        }
    }

    // 5. Consume events after head.
    loop {
        let event = stream
            .next_event()
            .await?
            .ok_or(WatchError::StreamClosed { last_seq })?;
        last_seq = event.seq;

        let Some(work_id) = event.work_id.clone() else {
            continue; // Not tied to any Work (e.g. daemon.started) — nothing to re-read.
        };
        if let Some(scoped) = &options.work_id
            && *scoped != work_id
        {
            continue;
        }

        let snapshot = client.work(&work_id).await?;
        let Some(state) = classify_snapshot(&snapshot) else {
            continue; // Re-read landed on pending/active — not watched.
        };
        let fingerprint = Fingerprint::from_snapshot(&snapshot);

        let is_duplicate = if options.work_id.is_some() {
            scoped_fingerprint.as_ref() == Some(&fingerprint)
        } else {
            fleet_fingerprints.get(&work_id) == Some(&fingerprint)
        };
        if is_duplicate {
            continue;
        }

        let trigger = WatchTrigger {
            seq: event.seq,
            id: event.id.clone(),
            timestamp: event.timestamp.clone(),
            kind: event.kind.clone(),
        };
        emit(
            &mut on_notice,
            WatchReason::StateTransition,
            Some(trigger),
            snapshot,
        );

        if options.work_id.is_some() {
            scoped_fingerprint = Some(fingerprint);
            if !options.follow {
                return Ok(WatchExit::Matched);
            }
            if state.is_terminal() {
                return Ok(WatchExit::Terminal);
            }
        } else {
            fleet_fingerprints.insert(work_id, fingerprint);
            if !options.follow {
                return Ok(WatchExit::Matched);
            }
            // §6.4: an estate-wide watcher never exits on any one Work's
            // terminal state — it stays attached for every Work in the
            // estate until interrupted, the stream closes, or an
            // unrecoverable error occurs.
        }
    }
}

fn classify_snapshot(snapshot: &Value) -> Option<WatchState> {
    WatchState::classify(snapshot["work"]["state"].as_str().unwrap_or(""))
}

fn emit(
    on_notice: &mut impl FnMut(&WatchNotice),
    reason: WatchReason,
    trigger: Option<WatchTrigger>,
    snapshot: Value,
) {
    on_notice(&WatchNotice {
        schema: WATCH_SCHEMA,
        reason,
        trigger,
        snapshot,
    });
}

/// R-WATCH-6's seam: when `SGT_WATCH_TEST_HOLD=<path>` is set, touch
/// `<path>.ready` (proving to a waiting test that the client has read the
/// journal head and attached the stream), then block until `<path>` exists —
/// bounded by [`WATCH_TEST_HOLD_DEADMAN`], so a test that forgets to release
/// the hold fails this process instead of orphaning it. Unset — every
/// production invocation — this reads one environment variable and returns
/// immediately: zero-cost.
async fn test_hold_rendezvous() -> Result<(), WatchError> {
    let Ok(path) = std::env::var(WATCH_TEST_HOLD_ENV) else {
        return Ok(());
    };
    test_hold_wait(path, WATCH_TEST_HOLD_DEADMAN, WATCH_TEST_HOLD_POLL).await
}

/// The waiting half of [`test_hold_rendezvous`], with the dead-man and poll
/// durations taken as parameters rather than the production constants
/// directly — so a unit test can exercise the dead-man's timeout branch
/// (`WatchError::TestHoldTimedOut`) in milliseconds instead of the real
/// [`WATCH_TEST_HOLD_DEADMAN`] (60s). `test_hold_rendezvous` is the only
/// production caller and always passes the real constants; production
/// behavior is unchanged by this split.
async fn test_hold_wait(path: String, deadman: Duration, poll: Duration) -> Result<(), WatchError> {
    let ready = format!("{path}.ready");
    // Best-effort: a write failure here is a broken test harness, not a
    // production concern (the env var is never set in production), so it
    // falls through to the same bounded wait/timeout rather than a distinct
    // error variant.
    let _ = std::fs::write(&ready, b"");
    let deadline = Instant::now() + deadman;
    loop {
        if std::path::Path::new(&path).exists() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(WatchError::TestHoldTimedOut { path });
        }
        tokio::time::sleep(poll).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn snapshot(state: &str, stage_id: &str, attempt: u64, detail: Option<&str>) -> Value {
        json!({
            "work": {"id": "01WORK", "state": state, "intent": "fix the settlement retry bug"},
            "stage": {
                "stage_id": stage_id,
                "index": 0,
                "attempt": attempt,
                "status": state,
                "detail": detail,
                "of": 2,
            },
            "surface": null,
            "execution": null,
            "reservation": null,
            "workflow": null,
            "backend": "fake",
            "route_source": null,
            "teardown": null,
            "output": null,
            "envelope": {"turns_spawned": 1, "turn_cap": 12, "turn_cap_bonus": 0, "turn_ceiling_secs": 900.0},
        })
    }

    // ---- R-WATCH-1: classification -----------------------------------

    #[test]
    fn classify_covers_exactly_the_six_watch_states() {
        for (state, expected) in [
            ("needs_input", Some(WatchState::NeedsInput)),
            ("blocked", Some(WatchState::Blocked)),
            ("waiting", Some(WatchState::Waiting)),
            ("failed", Some(WatchState::Failed)),
            ("completed", Some(WatchState::Completed)),
            ("canceled", Some(WatchState::Canceled)),
            ("pending", None),
            ("active", None),
            ("some-future-state", None),
        ] {
            assert_eq!(
                WatchState::classify(state),
                expected,
                "state {state:?} must classify to {expected:?}"
            );
        }
    }

    #[test]
    fn only_completed_and_canceled_are_terminal_for_scoped_follow() {
        for (state, terminal) in [
            (WatchState::NeedsInput, false),
            (WatchState::Blocked, false),
            (WatchState::Waiting, false),
            (WatchState::Failed, false),
            (WatchState::Completed, true),
            (WatchState::Canceled, true),
        ] {
            assert_eq!(
                state.is_terminal(),
                terminal,
                "{state:?}.is_terminal() must be {terminal} (R-WATCH-8's abandoned-failure row \
                 depends on failed staying non-terminal)"
            );
        }
    }

    // ---- R-WATCH-2: fingerprint ---------------------------------------

    #[test]
    fn two_different_questions_in_the_same_attempt_fingerprint_differently() {
        let first = snapshot("needs_input", "20-review", 1, Some("question A?"));
        let second = snapshot("needs_input", "20-review", 1, Some("question B?"));
        assert_ne!(
            Fingerprint::from_snapshot(&first),
            Fingerprint::from_snapshot(&second),
            "R-WATCH-2's own falsifier: two different questions inside one stage attempt \
             change none of {{state, stage_id, attempt, stage_status}} — only detail_identity \
             tells them apart"
        );
    }

    #[test]
    fn the_same_snapshot_fingerprints_identically() {
        let a = snapshot("needs_input", "20-review", 1, Some("question A?"));
        let b = snapshot("needs_input", "20-review", 1, Some("question A?"));
        assert_eq!(
            Fingerprint::from_snapshot(&a),
            Fingerprint::from_snapshot(&b),
            "a re-triggered but materially identical snapshot must fingerprint identically, \
             so queued duplicate events collapse to one notice"
        );
    }

    #[test]
    fn empty_detail_hashes_the_same_whether_null_or_absent_string() {
        let null_detail = snapshot("completed", "30-close", 1, None);
        assert_eq!(
            hash_detail(&null_detail["stage"]["detail"]),
            hash_detail(&Value::String(String::new())),
            "R-WATCH-2: empty/None (completed, canceled) hashes as empty"
        );
    }

    // ---- §9.5: human rendering ------------------------------------------

    #[test]
    fn human_line_prefers_stage_detail_over_intent() {
        let notice = WatchNotice {
            schema: WATCH_SCHEMA,
            reason: WatchReason::StateTransition,
            trigger: None,
            snapshot: snapshot(
                "needs_input",
                "20-review",
                1,
                Some("which retry policy should govern this adapter?"),
            ),
        };
        let line = render_human(&notice);
        assert_eq!(
            line,
            "01WORK  needs_input  20-review#1  which retry policy should govern this adapter?"
        );
        assert_eq!(
            line.lines().count(),
            1,
            "must be one physical line: {line:?}"
        );
    }

    #[test]
    fn human_line_falls_back_to_intent_when_detail_is_empty() {
        let notice = WatchNotice {
            schema: WATCH_SCHEMA,
            reason: WatchReason::StateTransition,
            trigger: None,
            snapshot: snapshot("completed", "30-close", 1, None),
        };
        assert_eq!(
            render_human(&notice),
            "01WORK  completed  30-close#1  fix the settlement retry bug"
        );
    }

    #[test]
    fn human_line_normalizes_embedded_whitespace() {
        let notice = WatchNotice {
            schema: WATCH_SCHEMA,
            reason: WatchReason::StateTransition,
            trigger: None,
            snapshot: snapshot(
                "blocked",
                "10-implement",
                2,
                Some("dependency contract\ncould   not be reconciled"),
            ),
        };
        let line = render_human(&notice);
        assert_eq!(
            line,
            "01WORK  blocked  10-implement#2  dependency contract could not be reconciled"
        );
        assert_eq!(line.lines().count(), 1);
    }

    // ---- §9.1/§9.2: JSON shape -------------------------------------------

    #[test]
    fn notice_json_shape_matches_the_contract_and_excludes_the_raw_payload() {
        let notice = WatchNotice {
            schema: WATCH_SCHEMA,
            reason: WatchReason::StateTransition,
            trigger: Some(WatchTrigger {
                seq: 1842,
                id: "01EVT".to_string(),
                timestamp: "2026-08-13T18:42:31.417Z".to_string(),
                kind: "work.completed".to_string(),
            }),
            snapshot: snapshot("completed", "30-close", 1, None),
        };
        let value = serde_json::to_value(&notice).expect("serialize");
        assert_eq!(value["schema"], WATCH_SCHEMA);
        assert_eq!(value["reason"], "state_transition");
        assert_eq!(value["trigger"]["seq"], 1842);
        assert_eq!(value["trigger"]["kind"], "work.completed");
        // §9.4: only the four provenance fields — never a `payload` key, the
        // arbitrary event body this wrapper deliberately excludes.
        assert!(
            value["trigger"].get("payload").is_none(),
            "the trigger must never carry the raw event payload: {value}"
        );
        let trigger_keys: std::collections::BTreeSet<&str> = value["trigger"]
            .as_object()
            .expect("trigger object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            trigger_keys,
            ["id", "kind", "seq", "timestamp"].into_iter().collect(),
            "trigger must be exactly §9.4's four fields"
        );
        assert_eq!(value["snapshot"]["work"]["state"], "completed");

        let current_state = WatchNotice {
            schema: WATCH_SCHEMA,
            reason: WatchReason::CurrentState,
            trigger: None,
            snapshot: snapshot("completed", "30-close", 1, None),
        };
        let value = serde_json::to_value(&current_state).expect("serialize");
        assert_eq!(value["reason"], "current_state");
        assert_eq!(
            value["trigger"],
            Value::Null,
            "an already-matching scoped Work's notice carries a null trigger (§9.2)"
        );
    }

    // ---- R-WATCH-6: the test-hold dead-man ------------------------------

    #[tokio::test]
    async fn test_hold_wait_times_out_with_a_distinct_error_when_the_release_path_never_appears() {
        // A path under the process-unique temp dir that this test never
        // creates, so the hold can never be released — proving the dead-man
        // branch (contract-fidelity:F2) rather than the happy path W3/W4
        // already pin (wait_for_hold_ready followed by a prompt release).
        let path = std::env::temp_dir()
            .join(format!(
                "sgt-watch-test-hold-never-released-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ))
            .to_string_lossy()
            .into_owned();

        let result = test_hold_wait(
            path.clone(),
            Duration::from_millis(60),
            Duration::from_millis(5),
        )
        .await;

        match result {
            Err(WatchError::TestHoldTimedOut {
                path: timed_out_path,
            }) => {
                assert_eq!(
                    timed_out_path, path,
                    "the timeout error must name the exact path the hold was waiting on"
                );
            }
            other => panic!(
                "a hold whose release path never appears must return \
                 Err(WatchError::TestHoldTimedOut), not {other:?}"
            ),
        }
    }

    #[tokio::test]
    async fn test_hold_wait_returns_ok_once_the_release_path_appears() {
        let path = std::env::temp_dir()
            .join(format!(
                "sgt-watch-test-hold-released-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ))
            .to_string_lossy()
            .into_owned();
        // Release the hold from a concurrent task shortly after the wait
        // starts, well inside the dead-man — the mirror image of the
        // timeout test above, proving the non-error branch still returns
        // Ok(()) through the same parameterized helper.
        let release_path = path.clone();
        let releaser = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(15)).await;
            let _ = std::fs::write(&release_path, b"");
        });

        let result = test_hold_wait(
            path.clone(),
            Duration::from_secs(5),
            Duration::from_millis(5),
        )
        .await;

        releaser.await.expect("releaser task must not panic");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(format!("{path}.ready"));

        assert!(
            result.is_ok(),
            "the hold must resolve Ok(()) once the release path appears: {result:?}"
        );
    }

    #[test]
    fn one_shot_json_notice_serializes_as_one_compact_line() {
        let notice = WatchNotice {
            schema: WATCH_SCHEMA,
            reason: WatchReason::CurrentState,
            trigger: None,
            snapshot: snapshot("completed", "30-close", 1, None),
        };
        let line = serde_json::to_string(&notice).expect("serialize");
        assert_eq!(
            line.lines().count(),
            1,
            "must be one physical line: {line:?}"
        );
        // Round-trips as one independently-parseable JSON object (§9.1/W8).
        let _: Value = serde_json::from_str(&line).expect("must parse as one JSON object");
    }
}
