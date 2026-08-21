//! Codex adapter: `codex exec` non-interactive turns over a durable thread
//! (W1 of the *Sergeant speaks Codex* sprint,
//! `knowledge/evidence/resources/h-series/w1-spec.md`, closing deviation D6).
//!
//! Every claim below carries the spec's own provenance marker. Unless stated
//! otherwise, **[measured]** means codex-cli **0.149.0**, standalone musl
//! build at `~/.local/bin/codex`, measured on Cerberus 2026-08-21 (the H0
//! evidence packet beside the spec, re-measured token-free while the spec was
//! authored). [`MEASURED_FLOOR`] records that version — as **provenance, not
//! a gate** (owner ruling R1): a build below it is still `available`, with an
//! unmeasured-provenance detail, never refused. What *is* refused is a CLI
//! whose version cannot be parsed at all, or whose `--help` does not offer
//! this adapter's launch grammar (panel amendment A2) — neither is a
//! version-policy question.
//!
//! - `codex exec --json --skip-git-repo-check -C <cwd> [-m <model>]` writes
//!   line-delimited JSON to stdout, prompt on stdin; stderr carries only the
//!   `Reading additional input from stdin...` banner and plain-text errors
//!   [measured — upstream's own docs claim the opposite and are wrong on this
//!   build].
//! - `codex exec resume <thread_id> --json --skip-git-repo-check [-m <model>]`
//!   continues the same conversation from a different process, with
//!   `Command::current_dir` doing the work `-C` did on turn 1: `exec resume`
//!   has no `-C`/`--add-dir`/`-s`/`-p` on this build [measured-negative,
//!   Appendix B].
//! - The thread id is **minted by the harness, not by sergeant**: it first
//!   appears on the wire as `{"type":"thread.started","thread_id":"…"}`
//!   [measured, first line of every recorded stream]. Unlike Claude, this
//!   adapter cannot pre-mint an identity at PREPARE — [`PreparedExecution`]'s
//!   own contract blesses that as honest, not a failure — so LAUNCH spawns
//!   the first turn and **waits, bounded, for `thread.started`**
//!   ([`THREAD_ID_BUDGET`]) before returning a handle at all.
//! - `agent_message` text is transcript content, never tool evidence (§4.3).
//!   Three live recordings on `gpt-5.6-luna` show the model narrating a
//!   specific command failure with **no** corroborating `command_execution`
//!   item anywhere in the stream and no filesystem effect [measured, H0 §C.3
//!   finding 2]. This module's decoder has exactly one code path that
//!   produces `tool.*` events — the `command_execution` item — and no branch
//!   anywhere reads `agent_message.text` for evidence of anything. A unit test
//!   replays the recorded narration turn and asserts zero tool events came out
//!   of it.
//! - Model pins: `-m` is composed on every turn including resumes
//!   [measured: present on `exec resume --help`], and a bad slug fails loud —
//!   `item.completed{error}` metadata warning, `{"type":"error"}`, then
//!   `turn.failed`, exit 1 [measured, the recorded `runB` fixture]. But
//!   `turn.completed.usage` carries **no model field**
//!   [measured-negative, every recorded `turn.completed` line]: substitution
//!   detection, Claude's third pin-verification layer, does not exist on this
//!   transport. [`model_pin_evidence`] reports a pin as `attempted`, never as
//!   `honored`.
//! - A resume turn's own `thread.started` is checked against the thread this
//!   execution resumed [measured twice, independently, two nonces, two fresh
//!   OS processes, same id echoed]: a mismatch ([`thread_pin_mismatch`]) fails
//!   the turn regardless of how it otherwise ended, checked *before* the
//!   completion branch.
//! - A SIGKILLed turn leaves the durable rollout intact and the thread fully
//!   resumable — but the harness never marks the killed turn's row failed on
//!   its own, and the next `codex exec resume` silently starts a *new* turn on
//!   the same rollout with no stale-lock complaint [measured, H0]. That is
//!   why a turn that exits with no `turn.completed`/`turn.failed` and no
//!   requested interrupt is `NativeState::Unknown`, never inferred `Exited`
//!   (§5.2) — the same fail-closed row issue #46 was filed about on Claude.
//! - `codex exec` spawns shell commands as its own children
//!   (`/bin/bash -lc '…'` [measured, Appendix A.2]), so INTERRUPT kills the
//!   turn's whole **process group**, not just the direct child (§5.5) —
//!   otherwise a background grandchild survives the kill. The group id is
//!   recorded at spawn and signalled unconditionally, because the group
//!   outlives its leader: a backgrounded command is still running after the
//!   codex process has exited and been reaped, which is exactly when every
//!   liveness check says there is nothing left to kill.
//! - There is no approval or ask channel on this transport at all
//!   [measured-negative: no `-a`/`--ask-for-approval` flag, exit 2 if passed;
//!   binary-string: `"command execution approval is not supported in exec
//!   mode…"`, `"request_user_input is not supported in exec mode…"`].
//!   `Capabilities::ask` is `false`, structurally and unconditionally — never
//!   guessed from prose, for the same §4.3 reason.
//!
//! **The crash window this adapter does not close (stated, not papered
//! over).** Because the thread id cannot be pre-minted, the journaled
//! `execution.reserved` carries `native_id: null`. A daemon that dies between
//! LAUNCH's spawn and the engine's `execution.started` commit leaves a live
//! `codex exec` whose thread id is in no journal, plus a durable rollout file,
//! and nothing here reaps them. Restart reconciliation fails the work closed
//! (no native id to reconcile, so OBSERVE returns `UnknownExecution`), which
//! is the right direction — but the orphan is real. Closing it needs an
//! identity codex does not offer on this transport (W3's app-server, whose
//! `thread/start` returns the id to a long-lived client, is where it can
//! actually close).
//!
//! **Durable global state this adapter causes (recorded, not fixed, §3.8).**
//! Running `codex exec` in a directory permanently records
//! `[projects."<abs-path>"] trust_level = "trusted"` in
//! `$CODEX_HOME/config.toml`, surviving deletion of the directory [measured
//! before/after]. Sergeant mints a fresh surface per Work, so a busy daemon
//! accumulates one stale trust entry per Work, forever, in a file the human
//! owns. This module records the fact in the probe's `detail` (when
//! `CODEX_HOME` resolves to the operator's own `~/.codex`) and does not
//! garbage-collect another tool's config file.
//!
//! Not abstracted from `claude.rs` (R2 rung log, spec §1.2): `parse_version`
//! (different wire shape — codex prints the vendor token *first*),
//! `truncate` (5 lines, every owner keeps its own — `runtime/graph.rs` already
//! has a second copy), the launch-grammar liveness rule (codex's argv shape
//! and its extra `SurfaceAmbiguous` attribution are its own), and
//! `PinVerdict` (its `Honored`/`Substituted` arms are structurally
//! unreachable here). `EXECUTION_MODEL_CONTRACT` and `ENVIRONMENT_CONTRACT`
//! are copied, not imported, so an edit to Claude's prompt is never an
//! unreviewed edit to this one; a unit test pins that the environment text
//! matches today.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use serde_json::{Value, json};

use super::{
    Backend, BackendError, BackendSignal, BindingSummary, Capabilities, Completion, EventSink,
    ExecutionHandle, NativeEvent, NativeState, Observation, PreparedExecution, ProbeReport,
    ResumeRequest, RuntimeScope, StartRequest,
};
use crate::domain::event::{EventDraft, EventSource};
use crate::domain::profile::Profile;
use crate::platform::process::ProcessArgv;
use crate::runtime::blob::BlobStore;
use crate::runtime::graph::{
    KIND_CONVERSATION_ASSISTANT_COMPLETED, KIND_CONVERSATION_TURN_ENDED, KIND_CONVERSATION_USER,
    KIND_TOOL_COMPLETED, KIND_TOOL_REQUESTED, KIND_USAGE_UPDATED,
};

// ------------------------------------------------------------------ consts

/// Name this backend registers under.
pub const CODEX_BACKEND_NAME: &str = "codex";

/// Environment variable naming the `codex` executable to use — the
/// `SGT_CLAUDE_BIN` pattern (`claude.rs`'s `CLAUDE_BIN_ENV`), byte-for-byte
/// and for the same reason: *which* `codex` is on the daemon's PATH is an
/// operator fact, and `sgt doctor` must be able to report on the same binary
/// the daemon would run.
pub const CODEX_BIN_ENV: &str = "SGT_CODEX_BIN";

/// The codex-cli version every behavioural claim in this module was measured
/// against (H0 evidence packet, 2026-08-21, Cerberus). **This is provenance,
/// not a gate** (owner ruling R1: "version floors are provenance, not gates …
/// below it: report honestly, never block"). A build below this floor is
/// `available: true` with an unmeasured-provenance detail; a build at or
/// above it is `available: true` with measured provenance. Neither is
/// refused. What *is* refused is a CLI whose version cannot be parsed at all,
/// or whose `--help` does not offer the flags this module's launch grammar
/// composes — neither of which is a version-policy question (panel amendment
/// A2).
pub const MEASURED_FLOOR: (u64, u64, u64) = (0, 149, 0);

/// Flags the probe requires to appear in `codex exec --help`. Each is
/// load-bearing for this adapter's launch grammar; a build without one of
/// them is a CLI whose grammar this adapter has never measured, so it is
/// refused (A2 condition 2 — not a version question).
pub const REQUIRED_EXEC_FLAGS: &[&str] = &[
    "--json",                // the whole event transport (§4)
    "--model",               // model pin (§3.2); `-m` is its short form
    "--cd",                  // working root (§3.2); `-C` is its short form
    "--skip-git-repo-check", // measured-necessary in an untrusted dir (§3.3)
    "--profile",             // presence lets §3.5's refusal name a flag that
    //   exists rather than one this build never had
    "--sandbox", // not composed by W1; presence is what makes
    //   W3's enforcement mapping a change, not a
    //   discovery
    "--ephemeral", // the durable-persistence opt-out this adapter
                   //   deliberately never passes (§3.3)
];

/// Flags the probe requires to appear in `codex exec resume --help`.
/// Measured-negative and load-bearing: `resume` has **no** `--cd`,
/// `--add-dir`, `--sandbox` or `--profile` on 0.149.0, which is why the
/// resume grammar looks different from the first-turn grammar and why §3.5
/// refuses a codex-native profile layer outright.
pub const REQUIRED_RESUME_FLAGS: &[&str] =
    &["--json", "--model", "--skip-git-repo-check", "--ephemeral"];

/// Subcommands `codex exec --help` must list.
pub const REQUIRED_EXEC_SUBCOMMANDS: &[&str] = &["resume"];

/// ADR 0007(a)'s execution-model half, codex-worded (spec §3.4). Reworded
/// from `claude.rs`'s constant of the same purpose rather than imported: the
/// two adapters' execution models are not guaranteed to stay identical (a
/// future app-server adapter may have callbacks), so an edit to one must not
/// silently change the other (spec §1.3).
pub const EXECUTION_MODEL_CONTRACT: &str = "\
Execution model: this is a single non-interactive turn (`codex exec`). You get one turn and no \
callbacks — nothing wakes you when a command you backgrounded finishes after you end your turn. \
There is also no approval channel and no way to ask a human anything during this turn: any \
action that would need a human decision is refused by the harness and returned to you as a \
failure, so plan around it rather than waiting for one. If a command might take a while, run it \
in the foreground with an adequate timeout and wait for it to finish before ending your turn.";

/// `claude.rs`'s `ENVIRONMENT_CONTRACT`, copied verbatim rather than imported
/// (spec §1.3): its text already names `sgt codex` and is transport-agnostic,
/// but a `codex.rs -> claude.rs` import would make an edit to the Claude
/// adapter's prompt an unreviewed edit to this one. A unit test
/// (`the_environment_contract_matches_claudes_today`) pins that the two texts
/// are equal *today*, so a divergence is a decision, not drift nobody
/// noticed.
pub const ENVIRONMENT_CONTRACT: &str = "\
Environment: if this session was reached through `sgt claude` (or `sgt codex`/`opencode`/\
`goose`), your PATH was deliberately composed before this turn was launched to include your \
toolchain (e.g. `~/.cargo/bin`, `~/.local/bin`), and you are bound to the estate that launch \
discovered — sergeant's daemon and every actor beneath it inherit that same environment. This \
does not hold for a daemon reached any other way: a terminal that never went through `sgt \
<harness>` inherits whatever environment it happened to have. If a tool you expect is missing, \
that is more likely an unenriched PATH than a permissions fault — run `sgt doctor` to check what \
this installation's environment actually guarantees before assuming otherwise.";

/// §10.1's section header, codex-local copy of `claude.rs`'s private
/// constant of the same text (spec §1.3 — same reasoning as
/// [`ENVIRONMENT_CONTRACT`]).
const MUTATION_SURFACE_HEADER: &str = "\
Mutation surface: this Work may modify exactly the worktree(s) listed below, and nothing else. \
The estate root, the `repos/` mounts those worktrees were cut from, unselected repositories, \
other Works' surfaces, and any other path on this machine are outside what this Work is \
authorized to change. Each worktree is already checked out on its own branch at its own base \
commit:";

/// How long LAUNCH waits for `thread.started` before concluding the launch
/// failed (§3.1). Generous by an order of magnitude: `thread.started` is the
/// first line of every measured stream, emitted before any model call — even
/// in the one recorded failing run it preceded the model-metadata error.
const THREAD_ID_BUDGET: Duration = Duration::from_secs(30);

/// How long the turn reader waits for stderr after the turn's process has
/// been reaped — `claude.rs`'s identical fix for the same race (both pipes
/// reach EOF at the same instant; a reader that snapshots a shared buffer the
/// instant it closes stdout is racing the thread still filling it). Codex
/// needs it *more*, not less: for the pre-turn refusals (§3.1) stderr is the
/// entire evidence.
const STDERR_DRAIN_BUDGET: Duration = Duration::from_secs(5);

