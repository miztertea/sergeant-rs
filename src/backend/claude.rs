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
//! Root constraint (measured): `--dangerously-skip-permissions` is refused
//! outright under root/sudo — `--dangerously-skip-permissions cannot be
//! used with root/sudo privileges for security reasons`, exit 1 — unless
//! the environment sets `IS_SANDBOX=1`. The adapter does not set that
//! variable itself; the operator opts in via profile env or daemon
//! environment, because silently bypassing the CLI's own refusal would be
//! this adapter making a security decision that belongs to the human.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};

use serde_json::{Value, json};

use super::{
    Backend, BackendError, BackendSignal, Capabilities, EventSink, ExecutionHandle, NativeEvent,
    NativeState, Observation, ProbeReport, ResumeRequest, RuntimeScope, StartRequest,
};
use crate::domain::event::{EventDraft, EventSource};
use crate::domain::profile::Profile;
use crate::runtime::blob::BlobStore;

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
    "--dangerously-skip-permissions",
];

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
    pub fn new(data_dir: &Path) -> Self {
        Self {
            executable: PathBuf::from("claude"),
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
    /// Number of turns launched so far.
    turns: u32,
    turn: TurnState,
    stopped: bool,
    /// Interrupt was requested while a turn was in flight; consumed by the
    /// reader thread into `TurnOutcome::interrupted`.
    interrupt_requested: bool,
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
                    detail: format!("capability probe: cannot run {exe:?} --version: {e}"),
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
            };
        };
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
            };
        }
        let help_out = match Command::new(exe).arg("--help").output() {
            Ok(out) => out,
            Err(e) => {
                return ProbeOutcome {
                    available: false,
                    detail: format!("capability probe: cannot run {exe:?} --help: {e}"),
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
            };
        }
        ProbeOutcome {
            available: true,
            detail: format!(
                "claude {version_text}; all {} required flags present",
                REQUIRED_FLAGS.len()
            ),
        }
    }

    /// Resolve one execution's launch configuration from adapter config plus
    /// the profile (§14). One function, used by both START and RESUME, so a
    /// re-adopted execution cannot end up launching under different rules
    /// than the one it re-adopts — in particular, cannot silently escalate
    /// past a profile-pinned `--permission-mode`.
    fn launch_config(&self, profile: Option<&Profile>) -> LaunchConfig {
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
        // Permission mode is profile-pinned; the default is L2's production
        // default. Root refusal + IS_SANDBOX constraint: module docs.
        let permission_args = match profile.and_then(|p| p.options.get("permission_mode")) {
            Some(mode) => vec!["--permission-mode".to_string(), mode.clone()],
            None => vec!["--dangerously-skip-permissions".to_string()],
        };
        LaunchConfig {
            executable,
            env,
            permission_args,
        }
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
        let (exe, cwd, env, permission_args, session_id, model, first_turn, work_id) = {
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
            )
        };

        let mut command = Command::new(&exe);
        command
            .arg("-p")
            .arg("--verbose")
            .args(["--output-format", "stream-json"])
            // L2's capture hazard: the target repo's project memory must not
            // be able to install a different identity on the execution agent.
            .args(["--setting-sources", "user"]);
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
        let stderr_buf = Arc::new(Mutex::new(String::new()));
        if let Some(mut stderr) = stderr {
            let stderr_buf = Arc::clone(&stderr_buf);
            std::thread::spawn(move || {
                let mut text = String::new();
                let _ = stderr.read_to_string(&mut text);
                *stderr_buf.lock().expect("stderr buffer lock") = text;
            });
        }

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
            sink: self.sink.lock().expect("claude sink lock").clone(),
            data_dir: self.config.data_dir.clone(),
            execution_id: execution_id.to_string(),
            work_id,
            session_id,
            model,
            child,
            stderr_buf,
        };
        std::thread::spawn(move || reader.run(stdout));
        Ok(())
    }
}

/// Everything the per-turn stdout reader thread needs. It owns turn
/// ingestion end to end: raw archive, normalization, outcome recording.
struct TurnReader {
    backend_state: Arc<Mutex<AdapterState>>,
    sink: Option<EventSink>,
    data_dir: PathBuf,
    execution_id: String,
    work_id: String,
    session_id: String,
    model: Option<String>,
    child: Arc<Mutex<Child>>,
    stderr_buf: Arc<Mutex<String>>,
}

