//! `opencode serve` HTTP+SSE client (W3 spec, §1.1). A protocol client and
//! nothing else: it knows HTTP, SSE framing, and opencode's serve operation
//! names, and it knows nothing about `StartRequest`, `Observation`, or the
//! journal — `opencode.rs` (its parent module, which declares it via
//! `#[path]`) is the only caller and the only place any of that vocabulary
//! appears.
//!
//! Split along `codex_appserver.rs`'s own seam (§6.2 of that spec, reused
//! verbatim as a rule): the top half of this file — [`parse_listening_line`],
//! [`compute_doc_fingerprint`], [`serve_event_view`], [`serve_part_envelope`],
//! [`PendingGate`], [`parse_sse_frames`] — is **pure**: no process, no
//! socket, no clock. It is drivable from an iterator of lines and a `Vec` of
//! outputs, which is exactly how this module's own unit tests drive it. The
//! bottom half ([`ServeChild`], [`ServeHandle`], [`drive_sse_reader`]) is
//! process/socket-bound: spawn, port learning, readiness, the SSE reader,
//! process-group kill, stderr drain.
//!
//! Every claim below carries the same provenance discipline W1 established:
//! **[measured]** means opencode 1.18.19 on Cerberus, 2026-08-23, either the
//! probe packet or the six-fixture capture beside this module's own suite
//! (`tests/fixtures/opencode-serve-1.18.19-*`).

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};

use crate::backend::child::{self, ChildLifetime};

// ------------------------------------------------------------------- consts

/// `OPENCODE_SERVER_PASSWORD`'s required username (§3.4, C6): the *literal*
/// string `"opencode"`. Measured: an empty username or any other username
/// with the correct password 401s; only `opencode:<password>` succeeds. The
/// OpenAPI document's `security` is `[]` and `securitySchemes` is `null` —
/// nothing in `/doc` says this, so it lives here, on the one code path that
/// applies it, and nowhere else.
pub(super) const SERVE_AUTH_USERNAME: &str = "opencode";

/// Bounded tail of a serve child's stderr retained for evidence (mirrors
/// `codex_appserver::STDERR_TAIL_BYTES` — the tail is what a failure's last
/// words live in).
const SERVE_STDERR_TAIL_BYTES: usize = 4096;

/// The exact operations this adapter's client depends on (§5.1), in digest
/// order. Named by `operationId`, not by path: the path is a router
/// implementation detail, the operationId is the stable identity the
/// generator emits.
pub(super) const PINNED_OPERATIONS: &[&str] = &[
    "session.create",
    "session.prompt",
    "session.messages",
    "session.abort",
    "permission.respond",
    "question.reply",
    "event.subscribe",
];

/// Digest algorithm for [`compute_doc_fingerprint`] — `blake3`, already an
/// in-tree dependency, for the same reason `codex_appserver.rs` pins it.
pub(super) const FINGERPRINT_ALGORITHM: &str = "blake3";

/// The scoped fingerprint measured against
/// `tests/fixtures/opencode-serve-1.18.19-openapi-doc.json` at
/// implementation time. `compute_doc_fingerprint_matches_the_measured_constant`
/// re-derives it from the committed fixture on every build — a build with no
/// `opencode` binary anywhere in the loop can still prove this constant is
/// reproducible.
pub(super) const MEASURED_DOC_FINGERPRINT: &str =
    "292ebbce2cba22878a469e419251ec7740f8e8b68968ec6e6a9639c7102555e0";

// ---------------------------------------------------------------- top half

/// Parse the bound base URL out of one `opencode serve` stdout line (§3.3).
///
/// Exact measured shape: `"opencode server listening on
/// http://127.0.0.1:<port>"`, single, newline-terminated (the newline is not
/// part of `line`), no trailing whitespace. Prefix-matched; the remainder is
/// trimmed and must parse as `http://127.0.0.1:<u16>` with a **non-zero**
/// port. Anything else is startup chatter, not this line — `None`, never a
/// partial guess.
///
/// Backed by `tests/fixtures/opencode-serve-1.18.19-listening-stdout.txt`,
/// four independent port captures against the real, installed 1.18.19
/// binary, re-captured fresh by the W3 fixer session (2026-08-23) rather than
/// carried over from the implementer's own session-local claim: `4096` (the
/// conventional default, port free), `46701` and `41747` (two separate runs
/// with 4096 pre-occupied by another listener, forcing a true OS-assigned
/// ephemeral port), and `36873` (a second instance started concurrently with
/// a first that took 4096). `parse_listening_line_pins_the_measured_shape`
/// parses every line in that committed fixture, so this claim is checkable
/// without re-running the binary.
pub(super) fn parse_listening_line(line: &str) -> Option<String> {
    const PREFIX: &str = "opencode server listening on ";
    let remainder = line.strip_prefix(PREFIX)?.trim();
    let rest = remainder.strip_prefix("http://127.0.0.1:")?;
    if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let port: u16 = rest.parse().ok()?;
    if port == 0 {
        return None;
    }
    Some(remainder.to_string())
}

/// Fold a whole SSE body's lines into complete frames (pure; drivable from a
/// fixture split on `.lines()`). `data: ` (or bare `data:`) prefixed lines
/// accumulate; a blank line terminates a frame. Multi-line `data:` frames are
/// concatenated with `\n` per the SSE spec — unmeasured on this server (every
/// captured frame was single-line), so this is spec-conformant handling of a
/// shape never observed, not a claim that it occurs. A frame whose
/// accumulated data does not parse as JSON is returned as `Err`, counted by
/// the caller, never decoded.
///
/// Production code never calls this: the live path reads a growing socket
/// and cannot afford to re-join and re-parse the whole buffer per frame, so
/// it uses the equivalent incremental loop in [`drive_sse_reader`] instead.
/// This is the pure half of that same framing rule (§4.1's split) — the one
/// a fixture can drive with no socket in the loop, and the one this module's
/// own SSE-framing tests exercise directly.
#[allow(dead_code)]
pub(super) fn parse_sse_frames(text: &str) -> Vec<Result<Value, String>> {
    let mut frames = Vec::new();
    let mut buf: Vec<&str> = Vec::new();
    let flush = |buf: &mut Vec<&str>, frames: &mut Vec<Result<Value, String>>| {
        if buf.is_empty() {
            return;
        }
        let data = buf.join("\n");
        frames.push(serde_json::from_str::<Value>(&data).map_err(|e| e.to_string()));
        buf.clear();
    };
    for line in text.lines() {
        if line.is_empty() {
            flush(&mut buf, &mut frames);
            continue;
        }
        if let Some(rest) = line.strip_prefix("data: ") {
            buf.push(rest);
        } else if let Some(rest) = line.strip_prefix("data:") {
            buf.push(rest);
        }
        // Any other SSE field (`event:`, `id:`, `retry:`, a `:` comment) is
        // not part of this server's measured frames and is ignored.
    }
    flush(&mut buf, &mut frames);
    frames
}

/// Server-scoped SSE types that carry no `sessionID` and are not this
/// execution's own noise (§4.2's filter's one exception list).
const SERVER_SCOPED_TYPES: &[&str] = &["server.connected", "server.heartbeat"];

/// §4.2's mandatory filter: is this frame in scope for `session_id`? A frame
/// naming a *different* session, or naming none at all outside
/// [`SERVER_SCOPED_TYPES`], is out of scope — the bus is server-wide (C9: one
/// capture carried 45 unrelated `plugin.added` plus catalog/integration/
/// reference updates) and every frame is filtered before it reaches any
/// decoding.
pub(super) fn frame_in_scope(event_type: &str, properties: &Value, session_id: &str) -> bool {
    if SERVER_SCOPED_TYPES.contains(&event_type) {
        return true;
    }
    properties.get("sessionID").and_then(Value::as_str) == Some(session_id)
}