/// Bounded tail of a `command_execution` item's aggregated output kept
/// inline in `tool.completed`'s payload. The full bytes are never dropped —
/// they are in the turn's raw blob by construction (§20) — this only bounds
/// what an unrelated unbounded command output costs the journal a second
/// time. `docker.rs`'s `TAIL_BYTES` bounded-tail-plus-blob pattern is the
/// precedent (spec §4.2).
const TOOL_OUTPUT_TAIL: usize = 1024;

/// One new module-local normalized kind (spec §1.2): a codex-specific harness
/// or stream-level problem that is journaled but is never, by itself, a
/// terminal (§4.4). The precedent is `claude.rs`'s own module-local
/// `"conversation.turn.grammar_unmeasured"` literal — the graph projection
/// ignores kinds it does not fold, and adding one to `graph.rs` for a
/// projection that would not fold it is a core edit this wave does not make
/// (R3). Unlike that literal, this one is a proper `pub const KIND_*` (its
/// own events need a stable name other code can match on), so it *is* named
/// in `api::SSE_EVENT_KINDS` — `tests/m6_surfaces.rs`'s `t6` scans the crate
/// for every `KIND_*` constant and fails if a journaled one is missing from
/// that list, and this is a journaled kind like any other.
pub const KIND_TURN_HARNESS_ERROR: &str = "conversation.turn.harness_error";

// ----------------------------------------------------------------- config

/// Launch configuration for the adapter, resolved once at construction from
/// the daemon's own environment.
#[derive(Debug, Clone)]
pub struct CodexConfig {
    /// The CLI executable (a profile may override per execution).
    pub executable: PathBuf,
    /// Sergeant's data dir; raw per-turn stdout is archived to its blob store
    /// (§20).
    pub data_dir: PathBuf,
    /// Where codex keeps its durable session state (`$CODEX_HOME`). `None`
    /// resolves to the environment variable or `~/.codex` at probe/launch
    /// time. Tests point this at a scratch directory.
    pub codex_home: Option<PathBuf>,
    /// Extra environment for every spawned turn.
    pub env: BTreeMap<String, String>,
    /// Override for [`THREAD_ID_BUDGET`], `None` in every production path.
    /// A per-instance field rather than an environment variable: each test
    /// builds its own `CodexConfig`, so a shrunk budget in one test's
    /// `CodexBackend` can never leak into another test's `launch()` — no
    /// process-global mutable state, no `--test-threads` ordering hazard, no
    /// `unsafe { std::env::set_var }` to serialize.
    pub thread_id_budget: Option<Duration>,
}

impl CodexConfig {
    /// Config for a daemon owning `data_dir`, with the system `codex`.
    pub fn new(data_dir: &Path) -> Self {
        Self {
            executable: std::env::var_os(CODEX_BIN_ENV)
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("codex")),
            data_dir: data_dir.to_path_buf(),
            codex_home: None,
            env: BTreeMap::new(),
            thread_id_budget: None,
        }
    }
}

// ------------------------------------------------------------------- probe

/// Whether the installed build is at or above [`MEASURED_FLOOR`] (R1: this is
/// provenance carried alongside `available`, never a gate on it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VersionProvenance {
    /// `>= MEASURED_FLOOR` — this module's measurements apply directly.
    Measured,
    /// `< MEASURED_FLOOR` — available, but nothing here was re-measured
    /// against this exact build.
    BelowFloor,
}

/// What `codex login status` said (§2.6). The probe never gates on this —
/// only the live test suite does (A3).
#[derive(Debug, Clone, PartialEq, Eq)]
enum AuthState {
    /// The first line began `Logged in using …` [measured verbatim on this
    /// host: "Logged in using ChatGPT", exit 0 — on **stderr**, not stdout,
    /// when the process is not attached to a TTY, which a spawned child
    /// never is; see `run_auth_probe`].
    LoggedIn {
        /// The method named after "Logged in using ".
        method: String,
    },
    /// Anything else. Deliberately not called "logged out": the logged-out
    /// string was never measured on this build, and naming a state from an
    /// unmeasured string is the promotion §0.2 forbids. The raw first line
    /// travels inside, because that is what an operator acts on.
    Unreported(String),
}

/// Outcome of the registration-time capability/version/auth probe.
///
/// `version`, `provenance` and `auth` are carried for a future `sgt doctor`
/// / provenance reader (W2's own hand-off, spec §2.4/§8.2) even though
/// nothing in this wave reads them back out — `detail` already carries their
/// rendering for today's one reader (`ProbeReport`). Allowed dead code
/// rather than dropped: W1 must not invent W2's reader, but must not throw
/// away the fields it will need either.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ProbeOutcome {
    available: bool,
    detail: String,
    version: Option<String>,
    provenance: Option<VersionProvenance>,
    auth: Option<AuthState>,
}

/// Parse `"codex-cli 0.149.0"` into a comparable triple. The vendor token,
/// when present, is skipped by taking the *last* whitespace-separated token;
/// a bare `"0.149.0"` (no vendor token at all) is accepted the same way. The
/// patch segment is parsed up to its first non-digit, so a pre-release
/// suffix (`"0.149.0-rc.1"`) still yields a comparable triple — the full
/// string still travels in the probe's `detail`, never silently dropped.
fn parse_codex_version(text: &str) -> Option<(u64, u64, u64)> {
    let candidate = text.trim().rsplit(char::is_whitespace).next()?;
    if candidate.is_empty() {
        return None;
    }
    let mut parts = candidate.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch_field = parts.next()?;
    let patch_digits: String = patch_field
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    if patch_digits.is_empty() {
        return None;
    }
    let patch: u64 = patch_digits.parse().ok()?;
    Some((major, minor, patch))
}

/// Parse `codex login status`'s stdout into an [`AuthState`] (§2.6). Only the
/// first line is read — that is what a real run carries — and only the
/// measured `"Logged in using …"` shape is recognized as logged in.
fn parse_auth_status(stdout: &str) -> AuthState {
    let first_line = stdout.lines().next().unwrap_or("").trim();
    match first_line.strip_prefix("Logged in using ") {
        Some(method) => AuthState::LoggedIn {
            method: method.trim().to_string(),
        },
        None => AuthState::Unreported(first_line.to_string()),
    }
}

/// Every entry of `required` not found in `help` (substring containment,
/// exactly `claude.rs`'s `help_text.contains(flag)` rule). Long forms only:
/// the help renders `-m, --model <MODEL>`, so `"--model"` matches and is
/// stable against a short-form change.
fn missing_flags(help: &str, required: &[&'static str]) -> Vec<&'static str> {
    required
        .iter()
        .copied()
        .filter(|flag| !help.contains(flag))
        .collect()
}

// -------------------------------------------------------------- launch grammar

/// One execution's resolved launch configuration (§14 applied to this CLI).
/// One function ([`CodexBackend::launch_config`]) produces it for both LAUNCH
/// and RESUME, so a re-adopted execution cannot launch under different rules
/// than the one it re-adopts.
#[derive(Debug, Clone)]
struct LaunchConfig {
    executable: PathBuf,
    env: BTreeMap<String, String>,
    codex_home: Option<PathBuf>,
}

/// Turn 1's argv, after `<executable>` (spec §3.2): `exec --json
/// --skip-git-repo-check -C <cwd> [-m <model>]`. Prompt travels on stdin, no
/// positional argument — see the module docs for why.
fn first_turn_argv(cwd: &Path, model: Option<&str>) -> Vec<String> {
    let mut argv = vec![
        "exec".to_string(),
        "--json".to_string(),
        "--skip-git-repo-check".to_string(),
        "-C".to_string(),
        cwd.to_string_lossy().into_owned(),
    ];
    if let Some(model) = model {
        argv.push("-m".to_string());
        argv.push(model.to_string());
    }
    argv
}

/// Turn N >= 2's argv, after `<executable>` (spec §3.2): `exec resume
/// <thread_id> --json --skip-git-repo-check [-m <model>]`. The thread id sits
/// immediately after `resume`, before any flag — what makes §5.4's liveness
/// rule an adjacency check rather than a substring search. `-C` is never
/// passed here: `exec resume` has no such flag on this build
/// [measured-negative]; `Command::current_dir` is the only mechanism left,
/// and is set on every spawn regardless of turn number.
fn resume_turn_argv(thread_id: &str, model: Option<&str>) -> Vec<String> {
    let mut argv = vec![
        "exec".to_string(),
        "resume".to_string(),
        thread_id.to_string(),
        "--json".to_string(),
        "--skip-git-repo-check".to_string(),
    ];
    if let Some(model) = model {
        argv.push("-m".to_string());
        argv.push(model.to_string());
    }
    argv
}

/// §10.1's section body: the header, then one line per bound repository —
/// identical shape to `claude.rs`'s `mutation_surface_section`, codex-local
/// copy per spec §1.3.
fn mutation_surface_section(bindings: &[BindingSummary]) -> String {
    let mut section = String::from(MUTATION_SURFACE_HEADER);
    for binding in bindings {
        section.push_str(&format!(
            "\n- {}: {} (branch {}, cut from {} at {})",
            binding.repository,
            binding.worktree_path.display(),
            binding.work_branch,
            binding
                .base_branch
                .as_deref()
                .unwrap_or("no named base branch (detached admission)"),
            binding.base_sha,
        ));
    }
    section
}

/// The full first-turn prompt, same five-section fixed order as `claude.rs`
/// (spec §3.4): [`EXECUTION_MODEL_CONTRACT`], [`ENVIRONMENT_CONTRACT`], the
/// mutation surface (omitted entirely when `bindings` is empty), the intent,
/// then `CONTEXT.md` — the last two carried verbatim and uninterpreted (§12).
fn compose_launch_prompt(request: &StartRequest) -> String {
    let mut sections = vec![
        EXECUTION_MODEL_CONTRACT.to_string(),
        ENVIRONMENT_CONTRACT.to_string(),
    ];
    if !request.bindings.is_empty() {
        sections.push(mutation_surface_section(&request.bindings));
    }
    sections.push(request.intent.clone());
    sections.push(request.context.clone());
    sections.join("\n\n")
}

/// §3.3's `--add-dir` rung: every binding path this request carries that does
/// not lie at or under `cwd`. Empty for sergeant's own binding shape today
/// (`runtime/surface.rs` always composes worktree paths at or under the
/// surface root) — this function exists so that assumption is *checked*
/// rather than trusted, and a future binding shape that breaks it is a fact
/// in the launch evidence, not a silent gap.
fn bindings_outside_cwd(cwd: &Path, bindings: &[BindingSummary]) -> Vec<PathBuf> {
    bindings
        .iter()
        .filter(|binding| !binding.worktree_path.starts_with(cwd))
        .map(|binding| binding.worktree_path.clone())
        .collect()
}

/// Layer 1 of pin verification (spec §3.6): refuse only an empty or
/// whitespace-only pin. Claude's provider-qualification rule is deliberately
/// **not** copied — no codex refusal shape for a slash-qualified slug has
/// been measured, and `--oss`/`--local-provider` are real surfaces on this
/// CLI, so inventing a refusal here could refuse a pin that is actually legal
/// (e.g. `"openai/gpt-5.6-luna"`).
fn preflight_model_pin(model: &str) -> Result<(), String> {
    if model.trim().is_empty() {
        return Err("model pin is empty".to_string());
    }
    Ok(())
}

/// The model-pin evidence this transport can honestly offer (spec §3.6):
/// `turn.completed.usage` carries no model field on this build
/// [measured-negative], so substitution is undetectable and "honored" is
/// never emitted by this adapter — only `attempted` or `unpinned`.
fn model_pin_evidence(requested: Option<&str>) -> Value {
    match requested {
        None => json!({"verdict": "unpinned"}),
        Some(requested) => json!({
            "verdict": "attempted",
            "requested": requested,
            "detail": "codex-cli 0.149.0's exec event stream carries no model field on \
                       turn.completed, so this adapter reports the pin as attempted and never \
                       as honored; substitution is undetectable on this transport",
        }),
    }
}

/// §3.7's honest analog of Claude's substitution check: a resume turn whose
/// own `thread.started` names a different thread than the one this execution
/// resumed ran somewhere else, whatever it produced. `None` when they agree
/// (including when nothing was seen to compare, which the caller never
/// invokes this with).
fn thread_pin_mismatch(expected: &str, seen: &str) -> Option<String> {
    if expected == seen {
        None
    } else {
        Some(format!(
            "thread pin not honored: resumed {expected}, stream announced {seen}"
        ))
    }
}

// ------------------------------------------------------------------ decoding

/// One finished turn's terminal shape, before any process-exit evidence is
/// folded in (that happens in [`classify_terminal`]).
#[derive(Debug, Clone, Default)]
enum Terminal {
    /// No `turn.completed`/`turn.failed` line was ever seen.
    #[default]
    None,
    /// `turn.completed` was seen. The `usage` object itself travels on
    /// [`TurnAccumulator::usage`], not duplicated here.
    Completed,
    /// `turn.failed` was seen; the API/harness message travels here.
    Failed {
        /// `error.message`, verbatim.
        message: String,
    },
}

/// The whole decoder: folds one turn's line-delimited JSON stream into
/// normalized events plus the honest counts §4.3 asks for. Pure — no I/O, no
/// process — which is what makes the fixture-driven suite possible with no
/// `codex` binary anywhere in the loop.
#[derive(Debug, Default)]
struct TurnAccumulator {
    /// `thread_id` from this turn's own `thread.started` line, if seen.
    thread_id: Option<String>,
    /// Count of `agent_message` items completed this turn.
    message_items: u32,
    /// Count of `command_execution` items completed this turn.
    tool_items: u32,
    /// Type strings of every unrecognized item type seen (§4.2's "archived
    /// only, counted" row).
    unknown_items: Vec<String>,
    /// Lines that did not parse as JSON at all (§4.6).
    unparsed_lines: u32,
    /// The most recent `agent_message` item's text — §5.1's summary rule
    /// ("the last one of the turn").
    last_agent_message: Option<String>,
    /// The most recent bare stream `error` line's message (§4.4) — evidence
    /// for the ambiguous-unknown case, never a terminal by itself.
    last_error: Option<String>,
    /// `turn.completed`'s usage object, verbatim, when seen.
    usage: Option<Value>,
    /// This turn's terminal, if any.
    terminal: Terminal,
}

impl TurnAccumulator {
    fn new() -> Self {
        Self::default()
    }

    /// Ingest one already-parsed line, returning the normalized events it
    /// produced (§4.2's mapping table). Malformed-line counting happens in
    /// the caller, which only reaches this function on a line that parsed.
    fn ingest_line(&mut self, value: &Value) -> Vec<NativeEvent> {
        let mut out = Vec::new();
        match value.get("type").and_then(Value::as_str) {
            Some("thread.started") => {
                if let Some(id) = value.get("thread_id").and_then(Value::as_str) {
                    self.thread_id = Some(id.to_string());
                }
            }
            Some("turn.started") => {}
            Some("item.started") => self.ingest_item(value.get("item"), true, &mut out),
            Some("item.completed") => self.ingest_item(value.get("item"), false, &mut out),
            Some("turn.completed") => {
                let usage = value.get("usage").cloned().unwrap_or(Value::Null);
                self.usage = Some(usage);
                self.terminal = Terminal::Completed;
            }
            Some("turn.failed") => {
                let message = value
                    .pointer("/error/message")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                out.push(NativeEvent {
                    kind: KIND_TURN_HARNESS_ERROR.to_string(),
                    payload: json!({"phase": "turn_failed", "message": message}),
                });
                self.last_error = Some(message.clone());
                self.terminal = Terminal::Failed { message };
            }
            // §4.4: an `error` line says something went wrong, not that the
            // turn is over. Journaled, remembered, concludes nothing.
            Some("error") => {
                let message = value
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                out.push(NativeEvent {
                    kind: KIND_TURN_HARNESS_ERROR.to_string(),
                    payload: json!({"phase": "stream_error", "message": message}),
                });
                self.last_error = Some(message);
            }
            // Anything else at the envelope level is archived-only: it is
            // still in the raw blob by construction, never decoded.
            _ => {}
        }
        out
    }

    fn ingest_item(&mut self, item: Option<&Value>, started: bool, out: &mut Vec<NativeEvent>) {
        let Some(item) = item else { return };
        let item_type = item.get("type").and_then(Value::as_str).unwrap_or("");
        let item_id = item.get("id").cloned().unwrap_or(Value::Null);
        match item_type {
            "agent_message" => {
                // No `item.started` for this type was ever observed on exec;
                // emitting a partial assistant message on an unmeasured shape
                // would double-count the completed one (§4.2).
                if started {
                    return;
                }
                self.message_items += 1;
                let text = item
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                self.last_agent_message = Some(text.clone());
                out.push(NativeEvent {
                    kind: KIND_CONVERSATION_ASSISTANT_COMPLETED.to_string(),
                    payload: json!({"thread_id": self.thread_id, "text": text, "item_id": item_id}),
                });
            }
            // §4.3: the *only* code path that produces `tool.*` events.
            // Nothing anywhere else in this decoder reads `agent_message`
            // text as evidence that a command ran.
            "command_execution" => {
                if started {
                    out.push(NativeEvent {
                        kind: KIND_TOOL_REQUESTED.to_string(),
                        payload: json!({
                            "id": item_id,
                            "name": "command_execution",
                            "input": {"command": item.get("command").cloned().unwrap_or(Value::Null)},
                        }),
                    });
                    return;
                }
                self.tool_items += 1;
                let exit_code = item.get("exit_code").cloned().unwrap_or(Value::Null);
                let status = item
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let is_error = exit_code.as_i64() != Some(0) || status != "completed";
                let output = item
                    .get("aggregated_output")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                out.push(NativeEvent {
                    kind: KIND_TOOL_COMPLETED.to_string(),
                    payload: json!({
                        "tool_use_id": item_id,
                        "is_error": is_error,
                        "exit_code": exit_code,
                        "status": status,
                        "output_tail": truncate(output, TOOL_OUTPUT_TAIL),
                    }),
                });
            }
            "error" => {
                // Unobserved on `item.started`; the one measured instance was
                // a warning on a turn that continued, never a terminal.
                if !started {
                    let message = item
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    out.push(NativeEvent {
                        kind: KIND_TURN_HARNESS_ERROR.to_string(),
                        payload: json!({"phase": "item_error", "message": message, "item_id": item_id}),
                    });
                }
            }
            other => {
                self.unknown_items.push(other.to_string());
            }
        }
    }
}

/// The shape one finished turn resolves to, from its own stream evidence plus
/// whether sergeant asked for the kill (spec §5.1/§5.2). A pure function of
/// [`TurnAccumulator`]'s terminal field and the interrupt bit — process exit
/// code and stderr are not decision inputs here, only evidence carried
/// separately into [`TurnOutcome`] for the ambiguous case's detail string.
#[derive(Debug, Clone)]
enum TerminalOutcome {
    /// `turn.completed` was seen.
    Completed,
    /// `turn.failed` was seen.
    Failed {
        /// `error.message`, verbatim.
        message: String,
    },
    /// No terminal arrived, but sergeant requested the kill: no conclusion
    /// about the stage, the conversation stays resumable.
    InterruptedRunning,
    /// No terminal arrived and nobody asked for that: §5.2's ambiguity,
    /// fails closed.
    AmbiguousUnknown,
}

fn classify_terminal(acc: &TurnAccumulator, interrupted: bool) -> TerminalOutcome {
    match &acc.terminal {
        Terminal::Completed => TerminalOutcome::Completed,
        Terminal::Failed { message } => TerminalOutcome::Failed {
            message: message.clone(),
        },
        Terminal::None if interrupted => TerminalOutcome::InterruptedRunning,
        Terminal::None => TerminalOutcome::AmbiguousUnknown,
    }
}

// ----------------------------------------------------------------- liveness

/// Positive identity (spec §5.4): some argv element is exactly `resume` and
/// the *next* one is exactly this thread id, and some element is exactly
/// `exec`. That is the grammar [`resume_turn_argv`] emits for every turn
/// after the first, and nothing else — never a substring match on a joined
/// command line (`claude.rs`'s `cmdline_names_session` doc explains at length
/// why that rule is a false-positive machine).
fn argv_names_thread(argv: &[String], thread_id: &str) -> bool {
    let names_resume = argv
        .windows(2)
        .any(|pair| pair[0] == "resume" && pair[1] == thread_id);
    names_resume && argv.iter().any(|arg| arg == "exec")
}

/// Weak, over-approximating attribution (spec §5.4): some element is exactly
/// `-C` (or `--cd`) and the next is exactly this execution's cwd, and some
/// element is exactly `exec`. A first turn carries no thread id — it does not
/// exist until the harness mints it — so this is all the evidence a first
/// turn leaves behind; it cannot distinguish this execution's turn from
/// another on the same surface, which is why it can only ever attribute
/// [`Attribution::SurfaceAmbiguous`], never a positive identity claim.
fn argv_names_surface(argv: &[String], cwd: &Path) -> bool {
    let cwd_str = cwd.to_string_lossy();
    let names_cwd = argv
        .windows(2)
        .any(|pair| (pair[0] == "-C" || pair[0] == "--cd") && pair[1] == cwd_str);
    names_cwd && argv.iter().any(|arg| arg == "exec")
}

/// Who a live process is attributed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Attribution {
    /// Its argv names this exact thread id after `resume`.
    ThreadId,
    /// Its argv only names this surface's cwd — could be any execution's
    /// first turn on the same surface.
    SurfaceAmbiguous,
}

/// What a process scan can say about a conversation's per-turn process.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Liveness {
    /// A running process's argv attributes it as below.
    Alive {
        /// Its pid.
        pid: u32,
        /// How strongly its argv attributes it to this thread.
        attribution: Attribution,
    },
    /// No running process attributes to this thread (or, when `cwd` is
    /// supplied, to this surface either).
    Dead,
    /// Liveness cannot be evidenced here. §25: the caller fails closed.
    Unknowable(String),
}