impl TurnReader {
    fn run(self, stdout: std::process::ChildStdout) {
        let mut raw = String::new();
        let mut envelope: Option<Value> = None;
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
            raw.push_str(&line);
            raw.push('\n');
            if let Ok(value) = serde_json::from_str::<Value>(&line) {
                self.ingest_line(&value, &mut envelope);
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

        let stderr = self.stderr_buf.lock().expect("stderr buffer lock").clone();
        let mut state = self
            .backend_state
            .lock()
            .expect("claude adapter state lock");
        let Some(execution) = state.executions.get_mut(&self.execution_id) else {
            return;
        };
        let interrupted = execution.interrupt_requested;
        execution.turn = TurnState::Finished(TurnOutcome {
            envelope: envelope.clone(),
            interrupted,
            raw_blob: raw_blob.clone(),
            raw_error: raw_error.clone(),
            stderr: stderr.clone(),
        });
        drop(state);

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
    fn ingest_line(&self, value: &Value, envelope: &mut Option<Value>) {
        match value.get("type").and_then(Value::as_str) {
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

    fn start(&self, request: &StartRequest) -> Result<ExecutionHandle, BackendError> {
        // Version/capability gate: an unmeasured CLI is refused with the
        // probe's own evidence (fail closed, structured, actionable).
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

        // Launch details pinned now, from profile + config (§14: launch
        // configuration only — credentials stay with the native harness).
        let LaunchConfig {
            executable,
            env,
            permission_args,
        } = self.launch_config(request.profile.as_ref());

        // The session identity exists before the process does (see module
        // docs: this is both the nested-env hazard fix and the L6 fix).
        let session_id = new_session_uuid();
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
                    turns: 0,
                    turn: TurnState::Unlaunched,
                    stopped: false,
                    interrupt_requested: false,
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
    fn interrupt(&self, handle: &ExecutionHandle) -> Result<(), BackendError> {
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
        Ok(())
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
    /// looks like: it would silently replace a profile's `--permission-mode`
    /// with `--dangerously-skip-permissions` (a security decision that
    /// belongs to the human, as the module docs say of the root refusal),
    /// drop the model pin so every later turn verified as "unpinned" while
    /// the work's journal still records a pin, and journal normalized events
    /// under an empty work id. A pin the caller does not re-supply is
    /// genuinely absent — reported as unpinned, which is then true.
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
        } = self.launch_config(request.profile.as_ref());
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
                turns: 1, // there was at least the turn that created it
                // Adoption draws no conclusion — and says so. (It used to
                // borrow an interrupted turn's shape here, which made OBSERVE
                // report "turn interrupted by request" for an interrupt
                // nobody requested.)
                turn: TurnState::Adopted,
                stopped: false,
                interrupt_requested: false,
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
    fn stop(&self, handle: &ExecutionHandle) -> Result<(), BackendError> {
        self.interrupt(handle)?;
        let mut state = self.lock();
        self.check_identity(&state, handle)?;
        let execution = state
            .executions
            .get_mut(&handle.execution_id)
            .expect("presence checked above");
        execution.stopped = true;
        Ok(())
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
        verdict => Observation {
            native: NativeState::Exited,
            signal: BackendSignal::StageCompleted {
                summary: Some(result_text),
            },
            evidence: Some(format!(
                "session {session}; model_pin={}; raw={}",
                verdict.as_json(),
                outcome.raw_evidence()
            )),
        },
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

    fn test_execution(model: Option<&str>) -> ClaudeExecution {
        ClaudeExecution {
            session_id: "s".to_string(),
            work_id: "w".to_string(),
            cwd: PathBuf::from("/tmp"),
            model: model.map(str::to_string),
            executable: PathBuf::from("claude"),
            env: BTreeMap::new(),
            permission_args: vec![],
            turns: 1,
            turn: TurnState::Finished(TurnOutcome {
                envelope: None,
                interrupted: false,
                raw_blob: None,
                raw_error: None,
                stderr: String::new(),
            }),
            stopped: false,
            interrupt_requested: false,
        }
    }
}