/// Envelope-level dispositions this decoder gives an in-scope SSE frame
/// (§4.3's table). Pure classification by `type` string; nothing here reads
/// `part`/`properties` contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ServeEventDisposition {
    /// `message.part.updated` — bridge through [`serve_part_envelope`].
    PartUpdated,
    /// `message.updated` — fold into the session's `message_roles` map.
    MessageUpdated,
    /// `permission.asked` — park a [`PendingGate::Permission`].
    PermissionAsked,
    /// `permission.replied` — clear the gate.
    PermissionReplied,
    /// `question.asked` — park a [`PendingGate::Question`].
    QuestionAsked,
    /// `question.replied` — clear the gate.
    QuestionReplied,
    /// `session.error` — records the typed error; `MessageAbortedError`
    /// specifically marks the turn aborted.
    SessionError,
    /// Archived-not-decoded: the *completed* counterpart already produces
    /// the event, so decoding the delta/progress notification too would
    /// double-count. `session.idle` is deliberately in this set (§4.3,
    /// C9): it fires twice on the abort capture and *before* the tool
    /// part's final `completed` snapshot, so it is not a turn terminal.
    Archived,
    /// Counted by its own wire string, never decoded.
    Unknown,
}

const ARCHIVED_NOT_DECODED_TYPES: &[&str] = &[
    "message.part.delta",
    "session.status",
    "session.diff",
    "session.idle",
    "session.updated",
    "server.heartbeat",
    "server.connected",
    "plugin.added",
    "catalog.updated",
    "integration.updated",
    "reference.updated",
];

/// Classify one SSE frame's `type` string into §4.3's disposition table.
/// Pure — takes no properties, decides nothing about their content.
pub(super) fn serve_event_view(event_type: &str) -> ServeEventDisposition {
    match event_type {
        "message.part.updated" => ServeEventDisposition::PartUpdated,
        "message.updated" => ServeEventDisposition::MessageUpdated,
        "permission.asked" => ServeEventDisposition::PermissionAsked,
        "permission.replied" => ServeEventDisposition::PermissionReplied,
        "question.asked" => ServeEventDisposition::QuestionAsked,
        "question.replied" => ServeEventDisposition::QuestionReplied,
        "session.error" => ServeEventDisposition::SessionError,
        t if ARCHIVED_NOT_DECODED_TYPES.contains(&t) => ServeEventDisposition::Archived,
        _ => ServeEventDisposition::Unknown,
    }
}

/// run-json envelope `type` a serve `part.type` maps to (§6.2's table).
/// `reasoning` has no run-json envelope type — return `None`, and the caller
/// counts it under a dedicated counter rather than `unknown_events` (W1's
/// `decode_export` precedent: a known type this vocabulary has none for).
fn envelope_type_for(part_type: &str) -> Option<&'static str> {
    match part_type {
        "step-start" => Some("step_start"),
        "text" => Some("text"),
        "tool" => Some("tool_use"),
        "step-finish" => Some("step_finish"),
        _ => None,
    }
}

/// Translate one filtered `message.part.updated` frame's `properties` into
/// the run-json envelope [`super::TurnAccumulator::ingest_line`] already
/// reads, or `None` when this snapshot must not be decoded. Pure. The
/// **only** bridge between the serve transport and the decoder (§6.1's
/// one-decoder rule).
///
/// `role_of` answers "what role does this messageID belong to", from the
/// session's own `message_roles` map (populated from `message.updated`
/// frames, §4.3). Four gates return `None`, each measured (§6.2):
///
/// 1. **Role gate (C10).** Not `Some("assistant")` — including "not yet
///    known", fail-closed — never decodes. The user's own prompt arrives as
///    a `message.part.updated` with `part.type: "text"`; without this gate
///    it would decode as an assistant-completed event.
/// 2. **Completeness gate (C10).** `text`/`reasoning` parts are cumulative
///    snapshots, not deltas (`{text:""}` then `{text:"pong", time:{start,
///    end}}`); only `time.end`-present decodes, or the empty snapshot would
///    also emit.
/// 3. **Pending-tool gate (C11).** A `tool` part's `pending` snapshot
///    carries `state.input: {}`; decoding it would latch `tool.requested`
///    with an empty input via `requested_calls`' first-sight rule.
/// 4. **Mapping gate.** `reasoning` (and anything `envelope_type_for` does
///    not know) has no run-json envelope type at all.
pub(super) fn serve_part_envelope(
    properties: &Value,
    role_of: impl Fn(&str) -> Option<String>,
) -> Option<Value> {
    let part = properties.get("part")?;
    let part_type = part.get("type").and_then(Value::as_str)?;
    let mapped = envelope_type_for(part_type)?;
    let message_id = part.get("messageID").and_then(Value::as_str).unwrap_or("");
    if role_of(message_id).as_deref() != Some("assistant") {
        return None;
    }
    if matches!(part_type, "text" | "reasoning") && part.pointer("/time/end").is_none() {
        return None;
    }
    if part_type == "tool"
        && part.pointer("/state/status").and_then(Value::as_str) == Some("pending")
    {
        return None;
    }
    Some(json!({
        "type": mapped,
        "sessionID": properties.get("sessionID").cloned().unwrap_or(Value::Null),
        "part": part.clone(),
    }))
}