/// [`turn_liveness_among`]'s decision logic, taking the process scan as a
/// parameter (ADR 0002 D3, `claude.rs`'s `session_liveness_among`
/// precedent). `cwd` is `None` when the caller has no surface to check
/// against — the un-adopted OBSERVE-after-restart path, where a positive
/// `ThreadId` match is the only attribution that could ever fire (a
/// registered execution always already has a thread id, so its live turns'
/// argv always names `resume`, never only `-C`). `ThreadId` wins over
/// `SurfaceAmbiguous` whichever process order they are found in.
fn turn_liveness_among(
    thread_id: &str,
    cwd: Option<&Path>,
    skip_pid: u32,
    processes: Option<Vec<ProcessArgv>>,
) -> Liveness {
    let Some(processes) = processes else {
        return Liveness::Unknowable(
            "process liveness needs a process-listing mechanism; none is available here"
                .to_string(),
        );
    };
    let mut surface_hit: Option<u32> = None;
    for process in processes {
        if process.pid == skip_pid {
            continue;
        }
        if argv_names_thread(&process.argv, thread_id) {
            return Liveness::Alive {
                pid: process.pid,
                attribution: Attribution::ThreadId,
            };
        }
        if surface_hit.is_none()
            && let Some(cwd) = cwd
            && argv_names_surface(&process.argv, cwd)
        {
            surface_hit = Some(process.pid);
        }
    }
    match surface_hit {
        Some(pid) => Liveness::Alive {
            pid,
            attribution: Attribution::SurfaceAmbiguous,
        },
        None => Liveness::Dead,
    }
}

// ------------------------------------------------------------- adapter state

/// One finished turn's outcome, kept for OBSERVE.
#[derive(Debug, Clone)]
struct TurnOutcome {
    terminal: TerminalOutcome,
    /// §3.7: checked before the completion branch, fatal whatever else the
    /// turn produced.
    pin_mismatch: Option<String>,
    message_items: u32,
    tool_items: u32,
    unknown_items: Vec<String>,
    unparsed_lines: u32,
    last_agent_message: Option<String>,
    last_error: Option<String>,
    exit_code: Option<i32>,
    raw_blob: Option<String>,
    raw_error: Option<String>,
    stderr: String,
}

impl TurnOutcome {
    /// How the §20 archive turned out, rendered for evidence — a ref, a named
    /// failure, or an explicitly empty stream. No value means "absent for
    /// reasons unknown".
    fn raw_evidence(&self) -> String {
        match (&self.raw_blob, &self.raw_error) {
            (Some(blob), _) => blob.clone(),
            (None, Some(error)) => format!("unarchived ({error})"),
            (None, None) => "unarchived (the turn streamed nothing)".to_string(),
        }
    }
}

/// Turn lifecycle for one execution. Mirrors `claude.rs`'s `TurnState`
/// exactly, including why `Unlaunched`/`Adopted` are their own variants
/// rather than a borrowed `Finished` placeholder: fabricating either would
/// have OBSERVE state a fact ("interrupted by request", "a turn is running")
/// that never happened.
#[derive(Debug)]
enum TurnState {
    /// Registered, no turn launched yet.
    Unlaunched,
    /// Re-adopted after a restart (§15 RESUME): this daemon launched no turn
    /// here, and the in-flight turn's outcome at the time of the previous
    /// daemon's death is not something it can read.
    Adopted,
    /// A per-turn process is running.
    InFlight(Arc<Mutex<Child>>),
    /// The last turn finished (or was killed) and left this outcome. Boxed:
    /// `TurnOutcome` carries several owned `String`/`Vec` fields, and boxing
    /// keeps this enum from ballooning to its size on every `TurnState` that
    /// is not `Finished`.
    Finished(Box<TurnOutcome>),
}

/// Adapter-side record of one execution (one durable codex thread).
#[derive(Debug)]
struct CodexExecution {
    /// `None` only during the narrow in-process window between spawning turn
    /// 1 and its `thread.started` arriving — never visible to a caller, since
    /// LAUNCH does not return a handle until this is `Some`.
    thread_id: Option<String>,
    work_id: String,
    cwd: PathBuf,
    model: Option<String>,
    executable: PathBuf,
    env: BTreeMap<String, String>,
    codex_home: Option<PathBuf>,
    /// §3.3: any binding path this Work carries that does not lie at or
    /// under `cwd` — checked once at LAUNCH ([`bindings_outside_cwd`]) and
    /// carried into the launch evidence and every `conversation.turn.ended`,
    /// since `--add-dir` is never composed and this is the signal a future
    /// enforcement mapping (W3) would need.
    bindings_outside_cwd: Vec<PathBuf>,
    turns: u32,
    turn: TurnState,
    /// The process group id of the most recent turn, recorded at **spawn**
    /// (§5.5). `process_group(0)` makes the turn's direct child its own group
    /// leader, so this is that child's pid — but it is kept here, on the
    /// execution, rather than read back out of [`TurnState::InFlight`] at
    /// kill time, because the group outlives the leader: a command the turn
    /// started in the background stays in this group after the codex process
    /// itself has exited and been reaped, and that is precisely the case
    /// INTERRUPT exists to clean up. Deriving the group from a live child
    /// instead makes the kill unreachable exactly when it is needed.
    ///
    /// It stays valid to signal for as long as it is worth signalling:
    /// Linux keeps a pid number allocated while any process still uses it as
    /// its process-group id, so either this still names *our* group or the
    /// group is empty and the kill is a no-op (`ESRCH`).
    turn_pgid: Option<u32>,
    stopped: bool,
    interrupt_requested: bool,
    reader: Option<std::thread::JoinHandle<()>>,
}

#[derive(Debug, Default)]
struct AdapterState {
    executions: BTreeMap<String, CodexExecution>,
}

/// Whether this daemon has re-adopted the thread being classified — changes
/// nothing about the evidence, only what the human-readable reason says
/// (`claude.rs`'s `Adoption`, codex-local copy of the same idea).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Adoption {
    Unowned,
    Adopted,
}

/// One outcome of spawning turn 1, delivered from the reader thread back to
/// the LAUNCH call blocking on it (spec §3.1).
enum FirstTurnSignal {
    /// `thread.started` arrived; this is its `thread_id`.
    ThreadStarted(String),
    /// The process exited (or was reaped) before `thread.started` ever
    /// arrived — the measured shape of every pre-turn refusal.
    ExitedWithoutThread {
        exit_code: Option<i32>,
        stderr: String,
        raw_blob: Option<String>,
    },
}

