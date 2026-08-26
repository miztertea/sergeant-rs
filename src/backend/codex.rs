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
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use serde_json::{Value, json};

use super::{
    Backend, BackendError, BackendSignal, BindingSummary, Capabilities, Completion, EventSink,
    ExecutionHandle, NativeEvent, NativeState, Observation, PreparedExecution, ProbeReport,
    ResumeRequest, RuntimeScope, StartRequest,
};
use crate::backend::child;
use crate::domain::event::{EventDraft, EventSource};
use crate::domain::profile::Profile;
use crate::platform::process::ProcessArgv;
use crate::runtime::blob::BlobStore;
use crate::runtime::graph::{
    KIND_CONVERSATION_ASSISTANT_COMPLETED, KIND_CONVERSATION_TURN_ENDED, KIND_CONVERSATION_USER,
    KIND_TOOL_COMPLETED, KIND_TOOL_REQUESTED, KIND_USAGE_UPDATED,
};

/// The app-server JSON-RPC client (W3 spec, `knowledge/evidence/resources/
/// h-series/w3-spec.md` §1.5). `#[path]` because this file is a non-`mod.rs`
/// module (`codex.rs` itself): the spec names the file
/// `src/backend/codex_appserver.rs`, a sibling of this one, and this
/// attribute is what makes `mod codex_appserver;` look there instead of
/// `src/backend/codex/codex_appserver.rs`. Declared here rather than in
/// `backend/mod.rs` is the visibility the spec asks for ("pub(crate) to
/// src/backend/codex.rs"): a private child of *this* module can see every
/// private item this module defines or imports (`ItemView`, `ItemKind`,
/// `TurnAccumulator`, the `KIND_*` constants above), which is exactly how it
/// reuses the one decoder rather than owning a second copy of it (§1.6).
#[path = "codex_appserver.rs"]
mod codex_appserver;

/// Re-exported so [`CodexConfig::appserver_budgets`] can name it in a public
/// field: `codex_appserver` itself stays a private child module (the spec's
/// own visibility rule, above), but a `pub use` of one item out of a private
/// module is the standard way to give that one item a public path without
/// making the whole module public — exactly the seam `CONTRIBUTING.md`'s
/// "narrowest visibility that works" rule asks for.
pub use codex_appserver::Budgets;

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
    /// Which transport this registration resolves to (W3 spec §5.2), default
    /// [`TransportChoice::Auto`]. Resolved **once**, at probe time, and
    /// journaled; never re-resolved per execution (§5.3).
    pub transport: TransportChoice,
    /// Sandbox policy for sergeant-launched turns (W3 spec §3.3), default
    /// [`SandboxChoice::WorkspaceWrite`].
    pub sandbox: SandboxChoice,
    /// A JSON Schema constraining the final assistant message of every turn
    /// in executions launched from this config (W3 spec §4.2). Adapter-local
    /// until a contract revision gives native structured output a home in
    /// `Capabilities`; unused unless a profile or this field sets it.
    pub output_schema: Option<Value>,
    /// Overrides for the app-server child's own budgets (spec §1.5.4). `None`
    /// in every production path — the same per-instance-not-global posture as
    /// `thread_id_budget` above, and for the same reason. A named
    /// [`codex_appserver::Budgets`] rather than a positional tuple: every
    /// field is a `Duration`, so a tuple would let two same-typed slots swap
    /// silently and still type-check.
    pub appserver_budgets: Option<codex_appserver::Budgets>,
    /// #262: composes codex-cli's own documented `-c
    /// sandbox_workspace_write.network_access=true` config override on every
    /// turn 1 this config launches (§ [`first_turn_argv`]'s own doc comment).
    /// `false` by default and never implied by any other setting.
    ///
    /// **This is the daemon-global default, not a per-profile setting on its
    /// own** — one `CodexBackend` (and the `CodexConfig` it was built from)
    /// is shared by every codex-backed profile in a daemon. A profile that
    /// sets its own `network_access` option (`"true"`/`"false"`,
    /// `domain::profile::NETWORK_ACCESS_OPTION`) overrides this field for
    /// its own launches (`CodexBackend::launch_config` resolves the two, W5
    /// fix for split-hardening's #262 review finding); a profile that never
    /// mentions the option falls back to whatever this field is set to.
    /// Never default-on, and never `danger-full-access`. Grants sandbox
    /// network access only — external egress is not the intent and this
    /// setting does not provide it.
    pub workspace_write_network_access: bool,
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
            transport: TransportChoice::Auto,
            sandbox: SandboxChoice::WorkspaceWrite,
            output_schema: None,
            appserver_budgets: None,
            workspace_write_network_access: false,
        }
    }
}

/// Which transport a `CodexBackend` registration resolves to (W3 spec §5.2).
/// `CodexConfig`'s own field, default [`TransportChoice::Auto`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransportChoice {
    /// App-server if every gate passes (§5.2 rule 3), else exec — the
    /// honest fallback named in the probe detail.
    #[default]
    Auto,
    /// Always exec, regardless of what the app-server gates would say.
    ExecOnly,
    /// App-server or refuse: if the gates fail, `PROBE` reports
    /// `available: false` naming the failed gate, rather than silently
    /// giving the operator a different transport with a different
    /// capability row (§5.2 rule 2).
    AppServerOnly,
}

/// The transport actually resolved for one registration (§5.2) — internal;
/// `TransportChoice` is the operator-facing configuration, this is the
/// outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Transport {
    Exec,
    AppServer,
}

impl Transport {
    fn as_str(self) -> &'static str {
        match self {
            Transport::Exec => "exec",
            Transport::AppServer => "app-server (stdio)",
        }
    }
}

/// Sandbox policy for sergeant-launched turns (W3 spec §3.3). Layered
/// exactly as every other §14 axis: this is `CodexConfig`'s own default,
/// applied uniformly to every execution this backend launches — a
/// per-execution override is not a surface this wave adds (nothing in
/// `StartRequest` names one, and R3 forbids inventing a core field for it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SandboxChoice {
    /// `-s read-only` / `sandbox: "readOnly"`. Never sergeant's own default —
    /// a Work exists to change its worktree.
    ReadOnly,
    /// `-s workspace-write` / `sandbox: "workspaceWrite"`, scoped to the
    /// Work's declared surfaces. The default: enforcement the owner
    /// welcomed (R4), never wider than the Work's own bindings.
    #[default]
    WorkspaceWrite,
    /// Send no sandbox parameter at all — the operator's own
    /// `~/.codex/config.toml` decides. The escape hatch for an operator with
    /// a considered policy of their own; never the default, because a
    /// default that silently defers is a default nobody chose.
    Inherit,
}

impl SandboxChoice {
    /// The app-server `thread/start.sandbox` string, or `None` for
    /// [`SandboxChoice::Inherit`] (send no `sandbox` param at all).
    fn appserver_value(self) -> Option<&'static str> {
        match self {
            // Re-measured while wiring LAUNCH (a live -32600 caught this):
            // `thread/start.sandbox` takes the same kebab-case request
            // values as exec's own `-s`/`--sandbox` (`read-only` /
            // `workspace-write` / `danger-full-access`) — the camelCase
            // spelling (`workspaceWrite`) is only what the *response*
            // echoes back in `sandbox.type`, a different field entirely.
            SandboxChoice::ReadOnly => Some("read-only"),
            SandboxChoice::WorkspaceWrite => Some("workspace-write"),
            SandboxChoice::Inherit => None,
        }
    }

    /// The exec `-s`/`--sandbox` value, or `None` for
    /// [`SandboxChoice::Inherit`] (compose no `--sandbox`/`--add-dir` at all
    /// — §3.2's turn-1-only asymmetry, unchanged by this choice).
    fn exec_value(self) -> Option<&'static str> {
        match self {
            SandboxChoice::ReadOnly => Some("read-only"),
            SandboxChoice::WorkspaceWrite => Some("workspace-write"),
            SandboxChoice::Inherit => None,
        }
    }
}

// ---------------------------------------------------------- admission rows

/// How a capability's `true`/`false` was established (W3 spec §0.2/§2.1).
/// A schema entry is a doc claim; it proves the protocol *names* a thing,
/// never that it fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Evidence {
    /// Driven against the real, installed harness (a `#[ignore]`d live test,
    /// gated behind `SERGEANT_CODEX_TESTS=1`).
    LiveMeasured,
    /// Proven deterministically (a fixture, a stub) without a live run —
    /// still a real assertion, just not against the installed binary today.
    LocallyMeasured,
    /// Named by `generate-json-schema`'s own dump; never promoted to
    /// `claimed: true` on this evidence alone (§0.2's promotion rule).
    SchemaClaimed,
    /// Looked for and not found — a probe ran, no assertion could be made.
    Unmeasured,
}

/// Protocol stability, as the app-server subcommand's own header names it
/// (`[experimental] Run the app server or related tooling`, M1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stability {
    Stable,
    Experimental,
}

/// One row of the wave's own capability ledger (W3 spec §2.1): the v1
/// boolean `capabilities()` returns is the contract; this is the *evidence*
/// behind it, adapter-local until a contract revision gives it a home.
/// Rendered into `ProbeReport::detail` and the wave PR body (§2.7).
#[derive(Debug, Clone, Copy)]
struct AdmissionRow {
    /// The v1 flag name, or a name v1 has no row for at all
    /// (`structured_output`, `sandbox_enforcement`).
    capability: &'static str,
    transport: Transport,
    /// What `capabilities()` claims for this transport (always the same
    /// value on both transports today — see the row's own note when that
    /// is itself the interesting fact).
    claimed: bool,
    /// The typed tier this row's evidence actually supports
    /// (`ProcessTreeTermination`, `NativeTurnInterrupt`, `NativeSchema`,
    /// `NativeOsSandbox(workspace-write)`, or `"-"` when the flag is a
    /// plain boolean with no tier of its own).
    tier: &'static str,
    evidence: Evidence,
    stability: Stability,
    /// The exact test name backing a `claimed: true`, or `""` when
    /// `claimed` is `false` (the structural reason lives in `note`).
    admission_test: &'static str,
    note: &'static str,
}

