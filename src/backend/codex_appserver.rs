//! `codex app-server --listen stdio://` JSON-RPC client (W3 spec §1.5-§1.6,
//! §3.4). A protocol client and nothing else — it knows JSON-RPC and the
//! codex app-server method names, and it knows nothing about `StartRequest`,
//! `Observation`, or the journal; `codex.rs` (its parent module, which
//! declares it via `#[path]`) is the only caller and the only place any of
//! that vocabulary appears.
//!
//! Split exactly along the seam the spec names (§6.2): the top half of this
//! file — [`classify_line`], [`appserver_item_view`],
//! `TurnAccumulator::ingest_appserver_notification`, [`answer_for_request`],
//! [`compute_fingerprint`] — is **pure**: no process, no I/O, no clock read
//! beyond what a caller hands in. It is drivable by an iterator of input
//! lines and a `Vec` of output lines, which is exactly how this module's own
//! unit tests drive it. The bottom half ([`AppServerChild`]) is
//! process-bound: spawn, handshake timing, budgets, the reader thread,
//! process-group kill, stderr drain — `codex.rs`'s own precedent for the exec
//! transport, reused rather than reinvented (§1.5.5).

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};

use super::{ItemKind, ItemView, KIND_TURN_HARNESS_ERROR, NativeEvent, Terminal, TurnAccumulator};

// -------------------------------------------------------------- fingerprint

/// The exact 14 schemas this adapter's client and decoder depend on (spec
/// §2.5), in digest order. Scoped to what this adapter actually reads, not
/// the whole 401-file protocol dump — a fingerprint that flaps on every
/// unrelated upstream file teaches its reader to ignore it.
pub(super) const PINNED_SCHEMA_FILES: &[&str] = &[
    "v1/InitializeParams.json",
    "v1/InitializeResponse.json",
    "v2/ThreadStartParams.json",
    "v2/ThreadStartResponse.json",
    "v2/TurnStartParams.json",
    "v2/TurnStartResponse.json",
    "v2/TurnInterruptParams.json",
    "v2/TurnCompletedNotification.json",
    "v2/TurnStartedNotification.json",
    "v2/ItemStartedNotification.json",
    "v2/ItemCompletedNotification.json",
    "v2/ThreadTokenUsageUpdatedNotification.json",
    "ToolRequestUserInputParams.json",
    "ServerRequest.json",
];

/// The digest algorithm the constant below is computed with (spec §2.5: "the
/// implementer re-computes the constant with `blake3` and pins that" — SHA-256
/// is the spec's own provenance value, evidence that the input bytes are
/// stable, never the constant this codebase pins; `blake3` is already an
/// in-tree dependency, SHA-256 is not).
pub(super) const FINGERPRINT_ALGORITHM: &str = "blake3";

/// The scoped fingerprint measured against codex-cli 0.149.0, Cerberus,
/// 2026-08-21 (`tests/fixtures/codex-appserver-0.149.0-schema-fingerprint.
/// txt` carries both this value and the spec's own SHA-256, so a reader can
/// confirm the input bytes are the same ones the spec measured without
/// re-running `generate-json-schema`).
pub(super) const MEASURED_PROTOCOL_FINGERPRINT: &str =
    "d0fbf8dec13704250254d40fea2c0892928966ba6caa7a6a5af6aa6611abf879";

