//! Claude adapter: headless print-mode turns over a durable session (D2).
//!
//! Every claim in this module is backed by a measurement against the
//! installed CLI (Claude Code **2.1.226**, measured 2026-08-08 in this
//! container) or is explicitly marked documented-not-measured and fails
//! closed (LESSONS L1; the spike's own doctrine). The measured facts this
//! design rests on:
//!
//! - `claude -p --verbose --output-format stream-json`, prompt on stdin,
//!   emits one JSON object per line ending in a `type:"result"` envelope
//!   carrying `session_id`, `is_error`, `result`, `usage`, `modelUsage`,
//!   `api_error_status` (L2's production pattern, re-measured here).
//! - `--session-id <uuid>` pins the session identity **before launch**, and
//!   overrides the `CLAUDE_CODE_SESSION_ID` a nested environment injects
//!   (measured hazard: without it, a `claude` spawned from inside another
//!   Claude session silently adopts the parent's session id).
//! - Every turn carries its session id in its own argv (`--session-id` on the
//!   first turn, `--resume` after), which is what makes process liveness
//!   *evidence* rather than inference after a restart: a `/proc` scan for
//!   that argv shape answers "is a turn of this conversation still running"
//!   without any pid having been recorded, and without a recycled pid being
//!   mistakable for ours (see [`session_liveness`]). The scan matches the
//!   flag-and-value pair, never the id as a substring of a joined command
//!   line — a process that merely *quotes* the id (an operator reading the
//!   transcript, a shell wrapper) is not a running turn.
//! - `--resume <session_id>` continues the same conversation from a
//!   different process *and* a different cwd (measured: nonce set in turn 1
//!   recalled after resume from a sibling directory; `result.session_id`
//!   identical). Stream-json is per-invocation, not cumulative.
//! - A turn SIGKILLed mid-generation leaves **no result envelope** on
//!   stdout, and the conversation fully resumable (measured: nonce recalled
//!   on the next `--resume` turn). Interrupt = kill the per-turn process;
//!   the execution — the durable conversation — survives (§25 natively).
//! - Two concurrent print-mode sessions in one cwd complete independently
//!   with their own identities (measured) — no per-surface serialization is
//!   forced here.
//! - `--resume <nonexistent-uuid>` fails fast and free: exit 1, stderr
//!   `No conversation found with session ID: <uuid>`, a structured
//!   `subtype:"error_during_execution"` result envelope, zero tokens.
//! - The durable transcript lives at
//!   `<claude_home>/projects/<munged-cwd>/<session_id>.jsonl`; restart
//!   reconciliation gathers session-existence evidence by globbing for
//!   `<session_id>.jsonl` (filename only — the cwd-munging rule is *not*
//!   relied on). Private layout, so it stays an adapter detail (§16) and
//!   its absence fails closed, never open.
//! - Model pins: a provider-qualified value (`anthropic/...`) is accepted
//!   by the CLI and then fails **after launch** with `is_error:true`,
//!   `api_error_status:404` — and, measured on 2.1.226, that failing
//!   envelope says `subtype:"success"`. Exit codes and subtypes are never
//!   sufficient (L1); `is_error` and the model fields are what this adapter
//!   reads. A valid alias resolves visibly: `system:init.model` and the
//!   result's `modelUsage` keys carry the full resolved id.
//! - `--setting-sources` (MVP-2 D2 item 1, measured 2026-08-12) governs
//!   `.claude/settings*.json`-style configuration only — it does not gate
//!   whether the actor's agentic turn reads a memory file already sitting in
//!   its cwd. That native discovery is a separate, always-on mechanism keyed
//!   to the literal filename `CLAUDE.md`, orthogonal to this flag's value in
//!   every combination measured, and it never fires for a file named
//!   `AGENTS.md` (Sergeant's own instruction filename) at all. See
//!   [`setting_sources_args`] for the full measurement and what the flag
//!   actually widens under `InstructionPolicy::Local`.
//! - A SIGKILLed turn (`interrupt`/`stop`, `Child::kill()`) leaves no final
//!   `type:"result"` envelope — confirmed above. A SIGTERM sent to the same
//!   process (not a mechanism this adapter uses) is measured to behave
//!   differently: the CLI traps it, aborts the stream gracefully, and emits
//!   a terminal `result` envelope (`is_error:true`,
//!   `terminal_reason:"aborted_streaming"`) carrying real `usage`/
//!   `modelUsage`/`total_cost_usd` fields (MVP-2 D2 item 3, measured
//!   2026-08-12: one turn killed early with no assistant content yet
//!   streamed produced zero usable lines under SIGKILL; a second, killed
//!   after content began streaming, still ended with no envelope under
//!   SIGKILL but did carry a `usage` object on its last in-flight
//!   `assistant` chunk — a snapshot of that chunk's own accounting, not a
//!   cumulative turn total, and not currently parsed out by this adapter).
//! - Fact-finding only, no promise attached: recorded here and in
//!   `docs/environments/cerberus.md` because it is the honest answer to
//!   "what can interrupt actually tell us", not because anything in this
//!   milestone changes `interrupt`'s mechanism.
//!
//! **The one crash window this adapter does not close (stated, not papered
//! over).** START chooses the session id before spawning, so the identity
//! exists in adapter memory before the process does — but it becomes
//! *durable* only when the engine commits `execution.started`, which happens
//! after `start()` returns, and `start()` launches the first (token-burning)
//! turn inside itself. A daemon that dies between the spawn and that commit
//! leaves a live `claude` whose session id is in no journal: restart
//! reconciliation fails the work closed ("no execution to reconcile", pinned
//! by a test), but the orphan process and its transcript are not linked to
//! any record, and nothing here reaps them. Closing it properly needs a
//! two-phase START (reserve the identity, journal it, then launch) — a change
//! to the §15 trait and the engine's append order, which is more than this
//! milestone's contract asks for. Recorded rather than claimed closed.
//!
//! Root constraint (measured, historical): `--dangerously-skip-permissions`
//! is refused outright under root/sudo — `--dangerously-skip-permissions
//! cannot be used with root/sudo privileges for security reasons`, exit 1 —
//! unless the environment sets `IS_SANDBOX=1`. This adapter no longer emits
//! that flag at all (#47): it is recorded here because the refusal's stderr
//! shape is still what an envelope-less dead turn's evidence can look like
//! (`docs/gauntlet/runs/runB/run-manifest.md`, attempt 1), not because the
//! adapter still sends the flag.
//!
//! **Permission mode is profile configuration (#47).** `permission_mode` in
//! a profile's `options` passes through as the CLI's own `--permission-mode`
//! vocabulary (`default` | `acceptEdits` | `bypassPermissions` | `dontAsk` |
//! `plan` — [`crate::domain::profile::PermissionMode`]), validated at
//! profile load so an unrecognized string never reaches argv. Unspecified —
//! no `permission_mode` option at all — means **no permission flag on the
//! launch at all**, the CLI's own unflagged default; measured live via the
//! opt-in `bs2_default_mode_headless_turn_cannot_write_without_an_explicit_
//! permission_mode` test (`tests/m4_backends.rs`) rather than assumed.
//! `bypassPermissions` requires a profile to name it explicitly: sending
//! `--dangerously-skip-permissions` unconditionally, as this adapter used
//! to, made a security decision belong to the adapter instead of the human
//! who writes the profile.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use serde_json::{Value, json};

use super::{
    Backend, BackendError, BackendSignal, Capabilities, Completion, EventSink, ExecutionHandle,
    NativeEvent, NativeState, Observation, PreparedExecution, ProbeReport, ResumeRequest,
    RuntimeScope, StartRequest,
};
use crate::domain::event::{Event, EventDraft, EventSource};
use crate::domain::profile::Profile;
use crate::domain::workspace::InstructionPolicy;
use crate::runtime::blob::BlobStore;
use crate::runtime::graph::KIND_CONVERSATION_ASK;
use crate::runtime::journal::JournalError;

/// Name this backend registers under.
pub const CLAUDE_BACKEND_NAME: &str = "claude";

/// Minimum CLI version the M4 contract tests trust. The spike measured
/// 2.1.220; this milestone re-measured everything on 2.1.226, so 2.1.226 is
/// what "measured" means here. Older versions are refused, not assumed
/// compatible (L1: differences are findings, not surprises).
pub const MIN_TRUSTED_VERSION: (u64, u64, u64) = (2, 1, 226);

/// Flags the capability probe requires to appear in `claude --help`.
/// Each one is load-bearing for this adapter's launch grammar; a build
/// without one of them is a CLI we have never measured, so it is refused.
pub const REQUIRED_FLAGS: &[&str] = &[
    "--print",
    "--verbose",
    "--output-format",
    "--session-id",
    "--resume",
    "--setting-sources",
    "--model",
    "--permission-mode",
];

/// Environment variable naming the `claude` executable to use.
pub const CLAUDE_BIN_ENV: &str = "SGT_CLAUDE_BIN";

/// The stream-json `system` subtype carrying a turn's end-of-turn status
/// (measured on 2.1.226 — see [`actor_question`]).
pub const POST_TURN_SUMMARY_SUBTYPE: &str = "post_turn_summary";

/// The actor's question from one `post_turn_summary` line, when it asked one.
///
/// **Measured, 2.1.226** (`docs/gauntlet/notes/n3-claude-ask-measurement.md`),
/// two haiku turns in print-mode stream-json:
///
/// ```text
/// turn told to ask a question →
///   {"type":"system","subtype":"post_turn_summary",
///    "status_category":"blocked",
///    "status_detail":"Which database should I target: **postgres** or **sqlite**?",
///    "needs_action":"Which database should I target: **postgres** or **sqlite**?"}
/// turn told to answer and ask nothing →
///   {"type":"system","subtype":"post_turn_summary",
///    "status_category":"review_ready","status_detail":"user requested confirmation",
///    "needs_action":""}
/// ```
///
/// So `needs_action` is the discriminator, and it carries the actor's own
/// words. The rule here is deliberately the narrow one: a **non-empty
/// `needs_action` string** is an ask, and nothing else is. Two reasons for not
/// also keying on `status_category`:
///
/// - its vocabulary is unmeasured beyond the two values above, and L1's rule
///   is that an unmeasured field is not evidence;
/// - the two possible errors are not symmetric. Parking a stage that did not
///   really need parking costs a human one `respond`; completing a stage
///   whose actor was waiting for an answer is the silent-invention failure
///   GP-2 was filed about. The branch that fails closed is the one that
///   parks, so the narrow rule is the one that parks.
///
/// The category travels into the evidence regardless, because it is what an
/// operator reads when the decision looks wrong.
fn actor_question(summary: &Value) -> Option<String> {
    let question = summary.get("needs_action")?.as_str()?.trim();
    if question.is_empty() {
        return None;
    }
    Some(question.to_string())
}

/// Did this finished turn show that the CLI no longer speaks the ask grammar?
///
/// The narrow rule, for the same reason [`actor_question`] has one: only a
/// turn that ran to completion — a `type:"result"` envelope, not killed by
/// sergeant — is evidence about the *grammar*. An interrupted or crashed turn
/// legitimately has no `post_turn_summary`, and reading that as "the line is
/// gone" would withdraw a measured capability because a human hit cancel.
///
/// The distinction this makes is the one INV-N3-06 named: `post_turn_summary
/// == None` (no such line at all) and `Some({needs_action: ""})` (the line is
/// there and the actor asked nothing) were treated identically, so the
/// disappearance of the line was indistinguishable from a turn that ended
/// cleanly — which is the direction that fails open.
fn turn_lost_the_ask_grammar(outcome: &TurnOutcome) -> bool {
    outcome.envelope.is_some() && !outcome.interrupted && outcome.post_turn_summary.is_none()
}

/// Scan a journal replay for this adapter's own `ask`-withdrawal record
/// (`conversation.turn.grammar_unmeasured`, §27) and return the CLI version
/// the *most recent* one was journaled against, if any.
///
/// Pure with respect to the live CLI — it never runs a probe, only reads
/// events — which is what makes it independently testable (L13): feed it
/// constructed events, assert the version it picks, with no `claude` binary
/// anywhere in the loop. [`ClaudeBackend::apply_capability_provenance`] is
/// the half that compares this answer against a *live* measurement.
///
/// "Most recent" is by journal `seq`, not by payload timestamp or event
/// order in the iterator (a `Replay` already yields in seq order, but this
/// does not assume its caller does): the withdrawal is a one-way claim
/// (`note_ask_grammar` never re-raises it, so any recorded withdrawal for a
/// version implies every later one *for that version* would say the same
/// thing) and only the version matters to the caller, not which specific
/// occurrence. Events this adapter did not emit, or whose `capability` is
/// not `"ask"`, or that carry no `version` at all (a build predating this
/// field), are skipped rather than treated as evidence either way.
fn latest_ask_withdrawal_version<I>(events: I) -> Result<Option<String>, JournalError>
where
    I: IntoIterator<Item = Result<Event, JournalError>>,
{
    let mut latest: Option<(u64, String)> = None;
    for event in events {
        let event = event?;
        if event.source.source_type != "backend"
            || event.source.name != CLAUDE_BACKEND_NAME
            || event.kind != "conversation.turn.grammar_unmeasured"
            || event.payload.get("capability").and_then(Value::as_str) != Some("ask")
        {
            continue;
        }
        let Some(version) = event.payload.get("version").and_then(Value::as_str) else {
            continue;
        };
        if latest.as_ref().is_none_or(|(seq, _)| event.seq > *seq) {
            latest = Some((event.seq, version.to_string()));
        }
    }
    Ok(latest.map(|(_, version)| version))
}

/// Launch configuration for the adapter.
#[derive(Debug, Clone)]
pub struct ClaudeConfig {
    /// The CLI executable (a profile may override per execution).
    pub executable: PathBuf,
    /// Sergeant's data dir; raw per-turn transcripts are archived to its
    /// blob store (§20).
    pub data_dir: PathBuf,
    /// Where the CLI keeps durable session transcripts. `None` resolves to
    /// `$CLAUDE_CONFIG_DIR` or `~/.claude` at probe time. Tests point this
    /// at a scratch directory to fabricate restart evidence.
    pub claude_home: Option<PathBuf>,
    /// Extra environment for every spawned turn (e.g. `IS_SANDBOX=1` where
    /// the operator runs the daemon as root — see module docs).
    pub env: BTreeMap<String, String>,
}

impl ClaudeConfig {
    /// Config for a daemon owning `data_dir`, with the system `claude`.
    ///
    /// `SGT_CLAUDE_BIN` overrides which binary that is. It exists because
    /// "which `claude` is on the daemon's PATH" is an operator fact, not a
    /// compile-time one — a daemon started from a login shell, a service
    /// manager and a test harness can each see a different one, and `sgt
    /// doctor` has to be able to report on the same binary the daemon would
    /// actually run.
    pub fn new(data_dir: &Path) -> Self {
        Self {
            executable: std::env::var_os(CLAUDE_BIN_ENV)
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("claude")),
            data_dir: data_dir.to_path_buf(),
            claude_home: None,
            env: BTreeMap::new(),
        }
    }
}

/// Outcome of the registration-time capability/version probe.
#[derive(Debug, Clone)]
struct ProbeOutcome {
    /// Whether every gate passed.
    available: bool,
    /// Human-readable evidence: version found, or exactly which gate failed.
    detail: String,
    /// The canonical `major.minor.patch` this probe parsed from `--version`,
    /// when parsing got that far — even on a probe that later fails a gate
    /// (below [`MIN_TRUSTED_VERSION`], missing a required flag). Capability
    /// provenance (MVP-2 D2 item 2) keys a withdrawal to the exact version
    /// that showed it, so this is recorded independent of `available`.
    version: Option<String>,
}

/// The verdict of three-layer model-pin verification (adapted from the
/// spike to print mode). "Honored" requires positive evidence from the
/// result envelope's model fields; anything less is at best "attempted".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinVerdict {
    /// No pin was requested; nothing to verify.
    Unpinned,
    /// Positive evidence: a model field in the result envelope matches the
    /// requested pin. Carries the resolved full id.
    Honored(String),
    /// The result envelope names a model that does not match the pin — the
    /// silent-substitution failure the spike measured. Carries what ran.
    Substituted(String),
    /// The envelope carried no usable model evidence. The pin is recorded
    /// as attempted, never as honored (fail closed on evidence, but not a
    /// stage failure by itself).
    Attempted,
}

