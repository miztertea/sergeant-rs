//! v1 HTTP/JSON + SSE API (proposal §7–§8 subset, §26 idempotency).
//!
//! Handlers never mutate state directly: every mutation appends events to the
//! journal (via [`Core::commit`]) and all reads come from the registry
//! projection. Every mutation carries a client-supplied ULID `command_id`;
//! the outcome is journaled (`command.accepted` / `command.rejected`) and a
//! repeated `command_id` replays the recorded outcome byte-identically
//! without re-executing — including across a daemon restart.
//!
//! All `/v1/*` routes require the bearer token from the runtime descriptor;
//! `/healthz` is unauthenticated. Errors are structured JSON:
//! `{"error": {"code": "...", "message": "..."}}`.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::extract::{Path, Query, Request, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::{broadcast, mpsc, watch};
use tokio_stream::wrappers::ReceiverStream;

use crate::backend::Deferred;
use crate::cli::doctor;
use crate::daemon::{
    KIND_ADMISSION_PAUSED, KIND_ADMISSION_RESUMED, KIND_BACKEND_PROBED, KIND_DAEMON_STARTED,
    KIND_DAEMON_STOPPED,
};
use crate::domain::event::{Event, EventDraft, EventSource, rfc3339_utc_now};
use crate::domain::execution::{
    KIND_EXECUTION_ABANDONED, KIND_EXECUTION_RECONCILED, KIND_EXECUTION_RESERVED,
    KIND_EXECUTION_STARTED, KIND_EXECUTION_STOPPED,
};
use crate::domain::manifest::{self, ManifestError};
use crate::domain::work::{
    EnvelopeRequest, IntentDetail, KIND_COMMAND_ACCEPTED, KIND_COMMAND_REJECTED, KIND_WORK_BLOCKED,
    KIND_WORK_CANCELED, KIND_WORK_COMPLETED, KIND_WORK_FAILED, KIND_WORK_NEEDS_INPUT,
    KIND_WORK_RESUMED, KIND_WORK_STARTED, KIND_WORK_SUBMITTED, KIND_WORK_WAITING, Work, WorkState,
};
use crate::domain::workflow::{
    self, KIND_STAGE_BLOCKED, KIND_STAGE_CANCELED, KIND_STAGE_COMPLETED, KIND_STAGE_ENTERED,
    KIND_STAGE_FAILED, KIND_STAGE_INPUT_RECEIVED, KIND_STAGE_NEEDS_INPUT, KIND_STAGE_RESUMED,
    KIND_STAGE_WAITING, KIND_WORKFLOW_BOUND, WorkflowDefinition,
};
use crate::domain::workspace::{InstructionPolicy, Workspace};
use crate::runtime::analytics::{Analytics, AnalyticsError, CANNED_QUERIES};
use crate::runtime::engine::{
    Engine, EngineError, KIND_TURN_CEILING_INTERRUPTED, KIND_TURN_ENVELOPE_EXTENDED,
    Next as EngineNext, Step, SubmitContext,
};
use crate::runtime::graph::{
    KIND_CONVERSATION_ASK, KIND_CONVERSATION_ASSISTANT_COMPLETED, KIND_CONVERSATION_TURN_ENDED,
    KIND_CONVERSATION_USER, KIND_TOOL_COMPLETED, KIND_TOOL_REQUESTED, KIND_USAGE_UPDATED,
};
use crate::runtime::integrity::IntegrityDisposition;
use crate::runtime::journal::{Journal, JournalError};
use crate::runtime::projection::{
    Projection, ProjectionError, WorkRegistry, WorkRun, is_absorbing, rederive_run,
};
use crate::runtime::surface::{
    BindingDisposition, KIND_SURFACE_MATERIALIZED, KIND_SURFACE_MATERIALIZING,
    KIND_SURFACE_TORN_DOWN, reap, retained_bindings,
};

/// API revision served by this build (`GET /v1/system`, runtime descriptor).
pub const API_REVISION: &str = "v1";

/// SSE keep-alive cadence. Contract Unknown resolved by fiat: 15 s comment
/// frames; nothing depends on the value yet, it only keeps idle proxies and
/// clients from timing out the stream.
pub const SSE_KEEP_ALIVE: Duration = Duration::from_secs(15);

/// The daemon's mutable heart: the journal plus the registry projection.
/// One instance, behind one lock — the single-owner architecture in a struct.
#[derive(Debug)]
pub struct Core {
    /// The single journal writer.
    pub journal: Journal,
    /// The Work registry folded from the journal.
    pub registry: Projection<WorkRegistry>,
    /// Live event fan-out for SSE subscribers.
    pub events_tx: broadcast::Sender<Event>,
    /// The **open group**: events written and folded during the current lock
    /// hold, awaiting the hold's single fsync (#44).
    ///
    /// Private, and the only private field here, because it is the one piece
    /// of `Core` whose invariant a caller could break: an event sitting in
    /// here is a promise that has not been kept yet. [`Core::flush`] is the
    /// only thing that empties it, and [`CoreGuard`] is what guarantees
    /// `flush` runs.
    open_group: Vec<Event>,
}

/// Failure while committing an event (journal append or projection fold).
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// The journal refused or failed the append.
    #[error(transparent)]
    Journal(#[from] JournalError),
    /// The projection refused the committed event (seq mismatch — a bug).
    #[error(transparent)]
    Projection(#[from] ProjectionError),
}

impl Core {
    /// Assemble the core over an open journal and its folded registry.
    ///
    /// A constructor rather than a struct literal because the open group must
    /// start empty and stay the module's business — see [`Core::open_group`].
    pub fn new(
        journal: Journal,
        registry: Projection<WorkRegistry>,
        events_tx: broadcast::Sender<Event>,
    ) -> Self {
        Self {
            journal,
            registry,
            events_tx,
            open_group: Vec::new(),
        }
    }

    /// Append one event to the journal, fold it into the registry, and add it
    /// to the lock hold's open group. The only mutation path.
    ///
    /// **What "committed" means here (#44).** The line is written and the
    /// projections have moved; the fsync that makes it survive a crash, and
    /// the fan-out that lets anyone else find out about it, both belong to
    /// [`Core::flush`] at the end of this lock hold. Nothing outside the core
    /// can observe the difference, because nothing outside the core runs
    /// until the hold ends — which is exactly why the group boundary is drawn
    /// there and not anywhere cheaper. See [`CoreGuard`].
    pub fn commit(&mut self, draft: EventDraft) -> Result<Event, CoreError> {
        let event = self.journal.append(draft)?;
        self.registry.apply(&event)?;
        self.open_group.push(event.clone());
        Ok(event)
    }

    /// Close the open group: one fsync for everything this lock hold
    /// appended, then fan those events out to live SSE subscribers.
    ///
    /// Ordering is the point. The fsync comes first, so **no subscriber, no
    /// HTTP response and no external effect can ever be caused by an event
    /// that a crash would take back** — the property that lets the group
    /// boundary be invisible from outside. A failed fsync therefore publishes
    /// nothing to `events_tx` at all: the journal poisons itself (see
    /// [`Journal::sync`]), the error goes to the caller, and the group is
    /// dropped unannounced on the broadcast channel rather than announced
    /// unrecoverably.
    ///
    /// **Scoped to the broadcast, not the whole API surface (round-2 finding
    /// INV-R2-06).** By the time `sync` fails, `write_all` already put the
    /// lines on disk and [`Core::commit`] already folded them into
    /// `registry` — poisoning happens *after* both. So a failed group still
    /// leaves its events visible to every read-only path: `events_after`
    /// (`GET /v1/events`, the SSE history/refill and the analytics catch-up)
    /// and every projection-backed endpoint (`show_work` and friends) will
    /// serve them on the next hold, from a journal that has since refused
    /// further appends. "Unannounced" describes the live SSE push only.
    ///
    /// Free when the group is empty, so a read-only hold costs nothing —
    /// but `sync` is still consulted, never short-circuited: a poisoned
    /// journal must surface from *every* flush, or a caller that flushes an
    /// empty group as a durability backstop (`daemon::start_with`'s
    /// pre-publish `core.flush()?`) would read `Ok` over a handle that
    /// refuses all further appends.
    pub fn flush(&mut self) -> Result<(), CoreError> {
        let synced = self.journal.sync();
        // Taken unconditionally: a group that failed to sync is not retried
        // on the next hold, it is abandoned along with the poisoned handle.
        let group = std::mem::take(&mut self.open_group);
        synced?;
        for event in group {
            // No live subscriber is not an error.
            let _ = self.events_tx.send(event);
        }
        Ok(())
    }

    /// How many events the current lock hold has appended and not yet
    /// published. Zero everywhere outside a hold that is mid-commit.
    pub fn open_group_len(&self) -> usize {
        self.open_group.len()
    }

    /// Every journaled event with `seq > after`, in seq order.
    ///
    /// Every caller of this runs while holding the core's lock — the
    /// daemon's single mutation choke point — so its cost is a cost paid by
    /// `submit`/`cancel`/`input`. It is therefore bounded by the *answer*
    /// and not by history: nothing committed past `after` is an in-memory
    /// comparison against the journal head, and otherwise
    /// [`Journal::replay_after`] skips whole segments instead of parsing the
    /// whole chain. The steady state of the analytical projection's
    /// read-time catch-up — already current, nothing to fold — is the first
    /// branch, and costs no I/O at all.
    pub fn events_after(&self, after: u64) -> Result<Vec<Event>, JournalError> {
        if after.saturating_add(1) >= self.journal.next_seq() {
            return Ok(Vec::new());
        }
        let mut events = Vec::new();
        for event in self.journal.replay_after(after)? {
            let event = event?;
            if event.seq > after {
                events.push(event);
            }
        }
        Ok(events)
    }
}

/// The only way to hold the core lock — and therefore the group-commit
/// boundary (#44).
///
/// A hold ends here: whatever the hold appended is fsynced once, in one
/// syscall, and only then published. Every other way of taking the mutex is
/// banned by `m6`'s
/// `t11c_every_core_lock_hold_ends_in_the_group_commit_guard`, because the
/// boundary is only worth anything if there is no second door.
///
/// # Why the boundary is here and not per append
///
/// A submit's first hold appends two events (`work.submitted`,
/// `surface.materializing`); the settle hold that follows appends five more
/// (`surface.materialized`, `workflow.bound`, `work.started`,
/// `stage.entered`, `execution.reserved`) — several holds per work, not one,
/// and the two-phase boundary added two of those events per work
/// (`docs/perf/n3-two-phase-boundary-2026-08-10.md`); at one fsync each, on a
/// single-writer path, that volume *is* the cost. Sharing the fsync is legal
/// because **almost nothing outside the core runs during a hold**: the mutex
/// is held, no response has been rendered, no SSE frame sent, and §22.6
/// forbids performing an external effect under the guard for every path but
/// one. The one exception is `backend.stop()` — a reviewed, pre-existing
/// exemption (issue #14/B3), not a gap this change opened; see
/// `runtime::engine::Engine::stop_execution` and `settle_launch` for what it
/// costs group commit specifically (a kill can now precede the durability of
/// the event that records it) and `m6`'s `t11_external_effects_live_only_in_the_out_of_lock_performers`
/// for where it is named. For every other event, the set of things that
/// could have been caused by it is empty until the hold ends — and at the
/// instant the hold ends, every one of its events is durable, exactly as it
/// was before this change.
///
/// # L6 adjacent-append analysis (the mandatory one)
///
/// L6's hazard is a crash between two causally-linked appends. Group commit
/// makes that gap cheaper, so it must not make it *newer*. It does not, and
/// the argument is about reachable states rather than about probabilities:
///
/// - **Before.** Per-append fsync guarantees a floor (everything acknowledged
///   is durable) but no ceiling — the OS is free to have written more. After
///   a crash the segment holds *some byte prefix* of what was written, with a
///   possibly torn final line. Crashing between `stage.entered` and
///   `execution.reserved` was always reachable; that is why §22.5 enumerates
///   both sides of that pair as separate injection windows.
/// - **After.** A crash mid-group, or after the group's last append and
///   before its fsync, leaves… *some byte prefix* of what was written, with a
///   possibly torn final line. The floor drops to the previous group
///   boundary; the **set of reachable on-disk states is unchanged**, because
///   it was already "any byte prefix".
/// - **Therefore** no recovery obligation is new. Every prefix a grouped
///   crash can produce is one a per-append crash could already produce, and
///   the existing machinery is what handles it: [`Journal::open`]'s
///   `recover_tail` quarantines the torn final line and truncates to the last
///   complete one, replay validates seq continuity, the projection is rebuilt
///   from what survived, and `runtime::recovery` fails closed on the
///   resulting ambiguity. What changed is the *distribution* over those
///   states, not the set — and a distribution is not something recovery is
///   allowed to depend on.
///
/// Proven, not asserted: `m4`'s
/// `n32_every_prefix_a_grouped_lock_hold_can_crash_at_is_one_recovery_already_handles`
/// writes a hand-built, worst-case-sized group of six events — at least as
/// large as any single hold group commit produces today, though not the
/// literal contents of one (see that test's module comment) — never closes
/// the group, and then truncates that journal at **every** byte offset a crash
/// could stop at — each one must reopen, replay to an exact prefix, rebuild
/// and reconcile fail-closed. `m4`'s `n10`–`n12` remain the §22.5 windows
/// themselves, unchanged and still passing, because the events and their
/// order are untouched — and n32's final case is byte-for-byte n11's. No
/// compound event was introduced, no event was removed, and no crash window
/// was deleted — A-N3-1's rejection of the compound-event alternative is
/// undisturbed.
///
/// # Failure
///
/// `Drop` cannot report, so the reportable path is [`CoreGuard::flush`],
/// which handlers call explicitly where an error can still become a response.
/// `Drop` is the backstop for every other hold: it flushes, and on failure
/// logs against a journal that has already poisoned itself, so the daemon
/// refuses further appends rather than continuing over an unknown durability
/// state.
pub struct CoreGuard<'a> {
    inner: tokio::sync::MutexGuard<'a, Core>,
}

impl<'a> CoreGuard<'a> {
    /// Take the core lock (async — the request path).
    pub async fn acquire(core: &'a tokio::sync::Mutex<Core>) -> Self {
        Self {
            inner: core.lock().await,
        }
    }

    /// Take the core lock from a plain thread.
    ///
    /// Only the adapter's normalized-event committer thread is entitled to
    /// this: it is a std thread by construction (see `daemon::journaling_sink`
    /// for why), which is what makes `blocking_lock` correct there and a
    /// runtime-panicking bug anywhere else.
    pub fn acquire_blocking(core: &'a tokio::sync::Mutex<Core>) -> Self {
        Self {
            inner: core.blocking_lock(),
        }
    }

    /// Close the group early, with the failure reportable.
    ///
    /// Idempotent: the group is empty afterwards, so the `Drop` backstop finds
    /// nothing to do. Call this at the end of a hold that appended something
    /// and can still turn an error into a response.
    pub fn flush(&mut self) -> Result<(), CoreError> {
        self.inner.flush()
    }
}

impl std::ops::Deref for CoreGuard<'_> {
    type Target = Core;
    fn deref(&self) -> &Core {
        &self.inner
    }
}

impl std::ops::DerefMut for CoreGuard<'_> {
    fn deref_mut(&mut self) -> &mut Core {
        &mut self.inner
    }
}

impl Drop for CoreGuard<'_> {
    fn drop(&mut self) {
        if let Err(error) = self.inner.flush() {
            // The journal poisoned itself on the way out of `sync`, so this
            // is a report, not a decision: every later append is already
            // refused and the daemon is failing closed.
            tracing::error!(
                %error,
                "the core-lock hold's group commit failed; the journal is \
                 poisoned and further appends are refused"
            );
        }
    }
}

/// Shared state handed to every handler.
#[derive(Clone)]
pub struct ApiState {
    /// The single-owner core, serialized behind one async lock.
    pub core: Arc<tokio::sync::Mutex<Core>>,
    /// Bearer token all `/v1/*` requests must present.
    pub token: String,
    /// Data dir this daemon owns (reported by `/v1/system`).
    pub data_dir: PathBuf,
    /// Flipped to `true` when the daemon starts shutting down. Endpoints
    /// whose response body never ends on its own — the SSE tail — watch this
    /// and finish, because graceful shutdown waits for in-flight responses.
    pub closing: watch::Receiver<bool>,
    /// The workflow engine: backends, routing defaults, and the surfaces dir.
    pub engine: Arc<Engine>,
    /// The disposable DuckDB analytical + graph projection (§21–§23).
    ///
    /// Behind its own lock, not the core's: an analytics query is a read of a
    /// derived file and must never be able to stall a mutation. It is caught
    /// up from the journal at query time (see [`with_analytics`]), so a
    /// failure anywhere in here costs an answer, never a fact.
    pub analytics: Arc<tokio::sync::Mutex<Analytics>>,
}

/// Build the axum router for the full v1 surface.
///
/// The dashboard (`src/web.rs`) that used to be mounted alongside `/v1` here
/// is gone (ADR 0011, D7) — the CLI, TUI and every future client reach state
/// through this router and nothing else.
pub fn router(state: ApiState) -> Router {
    let v1 = Router::new()
        .route("/work", post(submit_work).get(list_work))
        .route("/work/{id}", get(show_work))
        .route("/work/{id}/transcript", get(work_transcript))
        .route("/work/{id}/cancel", post(cancel_work))
        .route("/work/{id}/input", post(work_input))
        .route("/work/{id}/retry", post(work_retry))
        .route("/work/{id}/extend", post(work_extend))
        .route("/work/{id}/reap", post(reap_work))
        .route("/retained", get(list_retained))
        .route(
            "/estate/repos",
            get(estate_list_repos).post(estate_add_repo),
        )
        .route("/estate/repos/{name}", delete(estate_remove_repo))
        .route(
            "/estate/groups",
            get(estate_list_groups).post(estate_add_group),
        )
        .route("/estate/groups/{name}", delete(estate_remove_group))
        .route("/doctor", get(doctor_report))
        .route("/admission/pause", post(pause_admission))
        .route("/graph/work/{id}", get(work_graph))
        .route("/analytics", get(analytics_index))
        .route("/analytics/{name}", get(analytics_query))
        .route("/events", get(event_history))
        .route("/events/stream", get(event_stream))
        .route("/system", get(system_info))
        .route("/workflows", get(list_workflows))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_bearer,
        ));
    Router::new()
        .route("/healthz", get(healthz))
        .nest("/v1", v1)
        // Errors are structured JSON — including the router's own: unknown
        // routes and known routes hit with the wrong method must not answer
        // with axum's stock empty bodies.
        .fallback(not_found)
        .method_not_allowed_fallback(method_not_allowed)
        .with_state(state)
}

/// Run one blocking closure without occupying an async worker.
///
/// `block_in_place` rather than `spawn_blocking` (R1: the cheaper primitive
/// that already does the job). Both keep the effect off the async worker;
/// `spawn_blocking` additionally moves the value to another thread and back,
/// which costs two task hops per launch — measured at burst 50, that showed up
/// as a throughput regression against R-N0-4's floor. `block_in_place` hands
/// the current worker's other tasks to a replacement thread and runs the
/// closure right here, so the launch costs no migration at all.
///
/// It requires a multi-thread runtime; the single-thread fallback runs the
/// closure inline, which is what a current-thread runtime can do anyway.
async fn blocking<T>(f: impl FnOnce() -> T) -> T {
    blocking_sync(f)
}

/// [`blocking`]'s body, callable from sync code that is already running on
/// an async worker (a handler's view composition, e.g. `resolve_run`'s
/// journal replay): same primitive, same single-thread fallback.
fn blocking_sync<T>(f: impl FnOnce() -> T) -> T {
    let multi_thread = tokio::runtime::Handle::try_current()
        .map(|h| h.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread)
        .unwrap_or(false);
    if multi_thread {
        tokio::task::block_in_place(f)
    } else {
        f()
    }
}

/// Turn the engine's crank to a park, **holding no core lock across any
/// external effect** (§14.2's middle phase, §22.6's budget).
///
/// The handler commits its authoritative half under the guard, drops it, and
/// hands the resulting [`Step`] here. Everything this function blocks on —
/// the launch itself, and any adapter tail work an earlier crank collected
/// (issue #14/B3) — happens with the guard released and on a blocking thread,
/// so another request can take the lock while a harness is spawning or a
/// transcript is flushing. Each settle re-acquires the lock, re-validates the
/// reservation against durable state (§14.5), and releases it again.
///
/// A `settle_launch` failure is logged and the crank stops. It cannot be
/// returned to the client, because the client's own command already
/// succeeded: the reservation is durable and, if the launch got that far, the
/// journal says so. Recovery re-derives from that record; inventing a failure
/// response for a command that was accepted would be worse than the log line.
///
/// It **returns the guard it is still holding** when the crank ends with the
/// lock in hand and nothing outstanding, so the caller can render its response
/// without queueing for the mutex a fourth time. That is not a shortcut around
/// the boundary: the guard is dropped before every effect and before every
/// completion wait, which are the only places §22.6 cares about — in one place
/// each, so there is no arm that can be missed and no arm whose drop is
/// decoration.
async fn crank(state: &ApiState, step: Step) -> Option<CoreGuard<'_>> {
    let mut step = step;
    let mut held: Option<CoreGuard<'_>> = None;
    loop {
        let Step { next, deferred } = step;
        if deferred.is_pending() {
            drop(held.take()); // never wait on an adapter under the guard
            blocking(move || deferred.wait()).await;
        }
        if matches!(next, EngineNext::Parked) {
            return held;
        }
        // Every remaining arm performs an external effect — a harness spawn, a
        // `claude -p --resume` turn, a `git worktree add` fork/exec/wait on a
        // checkout the daemon does not own and cannot bound. So the guard goes
        // first, once, for all of them (§14.2, §22.6).
        //
        // Once, rather than a line per arm: the per-arm drops were three
        // copies of one invariant, and the copy in the SEND arm could not run
        // — `provide_input` drops the guard before calling `crank`, and no
        // settle produces `Next::Send`, so that arm is only ever reached on the
        // first iteration with nothing held. A line that cannot execute is not
        // a boundary, whatever its comment says (round-2 finding N3R2-07).
        // Stated here it executes for every effect, including any later path
        // that does hand a SEND back from a settle.
        drop(held.take());
        let settled = match next {
            EngineNext::Parked => unreachable!("returned above"),
            EngineNext::Launch(pending) => {
                let outcome = blocking(|| pending.perform()).await;
                let work_id = pending.work_id().to_string();
                let mut core = CoreGuard::acquire(&state.core).await;
                (
                    work_id,
                    state.engine.settle_launch(&mut core, pending, outcome),
                    core,
                )
            }
            EngineNext::Send(pending) => {
                let outcome = blocking(|| pending.perform()).await;
                let work_id = pending.work_id().to_string();
                let mut core = CoreGuard::acquire(&state.core).await;
                (
                    work_id,
                    state.engine.settle_send(&mut core, pending, outcome),
                    core,
                )
            }
            EngineNext::Surface(pending) => {
                let outcome = blocking(|| pending.perform()).await;
                let work_id = pending.work_id().to_string();
                let mut core = CoreGuard::acquire(&state.core).await;
                (
                    work_id,
                    state.engine.settle_surface(&mut core, pending, outcome),
                    core,
                )
            }
            EngineNext::Observe(pending) => {
                let outcome = blocking(|| pending.perform()).await;
                let work_id = pending.work_id().to_string();
                let mut core = CoreGuard::acquire(&state.core).await;
                (
                    work_id,
                    state.engine.settle_observe(&mut core, pending, outcome),
                    core,
                )
            }
            EngineNext::Interrupt(pending) => {
                let outcome = blocking(|| pending.perform()).await;
                let work_id = pending.work_id().to_string();
                let mut core = CoreGuard::acquire(&state.core).await;
                (
                    work_id,
                    state.engine.settle_interrupt(&mut core, *pending, outcome),
                    core,
                )
            }
        };
        let (work_id, outcome, core) = settled;
        match outcome {
            Ok(next_step) => {
                held = Some(core);
                step = next_step;
            }
            Err(e) => {
                tracing::error!(work_id = %work_id, error = %e, "settling an external effect failed");
                return Some(core);
            }
        }
    }
}

/// How often the completion driver asks live executions where they are.
///
/// A turn that ends on its own is the only state change in this daemon that
/// nothing requests, so it is the only one that needs a clock. 200 ms is
/// chosen against what the tick actually costs, not against how fast a human
/// wants a cascade: one `Backend::observe` per *live* execution, which for the
/// Claude adapter is a mutex and a match over in-memory turn state (issue
/// #46's own framing — the classification was already right and merely
/// starved), plus one core-lock hold to enumerate them. A run with no
/// execution in flight costs the enumeration and nothing else.
pub const COMPLETION_POLL_INTERVAL: Duration = Duration::from_millis(200);