/// Digest rule (§5.2, stated exactly so it is reproducible): for each
/// `operationId` in [`PINNED_OPERATIONS`], in order, feed the operationId's
/// UTF-8 bytes, one `NUL`, the unique `(path, method)` operation object's
/// canonical `serde_json::to_string` bytes (this crate builds `serde_json`
/// without `preserve_order`, so `Map` is a `BTreeMap` and this is key-sorted
/// and canonical), one `NUL`. A missing or duplicated operationId is an
/// error, never a zero.
pub(super) fn compute_doc_fingerprint(doc: &Value) -> Result<String, String> {
    let paths = doc
        .get("paths")
        .and_then(Value::as_object)
        .ok_or_else(|| "openapi document has no `paths` object".to_string())?;
    let mut hasher = blake3::Hasher::new();
    for operation_id in PINNED_OPERATIONS {
        hasher.update(operation_id.as_bytes());
        hasher.update(&[0u8]);
        let mut found: Option<&Value> = None;
        for methods in paths.values() {
            let Some(methods) = methods.as_object() else {
                continue;
            };
            for op in methods.values() {
                if op.get("operationId").and_then(Value::as_str) == Some(*operation_id) {
                    if found.is_some() {
                        return Err(format!(
                            "operationId {operation_id} names more than one operation"
                        ));
                    }
                    found = Some(op);
                }
            }
        }
        let op = found.ok_or_else(|| {
            format!("pinned operationId {operation_id} is missing from the document")
        })?;
        let bytes = serde_json::to_string(op).map_err(|e| e.to_string())?;
        hasher.update(bytes.as_bytes());
        hasher.update(&[0u8]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

// -------------------------------------------------------------- pending gate

/// One outstanding harness-issued gate on this session: a permission the
/// harness is blocked on, or a question the actor authored (§6.3). Measured
/// payload shapes (probe packet's ask/approval capture): `per_`/`que_` id
/// prefixes are wholly disjoint, which is what makes gate authorship
/// schema-distinguishable rather than guessed.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum PendingGate {
    Permission {
        id: String,
        permission: String,
        patterns: Vec<String>,
        metadata: Value,
        call_id: String,
    },
    Question {
        id: String,
        questions: Value,
        call_id: String,
    },
}

impl PendingGate {
    /// Parse a `permission.asked` frame's `properties`. `None` when the
    /// frame is missing the fields this gate needs to be actionable.
    pub(super) fn from_permission_asked(properties: &Value) -> Option<Self> {
        let id = properties.get("id").and_then(Value::as_str)?.to_string();
        let permission = properties
            .get("permission")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let patterns = properties
            .get("patterns")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let metadata = properties.get("metadata").cloned().unwrap_or(Value::Null);
        let call_id = properties
            .pointer("/tool/callID")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        Some(PendingGate::Permission {
            id,
            permission,
            patterns,
            metadata,
            call_id,
        })
    }

    /// Parse a `question.asked` frame's `properties`.
    pub(super) fn from_question_asked(properties: &Value) -> Option<Self> {
        let id = properties.get("id").and_then(Value::as_str)?.to_string();
        let questions = properties.get("questions").cloned().unwrap_or(Value::Null);
        let call_id = properties
            .pointer("/tool/callID")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        Some(PendingGate::Question {
            id,
            questions,
            call_id,
        })
    }

    /// A human-readable prompt composed **only** from measured structured
    /// fields (§7.1/§7.2) — never from model prose.
    pub(super) fn prompt(&self) -> String {
        match self {
            PendingGate::Permission {
                permission,
                patterns,
                metadata,
                ..
            } => format!(
                "opencode is asking for permission to run a `{permission}` action matching \
                 {patterns:?} (metadata: {metadata}). Reply with exactly one of: once, always, \
                 reject.",
            ),
            PendingGate::Question { questions, .. } => {
                format!(
                    "the actor is asking a question and is waiting for an answer: {questions}. \
                     Reply with the exact label of one option."
                )
            }
        }
    }
}

/// §7.1's closed reply vocabulary for a permission gate: exactly `once`,
/// `always`, `reject` (case-insensitive, trimmed, exact). Anything else is a
/// structured refusal naming the three accepted values — no heuristic, no
/// fuzzy match, no default.
pub(super) fn parse_permission_reply(input: &str) -> Result<&'static str, String> {
    match input.trim().to_ascii_lowercase().as_str() {
        "once" => Ok("once"),
        "always" => Ok("always"),
        "reject" => Ok("reject"),
        other => Err(format!(
            "permission reply must be exactly one of: once, always, reject (got {other:?})"
        )),
    }
}

/// §7.2's answering rule: the input must exactly match one `options[].label`
/// (trimmed, case-insensitive) of the **single** pending question. More than
/// one question, or an unmatched label, is a structured refusal naming the
/// available labels — multi-question/multi-select was never measured.
pub(super) fn parse_question_reply(gate: &PendingGate, input: &str) -> Result<Value, String> {
    let PendingGate::Question { questions, .. } = gate else {
        return Err("no question is pending on this gate".to_string());
    };
    let questions = questions
        .as_array()
        .ok_or_else(|| "malformed question gate: `questions` is not an array".to_string())?;
    if questions.len() != 1 {
        return Err(format!(
            "this gate carries {} questions; only a single-question reply is supported",
            questions.len()
        ));
    }
    let options = questions[0]
        .get("options")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let labels: Vec<String> = options
        .iter()
        .filter_map(|o| o.get("label").and_then(Value::as_str))
        .map(str::to_string)
        .collect();
    let trimmed = input.trim();
    let matched = labels
        .iter()
        .find(|label| label.eq_ignore_ascii_case(trimmed));
    match matched {
        Some(label) => Ok(json!([[label]])),
        None => Err(format!(
            "question reply must exactly match one of the available labels {labels:?} (got \
             {trimmed:?})"
        )),
    }
}

/// Mint a 160-bit server password, per execution. `ulid` is already a direct
/// dependency; `claude.rs::new_session_uuid` set this precedent for exactly
/// this reason (R5 — no new dependency for random bits).
pub(super) fn mint_server_password() -> String {
    let a = ulid::Ulid::generate();
    let b = ulid::Ulid::generate();
    format!("{a}{b}")
}

/// Best-effort text for a caught `std::thread::JoinHandle::join` panic
/// payload — used only by `opencode.rs`'s two runtime-isolation joins
/// (`run_serve_gates`, `launch_serve`; W4 fixer) to report *why* an
/// isolated thread died, since a payload is `Box<dyn Any + Send>` and
/// almost always one of these two concrete types in practice.
pub(super) fn describe_panic(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

// ------------------------------------------------------------ process-bound

/// Why `ServeHandle::post_message` did not return a response body (§9.2's own
/// distinction — see that method's doc for why the two are not one string).
#[derive(Debug, Clone)]
pub(super) enum PostMessageError {
    Http { status: u16, body: String },
    Transport(String),
}

impl std::fmt::Display for PostMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostMessageError::Http { status, body } => write!(f, "HTTP {status}: {body}"),
            PostMessageError::Transport(detail) => write!(f, "{detail}"),
        }
    }
}

/// What a non-blocking peek at the serve child can prove (mirrors
/// `codex_appserver::ChildStatus`).
#[derive(Debug, Clone)]
pub(super) enum ServeChildStatus {
    Running,
    Exited { detail: String, code: Option<i32> },
    Unknown(String),
}

/// One `opencode serve` child, owned for the whole execution (§3.1: one per
/// [`super::OpencodeExecution`], never per turn). Spawns the process, learns
/// its port from stdout (§3.3), and hands back the base URL; the caller
/// (`opencode.rs`) drives everything HTTP/SSE-specific through
/// [`ServeHandle`].
#[derive(Debug)]
pub(super) struct ServeChild {
    child: Arc<Mutex<Child>>,
    exit_status: Arc<Mutex<Option<ExitStatus>>>,
    stderr_tail: Arc<Mutex<Vec<u8>>>,
    pgid: u32,
    /// `Some` only for a [`ChildLifetime::Probe`] child (#310): deregisters
    /// this pgid from the owning probe walk's live set when this handle
    /// drops. Held, never read.
    _registration: Option<child::ProbeChildRegistration>,
}