/// The wave's own ledger (§2.6's table, plus §4.1/§3.1's two rows with no
/// v1 boolean). [`admission_rows_agree_with_capabilities`] is the
/// structural check that keeps this honest: a `claimed: true` with no
/// `admission_test` fails the build.
const ADMISSION_ROWS: &[AdmissionRow] = &[
    AdmissionRow {
        capability: "persistent_sessions",
        transport: Transport::Exec,
        claimed: true,
        tier: "-",
        // `launch_binds_the_thread_id_from_thread_started` is a plain
        // `#[test]` driven by `StubCodex` -- no `#[ignore]`, no
        // SERGEANT_CODEX_TESTS gate, never touches the installed binary.
        // `LiveMeasured` here would misstate its provenance (`Evidence`'s own
        // doc comment reserves that tier for a real, installed-harness run).
        evidence: Evidence::LocallyMeasured,
        stability: Stability::Stable,
        admission_test: "launch_binds_the_thread_id_from_thread_started",
        note: "the rollout jsonl under <codex_home>/sessions/**",
    },
    AdmissionRow {
        capability: "persistent_sessions",
        transport: Transport::AppServer,
        claimed: true,
        tier: "-",
        evidence: Evidence::LiveMeasured,
        stability: Stability::Experimental,
        admission_test: "live_appserver_handshake_and_thread_start",
        note: "thread/start's own result.thread.path names the rollout file directly (M4)",
    },
    AdmissionRow {
        capability: "native_background",
        transport: Transport::Exec,
        claimed: false,
        tier: "-",
        evidence: Evidence::Unmeasured,
        stability: Stability::Stable,
        admission_test: "",
        note: "no measured mechanism for a turn to survive client disconnect",
    },
    AdmissionRow {
        capability: "native_background",
        transport: Transport::AppServer,
        claimed: false,
        tier: "-",
        evidence: Evidence::Unmeasured,
        stability: Stability::Experimental,
        admission_test: "",
        note: "our child dies with the execution by construction (§1.4); not even meaningful \
               to ask on a per-execution child",
    },
    AdmissionRow {
        capability: "streaming",
        transport: Transport::Exec,
        claimed: true,
        tier: "-",
        evidence: Evidence::LiveMeasured,
        stability: Stability::Stable,
        admission_test: "live_codex_turn_streams_events_before_it_ends",
        note: "item.started/item.completed",
    },
    AdmissionRow {
        capability: "streaming",
        transport: Transport::AppServer,
        claimed: true,
        tier: "-",
        evidence: Evidence::LiveMeasured,
        stability: Stability::Experimental,
        admission_test: "live_appserver_turn_completes_and_streams",
        note: "finer-grained: item + delta notifications; deltas counted, never decoded",
    },
    AdmissionRow {
        capability: "history",
        transport: Transport::Exec,
        claimed: false,
        tier: "-",
        evidence: Evidence::Unmeasured,
        stability: Stability::Stable,
        admission_test: "",
        note: "the only complete native record is the rollout jsonl, an unmeasured on-disk \
               format this milestone never reads for content",
    },
    AdmissionRow {
        capability: "history",
        transport: Transport::AppServer,
        claimed: false,
        tier: "-",
        evidence: Evidence::SchemaClaimed,
        stability: Stability::Experimental,
        admission_test: "",
        note: "thread/read / thread/items/list / thread/turns/list exist [schema-claimed] and \
               are never called (§1.5.3) — proving completeness against the rollout is the \
               largest named gap this wave leaves, handed to a future wave",
    },
    AdmissionRow {
        capability: "resume",
        transport: Transport::Exec,
        claimed: true,
        tier: "-",
        evidence: Evidence::LiveMeasured,
        stability: Stability::Stable,
        admission_test: "live_codex_thread_survives_turns_and_a_restart",
        note: "codex exec resume <thread_id>, measured across fresh OS processes",
    },
    AdmissionRow {
        capability: "resume",
        transport: Transport::AppServer,
        claimed: true,
        tier: "-",
        evidence: Evidence::LiveMeasured,
        stability: Stability::Experimental,
        admission_test: "live_codex_thread_survives_turns_and_a_restart",
        note: "served by the exec transport across a daemon restart (§5.4); thread/resume is \
               never called — a re-adopted execution journals a transport_withdrawn_on_readopt \
               capability withdrawal",
    },
    AdmissionRow {
        capability: "interrupt",
        transport: Transport::Exec,
        claimed: true,
        tier: "ProcessTreeTermination",
        // `codex_interrupt_kills_the_process_group` is StubCodex-driven and
        // deterministic -- its own doc comment names the live half as a
        // *different* test (`live_codex_interrupt_leaves_the_conversation_
        // resumable`). Tagging this row `LiveMeasured` would credit the
        // wrong test with a live run it never performs.
        evidence: Evidence::LocallyMeasured,
        stability: Stability::Stable,
        admission_test: "codex_interrupt_kills_the_process_group",
        note: "kills the turn's process group; no terminal — InterruptedRunning is inferred",
    },
    AdmissionRow {
        capability: "interrupt",
        transport: Transport::AppServer,
        claimed: true,
        tier: "NativeTurnInterrupt",
        evidence: Evidence::LiveMeasured,
        stability: Stability::Experimental,
        admission_test: "live_appserver_interrupt_yields_an_interrupted_terminal",
        note: "turn/interrupt -> turn/completed{status:\"interrupted\"}, a first-class \
               harness-confirmed terminal (§2.2) -- measured live end to end, including a \
               second turn on the same thread proving resumability. On any turn/interrupt \
               failure the adapter falls back to the process-group kill and journals \
               phase:\"interrupt_downgraded\"",
    },
    AdmissionRow {
        capability: "model_selection",
        transport: Transport::Exec,
        claimed: true,
        tier: "-",
        evidence: Evidence::LiveMeasured,
        stability: Stability::Stable,
        admission_test: "live_codex_bad_model_pin_fails_loud_not_silent",
        note: "substitution undetectable: turn.completed.usage carries no model field",
    },
    AdmissionRow {
        capability: "model_selection",
        transport: Transport::AppServer,
        claimed: true,
        tier: "-",
        evidence: Evidence::LiveMeasured,
        stability: Stability::Experimental,
        admission_test: "live_appserver_thread_start_echoes_the_requested_policy",
        note: "detectable here: thread/start echoes \"model\":\"<pin>\", and model/rerouted is a \
               live notification method [schema-claimed] -- a verification layer exec never had",
    },
    AdmissionRow {
        capability: "profiles",
        transport: Transport::Exec,
        claimed: true,
        tier: "-",
        evidence: Evidence::LocallyMeasured,
        stability: Stability::Stable,
        admission_test: "a_profile_config_home_sets_codex_home_on_every_turn",
        note: "codex-native -p/--profile refused: cannot re-apply on exec resume",
    },
    AdmissionRow {
        capability: "profiles",
        transport: Transport::AppServer,
        claimed: true,
        tier: "-",
        evidence: Evidence::LocallyMeasured,
        stability: Stability::Experimental,
        admission_test: "a_profile_config_home_sets_codex_home_on_every_turn",
        note: "same axis, same refusal -- thread/start.permissions [schema-claimed] cannot be \
               combined with sandbox, which §3 needs, so the codex-native layer stays refused",
    },
    AdmissionRow {
        capability: "approval_flow",
        transport: Transport::Exec,
        claimed: false,
        tier: "-",
        evidence: Evidence::Unmeasured,
        stability: Stability::Stable,
        admission_test: "",
        note: "structural: no approval channel exists on this transport at all",
    },
    AdmissionRow {
        capability: "approval_flow",
        transport: Transport::AppServer,
        claimed: false,
        tier: "-",
        // `appserver_answers_every_server_request_including_unknown_ones` is
        // a pure unit test against `answer_for_request()` -- no process
        // spawned, no live app-server touched. `LiveMeasured` would overstate
        // it; this is exactly the deterministic tier the enum defines.
        evidence: Evidence::LocallyMeasured,
        stability: Stability::Experimental,
        admission_test: "appserver_answers_every_server_request_including_unknown_ones",
        note: "false by policy, deliberately: approvalPolicy is always \"never\", so no \
               approval-gate request should ever fire; the five approval methods are answered \
               denied if one ever does (J5 safety), never advertised as a real flow",
    },
    AdmissionRow {
        capability: "human_attach",
        transport: Transport::Exec,
        claimed: false,
        tier: "-",
        evidence: Evidence::Unmeasured,
        stability: Stability::Stable,
        admission_test: "",
        note: "no attach mechanism on print-mode turns",
    },
    AdmissionRow {
        capability: "human_attach",
        transport: Transport::AppServer,
        claimed: false,
        tier: "-",
        evidence: Evidence::Unmeasured,
        stability: Stability::Experimental,
        admission_test: "",
        note: "a scope decision (§1.4), not an unlooked-for absence: codex --remote / codex \
               agents attach to the shared daemon, which this wave refuses on ladder grounds \
               (R1/R5) -- our child is per-execution and adapter-owned",
    },
    AdmissionRow {
        capability: "usage",
        transport: Transport::Exec,
        claimed: true,
        tier: "-",
        evidence: Evidence::LiveMeasured,
        stability: Stability::Stable,
        admission_test: "live_codex_turn_reports_usage",
        note: "known only at turn.completed",
    },
    AdmissionRow {
        capability: "usage",
        transport: Transport::AppServer,
        claimed: true,
        tier: "-",
        evidence: Evidence::LiveMeasured,
        stability: Stability::Experimental,
        admission_test: "live_appserver_reports_usage_during_the_turn",
        note: "thread/tokenUsage/updated pushed mid-turn, turn-attributed, and measured live to \
               arrive before the terminal -- known earlier and more precisely than exec",
    },
    AdmissionRow {
        capability: "native_subagents",
        transport: Transport::Exec,
        claimed: false,
        tier: "-",
        evidence: Evidence::Unmeasured,
        stability: Stability::Stable,
        admission_test: "",
        note: "no subagent mechanism on this transport",
    },
    AdmissionRow {
        capability: "native_subagents",
        transport: Transport::AppServer,
        claimed: false,
        tier: "-",
        evidence: Evidence::SchemaClaimed,
        stability: Stability::Experimental,
        admission_test: "",
        note: "collabAgentToolCall / subAgentActivity item types exist [schema-claimed]; no \
               subagent was ever run -- documented is not supported (§15)",
    },
    AdmissionRow {
        capability: "ask",
        transport: Transport::Exec,
        claimed: false,
        tier: "-",
        evidence: Evidence::Unmeasured,
        stability: Stability::Stable,
        admission_test: "",
        note: "no ask channel on this transport at all [measured-negative]",
    },
    AdmissionRow {
        capability: "ask",
        transport: Transport::AppServer,
        claimed: false,
        tier: "-",
        evidence: Evidence::Unmeasured,
        stability: Stability::Experimental,
        // §2.7 outcome 2: "the test stays in the suite as an #[ignore]d live
        // probe so the next person re-runs it instead of re-deriving it."
        // The prose this row previously carried in place of that test
        // (specific prompt formulations, an exact quoted model refusal) was
        // never backed by a committed, re-runnable test -- fixed by adding
        // one rather than by trusting the prose.
        admission_test: "live_appserver_actor_authored_question_is_typed",
        note: "§2.4's five-step admission test: step 1 (approvalPolicy: never) always holds by \
               construction (§3.3); steps 2-3 are a live, gpt-5.6-luna, two-formulation model- \
               behaviour probe -- re-run it to see this build's current measurement. Steps 4-5 \
               are not attempted regardless of what the probe measures: no NeedsInput mapping or \
               answering path exists in this build, so ask stays false either way (§2.4's \
               \"only if admitted\" scope). Recorded here rather than promoted: absence of a \
               probe result under this launch grammar is not a measured negative of the tool's \
               existence (§2.4's own outcome table, first bullet)",
    },
    AdmissionRow {
        capability: "structured_output",
        transport: Transport::Exec,
        claimed: true,
        tier: "NativeSchema",
        evidence: Evidence::LiveMeasured,
        stability: Stability::Stable,
        admission_test: "live_codex_output_schema_round_trips_on_both_transports",
        note: "--output-schema <path>; re-measured for W3: DOES re-apply on `exec resume` \
               (present in --help), correcting W1's own turn-1-only placeholder -- not a v1 \
               flag; recorded here as adapter evidence only (§4.1)",
    },
    AdmissionRow {
        capability: "structured_output",
        transport: Transport::AppServer,
        claimed: true,
        tier: "NativeSchema",
        evidence: Evidence::LiveMeasured,
        stability: Stability::Experimental,
        admission_test: "live_codex_output_schema_round_trips_on_both_transports",
        note: "turn/start.outputSchema, every turn -- measured live with the spec's own \
               discriminating control (an unschema'd run must not coincidentally match)",
    },
    AdmissionRow {
        capability: "sandbox_enforcement",
        transport: Transport::Exec,
        claimed: true,
        tier: "NativeOsSandbox(workspace-write)",
        evidence: Evidence::LocallyMeasured,
        stability: Stability::Stable,
        // Spec §3.6 proposed `exec_first_turn_argv_carries_the_sandbox_and_
        // extra_dirs` as this test's name; the implementation folded the
        // same coverage into the pre-existing, extended
        // `first_turn_argv_carries_the_measured_shape` instead of adding a
        // second function under the spec's literal name. Naming the real
        // function here keeps this citation resolvable.
        admission_test: "first_turn_argv_carries_the_measured_shape",
        note: "turn-1-only: exec resume has neither -s nor --add-dir on this build -- the \
               composed-flags handshake is proven, enforcement itself is not (bwrap cannot \
               initialize a network namespace on Cerberus, H0 §C.3 finding 4)",
    },
    AdmissionRow {
        capability: "sandbox_enforcement",
        transport: Transport::AppServer,
        claimed: true,
        tier: "NativeOsSandbox(workspace-write)",
        // `LiveMeasured`, because what this row's note claims -- that the
        // harness *accepts and echoes* the requested policy -- is a fact
        // about the installed binary, and only the `#[ignore]`d,
        // SERGEANT_CODEX_TESTS-gated test named below can establish it. The
        // deterministic sibling (`appserver_thread_start_names_exactly_the_
        // works_surfaces`) proves what this adapter *sends*, which is a
        // different claim and not the one written here.
        // `a_claimed_row_naming_a_live_test_is_labelled_live_measured` is the
        // structural check that keeps the label and the citation from
        // drifting apart again.
        evidence: Evidence::LiveMeasured,
        stability: Stability::Experimental,
        admission_test: "live_appserver_thread_start_echoes_the_requested_policy",
        note: "thread-scoped, holds for every turn (unlike exec's turn-1-only lapse) -- the \
               harness accepts and echoes the requested policy naming exactly this Work's \
               surfaces as writable roots; whether the OS sandbox actually denies an \
               out-of-surface write could not be proven on this host for the same reason exec's \
               row could not. Enforcement-claimed, not locally proven -- sergeant's own \
               observation layer remains the source of truth for what a Work actually changed",
    },
];