// ---------------------------------------------------------------- the backend

/// The Codex backend.
pub struct CodexBackend {
    config: CodexConfig,
    probe_outcome: OnceLock<ProbeOutcome>,
    state: Arc<Mutex<AdapterState>>,
    sink: Mutex<Option<EventSink>>,
}

impl std::fmt::Debug for CodexBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodexBackend")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl CodexBackend {
    /// Build the adapter. Probing is lazy (first PROBE/PREPARE), so
    /// constructing one costs nothing on daemons that never route to it.
    pub fn new(config: CodexConfig) -> Self {
        Self {
            config,
            probe_outcome: OnceLock::new(),
            state: Arc::new(Mutex::new(AdapterState::default())),
            sink: Mutex::new(None),
        }
    }

    /// Install the event sink normalized events are pushed through (§27).
    pub fn set_event_sink(&self, sink: EventSink) {
        *self.sink.lock().expect("codex sink lock") = Some(sink);
    }

    /// Execution ids this adapter currently holds state for — the diagnostic
    /// answer to "did a refused LAUNCH leave a phantom execution behind?".
    pub fn tracked_executions(&self) -> Vec<String> {
        self.lock().executions.keys().cloned().collect()
    }

    /// `$CODEX_HOME`: config override, else the environment variable, else
    /// `~/.codex`.
    fn codex_home(&self) -> PathBuf {
        if let Some(home) = &self.config.codex_home {
            return home.clone();
        }
        if let Ok(dir) = std::env::var("CODEX_HOME") {
            return PathBuf::from(dir);
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        PathBuf::from(home).join(".codex")
    }

    /// A bounded, depth-capped walk of `<codex_home>/sessions/**` for a file
    /// whose name ends with `-<thread_id>.jsonl` (spec §5.3, Appendix A.7).
    /// Only the filename suffix is relied on — never the date partitioning,
    /// never the file's contents — the identical posture and justification as
    /// `claude.rs`'s `<session_id>.jsonl` glob.
    fn thread_rollout(&self, thread_id: &str) -> Option<PathBuf> {
        let root = self.codex_home().join("sessions");
        let suffix = format!("-{thread_id}.jsonl");
        Self::find_rollout(&root, &suffix, 8)
    }

    fn find_rollout(dir: &Path, suffix: &str, depth_remaining: u32) -> Option<PathBuf> {
        let entries = std::fs::read_dir(dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if is_dir {
                if depth_remaining == 0 {
                    continue;
                }
                if let Some(found) = Self::find_rollout(&path, suffix, depth_remaining - 1) {
                    return Some(found);
                }
            } else if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(suffix))
            {
                return Some(path);
            }
        }
        None
    }

    /// Run the capability/version/flag/auth probe once and cache the outcome.
    /// All four gates are offline and token-free (`--version`, both
    /// `--help`s, `login status`); the outcome is cached exactly as
    /// `claude.rs` does.
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
                    provenance: None,
                    auth: None,
                };
            }
        };
        let version_text = String::from_utf8_lossy(&version_out.stdout)
            .trim()
            .to_string();
        let Some(triple) = parse_codex_version(&version_text) else {
            return ProbeOutcome {
                available: false,
                detail: format!(
                    "capability probe: cannot parse a version from {exe:?} --version output \
                     {version_text:?}; refusing an unmeasurable CLI"
                ),
                version: None,
                provenance: None,
                auth: None,
            };
        };
        let canonical = format!("{}.{}.{}", triple.0, triple.1, triple.2);
        let provenance = if triple >= MEASURED_FLOOR {
            VersionProvenance::Measured
        } else {
            VersionProvenance::BelowFloor
        };

        let exec_help = match Command::new(exe).args(["exec", "--help"]).output() {
            Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
            Err(e) => {
                return ProbeOutcome {
                    available: false,
                    detail: format!(
                        "capability probe: cannot run {exe:?} exec --help: {e} (kind: {:?})",
                        e.kind()
                    ),
                    version: Some(canonical),
                    provenance: Some(provenance),
                    auth: None,
                };
            }
        };
        let resume_help = match Command::new(exe)
            .args(["exec", "resume", "--help"])
            .output()
        {
            Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
            Err(e) => {
                return ProbeOutcome {
                    available: false,
                    detail: format!(
                        "capability probe: cannot run {exe:?} exec resume --help: {e} (kind: {:?})",
                        e.kind()
                    ),
                    version: Some(canonical),
                    provenance: Some(provenance),
                    auth: None,
                };
            }
        };

        // §2.5: the primary surface (`exec --help`) is already named by the
        // sentence this builds into, so its own gap is stated bare (just the
        // items) — every other gap names its own surface in an "and" clause.
        // Avoids the doubled "is missing required flag(s): exec --help is
        // missing ..." a naive per-clause template would produce.
        let mut missing_clauses: Vec<String> = Vec::new();
        let missing_exec = missing_flags(&exec_help, REQUIRED_EXEC_FLAGS);
        if !missing_exec.is_empty() {
            let items = missing_exec.join(", ");
            missing_clauses.push(if missing_clauses.is_empty() {
                format!("required flag(s) {items}")
            } else {
                format!("and exec --help is missing required flag(s) {items}")
            });
        }
        let missing_subcommands: Vec<&str> = REQUIRED_EXEC_SUBCOMMANDS
            .iter()
            .copied()
            .filter(|c| !exec_help.contains(c))
            .collect();
        if !missing_subcommands.is_empty() {
            let items = missing_subcommands.join(", ");
            missing_clauses.push(if missing_clauses.is_empty() {
                format!("required subcommand(s) {items}")
            } else {
                format!("and exec --help is missing subcommand(s) {items}")
            });
        }
        let missing_resume = missing_flags(&resume_help, REQUIRED_RESUME_FLAGS);
        if !missing_resume.is_empty() {
            let items = missing_resume.join(", ");
            missing_clauses.push(if missing_clauses.is_empty() {
                items
            } else {
                format!("and exec resume --help is missing {items}")
            });
        }
        if !missing_clauses.is_empty() {
            return ProbeOutcome {
                available: false,
                detail: format!(
                    "capability probe: {exe:?} exec --help (version {version_text}) is missing \
                     {}; this launch grammar was never measured against it",
                    missing_clauses.join("; ")
                ),
                version: Some(canonical),
                provenance: Some(provenance),
                auth: None,
            };
        }

        // Gate 4 (§2.6): reported, never gates.
        let auth = self.run_auth_probe();

        let version_clause = match provenance {
            VersionProvenance::Measured => format!("codex-cli {canonical}"),
            VersionProvenance::BelowFloor => format!(
                "codex-cli {canonical}; usable, but BELOW the measured floor {}.{}.{} — every \
                 behavioural claim in this adapter (event grammar, resume, usage block, terminal \
                 shapes) was measured on {}.{}.{} and has not been re-measured here. Capabilities \
                 carry unmeasured provenance; upgrade to >= {}.{}.{} or expect surprises to be \
                 findings, not failures.",
                MEASURED_FLOOR.0,
                MEASURED_FLOOR.1,
                MEASURED_FLOOR.2,
                MEASURED_FLOOR.0,
                MEASURED_FLOOR.1,
                MEASURED_FLOOR.2,
                MEASURED_FLOOR.0,
                MEASURED_FLOOR.1,
                MEASURED_FLOOR.2,
            ),
        };
        let flags_clause = format!(
            "all {} required exec flags and {} required resume flags present",
            REQUIRED_EXEC_FLAGS.len(),
            REQUIRED_RESUME_FLAGS.len()
        );
        let auth_clause = match &auth {
            AuthState::LoggedIn { method } => format!("logged in using {method}"),
            AuthState::Unreported(line) if line.is_empty() => "unreported".to_string(),
            AuthState::Unreported(line) => format!("unreported ({line})"),
        };
        let mut detail = format!("{version_clause}; {flags_clause}; auth: {auth_clause}");
        if self.config.codex_home.is_none() {
            detail.push_str(&format!(
                "; note: turns write durable trust entries into {}/config.toml, one per work surface",
                self.codex_home().display()
            ));
        }

        ProbeOutcome {
            available: true,
            detail,
            version: Some(canonical),
            provenance: Some(provenance),
            auth: Some(auth),
        }
    }

    fn run_auth_probe(&self) -> AuthState {
        match Command::new(&self.config.executable)
            .args(["login", "status"])
            .output()
        {
            // Re-measured while wiring the live suite (this is *not* what an
            // interactive shell shows): `codex login status` writes its
            // answer to **stderr**, not stdout, when it is not attached to a
            // TTY [measured — a spawned child never has one]. Stdout is
            // checked first for forward compatibility with a build that
            // moves the line, but on 0.149.0 it is stderr that carries it.
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let text = if stdout.trim().is_empty() {
                    String::from_utf8_lossy(&out.stderr).into_owned()
                } else {
                    stdout.into_owned()
                };
                parse_auth_status(&text)
            }
            Err(e) => AuthState::Unreported(format!("cannot run login status: {e}")),
        }
    }

    /// Resolve one execution's launch configuration from adapter config plus
    /// the profile (§14, §3.5). One function, used by PREPARE, LAUNCH and
    /// RESUME alike — the same rule `claude.rs::launch_config` follows.
    ///
    /// The `codex_profile` option is refused here, unconditionally: `exec
    /// resume` has no `-p/--profile` on this build [measured-negative], so a
    /// profile layer applied on turn 1 would silently lapse on every later
    /// turn — a launch decision the human made, quietly dropped by the
    /// adapter. `config_home` (`CODEX_HOME`) is the axis that *does*
    /// re-apply on every turn, and the refusal names it.
    fn launch_config(&self, profile: Option<&Profile>) -> Result<LaunchConfig, BackendError> {
        if let Some(profile) = profile
            && profile.options.contains_key("codex_profile")
        {
            return Err(self.err_failed(format!(
                "profile {:?}: option codex_profile is not supported by this adapter. \
                 `codex exec resume` has no -p/--profile flag on codex-cli 0.149.0 \
                 (measured), so a profile layer applied to the first turn silently lapses \
                 on every later turn of the same conversation — the adapter would be \
                 dropping a launch decision the human made. Use the profile's `config_home` \
                 instead: it sets CODEX_HOME, which every turn re-reads.",
                profile.name
            )));
        }
        let executable = profile
            .and_then(|p| p.executable.clone())
            .unwrap_or_else(|| self.config.executable.clone());
        let mut env = self.config.env.clone();
        let mut codex_home = self.config.codex_home.clone();
        if let Some(profile) = profile {
            for (key, value) in &profile.env {
                env.insert(key.clone(), value.clone());
            }
            if let Some(config_home) = &profile.config_home {
                codex_home = Some(config_home.clone());
            }
        }
        Ok(LaunchConfig {
            executable,
            env,
            codex_home,
        })
    }

    fn err_failed(&self, detail: impl Into<String>) -> BackendError {
        BackendError::Failed {
            backend: CODEX_BACKEND_NAME.to_string(),
            detail: detail.into(),
        }
    }

    fn err_unknown(&self, execution_id: &str) -> BackendError {
        BackendError::UnknownExecution {
            backend: CODEX_BACKEND_NAME.to_string(),
            execution_id: execution_id.to_string(),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, AdapterState> {
        self.state.lock().expect("codex adapter state lock")
    }

    /// §25's identity rule: an execution is resolved by sergeant's id *and*
    /// the native (thread) identity the handle carries.
    fn check_identity(
        &self,
        state: &AdapterState,
        handle: &ExecutionHandle,
    ) -> Result<(), BackendError> {
        let execution = state
            .executions
            .get(&handle.execution_id)
            .ok_or_else(|| self.err_unknown(&handle.execution_id))?;
        if handle.native_id.as_deref() != execution.thread_id.as_deref() {
            return Err(self.err_unknown(&handle.execution_id));
        }
        Ok(())
    }

    fn emit(&self, execution_id: &str, work_id: &str, kind: &str, payload: Value) {
        let sink = self.sink.lock().expect("codex sink lock").clone();
        if let Some(sink) = sink {
            sink(EventDraft {
                source: EventSource::new("backend", CODEX_BACKEND_NAME),
                workspace_id: None,
                work_id: Some(work_id.to_string()),
                execution_id: Some(execution_id.to_string()),
                correlation_id: Some(execution_id.to_string()),
                causation_id: None,
                kind: kind.to_string(),
                payload,
            });
        }
    }

    fn turn_liveness(&self, thread_id: &str, cwd: Option<&Path>) -> Liveness {
        turn_liveness_among(
            thread_id,
            cwd,
            std::process::id(),
            crate::platform::process::running_processes(),
        )
    }

    /// Classify a thread this daemon did not run a turn on, from liveness and
    /// rollout existence (spec §5.3). `None` means there is no durable thread
    /// to classify — the caller turns that into `UnknownExecution`.
    fn classify_restart(
        &self,
        thread_id: &str,
        cwd: Option<&Path>,
        adoption: Adoption,
    ) -> Option<Observation> {
        let liveness = self.turn_liveness(thread_id, cwd);
        let rollout = self.thread_rollout(thread_id);
        match (liveness, rollout) {
            (
                Liveness::Alive {
                    pid,
                    attribution: Attribution::ThreadId,
                },
                _,
            ) => Some(Observation {
                native: NativeState::Running,
                signal: BackendSignal::Blocked {
                    reason: format!(
                        "daemon restarted while a turn of thread {thread_id} was still running \
                         (pid {pid}); that turn is unowned — its output is going nowhere and \
                         sergeant did not adopt it"
                    ),
                },
                evidence: Some(format!(
                    "live turn: pid {pid} runs exec resume {thread_id} in its argv"
                )),
            }),
            (
                Liveness::Alive {
                    pid,
                    attribution: Attribution::SurfaceAmbiguous,
                },
                _,
            ) => Some(Observation {
                native: NativeState::Unknown,
                signal: BackendSignal::Blocked {
                    reason: format!(
                        "a `codex exec` process (pid {pid}) is running against this work's \
                         surface; whether it is a turn of thread {thread_id} cannot be \
                         established from its argv — the first turn of a codex conversation does \
                         not carry the thread id"
                    ),
                },
                evidence: Some(format!(
                    "live process: pid {pid} runs exec against this surface; no thread id in its argv"
                )),
            }),
            (Liveness::Dead, Some(path)) => Some(Observation {
                native: NativeState::Exited,
                signal: BackendSignal::Blocked {
                    reason: match adoption {
                        Adoption::Unowned => format!(
                            "daemon restarted mid-execution; thread {thread_id} is resumable \
                             (durable rollout present) but the in-flight turn's outcome is unknown"
                        ),
                        Adoption::Adopted => format!(
                            "thread {thread_id} was re-adopted after a daemon restart and is \
                             resumable (durable rollout present, no turn running), but the turn \
                             that was in flight when the daemon died left no outcome this daemon \
                             can read — the stage's result is unknown, not absent"
                        ),
                    },
                },
                evidence: Some(format!(
                    "no live process carries thread {thread_id}; rollout: {}; adopted={}",
                    path.display(),
                    adoption == Adoption::Adopted
                )),
            }),
            (Liveness::Unknowable(why), Some(path)) => Some(Observation {
                native: NativeState::Unknown,
                signal: BackendSignal::Blocked {
                    reason: format!(
                        "daemon restarted mid-execution; thread {thread_id} is resumable \
                         (durable rollout present) but whether its turn process is still \
                         running cannot be evidenced here"
                    ),
                },
                evidence: Some(format!(
                    "rollout: {}; process liveness unknowable: {why}",
                    path.display()
                )),
            }),
            (_, None) => None,
        }
    }

    /// Spawn one turn for an execution already registered in adapter state.
    /// `first_turn_signal`, when present, is used only for turn 1's
    /// synchronization with LAUNCH (§3.1) — SEND passes `None`.
    fn spawn_turn(
        &self,
        execution_id: &str,
        prompt: String,
        first_turn_signal: Option<SyncSender<FirstTurnSignal>>,
    ) -> Result<(), BackendError> {
        let (
            executable,
            cwd,
            env,
            codex_home,
            thread_id,
            model,
            first_turn,
            work_id,
            bindings_outside_cwd,
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
                execution.codex_home.clone(),
                execution.thread_id.clone(),
                execution.model.clone(),
                execution.turns == 0,
                execution.work_id.clone(),
                execution.bindings_outside_cwd.clone(),
            )
        };

        let mut command = Command::new(&executable);
        if first_turn {
            command.args(first_turn_argv(&cwd, model.as_deref()));
        } else {
            let thread_id = thread_id.clone().ok_or_else(|| {
                self.err_failed("cannot send: no thread id recorded for this execution")
            })?;
            command.args(resume_turn_argv(&thread_id, model.as_deref()));
        }
        command
            .current_dir(&cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (key, value) in &env {
            command.env(key, value);
        }
        if let Some(home) = &codex_home {
            command.env("CODEX_HOME", home);
        }
        // §5.5: every turn's shell commands run as children of this process;
        // a new process group is what lets INTERRUPT kill the whole tree.
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }

        let mut child = command
            .spawn()
            .map_err(|e| self.err_failed(format!("cannot spawn {executable:?}: {e}")))?;

        let stdin = child.stdin.take();
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| self.err_failed("child stdout was not piped"))?;
        let stderr = child.stderr.take();

        // Prompt on its own thread: a CONTEXT.md larger than the pipe buffer
        // must not deadlock the spawn path (§3.4).
        let stdin_prompt = prompt.clone();
        std::thread::spawn(move || {
            if let Some(mut stdin) = stdin {
                let _ = stdin.write_all(stdin_prompt.as_bytes());
            }
        });
        let stderr_rx = stderr.map(|mut stderr| {
            let (tx, rx) = std::sync::mpsc::sync_channel::<String>(1);
            std::thread::spawn(move || {
                let mut text = String::new();
                let _ = stderr.read_to_string(&mut text);
                let _ = tx.send(text);
            });
            rx
        });

        // §5.5: recorded here, at spawn, never derived from the child at kill
        // time — `process_group(0)` above made this child its own group
        // leader, so its pid *is* the group's id, and that id stays the right
        // thing to signal after the child itself has exited.
        let turn_pgid = child.id();
        let child = Arc::new(Mutex::new(child));
        {
            let mut state = self.lock();
            let execution = state
                .executions
                .get_mut(execution_id)
                .ok_or_else(|| self.err_unknown(execution_id))?;
            execution.turn = TurnState::InFlight(Arc::clone(&child));
            execution.turn_pgid = Some(turn_pgid);
            execution.turns += 1;
            execution.interrupt_requested = false;
        }
        self.emit(
            execution_id,
            &work_id,
            KIND_CONVERSATION_USER,
            json!({
                "text": prompt,
                "thread_id": thread_id,
                "bindings_outside_cwd": bindings_outside_cwd,
            }),
        );

        let reader = TurnReader {
            backend_state: Arc::clone(&self.state),
            sink: self.sink.lock().expect("codex sink lock").clone(),
            data_dir: self.config.data_dir.clone(),
            execution_id: execution_id.to_string(),
            work_id,
            expected_thread_id: if first_turn { None } else { thread_id },
            model,
            bindings_outside_cwd,
            child: Arc::clone(&child),
            stderr_rx,
            first_turn_signal,
        };
        let reader_handle = std::thread::spawn(move || reader.run(stdout));
        if let Some(execution) = self.lock().executions.get_mut(execution_id) {
            execution.reader = Some(reader_handle);
        }
        Ok(())
    }

    /// LAUNCH's own spawn: fires turn 1 and blocks, bounded, for
    /// `thread.started` (§3.1). Does *not* remove adapter state on failure —
    /// the caller (`launch`) owns that, so there is one place a failed launch
    /// leaves no phantom.
    fn spawn_first_turn(&self, execution_id: &str, prompt: String) -> Result<String, BackendError> {
        let (tx, rx) = std::sync::mpsc::sync_channel::<FirstTurnSignal>(1);
        self.spawn_turn(execution_id, prompt, Some(tx))?;
        let budget = self.config.thread_id_budget.unwrap_or(THREAD_ID_BUDGET);
        match rx.recv_timeout(budget) {
            Ok(FirstTurnSignal::ThreadStarted(thread_id)) => Ok(thread_id),
            Ok(FirstTurnSignal::ExitedWithoutThread {
                exit_code,
                stderr,
                raw_blob,
            }) => Err(self.err_failed(format!(
                "codex exec exited before thread.started arrived (exit_code={exit_code:?}); \
                 stderr: {}; raw={}",
                truncate(&stderr, 400),
                raw_blob.unwrap_or_else(|| "unarchived (the turn streamed nothing)".to_string()),
            ))),
            Err(_) => {
                let (pgid, child) = self.inflight_turn(execution_id);
                kill_turn(pgid, child.as_ref());
                Err(self.err_failed(format!(
                    "codex exec did not announce thread.started within {budget:?}; the process \
                     group was killed"
                )))
            }
        }
    }

    /// This execution's turn process group (recorded at spawn, present
    /// whether or not the turn is still running) and its direct child (only
    /// while one is in flight). The two are returned separately on purpose:
    /// see [`kill_process_group`] for why the group must not be reached
    /// through the child.
    fn inflight_turn(&self, execution_id: &str) -> (Option<u32>, Option<Arc<Mutex<Child>>>) {
        let state = self.lock();
        let Some(execution) = state.executions.get(execution_id) else {
            return (None, None);
        };
        let child = match &execution.turn {
            TurnState::InFlight(child) => Some(Arc::clone(child)),
            _ => None,
        };
        (execution.turn_pgid, child)
    }
}