impl ServeChild {
    /// Spawn the child and block, bounded by `port_budget`, for its listening
    /// line. `--port 0 --hostname 127.0.0.1` are passed explicitly even
    /// though both are the CLI's own defaults (§3.2): the adapter must not
    /// inherit a default that can change under it. `--pure` and
    /// `--print-logs` are deliberately **not** passed (§3.2, §10).
    ///
    /// C5, load-bearing: `--port 0` is a real ephemeral request, but the
    /// first server on a fresh host was measured binding **4096** —
    /// opencode's conventional default, not a true ephemeral port. This is
    /// not a correctness hazard because the port is *learned*, never
    /// assumed, from this stdout line — but a reader must not conclude "port
    /// 0 means never 4096".
    ///
    /// `lifetime` is #310's fix and must be stated by every caller: a
    /// [`ChildLifetime::Probe`] child is additionally hardened
    /// ([`child::harden_probe_child`]) so a `SIGKILL`ed daemon takes it with
    /// it, and recorded against the probe walk that owns the calling thread.
    /// The `ServeOnly`/`Auto` gate is that caller; `launch_serve`'s child is
    /// [`ChildLifetime::Execution`] and must never be hardened — see that
    /// enum's own doc for why.
    pub(super) fn spawn(
        executable: &Path,
        cwd: &Path,
        env: &BTreeMap<String, String>,
        config_content: Option<&str>,
        password: &str,
        port_budget: Duration,
        lifetime: ChildLifetime,
    ) -> Result<(Self, String), String> {
        let mut command = Command::new(executable);
        command
            .args(["serve", "--port", "0", "--hostname", "127.0.0.1"])
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (key, value) in env {
            command.env(key, value);
        }
        if let Some(content) = config_content {
            command.env(super::OPENCODE_CONFIG_CONTENT_ENV, content);
        }
        command.env("OPENCODE_SERVER_PASSWORD", password);
        // §3.7: a serve child's tool subprocesses are its grandchildren, so
        // the identical probe-11 orphaning hazard applies — process_group(0)
        // is what makes the recorded-pgid kill (never by name/pattern) reach
        // the whole tree. A probe child gets that from `harden_probe_child`
        // (plus `PR_SET_PDEATHSIG`, #310); an execution child gets the group
        // and nothing else.
        match lifetime {
            ChildLifetime::Probe => child::harden_probe_child(&mut command),
            ChildLifetime::Execution => {
                #[cfg(unix)]
                {
                    use std::os::unix::process::CommandExt;
                    command.process_group(0);
                }
            }
        }

        let mut child = command
            .spawn()
            .map_err(|e| format!("cannot spawn {executable:?} serve: {e}"))?;
        let pgid = child.id();
        // Recorded before anything below can fail: from here on this pgid is
        // reachable by `ProbeChildren::kill_all`, so an early return that
        // kills the group leaves nothing behind either way.
        let registration =
            matches!(lifetime, ChildLifetime::Probe).then(|| child::register_probe_child(pgid));
        // #310: an early return here used to drop `child` without signalling
        // it, leaving a live `opencode serve` with nothing holding it. The arm
        // is "cannot happen" — stdout was piped a dozen lines above — which is
        // precisely the kind of path this issue is about.
        let Some(stdout) = child.stdout.take() else {
            super::kill_process_group(Some(pgid));
            let _ = child.kill();
            let _ = child.wait();
            return Err("serve child stdout was not piped".to_string());
        };

        let stderr_tail = Arc::new(Mutex::new(Vec::<u8>::new()));
        if let Some(mut stderr) = child.stderr.take() {
            let sink = Arc::clone(&stderr_tail);
            std::thread::spawn(move || {
                let mut buffer = [0u8; 1024];
                loop {
                    match stderr.read(&mut buffer) {
                        Ok(0) | Err(_) => break,
                        Ok(read) => {
                            let mut held = sink.lock().expect("serve stderr tail lock");
                            held.extend_from_slice(&buffer[..read]);
                            let overflow = held.len().saturating_sub(SERVE_STDERR_TAIL_BYTES);
                            if overflow > 0 {
                                held.drain(..overflow);
                            }
                        }
                    }
                }
            });
        }

        // §3.3: the port is read, never assumed. The reader thread hands the
        // first accepted line back over a bounded channel and then keeps
        // draining stdout to EOF (never blocking the child on a full pipe),
        // discarding startup chatter past the port line.
        let (tx, rx) = sync_channel::<Option<String>>(1);
        std::thread::spawn(move || {
            let mut sent = false;
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if !sent && let Some(url) = parse_listening_line(&line) {
                    sent = true;
                    let _ = tx.send(Some(url));
                }
            }
            if !sent {
                let _ = tx.send(None);
            }
        });

        let base_url = match rx.recv_timeout(port_budget) {
            Ok(Some(url)) => url,
            // #310: both failure arms below kill *and reap*. Killing alone
            // leaves a zombie — the pid is still in the process table, still
            // answers `kill(pid, 0)` as alive, and still shows up under
            // `pgrep -x opencode`, so an orphan check cannot tell it from a
            // live 265 MB server. Measured: this is the arm that fired when
            // `DaemonHandle::kill` reaped a live gate child out from under a
            // spawn still waiting for its listening line.
            Ok(None) => {
                let tail =
                    String::from_utf8_lossy(&stderr_tail.lock().expect("serve stderr tail lock"))
                        .into_owned();
                super::kill_process_group(Some(pgid));
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "serve child's stdout reached EOF before a listening line arrived; stderr: {tail}"
                ));
            }
            Err(_) => {
                let tail =
                    String::from_utf8_lossy(&stderr_tail.lock().expect("serve stderr tail lock"))
                        .into_owned();
                super::kill_process_group(Some(pgid));
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "serve child emitted no listening line within {port_budget:?}; stderr: {tail}"
                ));
            }
        };

        Ok((
            Self {
                child: Arc::new(Mutex::new(child)),
                exit_status: Arc::new(Mutex::new(None)),
                stderr_tail,
                pgid,
                _registration: registration,
            },
            base_url,
        ))
    }

    /// This child's process-group id, recorded at spawn (§3.7) — never
    /// derived at kill time.
    pub(super) fn pgid(&self) -> u32 {
        self.pgid
    }

    /// A non-blocking peek at whether this child is still alive.
    pub(super) fn status(&self) -> ServeChildStatus {
        if let Some(status) = *self.exit_status.lock().expect("serve exit status lock") {
            return ServeChildStatus::Exited {
                detail: status.to_string(),
                code: status.code(),
            };
        }
        match self.child.lock().expect("serve child lock").try_wait() {
            Ok(None) => ServeChildStatus::Running,
            Ok(Some(status)) => {
                *self.exit_status.lock().expect("serve exit status lock") = Some(status);
                ServeChildStatus::Exited {
                    detail: status.to_string(),
                    code: status.code(),
                }
            }
            Err(e) => ServeChildStatus::Unknown(e.to_string()),
        }
    }

    /// The last [`SERVE_STDERR_TAIL_BYTES`] this child wrote to stderr.
    pub(super) fn stderr_tail(&self) -> String {
        String::from_utf8_lossy(&self.stderr_tail.lock().expect("serve stderr tail lock"))
            .into_owned()
    }

    /// Kill this child's whole process group, by recorded pgid — never by
    /// name or pattern (§3.7: the probe's own cleanup discipline; `pkill -f
    /// opencode` would take out the operator's own editor session).
    /// Idempotent.
    pub(super) fn kill(&mut self) {
        super::kill_process_group(Some(self.pgid));
        let _ = self.child.lock().expect("serve child lock").kill();
        if let Ok(status) = self.child.lock().expect("serve child lock").wait() {
            *self.exit_status.lock().expect("serve exit status lock") = Some(status);
        }
    }
}

impl Drop for ServeChild {
    /// Best-effort (§3.7, mirrors `AppServerChild::drop`): adapter state
    /// dropping is not a supported lifecycle path, but it must not orphan a
    /// process — nor leave a zombie, which is #310's addition here. A killed
    /// child nobody reaps stays in the process table answering `kill(pid, 0)`
    /// as alive and matching `pgrep -x opencode`, which is exactly the shape
    /// an orphan check cannot distinguish from the leak.
    fn drop(&mut self) {
        super::kill_process_group(Some(self.pgid));
        let mut child = self.child.lock().expect("serve child lock");
        let _ = child.kill();
        let _ = child.wait();
    }
}

/// The HTTP+SSE half of one serve child: a `reqwest::blocking::Client`, the
/// base URL learned at spawn, and the minted password. Cheap to clone (an
/// `Arc`-backed `Client`), so both the turn-driving thread and the SSE
/// reader thread can hold one.
#[derive(Clone)]
pub(super) struct ServeHandle {
    client: reqwest::blocking::Client,
    base_url: String,
    password: String,
}

impl std::fmt::Debug for ServeHandle {
    /// Redacts the password (§3.4) — never logged, never journaled.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServeHandle")
            .field("base_url", &self.base_url)
            .field("password", &"<redacted>")
            .finish()
    }
}

