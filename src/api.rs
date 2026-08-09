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
use axum::http::{StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::{broadcast, mpsc, watch};
use tokio_stream::wrappers::ReceiverStream;

use crate::domain::event::{Event, EventDraft, EventSource, rfc3339_utc_now};
use crate::domain::work::{
    KIND_COMMAND_ACCEPTED, KIND_COMMAND_REJECTED, KIND_WORK_CANCELED, KIND_WORK_SUBMITTED, Work,
    WorkState,
};
use crate::runtime::analytics::{Analytics, AnalyticsError, CANNED_QUERIES};
use crate::runtime::engine::{Engine, EngineError, SubmitContext};
use crate::runtime::journal::{Journal, JournalError};
use crate::runtime::projection::{Projection, ProjectionError, WorkRegistry, WorkRun};

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

/// Build the axum router for the full v1 surface.
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

/// Bearer-token gate for `/v1/*`. `/healthz` is mounted outside this layer.
async fn require_bearer(State(state): State<ApiState>, req: Request, next: Next) -> Response {
    let presented = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    if presented == Some(state.token.as_str()) {
        next.run(req).await
    } else {
        error_response(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "missing or invalid bearer token",
        )
    }
}

/// `GET /healthz` — liveness, unauthenticated.
async fn healthz() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

/// `GET /v1/system` — version, API revision, data dir.
async fn system_info(State(state): State<ApiState>) -> Json<Value> {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "api_revision": API_REVISION,
        "data_dir": state.data_dir,
    }))
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
        if let Err(e) = state.engine.start(&mut core, &work, &plan) {
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
    }))
}

/// `GET /v1/work` — list all work (ULID key order = submission order).
async fn list_work(State(state): State<ApiState>) -> Response {
    let core = state.core.lock().await;
    let works: Vec<&Work> = core.registry.state().works.values().collect();
    Json(json!({"works": works})).into_response()
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
    if let Err(e) = state.engine.retire_run(&mut core, &id, "work canceled") {
        tracing::warn!(work_id = %id, error = %e, "retiring the canceled run failed");
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
    match engine.provide_input(&mut core, &id, &req.input) {
        Ok(()) => {
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
    match engine.retry(&mut core, &id) {
        Ok(()) => {
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

#[derive(Debug, Deserialize)]
struct EventsQuery {
    /// Return events with seq strictly greater than this (default 0 = all).
    #[serde(default)]
    from: u64,
}

/// `GET /v1/events?from=N` — journaled history after seq N.
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
        Ok(events) => Json(json!({"events": events})).into_response(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::BackendRegistry;
    use crate::runtime::projection::work_registry_projection;

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