/// Render [`ADMISSION_ROWS`] into the plain-text table the wave PR body
/// pastes verbatim (§2.7's three-outcome labelling) and the probe's own
/// journaled detail carries. One line per row: capability, transport,
/// claimed, tier, evidence, stability, the admission test (or a dash), then
/// the note.
fn render_admission_rows() -> String {
    let mut out = String::from(
        "capability | transport | claimed | tier | evidence | stability | admission_test | note\n",
    );
    for row in ADMISSION_ROWS {
        out.push_str(&format!(
            "{} | {} | {} | {} | {:?} | {:?} | {} | {}\n",
            row.capability,
            row.transport.as_str(),
            row.claimed,
            row.tier,
            row.evidence,
            row.stability,
            if row.admission_test.is_empty() {
                "-"
            } else {
                row.admission_test
            },
            row.note,
        ));
    }
    out
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

/// Outcome of W3's four app-server admission gates (§5.2), memoized
/// alongside [`ProbeOutcome`].
#[derive(Debug, Clone)]
struct AppServerGates {
    /// `Ok(())` when G1, G2 and G4 all passed. `Err` names the gate and why
    /// (each message is prefixed `"G1: "`/`"G2: "`/`"G4: "`).
    result: Result<(), String>,
    /// G3: whether the installed protocol's scoped fingerprint disagreed
    /// with [`codex_appserver::MEASURED_PROTOCOL_FINGERPRINT`] — provenance,
    /// never a gate failure (R1). Only meaningful when `result` reached G3
    /// (i.e. is not a G1 failure); `false` on a G1 failure, harmlessly.
    stale: bool,
}

/// §5.2's resolved outcome for one registration — computed once, journaled,
/// never revisited per execution (§5.3).
#[derive(Debug, Clone)]
struct TransportResolution {
    transport: Transport,
    detail: String,
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
    /// #262 (per-profile, W5 fix): this execution's resolved
    /// `sandbox_workspace_write.network_access` — the profile's own
    /// `network_access` option when it sets one, else `CodexConfig::
    /// workspace_write_network_access`. See [`CodexBackend::launch_config`].
    network_access: bool,
}

/// Turn 1's argv, after `<executable>` (spec §3.2, updated by W3 §3.2/§3.6):
/// `exec --json --skip-git-repo-check -C <cwd> [-m <model>] [--sandbox
/// <value>] [--add-dir <path> ...]`. Prompt travels on stdin, no positional
/// argument — see the module docs for why.
///
/// `sandbox`/`extra_dirs`/`network_access` are **turn-1-only** — `exec
/// resume` has neither `-s`/`--add-dir` nor `-c` on this build
/// (`resume_turn_argv` below), so enforcement lapses on turn 2 of an
/// exec-transport conversation. That asymmetry is recorded, not routed
/// around (W3 spec §3.2): a first turn is where a Work does most of its
/// damage, and it is itself an argument for app-server as the resolved
/// transport, where the policy is thread-scoped and holds for every turn.
fn first_turn_argv(
    cwd: &Path,
    model: Option<&str>,
    sandbox: SandboxChoice,
    extra_dirs: &[PathBuf],
    output_schema_path: Option<&Path>,
    network_access: bool,
) -> Vec<String> {
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
    if let Some(value) = sandbox.exec_value() {
        argv.push("--sandbox".to_string());
        argv.push(value.to_string());
    }
    for dir in extra_dirs {
        argv.push("--add-dir".to_string());
        argv.push(dir.to_string_lossy().into_owned());
    }
    if let Some(path) = output_schema_path {
        argv.push("--output-schema".to_string());
        argv.push(path.to_string_lossy().into_owned());
    }
    // #262: codex-cli's own documented `sandbox_workspace_write.network_access`
    // config override, composed only when this launch's resolved
    // `network_access` is true (`CodexBackend::launch_config` — a profile's
    // own `network_access` option, falling back to `CodexConfig::
    // workspace_write_network_access` when the profile does not set one) —
    // never the adapter's default, and scoped to the sandboxed
    // workspace-write child, never `danger-full-access`. This grants sandbox
    // network access (e.g. binding `127.0.0.1` for a repository's own test
    // harness); it is not external egress and does not widen what
    // `--add-dir` above already scopes as writable.
    if network_access {
        argv.push("-c".to_string());
        argv.push("sandbox_workspace_write.network_access=true".to_string());
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
///
/// `output_schema_path` **does** re-apply here, unlike `sandbox`/`extra_dirs`
/// above: `exec resume --help` on 0.149.0 lists `--output-schema` (and
/// `-o`/`--output-last-message`) — re-measured while implementing W3, and a
/// correction to W1's own "verify at implementation time" placeholder
/// (spec §4.2). So `structured_output`'s exec row is **not** turn-1-only.
fn resume_turn_argv(
    thread_id: &str,
    model: Option<&str>,
    output_schema_path: Option<&Path>,
) -> Vec<String> {
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
    if let Some(path) = output_schema_path {
        argv.push("--output-schema".to_string());
        argv.push(path.to_string_lossy().into_owned());
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

/// #259, re-scoped after the independent review of split/w5-codex-commit-path
/// (findings #259/#262, wave W5) showed the narrower grant was insufficient:
/// the one directory a linked worktree's own `git add` needs write access to
/// that is not `worktree_path` itself — git's private per-worktree
/// administrative directory, `<common-dir>/worktrees/<name>`. `git worktree
/// add` never checks that directory out under `worktree_path`
/// (`runtime::surface::add_worktree_no_checkout`); it lives beside the
/// repository's shared `.git`, which is exactly why a sandbox scoped to the
/// Work's own surfaces denies `index.lock` there.
///
/// This directory alone is **not** enough for `git commit`: measured under a
/// deny-outside-grants bubblewrap sandbox (grant exactly this directory plus
/// `worktree_path`, nothing else), `git add` succeeds but `git commit` fails
/// — first on `.git/objects` (new loose objects), then on
/// `.git/logs/refs/heads/<branch>` (the reflog) and `.git/refs/heads/<branch>`
/// (the ref update itself), all of which live in the shared common
/// directory, never under this per-worktree admin path. See
/// [`worktree_git_common_dir`], which this function is now a step of rather
/// than the whole answer.
///
/// Resolved by reading `worktree_path`'s own `.git` file rather than
/// spawning `git rev-parse` — `Backend::prepare`'s own contract forbids an
/// external effect (no process, no blocking wait), and every linked worktree
/// git ever creates has a `.git` **file** (never a directory) whose one line
/// reads `gitdir: <absolute path>` naming this exact directory [git
/// worktree(1)].
fn worktree_git_admin_dir(worktree_path: &Path) -> Result<PathBuf, String> {
    let dot_git = worktree_path.join(".git");
    let contents = std::fs::read_to_string(&dot_git)
        .map_err(|e| format!("cannot read {}: {e}", dot_git.display()))?;
    let line = contents.trim();
    let raw = line.strip_prefix("gitdir:").ok_or_else(|| {
        format!(
            "{} does not name a linked worktree's git directory (expected a \"gitdir: \
             <path>\" file, found {:?})",
            dot_git.display(),
            line
        )
    })?;
    let path = PathBuf::from(raw.trim());
    if !path.is_absolute() {
        return Err(format!(
            "{} named a non-absolute gitdir {}",
            dot_git.display(),
            path.display()
        ));
    }
    Ok(path)
}

/// Collapse `.` and `..` path components lexically — no filesystem access,
/// no symlink resolution. Used to turn `<admin-dir>/../..` (the literal
/// shape [`worktree_git_common_dir`] joins) into a clean absolute path
/// without canonicalizing it, which would silently change the grant if any
/// component were a symlink.
fn lexically_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// #259 (re-scoped, W5 fix): the directory a linked worktree's `git
/// add`/`git commit` actually need write access to — the repository's
/// **shared** `.git` (its "common directory" [git-worktree(1)],
/// `git rev-parse --git-common-dir`), not merely the per-worktree admin
/// subdirectory [`worktree_git_admin_dir`] resolves. Loose objects
/// (`objects/`), the ref update (`refs/heads/<branch>`), the reflog
/// (`logs/refs/heads/<branch>`), and — opportunistically, on some commits —
/// `packed-refs` all live directly under the common dir, never under
/// `<common-dir>/worktrees/<name>`. Measured empirically (bubblewrap,
/// deny-outside-grants) before this fix: granting only the admin
/// subdirectory lets `git add` succeed but `git commit` fail with
/// `Read-only file system` on `.git/objects`; granting the whole common dir
/// resolves both.
///
/// **This is a wider grant than the admin subdirectory alone, and it is
/// honest about the isolation cost**: `<common-dir>/worktrees/<name>` is the
/// admin state for *this* linked worktree only, but the common dir also
/// holds every other linked worktree's refs, reflogs, and the full object
/// store for the whole repository. A Work whose codex actor is granted this
/// path has filesystem-level write access to any other Work's branch ref or
/// reflog in the *same* repository, if one happens to be checked out
/// concurrently (`runtime::repolock` only serializes the `git worktree add`
/// span, not a Work's whole lifetime) — that isolation is no longer sandbox-
/// enforced, only trust-based (each actor is instructed to touch only its
/// own worktree, and `common_dir_finding` / branch-tip assertions in the
/// close pipeline would catch a rewritten ref after the fact, not prevent
/// it). Isolation **between different repositories** is untouched: each
/// repository has its own common dir, so this grant never crosses a
/// repository boundary. `m6_surfaces.rs`'s
/// `git_worktree_common_dir_grant_is_shared_within_a_repository_but_never_
/// crosses_repositories` regression records both halves of this trade.
///
/// Resolved from the admin dir's own `commondir` file — the same file `git`
/// itself reads to find its common directory [git-worktree(1)] — rather
/// than assumed from path shape (`<admin-dir>/../..`), so a future git
/// layout change is a resolution error here, not a silently wrong grant.
fn worktree_git_common_dir(worktree_path: &Path) -> Result<PathBuf, String> {
    let admin_dir = worktree_git_admin_dir(worktree_path)?;
    let commondir_file = admin_dir.join("commondir");
    let contents = std::fs::read_to_string(&commondir_file)
        .map_err(|e| format!("cannot read {}: {e}", commondir_file.display()))?;
    let relative = contents.trim();
    if relative.is_empty() {
        return Err(format!("{} is empty", commondir_file.display()));
    }
    Ok(lexically_normalize(&admin_dir.join(relative)))
}

/// [`worktree_git_common_dir`] over every binding this request carries.
/// `Err` on the first binding whose common directory cannot be resolved —
/// the caller (`prepare`/`launch`) turns that into a refusal rather than
/// launching an actor that can edit files but cannot commit them.
fn git_worktree_common_dirs(bindings: &[BindingSummary]) -> Result<Vec<PathBuf>, String> {
    bindings
        .iter()
        .map(|binding| {
            worktree_git_common_dir(&binding.worktree_path).map_err(|reason| {
                format!(
                    "{} ({}): {reason}",
                    binding.repository,
                    binding.worktree_path.display()
                )
            })
        })
        .collect()
}

/// [`worktree_git_admin_dir`] over every binding this request carries —
/// the app-server-only companion to [`git_worktree_common_dirs`].
///
/// **Measured 2026-08-25 against codex-cli 0.149.1's `app-server`**: unlike
/// `exec`'s `--add-dir` (whose grant of the common dir alone already covers
/// `git add`/`git commit`), `thread/start.runtimeWorkspaceRoots` resolves
/// into a `sandbox.writableRoots` grant that treats each linked worktree's
/// private admin subdirectory (`<common-dir>/worktrees/<name>`) as protected
/// in its own right, not merely inherited from a parent directory's grant.
/// Reproduced end-to-end through a real Work (`sgt run` against a scratch
/// estate on the `codex-terra` profile, app-server transport): `git commit`
/// fails closed with `Read-only file system` on this exact admin
/// subdirectory's `index.lock` even though the common dir that contains it
/// was granted. Granting this directory *in addition to* the common dir —
/// both, not either — resolves it; that combination is what this function
/// adds to `launch_appserver`'s `runtimeWorkspaceRoots`, on top of (never
/// instead of) [`git_worktree_common_dirs`].
fn git_worktree_admin_dirs(bindings: &[BindingSummary]) -> Result<Vec<PathBuf>, String> {
    bindings
        .iter()
        .map(|binding| {
            worktree_git_admin_dir(&binding.worktree_path).map_err(|reason| {
                format!(
                    "{} ({}): {reason}",
                    binding.repository,
                    binding.worktree_path.display()
                )
            })
        })
        .collect()
}

/// The named, actionable refusal text for #259's fail-closed rule: a
/// mutation-shaped launch (one or more bindings) whose git common-dir grant
/// cannot be resolved is refused rather than admitted to run and later
/// retire `completed_dirty` with no durable commit.
///
/// Ends in a remedy sentence naming what the operator should actually do
/// about it — `src/runtime/preflight.rs`'s `PreflightCheck::remedy` and
/// `src/cli.rs`'s `Check::warn` precedent (§15's "named remedy": a refusal
/// that only says what failed, with nothing to do about it, is half the
/// contract). Every reason this function is ever called with traces back to
/// the same root cause — `worktree_path` is not a linked worktree
/// `runtime::surface` actually created (its `.git` file is missing, or does
/// not name a `gitdir`) — so one remedy sentence covers every case, rather
/// than inventing a different one per parse failure.
fn git_admin_dir_refusal(reason: &str) -> String {
    format!(
        "cannot grant this Work's git common directory ({reason}); a codex actor needs write \
         access to its own worktree's shared git directory to `git add`/`git commit` (sergeant \
         issue #259) — refusing rather than admitting a Work that can edit files but can never \
         commit them. Remedy: this worktree path was not created by a real `git worktree add` \
         (or its admin state was since removed) — re-admit the repository through sergeant's \
         own surface admission so a genuine linked worktree exists at this path, rather than \
         pointing a binding at a hand-placed or foreign directory; sergeant never repairs one \
         automatically."
    )
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
    /// **App-server only** (W3 spec §2.2): `turn/completed` itself carried
    /// `turn.status == "interrupted"` — never constructed by exec's own
    /// `ingest_line` (exec's stream has no wire shape that means this).
    Interrupted,
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
    /// `turn.completed`'s usage object, verbatim, when seen. On app-server
    /// this is populated from `thread/tokenUsage/updated` instead (§2.3) —
    /// same field, two producers, one reader (OBSERVE's evidence string).
    usage: Option<Value>,
    /// This turn's terminal, if any.
    terminal: Terminal,
    /// **App-server only**: method names archived-but-not-decoded at the
    /// envelope level (`remoteControl/status/changed`,
    /// `mcpServer/startupStatus/updated`, and the remaining ~60) — the same
    /// "counted, never decoded" posture `unknown_items` gives exec's
    /// unrecognized item types, just keyed by method instead of item type
    /// since these notifications carry no item at all.
    unknown_methods: Vec<String>,
    /// **App-server only**: the most recent `turn.error.codexErrorInfo` (or
    /// the standalone `error` notification's), when the harness supplied
    /// one (§2.8). Kept separate from [`Terminal::Failed`]'s `message` field
    /// rather than added to it, so exec's identical-looking `Failed{message}`
    /// match arms elsewhere in this module need no change for a fact only
    /// this transport can ever populate.
    last_codex_error_info: Option<String>,
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
        self.ingest_view(&exec_item_view(item), started, out);
    }

    /// The transport-agnostic half of item decoding (spec §1.6): everything
    /// below reads only [`ItemView`], never a raw `Value` — so a second
    /// transport needs nothing but its own `fn view(&Value) -> Option<
    /// ItemView>` (here, always-`Some`; app-server's sibling in
    /// `codex_appserver.rs` is the same shape) to reuse every line of this
    /// function unchanged. This is the seam the W3 spec calls out by name:
    /// "the event-producing code — which `NativeEvent` kinds, which payload
    /// fields, the `is_error` rule, the `TOOL_OUTPUT_TAIL` truncation —
    /// exists once."
    fn ingest_view(&mut self, view: &ItemView, started: bool, out: &mut Vec<NativeEvent>) {
        match &view.item_type {
            ItemKind::AgentMessage => {
                // No `item.started` for this type was ever observed on exec;
                // emitting a partial assistant message on an unmeasured shape
                // would double-count the completed one (§4.2). App-server
                // *does* emit `item/started`/`item/agentMessage/delta` for
                // this type, and §1.6 deliberately does not decode either —
                // the caller (`ingest_appserver_notification`) never routes
                // a started `agentMessage` or a delta line through this
                // function at all, so `started` here is always `false` for
                // this variant on both transports today; the guard stays as
                // the structural invariant it always was.
                if started {
                    return;
                }
                self.message_items += 1;
                let text = view.text.clone().unwrap_or_default();
                self.last_agent_message = Some(text.clone());
                out.push(NativeEvent {
                    kind: KIND_CONVERSATION_ASSISTANT_COMPLETED.to_string(),
                    payload: json!({"thread_id": self.thread_id, "text": text, "item_id": view.item_id}),
                });
            }
            // §4.3: the *only* code path that produces `tool.*` events.
            // Nothing anywhere else in this decoder reads `agent_message`
            // text as evidence that a command ran.
            ItemKind::CommandExecution => {
                if started {
                    out.push(NativeEvent {
                        kind: KIND_TOOL_REQUESTED.to_string(),
                        payload: json!({
                            "id": view.item_id,
                            "name": "command_execution",
                            "input": {"command": view.command.clone().unwrap_or(Value::Null)},
                        }),
                    });
                    return;
                }
                self.tool_items += 1;
                let exit_code = view.exit_code.clone().unwrap_or(Value::Null);
                let status = view.status.clone().unwrap_or_default();
                let is_error = exit_code.as_i64() != Some(0) || status != "completed";
                let output = view.aggregated_output.as_deref().unwrap_or("");
                out.push(NativeEvent {
                    kind: KIND_TOOL_COMPLETED.to_string(),
                    payload: json!({
                        "tool_use_id": view.item_id,
                        "is_error": is_error,
                        "exit_code": exit_code,
                        "status": status,
                        "output_tail": truncate(output, TOOL_OUTPUT_TAIL),
                    }),
                });
            }
            ItemKind::Error => {
                // Unobserved on `item.started`; the one measured instance was
                // a warning on a turn that continued, never a terminal.
                // App-server has no item-level `error` type at all (M9's 18
                // variants do not include one) — this arm is exec-only in
                // practice, kept on the shared type because nothing about it
                // is exec-specific.
                if !started {
                    let message = view.text.clone().unwrap_or_default();
                    out.push(NativeEvent {
                        kind: KIND_TURN_HARNESS_ERROR.to_string(),
                        payload: json!({"phase": "item_error", "message": message, "item_id": view.item_id}),
                    });
                }
            }
            ItemKind::Unknown(other) => {
                self.unknown_items.push(other.clone());
            }
        }
    }
}

/// Which of the decoder's four branches an item belongs to (spec §1.6). Every
/// one of M9's other 14 app-server variants — `plan`, `reasoning`,
/// `fileChange`, `mcpToolCall`, ... — lands in `Unknown`, named by its own
/// wire string, and is counted, never decoded; so does every unrecognized
/// exec item type.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ItemKind {
    AgentMessage,
    CommandExecution,
    /// Exec-only (§4.4): a bare `{"type":"error", ...}` item. App-server has
    /// no item-level error variant among M9's 18.
    Error,
    /// Carries the item's own wire-level type string, for `unknown_items`.
    Unknown(String),
}

/// One item, reduced to exactly the fields the decoder branches on (spec
/// §1.6) — nothing here is transport-specific; each transport's own `fn
/// view(&Value) -> ItemView` (`exec_item_view` below, `codex_appserver::
/// appserver_item_view`) is the only place that ever reads a raw item's key
/// spellings.
#[derive(Debug, Clone)]
struct ItemView {
    item_type: ItemKind,
    /// The item's own id, whatever shape it takes on the wire (always a
    /// string on both transports, but carried as `Value` so a missing id
    /// serializes as `null` rather than an invented string).
    item_id: Value,
    /// `agent_message.text` (exec) / `agentMessage.text` (app-server).
    text: Option<String>,
    /// `command_execution.command` (exec) / `commandExecution.command`
    /// (app-server), carried as `Value` since a future shape could be
    /// structured rather than a bare string and this decoder never
    /// interprets it — only the event payload does.
    command: Option<Value>,
    /// `exit_code` (exec) / `exitCode` (app-server).
    exit_code: Option<Value>,
    /// `status` — same key on both transports.
    status: Option<String>,
    /// `aggregated_output` (exec) / `aggregatedOutput` (app-server).
    aggregated_output: Option<String>,
}

impl ItemView {
    fn unknown(kind: impl Into<String>, item_id: Value) -> Self {
        Self {
            item_type: ItemKind::Unknown(kind.into()),
            item_id,
            text: None,
            command: None,
            exit_code: None,
            status: None,
            aggregated_output: None,
        }
    }
}

/// Exec's own `fn view` (spec §1.6): `snake_case` keys, and an `"error"` item
/// variant app-server does not have.
fn exec_item_view(item: &Value) -> ItemView {
    let item_id = item.get("id").cloned().unwrap_or(Value::Null);
    match item.get("type").and_then(Value::as_str).unwrap_or("") {
        "agent_message" => ItemView {
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
        "command_execution" => ItemView {
            item_type: ItemKind::CommandExecution,
            item_id,
            text: None,
            command: Some(item.get("command").cloned().unwrap_or(Value::Null)),
            exit_code: Some(item.get("exit_code").cloned().unwrap_or(Value::Null)),
            status: Some(
                item.get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            ),
            aggregated_output: Some(
                item.get("aggregated_output")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            ),
        },
        "error" => ItemView {
            item_type: ItemKind::Error,
            item_id,
            text: Some(
                item.get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            ),
            command: None,
            exit_code: None,
            status: None,
            aggregated_output: None,
        },
        other => ItemView::unknown(other, item_id),
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
    /// about the stage, the conversation stays resumable. Always *inferred*
    /// from "we asked and nothing else arrived", never harness-confirmed —
    /// and `via` records which of the three ways that inference was reached,
    /// because OBSERVE's evidence string must name the one that actually
    /// happened rather than the one that is easiest to write down.
    InterruptedRunning { via: InterruptedVia },
    /// **App-server only** (W3 spec §2.2): `turn/completed` itself carried
    /// `turn.status == "interrupted"` — the harness *told* sergeant the turn
    /// was interrupted, a first-class, harness-confirmed terminal, distinct
    /// from [`TerminalOutcome::InterruptedRunning`]'s inference. Exec's
    /// decoder never produces this arm: `Terminal` (exec's own accumulator
    /// field) has no `Interrupted` variant, because exec's stream has no
    /// wire shape that means it.
    Interrupted,
    /// No terminal arrived and nobody asked for that: §5.2's ambiguity,
    /// fails closed.
    AmbiguousUnknown,
}

/// How an [`TerminalOutcome::InterruptedRunning`] was arrived at. The three
/// arms are three different *facts*, and the one OBSERVE reports has to be
/// the one that happened: a single hardcoded sentence covering all three
/// necessarily lies about two of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InterruptedVia {
    /// Exec transport: there is no interrupt RPC to confirm anything with.
    /// Sergeant killed the turn's process group and nothing else arrived.
    ProcessKill,
    /// App-server: `turn/interrupt` itself failed, and the adapter fell back
    /// to the process-group kill (§2.2's downgrade).
    RpcFailed,
    /// App-server: `turn/interrupt` returned `Ok` — the harness accepted the
    /// request — but the child's stdout closed before `turn/completed` could
    /// carry the harness's own verdict. The routine STOP path lands here.
    ClosedAfterAcknowledgedRpc,
    /// App-server: the stream ended while `turn/interrupt` was still
    /// outstanding — no answer to it was ever written — so neither of the two
    /// above is known to be what happened. The honest third answer, and a
    /// narrow one: an answer that *did* arrive is recorded on the reader
    /// thread as it is decoded (`InboundLine::Resolved`), ahead of the EOF
    /// that settles the turn, so "the child answered and then died" never
    /// lands here.
    RpcUnresolved,
}

/// How far `turn/interrupt` has got for the turn currently in flight. Kept
/// on the cell (not inferred at settlement) because the settlement can
/// happen on the reader thread at any moment, including between the RPC
/// being sent and its answer arriving.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum InterruptProgress {
    /// Nobody has asked for this turn to stop.
    #[default]
    NotRequested,
    /// `turn/interrupt` is in flight under this JSON-RPC id; it has not
    /// resolved either way. The id is carried so that the *reader* thread can
    /// recognise the answer when it decodes it and mark the outcome there —
    /// the only thread whose marks are guaranteed to precede the EOF that
    /// settles the turn.
    Requested { id: u64 },
    /// The harness answered `turn/interrupt` with a result.
    Acknowledged,
    /// `turn/interrupt` itself failed; the process-group kill is the
    /// fallback. **Written before the kill is issued**, never after: the
    /// kill is what ends the child's stdout, and the EOF that follows is
    /// what settles the turn, so a mark made afterwards would routinely
    /// arrive at a cell that had already recorded the wrong path.
    RpcFailed,
}

impl InterruptProgress {
    /// Record what the reader thread just decoded as the answer to request
    /// `id`, but only if that is the request this cell is actually waiting on.
    /// Anything else — a different request's answer, a turn whose interrupt
    /// already resolved, a turn nobody interrupted — leaves the cell alone.
    fn resolve(&mut self, id: u64, ok: bool) {
        if matches!(*self, InterruptProgress::Requested { id: mine } if mine == id) {
            *self = if ok {
                InterruptProgress::Acknowledged
            } else {
                InterruptProgress::RpcFailed
            };
        }
    }

    /// Whether this turn's interrupt is still worth actually asking the
    /// harness for: only `NotRequested` (nobody has asked) or `RpcFailed`
    /// (the last ask never got an honest answer, one way or the other) mean
    /// that. `Requested { .. }` (an ask is already outstanding) and
    /// `Acknowledged` (an ask already got the honest answer) both mean a
    /// repeat call has nothing to add -- and, just as importantly, must not
    /// overwrite either of them with a fresh, untracked `Requested`: doing so
    /// is how a turn the harness genuinely acknowledged could end up
    /// settling as if nothing had ever answered it, merely because *this*
    /// call's own redundant RPC went unanswered.
    fn worth_asking_again(&self) -> bool {
        matches!(
            self,
            InterruptProgress::NotRequested | InterruptProgress::RpcFailed
        )
    }
}

/// Fold one turn's stream evidence plus the interrupt request, if any, into
/// its outcome. `interrupt` is `None` when nobody asked for the kill; when
/// somebody did, it names *how* the asking went, which is the only thing
/// that separates the two honest sentences for the same outcome.
fn classify_terminal(acc: &TurnAccumulator, interrupt: Option<InterruptedVia>) -> TerminalOutcome {
    match &acc.terminal {
        Terminal::Completed => TerminalOutcome::Completed,
        Terminal::Failed { message } => TerminalOutcome::Failed {
            message: message.clone(),
        },
        // App-server only (§2.2): the harness itself said the turn ended
        // interrupted, whether or not sergeant asked — a first-class
        // terminal, never inferred the way `InterruptedRunning` below is.
        Terminal::Interrupted => TerminalOutcome::Interrupted,
        Terminal::None => match interrupt {
            Some(via) => TerminalOutcome::InterruptedRunning { via },
            None => TerminalOutcome::AmbiguousUnknown,
        },
    }
}

/// One settled outcome's stable, snake_case name — the string the journal
/// carries, kept out of `{:?}` so a payload consumer is not reading a
/// derived Debug rendering that changes shape whenever a field is added.
fn terminal_outcome_label(outcome: &TerminalOutcome) -> &'static str {
    match outcome {
        TerminalOutcome::Completed => "completed",
        TerminalOutcome::Failed { .. } => "failed",
        TerminalOutcome::InterruptedRunning { .. } => "interrupted_running",
        TerminalOutcome::Interrupted => "interrupted",
        TerminalOutcome::AmbiguousUnknown => "ambiguous_unknown",
    }
}

/// Everything one settled app-server turn owes its journal and OBSERVE.
/// Returned by [`settle_appserver_turn`] so that a caller which settles a
/// turn cannot end up holding *less* than the `turn/completed` path holds:
/// the fail-closed routes used to drop the accumulator on the floor and
/// journal a bare `harness_error`, which is how "every turn ends with
/// `conversation.turn.ended`, however it ended" became true of one route and
/// false of the others.
#[derive(Debug, Clone)]
struct AppServerSettlement {
    outcome: TerminalOutcome,
    thread_id: Option<String>,
    interrupted: bool,
    message_items: u32,
    tool_items: u32,
    unknown_items: Vec<String>,
    unknown_methods: Vec<String>,
    usage: Option<Value>,
}

impl AppServerSettlement {
    /// This settlement's `conversation.turn.ended` payload — one shape for
    /// every route, so the fail-closed ones cannot drift from the happy one.
    /// No `raw`/`raw_error` keys: unlike exec, this transport archives no
    /// per-turn blob (there is no per-turn stream to archive — one child
    /// carries every turn), and inventing a null-valued key would read as
    /// "the archive failed" rather than "there is no archive here".
    fn turn_ended_payload(&self) -> Value {
        json!({
            "thread_id": self.thread_id,
            "interrupted": self.interrupted,
            "outcome": terminal_outcome_label(&self.outcome),
            "message_items": self.message_items,
            "tool_items": self.tool_items,
            "unknown_items": self.unknown_items,
            "unknown_methods": self.unknown_methods,
        })
    }
}

/// The **one** place an app-server turn ever becomes `Finished`. Takes an
/// already-locked cell; settles it if (and only if) it is `InFlight`, and
/// hands back the facts both the journal and OBSERVE read from. `None` means
/// there was nothing in flight to settle, which is what makes every caller
/// idempotent and safe to race: the reader thread's own EOF handling, the
/// `turn/completed` handler, and `interrupt_appserver`'s RPC-failure
/// fallback all funnel through here, and the first one to arrive decides.
fn settle_appserver_turn(state: &mut AppServerTurnState) -> Option<AppServerSettlement> {
    let AppServerTurnState::InFlight { interrupt, .. } = &*state else {
        return None;
    };
    let interrupted = *interrupt != InterruptProgress::NotRequested;
    let via = match interrupt {
        InterruptProgress::NotRequested => None,
        InterruptProgress::Requested { .. } => Some(InterruptedVia::RpcUnresolved),
        InterruptProgress::Acknowledged => Some(InterruptedVia::ClosedAfterAcknowledgedRpc),
        InterruptProgress::RpcFailed => Some(InterruptedVia::RpcFailed),
    };
    let taken = std::mem::replace(state, AppServerTurnState::Idle);
    let AppServerTurnState::InFlight { acc, turn_id, .. } = taken else {
        unreachable!("just matched InFlight above");
    };
    let outcome = classify_terminal(&acc, via);
    *state = AppServerTurnState::Finished {
        turn_id,
        outcome: outcome.clone(),
        last_agent_message: acc.last_agent_message.clone(),
        message_items: acc.message_items,
        tool_items: acc.tool_items,
        unknown_items: acc.unknown_items.clone(),
        unknown_methods: acc.unknown_methods.clone(),
        last_codex_error_info: acc.last_codex_error_info.clone(),
        last_error: acc.last_error.clone(),
        late: LateEvidence::default(),
    };
    Some(AppServerSettlement {
        outcome,
        thread_id: acc.thread_id,
        interrupted,
        message_items: acc.message_items,
        tool_items: acc.tool_items,
        unknown_items: acc.unknown_items,
        unknown_methods: acc.unknown_methods,
        usage: acc.usage,
    })
}

/// Fail closed (§3.4 point 3 / §15's invariant): if a turn is `InFlight` with
/// no confirmation it ever reached a terminal, settle it — through the same
/// [`settle_appserver_turn`] the `turn/completed` handler uses, so a turn
/// that was interrupted (even by the process-group-kill fallback when
/// `turn/interrupt` itself failed) still resolves to `InterruptedRunning`
/// rather than `AmbiguousUnknown`, exactly the distinction exec's own decoder
/// draws. Returns the settlement, or `None` if there was no in-flight turn to
/// close — idempotent, so it is safe to call from more than one place without
/// double-finalizing: the reader thread's own EOF handling and
/// `interrupt_appserver`'s RPC-failure fallback both call this, because
/// killing the child's process group ends its stdout on its own accord too,
/// and the two must not race each other into two different outcomes.
fn fail_closed_appserver_turn(
    turn_cell: &Mutex<AppServerTurnState>,
) -> Option<AppServerSettlement> {
    let mut state = turn_cell.lock().expect("appserver turn lock");
    settle_appserver_turn(&mut state)
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

/// Which transport one execution runs on, and that transport's own runtime
/// (W3 spec §1.4/§5). `Exec`-transport executions keep using
/// [`CodexExecution`]'s own `turn`/`turn_pgid`/`reader` fields exactly as
/// before this wave; app-server executions carry their entire runtime here
/// instead, because the two transports' lifecycles do not share a shape
/// (one process per turn vs. one child for the whole execution).
#[derive(Debug)]
enum CodexTransportState {
    Exec,
    AppServer(Arc<AppServerRuntime>),
}

/// One app-server execution's runtime: the long-lived child (§1.4) plus its
/// current-or-last turn, mutated by the reader thread's own callback
/// ([`appserver_on_line`]) and read from OBSERVE/SEND/INTERRUPT on any
/// thread.
#[derive(Debug)]
struct AppServerRuntime {
    child: Mutex<codex_appserver::AppServerChild>,
    /// `Arc`-wrapped rather than a bare `Mutex` because the reader thread's
    /// own callback (`appserver_on_line`, `'static`, spawned before this
    /// struct exists) holds an independent clone of the same cell — the
    /// callback is what constructs the very first `AppServerTurnState`, one
    /// step before `AppServerRuntime` itself is assembled.
    turn: Arc<Mutex<AppServerTurnState>>,
    /// `thread/start`'s own result, verbatim (§3.1/§3.6) — set once at
    /// LAUNCH and never mutated again, so no lock is needed to read it.
    /// Exists so a test can assert the wire evidence a `claimed: true`
    /// `sandbox_enforcement`/`model_selection` row is credited with, the
    /// same diagnostic-seam posture as [`CodexBackend::tracked_executions`].
    policy_echo: Value,
    /// Mints [`AppServerTurnState::InFlight::epoch`]. Monotonic for this
    /// runtime's whole life, never reset, so no two `turn/start` attempts
    /// can ever mistake each other's cell for their own.
    turn_epochs: AtomicU64,
}

/// One execution's current-or-last turn on the app-server transport.
#[derive(Debug)]
enum AppServerTurnState {
    /// No turn has ever been sent. Transient in practice — LAUNCH always
    /// sends turn 1 as part of the same call that registers the execution —
    /// kept as a real variant rather than assumed away, the same posture
    /// `TurnState::Unlaunched` takes on the exec side.
    Idle,
    /// A turn is in flight. `turn_id` is briefly empty (spec's own ordering
    /// caveat, §1.5.1: a notification may race ahead of `turn/start`'s own
    /// response) — the accumulator exists and accepts notifications from the
    /// moment this variant is set, *before* `turn/start` is even written, so
    /// nothing racing ahead of the id is ever silently dropped.
    InFlight {
        turn_id: String,
        acc: TurnAccumulator,
        /// How far this turn's `turn/interrupt` has got — the request bit
        /// and the RPC's fate in one field, recorded as each step happens so
        /// the settlement can name the path rather than guess at it.
        interrupt: InterruptProgress,
        /// Which `turn/start` attempt owns this cell. `appserver_send_turn`
        /// stamps it before writing the request and compares it before
        /// touching the cell again: by the time a failed `turn/start`
        /// returns, the cell may already have been settled by the reader
        /// thread (a dead child) or replaced by a later turn, and a blind
        /// write-back would erase a terminal the adapter had already
        /// journaled. Never reused: a monotonic counter per runtime.
        epoch: u64,
        /// The turn id [`AppServerRuntime::turn`] held just before this one
        /// replaced it, if that turn ever got one (`""` when the previous
        /// state was [`AppServerTurnState::Idle`], or a settled turn whose
        /// `turn/start` never got far enough to learn its id). Lets a
        /// notification that names this id, arriving after this cell already
        /// moved on, be recognised as a straggler from the turn *this one
        /// displaced* rather than folded into `acc` as if it were this turn's
        /// own evidence -- the failure mode the "displacement" tests only
        /// cover for a straggler that arrives *before* the displacement,
        /// never one that arrives during the new turn's own `InFlight`
        /// window.
        superseded_turn_id: String,
    },
    /// The last turn's terminal, plus the evidence OBSERVE reports.
    Finished {
        /// This turn's own id, carried forward so a later `InFlight` that
        /// displaces this `Finished` can name it as `superseded_turn_id` and
        /// keep recognising its stragglers even after the cell moves on.
        turn_id: String,
        outcome: TerminalOutcome,
        last_agent_message: Option<String>,
        message_items: u32,
        tool_items: u32,
        unknown_items: Vec<String>,
        unknown_methods: Vec<String>,
        last_codex_error_info: Option<String>,
        /// The last stream `error`/warning message this turn carried (§4.4),
        /// kept for the ambiguous terminal's evidence: it is often the only
        /// thing that says *why* the turn never reached one.
        last_error: Option<String>,
        /// What arrived on this thread after the turn was already settled.
        late: LateEvidence,
    },
}

/// Notifications that arrived for a turn that was already settled (§3.4's
/// ordering caveat, from the other end): the settlement is final — nothing
/// here ever re-decides an outcome — but a line that arrives late is still
/// evidence, and dropping it silently is how a genuine `turn/completed`
/// buffered behind a fail-closed settlement used to vanish without trace.
#[derive(Debug, Clone, Default)]
struct LateEvidence {
    /// How many post-settlement notifications arrived.
    lines: u32,
    /// Which methods, deduplicated and capped — a summary, not a second
    /// unbounded journal.
    methods: Vec<String>,
    /// Whether a real terminal (`turn/completed`) was among them.
    terminal_seen: bool,
    /// Whether that terminal has already been journaled on arrival, so the
    /// immediate report fires once rather than once per late terminal.
    terminal_reported: bool,
}

/// How many distinct late method names are kept before the summary stops
/// growing (`lines` keeps counting either way).
const LATE_EVIDENCE_METHOD_CAP: usize = 8;

impl LateEvidence {
    /// Record one post-settlement notification. Returns the payload of the
    /// report that must be journaled *now* rather than at end of stream —
    /// only for the first real terminal, the one line whose silent loss
    /// would mean the journal disagrees with what the harness actually said.
    fn record(&mut self, method: &str, settled_as: &TerminalOutcome) -> Option<Value> {
        self.lines = self.lines.saturating_add(1);
        if self.methods.len() < LATE_EVIDENCE_METHOD_CAP
            && !self.methods.iter().any(|seen| seen == method)
        {
            self.methods.push(method.to_string());
        }
        if method != "turn/completed" {
            return None;
        }
        self.terminal_seen = true;
        if self.terminal_reported {
            return None;
        }
        self.terminal_reported = true;
        Some(json!({
            "phase": "post_settlement_terminal",
            "method": method,
            "settled_as": terminal_outcome_label(settled_as),
            "detail": "a real terminal arrived after this turn was already settled; \
                       the settled outcome stands (a turn is never re-settled) and the \
                       terminal is recorded here rather than dropped",
        }))
    }

    /// The end-of-stream summary, or `None` when nothing arrived late.
    fn summary(&self) -> Option<Value> {
        (self.lines > 0).then(|| {
            json!({
                "phase": "post_settlement_lines",
                "lines": self.lines,
                "methods": self.methods,
                "terminal_seen": self.terminal_seen,
                "capped": self.methods.len() >= LATE_EVIDENCE_METHOD_CAP,
            })
        })
    }
}

/// Owned context for the app-server reader-thread callback (spec §1.4):
/// exactly what `TurnReader` owns for the exec transport, minus everything
/// that lives on `AppServerRuntime` instead (the callback is `'static` and
/// cannot borrow `&CodexBackend`).
struct AppServerLineContext {
    sink: Option<EventSink>,
    execution_id: String,
    work_id: String,
    model: Option<String>,
}

impl AppServerLineContext {
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

/// The turn identity one app-server notification's envelope carries, when it
/// carries one at all (spec §1.6's wire table). `turn/completed` nests it
/// under `turn.id`; every other per-turn notification (`item/started`,
/// `item/completed`, `thread/tokenUsage/updated`, the standalone `error`,
/// ...) carries a flat `turnId` alongside its `threadId`. `thread/started`
/// and the connection-level warning methods carry neither, and `None` here
/// is read as "cannot be identified as a straggler" rather than "definitely
/// belongs to the current turn" — the existing accept-when-unsure posture
/// [`AppServerTurnState::InFlight`]'s own doc describes for the empty-
/// `turn_id` window is left exactly as permissive as it was.
fn notification_turn_id<'a>(method: &str, params: &'a Value) -> Option<&'a str> {
    if method == "turn/completed" {
        params.pointer("/turn/id").and_then(Value::as_str)
    } else {
        params.get("turnId").and_then(Value::as_str)
    }
}

/// The reader-thread callback every app-server child runs (spec §1.4/§3.4):
/// decodes notifications through the shared [`TurnAccumulator`], emits their
/// events immediately, finalizes the turn on `turn/completed`, and answers
/// every server request — the non-hang guarantee, kept alive on the same
/// thread that saw the request, never deferred to a second round trip.
fn appserver_on_line(
    ctx: &AppServerLineContext,
    turn_cell: &Mutex<AppServerTurnState>,
    handle: &codex_appserver::AppServerHandle,
    line: codex_appserver::InboundLine,
) {
    match line {
        codex_appserver::InboundLine::Notification { method, params } => {
            let mut state = turn_cell.lock().expect("appserver turn lock");
            let mut late_report = None;
            let mut stray = false;
            let events = match &mut *state {
                AppServerTurnState::InFlight {
                    acc,
                    turn_id,
                    superseded_turn_id,
                    ..
                } => {
                    // A notification that names the turn this cell's own
                    // `turn/start` *displaced* — never this cell's own turn,
                    // and only when that displaced id is actually known — is
                    // a straggler from the previous turn arriving inside the
                    // new one's `InFlight` window, not evidence about the new
                    // turn. Folding it into `acc` used to be how a stray
                    // `turn/completed{status:"failed"}` for the old turn
                    // could stamp `Terminal::Failed` onto a turn that never
                    // failed; recognising it here instead of unconditionally
                    // ingesting keeps this turn's own accumulator honest.
                    stray = !superseded_turn_id.is_empty()
                        && notification_turn_id(&method, &params)
                            .is_some_and(|id| id == superseded_turn_id && id != turn_id.as_str());
                    if stray {
                        Vec::new()
                    } else {
                        acc.ingest_appserver_notification(&method, &params)
                    }
                }
                // §3.4, from the far end: this turn is already settled, and
                // nothing that arrives now re-decides it. What it must not do
                // is disappear — a buffered `turn/completed` behind a
                // fail-closed settlement is exactly the line whose silent
                // loss makes the journal disagree with the harness.
                AppServerTurnState::Finished { late, outcome, .. } => {
                    late_report = late.record(&method, outcome);
                    Vec::new()
                }
                // No turn has ever been started on this thread, or the last
                // `turn/start` failed before the harness acknowledged one.
                // There is no turn for these lines to be evidence *about*,
                // which is why they are not tallied the way a settled turn's
                // late lines are.
                AppServerTurnState::Idle => Vec::new(),
            };
            for event in &events {
                ctx.emit(&event.kind, event.payload.clone());
            }
            // A stray `turn/completed` for the displaced turn must not
            // settle *this* one — that would be the ceiling case above, one
            // step further: the new turn's evidence has already been kept
            // clean of it, and letting it trigger settlement anyway would
            // finalize a turn that is still genuinely running.
            let settlement = (method == "turn/completed" && !stray)
                .then(|| settle_appserver_turn(&mut state))
                .flatten();
            drop(state);
            if stray {
                ctx.emit(
                    KIND_TURN_HARNESS_ERROR,
                    json!({
                        "phase": "stray_notification_for_superseded_turn",
                        "method": method,
                    }),
                );
            }
            if let Some(payload) = late_report {
                ctx.emit(KIND_TURN_HARNESS_ERROR, payload);
            }
            if let Some(settlement) = settlement {
                ctx.emit(
                    KIND_CONVERSATION_TURN_ENDED,
                    settlement.turn_ended_payload(),
                );
                if let Some(usage) = &settlement.usage {
                    ctx.emit(
                        KIND_USAGE_UPDATED,
                        json!({
                            "thread_id": settlement.thread_id,
                            "usage": usage,
                            "model_pin": model_pin_evidence(ctx.model.as_deref()),
                        }),
                    );
                }
            }
        }
        codex_appserver::InboundLine::ServerRequest { id, method, params } => {
            // §3.4: every server request is answered, without exception —
            // `ask` stays structurally `false` (see `ADMISSION_ROWS`), so
            // this is always the "otherwise" branch of §2.4's conditional.
            let answered = codex_appserver::answer_for_request(&method, false);
            let _ = handle.answer(&id, answered.answer);
            if let Some(phase) = answered.journal_phase {
                let mut payload = json!({"phase": phase, "method": method});
                if method == "item/tool/requestUserInput" {
                    // §2.4 step 3's own wire evidence, carried into the
                    // journal even though `ask` stays declined here: the
                    // live admission probe (`live_appserver_actor_authored_
                    // question_is_typed`) reads these back to prove the
                    // request it caught actually named a non-empty question
                    // set for the in-flight turn, not just that *a* request
                    // arrived.
                    payload["questions"] = params.get("questions").cloned().unwrap_or(Value::Null);
                    payload["turn_id"] = params.get("turnId").cloned().unwrap_or(Value::Null);
                }
                ctx.emit(KIND_TURN_HARNESS_ERROR, payload);
            }
        }
        // One of this client's own requests has been answered. The only one
        // whose fate the turn cell tracks is `turn/interrupt`, and it is
        // tracked *here* rather than where the answer is awaited because the
        // awaiting thread is woken by a channel and then has to race this one
        // for the cell — a race it loses on exactly the child this matters
        // for, one that answers the interrupt and dies in the same breath.
        codex_appserver::InboundLine::Resolved { id, ok } => {
            if let AppServerTurnState::InFlight { interrupt, .. } =
                &mut *turn_cell.lock().expect("appserver turn lock")
            {
                interrupt.resolve(id, ok);
            }
        }
        codex_appserver::InboundLine::Unparsed => {}
        codex_appserver::InboundLine::Eof => {
            if let Some(settlement) = fail_closed_appserver_turn(turn_cell) {
                ctx.emit(
                    KIND_TURN_HARNESS_ERROR,
                    json!({
                        "phase": "child_exited_mid_turn",
                        "outcome": terminal_outcome_label(&settlement.outcome),
                    }),
                );
                // §3.5's invariant, which exec's own finalizer states in
                // one line ("every turn ends with this event, however it
                // ended") and which this route used to be the exception to.
                ctx.emit(
                    KIND_CONVERSATION_TURN_ENDED,
                    settlement.turn_ended_payload(),
                );
                if let Some(usage) = &settlement.usage {
                    ctx.emit(
                        KIND_USAGE_UPDATED,
                        json!({
                            "thread_id": settlement.thread_id,
                            "usage": usage,
                            "model_pin": model_pin_evidence(ctx.model.as_deref()),
                        }),
                    );
                }
            }
            // End of stream is when the last settled turn's post-settlement
            // tally is final, so it is where that turn's summary goes. The
            // other exit is `appserver_send_turn`, for a turn whose cell a
            // later turn overwrites before the stream ever ends.
            let summary = match &*turn_cell.lock().expect("appserver turn lock") {
                AppServerTurnState::Finished { late, .. } => late.summary(),
                AppServerTurnState::Idle | AppServerTurnState::InFlight { .. } => None,
            };
            if let Some(summary) = summary {
                ctx.emit(KIND_TURN_HARNESS_ERROR, summary);
            }
        }
    }
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
    /// #259: every binding's own git common (shared `.git`) directory
    /// ([`git_worktree_common_dirs`]), resolved once at LAUNCH and granted as
    /// additional `--add-dir` roots on turn 1 — the write access a linked
    /// worktree's `git add`/`git commit` needs that `worktree_path` alone
    /// does not cover. Wider than just the per-worktree admin subdirectory;
    /// see [`worktree_git_common_dir`] for why.
    git_worktree_common_dirs: Vec<PathBuf>,
    /// W3 §3.3: the sandbox policy this execution's turns compose. Resolved
    /// once at LAUNCH/RESUME from `CodexConfig.sandbox` and carried here so
    /// SEND's later turns compose the same policy the first one did (never
    /// re-read from a config that might have changed under a long-lived
    /// daemon).
    sandbox: SandboxChoice,
    /// #262: this execution's resolved `network_access`
    /// (`CodexBackend::launch_config` — the profile's own option if set,
    /// else `CodexConfig.workspace_write_network_access`), resolved once at
    /// LAUNCH for the same reason `sandbox` is — never re-read per turn.
    network_access: bool,
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
    /// Which transport this execution runs on (W3 §5). `Exec` for every
    /// execution this wave's predecessor could produce; `AppServer` only
    /// when this registration resolved to that transport (§5.2/§5.3) — a
    /// re-adopted execution (RESUME) is always `Exec`, even if it was
    /// originally launched on app-server (§5.4).
    transport_state: CodexTransportState,
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
    /// The four app-server admission gates (W3 spec §5.2), memoized exactly
    /// as `probe_outcome` is: G4 spawns a real (bounded, token-free) process,
    /// so this happens once per daemon lifetime, not once per PROBE call.
    appserver_gates: OnceLock<AppServerGates>,
    /// Which transport this registration resolved to (§5.2/§5.3) — computed
    /// once, journaled, and never revisited per execution.
    transport_resolution: OnceLock<TransportResolution>,
    /// `CodexConfig.output_schema`, materialized to a file under `data_dir`
    /// once (§4.2) — every execution this backend launches on the exec
    /// transport shares the one file, since the schema is backend-config-
    /// level, not per-execution.
    output_schema_path: OnceLock<Option<PathBuf>>,
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
            appserver_gates: OnceLock::new(),
            transport_resolution: OnceLock::new(),
            output_schema_path: OnceLock::new(),
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

    /// `thread/start`'s own result, verbatim, for an execution running (or
    /// that ran) on the app-server transport — `None` on exec, where no such
    /// echo exists, and `None` for an execution id this adapter does not
    /// hold. The diagnostic seam spec §3.6's live admission test needs: the
    /// wire evidence a `sandbox_enforcement`/`model_selection` row is
    /// credited with (`result.sandbox.type`, `.writableRoots`, `.cwd`,
    /// `.approvalPolicy`, `.model`) is otherwise unreachable from outside
    /// this module, since `codex_appserver::AppServerChild` stays
    /// `pub(super)` (§1.5's own visibility rule) rather than being widened
    /// just so a test could hold one directly.
    pub fn appserver_policy_echo(&self, handle: &ExecutionHandle) -> Option<Value> {
        let state = self.lock();
        let execution = state.executions.get(&handle.execution_id)?;
        match &execution.transport_state {
            CodexTransportState::AppServer(runtime) => Some(runtime.policy_echo.clone()),
            CodexTransportState::Exec => None,
        }
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

    /// The exec transport's own materialization of `CodexConfig.output_
    /// schema` (spec §4.2): written once to `<data_dir>/codex-output-schema.
    /// json`, `None` when no schema is configured. A write failure is
    /// treated as "no schema" rather than a launch failure — this feature is
    /// adapter-local evidence with no contract row (§4.1), so a filesystem
    /// problem here must not turn into a refused Work.
    fn output_schema_path(&self) -> Option<&Path> {
        self.output_schema_path
            .get_or_init(|| {
                let schema = self.config.output_schema.as_ref()?;
                let path = self.config.data_dir.join("codex-output-schema.json");
                match serde_json::to_vec(schema).map(|bytes| std::fs::write(&path, bytes)) {
                    Ok(Ok(())) => Some(path),
                    _ => None,
                }
            })
            .as_deref()
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
        let version_out = match probe_output(exe, &["--version"]) {
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

        let exec_help = match probe_output(exe, &["exec", "--help"]) {
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
        let resume_help = match probe_output(exe, &["exec", "resume", "--help"]) {
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
        match probe_output(&self.config.executable, &["login", "status"]) {
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

    // ---------------------------------------------- W3 §5.2: transport gates

    /// The four app-server admission gates, run once and cached.
    fn appserver_gates(&self) -> &AppServerGates {
        self.appserver_gates
            .get_or_init(|| self.run_appserver_gates())
    }

    fn run_appserver_gates(&self) -> AppServerGates {
        if let Err(reason) = self.gate_g1_help() {
            return AppServerGates {
                result: Err(reason),
                stale: false,
            };
        }
        let stale = match self.gate_g2_g3_fingerprint() {
            Ok(stale) => stale,
            Err(reason) => {
                return AppServerGates {
                    result: Err(reason),
                    stale: false,
                };
            }
        };
        if let Err(reason) = self.gate_g4_handshake() {
            return AppServerGates {
                result: Err(reason),
                stale,
            };
        }
        AppServerGates {
            result: Ok(()),
            stale,
        }
    }

    /// G1: `codex app-server --help` exits 0 and its `--listen` line offers
    /// `stdio://`. Token-free.
    fn gate_g1_help(&self) -> Result<(), String> {
        let exe = &self.config.executable;
        let out = probe_output(exe, &["app-server", "--help"])
            .map_err(|e| format!("G1: cannot run {exe:?} app-server --help: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "G1: {exe:?} app-server --help exited {:?}",
                out.status.code()
            ));
        }
        let help = String::from_utf8_lossy(&out.stdout);
        if !help.contains("stdio://") {
            return Err("G1: app-server --help does not offer a stdio:// transport".to_string());
        }
        Ok(())
    }

    /// G2/G3: `generate-json-schema` succeeds, every pinned file exists, and
    /// the scoped fingerprint is computed. Returns whether it is *stale*
    /// (mismatched against [`codex_appserver::MEASURED_PROTOCOL_FINGERPRINT`])
    /// — never a gate failure by itself (R1): staleness is provenance,
    /// folded into the probe detail, not a refusal. Token-free, ~50ms
    /// (M7/re-measured while authoring this wave).
    fn gate_g2_g3_fingerprint(&self) -> Result<bool, String> {
        let exe = &self.config.executable;
        let scratch = self.config.data_dir.join(".codex-appserver-schema-probe");
        let _ = std::fs::remove_dir_all(&scratch);
        let mut command = Command::new(exe);
        command
            .args(["app-server", "generate-json-schema", "--out"])
            .arg(&scratch)
            .arg("--experimental");
        child::harden_probe_child(&mut command);
        let out = command
            .output()
            .map_err(|e| format!("G2: cannot run {exe:?} app-server generate-json-schema: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "G2: generate-json-schema exited {:?}",
                out.status.code()
            ));
        }
        for file in codex_appserver::PINNED_SCHEMA_FILES {
            if !scratch.join(file).exists() {
                let _ = std::fs::remove_dir_all(&scratch);
                return Err(format!(
                    "G2: pinned schema file {file} is missing from the generated dump"
                ));
            }
        }
        let fingerprint = codex_appserver::compute_fingerprint(&scratch).map_err(|e| {
            let _ = std::fs::remove_dir_all(&scratch);
            format!("G2: {e}")
        })?;
        let _ = std::fs::remove_dir_all(&scratch);
        Ok(fingerprint != codex_appserver::MEASURED_PROTOCOL_FINGERPRINT)
    }

    /// G4: spawn `--listen stdio://`, round-trip `initialize` within budget,
    /// kill the child. Token-free — no `turn/start` is ever sent.
    fn gate_g4_handshake(&self) -> Result<(), String> {
        let budget = self
            .config
            .appserver_budgets
            .map(|b| b.handshake)
            .unwrap_or_else(|| codex_appserver::Budgets::default().handshake);
        let mut child = codex_appserver::AppServerChild::spawn(
            &self.config.executable,
            &self.config.data_dir,
            &self.config.env,
            self.config.codex_home.as_deref(),
            // #310: this child exists only for the `initialize` round trip
            // below, and must not survive a daemon that dies during it.
            child::ChildLifetime::Probe,
            |_handle, _line| {},
        )
        .map_err(|e| format!("G4: {e}"))?;
        let result = child.handle().call(
            "initialize",
            json!({
                "clientInfo": {
                    "name": "sergeant", "title": "sergeant",
                    "version": env!("CARGO_PKG_VERSION"),
                },
                "capabilities": {"experimentalApi": true},
            }),
            budget,
        );
        child.kill(Duration::from_secs(1));
        result.map(|_| ()).map_err(|e| format!("G4: {e}"))
    }

    /// §5.2's resolution rule, memoized.
    fn transport_resolution(&self) -> &TransportResolution {
        self.transport_resolution
            .get_or_init(|| self.resolve_transport())
    }

    fn resolve_transport(&self) -> TransportResolution {
        match self.config.transport {
            TransportChoice::ExecOnly => TransportResolution {
                transport: Transport::Exec,
                detail: format!(
                    "transport: {} (ExecOnly configured)",
                    Transport::Exec.as_str()
                ),
            },
            TransportChoice::AppServerOnly => match &self.appserver_gates().result {
                Ok(()) => TransportResolution {
                    transport: Transport::AppServer,
                    detail: self.appserver_detail("AppServerOnly configured"),
                },
                Err(reason) => TransportResolution {
                    // Irrelevant: `probe()` reports `available: false` for
                    // this exact case (§5.2 rule 2) before this value is
                    // ever read for a launch decision.
                    transport: Transport::Exec,
                    detail: format!(
                        "transport: app-server requested (AppServerOnly) but refused: {reason}"
                    ),
                },
            },
            TransportChoice::Auto => match &self.appserver_gates().result {
                Ok(()) => TransportResolution {
                    transport: Transport::AppServer,
                    detail: self.appserver_detail("Auto"),
                },
                Err(reason) => TransportResolution {
                    transport: Transport::Exec,
                    detail: format!(
                        "transport: {} (Auto: app-server gate failed: {reason})",
                        Transport::Exec.as_str()
                    ),
                },
            },
        }
    }

    fn appserver_detail(&self, why: &str) -> String {
        let gates = self.appserver_gates();
        let mut detail = format!("transport: {} ({why})", Transport::AppServer.as_str());
        if gates.stale {
            detail.push_str(&format!(
                "; protocol: stale (fingerprint [{}] != measured {})",
                codex_appserver::FINGERPRINT_ALGORITHM,
                codex_appserver::MEASURED_PROTOCOL_FINGERPRINT
            ));
        } else {
            detail.push_str(&format!(
                "; protocol: fresh (fingerprint [{}] matches measured)",
                codex_appserver::FINGERPRINT_ALGORITHM
            ));
        }
        detail
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
    ///
    /// #262 (per-profile, W5 fix): `network_access` follows the same
    /// per-option override pattern as `permission_mode` (#47,
    /// `domain::profile::PERMISSION_MODE_OPTION`) — a profile's own
    /// `network_access` option, when set, wins over `CodexConfig::
    /// workspace_write_network_access`; the daemon-global config value is
    /// only the fallback for a profile that never mentions it (or no
    /// profile at all). This is what makes two profiles on one
    /// `CodexBackend` (one shared adapter per daemon) able to actually
    /// disagree, closing the gap the independent review of
    /// split/w5-codex-commit-path named: the field used to be read straight
    /// off `self.config` at every LAUNCH call site, so no profile could ever
    /// change it.
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
        let mut network_access = self.config.workspace_write_network_access;
        if let Some(profile) = profile {
            for (key, value) in &profile.env {
                env.insert(key.clone(), value.clone());
            }
            if let Some(config_home) = &profile.config_home {
                codex_home = Some(config_home.clone());
            }
            match profile.network_access() {
                Ok(Some(value)) => network_access = value,
                Ok(None) => {}
                Err(e) => return Err(self.err_failed(format!("profile {:?}: {e}", profile.name))),
            }
        }
        Ok(LaunchConfig {
            executable,
            env,
            codex_home,
            network_access,
        })
    }

    fn err_failed(&self, detail: impl Into<String>) -> BackendError {
        BackendError::Failed {
            backend: CODEX_BACKEND_NAME.to_string(),
            detail: detail.into(),
        }
    }

    /// #259: resolve each binding's git common dir, or refuse with the named,
    /// actionable error. Shared by every call site that needs the grant for
    /// real (PREPARE's own preflight, and each transport's LAUNCH) — RESUME
    /// does not call this (see its own comment): the grant is provably inert
    /// there, so resolving it would add a fail-closed check with no
    /// corresponding capability it protects.
    fn resolve_git_worktree_common_dirs(
        &self,
        bindings: &[BindingSummary],
    ) -> Result<Vec<PathBuf>, BackendError> {
        git_worktree_common_dirs(bindings)
            .map_err(|reason| self.err_failed(git_admin_dir_refusal(&reason)))
    }

    /// The app-server-only companion to [`Self::resolve_git_worktree_common_dirs`]
    /// — see [`git_worktree_admin_dirs`] for why `launch_appserver` needs
    /// this grant in addition to (never instead of) the common dir.
    fn resolve_git_worktree_admin_dirs(
        &self,
        bindings: &[BindingSummary],
    ) -> Result<Vec<PathBuf>, BackendError> {
        git_worktree_admin_dirs(bindings)
            .map_err(|reason| self.err_failed(git_admin_dir_refusal(&reason)))
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
            git_worktree_common_dirs,
            sandbox,
            network_access,
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
                execution.git_worktree_common_dirs.clone(),
                execution.sandbox,
                execution.network_access,
            )
        };

        let output_schema_path = self.output_schema_path();
        let mut command = Command::new(&executable);
        if first_turn {
            let mut extra_dirs = bindings_outside_cwd.clone();
            extra_dirs.extend(git_worktree_common_dirs.iter().cloned());
            command.args(first_turn_argv(
                &cwd,
                model.as_deref(),
                sandbox,
                &extra_dirs,
                output_schema_path,
                network_access,
            ));
        } else {
            let thread_id = thread_id.clone().ok_or_else(|| {
                self.err_failed("cannot send: no thread id recorded for this execution")
            })?;
            command.args(resume_turn_argv(
                &thread_id,
                model.as_deref(),
                output_schema_path,
            ));
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

    /// LAUNCH over `codex exec` (§3.1): register the execution, spawn turn
    /// 1, and wait bounded for `thread.started` before returning a handle at
    /// all. A failed launch leaves no phantom: adapter state is removed on
    /// every error path.
    fn launch_exec(&self, prepared: &PreparedExecution) -> Result<ExecutionHandle, BackendError> {
        let request = &prepared.request;
        let LaunchConfig {
            executable,
            env,
            codex_home,
            network_access,
        } = self.launch_config(request.profile.as_ref())?;
        // #259: re-resolved here rather than trusting PREPARE's own check —
        // `launch_config`'s own doc comment states the same rule ("LAUNCH
        // re-resolves it, so the two phases can never disagree").
        let git_worktree_common_dirs = self.resolve_git_worktree_common_dirs(&request.bindings)?;
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
                    git_worktree_common_dirs,
                    sandbox: self.config.sandbox,
                    network_access,
                    turns: 0,
                    turn: TurnState::Unlaunched,
                    turn_pgid: None,
                    stopped: false,
                    interrupt_requested: false,
                    reader: None,
                    transport_state: CodexTransportState::Exec,
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

    /// LAUNCH over `codex app-server --listen stdio://` (W3 spec §1.4/§3.2):
    /// spawn the child, handshake, `thread/start` (identity + policy, token-
    /// free — M4/M5), register the execution, then `turn/start` for turn 1.
    /// Unlike exec, the native id (`thread.id`) comes back from a
    /// **synchronous RPC response**, before any turn exists — the crash
    /// window exec's own module docs name is closed on this transport by
    /// construction.
    fn launch_appserver(
        &self,
        prepared: &PreparedExecution,
    ) -> Result<ExecutionHandle, BackendError> {
        let request = &prepared.request;
        let LaunchConfig {
            executable,
            env,
            codex_home,
            network_access,
        } = self.launch_config(request.profile.as_ref())?;
        // #259, mirrored from `launch_exec` — resolved fresh here rather
        // than trusted from PREPARE.
        let git_worktree_common_dirs = self.resolve_git_worktree_common_dirs(&request.bindings)?;
        // #259 (W5c fix, live-reproduced 2026-08-25 on this branch): app-
        // server-only, on top of the common dir above — see
        // `git_worktree_admin_dirs` for the measurement that shows the
        // common dir alone is not enough on this transport.
        let git_worktree_admin_dirs = self.resolve_git_worktree_admin_dirs(&request.bindings)?;
        let budgets = self.config.appserver_budgets.unwrap_or_default();
        let (handshake_budget, thread_start_budget, turn_start_budget) =
            (budgets.handshake, budgets.thread_start, budgets.turn_start);

        let turn_cell: Arc<Mutex<AppServerTurnState>> =
            Arc::new(Mutex::new(AppServerTurnState::Idle));
        let ctx = AppServerLineContext {
            sink: self.sink.lock().expect("codex sink lock").clone(),
            execution_id: request.execution_id.clone(),
            work_id: request.work_id.clone(),
            model: request.model.clone(),
        };
        let cb_cell = Arc::clone(&turn_cell);
        let mut child = codex_appserver::AppServerChild::spawn(
            &executable,
            &request.cwd,
            &env,
            codex_home.as_deref(),
            // Owned for the whole execution, so deliberately *not* hardened:
            // `PR_SET_PDEATHSIG` fires on the spawning thread's death, and
            // this thread returns long before the turn ends.
            child::ChildLifetime::Execution,
            move |handle, line| appserver_on_line(&ctx, &cb_cell, handle, line),
        )
        .map_err(|e| self.err_failed(format!("cannot spawn app-server child: {e}")))?;

        let handshake = child.handle().call(
            "initialize",
            json!({
                "clientInfo": {
                    "name": "sergeant", "title": "sergeant",
                    "version": env!("CARGO_PKG_VERSION"),
                },
                "capabilities": {"experimentalApi": true},
            }),
            handshake_budget,
        );
        if let Err(e) = handshake {
            child.kill(Duration::from_secs(5));
            return Err(self.err_failed(format!("app-server handshake failed: {e}")));
        }
        if let Err(e) = child.handle().notify("initialized", json!({})) {
            child.kill(Duration::from_secs(5));
            return Err(self.err_failed(format!("app-server handshake failed: {e}")));
        }

        let extra_roots = bindings_outside_cwd(&request.cwd, &request.bindings);
        let mut roots = vec![request.cwd.to_string_lossy().into_owned()];
        roots.extend(extra_roots.iter().map(|p| p.to_string_lossy().into_owned()));
        // #259: the app-server's own writable-roots mechanism, granted the
        // same common-dir-per-binding that exec's `--add-dir` is...
        roots.extend(
            git_worktree_common_dirs
                .iter()
                .map(|p| p.to_string_lossy().into_owned()),
        );
        // ...plus, app-server-only, each binding's own admin subdirectory —
        // see `git_worktree_admin_dirs`'s doc comment for the live
        // measurement showing the common dir alone leaves this transport's
        // `sandbox.writableRoots` denying `index.lock` under it.
        roots.extend(
            git_worktree_admin_dirs
                .iter()
                .map(|p| p.to_string_lossy().into_owned()),
        );
        let mut thread_start_params = json!({
            "model": request.model,
            "cwd": request.cwd.to_string_lossy(),
            "approvalPolicy": "never",
            "runtimeWorkspaceRoots": roots,
            "ephemeral": false,
        });
        if let Some(value) = self.config.sandbox.appserver_value() {
            thread_start_params["sandbox"] = json!(value);
        }
        let thread_start_result =
            match child
                .handle()
                .call("thread/start", thread_start_params, thread_start_budget)
            {
                Ok(v) => v,
                Err(e) => {
                    child.kill(Duration::from_secs(5));
                    return Err(self.err_failed(format!("thread/start failed: {e}")));
                }
            };
        let Some(thread_id) = thread_start_result
            .pointer("/thread/id")
            .and_then(Value::as_str)
            .map(str::to_string)
        else {
            child.kill(Duration::from_secs(5));
            return Err(self.err_failed(
                "thread/start's result carried no thread.id (spec §1.5.2 expects one)",
            ));
        };

        let runtime = Arc::new(AppServerRuntime {
            child: Mutex::new(child),
            turn: turn_cell,
            policy_echo: thread_start_result.clone(),
            turn_epochs: AtomicU64::new(1),
        });
        {
            let mut state = self.lock();
            state.executions.insert(
                request.execution_id.clone(),
                CodexExecution {
                    thread_id: Some(thread_id.clone()),
                    work_id: request.work_id.clone(),
                    cwd: request.cwd.clone(),
                    model: request.model.clone(),
                    executable,
                    env,
                    codex_home,
                    bindings_outside_cwd: extra_roots,
                    git_worktree_common_dirs,
                    sandbox: self.config.sandbox,
                    network_access,
                    turns: 0,
                    turn: TurnState::Unlaunched,
                    turn_pgid: None,
                    stopped: false,
                    interrupt_requested: false,
                    reader: None,
                    transport_state: CodexTransportState::AppServer(Arc::clone(&runtime)),
                },
            );
        }

        let prompt = compose_launch_prompt(request);
        match self.appserver_send_turn(
            &request.execution_id,
            &request.work_id,
            &runtime,
            &thread_id,
            &prompt,
            turn_start_budget,
        ) {
            Ok(()) => {
                self.emit(
                    &request.execution_id,
                    &request.work_id,
                    KIND_CONVERSATION_USER,
                    json!({
                        "text": prompt,
                        "thread_id": thread_id,
                        "bindings_outside_cwd": bindings_outside_cwd(&request.cwd, &request.bindings),
                    }),
                );
                Ok(ExecutionHandle {
                    execution_id: request.execution_id.clone(),
                    native_id: Some(thread_id),
                })
            }
            Err(e) => {
                self.lock().executions.remove(&request.execution_id);
                Err(e)
            }
        }
    }

    /// Send one turn over an app-server thread, shared by LAUNCH's turn 1
    /// and SEND's later turns. Sets `AppServerTurnState::InFlight` **before**
    /// `turn/start` is even written (see the variant's own doc comment for
    /// why), so no notification racing ahead of the response can be dropped.
    fn appserver_send_turn(
        &self,
        execution_id: &str,
        work_id: &str,
        runtime: &AppServerRuntime,
        thread_id: &str,
        prompt: &str,
        turn_start_budget: Duration,
    ) -> Result<(), BackendError> {
        let epoch = runtime.turn_epochs.fetch_add(1, Ordering::SeqCst);
        let displaced_late = {
            let mut turn_state = runtime.turn.lock().expect("appserver turn lock");
            if let AppServerTurnState::InFlight { .. } = &*turn_state {
                return Err(self.err_failed(
                    "already has a turn in flight; a codex thread runs one turn at a time",
                ));
            }
            // The outgoing turn's post-settlement tally dies with the cell
            // this line overwrites, so it is reported here rather than lost.
            // End of stream is the *other* place this summary is emitted, and
            // on a thread that runs more than one turn it is not the first:
            // "never silently drop" has to hold at both exits, not the one
            // that is easier to remember.
            let displaced = match &*turn_state {
                AppServerTurnState::Finished { late, .. } => late.summary(),
                AppServerTurnState::Idle | AppServerTurnState::InFlight { .. } => None,
            };
            // The outgoing turn's own id, if it ever got one, so a straggler
            // that names it after this line overwrites the cell is still
            // recognisable as *that* turn's evidence rather than this new
            // one's (see `superseded_turn_id`'s own doc).
            let superseded_turn_id = match &*turn_state {
                AppServerTurnState::Finished { turn_id, .. } => turn_id.clone(),
                AppServerTurnState::Idle | AppServerTurnState::InFlight { .. } => String::new(),
            };
            *turn_state = AppServerTurnState::InFlight {
                turn_id: String::new(),
                acc: TurnAccumulator::new(),
                interrupt: InterruptProgress::NotRequested,
                epoch,
                superseded_turn_id,
            };
            displaced
        };
        if let Some(summary) = displaced_late {
            self.emit(execution_id, work_id, KIND_TURN_HARNESS_ERROR, summary);
        }
        let handle = runtime.child.lock().expect("appserver child lock").handle();
        let mut params = json!({
            "threadId": thread_id,
            "input": [{"type": "text", "text": prompt}],
        });
        if let Some(schema) = &self.config.output_schema {
            params["outputSchema"] = schema.clone();
        }
        let result = match handle.call("turn/start", params, turn_start_budget) {
            Ok(v) => v,
            Err(e) => {
                // Roll back only what this call actually owns. Every other
                // writer of this cell is state-guarded; this one used to be
                // a blind `= Idle`, which meant a `turn/start` that failed
                // *because the child died* would overwrite the fail-closed
                // `Finished` the reader thread had already installed for the
                // very same turn — leaving OBSERVE to report "has not run a
                // turn yet" about a turn that had run, ended, and been
                // journaled. A settled cell is never rolled back, and
                // neither is a later turn's.
                let mut turn_state = runtime.turn.lock().expect("appserver turn lock");
                if matches!(
                    &*turn_state,
                    AppServerTurnState::InFlight { epoch: mine, .. } if *mine == epoch
                ) {
                    *turn_state = AppServerTurnState::Idle;
                }
                drop(turn_state);
                return Err(self.err_failed(format!("turn/start failed: {e}")));
            }
        };
        let turn_id = result
            .pointer("/turn/id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if let AppServerTurnState::InFlight {
            turn_id: slot,
            epoch: mine,
            ..
        } = &mut *runtime.turn.lock().expect("appserver turn lock")
            && *mine == epoch
        {
            *slot = turn_id;
        }
        Ok(())
    }

    /// The app-server half of INTERRUPT (spec §2.2): `turn/interrupt`, and
    /// on any failure the honest downgrade — kill the child's process group
    /// and journal `phase:"interrupt_downgraded"` (§2.2's own rule: "a
    /// downgrade that nobody can see is the dishonesty this wave exists to
    /// avoid").
    fn interrupt_appserver(
        &self,
        execution_id: &str,
        runtime: &AppServerRuntime,
        thread_id: &str,
        interrupt_budget: Duration,
    ) -> Completion {
        let handle = runtime.child.lock().expect("appserver child lock").handle();
        // Mint the id before the cell is marked, and mark the cell before the
        // request goes out: the reader thread resolves this id the instant it
        // decodes the answer, and it can only do that if the cell already
        // names the id it is waiting on.
        //
        // Only fires the RPC (and stamps `Requested`) from `NotRequested` or
        // `RpcFailed` -- the two states in which nobody has ever gotten an
        // honest answer for this turn yet. `Backend::interrupt`/`stop` are
        // expected to be called more than once on the same still-`InFlight`
        // turn (the engine's own ceiling-triggered auto-interrupt followed by
        // a later human cancel is one real path), and a repeat call landing
        // on `Requested { .. }` (already outstanding) or `Acknowledged`
        // (already confirmed) must not clobber that state: doing so is
        // exactly how a turn the harness genuinely acknowledged could settle
        // as `RpcUnresolved` because *this* call's redundant RPC went
        // unanswered. A second ask has nothing to add in either case -- the
        // outstanding request will still resolve on the reader thread, and
        // an acknowledged one already got the honest answer -- so it is a
        // no-op rather than a fresh, tracked attempt.
        let attempt = {
            let mut turn_state = runtime.turn.lock().expect("appserver turn lock");
            match &mut *turn_state {
                AppServerTurnState::InFlight {
                    turn_id, interrupt, ..
                } if interrupt.worth_asking_again() => {
                    let id = handle.reserve_id();
                    *interrupt = InterruptProgress::Requested { id };
                    Some((turn_id.clone(), id))
                }
                AppServerTurnState::InFlight { .. }
                | AppServerTurnState::Idle
                | AppServerTurnState::Finished { .. } => None,
            }
        };
        let Some((turn_id, interrupt_id)) = attempt else {
            return Completion::immediate();
        };
        let work_id = {
            let state = self.lock();
            state
                .executions
                .get(execution_id)
                .map(|e| e.work_id.clone())
                .unwrap_or_default()
        };
        let result = handle.call_reserved(
            interrupt_id,
            "turn/interrupt",
            json!({"threadId": thread_id, "turnId": turn_id}),
            interrupt_budget,
        );
        // An answer that arrived has already been recorded, on the reader
        // thread, in stream order. What is left for this thread is the case
        // the reader never saw: no answer was written at all — the budget
        // expired, or the request could not even be sent. That is an RPC
        // failure, and it is marked here *before* the process-group kill
        // below, because the kill is what ends the child's stdout and the EOF
        // that follows is what settles the turn.
        //
        // `resolve` is what keeps this from overwriting the reader's own
        // verdict: it only fires while the cell still names this request as
        // outstanding.
        if result.is_err()
            && let AppServerTurnState::InFlight { interrupt, .. } =
                &mut *runtime.turn.lock().expect("appserver turn lock")
        {
            interrupt.resolve(interrupt_id, false);
        }
        if let Err(e) = result {
            let pgid = runtime.child.lock().expect("appserver child lock").pgid();
            kill_process_group(Some(pgid));
            self.emit(
                execution_id,
                &work_id,
                KIND_TURN_HARNESS_ERROR,
                json!({"phase": "interrupt_downgraded", "detail": e}),
            );
            // §15 / §3.4 point 3: a downgrade that nobody can see is the
            // dishonesty this wave exists to avoid, and a downgrade OBSERVE
            // can never resolve is the same dishonesty one step further —
            // `interrupt` was already stamped `Requested` above (this branch
            // is only reached for a call that actually sent the RPC; a
            // repeat call landing on an already-`Acknowledged` or already-
            // `Requested` cell returns before ever getting here), so this
            // resolves through the same `classify_terminal` the reader
            // thread's own EOF handling uses, landing on
            // `InterruptedRunning` (never `AmbiguousUnknown`): sergeant did
            // ask for the kill, even though the RPC that would have
            // confirmed it failed. Without this, the turn stays `InFlight`
            // forever and OBSERVE reports `Running` with no way to learn the
            // stage ended.
            if let Some(settlement) = fail_closed_appserver_turn(&runtime.turn) {
                self.emit(
                    execution_id,
                    &work_id,
                    KIND_TURN_HARNESS_ERROR,
                    json!({"phase": "turn_closed_after_interrupt_downgrade"}),
                );
                // Same invariant as the EOF route: a turn that ends here
                // ends with `conversation.turn.ended` too.
                self.emit(
                    execution_id,
                    &work_id,
                    KIND_CONVERSATION_TURN_ENDED,
                    settlement.turn_ended_payload(),
                );
            }
        }
        Completion::immediate()
    }
}

/// One bounded probe invocation, hardened per #310.
///
/// Every gate in this adapter that is a plain "run it and read the text" call
/// goes through here rather than building its own `Command`, so none of them
/// can forget [`child::harden_probe_child`]. `output()` waits, so the child
/// cannot outlive *this call* — but it can outlive a daemon `SIGKILL`ed
/// during it, and a `--help` or a `login status` that never returns is
/// exactly how one gets stuck there.
fn probe_output(exe: &Path, args: &[&str]) -> std::io::Result<std::process::Output> {
    let mut command = Command::new(exe);
    command.args(args);
    child::harden_probe_child(&mut command);
    command.output()
}

/// Kill a turn's whole process group, by the pgid recorded at spawn (§5.5).
///
/// One line, because #310 moved the implementation to
/// [`crate::backend::child::kill_process_group`] — three adapters carried a
/// byte-identical private copy of it and the probe path needed a fourth (R2).
/// The behaviour and every reason for it are documented there; this alias
/// stays so the call sites in this module keep reading as adapter-local
/// vocabulary.
fn kill_process_group(pgid: Option<u32>) {
    crate::backend::child::kill_process_group(pgid);
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
        // Exec has no interrupt RPC at all: the kill *is* the request, so a
        // turn that ends with no terminal after one can only ever be
        // `ProcessKill`.
        let terminal = classify_terminal(&acc, interrupted.then_some(InterruptedVia::ProcessKill));
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
        if !outcome.available {
            return ProbeReport {
                available: false,
                detail: Some(outcome.detail.clone()),
            };
        }
        // §5.2 rule 2: `AppServerOnly` + a failed gate is the one place a
        // refusal is right — the operator asked for exactly this transport,
        // and silently giving them exec (a different capability row) is the
        // dishonesty this rule exists to prevent. Not an R1 violation: this
        // refuses a *configuration* that cannot be satisfied here, not a
        // version.
        if self.config.transport == TransportChoice::AppServerOnly
            && let Err(reason) = &self.appserver_gates().result
        {
            return ProbeReport {
                available: false,
                detail: Some(format!(
                    "{}; app-server requested (AppServerOnly) but refused: {reason}",
                    outcome.detail
                )),
            };
        }
        let resolution = self.transport_resolution();
        ProbeReport {
            available: true,
            detail: Some(format!(
                "{}; {}\nadmission rows:\n{}",
                outcome.detail,
                resolution.detail,
                render_admission_rows()
            )),
        }
    }

    /// PREPARE (§3.1): refuse an unavailable probe or an impossible pin;
    /// resolve and validate the launch configuration so an impossible
    /// profile never reaches a spawn; reserve **no** native id — it cannot
    /// be pre-minted on this transport, and `PreparedExecution::native_id:
    /// None` is exactly the honest answer its own contract blesses.
    ///
    /// #259's fail-closed rule lives here too: a mutation-shaped request
    /// (one or more `bindings` — the only case `compose_launch_prompt` even
    /// names a mutation surface for) whose git common dir grant cannot be
    /// resolved is refused before LAUNCH ever spawns anything, rather than
    /// admitted to run and mechanically fail to commit. This is a pure local
    /// read (a worktree's own `.git` file, already materialized by the time
    /// a Work is submitted) — never a process, network call, or blocking
    /// wait, so it stays inside PREPARE's own no-external-effect contract.
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
        if !request.bindings.is_empty() {
            self.resolve_git_worktree_common_dirs(&request.bindings)?;
        }
        Ok(PreparedExecution {
            execution_id: request.execution_id.clone(),
            native_id: None,
            request: request.clone(),
        })
    }

    /// LAUNCH (§3.1/W3 §5): dispatches to whichever transport this
    /// registration resolved to. Resolution happens once, at PROBE, and is
    /// never revisited per execution (§5.3) — this is the only place that
    /// reads it to make a launch decision.
    fn launch(&self, prepared: &PreparedExecution) -> Result<ExecutionHandle, BackendError> {
        match self.transport_resolution().transport {
            Transport::Exec => self.launch_exec(prepared),
            Transport::AppServer => self.launch_appserver(prepared),
        }
    }

    fn send(&self, handle: &ExecutionHandle, input: &str) -> Result<(), BackendError> {
        let appserver = {
            let state = self.lock();
            self.check_identity(&state, handle)?;
            let execution = &state.executions[&handle.execution_id];
            if execution.stopped {
                return Err(self.err_failed(format!(
                    "execution {} is stopped; not accepting input",
                    handle.execution_id
                )));
            }
            match &execution.transport_state {
                CodexTransportState::Exec => {
                    if let TurnState::InFlight(_) = execution.turn {
                        return Err(self.err_failed(format!(
                            "execution {} already has a turn in flight; a codex exec \
                             conversation runs one turn at a time",
                            handle.execution_id
                        )));
                    }
                    None
                }
                CodexTransportState::AppServer(runtime) => Some(Arc::clone(runtime)),
            }
        };
        match appserver {
            None => self.spawn_turn(&handle.execution_id, input.to_string(), None),
            Some(runtime) => {
                let thread_id = handle
                    .native_id
                    .clone()
                    .ok_or_else(|| self.err_unknown(&handle.execution_id))?;
                let turn_start_budget = self
                    .config
                    .appserver_budgets
                    .map(|b| b.turn_start)
                    .unwrap_or_else(|| codex_appserver::Budgets::default().turn_start);
                let work_id = self.lock().executions[&handle.execution_id].work_id.clone();
                self.appserver_send_turn(
                    &handle.execution_id,
                    &work_id,
                    &runtime,
                    &thread_id,
                    input,
                    turn_start_budget,
                )?;
                self.emit(
                    &handle.execution_id,
                    &work_id,
                    KIND_CONVERSATION_USER,
                    json!({"text": input, "thread_id": thread_id}),
                );
                Ok(())
            }
        }
    }

    fn observe(&self, handle: &ExecutionHandle) -> Result<Observation, BackendError> {
        let state = self.lock();
        if state.executions.contains_key(&handle.execution_id) {
            self.check_identity(&state, handle)?;
            let execution = &state.executions[&handle.execution_id];
            if let CodexTransportState::AppServer(runtime) = &execution.transport_state {
                return Ok(observe_appserver(execution, runtime));
            }
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

    /// §5.5 (exec) / W3 §2.2 (app-server): stop the current turn without
    /// retiring the execution.
    fn interrupt(&self, handle: &ExecutionHandle) -> Result<Completion, BackendError> {
        let appserver = {
            let state = self.lock();
            self.check_identity(&state, handle)?;
            let execution = &state.executions[&handle.execution_id];
            match &execution.transport_state {
                CodexTransportState::AppServer(runtime) => Some(Arc::clone(runtime)),
                CodexTransportState::Exec => None,
            }
        };
        if let Some(runtime) = appserver {
            let interrupt_budget = self
                .config
                .appserver_budgets
                .map(|b| b.interrupt)
                .unwrap_or_else(|| codex_appserver::Budgets::default().interrupt);
            return Ok(self.interrupt_appserver(
                &handle.execution_id,
                &runtime,
                handle.native_id.as_deref().unwrap_or_default(),
                interrupt_budget,
            ));
        }
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
            network_access,
        } = self.launch_config(request.profile.as_ref())?;
        // #259: NOT resolved here. A re-adopted execution's `turns` starts
        // at 1, so the `execution.turns == 0` gate that reads this field for
        // argv construction never fires for it — the grant is provably dead
        // on this path. Resolving it anyway would add a fail-closed check
        // that protects nothing (daemon-restart reattach would refuse to
        // reconnect to a live, already-running turn over a transient read of
        // a `.git` file whose outcome nothing downstream ever consumes), so
        // this field is left empty rather than recomputed.
        let git_worktree_common_dirs = Vec::new();
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
                thread_id: Some(thread_id.clone()),
                work_id: request.work_id.clone(),
                cwd: request.cwd.clone(),
                model: request.model.clone(),
                executable,
                env,
                codex_home,
                bindings_outside_cwd: bindings_outside_cwd(&request.cwd, &request.bindings),
                git_worktree_common_dirs,
                sandbox: self.config.sandbox,
                network_access,
                turns: 1,
                turn: TurnState::Adopted,
                // A re-adopted thread's turn was spawned by a previous
                // daemon: this one never learned that group, and inventing
                // one would aim a SIGKILL at a pid it cannot account for.
                turn_pgid: None,
                stopped: false,
                interrupt_requested: false,
                reader: None,
                // W3 §5.4: a restarted daemon has no live app-server child to
                // re-adopt — the previous one died with the previous daemon
                // — so re-adoption always continues on the exec transport,
                // whatever the execution originally launched on. This is a
                // transport change for a re-adopted execution whose original
                // transport is provably gone, happening at re-adoption
                // rather than mid-flight, and it is journaled below as a
                // capability withdrawal exactly when this backend's own
                // resolved transport is app-server (the only case where a
                // reader could otherwise be surprised the row changed).
                transport_state: CodexTransportState::Exec,
            },
        );
        drop(state);
        if self.transport_resolution().transport == Transport::AppServer {
            self.emit(
                &handle.execution_id,
                &request.work_id,
                KIND_TURN_HARNESS_ERROR,
                json!({
                    "phase": "transport_withdrawn_on_readopt",
                    "thread_id": thread_id,
                    "detail": "re-adopted after a daemon restart; this execution continues on \
                               the exec transport (no live app-server child survives a restart), \
                               losing NativeTurnInterrupt tier and any admitted ask capability \
                               for its remaining turns",
                }),
            );
        }
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
    /// the reader's join as the completion's tail (issue #14/B3's rule). On
    /// app-server, the "turn" and "execution" processes are the same thing
    /// (§1.4: one child for the whole execution) — STOP is what actually
    /// ends the child's life; INTERRUPT never does.
    fn stop(&self, handle: &ExecutionHandle) -> Result<Completion, BackendError> {
        self.interrupt(handle)?.wait();
        let (reader, appserver) = {
            let mut state = self.lock();
            self.check_identity(&state, handle)?;
            let execution = state
                .executions
                .get_mut(&handle.execution_id)
                .expect("presence checked above");
            execution.stopped = true;
            let appserver = match &execution.transport_state {
                CodexTransportState::AppServer(runtime) => Some(Arc::clone(runtime)),
                CodexTransportState::Exec => None,
            };
            (execution.reader.take(), appserver)
        };
        if let Some(runtime) = appserver {
            let stderr_drain = codex_appserver::Budgets::default().stderr_drain;
            return Ok(Completion::deferred(move || {
                runtime
                    .child
                    .lock()
                    .expect("appserver child lock")
                    .kill(stderr_drain);
            }));
        }
        match reader {
            None => Ok(Completion::immediate()),
            Some(reader) => Ok(Completion::deferred(move || {
                let _ = reader.join();
            })),
        }
    }
}

/// Map an app-server execution's runtime to an Observation (W3 spec §1.4/
/// §2.2/§2.8). `native` answers "is my child alive" — a property of the
/// long-lived child, decoupled from whether a turn has finished (unlike
/// exec, where the per-turn process exiting *is* what "native: Exited"
/// means). `signal` comes from the current-or-last turn.
fn observe_appserver(execution: &CodexExecution, runtime: &AppServerRuntime) -> Observation {
    let thread_ref = execution.thread_id.as_deref().unwrap_or("<unminted>");
    let (child_status, stderr_tail) = {
        let child = runtime.child.lock().expect("appserver child lock");
        (child.status(), child.stderr_tail())
    };
    let (native, child_note) = match &child_status {
        codex_appserver::ChildStatus::Running => (NativeState::Running, "child alive".to_string()),
        codex_appserver::ChildStatus::Exited { detail, code } => (
            NativeState::Exited,
            format!("child exited ({detail}, code={code:?})"),
        ),
        // The one honest `Unknown`: the peek itself failed, so this really
        // is a state the adapter cannot tell.
        codex_appserver::ChildStatus::Unknown(e) => (
            NativeState::Unknown,
            format!("child state could not be read: {e}"),
        ),
    };
    let turn_state = runtime.turn.lock().expect("appserver turn lock");
    match &*turn_state {
        AppServerTurnState::Idle => Observation {
            native,
            signal: BackendSignal::Running,
            evidence: Some(format!(
                "app-server child for thread {thread_ref} has not run a turn yet"
            )),
        },
        AppServerTurnState::InFlight { turn_id, .. } => Observation {
            native,
            signal: BackendSignal::Running,
            evidence: Some(format!("turn {turn_id} in flight on thread {thread_ref}")),
        },
        AppServerTurnState::Finished {
            turn_id: _,
            outcome,
            last_agent_message,
            message_items,
            tool_items,
            unknown_items,
            unknown_methods,
            last_codex_error_info,
            last_error,
            late: _,
        } => match outcome {
            TerminalOutcome::Completed => Observation {
                native,
                signal: BackendSignal::StageCompleted {
                    summary: last_agent_message.clone(),
                },
                evidence: Some(format!(
                    "thread_id={thread_ref}; message_items={message_items}, \
                     tool_items={tool_items}, unknown_items={unknown_items:?}, \
                     unknown_methods={unknown_methods:?}"
                )),
            },
            TerminalOutcome::Failed { message } => {
                // §2.8: a typed `unauthorized` arm names auth explicitly —
                // exec could only ever produce an opaque failure string.
                let auth_note = if last_codex_error_info.as_deref() == Some("unauthorized") {
                    " (auth)"
                } else {
                    ""
                };
                Observation {
                    native,
                    signal: BackendSignal::Failed {
                        reason: format!("turn failed{auth_note}: {}", truncate(message, 400)),
                    },
                    evidence: Some(format!(
                        "thread_id={thread_ref}; codex_error_info={last_codex_error_info:?}"
                    )),
                }
            }
            // §2.2: a first-class, harness-confirmed terminal — the
            // conversation stays resumable, exactly as exec's inferred
            // `InterruptedRunning` reports, but never inferred here.
            TerminalOutcome::Interrupted => Observation {
                native,
                signal: BackendSignal::Running,
                evidence: Some(format!(
                    "turn interrupted; thread {thread_ref} resumable (app-server: \
                     harness-confirmed)"
                )),
            },
            // Reached only via `fail_closed_appserver_turn`'s own use of
            // `classify_terminal` (§15/§3.4 point 3). Sergeant did ask for
            // the kill, but nothing confirmed it the way a real
            // `turn/completed{status:"interrupted"}` would — never claim
            // `harness-confirmed` for this arm. *Which* unconfirmed path it
            // was is recorded at settlement time and reported verbatim here:
            // the two app-server paths are different facts, and the routine
            // STOP path (an interrupt the harness accepted, then a closed
            // stream) is not the RPC failure this arm used to assert it was.
            TerminalOutcome::InterruptedRunning { via } => Observation {
                native,
                signal: BackendSignal::Running,
                evidence: Some(format!(
                    "turn interrupted; thread {thread_ref} resumable (app-server: inferred, \
                     not harness-confirmed -- {})",
                    match via {
                        InterruptedVia::RpcFailed =>
                            "turn/interrupt's own RPC failed and sergeant fell back to the \
                             process-group kill",
                        InterruptedVia::ClosedAfterAcknowledgedRpc =>
                            "turn/interrupt was acknowledged, but the child's stdout closed \
                             before turn/completed carried the harness's own verdict",
                        InterruptedVia::RpcUnresolved =>
                            "the child's stdout closed while turn/interrupt was still \
                             outstanding, so nothing ever answered it either way",
                        // Unreachable on this transport (`ProcessKill` is
                        // exec's own arm, minted only by `TurnReader`), and
                        // named rather than collapsed into one of the three
                        // above, which is how this arm went wrong before.
                        InterruptedVia::ProcessKill =>
                            "the turn's process group was killed with no interrupt RPC involved",
                    }
                )),
            },
            // §5.2's ambiguity. `native` is whatever the child actually
            // proved one lock ago — a child this observation just reaped is
            // `Exited`, and reporting `Unknown` over the top of that proof
            // would be the adapter lying about what it can see (the enum's
            // own definition: "the backend cannot tell"). The fail-closed
            // guarantee this arm owes is that it never reports a *stage
            // verdict*, and it does not: `signal` stays `Running`.
            //
            // Worth naming, because it is a real consequence rather than a
            // free improvement: the engine blocks a Work outright on
            // `native: Unknown` (§25), so this arm used to fail closed *at
            // the engine* by mislabelling a proven exit as ignorance. It now
            // parks like any other `Running` observation, and the per-turn
            // ceiling sweep (`Engine::due_interrupts`) is what bounds it —
            // later than an immediate block, and honest. Exec's own
            // ambiguous arm still reports `Unknown`; aligning it is a
            // separate change to a separate transport's contract, not
            // something to smuggle in here.
            TerminalOutcome::AmbiguousUnknown => Observation {
                native,
                signal: BackendSignal::Running,
                evidence: Some(format!(
                    "no turn/completed observed for thread {thread_ref}; {child_note}; \
                     last_error={last_error:?}; codex_error_info={last_codex_error_info:?}; \
                     stderr tail: {}",
                    truncate(&stderr_tail, 400)
                )),
            },
        },
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
                TerminalOutcome::InterruptedRunning { .. } => Observation {
                    native: NativeState::Exited,
                    signal: BackendSignal::Running,
                    evidence: Some(format!(
                        "turn interrupted by request; conversation {thread_ref} resumable; raw={}",
                        outcome.raw_evidence()
                    )),
                },
                // Exec's own `TurnOutcome` never actually carries this arm
                // (its `classify_terminal` only ever builds `Completed`,
                // `Failed`, `InterruptedRunning` or `AmbiguousUnknown` from
                // exec's own `Terminal` type) — kept here so the shared
                // `TerminalOutcome` enum stays exhaustively matched, and
                // because the *meaning* is identical to `InterruptedRunning`
                // from OBSERVE's point of view: no stage verdict, the
                // conversation stays resumable. The app-server path
                // (`codex_appserver.rs`) reports this distinctly in its own
                // evidence rather than through this exec-only function.
                TerminalOutcome::Interrupted => Observation {
                    native: NativeState::Exited,
                    signal: BackendSignal::Running,
                    evidence: Some(format!(
                        "turn interrupted (harness-confirmed); conversation {thread_ref} \
                         resumable; raw={}",
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
        // W3 §3.2/§3.6, deliberately inverted from W1's own tripwire: turn 1
        // now composes `--sandbox`/`--add-dir` by default. `Inherit` +
        // `sandbox_choice_inherit_sends_no_sandbox_param` below is what
        // proves the *absence* path still works.
        let cwd = PathBuf::from("/work/surface");
        let argv = first_turn_argv(&cwd, None, SandboxChoice::WorkspaceWrite, &[], None, false);
        assert_eq!(
            argv,
            vec![
                "exec",
                "--json",
                "--skip-git-repo-check",
                "-C",
                "/work/surface",
                "--sandbox",
                "workspace-write",
            ]
        );
        assert!(!argv.contains(&"-m".to_string()), "no model, no -m flag");

        let pinned = first_turn_argv(
            &cwd,
            Some("gpt-5.6-luna"),
            SandboxChoice::WorkspaceWrite,
            &[],
            None,
            false,
        );
        assert!(pinned.contains(&"-m".to_string()));
        let m_idx = pinned.iter().position(|a| a == "-m").unwrap();
        assert_eq!(pinned[m_idx + 1], "gpt-5.6-luna");

        let with_extra_dir = first_turn_argv(
            &cwd,
            None,
            SandboxChoice::WorkspaceWrite,
            &[PathBuf::from("/other/repo")],
            None,
            false,
        );
        assert_eq!(with_extra_dir.last().unwrap(), "/other/repo");
        assert_eq!(with_extra_dir[with_extra_dir.len() - 2], "--add-dir");

        let with_schema = first_turn_argv(
            &cwd,
            None,
            SandboxChoice::WorkspaceWrite,
            &[],
            Some(Path::new("/data/codex-output-schema.json")),
            false,
        );
        assert_eq!(
            with_schema.last().unwrap(),
            "/data/codex-output-schema.json"
        );
        assert_eq!(with_schema[with_schema.len() - 2], "--output-schema");

        // §4.2's own correction to W1's placeholder: --output-schema DOES
        // re-apply on resume, unlike --sandbox/--add-dir.
        let resume_with_schema = resume_turn_argv(
            "thread-y",
            None,
            Some(Path::new("/data/codex-output-schema.json")),
        );
        assert_eq!(
            resume_with_schema.last().unwrap(),
            "/data/codex-output-schema.json"
        );
        assert_eq!(
            resume_with_schema[resume_with_schema.len() - 2],
            "--output-schema"
        );

        for absent in ["-p", "--profile", "--ignore-user-config", "--ephemeral"] {
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
    fn sandbox_choice_inherit_sends_no_sandbox_param() {
        let cwd = PathBuf::from("/work/surface");
        let argv = first_turn_argv(&cwd, None, SandboxChoice::Inherit, &[], None, false);
        assert!(!argv.contains(&"--sandbox".to_string()));
        assert!(!argv.contains(&"-s".to_string()));
        assert_eq!(SandboxChoice::Inherit.appserver_value(), None);
        assert_eq!(SandboxChoice::ReadOnly.exec_value(), Some("read-only"));
        assert_eq!(SandboxChoice::ReadOnly.appserver_value(), Some("read-only"));
    }

    #[test]
    fn resume_turn_argv_omits_cd_and_places_the_thread_id_right_after_resume() {
        let argv = resume_turn_argv("01a02508-5880-7980-95b7-1d8bc22d5139", None, None);
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

        let pinned = resume_turn_argv("thread-x", Some("gpt-5.6-luna"), None);
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
                ppid: None,
                argv: vec!["exec".to_string(), "-C".to_string(), "/work".to_string()],
            },
            ProcessArgv {
                pid: 20,
                ppid: None,
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
            ppid: None,
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
            ppid: None,
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
            ppid: None,
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
        let terminal_outcome = classify_terminal(&acc, None);
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
            classify_terminal(&acc, None),
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
            classify_terminal(&acc, None),
            TerminalOutcome::AmbiguousUnknown
        ));
        assert!(matches!(
            classify_terminal(&acc, Some(InterruptedVia::ProcessKill)),
            TerminalOutcome::InterruptedRunning {
                via: InterruptedVia::ProcessKill
            }
        ));
    }

    /// One in-flight app-server turn whose interrupt is outstanding under a
    /// known request id.
    fn interrupted_turn(id: u64) -> AppServerTurnState {
        AppServerTurnState::InFlight {
            turn_id: "turn-1".to_string(),
            acc: TurnAccumulator::new(),
            interrupt: InterruptProgress::Requested { id },
            epoch: 1,
            superseded_turn_id: String::new(),
        }
    }

    /// What `via` the settlement lands on for `state`.
    fn settled_via(mut state: AppServerTurnState) -> Option<InterruptedVia> {
        match settle_appserver_turn(&mut state)?.outcome {
            TerminalOutcome::InterruptedRunning { via } => Some(via),
            other => panic!("expected an inferred interrupt, got {other:?}"),
        }
    }

    /// The truthfulness rule `InterruptedVia` exists for, at the one seam
    /// where it is decided. `turn/interrupt`'s fate is recorded against its
    /// own request id, by the reader thread, as the answer is decoded — so
    /// whether the blocked caller or the reader-thread settlement runs next
    /// changes nothing about which sentence OBSERVE prints. Recording it from
    /// the woken caller instead leaves a schedule in which the adapter says
    /// "nothing ever answered it either way" about an RPC the harness had
    /// already answered, which is exactly the lie this type forbids.
    #[test]
    fn an_answered_interrupt_settles_as_acknowledged_whoever_wakes_first() {
        // Some other request's answer is not this request's answer.
        let mut state = interrupted_turn(7);
        if let AppServerTurnState::InFlight { interrupt, .. } = &mut state {
            interrupt.resolve(6, true);
        }
        assert_eq!(
            settled_via(state),
            Some(InterruptedVia::RpcUnresolved),
            "id 6's answer must not settle the request sent under id 7"
        );

        let mut state = interrupted_turn(7);
        if let AppServerTurnState::InFlight { interrupt, .. } = &mut state {
            interrupt.resolve(7, true);
        }
        assert_eq!(
            settled_via(state),
            Some(InterruptedVia::ClosedAfterAcknowledgedRpc),
            "the harness answered: the stream closing afterwards does not un-answer it"
        );

        let mut state = interrupted_turn(7);
        if let AppServerTurnState::InFlight { interrupt, .. } = &mut state {
            interrupt.resolve(7, false);
        }
        assert_eq!(settled_via(state), Some(InterruptedVia::RpcFailed));

        // And nothing arrived at all: the honest third answer, reachable only
        // when the id really was never answered.
        assert_eq!(
            settled_via(interrupted_turn(7)),
            Some(InterruptedVia::RpcUnresolved)
        );
    }

    /// `turn/completed` nests its turn id under `turn.id`; every other
    /// per-turn notification carries a flat `turnId`. A notification neither
    /// shape recognizes (`thread/started`, a connection-level warning) names
    /// no turn at all -- `None`, not a guess.
    #[test]
    fn notification_turn_id_reads_the_shape_each_method_actually_carries() {
        assert_eq!(
            notification_turn_id(
                "turn/completed",
                &json!({"turn": {"id": "turn-9", "status": "completed"}})
            ),
            Some("turn-9")
        );
        assert_eq!(
            notification_turn_id(
                "item/completed",
                &json!({"turnId": "turn-9", "item": {"id": "i1"}})
            ),
            Some("turn-9")
        );
        assert_eq!(
            notification_turn_id("thread/started", &json!({"thread": {"id": "t1"}})),
            None
        );
        assert_eq!(
            notification_turn_id("configWarning", &json!({"message": "no bubblewrap"})),
            None
        );
    }

    /// A repeat `Backend::interrupt`/`stop` call on a turn that is still
    /// `InFlight` -- the engine's own ceiling-triggered auto-interrupt
    /// followed by a later human cancel is one real path -- must never
    /// re-ask the harness once an ask has already gotten (or is already
    /// getting) an honest answer. Only `NotRequested` and `RpcFailed` are
    /// worth a fresh, tracked attempt.
    #[test]
    fn only_not_requested_and_rpc_failed_are_worth_asking_again() {
        assert!(InterruptProgress::NotRequested.worth_asking_again());
        assert!(InterruptProgress::RpcFailed.worth_asking_again());
        assert!(!InterruptProgress::Requested { id: 1 }.worth_asking_again());
        assert!(!InterruptProgress::Acknowledged.worth_asking_again());
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

    /// coverage-spec.md §2f: `codex_home`'s three-tier fallback (config
    /// override, then `$CODEX_HOME`, then `~/.codex`) had no test for its
    /// middle tier. No config override is set here, so a `CODEX_HOME` the
    /// process actually carries must win over the `~/.codex` default.
    #[test]
    fn codex_home_falls_back_to_the_codex_home_env_var_with_no_config_override() {
        // SAFETY: mirrors codex_config_new_reads_the_bin_env_override's own
        // save/restore discipline immediately above -- this test does not
        // run concurrently with another that reads CODEX_HOME, and the
        // previous value (if any) is restored before returning.
        let previous = std::env::var_os("CODEX_HOME");
        unsafe { std::env::set_var("CODEX_HOME", "/env-codex-home") };
        let config = CodexConfig::new(Path::new("/data")); // codex_home: None
        let backend = CodexBackend::new(config);
        let home = backend.codex_home();
        match previous {
            Some(value) => unsafe { std::env::set_var("CODEX_HOME", value) },
            None => unsafe { std::env::remove_var("CODEX_HOME") },
        }
        assert_eq!(
            home,
            PathBuf::from("/env-codex-home"),
            "no config override was set; the env var must be honored over ~/.codex"
        );
    }

    /// coverage-spec.md §2f: a write failure while materializing `--output-
    /// schema`'s file is treated as "no schema" (§4.1: this feature is
    /// adapter-local evidence with no contract row, so a filesystem problem
    /// here must never turn into a refused Work) -- never surfaced as an
    /// error and never panics. `data_dir` itself is made read-only so the
    /// write the method attempts fails outright.
    #[test]
    fn output_schema_path_returns_none_when_the_write_itself_fails() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::TempDir::new().expect("tempdir");
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555))
            .expect("chmod data_dir read-only");
        struct RestorePerms(PathBuf);
        impl Drop for RestorePerms {
            fn drop(&mut self) {
                let _ = std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o755));
            }
        }
        let _restore = RestorePerms(dir.path().to_path_buf());

        // Environment probe (CONTRIBUTING.md's testing rules): a root
        // dev container silently ignores permission-bit restrictions, so
        // this fixture cannot be armed there -- skip loudly rather than
        // assert something that cannot hold in that environment.
        if std::fs::write(dir.path().join("probe"), b"x").is_ok() {
            eprintln!(
                "SKIPPED-ENV: permission bits are not enforced on this host (root container \
                 shape) -- the unwritable-data_dir fault cannot be armed here"
            );
            return;
        }

        let mut config = CodexConfig::new(dir.path());
        config.output_schema = Some(json!({"type": "object"}));
        let backend = CodexBackend::new(config);
        assert_eq!(
            backend.output_schema_path(),
            None,
            "a write failure must be treated as 'no schema', not surfaced as an error"
        );
    }

    /// coverage-spec.md §2f: `TurnOutcome::raw_evidence`'s `unarchived
    /// (error)` arm -- named separately from the "streamed nothing" arm
    /// because a real archive failure should read differently from a turn
    /// that legitimately produced no bytes to archive.
    #[test]
    fn raw_evidence_names_the_archive_error_when_the_blob_never_landed() {
        let outcome = TurnOutcome {
            terminal: TerminalOutcome::Completed,
            pin_mismatch: None,
            message_items: 0,
            tool_items: 0,
            unknown_items: Vec::new(),
            unparsed_lines: 0,
            last_agent_message: None,
            last_error: None,
            exit_code: None,
            raw_blob: None,
            raw_error: Some("blob store unwritable".to_string()),
            stderr: String::new(),
        };
        assert_eq!(outcome.raw_evidence(), "unarchived (blob store unwritable)");
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

    // ------------------------------------------------------- W3 admission rows

    /// L8, made structural (spec §2.1): every `claimed: true` names a real
    /// (non-empty) admission test, and every v1 flag `capabilities()`
    /// reports `true` has a matching, claimed row on *both* transports —
    /// `capabilities()` takes no transport argument (§5.3), so the two
    /// transports must agree on every boolean or the contract itself would
    /// be a lie for whichever transport a registration resolves to.
    #[test]
    fn admission_rows_agree_with_capabilities() {
        let config = CodexConfig::new(Path::new("/nonexistent"));
        let backend = CodexBackend::new(config);
        let caps = backend.capabilities();

        let flags: &[(&str, bool)] = &[
            ("persistent_sessions", caps.persistent_sessions),
            ("native_background", caps.native_background),
            ("streaming", caps.streaming),
            ("history", caps.history),
            ("resume", caps.resume),
            ("interrupt", caps.interrupt),
            ("model_selection", caps.model_selection),
            ("profiles", caps.profiles),
            ("approval_flow", caps.approval_flow),
            ("human_attach", caps.human_attach),
            ("usage", caps.usage),
            ("native_subagents", caps.native_subagents),
            ("ask", caps.ask),
        ];

        for &(flag, claimed_by_contract) in flags {
            for transport in [Transport::Exec, Transport::AppServer] {
                let row = ADMISSION_ROWS
                    .iter()
                    .find(|r| r.capability == flag && r.transport == transport)
                    .unwrap_or_else(|| {
                        panic!("no ADMISSION_ROWS entry for {flag} on {transport:?}")
                    });
                assert_eq!(
                    row.claimed, claimed_by_contract,
                    "{flag} on {transport:?}: capabilities() says {claimed_by_contract}, the row \
                     says {}",
                    row.claimed
                );
                // §2.1's structural rule is one-directional: a `claimed:
                // true` MUST name a real test (L8, made structural). A
                // `claimed: false` row MAY still name one — a deterministic
                // test proving a negative deliberately (e.g. `approval_flow`
                // on app-server: "every server request answered, the five
                // approval methods always denied") is real evidence too,
                // just not evidence *for* the flag.
                if row.claimed {
                    assert!(
                        !row.admission_test.is_empty(),
                        "{flag} on {transport:?}: claimed true with no admission_test named"
                    );
                }
            }
        }

        // The two non-v1-flag rows (§4.1's structured_output, §3.1's
        // sandbox_enforcement) hold the same invariant even though no
        // `Capabilities` field checks them.
        for capability in ["structured_output", "sandbox_enforcement"] {
            for transport in [Transport::Exec, Transport::AppServer] {
                let row = ADMISSION_ROWS
                    .iter()
                    .find(|r| r.capability == capability && r.transport == transport)
                    .unwrap_or_else(|| {
                        panic!("no ADMISSION_ROWS entry for {capability} on {transport:?}")
                    });
                if row.claimed {
                    assert!(!row.admission_test.is_empty());
                }
            }
        }
    }

    /// [`Evidence`]'s own definitions are the contract: `LiveMeasured` means
    /// "driven against the real, installed harness (a `#[ignore]`d live test,
    /// gated behind SERGEANT_CODEX_TESTS=1)" and `LocallyMeasured` means
    /// "without a live run". This suite's naming convention makes that
    /// checkable: every live test in `tests/codex_backend.rs` is named
    /// `live_*` and nothing else is. So a `claimed: true` row is only
    /// internally consistent if the two agree.
    ///
    /// Scoped to `claimed: true` deliberately. A `claimed: false` row may
    /// honestly cite a live test under a *different* tier — `ask` on
    /// app-server names `live_appserver_actor_authored_question_is_typed`
    /// while staying `Unmeasured`, because that probe measures model
    /// behaviour, not the capability, and the row says so.
    #[test]
    fn a_claimed_row_naming_a_live_test_is_labelled_live_measured() {
        for row in ADMISSION_ROWS.iter().filter(|row| row.claimed) {
            let names_live_test = row.admission_test.starts_with("live_");
            assert_eq!(
                names_live_test,
                row.evidence == Evidence::LiveMeasured,
                "{} on {:?}: evidence {:?} disagrees with its admission test {:?} -- \
                 LiveMeasured is exactly the tier a live_* test establishes, and the only \
                 tier it establishes",
                row.capability,
                row.transport,
                row.evidence,
                row.admission_test,
            );
        }
    }

    #[test]
    fn render_admission_rows_names_every_row_and_every_test() {
        let rendered = render_admission_rows();
        for row in ADMISSION_ROWS {
            assert!(rendered.contains(row.capability));
            if row.claimed {
                assert!(
                    rendered.contains(row.admission_test),
                    "rendered table must name {}'s admission test",
                    row.capability
                );
            }
        }
    }
}