impl ServeHandle {
    /// W4 fixer (release-blocking daemon-boot panic): building this client
    /// -- and, measured separately, sending *any* request on it -- panics
    /// ("Cannot drop a runtime in a context where blocking is not allowed")
    /// when called from a thread already inside a tokio runtime. That is
    /// `reqwest`'s own `blocking::wait::enter()` sanity check
    /// (`reqwest-0.13.4/src/blocking/wait.rs`, debug-build only, but every
    /// blocking call — `build()` included, via `ClientHandle::new`'s own
    /// internal wait — routes through it), not something isolating just
    /// this constructor can fix: an already-built client's later `.send()`
    /// panics exactly the same way if *that* call happens on a
    /// runtime-owned thread. See `run_serve_gates` and `launch_serve` for
    /// where the whole reqwest-touching sequence is isolated instead — this
    /// constructor stays plain so it behaves identically regardless of
    /// which of them (or a future caller) is holding the isolation.
    pub(super) fn new(base_url: String, password: String) -> Result<Self, String> {
        let client = reqwest::blocking::Client::builder()
            .build()
            .map_err(|e| format!("cannot build the serve HTTP client: {e}"))?;
        Ok(Self {
            client,
            base_url,
            password,
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    /// `GET {base}/doc` — the readiness liveness gate (§3.5) and the
    /// fingerprint gate's own fetch (§5.4). A `401` here is reported with a
    /// distinct message naming the username rule (§3.4), because it is the
    /// failure a future maintainer will hit.
    pub(super) fn get_doc(&self, budget: Duration) -> Result<Value, String> {
        let response = self
            .client
            .get(self.url("/doc"))
            .basic_auth(SERVE_AUTH_USERNAME, Some(&self.password))
            .timeout(budget)
            .send()
            .map_err(|e| format!("GET /doc: {e}"))?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(
                "GET /doc: 401 Unauthorized -- opencode's basic auth requires the literal \
                 username \"opencode\" (any other username, including empty, 401s even with the \
                 correct password); this is not documented in /doc's own security scheme"
                    .to_string(),
            );
        }
        if !response.status().is_success() {
            return Err(format!("GET /doc: HTTP {}", response.status()));
        }
        response
            .json::<Value>()
            .map_err(|e| format!("GET /doc: response body is not JSON: {e}"))
    }

    /// `POST /session` — synchronous session creation (§3.6): the session id
    /// comes back **before any turn**, closing W1's `SESSION_ID_BUDGET`
    /// hazard by construction on this transport.
    pub(super) fn create_session(&self, budget: Duration) -> Result<String, String> {
        let response = self
            .client
            .post(self.url("/session"))
            .basic_auth(SERVE_AUTH_USERNAME, Some(&self.password))
            .json(&json!({}))
            .timeout(budget)
            .send()
            .map_err(|e| format!("POST /session: {e}"))?;
        if !response.status().is_success() {
            return Err(format!("POST /session: HTTP {}", response.status()));
        }
        let body: Value = response
            .json()
            .map_err(|e| format!("POST /session: response body is not JSON: {e}"))?;
        body.get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| format!("POST /session: response carries no `id` field ({body})"))
    }

    /// `POST /session/{id}/message` (operationId `session.prompt`; §9.1's
    /// "there is no separate `/prompt` path" — the SDK vocabulary invites
    /// that guess). Synchronous: the response `{info, parts}` is the
    /// authoritative terminal.
    ///
    /// Returns [`PostMessageError`], not a flat string: §9.2's terminal
    /// table treats a non-2xx response and a transport-level failure
    /// (connection reset, budget expiry) differently — the first is always
    /// `Failed`, the second is `InterruptedRunning` only when the SSE side
    /// separately recorded `MessageAbortedError`, else `AmbiguousUnknown`.
    /// Flattening both to one string would lose exactly the distinction
    /// that table depends on.
    pub(super) fn post_message(
        &self,
        session_id: &str,
        body: &Value,
        budget: Duration,
    ) -> Result<Value, PostMessageError> {
        let response = self
            .client
            .post(self.url(&format!("/session/{session_id}/message")))
            .basic_auth(SERVE_AUTH_USERNAME, Some(&self.password))
            .json(body)
            .timeout(budget)
            .send()
            .map_err(|e| PostMessageError::Transport(e.to_string()))?;
        let status = response.status();
        let text = response
            .text()
            .unwrap_or_else(|e| format!("<cannot read body: {e}>"));
        if !status.is_success() {
            return Err(PostMessageError::Http {
                status: status.as_u16(),
                body: super::truncate(&text, 800).to_string(),
            });
        }
        serde_json::from_str(&text)
            .map_err(|e| PostMessageError::Transport(format!("response body is not JSON: {e}")))
    }

    /// `GET /session/{id}/message` (operationId `session.messages`, §7.4).
    /// Returns a **bare array** — the caller shims it into the export
    /// envelope before handing it to `decode_export`.
    pub(super) fn get_messages(&self, session_id: &str, budget: Duration) -> Result<Value, String> {
        let response = self
            .client
            .get(self.url(&format!("/session/{session_id}/message")))
            .basic_auth(SERVE_AUTH_USERNAME, Some(&self.password))
            .timeout(budget)
            .send()
            .map_err(|e| format!("GET /session/{session_id}/message: {e}"))?;
        if !response.status().is_success() {
            return Err(format!(
                "GET /session/{session_id}/message: HTTP {}",
                response.status()
            ));
        }
        response.json().map_err(|e| {
            format!("GET /session/{session_id}/message: response body is not JSON: {e}")
        })
    }

    /// `POST /session/{id}/abort` (§7.3): 200/true on success.
    pub(super) fn post_abort(&self, session_id: &str, budget: Duration) -> Result<Value, String> {
        let response = self
            .client
            .post(self.url(&format!("/session/{session_id}/abort")))
            .basic_auth(SERVE_AUTH_USERNAME, Some(&self.password))
            .timeout(budget)
            .send()
            .map_err(|e| format!("POST /session/{session_id}/abort: {e}"))?;
        if !response.status().is_success() {
            return Err(format!(
                "POST /session/{session_id}/abort: HTTP {}",
                response.status()
            ));
        }
        response
            .json()
            .map_err(|e| format!("POST /session/{session_id}/abort: response is not JSON: {e}"))
    }

    /// `POST /session/{id}/permissions/{permissionID}` — the **deprecated**,
    /// measured-live endpoint (C1/§7.1). Body `{"response": once|always|
    /// reject}`.
    pub(super) fn post_permission_reply(
        &self,
        session_id: &str,
        permission_id: &str,
        response_value: &str,
        budget: Duration,
    ) -> Result<Value, String> {
        let response = self
            .client
            .post(self.url(&format!(
                "/session/{session_id}/permissions/{permission_id}"
            )))
            .basic_auth(SERVE_AUTH_USERNAME, Some(&self.password))
            .json(&json!({"response": response_value}))
            .timeout(budget)
            .send()
            .map_err(|e| format!("POST /session/{session_id}/permissions/{permission_id}: {e}"))?;
        if !response.status().is_success() {
            return Err(format!(
                "POST /session/{session_id}/permissions/{permission_id}: HTTP {}",
                response.status()
            ));
        }
        response.json().map_err(|e| {
            format!(
                "POST /session/{session_id}/permissions/{permission_id}: response is not JSON: {e}"
            )
        })
    }

    /// `POST /question/{requestID}/reply` (§7.2). Body `{"answers":
    /// [[label]]}`.
    pub(super) fn post_question_reply(
        &self,
        request_id: &str,
        answers: &Value,
        budget: Duration,
    ) -> Result<Value, String> {
        let response = self
            .client
            .post(self.url(&format!("/question/{request_id}/reply")))
            .basic_auth(SERVE_AUTH_USERNAME, Some(&self.password))
            .json(&json!({"answers": answers}))
            .timeout(budget)
            .send()
            .map_err(|e| format!("POST /question/{request_id}/reply: {e}"))?;
        if !response.status().is_success() {
            return Err(format!(
                "POST /question/{request_id}/reply: HTTP {}",
                response.status()
            ));
        }
        response
            .json()
            .map_err(|e| format!("POST /question/{request_id}/reply: response is not JSON: {e}"))
    }

    /// Open `GET /event` (operationId `event.subscribe`) with no
    /// request-level timeout — the connection is meant to stay open for the
    /// life of the execution, closed only by dropping/killing the child.
    pub(super) fn open_event_stream(&self) -> Result<reqwest::blocking::Response, String> {
        let response = self
            .client
            .get(self.url("/event"))
            .basic_auth(SERVE_AUTH_USERNAME, Some(&self.password))
            .send()
            .map_err(|e| format!("GET /event: {e}"))?;
        if !response.status().is_success() {
            return Err(format!("GET /event: HTTP {}", response.status()));
        }
        Ok(response)
    }
}

/// Drive one already-open SSE response to EOF, calling `on_frame` with each
/// parsed JSON frame and `on_raw` with every raw line (for the §4.4 archive).
/// `reqwest::blocking::Response` implements `std::io::Read`, so this is a
/// plain `BufReader` line loop — no new crate. Blocks the calling thread
/// until the stream closes (child death, or the caller dropping/killing the
/// child out from under the socket).
pub(super) fn drive_sse_reader(
    response: reqwest::blocking::Response,
    mut on_frame: impl FnMut(Value),
    mut on_raw: impl FnMut(&str),
) {
    let mut buf: Vec<String> = Vec::new();
    for line in BufReader::new(response).lines() {
        let Ok(line) = line else { break };
        on_raw(&line);
        if line.is_empty() {
            if !buf.is_empty() {
                let data = buf.join("\n");
                if let Ok(value) = serde_json::from_str::<Value>(&data) {
                    on_frame(value);
                }
                buf.clear();
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("data: ") {
            buf.push(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("data:") {
            buf.push(rest.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPENAPI_DOC: &str =
        include_str!("../../tests/fixtures/opencode-serve-1.18.19-openapi-doc.json");
    const SSE_SYNC_TURN: &str =
        include_str!("../../tests/fixtures/opencode-serve-1.18.19-sse-sync-turn.txt");
    const SSE_ABORT: &str =
        include_str!("../../tests/fixtures/opencode-serve-1.18.19-sse-abort.txt");
    const SSE_PERMISSION_ASKED: &str =
        include_str!("../../tests/fixtures/opencode-serve-1.18.19-sse-permission-asked.txt");
    const SSE_QUESTION_ASKED: &str =
        include_str!("../../tests/fixtures/opencode-serve-1.18.19-sse-question-asked.txt");
    const LISTENING_STDOUT: &str =
        include_str!("../../tests/fixtures/opencode-serve-1.18.19-listening-stdout.txt");

    // -------------------------------------------------------- port learning

    /// Every line in the committed fixture is a real capture (see the
    /// fixture's own provenance note on [`parse_listening_line`]'s doc
    /// comment, above) against the installed 1.18.19 binary, not a
    /// synthesized string — this is what makes the claim checkable rather
    /// than circular.
    #[test]
    fn parse_listening_line_pins_the_measured_shape() {
        let mut lines_checked = 0;
        for line in LISTENING_STDOUT.lines() {
            if line.is_empty() {
                continue;
            }
            let base = parse_listening_line(line)
                .unwrap_or_else(|| panic!("fixture line failed to parse: {line:?}"));
            assert!(
                base.starts_with("http://127.0.0.1:"),
                "unexpected base url: {base}"
            );
            let port: u16 = base
                .rsplit(':')
                .next()
                .expect("port suffix")
                .parse()
                .expect("numeric port");
            assert_ne!(port, 0, "a captured port must be a real bound port");
            lines_checked += 1;
        }
        assert_eq!(
            lines_checked, 4,
            "expected all four committed captures to parse"
        );
        assert_eq!(
            parse_listening_line("opencode server listening on http://127.0.0.1:0"),
            None,
            "port 0 is not a real bound port"
        );
        assert_eq!(
            parse_listening_line("opencode server listening on http://0.0.0.0:4096"),
            None,
            "not the loopback host this adapter always requests"
        );
        assert_eq!(parse_listening_line("some other startup chatter"), None);
        assert_eq!(
            parse_listening_line("opencode server listening on http://127.0.0.1:not-a-port"),
            None
        );
    }

    // ------------------------------------------------------------ fingerprint

    #[test]
    fn compute_doc_fingerprint_matches_the_measured_constant() {
        let doc: Value = serde_json::from_str(OPENAPI_DOC).expect("fixture is valid JSON");
        let fingerprint = compute_doc_fingerprint(&doc).expect("every pinned operation is present");
        assert_eq!(fingerprint, MEASURED_DOC_FINGERPRINT);
    }

    #[test]
    fn compute_doc_fingerprint_refuses_a_missing_pinned_operation() {
        let mut doc: Value = serde_json::from_str(OPENAPI_DOC).expect("valid JSON");
        // Remove every operation named `session.create` from the document.
        if let Some(paths) = doc.get_mut("paths").and_then(Value::as_object_mut) {
            for methods in paths.values_mut() {
                if let Some(methods) = methods.as_object_mut() {
                    methods.retain(|_, op| {
                        op.get("operationId").and_then(Value::as_str) != Some("session.create")
                    });
                }
            }
        }
        let err = compute_doc_fingerprint(&doc).expect_err("session.create was removed");
        assert!(err.contains("session.create"));
    }

    #[test]
    fn the_openapi_doc_declares_no_security_scheme() {
        // Pins C6 as a test, not only a comment: if upstream ever documents
        // auth, this test tells us.
        let doc: Value = serde_json::from_str(OPENAPI_DOC).expect("valid JSON");
        assert_eq!(doc.get("security"), Some(&json!([])));
        assert!(
            doc.pointer("/components/securitySchemes")
                .is_none_or(Value::is_null),
            "if this ever names a scheme, SERVE_AUTH_USERNAME's load-bearing comment is stale"
        );
    }

    // ---------------------------------------------------------------- SSE bus

    #[test]
    fn the_sse_bus_is_global_and_is_filtered_by_session() {
        let frames = parse_sse_frames(SSE_SYNC_TURN);
        let ok: Vec<Value> = frames.into_iter().filter_map(Result::ok).collect();
        let plugin_added = ok
            .iter()
            .filter(|f| f.get("type").and_then(Value::as_str) == Some("plugin.added"))
            .count();
        assert_eq!(plugin_added, 45, "the fixture's own measured noise count");

        let session_id = "ses_fd129fa9dffeEu4VtdHFe5AH2X";
        let mut in_scope = 0;
        let mut out_of_scope = 0;
        for frame in &ok {
            let event_type = frame.get("type").and_then(Value::as_str).unwrap_or("");
            let properties = frame.get("properties").cloned().unwrap_or(Value::Null);
            if frame_in_scope(event_type, &properties, session_id) {
                in_scope += 1;
            } else {
                out_of_scope += 1;
            }
        }
        assert!(
            out_of_scope >= 45,
            "every plugin.added frame is out of scope"
        );
        assert!(in_scope > 0);
        // None of the 45 plugin.added frames may ever be counted in scope.
        for frame in &ok {
            if frame.get("type").and_then(Value::as_str) == Some("plugin.added") {
                let properties = frame.get("properties").cloned().unwrap_or(Value::Null);
                assert!(!frame_in_scope("plugin.added", &properties, session_id));
            }
        }
    }

    // -------------------------------------------------------- envelope gates

    fn role_map_from_sync_turn() -> BTreeMap<String, String> {
        let frames = parse_sse_frames(SSE_SYNC_TURN);
        let mut roles = BTreeMap::new();
        for frame in frames.into_iter().flatten() {
            if frame.get("type").and_then(Value::as_str) == Some("message.updated") {
                let properties = frame.get("properties").cloned().unwrap_or(Value::Null);
                if let (Some(id), Some(role)) = (
                    properties.pointer("/info/id").and_then(Value::as_str),
                    properties.pointer("/info/role").and_then(Value::as_str),
                ) {
                    roles.insert(id.to_string(), role.to_string());
                }
            }
        }
        roles
    }

    #[test]
    fn a_user_authored_text_part_never_becomes_an_assistant_event() {
        let roles = role_map_from_sync_turn();
        let frames = parse_sse_frames(SSE_SYNC_TURN);
        let mut saw_user_text_part = false;
        for frame in frames.into_iter().flatten() {
            if frame.get("type").and_then(Value::as_str) != Some("message.part.updated") {
                continue;
            }
            let properties = frame.get("properties").cloned().unwrap_or(Value::Null);
            let part = properties.get("part").cloned().unwrap_or(Value::Null);
            if part.get("text").and_then(Value::as_str) == Some("Reply with exactly: pong") {
                saw_user_text_part = true;
                let out = serve_part_envelope(&properties, |id| roles.get(id).cloned());
                assert!(out.is_none(), "the user's own prompt must never decode");
            }
        }
        assert!(
            saw_user_text_part,
            "fixture must carry the user's echoed prompt"
        );
    }

    #[test]
    fn an_unfinished_text_snapshot_is_not_decoded_twice() {
        let roles = role_map_from_sync_turn();
        let frames = parse_sse_frames(SSE_SYNC_TURN);
        let mut decoded_texts = Vec::new();
        for frame in frames.into_iter().flatten() {
            if frame.get("type").and_then(Value::as_str) != Some("message.part.updated") {
                continue;
            }
            let properties = frame.get("properties").cloned().unwrap_or(Value::Null);
            if let Some(envelope) = serve_part_envelope(&properties, |id| roles.get(id).cloned())
                && envelope.get("type").and_then(Value::as_str) == Some("text")
            {
                decoded_texts.push(
                    envelope
                        .pointer("/part/text")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                );
            }
        }
        assert_eq!(
            decoded_texts,
            vec!["pong".to_string()],
            "exactly one decoded assistant text, the finished snapshot, none for the empty one"
        );
    }

    #[test]
    fn a_pending_tool_snapshot_never_latches_an_empty_input() {
        let frames = parse_sse_frames(SSE_ABORT);
        let mut roles = BTreeMap::new();
        for frame in &frames {
            let Ok(frame) = frame else { continue };
            if frame.get("type").and_then(Value::as_str) == Some("message.updated") {
                let properties = frame.get("properties").cloned().unwrap_or(Value::Null);
                if let (Some(id), Some(role)) = (
                    properties.pointer("/info/id").and_then(Value::as_str),
                    properties.pointer("/info/role").and_then(Value::as_str),
                ) {
                    roles.insert(id.to_string(), role.to_string());
                }
            }
        }
        let mut decoded_tool_inputs = Vec::new();
        for frame in frames.into_iter().flatten() {
            if frame.get("type").and_then(Value::as_str) != Some("message.part.updated") {
                continue;
            }
            let properties = frame.get("properties").cloned().unwrap_or(Value::Null);
            if let Some(envelope) = serve_part_envelope(&properties, |id| roles.get(id).cloned())
                && envelope.get("type").and_then(Value::as_str) == Some("tool_use")
            {
                decoded_tool_inputs.push(
                    envelope
                        .pointer("/part/state/input")
                        .cloned()
                        .unwrap_or(Value::Null),
                );
            }
        }
        assert!(
            !decoded_tool_inputs.is_empty(),
            "at least one tool snapshot must decode"
        );
        for input in &decoded_tool_inputs {
            assert_ne!(
                input,
                &json!({}),
                "a pending snapshot's empty input must never decode"
            );
            assert_eq!(
                input.get("command").and_then(Value::as_str),
                Some("sleep 30 && echo done-sleeping")
            );
        }
    }

    #[test]
    fn reasoning_parts_are_counted_never_decoded() {
        let roles = role_map_from_sync_turn();
        let frames = parse_sse_frames(SSE_SYNC_TURN);
        for frame in frames.into_iter().flatten() {
            if frame.get("type").and_then(Value::as_str) != Some("message.part.updated") {
                continue;
            }
            let properties = frame.get("properties").cloned().unwrap_or(Value::Null);
            let part = properties.get("part").cloned().unwrap_or(Value::Null);
            if part.get("type").and_then(Value::as_str) == Some("reasoning") {
                assert!(serve_part_envelope(&properties, |id| roles.get(id).cloned()).is_none());
            }
        }
    }

    #[test]
    fn session_idle_is_archived_not_a_terminal() {
        // Replays the exact measured ordering (§4.3/C9): session.idle fires
        // twice and BEFORE the tool part's final `completed` snapshot.
        let frames = parse_sse_frames(SSE_ABORT);
        let ok: Vec<Value> = frames.into_iter().filter_map(Result::ok).collect();
        let types: Vec<&str> = ok
            .iter()
            .map(|f| f.get("type").and_then(Value::as_str).unwrap_or(""))
            .collect();
        assert_eq!(
            serve_event_view("session.idle"),
            ServeEventDisposition::Archived
        );
        let first_idle = types.iter().position(|t| *t == "session.idle");
        let last_completed_tool = ok.iter().rposition(|f| {
            f.get("type").and_then(Value::as_str) == Some("message.part.updated")
                && f.pointer("/properties/part/state/status")
                    .and_then(Value::as_str)
                    == Some("completed")
        });
        let (Some(first_idle), Some(last_completed_tool)) = (first_idle, last_completed_tool)
        else {
            panic!("fixture must carry both a session.idle and a completed tool snapshot");
        };
        assert!(
            first_idle < last_completed_tool,
            "session.idle fires before the tool's terminal snapshot in the measured trace -- an \
             implementation that settled the turn on session.idle would lose that evidence"
        );
    }

    // ------------------------------------------------------------ pending gate

    #[test]
    fn the_permission_asked_fixture_parks_the_stage_as_adapter_authored() {
        let frames = parse_sse_frames(SSE_PERMISSION_ASKED);
        let asked = frames
            .into_iter()
            .flatten()
            .find(|f| f.get("type").and_then(Value::as_str) == Some("permission.asked"))
            .expect("fixture carries a permission.asked frame");
        let properties = asked.get("properties").cloned().unwrap_or(Value::Null);
        let gate = PendingGate::from_permission_asked(&properties).expect("parses");
        match &gate {
            PendingGate::Permission {
                id,
                permission,
                patterns,
                call_id,
                ..
            } => {
                assert!(id.starts_with("per_"));
                assert_eq!(permission, "bash");
                assert_eq!(patterns, &vec!["echo probe-ask-42".to_string()]);
                assert!(!call_id.is_empty());
            }
            other => panic!("expected Permission, got {other:?}"),
        }
        let prompt = gate.prompt();
        assert!(prompt.contains("bash"));
        assert!(prompt.contains("once"));
        assert!(prompt.contains("always"));
        assert!(prompt.contains("reject"));
    }

    #[test]
    fn the_question_asked_fixture_parks_the_stage_as_actor_authored() {
        let frames = parse_sse_frames(SSE_QUESTION_ASKED);
        let asked = frames
            .into_iter()
            .flatten()
            .find(|f| f.get("type").and_then(Value::as_str) == Some("question.asked"))
            .expect("fixture carries a question.asked frame");
        let properties = asked.get("properties").cloned().unwrap_or(Value::Null);
        let gate = PendingGate::from_question_asked(&properties).expect("parses");
        match &gate {
            PendingGate::Question { id, call_id, .. } => {
                assert!(id.starts_with("que_"));
                assert!(!call_id.is_empty());
                // `que_` and `per_` prefixes must be wholly disjoint.
                assert!(!id.starts_with("per_"));
            }
            other => panic!("expected Question, got {other:?}"),
        }
        assert_eq!(
            parse_question_reply(&gate, "blue"),
            Ok(json!([["Blue"]])),
            "case-insensitive, trimmed, exact label match"
        );
        assert!(parse_question_reply(&gate, "purple").is_err());
    }

    #[test]
    fn a_permission_reply_accepts_exactly_three_values_and_refuses_the_rest() {
        for good in ["once", "Always", " reject "] {
            assert!(parse_permission_reply(good).is_ok());
        }
        let err = parse_permission_reply("approve").unwrap_err();
        assert!(err.contains("once"));
        assert!(err.contains("always"));
        assert!(err.contains("reject"));
    }

    #[test]
    fn a_question_reply_requires_an_exact_label_and_refuses_the_rest() {
        let gate = PendingGate::Question {
            id: "que_1".to_string(),
            questions: json!([{"question": "q", "options": [{"label": "Red"}, {"label": "Blue"}]}]),
            call_id: "call_1".to_string(),
        };
        assert_eq!(parse_question_reply(&gate, "Red"), Ok(json!([["Red"]])));
        let err = parse_question_reply(&gate, "Green").unwrap_err();
        assert!(err.contains("Red"));
        assert!(err.contains("Blue"));

        let multi = PendingGate::Question {
            id: "que_2".to_string(),
            questions: json!([{"options": []}, {"options": []}]),
            call_id: "call_2".to_string(),
        };
        assert!(parse_question_reply(&multi, "anything").is_err());
    }

    // ------------------------------------------------------------------ misc

    #[test]
    fn mint_server_password_is_not_trivially_guessable_or_reused() {
        let a = mint_server_password();
        let b = mint_server_password();
        assert_ne!(a, b);
        assert!(a.len() >= 40, "two concatenated ULIDs, well over 40 chars");
    }

    #[test]
    fn parse_sse_frames_counts_unparseable_data_never_decodes_it() {
        let frames = parse_sse_frames("data: not json\n\ndata: {\"type\":\"ok\"}\n\n");
        assert_eq!(frames.len(), 2);
        assert!(frames[0].is_err());
        assert!(frames[1].is_ok());
    }

    // -------------------------------------------------------- ServeHandle's
    // own client-error arms (W4 coverage lift). A real `opencode serve`
    // (or `StubServe`, its python stand-in in `tests/opencode_backend.rs`)
    // is always well-behaved on these paths in every other test in this
    // codebase, so no existing suite reaches a non-2xx status or a
    // malformed body from any of `ServeHandle`'s six HTTP methods -- this
    // is a bare `TcpListener` speaking exactly the one canned response each
    // test needs, with no HTTP semantics of its own beyond what the test
    // writes by hand, so a genuinely bad response is trivial to construct.

    /// Binds an ephemeral port, accepts exactly one connection, drains the
    /// request (best-effort -- this helper parses nothing), writes `response`
    /// verbatim, and closes. `response` must be a complete HTTP/1.1 response
    /// (status line, headers including `Content-Length` and `Connection:
    /// close`, body) -- this is socket plumbing only, not an HTTP stack.
    fn one_shot_http_server(response: String) -> String {
        use std::io::Write;
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind an ephemeral port");
        let addr = listener.local_addr().expect("local addr");
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                let mut buf = [0u8; 8192];
                // Draining before writing keeps a small POST body from
                // deadlocking against this thread's own write below on a
                // platform where the request hasn't fully arrived yet.
                let _ = stream.read(&mut buf);
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });
        format!("http://{addr}")
    }

    fn http_response(status_line: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
             Connection: close\r\n\r\n{body}",
            body.len()
        )
    }

    fn handle_against(response: String) -> super::ServeHandle {
        let base_url = one_shot_http_server(response);
        super::ServeHandle::new(base_url, "pw".to_string()).expect("build handle")
    }

    #[test]
    fn get_doc_names_the_username_rule_on_401() {
        let handle = handle_against(http_response("401 Unauthorized", ""));
        let err = handle
            .get_doc(Duration::from_secs(5))
            .expect_err("401 is refused with a named reason");
        assert!(err.contains("username"), "{err}");
    }

    #[test]
    fn get_doc_reports_a_non_2xx_status_that_is_not_401() {
        let handle = handle_against(http_response("500 Internal Server Error", ""));
        let err = handle
            .get_doc(Duration::from_secs(5))
            .expect_err("a non-2xx, non-401 status is still a refusal");
        assert!(err.contains("500"), "{err}");
    }

    #[test]
    fn get_doc_reports_when_the_body_is_not_json() {
        let handle = handle_against(http_response("200 OK", "not json"));
        let err = handle
            .get_doc(Duration::from_secs(5))
            .expect_err("a 2xx status with a non-JSON body is still a refusal");
        assert!(err.contains("not JSON"), "{err}");
    }

    #[test]
    fn create_session_reports_a_non_2xx_status() {
        let handle = handle_against(http_response("500 Internal Server Error", ""));
        let err = handle
            .create_session(Duration::from_secs(5))
            .expect_err("a non-2xx status is a refusal");
        assert!(err.contains("500"), "{err}");
    }

    #[test]
    fn create_session_reports_when_the_response_carries_no_id_field() {
        let handle = handle_against(http_response("200 OK", "{}"));
        let err = handle
            .create_session(Duration::from_secs(5))
            .expect_err("a response with no id is unusable, not silently accepted");
        assert!(err.contains("no `id` field") || err.contains("id"), "{err}");
    }

    #[test]
    fn get_messages_reports_a_non_2xx_status() {
        let handle = handle_against(http_response("404 Not Found", ""));
        let err = handle
            .get_messages("ses_x", Duration::from_secs(5))
            .expect_err("a non-2xx status is a refusal");
        assert!(err.contains("404"), "{err}");
    }

    #[test]
    fn post_message_reports_when_the_response_body_is_not_json() {
        let handle = handle_against(http_response("200 OK", "not json"));
        let err = handle
            .post_message("ses_x", &json!({}), Duration::from_secs(5))
            .expect_err("a 2xx status with a non-JSON body is still a refusal");
        match err {
            super::PostMessageError::Transport(detail) => {
                assert!(detail.contains("not JSON"), "{detail}");
            }
            other => panic!("expected a Transport error naming the JSON failure, got {other:?}"),
        }
    }

    #[test]
    fn post_abort_reports_a_non_2xx_status() {
        let handle = handle_against(http_response("500 Internal Server Error", ""));
        let err = handle
            .post_abort("ses_x", Duration::from_secs(5))
            .expect_err("a non-2xx status is a refusal");
        assert!(err.contains("500"), "{err}");
    }

    #[test]
    fn post_permission_reply_reports_a_non_2xx_status() {
        let handle = handle_against(http_response("404 Not Found", ""));
        let err = handle
            .post_permission_reply("ses_x", "per_1", "once", Duration::from_secs(5))
            .expect_err("a non-2xx status is a refusal");
        assert!(err.contains("404"), "{err}");
    }

    #[test]
    fn post_question_reply_reports_a_non_2xx_status() {
        let handle = handle_against(http_response("404 Not Found", ""));
        let err = handle
            .post_question_reply("que_1", &json!([["Blue"]]), Duration::from_secs(5))
            .expect_err("a non-2xx status is a refusal");
        assert!(err.contains("404"), "{err}");
    }
}