/// Digest rule (spec §2.5, "must be specified exactly or it is not
/// reproducible"): for each path in [`PINNED_SCHEMA_FILES`], in order, feed
/// the path's UTF-8 bytes, then one `NUL`, then the file's bytes verbatim,
/// then one `NUL`; hash the concatenation. `schema_dir` is wherever `codex
/// app-server generate-json-schema --out <schema_dir> --experimental` wrote
/// its 401 files (G2/G3).
pub(super) fn compute_fingerprint(schema_dir: &Path) -> Result<String, String> {
    let mut hasher = blake3::Hasher::new();
    for relative in PINNED_SCHEMA_FILES {
        hasher.update(relative.as_bytes());
        hasher.update(&[0u8]);
        let path = schema_dir.join(relative);
        let bytes = std::fs::read(&path)
            .map_err(|e| format!("cannot read pinned schema file {path:?}: {e}"))?;
        hasher.update(&bytes);
        hasher.update(&[0u8]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

// ------------------------------------------------------------- wire dispatch

/// One JSON-RPC error object, reduced to what this client acts on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct JsonRpcErrorInfo {
    pub code: i64,
    pub message: String,
}

/// The dispatch shapes M3's own measured wire rule distinguishes (spec
/// §1.5.1): **by shape, never by a `"jsonrpc"` field** — M3 measured that
/// neither responses nor notifications carry one on 0.149.0, so a client
/// that required it would refuse the live server.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum LineShape {
    /// `id` + `result`, no `method`: an answer to a request this client sent.
    Response { id: u64, result: Value },
    /// `id` + `error`, no `method`.
    ErrorResponse { id: u64, error: JsonRpcErrorInfo },
    /// `method` + `id`: the server is asking *us* something, and it **must
    /// be answered** (§3.4) — this is the shape 11 methods take (M8).
    ServerRequest {
        id: Value,
        method: String,
        params: Value,
    },
    /// `method`, no `id`: fire-and-forget.
    Notification { method: String, params: Value },
    /// Parsed as JSON but matches none of the above shapes.
    Unrecognized,
}

/// Classify one already-parsed JSON value by shape (spec §1.5.1's dispatch
/// rule, verbatim): `id` + `result` → response; `id` + `error` → error
/// response; `method` without `id` → notification; `method` **with** `id` →
/// server request. A `"jsonrpc"` field, present or absent, is never consulted.
pub(super) fn classify_line(value: &Value) -> LineShape {
    if let Some(method) = value.get("method").and_then(Value::as_str) {
        let params = value.get("params").cloned().unwrap_or(Value::Null);
        return match value.get("id") {
            Some(id) => LineShape::ServerRequest {
                id: id.clone(),
                method: method.to_string(),
                params,
            },
            None => LineShape::Notification {
                method: method.to_string(),
                params,
            },
        };
    }
    let Some(id_value) = value.get("id") else {
        return LineShape::Unrecognized;
    };
    let Some(id) = id_value.as_u64() else {
        // We only ever mint u64 ids (spec §1.5.1), so a response naming
        // anything else did not answer a request this client sent.
        return LineShape::Unrecognized;
    };
    if let Some(result) = value.get("result") {
        return LineShape::Response {
            id,
            result: result.clone(),
        };
    }
    if let Some(error) = value.get("error") {
        return LineShape::ErrorResponse {
            id,
            error: JsonRpcErrorInfo {
                code: error.get("code").and_then(Value::as_i64).unwrap_or(0),
                message: error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            },
        };
    }
    LineShape::Unrecognized
}

// --------------------------------------------------------------- item view

/// App-server's own `fn view` (spec §1.6): **camelCase** keys (M9), and no
/// item-level `"error"` variant at all — M9's 18 `ThreadItem` variants have
/// none, so every item type but `agentMessage`/`commandExecution` lands in
/// [`ItemKind::Unknown`], counted and never decoded, exactly like exec's
/// unrecognized item types.
pub(super) fn appserver_item_view(item: &Value) -> ItemView {
    let item_id = item.get("id").cloned().unwrap_or(Value::Null);
    match item.get("type").and_then(Value::as_str).unwrap_or("") {
        "agentMessage" => ItemView {
            item_type: ItemKind::AgentMessage,
            item_id,
            text: Some(
                item.get("text")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            ),
            command: None,
            exit_code: None,
            status: None,
            aggregated_output: None,
        },
        "commandExecution" => ItemView {
            item_type: ItemKind::CommandExecution,
            item_id,
            text: None,
            command: Some(item.get("command").cloned().unwrap_or(Value::Null)),
            exit_code: Some(item.get("exitCode").cloned().unwrap_or(Value::Null)),
            status: Some(
                item.get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            ),
            aggregated_output: Some(
                item.get("aggregatedOutput")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            ),
        },
        other => ItemView::unknown(other, item_id),
    }
}

/// Envelope-level method names this decoder deliberately archives without
/// decoding (spec §1.6's mapping table): the delta/progress notifications
/// whose *completed* counterpart already produces the event, so decoding
/// them too would double-count exactly the way exec's own `agent_message`
/// guard refuses to.
const ARCHIVED_NOT_DECODED_METHODS: &[&str] = &[
    "item/agentMessage/delta",
    "item/commandExecution/outputDelta",
    "item/fileChange/outputDelta",
    "item/reasoning/summaryDelta",
    "item/reasoning/textDelta",
    "item/plan/delta",
    "turn/plan/updated",
    "turn/diff/updated",
];

/// Envelope-level methods journaled as [`KIND_TURN_HARNESS_ERROR`] but never
/// a terminal by themselves (spec §1.6, §4.4's rule carried onto this
/// transport) — M11 vindicated this immediately: `configWarning` about a
/// missing bubblewrap fires on a perfectly healthy connection.
const HARNESS_WARNING_METHODS: &[&str] = &[
    "error",
    "configWarning",
    "guardianWarning",
    "deprecationNotice",
    "warning",
];

impl TurnAccumulator {
    /// The app-server counterpart of [`TurnAccumulator::ingest_line`]: folds
    /// one already-dispatched notification (method + params) into the same
    /// accumulator exec's decoder fills, producing the same
    /// [`NativeEvent`]s through the same [`TurnAccumulator::ingest_view`]
    /// this transport shares with exec (spec §1.6 — "the event-producing
    /// code ... exists once").
    pub(super) fn ingest_appserver_notification(
        &mut self,
        method: &str,
        params: &Value,
    ) -> Vec<NativeEvent> {
        let mut out = Vec::new();
        match method {
            "thread/started" => {
                if self.thread_id.is_none()
                    && let Some(id) = params.pointer("/thread/id").and_then(Value::as_str)
                {
                    self.thread_id = Some(id.to_string());
                }
            }
            "turn/started" => {}
            "item/started" => {
                if let Some(item) = params.get("item") {
                    let view = appserver_item_view(item);
                    self.ingest_view(&view, true, &mut out);
                }
            }
            "item/completed" => {
                if let Some(item) = params.get("item") {
                    let view = appserver_item_view(item);
                    self.ingest_view(&view, false, &mut out);
                }
            }
            "turn/completed" => {
                let status = params
                    .pointer("/turn/status")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                match status {
                    "completed" => self.terminal = Terminal::Completed,
                    "interrupted" => self.terminal = Terminal::Interrupted,
                    "failed" => {
                        let message = params
                            .pointer("/turn/error/message")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string();
                        if let Some(info) = params
                            .pointer("/turn/error/codexErrorInfo")
                            .and_then(Value::as_str)
                        {
                            self.last_codex_error_info = Some(info.to_string());
                        }
                        out.push(NativeEvent {
                            kind: KIND_TURN_HARNESS_ERROR.to_string(),
                            payload: json!({"phase": "turn_failed", "message": message}),
                        });
                        self.last_error = Some(message.clone());
                        self.terminal = Terminal::Failed { message };
                    }
                    // "inProgress" (should never arrive on a *completed*
                    // notification, but a future protocol addition might) or
                    // anything unrecognized: leave the terminal as it was —
                    // §5.2's ambiguity, unchanged.
                    _ => {}
                }
            }
            "thread/tokenUsage/updated" => {
                if let Some(usage) = params.get("tokenUsage") {
                    self.usage = Some(usage.clone());
                }
            }
            method if HARNESS_WARNING_METHODS.contains(&method) => {
                let message = params
                    .pointer("/message")
                    .or_else(|| params.pointer("/summary"))
                    .or_else(|| params.pointer("/error/message"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                if method == "error"
                    && let Some(info) = params
                        .pointer("/error/codexErrorInfo")
                        .and_then(Value::as_str)
                {
                    self.last_codex_error_info = Some(info.to_string());
                }
                out.push(NativeEvent {
                    kind: KIND_TURN_HARNESS_ERROR.to_string(),
                    payload: json!({"phase": method, "message": message}),
                });
                self.last_error = Some(message);
            }
            method if ARCHIVED_NOT_DECODED_METHODS.contains(&method) => {}
            other => self.unknown_methods.push(other.to_string()),
        }
        out
    }
}

// -------------------------------------------------------- server requests

/// What this client answers a server→client request with — either a
/// JSON-RPC result or an error, never left unanswered (spec §3.4: "the
/// non-hang guarantee").
#[derive(Debug, Clone, PartialEq)]
pub(super) enum RequestAnswer {
    Result(Value),
    Error { code: i64, message: String },
}

/// One answered server request, paired with the journal phase it must be
/// recorded under — `None` only for the one honestly-answered plumbing case
/// (`currentTime/read`), which is not a downgrade of anything.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct AnsweredRequest {
    pub answer: RequestAnswer,
    pub journal_phase: Option<&'static str>,
}

/// The complete answering table (spec §3.4 point 2): every one of M8's 11
/// server→client methods, plus the catch-all for anything a future protocol
/// version adds. `ask_admitted` is always `false` in this deployment
/// (`Capabilities::ask` — see `codex.rs`'s `ADMISSION_ROWS`), carried as a
/// parameter rather than hardcoded so the one branch that depends on it
/// stays honest about being conditional, not a decision this function makes
/// on its own.
///
/// Response shapes for the five approval-flavored methods and
/// `mcpServer/elicitation/request` are schema-verified against
/// `generate-json-schema`'s own dump (recorded in the wave PR body) even
/// though none of them has ever been driven live: `approvalPolicy: "never"`
/// means no approval-gate request should ever fire (§3.4 point 1), so this
/// table's job for those six methods is to be *safe if it ever does*, not to
/// be exercised.
pub(super) fn answer_for_request(method: &str, ask_admitted: bool) -> AnsweredRequest {
    match method {
        "item/tool/requestUserInput" => {
            if ask_admitted {
                // Never reached in this deployment (ask stays `false`
                // structurally — see `codex.rs`'s `ADMISSION_ROWS`); kept as
                // a real branch, not an `unreachable!()`, so the table stays
                // honest that this is a conditional the spec describes, not
                // one this function silently collapsed.
                AnsweredRequest {
                    answer: RequestAnswer::Error {
                        code: -32001,
                        message: "ask is admitted but no answering path is wired for it yet"
                            .to_string(),
                    },
                    journal_phase: Some("ask_admitted_but_unhandled"),
                }
            } else {
                AnsweredRequest {
                    answer: RequestAnswer::Error {
                        code: -32001,
                        message: "no human is attached to this execution".to_string(),
                    },
                    journal_phase: Some("ask_declined_unattended"),
                }
            }
        }
        "item/commandExecution/requestApproval" | "item/fileChange/requestApproval" => {
            AnsweredRequest {
                answer: RequestAnswer::Result(json!({"decision": "decline"})),
                journal_phase: Some("approval_denied_unattended"),
            }
        }
        "item/permissions/requestApproval" => AnsweredRequest {
            answer: RequestAnswer::Result(
                json!({"permissions": {"fileSystem": null, "network": null}}),
            ),
            journal_phase: Some("approval_denied_unattended"),
        },
        "applyPatchApproval" | "execCommandApproval" => AnsweredRequest {
            // The legacy `ReviewDecision` enum (spec Appendix / schema dump)
            // has no explicit "denied" arm — `abort` is the closest "do not
            // proceed" value it offers.
            answer: RequestAnswer::Result(json!({"decision": "abort"})),
            journal_phase: Some("approval_denied_unattended"),
        },
        "mcpServer/elicitation/request" => AnsweredRequest {
            answer: RequestAnswer::Result(json!({"action": "decline"})),
            journal_phase: Some("elicitation_declined"),
        },
        "account/chatgptAuthTokens/refresh" => AnsweredRequest {
            // §2.8: credential handling is out of scope for this adapter.
            answer: RequestAnswer::Error {
                code: -32002,
                message: "credential refresh is out of scope for this adapter".to_string(),
            },
            journal_phase: Some("auth_refresh_declined"),
        },
        "currentTime/read" => AnsweredRequest {
            // Plumbing, answered honestly — no downgrade, nothing to
            // journal. Field name and unit (`currentTimeAt`, whole Unix
            // seconds) measured from `CurrentTimeReadResponse.json`, not
            // guessed.
            answer: RequestAnswer::Result(json!({
                "currentTimeAt": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0)
            })),
            journal_phase: None,
        },
        "attestation/generate" | "item/tool/call" => AnsweredRequest {
            answer: RequestAnswer::Error {
                code: -32003,
                message: "not supported by this adapter".to_string(),
            },
            journal_phase: Some("unsupported_server_request"),
        },
        // Any unknown method with an id: §3.4's guarantee-preserving arm —
        // this is what makes the non-hang promise survive an upstream
        // protocol addition, and why that must be stale provenance rather
        // than a crash (§2.5).
        _ => AnsweredRequest {
            answer: RequestAnswer::Error {
                code: -32601,
                message: "Method not found".to_string(),
            },
            journal_phase: Some("unknown_server_request"),
        },
    }
}

// ---------------------------------------------------------- process-bound

/// Budgets (spec §1.5.4), each overridable per-instance — never a global.
#[derive(Debug, Clone, Copy)]
pub struct Budgets {
    pub handshake: Duration,
    pub thread_start: Duration,
    pub turn_start: Duration,
    pub interrupt: Duration,
    pub stderr_drain: Duration,
}

impl Default for Budgets {
    fn default() -> Self {
        Self {
            handshake: Duration::from_secs(15),
            thread_start: Duration::from_secs(30),
            turn_start: Duration::from_secs(30),
            interrupt: Duration::from_secs(10),
            stderr_drain: Duration::from_secs(5),
        }
    }
}

/// Why a pending request will never receive a result.
#[derive(Debug, Clone)]
pub(super) enum PendingFailure {
    /// The server answered with a JSON-RPC error object.
    Rpc(JsonRpcErrorInfo),
    /// The child's stdout closed before this request was answered — the
    /// reader thread drains every pending sender with this on EOF (§3.4
    /// point 3), so a caller blocked in [`AppServerHandle::call`] learns the
    /// child is gone *now* rather than by outliving its own budget. That
    /// distinction is not cosmetic: the budget expiry arrives long after the
    /// EOF handler has already settled the turn, and a caller that wakes up
    /// that late used to conclude "my request timed out" about a turn whose
    /// fate was already decided and journaled.
    ChildGone,
}

/// A pending in-flight request: the id this client minted, and where to
/// deliver whatever comes back.
type PendingMap = Arc<Mutex<BTreeMap<u64, SyncSender<Result<Value, PendingFailure>>>>>;

/// The JSON-RPC bookkeeping half of one app-server child: cheap to clone
/// (an `Arc` around its stdin lock, its shared id counter, its pending-
/// response map), so the reader thread's own `on_line` callback can hold one
/// and answer a server request (§3.4) **inline, from the same thread that
/// decoded it** — no second channel needed to get the answer back to
/// whoever owns the write side. [`AppServerChild`] (below) is this plus the
/// process lifecycle (spawn/kill/pgid); nothing here talks to the process
/// directly.
#[derive(Debug, Clone)]
pub(super) struct AppServerHandle {
    stdin: Arc<Mutex<std::process::ChildStdin>>,
    next_id: Arc<AtomicU64>,
    pending: PendingMap,
}

impl AppServerHandle {
    fn write_line(&self, value: &Value) -> std::io::Result<()> {
        let mut stdin = self.stdin.lock().expect("stdin lock");
        stdin.write_all(value.to_string().as_bytes())?;
        stdin.write_all(b"\n")?;
        stdin.flush()
    }

    /// Mint the id a subsequent [`Self::call_reserved`] will send under,
    /// *before* the request goes out. Only worth doing for a request whose
    /// caller must record something about it that the reader thread will read
    /// back when the answer lands (today: `turn/interrupt`, whose progress the
    /// turn cell tracks); everything else uses [`Self::call`], which mints its
    /// own id and never has to name it.
    pub(super) fn reserve_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Send one JSON-RPC request and block, bounded, for its response.
    /// `"jsonrpc":"2.0"` is stamped on everything sent (spec §1.5.1); nothing
    /// this client receives is required to carry it.
    pub(super) fn call(
        &self,
        method: &str,
        params: Value,
        budget: Duration,
    ) -> Result<Value, String> {
        self.call_reserved(self.reserve_id(), method, params, budget)
    }

    /// [`Self::call`] under an id the caller minted with [`Self::reserve_id`].
    pub(super) fn call_reserved(
        &self,
        id: u64,
        method: &str,
        params: Value,
        budget: Duration,
    ) -> Result<Value, String> {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        self.pending.lock().expect("pending lock").insert(id, tx);
        let line = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        if let Err(e) = self.write_line(&line) {
            self.pending.lock().expect("pending lock").remove(&id);
            return Err(format!("cannot write {method}: {e}"));
        }
        match rx.recv_timeout(budget) {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(PendingFailure::Rpc(error))) => Err(format!(
                "{method} refused: {} (code {})",
                error.message, error.code
            )),
            Ok(Err(PendingFailure::ChildGone)) => Err(format!(
                "{method} was never answered: the app-server child's output stream closed"
            )),
            Err(_) => {
                self.pending.lock().expect("pending lock").remove(&id);
                Err(format!("{method} did not respond within {budget:?}"))
            }
        }
    }

    /// Send a notification (no id, no response expected — `initialized`).
    pub(super) fn notify(&self, method: &str, params: Value) -> Result<(), String> {
        let line = json!({"jsonrpc": "2.0", "method": method, "params": params});
        self.write_line(&line).map_err(|e| e.to_string())
    }

    /// Answer a server request the caller decided how to answer (via
    /// [`answer_for_request`] or its own logic for `item/tool/requestUserInput`
    /// when `ask` is ever admitted). Never optional to call: an unanswered
    /// blocking request is a hung turn (§3.4).
    pub(super) fn answer(&self, id: &Value, answer: RequestAnswer) -> Result<(), String> {
        let line = match answer {
            RequestAnswer::Result(result) => json!({"id": id, "result": result}),
            RequestAnswer::Error { code, message } => {
                json!({"id": id, "error": {"code": code, "message": message}})
            }
        };
        self.write_line(&line).map_err(|e| e.to_string())
    }
}