/// Settle turns that finish with nobody watching (issue #46).
///
/// **The measured defect.** Run B's stage `00-contract` sat `active` for
/// 45m28s after a clean turn end, and again for 34m58s after an envelope-less
/// one. Nothing was wrong with the classification — `observe_in_memory`
/// derives `StageCompleted` for the first shape and `native: Unknown` (→
/// blocked) for the second, and the adapter's own unit tests pin both. What
/// was missing is that after launch-settle parked a still-`InFlight` turn,
/// **no observer ever came back**: the engine observes at launch-settle, at
/// SEND-settle, at restart recovery, and on client cranks, and a real turn
/// finishes at none of those instants. The deterministic fake finishes inside
/// the launch effect, so every test settled at launch and the gap was
/// structurally invisible to the whole suite.
///
/// **Rung note (R2).** This is a loop over the machinery that already exists:
/// [`Engine::due_observations`] reads the same projection every handler reads,
/// [`crank`] performs the effect off the lock and settles it exactly as it
/// does for a client's request, and [`Engine::settle_observe`] re-checks
/// §14.5 exactly as `settle_launch` does. Lower rungs, checked: R1 — the
/// daemon cannot skip this, the defect is a measured 45-minute stall; the
/// notification alternative (the adapter's turn-end reaching the daemon
/// through the event sink it already writes to) is the *higher* rung and is
/// not taken, because it would make settling depend on a broadcast delivery
/// whose loss is silent, and because it would settle only for adapters that
/// emit that event while this loop is true of every `Backend`.
///
/// **What it does not do.** It never observes under the core lock (§22.6:
/// every observation goes through `crank`, which drops the guard before every
/// effect), it never journals a poll that found nothing, and it never settles
/// an observation the run has moved past. It also stops promptly: the daemon's
/// `closing` watch is checked before every tick and between candidates, so a
/// shutdown never waits out a full interval, and `start_with` joins this task
/// before it journals `daemon.stopped` — the driver is the one writer that is
/// not a request, so it is also the one that has to be told to stop writing.
pub async fn drive_completions(state: ApiState, interval: Duration) {
    let mut closing = state.closing.clone();
    loop {
        tokio::select! {
            _ = tokio::time::sleep(interval) => {}
            _ = closing.changed() => return,
        }
        if *closing.borrow() {
            return;
        }
        let (due, overdue) = {
            let core = CoreGuard::acquire(&state.core).await;
            // Read-only, side-effect-free question asked of the projection
            // (§22.6): nothing here is consumed, so abandoning a `due`
            // candidate mid-loop below (shutdown) just means it is asked
            // again next tick — or, on a restart, freshly re-derived from
            // the projection with nothing lost.
            let due = state.engine.due_observations(&core);
            // R-MVP1-7's per-turn wall-clock ceiling rides this same 200 ms
            // sweep, read under the same guard hold `due_observations`
            // already takes — but unlike `due_observations`, this one is
            // NOT side-effect-free: `due_interrupts` destructively removes
            // every overdue entry from `Engine`'s in-memory `turn_started`
            // map as it collects them ("one attempt per crossing", its own
            // doc). Once collected here, an entry in `overdue` below is the
            // *only* record that this crossing ever happened — dropping it
            // (e.g. on a shutdown race) rather than delivering it would
            // lose the attempt with no journal trace at all, not merely
            // defer it to a later tick the way an abandoned `due` observe
            // is deferred.
            let overdue = state.engine.due_interrupts(&core, Instant::now());
            (due, overdue)
        };
        // Every `overdue` entry was already destructively dequeued above —
        // deliver all of them before honoring a shutdown signal that lands
        // mid-loop, so a crossing already consumed from the clock is never
        // silently discarded (see `due_interrupts`'s doc just above). This
        // is why this loop, alone among the two below, does not re-check
        // `closing` per iteration.
        for pending in overdue {
            // §14.5 for the perform itself: each crank below is awaited
            // serially, so a late entry's collection→perform window spans
            // every earlier interrupt — long enough for the targeted turn to
            // end and a SEND to spawn a fresh one on the same execution
            // handle. Re-validate under the guard immediately before the
            // kill; a stale crossing is journaled (`settle_stale_interrupt`),
            // never silently dropped.
            let stale = {
                let mut core = CoreGuard::acquire(&state.core).await;
                match state.engine.interrupt_is_stale(&core, &pending) {
                    Some(reason) => {
                        if let Err(e) = state
                            .engine
                            .settle_stale_interrupt(&mut core, &pending, reason)
                        {
                            tracing::error!(
                                work_id = %pending.work_id(),
                                error = %e,
                                "journaling a stale ceiling crossing failed"
                            );
                        }
                        true
                    }
                    None => false,
                }
            };
            if stale {
                continue;
            }
            crank(
                &state,
                Step {
                    next: EngineNext::Interrupt(pending),
                    deferred: Deferred::new(),
                },
            )
            .await;
        }
        for pending in due {
            if *closing.borrow() {
                return;
            }
            crank(
                &state,
                Step {
                    next: EngineNext::Observe(pending),
                    deferred: Deferred::new(),
                },
            )
            .await;
        }
    }
}

/// Take the guard [`crank`] handed back, or acquire one if it kept none.
async fn relock<'a>(state: &'a ApiState, held: Option<CoreGuard<'a>>) -> CoreGuard<'a> {
    match held {
        Some(guard) => guard,
        None => CoreGuard::acquire(&state.core).await,
    }
}

/// Structured 404 for routes the router does not know.
async fn not_found() -> Response {
    error_response(StatusCode::NOT_FOUND, "not_found", "no such route")
}

/// Structured 405 for known routes hit with an unsupported method.
async fn method_not_allowed() -> Response {
    error_response(
        StatusCode::METHOD_NOT_ALLOWED,
        "method_not_allowed",
        "method not allowed on this route",
    )
}

/// The 401 this listener answers with, wherever it is answered.
///
/// One constructor, so a client cannot learn a different error vocabulary
/// depending on which route it knocked on.
fn unauthorized_response() -> Response {
    error_response(
        StatusCode::UNAUTHORIZED,
        "unauthorized",
        "missing or invalid bearer token",
    )
}

/// Structured JSON error response.
fn error_response(status: StatusCode, code: &str, message: impl Into<String>) -> Response {
    (status, Json(error_body(code, message))).into_response()
}

/// The `{"error": {...}}` body alone (also journaled in `command.rejected`).
fn error_body(code: &str, message: impl Into<String>) -> Value {
    json!({"error": {"code": code, "message": message.into()}})
}

/// The error body an [`EngineError`] answers with.
///
/// A routing failure carries its available options as data, not only prose:
/// §13's terminal state is "fail with available options", and a client that
/// has to regex an error message to learn them has not been told them.
fn engine_error_body(e: &EngineError) -> Value {
    let mut body = error_body(e.code(), e.to_string());
    if let Some(available) = e.available_backends() {
        body["error"]["available_backends"] = json!(available);
    }
    body
}

/// HTTP status for an engine failure: 4xx where the client can fix it, 500
/// only where sergeant itself broke.
fn engine_error_status(e: &EngineError) -> StatusCode {
    match e {
        EngineError::Core(_) => StatusCode::INTERNAL_SERVER_ERROR,
        EngineError::NotAwaitingInput { .. }
        | EngineError::NotRetryable { .. }
        | EngineError::NotBlocked { .. }
        | EngineError::IllegalTransition { .. } => StatusCode::CONFLICT,
        EngineError::NoRun { .. } => StatusCode::NOT_FOUND,
        _ => StatusCode::UNPROCESSABLE_ENTITY,
    }
}

/// Name of the query parameter that may carry the token on safe requests.
const TOKEN_QUERY_PARAM: &str = "token";

/// The bearer token presented by a request, if any.
///
/// Two carriers, deliberately unequal:
///
/// - `Authorization: Bearer <token>` — always accepted;
/// - `?token=<token>` — accepted on **GET and HEAD only**.
///
/// The query form exists because a browser-based client of `/v1/events/stream`
/// cannot set a header on an `EventSource`, which has no header API at all —
/// the embedded dashboard (deleted, ADR 0011) was the original such client,
/// and any future one would need the same carve-out. The tradeoff is real —
/// a URL-borne secret lands in shell history and in any log that records
/// request lines — and it is accepted for P0 on the grounds that the
/// listener is loopback-only, the token is regenerated on every daemon
/// start, and the descriptor holding it is already 0600. The post-P0
/// alternative, for the ledger entry this milestone's MARK & LOG still owes
/// (R1 — ship the cheapest thing that closes the P0 and name the better
/// shape rather than build it): exchange the URL token once for a
/// `HttpOnly; SameSite=Strict` cookie and drop the query form. That entry is
/// not written yet, and this comment does not claim it is.
///
/// Restricting it to safe methods is the part that costs nothing: a page that
/// learns the token can already read everything, but a *mutating* request
/// authorized by a URL alone is the shape a cross-site form post can forge
/// without ever reading a response. Mutations therefore keep requiring a
/// header, which no cross-origin form can set.
///
/// **This is the only copy of the rule.** The dashboard used to need the same
/// extraction for its own pages, and the second copy it kept had already
/// drifted from this one: it took a query token on any method and
/// percent-decoded it. `require_bearer` below is the one caller now.
fn presented_token(method: &Method, headers: &HeaderMap, query: Option<&str>) -> Option<String> {
    if let Some(header) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        return Some(header.to_string());
    }
    if !matches!(*method, Method::GET | Method::HEAD) {
        return None;
    }
    query_token(query?)
}

/// Pull `token=<value>` out of a raw query string.
///
/// The token alphabet is Crockford base32 (two ULIDs), so no percent-decoding
/// is needed and none is done: a value that had to be decoded to match was
/// not the token this daemon published.
fn query_token(query: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        pair.strip_prefix(TOKEN_QUERY_PARAM)
            .and_then(|rest| rest.strip_prefix('='))
            .map(str::to_string)
    })
}

/// Bearer-token gate for `/v1/*`. `/healthz` is the only route mounted
/// outside this layer.
///
/// The embedded dashboard (deleted, ADR 0011) used to re-implement this: its
/// own extraction, its own comparison, and its own hand-written copy of the
/// 401 body below. One gate decides for every route on this listener now —
/// a second implementation of an authorization rule is a second rule, and it
/// had already diverged on the safe-method bound.
async fn require_bearer(State(state): State<ApiState>, req: Request, next: Next) -> Response {
    let presented = presented_token(req.method(), req.headers(), req.uri().query());
    if presented.as_deref() == Some(state.token.as_str()) {
        next.run(req).await
    } else {
        unauthorized_response()
    }
}

/// `GET /healthz` — liveness, unauthenticated.
async fn healthz() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

/// The `GET /v1/system` body.
///
/// `journal_head` is the seq of the last committed event. It is here because
/// every live client needs it and none of them may read the journal to find
/// it: it is the resume point an SSE subscriber passes as `from` so that
/// attaching to the tail does not replay all of history first.
fn system_body(state: &ApiState, journal_head: u64, admission_paused: bool) -> Value {
    json!({
        "version": env!("CARGO_PKG_VERSION"),
        "api_revision": API_REVISION,
        "data_dir": state.data_dir,
        "journal_head": journal_head,
        // MVP-3's admission drain flag (`sgt daemon stop`): whether new
        // `POST /v1/work` submissions are currently refused. Surfaced here
        // so `sgt status` can say why a submit is being turned away without
        // an operator having to decode the journal for `admission.paused`.
        "admission_paused": admission_paused,
    })
}

/// `GET /v1/system` — version, API revision, data dir, journal head.
async fn system_info(State(state): State<ApiState>) -> Json<Value> {
    let core = CoreGuard::acquire(&state.core).await;
    let head = core.registry.last_seq();
    let admission_paused = core.registry.state().admission_paused;
    Json(system_body(&state, head, admission_paused))
}

/// The event source all API-origin events carry.
fn api_source() -> EventSource {
    EventSource::new("daemon", "api")
}

/// Unwrap a JSON body extraction, answering a structured error on failure
/// (axum's stock `Json` rejection is plain text; errors here are contracted
/// to be structured JSON). The rejection's own status is kept, so a
/// wrong content type stays a 415 and a bad shape a 422.
fn parse_body<T>(body: Result<Json<T>, JsonRejection>) -> Result<T, Box<Response>> {
    match body {
        Ok(Json(value)) => Ok(value),
        Err(rejection) => Err(Box::new(error_response(
            rejection.status(),
            "invalid_request",
            format!("invalid JSON body: {}", rejection.body_text()),
        ))),
    }
}

/// Unwrap a query-string extraction, answering a structured 400 on failure
/// (axum's stock `Query` rejection is plain text; errors here are contracted
/// to be structured JSON, exactly as for malformed bodies).
fn parse_query<T>(query: Result<Query<T>, QueryRejection>) -> Result<T, Box<Response>> {
    match query {
        Ok(Query(value)) => Ok(value),
        Err(rejection) => Err(Box::new(error_response(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            format!("invalid query string: {}", rejection.body_text()),
        ))),
    }
}

/// Validate a client-supplied command id (must be a ULID, §26).
fn parse_command_id(raw: &str) -> Result<(), Box<Response>> {
    ulid::Ulid::from_string(raw).map(|_| ()).map_err(|_| {
        Box::new(error_response(
            StatusCode::BAD_REQUEST,
            "invalid_command_id",
            "command_id must be a ULID",
        ))
    })
}

/// Replay a recorded command outcome, if this `command_id` was seen before.
/// The stored `Value` serializes to the same bytes every time, so duplicates
/// are byte-identical to the original response.
fn replay_command(core: &Core, command_id: &str) -> Option<Response> {
    core.registry.state().commands.get(command_id).map(|o| {
        let status = StatusCode::from_u16(o.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(o.result.clone())).into_response()
    })
}

/// Journal a command outcome (`command.accepted` / `command.rejected`) and
/// answer with exactly the journaled result. A successful status is an
/// acceptance and a failing one a rejection — the classification is derived
/// from the status rather than passed alongside it, so the two can never
/// disagree.
fn record_and_respond(
    core: &mut Core,
    command_id: &str,
    operation: &str,
    work_id: Option<&str>,
    status: StatusCode,
    result: Value,
) -> Response {
    let kind = if status.is_success() {
        KIND_COMMAND_ACCEPTED
    } else {
        KIND_COMMAND_REJECTED
    };
    let mut draft = EventDraft::new(
        api_source(),
        kind,
        json!({
            "command_id": command_id,
            "operation": operation,
            "status": status.as_u16(),
            "result": result,
        }),
    );
    draft.correlation_id = Some(command_id.to_string());
    if let Some(id) = work_id {
        draft = draft.with_work_id(id);
    }
    if let Err(e) = core.commit(draft) {
        return internal_error(e);
    }
    // Every command handler ends here, so this is the one place where a
    // command's whole journal record — its work events and the outcome that
    // makes it replayable — becomes durable, and the one place where a failed
    // group commit can still be answered with a 500 instead of only logged
    // (#44). Without it the `CoreGuard` backstop would flush the same events
    // moments later, after the client had already been told the command
    // succeeded.
    if let Err(e) = core.flush() {
        return internal_error(e);
    }
    (status, Json(result)).into_response()
}

fn internal_error(e: impl std::fmt::Display) -> Response {
    error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string())
}

#[derive(Debug, Deserialize)]
struct SubmitRequest {
    command_id: String,
    intent: String,
    #[serde(default)]
    workspace: Option<String>,
    #[serde(default)]
    repositories: Vec<String>,
    #[serde(default)]
    workflow: Option<String>,
    #[serde(default)]
    backend: Option<String>,
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    created_by: Option<String>,
    #[serde(default)]
    origin: Option<Origin>,
    /// R-MVP1-6's structured-intent schema slot. Progressive elaboration of
    /// `intent`, checked for agreement against `workflow`/`repositories`
    /// above (§13's one-source-of-truth rule) and journaled verbatim
    /// otherwise — nothing here drives routing.
    #[serde(default)]
    intent_detail: Option<IntentDetail>,
    /// MVP-3's submit-time envelope override (`--turns`/`--ceiling-secs`):
    /// see [`EnvelopeRequest`]. Validated at submit (both fields, when
    /// present, must be at least 1) and journaled verbatim otherwise —
    /// nothing here drives routing, exactly like `intent_detail` above.
    #[serde(default)]
    envelope: Option<EnvelopeRequest>,
}

/// R-MVP1-7's envelope override, sanity-checked at submit: a turn cap of 0
/// would block a Work before its first turn ever spawns (indistinguishable
/// from "already exhausted"), and a ceiling of 0 would interrupt a turn
/// before the completion driver's very first tick could ever see it finish —
/// both are certainly a client mistake, never an intentional bound, so both
/// fail closed here rather than producing a Work that can never make
/// progress.
fn envelope_out_of_range(req: &SubmitRequest) -> Option<String> {
    let envelope = req.envelope.as_ref()?;
    if envelope.turn_cap == Some(0) {
        return Some(
            "envelope.turn_cap must be at least 1 (0 would block before any turn)".to_string(),
        );
    }
    if envelope.ceiling_secs == Some(0) {
        return Some(
            "envelope.ceiling_secs must be at least 1 (0 would interrupt before any turn could \
             ever finish)"
                .to_string(),
        );
    }
    None
}

/// R-MVP1-6's submit-time agreement check: a structured elaboration that
/// names a `workflow`/`repos` different from what the submission's own
/// flags say is two answers to "what is this Work about" in one Work, and
/// §13 requires exactly one source of truth. Absent on either side is not a
/// disagreement — only *both present and different* is. Repository sets are
/// compared unordered (a client naming the same repos in a different order
/// has not disagreed with itself).
fn intent_detail_disagreement(req: &SubmitRequest) -> Option<String> {
    let detail = req.intent_detail.as_ref()?;
    if let (Some(declared), Some(flag)) = (detail.workflow.as_deref(), req.workflow.as_deref())
        && declared != flag
    {
        return Some(format!(
            "intent_detail.workflow is {declared:?}, but the submission's workflow flag is \
             {flag:?}; the two must agree"
        ));
    }
    if let Some(declared) = detail.repos.as_ref()
        && !req.repositories.is_empty()
    {
        let declared_set: std::collections::BTreeSet<&str> =
            declared.iter().map(String::as_str).collect();
        let flag_set: std::collections::BTreeSet<&str> =
            req.repositories.iter().map(String::as_str).collect();
        if declared_set != flag_set {
            return Some(format!(
                "intent_detail.repos is {declared:?}, but the submission's repositories are \
                 {:?}; the two must agree",
                req.repositories
            ));
        }
    }
    // `detail.group` is deliberately not checked here (I3): R-MVP1-5(b)
    // gives group membership "no new engine surface" in MVP-1 — there is no
    // `--group` flag yet for it to disagree with, and expanding it into a
    // repository set to compare is MVP-3's CLI-side job. Revisit this
    // function when `--group` lands, not before.
    None
}

/// §13's origin metadata: who is asking, and from where.
#[derive(Debug, Clone, Default, Deserialize)]
struct Origin {
    /// Front-end harness name, e.g. `claude`. Drives origin affinity.
    #[serde(default)]
    client: Option<String>,
    /// The client's working directory. Workspace discovery (§9) starts here;
    /// a daemon has no cwd of its own, so this is the only honest source for
    /// "which repository is this work about".
    #[serde(default)]
    cwd: Option<PathBuf>,
}

/// `POST /v1/work` — submit work; it enters `pending`.
///
/// Everything past the `command_id` check is journaled under that id, accepted
/// or rejected, so a repeat of the same `command_id` replays one recorded
/// result forever (§26). Only the two failures that leave no usable command
/// identity — an unparseable body, and a `command_id` that is not a ULID —
/// answer without a journal record: there is no key to record them under.
async fn submit_work(
    State(state): State<ApiState>,
    body: Result<Json<SubmitRequest>, JsonRejection>,
) -> Response {
    let req = match parse_body(body) {
        Ok(r) => r,
        Err(resp) => return *resp,
    };
    if let Err(resp) = parse_command_id(&req.command_id) {
        return *resp;
    }
    // Planning happens with **no lock held**. It reads the filesystem —
    // `Workspace::discover` walks up to the git root and parses
    // `sergeant.toml`, `WorkflowDefinition::resolve` reads every stage's
    // `CONTEXT.md` — and probes every harness the run will use (§17.5). All of
    // that is external I/O on paths the daemon does not own, which §22.6 keeps
    // out from under the core lock; it is also, by construction, side-effect
    // free, so running it before the guard costs nothing but a re-check.
    //
    // The re-check is the price. Two concurrent submissions of the same
    // `command_id` can now both plan; exactly-once is preserved by taking the
    // guard afterwards and consulting the recorded outcome there, which is the
    // same single-writer decision it always was — just made after the reading
    // rather than around it.
    let origin = req.origin.clone().unwrap_or_default();
    let planned = if req.intent.trim().is_empty() {
        None
    } else {
        let engine = state.engine.clone();
        let backend = req.backend.clone();
        let workflow = req.workflow.clone();
        let profile = req.profile.clone();
        let repositories = req.repositories.clone();
        let origin_cwd = origin.cwd.clone();
        let origin_client = origin.client.clone();
        Some(
            blocking(move || {
                engine.plan(&SubmitContext {
                    cwd: origin_cwd.as_deref(),
                    origin_client: origin_client.as_deref(),
                    backend: backend.as_deref(),
                    workflow: workflow.as_deref(),
                    profile: profile.as_deref(),
                    repositories: &repositories,
                })
            })
            .await,
        )
    };

    let mut core = CoreGuard::acquire(&state.core).await;
    if let Some(resp) = replay_command(&core, &req.command_id) {
        return resp;
    }
    // Crash-window recovery. `work.submitted` and the command record are two
    // separate fsynced appends; a daemon that died between them (or whose
    // second append failed) leaves a durable Work record with no recorded
    // outcome. The command *did* execute, exactly once — so re-record its
    // outcome from the replayed state and answer that, instead of executing a
    // second time under the same command_id.
    if let Some(work_id) = core
        .registry
        .state()
        .command_works
        .get(&req.command_id)
        .cloned()
    {
        let result = work_view(&core, &state.engine, &work_id);
        return record_and_respond(
            &mut core,
            &req.command_id,
            "work.submit",
            Some(&work_id),
            StatusCode::CREATED,
            result,
        );
    }
    if req.intent.trim().is_empty() {
        // A semantic rejection is an outcome like any other: journal it under
        // this command_id so the retry replays the 400 rather than being
        // re-validated (possibly by a future build with different rules).
        let result = error_body("invalid_request", "intent must not be empty");
        return record_and_respond(
            &mut core,
            &req.command_id,
            "work.submit",
            None,
            StatusCode::BAD_REQUEST,
            result,
        );
    }
    if let Some(reason) = intent_detail_disagreement(&req) {
        // Same shape as the empty-intent rejection above: a fail-closed
        // semantic outcome, journaled under this command_id so a retry
        // replays it rather than re-validating against a possibly different
        // future rule (R-MVP1-6: "one source of truth, fail closed").
        let result = error_body("intent_detail_disagreement", reason);
        return record_and_respond(
            &mut core,
            &req.command_id,
            "work.submit",
            None,
            StatusCode::BAD_REQUEST,
            result,
        );
    }
    if let Some(reason) = envelope_out_of_range(&req) {
        let result = error_body("invalid_envelope", reason);
        return record_and_respond(
            &mut core,
            &req.command_id,
            "work.submit",
            None,
            StatusCode::BAD_REQUEST,
            result,
        );
    }
    // MVP-3's admission drain (`sgt daemon stop`, scoped exactly to that
    // use): a genuinely new submission arriving while admission is paused is
    // refused, journaled like any other rejected outcome so a retry after
    // admission resumes is a distinct, freshly-evaluated command rather than
    // a replay of this refusal. Checked here, *after* the replay/crash-window
    // lookups above, so a duplicate `command_id` from before the pause still
    // replays its original (accepted) outcome — pausing admission stops new
    // work, not the idempotent replay of work already accepted.
    if core.registry.state().admission_paused {
        let result = error_body(
            "admission_paused",
            "admission is paused (the daemon is draining for `sgt daemon stop`) — refusing new \
             work until it resumes",
        );
        return record_and_respond(
            &mut core,
            &req.command_id,
            "work.submit",
            None,
            StatusCode::SERVICE_UNAVAILABLE,
            result,
        );
    }

    // The plan decided above, before the guard: workspace topology, workflow
    // content, routing, profiles and the §17.5 stage preflight, all with no
    // side effects — so a submission that cannot be routed is rejected with
    // §13's available options instead of creating work that immediately dies.
    // `Ok(None)` means the client offered no repository context; the work is
    // accepted and stays `pending`, exactly as it did before there was an
    // engine.
    let plan = match planned.expect("planned whenever the intent is non-empty") {
        Ok(plan) => plan,
        Err(e) => {
            let status = engine_error_status(&e);
            let result = engine_error_body(&e);
            return record_and_respond(
                &mut core,
                &req.command_id,
                "work.submit",
                None,
                status,
                result,
            );
        }
    };

    let work = Work {
        id: ulid::Ulid::generate().to_string(),
        workspace: req
            .workspace
            .or_else(|| plan.as_ref().map(|p| p.workspace.name.clone())),
        intent: req.intent,
        repositories: req.repositories,
        workflow: req.workflow,
        backend: req.backend,
        origin_client: origin.client,
        profile: req.profile,
        // I3: an empty `{}` object is the same fact as sending nothing at
        // all (IntentDetail::is_empty's own doc) — normalized here so that
        // promise is actually kept, not merely stated.
        intent_detail: req.intent_detail.filter(|d| !d.is_empty()),
        // I3's own rule, reused: an empty `{}` override is the same fact as
        // sending nothing.
        envelope: req.envelope.filter(|e| !e.is_empty()),
        state: WorkState::Pending,
        created_by: req.created_by.unwrap_or_else(|| "api".to_string()),
        created_at: rfc3339_utc_now(),
    };
    let work_id = work.id.clone();
    let mut draft = EventDraft::new(api_source(), KIND_WORK_SUBMITTED, json!({"work": work}))
        .with_work_id(&work_id);
    draft.correlation_id = Some(req.command_id.clone());
    if let Err(e) = core.commit(draft) {
        return internal_error(e);
    }

    if let Some(plan) = plan {
        // The Work is durable now. A start failure cannot un-accept it, so
        // the engine fails it closed to `blocked` with the reason recorded;
        // only an internal failure escapes as an error here.
        //
        // Two-phase (§14.2): everything authoritative — surface, binding,
        // `work.started`, the first stage's `execution.reserved` — lands
        // under this guard; the harness launch happens after it is dropped.
        let step = match state.engine.begin_start(&mut core, &work, &plan) {
            Ok(step) => step,
            Err(e) => {
                tracing::error!(work_id = %work_id, error = %e, "starting the run failed");
                let status = engine_error_status(&e);
                let result = engine_error_body(&e);
                return record_and_respond(
                    &mut core,
                    &req.command_id,
                    "work.submit",
                    Some(&work_id),
                    status,
                    result,
                );
            }
        };
        // Same boundary as the arms below, one indent shallower: everything
        // `begin_start` journaled is made durable before the guard drops and
        // `git worktree add` / the harness launch run (#44).
        if let Err(e) = core.flush() {
            return internal_error(e);
        }
        drop(core);
        core = relock(&state, crank(&state, step).await).await;
    }

    // Answer from the projection, not the request: proves the read path.
    let result = work_view(&core, &state.engine, &work_id);
    record_and_respond(
        &mut core,
        &req.command_id,
        "work.submit",
        Some(&work_id),
        StatusCode::CREATED,
        result,
    )
}