/// Kill a turn's whole process group (§5.5): `SIGKILL` to the negated group
/// id recorded at spawn, through a shell rather than a `libc`/`nix`
/// dependency for one signal — the reason `tests/support/mod.rs` gives (R5).
///
/// Through `/bin/sh -c` specifically, and not by spawning `kill` as a program.
/// `kill` is a **shell builtin** that every POSIX shell is required to have,
/// while `kill(1)` as an executable on `PATH` is a package that a host need
/// not install — and `Command::new("kill")` fails with `ENOENT` on such a
/// host, which is a silent no-op when the caller drops the result. That is
/// not hypothetical: it is measured, and it is why this call reports a spawn
/// failure instead of discarding it. `/bin/sh` is an absolute path for the
/// same reason, so a `PATH` this process never chose cannot decide whether
/// INTERRUPT works.
///
/// **Nothing gates this on the leader being alive**, and that is the whole
/// point. The group is what INTERRUPT promises to kill, and the group
/// routinely outlives its leader: a command the turn started in the
/// background survives the codex process, and once that process has exited
/// and the reader has reaped it, every liveness test one could run — the
/// turn's `TurnState`, `try_wait`, the child handle at all — says "nothing to
/// kill" about a group that is still very much running. So the group id is
/// signalled unconditionally and `ESRCH` (an already-empty group) is success,
/// not an error to report: the shell's *exit status* is deliberately not
/// consulted, because "no such group" and "killed the group" are the same
/// outcome here. Failing to run the kill at all is not — that one is logged.
///
/// Signalling a recorded id after its leader is gone is safe as well as
/// necessary: Linux keeps a pid number allocated for as long as any process
/// still uses it as a process-group id, so while there is anything in this
/// group to kill, this id cannot have come to mean another one.
fn kill_process_group(pgid: Option<u32>) {
    let Some(pgid) = pgid else { return };
    #[cfg(unix)]
    {
        if let Err(e) = Command::new("/bin/sh")
            .arg("-c")
            .arg(format!("kill -KILL -{pgid}"))
            .output()
        {
            tracing::warn!(
                pgid,
                error = %e,
                "could not run the process-group kill; the turn's direct child is all that \
                 INTERRUPT reached — any commands it spawned may still be running"
            );
        }
    }
    #[cfg(not(unix))]
    {
        tracing::warn!(
            pgid,
            "no process-group signal mechanism on this platform; killing only the direct \
             child — any commands it spawned may still be running"
        );
    }
}

/// The group kill above plus `Child::kill()` on the direct child as a belt,
/// for the callers that still hold a live child handle. The group goes first:
/// the child's own death must never be what decides whether the group is
/// signalled.
fn kill_turn(pgid: Option<u32>, child: Option<&Arc<Mutex<Child>>>) {
    kill_process_group(pgid);
    if let Some(child) = child {
        let _ = child.lock().expect("codex turn child lock").kill();
    }
}

/// Everything the per-turn stdout reader thread needs. Owns ingestion end to
/// end: raw archive, normalization, outcome recording — `claude.rs`'s
/// `TurnReader`, codex-shaped.
struct TurnReader {
    backend_state: Arc<Mutex<AdapterState>>,
    sink: Option<EventSink>,
    data_dir: PathBuf,
    execution_id: String,
    work_id: String,
    /// `Some(thread_id)` on a resume turn — the identity §3.7 checks this
    /// turn's own `thread.started` against; `None` on turn 1, which has
    /// nothing to compare against yet.
    expected_thread_id: Option<String>,
    model: Option<String>,
    /// §3.3's evidence, carried into every `conversation.turn.ended`.
    bindings_outside_cwd: Vec<PathBuf>,
    child: Arc<Mutex<Child>>,
    stderr_rx: Option<std::sync::mpsc::Receiver<String>>,
    /// Present only for turn 1: how this reader tells LAUNCH the thread id
    /// (or that the process died first).
    first_turn_signal: Option<SyncSender<FirstTurnSignal>>,
}

impl TurnReader {
    fn run(self, stdout: std::process::ChildStdout) {
        let mut raw = String::new();
        let mut acc = TurnAccumulator::new();
        let mut sent_thread_started = false;

        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
            raw.push_str(&line);
            raw.push('\n');
            match serde_json::from_str::<Value>(&line) {
                Ok(value) => {
                    for event in acc.ingest_line(&value) {
                        self.emit(&event.kind, event.payload);
                    }
                    if !sent_thread_started && let Some(id) = acc.thread_id.clone() {
                        sent_thread_started = true;
                        if let Some(execution) = self
                            .backend_state
                            .lock()
                            .expect("codex adapter state lock")
                            .executions
                            .get_mut(&self.execution_id)
                        {
                            execution.thread_id = Some(id.clone());
                        }
                        if let Some(tx) = &self.first_turn_signal {
                            let _ = tx.send(FirstTurnSignal::ThreadStarted(id));
                        }
                    }
                }
                Err(_) => acc.unparsed_lines += 1,
            }
        }

        // Stdout is closed; reap. The lock is only taken after EOF so
        // `interrupt` can always kill.
        let exit_status = self
            .child
            .lock()
            .expect("codex turn child lock")
            .wait()
            .ok();
        let exit_code = exit_status.and_then(|status| status.code());

        // §20: archived before any conclusion is drawn from it.
        let (raw_blob, raw_error) = if raw.is_empty() {
            (None, None)
        } else {
            match BlobStore::open(&self.data_dir).and_then(|store| store.put(raw.as_bytes())) {
                Ok(blob_ref) => (Some(blob_ref.to_string()), None),
                Err(e) => (None, Some(e.to_string())),
            }
        };

        let stderr = self
            .stderr_rx
            .as_ref()
            .and_then(|rx| rx.recv_timeout(STDERR_DRAIN_BUDGET).ok())
            .unwrap_or_default();

        if !sent_thread_started && let Some(tx) = &self.first_turn_signal {
            let _ = tx.send(FirstTurnSignal::ExitedWithoutThread {
                exit_code,
                stderr: stderr.clone(),
                raw_blob: raw_blob.clone(),
            });
        }

