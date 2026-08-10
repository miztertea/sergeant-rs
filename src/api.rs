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
use std::time::Duration;

use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::extract::{Path, Query, Request, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::{broadcast, mpsc, watch};
use tokio_stream::wrappers::ReceiverStream;

use crate::daemon::{KIND_BACKEND_PROBED, KIND_DAEMON_STARTED, KIND_DAEMON_STOPPED};
use crate::domain::event::{Event, EventDraft, EventSource, rfc3339_utc_now};
use crate::domain::execution::{
    KIND_EXECUTION_ABANDONED, KIND_EXECUTION_RECONCILED, KIND_EXECUTION_RESERVED,
    KIND_EXECUTION_STARTED, KIND_EXECUTION_STOPPED,
};
use crate::domain::work::{
    KIND_COMMAND_ACCEPTED, KIND_COMMAND_REJECTED, KIND_WORK_BLOCKED, KIND_WORK_CANCELED,
    KIND_WORK_COMPLETED, KIND_WORK_FAILED, KIND_WORK_NEEDS_INPUT, KIND_WORK_RESUMED,
    KIND_WORK_STARTED, KIND_WORK_SUBMITTED, KIND_WORK_WAITING, Work, WorkState,
};
use crate::domain::workflow::{
    KIND_STAGE_BLOCKED, KIND_STAGE_CANCELED, KIND_STAGE_COMPLETED, KIND_STAGE_ENTERED,
    KIND_STAGE_FAILED, KIND_STAGE_INPUT_RECEIVED, KIND_STAGE_NEEDS_INPUT, KIND_STAGE_WAITING,
    KIND_WORKFLOW_BOUND,
};
use crate::runtime::analytics::{Analytics, AnalyticsError, CANNED_QUERIES};
use crate::runtime::engine::{Engine, EngineError, Next as EngineNext, Step, SubmitContext};
use crate::runtime::graph::{
    KIND_CONVERSATION_ASK, KIND_CONVERSATION_ASSISTANT_COMPLETED, KIND_CONVERSATION_TURN_ENDED,
    KIND_CONVERSATION_USER, KIND_TOOL_COMPLETED, KIND_TOOL_REQUESTED, KIND_USAGE_UPDATED,
};
use crate::runtime::journal::{Journal, JournalError};
use crate::runtime::projection::{Projection, ProjectionError, WorkRegistry, WorkRun};
use crate::runtime::surface::{
    KIND_SURFACE_MATERIALIZED, KIND_SURFACE_MATERIALIZING, KIND_SURFACE_TORN_DOWN,
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
    /// Append one event to the journal (fsynced), fold it into the registry,
    /// and fan it out to live SSE subscribers. The only mutation path.
    pub fn commit(&mut self, draft: EventDraft) -> Result<Event, CoreError> {
        let event = self.journal.append(draft)?;
        self.registry.apply(&event)?;
        // No live subscriber is not an error.
        let _ = self.events_tx.send(event.clone());
        Ok(event)
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

/// Build the axum router for the full v1 surface plus the embedded dashboard.
///
/// The dashboard is mounted here, but it is not part of `/v1`: it is a
/// *client* of it, and it is handed [`ApiViews`] — the same response bodies
/// the endpoints below return and nothing else (§29's "the dashboard is not a
/// second backend; it is an API projection", made structural).
pub fn router(state: ApiState) -> Router {
    let v1 = Router::new()
        .route("/work", post(submit_work).get(list_work))
        .route("/work/{id}", get(show_work))
        .route("/work/{id}/cancel", post(cancel_work))
        .route("/work/{id}/input", post(work_input))
        .route("/work/{id}/retry", post(work_retry))
        .route("/graph/work/{id}", get(work_graph))
        .route("/analytics", get(analytics_index))
        .route("/analytics/{name}", get(analytics_query))
        .route("/events", get(event_history))
        .route("/events/stream", get(event_stream))
        .route("/system", get(system_info))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_bearer,
        ));
    // The dashboard goes behind the *same* gate, rather than carrying a
    // second one: `web.rs` renders pages, it does not decide authorization
    // (R2 — the middleware is already here; reuse it). Its own
    // `method_not_allowed_fallback` is set here because the outer one below
    // only rewrites the routes the router already holds, and a merge brings
    // these in afterwards: without this line `POST /ui` answered with axum's
    // stock empty body while every other route on the listener answered
    // structured JSON.
    let ui = crate::web::routes(ApiViews::new(state.clone()))
        .method_not_allowed_fallback(method_not_allowed)
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
        .merge(ui)
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
/// the boundary: the guard is dropped before every launch and before every
/// completion wait, which are the only places §22.6 cares about.
async fn crank(state: &ApiState, step: Step) -> Option<tokio::sync::MutexGuard<'_, Core>> {
    let mut step = step;
    let mut held: Option<tokio::sync::MutexGuard<'_, Core>> = None;
    loop {
        let Step { next, deferred } = step;
        if deferred.is_pending() {
            drop(held.take()); // never wait on an adapter under the guard
            blocking(move || deferred.wait()).await;
        }
        let settled = match next {
            EngineNext::Parked => return held,
            EngineNext::Launch(pending) => {
                drop(held.take()); // never launch under the guard
                let outcome = blocking(|| pending.perform()).await;
                let work_id = pending.work_id().to_string();
                let mut core = state.core.lock().await;
                (
                    work_id,
                    state.engine.settle_launch(&mut core, pending, outcome),
                    core,
                )
            }
            EngineNext::Send(pending) => {
                drop(held.take()); // nor deliver input under it (§14.2 for SEND)
                let outcome = blocking(|| pending.perform()).await;
                let work_id = pending.work_id().to_string();
                let mut core = state.core.lock().await;
                (
                    work_id,
                    state.engine.settle_send(&mut core, pending, outcome),
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

/// Take the guard [`crank`] handed back, or acquire one if it kept none.
async fn relock<'a>(
    state: &'a ApiState,
    held: Option<tokio::sync::MutexGuard<'a, Core>>,
) -> tokio::sync::MutexGuard<'a, Core> {
    match held {
        Some(guard) => guard,
        None => state.core.lock().await,
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
/// depending on which route it knocked on. The dashboard's extractor needs it
/// for a rejection the gate makes unreachable; the gate itself uses it below.
pub fn unauthorized_response() -> Response {
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
        | EngineError::IllegalTransition { .. } => StatusCode::CONFLICT,
        EngineError::NoRun { .. } => StatusCode::NOT_FOUND,
        _ => StatusCode::UNPROCESSABLE_ENTITY,
    }
}

/// Name of the query parameter that may carry the token on safe requests.
pub const TOKEN_QUERY_PARAM: &str = "token";

/// The bearer token presented by a request, if any.
///
/// Two carriers, deliberately unequal:
///
/// - `Authorization: Bearer <token>` — always accepted;
/// - `?token=<token>` — accepted on **GET and HEAD only**.
///
/// The query form exists because the browser is a client too and cannot set a
/// header on a `<link>`, an `<img>`, or an `EventSource` (§29 requires the
/// dashboard to live-update over the existing SSE endpoint, and `EventSource`
/// has no header API at all). The tradeoff is real — a URL-borne secret lands
/// in shell history, in the terminal scrollback `sgt web` printed it to, and
/// in any log that records request lines — and it is accepted for P0 on the
/// grounds that the listener is loopback-only, the token is regenerated on
/// every daemon start, and the descriptor holding it is already 0600. The
/// post-P0 alternative, for the ledger entry this milestone's MARK & LOG
/// still owes (R1 — ship the cheapest thing that closes the P0 and name the
/// better shape rather than build it): exchange the URL token once for a
/// `HttpOnly; SameSite=Strict` cookie and drop the query form. That entry is
/// not written yet, and this comment does not claim it is.
///
/// Restricting it to safe methods is the part that costs nothing: a page that
/// learns the token can already read everything, but a *mutating* request
/// authorized by a URL alone is the shape a cross-site form post can forge
/// without ever reading a response. Mutations therefore keep requiring a
/// header, which no cross-origin form can set.
///
/// **This is the only copy of the rule.** The dashboard needs the same
/// extraction — its pages have to put the token back on every link they
/// render — and the second copy it used to keep had already drifted from this
/// one: it took a query token on any method and percent-decoded it. Both
/// clients call this function now.
pub fn presented_token(
    method: &Method,
    headers: &HeaderMap,
    query: Option<&str>,
) -> Option<String> {
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

/// Bearer-token gate for `/v1/*` **and for the dashboard**. `/healthz` is the
/// only route mounted outside this layer.
///
/// The dashboard used to re-implement this: its own extraction, its own
/// comparison, and its own hand-written copy of the 401 body below. One gate
/// decides for every route on this listener now — a second implementation of
/// an authorization rule is a second rule, and it had already diverged on the
/// safe-method bound.
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
fn system_body(state: &ApiState, journal_head: u64) -> Value {
    json!({
        "version": env!("CARGO_PKG_VERSION"),
        "api_revision": API_REVISION,
        "data_dir": state.data_dir,
        "journal_head": journal_head,
    })
}

/// `GET /v1/system` — version, API revision, data dir, journal head.
async fn system_info(State(state): State<ApiState>) -> Json<Value> {
    let head = state.core.lock().await.registry.last_seq();
    Json(system_body(&state, head))
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
}

/// §13's origin metadata: who is asking, and from where.
#[derive(Debug, Default, Deserialize)]
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
    let mut core = state.core.lock().await;
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
        let result = work_view(&core, &work_id);
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

    // Plan before creating anything. Workspace topology, workflow content,
    // routing and profile are all decided here, with no side effects — so a
    // submission that cannot be routed is rejected with §13's available
    // options instead of creating work that immediately dies. `Ok(None)`
    // means the client offered no repository context; the work is accepted
    // and stays `pending`, exactly as it did before there was an engine.
    let origin = req.origin.unwrap_or_default();
    let plan = match state.engine.plan(&SubmitContext {
        cwd: origin.cwd.as_deref(),
        origin_client: origin.client.as_deref(),
        backend: req.backend.as_deref(),
        workflow: req.workflow.as_deref(),
        profile: req.profile.as_deref(),
        repositories: &req.repositories,
    }) {
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
        drop(core);
        core = relock(&state, crank(&state, step).await).await;
    }

    // Answer from the projection, not the request: proves the read path.
    let result = work_view(&core, &work_id);
    record_and_respond(
        &mut core,
        &req.command_id,
        "work.submit",
        Some(&work_id),
        StatusCode::CREATED,
        result,
    )
}

/// The full view of a work: the §10 record, plus the orthogonal run
/// coordinates the M3 contract asks `work show` to include — current stage,
/// surface, and execution state. They are siblings of `work`, not fields
/// inside it: §10 keeps stage orthogonal to work state, and flattening them
/// into one record is how "in review" becomes a state-machine value.
fn work_view(core: &Core, work_id: &str) -> Value {
    let registry = core.registry.state();
    let work = registry.works.get(work_id);
    let run = registry.runs.get(work_id);
    json!({
        "work": work,
        "stage": run.and_then(run_stage_view),
        "surface": run.and_then(|r| r.surface.clone()),
        "execution": run.and_then(|r| r.execution.clone()),
        // Additive (§20.5): a run whose launch phase is in flight, or whose
        // launch phase a crash left unaccounted for, is a state a client can
        // now see rather than infer from a gap between events.
        "reservation": run.and_then(|r| r.reservation.clone()),
        "workflow": run.and_then(|r| r.workflow.as_ref().map(|w| json!({
            "name": w.name,
            "version": w.version,
            "source": w.source,
            "stages": w.stages.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
        }))),
        "backend": run.and_then(|r| r.backend.clone()),
        "route_source": run.and_then(|r| r.route_source.clone()),
        "teardown": run.and_then(|r| r.teardown.clone()),
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
fn fleet_body(core: &Core) -> Value {
    let registry = core.registry.state();
    let works: Vec<Value> = registry
        .works
        .values()
        .map(|work| {
            let run = registry.runs.get(&work.id);
            let mut row = serde_json::to_value(work).unwrap_or(Value::Null);
            if let Some(object) = row.as_object_mut() {
                object.insert(
                    "stage".to_string(),
                    run.and_then(run_stage_view).unwrap_or(Value::Null),
                );
                object.insert(
                    "resolved_backend".to_string(),
                    run.and_then(|r| r.backend.clone())
                        .map_or(Value::Null, Value::String),
                );
            }
            row
        })
        .collect();
    json!({"works": works})
}

/// `GET /v1/work` — list all work (ULID key order = submission order).
async fn list_work(State(state): State<ApiState>) -> Response {
    let core = state.core.lock().await;
    Json(fleet_body(&core)).into_response()
}

/// `GET /v1/work/{id}` — one work record, with its stage, surface and
/// execution state (the M3 contract's `work show` surface).
async fn show_work(State(state): State<ApiState>, Path(id): Path<String>) -> Response {
    let core = state.core.lock().await;
    match core.registry.state().works.get(&id) {
        Some(_) => Json(work_view(&core, &id)).into_response(),
        None => error_response(
            StatusCode::NOT_FOUND,
            "work_not_found",
            format!("no work with id {id}"),
        ),
    }
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
    let mut core = state.core.lock().await;
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
            drop(core);
            crank(&state, step).await;
            core = state.core.lock().await;
        }
        Err(e) => tracing::warn!(work_id = %id, error = %e, "retiring the canceled run failed"),
    }
    let result = work_view(&core, &id);
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
    let mut core = state.core.lock().await;
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
            drop(core);
            let mut core = relock(&state, crank(&state, step).await).await;
            let result = work_view(&core, &id);
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
    let mut core = state.core.lock().await;
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
            drop(core);
            let mut core = relock(&state, crank(&state, step).await).await;
            let result = work_view(&core, &id);
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
        let pending = match state.core.lock().await.events_after(from) {
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
        let core = state.core.lock().await;
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
/// carry the `from` it already knows (the TUI and the dashboard both do, from
/// the SSE stream's last id) rather than re-asking from 0.
async fn event_history(
    State(state): State<ApiState>,
    query: Result<Query<EventsQuery>, QueryRejection>,
) -> Response {
    let query = match parse_query(query) {
        Ok(q) => q,
        Err(resp) => return *resp,
    };
    let core = state.core.lock().await;
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
        let core = state.core.lock().await;
        core.events_tx.subscribe()
    };
    let mut last_sent = from;
    let history = {
        let core = state.core.lock().await;
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
                    let core = state.core.lock().await;
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
    KIND_STAGE_BLOCKED,
    KIND_STAGE_FAILED,
    KIND_STAGE_CANCELED,
    KIND_EXECUTION_RESERVED,
    KIND_EXECUTION_STARTED,
    KIND_EXECUTION_STOPPED,
    KIND_EXECUTION_ABANDONED,
    KIND_EXECUTION_RECONCILED,
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
// The two client halves of this contract
// ---------------------------------------------------------------------------

/// The read surface the embedded dashboard is allowed to use.
///
/// §30's architectural test — "if the TUI needs a private shortcut, the API is
/// incomplete" — applies to the dashboard too, but the dashboard is served
/// *by* the daemon, so it cannot be held to it by "it only has a socket". This
/// type is how it is held to it anyway: the [`ApiState`] inside is **private**,
/// and every method below returns exactly the body a `/v1` endpoint returns,
/// built by the same function the endpoint uses. `web.rs` therefore cannot
/// reach the core, the engine, the journal or the projection *directly* — not
/// by convention, but because there is no path to them from `web.rs` that the
/// compiler will accept.
///
/// **What the compiler does not decide is this list.** The private field
/// forecloses reaching *around* this type; it says nothing about a method
/// added *to* it, and a `raw_journal_tail` here would hand `web.rs` the
/// journal with every structural test still green (measured — that probe is
/// why this paragraph exists). The list is therefore pinned as data by
/// `t5_the_tui_and_the_dashboard_are_clients_like_any_other`, which reads the
/// `pub fn`s of this block and requires them to be exactly the set below.
/// Adding a method means changing that test, which is where a reviewer is
/// asked whether the new method is a `/v1` body or a shortcut.
///
/// Extending the dashboard therefore means extending the API first, which is
/// the rule §7 states and this milestone is meant to prove.
#[derive(Clone)]
pub struct ApiViews(ApiState);

impl ApiViews {
    /// Wrap a daemon's state as the dashboard's read surface.
    pub fn new(state: ApiState) -> Self {
        Self(state)
    }

    /// The `GET /v1/system` body.
    pub async fn system(&self) -> Value {
        let head = self.0.core.lock().await.registry.last_seq();
        system_body(&self.0, head)
    }

    /// The `GET /v1/work` body.
    pub async fn fleet(&self) -> Value {
        let core = self.0.core.lock().await;
        fleet_body(&core)
    }

    /// The `GET /v1/work/{id}` body, or `None` for an unknown work (the 404
    /// the endpoint would answer with).
    pub async fn work(&self, id: &str) -> Option<Value> {
        let core = self.0.core.lock().await;
        core.registry
            .state()
            .works
            .contains_key(id)
            .then(|| work_view(&core, id))
    }

    /// The `GET /v1/events?work_id=…&limit=…` body, or the structured error
    /// body the endpoint would answer `500` with.
    ///
    /// The `Result` is the point. A journal that cannot be read is not an
    /// empty journal, and a client that cannot tell those apart renders
    /// "nothing happened" over a fault. The HTTP endpoint answers 500 here;
    /// so must the dashboard, or the two clients are not equal (§7).
    pub async fn work_events(&self, work_id: &str, limit: usize) -> Result<Value, Value> {
        let query = EventsQuery {
            from: 0,
            work_id: Some(work_id.to_string()),
            limit: Some(limit),
        };
        let core = self.0.core.lock().await;
        match core.events_after(0) {
            Ok(events) => Ok(events_body(events, &query)),
            Err(e) => Err(error_body("internal", e.to_string())),
        }
    }
}

// ---------------------------------------------------------------------------
// Shared view helpers
// ---------------------------------------------------------------------------
//
// Both clients project the same `/v1` bodies onto a screen, so the small rules
// for *how a JSON field reads as text* belong to the API surface they share
// rather than to either screen. Two copies of "a missing value is `-`" is two
// chances for the TUI and the dashboard to tell a different story about the
// same work.

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

    /// The dashboard URL, token included (§29's `sgt web` handoff).
    pub fn dashboard_url(&self) -> String {
        format!(
            "{}/ui?{TOKEN_QUERY_PARAM}={}",
            self.endpoint,
            urlencode(&self.token)
        )
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
/// (work ids and the bearer token, both Crockford base32 today). Anything
/// outside the unreserved set is escaped rather than trusted.
///
/// Public because the dashboard builds URLs into this same API and must
/// escape them by the same rule; a second copy of an escaping rule is a
/// second rule.
pub fn urlencode(value: &str) -> String {
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

impl EventStream {
    /// The next journal event, or `None` once the stream ends.
    pub async fn next_event(&mut self) -> Option<Event> {
        loop {
            while let Some(frame) = take_frame(&mut self.pending) {
                if let Some(event) = decode_frame(&frame) {
                    return Some(event);
                }
            }
            let chunk = self.response.chunk().await.ok()??;
            self.pending.push_str(&String::from_utf8_lossy(&chunk));
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

/// Decode one SSE frame's `data:` lines into an [`Event`].
fn decode_frame(frame: &str) -> Option<Event> {
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
        return None;
    }
    serde_json::from_str(&data).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::BackendRegistry;
    use crate::runtime::projection::work_registry_projection;

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

    async fn test_state(data_dir: &std::path::Path) -> ApiState {
        let journal = Journal::open(data_dir).expect("open journal");
        let mut registry = work_registry_projection();
        registry
            .catch_up(journal.replay().expect("replay"))
            .expect("catch up registry");
        let (events_tx, _) = broadcast::channel(16);
        let core = Core {
            journal,
            registry,
            events_tx,
        };
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
            let mut core = state.core.lock().await;
            for n in 1..=3u32 {
                core.commit(seeded(n)).expect("commit");
            }
        }
        let pending = state
            .core
            .lock()
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
            let mut core = state.core.lock().await;
            for n in 4..=5u32 {
                core.commit(seeded(n)).expect("commit");
            }
        }

        // Hold the core lock so the spawned request is guaranteed to be
        // parked between its `last_seq` read and its `events_after` fetch
        // (both uncontended locks resolve without yielding, so it reaches
        // exactly that point on its first poll) when the concurrent
        // failure below runs.
        let core_guard = state.core.lock().await;

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
}