/// The [`WorkRun`] to render for a work: the live or R-MVP1-9-evicted-but-
/// cached registry entry (`cached`, which callers pass as `WorkRegistry::
/// run_view`'s answer — checking both `runs` and `terminal_runs`), or —
/// only when *neither* has it — the read-side full-journal re-derivation.
///
/// The full-replay path is a bounded fallback, not the ordinary case:
/// every eviction populates `terminal_runs` at the moment it reclaims
/// `runs` (`maybe_evict`), so a recently terminal work's view is answered
/// from that in-memory cache, never a per-request journal walk (W2/TH-08:
/// this used to replay the *entire* journal on every view of every terminal
/// work, including ones with no run at all, which is exactly the unbounded-
/// I/O-under-the-guard shape §22.6 forbids). What remains of that shape is
/// an acknowledged tradeoff, not an anomaly path: `terminal_runs` holds only
/// `TERMINAL_RUN_CACHE_CAPACITY` entries, so on an installation with more
/// terminal works than that, viewing one aged out of the cache replays the
/// journal while the guard is held, queueing other requests behind it —
/// rate-limited to cache misses rather than eliminated. The replay runs via
/// [`blocking_sync`] so it at least never starves the async worker's other
/// tasks; moving it off the guard entirely would need a journal reader the
/// core does not own, which is the named follow-up shape, not this build's.
/// The same fallback also covers a registry an older build populated before
/// the cache existed.
///
/// Only `is_absorbing` states (`Completed`/`Canceled`) are ever evicted, so
/// that is the only case worth even trying the fallback for; every other
/// state's `None` here means exactly what it always meant — no run exists
/// yet — and costs nothing extra to answer.
///
/// A replay failure is logged and treated as "nothing to show" rather than
/// failing the whole view: `work_view` composes many independent fields from
/// this one `Option`, and a journal I/O error re-deriving a *terminal* work's
/// history should not turn into a 500 for `work.state` and every other field
/// that needed no replay at all.
fn resolve_run(core: &Core, work: &Work, cached: Option<&WorkRun>) -> Option<WorkRun> {
    if let Some(run) = cached {
        return Some(run.clone());
    }
    if !is_absorbing(work.state) {
        return None;
    }
    match blocking_sync(|| rederive_run(&core.journal, &work.id)) {
        Ok(run) => run,
        Err(e) => {
            tracing::error!(
                work_id = %work.id,
                error = %e,
                "could not re-derive an evicted work's run from the journal"
            );
            None
        }
    }
}

/// R-MVP1-2's output pointer (the sibling half of the ruling, not itself a
/// ruling): once a surface has been torn down, name per repository what §11
/// already recorded — the source repository, the retained branch, the
/// worktree path — plus the finalize commit `surface::teardown` now captures
/// as it reads the branch (R-MVP1-2's own extension to `BindingTeardown`).
/// That commit is whatever a closing stage's `promote` disposition landed
/// before teardown ran, per the ruled timing, or just the surface's base SHA
/// if the stage declared nothing.
///
/// `None` before teardown has run: there is no output to point at yet, and a
/// null here (rather than a partial view built from `run.surface` alone) is
/// the honest answer for a work still in flight.
fn output_pointer(work: &Work, run: &WorkRun) -> Option<Value> {
    let teardown = run.teardown.as_ref()?;
    let surface = run.surface.as_ref();
    let repositories: Vec<Value> = teardown
        .bindings
        .iter()
        .map(|binding| {
            let source_repo = surface
                .and_then(|s| {
                    s.bindings
                        .iter()
                        .find(|b| b.repository == binding.repository)
                })
                .map(|b| b.source_path.display().to_string());
            json!({
                "repository": binding.repository,
                "source_repo": source_repo,
                "retained_branch": binding.work_branch,
                "worktree_path": binding.worktree_path,
                "finalize_commit": binding.final_sha,
                "disposition": disposition_tag(&binding.disposition),
            })
        })
        .collect();
    Some(json!({
        "work_id": work.id,
        "clean": teardown.clean,
        "repositories": repositories,
    }))
}

/// The bare tag of a [`BindingDisposition`] — `"removed"`, `"missing"`, etc.
/// — without its internally-tagged detail fields (`changes`/`detail`), which
/// the output pointer does not repeat: the full `teardown` field elsewhere
/// in [`work_view`] already carries them, and serializing the enum directly
/// here would nest a second `"disposition"` key inside this one (its own
/// `#[serde(tag = "disposition")]`), not replace it.
fn disposition_tag(disposition: &BindingDisposition) -> &'static str {
    match disposition {
        BindingDisposition::Removed => "removed",
        BindingDisposition::RetainedDirty { .. } => "retained_dirty",
        BindingDisposition::Missing => "missing",
        BindingDisposition::RetainedError { .. } => "retained_error",
        BindingDisposition::RetainedUnreferenced { .. } => "retained_unreferenced",
    }
}

/// §11's integrity axis as a sibling key of `work`, for the two views C5
/// makes mandatory (`sgt work list`, `sgt work show`).
///
/// `None` — the key renders as `null` — is **not assessed**: a Work whose
/// surface never tore down, and a `surface.torn_down` journaled before Phase
/// A. It never means clean, which is why this reads `run.integrity` (recorded
/// by the retirement that assessed it) rather than deriving a verdict here
/// from a teardown report that may predate the assessment entirely.
///
/// The findings and drift travel with the disposition rather than in a third
/// key: a Work is dirty *because of* something, and a client that has to
/// join two keys to say why will not.
fn integrity_view(run: &WorkRun) -> Option<Value> {
    let disposition = run.integrity?;
    let teardown = run.teardown.as_ref();
    Some(json!({
        "disposition": disposition,
        "findings": teardown
            .map(|t| t.findings().cloned().collect::<Vec<_>>())
            .unwrap_or_default(),
        // §11.4: reported beside the findings, never among them, and never
        // part of the disposition.
        "drift": teardown.map(|t| t.drift.clone()).unwrap_or_default(),
    }))
}

/// ADR 0007(b): a closing stage that declares a commit as its durable
/// outcome must not be reported as plain `completed` when the branch never
/// advanced and the worktree was left dirty — the safety net for when an
/// actor guesses wrong about its own runtime model anyway
/// (`docs/adr/0007-actor-runtime-contract.md`). The engine still learns
/// nothing about what a commit *is* (NORTH-STAR: "the engine learns no
/// output vocabulary; only the pointer is core"): this reads two facts the
/// pointer already computes — a binding's teardown disposition, and whether
/// its finalize commit ever moved past the surface's own base SHA — rather
/// than asking any workflow what it meant to do.
fn stranded_completion(work: &Work, run: &WorkRun) -> bool {
    if work.state != WorkState::Completed {
        return false;
    }
    let Some(teardown) = run.teardown.as_ref() else {
        return false;
    };
    let Some(surface) = run.surface.as_ref() else {
        return false;
    };
    teardown.bindings.iter().any(|binding| {
        let never_advanced = surface
            .bindings
            .iter()
            .find(|b| b.repository == binding.repository)
            .is_some_and(|b| binding.final_sha.as_deref() == Some(b.base_sha.as_str()));
        never_advanced
            && matches!(
                binding.disposition,
                BindingDisposition::RetainedDirty { .. }
            )
    })
}

/// The `state` `work list`/`work show` report: verbatim for every ordinary
/// case, but not plain `completed` when [`stranded_completion`] holds. The
/// persisted [`WorkState`] this is derived from is untouched — the run is
/// genuinely terminal, neither blocked nor failed — so retry, cancel, and
/// every other state-machine consumer still see `Completed`; only the
/// string an operator reads first changes.
fn reported_state(work: &Work, run: Option<&WorkRun>) -> &'static str {
    // §11.5: `completed_dirty` is the compact label for completed + dirty,
    // and it is now reached two ways that union rather than compete. ADR
    // 0007(b)'s `stranded_completion` infers dirtiness from the output
    // pointer alone, which is all a pre-Phase-A journal can offer; §11's
    // integrity disposition is the assessment retirement actually recorded.
    // Either one is enough.
    //
    // `failed` and `canceled` keep their true state strings even when dirty
    // (§11.5 adds no `failed_dirty` label and no transition target); the
    // `integrity` sibling key carries that axis, and `sgt work list` renders
    // it beside the state so C5's "distinguishable in default output" holds
    // for all three terminal states.
    let dirty_completion = run.is_some_and(|r| {
        stranded_completion(work, r)
            || (work.state == WorkState::Completed
                && r.integrity == Some(IntegrityDisposition::Dirty))
    });
    if dirty_completion {
        "completed_dirty"
    } else {
        work.state.as_str()
    }
}

/// The full view of a work: the §10 record, plus the orthogonal run
/// coordinates the M3 contract asks `work show` to include — current stage,
/// surface, and execution state. They are siblings of `work`, not fields
/// inside it: §10 keeps stage orthogonal to work state, and flattening them
/// into one record is how "in review" becomes a state-machine value.
fn work_view(core: &Core, engine: &Engine, work_id: &str) -> Value {
    let registry = core.registry.state();
    let work = registry.works.get(work_id);
    let cached_run = registry.run_view(work_id);
    let run = work.and_then(|w| resolve_run(core, w, cached_run));
    // ADR 0007(b): the persisted `Work` serializes with its true `state`
    // (`Completed`) intact; only this view's own `state` key is overridden,
    // so a work still in flight and every non-stranded completion look
    // exactly as before.
    let work_json = work.map(|w| {
        let mut value = serde_json::to_value(w).unwrap_or(Value::Null);
        if let Some(object) = value.as_object_mut() {
            object.insert("state".to_string(), json!(reported_state(w, run.as_ref())));
        }
        value
    });
    json!({
        "work": work_json,
        "stage": run.as_ref().and_then(run_stage_view),
        "surface": run.as_ref().and_then(|r| r.surface.clone()),
        "execution": run.as_ref().and_then(|r| r.execution.clone()),
        // Additive (§20.5): a run whose launch phase is in flight, or whose
        // launch phase a crash left unaccounted for, is a state a client can
        // now see rather than infer from a gap between events.
        "reservation": run.as_ref().and_then(|r| r.reservation.clone()),
        "workflow": run.as_ref().and_then(|r| r.workflow.as_ref().map(|w| json!({
            "name": w.name,
            "version": w.version,
            "source": w.source,
            "stages": w.stages.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
        }))),
        "backend": run.as_ref().and_then(|r| r.backend.clone()),
        "route_source": run.as_ref().and_then(|r| r.route_source.clone()),
        "teardown": run.as_ref().and_then(|r| r.teardown.clone()),
        // §11.5's orthogonal axis (C5, mandatory): the disposition retirement
        // recorded, the §11.3 findings behind it, and §11.4's unattributed
        // estate drift. `null` until a retirement assessed it.
        "integrity": run.as_ref().and_then(integrity_view),
        // R-MVP1-2's sibling: named per repository once there is something to
        // point at.
        "output": work.and_then(|w| run.as_ref().and_then(|r| output_pointer(w, r))),
        // MVP-3's envelope-visibility item: how much of R-MVP1-7's turn
        // budget this Work has spent and how much it has, without decoding
        // the journal for `execution.started`/`stage.resumed` counts or the
        // submit-time override. `Engine::effective_turn_cap`/
        // `effective_turn_ceiling` are the exact formulas
        // `check_turn_envelope`/`due_interrupts` gate on, so this can never
        // show a budget the engine is not actually enforcing. `None` (no
        // run yet — e.g. a `pending` Work with no repository context) still
        // reports a real, honest cap/ceiling: zero turns spent against
        // whatever this Work would be checked against once it starts.
        "envelope": work.map(|w| {
            let default_run = WorkRun::default();
            let run_ref = run.as_ref().unwrap_or(&default_run);
            json!({
                "turns_spawned": run_ref.turns_spawned,
                "turn_cap": engine.effective_turn_cap(Some(w), run_ref),
                "turn_cap_bonus": run_ref.turn_cap_bonus,
                "turn_ceiling_secs": engine.effective_turn_ceiling(Some(w)).as_secs_f64(),
            })
        }),
    })
}

/// The current stage coordinate: which stage, where in the order, which
/// attempt, and what it is doing.
fn run_stage_view(run: &WorkRun) -> Option<Value> {
    let stage = run.current_stage()?;
    Some(json!({
        "stage_id": stage.stage_id,
        "index": stage.index,
        "attempt": stage.attempt,
        "status": stage.status.as_str(),
        "detail": stage.detail,
        "of": run.workflow.as_ref().map(|w| w.stages.len()),
        // Additive (§20.5, §13.3's "expose current-stage executor details in
        // API views"). A mixed-harness run whose clients can only see the
        // Work-level actor default cannot tell an operator which harness is
        // actually holding the current checkpoint — which is the whole point
        // of declaring one per stage.
        "executor": run.stage_binding(&stage.stage_id, stage.index),
    }))
}

/// The `GET /v1/work` body: every work, plus each one's stage coordinate.
///
/// The stage rides alongside the §10 record rather than inside it — flattening
/// them is exactly how "in review" becomes a state-machine value — but a fleet
/// view that cannot say which stage a running work is on is not a fleet view,
/// so `stage` is a sibling key of `work` here as it is in `work_view`.
fn fleet_body(core: &Core, engine: &Engine) -> Value {
    let registry = core.registry.state();
    let works: Vec<Value> = registry
        .works
        .values()
        .map(|work| {
                // `registry.run_view` alone only reaches the bounded
                // in-memory cache (`TERMINAL_RUN_CACHE_CAPACITY`); once a
                // terminal run ages out of it, `resolve_run`'s journal-
                // replay fallback is what `work_view` already relies on to
                // keep `sgt work show` correct for the identical work. This
                // row must use the same fallback, or an evicted work's
                // `state` here would silently fall back to plain
                // `completed` (ADR 0007(b)) while `work show` still says
                // `completed_dirty` for it.
            let run = resolve_run(core, work, registry.run_view(&work.id));
            let mut row = serde_json::to_value(work).unwrap_or(Value::Null);
            if let Some(object) = row.as_object_mut() {
                // ADR 0007(b): `sgt work list` is where an operator looks
                // first, and its plain `state` column must not read
                // `completed` for a closing stage that never actually
                // advanced the branch and left the worktree dirty.
                object.insert(
                    "state".to_string(),
                    json!(reported_state(work, run.as_ref())),
                );
                object.insert(
                    "stage".to_string(),
                    run.as_ref().and_then(run_stage_view).unwrap_or(Value::Null),
                );
                object.insert(
                    "resolved_backend".to_string(),
                    run.as_ref()
                        .and_then(|r| r.backend.clone())
                        .map_or(Value::Null, Value::String),
                );
                // C5: `sgt work list` is the default output a terminal-dirty
                // Work must be distinguishable in. `state` above already
                // carries it for a dirty *completion* (`completed_dirty`);
                // this is what carries it for a dirty `failed`/`canceled`,
                // whose state strings §11.5 leaves alone.
                object.insert(
                    "integrity".to_string(),
                    run.as_ref().and_then(integrity_view).unwrap_or(Value::Null),
                );
                // MVP-3's envelope-visibility item, folded onto the fleet
                // view exactly the way `work_view` folds it onto a single
                // work — `sgt status`/`sgt work list --json` see the same
                // turns-spent/cap/ceiling `sgt work show` does, no second
                // request per work required.
                let default_run = WorkRun::default();
                let run_ref = run.as_ref().unwrap_or(&default_run);
                object.insert(
                    "envelope".to_string(),
                    json!({
                        "turns_spawned": run_ref.turns_spawned,
                        "turn_cap": engine.effective_turn_cap(Some(work), run_ref),
                        "turn_cap_bonus": run_ref.turn_cap_bonus,
                        "turn_ceiling_secs": engine.effective_turn_ceiling(Some(work)).as_secs_f64(),
                    }),
                );
            }
            row
        })
        .collect();
    json!({"works": works})
}

#[derive(Debug, Deserialize)]
struct PauseAdmissionRequest {
    command_id: String,
}

/// `POST /v1/admission/pause` — MVP-3's admission drain flag, scoped exactly
/// to `sgt daemon stop`: refuse every new `POST /v1/work` submission from
/// this point on ([`submit_work`]'s own `admission_paused` check) until a
/// fresh daemon process resumes it at startup (`daemon.rs`'s `start_with`,
/// `KIND_ADMISSION_RESUMED`'s doc has the L6 crash-window argument for why
/// resume is startup-only rather than a second endpoint here).
///
/// **Idempotent, not merely retried.** A `daemon stop` that finds admission
/// already paused (its own earlier attempt, or a concurrent one) journals
/// nothing new — the `bool` this folds into is already true, and appending a
/// second `admission.paused` would answer "is admission paused" no more
/// truthfully while making the journal's pause/resume pairing look like it
/// happened twice. This is what keeps a `daemon stop` retry from
/// "double-pausing incoherently", the other half of the L6 argument this
/// milestone's task names.
async fn pause_admission(
    State(state): State<ApiState>,
    body: Result<Json<PauseAdmissionRequest>, JsonRejection>,
) -> Response {
    let req = match parse_body(body) {
        Ok(r) => r,
        Err(resp) => return *resp,
    };
    if let Err(resp) = parse_command_id(&req.command_id) {
        return *resp;
    }
    let mut core = CoreGuard::acquire(&state.core).await;
    if let Some(resp) = replay_command(&core, &req.command_id) {
        return resp;
    }
    if core.registry.state().admission_paused {
        let result = json!({"admission": {"paused": true}});
        return record_and_respond(
            &mut core,
            &req.command_id,
            "admission.pause",
            None,
            StatusCode::OK,
            result,
        );
    }
    let mut draft = EventDraft::new(api_source(), KIND_ADMISSION_PAUSED, json!({}));
    draft.correlation_id = Some(req.command_id.clone());
    if let Err(e) = core.commit(draft) {
        return internal_error(e);
    }
    let result = json!({"admission": {"paused": true}});
    record_and_respond(
        &mut core,
        &req.command_id,
        "admission.pause",
        None,
        StatusCode::OK,
        result,
    )
}

/// `GET /v1/work` — list all work (ULID key order = submission order).
async fn list_work(State(state): State<ApiState>) -> Response {
    let core = CoreGuard::acquire(&state.core).await;
    Json(fleet_body(&core, &state.engine)).into_response()
}

#[derive(Debug, Deserialize)]
struct WorkflowsQuery {
    /// The client's working directory — workspace discovery (§9) starts
    /// here, exactly as [`Origin::cwd`] does for `POST /v1/work`, so what
    /// this route shows as available matches what submission would actually
    /// resolve (§19.4's "cwd discovery matches submission").
    cwd: String,
}

/// One `CatalogEntry.stages[]` element (§11.2's StageEntry): order, kind,
/// declared harness/profile, and whether the stage asks — nothing else.
/// `context` (a stage's full prompt) and `execute` (the pinned container
/// spec) are deliberately never serialized here; no current TUI surface
/// needs either (§11.2, §11.3, §6.2's capability-matrix exclusion).
fn stage_entry_json(stage: &crate::domain::workflow::StageDefinition) -> Value {
    json!({
        "id": stage.id,
        "kind": stage.kind.as_str(),
        "harness": stage.harness,
        "profile": stage.profile,
        "requires_ask": stage.requires_ask,
    })
}

/// One `workflows[]` element (§11.2's CatalogEntry): `WorkflowDefinition`
/// fields verbatim, plus `status`/`description`/`tags` from the workflow's
/// own `index.md` front matter when it has one — omitted entirely (not
/// `null`, not `[]`) for the embedded fallback, which has none.
fn catalog_entry_json(entry: &workflow::CatalogEntry) -> Value {
    let mut value = json!({
        "name": entry.definition.name,
        "version": entry.definition.version,
        "source": entry.definition.source,
        "content_hash": entry.definition.content_hash,
        "stages": entry.definition.stages.iter().map(stage_entry_json).collect::<Vec<_>>(),
    });
    if let Some(fm) = &entry.front_matter {
        let object = value.as_object_mut().expect("built as an object above");
        object.insert("status".to_string(), json!(fm.status));
        object.insert("description".to_string(), json!(fm.description));
        if let Some(tags) = &fm.tags {
            object.insert("tags".to_string(), json!(tags));
        }
    }
    value
}

/// The catalog `GET /v1/workflows` answers with, for a resolved `cwd`
/// (§11.2, Decisions T2-39/T2-40): the repository's own root-indexed
/// published workflows when one resolves and has any, else the embedded
/// `software-change` fallback, else (only when the embedded fallback itself
/// fails to load) an empty list — fails closed per §19.4, never a `4xx` for
/// this shape.
fn workflow_catalog_entries(cwd: &std::path::Path, data_dir: &std::path::Path) -> Vec<Value> {
    let repo_entries = match Workspace::discover_scoped(cwd, Some(data_dir)) {
        Ok(workspace) => workflow::catalog(&workspace.root),
        Err(_) => Vec::new(),
    };
    let entries = if !repo_entries.is_empty() {
        repo_entries
    } else {
        match WorkflowDefinition::embedded() {
            Ok(definition) => vec![workflow::CatalogEntry {
                definition,
                front_matter: None,
            }],
            Err(_) => Vec::new(),
        }
    };
    entries.iter().map(catalog_entry_json).collect()
}