        let mut state = self.backend_state.lock().expect("codex adapter state lock");
        let Some(execution) = state.executions.get_mut(&self.execution_id) else {
            return;
        };
        let interrupted = execution.interrupt_requested;
        // §3.7: checked from this turn's own observed thread id, independent
        // of whatever the terminal shape says.
        let pin_mismatch = self.expected_thread_id.as_deref().and_then(|expected| {
            acc.thread_id
                .as_deref()
                .and_then(|seen| thread_pin_mismatch(expected, seen))
        });
        let terminal = classify_terminal(&acc, interrupted);
        let thread_id_for_event = execution.thread_id.clone();
        let outcome = TurnOutcome {
            terminal,
            pin_mismatch,
            message_items: acc.message_items,
            tool_items: acc.tool_items,
            unknown_items: acc.unknown_items.clone(),
            unparsed_lines: acc.unparsed_lines,
            last_agent_message: acc.last_agent_message.clone(),
            last_error: acc.last_error.clone(),
            exit_code,
            raw_blob: raw_blob.clone(),
            raw_error: raw_error.clone(),
            stderr: stderr.clone(),
        };
        execution.turn = TurnState::Finished(Box::new(outcome));
        drop(state);

        // Every turn ends with this event, however it ended — the only place
        // the §20 blob ref reaches the journal for a turn with no terminal.
        self.emit(
            KIND_CONVERSATION_TURN_ENDED,
            json!({
                "thread_id": thread_id_for_event,
                "interrupted": interrupted,
                "message_items": acc.message_items,
                "tool_items": acc.tool_items,
                "unknown_items": acc.unknown_items,
                "unparsed_lines": acc.unparsed_lines,
                "bindings_outside_cwd": self.bindings_outside_cwd,
                "raw": raw_blob,
                "raw_error": raw_error,
                "stderr": truncate(&stderr, 400),
            }),
        );

        if let Some(usage) = &acc.usage {
            self.emit(
                KIND_USAGE_UPDATED,
                json!({
                    "thread_id": thread_id_for_event,
                    "usage": usage,
                    "model_pin": model_pin_evidence(self.model.as_deref()),
                    "raw": raw_blob,
                    "raw_error": raw_error,
                }),
            );
        }
    }

    fn emit(&self, kind: &str, payload: Value) {
        if let Some(sink) = &self.sink {
            sink(EventDraft {
                source: EventSource::new("backend", CODEX_BACKEND_NAME),
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

impl Backend for CodexBackend {
    fn name(&self) -> &str {
        CODEX_BACKEND_NAME
    }

    /// Capabilities as measured on 0.149.0 (spec §6). Every `true` names a
    /// contract test against the installed harness (L8); every `false` names
    /// its structural reason in the module docs. Unlike Claude's `ask`, every
    /// row here is a `const`-shaped fact, not a runtime-varying one: nothing
    /// in this row rests on evidence a probe cannot see.
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
            ask: false,
        }
    }

    /// §17: each turn is its own short-lived process; there is no
    /// backend-level service to start or attach to.
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

    /// PREPARE (§3.1): refuse an unavailable probe or an impossible pin;
    /// resolve and validate the launch configuration so an impossible
    /// profile never reaches a spawn; reserve **no** native id — it cannot
    /// be pre-minted on this transport, and `PreparedExecution::native_id:
    /// None` is exactly the honest answer its own contract blesses.
    fn prepare(&self, request: &StartRequest) -> Result<PreparedExecution, BackendError> {
        let probe = self.probe_outcome();
        if !probe.available {
            return Err(BackendError::Unavailable {
                backend: CODEX_BACKEND_NAME.to_string(),
                detail: probe.detail.clone(),
            });
        }
        if let Some(model) = &request.model {
            preflight_model_pin(model).map_err(|reason| self.err_failed(reason))?;
        }
        // Validates (in particular, refuses a codex-native profile layer)
        // without keeping the result: LAUNCH re-resolves it, so the two
        // phases can never disagree about it.
        self.launch_config(request.profile.as_ref())?;
        Ok(PreparedExecution {
            execution_id: request.execution_id.clone(),
            native_id: None,
            request: request.clone(),
        })
    }

    /// LAUNCH (§3.1): register the execution, spawn turn 1, and wait bounded
    /// for `thread.started` before returning a handle at all. A failed
    /// launch leaves no phantom: adapter state is removed on every error
    /// path.
    fn launch(&self, prepared: &PreparedExecution) -> Result<ExecutionHandle, BackendError> {
        let request = &prepared.request;
        let LaunchConfig {
            executable,
            env,
            codex_home,
        } = self.launch_config(request.profile.as_ref())?;
        {
            let mut state = self.lock();
            state.executions.insert(
                request.execution_id.clone(),
                CodexExecution {
                    thread_id: None,
                    work_id: request.work_id.clone(),
                    cwd: request.cwd.clone(),
                    model: request.model.clone(),
                    executable,
                    env,
                    codex_home,
                    bindings_outside_cwd: bindings_outside_cwd(&request.cwd, &request.bindings),
                    turns: 0,
                    turn: TurnState::Unlaunched,
                    turn_pgid: None,
                    stopped: false,
                    interrupt_requested: false,
                    reader: None,
                },
            );
        }
        match self.spawn_first_turn(&request.execution_id, compose_launch_prompt(request)) {
            Ok(thread_id) => Ok(ExecutionHandle {
                execution_id: request.execution_id.clone(),
                native_id: Some(thread_id),
            }),
            Err(e) => {
                self.lock().executions.remove(&request.execution_id);
                Err(e)
            }
        }
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
                    "execution {} already has a turn in flight; a codex exec conversation runs \
                     one turn at a time",
                    handle.execution_id
                )));
            }
        }
        self.spawn_turn(&handle.execution_id, input.to_string(), None)
    }

    fn observe(&self, handle: &ExecutionHandle) -> Result<Observation, BackendError> {
        let state = self.lock();
        if state.executions.contains_key(&handle.execution_id) {
            self.check_identity(&state, handle)?;
            let execution = &state.executions[&handle.execution_id];
            if matches!(execution.turn, TurnState::Adopted) {
                let thread_id = execution.thread_id.clone().unwrap_or_default();
                let cwd = execution.cwd.clone();
                drop(state);
                return self
                    .classify_restart(&thread_id, Some(&cwd), Adoption::Adopted)
                    .ok_or_else(|| self.err_unknown(&handle.execution_id));
            }
            return Ok(observe_in_memory(execution));
        }
        drop(state);
        let thread_id = handle
            .native_id
            .as_deref()
            .ok_or_else(|| self.err_unknown(&handle.execution_id))?;
        self.classify_restart(thread_id, None, Adoption::Unowned)
            .ok_or_else(|| self.err_unknown(&handle.execution_id))
    }

    /// §5.5: kill the turn's whole process group. The durable thread
    /// survives; the turn's evidence is STOP's promise, not this one.
    fn interrupt(&self, handle: &ExecutionHandle) -> Result<Completion, BackendError> {
        let (pgid, child) = {
            let mut state = self.lock();
            self.check_identity(&state, handle)?;
            let execution = state
                .executions
                .get_mut(&handle.execution_id)
                .expect("presence checked above");
            // The group id is taken whatever the turn state says. A turn that
            // has already ended can still have left a background command
            // running in its group — the exact thing §5.5 kills — so the
            // group kill is never gated on the direct child being alive.
            // Only the `interrupt_requested` bit, which is a claim about a
            // *running* turn's outcome, is still the in-flight turn's alone.
            let pgid = execution.turn_pgid;
            let child = match &execution.turn {
                TurnState::InFlight(child) => {
                    execution.interrupt_requested = true;
                    Some(Arc::clone(child))
                }
                TurnState::Finished(_) | TurnState::Unlaunched | TurnState::Adopted => None,
            };
            (pgid, child)
        };
        kill_turn(pgid, child.as_ref());
        Ok(Completion::immediate())
    }

    /// RESUME (§5.6): mirrors `claude.rs::resume`'s shape with codex's own
    /// evidence — liveness plus rollout existence, never the durable
    /// transcript's contents.
    fn resume(
        &self,
        handle: &ExecutionHandle,
        request: &ResumeRequest,
    ) -> Result<(), BackendError> {
        let thread_id = handle
            .native_id
            .clone()
            .ok_or_else(|| self.err_unknown(&handle.execution_id))?;
        if let Some(model) = &request.model {
            preflight_model_pin(model).map_err(|reason| self.err_failed(reason))?;
        }
        {
            let state = self.lock();
            if let Some(existing) = state.executions.get(&handle.execution_id) {
                if existing.thread_id.as_deref() != Some(thread_id.as_str()) {
                    return Err(self.err_unknown(&handle.execution_id));
                }
                return Ok(());
            }
        }
        if self.thread_rollout(&thread_id).is_none() {
            return Err(self.err_unknown(&handle.execution_id));
        }
        match self.turn_liveness(&thread_id, Some(&request.cwd)) {
            Liveness::Dead => {}
            Liveness::Alive {
                pid,
                attribution: Attribution::ThreadId,
            } => {
                return Err(self.err_failed(format!(
                    "cannot re-adopt thread {thread_id}: a turn of it is still running (pid \
                     {pid}) and this adapter does not own that process"
                )));
            }
            Liveness::Alive {
                pid,
                attribution: Attribution::SurfaceAmbiguous,
            } => {
                return Err(self.err_failed(format!(
                    "cannot re-adopt thread {thread_id}: a codex exec process (pid {pid}) is \
                     running against this surface and its argv cannot say whether it is this \
                     thread's turn"
                )));
            }
            Liveness::Unknowable(why) => {
                return Err(self.err_failed(format!(
                    "cannot re-adopt thread {thread_id}: whether a turn of it is still running \
                     cannot be evidenced here ({why})"
                )));
            }
        }
        let LaunchConfig {
            executable,
            env,
            codex_home,
        } = self.launch_config(request.profile.as_ref())?;
        let mut state = self.lock();
        if let Some(existing) = state.executions.get(&handle.execution_id) {
            if existing.thread_id.as_deref() != Some(thread_id.as_str()) {
                return Err(self.err_unknown(&handle.execution_id));
            }
            return Ok(());
        }
        state.executions.insert(
            handle.execution_id.clone(),
            CodexExecution {
                thread_id: Some(thread_id),
                work_id: request.work_id.clone(),
                cwd: request.cwd.clone(),
                model: request.model.clone(),
                executable,
                env,
                codex_home,
                bindings_outside_cwd: bindings_outside_cwd(&request.cwd, &request.bindings),
                turns: 1,
                turn: TurnState::Adopted,
                // A re-adopted thread's turn was spawned by a previous
                // daemon: this one never learned that group, and inventing
                // one would aim a SIGKILL at a pid it cannot account for.
                turn_pgid: None,
                stopped: false,
                interrupt_requested: false,
                reader: None,
            },
        );
        Ok(())
    }

    /// HISTORY is unsupported (§6): the only complete native record is the
    /// rollout jsonl and `thread_history_1.sqlite`, an unmeasured on-disk
    /// format this milestone never reads for content. The refusal names
    /// where the record actually is.
    fn history(&self, handle: &ExecutionHandle) -> Result<Vec<NativeEvent>, BackendError> {
        let state = self.lock();
        self.check_identity(&state, handle)?;
        drop(state);
        Err(BackendError::Unsupported {
            backend: CODEX_BACKEND_NAME.to_string(),
            verb: "history".to_string(),
            detail: "this adapter cannot retrieve durable native history: the only complete \
                     native record is the rollout jsonl and thread_history_1.sqlite, an \
                     unmeasured on-disk format, and reporting only the events this process \
                     happened to ingest would be a partial answer indistinguishable from a \
                     complete one (empty, in particular, after a restart). The normalized \
                     events are journaled through the event sink (§27); the native record is \
                     codex's own rollout file under <codex_home>/sessions"
                .to_string(),
        })
    }

    /// STOP (§5.7): kill any in-flight turn, refuse further input, hand back
    /// the reader's join as the completion's tail (issue #14/B3's rule).
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

/// Map an in-memory execution's turn state to an Observation (spec §5.1).
fn observe_in_memory(execution: &CodexExecution) -> Observation {
    let thread_ref = execution.thread_id.as_deref().unwrap_or("<unminted>");
    match &execution.turn {
        TurnState::Unlaunched => Observation {
            native: NativeState::Unknown,
            signal: BackendSignal::Running,
            evidence: Some(format!(
                "execution registered for thread {thread_ref} but no turn was ever launched"
            )),
        },
        TurnState::Adopted => Observation {
            native: NativeState::Unknown,
            signal: BackendSignal::Running,
            evidence: Some(format!(
                "thread {thread_ref} was re-adopted after a restart; no turn of this daemon's \
                 has run on it"
            )),
        },
        TurnState::InFlight(_) => Observation {
            native: NativeState::Running,
            signal: BackendSignal::Running,
            evidence: Some(format!(
                "turn {} in flight on thread {thread_ref}",
                execution.turns
            )),
        },
        TurnState::Finished(outcome) => {
            // §3.7: checked before the completion branch, whatever the turn
            // otherwise produced.
            if let Some(mismatch) = &outcome.pin_mismatch {
                return Observation {
                    native: NativeState::Exited,
                    signal: BackendSignal::Failed {
                        reason: mismatch.clone(),
                    },
                    evidence: Some(format!("raw={}", outcome.raw_evidence())),
                };
            }
            match &outcome.terminal {
                TerminalOutcome::Completed => Observation {
                    native: NativeState::Exited,
                    signal: BackendSignal::StageCompleted {
                        summary: outcome.last_agent_message.clone(),
                    },
                    evidence: Some(format!(
                        "thread_id={thread_ref}; model_pin={}; raw={}; message_items={}, \
                         tool_items={}, unknown_items={:?}, unparsed_lines={}",
                        model_pin_evidence(execution.model.as_deref()),
                        outcome.raw_evidence(),
                        outcome.message_items,
                        outcome.tool_items,
                        outcome.unknown_items,
                        outcome.unparsed_lines,
                    )),
                },
                TerminalOutcome::Failed { message } => Observation {
                    native: NativeState::Exited,
                    signal: BackendSignal::Failed {
                        reason: format!("turn failed: {}", truncate(message, 400)),
                    },
                    evidence: Some(format!(
                        "thread_id={thread_ref}; raw={}",
                        outcome.raw_evidence()
                    )),
                },
                TerminalOutcome::InterruptedRunning => Observation {
                    native: NativeState::Exited,
                    signal: BackendSignal::Running,
                    evidence: Some(format!(
                        "turn interrupted by request; conversation {thread_ref} resumable; raw={}",
                        outcome.raw_evidence()
                    )),
                },
                TerminalOutcome::AmbiguousUnknown => Observation {
                    native: NativeState::Unknown,
                    signal: BackendSignal::Running,
                    evidence: Some(format!(
                        "turn process exited without a turn.completed/turn.failed (thread \
                         {thread_ref}); exit_code={:?}; last_error={:?}; raw={}; stderr: {}",
                        outcome.exit_code,
                        outcome.last_error,
                        outcome.raw_evidence(),
                        truncate(&outcome.stderr, 400)
                    )),
                },
            }
        }
    }
}

