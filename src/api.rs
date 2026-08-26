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

use std::collections::BTreeMap;
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
use crate::backend::codex::KIND_TURN_HARNESS_ERROR;
use crate::cli::doctor;
use crate::daemon::{
    KIND_ADMISSION_PAUSED, KIND_ADMISSION_RESUMED, KIND_BACKEND_PROBED, KIND_DAEMON_STARTED,
    KIND_DAEMON_STOPPED,
};
use crate::domain::estate::{Estate, InstructionPolicy};
use crate::domain::event::{Event, EventDraft, EventSource, rfc3339_utc_now};
use crate::domain::execution::{
    KIND_EXECUTION_ABANDONED, KIND_EXECUTION_RECONCILED, KIND_EXECUTION_RESERVED,
    KIND_EXECUTION_STARTED, KIND_EXECUTION_STOPPED,
};
use crate::domain::manifest::{self, ManifestError};
use crate::domain::work::{
    EnvelopeRequest, IntentDetail, KIND_COMMAND_ACCEPTED, KIND_COMMAND_REJECTED, KIND_WORK_BLOCKED,
    KIND_WORK_CANCELED, KIND_WORK_COMPLETED, KIND_WORK_FAILED, KIND_WORK_NEEDS_INPUT,
    KIND_WORK_RESUMED, KIND_WORK_STARTED, KIND_WORK_SUBMITTED, KIND_WORK_WAITING, ScopeRequest,
    Work, WorkState,
};
use crate::domain::workflow::{
    self, KIND_STAGE_BLOCKED, KIND_STAGE_CANCELED, KIND_STAGE_COMPLETED, KIND_STAGE_ENTERED,
    KIND_STAGE_FAILED, KIND_STAGE_INPUT_RECEIVED, KIND_STAGE_NEEDS_INPUT,
    KIND_STAGE_OUTPUT_MISSING, KIND_STAGE_RESUMED, KIND_STAGE_WAITING, KIND_WORKFLOW_BOUND,
    WorkflowDefinition,
};
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
    Projection, ProjectionError, WorkIndexRow, WorkRegistry, WorkRun, is_absorbing, rederive_run,
    rederive_work,
};
use crate::runtime::startup::{FloorCommandClass, FloorCommandRow};
use crate::runtime::surface::{
    BindingDisposition, KIND_SURFACE_MATERIALIZED, KIND_SURFACE_MATERIALIZING,
    KIND_SURFACE_TORN_DOWN, reap, retained_bindings,
};
use crate::runtime::sweep::{self, SweepTarget};

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
    /// W2 §26 keys for commands recorded below this process's replay window
    /// (Q8). Loaded once from the startup cache and never mutated: every
    /// command this process records lands in `registry.commands`, which
    /// [`replay_command`] consults first. Empty on a full-replay start,
    /// because then the registry already holds every command the journal
    /// knows.
    pub floor_ledger: Arc<BTreeMap<String, FloorCommandRow>>,
    /// W3 §2.5: first seq this process has seen for each retained Work — the
    /// `first_seq(id)` half of the prune horizon's no-straddle predicate.
    /// Seeded once at start (cache rows ∪ the startup pass's `HorizonSink`),
    /// advanced by [`Core::commit`] (`entry().or_insert(event.seq)`), and
    /// pruned ids removed when a `prune.completed` is folded.
    pub first_seq_by_work: crate::runtime::prune::FirstSeqIndex,
    /// W3 §2.5 / §10.2: a prune cycle the last tick or start could not
    /// finish — because a re-validation aborted it, or because it failed.
    /// Re-arms the next tick's attempt without waiting for a rotation.
    pub prune_pending: bool,
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
            floor_ledger: Arc::new(std::collections::BTreeMap::new()),
            first_seq_by_work: std::collections::BTreeMap::new(),
            prune_pending: false,
            open_group: Vec::new(),
        }
    }

    /// W2: attach the startup cache's below-window command ledger. A
    /// separate builder rather than a `Core::new` parameter so the three
    /// existing test constructors stay untouched — an empty ledger (the
    /// `Core::new` default) is exactly right for them, since none replays a
    /// cache.
    pub fn with_floor_ledger(
        mut self,
        floor_ledger: Arc<BTreeMap<String, FloorCommandRow>>,
    ) -> Self {
        self.floor_ledger = floor_ledger;
        self
    }

    /// W3 §2.5: seed `first_seq_by_work` from the merged cache-rows ∪
    /// startup-pass index a fresh start computed. A separate builder for the
    /// same reason `with_floor_ledger` is one — the existing test
    /// constructors stay untouched, since an empty index (no Work has been
    /// seen yet) is exactly right for them.
    pub fn with_first_seq_index(
        mut self,
        first_seq_by_work: crate::runtime::prune::FirstSeqIndex,
    ) -> Self {
        self.first_seq_by_work = first_seq_by_work;
        self
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
        // W3 §2.5: the first seq this process has ever seen for `work_id`.
        // `or_insert` — once set, a Work's own `first_seq` never moves,
        // exactly like `HorizonSink`'s startup-time fold this seeds from.
        if let Some(work_id) = &event.work_id {
            self.first_seq_by_work
                .entry(work_id.clone())
                .or_insert(event.seq);
        }
        // A completed prune removes the pruned ids: nothing about them can
        // be asked of this index again (the events that would answer are
        // gone), and leaving stale entries here would grow it without bound
        // across the estate's life.
        if event.kind == crate::runtime::prune::KIND_PRUNE_COMPLETED {
            let live = &self.registry.state().work_index;
            self.first_seq_by_work.retain(|id, _| live.contains_key(id));
        }
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
/// (`sergeant-rs-workspace's knowledge/evidence/perf/n3-two-phase-boundary-2026-08-10.md`); at one fsync each, on a
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
    ///
    /// D10: it carries **no** estate. Every handler that needs topology
    /// admits the estate its request addressed (below) and hands the root to
    /// `Engine::plan` per call.
    pub engine: Arc<Engine>,
    /// H1 §4: the admitted-estate registry — this daemon's observational
    /// record of every estate it has validated, keyed by canonical root and
    /// rebuilt from requests. Nothing is served for an estate that is not in
    /// here, and nothing gets in here without passing admission.
    pub estates: Arc<crate::runtime::estates::EstateRegistry>,
    /// The disposable DuckDB analytical + graph projection (§21–§23).
    ///
    /// Behind its own lock, not the core's: an analytics query is a read of a
    /// derived file and must never be able to stall a mutation. It is caught
    /// up from the journal at query time (see [`with_analytics`]), so a
    /// failure anywhere in here costs an answer, never a fact.
    pub analytics: Arc<tokio::sync::Mutex<Analytics>>,
    /// W3: the retention policy this daemon resolved once at start, pinned
    /// for its whole life (§1.2) — read by [`drive_completions`]'s rotation
    /// trigger (§10.4).
    pub prune_policy: crate::runtime::prune::PrunePolicy,
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
        .route("/sweep", get(sweep_estate).post(sweep_delete))
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
        .route("/estates", get(list_estates))
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
        if *closing.borrow() {
            return;
        }
        maybe_run_rotation_triggered_prune(&state).await;
    }
}