/// `GET /v1/workflows?cwd=<percent-encoded path>` — the read-only workflow
/// catalog (§11.2): what the client's own submission could bind now. Reuses
/// the same workspace discovery [`submit_work`]'s plan does, the same
/// workflow loader and validation, the root publication boundary
/// (§11.1, [`workflow::catalog`]), and the same embedded fallback. Performs
/// no mutation, holds no core lock (nothing here reads or writes registry
/// state), and appends no event.
async fn list_workflows(
    State(state): State<ApiState>,
    query: Result<Query<WorkflowsQuery>, QueryRejection>,
) -> Response {
    let req = match parse_query(query) {
        Ok(r) => r,
        Err(resp) => return *resp,
    };
    let cwd = PathBuf::from(&req.cwd);
    if !cwd.is_absolute() {
        return error_response(
            StatusCode::BAD_REQUEST,
            "invalid_cwd",
            "cwd must be an absolute path",
        );
    }
    let data_dir = state.data_dir.clone();
    let workflows = blocking(move || workflow_catalog_entries(&cwd, &data_dir)).await;
    Json(json!({"workflows": workflows})).into_response()
}

/// `GET /v1/work/{id}` — one work record, with its stage, surface and
/// execution state (the M3 contract's `work show` surface).
async fn show_work(State(state): State<ApiState>, Path(id): Path<String>) -> Response {
    let core = CoreGuard::acquire(&state.core).await;
    match core.registry.state().works.get(&id) {
        Some(_) => Json(work_view(&core, &state.engine, &id)).into_response(),
        None => error_response(
            StatusCode::NOT_FOUND,
            "work_not_found",
            format!("no work with id {id}"),
        ),
    }
}

/// `GET /v1/work/{id}/transcript` — MVP-3's `sgt work transcript`: the
/// work's conversation, decoded into causal (journal seq) order.
///
/// Every `conversation.*` event already carries its content inline in the
/// payload (`text`/`question` — see `claude.rs`'s `ingest_line`), so
/// reconstructing the transcript is filtering the journal to this work's
/// conversation kinds and reading their causal order straight off — no new
/// projection, no daemon-internal type crosses this boundary (R-NS-4: a
/// client convenience over data the journal already owns).
///
/// The one gap: a turn that ended with **no** result envelope (interrupted
/// or crashed — see `TurnReader::run`) never got as far as emitting
/// `conversation.assistant.completed`, because that event is only produced
/// from a fully-parsed `assistant` stream-json line. For that turn alone,
/// the §20 raw archive `conversation.turn.ended` references by blob ref is
/// the *only* place any of its content reached the journal at all, so this
/// handler does the "minimal blob decode" MVP-3's plan calls for: split the
/// archive into lines, parse each as JSON, and recover whatever assistant
/// text blocks streamed before the cut. It deliberately does not replay tool
/// calls or system/vendor plumbing — that stays raw-archive-only.
///
/// §22.6 tradeoff, disclosed rather than hidden: `events_after(0)` below
/// runs a full from-seq-0 journal replay while `core` — the exclusive
/// `CoreGuard` — is still held. `blocking_sync` only keeps the tokio
/// scheduler's other, guard-independent tasks off this worker thread; it
/// does not release the guard itself, so every call here still queues
/// every other Core-guarded request (submit, cancel, retry, input,
/// `/v1/system`) for the full replay duration, exactly as `events_after`'s
/// own doc comment says every caller must expect. `resolve_run`'s
/// `terminal_runs` cache accepts the identical shape only as a rare,
/// capacity-bounded cache-miss fallback; there is no equivalent bound
/// here, because a work's conversation history is unbounded and not
/// capped by any terminal-state cache. Closing this for good needs a
/// journal reader the core does not own — the same named follow-up
/// `resolve_run` already defers, not this build's.
async fn work_transcript(State(state): State<ApiState>, Path(id): Path<String>) -> Response {
    let core = CoreGuard::acquire(&state.core).await;
    if !core.registry.state().works.contains_key(&id) {
        return error_response(
            StatusCode::NOT_FOUND,
            "work_not_found",
            format!("no work with id {id}"),
        );
    }
    let events = match blocking_sync(|| core.events_after(0)) {
        Ok(events) => events,
        Err(e) => return internal_error(e),
    };
    drop(core);

    let turns = transcript_turns(&id, events, &state.data_dir);
    Json(json!({"work_id": id, "turns": turns})).into_response()
}

/// The pure decode: filter `events` to `work_id`'s `conversation.*` kinds and
/// turn each into a `{seq, ts, role, text, source}` entry, in the journal's
/// own causal (seq) order — factored out of the handler above so the
/// role/source mapping and the blob-decode fallback can be pinned by a
/// direct test without spinning up a daemon.
fn transcript_turns(work_id: &str, events: Vec<Event>, data_dir: &std::path::Path) -> Vec<Value> {
    let mut turns = Vec::new();
    // Per-`execution_id` count of `conversation.assistant.completed` events
    // consumed since the last turn boundary — `ingest_line` emits exactly one
    // such event per successfully-parsed `assistant` stream-json line that
    // carried text, so a turn can legitimately emit several before ending
    // (or crashing) and the archive's own per-line decode (`decode_partial_
    // assistant_lines`) produces the same count of entries in the same
    // order. The blob-decode fallback below skips that many leading lines
    // from the archive rather than an all-or-nothing flag, so a crash that
    // lands between two of a turn's own assistant-line appends (docs/DEVELOPMENT.md's
    // "adjacent-append crash window") still recovers the lines that never
    // reached the journal, instead of either double-reporting the ones that
    // did or silently dropping the ones that didn't. Removed on consumption
    // so the next turn on the same execution starts fresh.
    let mut assistant_completed_since_last_turn: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for event in events
        .into_iter()
        .filter(|e| e.work_id.as_deref() == Some(work_id))
    {
        match event.kind.as_str() {
            KIND_CONVERSATION_USER => turns.push(json!({
                "seq": event.seq,
                "ts": event.timestamp,
                "role": "user",
                "text": event.payload["text"].as_str().unwrap_or(""),
                "source": "event",
            })),
            KIND_CONVERSATION_ASSISTANT_COMPLETED => {
                if let Some(execution_id) = &event.execution_id {
                    *assistant_completed_since_last_turn
                        .entry(execution_id.clone())
                        .or_insert(0) += 1;
                }
                turns.push(json!({
                    "seq": event.seq,
                    "ts": event.timestamp,
                    "role": "assistant",
                    "text": event.payload["text"].as_str().unwrap_or(""),
                    "source": "event",
                }));
            }
            KIND_CONVERSATION_ASK => turns.push(json!({
                "seq": event.seq,
                "ts": event.timestamp,
                "role": "ask",
                "text": event.payload["question"].as_str().unwrap_or(""),
                "source": "event",
            })),
            KIND_CONVERSATION_TURN_ENDED => {
                // This turn's own boundary: whatever `assistant.completed`
                // this execution emitted belongs to *this* turn (the two are
                // always emitted by the same `TurnReader` run), so consuming
                // it here — whether or not it changes what happens below —
                // keeps the count scoped to "since the last turn", not
                // "ever".
                let already_emitted_lines = event
                    .execution_id
                    .as_ref()
                    .and_then(|id| assistant_completed_since_last_turn.remove(id))
                    .unwrap_or(0);
                // A turn that closed with a result envelope already emitted
                // its content, if any, as its own `conversation.*` event(s)
                // above — nothing to recover. Only the envelope-less case
                // needs the archive.
                if event.payload["result_envelope"].as_bool().unwrap_or(true) {
                    continue;
                }
                let Some(raw_ref) = event.payload["raw"].as_str() else {
                    continue;
                };
                let Ok(blob_ref) = raw_ref.parse::<crate::runtime::blob::BlobRef>() else {
                    continue;
                };
                let Ok(store) = crate::runtime::blob::BlobStore::open(data_dir) else {
                    continue;
                };
                let Ok(bytes) = store.get(&blob_ref) else {
                    continue;
                };
                // Skip exactly the lines this execution already reported
                // live; only genuinely unreported lines are recovered.
                let text: String = decode_partial_assistant_lines(&bytes)
                    .into_iter()
                    .skip(already_emitted_lines)
                    .collect();
                if !text.is_empty() {
                    turns.push(json!({
                        "seq": event.seq,
                        "ts": event.timestamp,
                        "role": "assistant",
                        "text": text,
                        "source": "blob_decode",
                        "interrupted": event.payload["interrupted"].as_bool().unwrap_or(false),
                    }));
                }
            }
            _ => {}
        }
    }
    turns
}

/// The "minimal blob decode" itself: recover whatever assistant `text`
/// content blocks appear in a raw stream-json archive, one entry per
/// `assistant` line that carried text — the same granularity `ingest_line`'s
/// `Some("assistant")` arm reads a live line at (one `conversation.assistant.
/// completed` event per such line), so callers can line an archive's entries
/// up against how many of them already reached the journal live. Not a
/// general stream-json parser. Lines with no text block (tool-only, or
/// unparseable) contribute no entry, matching `ingest_line`'s own
/// `!text.is_empty()` gate on emitting the event.
fn decode_partial_assistant_lines(raw: &[u8]) -> Vec<String> {
    let raw = String::from_utf8_lossy(raw);
    let mut lines_text = Vec::new();
    for line in raw.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if value.get("type").and_then(Value::as_str) != Some("assistant") {
            continue;
        }
        let Some(blocks) = value.pointer("/message/content").and_then(Value::as_array) else {
            continue;
        };
        let mut text = String::new();
        for block in blocks {
            if block.get("type").and_then(Value::as_str) == Some("text")
                && let Some(t) = block.get("text").and_then(Value::as_str)
            {
                text.push_str(t);
            }
        }
        if !text.is_empty() {
            lines_text.push(text);
        }
    }
    lines_text
}

/// The full-archive convenience `decode_partial_assistant_lines` factors
/// out of: every recovered line's text, concatenated in file order.
#[cfg(test)]
fn decode_partial_assistant_text(raw: &[u8]) -> String {
    decode_partial_assistant_lines(raw).concat()
}

#[derive(Debug, Deserialize)]
struct CancelRequest {
    command_id: String,
}

/// `POST /v1/work/{id}/cancel` — request cancellation.
///
/// `pending → canceled` appends `work.canceled`; canceling an
/// already-canceled work is an idempotent no-op success (no state
/// transition, no `work.canceled` event); unknown ids and illegal
/// transitions are structured, journaled rejections.
async fn cancel_work(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    body: Result<Json<CancelRequest>, JsonRejection>,
) -> Response {
    let req = match parse_body(body) {
        Ok(r) => r,
        Err(resp) => return *resp,
    };
    if let Err(resp) = parse_command_id(&req.command_id) {
        return *resp;
    }
    let mut core = CoreGuard::acquire(&state.core).await;
    if let Some(resp) = replay_command(&core, &req.command_id) {
        return resp;
    }
    let Some(work) = core.registry.state().works.get(&id).cloned() else {
        let result = error_body("work_not_found", format!("no work with id {id}"));
        return record_and_respond(
            &mut core,
            &req.command_id,
            "work.cancel",
            None,
            StatusCode::NOT_FOUND,
            result,
        );
    };
    if work.state == WorkState::Canceled {
        // Duplicate cancel: idempotent success, no new transition event.
        let result = json!({"work": work});
        return record_and_respond(
            &mut core,
            &req.command_id,
            "work.cancel",
            Some(&id),
            StatusCode::OK,
            result,
        );
    }
    if !work.state.can_transition(WorkState::Canceled) {
        let result = error_body(
            "illegal_transition",
            format!("cannot cancel work in state {}", work.state),
        );
        return record_and_respond(
            &mut core,
            &req.command_id,
            "work.cancel",
            Some(&id),
            StatusCode::CONFLICT,
            result,
        );
    }
    let mut draft = EventDraft::new(
        api_source(),
        KIND_WORK_CANCELED,
        json!({"from": work.state}),
    )
    .with_work_id(&id);
    draft.correlation_id = Some(req.command_id.clone());
    if let Err(e) = core.commit(draft) {
        return internal_error(e);
    }
    // Work state changes first, then the run is retired: the cancellation is
    // a fact about the Work, not a request to the backend, and it does not
    // wait for — or depend on — the native context actually dying (§25).
    //
    // The STOP request and the teardown land under this guard; the adapter's
    // evidence tail (issue #14/B3) is awaited by `crank` after it is dropped.
    match state
        .engine
        .begin_retire_run(&mut core, &id, "work canceled")
    {
        Ok(step) => {
            // The authoritative half is journaled; make it durable before
            // the guard is dropped and the external effect runs (#44). A
            // group commit that fails here is still reportable — after this
            // point the client's command has been accepted and the only
            // honest answer is a log line.
            if let Err(e) = core.flush() {
                return internal_error(e);
            }
            drop(core);
            crank(&state, step).await;
            core = CoreGuard::acquire(&state.core).await;
        }
        Err(e) => tracing::warn!(work_id = %id, error = %e, "retiring the canceled run failed"),
    }
    let result = work_view(&core, &state.engine, &id);
    record_and_respond(
        &mut core,
        &req.command_id,
        "work.cancel",
        Some(&id),
        StatusCode::OK,
        result,
    )
}

#[derive(Debug, Deserialize)]
struct InputRequest {
    command_id: String,
    input: String,
}

/// `POST /v1/work/{id}/input` — answer a work that asked for input (§12's
/// needs-input verb; `work.respond` in §26's command vocabulary).
async fn work_input(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    body: Result<Json<InputRequest>, JsonRejection>,
) -> Response {
    let req = match parse_body(body) {
        Ok(r) => r,
        Err(resp) => return *resp,
    };
    if let Err(resp) = parse_command_id(&req.command_id) {
        return *resp;
    }
    let mut core = CoreGuard::acquire(&state.core).await;
    if let Some(resp) = replay_command(&core, &req.command_id) {
        return resp;
    }
    if !core.registry.state().works.contains_key(&id) {
        let result = error_body("work_not_found", format!("no work with id {id}"));
        return record_and_respond(
            &mut core,
            &req.command_id,
            "work.respond",
            None,
            StatusCode::NOT_FOUND,
            result,
        );
    }
    let engine = state.engine.clone();
    match engine.begin_input(&mut core, &id, &req.input) {
        Ok(step) => {
            // The authoritative half is journaled; make it durable before
            // the guard is dropped and the external effect runs (#44). A
            // group commit that fails here is still reportable — after this
            // point the client's command has been accepted and the only
            // honest answer is a log line.
            if let Err(e) = core.flush() {
                return internal_error(e);
            }
            drop(core);
            let mut core = relock(&state, crank(&state, step).await).await;
            let result = work_view(&core, &state.engine, &id);
            record_and_respond(
                &mut core,
                &req.command_id,
                "work.respond",
                Some(&id),
                StatusCode::OK,
                result,
            )
        }
        Err(e) => {
            let status = engine_error_status(&e);
            let result = engine_error_body(&e);
            record_and_respond(
                &mut core,
                &req.command_id,
                "work.respond",
                Some(&id),
                status,
                result,
            )
        }
    }
}

#[derive(Debug, Deserialize)]
struct RetryRequest {
    command_id: String,
}

/// `POST /v1/work/{id}/retry` — re-enter the current stage (§12's retry verb).
async fn work_retry(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    body: Result<Json<RetryRequest>, JsonRejection>,
) -> Response {
    let req = match parse_body(body) {
        Ok(r) => r,
        Err(resp) => return *resp,
    };
    if let Err(resp) = parse_command_id(&req.command_id) {
        return *resp;
    }
    let mut core = CoreGuard::acquire(&state.core).await;
    if let Some(resp) = replay_command(&core, &req.command_id) {
        return resp;
    }
    if !core.registry.state().works.contains_key(&id) {
        let result = error_body("work_not_found", format!("no work with id {id}"));
        return record_and_respond(
            &mut core,
            &req.command_id,
            "work.retry",
            None,
            StatusCode::NOT_FOUND,
            result,
        );
    }
    let engine = state.engine.clone();
    match engine.begin_retry(&mut core, &id) {
        Ok(step) => {
            // The authoritative half is journaled; make it durable before
            // the guard is dropped and the external effect runs (#44). A
            // group commit that fails here is still reportable — after this
            // point the client's command has been accepted and the only
            // honest answer is a log line.
            if let Err(e) = core.flush() {
                return internal_error(e);
            }
            drop(core);
            let mut core = relock(&state, crank(&state, step).await).await;
            let result = work_view(&core, &state.engine, &id);
            record_and_respond(
                &mut core,
                &req.command_id,
                "work.retry",
                Some(&id),
                StatusCode::OK,
                result,
            )
        }
        Err(e) => {
            let status = engine_error_status(&e);
            let result = engine_error_body(&e);
            record_and_respond(
                &mut core,
                &req.command_id,
                "work.retry",
                Some(&id),
                status,
                result,
            )
        }
    }
}

#[derive(Debug, Deserialize)]
struct ExtendRequest {
    command_id: String,
    additional_turns: u32,
}

/// `POST /v1/work/{id}/extend` — R-MVP1-10's exit door for R-MVP1-7's
/// envelope-exhausted landing (`Engine::extend_turn_envelope`). A pure
/// commit, unlike `retry`: raising the envelope has no external effect of
/// its own, so there is nothing to crank — `retry` afterward is what
/// actually re-enters the stage.
async fn work_extend(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    body: Result<Json<ExtendRequest>, JsonRejection>,
) -> Response {
    let req = match parse_body(body) {
        Ok(r) => r,
        Err(resp) => return *resp,
    };
    if let Err(resp) = parse_command_id(&req.command_id) {
        return *resp;
    }
    let mut core = CoreGuard::acquire(&state.core).await;
    if let Some(resp) = replay_command(&core, &req.command_id) {
        return resp;
    }
    if !core.registry.state().works.contains_key(&id) {
        let result = error_body("work_not_found", format!("no work with id {id}"));
        return record_and_respond(
            &mut core,
            &req.command_id,
            "work.extend",
            None,
            StatusCode::NOT_FOUND,
            result,
        );
    }
    if req.additional_turns == 0 {
        // A zero extension journals an event that changes nothing and hands
        // the operator a 200 whose follow-up retry re-blocks on the identical
        // envelope — a client bug answered as a success. Same shape as
        // submit's empty-intent rejection: journaled under this command_id so
        // a retry replays the 400.
        let result = error_body("invalid_request", "additional_turns must be at least 1");
        return record_and_respond(
            &mut core,
            &req.command_id,
            "work.extend",
            Some(&id),
            StatusCode::BAD_REQUEST,
            result,
        );
    }
    let engine = state.engine.clone();
    match engine.extend_turn_envelope(&mut core, &id, req.additional_turns) {
        Ok(()) => {
            let result = work_view(&core, &state.engine, &id);
            record_and_respond(
                &mut core,
                &req.command_id,
                "work.extend",
                Some(&id),
                StatusCode::OK,
                result,
            )
        }
        Err(e) => {
            let status = engine_error_status(&e);
            let result = engine_error_body(&e);
            record_and_respond(
                &mut core,
                &req.command_id,
                "work.extend",
                Some(&id),
                status,
                result,
            )
        }
    }
}

#[derive(Debug, Deserialize)]
struct ReapRequest {
    command_id: String,
    /// Must be `true`, or the request is refused (#109: reap has no undo,
    /// so intent is never guessed).
    #[serde(default)]
    confirm: bool,
}

/// `POST /v1/work/{id}/reap` — #109's dispose verb: permanently discard
/// whatever `RetainedDirty` bindings still hold for this Work's teardown
/// (`crate::runtime::surface::reap`'s own doc has the full contract — the
/// captured patch, or the submodule/capture-failure fallback directory;
/// never the retained branch, which stays outside anything this verb can
/// reach).
///
/// Refuses without `{"confirm": true}` in the body: unlike `cancel`/
/// `retry`/`extend`, there is no journaled state this can safely undo by
/// resubmitting a different command, so a bare or malformed body is
/// ambiguity, not an implicit yes (`AGENTS.md`'s fail-closed rule). An
/// operator reviews `GET /v1/retained` first — that is the read half of
/// this same pair — then re-sends with confirmation.
async fn reap_work(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    body: Result<Json<ReapRequest>, JsonRejection>,
) -> Response {
    let req = match parse_body(body) {
        Ok(r) => r,
        Err(resp) => return *resp,
    };
    if let Err(resp) = parse_command_id(&req.command_id) {
        return *resp;
    }
    let mut core = CoreGuard::acquire(&state.core).await;
    if let Some(resp) = replay_command(&core, &req.command_id) {
        return resp;
    }
    if !req.confirm {
        let result = error_body(
            "confirmation_required",
            "reap destroys retained dirty state with no undo; review GET /v1/retained for \
             this work, then resend with \"confirm\": true",
        );
        return record_and_respond(
            &mut core,
            &req.command_id,
            "work.reap",
            Some(&id),
            StatusCode::BAD_REQUEST,
            result,
        );
    }
    let Some(work) = core.registry.state().works.get(&id).cloned() else {
        let result = error_body("work_not_found", format!("no work with id {id}"));
        return record_and_respond(
            &mut core,
            &req.command_id,
            "work.reap",
            None,
            StatusCode::NOT_FOUND,
            result,
        );
    };
    let run = resolve_run(&core, &work, core.registry.state().run_view(&id));
    let bound = run.and_then(|r| match (r.surface.clone(), r.teardown.clone()) {
        (Some(surface), Some(teardown)) => Some((surface, teardown)),
        _ => None,
    });
    let Some((surface, teardown)) = bound else {
        let result = error_body(
            "nothing_to_reap",
            format!("work {id} has no recorded teardown to reap"),
        );
        return record_and_respond(
            &mut core,
            &req.command_id,
            "work.reap",
            Some(&id),
            StatusCode::NOT_FOUND,
            result,
        );
    };
    // §22.6 tradeoff, same disclosure as `work_transcript`'s: this runs git
    // and filesystem calls (potentially a real, if small, worktree removal)
    // while the core guard is held, queueing every other guarded request for
    // its duration. Reap is rare and explicit, never on a hot path, so this
    // is accepted rather than plumbed through the async effect system the
    // way `teardown` itself is.
    // `state.data_dir` is this daemon's own storage — where §9.4's
    // interprocess repository locks live. Reap's one guarded span (a `git
    // worktree remove --force` on a retained-dirty binding) takes the same
    // lock every other registry mutation in the surface module does.
    let report = blocking_sync(|| reap(&state.data_dir, &surface, &teardown));
    let result = json!({"report": report});
    record_and_respond(
        &mut core,
        &req.command_id,
        "work.reap",
        Some(&id),
        StatusCode::OK,
        result,
    )
}

/// `GET /v1/retained` — #109's inspect verb: every repository binding any
/// terminal Work's teardown left something on disk for, across the whole
/// estate — a captured patch, the submodule/capture-failure fallback
/// directory, or a `RetainedError`. Correct retention is otherwise
/// indistinguishable from a leak (#109); this is the read side of that
/// pair, `POST /v1/work/{id}/reap` the write side.
///
/// Same `resolve_run` journal-replay fallback `fleet_body`/`work_view`
/// already rely on for an evicted terminal work — and the same accepted
/// tradeoff `work_transcript` discloses: a terminal work not already in the
/// bounded cache costs a full journal replay under the guard. Inspecting
/// retained state is an occasional, operator-driven action, not a hot path.
async fn list_retained(State(state): State<ApiState>) -> Response {
    let core = CoreGuard::acquire(&state.core).await;
    let registry = core.registry.state();
    let entries: Vec<Value> = registry
        .works
        .values()
        .filter_map(|work| {
            let run = resolve_run(&core, work, registry.run_view(&work.id))?;
            let teardown = run.teardown?;
            Some((work.id.clone(), retained_bindings(&teardown)))
        })
        .flat_map(|(work_id, bindings)| {
            bindings.into_iter().map(move |b| {
                json!({
                    "work_id": work_id,
                    "repository": b.repository,
                    "path": b.path,
                    "disposition": b.reason,
                    "detail": b.detail,
                    "bytes": b.bytes,
                })
            })
        })
        .collect();
    Json(json!({"retained": entries})).into_response()
}

// ------------------------------------------------------------------ estate
//
// §16.2/§16.3: thin daemon-side wrappers over `crate::domain::manifest`'s
// existing add_repo/remove_repo/add_group/remove_group and
// `crate::cli::doctor`'s existing Report — no logic is duplicated here, and
// none of it touches the journal or `Core`: manifest edits are not engine
// state (R-NS-4, `src/domain/manifest.rs`'s own module doc), so unlike every
// other mutation in this file there is no `command_id`/replay pair.