/// A private per-module copy — `claude.rs`'s own precedent (spec §1.2: each
/// owner keeps its own; `runtime/graph.rs` already has a second copy too).
fn truncate(text: &str, max: usize) -> &str {
    match text.char_indices().nth(max) {
        Some((idx, _)) => &text[..idx],
        None => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::claude::ENVIRONMENT_CONTRACT as CLAUDE_ENVIRONMENT_CONTRACT;
    use crate::runtime::graph::KIND_CONVERSATION_ASK;

    // ---------------------------------------------------------- §2.3 version

    #[test]
    fn parse_codex_version_pins_every_table_row() {
        assert_eq!(parse_codex_version("codex-cli 0.149.0"), Some((0, 149, 0)));
        assert_eq!(
            parse_codex_version("codex-cli 0.150.3\n"),
            Some((0, 150, 3))
        );
        assert_eq!(parse_codex_version("0.149.0"), Some((0, 149, 0)));
        assert_eq!(
            parse_codex_version("codex-cli 0.149.0-rc.1"),
            Some((0, 149, 0)),
            "patch parsed up to the first non-digit"
        );
        assert_eq!(parse_codex_version("codex-cli nightly"), None);
        assert_eq!(
            parse_codex_version("0.149"),
            None,
            "two segments are not a version"
        );
        assert_eq!(parse_codex_version(""), None);
    }

    // ----------------------------------------------------------- §2.6 auth

    #[test]
    fn parse_auth_status_recognizes_only_the_measured_shape() {
        assert_eq!(
            parse_auth_status("Logged in using ChatGPT\n"),
            AuthState::LoggedIn {
                method: "ChatGPT".to_string()
            }
        );
        match parse_auth_status("Not logged in\n") {
            AuthState::Unreported(line) => assert_eq!(line, "Not logged in"),
            other => panic!("expected Unreported, got {other:?}"),
        }
        match parse_auth_status("") {
            AuthState::Unreported(line) => assert_eq!(line, ""),
            other => panic!("expected Unreported, got {other:?}"),
        }
    }

    // ------------------------------------------------------- §2.5 missing_flags

    #[test]
    fn missing_flags_names_exactly_the_absent_entries() {
        let help = "--json --model --cd";
        assert_eq!(
            missing_flags(help, REQUIRED_EXEC_FLAGS).len(),
            REQUIRED_EXEC_FLAGS.len() - 3
        );
        assert!(missing_flags(help, &["--json"]).is_empty());
        assert_eq!(missing_flags(help, &["--nope"]), vec!["--nope"]);
    }

    // -------------------------------------------------------------- §3.2 argv

    #[test]
    fn first_turn_argv_carries_the_measured_shape() {
        let cwd = PathBuf::from("/work/surface");
        let argv = first_turn_argv(&cwd, None);
        assert_eq!(
            argv,
            vec![
                "exec",
                "--json",
                "--skip-git-repo-check",
                "-C",
                "/work/surface"
            ]
        );
        assert!(!argv.contains(&"-m".to_string()), "no model, no -m flag");

        let pinned = first_turn_argv(&cwd, Some("gpt-5.6-luna"));
        assert_eq!(pinned.last().unwrap(), "gpt-5.6-luna");
        assert_eq!(pinned[pinned.len() - 2], "-m");

        for absent in [
            "--add-dir",
            "-s",
            "--sandbox",
            "-p",
            "--profile",
            "--ignore-user-config",
            "--ephemeral",
        ] {
            assert!(
                !argv.contains(&absent.to_string()),
                "{absent} must never appear on turn 1"
            );
        }
        // No positional prompt: the exact-equality assertion above already
        // pins every element, and none of them is the prompt text — the
        // prompt travels on stdin (module docs).
    }

    #[test]
    fn resume_turn_argv_omits_cd_and_places_the_thread_id_right_after_resume() {
        let argv = resume_turn_argv("01a02508-5880-7980-95b7-1d8bc22d5139", None);
        assert_eq!(
            argv,
            vec![
                "exec",
                "resume",
                "01a02508-5880-7980-95b7-1d8bc22d5139",
                "--json",
                "--skip-git-repo-check"
            ]
        );
        assert!(
            !argv.contains(&"-C".to_string()),
            "exec resume has no -C on this build"
        );
        let resume_idx = argv.iter().position(|a| a == "resume").unwrap();
        assert_eq!(argv[resume_idx + 1], "01a02508-5880-7980-95b7-1d8bc22d5139");

        let pinned = resume_turn_argv("thread-x", Some("gpt-5.6-luna"));
        assert!(pinned.contains(&"-m".to_string()));
        assert_eq!(pinned.last().unwrap(), "gpt-5.6-luna");

        for absent in [
            "--add-dir",
            "-s",
            "--sandbox",
            "-p",
            "--profile",
            "--ignore-user-config",
            "--ephemeral",
            "-C",
            "--cd",
        ] {
            assert!(
                !argv.contains(&absent.to_string()),
                "{absent} must never appear on resume"
            );
        }
    }

    // ------------------------------------------------------------ §3.4 prompt

    fn prompt_request(bindings: Vec<BindingSummary>) -> StartRequest {
        StartRequest {
            work_id: "w1".to_string(),
            execution_id: "e1".to_string(),
            stage_id: "s1".to_string(),
            attempt: 1,
            cwd: PathBuf::from("/work"),
            intent: "do the thing".to_string(),
            context: "context body".to_string(),
            model: None,
            profile: None,
            execute: None,
            instruction_policy: Default::default(),
            bindings,
        }
    }

    #[test]
    fn compose_launch_prompt_orders_five_sections_with_no_bindings() {
        let prompt = compose_launch_prompt(&prompt_request(vec![]));
        let sections: Vec<&str> = prompt.split("\n\n").collect();
        assert_eq!(
            sections.len(),
            4,
            "no bindings -> four sections, no mutation-surface claim"
        );
        assert_eq!(sections[0], EXECUTION_MODEL_CONTRACT);
        assert_eq!(sections[1], ENVIRONMENT_CONTRACT);
        assert_eq!(sections[2], "do the thing");
        assert_eq!(sections[3], "context body");
    }

    #[test]
    fn compose_launch_prompt_includes_the_mutation_surface_when_bindings_are_present() {
        let bindings = vec![BindingSummary {
            repository: "solo".to_string(),
            worktree_path: PathBuf::from("/work/solo"),
            work_branch: "sergeant/w1".to_string(),
            base_branch: Some("main".to_string()),
            base_sha: "a".repeat(40),
        }];
        let prompt = compose_launch_prompt(&prompt_request(bindings));
        let sections: Vec<&str> = prompt.split("\n\n").collect();
        assert_eq!(sections.len(), 5);
        assert!(sections[2].starts_with("Mutation surface:"));
        assert!(sections[2].contains("solo: /work/solo (branch sergeant/w1, cut from main at"));
    }

    #[test]
    fn compose_launch_prompt_names_a_detached_admission() {
        let bindings = vec![BindingSummary {
            repository: "solo".to_string(),
            worktree_path: PathBuf::from("/work/solo"),
            work_branch: "sergeant/w1".to_string(),
            base_branch: None,
            base_sha: "b".repeat(40),
        }];
        let prompt = compose_launch_prompt(&prompt_request(bindings));
        assert!(prompt.contains("no named base branch (detached admission)"));
    }

    #[test]
    fn the_environment_contract_matches_claudes_today() {
        assert_eq!(
            ENVIRONMENT_CONTRACT, CLAUDE_ENVIRONMENT_CONTRACT,
            "deliberate duplication (spec §1.3) — a divergence must be a decision, not drift"
        );
    }

    // -------------------------------------------------------- §3.6 model pin

    #[test]
    fn preflight_model_pin_refuses_only_empty_or_whitespace() {
        assert!(preflight_model_pin("gpt-5.6-luna").is_ok());
        assert!(
            preflight_model_pin("openai/gpt-5.6-luna").is_ok(),
            "no provider-qualification refusal on this transport (unlike claude): \
             --oss/--local-provider are real surfaces here and no refusal shape has been measured"
        );
        assert!(preflight_model_pin("").is_err());
        assert!(preflight_model_pin("   ").is_err());
    }

    #[test]
    fn model_pin_evidence_never_reports_honored() {
        assert_eq!(model_pin_evidence(None), json!({"verdict": "unpinned"}));
        let evidence = model_pin_evidence(Some("gpt-5.6-luna"));
        assert_eq!(evidence["verdict"], "attempted");
        assert_eq!(evidence["requested"], "gpt-5.6-luna");
        assert_ne!(evidence["verdict"], "honored");
    }

    // -------------------------------------------------------- §3.3 bindings

    #[test]
    fn bindings_outside_cwd_is_empty_for_the_real_layout_and_names_a_fabricated_escape() {
        let mut request = prompt_request(vec![BindingSummary {
            repository: "solo".to_string(),
            worktree_path: PathBuf::from("/work/solo"),
            work_branch: "b".to_string(),
            base_branch: None,
            base_sha: "0".repeat(40),
        }]);
        request.cwd = PathBuf::from("/work");
        assert!(bindings_outside_cwd(&request.cwd, &request.bindings).is_empty());

        request.bindings.push(BindingSummary {
            repository: "escapee".to_string(),
            worktree_path: PathBuf::from("/somewhere/else"),
            work_branch: "b2".to_string(),
            base_branch: None,
            base_sha: "1".repeat(40),
        });
        let outside = bindings_outside_cwd(&request.cwd, &request.bindings);
        assert_eq!(outside, vec![PathBuf::from("/somewhere/else")]);
    }

    // ----------------------------------------------------- §5.4/§5.3 liveness

    #[test]
    fn argv_names_thread_is_an_adjacency_check_not_a_substring_search() {
        let argv = vec![
            "exec".to_string(),
            "resume".to_string(),
            "thread-1".to_string(),
            "--json".to_string(),
        ];
        assert!(argv_names_thread(&argv, "thread-1"));
        assert!(!argv_names_thread(&argv, "thread-2"));

        // A process that merely *quotes* the id (one joined argv element) is
        // not a running turn.
        let quoting = vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo resume thread-1".to_string(),
        ];
        assert!(!argv_names_thread(&quoting, "thread-1"));

        // `resume` present with the id somewhere later, but not immediately
        // after, and/or no `exec` element at all, must not match either.
        let no_exec = vec!["resume".to_string(), "thread-1".to_string()];
        assert!(
            !argv_names_thread(&no_exec, "thread-1"),
            "no `exec` element"
        );
    }

    #[test]
    fn argv_names_surface_matches_only_the_cd_flag_pair() {
        let argv = vec![
            "exec".to_string(),
            "--json".to_string(),
            "-C".to_string(),
            "/work/surface".to_string(),
        ];
        assert!(argv_names_surface(&argv, Path::new("/work/surface")));
        assert!(!argv_names_surface(&argv, Path::new("/work/other")));
        let long_form = vec![
            "exec".to_string(),
            "--cd".to_string(),
            "/work/surface".to_string(),
        ];
        assert!(argv_names_surface(&long_form, Path::new("/work/surface")));
    }

    #[test]
    fn turn_liveness_among_fails_closed_with_no_scan() {
        assert!(matches!(
            turn_liveness_among("t1", None, 0, None),
            Liveness::Unknowable(_)
        ));
    }

    #[test]
    fn turn_liveness_among_thread_id_beats_surface_ambiguous() {
        let processes = vec![
            ProcessArgv {
                pid: 10,
                argv: vec!["exec".to_string(), "-C".to_string(), "/work".to_string()],
            },
            ProcessArgv {
                pid: 20,
                argv: vec!["exec".to_string(), "resume".to_string(), "t1".to_string()],
            },
        ];
        let liveness = turn_liveness_among("t1", Some(Path::new("/work")), 999, Some(processes));
        assert_eq!(
            liveness,
            Liveness::Alive {
                pid: 20,
                attribution: Attribution::ThreadId
            },
            "ThreadId must win even when a weaker SurfaceAmbiguous match is seen first"
        );
    }

    #[test]
    fn turn_liveness_among_reports_surface_ambiguous_for_a_first_turn_process() {
        let processes = vec![ProcessArgv {
            pid: 30,
            argv: vec!["exec".to_string(), "-C".to_string(), "/work".to_string()],
        }];
        let liveness = turn_liveness_among("t1", Some(Path::new("/work")), 999, Some(processes));
        assert_eq!(
            liveness,
            Liveness::Alive {
                pid: 30,
                attribution: Attribution::SurfaceAmbiguous
            }
        );
    }

    #[test]
    fn turn_liveness_among_is_dead_with_no_matching_process() {
        let processes = vec![ProcessArgv {
            pid: 1,
            argv: vec!["other-program".to_string()],
        }];
        assert_eq!(
            turn_liveness_among("t1", Some(Path::new("/work")), 999, Some(processes)),
            Liveness::Dead
        );
    }

    #[test]
    fn turn_liveness_among_skips_the_callers_own_pid() {
        let processes = vec![ProcessArgv {
            pid: 42,
            argv: vec!["exec".to_string(), "resume".to_string(), "t1".to_string()],
        }];
        assert_eq!(
            turn_liveness_among("t1", None, 42, Some(processes)),
            Liveness::Dead
        );
    }

    // -------------------------------------------------------------- §3.7 pin

    #[test]
    fn thread_pin_mismatch_only_fires_on_disagreement() {
        assert_eq!(thread_pin_mismatch("a", "a"), None);
        assert_eq!(
            thread_pin_mismatch("a", "b"),
            Some("thread pin not honored: resumed a, stream announced b".to_string())
        );
    }

    // -------------------------------------------------- §7.2 decoder, fixtures

    fn lines_of(fixture: &str) -> Vec<Value> {
        fixture
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).expect("fixture line is valid JSON"))
            .collect()
    }

    fn replay(fixture: &str) -> (TurnAccumulator, Vec<NativeEvent>) {
        let mut acc = TurnAccumulator::new();
        let mut events = Vec::new();
        for value in lines_of(fixture) {
            events.extend(acc.ingest_line(&value));
        }
        (acc, events)
    }

    #[test]
    fn a_plain_turn_decodes_to_one_assistant_message_and_one_usage_event() {
        let fixture = include_str!("../../tests/fixtures/codex-0.149.0-agent-message-turn.jsonl");
        let (acc, events) = replay(fixture);
        assert_eq!(
            acc.thread_id.as_deref(),
            Some("01a02508-5880-7980-95b7-1d8bc22d5139")
        );
        assert_eq!(acc.message_items, 1);
        assert_eq!(acc.tool_items, 0);
        assert!(acc.unknown_items.is_empty());
        assert_eq!(acc.unparsed_lines, 0);
        assert_eq!(acc.last_agent_message.as_deref(), Some("ok"));
        assert!(matches!(acc.terminal, Terminal::Completed));
        assert_eq!(
            events.len(),
            1,
            "one conversation.assistant.completed, nothing else"
        );
        assert_eq!(events[0].kind, KIND_CONVERSATION_ASSISTANT_COMPLETED);
        assert_eq!(events[0].payload["text"], "ok");
    }

    #[test]
    fn a_command_execution_item_decodes_to_tool_requested_then_tool_completed() {
        let fixture =
            include_str!("../../tests/fixtures/codex-0.149.0-command-execution-turn.jsonl");
        let (acc, events) = replay(fixture);
        assert_eq!(acc.tool_items, 1);
        assert_eq!(acc.message_items, 2);
        let kinds: Vec<&str> = events.iter().map(|e| e.kind.as_str()).collect();
        assert!(kinds.contains(&KIND_TOOL_REQUESTED));
        assert!(kinds.contains(&KIND_TOOL_COMPLETED));
        let requested_idx = kinds
            .iter()
            .position(|k| *k == KIND_TOOL_REQUESTED)
            .unwrap();
        let completed_idx = kinds
            .iter()
            .position(|k| *k == KIND_TOOL_COMPLETED)
            .unwrap();
        assert!(
            requested_idx < completed_idx,
            "requested must precede completed"
        );
        let completed = &events[completed_idx];
        assert_eq!(
            completed.payload["is_error"], false,
            "exit_code: 0 -> not an error"
        );
        assert_eq!(completed.payload["exit_code"], 0);
        assert_eq!(completed.payload["output_tail"], "unsandboxed-ok\n");
    }

    #[test]
    fn a_failing_command_item_is_an_error_by_exit_code_and_by_status() {
        let mut acc = TurnAccumulator::new();
        let item = json!({"type": "item.completed", "item": {
            "id": "item_9", "type": "command_execution", "command": "false",
            "aggregated_output": "", "exit_code": 1, "status": "completed"
        }});
        let events = acc.ingest_line(&item);
        assert_eq!(
            events[0].payload["is_error"], true,
            "nonzero exit is an error"
        );

        let mut acc2 = TurnAccumulator::new();
        let item2 = json!({"type": "item.completed", "item": {
            "id": "item_10", "type": "command_execution", "command": "true",
            "aggregated_output": "", "exit_code": 0, "status": "failed"
        }});
        let events2 = acc2.ingest_line(&item2);
        assert_eq!(
            events2[0].payload["is_error"], true,
            "status != completed is also an error"
        );
    }

    #[test]
    fn narration_produces_no_tool_events() {
        let fixture =
            include_str!("../../tests/fixtures/codex-0.149.0-uncorroborated-narration-turn.jsonl");
        let (acc, events) = replay(fixture);
        assert_eq!(acc.tool_items, 0, "§4.3: narration is never tool evidence");
        assert_eq!(acc.message_items, 2);
        assert!(
            !events
                .iter()
                .any(|e| e.kind == KIND_TOOL_REQUESTED || e.kind == KIND_TOOL_COMPLETED),
            "zero tool.* events from a turn whose prose claims a command ran and failed"
        );
        assert!(matches!(acc.terminal, Terminal::Completed));
    }

    #[test]
    fn an_error_item_is_journaled_and_is_not_a_terminal() {
        let mut acc = TurnAccumulator::new();
        let item = json!({"type": "item.completed", "item": {
            "id": "item_0", "type": "error", "message": "a metadata warning"
        }});
        let events = acc.ingest_line(&item);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, KIND_TURN_HARNESS_ERROR);
        assert_eq!(events[0].payload["phase"], "item_error");
        assert!(
            matches!(acc.terminal, Terminal::None),
            "an item error is never a terminal"
        );
    }

    #[test]
    fn a_stream_error_line_is_journaled_and_is_not_a_terminal() {
        let mut acc = TurnAccumulator::new();
        let events = acc.ingest_line(&json!({"type": "error", "message": "boom"}));
        assert_eq!(events[0].kind, KIND_TURN_HARNESS_ERROR);
        assert_eq!(events[0].payload["phase"], "stream_error");
        assert!(matches!(acc.terminal, Terminal::None));
        assert_eq!(acc.last_error.as_deref(), Some("boom"));
    }

    #[test]
    fn turn_failed_is_a_failed_terminal_carrying_the_api_message() {
        let fixture = include_str!("../../tests/fixtures/codex-0.149.0-turn-failed.jsonl");
        let (acc, events) = replay(fixture);
        assert!(matches!(acc.terminal, Terminal::Failed { .. }));
        let terminal_outcome = classify_terminal(&acc, false);
        match terminal_outcome {
            TerminalOutcome::Failed { message } => {
                assert!(message.contains("gpt-5.6-nonexistent-model"));
            }
            other => panic!("expected Failed, got {other:?}"),
        }
        // Both the item-level warning and the stream error are journaled;
        // neither is the terminal by itself.
        assert!(events.iter().any(|e| e.payload["phase"] == "item_error"));
        assert!(events.iter().any(|e| e.payload["phase"] == "stream_error"));
    }

    #[test]
    fn turn_completed_is_a_completed_terminal_summarized_by_the_last_agent_message() {
        let mut acc = TurnAccumulator::new();
        acc.ingest_line(&json!({"type": "item.completed", "item": {"id": "i0", "type": "agent_message", "text": "first"}}));
        acc.ingest_line(&json!({"type": "item.completed", "item": {"id": "i1", "type": "agent_message", "text": "last"}}));
        acc.ingest_line(&json!({"type": "turn.completed", "usage": {"input_tokens": 1}}));
        assert_eq!(acc.last_agent_message.as_deref(), Some("last"));
        assert!(matches!(
            classify_terminal(&acc, false),
            TerminalOutcome::Completed
        ));
    }

    #[test]
    fn a_turn_with_no_agent_message_completes_with_no_summary() {
        let mut acc = TurnAccumulator::new();
        acc.ingest_line(&json!({"type": "turn.completed", "usage": {}}));
        assert_eq!(
            acc.last_agent_message, None,
            "None, never Some(String::new())"
        );
    }

    #[test]
    fn an_unknown_item_type_is_counted_and_never_decoded() {
        let mut acc = TurnAccumulator::new();
        let events = acc.ingest_line(&json!({"type": "item.completed", "item": {"id": "i0", "type": "reasoning", "text": "..."}}));
        assert!(events.is_empty());
        assert_eq!(acc.unknown_items, vec!["reasoning".to_string()]);
    }

    #[test]
    fn a_malformed_line_is_counted_tolerated_and_still_archived() {
        // ingest_line only ever sees parsed JSON; the caller (the reader
        // thread) increments unparsed_lines on a parse failure. Pinned here
        // against the accumulator's own counter directly, mirroring what the
        // reader does line-by-line.
        let mut acc = TurnAccumulator::new();
        for line in [
            "not json at all",
            "{\"type\":\"turn.started\"}",
            "{also not json",
        ] {
            match serde_json::from_str::<Value>(line) {
                Ok(value) => {
                    acc.ingest_line(&value);
                }
                Err(_) => acc.unparsed_lines += 1,
            }
        }
        assert_eq!(acc.unparsed_lines, 2);
    }

    #[test]
    fn a_stream_with_no_terminal_classifies_unknown_and_carries_exit_and_stderr() {
        let mut acc = TurnAccumulator::new();
        acc.ingest_line(&json!({"type": "thread.started", "thread_id": "t1"}));
        assert!(matches!(
            classify_terminal(&acc, false),
            TerminalOutcome::AmbiguousUnknown
        ));
        assert!(matches!(
            classify_terminal(&acc, true),
            TerminalOutcome::InterruptedRunning
        ));
    }

    #[test]
    fn a_resume_turn_whose_thread_started_names_another_thread_fails_the_turn() {
        let mut acc = TurnAccumulator::new();
        acc.ingest_line(&json!({"type": "thread.started", "thread_id": "other-thread"}));
        acc.ingest_line(&json!({"type": "turn.completed", "usage": {}}));
        let mismatch = acc
            .thread_id
            .as_deref()
            .and_then(|seen| thread_pin_mismatch("expected-thread", seen));
        assert_eq!(
            mismatch,
            Some(
                "thread pin not honored: resumed expected-thread, stream announced other-thread"
                    .to_string()
            )
        );
    }

    // ------------------------------------------------------------- §5.8 ask

    #[test]
    fn codex_never_reports_an_actor_authored_question() {
        // Structural: `ask` is `false`, a constant, and this decoder has no
        // code path that could construct a KIND_CONVERSATION_ASK payload —
        // pinned by absence across every recorded fixture.
        for fixture in [
            include_str!("../../tests/fixtures/codex-0.149.0-agent-message-turn.jsonl"),
            include_str!("../../tests/fixtures/codex-0.149.0-command-execution-turn.jsonl"),
            include_str!("../../tests/fixtures/codex-0.149.0-turn-failed.jsonl"),
            include_str!("../../tests/fixtures/codex-0.149.0-uncorroborated-narration-turn.jsonl"),
        ] {
            let (_, events) = replay(fixture);
            assert!(!events.iter().any(|e| e.kind == KIND_CONVERSATION_ASK));
        }
        let config = CodexConfig::new(Path::new("/nonexistent"));
        let backend = CodexBackend::new(config);
        assert!(!backend.capabilities().ask);
    }

    // ---------------------------------------------------------- misc plumbing

    #[test]
    fn truncate_cuts_on_character_boundaries() {
        assert_eq!(truncate("hello", 3), "hel");
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("héllo", 2), "h\u{e9}");
    }

    #[test]
    fn codex_config_new_reads_the_bin_env_override() {
        // SAFETY: this test does not run concurrently with another that reads
        // SGT_CODEX_BIN, and is restored before returning.
        let previous = std::env::var_os(CODEX_BIN_ENV);
        unsafe { std::env::set_var(CODEX_BIN_ENV, "/tmp/my-codex") };
        let config = CodexConfig::new(Path::new("/data"));
        assert_eq!(config.executable, PathBuf::from("/tmp/my-codex"));
        match previous {
            Some(value) => unsafe { std::env::set_var(CODEX_BIN_ENV, value) },
            None => unsafe { std::env::remove_var(CODEX_BIN_ENV) },
        }
    }

    #[test]
    fn launch_config_refuses_a_codex_native_profile() {
        let config = CodexConfig::new(Path::new("/nonexistent"));
        let backend = CodexBackend::new(config);
        let mut options = BTreeMap::new();
        options.insert("codex_profile".to_string(), "whatever".to_string());
        let profile = Profile {
            name: "p".to_string(),
            backend: CODEX_BACKEND_NAME.to_string(),
            executable: None,
            config_home: None,
            env: BTreeMap::new(),
            default_model: None,
            options,
        };
        let err = backend
            .launch_config(Some(&profile))
            .expect_err("must be refused");
        let text = err.to_string();
        assert!(text.contains("codex_profile"));
        assert!(text.contains("config_home"));
    }

    #[test]
    fn launch_config_applies_executable_env_and_config_home_from_a_profile() {
        let config = CodexConfig::new(Path::new("/nonexistent"));
        let backend = CodexBackend::new(config);
        let mut env = BTreeMap::new();
        env.insert("FOO".to_string(), "bar".to_string());
        let profile = Profile {
            name: "p".to_string(),
            backend: CODEX_BACKEND_NAME.to_string(),
            executable: Some(PathBuf::from("/custom/codex")),
            config_home: Some(PathBuf::from("/custom/codex-home")),
            env,
            default_model: None,
            options: BTreeMap::new(),
        };
        let resolved = backend
            .launch_config(Some(&profile))
            .expect("valid profile");
        assert_eq!(resolved.executable, PathBuf::from("/custom/codex"));
        assert_eq!(resolved.env.get("FOO"), Some(&"bar".to_string()));
        assert_eq!(
            resolved.codex_home,
            Some(PathBuf::from("/custom/codex-home"))
        );
    }

    #[test]
    fn tracked_executions_is_empty_on_a_fresh_backend() {
        let config = CodexConfig::new(Path::new("/nonexistent"));
        let backend = CodexBackend::new(config);
        assert!(backend.tracked_executions().is_empty());
    }

    #[test]
    fn runtime_scope_is_per_execution() {
        let config = CodexConfig::new(Path::new("/nonexistent"));
        let backend = CodexBackend::new(config);
        assert_eq!(backend.runtime_scope(), RuntimeScope::PerExecution);
        assert_eq!(backend.name(), CODEX_BACKEND_NAME);
    }
}