/// W3 §10.4: the rotation-triggered prune maintenance step, run once per
/// tick after the observe/interrupt work above. A failure anywhere here is
/// logged and the tick continues — a daemon that cannot prune must keep
/// serving; [`crate::runtime::prune::stall_report`] is how that becomes
/// visible, not a blocked tick.
///
/// Phase A ([`crate::runtime::prune::candidate_horizon_multi_estate`]) and
/// the cheap `take_rotation_signal`/`segment_bounds` reads run under the
/// guard (they are in-memory and fast); Phase B
/// ([`crate::runtime::prune::plan_multi_estate`], the mark scan) runs on a
/// blocking thread with the guard released, since it is the unbounded part
/// (§10.1's own split); [`crate::runtime::prune::run`] re-acquires the guard
/// to re-validate and commit.
///
/// H1 brief deliverable 1: `state.prune_policy` is no longer applied
/// uniformly to every retained Work — it is the fallback
/// [`crate::runtime::prune::EstatePolicies::from_registry`] resolves for a
/// Work whose estate is unknown or no longer admitted. Each admitted
/// estate's own `[estate] retention` is read fresh every tick (the same
/// "re-read, never cache" discipline `Engine::plan` already has, now applied
/// to this policy map too), from `state.estates` — the registry the
/// estate-scoped HTTP surface already populates.
async fn maybe_run_rotation_triggered_prune(state: &ApiState) {
    let policies =
        crate::runtime::prune::EstatePolicies::from_registry(&state.estates, state.prune_policy);
    let snapshot = {
        let mut core = CoreGuard::acquire(&state.core).await;
        let rotated = core.journal.take_rotation_signal();
        if !rotated && !core.prune_pending {
            return;
        }
        let bounds = match core.journal.segment_bounds() {
            Ok(bounds) => bounds,
            Err(e) => {
                tracing::error!(error = %e, "prune tick: segment_bounds failed");
                return;
            }
        };
        let (candidate, _stall) = crate::runtime::prune::candidate_horizon_multi_estate(
            &bounds,
            core.registry.state(),
            &core.first_seq_by_work,
            &policies,
        );
        let eligible_segments = bounds.iter().filter(|b| b.last_seq <= candidate).count();
        if candidate == 0 || eligible_segments < crate::runtime::prune::PRUNE_BATCH_MIN_SEGMENTS {
            return;
        }
        (
            bounds,
            candidate,
            core.registry.state().clone(),
            core.first_seq_by_work.clone(),
        )
    };
    let (bounds, candidate, registry_snapshot, first_seq_snapshot) = snapshot;

    let data_dir = state.data_dir.clone();
    let planned = tokio::task::spawn_blocking(move || {
        crate::runtime::prune::plan_multi_estate(
            &data_dir,
            &bounds,
            candidate,
            &registry_snapshot,
            &first_seq_snapshot,
            &policies,
        )
    })
    .await;

    match planned {
        Ok(Ok(Some(plan))) => {
            let mut core = CoreGuard::acquire(&state.core).await;
            if let Err(e) = crate::runtime::prune::run(&mut core, &state.data_dir, plan, false) {
                tracing::error!(error = %e, "rotation-triggered prune failed");
                core.prune_pending = true;
            }
        }
        Ok(Ok(None)) => {}
        Ok(Err(e)) => {
            tracing::error!(error = %e, "rotation-triggered prune planning failed");
            let mut core = CoreGuard::acquire(&state.core).await;
            core.prune_pending = true;
        }
        Err(join_err) => {
            tracing::error!(error = %join_err, "rotation-triggered prune planning task panicked");
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
    // estate-root §7.1/§15: a missing-scope refusal carries its remedy as
    // data too — repo count, declared repos, declared groups, and the three
    // example invocations (--repo/--group/--all) — so a client never has to
    // regex the message to build the "select repositories" prompt.
    if let EngineError::MissingScope {
        repo_count,
        repos,
        groups,
    } = e
    {
        body["error"]["repo_count"] = json!(repo_count);
        body["error"]["repos"] = json!(repos);
        body["error"]["groups"] = json!(groups);
        body["error"]["examples"] = missing_scope_examples(repos, groups);
    }
    // §15: an unknown group is refused naming the available ones.
    if let EngineError::UnknownGroup { available, .. } = e {
        body["error"]["available_groups"] = json!(available);
    }
    // estate-root §8.1/§15: a Git admission refusal carries its whole
    // taxonomy as data. `error.code` is already the *first* failed check's
    // stable code (`EngineError::code`), which is what a client branches on;
    // `findings` is every unresolved check across the whole selected scope,
    // each naming its repository, its §8.1 check number, its own code, the
    // evidence, and its remedy — so a multi-repository scope is fixed in one
    // pass rather than one submission per repository.
    //
    // `override_available` is §15's dirty/detached row ("mention the bounded
    // --override-git-preflight escape hatch") answered as a boolean rather
    // than left for a client to infer from the code — and answered `false`
    // for exactly the row that must not offer it: an unresolvable HEAD, where
    // no exact base can be pinned at all.
    if let EngineError::GitPreflight(refusal) = e {
        body["error"]["findings"] = json!(
            refusal
                .findings
                .iter()
                .map(|finding| json!({
                    "repository": finding.repository,
                    "check": finding.check,
                    "check_number": finding.check.number(),
                    "code": finding.code(),
                    "detail": finding.detail,
                    "remedy": finding.remedy(),
                    "waivable": finding.check.waivable(),
                    "porcelain": finding.porcelain,
                }))
                .collect::<Vec<Value>>()
        );
        body["error"]["override_available"] = json!(refusal.override_would_help());
    }
    body
}

/// §7.1's three example invocations for a missing-scope refusal, using real
/// declared names when the estate has any (more useful than a placeholder),
/// falling back to a generic `<repo>`/`<group>` when it does not.
fn missing_scope_examples(repos: &[String], groups: &[String]) -> Value {
    let repo_example = match repos {
        [] => "sgt run \"<intent>\" --repo <repo>".to_string(),
        [one] => format!("sgt run \"<intent>\" --repo {one}"),
        many => format!("sgt run \"<intent>\" --repo {} --repo {}", many[0], many[1]),
    };
    let group_example = match groups.first() {
        Some(g) => format!("sgt run \"<intent>\" --group {g}"),
        None => "sgt run \"<intent>\" --group <group>".to_string(),
    };
    json!({
        "repo": repo_example,
        "group": group_example,
        "all": "sgt run \"<intent>\" --all",
    })
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
///
/// **H1 keeps it a single number, deliberately** (brief deliverable 5): a
/// host daemon owns exactly one journal, shared by every admitted estate,
/// so there is one head and one resume point — the estate coordinate lives
/// on each event (D1), not on the stream. `data_dir` likewise still names
/// one directory; what changed is *which* — the host runtime root (D2),
/// no longer any estate's. Which estates this daemon serves is a different
/// question, and it has its own route: `GET /v1/estates`.
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
///
/// Two arms, in order:
///
/// 1. **In-window** — `registry.commands` has the recorded `CommandOutcome`.
///    The stored `Value` serializes to the same bytes every time, so the
///    duplicate is byte-identical to the original response. Unchanged.
/// 2. **Below the window (W2, Q8)** — the startup cache's ledger has the key
///    but not the body. The command is *refused by name*, never re-executed
///    and never byte-replayed: the cache deliberately carries keys only
///    (full outcome bodies were measured at 250-500 MB and rejected), so the
///    honest answer is "this already happened, here is what it did", not a
///    second execution under the same id. For a submit the refusal names the
///    Work the command created; for anything else it names the outcome
///    class.
fn replay_command(core: &Core, command_id: &str) -> Option<Response> {
    if let Some(outcome) = core.registry.state().commands.get(command_id) {
        let status =
            StatusCode::from_u16(outcome.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        return Some((status, Json(outcome.result.clone())).into_response());
    }
    // W3 §6.3: a command pruned along with its Work's segments — Q8's
    // exemption. Same 409, same `command_below_replay_window` code, same
    // "refused by name, not re-executed" shape as the below-window arm just
    // below: W3 reuses it verbatim rather than inventing a new response
    // shape (§4 of the brief).
    if let Some(row) = core.registry.state().pruned_commands.get(command_id) {
        return Some(below_floor_refusal(&row.as_floor_row()));
    }
    core.floor_ledger.get(command_id).map(below_floor_refusal)
}

/// The §26 refusal for a command whose recorded outcome is below this
/// process's replay window: 409 Conflict, never 410 Gone (the Work is not
/// gone — it is retained and readable by name; only the recorded *response
/// body* is not), never 200 with a synthesized body (that would be a
/// fabricated byte-identical replay, which is exactly what Q8 refused), and
/// never 400 (the request is well-formed).
///
/// Not journaled as a `command.rejected`: `record_and_respond` exists to
/// make an outcome replayable, and journaling one here would append a new
/// `commands` entry for an id whose real outcome is older and different —
/// turning a refusal into a fabricated record. The response is returned
/// directly.
///
/// `pub` (rather than the module-private default every other handler
/// helper here uses) solely so spec §6.3's compensating assertion (c) — "for
/// every such key, `below_floor_refusal` produces a 409 naming the right
/// Work" — can call the real function from `tests/i9_floor_pinning.rs`
/// instead of re-deriving the wire shape there.
pub fn below_floor_refusal(row: &FloorCommandRow) -> Response {
    let message = match (&row.class, row.work_id.as_deref()) {
        (FloorCommandClass::Accepted | FloorCommandClass::Submitted, Some(work_id)) => format!(
            "command_id {} was already applied before this daemon's replay window; \
             it created work {work_id}. It is refused rather than re-executed — \
             re-running it would create a second Work.",
            row.command_id
        ),
        (FloorCommandClass::Rejected, _) => format!(
            "command_id {} was already applied before this daemon's replay window; \
             it was rejected. It is refused rather than re-executed, and the \
             original response body is no longer retained.",
            row.command_id
        ),
        (_, None) => format!(
            "command_id {} was already applied before this daemon's replay window; \
             it was accepted. It is refused rather than re-executed, and the \
             original response body is no longer retained.",
            row.command_id
        ),
    };
    let mut body = error_body("command_below_replay_window", message);
    body["error"]["command_id"] = json!(row.command_id);
    body["error"]["outcome"] = json!(row.class);
    body["error"]["work_id"] = json!(row.work_id);
    (StatusCode::CONFLICT, Json(body)).into_response()
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
    /// D4: the estate this submission addresses — the canonical exact root
    /// (D1/H1-06: no UUID), validated by admission and never by trust.
    ///
    /// §7.4 removed `workspace` because the daemon was bound to exactly one
    /// estate and a wire field would have been a second, conflicting
    /// authority. A host daemon is bound to none, so the field is back — but
    /// as an *address*, not a binding: what it names is re-admitted per
    /// submission ([`crate::runtime::estates::EstateRegistry::admit`]), and
    /// a root that does not validate is refused rather than served.
    ///
    /// Absent keeps the meaning "no repository context offered": the Work is
    /// accepted and stays `pending`, exactly as a submission to an
    /// estate-less daemon always did. That is not a hole in H1 §11.3 — the
    /// client-side gate is what refuses `sgt run` outside an estate, before
    /// a request is ever built — it is the existing, journaled,
    /// non-error meaning of "I have no estate to offer".
    #[serde(default)]
    estate_root: Option<PathBuf>,
    /// estate-root proposal §7/§13.3's structured scope request
    /// (`--repo`/`--group`/`--all`). §7.4: `estate` no longer exists as
    /// a wire field at all — the daemon is bound to exactly one estate, and
    /// this is its replacement.
    #[serde(default)]
    scope: ScopeRequest,
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
    /// `intent`, checked for agreement against `workflow`/`scope.repos`/
    /// `scope.group` above (§13's one-source-of-truth rule) and journaled
    /// verbatim otherwise — nothing here drives routing.
    #[serde(default)]
    intent_detail: Option<IntentDetail>,
    /// MVP-3's submit-time envelope override (`--turns`/`--ceiling-secs`):
    /// see [`EnvelopeRequest`]. Validated at submit (both fields, when
    /// present, must be at least 1) and journaled verbatim otherwise —
    /// nothing here drives routing, exactly like `intent_detail` above.
    #[serde(default)]
    envelope: Option<EnvelopeRequest>,
    /// estate-root §8.3's one bounded override, as `sgt run
    /// --override-git-preflight` sends it.
    ///
    /// **It has exactly one source: this field, on this request.** §8.3: "it
    /// is never available from run defaults or a run template; the operator
    /// must type it for that submission." There is deliberately no daemon
    /// config key, no `[estate]` manifest key, and no profile field that can
    /// set it — `#[serde(default)]` means a submission that does not say so
    /// is not overriding anything, and nothing else in the process can make
    /// this `true`.
    #[serde(default)]
    override_git_preflight: bool,
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
/// names a `workflow`/`repos`/`group` different from what the submission's
/// own flags say is two answers to "what is this Work about" in one Work,
/// and §13 requires exactly one source of truth. Absent on either side is
/// not a disagreement — only *both present and different* is. Repository
/// sets are compared unordered (a client naming the same repos in a
/// different order has not disagreed with itself).
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
        && !req.scope.repos.is_empty()
    {
        let declared_set: std::collections::BTreeSet<&str> =
            declared.iter().map(String::as_str).collect();
        let flag_set: std::collections::BTreeSet<&str> =
            req.scope.repos.iter().map(String::as_str).collect();
        if declared_set != flag_set {
            return Some(format!(
                "intent_detail.repos is {declared:?}, but the submission's scope.repos are \
                 {:?}; the two must agree",
                req.scope.repos
            ));
        }
    }
    // estate-root Phase C: `scope.group` is now a real wire field (§13.3),
    // so `detail.group` gets the same one-source-of-truth check `repos`
    // and `workflow` already get above — the comment this replaces (I3)
    // predates `--group` existing as anything but MVP-3's CLI-side-only
    // expansion, which is exactly what Phase C retires.
    if let (Some(declared), Some(group)) = (detail.group.as_deref(), req.scope.group.as_deref())
        && declared != group
    {
        return Some(format!(
            "intent_detail.group is {declared:?}, but the submission's scope.group is \
             {group:?}; the two must agree"
        ));
    }
    None
}

/// §13's origin metadata: who is asking, and from where.
#[derive(Debug, Clone, Default, Deserialize)]
struct Origin {
    /// Front-end harness name, e.g. `claude`. Drives origin affinity.
    #[serde(default)]
    client: Option<String>,
    /// The client's working directory. Estate discovery (§9) starts here;
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
    // `Estate::resolve` parses the bound estate's `sergeant.toml` and
    // validates every derived mount, `WorkflowDefinition::resolve` reads
    // every stage's `CONTEXT.md` — and probes every harness the run will use
    // (§17.5). All of
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
    // §8.1 checks 9 and 10 are questions about `sergeant/<work-id>` and this
    // Work's own surface directory, so the id has to exist before the record
    // does. Minting it here — side-effect free, and *not* durable until the
    // `work.submitted` append far below — is what lets the whole Git
    // admission contract run "before a Work record exists". Every path that
    // does not reach that append (a preflight refusal, a replayed
    // `command_id`, an empty intent) simply discards it.
    let work_id_candidate = ulid::Ulid::generate().to_string();
    // D4: admit the addressed estate **before** planning. A root that does
    // not validate is this submission's refusal (journaled under its
    // `command_id` like every other semantic rejection, so a retry replays
    // it), never a plan against topology nobody checked — and never a reason
    // to refuse some *other* estate's request.
    let addressed = match req.estate_root.as_deref() {
        None => None,
        Some(root) => match state.estates.admit(root) {
            Ok(estate) => Some(estate.root),
            Err(e) => {
                let result = error_body(e.code(), e.to_string());
                let mut core = CoreGuard::acquire(&state.core).await;
                // §26 first, exactly as the accepting path does it below: a
                // `command_id` whose outcome is already recorded replays
                // that outcome, and one caught in the `work.submitted`/
                // `command.accepted` crash window re-records its real
                // outcome from the replayed state. An estate that broke
                // *after* a submission was accepted must not turn a retry
                // of that submission into a refusal — the Work exists, and
                // §26's promise is about what already happened, not about
                // whether it could happen again now.
                if let Some(resp) = replay_command(&core, &req.command_id) {
                    return resp;
                }
                if let Some(work_id) = core
                    .registry
                    .state()
                    .command_works
                    .get(&req.command_id)
                    .cloned()
                {
                    let replayed = work_view(&core, &state.engine, &work_id);
                    return record_and_respond(
                        &mut core,
                        &req.command_id,
                        "work.submit",
                        Some(&work_id),
                        StatusCode::CREATED,
                        replayed,
                    );
                }
                return record_and_respond(
                    &mut core,
                    &req.command_id,
                    "work.submit",
                    None,
                    StatusCode::UNPROCESSABLE_ENTITY,
                    result,
                );
            }
        },
    };
    let planned = if req.intent.trim().is_empty() {
        None
    } else {
        let engine = state.engine.clone();
        let estate_root = addressed.clone();
        let backend = req.backend.clone();
        let workflow = req.workflow.clone();
        let profile = req.profile.clone();
        let scope = req.scope.clone();
        let origin_cwd = origin.cwd.clone();
        let origin_client = origin.client.clone();
        let candidate = work_id_candidate.clone();
        let override_git_preflight = req.override_git_preflight;
        Some(
            blocking(move || {
                engine.plan(
                    estate_root.as_deref(),
                    &SubmitContext {
                        cwd: origin_cwd.as_deref(),
                        origin_client: origin_client.as_deref(),
                        backend: backend.as_deref(),
                        workflow: workflow.as_deref(),
                        profile: profile.as_deref(),
                        repos: &scope.repos,
                        group: scope.group.as_deref(),
                        all: scope.all,
                        work_id: Some(&candidate),
                        override_git_preflight,
                    },
                )
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

    // The plan decided above, before the guard: estate topology, workflow
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

    // §7.3: journaled twice — `scope_request` is exactly what was submitted;
    // `repositories` is what `plan` (when there was a estate to resolve
    // against at all) actually resolved it to. When there was no estate
    // (`plan` is `None` — the client offered no repository context at all),
    // there is nothing to resolve against, so the raw request is the best
    // honest answer for `repositories` too, same as before this field had a
    // resolution step in front of it.
    let scope_request = req.scope.clone();
    let resolved_repositories = plan
        .as_ref()
        .map(|p| p.repositories.iter().map(|r| r.name.clone()).collect())
        .unwrap_or_else(|| scope_request.repos.clone());

    let work = Work {
        // The id §8.1's checks 9 and 10 were already asked about, above.
        id: work_id_candidate,
        // §7.4: `--workspace`/`SubmitRequest.workspace` no longer exist.
        // `Work.workspace` is kept only so a pre-Phase-C journal event still
        // deserializes (`Work::workspace`'s doc comment, which also says why
        // §13.2's rename deliberately left its name alone); every new Work
        // leaves it `None`.
        workspace: None,
        intent: req.intent,
        repositories: resolved_repositories,
        scope_request,
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
        // §8.3: the authorization the operator gave for this one submission.
        git_preflight_override: req.override_git_preflight,
        state: WorkState::Pending,
        created_by: req.created_by.unwrap_or_else(|| "api".to_string()),
        created_at: rfc3339_utc_now(),
    };
    let work_id = work.id.clone();
    let mut draft = EventDraft::new(api_source(), KIND_WORK_SUBMITTED, json!({"work": work}))
        .with_work_id(&work_id);
    // H1 touch point #4: `work.submitted` is the other emission point
    // outside `Engine::commit` (`begin_start`'s later events go through
    // that chokepoint and pick it up there). `plan` carries the estate this
    // submission actually resolved against — `None` (no repository
    // context offered) leaves the field absent, same as a Work that never
    // reaches an estate at all.
    if let Some(plan) = &plan {
        draft = draft.with_workspace_id(plan.estate.root.to_string_lossy().into_owned());
    }
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

/// #4's Work cache read side, the full-[`Work`] analog of [`resolve_run`]
/// just below: `works` (active) -> `terminal_works` (cache) -> journal
/// re-derivation, in that order. Every site that used to read
/// `registry.works.get`/`.contains_key` directly for a Work it needed the
/// full struct from — `work_view`, `cancel`/`extend`/`reap`-shaped
/// handlers — routes through this instead, so a Work aged out of the
/// bounded cache is still found, exactly the way `resolve_run` already
/// keeps working for its run.
///
/// `None` only for a work id `WorkRegistry::work_index` has no row for —
/// genuinely unknown, never journaled. The index is checked before ever
/// paying for a replay: every Work this daemon has journaled has a row
/// there for as long as the journal exists, evicted from the full-struct
/// cache or not, so a miss there is conclusive without reading a single
/// event.
fn resolve_work(core: &Core, work_id: &str) -> Option<Work> {
    let registry = core.registry.state();
    if let Some(work) = registry.works.get(work_id) {
        return Some(work.clone());
    }
    if let Some(work) = registry.terminal_works.get(work_id) {
        return Some(work.clone());
    }
    // W3 §11.2: a pruned id short-circuits to `None` here rather than
    // paying a replay that is guaranteed to find nothing — its events are
    // gone. `pruned_works` and `work_index` are disjoint by construction
    // (§6.2), so this check is conclusive without touching the journal.
    if registry.pruned_works.contains_key(work_id) {
        return None;
    }
    registry.work_index.get(work_id)?;
    match blocking_sync(|| rederive_work(&core.journal, work_id)) {
        Ok(work) => work,
        Err(e) => {
            tracing::error!(
                work_id = %work_id,
                error = %e,
                "could not re-derive an evicted work from the journal"
            );
            None
        }
    }
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
/// (`ADR 0007`). The engine still learns
/// nothing about what a commit *is* (the workspace knowledge library's
/// North Star ruling: "the engine learns no
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
    // The structural predicate itself lives on `TeardownReport` (surface.rs,
    // beside `integrity()`) so the projection reducer can compute the same
    // thing at `surface.torn_down` time, with no live `Work` at hand, for
    // `WorkIndexRow`'s effective disposition (#4's slim-row eviction gap).
    // Only the `work.state == Completed` gate stays here — that is the one
    // fact the reducer already knows a different way (it runs after
    // `work.completed` has already updated `work.state`/`row.state`).
    teardown.stranded_completion(surface)
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
    // #4: `works` may have already evicted this id into `terminal_works` or
    // beyond — `resolve_work` is the same works -> cache -> journal chain
    // `resolve_run` below already uses for the run.
    let work = resolve_work(core, work_id);
    let cached_run = registry.run_view(work_id);
    let run = work.as_ref().and_then(|w| resolve_run(core, w, cached_run));
    // ADR 0007(b): the persisted `Work` serializes with its true `state`
    // (`Completed`) intact; only this view's own `state` key is overridden,
    // so a work still in flight and every non-stranded completion look
    // exactly as before.
    let work_json = work.as_ref().map(|w| {
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
        "output": work.as_ref().and_then(|w| run.as_ref().and_then(|r| output_pointer(w, r))),
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
        "envelope": work.as_ref().map(|w| {
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
///
/// #4: iterates `registry.work_index` — the always-retained key set — not
/// `registry.works`, which only holds active Works now. For each id this
/// renders the same full row as before when the Work is still active or
/// still in the bounded `terminal_works` cache; a Work aged out of that
/// cache renders a narrowed row built from its [`WorkIndexRow`] alone,
/// with `"evicted": true` naming the shape — see
/// [`evicted_fleet_row`]'s own doc for exactly what that narrows.
fn fleet_body(core: &Core, engine: &Engine) -> Value {
    let registry = core.registry.state();
    let works: Vec<Value> = registry
        .work_index
        .keys()
        .map(|id| {
            let Some(work) = registry
                .works
                .get(id)
                .or_else(|| registry.terminal_works.get(id))
            else {
                return evicted_fleet_row(&registry.work_index[id]);
            };
            // `registry.run_view` alone only reaches the bounded
            // in-memory cache (`TERMINAL_RUN_CACHE_CAPACITY`); once a
            // terminal run ages out of it, `resolve_run`'s journal-
            // replay fallback is what `work_view` already relies on to
            // keep `sgt work show` correct for the identical work. This
            // row must use the same fallback, or an evicted work's
            // `state` here would silently fall back to plain
            // `completed` (ADR 0007(b)) while `work show` still says
            // `completed_dirty` for it.
            let run = resolve_run(core, work, registry.run_view(id));
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

/// The `sgt work list` row for a Work that has aged out of `terminal_works`
/// entirely — #4's bounded-cost tradeoff made visible. Built from
/// [`WorkIndexRow`] alone, so it carries only what that row keeps: `id`,
/// `intent`, `state` (with the same `completed_dirty` compaction
/// `reported_state` applies, when the retained disposition alone is enough
/// to tell), `integrity` (disposition only — the findings and drift
/// `integrity_view` composes from a live [`TeardownReport`] are not part of
/// the slim row), and the two timestamps. No `stage`/`resolved_backend`/
/// `envelope`/`output` — those need the run, which a Work this old no
/// longer has cached, and re-deriving it here would be the very per-row
/// journal replay this cache exists to avoid. `"evicted": true` names the
/// narrowing explicitly rather than leaving a client to infer it from
/// absent keys.
///
/// No gap versus the full row's `reported_state` for the `completed_dirty`
/// question, despite the slim row never retaining `run.teardown`/
/// `run.surface`: `WorkIndexRow::integrity` (projection.rs) is the *effective*
/// disposition — explicit `Dirty` OR [`stranded_completion`]'s structural
/// check — already folded in at `surface.torn_down` time, while both were
/// still in hand. So the plain `== Some(Dirty)` check below is enough; it
/// does not need to re-derive stranded-ness from fields this row never kept.
fn evicted_fleet_row(row: &WorkIndexRow) -> Value {
    let state = if row.state == WorkState::Completed
        && row.integrity == Some(IntegrityDisposition::Dirty)
    {
        "completed_dirty"
    } else {
        row.state.as_str()
    };
    json!({
        "id": row.id,
        "intent": row.intent,
        "state": state,
        "integrity": row.integrity.map(|d| json!({"disposition": d})),
        "created_at": row.created_at,
        "updated_at": row.updated_at,
        "evicted": true,
    })
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
    /// The client's working directory — evidence only since §5.2 removed
    /// its authority over topology; still validated so an obviously-broken
    /// client hears about it.
    cwd: String,
    /// D4: the estate whose local catalog half this request addresses.
    #[serde(default)]
    estate_root: Option<PathBuf>,
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

/// The catalog `GET /v1/workflows` answers with (§11.2, Decisions
/// T2-39/T2-40): the **bound estate's** own root-indexed published workflows
/// when it has any, else the embedded `software-change` fallback, else (only
/// when the embedded fallback itself fails to load) an empty list — fails
/// closed per §19.4, never a `4xx` for this shape.
///
/// §5.2, estate-root Phase D: this used to discover a estate from the
/// client's `cwd`, which made "what could I bind" a question about wherever
/// the caller happened to be standing. It is now a question about the one
/// estate this daemon is bound to, the same estate `submit_work`'s plan
/// resolves against — a client cannot be shown a catalog it could not
/// actually submit into.
fn workflow_catalog_entries(estate_root: Option<&std::path::Path>) -> Vec<Value> {
    let repo_entries = match estate_root {
        Some(root) => workflow::catalog(root),
        None => Vec::new(),
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
/// the same estate discovery [`submit_work`]'s plan does, the same
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
    // `cwd` stays in the query grammar (a client that has one still sends
    // it) but is evidence only now — §5.2 removed its authority over
    // topology. It is still validated so an obviously-broken client hears
    // about it rather than silently getting the estate's catalog.
    let cwd = PathBuf::from(&req.cwd);
    if !cwd.is_absolute() {
        return error_response(
            StatusCode::BAD_REQUEST,
            "invalid_cwd",
            "cwd must be an absolute path",
        );
    }
    // D4: the catalog's estate-local half is the *addressed* estate's, and
    // only once admitted. Unaddressed (or unadmitted) answers the embedded
    // catalog alone — an honest "no estate in play here", never another
    // estate's forks.
    let estate_root = req
        .estate_root
        .as_deref()
        .and_then(|root| state.estates.admit(root).ok())
        .map(|estate| estate.root);
    let workflows = blocking(move || workflow_catalog_entries(estate_root.as_deref())).await;
    Json(json!({"workflows": workflows})).into_response()
}

/// `GET /v1/work/{id}` (§16.3) — one work record, or (W4, Q10) the named
/// pruned answer, or 404 for an id this estate has never journaled at all.
/// The three cases are mutually exclusive by construction (`pruned_works`
/// and `work_index` are disjoint, W3 §6.2) and this handler is the only
/// place that has to know all three exist.
async fn show_work(State(state): State<ApiState>, Path(id): Path<String>) -> Response {
    let core = CoreGuard::acquire(&state.core).await;
    // #4: existence is answered from the always-retained slim index, not
    // the (now bounded) `works` map — an evicted-beyond-cache Work still
    // has a row there, and `work_view`'s own `resolve_work` re-derives it.
    let registry = core.registry.state();
    if registry.work_index.contains_key(&id) {
        return Json(work_view(&core, &state.engine, &id)).into_response();
    }
    if let Some(row) = registry.pruned_works.get(&id) {
        return Json(pruned_work_view(row, &state.prune_policy)).into_response();
    }
    error_response(
        StatusCode::NOT_FOUND,
        "work_not_found",
        format!("no work with id {id}"),
    )
}

/// The named answer for a pruned Work (Q10: "pruned on `<date>` under
/// policy", never a blank 404). `work: null` keeps the top-level shape a
/// client already parsing `work_view`'s `{"work": {...}, "stage": ..., ...}`
/// envelope from breaking outright; `state: "pruned"` is the discriminator
/// no real `WorkState` variant can ever produce (`WorkState`'s own variants
/// are `pending`/`active`/`needs_input`/`blocked`/`waiting`/`completed`/
/// `failed`/`canceled` — never `pruned`), so a client can `match` on it
/// unambiguously rather than infer "gone" from `work == null` alone (which
/// would still be indistinguishable from never-existed).
///
/// `policy` is this estate's **current** declared retention — not
/// necessarily the exact policy in force at the instant this Work was
/// pruned, if an operator has since edited `[estate] retention`. The
/// historically-exact record of what actually authorized this deletion is
/// the `prune.intent`/`prune.completed` pair at the seq the residue came
/// from, discoverable via `GET /v1/events` if ever needed forensically —
/// `PrunedWorkRow` does not carry a per-row policy snapshot (§2.3 of
/// `w3-spec.md` did not put one there), and inventing one here would assert
/// state the journal does not actually hold.
fn pruned_work_view(
    row: &crate::runtime::prune::PrunedWorkRow,
    policy: &crate::runtime::prune::PrunePolicy,
) -> Value {
    json!({
        "work": null,
        "state": "pruned",
        "id": row.id,
        "intent": row.intent,
        "last_known_state": row.state,
        "created_at": row.created_at,
        "updated_at": row.updated_at,
        "last_seq": row.last_seq,
        "pruned_at": row.pruned_at,
        "policy": {"retention": policy.retention, "source": policy.source},
        // H1 brief deliverable 3: the report names estates. `policy` above
        // stays the daemon-wide fallback the caller passed in (unchanged
        // shape); `estate_root` is this specific residue row's own recorded
        // coordinate, `None` for a Work pruned before H1 or submitted with
        // no estate context at all.
        "estate_root": row.estate_root,
    })
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
/// §22.6 tradeoff, narrowed by W4, not eliminated: `events_after(from)`
/// below still runs while `core` — the exclusive `CoreGuard` — is held, so
/// every call here still queues every other Core-guarded request for the
/// read's duration, exactly as `events_after`'s own doc comment says every
/// caller must expect. What changed is the *lower bound*: W3's
/// `first_seq_by_work` (§2.5) already tracks exactly where each retained
/// Work's own history begins, so this reads from there instead of from the
/// floor — free on any Work whose first event is not itself near the
/// floor, and no worse than before on one that is. `resolve_run`'s
/// `terminal_runs` cache accepts the identical guard-held-replay shape only
/// as a rare, capacity-bounded cache-miss fallback; there is still no
/// equivalent bound *within* one Work's own transcript, because a Work's
/// conversation history is unbounded and not capped by any terminal-state
/// cache. Closing this for good needs a journal reader the core does not
/// own — the same named follow-up `resolve_run` already defers, not this
/// build's.
async fn work_transcript(State(state): State<ApiState>, Path(id): Path<String>) -> Response {
    let core = CoreGuard::acquire(&state.core).await;
    let registry = core.registry.state();
    // W4 §1.2's pruned answer, restated here: a pruned Work's transcript is
    // exactly its slim row and prune date, same as `work show` — not a 404
    // and not an attempted replay of events that no longer exist (§3 of
    // `w3-spec.md`'s Approved Tradeoffs already names this as the intended
    // shape; this wave is what actually renders it).
    if let Some(row) = registry.pruned_works.get(&id) {
        let mut body = pruned_work_view(row, &state.prune_policy);
        body["work_id"] = json!(id);
        body["turns"] = json!([]);
        return Json(body).into_response();
    }
    if !registry.work_index.contains_key(&id) {
        return error_response(
            StatusCode::NOT_FOUND,
            "work_not_found",
            format!("no work with id {id}"),
        );
    }
    // W4 §1.3: bounded to this Work's own segment range, not the floor. On
    // a Work created long after this journal's floor, this skips whole
    // segments below it (`Replay::after`'s existing segment-skip
    // optimization, unchanged) instead of walking every retained event of
    // every other Work first. On a Work as old as the floor itself, this is
    // exactly today's cost — never worse, sometimes much better.
    let from = core
        .first_seq_by_work
        .get(&id)
        .map(|seq| seq.saturating_sub(1))
        .unwrap_or(0);
    let events = match blocking_sync(|| core.events_after(from)) {
        Ok(events) => events,
        Err(e) => return internal_error(e),
    };
    drop(core);

    let turns = transcript_turns(&id, events, &state.data_dir);
    Json(json!({"work_id": id, "turns": turns})).into_response()
}

/// The pure decode: filter `events` to `work_id`'s `conversation.*` and
/// `tool.*` (#240) kinds and turn each into a `{seq, ts, role, text,
/// source}` entry, in the journal's own causal (seq) order — factored out
/// of the handler above so the role/source mapping and the blob-decode
/// fallback can be pinned by a direct test without spinning up a daemon.
/// `tool.*` events carry `role: "tool_use"` plus a `phase`
/// (`"requested"`/`"completed"`) — previously these fell into the
/// catch-all below and vanished, so a degraded run that silently invoked
/// zero tools read identically to one that made real progress.
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
    // lands between two of a turn's own assistant-line appends (CONTRIBUTING.md's
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
            // #240: `tool.*` events were previously falling into the
            // catch-all below and vanishing from the transcript entirely —
            // a degraded run that silently skipped every tool call read
            // identically to one that made real progress. Surfaced as
            // `role: "tool_use"` with a `phase` distinguishing the request
            // from its result, so a reader (or `render_transcript`) can
            // tell the two apart without a second lookup.
            KIND_TOOL_REQUESTED => {
                let name = event.payload["name"].as_str().unwrap_or("tool");
                let input = &event.payload["input"];
                turns.push(json!({
                    "seq": event.seq,
                    "ts": event.timestamp,
                    "role": "tool_use",
                    "phase": "requested",
                    "tool_use_id": event.payload["id"].as_str().unwrap_or(""),
                    "name": name,
                    "input": input,
                    "text": format!("{name} {input}"),
                    "source": "event",
                }));
            }
            KIND_TOOL_COMPLETED => {
                let is_error = event.payload["is_error"].as_bool().unwrap_or(false);
                let name = event.payload["name"].as_str().unwrap_or("tool");
                let outcome = if is_error { "error" } else { "ok" };
                turns.push(json!({
                    "seq": event.seq,
                    "ts": event.timestamp,
                    "role": "tool_use",
                    "phase": "completed",
                    "tool_use_id": event.payload["tool_use_id"].as_str().unwrap_or(""),
                    "name": name,
                    "is_error": is_error,
                    "text": format!("{name} -> {outcome}"),
                    "source": "event",
                }));
            }
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
    // #4: a duplicate cancel (fresh `command_id`, same target) against a
    // Work canceled earlier — possibly already evicted, since `Canceled` is
    // absorbing — must still find it to answer the idempotent-success arm
    // below, not 404. `resolve_work` is the same works -> cache -> journal
    // chain `work_view` uses.
    let Some(work) = resolve_work(&core, &id) else {
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
    if !core.registry.state().work_index.contains_key(&id) {
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
    if !core.registry.state().work_index.contains_key(&id) {
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
    if !core.registry.state().work_index.contains_key(&id) {
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
    // #4: reap targets a terminal Work's teardown, so this id is exactly
    // the shape likely to have aged out of the live map already.
    let Some(work) = resolve_work(&core, &id) else {
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
///
/// #4 doubles that accepted tradeoff rather than closing it: this walks
/// every id in `registry.work_index` (every Work ever journaled, active or
/// not — `registry.works` alone would silently stop finding any *terminal*
/// Work the instant it aged out of the live map, and terminal is exactly
/// what a teardown requires) and now resolves the Work itself through
/// `resolve_work`'s cache-then-journal chain, on top of `resolve_run`'s own.
/// Worst case — an estate with many more historical Works than either
/// bounded cache holds — pays up to two replays per id instead of one.
/// Still bounded by the same "occasional, operator-driven, not a hot path"
/// argument above, not a new category of cost; called out here rather than
/// left implicit for whoever tunes the cache capacities next.
async fn list_retained(State(state): State<ApiState>) -> Response {
    let core = CoreGuard::acquire(&state.core).await;
    let registry = core.registry.state();
    let entries: Vec<Value> = registry
        .work_index
        .keys()
        .filter_map(|id| {
            let work = resolve_work(&core, id)?;
            let run = resolve_run(&core, &work, registry.run_view(id))?;
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

// ------------------------------------------------------------------- sweep

/// The mount-validated topology a sweep reads and (only ever) deletes in.
///
/// `Estate::resolve`, not `declared_repos`: a verb that may run `git branch
/// -D` inside a directory must first have proof that directory is this
/// estate's own ordinary checkout — §6.1's derived mount, no symlink, no
/// linked worktree, no other estate's clone. A declared-but-unresolvable
/// repository fails the whole pass by name rather than being swept past.
fn sweep_topology(
    state: &ApiState,
    requested: Option<&std::path::Path>,
) -> Result<Estate, (StatusCode, Value)> {
    let root = estate_root_or_error(state, requested, "sweeping")?;
    Estate::resolve(&root).map_err(|e| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            error_body("invalid_estate", e.to_string()),
        )
    })
}

/// The journaled Work identity a sweep cross-references `sergeant/*` refs
/// against: every Work this daemon has ever journaled, with its current
/// state.
///
/// `work_index` is exactly that set by construction (#4: a slim row per
/// `work.submitted`, updated in place, never evicted), so this reads it
/// directly instead of replaying the journal for a fact the registry already
/// holds — which also means an *absent* id is real evidence of an
/// unjournaled ref (#172's orphan), not an eviction artifact.
fn journaled_work_states(core: &Core) -> std::collections::BTreeMap<String, WorkState> {
    core.registry
        .state()
        .work_index
        .iter()
        .map(|(id, row)| (id.clone(), row.state))
        .collect()
}

/// `GET /v1/sweep` — §12.3's deliberate sweep, classification half (#159):
/// every `sergeant/*` ref in every mount, classified `active`/`redundant`/
/// `retained`/`orphan`, plus each mount's prunable worktree registrations.
///
/// Read-only in the strongest sense — it mutates neither the estate nor the
/// journal — which is why it is a plain `GET` with no `command_id`, exactly
/// like `GET /v1/retained`, and why `sgt work sweep` connects to an existing
/// daemon rather than spawning one.
///
/// The registry read and the git walk are deliberately not held under one
/// lock: the Work states are snapshotted under the guard, the guard is
/// dropped, and the per-mount `for-each-ref`/`merge-base` walk — the
/// expensive half, and the whole reason §12.3 keeps this out of `doctor` —
/// runs with every other request free to proceed. A Work that changes state
/// mid-walk is therefore classified against the snapshot; that is safe
/// because nothing here acts on the answer, and the deletion half re-derives
/// its own classification from scratch anyway.
async fn sweep_estate(State(state): State<ApiState>, Query(query): Query<EstateQuery>) -> Response {
    let estate = match sweep_topology(&state, query.estate_root.as_deref()) {
        Ok(estate) => estate,
        Err((status, body)) => return (status, Json(body)).into_response(),
    };
    let works = {
        let core = CoreGuard::acquire(&state.core).await;
        journaled_work_states(&core)
    };
    let report = blocking(move || sweep::classify(&estate, &works)).await;
    Json(report).into_response()
}

#[derive(Debug, Deserialize)]
struct SweepRequest {
    command_id: String,
    /// D4: the estate whose `sergeant/*` refs this sweep addresses.
    #[serde(default)]
    estate_root: Option<PathBuf>,
    /// Must be `true`, or the request is refused — sweep deletes branches
    /// and git's reflog is the only undo.
    #[serde(default)]
    confirm: bool,
    /// The branches the client believes are redundant. A request, never an
    /// authorization: the server re-classifies every one of them.
    #[serde(default)]
    branches: Vec<SweepTarget>,
}

/// `POST /v1/sweep` — §12.3's deliberate sweep, deletion half (#159).
///
/// Refuses without `{"confirm": true}` for the same reason `reap` does:
/// there is no journaled state a resubmission could undo, so a bare body is
/// ambiguity, not an implicit yes (`AGENTS.md`'s fail-closed rule). The
/// operator reviews `GET /v1/sweep` first — that is the read half of this
/// same pair — then re-sends with the branches and confirmation.
///
/// **The client's `branches` list is a request, not a grant.** Classification
/// is re-derived here, at deletion time, and only a branch this daemon still
/// calls `redundant` — tip already contained in its mount's default branch —
/// is deleted. An `active`, `retained` or `orphan` branch, a ref outside
/// `sergeant/*`, and a branch that no longer exists are all refused with the
/// reason named, having mutated nothing.
///
/// §22.6 tradeoff, the same one `reap` discloses: the git calls run while the
/// core guard is held, queueing every other guarded request for their
/// duration. A sweep is rare, explicit and never on a hot path, and holding
/// the guard is what keeps the mutation and its `command.accepted` record —
/// which carries the deleted branches and their tip SHAs, so the journal
/// records what was destroyed and where it can be restored from — one
/// indivisible step.
async fn sweep_delete(
    State(state): State<ApiState>,
    body: Result<Json<SweepRequest>, JsonRejection>,
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
            "sweep deletes branches; review GET /v1/sweep, then resend with \"confirm\": true",
        );
        return record_and_respond(
            &mut core,
            &req.command_id,
            "work.sweep",
            None,
            StatusCode::BAD_REQUEST,
            result,
        );
    }
    let estate = match sweep_topology(&state, req.estate_root.as_deref()) {
        Ok(estate) => estate,
        Err((status, body)) => {
            return record_and_respond(
                &mut core,
                &req.command_id,
                "work.sweep",
                None,
                status,
                body,
            );
        }
    };
    let works = journaled_work_states(&core);
    let deleted =
        blocking_sync(|| sweep::delete_redundant(&state.data_dir, &estate, &works, &req.branches));
    let result = json!({"deleted": deleted});
    record_and_respond(
        &mut core,
        &req.command_id,
        "work.sweep",
        None,
        StatusCode::OK,
        result,
    )
}

// ------------------------------------------------------------------ estate
//
// §16.2/§16.3: thin daemon-side wrappers over `crate::domain::manifest`'s
// existing add_repo/remove_repo/add_group/remove_group and
// `crate::cli::doctor`'s existing Report — no logic is duplicated here, and
// none of it touches the journal or `Core`: manifest edits are not engine
// state (R-NS-4, `src/domain/manifest.rs`'s own module doc), so unlike every
// other mutation in this file there is no `command_id`/replay pair.

/// D4: the estate root these routes operate on comes from the **request**,
/// and is served only after this daemon's registry admits it.
///
/// It used to come from `state.engine.estate_root` — the one estate the
/// process was bound to. A host daemon has none, so every estate-scoped
/// route now carries an explicit `estate_root` (a canonical exact root per
/// D1/H1-06: no UUID), and the daemon validates it *by admission*, never by
/// trust. The refusal taxonomy is
/// [`crate::runtime::estates::EstateAdmissionError`]'s, unchanged between
/// this and the client-side gate.
///
/// The process's own working directory is still deliberately **not**
/// consulted, for exactly the reason it never was: a long-running daemon's
/// cwd is not reliably anything, and a cwd that happens to sit inside a
/// *different* estate would silently resolve to the wrong manifest rather
/// than failing closed — the one outcome R-NS-4's discipline exists to
/// prevent. H1 removes the daemon's binding; it does not give the daemon a
/// new way to guess.
fn resolve_estate_root(
    state: &ApiState,
    requested: Option<&std::path::Path>,
    operation: &str,
) -> Result<PathBuf, Box<Response>> {
    estate_root_or_error(state, requested, operation)
        .map_err(|(status, body)| Box::new((status, Json(body)).into_response()))
}

/// [`resolve_estate_root`]'s answer as a `(status, body)` pair rather than a
/// finished response — the shape a *command* handler needs, because its
/// refusal has to be journaled under a `command_id` before it is answered
/// with, and `record_and_respond` takes the body.
fn estate_root_or_error(
    state: &ApiState,
    requested: Option<&std::path::Path>,
    operation: &str,
) -> Result<PathBuf, (StatusCode, Value)> {
    admit_addressed_estate(state, requested, operation).map(|estate| estate.root)
}

/// Admit the estate a request addressed, or turn the refusal into this
/// file's one `{"error": {"code", "message"}}` shape.
///
/// `NOT_FOUND` for "no estate addressed" (the operation has no object) and
/// `UNPROCESSABLE_ENTITY` for a root that will not admit (the request named
/// something, and it is wrong) — the same split every other refusal in this
/// file already draws between an absent thing and an invalid one.
fn admit_addressed_estate(
    state: &ApiState,
    requested: Option<&std::path::Path>,
    operation: &str,
) -> Result<crate::runtime::estates::AdmittedEstate, (StatusCode, Value)> {
    let Some(requested) = requested else {
        return Err((
            StatusCode::NOT_FOUND,
            error_body(
                "no_estate",
                format!(
                    "{operation} is estate-scoped, but this request addressed no estate — send \
                     `estate_root` (the exact estate root, canonical)"
                ),
            ),
        ));
    };
    state.estates.admit(requested).map_err(|e| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            error_body(e.code(), e.to_string()),
        )
    })
}

/// `GET /v1/estates` (H1 §4, brief deliverable 6) — the admitted-estate
/// registry as this process holds it right now.
///
/// Observational, and says so: a row exists because some request addressed
/// that root and admission succeeded, never because the daemon went looking.
/// `available` is the last re-validation's verdict, so an estate whose
/// manifest has since been deleted is still *listed* — its Work is still in
/// the journal — and reported unavailable with the reason, rather than
/// vanishing as though it had never been served.
async fn list_estates(State(state): State<ApiState>) -> Response {
    let estates: Vec<Value> = state
        .estates
        .entries()
        .into_iter()
        .map(|entry| {
            json!({
                "root": entry.estate.root,
                "name": entry.estate.name,
                "manifest_path": entry.estate.manifest_path,
                "admitted_at": entry.estate.admitted_at,
                "available": entry.available,
                "unavailable_reason": entry.unavailable_reason,
                "last_touched_at": entry.last_touched_at,
                "retention": entry.estate.retention,
                "surfaces_dir": entry.estate.surfaces_dir,
            })
        })
        .collect();
    Json(json!({"estates": estates})).into_response()
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
        ManifestError::UpstreamRemoteFailed { .. } => "upstream_remote_failed",
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
        // Not 502: `remote add`/`set-url` are local config writes, so a
        // failure here is about this checkout, never about a network.
        ManifestError::UpstreamRemoteFailed { .. } => StatusCode::UNPROCESSABLE_ENTITY,
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

/// D4: how an estate-addressed **query** names its estate.
///
/// `GET`/`DELETE` routes have no body to carry `estate_root` in, so it rides
/// the query string — the canonical exact root, percent-encoded by the
/// client. Absent is refusal (c): the operation has no object, and the
/// daemon does not pick one.
#[derive(Debug, Deserialize, Default)]
struct EstateQuery {
    #[serde(default)]
    estate_root: Option<PathBuf>,
}

/// The declared repositories, read the same way `sgt repo list --json` does.
fn workspace_read(estate_root: &std::path::Path) -> Result<Estate, Box<Response>> {
    Estate::from_config_allow_empty(&estate_root.join(crate::domain::estate::MANIFEST_FILE))
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
async fn estate_list_repos(
    State(state): State<ApiState>,
    Query(query): Query<EstateQuery>,
) -> Response {
    let estate_root =
        match resolve_estate_root(&state, query.estate_root.as_deref(), "listing repositories") {
            Ok(root) => root,
            Err(resp) => return *resp,
        };
    let estate = match workspace_read(&estate_root) {
        Ok(w) => w,
        Err(resp) => return *resp,
    };
    let repos: Vec<Value> = estate
        .repositories
        .iter()
        .map(|r| {
            json!({
                "name": r.name,
                "path": r.path,
                "origin": estate.repository_origin(&r.name),
                "instructions": estate.instruction_policy(&r.name).as_str(),
            })
        })
        .collect();
    Json(json!({"repos": repos})).into_response()
}

#[derive(Debug, Deserialize)]
struct AddRepoRequest {
    /// D4: the estate this edit addresses (canonical exact root).
    #[serde(default)]
    estate_root: Option<PathBuf>,
    name: String,
    #[serde(default)]
    origin: Option<String>,
    /// #112's forge-neutral upstream declaration, forwarded verbatim — this
    /// route stays the thin wrapper §16.2 asks for, so what `sgt repo add
    /// --upstream` can declare, a direct API caller can declare too.
    #[serde(default)]
    upstream: Option<String>,
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
    let estate_root =
        match resolve_estate_root(&state, req.estate_root.as_deref(), "adding a repository") {
            Ok(root) => root,
            Err(resp) => return *resp,
        };
    let name = req.name.clone();
    let origin = req.origin.clone();
    let upstream = req.upstream.clone();
    let result = blocking_sync(|| {
        manifest::add_repo(
            &estate_root,
            &name,
            origin.as_deref(),
            upstream.as_deref(),
            instructions,
        )
    });
    match result {
        Ok(()) => (
            StatusCode::CREATED,
            Json(json!({
                "name": req.name,
                "path": format!("repos/{}", req.name),
                "origin": req.origin,
                "upstream": req.upstream,
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
async fn estate_remove_repo(
    State(state): State<ApiState>,
    Path(name): Path<String>,
    Query(query): Query<EstateQuery>,
) -> Response {
    let estate_root = match resolve_estate_root(
        &state,
        query.estate_root.as_deref(),
        "removing a repository",
    ) {
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
async fn estate_list_groups(
    State(state): State<ApiState>,
    Query(query): Query<EstateQuery>,
) -> Response {
    let estate_root =
        match resolve_estate_root(&state, query.estate_root.as_deref(), "listing groups") {
            Ok(root) => root,
            Err(resp) => return *resp,
        };
    let estate = match workspace_read(&estate_root) {
        Ok(w) => w,
        Err(resp) => return *resp,
    };
    let groups: Vec<Value> = estate
        .groups
        .iter()
        .map(|(name, g)| json!({"name": name, "repos": g.repos, "brief": g.brief}))
        .collect();
    Json(json!({"groups": groups})).into_response()
}

#[derive(Debug, Deserialize)]
struct AddGroupRequest {
    /// D4: the estate this edit addresses (canonical exact root).
    #[serde(default)]
    estate_root: Option<PathBuf>,
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
    let estate_root =
        match resolve_estate_root(&state, req.estate_root.as_deref(), "adding a group") {
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
    Query(query): Query<EstateQuery>,
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
    let estate_root =
        match resolve_estate_root(&state, query.estate_root.as_deref(), "removing a group") {
            Ok(root) => root,
            Err(resp) => return *resp,
        };
    match blocking_sync(|| manifest::remove_group(&estate_root, &name, &repos)) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => manifest_error_response(&e),
    }
}

/// The directory `GET /v1/doctor` reports the estate rows against: the
/// estate **this request addressed** (D4), never the daemon process's cwd —
/// a long-running daemon's cwd is not reliably anything, and a host daemon
/// no longer has a bound root to fall back on either.
///
/// An unaddressed or unadmitted estate falls back to the host runtime root,
/// where the estate-root row correctly fails — which is exactly what
/// `sgt doctor` asking a host daemon about no estate in particular should
/// report, and is the same answer a daemon started outside an estate has
/// always given.
fn doctor_root(state: &ApiState, requested: Option<&std::path::Path>) -> PathBuf {
    requested
        .and_then(|root| state.estates.admit(root).ok())
        .map(|estate| estate.root)
        .unwrap_or_else(|| state.data_dir.clone())
}

/// `GET /v1/doctor` (§16.3) — the same `doctor::Report::to_json()`
/// `sgt doctor --json` already prints, computed exactly once here.
///
/// `xdg_outranked` is `false` here, not omitted: #80's provenance is a fact
/// about *this process's* `cli::resolve_data_dir` call, computed by whatever
/// CLI invocation resolved `state.data_dir` before spawning or attaching to
/// this daemon — that resolution, and its winning rung, do not survive
/// across the process boundary to this handler.
async fn doctor_report(
    State(state): State<ApiState>,
    Query(query): Query<EstateQuery>,
) -> Response {
    let root = doctor_root(&state, query.estate_root.as_deref());
    let report = doctor::run(&state.data_dir, &root, false).await;
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
// `Response` is the error currency of every handler on this router; boxing it
// here (clippy 1.98's `result_large_err`) would cost an unbox at each `?` site
// for a helper called on read paths only.
#[allow(clippy::result_large_err)]
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
        let registry = core.registry.state();
        // #221: graph is a named read surface too. Its source events have
        // gone after pruning, but the retained residue gives the same honest
        // named answer as `work show` and `work transcript`.
        if let Some(row) = registry.pruned_works.get(&id) {
            return Json(pruned_work_view(row, &state.prune_policy)).into_response();
        }
        if !registry.work_index.contains_key(&id) {
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
    /// D4/D6: restrict to one estate's events, by canonical exact root,
    /// matched against each event's own `workspace_id` coordinate (D1).
    ///
    /// Optional, and **server-side on purpose**: without it, "estate-wide
    /// watch" would silently become "host-wide watch" for every existing
    /// `sgt watch` invocation the moment a daemon starts serving a second
    /// estate — a behavior change nobody asked for. The filter is not
    /// validated by admission: this is a read, it creates nothing, and a
    /// root that matches no event is an empty answer rather than a refusal.
    ///
    /// This wave lands the wire field and the filter. Routing the *client*
    /// onto it — what `sgt watch` inside an estate defaults to, and how a
    /// host-wide watch is asked for — is W3's, per this wave's brief.
    #[serde(default)]
    estate_root: Option<String>,
}

/// The `GET /v1/events` body for an already-fetched slice.
///
/// `floor_seq` (Q10, W4): the oldest seq this journal can still answer for.
/// Always present, never inferred by the client — `1` on a journal this
/// build has never pruned, the oldest surviving segment's `first_seq`
/// otherwise. A `from=` below this number is not an error (`Replay::after`'s
/// A1 clamp already serves from the floor, W3 §11.1); this field is what
/// lets a client tell "you are caught up" apart from "some of what you
/// asked for is gone under retention policy" — without it those two cases
/// are wire-identical.
fn events_body(events: Vec<Event>, query: &EventsQuery, floor_seq: u64) -> Value {
    let mut events: Vec<Event> = events
        .into_iter()
        .filter(|e| matches_query(e, query))
        .collect();
    if let Some(limit) = query.limit
        && events.len() > limit
    {
        events.drain(..events.len() - limit);
    }
    json!({"events": events, "floor_seq": floor_seq})
}

/// Does this event pass the query's filters?
///
/// One predicate, shared by the history route and the SSE pump, so a client
/// that asks the same question of both cannot get two different answers —
/// which is the whole point of `watch`'s attach-then-replay ordering.
///
/// D4/D6: `estate_root` matches the envelope's own coordinate (D1). An event
/// with no coordinate — every pre-envelope journal line, and everything not
/// bound to an estate at all — is filtered *out* when an estate is asked
/// for, because "I do not know which estate this belongs to" is not
/// evidence that it belongs to the one asked about.
fn matches_query(event: &Event, query: &EventsQuery) -> bool {
    if let Some(work_id) = &query.work_id
        && event.work_id.as_deref() != Some(work_id.as_str())
    {
        return false;
    }
    if let Some(estate_root) = &query.estate_root
        && event.workspace_id.as_deref() != Some(estate_root.as_str())
    {
        return false;
    }
    true
}

/// `GET /v1/events?from=N&work_id=X&limit=K&estate_root=R` — journaled
/// history after seq N.
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
    let floor_seq = match core.journal.floor_seq() {
        Ok(f) => f.unwrap_or(1), // `None` = empty journal; `1` is the honest floor of nothing
        Err(e) => return internal_error(e),
    };
    match core.events_after(query.from) {
        Ok(events) => Json(events_body(events, &query, floor_seq)).into_response(),
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
    tokio::spawn(pump_until_closing(state, from, query, tx));
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
    query: EventsQuery,
    tx: mpsc::Sender<Result<SseEvent, std::convert::Infallible>>,
) {
    let mut closing = state.closing.clone();
    tokio::select! {
        () = forward_events(state, from, query, tx) => {}
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
    query: EventsQuery,
    tx: mpsc::Sender<Result<SseEvent, std::convert::Infallible>>,
) {
    // Subscribe before reading history so nothing can fall in the gap.
    let mut live = {
        let core = CoreGuard::acquire(&state.core).await;
        core.events_tx.subscribe()
    };
    let mut last_sent = from;
    let (floor_seq, history) = {
        let core = CoreGuard::acquire(&state.core).await;
        let floor_seq = core.journal.floor_seq().ok().flatten().unwrap_or(1);
        (floor_seq, core.events_after(last_sent))
    };
    if send_sse_floor(&tx, floor_seq).await.is_err() {
        return;
    }
    match history {
        Ok(events) => {
            for event in events {
                // `last_sent` advances past a filtered-out event too: it is
                // the resume point, not a delivery count, and a client that
                // reconnects must not be handed the same skipped seqs again.
                if matches_query(&event, &query) && send_sse(&tx, &event).await.is_err() {
                    return;
                }
                last_sent = event.seq;
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "sse history replay failed; closing stream");
            let _ = send_sse_error(&tx, &e).await;
            return;
        }
    }
    loop {
        match live.recv().await {
            Ok(event) => {
                if event.seq <= last_sent {
                    continue; // already delivered from history
                }
                if matches_query(&event, &query) && send_sse(&tx, &event).await.is_err() {
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
                        let _ = send_sse_error(&tx, &e).await;
                        return;
                    }
                }
            }
            Err(broadcast::error::RecvError::Closed) => return,
        }
    }
}

/// Every **journaled** event kind [`send_sse`] can name a frame with — the
/// SSE stream's published vocabulary of `KIND_*` frames.
///
/// `EventSource` has no way to subscribe to "every named frame": a client that
/// wants all of them must name each one. Rather than let each client keep its
/// own copy of the list (the dashboard did, and it was already five kinds out
/// of date), the vocabulary is stated once, here, next to the function that
/// writes the frame names — and assembled from the journal's own `KIND_*`
/// constants so it cannot say a kind the journal does not have. `t6` in the M6
/// suite is the other half: it fails if a `KIND_*` constant is added to the
/// crate and not to this list.
///
/// This list is deliberately not the *complete* set of frame names this
/// stream can ever send: W4 adds one control frame outside it (the floor
/// marker, [`SSE_FLOOR_FRAME`]) precisely because it is not a journaled event
/// and forcing it into this list would either misname it as a `KIND_*` that
/// nothing journals, or weaken `tests/m6_surfaces.rs`'s bidirectional check
/// into a "some of these" assertion instead of "all of these." A client
/// that wants "every journaled kind" still gets exactly this list; a client
/// that wants "everything this stream can ever frame" also has to handle
/// the two names in [`SSE_CONTROL_FRAMES`] below.
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
    KIND_STAGE_OUTPUT_MISSING,
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
    // W1: the codex adapter's one module-local kind (§1.2 of the spec) — a
    // `pub const KIND_*`, unlike claude.rs's bare string-literal
    // "conversation.turn.grammar_unmeasured", so it is journaled and must be
    // named here for t6's bidirectional check to hold.
    KIND_TURN_HARNESS_ERROR,
    KIND_COMMAND_ACCEPTED,
    KIND_COMMAND_REJECTED,
    KIND_DAEMON_STARTED,
    KIND_DAEMON_STOPPED,
    KIND_BACKEND_PROBED,
    KIND_ADMISSION_PAUSED,
    KIND_ADMISSION_RESUMED,
    crate::runtime::prune::KIND_PRUNE_INTENT,
    crate::runtime::prune::KIND_PRUNE_COMPLETED,
];

/// Frame names this stream sends that are **not** journaled `KIND_*` events
/// — deliberately outside [`SSE_EVENT_KINDS`]'s contract (see its doc
/// comment). Both are sent with no `id:` field (SSE spec: an event with no
/// `id` does not update the client's last-event-id state), so neither can
/// ever poison a client's `Last-Event-ID` reconnect header with a
/// non-numeric value — `event_stream`'s `Last-Event-ID` parse
/// (`v.parse::<u64>()`) would 400 on anything else, which is exactly the
/// bug this constraint avoids.
pub const SSE_CONTROL_FRAMES: &[&str] = &[SSE_FLOOR_FRAME, SSE_STREAM_ERROR_FRAME];

/// Sent once per connection, immediately before history replay, naming the
/// floor at that moment — Q10's "no client may infer a floor of 1," carried
/// onto the one surface where there is no per-response JSON envelope to
/// carry it in.
pub const SSE_FLOOR_FRAME: &str = "sergeant.floor";

/// Sent once, only on the (pre-existing, W4 §1.1.3) journal-error close
/// path, naming why the stream is about to end rather than closing silently.
pub const SSE_STREAM_ERROR_FRAME: &str = "sergeant.stream_error";

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

/// The floor control frame ([`SSE_FLOOR_FRAME`]). No `id:` — see
/// [`SSE_CONTROL_FRAMES`]'s doc comment for why that is load-bearing, not
/// incidental.
async fn send_sse_floor(
    tx: &mpsc::Sender<Result<SseEvent, std::convert::Infallible>>,
    floor_seq: u64,
) -> Result<(), ()> {
    let data = serde_json::to_string(&json!({"floor_seq": floor_seq})).map_err(|_| ())?;
    let frame = SseEvent::default().event(SSE_FLOOR_FRAME).data(data);
    tx.send(Ok(frame)).await.map_err(|_| ())
}

/// The one error control frame this stream ever sends. Best-effort: if the
/// channel is already gone, sending fails the same way `send_sse` already
/// tolerates (the receiver dropped, nothing to report to).
async fn send_sse_error(
    tx: &mpsc::Sender<Result<SseEvent, std::convert::Infallible>>,
    error: &JournalError,
) -> Result<(), ()> {
    let data = serde_json::to_string(&json!({"error": error.to_string()})).map_err(|_| ())?;
    let frame = SseEvent::default().event(SSE_STREAM_ERROR_FRAME).data(data);
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
    estate_root: Option<PathBuf>,
}

impl ApiClient {
    /// Build a client for a daemon endpoint and its bearer token.
    ///
    /// D4: the client addresses **no** estate until one is named with
    /// [`Self::with_estate_root`]. That is the honest default under H1 — the
    /// daemon is host-scoped, so a client that has not said which estate it
    /// means has not said it — and it is what lets the host-scoped verb
    /// bucket W3 lands connect with no estate at all.
    pub fn new(endpoint: &str, token: &str) -> Result<Self, ClientError> {
        Ok(Self {
            http: reqwest::Client::builder()
                .timeout(client_timeout())
                .build()?,
            endpoint: endpoint.trim_end_matches('/').to_string(),
            token: token.to_string(),
            estate_root: None,
        })
    }

    /// Address every estate-scoped request from this client at `root` — the
    /// exact estate root the caller already admitted (§4.3 admits before a
    /// descriptor is even read, so this is never a fresh guess).
    pub fn with_estate_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.estate_root = Some(root.into());
        self
    }

    /// The estate this client addresses, if any.
    pub fn estate_root(&self) -> Option<&std::path::Path> {
        self.estate_root.as_deref()
    }

    /// `?estate_root=<root>` for the GET/DELETE routes that have no body to
    /// carry it in. Empty when this client addresses no estate, so the
    /// daemon answers with refusal (c) rather than being handed a blank.
    fn estate_query(&self) -> String {
        match &self.estate_root {
            Some(root) => format!("?estate_root={}", urlencode(&root.to_string_lossy())),
            None => String::new(),
        }
    }

    /// Add this client's estate address to a request body, if it has one.
    fn addressed(&self, mut body: Value) -> Value {
        if let (Some(root), Some(map)) = (&self.estate_root, body.as_object_mut()) {
            map.insert("estate_root".to_string(), json!(root));
        }
        body
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
            "/v1/workflows?cwd={}{}",
            urlencode(&cwd.display().to_string()),
            match &self.estate_root {
                Some(root) => format!("&estate_root={}", urlencode(&root.to_string_lossy())),
                None => String::new(),
            }
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

    /// `GET /v1/sweep` — §12.3's deliberate sweep, classification half
    /// (#159). Mutates nothing.
    pub async fn sweep(&self) -> Result<Value, ClientError> {
        self.get(&format!("/v1/sweep{}", self.estate_query())).await
    }

    /// `POST /v1/sweep` with a fresh command id (§26) — the deletion half.
    /// `confirm` must be `true` or the daemon refuses, and the daemon
    /// re-classifies every branch before deleting any of it.
    pub async fn sweep_delete(
        &self,
        branches: &[SweepTarget],
        confirm: bool,
    ) -> Result<Value, ClientError> {
        self.post(
            "/v1/sweep",
            &self.addressed(json!({
                "command_id": ulid::Ulid::generate().to_string(),
                "confirm": confirm,
                "branches": branches,
            })),
        )
        .await
    }

    /// `GET /v1/estate/repos` (§16.2/§20.4) — declared repositories.
    pub async fn repos(&self) -> Result<Value, ClientError> {
        self.get(&format!("/v1/estate/repos{}", self.estate_query()))
            .await
    }

    /// `POST /v1/estate/repos` (§16.2/§20.4) — `manifest::add_repo`.
    pub async fn add_repo(
        &self,
        name: &str,
        origin: Option<&str>,
        upstream: Option<&str>,
        instructions: Option<&str>,
    ) -> Result<Value, ClientError> {
        self.post(
            "/v1/estate/repos",
            &self.addressed(json!({
                "name": name,
                "origin": origin,
                "upstream": upstream,
                "instructions": instructions,
            })),
        )
        .await
    }

    /// `DELETE /v1/estate/repos/{name}` (§16.2/§20.4) — `manifest::remove_repo`.
    pub async fn remove_repo(&self, name: &str) -> Result<Value, ClientError> {
        self.delete(
            &format!(
                "/v1/estate/repos/{}{}",
                urlencode(name),
                self.estate_query()
            ),
            &json!({}),
        )
        .await
    }

    /// `GET /v1/estate/groups` (§16.2/§20.4) — declared groups.
    pub async fn groups(&self) -> Result<Value, ClientError> {
        self.get(&format!("/v1/estate/groups{}", self.estate_query()))
            .await
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
            &self.addressed(json!({"name": name, "repos": repos, "brief": brief})),
        )
        .await
    }

    /// `DELETE /v1/estate/groups/{name}` (§16.2/§20.4) — `manifest::remove_group`;
    /// empty `repos` removes the whole group, otherwise just those members.
    pub async fn remove_group(&self, name: &str, repos: &[String]) -> Result<Value, ClientError> {
        self.delete(
            &format!(
                "/v1/estate/groups/{}{}",
                urlencode(name),
                self.estate_query()
            ),
            &json!({"repos": repos}),
        )
        .await
    }

    /// `GET /v1/doctor` (§16.3/§20.4) — the same `doctor::Report` `sgt doctor
    /// --json` prints.
    pub async fn doctor(&self) -> Result<Value, ClientError> {
        self.get(&format!("/v1/doctor{}", self.estate_query()))
            .await
    }

    /// `GET /v1/estates` (H1 §4) — every estate this daemon has admitted.
    /// Host-scoped: it addresses no estate, because the answer *is* the set.
    pub async fn estates(&self) -> Result<Value, ClientError> {
        self.get("/v1/estates").await
    }

    /// Open the SSE live tail at
    /// `GET /v1/events/stream?from=N[&estate_root=R]`.
    ///
    /// `estate_root` (H1 sprint-plan D4/D6) is a parameter of this call, not
    /// of the client itself: `sgt watch`'s filter (`WatchOptions`,
    /// `src/watch.rs`) is a separate decision from what estate this client
    /// otherwise addresses (`Self::with_estate_root`) — `--all` must be able
    /// to ask for every estate's events even from a client that *does*
    /// address one for its other requests. `None` is host-wide: every
    /// admitted estate's events, unfiltered.
    ///
    /// A separate reqwest client is built with no total timeout: the response
    /// body of a live tail is *supposed* to stay open, and the per-request
    /// timeout that keeps a stuck command honest would kill it on schedule.
    pub async fn stream_events(
        &self,
        from: u64,
        estate_root: Option<&std::path::Path>,
    ) -> Result<EventStream, ClientError> {
        let http = reqwest::Client::builder().build()?;
        let url = match estate_root {
            Some(root) => format!(
                "{}/v1/events/stream?from={from}&estate_root={}",
                self.endpoint,
                urlencode(&root.to_string_lossy())
            ),
            None => format!("{}/v1/events/stream?from={from}", self.endpoint),
        };
        let response = http.get(url).bearer_auth(&self.token).send().await?;
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
///
/// W4 §1.1.2: a frame named for one of [`SSE_CONTROL_FRAMES`] (`sergeant.floor`,
/// `sergeant.stream_error`) is not a journaled event at all — checked by
/// name, before the parse, so this client treats it exactly like axum's own
/// keep-alive comment (silently skipped) rather than a decode failure. This
/// is what keeps the two control frames additive to the wire contract: an
/// older client (or one that has not yet been taught to *use* either frame)
/// still reads the rest of the stream correctly instead of erroring on the
/// first connection. A genuinely malformed journaled-event frame — the case
/// [`MalformedFrame`] exists for — is unaffected: it is still whatever
/// `event:` name a real `KIND_*` carries, never one of these two.
fn decode_frame(frame: &str) -> FrameDecode {
    let mut data = String::new();
    let mut event_name: Option<&str> = None;
    for line in frame.lines() {
        if let Some(rest) = line.strip_prefix("event:") {
            event_name = Some(rest.trim());
        } else if let Some(rest) = line.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(rest.strip_prefix(' ').unwrap_or(rest));
        }
    }
    if event_name.is_some_and(|name| SSE_CONTROL_FRAMES.contains(&name)) {
        return FrameDecode::KeepAlive;
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

    // ------------------------------------------------------------------
    // §26 refuse-by-name (Q8) — `below_floor_refusal`'s own wire shape
    // (spec §4.3). Finding: no test anywhere in the diff constructed a
    // `FloorCommandRow` and called this function before this wave's fixer
    // pass — the entire 409/`command_below_replay_window` contract shipped
    // unverified. Four cases, one per `below_floor_refusal` match arm.
    // ------------------------------------------------------------------

    /// A submit's accepted outcome, below the window: 409, naming the Work
    /// it created — spec §4.3's own worked JSON example.
    #[tokio::test]
    async fn below_floor_refusal_names_the_work_for_an_accepted_submit() {
        let row = FloorCommandRow {
            command_id: "01JZTESTCOMMAND0000000000".to_string(),
            class: FloorCommandClass::Accepted,
            work_id: Some("01JWTESTWORK000000000000".to_string()),
        };
        let response = below_floor_refusal(&row);
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let body: Value = serde_json::from_slice(&bytes).expect("json body");
        assert_eq!(body["error"]["code"], "command_below_replay_window");
        assert_eq!(body["error"]["command_id"], row.command_id);
        assert_eq!(body["error"]["outcome"], "accepted");
        assert_eq!(body["error"]["work_id"], json!(row.work_id));
        let message = body["error"]["message"].as_str().expect("message string");
        assert!(
            message.contains(&row.command_id) && message.contains(row.work_id.as_ref().unwrap()),
            "message must name both the command and the Work it created: {message}"
        );
    }

    /// The crash-window `Submitted` class (`work.submitted` with no
    /// following `command.accepted`) hits the same "names the Work" arm as
    /// `Accepted` — the message and `work_id` must not depend on whether the
    /// accepting event ever actually landed.
    #[tokio::test]
    async fn below_floor_refusal_names_the_work_for_a_crash_window_submit() {
        let row = FloorCommandRow {
            command_id: "cmd-submitted".to_string(),
            class: FloorCommandClass::Submitted,
            work_id: Some("work-submitted".to_string()),
        };
        let response = below_floor_refusal(&row);
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let body: Value = serde_json::from_slice(&bytes).expect("json body");
        assert_eq!(body["error"]["code"], "command_below_replay_window");
        assert_eq!(body["error"]["outcome"], "submitted");
        assert_eq!(body["error"]["work_id"], json!("work-submitted"));
    }

    /// A rejected command: 409, `work_id` present-but-null (never omitted —
    /// spec §4.3), and a message that says it was rejected rather than
    /// naming a Work.
    #[tokio::test]
    async fn below_floor_refusal_reports_a_rejected_command_with_null_work_id() {
        let row = FloorCommandRow {
            command_id: "cmd-rejected".to_string(),
            class: FloorCommandClass::Rejected,
            work_id: None,
        };
        let response = below_floor_refusal(&row);
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let body: Value = serde_json::from_slice(&bytes).expect("json body");
        assert_eq!(body["error"]["code"], "command_below_replay_window");
        assert_eq!(body["error"]["outcome"], "rejected");
        assert_eq!(
            body["error"]["work_id"],
            Value::Null,
            "work_id must be present-but-null, never omitted, for a non-submit"
        );
        assert!(
            body["error"]["message"]
                .as_str()
                .expect("message string")
                .contains("rejected")
        );
    }

    /// An accepted command with no Work at all (an admin-scoped command,
    /// e.g. `admission.pause`): 409, `work_id` present-but-null, message says
    /// it was accepted without naming a Work.
    #[tokio::test]
    async fn below_floor_refusal_reports_an_accepted_admin_command_with_null_work_id() {
        let row = FloorCommandRow {
            command_id: "cmd-admin".to_string(),
            class: FloorCommandClass::Accepted,
            work_id: None,
        };
        let response = below_floor_refusal(&row);
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let body: Value = serde_json::from_slice(&bytes).expect("json body");
        assert_eq!(body["error"]["code"], "command_below_replay_window");
        assert_eq!(body["error"]["outcome"], "accepted");
        assert_eq!(body["error"]["work_id"], Value::Null);
        assert!(
            body["error"]["message"]
                .as_str()
                .expect("message string")
                .contains("accepted")
        );
    }

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
            .stream_events(0, None)
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
            estates: Arc::new(crate::runtime::estates::EstateRegistry::new()),
            analytics: Arc::new(tokio::sync::Mutex::new(analytics)),
            prune_policy: crate::runtime::prune::PrunePolicy {
                retention: crate::domain::estate::DEFAULT_RETENTION,
                source: crate::runtime::prune::PolicySource::Default,
            },
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
        let pump = tokio::spawn(forward_events(state.clone(), 0, EventsQuery::default(), tx));

        // W4 §1.1.2: the very first frame on any connection is the floor
        // control frame, sent before history — consumed here, separately,
        // so it does not throw off this test's exact history/live frame
        // count below (it is not one of `send_sse`'s per-event frames).
        // `sse::Event` exposes no field getters; its `Debug` renders the
        // already-encoded wire bytes, which is enough to pin the frame name.
        let floor = rx
            .recv()
            .await
            .expect("the floor frame")
            .expect("not an error");
        assert!(
            format!("{floor:?}").contains(&format!("event: {SSE_FLOOR_FRAME}")),
            "the first frame must be the floor control frame: {floor:?}"
        );

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

    /// w4-spec.md §1.1.3 (A6) names two tests for the SSE error-frame fix:
    /// one for the initial history-replay `Err` arm
    /// (`sse_stream_sends_an_error_frame_before_closing_on_a_journal_failure`,
    /// `tests/w4_read_surfaces.rs`) and one for the `Lagged` refill arm —
    /// this one. Both arms call the same `send_sse_error`, but only the
    /// first had a dedicated test; the refill arm shipped with no coverage
    /// anywhere in the suite (a revert of its `send_sse_error` call would
    /// have passed every test that existed at the time).
    ///
    /// Lag cannot be provoked deterministically through the real HTTP
    /// surface without pushing past the daemon's 1024-slot broadcast, so
    /// this drives `forward_events` directly — the same wedge
    /// `a_lag_refill_resumes_from_the_last_history_frame_the_pump_actually_sent`
    /// above uses to force the subscriber's first `recv()` to be `Lagged` by
    /// the channel's own overwrite contract. The journal corruption is the
    /// same fault-injection seam
    /// `sse_stream_sends_an_error_frame_before_closing_on_a_journal_failure`
    /// uses (a malformed line appended straight to the segment file),
    /// applied here *after* the initial history fetch has already happened
    /// (it happens synchronously, before the floor frame is even sent, so
    /// receiving the floor frame proves it is already in memory) — so it
    /// cannot affect history replay, only the later refill.
    #[tokio::test]
    async fn sse_lag_refill_failure_also_sends_the_error_frame() {
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
        let pump = tokio::spawn(forward_events(state.clone(), 0, EventsQuery::default(), tx));

        let floor = rx
            .recv()
            .await
            .expect("the floor frame")
            .expect("not an error");
        assert!(
            format!("{floor:?}").contains(&format!("event: {SSE_FLOOR_FRAME}")),
            "the first frame must be the floor control frame: {floor:?}"
        );

        let first = rx.recv().await.expect("the first history frame");
        assert!(
            first.is_ok(),
            "history replay itself must not error before any corruption"
        );

        // From here on the on-disk journal is corrupted. The `history` Vec
        // the pump is still draining was already fetched (before the floor
        // frame above was sent), so this cannot touch it — only the later
        // refill, which reads the journal fresh, sees the damage.
        let journal_dir = dir.path().join("journal");
        let mut segments: Vec<_> = std::fs::read_dir(&journal_dir)
            .expect("journal dir")
            .filter_map(|entry| {
                let path = entry.expect("entry").path();
                (path.extension().is_some_and(|ext| ext == "ndjson")).then_some(path)
            })
            .collect();
        segments.sort();
        let segment = segments.last().expect("a segment exists").clone();
        let mut text = std::fs::read_to_string(&segment).expect("read segment");
        text.push_str("{ \"not\": \"an event\" }\n");
        std::fs::write(&segment, text).expect("append malformed line");

        // Commit past the broadcast's 16-slot capacity while the sink still
        // holds only one frame's room — everything committed here piles up
        // unread in the broadcast ring, exactly as in
        // `a_lag_refill_resumes_...` above.
        {
            let mut core = CoreGuard::acquire(&state.core).await;
            for n in HISTORY + 1..=HISTORY + LIVE {
                core.commit(seeded(n)).expect("commit live");
            }
        }

        // Drain the remaining history frames — none of them error, since
        // they were already fetched before the corruption above.
        for _ in 0..(HISTORY - 1) {
            let frame = tokio::time::timeout(Duration::from_secs(5), rx.recv())
                .await
                .expect("history frame")
                .expect("channel still open");
            assert!(
                frame.is_ok(),
                "history replay itself must not error: {frame:?}"
            );
        }

        // The live loop now makes its first `recv()` — which lags, by the
        // broadcast's own overwrite contract, since nothing drained it while
        // 20 events piled up. The refill that follows hits the corrupted
        // journal: the stream must end with the named error frame, not hang
        // or close silently.
        let error_frame = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("an error frame must arrive, not hang")
            .expect("channel still open")
            .expect("SseEvent frames are never Err — the channel item type is Infallible");
        assert!(
            format!("{error_frame:?}").contains(&format!("event: {SSE_STREAM_ERROR_FRAME}")),
            "a lag refill's journal failure must be named, not silent: {error_frame:?}"
        );

        let closed = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("the channel must close promptly after the error frame, not hang");
        assert!(
            closed.is_none(),
            "the stream must end after the error frame, not keep sending"
        );

        pump.await.expect("the pump task must not panic");
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
            // Audited (H1 touch point #4): this fixture exercises
            // `transcript_turns`, which reads only `work_id`/`kind`/
            // `payload` — never estate-bound, so there is nothing to
            // populate here.
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

    /// #240: `tool.requested`/`tool.completed` used to fall into
    /// `transcript_turns`'s catch-all and vanish from the transcript
    /// entirely — a degraded run that silently invoked zero tools read
    /// identically to one that made real progress. Both kinds must now
    /// decode as `role: "tool_use"` entries, distinguished by `phase`, in
    /// the same causal order as every other turn kind.
    #[test]
    fn transcript_turns_surfaces_tool_requested_and_completed_events() {
        let events = vec![
            ev(
                1,
                "w1",
                KIND_CONVERSATION_USER,
                json!({"text": "run the tests"}),
            ),
            ev(
                2,
                "w1",
                KIND_TOOL_REQUESTED,
                json!({"id": "call-1", "name": "bash", "input": {"command": "cargo test"}}),
            ),
            ev(
                3,
                "w1",
                KIND_TOOL_COMPLETED,
                json!({"tool_use_id": "call-1", "name": "bash", "is_error": false}),
            ),
        ];
        let data_dir = tempfile::TempDir::new().expect("tempdir");
        let turns = transcript_turns("w1", events, data_dir.path());

        let requested = turns
            .iter()
            .find(|t| t["seq"] == 2)
            .expect("tool.requested must decode into a turn");
        assert_eq!(requested["role"], "tool_use");
        assert_eq!(requested["phase"], "requested");
        assert_eq!(requested["tool_use_id"], "call-1");
        assert_eq!(requested["name"], "bash");

        let completed = turns
            .iter()
            .find(|t| t["seq"] == 3)
            .expect("tool.completed must decode into a turn");
        assert_eq!(completed["role"], "tool_use");
        assert_eq!(completed["phase"], "completed");
        assert_eq!(completed["tool_use_id"], "call-1");
        assert_eq!(completed["is_error"], false);
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
    /// `conversation.assistant.completed` — modeling CONTRIBUTING.md's own
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
                    // Audited (H1 touch point #4): mirrors whatever the
                    // backend's own draft carried rather than hardcoding
                    // `None` — backend-emitted conversation events are
                    // outside this chokepoint's scope (`Engine::commit`),
                    // so nothing here should invent a value the adapter
                    // never set.
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
            instruction_policy: crate::domain::estate::InstructionPolicy::default(),
            bindings: Vec::new(),
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

    /// #4's own version of the test above, one layer further out: not just
    /// the *run* cache but the *Work* cache too. Once `target_id` ages past
    /// `TERMINAL_WORK_CACHE_CAPACITY`, `work_view` must route through
    /// `resolve_work`'s journal fallback and answer byte-identically to
    /// what it answered while the Work was still cached — R-MVP1-9's own
    /// pin ("an evicted view is byte-identical to a non-evicted one"),
    /// restated for #4. `fleet_body`'s row for the same id must also still
    /// be there (narrowed, `"evicted": true`), and an id that was never
    /// journaled at all must still 404 through the slim-index existence
    /// check, exactly as it did before any of this churn.
    #[test]
    fn a_work_view_survives_terminal_work_cache_eviction_and_unknown_ids_still_404() {
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

        let target_id = "01TARGETWORK00000001";
        testing::submit(&mut core, target_id, "declares a commit, never makes one");
        testing::commit(
            &mut core,
            target_id,
            KIND_SURFACE_MATERIALIZED,
            surface(target_id),
        );
        testing::commit(&mut core, target_id, KIND_WORK_COMPLETED, json!({}));
        testing::commit(
            &mut core,
            target_id,
            KIND_SURFACE_TORN_DOWN,
            ordinary_teardown(target_id),
        );

        let backends = BackendRegistry::new().with(Arc::new(
            crate::backend::fake::FakeBackend::new(crate::backend::fake::FAKE_BACKEND_NAME),
        ));
        let engine = Engine::new(
            Arc::new(backends),
            Some(crate::backend::fake::FAKE_BACKEND_NAME.to_string()),
            dir.path(),
        );

        assert!(
            core.registry.state().terminal_works.contains_key(target_id),
            "precondition: still cached, or the before/after comparison below \
             proves nothing about eviction specifically"
        );
        let before = work_view(&core, &engine, target_id);

        // Churn ordinary completions past `projection.rs`'s own
        // `TERMINAL_WORK_CACHE_CAPACITY` (1024, private to that module) so
        // `target_id` — submitted first, so it is the oldest — actually
        // ages out of it.
        for i in 0..1224 {
            let id = format!("01WORKCACHECHURN{i:06}");
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
            !core.registry.state().terminal_works.contains_key(target_id),
            "the target must actually have aged out of the bounded cache for \
             this test to prove anything"
        );
        assert!(
            core.registry.state().work_index.contains_key(target_id),
            "the slim index must still know this id exists"
        );

        let after = work_view(&core, &engine, target_id);
        assert_eq!(
            before, after,
            "an evicted Work's view must be byte-identical to what it answered \
             while still cached: before={before}, after={after}"
        );

        let fleet = fleet_body(&core, &engine);
        let row = fleet["works"]
            .as_array()
            .expect("works")
            .iter()
            .find(|w| w["id"] == target_id)
            .expect("the evicted-past-cache work is still listed, narrowed");
        assert_eq!(row["evicted"], true, "{row}");
        assert_eq!(row["state"], after["work"]["state"], "{row}");
        assert_eq!(row["intent"], after["work"]["intent"], "{row}");

        // Existence checks must still 404 correctly for a truly unknown id
        // — unaffected by any of the churn above.
        assert!(
            !core
                .registry
                .state()
                .work_index
                .contains_key("01NOSUCHWORKATALL"),
            "an id nothing ever journaled must not be found"
        );
    }

    /// Panel finding (the explicit-Dirty half): `evicted_fleet_row` reads
    /// `WorkIndexRow::integrity` directly, and an explicit `Dirty`
    /// disposition was always mirrored into it verbatim — so this half
    /// already passed before the fix. It is kept here as a regression guard
    /// alongside its stranded-completion sibling just below, which the fix
    /// was actually for.
    ///
    /// Same churn shape as
    /// `a_work_view_survives_terminal_work_cache_eviction_and_unknown_ids_still_404`
    /// just above: submit the target first (so it is oldest and ages out
    /// first), then churn ordinary completions past
    /// `TERMINAL_WORK_CACHE_CAPACITY` (1024) so `evicted_fleet_row` is the
    /// branch `fleet_body` actually takes for it, not the full-row branch.
    #[test]
    fn evicted_fleet_row_reports_completed_dirty_for_an_explicit_dirty_disposition() {
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

        fn explicit_dirty_teardown(work_id: &str) -> Value {
            // A clean removal in every way `TeardownReport::integrity()`
            // itself would score, but retired with an explicit `Dirty`
            // disposition anyway — the two ways to `completed_dirty` §11.5's
            // own doc calls out as a union rather than a competition.
            let mut payload = ordinary_teardown(work_id);
            payload["integrity"] = json!("dirty");
            payload
        }

        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut core = testing::core(dir.path());

        let target_id = "01EXPLICITDIRTY0000001";
        testing::submit(&mut core, target_id, "retired dirty on purpose");
        testing::commit(
            &mut core,
            target_id,
            KIND_SURFACE_MATERIALIZED,
            surface(target_id),
        );
        testing::commit(&mut core, target_id, KIND_WORK_COMPLETED, json!({}));
        testing::commit(
            &mut core,
            target_id,
            KIND_SURFACE_TORN_DOWN,
            explicit_dirty_teardown(target_id),
        );

        let backends = BackendRegistry::new().with(Arc::new(
            crate::backend::fake::FakeBackend::new(crate::backend::fake::FAKE_BACKEND_NAME),
        ));
        let engine = Engine::new(
            Arc::new(backends),
            Some(crate::backend::fake::FAKE_BACKEND_NAME.to_string()),
            dir.path(),
        );

        // Churn ordinary completions past `TERMINAL_WORK_CACHE_CAPACITY`
        // (1024, private to `projection.rs`) so `target_id` — submitted
        // first, so it is the oldest — actually ages out of `terminal_works`
        // and `evicted_fleet_row` is the branch `fleet_body` actually takes.
        for i in 0..1224 {
            let id = format!("01EXPLICITDIRTYCHURN{i:06}");
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
            !core.registry.state().terminal_works.contains_key(target_id),
            "the target must actually have aged out of the bounded cache for \
             this test to prove anything"
        );

        let fleet = fleet_body(&core, &engine);
        let row = fleet["works"]
            .as_array()
            .expect("works")
            .iter()
            .find(|w| w["id"] == target_id)
            .expect("the evicted-past-cache work is still listed, narrowed");
        assert_eq!(row["evicted"], true, "{row}");
        assert_eq!(
            row["state"], "completed_dirty",
            "an explicit Dirty disposition must still read completed_dirty \
             once the Work has aged past TERMINAL_WORK_CACHE_CAPACITY: {row}"
        );
    }

    /// Panel finding (the core fix, stranded-completion half): before this
    /// fix, `WorkIndexRow::integrity` only ever mirrored the *explicit*
    /// disposition a `surface.torn_down` carried — never ADR 0007(b)'s
    /// structural inference (`stranded_completion`: a closing stage's
    /// finalize commit that never moved past the surface's base SHA, with a
    /// `RetainedDirty` binding). A stranded completion with no explicit
    /// `Dirty` disposition therefore read `completed_dirty` in `sgt work
    /// show`/`sgt work list` right up until its Work aged past
    /// `TERMINAL_WORK_CACHE_CAPACITY` — at which point `evicted_fleet_row`,
    /// reading `WorkIndexRow` alone, would find `row.integrity` still `None`
    /// and silently revert it to plain `completed`. The fix folds the
    /// structural check into `WorkIndexRow::integrity` itself, at
    /// `surface.torn_down` time in the reducer (`projection.rs`), while
    /// `run.surface`/`run.teardown` are still in hand.
    ///
    /// **Why this fails without the fix.** This fixture's teardown payload
    /// carries no `integrity` key at all (mirroring
    /// `a_stranded_completion_survives_terminal_run_cache_eviction`'s
    /// `stranded_teardown` shape exactly: `clean: false`, a `retained_dirty`
    /// binding whose `final_sha` equals the surface binding's own
    /// `base_sha`, i.e. never advanced) — so on the pre-fix reducer,
    /// `row.integrity` is set from `run.integrity`, which
    /// `serde_json::from_value` on an absent key resolves to `None`.
    /// `evicted_fleet_row` only special-cases `row.integrity ==
    /// Some(Dirty)`; with `row.integrity == None` it falls straight to
    /// `row.state.as_str()`, i.e. `"completed"`. Only after the Work ages
    /// past `TERMINAL_WORK_CACHE_CAPACITY` does that gap become visible —
    /// while cached, `fleet_body`/`work_view` both still resolve the full
    /// run and re-run `stranded_completion` directly, which is exactly why
    /// this test churns past the cache rather than asserting immediately.
    #[test]
    fn evicted_fleet_row_reports_completed_dirty_for_a_stranded_completion() {
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
                    // Never advanced past the base SHA `surface` recorded —
                    // no explicit `integrity` key anywhere in this payload.
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

        let stranded_id = "01STRANDEDEVICTED0001";
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

        let backends = BackendRegistry::new().with(Arc::new(
            crate::backend::fake::FakeBackend::new(crate::backend::fake::FAKE_BACKEND_NAME),
        ));
        let engine = Engine::new(
            Arc::new(backends),
            Some(crate::backend::fake::FAKE_BACKEND_NAME.to_string()),
            dir.path(),
        );

        // Churn ordinary completions past `TERMINAL_WORK_CACHE_CAPACITY`
        // (1024) so `stranded_id` — submitted first, so it is the oldest —
        // actually ages out of `terminal_works` and `evicted_fleet_row` is
        // the branch `fleet_body` actually takes for it.
        for i in 0..1224 {
            let id = format!("01STRANDEDEVICTEDCHURN{i:06}");
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
            !core
                .registry
                .state()
                .terminal_works
                .contains_key(stranded_id),
            "the stranded work must actually have aged out of the bounded \
             cache for this test to prove anything"
        );

        let fleet = fleet_body(&core, &engine);
        let row = fleet["works"]
            .as_array()
            .expect("works")
            .iter()
            .find(|w| w["id"] == stranded_id)
            .expect("the evicted-past-cache work is still listed, narrowed");
        assert_eq!(row["evicted"], true, "{row}");
        assert_eq!(
            row["state"], "completed_dirty",
            "an evicted stranded completion (ADR 0007(b)'s structural \
             inference, no explicit Dirty disposition) must not silently \
             revert to plain completed just because its Work aged past \
             TERMINAL_WORK_CACHE_CAPACITY: {row}"
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