/// The estate root these routes edit: an upward walk from this daemon's own
/// `data_dir`, unscoped — correct for the default `<estate_root>/.sergeant/
/// data` layout `sgt init` always produces, deterministic, and dependent on
/// nothing but the one fact [`ApiState`] already carries.
///
/// The process's own working directory is deliberately **not** consulted —
/// a long-running daemon's cwd is not reliably the estate root in every real
/// deployment, and unlike a walk that simply finds nothing, a cwd that
/// happens to sit inside a *different* estate (an unrelated git checkout
/// that is itself one, `sgt`'s own self-hosting shape among them) would
/// silently resolve to the wrong manifest rather than failing closed — the
/// one outcome R-NS-4's discipline exists to prevent.
fn resolve_estate_root(data_dir: &std::path::Path) -> Result<PathBuf, Box<Response>> {
    match Workspace::estate_root(data_dir, None) {
        Ok(Some(root)) => Ok(root),
        Ok(None) => Err(Box::new(error_response(
            StatusCode::NOT_FOUND,
            "no_estate",
            "no estate found walking up from this daemon's data dir (bounded at $HOME) — run \
             `sgt init` first, or start the daemon on a data dir under the estate root",
        ))),
        Err(e) => Err(Box::new(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "estate_invalid",
            e.to_string(),
        ))),
    }
}

/// Stable per-variant code for a [`ManifestError`], for the same
/// `{"error": {"code", "message"}}` shape every other route answers with.
fn manifest_error_code(e: &ManifestError) -> &'static str {
    match e {
        ManifestError::Io { .. } => "io",
        ManifestError::Parse { .. } => "parse",
        ManifestError::Locked { .. } => "locked",
        ManifestError::Invalid { .. } => "invalid",
        ManifestError::NoEstate { .. } => "no_estate",
        ManifestError::InvalidName { .. } => "invalid_name",
        ManifestError::RepoAlreadyDeclared { .. } => "repo_already_declared",
        ManifestError::RepoNotDeclared { .. } => "repo_not_declared",
        ManifestError::RepoInUseByGroups { .. } => "repo_in_use_by_groups",
        ManifestError::GroupNotDeclared { .. } => "group_not_declared",
        ManifestError::NotAGroupMember { .. } => "not_a_group_member",
        ManifestError::ExistingPathNotAGitRepository { .. } => "existing_path_not_a_git_repository",
        ManifestError::NoPathAndNoOrigin { .. } => "no_path_and_no_origin",
        ManifestError::CloneFailed { .. } => "clone_failed",
        ManifestError::MalformedSection { .. } => "malformed_section",
    }
}

/// HTTP status for a [`ManifestError`]: 4xx where the caller can fix the
/// request (a bad name, a dangling reference, a missing declaration), 502
/// for a failed outbound clone, 409 for a concurrent-edit or already-exists
/// conflict, 500 only where the local filesystem itself is unavailable.
fn manifest_error_status(e: &ManifestError) -> StatusCode {
    match e {
        ManifestError::Io { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        ManifestError::Parse { .. }
        | ManifestError::Invalid { .. }
        | ManifestError::ExistingPathNotAGitRepository { .. }
        | ManifestError::MalformedSection { .. } => StatusCode::UNPROCESSABLE_ENTITY,
        ManifestError::Locked { .. } | ManifestError::RepoAlreadyDeclared { .. } => {
            StatusCode::CONFLICT
        }
        ManifestError::NoEstate { .. }
        | ManifestError::RepoNotDeclared { .. }
        | ManifestError::GroupNotDeclared { .. }
        | ManifestError::NotAGroupMember { .. } => StatusCode::NOT_FOUND,
        ManifestError::InvalidName { .. } | ManifestError::NoPathAndNoOrigin { .. } => {
            StatusCode::BAD_REQUEST
        }
        ManifestError::RepoInUseByGroups { .. } => StatusCode::CONFLICT,
        ManifestError::CloneFailed { .. } => StatusCode::BAD_GATEWAY,
    }
}

/// The error body a [`ManifestError`] answers with — the exact refusal text
/// (remedy included, per its own `Display`) `sgt repo`/`sgt group` already
/// print, never a second explanation of the same defect.
fn manifest_error_response(e: &ManifestError) -> Response {
    error_response(
        manifest_error_status(e),
        manifest_error_code(e),
        e.to_string(),
    )
}

/// The declared repositories, read the same way `sgt repo list --json` does.
fn workspace_read(estate_root: &std::path::Path) -> Result<Workspace, Box<Response>> {
    Workspace::from_config_allow_empty(&estate_root.join(crate::domain::workspace::WORKSPACE_FILE))
        .map_err(|e| {
            Box::new(error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "estate_invalid",
                e.to_string(),
            ))
        })
}

/// `GET /v1/estate/repos` (§16.2) — the same read `sgt repo list --json`
/// already performs, over the daemon's own estate.
async fn estate_list_repos(State(state): State<ApiState>) -> Response {
    let estate_root = match resolve_estate_root(&state.data_dir) {
        Ok(root) => root,
        Err(resp) => return *resp,
    };
    let workspace = match workspace_read(&estate_root) {
        Ok(w) => w,
        Err(resp) => return *resp,
    };
    let repos: Vec<Value> = workspace
        .repositories
        .iter()
        .map(|r| {
            json!({
                "name": r.name,
                "path": r.path,
                "origin": workspace.repository_origin(&r.name),
                "instructions": workspace.instruction_policy(&r.name).as_str(),
            })
        })
        .collect();
    Json(json!({"repos": repos})).into_response()
}

#[derive(Debug, Deserialize)]
struct AddRepoRequest {
    name: String,
    #[serde(default)]
    origin: Option<String>,
    #[serde(default)]
    instructions: Option<String>,
}

/// `POST /v1/estate/repos` (§16.2) — `manifest::add_repo` exactly, including
/// its populate-or-verify clone behavior, which is why this runs off the
/// blocking pool rather than inline (§22.6's tradeoff, same shape as
/// `reap_work`'s filesystem call): a real `git clone` can take real time,
/// and nothing here holds the core guard while it runs — this route never
/// touches `Core` at all.
async fn estate_add_repo(
    State(state): State<ApiState>,
    body: Result<Json<AddRepoRequest>, JsonRejection>,
) -> Response {
    let req = match parse_body(body) {
        Ok(r) => r,
        Err(resp) => return *resp,
    };
    let instructions = match req.instructions.as_deref() {
        None => None,
        Some("local") => Some(InstructionPolicy::Local),
        Some("suppress") => Some(InstructionPolicy::Suppress),
        Some(other) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                format!("instructions {other:?} is not recognized (use \"local\" or \"suppress\")"),
            );
        }
    };
    let estate_root = match resolve_estate_root(&state.data_dir) {
        Ok(root) => root,
        Err(resp) => return *resp,
    };
    let name = req.name.clone();
    let origin = req.origin.clone();
    let result =
        blocking_sync(|| manifest::add_repo(&estate_root, &name, origin.as_deref(), instructions));
    match result {
        Ok(()) => (
            StatusCode::CREATED,
            Json(json!({
                "name": req.name,
                "path": format!("repos/{}", req.name),
                "origin": req.origin,
                "instructions": req.instructions,
            })),
        )
            .into_response(),
        Err(e) => manifest_error_response(&e),
    }
}

/// `DELETE /v1/estate/repos/{name}` (§16.2) — `manifest::remove_repo`
/// exactly; the group-reference refusal it returns reaches the caller
/// structured, not reworded.
async fn estate_remove_repo(State(state): State<ApiState>, Path(name): Path<String>) -> Response {
    let estate_root = match resolve_estate_root(&state.data_dir) {
        Ok(root) => root,
        Err(resp) => return *resp,
    };
    match blocking_sync(|| manifest::remove_repo(&estate_root, &name)) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => manifest_error_response(&e),
    }
}

/// `GET /v1/estate/groups` (§16.2) — the same read `sgt group list --json`
/// already performs.
async fn estate_list_groups(State(state): State<ApiState>) -> Response {
    let estate_root = match resolve_estate_root(&state.data_dir) {
        Ok(root) => root,
        Err(resp) => return *resp,
    };
    let workspace = match workspace_read(&estate_root) {
        Ok(w) => w,
        Err(resp) => return *resp,
    };
    let groups: Vec<Value> = workspace
        .groups
        .iter()
        .map(|(name, g)| json!({"name": name, "repos": g.repos, "brief": g.brief}))
        .collect();
    Json(json!({"groups": groups})).into_response()
}

#[derive(Debug, Deserialize)]
struct AddGroupRequest {
    name: String,
    #[serde(default)]
    repos: Vec<String>,
    #[serde(default)]
    brief: Option<String>,
}

/// `POST /v1/estate/groups` (§16.2) — `manifest::add_group` exactly,
/// including its mkdir-p create/extend semantics (creating an existing group
/// unions in new members; re-adding a member already present is a no-op).
async fn estate_add_group(
    State(state): State<ApiState>,
    body: Result<Json<AddGroupRequest>, JsonRejection>,
) -> Response {
    let req = match parse_body(body) {
        Ok(r) => r,
        Err(resp) => return *resp,
    };
    let estate_root = match resolve_estate_root(&state.data_dir) {
        Ok(root) => root,
        Err(resp) => return *resp,
    };
    let name = req.name.clone();
    let repos = req.repos.clone();
    let brief = req.brief.clone();
    let result =
        blocking_sync(|| manifest::add_group(&estate_root, &name, &repos, brief.as_deref()));
    match result {
        Ok(()) => {
            let members = match workspace_read(&estate_root) {
                Ok(w) => w
                    .groups
                    .get(&req.name)
                    .map(|g| g.repos.clone())
                    .unwrap_or_else(|| req.repos.clone()),
                Err(_) => req.repos.clone(),
            };
            Json(json!({"name": req.name, "repos": members, "brief": req.brief})).into_response()
        }
        Err(e) => manifest_error_response(&e),
    }
}

#[derive(Debug, Deserialize, Default)]
struct RemoveGroupRequest {
    #[serde(default)]
    repos: Vec<String>,
}

/// `DELETE /v1/estate/groups/{name}` (§16.2) — `manifest::remove_group`
/// exactly: an omitted or empty `repos` body removes the whole group,
/// otherwise only the named members (each of which must already be one).
async fn estate_remove_group(
    State(state): State<ApiState>,
    Path(name): Path<String>,
    body: axum::body::Bytes,
) -> Response {
    let repos = if body.is_empty() {
        Vec::new()
    } else {
        match serde_json::from_slice::<RemoveGroupRequest>(&body) {
            Ok(r) => r.repos,
            Err(e) => {
                return error_response(
                    StatusCode::BAD_REQUEST,
                    "invalid_request",
                    format!("invalid JSON body: {e}"),
                );
            }
        }
    };
    let estate_root = match resolve_estate_root(&state.data_dir) {
        Ok(root) => root,
        Err(resp) => return *resp,
    };
    match blocking_sync(|| manifest::remove_group(&estate_root, &name, &repos)) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => manifest_error_response(&e),
    }
}

/// `GET /v1/doctor` (§16.3) — the same `doctor::Report::to_json()`
/// `sgt doctor --json` already prints, computed exactly once here.
async fn doctor_report(State(state): State<ApiState>) -> Response {
    let report = doctor::run(&state.data_dir).await;
    Json(report.to_json()).into_response()
}

/// Catch the analytical projection up to the journal, then hand it to `f`.
///
/// The projection is folded lazily, at read time, rather than on every
/// commit. Three things follow, and all three are the point:
///
/// - the mutation path never waits on DuckDB, and a DuckDB failure can never
///   fail a journal append — §40's "projections are disposable" made
///   structural rather than promised;
/// - an answer is always as fresh as the journal, because the catch-up runs
///   *before* the query, not on a timer;
/// - rebuild and catch-up run the identical fold, so "delete the file and
///   restart" and "keep it current" cannot produce different tables.
///
/// A catch-up failure is answered as a 503 against the projection, never as a
/// failure of the work it describes: the journal is untouched and a restart
/// rebuilds.
async fn with_analytics<T>(
    state: &ApiState,
    f: impl FnOnce(&mut Analytics) -> Result<T, AnalyticsError>,
) -> Result<(T, u64), Response> {
    // Never hold both locks: the core's lock is the daemon's mutation path,
    // and a read of a derived file has no business being in the way of it.
    // Reading `last_seq` outside the catch-up is safe because `catch_up`
    // skips anything already folded — a concurrent reader that got there
    // first costs this one a re-read of nothing.
    //
    // But the gap between that read and re-acquiring the analytics lock is
    // a window: a concurrent `with_analytics` call can run its own
    // `catch_up` in between — succeeding (advancing `last_seq` past what
    // `pending` covers) or failing (resetting `last_seq` to 0 behind our
    // back). Either way, `pending` was computed against a `from` that is no
    // longer the projection's real position, and folding it as-is would
    // fold the wrong tail. So the read-fetch-fold cycle is retried, under
    // the re-acquired lock, until `last_seq` still matches what `pending`
    // was fetched against.
    let mut analytics = loop {
        let from = state.analytics.lock().await.last_seq();
        let pending = match CoreGuard::acquire(&state.core).await.events_after(from) {
            Ok(events) => events,
            Err(e) => return Err(projection_unavailable(e)),
        };
        let mut analytics = state.analytics.lock().await;
        if analytics.last_seq() != from {
            continue;
        }
        if let Err(e) = analytics.catch_up(pending.into_iter().map(Ok)) {
            return Err(projection_unavailable(e));
        }
        break analytics;
    };
    match f(&mut analytics) {
        Ok(value) => Ok((value, analytics.last_seq())),
        Err(AnalyticsError::UnknownQuery { name }) => Err(error_response(
            StatusCode::NOT_FOUND,
            "unknown_query",
            format!("no analytics query named {name:?}"),
        )),
        Err(e) => Err(projection_unavailable(e)),
    }
}

/// A projection failure: the derived read model is unavailable, and that is
/// all it is. 503 rather than 500 — the journal is fine and a rebuild fixes
/// it — with the code naming the projection so no client mistakes this for a
/// failure of the work itself.
fn projection_unavailable(e: impl std::fmt::Display) -> Response {
    error_response(
        StatusCode::SERVICE_UNAVAILABLE,
        "projection_unavailable",
        format!("the analytical projection could not answer: {e}"),
    )
}

/// `GET /v1/graph/work/{id}` — the work's §23 graph neighborhood (§8).
///
/// Every edge carries the `source_seq` of the journal event that justifies
/// it: the graph is inspectable back to the chronology, which is what makes
/// it a derivation rather than an opinion.
async fn work_graph(State(state): State<ApiState>, Path(id): Path<String>) -> Response {
    {
        let core = CoreGuard::acquire(&state.core).await;
        if !core.registry.state().works.contains_key(&id) {
            return error_response(
                StatusCode::NOT_FOUND,
                "work_not_found",
                format!("no work with id {id}"),
            );
        }
    }
    match with_analytics(&state, |analytics| analytics.graph_neighborhood(&id)).await {
        Ok((view, last_seq)) => Json(json!({
            "work_id": id,
            "nodes": view.nodes,
            "edges": view.edges,
            "projection": {"last_seq": last_seq},
        }))
        .into_response(),
        Err(response) => response,
    }
}

/// `GET /v1/analytics` — the canned §22 questions this daemon can answer.
async fn analytics_index(State(state): State<ApiState>) -> Response {
    match with_analytics(&state, |analytics| analytics.table_counts()).await {
        Ok((counts, last_seq)) => Json(json!({
            "queries": CANNED_QUERIES.iter().map(|q| json!({
                "name": q.name,
                "question": q.question,
            })).collect::<Vec<_>>(),
            "tables": counts.into_iter()
                .map(|(name, rows)| json!({"table": name, "rows": rows}))
                .collect::<Vec<_>>(),
            "projection": {"last_seq": last_seq},
        }))
        .into_response(),
        Err(response) => response,
    }
}

/// `GET /v1/analytics/{name}` — run one canned §22 query.
async fn analytics_query(State(state): State<ApiState>, Path(name): Path<String>) -> Response {
    match with_analytics(&state, |analytics| analytics.query(&name)).await {
        Ok((result, last_seq)) => {
            let mut body = result.to_json();
            body["projection"] = json!({"last_seq": last_seq});
            Json(body).into_response()
        }
        Err(response) => response,
    }
}

#[derive(Debug, Default, Deserialize)]
struct EventsQuery {
    /// Return events with seq strictly greater than this (default 0 = all).
    #[serde(default)]
    from: u64,
    /// Restrict to one work's events (the detail screens' "recent events
    /// tail"). Absent = the whole stream.
    #[serde(default)]
    work_id: Option<String>,
    /// Keep only the newest `limit` matching events. Absent = all of them.
    #[serde(default)]
    limit: Option<usize>,
}

/// The `GET /v1/events` body for a already-fetched slice.
fn events_body(events: Vec<Event>, query: &EventsQuery) -> Value {
    let mut events: Vec<Event> = match &query.work_id {
        Some(work_id) => events
            .into_iter()
            .filter(|e| e.work_id.as_deref() == Some(work_id.as_str()))
            .collect(),
        None => events,
    };
    if let Some(limit) = query.limit
        && events.len() > limit
    {
        events.drain(..events.len() - limit);
    }
    json!({"events": events})
}

/// `GET /v1/events?from=N&work_id=X&limit=K` — journaled history after seq N.
///
/// `from` is the only bound on how much journal is read; `work_id` and `limit`
/// shape the answer, not the scan. A client that wants a cheap tail should
/// carry the `from` it already knows (the SSE stream's last id) rather than
/// re-asking from 0.
async fn event_history(
    State(state): State<ApiState>,
    query: Result<Query<EventsQuery>, QueryRejection>,
) -> Response {
    let query = match parse_query(query) {
        Ok(q) => q,
        Err(resp) => return *resp,
    };
    let core = CoreGuard::acquire(&state.core).await;
    match core.events_after(query.from) {
        Ok(events) => Json(events_body(events, &query)).into_response(),
        Err(e) => internal_error(e),
    }
}

/// `GET /v1/events/stream` — SSE live tail with resume-by-seq.
///
/// Resume point: `Last-Event-ID` header (SSE ids are seqs) wins over the
/// `from` query param; both mean "I have everything up to and including N".
/// The subscriber attaches to the live broadcast *before* reading history,
/// then dedups by seq, so the switchover can neither drop nor repeat events.
async fn event_stream(
    State(state): State<ApiState>,
    headers: axum::http::HeaderMap,
    query: Result<Query<EventsQuery>, QueryRejection>,
) -> Response {
    let query = match parse_query(query) {
        Ok(q) => q,
        Err(resp) => return *resp,
    };
    let from = match headers.get("last-event-id") {
        Some(v) => match v.to_str().ok().and_then(|s| s.parse::<u64>().ok()) {
            Some(seq) => seq,
            None => {
                return error_response(
                    StatusCode::BAD_REQUEST,
                    "invalid_request",
                    "Last-Event-ID must be a decimal event seq",
                );
            }
        },
        None => query.from,
    };
    let (tx, rx) = mpsc::channel::<Result<SseEvent, std::convert::Infallible>>(64);
    tokio::spawn(pump_until_closing(state, from, tx));
    Sse::new(ReceiverStream::new(rx))
        .keep_alive(KeepAlive::new().interval(SSE_KEEP_ALIVE))
        .into_response()
}

/// Run one subscriber's pump, but never past the daemon's shutdown.
///
/// Graceful shutdown waits for in-flight responses, and an SSE body is
/// in-flight until its stream ends — which a live tail never does on its own.
/// Without this, one attached tail pins the daemon alive forever: no
/// `daemon.stopped`, no descriptor removal, every later client stuck in the
/// ambiguous "PID alive but endpoint unresponsive" branch. Cancelling the
/// pump drops its channel, which ends the stream and completes the response
/// wherever the pump happened to be.
async fn pump_until_closing(
    state: ApiState,
    from: u64,
    tx: mpsc::Sender<Result<SseEvent, std::convert::Infallible>>,
) {
    let mut closing = state.closing.clone();
    tokio::select! {
        () = forward_events(state, from, tx) => {}
        () = daemon_closing(&mut closing) => {}
    }
}

/// Resolve once the daemon is shutting down — immediately if it already is,
/// so a stream opened during shutdown cannot miss the signal and hang.
async fn daemon_closing(closing: &mut watch::Receiver<bool>) {
    let already = *closing.borrow();
    if already {
        return;
    }
    // A dropped sender means the daemon is gone too: either way, stop.
    let _ = closing.changed().await;
}

/// Pump for one SSE subscriber: history after `from`, then the live tail,
/// deduplicated by seq. Exits when the client goes away or the daemon stops.
async fn forward_events(
    state: ApiState,
    from: u64,
    tx: mpsc::Sender<Result<SseEvent, std::convert::Infallible>>,
) {
    // Subscribe before reading history so nothing can fall in the gap.
    let mut live = {
        let core = CoreGuard::acquire(&state.core).await;
        core.events_tx.subscribe()
    };
    let mut last_sent = from;
    let history = {
        let core = CoreGuard::acquire(&state.core).await;
        core.events_after(last_sent)
    };
    match history {
        Ok(events) => {
            for event in events {
                if send_sse(&tx, &event).await.is_err() {
                    return;
                }
                last_sent = event.seq;
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "sse history replay failed; closing stream");
            return;
        }
    }
    loop {
        match live.recv().await {
            Ok(event) => {
                if event.seq <= last_sent {
                    continue; // already delivered from history
                }
                if send_sse(&tx, &event).await.is_err() {
                    return;
                }
                last_sent = event.seq;
            }
            Err(broadcast::error::RecvError::Lagged(_)) => {
                // Fell behind the broadcast buffer: refill from the journal,
                // which always has everything.
                let refill = {
                    let core = CoreGuard::acquire(&state.core).await;
                    core.events_after(last_sent)
                };
                match refill {
                    Ok(events) => {
                        for event in events {
                            if send_sse(&tx, &event).await.is_err() {
                                return;
                            }
                            last_sent = event.seq;
                        }
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "sse lag refill failed; closing stream");
                        return;
                    }
                }
            }
            Err(broadcast::error::RecvError::Closed) => return,
        }
    }
}

/// Every event kind [`send_sse`] can name a frame with — the SSE stream's
/// published vocabulary.
///
/// `EventSource` has no way to subscribe to "every named frame": a client that
/// wants all of them must name each one. Rather than let each client keep its
/// own copy of the list (the dashboard did, and it was already five kinds out
/// of date), the vocabulary is stated once, here, next to the function that
/// writes the frame names — and assembled from the journal's own `KIND_*`
/// constants so it cannot say a kind the journal does not have. `t6` in the M6
/// suite is the other half: it fails if a `KIND_*` constant is added to the
/// crate and not to this list.
pub const SSE_EVENT_KINDS: &[&str] = &[
    KIND_WORK_SUBMITTED,
    KIND_WORK_STARTED,
    KIND_WORK_RESUMED,
    KIND_WORK_WAITING,
    KIND_WORK_NEEDS_INPUT,
    KIND_WORK_BLOCKED,
    KIND_WORK_COMPLETED,
    KIND_WORK_FAILED,
    KIND_WORK_CANCELED,
    KIND_WORKFLOW_BOUND,
    KIND_STAGE_ENTERED,
    KIND_STAGE_COMPLETED,
    KIND_STAGE_WAITING,
    KIND_STAGE_NEEDS_INPUT,
    KIND_STAGE_INPUT_RECEIVED,
    KIND_STAGE_RESUMED,
    KIND_STAGE_BLOCKED,
    KIND_STAGE_FAILED,
    KIND_STAGE_CANCELED,
    KIND_EXECUTION_RESERVED,
    KIND_EXECUTION_STARTED,
    KIND_EXECUTION_STOPPED,
    KIND_EXECUTION_ABANDONED,
    KIND_EXECUTION_RECONCILED,
    KIND_TURN_CEILING_INTERRUPTED,
    KIND_TURN_ENVELOPE_EXTENDED,
    KIND_SURFACE_MATERIALIZING,
    KIND_SURFACE_MATERIALIZED,
    KIND_SURFACE_TORN_DOWN,
    KIND_CONVERSATION_USER,
    KIND_CONVERSATION_ASSISTANT_COMPLETED,
    KIND_CONVERSATION_ASK,
    KIND_CONVERSATION_TURN_ENDED,
    KIND_TOOL_REQUESTED,
    KIND_TOOL_COMPLETED,
    KIND_USAGE_UPDATED,
    KIND_COMMAND_ACCEPTED,
    KIND_COMMAND_REJECTED,
    KIND_DAEMON_STARTED,
    KIND_DAEMON_STOPPED,
    KIND_BACKEND_PROBED,
    KIND_ADMISSION_PAUSED,
    KIND_ADMISSION_RESUMED,
];

/// Encode one journal event as an SSE frame (`id` = seq for resume).
async fn send_sse(
    tx: &mpsc::Sender<Result<SseEvent, std::convert::Infallible>>,
    event: &Event,
) -> Result<(), ()> {
    let data = serde_json::to_string(event).map_err(|_| ())?;
    let frame = SseEvent::default()
        .id(event.seq.to_string())
        .event(event.kind.clone())
        .data(data);
    tx.send(Ok(frame)).await.map_err(|_| ())
}