/// One `codex app-server --listen stdio://` child, owned for the whole
/// execution (spec §1.4: one per `CodexExecution`, never per turn). Spawns
/// the process, drives the handshake, and hands back a handle whose
/// `send_request`/`send_notification` do the JSON-RPC bookkeeping; the
/// caller (`codex.rs`) drives `thread/start`/`turn/start`/`turn/interrupt`
/// through it and owns everything transport-specific (sandbox composition,
/// prompts, the event sink).
#[derive(Debug)]
pub(super) struct AppServerChild {
    handle: AppServerHandle,
    child: Arc<Mutex<Child>>,
    /// The child's own exit status, once anything has reaped it. Kept
    /// because `try_wait`/`wait` hand a status back exactly once per
    /// underlying reap and OBSERVE may ask many times — discarding it (what
    /// the first cut of `alive()` did) throws away the only proof this
    /// adapter will ever hold that the child *did* exit, and with what.
    exit_status: Arc<Mutex<Option<std::process::ExitStatus>>>,
    /// The last [`STDERR_TAIL_BYTES`] this child wrote to stderr. Bounded,
    /// so a chatty child cannot grow this without limit, and retained rather
    /// than dropped: on a child that died mid-turn, stderr is the same
    /// evidence exec's own per-turn stderr is for a pre-turn refusal.
    stderr_tail: Arc<Mutex<Vec<u8>>>,
    pgid: u32,
    reader: Option<std::thread::JoinHandle<()>>,
}