impl PinVerdict {
    /// Journal/evidence rendering.
    fn as_json(&self) -> Value {
        match self {
            PinVerdict::Unpinned => json!({"verdict": "unpinned"}),
            PinVerdict::Honored(model) => json!({"verdict": "honored", "model": model}),
            PinVerdict::Substituted(model) => json!({"verdict": "substituted", "ran": model}),
            PinVerdict::Attempted => json!({"verdict": "attempted"}),
        }
    }
}

/// Layer 1 of pin verification: the pre-flight shape check. Claude's
/// `--model` grammar has no provider qualification (measured: the CLI
/// accepts `anthropic/...` and then dies post-launch with a 404 envelope),
/// so a qualified value is refused *before* any process is spawned.
pub fn preflight_model_pin(model: &str) -> Result<(), String> {
    if model.trim().is_empty() {
        return Err("model pin is empty".to_string());
    }
    if model.contains('/') {
        return Err(format!(
            "model pin {model:?} is provider-qualified; Claude's --model grammar has no \
             provider syntax (measured on 2.1.226: such a pin launches and then fails with \
             api_error_status 404). Refused pre-flight."
        ));
    }
    Ok(())
}

/// Layer 3 of pin verification: substitution detection from the result
/// envelope's model fields. The result's `modelUsage` maps every model that
/// actually served the turn to usage records carrying `canonicalModel`
/// (measured shape). Exit codes and mission success are never consulted —
/// the spike proved both compatible with a dishonored pin.
pub fn verify_model_pin(requested: Option<&str>, envelope: &Value) -> PinVerdict {
    let Some(requested) = requested else {
        return PinVerdict::Unpinned;
    };
    let Some(usage) = envelope.get("modelUsage").and_then(Value::as_object) else {
        return PinVerdict::Attempted;
    };
    if usage.is_empty() {
        return PinVerdict::Attempted;
    }
    let matches_pin = |candidate: &str| -> bool {
        if candidate == requested || candidate.starts_with(requested) {
            return true;
        }
        // Bare alias ("haiku"): match a whole dash-separated segment of the
        // resolved id, so "haiku" matches "claude-haiku-4-5-20251001" but
        // never a coincidental substring of another family.
        !requested.contains('-') && candidate.split('-').any(|segment| segment == requested)
    };
    for (model_id, record) in usage {
        let canonical = record.get("canonicalModel").and_then(Value::as_str);
        if matches_pin(model_id) || canonical.is_some_and(matches_pin) {
            return PinVerdict::Honored(model_id.clone());
        }
    }
    let ran = usage.keys().cloned().collect::<Vec<_>>().join(", ");
    PinVerdict::Substituted(ran)
}

/// What one finished turn left behind.
#[derive(Debug, Clone)]
struct TurnOutcome {
    /// The parsed `type:"result"` envelope, when the turn produced one.
    /// A killed or crashed turn leaves none (measured).
    envelope: Option<Value>,
    /// Whether sergeant itself killed the turn (interrupt/stop). Without
    /// this bit, a missing envelope is ambiguity and fails closed.
    interrupted: bool,
    /// Blob ref of the archived raw stream-json transcript, when archiving
    /// succeeded.
    raw_blob: Option<String>,
    /// Why archiving failed, when it did. §20 evidence that could not be
    /// stored is reported, never dropped: the alternative is a turn whose
    /// raw capture silently does not exist.
    raw_error: Option<String>,
    /// Captured stderr, for evidence when things went wrong.
    stderr: String,
    /// The turn's last `system`/`post_turn_summary` line, verbatim.
    ///
    /// Measured on 2.1.226 (see the module docs and
    /// `docs/gauntlet/notes/n3-claude-ask-measurement.md`): print-mode
    /// stream-json emits exactly one of these per turn, and its
    /// `needs_action` field carries the actor's own question when the actor
    /// ended the turn asking one. This is the structured record GP-2's ask
    /// primitive needs; without it the adapter would be guessing from prose.
    post_turn_summary: Option<Value>,
}

impl TurnOutcome {
    /// How the §20 archive turned out, rendered for evidence. There is no
    /// value here that means "absent for reasons unknown": a ref, a named
    /// failure, or an explicitly empty stream.
    fn raw_evidence(&self) -> String {
        match (&self.raw_blob, &self.raw_error) {
            (Some(blob), _) => blob.clone(),
            (None, Some(error)) => format!("unarchived ({error})"),
            (None, None) => "unarchived (the turn streamed nothing)".to_string(),
        }
    }
}

/// Turn lifecycle for one execution. At most one turn is in flight per
/// conversation — `--resume` continues a session, it does not parallelize
/// one.
///
/// The two states that are *not* a turn outcome are separate variants on
/// purpose. Borrowing `Finished(TurnOutcome { envelope: None, interrupted:
/// true })` as a placeholder — which this adapter used to do at both START
/// and RESUME — makes OBSERVE state, in its own evidence string, that a turn
/// "was interrupted by request" when no interrupt was ever requested, and
/// report `BackendSignal::Running` for a conversation with no turn running.
/// That is a fabricated observation, and after a restart it is the
/// fail-*open* direction: the engine's Running branch makes no transition, so
/// the work sits `active` with nothing in flight and nothing to move it.
#[derive(Debug)]
enum TurnState {
    /// Registered, no turn launched yet: the window inside START between
    /// inserting the execution and spawning its first turn. START either
    /// spawns (replacing this) or removes the execution, so OBSERVE reaching
    /// it means something went wrong — which is why it is not silently
    /// mapped onto any turn outcome.
    Unlaunched,
    /// Re-adopted after a restart (§15 RESUME): this daemon launched no turn
    /// on this conversation, and the outcome of the turn that was in flight
    /// when the previous daemon died is not something it can read.
    Adopted,
    /// A per-turn process is running; the child handle is shared with
    /// `interrupt` so it can be killed without waiting for the reader.
    InFlight(Arc<Mutex<Child>>),
    /// The last turn finished (or was killed) and left this outcome.
    Finished(TurnOutcome),
}

/// One execution's resolved launch configuration (§14 applied to this CLI).
#[derive(Debug, Clone)]
struct LaunchConfig {
    executable: PathBuf,
    env: BTreeMap<String, String>,
    permission_args: Vec<String>,
}

/// What a `/proc` scan can say about a conversation's per-turn process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Liveness {
    /// A running process carries this session id in its argv. Positive
    /// evidence that a turn of this conversation is still alive.
    Alive(u32),
    /// No running process carries this session id. Positive evidence that no
    /// turn of this conversation is running.
    Dead,
    /// Liveness cannot be evidenced here (no `/proc`, or it is unreadable).
    /// §25: the caller fails closed; it never assumes either direction.
    Unknowable(String),
}

/// Does this NUL-separated `/proc/<pid>/cmdline` belong to a turn of
/// `session_id`?
///
/// The rule is deliberately narrow: some argv element must be exactly
/// `--session-id` or `--resume`, and the *next* element must be exactly the
/// session id. That is the launch grammar this adapter emits, and nothing
/// else. The wide rule — "the joined cmdline contains the id" — is what this
/// replaces, and it was a claim stronger than its evidence: an operator's
/// `less <session>.jsonl`, a `grep` for the id, or any harness that wraps
/// commands as `bash -c '<command text>'` (this project's own build
/// environment does) puts the id in *some* process's argv without any turn
/// running, and the adapter then reported `NativeState::Running` with the
/// evidence "pid N carries session id in argv". A quoted string is not a
/// running turn. Tokenizing also makes the false positive structurally
/// impossible rather than unlikely: a wrapper's whole command line is one
/// argv element, so it can never *be* the id, only contain it.
fn cmdline_names_session(cmdline: &[u8], session_id: &str) -> bool {
    let mut argv = cmdline
        .split(|byte| *byte == 0)
        .filter(|arg| !arg.is_empty())
        .map(String::from_utf8_lossy);
    while let Some(arg) = argv.next() {
        if (arg == "--session-id" || arg == "--resume")
            && argv.next().is_some_and(|value| value == session_id)
        {
            return true;
        }
    }
    false
}

/// Is a per-turn `claude` process for `session_id` still running?
///
/// Every turn this adapter launches names its session in its own argv
/// (`--session-id <uuid>` first, `--resume <uuid>` after), so scanning
/// `/proc/<pid>/cmdline` for *that argv shape* is evidence about **this
/// conversation** rather than about a pid — which matters precisely in the
/// case that needs it: after a restart, when no in-memory record survives and
/// a recorded pid (if anything had recorded one) could since have been
/// recycled. What is matched is the flag-and-value pair
/// ([`cmdline_names_session`]), not the id as a substring: the evidence this
/// function returns is quoted verbatim into an execution-state claim, so it
/// has to be about an execution.
///
/// Linux-only by construction. Elsewhere the answer is `Unknowable`, not a
/// guess: an adapter that reported "exited" from the absence of a mechanism
/// it does not have would be inventing execution state.
pub fn session_liveness(session_id: &str) -> Liveness {
    session_liveness_excluding(session_id, std::process::id())
}

/// [`session_liveness`], with the pid to skip made explicit.
///
/// The skip exists because sergeant's own process can carry a session id in
/// its argv (a `sgt` invocation naming one, a test binary) and must not
/// report itself as a live turn. It is a parameter so that the skip is
/// testable: a test can spawn one real stand-in turn and ask both questions
/// — excluding a bystander pid finds it, excluding the stand-in's own pid
/// does not — which is the only way to pin a rule about "self" from outside.
pub fn session_liveness_excluding(session_id: &str, skip_pid: u32) -> Liveness {
    if !cfg!(target_os = "linux") {
        return Liveness::Unknowable(
            "process liveness needs /proc; this platform has none".to_string(),
        );
    }
    let entries = match std::fs::read_dir("/proc") {
        Ok(entries) => entries,
        Err(e) => return Liveness::Unknowable(format!("/proc is unreadable: {e}")),
    };
    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        else {
            continue;
        };
        if pid == skip_pid {
            continue;
        }
        // A vanished process between readdir and read is not evidence of
        // anything; skip it. cmdline is NUL-separated argv.
        let Ok(cmdline) = std::fs::read(entry.path().join("cmdline")) else {
            continue;
        };
        if cmdline_names_session(&cmdline, session_id) {
            return Liveness::Alive(pid);
        }
    }
    Liveness::Dead
}

/// Adapter-side record of one execution (one durable conversation).
#[derive(Debug)]
struct ClaudeExecution {
    session_id: String,
    work_id: String,
    cwd: PathBuf,
    /// The requested model pin, kept for per-turn verification.
    model: Option<String>,
    /// Launch details pinned at start (profile executable/env overrides).
    executable: PathBuf,
    env: BTreeMap<String, String>,
    permission_args: Vec<String>,
    /// R-MVP1-4's resolved policy, pinned at launch (MVP-2 D2 item 1: the
    /// `--setting-sources` de-leak). Same rule as `executable`/`env`/
    /// `permission_args` above — resolved once at START and replayed
    /// unchanged on every subsequent turn of the conversation, never
    /// re-derived, so a manifest edit mid-flight cannot change what a
    /// running execution's turns launch with.
    instruction_policy: InstructionPolicy,
    /// Number of turns launched so far.
    turns: u32,
    turn: TurnState,
    stopped: bool,
    /// Interrupt was requested while a turn was in flight; consumed by the
    /// reader thread into `TurnOutcome::interrupted`.
    interrupt_requested: bool,
    /// The most recent turn's stdout reader thread.
    ///
    /// Kept so that STOP can *wait* for it (see [`ClaudeBackend::stop`]).
    /// Killing the child closes stdout, but the reader's remaining work —
    /// the §20 raw-transcript archive write and the `conversation.turn.ended`
    /// event that publishes its blob ref — happens after that, on this
    /// thread. Without a handle, "the turn is stopped" would be a claim about
    /// the process only, and the evidence would land at an unpredictable
    /// later moment (measured: a test's data dir was recreated by that write
    /// after the directory had been removed).
    reader: Option<std::thread::JoinHandle<()>>,
}

#[derive(Debug, Default)]
struct AdapterState {
    executions: BTreeMap<String, ClaudeExecution>,
}

/// The Claude backend.
pub struct ClaudeBackend {
    config: ClaudeConfig,
    probe_outcome: OnceLock<ProbeOutcome>,
    state: Arc<Mutex<AdapterState>>,
    sink: Mutex<Option<EventSink>>,
    /// Whether the installed CLI still emits the stream-json line
    /// [`Capabilities::ask`] rests on. Starts `true` — 2.1.226 was measured
    /// emitting one `system`/`post_turn_summary` per turn — and is lowered
    /// the first time a turn completes normally without one.
    ///
    /// The probe cannot check this: `--version` and `--help` say nothing
    /// about the stream grammar, and the only honest place to learn it is a
    /// turn. Before this existed the absence failed *open* — any later build
    /// that dropped, renamed or re-shaped the line silently restored the
    /// pre-N3 GP-2 behaviour (the stage completes carrying the unanswered
    /// question as its summary) with `ask: true` still on the wire.
    ask_grammar: Arc<AtomicBool>,
}