// ---------------------------------------------------------------------------
// Shared view helpers
// ---------------------------------------------------------------------------
//
// The dashboard (`src/web.rs`, deleted — ADR 0011) used to share these with
// the TUI so the two screens could not tell different stories about the same
// field; the TUI is the sole client of them now, but the rule they encode —
// "how a JSON field reads as text" belongs to the API surface, not to a
// screen — still holds for whatever client reads next.

/// A JSON value as human text: `-` when it is absent or an empty string, the
/// string itself when it is one, its JSON form otherwise.
pub fn field_text(value: &Value) -> String {
    match value {
        Value::Null => "-".to_string(),
        Value::String(s) if s.is_empty() => "-".to_string(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// A stage coordinate as `10-implement 2/4 · running`, or `-` when there is
/// no run. `index` is zero-based on the wire and one-based on a screen.
pub fn stage_label(stage: &Value) -> String {
    if stage.is_null() {
        return "-".to_string();
    }
    let position = match (stage["index"].as_u64(), stage["of"].as_u64()) {
        (Some(i), Some(of)) => format!(" {}/{of}", i + 1),
        (Some(i), None) => format!(" {}", i + 1),
        _ => String::new(),
    };
    format!(
        "{}{position} · {}",
        field_text(&stage["stage_id"]),
        field_text(&stage["status"])
    )
}

/// Default per-request timeout for the API client. Not applied to the SSE
/// stream, whose whole job is to stay open.
pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Environment override for [`CLIENT_TIMEOUT`], in whole seconds.
///
/// `POST /v1/work` and `POST /v1/work/{id}/input` drive the stage *inside* the
/// request — the engine is synchronous in P0 — so a call that lands on a real
/// model backend can outlast any fixed default (M4's ledger records a
/// six-turn Claude pair at 33 s). Raising the default instead would be worse:
/// a long default turns an unreachable daemon into a long hang for every
/// client. So the default stays short and the callers that knowingly wait on a
/// model — `scripts/demo.sh --real-claude` is the one in this repo — say so.
pub const CLIENT_TIMEOUT_ENV: &str = "SGT_CLIENT_TIMEOUT_SECS";

/// The per-request timeout this process's API clients use: [`CLIENT_TIMEOUT`]
/// unless [`CLIENT_TIMEOUT_ENV`] names a larger, positive number of seconds.
///
/// An override that is *present but not applied* is said out loud, once, on
/// stderr. Raise-only is the right rule (see [`timeout_from`]), but silently
/// ignoring the value someone exported is how an operator concludes the knob
/// is broken: `SGT_CLIENT_TIMEOUT_SECS=5` and `SGT_CLIENT_TIMEOUT_SECS=abc`
/// both behaved exactly like not setting it at all, with nothing anywhere
/// naming the timeout actually in force. One line, once per process (the
/// warning is about this process's configuration, not about each client
/// constructed from it), on stderr rather than through `tracing` — the
/// clients that read this knob are CLI processes with no subscriber
/// installed, so a `warn!` would go nowhere.
pub fn client_timeout() -> Duration {
    static SAID: std::sync::Once = std::sync::Once::new();
    let (timeout, warning) = timeout_from(std::env::var(CLIENT_TIMEOUT_ENV).ok().as_deref());
    if let Some(warning) = warning {
        SAID.call_once(|| eprintln!("warning: {warning}"));
    }
    timeout
}

/// [`client_timeout`]'s rule, as a function of the raw setting: the timeout to
/// use, and the warning to print when the setting was present and did not
/// produce it.
///
/// Separated from the `std::env` read — and from the printing — so both halves
/// can be tested: the whole knob is a few lines of parsing whose only
/// production caller is an opt-in `--real-claude` path, i.e. exactly the shape
/// that rots unobserved. It had already drifted from the sentence above it
/// twice: the filter once admitted *any* positive value, so
/// `SGT_CLIENT_TIMEOUT_SECS=1` silently shortened every client's timeout below
/// the default; and the fix for that swallowed the setting without a word. The
/// knob exists to let a caller that knowingly waits on a model wait longer; it
/// is not a way to make the daemon look unreachable, so the raise is
/// one-directional — and now audible when it declines.
fn timeout_from(raw: Option<&str>) -> (Duration, Option<String>) {
    let Some(raw) = raw else {
        return (CLIENT_TIMEOUT, None);
    };
    let default_secs = CLIENT_TIMEOUT.as_secs();
    // Every warning names the knob, the value that was ignored, and the
    // timeout actually in force — the three facts an operator needs to stop
    // guessing.
    let declined = |why: &str| {
        (
            CLIENT_TIMEOUT,
            Some(format!(
                "{CLIENT_TIMEOUT_ENV}={raw:?} {why}; using the {default_secs}s default"
            )),
        )
    };
    match raw.trim().parse::<u64>() {
        Ok(seconds) if seconds > default_secs => (Duration::from_secs(seconds), None),
        Ok(_) => declined("does not raise the timeout, and the knob only raises it"),
        Err(_) => declined("is not a whole number of seconds"),
    }
}

/// A failure talking to the daemon's v1 API.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// The request never produced a response (connect, timeout, body).
    #[error("{0}")]
    Transport(String),
    /// The daemon answered with a non-2xx status and a structured error.
    #[error("{status}: {message}")]
    Api {
        /// HTTP status the daemon answered with.
        status: u16,
        /// Machine-readable error code from the structured body.
        code: String,
        /// Human-readable message from the structured body.
        message: String,
    },
}

impl From<reqwest::Error> for ClientError {
    fn from(e: reqwest::Error) -> Self {
        Self::Transport(e.to_string())
    }
}

/// A client of the v1 API.
///
/// This is the *whole* surface a non-daemon client has: the CLI, the TUI and
/// anything else in-process reach state through this type and nothing else.
/// It lives beside the router on purpose — a request shape and its response
/// shape are one contract, and keeping the two halves in one file is what
/// makes "the client is a projection of the API" checkable by reading.
#[derive(Clone, Debug)]
pub struct ApiClient {
    http: reqwest::Client,
    endpoint: String,
    token: String,
}

impl ApiClient {
    /// Build a client for a daemon endpoint and its bearer token.
    pub fn new(endpoint: &str, token: &str) -> Result<Self, ClientError> {
        Ok(Self {
            http: reqwest::Client::builder()
                .timeout(client_timeout())
                .build()?,
            endpoint: endpoint.trim_end_matches('/').to_string(),
            token: token.to_string(),
        })
    }

    /// The daemon endpoint this client talks to.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Authenticated GET returning the parsed body.
    pub async fn get(&self, path: &str) -> Result<Value, ClientError> {
        let response = self
            .http
            .get(format!("{}{path}", self.endpoint))
            .bearer_auth(&self.token)
            .send()
            .await?;
        Self::into_value(response).await
    }

    /// Authenticated POST with a JSON body, returning the parsed body.
    pub async fn post(&self, path: &str, body: &Value) -> Result<Value, ClientError> {
        let response = self
            .http
            .post(format!("{}{path}", self.endpoint))
            .bearer_auth(&self.token)
            .json(body)
            .send()
            .await?;
        Self::into_value(response).await
    }

    /// Authenticated DELETE with a JSON body, returning the parsed body.
    pub async fn delete(&self, path: &str, body: &Value) -> Result<Value, ClientError> {
        let response = self
            .http
            .delete(format!("{}{path}", self.endpoint))
            .bearer_auth(&self.token)
            .json(body)
            .send()
            .await?;
        Self::into_value(response).await
    }

    /// `GET /v1/system`.
    pub async fn system(&self) -> Result<Value, ClientError> {
        self.get("/v1/system").await
    }

    /// `GET /v1/work`.
    pub async fn fleet(&self) -> Result<Value, ClientError> {
        self.get("/v1/work").await
    }

    /// `GET /v1/work/{id}`.
    pub async fn work(&self, id: &str) -> Result<Value, ClientError> {
        self.get(&format!("/v1/work/{id}")).await
    }

    /// `GET /v1/work/{id}/transcript` — the work's conversation, decoded in
    /// causal order (MVP-3's `sgt work transcript`).
    pub async fn work_transcript(&self, id: &str) -> Result<Value, ClientError> {
        self.get(&format!("/v1/work/{}/transcript", urlencode(id)))
            .await
    }

    /// `GET /v1/events` for one work's newest `limit` events.
    pub async fn work_events(&self, id: &str, limit: usize) -> Result<Value, ClientError> {
        self.get(&format!(
            "/v1/events?work_id={}&limit={limit}",
            urlencode(id)
        ))
        .await
    }

    /// `POST /v1/work/{id}/cancel` with a fresh command id (§26).
    pub async fn cancel(&self, id: &str) -> Result<Value, ClientError> {
        self.post(
            &format!("/v1/work/{id}/cancel"),
            &json!({"command_id": ulid::Ulid::generate().to_string()}),
        )
        .await
    }

    /// `POST /v1/work/{id}/input` with a fresh command id (§26).
    pub async fn respond(&self, id: &str, input: &str) -> Result<Value, ClientError> {
        self.post(
            &format!("/v1/work/{id}/input"),
            &json!({
                "command_id": ulid::Ulid::generate().to_string(),
                "input": input,
            }),
        )
        .await
    }

    /// `POST /v1/work/{id}/retry` with a fresh command id (§26).
    pub async fn retry(&self, id: &str) -> Result<Value, ClientError> {
        self.post(
            &format!("/v1/work/{id}/retry"),
            &json!({"command_id": ulid::Ulid::generate().to_string()}),
        )
        .await
    }

    /// `POST /v1/work/{id}/extend` with a fresh command id (§26).
    pub async fn extend(&self, id: &str, additional_turns: u32) -> Result<Value, ClientError> {
        self.post(
            &format!("/v1/work/{id}/extend"),
            &json!({
                "command_id": ulid::Ulid::generate().to_string(),
                "additional_turns": additional_turns,
            }),
        )
        .await
    }

    /// `GET /v1/retained` — #109's inspect verb: every repository binding
    /// any terminal Work's teardown left something on disk for, estate-wide.
    pub async fn retained(&self) -> Result<Value, ClientError> {
        self.get("/v1/retained").await
    }

    /// `GET /v1/workflows?cwd=<percent-encoded path>` — the read-only
    /// workflow catalog (§11.2): what new Work submitted from `cwd` could
    /// bind now.
    pub async fn workflows(&self, cwd: &std::path::Path) -> Result<Value, ClientError> {
        self.get(&format!(
            "/v1/workflows?cwd={}",
            urlencode(&cwd.display().to_string())
        ))
        .await
    }

    /// `POST /v1/work/{id}/reap` with a fresh command id (§26) — #109's
    /// dispose verb. `confirm` must be `true` or the daemon refuses (fail
    /// closed: reap has no undo).
    pub async fn reap(&self, id: &str, confirm: bool) -> Result<Value, ClientError> {
        self.post(
            &format!("/v1/work/{id}/reap"),
            &json!({
                "command_id": ulid::Ulid::generate().to_string(),
                "confirm": confirm,
            }),
        )
        .await
    }

    /// `GET /v1/estate/repos` (§16.2/§20.4) — declared repositories.
    pub async fn repos(&self) -> Result<Value, ClientError> {
        self.get("/v1/estate/repos").await
    }

    /// `POST /v1/estate/repos` (§16.2/§20.4) — `manifest::add_repo`.
    pub async fn add_repo(
        &self,
        name: &str,
        origin: Option<&str>,
        instructions: Option<&str>,
    ) -> Result<Value, ClientError> {
        self.post(
            "/v1/estate/repos",
            &json!({"name": name, "origin": origin, "instructions": instructions}),
        )
        .await
    }

    /// `DELETE /v1/estate/repos/{name}` (§16.2/§20.4) — `manifest::remove_repo`.
    pub async fn remove_repo(&self, name: &str) -> Result<Value, ClientError> {
        self.delete(&format!("/v1/estate/repos/{}", urlencode(name)), &json!({}))
            .await
    }

    /// `GET /v1/estate/groups` (§16.2/§20.4) — declared groups.
    pub async fn groups(&self) -> Result<Value, ClientError> {
        self.get("/v1/estate/groups").await
    }

    /// `POST /v1/estate/groups` (§16.2/§20.4) — `manifest::add_group`'s
    /// mkdir-p create/extend semantics.
    pub async fn add_group(
        &self,
        name: &str,
        repos: &[String],
        brief: Option<&str>,
    ) -> Result<Value, ClientError> {
        self.post(
            "/v1/estate/groups",
            &json!({"name": name, "repos": repos, "brief": brief}),
        )
        .await
    }

    /// `DELETE /v1/estate/groups/{name}` (§16.2/§20.4) — `manifest::remove_group`;
    /// empty `repos` removes the whole group, otherwise just those members.
    pub async fn remove_group(&self, name: &str, repos: &[String]) -> Result<Value, ClientError> {
        self.delete(
            &format!("/v1/estate/groups/{}", urlencode(name)),
            &json!({"repos": repos}),
        )
        .await
    }

    /// `GET /v1/doctor` (§16.3/§20.4) — the same `doctor::Report` `sgt doctor
    /// --json` prints.
    pub async fn doctor(&self) -> Result<Value, ClientError> {
        self.get("/v1/doctor").await
    }

    /// Open the SSE live tail at `GET /v1/events/stream?from=N`.
    ///
    /// A separate reqwest client is built with no total timeout: the response
    /// body of a live tail is *supposed* to stay open, and the per-request
    /// timeout that keeps a stuck command honest would kill it on schedule.
    pub async fn stream_events(&self, from: u64) -> Result<EventStream, ClientError> {
        let http = reqwest::Client::builder().build()?;
        let response = http
            .get(format!("{}/v1/events/stream?from={from}", self.endpoint))
            .bearer_auth(&self.token)
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            let body: Value = response.json().await.unwrap_or(Value::Null);
            return Err(Self::api_error(status.as_u16(), &body));
        }
        Ok(EventStream {
            response,
            pending: String::new(),
        })
    }

    async fn into_value(response: reqwest::Response) -> Result<Value, ClientError> {
        let status = response.status();
        let body: Value = response.json().await.unwrap_or(Value::Null);
        if status.is_success() {
            Ok(body)
        } else {
            Err(Self::api_error(status.as_u16(), &body))
        }
    }

    fn api_error(status: u16, body: &Value) -> ClientError {
        ClientError::Api {
            status,
            code: body["error"]["code"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            message: body["error"]["message"]
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| format!("HTTP {status}")),
        }
    }
}

/// Minimal percent-encoding for the few values this crate puts in a URL
/// (work ids, today). Anything outside the unreserved set is escaped rather
/// than trusted.
fn urlencode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

/// A live SSE tail of the journal, as [`Event`]s.
///
/// Frames are accumulated across chunk boundaries (a chunk is a transport
/// artifact, not a message) and decoded one blank-line-separated frame at a
/// time. Comment lines — which is what axum's keep-alive sends — carry no
/// `data:` and are skipped without ending the stream.
#[derive(Debug)]
pub struct EventStream {
    response: reqwest::Response,
    pending: String,
}

/// A `data:` frame that is present but did not decode as a Sergeant [`Event`]
/// (R-WATCH-7 / proposal §11.2).
///
/// Before this revision, [`EventStream::next_event`] collapsed this case and
/// a harmless keep-alive comment to the same `None` — a subscriber could not
/// tell "nothing to report yet" from "the daemon sent something this client
/// could not read", which §11.2 calls out by name: "a frame containing
/// `data:` that cannot decode as a Sergeant event is not a keep-alive and
/// must not be silently skipped." This type is that distinction, carrying the
/// raw payload for the diagnostic.
#[derive(Debug, Clone, thiserror::Error)]
#[error("malformed event frame: {0}")]
pub struct MalformedFrame(pub String);

impl EventStream {
    /// The next journal event.
    ///
    /// Three outcomes, matching §11.2's four-way split minus the one pair
    /// that collapses on purpose: `Ok(Some(event))` is a decoded [`Event`];
    /// `Ok(None)` is the stream ending — cleanly (the daemon closed it) or
    /// through a transport failure (`self.response.chunk()` erroring) — the
    /// proposal names both "transport end/error" as one outcome, not two, and
    /// a caller here has no durable cursor to act differently on the
    /// distinction; keep-alive/comment frames are consumed silently inside
    /// the loop, exactly as before, and never reach the caller as either
    /// variant. `Err(MalformedFrame)` is R-WATCH-7's new, previously-missing
    /// outcome: a `data:` frame that is not a keep-alive and does not parse
    /// ends the read rather than being silently dropped.
    pub async fn next_event(&mut self) -> Result<Option<Event>, MalformedFrame> {
        loop {
            while let Some(frame) = take_frame(&mut self.pending) {
                match decode_frame(&frame) {
                    FrameDecode::KeepAlive => continue,
                    FrameDecode::Event(event) => return Ok(Some(*event)),
                    FrameDecode::Malformed(raw) => return Err(MalformedFrame(raw)),
                }
            }
            match self.response.chunk().await {
                Ok(Some(chunk)) => self.pending.push_str(&String::from_utf8_lossy(&chunk)),
                Ok(None) | Err(_) => return Ok(None),
            }
        }
    }
}

/// Split off the first complete SSE frame (terminated by a blank line).
fn take_frame(pending: &mut String) -> Option<String> {
    let end = pending.find("\n\n")?;
    let frame = pending[..end].to_string();
    pending.drain(..end + 2);
    Some(frame)
}

/// The result of decoding one SSE frame's `data:` lines (R-WATCH-7): a
/// comment/keep-alive frame, a frame that decodes as an [`Event`], and a
/// `data:` frame that does not decode are three different outcomes, not two
/// — see [`MalformedFrame`] for why the third one used to disappear into the
/// first.
#[derive(Debug)]
enum FrameDecode {
    /// No `data:` line at all: axum's keep-alive comment (`: keep-alive`), or
    /// a frame with nothing in it.
    KeepAlive,
    /// A `data:` payload that did not parse as a Sergeant [`Event`]. Carries
    /// the joined raw payload for the diagnostic.
    Malformed(String),
    /// A valid, decoded event. Boxed: `Event` is by far this enum's largest
    /// variant, and every `Malformed`/`KeepAlive` decode would otherwise pay
    /// its stack size for nothing.
    Event(Box<Event>),
}

/// Decode one SSE frame's `data:` lines into a [`FrameDecode`].
fn decode_frame(frame: &str) -> FrameDecode {
    let mut data = String::new();
    for line in frame.lines() {
        if let Some(rest) = line.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(rest.strip_prefix(' ').unwrap_or(rest));
        }
    }
    if data.is_empty() {
        return FrameDecode::KeepAlive;
    }
    match serde_json::from_str(&data) {
        Ok(event) => FrameDecode::Event(Box::new(event)),
        Err(_) => FrameDecode::Malformed(data),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::BackendRegistry;
    use crate::domain::event::EVENT_SCHEMA;
    use crate::runtime::projection::work_registry_projection;

    /// The `-`-for-missing rule, where it is defined. It used to be tested
    /// beside the dashboard's renderers in `src/web.rs` (deleted, ADR 0011);
    /// `tui.rs` is the sole reader of [`field_text`] now, but the rule
    /// itself is an API-surface concern, so its test stays here.
    #[test]
    fn an_absent_field_reads_as_a_dash_and_a_present_one_as_itself() {
        assert_eq!(field_text(&Value::Null), "-");
        assert_eq!(
            field_text(&json!("")),
            "-",
            "an empty string is absence too"
        );
        assert_eq!(field_text(&json!("fake")), "fake");
        assert_eq!(
            field_text(&json!(3)),
            "3",
            "a number reads as its JSON form"
        );
        assert_eq!(field_text(&json!(false)), "false");
    }

    /// Ditto for [`stage_label`], moved from `src/web.rs` alongside the test
    /// above rather than dropped with the file that used to host it.
    #[test]
    fn a_stage_reads_as_position_and_status() {
        let stage = json!({"stage_id": "10-implement", "index": 1, "of": 4, "status": "running"});
        assert_eq!(stage_label(&stage), "10-implement 2/4 · running");
        assert_eq!(stage_label(&Value::Null), "-");
    }

    /// The one knob on the client, exercised. Its only production caller is
    /// `scripts/demo.sh --real-claude`, which the suite deliberately does not
    /// run, so without this the parse is unobserved code — and it had already
    /// drifted from its own doc comment (any positive value was accepted,
    /// including ones *below* the default).
    #[test]
    fn the_client_timeout_override_only_ever_raises_the_default() {
        assert_eq!(timeout_from(None).0, CLIENT_TIMEOUT, "unset: the default");
        assert_eq!(
            timeout_from(Some("300")).0,
            Duration::from_secs(300),
            "a larger value is what the knob is for"
        );
        assert_eq!(
            timeout_from(Some("  300\n")).0,
            Duration::from_secs(300),
            "an exported value carrying whitespace still parses"
        );
        for lowering in ["1", "0", "10"] {
            assert_eq!(
                timeout_from(Some(lowering)).0,
                CLIENT_TIMEOUT,
                "{lowering}s must not shorten the default: a short timeout makes a \
                 working daemon look unreachable to every client in the process"
            );
        }
        for nonsense in ["", "abc", "-5", "2.5", "9999999999999999999999"] {
            assert_eq!(
                timeout_from(Some(nonsense)).0,
                CLIENT_TIMEOUT,
                "an unparseable {nonsense:?} falls back rather than failing the client"
            );
        }
    }

    /// An override that is present and not applied is *said*, and the setting
    /// that is applied is not.
    ///
    /// The regression this pins: raise-only semantics silently swallowed
    /// every below-default, zero and unparseable value, so an operator who
    /// exported `SGT_CLIENT_TIMEOUT_SECS=5` (or typo'd it) got the 10s default
    /// with nothing anywhere saying so — the knob looked broken, and the next
    /// person's diagnosis was a code read. The decision is returned rather
    /// than printed inside `timeout_from` precisely so this test can make it.
    #[test]
    fn an_override_that_is_not_applied_says_so() {
        assert_eq!(timeout_from(None).1, None, "unset is not a complaint");
        assert_eq!(
            timeout_from(Some("300")).1,
            None,
            "an applied raise is silent: nothing was ignored"
        );

        for ignored in ["1", "0", "10", "", "abc", "-5", "2.5"] {
            let (timeout, warning) = timeout_from(Some(ignored));
            assert_eq!(
                timeout, CLIENT_TIMEOUT,
                "raise-only semantics are unchanged"
            );
            let warning =
                warning.unwrap_or_else(|| panic!("{ignored:?} was ignored without a word"));
            assert!(
                warning.contains(CLIENT_TIMEOUT_ENV),
                "the warning names the knob: {warning}"
            );
            assert!(
                warning.contains(&format!("{ignored:?}")),
                "the warning names the value it ignored: {warning}"
            );
            assert!(
                warning.contains(&format!("{}s", CLIENT_TIMEOUT.as_secs())),
                "the warning names the effective value: {warning}"
            );
            assert_eq!(warning.lines().count(), 1, "one line: {warning}");
        }
    }

    /// `data:` lines belonging to one SSE frame are joined by a literal `\n`
    /// (the SSE spec's join rule; the code is lines 1766-1767) — not by bare
    /// concatenation, which the second half proves: split across a number's
    /// minus sign and its digit, the two chunks only reassemble into the
    /// value `-5` when nothing is inserted between them, so a decoder that
    /// really joins with a newline must fail to parse that split rather than
    /// silently producing a value nobody sent.
    ///
    /// **R-WATCH-7 revision.** This test used to assert `decode_frame(..) ==
    /// None` for the un-joined split — the exact silent-skip §11.2 forbids: a
    /// `data:` frame that is present but does not parse is not a keep-alive,
    /// and collapsing it to the same `None` a comment frame produces is what
    /// this contract ruling revises `decode_frame`/`EventStream` to stop
    /// doing. The second half now asserts `FrameDecode::Malformed`, not
    /// `None` — the pin's *shape* (a joined split must fail to parse) is
    /// unchanged; only what "fails to parse" is allowed to look like moved.
    #[test]
    fn decode_frame_coalesces_data_lines_with_a_real_newline() {
        // An ordinary split, at an object-member boundary, decodes into
        // exactly the event that boundary was cut from.
        let frame = "data: {\"schema\":\"sergeant.event/v1\",\"seq\":7,\"id\":\"01H7X8Y9Z\",\"timestamp\":\"2026-01-01T00:00:00.000Z\",\"source\":{\"type\":\"test\",\"name\":\"harness\"},\ndata: \"kind\":\"test.seeded\",\"payload\":{\"n\":3}}";
        let event = match decode_frame(frame) {
            FrameDecode::Event(event) => event,
            other => {
                panic!("a frame whose data: is split across two lines still decodes: {other:?}")
            }
        };
        assert_eq!(event.seq, 7);
        assert_eq!(event.id, "01H7X8Y9Z");
        assert_eq!(event.kind, "test.seeded");
        assert_eq!(event.payload, json!({"n": 3}));

        // Split at a number's minus sign instead: without the joining
        // newline, `-` and `5` on separate data: lines read back together as
        // the valid number -5. The newline this test pins turns that into a
        // `-` followed by whitespace then `5`, which is not a legal JSON
        // number.
        let split_number = "data: {\"schema\":\"sergeant.event/v1\",\"seq\":9,\"id\":\"01NEG\",\"timestamp\":\"2026-01-01T00:00:02.000Z\",\"source\":{\"type\":\"test\",\"name\":\"harness\"},\"kind\":\"test.seeded\",\"payload\":{\"n\":-\ndata: 5}}";
        assert!(
            matches!(decode_frame(split_number), FrameDecode::Malformed(_)),
            "the two data: lines must not be concatenated without the newline the SSE spec \
             requires between them — and (R-WATCH-7) the failure to parse must surface as \
             Malformed, not silently collapse to the same outcome as a keep-alive comment"
        );
    }

    /// Comment lines — axum's keep-alive shape (`KeepAlive`, used above) —
    /// carry no `data:` and must be skipped without ending the frame or
    /// corrupting the real payload beside them.
    #[test]
    fn decode_frame_skips_comment_and_keep_alive_lines() {
        assert!(
            matches!(decode_frame(": keep-alive"), FrameDecode::KeepAlive),
            "a frame that is only a comment carries no event"
        );
        let single_line = "{\"schema\":\"sergeant.event/v1\",\"seq\":2,\"id\":\"01Z\",\"timestamp\":\"2026-01-01T00:00:01.000Z\",\"source\":{\"type\":\"test\",\"name\":\"harness\"},\"kind\":\"test.seeded\",\"payload\":{\"n\":2}}";
        let frame = format!(": keep-alive\ndata: {single_line}");
        let event = match decode_frame(&frame) {
            FrameDecode::Event(event) => event,
            other => {
                panic!("a comment line beside a data: line must not swallow the event: {other:?}")
            }
        };
        assert_eq!(event.seq, 2);
        assert_eq!(event.id, "01Z");
    }

    /// R-WATCH-7's own new case: a `data:` frame present but undecodable is
    /// neither a keep-alive nor silently dropped — it is `Malformed`, distinct
    /// from both `KeepAlive` (no `data:` at all) and a clean stream end (which
    /// only [`EventStream::next_event`], not `decode_frame`, can produce).
    #[test]
    fn decode_frame_reports_an_undecodable_data_frame_as_malformed_not_keep_alive() {
        let malformed = decode_frame("data: not valid json at all");
        assert!(
            matches!(malformed, FrameDecode::Malformed(ref raw) if raw == "not valid json at all"),
            "a present-but-undecodable data: frame must be Malformed, carrying the raw \
             payload, not collapse into the same outcome as a comment frame: {malformed:?}"
        );
    }

    /// The end-to-end half of R-WATCH-7: `decode_frame` classifying a frame
    /// as `Malformed` is not, by itself, proof that `EventStream::next_event`
    /// actually surfaces it as an error rather than silently `continue`-ing
    /// past it (measured: a probe that changed only that one match arm back
    /// to a skip left every `decode_frame`-level test above green, because
    /// none of them call `next_event`). A bare loopback TCP listener stands
    /// in for the daemon — enough to prove the real async read path, not
    /// just the pure decoder.
    #[tokio::test]
    async fn next_event_surfaces_a_malformed_frame_as_an_error_not_a_silent_skip() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind stand-in server");
        let addr = listener.local_addr().expect("local addr");
        let body = "data: not valid json\n\n";
        std::thread::spawn(move || {
            use std::io::{Read, Write};
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf); // drain the client's request
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });

        let client = ApiClient::new(&format!("http://{addr}"), "unused-token").expect("client");
        let mut stream = client
            .stream_events(0)
            .await
            .expect("connect to the stand-in server");
        let outcome = stream.next_event().await;
        match outcome {
            Err(MalformedFrame(raw)) => assert_eq!(raw, "not valid json"),
            other => {
                panic!("a malformed data: frame must surface as Err(MalformedFrame), not {other:?}")
            }
        }
    }

    /// The two `stage_label` arms besides "both known" and "nothing at
    /// all": an index with no known total still reads its one-based
    /// position, and the absence of an index drops the position segment
    /// entirely — whether or not a total happens to be present.
    #[test]
    fn stage_label_reads_an_index_without_a_total_and_no_index_at_all() {
        let index_only = json!({"stage_id": "10-implement", "index": 3, "status": "blocked"});
        assert_eq!(
            stage_label(&index_only),
            "10-implement 4 · blocked",
            "an index without a known total still reads its one-based position"
        );

        let of_only = json!({"stage_id": "20-test", "of": 5, "status": "queued"});
        assert_eq!(
            stage_label(&of_only),
            "20-test · queued",
            "no index means no position segment, even when `of` is present"
        );

        let neither = json!({"stage_id": "20-test", "status": "queued"});
        assert_eq!(
            stage_label(&neither),
            "20-test · queued",
            "no index and no total: still no position segment"
        );
    }

    async fn test_state(data_dir: &std::path::Path) -> ApiState {
        let journal = Journal::open(data_dir).expect("open journal");
        let mut registry = work_registry_projection();
        registry
            .catch_up(journal.replay().expect("replay"))
            .expect("catch up registry");
        let (events_tx, _) = broadcast::channel(16);
        let core = Core::new(journal, registry, events_tx);
        let analytics = Analytics::rebuild(data_dir, core.journal.replay().expect("replay"))
            .expect("rebuild analytics");
        let (_closing_tx, closing_rx) = watch::channel(false);
        ApiState {
            core: Arc::new(tokio::sync::Mutex::new(core)),
            token: "test-token".to_string(),
            data_dir: data_dir.to_path_buf(),
            closing: closing_rx,
            engine: Arc::new(Engine::new(
                Arc::new(BackendRegistry::new()),
                None,
                data_dir,
            )),
            analytics: Arc::new(tokio::sync::Mutex::new(analytics)),
        }
    }

    fn seeded(n: u32) -> EventDraft {
        EventDraft::new(
            EventSource::new("test", "harness"),
            "test.seeded",
            json!({"n": n}),
        )
    }

    /// The group commit, end to end (#44): one lock hold, N appends, **one**
    /// fsync, and nothing published until the hold closes.
    ///
    /// Three separate regressions die here, which is why the assertions are
    /// interleaved with the hold rather than taken after it:
    ///
    /// - `Core::commit` fsyncing per event again — the counter reads 6;
    /// - `Core::commit` broadcasting per event again — the subscriber has
    ///   frames while the hold is still open, i.e. an SSE client could learn
    ///   about an event a crash would take back;
    /// - `CoreGuard::drop` not flushing — the counter never leaves 0 and the
    ///   six events are never published at all.
    #[tokio::test]
    async fn one_lock_hold_costs_one_fsync_and_publishes_only_when_it_ends() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let state = test_state(dir.path()).await;
        let mut live = CoreGuard::acquire(&state.core).await.events_tx.subscribe();

        {
            let mut core = CoreGuard::acquire(&state.core).await;
            for n in 1..=6u32 {
                core.commit(seeded(n)).expect("commit");
            }
            assert_eq!(
                core.journal.fsync_count(),
                0,
                "six appends under one hold must not have cost six fsyncs — \
                 the fsync belongs to the hold, not to the append"
            );
            assert_eq!(core.open_group_len(), 6, "the group is still open");
            assert!(
                matches!(live.try_recv(), Err(broadcast::error::TryRecvError::Empty)),
                "nothing may be published before it is durable: a subscriber \
                 that sees an event mid-group can be told about a fact the \
                 next instant's crash removes"
            );
        }

        let core = CoreGuard::acquire(&state.core).await;
        assert_eq!(
            core.journal.fsync_count(),
            1,
            "closing the hold costs exactly one fsync for all six"
        );
        assert_eq!(core.open_group_len(), 0, "and leaves nothing open");
        drop(core);

        let mut published = Vec::new();
        while let Ok(event) = live.try_recv() {
            published.push(event.seq);
        }
        assert_eq!(
            published,
            vec![1, 2, 3, 4, 5, 6],
            "every event of the group is published, in order, once the group \
             is durable"
        );
    }

    /// A failed group fsync must publish nothing, in the order that makes
    /// that true — not just leave `open_group_len()` at zero afterward.
    ///
    /// Round-2 survivor (invariants-r2:INV-R2-01's mutation C1): reordering
    /// `Core::flush` to broadcast the group *before* consulting the fsync's
    /// result —
    ///
    /// ```text
    /// let synced = self.journal.sync();
    /// let group = std::mem::take(&mut self.open_group);
    /// for event in group { let _ = self.events_tx.send(event); }
    /// synced?;                                  // moved here, after the loop
    /// ```
    ///
    /// — is indistinguishable from the real code on the success path: every
    /// other test in this module only drives that path. It matters only when
    /// the fsync fails, and `flush`'s own doc comment states the property
    /// this guards: a failed group publishes nothing at all, because nothing
    /// outside the core may learn of an event a crash could still take back.
    /// This test is the one that actually fails the fsync (via
    /// [`crate::runtime::journal::tests::make_unsyncable_for_tests`], the
    /// same `O_PATH` injection `journal`'s own poisoning test uses) and
    /// checks the subscriber's mailbox, not just the group's length —
    /// `open_group_len() == 0` is equally true whether the events were
    /// dropped or published-then-dropped, so only `try_recv` tells the two
    /// apart.
    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn a_failed_group_sync_publishes_nothing_not_even_before_returning_the_error() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let state = test_state(dir.path()).await;
        let mut live = CoreGuard::acquire(&state.core).await.events_tx.subscribe();

        let mut core = CoreGuard::acquire(&state.core).await;
        for n in 1..=3u32 {
            core.commit(seeded(n)).expect("commit");
        }
        assert_eq!(core.open_group_len(), 3, "the group is open before flush");

        if !crate::runtime::journal::tests::make_unsyncable_for_tests(&mut core.journal) {
            eprintln!(
                "SKIPPED-ENV: this host cannot express the O_PATH fsync-failure injection \
                 (no O_PATH support, or fsync accepts an O_PATH descriptor here)"
            );
            return;
        }

        core.flush()
            .expect_err("a failed group fsync must return an error");
        assert_eq!(
            core.open_group_len(),
            0,
            "a failed group is abandoned, not retried on the next hold"
        );
        assert!(
            matches!(live.try_recv(), Err(broadcast::error::TryRecvError::Empty)),
            "a group whose fsync failed must never reach a live subscriber — \
             not even by publishing it first and reporting the error after: \
             an SSE client told about these events would be told about a \
             fact the crash that just poisoned the journal may have taken \
             back"
        );
    }

    /// Guard for the history loop's cursor bookkeeping in `forward_events`
    /// (`last_sent = event.seq`), which nothing else in the suite reaches.
    ///
    /// That assignment has no effect on what history itself delivers —
    /// `events_after` is called once, before the loop, so the tail is already
    /// fixed. It matters in exactly one place: it is the floor a *later*
    /// refill or dedup starts from. `m2_daemon_api.rs`'s mid-replay resume
    /// test cannot see it (it reads history frames and disconnects, never
    /// reaching the live half), and lag cannot be provoked through the real
    /// HTTP surface without pushing past the daemon's 1024-slot broadcast.
    /// Driving `forward_events` directly makes it deterministic instead:
    ///
    /// 1. a one-slot sink wedges the pump *inside* the history loop, so it
    ///    has not yet called `live.recv()`;
    /// 2. twenty commits then overflow the sixteen-slot broadcast, so the
    ///    subscriber's very first `recv()` is `Lagged` — by the channel's own
    ///    overwrite contract, not by scheduler luck;
    /// 3. draining resumes history to its end, and the refill that follows
    ///    starts from `last_sent`.
    ///
    /// One frame per event is `send_sse`'s contract, so an exact frame count
    /// is an exact "no gap, no repeat": a cursor left one short re-delivers
    /// the last history frame and this reads 26.
    #[tokio::test]
    async fn a_lag_refill_resumes_from_the_last_history_frame_the_pump_actually_sent() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let state = test_state(dir.path()).await;

        const HISTORY: u32 = 5;
        const LIVE: u32 = 20; // > the 16-slot broadcast `test_state` builds
        {
            let mut core = CoreGuard::acquire(&state.core).await;
            for n in 1..=HISTORY {
                core.commit(seeded(n)).expect("commit history");
            }
        }

        let (tx, mut rx) = mpsc::channel(1);
        let pump = tokio::spawn(forward_events(state.clone(), 0, tx));

        // One frame out: the pump is past `subscribe` and inside the history
        // loop. It cannot reach `live.recv()` from here — three history
        // frames remain and the sink holds one — so everything committed
        // below piles up unread in the broadcast ring.
        let first = rx.recv().await.expect("the first history frame");
        assert!(first.is_ok(), "the pump never sends a stream error");

        {
            let mut core = CoreGuard::acquire(&state.core).await;
            for n in HISTORY + 1..=HISTORY + LIVE {
                core.commit(seeded(n)).expect("commit live");
            }
        }

        let total = (HISTORY + LIVE) as usize;
        let mut frames = 1;
        while frames < total {
            let frame = tokio::time::timeout(Duration::from_secs(5), rx.recv())
                .await
                .expect("every committed event must reach the stream")
                .expect("the pump must not close the stream");
            assert!(frame.is_ok(), "the pump never sends a stream error");
            frames += 1;
        }
        let extra = tokio::time::timeout(Duration::from_millis(500), rx.recv()).await;
        assert!(
            extra.is_err(),
            "the lag refill must start after the last history frame the pump \
             sent, not repeat it: {extra:?}"
        );

        pump.abort();
    }

    /// The TOCTOU this test provokes: `with_analytics` reads `last_seq`,
    /// releases the analytics lock to fetch the journal tail, then
    /// re-acquires the lock to fold it. A concurrent request's `catch_up`
    /// can run to completion — and fail — in that gap, which resets the
    /// projection to seq 0 behind the first request's back. Folding the
    /// first request's *stale* tail on top of that reset would silently
    /// skip the events between 0 and its old `last_seq`, exactly the
    /// "silently short table" the fail-closed contract forbids. The fix
    /// must detect the mismatch and re-fetch from the projection's real
    /// position instead of trusting the stale batch.
    #[tokio::test]
    async fn a_concurrent_catch_up_failure_never_folds_a_stale_tail() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let state = test_state(dir.path()).await;

        // Seed three events and catch the projection up to them, so
        // `last_seq` starts at 3.
        {
            let mut core = CoreGuard::acquire(&state.core).await;
            for n in 1..=3u32 {
                core.commit(seeded(n)).expect("commit");
            }
        }
        let pending = CoreGuard::acquire(&state.core)
            .await
            .events_after(0)
            .expect("events_after");
        state
            .analytics
            .lock()
            .await
            .catch_up(pending.into_iter().map(Ok))
            .expect("seed catch-up");

        // Two more events arrive — the "work submission" racing the
        // analytics requests below.
        {
            let mut core = CoreGuard::acquire(&state.core).await;
            for n in 4..=5u32 {
                core.commit(seeded(n)).expect("commit");
            }
        }

        // Hold the core lock so the spawned request is guaranteed to be
        // parked between its `last_seq` read and its `events_after` fetch
        // (both uncontended locks resolve without yielding, so it reaches
        // exactly that point on its first poll) when the concurrent
        // failure below runs.
        let core_guard = CoreGuard::acquire(&state.core).await;

        let state_a = state.clone();
        let task_a =
            tokio::spawn(async move { with_analytics(&state_a, Analytics::table_counts).await });
        tokio::task::yield_now().await;
        tokio::task::yield_now().await;

        // A concurrent request's catch-up fails partway. The fail-closed
        // contract requires this to leave the projection reporting
        // `last_seq() == 0` until the next successful catch-up.
        {
            let mut analytics = state.analytics.lock().await;
            let failing = std::iter::once(Err(JournalError::Io(std::io::Error::other(
                "injected failure",
            ))));
            assert!(
                analytics.catch_up(failing).is_err(),
                "the injected failure must surface"
            );
            assert_eq!(analytics.last_seq(), 0, "a failed fold must fail closed");
        }

        drop(core_guard);

        let result = task_a.await.expect("task a joined");
        let (counts, seq) = match result {
            Ok(v) => v,
            Err(_) => panic!("a catch-up raced by a concurrent failure must not answer short"),
        };
        let events_count = counts
            .into_iter()
            .find(|(table, _)| table == "events")
            .map(|(_, count)| count)
            .expect("events table present");
        assert_eq!(
            events_count, 5,
            "catch-up raced by a concurrent failure must fold the whole journal, not just the stale tail it fetched before the reset"
        );
        assert_eq!(
            seq, 5,
            "the projection must report itself caught up to the real journal head"
        );
    }

    // ------------------------------------------------- work_transcript

    /// Build a minimal `Event` for `transcript_turns` — every field the
    /// function itself reads (`seq`, `kind`, `work_id`, `payload`), plus the
    /// envelope fields `Event` requires to exist at all.
    fn ev(seq: u64, work_id: &str, kind: &str, payload: Value) -> Event {
        ev_exec(seq, work_id, None, kind, payload)
    }

    /// [`ev`], plus an explicit `execution_id` — the de-dup fix's own tests
    /// need two events sharing one (`conversation.assistant.completed` and
    /// its turn's `conversation.turn.ended`, exactly as one `TurnReader` run
    /// emits both).
    fn ev_exec(
        seq: u64,
        work_id: &str,
        execution_id: Option<&str>,
        kind: &str,
        payload: Value,
    ) -> Event {
        Event {
            schema: EVENT_SCHEMA.to_string(),
            seq,
            id: format!("evt-{seq}"),
            timestamp: rfc3339_utc_now(),
            source: EventSource::new("backend", "test"),
            workspace_id: None,
            work_id: Some(work_id.to_string()),
            execution_id: execution_id.map(str::to_string),
            correlation_id: None,
            causation_id: None,
            kind: kind.to_string(),
            payload,
            extra: Default::default(),
        }
    }

    /// The mapping `transcript_turns` exists for: `conversation.user`,
    /// `conversation.assistant.completed` and `conversation.ask` events for
    /// *this* work decode to `{role, text, source: "event"}` in causal (seq)
    /// order, and events belonging to a different work are dropped.
    ///
    /// guard-map: mutating any `KIND_CONVERSATION_*` match arm's `role` or
    /// the payload key it reads (`"text"` vs `"question"`) makes this fail;
    /// so does dropping the `work_id` filter or reordering by anything other
    /// than input order (which is already seq-ascending, as the journal
    /// guarantees).
    #[test]
    fn transcript_turns_decodes_conversation_events_in_causal_order_for_one_work() {
        let events = vec![
            ev(
                1,
                "w1",
                KIND_CONVERSATION_USER,
                json!({"text": "please do the thing"}),
            ),
            // A different work's event must never leak into w1's transcript.
            ev(
                2,
                "w2",
                KIND_CONVERSATION_USER,
                json!({"text": "unrelated work"}),
            ),
            ev(
                3,
                "w1",
                KIND_CONVERSATION_ASSISTANT_COMPLETED,
                json!({"text": "on it"}),
            ),
            ev(
                4,
                "w1",
                KIND_CONVERSATION_ASK,
                json!({"question": "which environment?"}),
            ),
        ];
        let data_dir = tempfile::TempDir::new().expect("tempdir");
        let turns = transcript_turns("w1", events, data_dir.path());
        let shape: Vec<(u64, &str, &str, &str)> = turns
            .iter()
            .map(|t| {
                (
                    t["seq"].as_u64().unwrap(),
                    t["role"].as_str().unwrap(),
                    t["text"].as_str().unwrap(),
                    t["source"].as_str().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            shape,
            vec![
                (1, "user", "please do the thing", "event"),
                (3, "assistant", "on it", "event"),
                (4, "ask", "which environment?", "event"),
            ],
            "w2's event must be excluded and the rest must decode in seq order: {turns:?}"
        );
    }

    /// The "minimal blob decode" itself, end to end through
    /// `transcript_turns`: a `conversation.turn.ended` with
    /// `result_envelope: false` and a `raw` blob ref recovers whatever
    /// assistant `text` blocks the archived stream-json carries, tagged
    /// `source: "blob_decode"` so a reader can tell it apart from an
    /// ordinary journaled event.
    ///
    /// guard-map: removing the `store.get`/`decode_partial_assistant_text`
    /// call, or the `result_envelope` early-return, makes this fail (the
    /// former by never recovering the text, the latter by double-reporting
    /// or misfiring on ordinary completed turns).
    #[test]
    fn transcript_turns_recovers_partial_text_from_an_interrupted_turns_raw_archive() {
        let data_dir = tempfile::TempDir::new().expect("tempdir");
        let store =
            crate::runtime::blob::BlobStore::open(data_dir.path()).expect("open blob store");
        // A raw stream-json archive: one assistant line with a partial text
        // block, as if the turn were cut mid-stream (no trailing `result`
        // line — that absence is exactly why `conversation.assistant.
        // completed` never got emitted for this turn).
        let archive = concat!(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"partial reply before the cut"}]}}"#,
            "\n"
        );
        let blob_ref = store.put(archive.as_bytes()).expect("store blob");

        let events = vec![ev(
            1,
            "w1",
            KIND_CONVERSATION_TURN_ENDED,
            json!({
                "interrupted": true,
                "result_envelope": false,
                "raw": blob_ref.to_string(),
            }),
        )];
        let turns = transcript_turns("w1", events, data_dir.path());
        assert_eq!(
            turns.len(),
            1,
            "the interrupted turn must recover one entry: {turns:?}"
        );
        assert_eq!(turns[0]["role"], "assistant");
        assert_eq!(turns[0]["text"], "partial reply before the cut");
        assert_eq!(turns[0]["source"], "blob_decode");
        assert_eq!(turns[0]["interrupted"], true);
    }

    /// A turn that closed *with* a result envelope must never trigger the
    /// blob-decode fallback — its content, if any, already reached the
    /// journal as its own `conversation.assistant.completed` event, and
    /// decoding the archive too would double-report it (or fabricate an
    /// entry for a tool-only turn that produced no text at all).
    #[test]
    fn transcript_turns_never_decodes_the_archive_of_a_turn_that_ended_with_an_envelope() {
        let data_dir = tempfile::TempDir::new().expect("tempdir");
        let store =
            crate::runtime::blob::BlobStore::open(data_dir.path()).expect("open blob store");
        let archive = concat!(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"should never surface"}]}}"#,
            "\n"
        );
        let blob_ref = store.put(archive.as_bytes()).expect("store blob");
        let events = vec![ev(
            1,
            "w1",
            KIND_CONVERSATION_TURN_ENDED,
            json!({
                "interrupted": false,
                "result_envelope": true,
                "raw": blob_ref.to_string(),
            }),
        )];
        let turns = transcript_turns("w1", events, data_dir.path());
        assert!(
            turns.is_empty(),
            "a turn that ended with an envelope must not decode its archive: {turns:?}"
        );
    }

    /// MVP-3 test-honesty finding TH-1 (discovered while building its e2e
    /// closure, `tests/m4_backends.rs`'s
    /// `work_transcript_recovers_an_interrupted_turns_text_from_the_real_
    /// journal`): a turn interrupted *after* streaming a complete assistant
    /// text line ends with no `result` line — `result_envelope: false`, same
    /// as any other envelope-less turn — but `ingest_line` already emitted
    /// `conversation.assistant.completed` live for that line, independent of
    /// whether a `result` line ever follows. Before the fix, `transcript_
    /// turns` had no way to know that and decoded the archive too,
    /// reporting the same text twice. The `execution_id`-scoped fix must
    /// recover text ONLY for the execution/turn that genuinely never
    /// emitted it live.
    ///
    /// guard-map: dropping the `already_reached_the_journal` check (or its
    /// `execution_id` scoping) makes this fail with two "already said" turns
    /// instead of one.
    #[test]
    fn transcript_turns_never_double_reports_text_the_live_event_already_carried() {
        let data_dir = tempfile::TempDir::new().expect("tempdir");
        let store =
            crate::runtime::blob::BlobStore::open(data_dir.path()).expect("open blob store");
        let archive = concat!(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"already said"}]}}"#,
            "\n"
        );
        let blob_ref = store.put(archive.as_bytes()).expect("store blob");

        let events = vec![
            // Live-ingested during the turn, exactly as `ingest_line` does
            // for any complete `assistant` text line, whether or not a
            // `result` line ever follows.
            ev_exec(
                1,
                "w1",
                Some("e1"),
                KIND_CONVERSATION_ASSISTANT_COMPLETED,
                json!({"text": "already said"}),
            ),
            // The turn ends with no result envelope (interrupted mid-stream,
            // after the line above already landed).
            ev_exec(
                2,
                "w1",
                Some("e1"),
                KIND_CONVERSATION_TURN_ENDED,
                json!({
                    "interrupted": true,
                    "result_envelope": false,
                    "raw": blob_ref.to_string(),
                }),
            ),
        ];
        let turns = transcript_turns("w1", events, data_dir.path());
        assert_eq!(
            turns.len(),
            1,
            "the text already reached the journal live; the archive must not add a second \
             copy of it: {turns:?}"
        );
        assert_eq!(turns[0]["source"], "event");
        assert_eq!(turns[0]["text"], "already said");
    }

    /// The narrower case the finding above didn't cover: a turn that
    /// streamed *two* assistant lines, where only the first's
    /// `conversation.assistant.completed` reached the journal (the second
    /// lost in a simulated adjacent-append crash window) before
    /// `conversation.turn.ended` landed with `result_envelope: false`. The
    /// archive carries both lines' text. A per-execution boolean would treat
    /// "any line reached the journal" as "the whole turn did" and drop the
    /// second line's text entirely; the count-based fix must recover exactly
    /// the lines that never made it live.
    ///
    /// guard-map: reverting to a per-execution boolean (any emitted ⇒ skip
    /// the archive entirely) makes this fail by losing "line two" instead of
    /// recovering it.
    #[test]
    fn transcript_turns_recovers_only_the_lines_lost_to_a_partial_adjacent_append_crash() {
        let data_dir = tempfile::TempDir::new().expect("tempdir");
        let store =
            crate::runtime::blob::BlobStore::open(data_dir.path()).expect("open blob store");
        let archive = concat!(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"line one "}]}}"#,
            "\n",
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"line two"}]}}"#,
            "\n"
        );
        let blob_ref = store.put(archive.as_bytes()).expect("store blob");

        let events = vec![
            // Only line one's live event reached the journal; line two's own
            // `conversation.assistant.completed` was lost to the crash.
            ev_exec(
                1,
                "w1",
                Some("e1"),
                KIND_CONVERSATION_ASSISTANT_COMPLETED,
                json!({"text": "line one "}),
            ),
            ev_exec(
                2,
                "w1",
                Some("e1"),
                KIND_CONVERSATION_TURN_ENDED,
                json!({
                    "interrupted": true,
                    "result_envelope": false,
                    "raw": blob_ref.to_string(),
                }),
            ),
        ];
        let turns = transcript_turns("w1", events, data_dir.path());
        assert_eq!(
            turns.len(),
            2,
            "line one's live event plus line two's recovered text: {turns:?}"
        );
        assert_eq!(turns[0]["source"], "event");
        assert_eq!(turns[0]["text"], "line one ");
        assert_eq!(turns[1]["source"], "blob_decode");
        assert_eq!(
            turns[1]["text"], "line two",
            "line two never reached the journal live and must be recovered, not dropped: {turns:?}"
        );
    }

    /// A minimal scripted `claude` executable: passes the adapter's version/
    /// help probe, then on any other invocation prints `transcript` to
    /// stdout and exits 0. Written fresh per test so the probe cache (keyed
    /// per `ClaudeBackend` instance, not per path) never crosses tests.
    fn write_stub_claude(dir: &std::path::Path, transcript: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join("claude-stub");
        let replay = dir.join("replay.jsonl");
        std::fs::write(&replay, transcript).expect("write replay");
        let help = crate::backend::claude::REQUIRED_FLAGS.join(" ");
        let script = format!(
            "#!/bin/sh\ncase \"$1\" in\n  --version) echo '2.1.226 (Claude Code)';;\n  \
             --help) echo '{help}';;\n  *) cat '{replay}';;\nesac\n",
            replay = replay.display(),
        );
        std::fs::write(&path, script).expect("write stub");
        let mut permissions = std::fs::metadata(&path).expect("stat stub").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&path, permissions).expect("chmod stub");
        // A file another process still holds open for writing cannot be
        // exec'd; absorb the ETXTBSY window before handing it to the
        // adapter (#83 — measured under `cargo test --lib`'s default thread
        // parallelism: 3 failures in 40 runs, every one `os error 26`).
        crate::test_support::wait_until_executable(&path);
        path
    }

    /// MVP-3 test-honesty finding TH-1: a real producer (`backend::claude`'s
    /// `ClaudeBackend`/`TurnReader`, driven by a scripted `claude` CLI, not a
    /// hand-fabricated `Event`) all the way to `transcript_turns`'s blob-
    /// decode fallback — closing the gap the finding named: "no test spans
    /// producer→journal→transcript_turns", so a rename of the `raw`/
    /// `result_envelope` payload key on the producer side would leave this
    /// test failing instead of silently passing (its own guard-map below).
    ///
    /// The turn streams one real assistant text line and no `result` line
    /// (an envelope-less turn — same shape `backend::claude`'s own
    /// `partial_turn` fixture models). The event sink here converts every
    /// `EventDraft` into a real journaled `Event`, exactly as the daemon's
    /// own `journaling_sink` does, **except** it drops
    /// `conversation.assistant.completed` — modeling docs/DEVELOPMENT.md's own
    /// "adjacent-append crash window" (an event handed to the sink but
    /// never durably committed before the process holding it dies), which is
    /// this module's own doc comment's stated reason `decode_partial_
    /// assistant_text` exists at all (`work_transcript`'s doc, above). This
    /// is the *only* scenario in which the archive is not simply redundant
    /// with what the live event already carried (see the de-dup tests
    /// above) — proven by asserting the dropped kind really is absent from
    /// what reached the journal, not merely by not looking for it.
    ///
    /// guard-map: renaming the producer's `raw`/`result_envelope` payload
    /// keys (with `TurnReader::run`'s own emit call updated to match, so the
    /// producer is internally consistent) makes this fail — `transcript_
    /// turns` hits the `continue` fall-throughs and recovers nothing, while
    /// the two hand-fabricated unit tests above stay green regardless.
    #[test]
    fn transcript_turns_recovers_a_real_producers_text_across_a_simulated_adjacent_append_loss() {
        let data_dir = tempfile::TempDir::new().expect("tempdir");
        let stub_dir = tempfile::TempDir::new().expect("tempdir");
        let assistant_line = serde_json::json!({
            "type": "assistant",
            "message": {"content": [{"type": "text", "text": "cut off mid-thought"}]},
        })
        .to_string();
        let stub = write_stub_claude(stub_dir.path(), &format!("{assistant_line}\n"));

        let mut config = crate::backend::claude::ClaudeConfig::new(data_dir.path());
        config.executable = stub;
        let backend = crate::backend::claude::ClaudeBackend::new(config);

        let journaled: Arc<std::sync::Mutex<Vec<Event>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let next_seq = Arc::new(std::sync::atomic::AtomicU64::new(1));
        let (journaled_for_sink, seq_for_sink) = (Arc::clone(&journaled), Arc::clone(&next_seq));
        backend.set_event_sink(Arc::new(move |draft: EventDraft| {
            if draft.kind == KIND_CONVERSATION_ASSISTANT_COMPLETED {
                // Simulated crash-window loss: handed to the sink, never
                // committed. See this test's own doc comment.
                return;
            }
            let seq = seq_for_sink.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            journaled_for_sink
                .lock()
                .expect("journaled lock")
                .push(Event {
                    schema: EVENT_SCHEMA.to_string(),
                    seq,
                    id: format!("evt-{seq}"),
                    timestamp: rfc3339_utc_now(),
                    source: draft.source,
                    workspace_id: draft.workspace_id,
                    work_id: draft.work_id,
                    execution_id: draft.execution_id,
                    correlation_id: draft.correlation_id,
                    causation_id: draft.causation_id,
                    kind: draft.kind,
                    payload: draft.payload,
                    extra: Default::default(),
                });
        }));

        let request = crate::backend::StartRequest {
            work_id: "w-real".to_string(),
            execution_id: "e-real".to_string(),
            stage_id: "00-only".to_string(),
            attempt: 1,
            cwd: data_dir.path().to_path_buf(),
            intent: "say something and get cut off".to_string(),
            context: String::new(),
            model: None,
            profile: None,
            execute: None,
            instruction_policy: crate::domain::workspace::InstructionPolicy::default(),
        };
        let handle = {
            use crate::backend::Backend;
            backend.start(&request).expect("start")
        };

        // The stub exits right after replaying, so the turn settles on its
        // own — no interrupt/kill needed to get an envelope-less outcome.
        // A naturally-exited, envelope-less, non-interrupted turn reports
        // `NativeState::Unknown` (ambiguity fails closed — the same
        // classification `backend::claude`'s own
        // `a_turns_stderr_is_waited_for_rather_than_snapshotted` pins), not
        // `Exited`, so this waits for "no longer `Running`" rather than for
        // a specific terminal state.
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let observed = {
                use crate::backend::Backend;
                backend.observe(&handle).expect("observe")
            };
            if observed.native != crate::backend::NativeState::Running {
                break;
            }
            assert!(Instant::now() < deadline, "the stub turn never finished");
            std::thread::sleep(Duration::from_millis(20));
        }
        // The reader thread's archive write + emits land shortly after the
        // native state flips; a short settle margin avoids a flaky read of
        // `journaled` mid-write.
        let deadline = Instant::now() + Duration::from_secs(5);
        while journaled
            .lock()
            .expect("journaled lock")
            .iter()
            .all(|e| e.kind != KIND_CONVERSATION_TURN_ENDED)
        {
            assert!(
                Instant::now() < deadline,
                "conversation.turn.ended never reached the sink"
            );
            std::thread::sleep(Duration::from_millis(10));
        }

        let events = journaled.lock().expect("journaled lock").clone();
        assert!(
            events
                .iter()
                .all(|e| e.kind != KIND_CONVERSATION_ASSISTANT_COMPLETED),
            "the simulated loss must really have dropped it, not merely gone unchecked: {events:?}"
        );
        let ended = events
            .iter()
            .find(|e| e.kind == KIND_CONVERSATION_TURN_ENDED)
            .expect("the real producer must emit conversation.turn.ended");
        assert_eq!(
            ended.payload["result_envelope"], false,
            "no result line was replayed, so the real producer must say so: {}",
            ended.payload
        );

        let turns = transcript_turns("w-real", events, data_dir.path());
        let recovered = turns
            .iter()
            .find(|t| t["source"] == "blob_decode")
            .unwrap_or_else(|| panic!("must recover the real archive's text: {turns:?}"));
        assert_eq!(recovered["text"], "cut off mid-thought");
    }

    /// The de-dup above is scoped to *this* execution's *next* turn boundary
    /// only — a genuinely different execution (or a later turn on the same
    /// execution that streamed no assistant text of its own) must still get
    /// its own archive recovered independently.
    #[test]
    fn transcript_turns_still_recovers_a_different_executions_archive() {
        let data_dir = tempfile::TempDir::new().expect("tempdir");
        let store =
            crate::runtime::blob::BlobStore::open(data_dir.path()).expect("open blob store");
        let archive = concat!(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"never live-emitted"}]}}"#,
            "\n"
        );
        let blob_ref = store.put(archive.as_bytes()).expect("store blob");

        let events = vec![
            // `e1`'s assistant text reached the journal live.
            ev_exec(
                1,
                "w1",
                Some("e1"),
                KIND_CONVERSATION_ASSISTANT_COMPLETED,
                json!({"text": "e1 said this live"}),
            ),
            ev_exec(
                2,
                "w1",
                Some("e1"),
                KIND_CONVERSATION_TURN_ENDED,
                json!({"interrupted": false, "result_envelope": true, "raw": Value::Null}),
            ),
            // `e2` is a different execution that never got as far as
            // emitting any assistant line live (killed before one parsed).
            ev_exec(
                3,
                "w1",
                Some("e2"),
                KIND_CONVERSATION_TURN_ENDED,
                json!({
                    "interrupted": true,
                    "result_envelope": false,
                    "raw": blob_ref.to_string(),
                }),
            ),
        ];
        let turns = transcript_turns("w1", events, data_dir.path());
        let shape: Vec<(&str, &str)> = turns
            .iter()
            .map(|t| (t["source"].as_str().unwrap(), t["text"].as_str().unwrap()))
            .collect();
        assert_eq!(
            shape,
            vec![
                ("event", "e1 said this live"),
                ("blob_decode", "never live-emitted"),
            ],
            "e2's archive must still be recovered independently of e1's: {turns:?}"
        );
    }

    /// `decode_partial_assistant_text` in isolation: it reads only
    /// `type: "assistant"` lines' `/message/content` text blocks, in file
    /// order, and ignores lines it cannot parse or that carry no text block
    /// (system lines, tool_use blocks) rather than erroring on them.
    #[test]
    fn decode_partial_assistant_text_reads_only_assistant_text_blocks_in_order() {
        let archive = [
            r#"{"type":"system","subtype":"init"}"#,
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"first "}]}}"#,
            "not even json",
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"grep"}]}}"#,
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"second"}]}}"#,
        ]
        .join("\n");
        assert_eq!(
            decode_partial_assistant_text(archive.as_bytes()),
            "first second"
        );
    }

    /// ADR 0007(b) hardening found in review of the feature's first cut:
    /// `fleet_body` originally read a work's run straight out of
    /// `registry.run_view`'s *bounded* terminal-run cache, with no fallback
    /// once an entry aged out — the exact silent-revert gap `work_view`'s
    /// own `resolve_run` call already closes for `sgt work show`. Left
    /// unfixed, `sgt work list` would report plain `completed` for a
    /// stranded completion the moment its run fell out of the cache, while
    /// `sgt work show` for the identical id still said `completed_dirty`.
    ///
    /// Cheap at the unit level, the same way `projection.rs`'s own
    /// `the_terminal_run_cache_itself_stays_bounded_under_churn_beyond_its_
    /// capacity` is: journal commits directly against a bare `Core`, no
    /// HTTP, no daemon.
    #[test]
    fn a_stranded_completion_survives_terminal_run_cache_eviction() {
        use crate::runtime::testing;

        fn surface(work_id: &str) -> Value {
            json!({"surface": {
                "work_id": work_id,
                "root": "/data/surfaces/x",
                "bindings": [{
                    "repository": "solo",
                    "source_path": "/repos/solo",
                    "base_branch": "main",
                    "base_sha": "0".repeat(40),
                    "worktree_path": "/data/surfaces/x/solo",
                    "work_branch": format!("sergeant/{work_id}"),
                    "head_sha": "0".repeat(40),
                }],
            }})
        }

        fn stranded_teardown(work_id: &str) -> Value {
            json!({"report": {
                "work_id": work_id,
                "clean": false,
                "bindings": [{
                    "repository": "solo",
                    "worktree_path": "/data/surfaces/x/solo",
                    "work_branch": format!("sergeant/{work_id}"),
                    "disposition": "retained_dirty",
                    "changes": " M half-done.rs",
                    // Never advanced past the base SHA `surface` recorded.
                    "final_sha": "0".repeat(40),
                }],
            }})
        }

        fn ordinary_teardown(work_id: &str) -> Value {
            json!({"report": {
                "work_id": work_id,
                "clean": true,
                "bindings": [{
                    "repository": "solo",
                    "worktree_path": "/data/surfaces/x/solo",
                    "work_branch": format!("sergeant/{work_id}"),
                    "disposition": "removed",
                }],
            }})
        }

        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());

        let stranded_id = "01STRANDED0000000001";
        testing::submit(&mut core, stranded_id, "declares a commit, never makes one");
        testing::commit(
            &mut core,
            stranded_id,
            KIND_SURFACE_MATERIALIZED,
            surface(stranded_id),
        );
        testing::commit(&mut core, stranded_id, KIND_WORK_COMPLETED, json!({}));
        testing::commit(
            &mut core,
            stranded_id,
            KIND_SURFACE_TORN_DOWN,
            stranded_teardown(stranded_id),
        );

        // Churn ordinary completions past the terminal-run cache's own bound
        // (`projection.rs`'s `TERMINAL_RUN_CACHE_CAPACITY`, 512) so the
        // stranded work above — submitted first, so it is the oldest —
        // actually ages out of it.
        for i in 0..600 {
            let id = format!("01CACHECHURN{i:06}");
            testing::submit(&mut core, &id, "cache churn");
            testing::commit(&mut core, &id, KIND_SURFACE_MATERIALIZED, surface(&id));
            testing::commit(&mut core, &id, KIND_WORK_COMPLETED, json!({}));
            testing::commit(
                &mut core,
                &id,
                KIND_SURFACE_TORN_DOWN,
                ordinary_teardown(&id),
            );
        }

        assert!(
            core.registry.state().run_view(stranded_id).is_none(),
            "the stranded work's run must actually have aged out of the \
             bounded cache for this test to prove anything"
        );

        let backends = BackendRegistry::new().with(Arc::new(
            crate::backend::fake::FakeBackend::new(crate::backend::fake::FAKE_BACKEND_NAME),
        ));
        let engine = Engine::new(
            Arc::new(backends),
            Some(crate::backend::fake::FAKE_BACKEND_NAME.to_string()),
            dir.path(),
        );

        // `fleet_body` is what `sgt work list` actually serves. Before this
        // fix it read the run straight out of the bounded cache and had no
        // fallback once an entry aged out, so it would silently report
        // plain `completed` here — exactly what `sgt work show` (`work_view`,
        // via `resolve_run`) never does for the same work.
        let fleet = fleet_body(&core, &engine);
        let row = fleet["works"]
            .as_array()
            .expect("works")
            .iter()
            .find(|w| w["id"] == stranded_id)
            .expect("the evicted work is still listed");
        assert_eq!(
            row["state"], "completed_dirty",
            "an evicted stranded completion must not silently revert to plain \
             completed in `sgt work list` just because its run cache entry \
             aged out: {row}"
        );
    }

    /// Amendment C3, the whole of it: a `surface.torn_down` journaled before
    /// Phase A existed replays unchanged, and its integrity reads as **not
    /// assessed** — never defaulted to clean.
    ///
    /// The distinction is the entire point of the amendment. An absent
    /// assessment defaulted to `clean` would be core inventing a fact about
    /// a retirement that predates the machinery that could have established
    /// it — the exact silent-clean-report failure §17 says must become
    /// impossible. So the payload here is hand-built rather than serialized
    /// from today's types (the same technique
    /// `a_stranded_completion_survives_terminal_run_cache_eviction` above
    /// uses): a struct that gains a field cannot prove anything about an
    /// event that never had it, only a literal old-shape payload can.
    ///
    /// Three things are checked past the disposition itself: the report
    /// still deserializes into a `TeardownReport` (so `run.teardown` is
    /// `Some`, and `runtime::recovery`'s completion-tail sweep — keyed on
    /// `teardown.is_none()` — does not suddenly consider this work stranded);
    /// ADR 0007(b)'s `completed_dirty` still fires from the output pointer
    /// alone, which is all a pre-Phase-A journal can offer; and a new-shape
    /// payload beside it does report its assessment.
    #[test]
    fn an_old_shape_torn_down_payload_replays_as_integrity_not_assessed() {
        use crate::runtime::testing;

        fn surface(work_id: &str) -> Value {
            json!({"surface": {
                "work_id": work_id,
                "root": "/data/surfaces/x",
                "bindings": [{
                    "repository": "solo",
                    "source_path": "/repos/solo",
                    "base_branch": "main",
                    "base_sha": "0".repeat(40),
                    "worktree_path": "/data/surfaces/x/solo",
                    "work_branch": format!("sergeant/{work_id}"),
                    "head_sha": "0".repeat(40),
                }],
            }})
        }

        // Exactly the shape `surface.torn_down` had before this phase: no
        // `integrity` sibling key, no `findings`, no `observed_head`, no
        // `drift`.
        fn old_shape_teardown(work_id: &str, clean: bool, disposition: Value) -> Value {
            let mut binding = json!({
                "repository": "solo",
                "worktree_path": "/data/surfaces/x/solo",
                "work_branch": format!("sergeant/{work_id}"),
                "final_sha": "0".repeat(40),
            });
            let object = binding.as_object_mut().expect("binding object");
            for (key, value) in disposition.as_object().expect("disposition object") {
                object.insert(key.clone(), value.clone());
            }
            json!({"report": {"work_id": work_id, "clean": clean, "bindings": [binding]}})
        }

        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());

        // An ordinary old completion: clean removal, nothing to report.
        let ordinary = "01OLDSHAPE0000000001";
        testing::submit(&mut core, ordinary, "torn down before Phase A");
        testing::commit(
            &mut core,
            ordinary,
            KIND_SURFACE_MATERIALIZED,
            surface(ordinary),
        );
        testing::commit(&mut core, ordinary, KIND_WORK_COMPLETED, json!({}));
        testing::commit(
            &mut core,
            ordinary,
            KIND_SURFACE_TORN_DOWN,
            old_shape_teardown(ordinary, true, json!({"disposition": "removed"})),
        );

        // An old stranded completion: ADR 0007(b)'s inference is all the
        // journal can offer, and it must keep working untouched.
        let stranded = "01OLDSHAPE0000000002";
        testing::submit(&mut core, stranded, "declares a commit, never makes one");
        testing::commit(
            &mut core,
            stranded,
            KIND_SURFACE_MATERIALIZED,
            surface(stranded),
        );
        testing::commit(&mut core, stranded, KIND_WORK_COMPLETED, json!({}));
        testing::commit(
            &mut core,
            stranded,
            KIND_SURFACE_TORN_DOWN,
            old_shape_teardown(
                stranded,
                false,
                json!({"disposition": "retained_dirty", "changes": " M half-done.rs"}),
            ),
        );

        for work_id in [ordinary, stranded] {
            let registry = core.registry.state();
            let run = registry.run_view(work_id).expect("run");
            assert!(
                run.teardown.is_some(),
                "{work_id}: the old-shape report must still deserialize, or \
                 recovery's completion-tail sweep would treat this work as \
                 stranded on every restart"
            );
            assert_eq!(
                run.integrity, None,
                "{work_id}: an absent assessment is 'not assessed', never clean"
            );
        }

        let backends = BackendRegistry::new().with(Arc::new(
            crate::backend::fake::FakeBackend::new(crate::backend::fake::FAKE_BACKEND_NAME),
        ));
        let engine = Engine::new(
            Arc::new(backends),
            Some(crate::backend::fake::FAKE_BACKEND_NAME.to_string()),
            dir.path(),
        );

        let ordinary_view = work_view(&core, &engine, ordinary);
        assert!(
            ordinary_view["integrity"].is_null(),
            "not assessed renders as null, not as a clean disposition: {ordinary_view}"
        );
        assert_eq!(ordinary_view["work"]["state"], "completed");
        assert_eq!(
            work_view(&core, &engine, stranded)["work"]["state"],
            "completed_dirty",
            "ADR 0007(b)'s output-pointer inference is unchanged by §11"
        );

        // And the new shape does carry its assessment through the same fold.
        let assessed = "01NEWSHAPE0000000001";
        testing::submit(&mut core, assessed, "assessed at retirement");
        testing::commit(
            &mut core,
            assessed,
            KIND_SURFACE_MATERIALIZED,
            surface(assessed),
        );
        testing::commit(&mut core, assessed, KIND_WORK_COMPLETED, json!({}));
        let mut payload = old_shape_teardown(assessed, true, json!({"disposition": "removed"}));
        payload["report"]["bindings"][0]["findings"] = json!([{
            "finding": "assigned_branch_mismatch",
            "repository": "solo",
            "worktree_path": "/data/surfaces/x/solo",
            "expected_branch": format!("sergeant/{assessed}"),
            "expected_sha": "0".repeat(40),
            "observed_branch": "renegade",
            "observed_sha": "1".repeat(40),
            "evidence": "ended on renegade",
        }]);
        payload["integrity"] = json!("dirty");
        testing::commit(&mut core, assessed, KIND_SURFACE_TORN_DOWN, payload);

        let view = work_view(&core, &engine, assessed);
        assert_eq!(view["integrity"]["disposition"], "dirty");
        assert_eq!(
            view["integrity"]["findings"][0]["finding"],
            "assigned_branch_mismatch"
        );
        assert_eq!(
            view["work"]["state"], "completed_dirty",
            "a dirty completion reports the §11.5 compact label even without \
             ADR 0007(b)'s stranded inference"
        );
    }
}