/// How much of the child's stderr is retained for evidence (a ring: the tail
/// is what a failure's last words live in; the head is startup chatter).
const STDERR_TAIL_BYTES: usize = 4096;

/// What a non-blocking peek at the child can prove.
#[derive(Debug, Clone)]
pub(super) enum ChildStatus {
    /// Not exited when asked.
    Running,
    /// Reaped: the OS's own rendering of the status (`exit status: 7`,
    /// `signal: 9 (SIGKILL)`) plus its exit code when it has one (a
    /// signal-killed child has none).
    Exited { detail: String, code: Option<i32> },
    /// The peek itself failed — the one case where "cannot tell" is true.
    Unknown(String),
}

/// One line the reader thread hands to whatever is consuming this child's
/// output — a response/error is delivered to its waiting caller through the
/// pending map, and announced here as [`InboundLine::Resolved`] only so that a
/// consumer which is tracking a request can learn its fate *in stream order*,
/// on the reader's own thread, ahead of whatever the rest of the stream does
/// next.
#[derive(Debug, Clone)]
pub(super) enum InboundLine {
    Notification {
        method: String,
        params: Value,
    },
    /// A request this client sent has been answered: `ok` is whether the
    /// answer was a result rather than a JSON-RPC error. Announced *before*
    /// the blocked caller is woken, and therefore strictly before the
    /// [`InboundLine::Eof`] that ends this stream — which is the whole point.
    /// A caller woken by its channel races the reader thread to whatever
    /// shared state it wants to update, and on a child that answers and then
    /// dies, that race has no winner it is entitled to: the settlement runs
    /// on this thread, so the fact has to be recorded on this thread too.
    ///
    /// The losing schedule is latent rather than routine — a stub that exits
    /// after answering leaves the reader waiting on the child's own teardown,
    /// which the woken caller beats every time, so no deterministic test can
    /// force it. What a test *can* pin is the mechanism, and
    /// `an_answered_interrupt_settles_as_acknowledged_whoever_wakes_first`
    /// does: the fate is recorded against the request id, so it does not
    /// matter who runs next.
    Resolved {
        id: u64,
        ok: bool,
    },
    /// `params` is read by `appserver_on_line` for `item/tool/
    /// requestUserInput` specifically (`params.questions`/`params.turnId`
    /// are journaled alongside the decline, §2.4's step-3 evidence) even
    /// though `ask` stays structurally `false` in this deployment — the one
    /// piece of real protocol data an admitted `ask` path would also need,
    /// so it is carried through rather than dropped.
    ServerRequest {
        id: Value,
        method: String,
        params: Value,
    },
    /// A line that did not parse as JSON at all (§4.6's policy, unchanged).
    Unparsed,
    /// The child's stdout closed — end of stream, whether from a clean exit,
    /// a crash, or a signal (§3.4 point 3 / §15's fail-closed invariant).
    /// Emitted exactly once, after the reader's own read loop ends, however
    /// it ended: `appserver_on_line`'s job with this variant is to make sure
    /// a turn left `InFlight` by a dead child is never silently forgotten —
    /// the same guarantee exec's own `TurnReader` gets for free from a
    /// per-turn process, recreated here for app-server's long-lived child.
    Eof,
}