impl std::fmt::Debug for ClaudeBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClaudeBackend")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl ClaudeBackend {
    /// Build the adapter. Probing is lazy (first PROBE/START), so
    /// constructing one costs nothing on daemons that never route to it.
    pub fn new(config: ClaudeConfig) -> Self {
        Self {
            config,
            probe_outcome: OnceLock::new(),
            state: Arc::new(Mutex::new(AdapterState::default())),
            sink: Mutex::new(None),
            ask_grammar: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Whether this adapter has seen evidence that the installed CLI still
    /// speaks the ask grammar (§11.1's explicit semantic signal).
    ///
    /// Lowered — never raised — by [`turn_lost_the_ask_grammar`]: one
    /// completed turn with no `post_turn_summary` at all is enough to
    /// withdraw the claim, because the failure it guards against is silent.
    /// `sgt doctor` reads it through [`Backend::capabilities`].
    pub fn ask_grammar_intact(&self) -> bool {
        self.ask_grammar.load(Ordering::Relaxed)
    }

    /// MVP-2 D2 item 2 (capability provenance durability, cv2 item 7):
    /// re-derive [`Self::ask_grammar`]'s starting value from the journal's
    /// own record of a prior withdrawal, so a daemon restart does not
    /// silently re-optimism a capability this exact CLI version already
    /// proved absent.
    ///
    /// Before this existed, [`ClaudeBackend::new`] always started
    /// `ask_grammar` at `true`: a withdrawal recorded by a daemon that later
    /// restarted (or crashed) was visible only in the journal's history, not
    /// in the fresh process's in-memory claim, so `sgt run`'s R-MVP1-11
    /// preflight (`Engine::bind_stages`) would let a `requires_ask` stage
    /// back onto a CLI already measured not to speak the grammar — exactly
    /// once per restart, forever, because nothing ever asked the journal.
    ///
    /// Called once at daemon startup (`daemon.rs`, before the registration
    /// probe emits `capabilities` for the record), fed a full journal
    /// replay. Read-only: never raises the claim, matching
    /// [`Self::note_ask_grammar`]'s one-directional rule — a version bump
    /// makes an old record stale (the CLI that earned it no longer runs
    /// here) and reverts to the same measure-fresh optimism a brand new
    /// installation starts with, never to a claim this run has not itself
    /// measured or the journal has not itself evidenced *for this exact
    /// version*.
    pub fn seed_capability_provenance<I>(&self, events: I) -> Result<(), JournalError>
    where
        I: IntoIterator<Item = Result<Event, JournalError>>,
    {
        let latest = latest_ask_withdrawal_version(events)?;
        self.apply_capability_provenance(latest.as_deref());
        Ok(())
    }

    /// The pure half of [`Self::seed_capability_provenance`]: given the
    /// version a prior withdrawal was recorded against (if any), lower the
    /// claim only when it matches the version this probe measures *right
    /// now*. Split out from the journal-reading half so the decision itself
    /// — "does a stale-version record apply?" — is testable without a real
    /// `claude` binary on the test host driving `self.probe_outcome()`.
    fn apply_capability_provenance(&self, withdrawn_at_version: Option<&str>) {
        let Some(withdrawn_at_version) = withdrawn_at_version else {
            return;
        };
        if self.probe_outcome().version.as_deref() == Some(withdrawn_at_version) {
            self.ask_grammar.store(false, Ordering::Relaxed);
        }
    }

    /// Install the event sink normalized events are pushed through (§27).
    ///
    /// The daemon installs one as soon as its core exists and before it
    /// serves any request, which is before anything can start an execution.
    /// Events emitted with no sink installed are not delivered anywhere and
    /// are not kept: this adapter serves no HISTORY (see [`Backend::history`]),
    /// so a second in-memory copy would be an unbounded buffer with no
    /// reader.
    pub fn set_event_sink(&self, sink: EventSink) {
        *self.sink.lock().expect("claude sink lock") = Some(sink);
    }

    /// The session-evidence root: config override, else `$CLAUDE_CONFIG_DIR`,
    /// else `~/.claude`.
    fn claude_home(&self) -> PathBuf {
        if let Some(home) = &self.config.claude_home {
            return home.clone();
        }
        if let Ok(dir) = std::env::var("CLAUDE_CONFIG_DIR") {
            return PathBuf::from(dir);
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        PathBuf::from(home).join(".claude")
    }

    /// Find the durable transcript for a session, if one exists:
    /// `<claude_home>/projects/*/<session_id>.jsonl`. Only the filename
    /// convention is relied on; the per-cwd directory munging is not.
    fn session_transcript(&self, session_id: &str) -> Option<PathBuf> {
        let projects = self.claude_home().join("projects");
        let entries = std::fs::read_dir(projects).ok()?;
        let wanted = format!("{session_id}.jsonl");
        for entry in entries.flatten() {
            let candidate = entry.path().join(&wanted);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    }

    /// Run the capability/version probe once and cache the outcome.
    ///
    /// Both probes are offline and token-free: `--version` and `--help`.
    /// Gates, in order: the binary runs; the version parses; the version is
    /// at least [`MIN_TRUSTED_VERSION`]; every [`REQUIRED_FLAGS`] entry
    /// appears in `--help`. Any failure names itself — a refusal an
    /// operator cannot act on is not a refusal, it is an outage.
    fn probe_outcome(&self) -> &ProbeOutcome {
        self.probe_outcome.get_or_init(|| self.run_probe())
    }

    fn run_probe(&self) -> ProbeOutcome {
        let exe = &self.config.executable;
        let version_out = match Command::new(exe).arg("--version").output() {
            Ok(out) => out,
            Err(e) => {
                return ProbeOutcome {
                    available: false,
                    detail: format!(
                        "capability probe: cannot run {exe:?} --version: {e} (kind: {:?})",
                        e.kind()
                    ),
                    version: None,
                };
            }
        };
        let version_text = String::from_utf8_lossy(&version_out.stdout)
            .trim()
            .to_string();
        let Some(version) = parse_version(&version_text) else {
            return ProbeOutcome {
                available: false,
                detail: format!(
                    "capability probe: cannot parse a version from {exe:?} --version output \
                     {version_text:?}; refusing an unmeasurable CLI"
                ),
                version: None,
            };
        };
        let canonical_version = format!("{}.{}.{}", version.0, version.1, version.2);
        if version < MIN_TRUSTED_VERSION {
            return ProbeOutcome {
                available: false,
                detail: format!(
                    "capability probe: version {}.{}.{} is below the minimum trusted \
                     {}.{}.{} (the version these contract tests measured)",
                    version.0,
                    version.1,
                    version.2,
                    MIN_TRUSTED_VERSION.0,
                    MIN_TRUSTED_VERSION.1,
                    MIN_TRUSTED_VERSION.2
                ),
                version: Some(canonical_version),
            };
        }
        let help_out = match Command::new(exe).arg("--help").output() {
            Ok(out) => out,
            Err(e) => {
                return ProbeOutcome {
                    available: false,
                    detail: format!(
                        "capability probe: cannot run {exe:?} --help: {e} (kind: {:?})",
                        e.kind()
                    ),
                    version: Some(canonical_version),
                };
            }
        };
        let help_text = String::from_utf8_lossy(&help_out.stdout).to_string();
        let missing: Vec<&str> = REQUIRED_FLAGS
            .iter()
            .copied()
            .filter(|flag| !help_text.contains(flag))
            .collect();
        if !missing.is_empty() {
            return ProbeOutcome {
                available: false,
                detail: format!(
                    "capability probe: {exe:?} --help (version {version_text}) is missing \
                     required flag(s) {}; this launch grammar was never measured against it",
                    missing.join(", ")
                ),
                version: Some(canonical_version),
            };
        }
        ProbeOutcome {
            available: true,
            detail: format!(
                "claude {version_text}; all {} required flags present",
                REQUIRED_FLAGS.len()
            ),
            version: Some(canonical_version),
        }
    }

    /// Resolve one execution's launch configuration from adapter config plus
    /// the profile (§14). One function, used by both START and RESUME, so a
    /// re-adopted execution cannot end up launching under different rules
    /// than the one it re-adopts — in particular, cannot silently escalate
    /// past a profile-pinned `--permission-mode`.
    ///
    /// #47: the permission mode is validated here too (defense in depth
    /// alongside the profile-load check in
    /// [`crate::domain::workspace::Workspace::from_config`]) — a `Profile`
    /// can reach this adapter without ever having passed through a
    /// `sergeant.toml` parse (built directly by a test, or by a future
    /// caller), so the launch boundary itself must still refuse an
    /// unrecognized mode rather than pass the raw string to the CLI.
    fn launch_config(&self, profile: Option<&Profile>) -> Result<LaunchConfig, BackendError> {
        let executable = profile
            .and_then(|p| p.executable.clone())
            .unwrap_or_else(|| self.config.executable.clone());
        let mut env = self.config.env.clone();
        if let Some(profile) = profile {
            for (key, value) in &profile.env {
                env.insert(key.clone(), value.clone());
            }
            if let Some(config_home) = &profile.config_home {
                env.insert(
                    "CLAUDE_CONFIG_DIR".to_string(),
                    config_home.display().to_string(),
                );
            }
        }
        // Permission mode is profile-pinned. Unspecified -> no flag at all
        // (the CLI's own default); `bypassPermissions` only ever reaches
        // argv when a profile names it (module docs, #47).
        let permission_args = match profile {
            Some(profile) => profile
                .permission_mode()
                .map_err(|e| self.err_failed(format!("profile {:?}: {e}", profile.name)))?
                .unwrap_or(crate::domain::profile::PermissionMode::Default)
                .cli_args(),
            None => Vec::new(),
        };
        Ok(LaunchConfig {
            executable,
            env,
            permission_args,
        })
    }

    /// Execution ids this adapter currently holds state for.
    ///
    /// The diagnostic answer to "did that refused START leave a phantom
    /// execution behind?" — a question the identity-checked verbs cannot
    /// answer, because a phantom's session id was never handed out.
    pub fn tracked_executions(&self) -> Vec<String> {
        self.lock().executions.keys().cloned().collect()
    }

    fn err_failed(&self, detail: impl Into<String>) -> BackendError {
        BackendError::Failed {
            backend: CLAUDE_BACKEND_NAME.to_string(),
            detail: detail.into(),
        }
    }

    fn err_unknown(&self, execution_id: &str) -> BackendError {
        BackendError::UnknownExecution {
            backend: CLAUDE_BACKEND_NAME.to_string(),
            execution_id: execution_id.to_string(),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, AdapterState> {
        self.state.lock().expect("claude adapter state lock")
    }

    /// §25's identity rule, same as the fake's: an execution is resolved by
    /// sergeant's id *and* the native (session) identity the handle carries.
    fn check_identity(
        &self,
        state: &AdapterState,
        handle: &ExecutionHandle,
    ) -> Result<(), BackendError> {
        let execution = state
            .executions
            .get(&handle.execution_id)
            .ok_or_else(|| self.err_unknown(&handle.execution_id))?;
        if handle.native_id.as_deref() != Some(execution.session_id.as_str()) {
            return Err(self.err_unknown(&handle.execution_id));
        }
        Ok(())
    }

    /// Push one normalized event through the sink, when one is installed.
    ///
    /// There is deliberately no second copy kept per execution (R1). This
    /// adapter does not serve HISTORY — see [`Backend::history`] below — so a
    /// per-execution `Vec<NativeEvent>` would be an unbounded buffer, growing
    /// for the daemon's lifetime with a duplicate of every event the sink
    /// already journals durably, for a reader that does not exist. Events
    /// emitted before the daemon installs a sink are not delivered anywhere;
    /// the daemon installs one before it serves any request, so nothing that
    /// can start an execution runs before it.
    fn emit(&self, execution_id: &str, work_id: &str, kind: &str, payload: Value) {
        let sink = self.sink.lock().expect("claude sink lock").clone();
        if let Some(sink) = sink {
            let draft = EventDraft {
                source: EventSource::new("backend", CLAUDE_BACKEND_NAME),
                workspace_id: None,
                work_id: Some(work_id.to_string()),
                execution_id: Some(execution_id.to_string()),
                // Correlation groups the logical operation: this execution.
                correlation_id: Some(execution_id.to_string()),
                // Causation is chained by the committing sink (it alone sees
                // journal-assigned event ids).
                causation_id: None,
                kind: kind.to_string(),
                payload,
            };
            sink(draft);
        }
    }

    /// Spawn one turn for an execution already registered in adapter state.
    ///
    /// The caller has set `turn` expectations; this builds the command from
    /// the execution's pinned launch details, spawns it, hands stdin to a
    /// writer thread, and hands stdout to the reader thread that ingests
    /// stream-json, archives the raw transcript, and records the outcome.
    fn spawn_turn(&self, execution_id: &str, prompt: String) -> Result<(), BackendError> {
        let (
            exe,
            cwd,
            env,
            permission_args,
            session_id,
            model,
            first_turn,
            work_id,
            instruction_policy,
        ) = {
            let state = self.lock();
            let execution = state
                .executions
                .get(execution_id)
                .ok_or_else(|| self.err_unknown(execution_id))?;
            (
                execution.executable.clone(),
                execution.cwd.clone(),
                execution.env.clone(),
                execution.permission_args.clone(),
                execution.session_id.clone(),
                execution.model.clone(),
                execution.turns == 0,
                execution.work_id.clone(),
                execution.instruction_policy,
            )
        };

        let mut command = Command::new(&exe);
        command
            .arg("-p")
            .arg("--verbose")
            .args(["--output-format", "stream-json"])
            // MVP-2 D2 item 1 (the `--setting-sources` de-leak): R-MVP1-4's
            // pinned policy translates to the CLI's own setting-sources
            // grammar. See `setting_sources_args` for what this flag is
            // measured to actually control — it is narrower than L2's
            // original "project memory" framing (module docs).
            .args(setting_sources_args(instruction_policy));
        if first_turn {
            // Session identity is chosen by sergeant before the process
            // exists, so the handle START returns already names the
            // conversation the engine will journal (module docs are explicit
            // about what that does and does not close).
            command.args(["--session-id", &session_id]);
        } else {
            command.args(["--resume", &session_id]);
        }
        if let Some(model) = &model {
            command.args(["--model", model]);
        }
        for arg in &permission_args {
            command.arg(arg);
        }
        command
            .current_dir(&cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // Measured hazard: a nested environment's session identity must
            // never leak into an execution sergeant owns.
            .env_remove("CLAUDE_CODE_SESSION_ID");
        for (key, value) in &env {
            command.env(key, value);
        }

        let mut child = command
            .spawn()
            .map_err(|e| self.err_failed(format!("cannot spawn {exe:?}: {e}")))?;

        let stdin = child.stdin.take();
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| self.err_failed("child stdout was not piped"))?;
        let stderr = child.stderr.take();

        // Prompt goes to stdin from its own thread: a CONTEXT.md larger than
        // the pipe buffer must not deadlock the spawn path.
        let stdin_prompt = prompt.clone();
        std::thread::spawn(move || {
            if let Some(mut stdin) = stdin {
                let _ = stdin.write_all(stdin_prompt.as_bytes());
            }
        });
        // stderr is *delivered*, not deposited in a buffer someone snapshots.
        //
        // Both pipes reach EOF at the same instant — the turn process exits —
        // so a stdout reader that closes and immediately reads a shared stderr
        // buffer is racing the thread that fills it. Measured on this host
        // while pinning issue #46's attempt-1 shape: an envelope-less turn's
        // stderr was intermittently empty in the journal, which loses the one
        // thing the operator has to act on (Run B attempt 1's whole content
        // was a stderr line and no stream-json at all). A channel makes the
        // handoff a synchronization instead of a hope.
        let stderr_rx = stderr.map(|mut stderr| {
            let (tx, rx) = std::sync::mpsc::sync_channel::<String>(1);
            std::thread::spawn(move || {
                let mut text = String::new();
                let _ = stderr.read_to_string(&mut text);
                let _ = tx.send(text);
            });
            rx
        });

        let child = Arc::new(Mutex::new(child));
        {
            let mut state = self.lock();
            let execution = state
                .executions
                .get_mut(execution_id)
                .ok_or_else(|| self.err_unknown(execution_id))?;
            execution.turn = TurnState::InFlight(Arc::clone(&child));
            execution.turns += 1;
            execution.interrupt_requested = false;
        }
        self.emit(
            execution_id,
            &work_id,
            "conversation.user",
            json!({"text": prompt, "session_id": session_id}),
        );

        let reader = TurnReader {
            backend_state: Arc::clone(&self.state),
            ask_grammar: Arc::clone(&self.ask_grammar),
            version: self.probe_outcome().version.clone(),
            sink: self.sink.lock().expect("claude sink lock").clone(),
            data_dir: self.config.data_dir.clone(),
            execution_id: execution_id.to_string(),
            work_id,
            session_id,
            model,
            child,
            stderr_rx,
        };
        let reader = std::thread::spawn(move || reader.run(stdout));
        // Recorded after the spawn, and deliberately not fatal if the
        // execution has gone away in the meantime: the handle exists so STOP
        // can wait for the archive write, not so anything can depend on the
        // thread's identity. A previous turn's handle is replaced here —
        // that turn is `Finished`, its evidence already written.
        if let Some(execution) = self.lock().executions.get_mut(execution_id) {
            execution.reader = Some(reader);
        }
        Ok(())
    }
}

/// Everything the per-turn stdout reader thread needs. It owns turn
/// ingestion end to end: raw archive, normalization, outcome recording.
struct TurnReader {
    backend_state: Arc<Mutex<AdapterState>>,
    /// The adapter's ask-grammar claim, lowered from here when a turn
    /// completes without the line it rests on.
    ask_grammar: Arc<AtomicBool>,
    /// The canonical version this backend's registration probe measured
    /// (`ProbeOutcome::version`), carried down so a withdrawal this reader
    /// journals names the exact version that showed it (MVP-2 D2 item 2) —
    /// without this, `conversation.turn.grammar_unmeasured` events would be
    /// provenance with no key to seed a later restart from. `None` only when
    /// the probe itself could not parse a version at all (an unmeasurable
    /// CLI, refused elsewhere in that path already).
    version: Option<String>,
    sink: Option<EventSink>,
    data_dir: PathBuf,
    execution_id: String,
    work_id: String,
    session_id: String,
    model: Option<String>,
    child: Arc<Mutex<Child>>,
    /// The turn's stderr, delivered once the reading thread hits EOF. `None`
    /// when the spawn produced no stderr pipe at all.
    stderr_rx: Option<std::sync::mpsc::Receiver<String>>,
}

/// How long the turn reader waits for stderr after the turn's process has
/// been reaped.
///
/// The wait is normally instantaneous — the pipe closed when the process did,
/// so the reading thread is already at EOF — and this bound exists only for
/// the pathological case that could otherwise stall a turn's outcome forever:
/// a descendant of the turn that inherited stderr and outlived its parent.
/// Losing the evidence is bad; never recording the turn's outcome is issue
/// #46 again, so the ambiguity is bounded rather than waited out.
const STDERR_DRAIN_BUDGET: std::time::Duration = std::time::Duration::from_secs(5);

impl TurnReader {
    fn run(self, stdout: std::process::ChildStdout) {
        let mut raw = String::new();
        let mut envelope: Option<Value> = None;
        let mut summary: Option<Value> = None;
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
            raw.push_str(&line);
            raw.push('\n');
            if let Ok(value) = serde_json::from_str::<Value>(&line) {
                self.ingest_line(&value, &mut envelope, &mut summary);
            }
        }
        // Stdout is closed; the process has exited or is about to. Reap it.
        // The lock is only taken *after* EOF so `interrupt` can always kill.
        let _ = self.child.lock().expect("turn child lock").wait();

        // §20: the raw stream-json transcript — every line, verbatim — is
        // archived before any conclusion is drawn from it. An archive that
        // fails (full disk, unwritable store) is *reported*, not swallowed:
        // the turn's own outcome is not made ambiguous by a storage failure,
        // but no observer may be left thinking the bytes are on disk.
        let (raw_blob, raw_error) = if raw.is_empty() {
            (None, None)
        } else {
            match BlobStore::open(&self.data_dir).and_then(|store| store.put(raw.as_bytes())) {
                Ok(blob_ref) => (Some(blob_ref.to_string()), None),
                Err(e) => (None, Some(e.to_string())),
            }
        };

        // The child is reaped, so its end of the stderr pipe is closed and the
        // reading thread is at (or microseconds from) EOF. Waiting for its
        // delivery is what makes the evidence deterministic; the budget is
        // what keeps a leaked descendant from turning "no stderr" into "no
        // outcome".
        let stderr = self
            .stderr_rx
            .as_ref()
            .and_then(|rx| rx.recv_timeout(STDERR_DRAIN_BUDGET).ok())
            .unwrap_or_default();
        let mut state = self
            .backend_state
            .lock()
            .expect("claude adapter state lock");
        let Some(execution) = state.executions.get_mut(&self.execution_id) else {
            return;
        };
        let interrupted = execution.interrupt_requested;
        let outcome = TurnOutcome {
            envelope: envelope.clone(),
            interrupted,
            raw_blob: raw_blob.clone(),
            raw_error: raw_error.clone(),
            stderr: stderr.clone(),
            post_turn_summary: summary.clone(),
        };
        execution.turn = TurnState::Finished(outcome.clone());
        drop(state);

        // L1/L8 at runtime: a completed turn that carried no
        // `post_turn_summary` is the CLI telling this adapter that the
        // evidence `Capabilities::ask` rests on is gone.
        self.note_ask_grammar(&outcome);

        // Every turn ends with this event, whatever it left behind. It is
        // the only place the §20 blob ref reaches the journal for a turn
        // that produced no result envelope — the interrupted and crashed
        // cases — and archiving those turns while leaving the ref
        // unreachable would be capture nobody can resolve.
        self.emit(
            "conversation.turn.ended",
            json!({
                "session_id": self.session_id,
                "interrupted": interrupted,
                "result_envelope": envelope.is_some(),
                "raw": raw_blob,
                "raw_error": raw_error,
                "stderr": truncate(&stderr, 400),
            }),
        );

        // The turn summary event: usage, cost, model-pin verdict, raw blob
        // ref. Emitted after the outcome is recorded so an observer that
        // reacts to the event finds the state already settled.
        if let Some(envelope) = &envelope {
            let verdict = verify_model_pin(self.model.as_deref(), envelope);
            self.emit(
                "usage.updated",
                json!({
                    "session_id": self.session_id,
                    "usage": envelope.get("usage").cloned().unwrap_or(Value::Null),
                    "total_cost_usd": envelope.get("total_cost_usd").cloned().unwrap_or(Value::Null),
                    "model_usage": envelope.get("modelUsage").cloned().unwrap_or(Value::Null),
                    "model_pin": verdict.as_json(),
                    "is_error": envelope.get("is_error").cloned().unwrap_or(Value::Null),
                    "raw": raw_blob,
                    "raw_error": raw_error,
                }),
            );
        }
    }

    /// Normalize one stream-json line into §27 events. System lines are
    /// left to the raw archive: they are vendor plumbing, not conversation.
    fn ingest_line(
        &self,
        value: &Value,
        envelope: &mut Option<Value>,
        summary: &mut Option<Value>,
    ) {
        match value.get("type").and_then(Value::as_str) {
            // §11.1's explicit semantic signal, as 2.1.226 actually emits it.
            // Only this subtype is read; the rest of the `system` stream stays
            // vendor plumbing that lives in the raw archive.
            Some("system")
                if value.get("subtype").and_then(Value::as_str)
                    == Some(POST_TURN_SUMMARY_SUBTYPE) =>
            {
                if let Some(question) = actor_question(value) {
                    // The actor's own words, journaled as the actor's — a
                    // separate fact from the engine's `stage.needs_input`,
                    // which is sergeant's decision about what to do with it.
                    self.emit(
                        KIND_CONVERSATION_ASK,
                        json!({
                            "session_id": self.session_id,
                            "question": question,
                            "status_category": value.get("status_category").cloned().unwrap_or(Value::Null),
                        }),
                    );
                }
                *summary = Some(value.clone());
            }
            Some("assistant") => {
                let content = value
                    .pointer("/message/content")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let mut text = String::new();
                for block in &content {
                    match block.get("type").and_then(Value::as_str) {
                        Some("text") => {
                            if let Some(t) = block.get("text").and_then(Value::as_str) {
                                text.push_str(t);
                            }
                        }
                        Some("tool_use") => {
                            self.emit(
                                "tool.requested",
                                json!({
                                    "session_id": self.session_id,
                                    "id": block.get("id").cloned().unwrap_or(Value::Null),
                                    "name": block.get("name").cloned().unwrap_or(Value::Null),
                                    "input": block.get("input").cloned().unwrap_or(Value::Null),
                                }),
                            );
                        }
                        _ => {}
                    }
                }
                if !text.is_empty() {
                    self.emit(
                        "conversation.assistant.completed",
                        json!({"session_id": self.session_id, "text": text}),
                    );
                }
            }
            Some("user") => {
                // In stream-json, `user` lines mid-turn carry tool results.
                let content = value
                    .pointer("/message/content")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                for block in &content {
                    if block.get("type").and_then(Value::as_str) == Some("tool_result") {
                        self.emit(
                            "tool.completed",
                            json!({
                                "session_id": self.session_id,
                                "tool_use_id": block.get("tool_use_id").cloned().unwrap_or(Value::Null),
                                "is_error": block.get("is_error").cloned().unwrap_or(Value::Null),
                            }),
                        );
                    }
                }
            }
            Some("result") => {
                *envelope = Some(value.clone());
            }
            _ => {}
        }
    }

    /// Withdraw [`Capabilities::ask`] if this turn showed the grammar it
    /// rests on has gone, and say so once, where a reader will find it.
    fn note_ask_grammar(&self, outcome: &TurnOutcome) {
        if !turn_lost_the_ask_grammar(outcome) {
            return;
        }
        // Only the first one matters: the claim is already withdrawn after
        // that, and an event per turn thereafter would be noise.
        if !self.ask_grammar.swap(false, Ordering::Relaxed) {
            return;
        }
        tracing::warn!(
            execution_id = %self.execution_id,
            "the installed claude CLI completed a turn with no system/post_turn_summary \
             line; withdrawing the `ask` capability"
        );
        self.emit(
            "conversation.turn.grammar_unmeasured",
            json!({
                "capability": "ask",
                "expected": format!("system/{POST_TURN_SUMMARY_SUBTYPE}"),
                "session_id": self.session_id,
                // MVP-2 D2 item 2: the CLI version this withdrawal was
                // measured against, so a later daemon start can tell whether
                // the record still applies (`latest_ask_withdrawal_version`)
                // rather than blindly re-lowering a claim about a CLI that
                // has since been upgraded out from under it.
                "version": self.version,
                "detail":
                    "a turn completed with a result envelope and no post_turn_summary line; \
                     the evidence `Capabilities::ask` rests on is not present in this CLI's \
                     stream, so the claim is withdrawn rather than left failing open",
                "raw": outcome.raw_blob,
            }),
        );
    }

    fn emit(&self, kind: &str, payload: Value) {
        if let Some(sink) = &self.sink {
            sink(EventDraft {
                source: EventSource::new("backend", CLAUDE_BACKEND_NAME),
                workspace_id: None,
                work_id: Some(self.work_id.clone()),
                execution_id: Some(self.execution_id.clone()),
                correlation_id: Some(self.execution_id.clone()),
                causation_id: None,
                kind: kind.to_string(),
                payload,
            });
        }
    }
}

impl Backend for ClaudeBackend {
    fn name(&self) -> &str {
        CLAUDE_BACKEND_NAME
    }

    /// Capabilities as measured on 2.1.226. Every `true` below names a
    /// behaviour this module implements and this milestone measured (module
    /// docs carry the measurements). The `false`s are equally deliberate:
    /// `native_background` — D2 does not drive `--bg` (the spike's
    /// supervisor design is Sergeant-specific); `approval_flow` — permission
    /// modes are launch configuration here, not an interactive approval
    /// surface; `human_attach` — nothing holds a TTY to attach to;
    /// `native_subagents` — the CLI does spawn its own subagents, but this
    /// adapter has measured nothing about how they surface in print-mode
    /// stream-json, and an unmeasured capability is `false` until a
    /// measurement flips it (§15: unsupported means unsupported, and
    /// "documented" is not "supported").
    ///
    /// `history` is `false` for that last reason, and the reason is worth
    /// stating because it used to be `true`: the capability means *durable*
    /// native history retrieval, and the only durable native history here is
    /// the CLI's private `<session_id>.jsonl` transcript, whose record format
    /// this milestone never measured (§16 keeps such layouts adapter details;
    /// this adapter only ever asks whether the file exists). What the adapter
    /// could return instead — the events its own process happened to ingest —
    /// is a partial answer that reads exactly like a complete one, and after
    /// a RESUME it is empty for a conversation with a full transcript on
    /// disk: "nothing was said" and "this daemon was not here" would be the
    /// same value. So the claim is `false` and [`Backend::history`] refuses.
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            persistent_sessions: true,
            native_background: false,
            streaming: true,
            history: false,
            resume: true,
            interrupt: true,
            model_selection: true,
            profiles: true,
            approval_flow: false,
            human_attach: false,
            usage: true,
            native_subagents: false,
            // GP-2, measured on 2.1.226 rather than assumed: print-mode
            // stream-json emits one `system`/`post_turn_summary` per turn
            // whose `needs_action` field carries the actor's own question
            // when the actor ended the turn asking one, and is the empty
            // string when it did not (see `actor_question` for the two
            // measured records, and the opt-in contract test that re-measures
            // them against the installed CLI). That is a structured record,
            // not an inference from prose, so the claim is `true` — and it is
            // the only reason it is: without that line this would be `false`
            // and every ask would silently complete its stage. Which is why
            // it is a *runtime* answer rather than a constant — the probe
            // cannot see the stream grammar, so the first turn that completes
            // without the line withdraws the claim (`ask_grammar_intact`).
            ask: self.ask_grammar_intact(),
        }
    }

    /// §17: print-mode Claude has no backend-level service. An execution is a
    /// durable conversation and each turn is its own short-lived process, so
    /// the runtime this adapter needs comes into being per execution and
    /// leaves with it. (§17's own Claude example — "native supervisor/session
    /// infrastructure" — describes the `--bg` supervisor model D2 measured
    /// and did not take; scope follows the design that shipped, not the
    /// example.)
    fn runtime_scope(&self) -> RuntimeScope {
        RuntimeScope::PerExecution
    }

    fn probe(&self) -> ProbeReport {
        let outcome = self.probe_outcome();
        ProbeReport {
            available: outcome.available,
            detail: Some(outcome.detail.clone()),
        }
    }

    /// PREPARE: mint the conversation's session id and refuse anything that
    /// could never launch — with **no process, no adapter state, and no
    /// blocking call** (§14.3).
    ///
    /// This is where the "spawned before `execution.started`" window finally
    /// closes. The adapter has always chosen the session id before spawning;
    /// what was missing was a moment at which sergeant could *journal* that
    /// choice while it was still only a choice. PREPARE is that moment, and it
    /// is deliberately empty of side effects so the engine can run it under
    /// the core lock (§22.6) and append `execution.reserved` before anything
    /// exists to be orphaned.
    ///
    /// The probe and the pin pre-flight run here rather than at LAUNCH so a
    /// refusal happens before the reservation is journaled at all: a work that
    /// cannot start should not first acquire an execution identity.
    fn prepare(&self, request: &StartRequest) -> Result<PreparedExecution, BackendError> {
        // Version/capability gate: an unmeasured CLI is refused with the
        // probe's own evidence (fail closed, structured, actionable). The
        // probe is cached after the first call and is offline either way.
        let probe = self.probe_outcome();
        if !probe.available {
            return Err(BackendError::Unavailable {
                backend: CLAUDE_BACKEND_NAME.to_string(),
                detail: probe.detail.clone(),
            });
        }
        // Pin verification layer 1: pre-flight shape check, before launch.
        if let Some(model) = &request.model {
            preflight_model_pin(model).map_err(|reason| self.err_failed(reason))?;
        }
        // The session identity exists before the process does — and now,
        // before the *record* of the process does too.
        Ok(PreparedExecution {
            execution_id: request.execution_id.clone(),
            native_id: Some(new_session_uuid()),
            request: request.clone(),
        })
    }

    /// LAUNCH: register the reserved conversation and spawn its first turn.
    ///
    /// Everything external lives here, which is why the engine calls it with
    /// the core lock released. The launch configuration is resolved from the
    /// same profile PREPARE saw, through the same function START always used
    /// (§14) — a pure function of the request, so the two phases cannot
    /// disagree about permission mode or executable.
    fn launch(&self, prepared: &PreparedExecution) -> Result<ExecutionHandle, BackendError> {
        let request = &prepared.request;
        let session_id = prepared
            .native_id
            .clone()
            .ok_or_else(|| self.err_failed("prepared execution carries no session id"))?;
        let LaunchConfig {
            executable,
            env,
            permission_args,
        } = self.launch_config(request.profile.as_ref())?;
        {
            let mut state = self.lock();
            state.executions.insert(
                request.execution_id.clone(),
                ClaudeExecution {
                    session_id: session_id.clone(),
                    work_id: request.work_id.clone(),
                    cwd: request.cwd.clone(),
                    model: request.model.clone(),
                    executable,
                    env,
                    permission_args,
                    instruction_policy: request.instruction_policy,
                    turns: 0,
                    turn: TurnState::Unlaunched,
                    stopped: false,
                    interrupt_requested: false,
                    reader: None,
                },
            );
        }
        // §12: procedure is data — intent plus the stage's CONTEXT.md,
        // verbatim, uninterpreted.
        let prompt = format!("{}\n\n{}", request.intent, request.context);
        if let Err(e) = self.spawn_turn(&request.execution_id, prompt) {
            // A failed launch must not leave a phantom execution that
            // OBSERVE would misread as an interrupted-but-resumable turn.
            self.lock().executions.remove(&request.execution_id);
            return Err(e);
        }
        Ok(ExecutionHandle {
            execution_id: request.execution_id.clone(),
            native_id: Some(session_id),
        })
    }

    fn send(&self, handle: &ExecutionHandle, input: &str) -> Result<(), BackendError> {
        {
            let state = self.lock();
            self.check_identity(&state, handle)?;
            let execution = &state.executions[&handle.execution_id];
            if execution.stopped {
                return Err(self.err_failed(format!(
                    "execution {} is stopped; not accepting input",
                    handle.execution_id
                )));
            }
            if let TurnState::InFlight(_) = execution.turn {
                return Err(self.err_failed(format!(
                    "execution {} already has a turn in flight; a print-mode conversation \
                     runs one turn at a time",
                    handle.execution_id
                )));
            }
        }
        self.spawn_turn(&handle.execution_id, input.to_string())
    }

    fn observe(&self, handle: &ExecutionHandle) -> Result<Observation, BackendError> {
        let state = self.lock();
        if state.executions.contains_key(&handle.execution_id) {
            self.check_identity(&state, handle)?;
            let execution = &state.executions[&handle.execution_id];
            // A re-adopted execution has no turn of this daemon's to report,
            // so it is classified from the same restart evidence an
            // un-adopted one is — truthfully, as adopted (§15 RESUME).
            if matches!(execution.turn, TurnState::Adopted) {
                let session_id = execution.session_id.clone();
                drop(state);
                return classify_restart(
                    &session_id,
                    session_liveness(&session_id),
                    self.session_transcript(&session_id),
                    Adoption::Adopted,
                )
                .ok_or_else(|| self.err_unknown(&handle.execution_id));
            }
            return Ok(observe_in_memory(execution));
        }
        drop(state);

        // Not in memory: this daemon never started it (restart). Two
        // independent pieces of evidence are available, and the `native`
        // field reports only what one of them actually says:
        //
        // - process liveness, from the session id in a live turn's argv;
        // - session existence, from the durable transcript.
        //
        // The combination the old daemon's death makes possible — a turn
        // still running with nobody reading its stdout — is reported as
        // exactly that, `Running`, and blocked. Claiming `Exited` from the
        // mere existence of a transcript would be asserting a process fact
        // from a filesystem fact: §37's "worker reports done but the native
        // session is alive" class, committed by the adapter itself.
        let session_id = handle
            .native_id
            .as_deref()
            .ok_or_else(|| self.err_unknown(&handle.execution_id))?;
        classify_restart(
            session_id,
            session_liveness(session_id),
            self.session_transcript(session_id),
            Adoption::Unowned,
        )
        .ok_or_else(|| self.err_unknown(&handle.execution_id))
    }

    /// Kill the per-turn process. The conversation survives (measured); the
    /// reader thread archives whatever the turn streamed before dying.
    fn interrupt(&self, handle: &ExecutionHandle) -> Result<Completion, BackendError> {
        let child = {
            let mut state = self.lock();
            self.check_identity(&state, handle)?;
            let execution = state
                .executions
                .get_mut(&handle.execution_id)
                .expect("presence checked above");
            match &execution.turn {
                TurnState::InFlight(child) => {
                    execution.interrupt_requested = true;
                    Some(Arc::clone(child))
                }
                // No turn in flight: the goal state — no turn running —
                // already holds, whether this daemon finished one, launched
                // none, or re-adopted the conversation from a restart.
                TurnState::Finished(_) | TurnState::Unlaunched | TurnState::Adopted => None,
            }
        };
        if let Some(child) = child {
            let mut child = child.lock().expect("turn child lock");
            let _ = child.kill();
        }
        // Killing the child is the whole of INTERRUPT's promise: the turn's
        // evidence is STOP's promise, not this one, and the conversation
        // stays live and observable either way.
        Ok(Completion::immediate())
    }

    /// Re-adopt a conversation after a restart: verify that the durable
    /// transcript exists **and that no turn of it is still running**, then
    /// register the execution so SEND continues it with `--resume`.
    ///
    /// Both halves are the evidence RESUME's `Ok` claims (§15: "fails closed
    /// when the native context cannot be evidenced"), and restart
    /// reconciliation reattaches through exactly this call before it
    /// classifies anything. A conversation whose previous turn is still alive
    /// cannot be adopted at all: this adapter would have no child handle to
    /// interrupt, no stdout to ingest, and a later SEND would put a second
    /// process on a session the first one still holds. Liveness that cannot
    /// be read (no `/proc`) is refused for the same reason — the difference
    /// between the two is exactly what could not be established.
    ///
    /// Launch configuration comes from the [`ResumeRequest`] the caller
    /// rebuilt from the journal, through the same resolution START uses —
    /// never from defaults. Fabricating it here is what a fail-*open* resume
    /// looks like: it would silently drop a profile's pinned
    /// `--permission-mode` back to no-profile behavior (a security decision
    /// that belongs to the human, as the module docs say of #47), drop the
    /// model pin so every later turn verified as "unpinned" while the work's
    /// journal still records a pin, and journal normalized events under an
    /// empty work id. A pin the caller does not re-supply is genuinely
    /// absent — reported as unpinned, which is then true.
    fn resume(
        &self,
        handle: &ExecutionHandle,
        request: &ResumeRequest,
    ) -> Result<(), BackendError> {
        let session_id = handle
            .native_id
            .clone()
            .ok_or_else(|| self.err_unknown(&handle.execution_id))?;
        // Layer 1 of pin verification applies to a re-supplied pin exactly as
        // it does at START: a pin that could never be honored is refused
        // before anything is adopted on the strength of it — including
        // before the cheap "already adopted" answer, so a caller cannot slip
        // an impossible pin past the check by asking twice.
        if let Some(model) = &request.model {
            preflight_model_pin(model).map_err(|reason| self.err_failed(reason))?;
        }
        // Already ours: re-adoption is idempotent (restart reconciliation may
        // re-run after a crash inside its own append window), and an
        // execution this daemon already owns is not classified from restart
        // evidence — its own turn may legitimately be in flight.
        {
            let state = self.lock();
            if let Some(existing) = state.executions.get(&handle.execution_id) {
                if existing.session_id != session_id {
                    return Err(self.err_unknown(&handle.execution_id));
                }
                return Ok(());
            }
        }
        if self.session_transcript(&session_id).is_none() {
            return Err(self.err_unknown(&handle.execution_id));
        }
        match session_liveness(&session_id) {
            Liveness::Dead => {}
            Liveness::Alive(pid) => {
                return Err(self.err_failed(format!(
                    "cannot re-adopt conversation {session_id}: a turn of it is still running \
                     (pid {pid}) and this adapter does not own that process — adopting it \
                     would claim ownership of a turn whose output nothing is reading"
                )));
            }
            Liveness::Unknowable(why) => {
                return Err(self.err_failed(format!(
                    "cannot re-adopt conversation {session_id}: whether a turn of it is still \
                     running cannot be evidenced here ({why})"
                )));
            }
        }
        let LaunchConfig {
            executable,
            env,
            permission_args,
        } = self.launch_config(request.profile.as_ref())?;
        let mut state = self.lock();
        if let Some(existing) = state.executions.get(&handle.execution_id) {
            // Another thread adopted it while this one gathered evidence.
            if existing.session_id != session_id {
                return Err(self.err_unknown(&handle.execution_id));
            }
            return Ok(());
        }
        state.executions.insert(
            handle.execution_id.clone(),
            ClaudeExecution {
                session_id,
                work_id: request.work_id.clone(),
                cwd: request.cwd.clone(),
                model: request.model.clone(),
                executable,
                env,
                permission_args,
                instruction_policy: request.instruction_policy.unwrap_or_default(),
                turns: 1, // there was at least the turn that created it
                // Adoption draws no conclusion — and says so. (It used to
                // borrow an interrupted turn's shape here, which made OBSERVE
                // report "turn interrupted by request" for an interrupt
                // nobody requested.)
                turn: TurnState::Adopted,
                stopped: false,
                interrupt_requested: false,
                reader: None,
            },
        );
        Ok(())
    }

    /// HISTORY is unsupported here, and says so (§15: unsupported means
    /// unsupported, not emulation). See [`ClaudeBackend::capabilities`] for
    /// why the capability is `false`; the refusal names where the record
    /// actually is, because a caller that wanted history still needs an
    /// answer: sergeant's own journal holds every normalized event this
    /// adapter ever emitted, and the vendor's own durable transcript holds
    /// the native one.
    fn history(&self, handle: &ExecutionHandle) -> Result<Vec<NativeEvent>, BackendError> {
        // Identity is still checked first: an unrecognised execution is
        // unrecognised whatever the verb, and the caller learns that before
        // it learns anything about capabilities.
        let state = self.lock();
        self.check_identity(&state, handle)?;
        drop(state);
        Err(BackendError::Unsupported {
            backend: CLAUDE_BACKEND_NAME.to_string(),
            verb: "history".to_string(),
            detail: "this adapter cannot retrieve durable native history: the CLI's session \
                     transcript format is not measured, and reporting only the events this \
                     process happened to ingest would be a partial answer indistinguishable \
                     from a complete one (empty, in particular, after a restart). The \
                     normalized events are journaled through the event sink (§27); the native \
                     record is the CLI's own <session_id>.jsonl transcript"
                .to_string(),
        })
    }

    /// Retire: kill any in-flight turn and refuse further input. The
    /// durable transcript is untouched — recoverable state survives STOP by
    /// construction.
    ///
    /// STOP still *promises the turn's evidence*. Killing the child closes
    /// stdout, but the reader thread then archives the raw stream-json
    /// transcript (§20) and emits `conversation.turn.ended` carrying its blob
    /// ref — work that happens after `interrupt` returns. Declaring the
    /// execution retired while a write to its data dir is still in flight
    /// would make STOP a claim about the process only (measured in the M4
    /// suite, where that write recreated a data directory the test had
    /// already removed).
    ///
    /// **Issue #14 / backlog B3, resolved here.** The promise used to be kept
    /// by joining the reader inside this method — and this method is called
    /// from the engine while the daemon's `Core` mutex is held, so a
    /// concurrent request waited out one transcript flush. B3's registered
    /// trigger was "the first §15 trait revision", and this is it: the join
    /// is handed back as a [`Completion`] instead. The engine collects it
    /// ([`crate::backend::Deferred`]) and the daemon awaits it after
    /// releasing the core guard, so the evidence is still durable before the
    /// operation is over and nothing waits on a thread under the lock
    /// (§22.6). `block_in_place` is gone with the inline join.
    ///
    /// The join is safe wherever it eventually runs because the event sink
    /// never blocks on its caller (see [`crate::daemon::journaling_sink`])
    /// and the adapter lock is released before the completion is handed over
    /// — the reader takes that same lock on its way out.
    fn stop(&self, handle: &ExecutionHandle) -> Result<Completion, BackendError> {
        self.interrupt(handle)?.wait();
        let reader = {
            let mut state = self.lock();
            self.check_identity(&state, handle)?;
            let execution = state
                .executions
                .get_mut(&handle.execution_id)
                .expect("presence checked above");
            execution.stopped = true;
            execution.reader.take()
        };
        match reader {
            None => Ok(Completion::immediate()),
            Some(reader) => Ok(Completion::deferred(move || {
                let _ = reader.join();
            })),
        }
    }
}

/// Whether this daemon has re-adopted the conversation being classified.
///
/// It changes nothing about the *evidence* — the same `/proc` scan and the
/// same transcript answer either way — and everything about what the reason
/// says, which is the part a human reads: "sergeant did not adopt it" and
/// "sergeant re-adopted it, and the pre-restart turn's outcome is still
/// unknown" are different situations with the same liveness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Adoption {
    /// No RESUME has been performed: the context is unowned by this daemon.
    Unowned,
    /// RESUME succeeded; later SENDs continue this conversation.
    Adopted,
}

/// Classify a conversation this daemon did not run a turn on, from the two
/// independent pieces of restart evidence (§25's still-alive / resumable /
/// ambiguous). `None` means there is no durable conversation to classify —
/// the caller turns that into `UnknownExecution`.
///
/// Every branch fails the *work* closed. That is not redundancy with the
/// engine: the adapter is the only thing that knows which of these three
/// situations it is in, and the difference between them is the whole content
/// of the evidence an operator acts on.
fn classify_restart(
    session_id: &str,
    liveness: Liveness,
    transcript: Option<PathBuf>,
    adoption: Adoption,
) -> Option<Observation> {
    match (liveness, transcript) {
        (Liveness::Alive(pid), _) => Some(Observation {
            native: NativeState::Running,
            signal: BackendSignal::Blocked {
                reason: format!(
                    "daemon restarted while a turn of conversation {session_id} was still \
                     running (pid {pid}); that turn is unowned — its output is going \
                     nowhere and sergeant did not adopt it"
                ),
            },
            evidence: Some(format!(
                "live turn: pid {pid} runs with --resume/--session-id {session_id} in its argv"
            )),
        }),
        (Liveness::Dead, Some(path)) => Some(Observation {
            native: NativeState::Exited,
            signal: BackendSignal::Blocked {
                reason: match adoption {
                    Adoption::Unowned => format!(
                        "daemon restarted mid-execution; conversation {session_id} is \
                         resumable (durable transcript present) but the in-flight turn's \
                         outcome is unknown"
                    ),
                    Adoption::Adopted => format!(
                        "conversation {session_id} was re-adopted after a daemon restart and is \
                         resumable (durable transcript present, no turn running), but the turn \
                         that was in flight when the daemon died left no outcome this daemon \
                         can read — the stage's result is unknown, not absent"
                    ),
                },
            },
            evidence: Some(format!(
                "no live process carries session {session_id}; session transcript: {}; \
                 adopted={}",
                path.display(),
                adoption == Adoption::Adopted
            )),
        }),
        (Liveness::Unknowable(why), Some(path)) => Some(Observation {
            // The transcript proves the conversation; nothing here
            // proves what its last process is doing. §25: fail closed.
            native: NativeState::Unknown,
            signal: BackendSignal::Blocked {
                reason: format!(
                    "daemon restarted mid-execution; conversation {session_id} is \
                     resumable (durable transcript present) but whether its turn process \
                     is still running cannot be evidenced here"
                ),
            },
            evidence: Some(format!(
                "session transcript: {}; process liveness unknowable: {why}",
                path.display()
            )),
        }),
        (_, None) => None,
    }
}

/// Map an in-memory execution's turn state to an Observation.
fn observe_in_memory(execution: &ClaudeExecution) -> Observation {
    let session = &execution.session_id;
    match &execution.turn {
        // START inserts this and then either spawns (replacing it) or removes
        // the execution, so reaching it means the adapter's own invariant
        // broke. §25: report the ambiguity, invent nothing.
        TurnState::Unlaunched => Observation {
            native: NativeState::Unknown,
            signal: BackendSignal::Running,
            evidence: Some(format!(
                "execution registered for conversation {session} but no turn was ever launched"
            )),
        },
        // Handled by `observe` against restart evidence before it gets here.
        TurnState::Adopted => Observation {
            native: NativeState::Unknown,
            signal: BackendSignal::Running,
            evidence: Some(format!(
                "conversation {session} was re-adopted after a restart; no turn of this \
                 daemon's has run on it"
            )),
        },
        TurnState::InFlight(_) => Observation {
            native: NativeState::Running,
            signal: BackendSignal::Running,
            evidence: Some(format!(
                "turn {} in flight on session {session}",
                execution.turns
            )),
        },
        TurnState::Finished(outcome) => match &outcome.envelope {
            Some(envelope) => observe_envelope(execution, envelope, outcome),
            None if outcome.interrupted => Observation {
                // Sergeant killed the turn on purpose. No conclusion about
                // the stage is invented; the conversation is resumable. The
                // partial stream the turn did produce is archived, and the
                // ref travels with the evidence — an interrupted turn is
                // exactly when someone wants to read what it managed to say.
                native: NativeState::Exited,
                signal: BackendSignal::Running,
                evidence: Some(format!(
                    "turn interrupted by request; conversation {session} resumable; raw={}",
                    outcome.raw_evidence()
                )),
            },
            None => Observation {
                // The turn died without an envelope and nobody asked for
                // that. §25: this is ambiguity, and it fails closed — the
                // engine turns Unknown into blocked-with-evidence.
                native: NativeState::Unknown,
                signal: BackendSignal::Running,
                evidence: Some(format!(
                    "turn process exited without a result envelope (session {session}); \
                     raw={}; stderr: {}",
                    outcome.raw_evidence(),
                    truncate(&outcome.stderr, 400)
                )),
            },
        },
    }
}

/// Map a finished turn's result envelope to a signal. `is_error` is the
/// load-bearing field: measured on 2.1.226, a post-launch model failure
/// reports `subtype:"success"` with `is_error:true` — the subtype lies,
/// the exit code is unavailable here, and neither was ever sufficient (L1).
fn observe_envelope(
    execution: &ClaudeExecution,
    envelope: &Value,
    outcome: &TurnOutcome,
) -> Observation {
    let session = &execution.session_id;
    let result_text = envelope
        .get("result")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let is_error = envelope
        .get("is_error")
        .and_then(Value::as_bool)
        .unwrap_or(true); // an envelope that cannot say "no error" is an error
    if is_error {
        // Structured execution failure — the usage-limit / API-death class
        // surfaces here as a reason, never as ambiguity.
        let status = envelope
            .get("api_error_status")
            .cloned()
            .unwrap_or(Value::Null);
        return Observation {
            native: NativeState::Exited,
            signal: BackendSignal::Failed {
                reason: format!(
                    "turn failed (api_error_status={status}): {}",
                    truncate(&result_text, 400)
                ),
            },
            evidence: raw_evidence(session, outcome),
        };
    }
    // Pin verification layer 3: substitution detection from model fields.
    // Ordered *before* the ask check on purpose: a turn that ran on the wrong
    // model fails, whatever it asked. Parking such a stage in `needs_input`
    // would put a human in front of a question produced by a model the work
    // never authorized.
    match verify_model_pin(execution.model.as_deref(), envelope) {
        PinVerdict::Substituted(ran) => Observation {
            native: NativeState::Exited,
            signal: BackendSignal::Failed {
                reason: format!(
                    "model pin not honored: requested {:?}, turn ran on [{ran}] \
                     (silent substitution; the mission's own success is not evidence)",
                    execution.model.as_deref().unwrap_or("")
                ),
            },
            evidence: raw_evidence(session, outcome),
        },
        verdict => {
            // GP-2: the actor asked. A turn that ends on a question is not a
            // completed stage — it is a stage waiting on a human, resumable
            // by `respond` on this same conversation (`--resume` continues
            // it, U1's semantics unchanged). Before this branch existed, the
            // question became the completion summary and the workflow moved
            // on without it ever being answered.
            if let Some(question) = outcome.post_turn_summary.as_ref().and_then(actor_question) {
                let category = outcome
                    .post_turn_summary
                    .as_ref()
                    .and_then(|s| s.get("status_category"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                return Observation {
                    // The *turn's* process is gone; the conversation is not,
                    // and SEND resumes it. §25's separation, unchanged: the
                    // exited process says nothing about the stage.
                    native: NativeState::Exited,
                    signal: BackendSignal::ask(question),
                    evidence: Some(format!(
                        "session {session}; actor asked at turn end \
                         (post_turn_summary.status_category={category:?}); model_pin={}; raw={}",
                        verdict.as_json(),
                        outcome.raw_evidence()
                    )),
                };
            }
            Observation {
                native: NativeState::Exited,
                signal: BackendSignal::StageCompleted {
                    summary: Some(result_text),
                },
                evidence: Some(format!(
                    "session {session}; model_pin={}; raw={}",
                    verdict.as_json(),
                    outcome.raw_evidence()
                )),
            }
        }
    }
}

/// Translate R-MVP1-4's [`InstructionPolicy`] into this adapter's launch
/// grammar (MVP-2 D2 item 1, the `--setting-sources` de-leak).
///
/// Pure with respect to the process — no spawn, no I/O — so the exact argv
/// value for each policy is testable without a `claude` binary anywhere in
/// the loop (L13); `spawn_turn` is the only caller that turns the answer
/// into a real command.
///
/// **Measured** (four real bounded `claude-haiku-4-5` turns against a
/// scratch git repo with both `AGENTS.md` and `CLAUDE.md` present, one
/// codeword marker each, 2026-08-12 — logged in this build's commit notes,
/// L1): `--setting-sources` governs which `.claude/settings*.json`
/// configuration files the CLI loads (permissions, hooks, MCP server
/// config) — it does **not** gate whether the actor's own agentic turn
/// reads a memory file already sitting in its cwd. The CLI's native
/// "CLAUDE.md auto-discovery" (its own `--bare --help` text's name for the
/// mechanism) is a separate, always-on behaviour keyed to the literal
/// filename `CLAUDE.md`: it fired identically under no flag,
/// `--setting-sources user`, and `--setting-sources user,project` in this
/// measurement (the model actively `Read` the file each time, prompted by
/// its own default system prompt — not a settings-driven injection), and it
/// never fired at all for byte-identical content under Sergeant's actual
/// instruction filename, `AGENTS.md` — under any of those three source
/// sets. Two consequences follow directly:
///
/// - `Suppress`'s existing grammar (`--setting-sources user`, unchanged
///   below) was never actually suppressing `AGENTS.md` consumption for
///   *this* adapter — there was nothing native to suppress under that
///   filename in the first place. The flag still earns its keep for what it
///   *does* gate (below), so it stays.
/// - `InstructionPolicy::Local`'s doc comment ("the actor would consume the
///   repository's own instruction file natively") has no `--setting-sources`
///   value that implements it for `AGENTS.md`, because the mechanism that
///   comment describes does not exist for that filename at all. What
///   `Local` *actually* widens, honestly: whether the repository's own
///   `.claude/settings.json` / `.claude/settings.local.json` — hooks, tool
///   permissions, MCP servers, potentially arbitrary command execution via a
///   repo-authored hook — take effect for the turn. That is real,
///   repo-controlled surface, and a materially *larger* risk than "reads a
///   text file", not a smaller one. `Engine::resolve_instruction_identities`
///   still pins `AGENTS.md`'s content hash as evidence either way (R7's "the
///   file the actor will read is the one we recorded") — on this adapter,
///   under either policy, nothing reads it via this mechanism, and the pin
///   is honest bookkeeping about that, not proof of consumption.
fn setting_sources_args(policy: InstructionPolicy) -> [&'static str; 2] {
    match policy {
        InstructionPolicy::Suppress => ["--setting-sources", "user"],
        InstructionPolicy::Local => ["--setting-sources", "user,project,local"],
    }
}

fn raw_evidence(session: &str, outcome: &TurnOutcome) -> Option<String> {
    Some(format!("session {session}; raw={}", outcome.raw_evidence()))
}

fn truncate(text: &str, max: usize) -> &str {
    match text.char_indices().nth(max) {
        Some((idx, _)) => &text[..idx],
        None => text,
    }
}

/// Parse `"2.1.226 (Claude Code)"`-shaped output into a comparable triple.
fn parse_version(text: &str) -> Option<(u64, u64, u64)> {
    let first = text.split_whitespace().next()?;
    let mut parts = first.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some((major, minor, patch))
}

/// A v4-shaped UUID from ULID randomness (the crate already in tree; M2's
/// token took the same road — R5, no new dependency for 122 random bits).
/// `--session-id` requires UUID syntax (measured: a non-UUID is refused).
fn new_session_uuid() -> String {
    let a = ulid::Ulid::generate().to_bytes();
    let b = ulid::Ulid::generate().to_bytes();
    let mut bytes = [0u8; 16];
    for (i, slot) in bytes.iter_mut().enumerate() {
        // Each ULID's low 10 bytes are random and its high 6 are a
        // timestamp. XOR the two reversed against each other and every
        // output byte draws on at least one random byte: for i < 6, b's
        // random region supplies it; for i >= 6, a's does.
        *slot = a[i] ^ b[15 - i];
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40; // version 4
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // RFC 4122 variant
    let h = |r: std::ops::Range<usize>| {
        bytes[r]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    };
    format!(
        "{}-{}-{}-{}-{}",
        h(0..4),
        h(4..6),
        h(6..8),
        h(8..10),
        h(10..16)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The measured 2.1.226 result envelope for an honored haiku pin
    /// (verbatim fields that matter, recorded 2026-08-08 in this container).
    fn honored_envelope() -> Value {
        json!({
            "type": "result", "subtype": "success", "is_error": false,
            "session_id": "54148f06-ff05-4f02-a535-dea51faa7968",
            "result": "OK",
            "modelUsage": {
                "claude-haiku-4-5-20251001": {
                    "inputTokens": 541, "outputTokens": 100,
                    "canonicalModel": "claude-haiku-4-5",
                    "provider": "firstParty"
                }
            }
        })
    }

    /// Substitution envelope fixture. The *shape* (modelUsage keys +
    /// canonicalModel) is measured on 2.1.226; the substitution *scenario*
    /// is the spike's measurement on 2.1.220 (an unentitled `opus` pin ran
    /// on sonnet and succeeded) — not reproducible live here because this
    /// account is entitled to opus (measured: the pin was simply honored).
    /// Documented-not-measured for print mode, so detection fails closed:
    /// any envelope whose model fields do not match the pin is substitution.
    fn substitution_envelope() -> Value {
        json!({
            "type": "result", "subtype": "success", "is_error": false,
            "session_id": "00000000-0000-4000-8000-000000000000",
            "result": "mission accomplished",
            "modelUsage": {
                "claude-sonnet-5": {
                    "inputTokens": 100, "outputTokens": 50,
                    "canonicalModel": "claude-sonnet-5",
                    "provider": "firstParty"
                }
            }
        })
    }

    #[test]
    fn preflight_rejects_provider_qualified_and_empty_pins() {
        assert!(preflight_model_pin("haiku").is_ok());
        assert!(preflight_model_pin("claude-haiku-4-5-20251001").is_ok());
        let err = preflight_model_pin("anthropic/claude-sonnet-5").unwrap_err();
        assert!(err.contains("provider-qualified"), "{err}");
        assert!(preflight_model_pin("").is_err());
        assert!(preflight_model_pin("  ").is_err());
    }

    #[test]
    fn pin_verification_needs_positive_evidence() {
        // Honored: bare alias against the measured envelope.
        assert_eq!(
            verify_model_pin(Some("haiku"), &honored_envelope()),
            PinVerdict::Honored("claude-haiku-4-5-20251001".to_string())
        );
        // Honored: full id and canonical id both match.
        assert_eq!(
            verify_model_pin(Some("claude-haiku-4-5"), &honored_envelope()),
            PinVerdict::Honored("claude-haiku-4-5-20251001".to_string())
        );
        // Substituted: the spike's silent-substitution class.
        assert_eq!(
            verify_model_pin(Some("opus"), &substitution_envelope()),
            PinVerdict::Substituted("claude-sonnet-5".to_string())
        );
        // No model evidence at all: attempted, never honored.
        assert_eq!(
            verify_model_pin(Some("haiku"), &json!({"is_error": false})),
            PinVerdict::Attempted
        );
        assert_eq!(
            verify_model_pin(None, &honored_envelope()),
            PinVerdict::Unpinned
        );
    }

    /// An alias must match a whole segment of the resolved id, not a
    /// substring: "sonnet" honored by claude-sonnet-5, refused by an id
    /// that merely contains the letters.
    #[test]
    fn alias_matching_is_segment_wise() {
        let envelope =
            json!({"modelUsage": {"claude-sonnet-5": {"canonicalModel": "claude-sonnet-5"}}});
        assert_eq!(
            verify_model_pin(Some("sonnet"), &envelope),
            PinVerdict::Honored("claude-sonnet-5".to_string())
        );
        assert_eq!(
            verify_model_pin(Some("son"), &envelope),
            PinVerdict::Substituted("claude-sonnet-5".to_string())
        );
    }

    #[test]
    fn version_parses_the_measured_output_shape() {
        assert_eq!(parse_version("2.1.226 (Claude Code)"), Some((2, 1, 226)));
        assert_eq!(parse_version("3.0.0"), Some((3, 0, 0)));
        assert_eq!(parse_version("Claude Code"), None);
        assert_eq!(parse_version(""), None);
        assert!(
            parse_version("2.1").is_none(),
            "two segments are not a version"
        );
    }

    #[test]
    fn session_uuids_are_v4_shaped_and_unique() {
        let a = new_session_uuid();
        let b = new_session_uuid();
        assert_ne!(a, b);
        for id in [&a, &b] {
            let parts: Vec<&str> = id.split('-').collect();
            assert_eq!(parts.len(), 5, "{id}");
            assert_eq!(
                [
                    parts[0].len(),
                    parts[1].len(),
                    parts[2].len(),
                    parts[3].len(),
                    parts[4].len()
                ],
                [8, 4, 4, 4, 12],
                "{id}"
            );
            assert!(parts[2].starts_with('4'), "version nibble: {id}");
            assert!(
                matches!(parts[3].as_bytes()[0], b'8' | b'9' | b'a' | b'b'),
                "variant nibble: {id}"
            );
        }
    }

    /// A failed-turn envelope (the measured 404 shape: subtype "success",
    /// is_error true) maps to an explicit Failed signal — never ambiguity,
    /// never trust in the lying subtype.
    #[test]
    fn an_error_envelope_is_a_structured_failure_not_ambiguity() {
        let execution = test_execution(None);
        let envelope = json!({
            "type": "result", "subtype": "success", "is_error": true,
            "api_error_status": 404,
            "result": "There's an issue with the selected model (anthropic/claude-haiku-4-5). \
                       It may not exist or you may not have access to it.",
            "session_id": "s"
        });
        let outcome = TurnOutcome {
            post_turn_summary: None,
            envelope: Some(envelope.clone()),
            interrupted: false,
            raw_blob: None,
            raw_error: None,
            stderr: String::new(),
        };
        let observation = observe_envelope(&execution, &envelope, &outcome);
        assert_eq!(observation.native, NativeState::Exited);
        match observation.signal {
            BackendSignal::Failed { reason } => {
                assert!(reason.contains("404"), "{reason}");
                assert!(reason.contains("issue with the selected model"), "{reason}");
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    /// A substituted pin fails the stage even though the mission succeeded
    /// — the spike's core finding, enforced at the signal layer.
    #[test]
    fn a_substituted_pin_fails_the_stage_despite_mission_success() {
        let execution = test_execution(Some("opus"));
        let envelope = substitution_envelope();
        let outcome = TurnOutcome {
            post_turn_summary: None,
            envelope: Some(envelope.clone()),
            interrupted: false,
            raw_blob: Some("b3:aa".to_string()),
            raw_error: None,
            stderr: String::new(),
        };
        let observation = observe_envelope(&execution, &envelope, &outcome);
        match observation.signal {
            BackendSignal::Failed { reason } => {
                assert!(reason.contains("model pin not honored"), "{reason}");
                assert!(reason.contains("claude-sonnet-5"), "{reason}");
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    /// A turn that dies without an envelope: interrupted → resumable, no
    /// conclusion; not interrupted → Unknown, which the engine fails
    /// closed. Both directions pinned.
    #[test]
    fn a_missing_envelope_is_resumable_when_interrupted_and_unknown_otherwise() {
        let mut execution = test_execution(None);
        execution.turn = TurnState::Finished(TurnOutcome {
            post_turn_summary: None,
            envelope: None,
            interrupted: true,
            raw_blob: Some("b3:cc".to_string()),
            raw_error: None,
            stderr: String::new(),
        });
        let observation = observe_in_memory(&execution);
        assert_eq!(observation.native, NativeState::Exited);
        assert_eq!(observation.signal, BackendSignal::Running);

        execution.turn = TurnState::Finished(TurnOutcome {
            post_turn_summary: None,
            envelope: None,
            interrupted: false,
            raw_blob: None,
            raw_error: Some("blob store is full".to_string()),
            stderr: "boom".to_string(),
        });
        let observation = observe_in_memory(&execution);
        assert_eq!(observation.native, NativeState::Unknown);
        assert!(
            observation
                .evidence
                .as_deref()
                .unwrap_or("")
                .contains("boom"),
            "stderr is the evidence: {observation:?}"
        );
    }

    /// An envelope that does not say `is_error: false` is an error — the
    /// `.unwrap_or(true)` default, which is the only thing standing between a
    /// malformed or truncated envelope and a stage reported complete. The
    /// error-path test above supplies `is_error: true` explicitly, so it
    /// never reaches this default; flipping the default to `false` used to
    /// leave the whole suite green.
    #[test]
    fn an_envelope_that_cannot_say_no_error_is_an_error() {
        let execution = test_execution(None);
        let envelope = json!({
            "type": "result", "subtype": "success", "result": "looks fine to me"
        });
        let outcome = TurnOutcome {
            post_turn_summary: None,
            envelope: Some(envelope.clone()),
            interrupted: false,
            raw_blob: None,
            raw_error: None,
            stderr: String::new(),
        };
        match observe_envelope(&execution, &envelope, &outcome).signal {
            BackendSignal::Failed { reason } => assert!(reason.contains("turn failed"), "{reason}"),
            other => panic!("a missing is_error must not read as success, got {other:?}"),
        }
    }

    /// The restart classifier's three branches, including the one no
    /// environment here can produce: liveness that cannot be read at all
    /// (`/proc` absent or unreadable). Its `NativeState::Unknown` is what
    /// makes the engine fail the work closed, and nothing else in the suite
    /// can reach it — `session_liveness` only returns `Unknowable` off Linux
    /// or on an unreadable `/proc`.
    #[test]
    fn restart_classification_fails_closed_on_unreadable_liveness() {
        let transcript = Some(PathBuf::from("/claude/projects/x/s.jsonl"));
        let unknowable = classify_restart(
            "s",
            Liveness::Unknowable("no /proc here".to_string()),
            transcript.clone(),
            Adoption::Unowned,
        )
        .expect("a transcript means there is something to classify");
        assert_eq!(unknowable.native, NativeState::Unknown);
        assert!(
            unknowable
                .evidence
                .as_deref()
                .unwrap_or("")
                .contains("no /proc here"),
            "{unknowable:?}"
        );

        let alive = classify_restart(
            "s",
            Liveness::Alive(4242),
            transcript.clone(),
            Adoption::Unowned,
        )
        .expect("a live turn is classifiable with or without a transcript");
        assert_eq!(alive.native, NativeState::Running);
        assert!(alive.evidence.as_deref().unwrap_or("").contains("4242"));

        let dead = classify_restart("s", Liveness::Dead, transcript, Adoption::Unowned)
            .expect("dead + transcript is the resumable case");
        assert_eq!(dead.native, NativeState::Exited);

        assert!(
            classify_restart("s", Liveness::Dead, None, Adoption::Unowned).is_none(),
            "no durable conversation: there is nothing to classify, and the caller \
             turns that into UnknownExecution"
        );
    }

    /// A re-adopted conversation reports adoption, and never claims an
    /// interrupt nobody requested.
    ///
    /// The fabricated shape this replaces (`Finished { envelope: None,
    /// interrupted: true }`) rendered as `signal=Running` with the evidence
    /// "turn interrupted by request" — an affirmatively false statement, and
    /// the fail-*open* direction: the engine's Running branch makes no
    /// transition, so a reattached work would sit `active` with no turn in
    /// flight and nothing to move it.
    #[test]
    fn an_adopted_conversation_claims_no_interrupt_and_no_verdict() {
        let mut execution = test_execution(None);
        execution.turn = TurnState::Adopted;
        let observation = observe_in_memory(&execution);
        let evidence = observation.evidence.clone().unwrap_or_default();
        assert!(
            !evidence.contains("interrupted"),
            "adoption must not borrow the interrupted-turn shape: {observation:?}"
        );
        assert!(evidence.contains("re-adopted"), "{observation:?}");
        assert_eq!(
            observation.native,
            NativeState::Unknown,
            "no turn of this daemon's has run: {observation:?}"
        );

        // And the adopted classification an OBSERVE actually returns says the
        // same thing against real restart evidence.
        let adopted = classify_restart(
            "s",
            Liveness::Dead,
            Some(PathBuf::from("/claude/projects/x/s.jsonl")),
            Adoption::Adopted,
        )
        .expect("classifiable");
        let BackendSignal::Blocked { reason } = &adopted.signal else {
            panic!("an adopted conversation with an unknown turn outcome blocks: {adopted:?}");
        };
        assert!(reason.contains("re-adopted"), "{reason}");
        assert!(reason.contains("resumable"), "{reason}");
        assert!(!reason.contains("interrupt"), "{reason}");
    }

    /// Liveness is evidence about a *turn*, so the argv match is the launch
    /// grammar's flag-and-value pair — never the id as a substring of a
    /// joined command line. Everything in the second group puts the id in
    /// some process's argv without any turn running, and each one used to
    /// report `NativeState::Running` with "pid N carries session id in argv".
    #[test]
    fn liveness_matches_turn_argv_and_not_a_quoted_session_id() {
        let session = "11111111-2222-4333-8444-555555555555";
        let argv = |args: &[&str]| args.join("\0").into_bytes();

        for turn in [
            argv(&[
                "claude",
                "-p",
                "--verbose",
                "--session-id",
                session,
                "--dangerously-skip-permissions",
            ]),
            argv(&["claude", "-p", "--resume", session]),
            // A profile may name a different executable; the flag pair is
            // what identifies the turn, not the program's name.
            argv(&["/opt/harness/claude-wrapper", "-p", "--resume", session]),
        ] {
            assert!(
                cmdline_names_session(&turn, session),
                "a real turn's argv must match: {:?}",
                String::from_utf8_lossy(&turn)
            );
        }

        for bystander in [
            // The harness this project is built in wraps commands like this.
            argv(&["bash", "-c", &format!("eval 'grep {session} log.txt'")]),
            argv(&["less", &format!("/root/.claude/projects/p/{session}.jsonl")]),
            argv(&["vim", &format!("{session}.jsonl")]),
            // The id is present as its own argument, but no flag claims it.
            argv(&["echo", session]),
            // The flag is present, but names a different conversation.
            argv(&[
                "claude",
                "-p",
                "--resume",
                "99999999-8888-4777-8666-555555555555",
            ]),
            // A flag with nothing after it.
            argv(&["claude", "-p", "--resume"]),
        ] {
            assert!(
                !cmdline_names_session(&bystander, session),
                "a quoted id is not a running turn: {:?}",
                String::from_utf8_lossy(&bystander)
            );
        }
    }

    /// `truncate` cuts on character boundaries. The inputs are CLI stderr and
    /// the model's own `result` text, both of which routinely carry
    /// multibyte characters; byte slicing here is a panic in the adapter's
    /// evidence path, which is exactly where a panic destroys the evidence.
    #[test]
    fn truncate_cuts_on_character_boundaries() {
        let text = "航海日誌: ✅ done";
        assert_eq!(truncate(text, 0), "");
        assert_eq!(truncate(text, 2), "航海");
        assert_eq!(truncate(text, 4), "航海日誌");
        assert_eq!(truncate(text, 1000), text, "shorter than the limit");
        assert_eq!(truncate("", 4), "");
    }

    /// GP-2, measured: `post_turn_summary.needs_action` is the discriminator,
    /// and it is the *only* one this adapter reads.
    ///
    /// Both records below are the literal lines 2.1.226 emitted for the two
    /// prompts recorded in `docs/gauntlet/notes/n3-claude-ask-measurement.md`.
    /// The empty-string case is the one that matters most: a turn that asked
    /// nothing still emits the line, so "the line exists" would have been a
    /// capability claim with no discrimination in it at all.
    #[test]
    fn the_actor_question_comes_from_needs_action_and_nothing_else() {
        let asked = json!({
            "type": "system", "subtype": "post_turn_summary",
            "status_category": "blocked",
            "status_detail": "Which database should I target: **postgres** or **sqlite**?",
            "needs_action": "Which database should I target: **postgres** or **sqlite**?",
        });
        assert_eq!(
            actor_question(&asked).as_deref(),
            Some("Which database should I target: **postgres** or **sqlite**?")
        );
        let did_not_ask = json!({
            "type": "system", "subtype": "post_turn_summary",
            "status_category": "review_ready",
            "status_detail": "user requested confirmation",
            "needs_action": "",
        });
        assert_eq!(actor_question(&did_not_ask), None);
        // Whitespace is not a question, and a missing/non-string field is
        // not evidence of one either.
        assert_eq!(actor_question(&json!({"needs_action": "   "})), None);
        assert_eq!(actor_question(&json!({"needs_action": null})), None);
        assert_eq!(actor_question(&json!({})), None);
        // And a category alone never conjures one: an unmeasured vocabulary
        // is not a signal (L1).
        assert_eq!(actor_question(&json!({"status_category": "blocked"})), None);
    }

    /// A turn that ends on an actor-authored question parks the stage instead
    /// of completing it — and the question, not the adapter, is the author.
    ///
    /// This is the whole of GP-2 at the adapter boundary. Before it, the
    /// measured ask turn returned `StageCompleted { summary: "Which database
    /// should I target…" }`: the workflow moved to the next stage carrying the
    /// unanswered question as its predecessor's result.
    #[test]
    fn an_actor_authored_question_parks_the_stage_rather_than_completing_it() {
        let execution = test_execution(None);
        let envelope = json!({
            "type": "result", "subtype": "success", "is_error": false,
            "result": "Which database should I target: **postgres** or **sqlite**?",
        });
        let outcome = TurnOutcome {
            post_turn_summary: Some(json!({
                "type": "system", "subtype": "post_turn_summary",
                "status_category": "blocked",
                "needs_action": "Which database should I target: **postgres** or **sqlite**?",
            })),
            envelope: Some(envelope.clone()),
            interrupted: false,
            raw_blob: Some("b3:aa".to_string()),
            raw_error: None,
            stderr: String::new(),
        };
        let observation = observe_envelope(&execution, &envelope, &outcome);
        match &observation.signal {
            BackendSignal::NeedsInput { prompt, asked_by } => {
                assert_eq!(
                    prompt,
                    "Which database should I target: **postgres** or **sqlite**?"
                );
                assert_eq!(*asked_by, crate::backend::AskAuthor::Actor);
            }
            other => panic!("expected an actor ask, got {other:?}"),
        }
        // The turn's process is gone; the conversation is not (§25).
        assert_eq!(observation.native, NativeState::Exited);
        let evidence = observation.evidence.unwrap_or_default();
        assert!(
            evidence.contains("status_category"),
            "the unmeasured category travels as evidence, never as the decision: {evidence}"
        );

        // Same turn, no question: the stage completes exactly as before.
        let quiet = TurnOutcome {
            post_turn_summary: Some(json!({
                "type": "system", "subtype": "post_turn_summary",
                "status_category": "review_ready", "needs_action": "",
            })),
            ..outcome
        };
        assert!(matches!(
            observe_envelope(&execution, &envelope, &quiet).signal,
            BackendSignal::StageCompleted { .. }
        ));
    }

    /// An ask never outranks a model-pin substitution. Parking a stage would
    /// put a human in front of a question produced by a model the work never
    /// authorized; the pin failure is the outcome, and it stays the outcome.
    #[test]
    fn a_substituted_pin_still_fails_even_when_the_actor_asked() {
        let execution = test_execution(Some("opus"));
        let envelope = substitution_envelope();
        let outcome = TurnOutcome {
            post_turn_summary: Some(json!({
                "type": "system", "subtype": "post_turn_summary",
                "status_category": "blocked", "needs_action": "which one?",
            })),
            envelope: Some(envelope.clone()),
            interrupted: false,
            raw_blob: None,
            raw_error: None,
            stderr: String::new(),
        };
        assert!(matches!(
            observe_envelope(&execution, &envelope, &outcome).signal,
            BackendSignal::Failed { .. }
        ));
    }

    /// INV-N3-06: `Capabilities::ask` rests on a stream-json line the probe
    /// cannot check, and its absence used to fail *open*.
    ///
    /// `post_turn_summary == None` and `Some({needs_action: ""})` were treated
    /// identically, so a later CLI that dropped, renamed or re-shaped the line
    /// would silently restore the pre-N3 behaviour — the stage completes
    /// carrying the unanswered question as its summary — with `ask: true`
    /// still on the wire and nothing but an opt-in, token-spending test to
    /// notice. The adapter can tell at runtime, from the one place the answer
    /// exists: a turn.
    #[test]
    fn a_completed_turn_with_no_post_turn_summary_withdraws_the_ask_claim() {
        let completed = |summary: Option<Value>| TurnOutcome {
            envelope: Some(json!({"type": "result", "is_error": false, "result": "done"})),
            interrupted: false,
            raw_blob: None,
            raw_error: None,
            stderr: String::new(),
            post_turn_summary: summary,
        };

        // The measured 2.1.226 shapes, both of which carry the line: the
        // capability survives an ask *and* a turn that asked nothing.
        assert!(!turn_lost_the_ask_grammar(&completed(Some(json!({
            "type": "system", "subtype": POST_TURN_SUMMARY_SUBTYPE,
            "needs_action": "postgres or sqlite?",
        })))));
        assert!(!turn_lost_the_ask_grammar(&completed(Some(json!({
            "type": "system", "subtype": POST_TURN_SUMMARY_SUBTYPE,
            "needs_action": "",
        })))));

        // The line is gone: the evidence the claim rests on is not there.
        assert!(turn_lost_the_ask_grammar(&completed(None)));

        // A turn sergeant killed, and one that died without an envelope, are
        // not evidence about the grammar — withdrawing a measured capability
        // because a human hit cancel would be its own invention.
        let mut interrupted = completed(None);
        interrupted.interrupted = true;
        assert!(!turn_lost_the_ask_grammar(&interrupted));
        let mut crashed = completed(None);
        crashed.envelope = None;
        assert!(!turn_lost_the_ask_grammar(&crashed));
    }

    /// …and the withdrawal reaches the capability list, once.
    #[test]
    fn the_withdrawn_ask_claim_is_what_the_capability_list_reports() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let backend = ClaudeBackend::new(ClaudeConfig::new(dir.path()));
        assert!(
            backend.capabilities().ask && backend.ask_grammar_intact(),
            "measured on 2.1.226: see docs/gauntlet/notes/n3-claude-ask-measurement.md"
        );

        let emitted = Arc::new(Mutex::new(Vec::<EventDraft>::new()));
        let sink_events = Arc::clone(&emitted);
        backend.set_event_sink(Arc::new(move |draft| {
            sink_events.lock().expect("sink lock").push(draft);
        }));
        let reader = TurnReader {
            backend_state: Arc::clone(&backend.state),
            ask_grammar: Arc::clone(&backend.ask_grammar),
            version: Some("2.1.226".to_string()),
            sink: backend.sink.lock().expect("sink lock").clone(),
            data_dir: dir.path().to_path_buf(),
            execution_id: "e-grammar".to_string(),
            work_id: "w-grammar".to_string(),
            session_id: "s-grammar".to_string(),
            model: None,
            child: Arc::new(Mutex::new(
                std::process::Command::new("true")
                    .spawn()
                    .expect("spawn true"),
            )),
            stderr_rx: None,
        };
        let outcome = TurnOutcome {
            envelope: Some(json!({"type": "result", "is_error": false, "result": "done"})),
            interrupted: false,
            raw_blob: None,
            raw_error: None,
            stderr: String::new(),
            post_turn_summary: None,
        };

        reader.note_ask_grammar(&outcome);
        assert!(
            !backend.capabilities().ask,
            "an unmeasurable capability fails closed (L1, L8)"
        );
        reader.note_ask_grammar(&outcome);

        let events = emitted.lock().expect("sink lock");
        let notices: Vec<&EventDraft> = events
            .iter()
            .filter(|e| e.kind == "conversation.turn.grammar_unmeasured")
            .collect();
        assert_eq!(
            notices.len(),
            1,
            "the withdrawal is journaled once, not once per turn: {:?}",
            events.iter().map(|e| &e.kind).collect::<Vec<_>>()
        );
        assert_eq!(notices[0].payload["capability"], "ask");
        assert_eq!(
            notices[0].payload["expected"],
            format!("system/{POST_TURN_SUMMARY_SUBTYPE}")
        );
        assert_eq!(
            notices[0].payload["version"], "2.1.226",
            "the withdrawal must name the version it was measured against \
             (MVP-2 D2 item 2) — a restart's provenance seed has nothing to \
             match against otherwise"
        );
    }

    /// MVP-2 D2 item 2: [`latest_ask_withdrawal_version`] picks the
    /// highest-`seq` withdrawal, ignores events from other backends/kinds,
    /// and treats "no version field" (a pre-D2 build's record) as no
    /// evidence at all.
    #[test]
    fn latest_ask_withdrawal_version_picks_the_highest_seq_matching_event() {
        fn withdrawal(seq: u64, version: &str) -> Event {
            EventDraft::new(
                EventSource::new("backend", CLAUDE_BACKEND_NAME),
                "conversation.turn.grammar_unmeasured",
                json!({"capability": "ask", "version": version}),
            )
            .into_event(seq)
        }

        // Nothing recorded yet: no opinion.
        assert_eq!(latest_ask_withdrawal_version(Vec::new()).unwrap(), None);

        // A single matching record names its version.
        assert_eq!(
            latest_ask_withdrawal_version(vec![Ok(withdrawal(1, "2.1.226"))]).unwrap(),
            Some("2.1.226".to_string())
        );

        // Two records for two versions (a restart, an upgrade, another
        // withdrawal): the later `seq` wins, out of iterator order too.
        let out_of_order = vec![Ok(withdrawal(5, "2.1.228")), Ok(withdrawal(2, "2.1.226"))];
        assert_eq!(
            latest_ask_withdrawal_version(out_of_order).unwrap(),
            Some("2.1.228".to_string())
        );

        // SURVIVOR pin (MVP-2 D3 fixer pass): the mirrored ordering. The case above
        // alone cannot distinguish "pick the highest seq" from "pick the
        // first matching element" — the highest-seq record already comes
        // first in iterator order there, so a `latest.is_none()`-style
        // first-wins mutant returns the identical answer and survives. This
        // pairing puts the highest seq *last* in iterator order so only a
        // true highest-seq comparison passes both.
        let in_order = vec![Ok(withdrawal(2, "2.1.226")), Ok(withdrawal(5, "2.1.228"))];
        assert_eq!(
            latest_ask_withdrawal_version(in_order).unwrap(),
            Some("2.1.228".to_string())
        );

        // A same-adapter event of a different kind, a different backend's
        // event, and a record with no `version` field at all (pre-D2) are
        // all skipped, not misread as evidence.
        let noise = vec![
            Ok(EventDraft::new(
                EventSource::new("backend", CLAUDE_BACKEND_NAME),
                "conversation.user",
                json!({}),
            )
            .into_event(9)),
            Ok(EventDraft::new(
                EventSource::new("backend", "docker"),
                "conversation.turn.grammar_unmeasured",
                json!({"capability": "ask", "version": "2.1.226"}),
            )
            .into_event(10)),
            Ok(EventDraft::new(
                EventSource::new("backend", CLAUDE_BACKEND_NAME),
                "conversation.turn.grammar_unmeasured",
                json!({"capability": "ask"}),
            )
            .into_event(11)),
        ];
        assert_eq!(latest_ask_withdrawal_version(noise).unwrap(), None);
    }

    /// #83: a freshly written, freshly `chmod +x`'d stand-in script can
    /// transiently fail `execve(2)` with `ETXTBSY` ("text file busy",
    /// `os error 26`) under `cargo test --lib`'s default thread
    /// parallelism — measured directly (3 failures in 40 runs, every one
    /// this exact `io::ErrorKind::ExecutableFileBusy`) via the diagnostic
    /// `run_probe` now surfaces. This is not a version-parsing bug; the
    /// same absorb-and-retry idiom already fixed it in
    /// `tests/m5_projections.rs` and `tests/m6_surfaces.rs` before this
    /// call site existed. Retry until the exec stops being refused, or
    /// surface any other failure immediately — the window is real but
    /// bounded, never a reason to paper over an unrelated exec error.
    #[cfg(unix)]
    fn wait_until_executable(path: &std::path::Path) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while let Err(e) = std::process::Command::new(path).arg("--version").output() {
            assert!(
                e.raw_os_error() == Some(26) && std::time::Instant::now() < deadline,
                "the stand-in is not runnable: {e}"
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    /// #83, direct proof rather than a re-run-until-it-flakes gamble:
    /// manufactures the exact condition `execve(2)` refuses with `ETXTBSY`
    /// — a second fd open for writing on the same inode — and confirms
    /// `wait_until_executable` retries through it instead of surfacing the
    /// exec failure as if the file were simply unrunnable. Deterministic,
    /// not load-dependent: the writer fd stays open for 150ms, so a single
    /// attempt made at t=0 (i.e. no retry loop) always lands inside the
    /// busy window and this test would fail every time the fix is
    /// reverted, not just some fraction of runs.
    #[cfg(unix)]
    #[test]
    fn a_manufactured_etxtbsy_window_is_absorbed_not_surfaced_as_missing() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let script = dir.path().join("busy-script");
        std::fs::write(&script, "#!/bin/sh\necho ok\n").expect("write script");
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755))
            .expect("chmod +x");

        // A second fd open for writing on `script` is exactly what
        // `execve(2)` refuses — this is the manufactured busy window, held
        // open on another thread so `wait_until_executable` below observes
        // it on its very first attempt.
        let writer = std::fs::OpenOptions::new()
            .write(true)
            .open(&script)
            .expect("open second writer fd");
        let closer = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(150));
            drop(writer);
        });

        let start = std::time::Instant::now();
        wait_until_executable(&script);
        assert!(
            start.elapsed() >= std::time::Duration::from_millis(100),
            "wait_until_executable returned before the manufactured busy \
             window closed — it did not actually retry through ETXTBSY, it \
             got lucky (or the busy window was never real)"
        );
        closer.join().expect("writer-closing thread");
    }

    /// MVP-2 D2 item 2: the durability half. A record for the version this
    /// probe measures *right now* lowers the claim; a record for any other
    /// version (a CLI upgrade or downgrade since it was journaled) leaves
    /// the fresh-install optimism untouched — the mutation this guards
    /// against is a seed that applies regardless of version, which would
    /// wrongly keep an `ask` refusal alive across an upgrade that fixed it.
    #[test]
    fn capability_provenance_only_applies_to_a_matching_version() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let mut config = ClaudeConfig::new(dir.path());
        // A `claude` binary this probe can actually run and parse, entirely
        // offline: `--version`/`--help` are shelled out for real
        // (`run_probe`), so the stand-in must answer both without touching
        // the network or spending a token.
        let script = dir.path().join("fake-claude");
        std::fs::write(
            &script,
            format!(
                "#!/bin/sh\nif [ \"$1\" = --version ]; then echo '2.1.226 (Claude Code)'; else echo '{}'; fi\n",
                REQUIRED_FLAGS.join(" ")
            ),
        )
        .expect("write stand-in");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755))
                .expect("chmod +x");
            wait_until_executable(&script);
        }
        config.executable = script;
        let backend = ClaudeBackend::new(config);
        assert_eq!(
            backend.probe_outcome().version.as_deref(),
            Some("2.1.226"),
            "stand-in must parse as the version the test reasons about (probe detail: {})",
            backend.probe_outcome().detail
        );

        // A different version's withdrawal: stale, does not apply.
        backend.apply_capability_provenance(Some("2.1.225"));
        assert!(
            backend.ask_grammar_intact(),
            "a withdrawal recorded against a CLI version other than the one \
             installed now must not lower a fresh install's claim"
        );

        // The exact version just measured: applies.
        backend.apply_capability_provenance(Some("2.1.226"));
        assert!(
            !backend.ask_grammar_intact(),
            "a withdrawal recorded against exactly this version must seed \
             the restarted claim as withdrawn"
        );
    }

    /// A turn's stderr is **waited for**, never snapshotted (issue #46's
    /// attempt-1 shape).
    ///
    /// Both of a turn's pipes reach EOF at the same instant — the process
    /// exits — so a reader that closes stdout and immediately reads a shared
    /// stderr buffer wins that race only by luck. Measured while pinning Run B
    /// attempt 1 through the daemon: the journal's `conversation.turn.ended`
    /// intermittently carried `stderr: ""` for the one turn shape whose *only*
    /// evidence is that line, because the CLI refused before writing a byte of
    /// stream-json. Losing it leaves the operator a block with no cause.
    ///
    /// The 300 ms below is that race widened to something a test can see: the
    /// process (`true`) is gone before the delivery arrives, so a snapshot
    /// taken at stdout EOF reads `""` and a delivery reads the refusal. Both
    /// halves are asserted — the journaled event *and* the observation the
    /// engine blocks on — because they are two different readers of the same
    /// field.
    #[test]
    fn a_turns_stderr_is_waited_for_rather_than_snapshotted() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let backend = ClaudeBackend::new(ClaudeConfig::new(dir.path()));
        let emitted = Arc::new(Mutex::new(Vec::<EventDraft>::new()));
        let sink_events = Arc::clone(&emitted);
        backend.set_event_sink(Arc::new(move |draft| {
            sink_events.lock().expect("sink lock").push(draft);
        }));
        backend
            .state
            .lock()
            .expect("adapter state lock")
            .executions
            .insert("e-stderr".to_string(), test_execution(None));

        let mut child = Command::new("true")
            .stdout(Stdio::piped())
            .spawn()
            .expect("spawn true");
        let stdout = child.stdout.take().expect("piped stdout");
        let (tx, rx) = std::sync::mpsc::sync_channel::<String>(1);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(300));
            let _ = tx.send(
                "--dangerously-skip-permissions cannot be used with root/sudo privileges\n"
                    .to_string(),
            );
        });
        let reader = TurnReader {
            backend_state: Arc::clone(&backend.state),
            ask_grammar: Arc::clone(&backend.ask_grammar),
            version: None,
            sink: backend.sink.lock().expect("sink lock").clone(),
            data_dir: dir.path().to_path_buf(),
            execution_id: "e-stderr".to_string(),
            work_id: "w-stderr".to_string(),
            session_id: "s-stderr".to_string(),
            model: None,
            child: Arc::new(Mutex::new(child)),
            stderr_rx: Some(rx),
        };
        reader.run(stdout);

        let events = emitted.lock().expect("sink lock");
        let ended = events
            .iter()
            .find(|e| e.kind == "conversation.turn.ended")
            .expect("every turn ends with this event");
        assert_eq!(ended.payload["result_envelope"], false);
        assert!(
            ended.payload["stderr"]
                .as_str()
                .is_some_and(|s| s.contains("root/sudo")),
            "the CLI's own refusal must reach the journal rather than an empty \
             string it lost a race to: {}",
            ended.payload
        );

        let state = backend.state.lock().expect("adapter state lock");
        let observation = observe_in_memory(&state.executions["e-stderr"]);
        assert_eq!(
            observation.native,
            NativeState::Unknown,
            "an envelope-less turn is ambiguity, and ambiguity fails closed"
        );
        assert!(
            observation
                .evidence
                .as_deref()
                .is_some_and(|e| e.contains("root/sudo")),
            "…and the evidence the engine blocks with carries the same stderr: \
             {observation:?}"
        );
    }

    /// TH-4 (BS2 test-honesty finding): the sibling test above only proves
    /// the *delivery* half of `STDERR_DRAIN_BUDGET` — a sender that sends
    /// within the wait. Commit 39971e6 justifies the budget as bounded
    /// specifically so a descendant that leaks the stderr pipe (holds the
    /// write end open, never writes, never lets it close) cannot turn "no
    /// stderr" into "no turn outcome" — i.e. the read must give up and
    /// default to empty, not block forever. A sender that is kept alive but
    /// never sends and never drops is exactly that leaked-descendant shape;
    /// mutating `recv_timeout(STDERR_DRAIN_BUDGET)` to an unbounded `recv()`
    /// makes this test hang instead of complete.
    #[test]
    fn a_stderr_sender_that_never_sends_does_not_block_the_turn_outcome() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let backend = ClaudeBackend::new(ClaudeConfig::new(dir.path()));
        let emitted = Arc::new(Mutex::new(Vec::<EventDraft>::new()));
        let sink_events = Arc::clone(&emitted);
        backend.set_event_sink(Arc::new(move |draft| {
            sink_events.lock().expect("sink lock").push(draft);
        }));
        backend
            .state
            .lock()
            .expect("adapter state lock")
            .executions
            .insert("e-stderr-never".to_string(), test_execution(None));

        let mut child = Command::new("true")
            .stdout(Stdio::piped())
            .spawn()
            .expect("spawn true");
        let stdout = child.stdout.take().expect("piped stdout");
        // `tx` is deliberately held alive (never sent on, never dropped)
        // across `reader.run` below — simulating a descendant that leaked
        // the stderr pipe's write end open without ever writing to it. A
        // dropped sender would make `recv_timeout` return `Disconnected`
        // instantly, which proves nothing about the bound; a live, silent
        // sender is the only shape that actually exercises the timeout arm.
        let (tx, rx) = std::sync::mpsc::sync_channel::<String>(1);
        let reader = TurnReader {
            backend_state: Arc::clone(&backend.state),
            ask_grammar: Arc::clone(&backend.ask_grammar),
            version: None,
            sink: backend.sink.lock().expect("sink lock").clone(),
            data_dir: dir.path().to_path_buf(),
            execution_id: "e-stderr-never".to_string(),
            work_id: "w-stderr-never".to_string(),
            session_id: "s-stderr-never".to_string(),
            model: None,
            child: Arc::new(Mutex::new(child)),
            stderr_rx: Some(rx),
        };
        // Round-2 finding TH-R2-05: `reader.run` used to be called directly,
        // in-thread. That leaves the bounded-half assertion below unable to
        // fail cleanly against the one mutation it exists to catch —
        // `recv_timeout(STDERR_DRAIN_BUDGET)` turned into an unbounded
        // `recv()` — because the test harness has no per-test timeout, and
        // an infinite block just hangs the run rather than failing it. Doing
        // the read on its own thread and bounding *this* thread's wait with
        // `recv_timeout` turns that hang back into an ordinary, named
        // assertion failure.
        let started = std::time::Instant::now();
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            reader.run(stdout);
            let _ = done_tx.send(());
        });
        let watchdog = STDERR_DRAIN_BUDGET * 6;
        if done_rx.recv_timeout(watchdog).is_err() {
            panic!(
                "reader.run did not return within {watchdog:?} — STDERR_DRAIN_BUDGET's \
                 bound was not enforced (e.g. `recv_timeout` mutated to an unbounded \
                 `recv()`), which used to hang this test forever instead of failing it"
            );
        }
        let elapsed = started.elapsed();
        assert!(
            elapsed >= STDERR_DRAIN_BUDGET,
            "a silent-but-live sender must be waited out for the full budget \
             ({STDERR_DRAIN_BUDGET:?}), not returned early: took {elapsed:?}"
        );
        assert!(
            elapsed < STDERR_DRAIN_BUDGET * 3,
            "the wait must actually be bounded by the budget, not merely \
             coincidentally slow: took {elapsed:?}"
        );

        let events = emitted.lock().expect("sink lock");
        let ended = events
            .iter()
            .find(|e| e.kind == "conversation.turn.ended")
            .expect("every turn ends with this event");
        assert_eq!(
            ended.payload["stderr"], "",
            "no stderr ever arrived, so the field must default to empty rather \
             than the turn outcome hanging on it: {}",
            ended.payload
        );
        // Held alive (see above) through every assertion; dropped only now
        // that `reader.run` has already returned.
        drop(tx);
    }

    fn test_execution(model: Option<&str>) -> ClaudeExecution {
        ClaudeExecution {
            session_id: "s".to_string(),
            work_id: "w".to_string(),
            cwd: PathBuf::from("/tmp"),
            model: model.map(str::to_string),
            executable: PathBuf::from("claude"),
            env: BTreeMap::new(),
            permission_args: vec![],
            instruction_policy: InstructionPolicy::Suppress,
            turns: 1,
            turn: TurnState::Finished(TurnOutcome {
                post_turn_summary: None,
                envelope: None,
                interrupted: false,
                raw_blob: None,
                raw_error: None,
                stderr: String::new(),
            }),
            stopped: false,
            interrupt_requested: false,
            reader: None,
        }
    }
}