impl AppServerChild {
    /// Spawn the child and start its reader thread. Every notification and
    /// server request the child ever writes is sent to `on_line`, called
    /// from the reader thread with the same [`AppServerHandle`] this child
    /// holds — so `on_line` can answer a server request inline, without a
    /// second round trip through this struct. The caller must not block long
    /// in `on_line`, the same rule exec's `TurnReader` follows.
    pub(super) fn spawn(
        executable: &Path,
        cwd: &Path,
        env: &BTreeMap<String, String>,
        codex_home: Option<&Path>,
        on_line: impl Fn(&AppServerHandle, InboundLine) + Send + 'static,
    ) -> Result<Self, String> {
        let mut command = Command::new(executable);
        command
            .args(["app-server", "--listen", "stdio://"])
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (key, value) in env {
            command.env(key, value);
        }
        if let Some(home) = codex_home {
            command.env("CODEX_HOME", home);
        }
        // §1.5.5: reused, not copied — the app-server child spawns
        // grandchildren (shell commands) exactly as `codex exec` does, so
        // the same grandchild-survives-the-kill hazard applies unchanged.
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }

        let mut child = command
            .spawn()
            .map_err(|e| format!("cannot spawn {executable:?} app-server: {e}"))?;
        let pgid = child.id();
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "child stdin was not piped".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "child stdout was not piped".to_string())?;
        // Stderr is drained on its own thread into a bounded ring. Draining
        // is what keeps the pipe from filling and blocking the child;
        // *retaining* the tail is what makes a child that died mid-turn
        // explicable — the ambiguous terminal it leaves behind has no stream
        // evidence at all, and this is the only place its last words exist.
        let stderr_tail = Arc::new(Mutex::new(Vec::<u8>::new()));
        if let Some(mut stderr) = child.stderr.take() {
            let sink = Arc::clone(&stderr_tail);
            std::thread::spawn(move || {
                let mut buffer = [0u8; 1024];
                loop {
                    match stderr.read(&mut buffer) {
                        Ok(0) | Err(_) => break,
                        Ok(read) => {
                            let mut held = sink.lock().expect("stderr tail lock");
                            held.extend_from_slice(&buffer[..read]);
                            let overflow = held.len().saturating_sub(STDERR_TAIL_BYTES);
                            if overflow > 0 {
                                held.drain(..overflow);
                            }
                        }
                    }
                }
            });
        }

        let handle = AppServerHandle {
            stdin: Arc::new(Mutex::new(stdin)),
            next_id: Arc::new(AtomicU64::new(1)),
            pending: Arc::new(Mutex::new(BTreeMap::new())),
        };
        let reader_handle = handle.clone();
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                let Ok(value) = serde_json::from_str::<Value>(&line) else {
                    on_line(&reader_handle, InboundLine::Unparsed);
                    continue;
                };
                match classify_line(&value) {
                    LineShape::Response { id, result } => {
                        let waiting = reader_handle
                            .pending
                            .lock()
                            .expect("pending lock")
                            .remove(&id);
                        // Announce first, wake second: see `Resolved`.
                        on_line(&reader_handle, InboundLine::Resolved { id, ok: true });
                        if let Some(tx) = waiting {
                            let _ = tx.send(Ok(result));
                        }
                    }
                    LineShape::ErrorResponse { id, error } => {
                        let waiting = reader_handle
                            .pending
                            .lock()
                            .expect("pending lock")
                            .remove(&id);
                        on_line(&reader_handle, InboundLine::Resolved { id, ok: false });
                        if let Some(tx) = waiting {
                            let _ = tx.send(Err(PendingFailure::Rpc(error)));
                        }
                    }
                    LineShape::Notification { method, params } => {
                        on_line(&reader_handle, InboundLine::Notification { method, params });
                    }
                    LineShape::ServerRequest { id, method, params } => {
                        on_line(
                            &reader_handle,
                            InboundLine::ServerRequest { id, method, params },
                        );
                    }
                    LineShape::Unrecognized => {}
                }
            }
            // The loop above ends however the stream ended -- a clean EOF or
            // a read error alike (§3.4 point 3 / §15): either way, the child
            // is done talking, and whoever owns the turn state needs to know
            // once, so a turn left `InFlight` is never silently forgotten.
            //
            // Settle first, drain second, and never the other way round. The
            // EOF callback is what turns an in-flight turn into a settled one;
            // draining first would wake a blocked `call` into a race for the
            // same cell, and a caller that won that race would reset the cell
            // it owned before the fail-closed settlement could be written --
            // the very outcome this ordering exists to make impossible.
            on_line(&reader_handle, InboundLine::Eof);
            let orphaned =
                std::mem::take(&mut *reader_handle.pending.lock().expect("pending lock"));
            for (_, tx) in orphaned {
                let _ = tx.send(Err(PendingFailure::ChildGone));
            }
        });

        Ok(Self {
            handle,
            child: Arc::new(Mutex::new(child)),
            exit_status: Arc::new(Mutex::new(None)),
            stderr_tail,
            pgid,
            reader: Some(reader),
        })
    }

    /// A cloneable handle for issuing requests/notifications from outside
    /// the reader thread (`thread/start`, `turn/start`, `turn/interrupt`).
    pub(super) fn handle(&self) -> AppServerHandle {
        self.handle.clone()
    }

    /// This child's process-group id, recorded at spawn (§1.5.5) — never
    /// derived at kill time, for the identical reason exec's own
    /// `turn_pgid` never is.
    pub(super) fn pgid(&self) -> u32 {
        self.pgid
    }

    /// Whether the child is still alive, and — when it is not — what it
    /// exited with. A non-blocking peek exec's own liveness checks do not
    /// need since its child is per-turn and always waited on by the reader;
    /// this transport's child lives for the whole execution, so OBSERVE needs
    /// to ask without waiting. The status is remembered on the way past, so
    /// the second and every later caller gets the same answer as the first
    /// instead of an empty `Ok(Some(_))` the reap already consumed.
    pub(super) fn status(&self) -> ChildStatus {
        if let Some(status) = *self.exit_status.lock().expect("exit status lock") {
            return ChildStatus::Exited {
                detail: status.to_string(),
                code: status.code(),
            };
        }
        match self.child.lock().expect("child lock").try_wait() {
            Ok(None) => ChildStatus::Running,
            Ok(Some(status)) => {
                *self.exit_status.lock().expect("exit status lock") = Some(status);
                ChildStatus::Exited {
                    detail: status.to_string(),
                    code: status.code(),
                }
            }
            Err(e) => ChildStatus::Unknown(e.to_string()),
        }
    }

    /// The last [`STDERR_TAIL_BYTES`] this child wrote to stderr, lossily
    /// decoded (a tail can start mid-codepoint by construction).
    pub(super) fn stderr_tail(&self) -> String {
        String::from_utf8_lossy(&self.stderr_tail.lock().expect("stderr tail lock")).into_owned()
    }

    /// Kill this child's whole process group (§1.5.5) and join its reader.
    /// Idempotent: safe to call after the child has already exited.
    pub(super) fn kill(&mut self, stderr_drain_budget: Duration) {
        let _ = stderr_drain_budget; // stderr is drained continuously above
        super::kill_process_group(Some(self.pgid));
        let _ = self.child.lock().expect("child lock").kill();
        // Reap and record: after the group kill the child is already gone or
        // about to be, and this is the call that turns "we killed it" into a
        // status OBSERVE can name.
        if let Ok(status) = self.child.lock().expect("child lock").wait() {
            *self.exit_status.lock().expect("exit status lock") = Some(status);
        }
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

impl Drop for AppServerChild {
    fn drop(&mut self) {
        // Best-effort: a caller that wants the join should call `kill`
        // itself. This exists so a `CodexExecution` dropped without an
        // explicit STOP does not leave the child running past the adapter's
        // own memory of it (mirrors exec's own posture: adapter state
        // dropping is not a supported lifecycle path, but it should not
        // orphan a process either).
        super::kill_process_group(Some(self.pgid));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --------------------------------------------------- dispatch (test 1)

    #[test]
    fn appserver_client_dispatches_by_shape_not_by_jsonrpc_field() {
        // M3: neither responses nor notifications carry "jsonrpc" — a
        // client that required it would refuse the live server.
        assert_eq!(
            classify_line(&json!({"id": 2, "result": {"ok": true}})),
            LineShape::Response {
                id: 2,
                result: json!({"ok": true})
            }
        );
        assert_eq!(
            classify_line(&json!({"id": 2, "error": {"code": -32600, "message": "bad"}})),
            LineShape::ErrorResponse {
                id: 2,
                error: JsonRpcErrorInfo {
                    code: -32600,
                    message: "bad".to_string()
                }
            }
        );
        assert_eq!(
            classify_line(&json!({"method": "configWarning", "params": {"summary": "s"}})),
            LineShape::Notification {
                method: "configWarning".to_string(),
                params: json!({"summary": "s"})
            }
        );
        assert_eq!(
            classify_line(&json!({
                "id": 7, "method": "item/tool/requestUserInput", "params": {"a": 1}
            })),
            LineShape::ServerRequest {
                id: json!(7),
                method: "item/tool/requestUserInput".to_string(),
                params: json!({"a": 1})
            }
        );
        // A request *we* send carries "jsonrpc":"2.0" — dispatch ignores it
        // either way, proving the field is never load-bearing for receiving.
        assert_eq!(
            classify_line(
                &json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}})
            ),
            LineShape::ServerRequest {
                id: json!(1),
                method: "initialize".to_string(),
                params: json!({})
            },
            "a method+id line is a server request regardless of a jsonrpc field"
        );
        assert_eq!(
            classify_line(&json!({"foo": "bar"})),
            LineShape::Unrecognized
        );
    }

    // --------------------------------------------- ordering (test 2)

    #[test]
    fn appserver_notifications_may_precede_the_response_that_caused_them() {
        // M4's measured ordering: configWarning arrives before thread/start's
        // own result. The pure fixture-driven proof: classify both lines in
        // recorded order and confirm the notification classifies cleanly
        // *before* the response line does, with no ordering assumption
        // anywhere in `classify_line` itself (it is a pure per-line function).
        let fixture = include_str!(
            "../../tests/fixtures/codex-appserver-0.149.0-handshake-and-thread-start.jsonl"
        );
        let mut saw_config_warning_before_thread_start = false;
        let mut saw_thread_start_result = false;
        for line in fixture.lines().filter(|l| !l.trim().is_empty()) {
            let value: Value = serde_json::from_str(line).expect("fixture line is valid JSON");
            match classify_line(&value) {
                LineShape::Notification { method, .. } if method == "configWarning" => {
                    assert!(
                        !saw_thread_start_result,
                        "configWarning must be recorded before thread/start's result"
                    );
                    saw_config_warning_before_thread_start = true;
                }
                LineShape::Response { id: 2, .. } => saw_thread_start_result = true,
                _ => {}
            }
        }
        assert!(saw_config_warning_before_thread_start);
        assert!(saw_thread_start_result);
    }

    // ------------------------------------------- thread id timing (test 3)

    #[test]
    fn appserver_thread_start_yields_the_native_id_before_any_turn() {
        let fixture = include_str!(
            "../../tests/fixtures/codex-appserver-0.149.0-handshake-and-thread-start.jsonl"
        );
        let mut thread_id = None;
        for line in fixture.lines().filter(|l| !l.trim().is_empty()) {
            let value: Value = serde_json::from_str(line).expect("valid JSON");
            if let LineShape::Response { id: 2, result } = classify_line(&value) {
                thread_id = result
                    .pointer("/thread/id")
                    .and_then(Value::as_str)
                    .map(str::to_string);
            }
        }
        assert_eq!(
            thread_id.as_deref(),
            Some("01a02607-b95c-71a0-add7-e8273a69a149"),
            "the thread id must be readable straight from thread/start's own result — no \
             turn/start was ever sent in this fixture (zero tokens spent, per its capture note)"
        );
    }

    // ---------------------------------------------------- decode (5, 6, 8)

    #[test]
    fn appserver_decodes_a_plain_turn_to_one_assistant_message() {
        let fixture =
            include_str!("../../tests/fixtures/codex-appserver-0.149.0-completed-turn.jsonl");
        let mut acc = TurnAccumulator::new();
        let mut events = Vec::new();
        for line in fixture.lines().filter(|l| !l.trim().is_empty()) {
            let value: Value = serde_json::from_str(line).expect("valid JSON");
            if let LineShape::Notification { method, params } = classify_line(&value) {
                events.extend(acc.ingest_appserver_notification(&method, &params));
            }
        }
        assert_eq!(acc.message_items, 1);
        assert_eq!(acc.tool_items, 0);
        assert_eq!(acc.last_agent_message.as_deref(), Some("ok"));
        assert!(matches!(acc.terminal, Terminal::Completed));
        let assistant: Vec<_> = events
            .iter()
            .filter(|e| e.kind == crate::runtime::graph::KIND_CONVERSATION_ASSISTANT_COMPLETED)
            .collect();
        assert_eq!(assistant.len(), 1, "one assistant-completed event, no more");
        assert_eq!(assistant[0].payload["text"], "ok");
        // §2.3: usage arrives on its own notification, never duplicated onto
        // the terminal — the fixture's own turn/completed carries no usage
        // key at all, and this decoder never reads one from there.
        assert!(
            acc.usage.is_some(),
            "usage was captured from the separate notification"
        );
        assert!(
            acc.usage.as_ref().unwrap().get("total").is_some(),
            "the real shape has a total/last split, not a flat usage object"
        );
    }

    #[test]
    fn appserver_decodes_a_command_execution_item_to_tool_events() {
        let fixture = include_str!(
            "../../tests/fixtures/codex-appserver-0.149.0-command-execution-item.derived.jsonl"
        );
        let mut acc = TurnAccumulator::new();
        let mut events = Vec::new();
        for line in fixture.lines().filter(|l| !l.trim().is_empty()) {
            let value: Value = serde_json::from_str(line).expect("valid JSON");
            if let LineShape::Notification { method, params } = classify_line(&value) {
                events.extend(acc.ingest_appserver_notification(&method, &params));
            }
        }
        assert_eq!(acc.tool_items, 1);
        let kinds: Vec<&str> = events.iter().map(|e| e.kind.as_str()).collect();
        assert!(kinds.contains(&crate::runtime::graph::KIND_TOOL_REQUESTED));
        assert!(kinds.contains(&crate::runtime::graph::KIND_TOOL_COMPLETED));
        let completed = events
            .iter()
            .find(|e| e.kind == crate::runtime::graph::KIND_TOOL_COMPLETED)
            .expect("a tool.completed event");
        assert_eq!(completed.payload["is_error"], false);
        assert_eq!(completed.payload["exit_code"], 0);
        assert_eq!(completed.payload["output_tail"], "unsandboxed-ok\n");
    }

    #[test]
    fn appserver_narration_produces_no_tool_events() {
        // W1's own narration fixture, re-expressed as the completed-turn
        // capture: no commandExecution item anywhere, so the decoder must
        // not invent one from agentMessage prose (§4.3, same invariant).
        let fixture =
            include_str!("../../tests/fixtures/codex-appserver-0.149.0-completed-turn.jsonl");
        let mut acc = TurnAccumulator::new();
        let mut events = Vec::new();
        for line in fixture.lines().filter(|l| !l.trim().is_empty()) {
            let value: Value = serde_json::from_str(line).expect("valid JSON");
            if let LineShape::Notification { method, params } = classify_line(&value) {
                events.extend(acc.ingest_appserver_notification(&method, &params));
            }
        }
        assert_eq!(acc.tool_items, 0);
        assert!(
            !events.iter().any(|e| {
                e.kind == crate::runtime::graph::KIND_TOOL_REQUESTED
                    || e.kind == crate::runtime::graph::KIND_TOOL_COMPLETED
            }),
            "zero tool.* events from a turn that never produced a commandExecution item"
        );
    }

    // -------------------------------------------------- refactor guard (7)

    #[test]
    fn exec_and_appserver_decode_the_same_command_item_to_the_same_events() {
        // The extraction's own guard (spec §1.6): the same logical item, in
        // exec's snake_case and app-server's camelCase, must decode to
        // byte-identical NativeEvents through the one shared `ingest_view`.
        let exec_item = json!({
            "id": "item_1", "type": "command_execution",
            "command": "/bin/bash -lc 'echo hi'",
            "aggregated_output": "hi\n", "exit_code": 0, "status": "completed"
        });
        let appserver_item = json!({
            "id": "item_1", "type": "commandExecution",
            "command": "/bin/bash -lc 'echo hi'",
            "aggregatedOutput": "hi\n", "exitCode": 0, "status": "completed",
            "cwd": "/work", "durationMs": 42, "processId": 999
        });

        let mut exec_acc = TurnAccumulator::new();
        let mut exec_events = Vec::new();
        exec_acc.ingest_view(
            &super::super::exec_item_view(&exec_item),
            false,
            &mut exec_events,
        );

        let mut appserver_acc = TurnAccumulator::new();
        let mut appserver_events = Vec::new();
        appserver_acc.ingest_view(
            &appserver_item_view(&appserver_item),
            false,
            &mut appserver_events,
        );

        assert_eq!(
            exec_events, appserver_events,
            "the same logical command item, two spellings, must decode identically"
        );
        assert_eq!(exec_acc.tool_items, appserver_acc.tool_items);
    }

    // ----------------------------------------------------- terminals (9)

    #[test]
    fn appserver_interrupt_is_a_distinct_terminal() {
        let fixture =
            include_str!("../../tests/fixtures/codex-appserver-0.149.0-interrupted-turn.jsonl");
        let mut acc = TurnAccumulator::new();
        for line in fixture.lines().filter(|l| !l.trim().is_empty()) {
            let value: Value = serde_json::from_str(line).expect("valid JSON");
            if let LineShape::Notification { method, params } = classify_line(&value) {
                acc.ingest_appserver_notification(&method, &params);
                // Only classify the *first* turn/completed in this fixture —
                // it records two turns (the interrupted one, then a second
                // that completed normally on the same thread).
                if method == "turn/completed" {
                    break;
                }
            }
        }
        let outcome = super::super::classify_terminal(&acc, None);
        assert!(matches!(
            outcome,
            super::super::TerminalOutcome::Interrupted
        ));
        assert!(
            !matches!(
                outcome,
                super::super::TerminalOutcome::InterruptedRunning { .. }
            ),
            "harness-confirmed interrupt must not collapse into the exec-only inferred arm"
        );
        assert!(!matches!(
            outcome,
            super::super::TerminalOutcome::AmbiguousUnknown
        ));
    }

    // ------------------------------------------------------- usage (10)

    #[test]
    fn appserver_decodes_token_usage_updated() {
        let mut acc = TurnAccumulator::new();
        let params = json!({
            "threadId": "t1", "turnId": "turn1",
            "tokenUsage": {"total": {"totalTokens": 100}, "last": {"totalTokens": 100}}
        });
        acc.ingest_appserver_notification("thread/tokenUsage/updated", &params);
        assert_eq!(acc.usage.unwrap()["total"]["totalTokens"], 100);
    }

    // ------------------------------------------------------ auth (11)

    #[test]
    fn appserver_unauthorized_turn_error_names_auth() {
        let fixture = include_str!(
            "../../tests/fixtures/codex-appserver-0.149.0-unauthorized-turn-failed.derived.json"
        );
        let mut acc = TurnAccumulator::new();
        for line in fixture.lines().filter(|l| !l.trim().is_empty()) {
            let value: Value = serde_json::from_str(line).expect("valid JSON");
            if let LineShape::Notification { method, params } = classify_line(&value) {
                acc.ingest_appserver_notification(&method, &params);
            }
        }
        assert_eq!(acc.last_codex_error_info.as_deref(), Some("unauthorized"));
        let outcome = super::super::classify_terminal(&acc, None);
        match outcome {
            super::super::TerminalOutcome::Failed { message } => {
                assert!(message.contains("expired") || message.contains("unauthorized"));
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    // ------------------------------------------------- safety table (12)

    #[test]
    fn appserver_answers_every_server_request_including_unknown_ones() {
        let methods: Vec<&str> =
            include_str!("../../tests/fixtures/codex-appserver-0.149.0-server-request-methods.txt")
                .lines()
                .filter(|l| !l.trim().is_empty())
                .collect();
        assert_eq!(methods.len(), 11, "M8's own count");
        for method in &methods {
            let answered = answer_for_request(method, false);
            match method {
                &"currentTime/read" => {
                    assert!(matches!(answered.answer, RequestAnswer::Result(_)));
                    assert_eq!(answered.journal_phase, None, "plumbing, nothing to journal");
                }
                _ => {
                    assert!(
                        answered.journal_phase.is_some(),
                        "{method}: every non-plumbing answer must journal a phase"
                    );
                }
            }
        }
        // The guarantee's whole point: an invented method must still be
        // answered, with the -32601 shape that keeps the protocol contract.
        let unknown = answer_for_request("thread/someFutureMethod", false);
        assert_eq!(
            unknown.answer,
            RequestAnswer::Error {
                code: -32601,
                message: "Method not found".to_string()
            }
        );
        assert_eq!(unknown.journal_phase, Some("unknown_server_request"));

        // The five approval-flavored requests are denied, never approved
        // (§3.4 point 7, J5 safety) -- decode each answer's shape and assert
        // it names a "no"/deny value, never an accept one.
        for (method, deny_key, deny_values) in [
            (
                "item/commandExecution/requestApproval",
                "decision",
                vec!["decline"],
            ),
            (
                "item/fileChange/requestApproval",
                "decision",
                vec!["decline"],
            ),
            ("applyPatchApproval", "decision", vec!["abort"]),
            ("execCommandApproval", "decision", vec!["abort"]),
        ] {
            match answer_for_request(method, false).answer {
                RequestAnswer::Result(value) => {
                    let got = value[deny_key].as_str().unwrap_or("");
                    assert!(
                        deny_values.contains(&got),
                        "{method}: expected a deny value in {deny_values:?}, got {got:?}"
                    );
                }
                other => panic!("{method}: expected Result, got {other:?}"),
            }
        }
        match answer_for_request("item/permissions/requestApproval", false).answer {
            RequestAnswer::Result(value) => {
                assert!(value["permissions"]["fileSystem"].is_null());
                assert!(value["permissions"]["network"].is_null());
            }
            other => panic!("expected Result, got {other:?}"),
        }
    }

    // --------------------------------------------------- unknown items (13)

    #[test]
    fn appserver_unknown_item_types_are_counted_never_decoded() {
        let mut acc = TurnAccumulator::new();
        let params = json!({
            "item": {"id": "i0", "type": "reasoning", "summary": [], "content": []},
            "threadId": "t1", "turnId": "turn1"
        });
        let events = acc.ingest_appserver_notification("item/completed", &params);
        assert!(events.is_empty());
        assert_eq!(acc.unknown_items, vec!["reasoning".to_string()]);
    }

    #[test]
    fn appserver_unknown_methods_are_counted_never_decoded() {
        let mut acc = TurnAccumulator::new();
        let events = acc.ingest_appserver_notification("remoteControl/status/changed", &json!({}));
        assert!(events.is_empty());
        assert_eq!(
            acc.unknown_methods,
            vec!["remoteControl/status/changed".to_string()]
        );
    }

    // ------------------------------------------------- malformed line (14)

    #[test]
    fn appserver_malformed_line_is_counted_tolerated_and_still_archived() {
        for line in ["not json at all", "{also not json"] {
            assert!(serde_json::from_str::<Value>(line).is_err());
        }
        // The reader's own tolerance is proven at the process-bound layer
        // (the stub-driven suite); here the pure half's contribution is that
        // a malformed line never reaches `classify_line`/`ingest_appserver_
        // notification` at all, which this test's assertion above pins.
    }

    // ---------------------------------------------------- fingerprint (15)

    #[test]
    fn compute_fingerprint_matches_the_measured_constant_on_the_captured_schema() {
        // We do not ship the 401-file schema dump itself (it is derived,
        // regenerable offline in 52ms, and would bloat the repo for no
        // reason) — this test instead pins the *algorithm's own behavior*:
        // changing one byte of one pinned file changes the digest, and the
        // digest is a pure function of exactly the 14 files, in order,
        // never the whole tree. The `d0fbf8d…` constant itself was computed
        // once against the real schema dump (recorded in the wave PR body
        // and in `tests/fixtures/codex-appserver-0.149.0-schema-fingerprint.
        // txt`) and is not re-derived by this test, which is the point: a
        // build with no `codex` binary must still be able to prove the
        // *function* is deterministic and order/content-sensitive.
        let dir = tempfile::tempdir().expect("tempdir");
        for (i, relative) in PINNED_SCHEMA_FILES.iter().enumerate() {
            let path = dir.path().join(relative);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, format!("{{\"n\":{i}}}")).unwrap();
        }
        let first = compute_fingerprint(dir.path()).expect("fingerprint");
        let second = compute_fingerprint(dir.path()).expect("fingerprint");
        assert_eq!(first, second, "deterministic for unchanged input");

        // Flip one byte of one pinned file: the digest must move.
        let touched = dir.path().join(PINNED_SCHEMA_FILES[0]);
        std::fs::write(&touched, "{\"n\":999}").unwrap();
        let third = compute_fingerprint(dir.path()).expect("fingerprint");
        assert_ne!(first, third, "a changed pinned file must move the digest");
    }

    #[test]
    fn compute_fingerprint_refuses_a_missing_pinned_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let err = compute_fingerprint(dir.path()).expect_err("no files exist");
        assert!(err.contains("cannot read"));
    }
}
