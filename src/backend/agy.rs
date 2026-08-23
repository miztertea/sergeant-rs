//! Antigravity (`agy`) adapter: `agy -p <prompt> --output-format stream-json`
//! non-interactive turns over a **harness-minted, server-side durable
//! conversation** (W1 of the *Sergeant speaks Antigravity* sprint,
//! `docs/proposals/agy-adapter-2026-08-23.md`, including its panel
//! amendments). One OS process per turn; this is W1's only transport.
//! `--input-format stream-json` (a persistent stdin turn loop) is W3's and is
//! deliberately out of scope here.
//!
//! **Every behavioural claim below carries its provenance.** A claim with no
//! tag is a defect.
//!
//! - **[packet N]** — probe N of `knowledge/evidence/agy-adapter-probes-2026-08-23.md`,
//!   measured at agy **1.1.17**.
//! - **[W1 PN]** — measured by this wave's own probe PN, on Cerberus,
//!   2026-08-23, against the **installed agy 1.1.19**. Transcripts are in the
//!   packet's dated W1 section; the fixtures cut from them are committed under
//!   `tests/fixtures/agy-1.1.19-*`.
//! - **[changelog]** — `~/.gemini/antigravity-cli/cache/CHANGELOG.md`, the
//!   CLI's own bundled release notes (stronger than the website docs, which
//!   are measurably stale).
//! - **[doc-claimed]** — antigravity.google/docs; never promoted to a
//!   `claimed: true` on this evidence alone.
//!
//! # Two version numbers, and why they differ
//!
//! [`MEASURED_FLOOR`] is **1.1.17** — the sprint's ruling, and the version the
//! probe packet was measured at. The binary auto-updated to **1.1.19** between
//! the packet and this wave, so every fixture here is a *1.1.19* capture and is
//! named for it. R1 makes the floor **provenance, not a gate**: 1.1.19 is at or
//! above it and reads as `Measured`; a build below it is still `available`,
//! with an unmeasured-provenance detail, never refused. What *is* refused is a
//! CLI whose version cannot be parsed at all, or whose `--help` does not offer
//! this adapter's launch grammar (the A2 split, carried from the codex and
//! opencode sprints).
//!
//! **The empty-SUCCESS fix is 1.1.18, not 1.1.16.** The sprint plan and the
//! probe packet both attribute it to 1.1.16; 1.1.16's notes contain no such
//! entry. [changelog] `## 1.1.18` — *"Fixed print mode (`-p`) exiting
//! successfully with an empty response when the agent state stream was dropped
//! mid-run, which reported a failed turn as a clean success; it now surfaces
//! the stream error and exits non-zero."* This correction does **not** weaken
//! the panel's empty-SUCCESS amendment: that rule is fail-closed by
//! construction and version-independent (§9.2 / [`classify_terminal`] arm 3).
//!
//! # The four R4 deltas — what this adapter beats
//!
//! - **Identity, resolved model and permission mode all arrive on line 1**
//!   [packet 1, W1 P2]. The `init` event carries `conversation_id` plus
//!   `init.{model, cwd, tools, permission_mode}` *before any model output*.
//!   claude verifies its pin post-hoc from `modelUsage`; opencode verifies it
//!   post-turn from `export`; codex records substitution as undetectable. Here
//!   verification is **at launch, before output** — which is why
//!   [`verify_pin_from_init`]'s `Substituted` verdict *refuses the LAUNCH*
//!   rather than reporting after the fact.
//! - **Per-step usage** [packet 1, W1 P2]: every `step_update` may carry its
//!   own `{input,output,thinking,cache_read,total}`, so usage is known *during*
//!   the turn, not only at its end.
//! - **Native `--json-schema`** [packet 6, W1 fixture capture]: a CLI flag, not
//!   a protocol negotiation. The terminal `result` carries a validated
//!   `structured_output` object **beside** the prose `response`.
//! - **Zero-quota introspection** [changelog 1.1.12, W1 P0]: print mode answers
//!   read-only slash commands (`/config`, `/usage`, `/permissions`, `/agents`)
//!   with `usage.total_tokens: 0`, `num_turns: 0` and an empty
//!   `conversation_id` — no turn, no quota, no conversation left behind. No
//!   sibling adapter can read the harness's effective configuration or its
//!   remaining quota without spending a turn. This module uses it once, at
//!   probe time, for the permission posture and the trusted-workspace check
//!   ([`read_config_probe`]).
//!
//! # The four honesty hazards this module is built around
//!
//! 1. **Empty SUCCESS.** A `SUCCESS` terminal with no response and no
//!    text-producing `agent_response` step classifies **fail-closed ambiguous**,
//!    never completed-clean ([`classify_terminal`] arm 3). Pinned by
//!    `agy-1.1.19-dropped-stream-empty-success.jsonl`. Do not weaken this
//!    because the installed build is >= 1.1.18: a build-version argument for
//!    removing a fail-closed rule is exactly the reasoning the amendment exists
//!    to forbid.
//! 2. **A terminal that hides a denied tool.** See below — at 1.1.19 the
//!    measured shape is `CANCELED`, and stderr is the *only* evidence.
//! 3. **Silent resume fork.** An unknown `--conversation <id>` does **not**
//!    refuse: it prints a plain-text stderr `warning: conversation "…" not
//!    found` and starts a **fresh** conversation [W1 P0.6]. So a resumed turn
//!    is only a resume if the `init` line echoes back the id we asked for
//!    ([`AgyExecution`]'s resume-identity check), and the stderr warning is a
//!    second, independent detector of the same fact.
//! 4. **A write that silently lands outside the Work's surface** [W1 P3]. With
//!    a `cwd` outside `trustedWorkspaces` and `allowNonWorkspaceAccess: false`,
//!    a `write_to_file` call wrote to the CLI's own scratch directory, the
//!    Work's cwd stayed empty, the turn terminated `SUCCESS`, and **nothing on
//!    stderr or in the NDJSON said so**. LAUNCH emits
//!    `phase: "cwd_outside_trusted_workspaces"` for it, read from the
//!    zero-quota `/config` probe. This hazard was not anticipated by the W1
//!    spec; it is a declared addition, flagged for the panel.
//!
//! # The permission posture, as measured at 1.1.19 (this corrects the packet)
//!
//! [packet 2] measured a **hard** deny at 1.1.17: tool step `ACTIVE → ERROR`, a
//! typed `tool_info.error {type: "TOOL_ERROR", message: "permission check
//! failed … user denied permission to run command"}`, terminal `status:
//! "ERROR"`, exit 1. **That is not what 1.1.19 does.** [two independent live
//! reproductions, and only two: W1 P2's control turn (`p2-control`, terminal
//! `CANCELED`, 1.48s) and W1 P3's turn 1 (`p3-writefile`, the denied `pwd`,
//! terminal `CANCELED`, 2.32s), whose stderr captures are byte-identical]:
//!
//! - the tool step resolves `ACTIVE → **DONE**`, with **no** `tool_info.error`
//!   and **no** `output`;
//! - the terminal is **`CANCELED`**, `response: ""`;
//! - the process exits **0**;
//! - the *only* machine-readable evidence anywhere is a **plain-text stderr
//!   notice**: `jetski: no output produced — a tool required the "command"
//!   permission that headless mode cannot prompt for, so it was auto-denied.
//!   Add an allow-rule under permissions.allow in settings.json (e.g.
//!   command(<target>)).…`
//!
//! So the changelog hypothesis the spec carried — that soft-deny was 1.1.3
//! behaviour later *tightened* into a hard deny — is **inverted** by
//! measurement: soft-deny is the current behaviour and the packet's hard deny
//! no longer reproduces. Consequences, all of them structural:
//!
//! - [`denial_evidence_in_stderr`] is the detector that actually fires;
//!   [`tool_denial_evidence`] (the packet's typed shape) is kept because a
//!   build that emits it must still be handled, and its fixture with it.
//! - [`classify_terminal`] takes the drained **stderr** as an argument. The
//!   spec's three-argument signature could not see the only live signal.
//! - Exit 0 is not a completion and `CANCELED` is not an interrupt we caused:
//!   arm 6 fails that closed.
//!
//! **The file-edit surface is not gated the same way** [W1 P3]: `write_to_file`
//! ran under default `request-review` with no allow-rule at all and no
//! `--mode`. Only the `command` surface auto-denies. Nothing here claims to
//! know why.
//!
//! # The injection channel (permission ladder rung (a)) — measured
//!
//! The panel amendment's rung (a) asks for a measured clean injection channel,
//! and [W1 P2] found one. **Workspace-scope settings do not exist**: a
//! `settings.json` under `<cwd>/.agents/`, `<cwd>/.gemini/`,
//! `<cwd>/.antigravity/`, `<cwd>/.antigravitycli/` or `<cwd>/` itself changed
//! `/config`'s answer in none of the five cases. **No config-home environment
//! variable exists** either (a `strings` scan of the binary names none). What
//! *is* measured is that the CLI reads its settings from
//! `$HOME/.gemini/antigravity-cli/settings.json` — a Go `os.UserHomeDir()`
//! path, so `$HOME` is the only lever, and `$HOME` is per-process. With a
//! per-run `HOME` whose settings carry `permissions.allow: ["command(echo)",
//! "command(echo *)"]` and *nothing else* changed, the same tool that was
//! auto-denied in the control **ran**, output `"agy-w1-probe\r\n"`, terminal
//! `SUCCESS` — and nothing was written into the Work's own diff surface.
//!
//! [`AgyConfig::settings_home`] carries that channel, and the launch composes
//! `HOME=<dir>`. **W1 wires the mechanism and synthesizes no policy**: mapping
//! a Work's declared mutation surface onto agy's `command(...)` /
//! `read_file(...)` / `write_file(...)` / `read_url(...)` / `mcp(...)`
//! namespaces is W3's work, and a policy this wave invented would be a security
//! decision with no measurement behind it. **The blanket
//! `--dangerously-skip-permissions` is never a default** (claude #47) and this
//! module composes it nowhere.
//!
//! Two operator facts the channel owes its user, stated rather than assumed:
//! a `HOME` override also relocates the credential store and the conversation
//! store, so a settings home that does not carry (or symlink) the CLI's own
//! `antigravity-cli` state will fail authentication; and `toolPermission`
//! accepts exactly `request-review`, `strict` and `proceed-in-sandbox` — any
//! other value, including `accept-edits`, **silently falls back to
//! `request-review`** [W1 P2].
//!
//! # The two launch decisions this transport cannot carry
//!
//! - [`StartRequest::instruction_policy`] has no measured agy analog. Nothing
//!   is composed for it; the resolved policy travels into the launch evidence
//!   (`conversation.user`'s payload) so a reader can see it was **carried and
//!   not enforced** rather than assume it was applied.
//! - A profile's `config_home` is **refused, not ignored**
//!   ([`AgyBackend::launch_config`]). No agy config-home *variable* was
//!   measured, and honouring the field by guessing one would be the adapter
//!   inventing a launch decision. The measured channel is a settings **home**
//!   delivered through `HOME`, which is [`AgyConfig::settings_home`]'s own
//!   field — a different thing, named for what was actually measured.
//!
//! # R1/R2/R3 rung log
//!
//! R2 (reuse a shipped shape rather than re-derive it): the admission ledger,
//! the `TurnReader`, the bounded read, the process-group kill, the live-test
//! gate and the fail-closed terminal are `opencode.rs`'s shapes with agy's own
//! evidence in them. In particular [`kill_process_group`] carries opencode
//! probe 11's grandchild lesson **without re-deriving it** — and [W1 P4]
//! measured that `ps -g <pgid>` listed only the `agy` leader while a
//! tool-spawned `sleep 120` was in flight, so whether agy runs tool commands in
//! a different process group is *unmeasured*; a group kill is correct either
//! way, which is exactly why the rung is reused rather than re-argued.
//!
//! Deliberately **not** abstracted: the version parser (agy prints a bare
//! triple like opencode, not codex's vendor-token form), the terminal
//! classifier (agy's `status` enum is its own vocabulary), [`truncate`], and
//! the prompt constants. [`ENVIRONMENT_CONTRACT`] and [`MUTATION_SURFACE_HEADER`]
//! are **copied, not imported**, so an edit to another adapter's prompt is
//! never an unreviewed edit to this one; a unit test pins that the environment
//! text matches claude's today.
//!
//! R3/K2 (adapter only): nothing outside `src/backend/` is touched beyond the
//! wave's declared ledger. In particular this module declares **no new `KIND_*`
//! constant** — one would force an `api::SSE_EVENT_KINDS` edit to satisfy
//! `tests/m6_surfaces.rs`'s `t6`, which is core — and instead reuses
//! [`KIND_TURN_HARNESS_ERROR`](super::codex::KIND_TURN_HARNESS_ERROR), already
//! in that vocabulary and already meaning exactly "the harness said something
//! went wrong", distinguished by a `phase` field in the payload.
//!
//! # The crash window this adapter does not close
//!
//! `conversation_id` is **harness-minted** and first appears on the `init`
//! line, so [`PreparedExecution::native_id`] is `None` (its own contract
//! blesses that as honest) and a daemon that dies between LAUNCH's spawn and
//! the engine's `execution.started` commit leaves a live `agy` turn whose
//! conversation id is in no journal, plus a durable conversation in agy's own
//! store, and nothing here reaps them. It is the same window `codex.rs` and
//! `opencode.rs` record, for the same structural reason. Said plainly rather
//! than papered over.
//!
//! Registration in `daemon.rs` and every CLI surface remain W2's gap (K2: this
//! wave is the adapter and nothing else) — this module compiles, is unit- and
//! contract-tested, and is reachable by nothing yet.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use serde_json::{Value, json};

use super::codex::KIND_TURN_HARNESS_ERROR;
use super::{
    Backend, BackendError, BackendSignal, BindingSummary, Capabilities, Completion, EventSink,
    ExecutionHandle, NativeEvent, NativeState, Observation, PreparedExecution, ProbeReport,
    ResumeRequest, RuntimeScope, StartRequest,
};
use crate::domain::event::{EventDraft, EventSource};
use crate::domain::profile::Profile;
use crate::runtime::blob::BlobStore;
use crate::runtime::graph::{
    KIND_CONVERSATION_ASSISTANT_COMPLETED, KIND_CONVERSATION_TURN_ENDED, KIND_CONVERSATION_USER,
    KIND_TOOL_COMPLETED, KIND_TOOL_REQUESTED, KIND_USAGE_UPDATED,
};

// ------------------------------------------------------------------ consts

/// Name this backend registers under (K4). Registration itself is W2's gap.
pub const AGY_BACKEND_NAME: &str = "agy";

/// Environment variable naming the `agy` executable to use — the
/// `SGT_CLAUDE_BIN`/`SGT_CODEX_BIN`/`SGT_OPENCODE_BIN` pattern, for the same
/// reason: *which* `agy` the daemon runs is an operator fact `sgt doctor` must
/// be able to name. `~/.local/bin/agy` is already on `harness.rs`'s
/// `toolchain_path_dirs`, so no PATH line is needed this sprint.
pub const AGY_BIN_ENV: &str = "SGT_AGY_BIN";

/// The agy version the probe packet's behavioural claims were measured
/// against. **Provenance, not a gate** (R1). See the module doc for why the
/// fixtures here are named 1.1.19 while this stays 1.1.17.
pub const MEASURED_FLOOR: (u64, u64, u64) = (1, 1, 17);

/// Flags this adapter's launch grammar composes, which `agy --help` must
/// therefore offer. **Exactly the flags this adapter composes and nothing
/// more** (`opencode.rs`'s rule about `--auto`). `--print-timeout`,
/// `--input-format`, `--sandbox`, `--agent`, `--mode`, `--effort`, `--add-dir`,
/// `--project` and `--dangerously-skip-permissions` are deliberately absent:
/// W1 composes none of them.
///
/// `--json-schema` is here because [`AgyConfig::json_schema`] composes it when
/// set, and a build without the flag is a build whose grammar this adapter
/// never measured.
pub const REQUIRED_FLAGS: &[&str] = &[
    "--print",
    "--output-format",
    "--model",
    "--conversation",
    "--disable-slash-commands",
    "--json-schema",
];

/// How long LAUNCH waits for the `init` line — which is line 1, emitted before
/// the model's first token [packet 1, W1 P2] — before concluding the launch
/// failed. Thirty seconds is an order of magnitude of headroom, matching
/// `opencode.rs`'s `SESSION_ID_BUDGET` reasoning.
const INIT_LINE_BUDGET: Duration = Duration::from_secs(30);

/// How long the turn reader waits for stderr after the turn's process has been
/// reaped — the sibling adapters' identical fix for the same race (both pipes
/// reach EOF at the same instant). **Load-bearing here in a way it is nowhere
/// else**: the permission-denial notice [W1 P2] and the resume warning
/// [W1 P0.6] are stderr-only facts, and the first of them is the *sole*
/// evidence that a tool was denied at 1.1.19.
const STDERR_DRAIN_BUDGET: Duration = Duration::from_secs(5);

/// Wall-clock ceiling on [`AgyBackend::read_config_probe`]'s `-p "/config"`
/// call. **Measured (W2, agy-registration wave): this call is not always
/// zero-interaction the way the doc comment above it claims** — an
/// unauthenticated `agy` (no cached credentials under the effective
/// `settings_home`/`HOME`) answers it by printing an OAuth URL and blocking
/// on an interactive login for up to 60s before giving up on its own. `run_probe`
/// runs synchronously inside daemon registration (`daemon::start_with`,
/// before the descriptor is published), so an unbounded wait here is exactly
/// the class of regression this project already tracks for a blocking HTTP
/// client built during registration — its subprocess analogue, on any host
/// that has `agy` installed but not yet logged in. Five seconds is
/// generous headroom over the sub-second reply a real, authenticated `agy`
/// gave in measurement; a probe that cannot answer inside it is killed and
/// treated exactly like any other probe failure — best-effort by
/// construction either way.
const CONFIG_PROBE_BUDGET: Duration = Duration::from_secs(5);

/// Cap on the in-memory accumulation of one turn's raw stdout before it is
/// archived to the blob store (§20), and of its stderr. Every line is still
/// parsed and forwarded regardless of this cap, and both pipes are still
/// drained to EOF, so a capped turn's process never blocks on a full pipe;
/// what is lost past the cap is only the raw archive's completeness beyond it,
/// and the loss is marked in the archived text itself.
const STREAM_MEMORY_CAP: usize = 16 * 1024 * 1024;

/// Bounded tail of a `tool_info.output` kept inline in `tool.completed`. The
/// full bytes are in the turn's raw blob by construction (§20).
const TOOL_OUTPUT_TAIL: usize = 1024;

/// Composed-prompt bytes this transport can carry on argv.
///
/// **Measured** [W1 P0.4]: with `getconf ARG_MAX` = 2097152, a single argv
/// string of **131_072** bytes still fails `E2BIG` at `Command::spawn` —
/// `OSError errno 7, Argument list too long: 'agy'`, the exec never happens —
/// while 131_060 spawns fine. That is Linux's per-argument `MAX_ARG_STRLEN`,
/// not `ARG_MAX`. It fails *before* any adapter code can observe a stream, so
/// PREPARE refuses it; 120_000 leaves ~11 k of headroom for the flag bytes and
/// argv overhead around it, and is a real bound rather than none.
const ARGV_PROMPT_CAP: usize = 120_000;

/// ADR 0007(a)'s execution-model half, **agy-worded**. Copied in spirit from
/// the sibling adapters' constants of the same purpose, never imported: the
/// three execution models are not identical, and an edit to one must not
/// silently change the others.
///
/// The permission sentence states [W1 P2]'s measurement rather than the
/// packet's or opencode's: agy does not hand the model a tool error it can
/// plan around, and it does not merely fail the call — it **cancels the whole
/// turn** and discards the response.
pub const EXECUTION_MODEL_CONTRACT: &str = "\
Execution model: this is a single non-interactive turn (`agy --print`). You get one turn and no \
callbacks — nothing wakes you when a command you backgrounded finishes after you end your turn. \
There is no approval channel and no way to ask a human anything during this turn. A tool call \
that needs a permission you do not have does not merely fail: it is auto-denied and the whole \
turn is cancelled, and everything you wrote is discarded with it. So do not attempt a tool call \
speculatively to see whether it is allowed; if a step needs a permission you may not have, say \
so and stop rather than spending the turn discovering it. If a command might take a while, run \
it in the foreground with an adequate timeout and wait for it to finish before ending your turn.";

/// `claude.rs`'s `ENVIRONMENT_CONTRACT`, copied verbatim rather than imported
/// (the rule `codex.rs` and `opencode.rs` both follow). A unit test
/// (`the_environment_contract_matches_claudes_today`) pins that the two texts
/// are equal *today*, so a divergence is a decision, not drift nobody noticed.
pub const ENVIRONMENT_CONTRACT: &str = "\
Environment: if this session was reached through `sgt claude` (or `sgt codex`/`opencode`/\
`goose`), your PATH was deliberately composed before this turn was launched to include your \
toolchain (e.g. `~/.cargo/bin`, `~/.local/bin`), and you are bound to the estate that launch \
discovered — sergeant's daemon and every actor beneath it inherit that same environment. This \
does not hold for a daemon reached any other way: a terminal that never went through `sgt \
<harness>` inherits whatever environment it happened to have. If a tool you expect is missing, \
that is more likely an unenriched PATH than a permissions fault — run `sgt doctor` to check what \
this installation's environment actually guarantees before assuming otherwise.";

/// §10.1's section header, agy-local copy of the sibling adapters' private
/// constant of the same text (same reasoning as [`ENVIRONMENT_CONTRACT`]).
const MUTATION_SURFACE_HEADER: &str = "\
Mutation surface: this Work may modify exactly the worktree(s) listed below, and nothing else. \
The estate root, the `repos/` mounts those worktrees were cut from, unselected repositories, \
other Works' surfaces, and any other path on this machine are outside what this Work is \
authorized to change. Each worktree is already checked out on its own branch at its own base \
commit:";

/// Permission modes measured **not** to deny a tool call. Empty, deliberately.
///
/// [W1 P2] measured `request-review` (the default) auto-denying a `command`
/// tool, and also measured the *same* mode permitting that tool once an
/// allow-rule was injected — so the mode string is not a predictor either way.
/// `strict` and `proceed-in-sandbox` are the only other values the CLI accepts
/// [W1 P2] and neither was exercised. Fail closed: an unrecognized, unmeasured
/// or absent mode is **not evidence of permission**.
const NON_DENYING_MODES: &[&str] = &[];

// ----------------------------------------------------------------- config

/// Launch configuration for the adapter, resolved once at construction from the
/// daemon's own environment.
///
/// `Debug` is hand-written, not derived (below): `env` can plausibly carry
/// `GEMINI_API_KEY` (the CLI documents that variable [changelog 1.1.9]) and the
/// schema payload is caller data, and a derived `Debug` would print both in
/// full into any future `{:?}`, panic message or stray `dbg!()`.
#[derive(Clone)]
pub struct AgyConfig {
    /// The CLI executable (a profile may override it per execution).
    pub executable: PathBuf,
    /// Sergeant's data dir; raw per-turn stdout is archived to its blob store
    /// (§20).
    pub data_dir: PathBuf,
    /// Extra environment for every spawned turn (and for the zero-quota
    /// `--version`/`--help`/`/config` probe calls).
    pub env: BTreeMap<String, String>,
    /// A JSON Schema (text, or a path to a schema file) composed as
    /// `--json-schema` on every turn. `None` — the default — composes the flag
    /// at all. Adapter-local until a contract revision gives native structured
    /// output a home in `Capabilities`; `codex.rs::output_schema` and
    /// `OpencodeConfig::structured_format` are the precedents.
    pub json_schema: Option<String>,
    /// The measured permission-injection channel [W1 P2]: a directory composed
    /// as `HOME` for every spawned turn, whose
    /// `.gemini/antigravity-cli/settings.json` the CLI reads its `permissions`
    /// and `toolPermission` from. Named for what was actually measured — a
    /// settings **home**, not a generic `config`.
    ///
    /// W1 wires the mechanism and synthesizes **no** policy. `None` — the
    /// default — means the operator's own agy configuration decides, which
    /// [W1 P2]'s control turn measured auto-denying every `command` tool.
    ///
    /// Overriding `HOME` also relocates the credential and conversation stores:
    /// a settings home that does not carry the CLI's own `antigravity-cli`
    /// state (or symlink it) will fail authentication. That is the operator's
    /// call to make, and the probe detail says so.
    pub settings_home: Option<PathBuf>,
    /// Override for `INIT_LINE_BUDGET`, `None` in every production path. A
    /// per-instance field rather than an environment variable, for the reason
    /// `CodexConfig::thread_id_budget` documents: each test builds its own
    /// config, so a shrunk budget can never leak into another test's
    /// `launch()` — no process-global mutable state, no `--test-threads`
    /// ordering hazard, no `unsafe { std::env::set_var }` to serialize.
    pub init_line_budget: Option<Duration>,
}

impl std::fmt::Debug for AgyConfig {
    /// Redacts `env` (may carry `GEMINI_API_KEY`) and the schema payload to a
    /// count and a length — see the struct's own doc comment for why this is
    /// hand-written rather than derived.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgyConfig")
            .field("executable", &self.executable)
            .field("data_dir", &self.data_dir)
            .field("env", &format!("<{} vars, redacted>", self.env.len()))
            .field(
                "json_schema",
                &self
                    .json_schema
                    .as_ref()
                    .map(|s| format!("<redacted, {} bytes>", s.len())),
            )
            .field("settings_home", &self.settings_home)
            .field("init_line_budget", &self.init_line_budget)
            .finish()
    }
}

impl AgyConfig {
    /// Config for a daemon owning `data_dir`, with the system `agy`.
    pub fn new(data_dir: &Path) -> Self {
        Self {
            executable: std::env::var_os(AGY_BIN_ENV)
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("agy")),
            data_dir: data_dir.to_path_buf(),
            env: BTreeMap::new(),
            json_schema: None,
            settings_home: None,
            init_line_budget: None,
        }
    }
}

/// Apply the adapter's env plus, when a settings home is configured, the
/// measured `HOME` injection channel [W1 P2]. One function so a probe call and
/// a turn can never read two different configurations.
fn apply_env(command: &mut Command, env: &BTreeMap<String, String>, settings_home: Option<&Path>) {
    for (key, value) in env {
        command.env(key, value);
    }
    if let Some(home) = settings_home {
        command.env("HOME", home);
    }
}

// ---------------------------------------------------------- admission rows

/// Which transport a row's evidence was gathered on. One variant today, and the
/// column exists anyway: W3 adds `InputLoop` (the persistent
/// `--input-format stream-json` child), and the codex sprint's own experience
/// is that retrofitting the column *after* a second transport arrives silently
/// re-attributes every existing row to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Transport {
    /// `agy -p … --output-format stream-json`, W1's transport.
    Print,
}

impl Transport {
    fn as_str(self) -> &'static str {
        match self {
            Transport::Print => "print-stream-json",
        }
    }
}

/// How a capability's `true`/`false` was established. The codex/opencode four
/// tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Evidence {
    /// Driven against the real, installed harness (an `#[ignore]`d live test,
    /// gated behind `SERGEANT_AGY_TESTS=1`) that **was actually executed**.
    LiveMeasured,
    /// Proven deterministically (a committed fixture, the shell stub) without a
    /// live run — still a real assertion, just not against the installed binary
    /// today.
    LocallyMeasured,
    /// Named by antigravity's own documentation or changelog; never promoted to
    /// `claimed: true` on this evidence alone.
    DocClaimed,
    /// Looked for and not found — a probe ran, no assertion could be made.
    Unmeasured,
}

/// One row of the wave's capability ledger: the v1 boolean `capabilities()`
/// returns is the contract; this is the *evidence* behind it, adapter-local
/// until a contract revision gives it a home (R3). Rendered into
/// [`ProbeReport::detail`] and the wave PR body.
///
/// **No `stability` column, deliberately (R1)**: every row would carry the same
/// value, so the fact is stated once on [`render_admission_rows`]'s own header.
/// The column arrives when a second value does.
#[derive(Debug, Clone, Copy)]
struct AdmissionRow {
    /// The v1 flag name, or a name v1 has no row for at all.
    capability: &'static str,
    transport: Transport,
    /// What `capabilities()` claims for this transport.
    claimed: bool,
    /// The typed tier this row's evidence supports, or `"-"` when the flag is a
    /// plain boolean with no tier of its own.
    tier: &'static str,
    evidence: Evidence,
    /// The exact test name backing a `claimed: true`, or `""` when `claimed` is
    /// `false` (the structural reason lives in `note`). The name must resolve to
    /// a `fn` in this module or in `tests/agy_backend.rs`;
    /// `tests::every_admission_test_name_resolves_to_a_real_test` enforces it.
    admission_test: &'static str,
    note: &'static str,
}

/// The wave's own ledger. Three structural checks keep it honest, and each
/// fails the build rather than a review:
///
/// - `tests::admission_rows_agree_with_capabilities` — a `claimed: true` with
///   no `admission_test` (and an unclaimed row that names one) is a build
///   failure, as is a row whose `claimed` disagrees with [`Backend::capabilities`];
/// - `tests::every_admission_test_name_resolves_to_a_real_test` — the name is
///   read back against the text of this module and of `tests/agy_backend.rs`, so
///   a typo'd or later-renamed `admission_test` cannot sit here citing a test
///   that does not exist;
/// - `tests::a_claimed_row_naming_a_live_test_is_labelled_live_measured` — the
///   `Evidence` tier must agree with whether the named test is a `live_agy_*` one.
const ADMISSION_ROWS: &[AdmissionRow] = &[
    AdmissionRow {
        capability: "persistent_sessions",
        transport: Transport::Print,
        claimed: true,
        tier: "-",
        evidence: Evidence::LocallyMeasured,
        admission_test: "a_resume_turn_names_the_conversation_and_keeps_the_pin",
        note: "the conversation is minted by the harness on the `init` line of turn 1 and reused, \
               unprompted, as `--conversation <id>` on turn 2's separately-spawned process. The \
               test drives SEND after LAUNCH's turn settles and asserts the same id is composed — \
               the conversation surviving past turn 1's process, which is exactly this flag's \
               claim. Conversation state is server-side [packet 5, W1 P4]; a local \
               ~/.gemini/antigravity-cli/conversations/<id>.db also exists and nothing here reads \
               it",
    },
    AdmissionRow {
        capability: "native_background",
        transport: Transport::Print,
        claimed: false,
        tier: "-",
        evidence: Evidence::DocClaimed,
        admission_test: "",
        note: "the tool roster names a `schedule` tool and the TUI has background tasks [packet 1, \
               W1 P2 init roster, doc-claimed]; no measured mechanism for a PRINT-mode turn to \
               survive its own process — the turn IS the process. Documented is not supported",
    },
    AdmissionRow {
        capability: "streaming",
        transport: Transport::Print,
        claimed: true,
        tier: "-",
        evidence: Evidence::LocallyMeasured,
        admission_test: "events_are_delivered_before_the_turn_process_exits",
        note: "typed NDJSON, one event per line, read and normalized as it arrives; the stub \
               stalls mid-stream and the assertion is that the first step's usage event already \
               landed. `text_delta` deltas are accumulated per step and never emitted \
               individually — the step's DONE/ERROR state produces the one assistant event \
               (measured: a single step can emit ACTIVE with a partial delta then DONE with the \
               rest, W1 json-schema capture)",
    },
    AdmissionRow {
        capability: "history",
        transport: Transport::Print,
        claimed: false,
        tier: "-",
        evidence: Evidence::Unmeasured,
        admission_test: "",
        note: "headless is stateless by default [doc-claimed] and NO export verb exists — the \
               whole reason opencode can claim this flag and agy cannot. Two leads recorded and \
               neither promoted: ~/.gemini/antigravity-cli/conversations/<id>.db (one SQLite file \
               per conversation, schema unmeasured, a private path of another product) and \
               cache/last_conversations.json (cwd->id). §15 forbids emulation, so `Backend::\
               history` returns BackendError::Unsupported naming sergeant's journal as its own \
               record",
    },
    AdmissionRow {
        capability: "resume",
        transport: Transport::Print,
        claimed: true,
        tier: "ConversationIdEchoOnNextTurn",
        evidence: Evidence::LiveMeasured,
        admission_test: "live_agy_resume_recalls_a_nonce_and_echoes_the_same_conversation_id",
        note: "`--conversation <id>` from a separate OS process recalls prior state [packet 5, and \
               W1 P4 measured it even after the previous turn was SIGKILLed mid-tool]. The tier is \
               the interesting part: an unknown id does NOT refuse — it emits a plain-text stderr \
               `warning: conversation \"…\" not found` and starts a FRESH conversation [W1 P0.6] — \
               so re-adoption is evidenced by the next turn's init.conversation_id echoing the id \
               we asked for, checked before any output is trusted. A mismatch fails the turn",
    },
    AdmissionRow {
        capability: "interrupt",
        transport: Transport::Print,
        claimed: true,
        tier: "ProcessTreeTermination",
        evidence: Evidence::LocallyMeasured,
        admission_test: "agy_interrupt_kills_the_process_group",
        note: "process_group(0) at spawn, negated-pgid SIGKILL at interrupt — carried from \
               opencode probe 11's grandchild lesson without re-deriving it (R2). \
               StubAgy-driven and deterministic, so the row is NOT tagged LiveMeasured. W1 P4's \
               live findings ride here, not the tier: a group SIGKILL truncated the stream with \
               NO terminal event of any kind, left `pgrep -x agy` empty, and left the \
               conversation fully resumable with recall. Open and unclaimed: `ps -g <pgid>` \
               listed only the agy leader while a tool-spawned `sleep 120` was in flight, so \
               whether agy runs tool commands in a different process group is unmeasured — a \
               group kill is correct either way",
    },
    AdmissionRow {
        capability: "model_selection",
        transport: Transport::Print,
        claimed: true,
        tier: "InitEchoVerifiedPin",
        evidence: Evidence::LiveMeasured,
        admission_test: "live_agy_init_line_echoes_the_pinned_model_and_mints_the_conversation",
        note: "two layers, and the second is a registry first. Layer 1: a bad pin is refused by \
               the harness BEFORE identity is minted, with the whole catalog enumerated \
               (conversation_id:\"\", exit 1) [packet 4, W1 P0.3 row A] — mapped to a LAUNCH \
               refusal carrying the catalog verbatim. Layer 2: init.model echoes the RESOLVED \
               model on line 1, before any model output, so substitution is caught at launch \
               [packet 1, W1 P2]. Every prior adapter records substitution-undetectable (codex) \
               or post-hoc (claude's modelUsage, opencode's export). Ids are flat \
               (gemini-3.7-flash-low, from the zero-quota `agy models`), so the comparison is \
               exact string equality with none of opencode's provider-splitting",
    },
    AdmissionRow {
        capability: "profiles",
        transport: Transport::Print,
        claimed: true,
        tier: "-",
        evidence: Evidence::LocallyMeasured,
        admission_test: "a_profile_executable_and_env_reach_every_turn",
        note: "DECLARED DIVERGENCE from one reading of the plan's panel amendment — see the \
               module doc and the wave PR body; flagged for the panel, not silent. The generic \
               sergeant axes only: executable + env, reaching every spawned turn. `config_home` \
               is REFUSED, not ignored. agy's own `--agent` is NOT wired this wave: W1 P6's free \
               step defined <workspace>/.agents/agents/probe/agent.md per the documented \
               mechanism and `/agents` still answered {\"agents\":[]} — the mechanism does not \
               work as documented on this host at 1.1.19, so NO live turn was spent on it and \
               nothing is claimed. opencode's precedent verbatim: an agent applied to turn 1 must \
               be re-applied on every resume and that re-application is unmeasured",
    },
    AdmissionRow {
        capability: "approval_flow",
        transport: Transport::Print,
        claimed: false,
        tier: "-",
        evidence: Evidence::LocallyMeasured,
        admission_test: "",
        note: "structural on this transport: the roster names `ask_permission`/\
               `ask_custom_permission` but print mode has no reply channel, and the default \
               request-review mode auto-denies AND cancels the whole turn [W1 P2 control] — there \
               is nobody to approve to. W3's stdin loop is the first-true candidate",
    },
    AdmissionRow {
        capability: "human_attach",
        transport: Transport::Print,
        claimed: false,
        tier: "-",
        evidence: Evidence::Unmeasured,
        admission_test: "",
        note: "no attach mechanism on a non-interactive print turn; `--prompt-interactive`/`-i` is \
               a different (interactive) execution mode this adapter never composes",
    },
    AdmissionRow {
        capability: "usage",
        transport: Transport::Print,
        claimed: true,
        tier: "-",
        evidence: Evidence::LocallyMeasured,
        admission_test: "per_step_and_terminal_usage_become_usage_events",
        note: "one usage.updated per step_update carrying that step's native \
               {input,output,thinking,cache_read,total} — known DURING the turn, not only at its \
               end — plus one final usage.updated tagged scope:\"turn\" from result.usage. Never \
               a synthetic sum: the per-step and terminal objects are both carried verbatim and a \
               reader can see which is which",
    },
    AdmissionRow {
        capability: "native_subagents",
        transport: Transport::Print,
        claimed: false,
        tier: "-",
        evidence: Evidence::DocClaimed,
        admission_test: "",
        note: "agy ships define_subagent/invoke_subagent/manage_subagents/browser_subagent \
               natively [packet 1 roster, re-measured in W1 P2's own init line] — no adapter in \
               the registry claims this flag anywhere, so it is W3's headline candidate. How \
               subagent activity surfaces in the stream is unmeasured (packet open question 7). \
               Documented is not supported",
    },
    AdmissionRow {
        capability: "ask",
        transport: Transport::Print,
        claimed: false,
        tier: "-",
        evidence: Evidence::Unmeasured,
        admission_test: "",
        note: "the roster names `ask_question` [packet 1, W1 P2], which says a question CATEGORY \
               exists — not that an actor's question is schema-distinguishable in this stream. \
               Guessing one from a text_delta is precisely the heuristic Capabilities::ask \
               forbids. [changelog 1.1.12] 'headless -p runs … settle a choice themselves where \
               they would otherwise ask' is evidence AGAINST a question surfacing here at all. W3 \
               candidate",
    },
    AdmissionRow {
        capability: "config_injection",
        transport: Transport::Print,
        claimed: true,
        tier: "SettingsHomeViaHome",
        evidence: Evidence::LocallyMeasured,
        admission_test: "a_permission_config_reaches_every_turn_without_dirtying_the_work_diff",
        note: "PANEL LADDER RUNG (a), measured [W1 P2]. The CLI reads permissions from \
               $HOME/.gemini/antigravity-cli/settings.json and $HOME is per-process, so a per-run \
               settings home is the channel; AgyConfig::settings_home composes HOME=<dir>. \
               Measured evidence that it works: with permissions.allow \
               [\"command(echo)\",\"command(echo *)\"] as the ONLY delta, the same run_command \
               that was auto-denied in the control ran, output \"agy-w1-probe\\r\\n\", terminal \
               SUCCESS, and nothing was written into the Work's cwd. Ruled out first, all free: \
               workspace-scope settings.json under .agents/.gemini/.antigravity/.antigravitycli/ \
               and the cwd root changed /config in none of five cases; no GEMINI_*/AGY_* \
               config-home variable exists in the binary's strings. W1 wires the MECHANISM and \
               synthesizes NO policy: mapping a Work's mutation surface onto command(...)/\
               read_file(...)/write_file(...)/read_url(...)/mcp(...) is W3's work, and a policy \
               this wave invented would be a security decision with no measurement behind it. \
               The blanket --dangerously-skip-permissions is NEVER a default (claude #47)",
    },
    AdmissionRow {
        capability: "permission_mode_reported_at_launch",
        transport: Transport::Print,
        claimed: true,
        tier: "InitEchoPermissionMode",
        evidence: Evidence::LocallyMeasured,
        admission_test: "a_denying_permission_mode_is_reported_at_launch_not_mid_turn",
        note: "the panel amendment's rung-(b) honesty check, shipped REGARDLESS of the rung-(a) \
               outcome: init.permission_mode is read on line 1 and, when it is not on the \
               measured non-denying allowlist (which is empty), the fact is emitted and journaled \
               AT LAUNCH instead of being discovered as a mid-run turn cancellation [packet 1, \
               W1 P2]. Deliberately over-warns: W1 P2 measured request-review echoed on BOTH the \
               denied and the permitted turn, so the mode string predicts nothing — which is \
               exactly why the notice says 'any tool call not covered by an allow-rule' rather \
               than 'every tool call'",
    },
    AdmissionRow {
        capability: "non_blocking_run",
        transport: Transport::Print,
        claimed: true,
        tier: "DeniedToolCancelsTheTurn",
        evidence: Evidence::LocallyMeasured,
        admission_test: "a_denied_tool_call_is_a_cancelled_turn_not_a_hang",
        note: "the same non-hang guarantee codex/opencode have, reached by a third route. \
               [packet 2] measured a hard deny at 1.1.17 (typed TOOL_ERROR, terminal ERROR, exit \
               1, ~4.5 s). [W1 P2/P3] measured that 1.1.19 does something else entirely and \
               CORRECTS the packet: the tool step resolves ACTIVE->DONE with no error and no \
               output, the terminal is CANCELED, the process exits 0 in ~1.5 s, and the only \
               evidence anywhere is a plain-text stderr notice. Either way it resolves promptly \
               and never hangs; both fixtures are committed and the classifier handles both",
    },
    AdmissionRow {
        capability: "structured_output",
        transport: Transport::Print,
        claimed: true,
        tier: "NativeSchemaFlag",
        evidence: Evidence::LocallyMeasured,
        admission_test: "the_json_schema_fixture_carries_structured_output_beside_the_response",
        note: "`--json-schema` yields a validated `structured_output` object BESIDE the prose \
               `response`, plus a `json_schema` echo of the schema itself [packet 6, re-captured \
               live in W1 as agy-1.1.19-json-schema.jsonl]. W1 wires the CHANNEL \
               (AgyConfig::json_schema) and synthesizes no schema — sergeant has no per-stage \
               output-schema surface and inventing one is a core change (K2). Adapter-local, no \
               v1 boolean invented (R3), the posture codex's and opencode's own structured_output \
               rows take",
    },
];

fn render_admission_rows() -> String {
    let mut out = String::from(
        "stability (all rows): Antigravity publishes no CLI breaking-change policy and the \
         installed build moved 1.1.17->1.1.19 during this sprint; MEASURED_FLOOR 1.1.17 is \
         provenance, not a gate (R1)\n\
         capability | transport | claimed | tier | evidence | admission_test | note\n",
    );
    for row in ADMISSION_ROWS {
        out.push_str(&format!(
            "{} | {} | {} | {} | {:?} | {} | {}\n",
            row.capability,
            row.transport.as_str(),
            row.claimed,
            row.tier,
            row.evidence,
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

/// The capability set this transport honestly supports. A pure function of the
/// transport, so the structural admission check can drive it without a backend
/// instance.
fn capabilities_for(transport: Transport) -> Capabilities {
    match transport {
        Transport::Print => Capabilities {
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
        },
    }
}

// --------------------------------------------------------------- the probe

/// Whether the installed build is at or above [`MEASURED_FLOOR`] (R1: this is
/// provenance carried alongside `available`, never a gate on it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VersionProvenance {
    /// `>= MEASURED_FLOOR` — this module's measurements apply directly.
    Measured,
    /// `< MEASURED_FLOOR` — available, but nothing here was re-measured against
    /// this exact build.
    BelowFloor,
}

/// What the zero-quota `/config` read learned [W1 P0.2]. Best-effort in every
/// field: a CLI that does not answer it is not a CLI this adapter refuses — it
/// is a CLI whose effective configuration this adapter cannot report, and the
/// probe detail says so.
#[derive(Debug, Clone, Default)]
struct ConfigProbe {
    tool_permission: Option<String>,
    trusted_workspaces: Vec<PathBuf>,
    allow_non_workspace_access: bool,
    allow_rules: usize,
    read: bool,
}

/// Outcome of the registration-time version/grammar probe.
///
/// `version` and `provenance` are carried for a future `sgt doctor` reader
/// (W2's hand-off) even though nothing in this wave reads them back out —
/// `detail` already carries their rendering for today's one reader. Allowed
/// dead code rather than dropped: W1 must not invent W2's reader, but must not
/// throw away the fields it will need either.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ProbeOutcome {
    available: bool,
    detail: String,
    version: Option<String>,
    provenance: Option<VersionProvenance>,
    config: ConfigProbe,
}

/// Parse `agy --version`'s output into a comparable triple.
///
/// agy prints a **bare** `1.1.19\n` [W1 P0], like opencode and unlike codex's
/// `codex-cli 0.149.0`, so this takes the **first** whitespace-separated token:
/// a build that later prefixes a vendor name is a grammar change worth
/// noticing, and a build that suffixes a git hash still parses. The patch field
/// is read up to its first non-digit so `1.1.19-rc.1` still yields a comparable
/// triple; the full string always travels in the probe's `detail`, never
/// silently dropped.
fn parse_agy_version(text: &str) -> Option<(u64, u64, u64)> {
    let candidate = text.split_whitespace().next()?;
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

/// Every entry of `required` not found in `help` (substring containment, the
/// rule both sibling adapters use).
fn missing_entries(help: &str, required: &[&'static str]) -> Vec<&'static str> {
    required
        .iter()
        .copied()
        .filter(|entry| !help.contains(entry))
        .collect()
}

/// Decode the zero-quota `/config` answer [W1 P0.2]. Reads only the four fields
/// this module has a use for; everything else is left in the raw answer.
fn decode_config_probe(value: &Value) -> ConfigProbe {
    let config = value.pointer("/command/data/config");
    let Some(config) = config else {
        return ConfigProbe::default();
    };
    ConfigProbe {
        tool_permission: config
            .get("toolPermission")
            .and_then(Value::as_str)
            .filter(|mode| !mode.is_empty())
            .map(str::to_string),
        trusted_workspaces: config
            .get("trustedWorkspaces")
            .and_then(Value::as_array)
            .map(|paths| {
                paths
                    .iter()
                    .filter_map(Value::as_str)
                    .map(PathBuf::from)
                    .collect()
            })
            .unwrap_or_default(),
        allow_non_workspace_access: config
            .get("allowNonWorkspaceAccess")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        allow_rules: config
            .pointer("/permissions/allow")
            .and_then(Value::as_array)
            .map(Vec::len)
            .unwrap_or(0),
        read: true,
    }
}

// --------------------------------------------------------- launch grammar

/// One execution's resolved launch configuration (§14 applied to this CLI). One
/// function ([`AgyBackend::launch_config`]) produces it for both LAUNCH and
/// RESUME, so a re-adopted execution cannot launch under different rules than
/// the one it re-adopts.
#[derive(Debug, Clone)]
struct LaunchConfig {
    executable: PathBuf,
    env: BTreeMap<String, String>,
}

/// Turn 1's argv, after `<executable>`.
///
/// **The prompt is the VALUE of `-p`** [W1 P0.3]: stdin is not read as the
/// prompt in text input mode (row B: `--print=` with piped stdin answers
/// `"Error: empty prompt"`), and a valueless `-p` is a hard parse error that
/// swallows the next flag (row C, exit 2). There is no positional-prompt
/// fallback either — [changelog 1.1.18] made a stray trailing argument an
/// error. So the prompt rides argv and only argv, which is why
/// [`ARGV_PROMPT_CAP`] is a real bound rather than a precaution.
fn first_turn_argv(prompt: &str, model: Option<&str>, json_schema: Option<&str>) -> Vec<String> {
    let mut argv = vec![
        "-p".to_string(),
        prompt.to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        // Composed on EVERY turn. §12: procedure is data — sergeant carries a
        // stage's CONTEXT.md to the actor and does not interpret it, and
        // letting the CLI expand a `/skill` token inside carried data would be
        // the harness interpreting sergeant's data. It also closes the
        // [W1 P0.5] hazard where a prompt is answered as a CLI command and
        // returns an empty-SUCCESS terminal with a `command` object.
        "--disable-slash-commands".to_string(),
    ];
    if let Some(model) = model {
        argv.push("--model".to_string());
        argv.push(model.to_string());
    }
    if let Some(schema) = json_schema {
        argv.push("--json-schema".to_string());
        argv.push(schema.to_string());
    }
    argv
}

/// Turn N >= 2's argv: turn 1's, plus `--conversation <id>`.
///
/// The model pin is composed here too, exactly as on turn 1 — a pin the human
/// asked for that silently lapses after the first turn is the adapter dropping
/// a launch decision.
fn resume_turn_argv(
    prompt: &str,
    model: Option<&str>,
    json_schema: Option<&str>,
    conversation: &str,
) -> Vec<String> {
    let mut argv = first_turn_argv(prompt, model, json_schema);
    argv.push("--conversation".to_string());
    argv.push(conversation.to_string());
    argv
}

/// §10.1's section body: the header, then one line per bound repository —
/// identical shape to the sibling adapters' own copies.
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

/// The full first-turn prompt, in the same five-section fixed order the other
/// adapters use: [`EXECUTION_MODEL_CONTRACT`], [`ENVIRONMENT_CONTRACT`], the
/// mutation surface (omitted entirely when `bindings` is empty, because "you
/// may modify nothing" is a claim its silence does not make), the intent, then
/// `CONTEXT.md` — the last two carried verbatim and uninterpreted (§12).
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

/// Every binding path this request carries that does not lie at or under `cwd`.
/// This adapter composes no `--add-dir` at all (its scope versus `--sandbox` is
/// undocumented and nothing unmeasured is claimed), so nothing here changes a
/// launch — this exists so the assumption that sergeant's own binding shape
/// keeps every worktree under the surface root is *checked* rather than
/// trusted.
fn bindings_outside_cwd(cwd: &Path, bindings: &[BindingSummary]) -> Vec<PathBuf> {
    bindings
        .iter()
        .filter(|binding| !binding.worktree_path.starts_with(cwd))
        .map(|binding| binding.worktree_path.clone())
        .collect()
}

/// Pre-flight pin check: refuse only an **empty or whitespace-only** pin.
///
/// An unrecognized model is deliberately *not* refused here: the harness's own
/// typed refusal enumerates the whole catalog [W1 P0.3 row A] and is strictly
/// better evidence than a local allowlist this adapter would have to maintain
/// (R1).
fn preflight_model_pin(model: &str) -> Result<(), String> {
    if model.trim().is_empty() {
        return Err("model pin is empty".to_string());
    }
    Ok(())
}

/// Whether a composed prompt fits on argv, and the refusal text if not (§7.4).
///
/// **MUST NOT truncate.** Truncating a `CONTEXT.md` would be the adapter
/// silently dropping §12 procedure data — the exact class of dishonesty this
/// whole ledger exists to prevent.
fn check_argv_prompt_budget(prompt: &str, request: &StartRequest) -> Result<(), String> {
    let len = prompt.len();
    if len <= ARGV_PROMPT_CAP {
        return Ok(());
    }
    let intent = request.intent.len();
    let context = request.context.len();
    let bindings: usize = request
        .bindings
        .iter()
        .map(|b| b.worktree_path.as_os_str().len() + b.repository.len() + b.base_sha.len())
        .sum();
    let largest = if context >= intent && context >= bindings {
        "context (CONTEXT.md)"
    } else if intent >= bindings {
        "intent"
    } else {
        "the mutation-surface section"
    };
    Err(format!(
        "the composed prompt is {len} bytes, over this transport's {ARGV_PROMPT_CAP}-byte argv \
         cap. The prompt rides argv and only argv on `agy --print` (stdin is not read as the \
         prompt in text input mode), and a single argv string of 131072 bytes fails E2BIG at \
         spawn — before any adapter code can observe anything — while 131060 spawns (Linux \
         MAX_ARG_STRLEN, measured W1 P0.4). The largest section is {largest} (intent {intent} B, \
         context {context} B, bindings {bindings} B). Nothing is truncated: dropping part of a \
         stage's CONTEXT.md would be the adapter silently discarding procedure data. The measured \
         channel for prompts this size is the persistent stdin turn loop (`--input-format \
         stream-json`), which is W3's transport."
    ))
}

// ------------------------------------------------------- model pin verdict

/// The verdict of this transport's **launch-time** pin verification.
///
/// Stronger than every sibling's because the evidence is: `init.model` echoes
/// the resolved model on line 1, before any model output [packet 1, W1 P2].
/// Comparison is **exact string equality** — agy's ids are flat
/// (`gemini-3.7-flash-low`, per the zero-quota `agy models`), with no provider
/// prefix, so none of opencode's slash-splitting applies.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PinVerdict {
    /// No pin was requested; nothing to verify.
    Unpinned,
    /// `init.model` equals the requested pin. Carries the served id.
    Honored(String),
    /// `init.model` names something else. Carries what ran.
    Substituted(String),
    /// No `init.model` to check against. The pin is recorded as attempted,
    /// never as honored — and, unlike a substitution, this is **not** by itself
    /// a stage failure.
    Attempted(String),
}

impl PinVerdict {
    /// Journal/evidence rendering.
    fn as_json(&self, requested: Option<&str>) -> Value {
        match self {
            PinVerdict::Unpinned => json!({"verdict": "unpinned"}),
            PinVerdict::Honored(served) => {
                json!({"verdict": "honored", "requested": requested, "served": served})
            }
            PinVerdict::Substituted(served) => {
                json!({"verdict": "substituted", "requested": requested, "ran": served})
            }
            PinVerdict::Attempted(detail) => {
                json!({"verdict": "attempted", "requested": requested, "detail": detail})
            }
        }
    }

    /// The failure sentence a substitution owes OBSERVE, or `None` for every
    /// other verdict. Only a substitution is a stage failure: a pin that could
    /// not be checked is missing evidence, and failing a stage on missing
    /// evidence would be this adapter deciding a Work's fate on something it
    /// never saw.
    fn mismatch(&self, requested: Option<&str>) -> Option<String> {
        match self {
            PinVerdict::Substituted(served) => Some(format!(
                "model pin not honored: requested {}, agy's init line names {served} as the model \
                 serving this conversation",
                requested.unwrap_or("<none>")
            )),
            PinVerdict::Unpinned | PinVerdict::Honored(_) | PinVerdict::Attempted(_) => None,
        }
    }
}

fn verify_pin_from_init(requested: Option<&str>, init_model: Option<&str>) -> PinVerdict {
    let Some(requested) = requested else {
        return PinVerdict::Unpinned;
    };
    match init_model.filter(|model| !model.is_empty()) {
        None => PinVerdict::Attempted(
            "agy's init line named no model, so nothing here evidences which model is serving this \
             conversation"
                .to_string(),
        ),
        Some(served) if served == requested => PinVerdict::Honored(served.to_string()),
        Some(served) => PinVerdict::Substituted(served.to_string()),
    }
}

// ------------------------------------------------ permission posture

/// The panel amendment's permission posture, computed at launch from the
/// `init` line and the configured injection channel.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PermissionPosture {
    /// `init.permission_mode`, verbatim.
    effective_mode: Option<String>,
    /// True unless a **measured** non-denying mode is echoed. The allowlist
    /// ([`NON_DENYING_MODES`]) is empty, so this is `true` for every mode seen
    /// so far — fail closed: an unrecognized permission mode is not evidence of
    /// permission.
    denies_tools: bool,
    /// The channel actually used, or the operator-config requirement.
    injection: String,
}

impl PermissionPosture {
    fn from_init(mode: Option<&str>, settings_home: Option<&Path>) -> Self {
        let denies_tools = !mode.is_some_and(|mode| NON_DENYING_MODES.contains(&mode));
        Self {
            effective_mode: mode.map(str::to_string),
            denies_tools,
            injection: match settings_home {
                Some(home) => format!(
                    "settings home composed as HOME={} (measured channel, W1 P2)",
                    home.display()
                ),
                None => "none (operator config required: this run reads the daemon user's own \
                         ~/.gemini/antigravity-cli/settings.json)"
                    .to_string(),
            },
        }
    }

    fn as_json(&self) -> Value {
        json!({
            "effective_mode": self.effective_mode,
            "denies_tools": self.denies_tools,
            "injection": self.injection,
        })
    }
}

// ---------------------------------------------------------------- decoding

/// One finished turn's terminal shape, before any process-exit evidence is
/// folded in (that happens in [`classify_terminal`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum Terminal {
    /// No `result` event was seen at all — the measured SIGKILL shape
    /// [W1 P4].
    #[default]
    None,
    /// A terminal `result` was seen, with agy's own `status` vocabulary.
    Status {
        status: String,
        response: String,
        error: String,
    },
}

/// The whole decoder: folds one turn's NDJSON stream into normalized events
/// plus the honest counts the narration rule needs. **Pure** — no I/O, no
/// process — which is what makes the fixture-driven suite possible with no
/// `agy` binary anywhere in the loop.
#[derive(Debug, Default)]
struct TurnAccumulator {
    /// From the `init` event's own top-level `conversation_id`.
    conversation_id: Option<String>,
    init_model: Option<String>,
    init_permission_mode: Option<String>,
    init_cwd: Option<String>,
    init_tool_count: usize,
    /// `step_update` lines seen.
    steps: u32,
    /// `agent_response` steps that resolved carrying **non-empty** text.
    ///
    /// Deliberately not "every agent_response step": [W1 P2 control] measured a
    /// textless `agent_response` step on a turn that produced no output at all,
    /// so counting those would let an empty-SUCCESS terminal slip past
    /// [`classify_terminal`] arm 3. A textless step is not agent output.
    agent_response_steps: u32,
    /// `text_delta` fragments seen — counted, never emitted individually.
    text_deltas: u32,
    /// Tool steps that reached a resolved state.
    tool_steps: u32,
    /// Tool ids a `tool.requested` has already been emitted for, so a harness
    /// that emits an in-flight `tool_info` followed by a resolved one cannot
    /// produce two requests for one call.
    requested_tools: BTreeSet<String>,
    /// Tools whose own `tool_info.error` evidenced a permission denial
    /// ([`tool_denial_evidence`] — the packet's 1.1.17 shape).
    denied_tools: Vec<String>,
    /// Text accumulated per `step_index`, because one step can emit `ACTIVE`
    /// with a partial `text_delta` and then `DONE` with the rest (measured, the
    /// W1 json-schema capture). Keyed rather than single-slotted so an
    /// interleaved build cannot silently merge two steps' text.
    step_texts: BTreeMap<i64, String>,
    /// `result.response`, or the last resolved `agent_response` step's text.
    last_response: Option<String>,
    last_step_usage: Option<Value>,
    terminal_usage: Option<Value>,
    /// `result.structured_output`, verbatim when `--json-schema` was composed.
    structured_output: Option<Value>,
    terminal: Terminal,
    last_error: Option<String>,
    /// [W1 P0.5]: a slash command emits this before its `result`. It cannot
    /// occur once `--disable-slash-commands` is composed; decoding it
    /// defensively costs one match arm and turns a would-be silent
    /// empty-SUCCESS into a named fact.
    saw_command_result: bool,
    /// Event kinds and `step_type`s this decoder does not know — counted and
    /// named by their own wire string, never interpreted. They are in the raw
    /// blob by construction.
    unknown_events: Vec<String>,
    unparsed_lines: u32,
}

impl TurnAccumulator {
    fn new() -> Self {
        Self::default()
    }

    /// Ingest one already-parsed line, returning the normalized events it
    /// produced. Malformed-line counting happens in the caller.
    fn ingest_line(&mut self, value: &Value) -> Vec<NativeEvent> {
        let mut out = Vec::new();
        match value.get("event").and_then(Value::as_str) {
            Some("init") => self.ingest_init(value),
            Some("step_update") => self.ingest_step_update(value.get("step_update"), &mut out),
            Some("result") => self.ingest_result(value.get("result"), &mut out),
            Some("command_result") => self.saw_command_result = true,
            Some(other) => self.unknown_events.push(other.to_string()),
            None => self.unknown_events.push("<no event field>".to_string()),
        }
        out
    }

    /// `init` is line 1 and carries identity, the resolved model, the cwd, the
    /// tool roster and the permission mode — before any model output
    /// [packet 1, W1 P2]. It emits **no** normalized event: sergeant's §27
    /// vocabulary has no "session init" kind and minting one is a core edit
    /// (K2). The facts reach the journal three other ways — the
    /// [`FirstTurnSignal`], the `conversation.turn.ended` payload, and (when
    /// the mode denies) the launch-time notice.
    fn ingest_init(&mut self, value: &Value) {
        // An `init` whose conversation_id is empty is NOT an identity: treat it
        // as absent. (The zero-quota refusals all carry `conversation_id: ""`.)
        self.conversation_id = value
            .get("conversation_id")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .map(str::to_string);
        let init = value.get("init");
        self.init_model = init
            .and_then(|i| i.get("model"))
            .and_then(Value::as_str)
            .filter(|model| !model.is_empty())
            .map(str::to_string);
        self.init_permission_mode = init
            .and_then(|i| i.get("permission_mode"))
            .and_then(Value::as_str)
            .filter(|mode| !mode.is_empty())
            .map(str::to_string);
        self.init_cwd = init
            .and_then(|i| i.get("cwd"))
            .and_then(Value::as_str)
            .map(str::to_string);
        self.init_tool_count = init
            .and_then(|i| i.get("tools"))
            .and_then(Value::as_array)
            .map(Vec::len)
            .unwrap_or(0);
    }

    fn ingest_step_update(&mut self, step: Option<&Value>, out: &mut Vec<NativeEvent>) {
        let Some(step) = step else {
            self.unknown_events
                .push("step_update (no step_update body)".to_string());
            return;
        };
        self.steps += 1;
        let index = step.get("step_index").and_then(Value::as_i64).unwrap_or(-1);
        let state = step
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let step_type = step
            .get("step_type")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let resolved = state == "DONE" || state == "ERROR";

        // Usage first, and for every step type: it is the one field that is
        // known DURING the turn rather than only at its end (the R4 delta).
        if let Some(usage) = step.get("usage") {
            self.last_step_usage = Some(usage.clone());
            out.push(NativeEvent {
                kind: KIND_USAGE_UPDATED.to_string(),
                payload: json!({
                    "conversation_id": self.conversation_id,
                    "step": index,
                    "step_type": step_type,
                    "state": state,
                    "scope": "step",
                    "usage": usage,
                }),
            });
        }

        // The ONE tool path. Routed on the presence of a `tool_info` object
        // rather than on `step_type == "tool"`, so a build that renames the
        // step type still produces tool events from the same structured record
        // — and so no branch anywhere can reach this from a `text_delta`.
        if let Some(tool_info) = step.get("tool_info") {
            self.ingest_tool_info(index, &state, resolved, tool_info, out);
            return;
        }

        match step_type.as_str() {
            // Our own prompt echoed back. Counted, no event.
            "user_input" => {}
            "agent_response" => {
                if let Some(delta) = step.get("text_delta").and_then(Value::as_str) {
                    self.text_deltas += 1;
                    self.step_texts.entry(index).or_default().push_str(delta);
                }
                if resolved {
                    let text = self.step_texts.remove(&index).unwrap_or_default();
                    if !text.is_empty() {
                        self.agent_response_steps += 1;
                        let normalized = normalize_pty(&text);
                        self.last_response = Some(normalized.clone());
                        out.push(NativeEvent {
                            kind: KIND_CONVERSATION_ASSISTANT_COMPLETED.to_string(),
                            payload: json!({
                                "conversation_id": self.conversation_id,
                                "text": normalized,
                                "step": index,
                                "state": state,
                                "duration_seconds": step
                                    .get("duration_seconds")
                                    .cloned()
                                    .unwrap_or(Value::Null),
                            }),
                        });
                    }
                }
            }
            // `checkpoint`, `system_message` and `finish` are all measured step
            // types this vocabulary has no kind for [packet 1, W1 P4, W1
            // json-schema capture]. Named by their own wire string and never
            // interpreted — they are in the raw blob by construction.
            other => self.unknown_events.push(format!("step_type:{other}")),
        }
    }

    /// **The only code path in this module that produces `tool.*` events**, and
    /// no branch anywhere reads a `text_delta` as evidence that anything ran.
    /// The narration rule is structural here, not stylistic.
    fn ingest_tool_info(
        &mut self,
        index: i64,
        state: &str,
        resolved: bool,
        tool_info: &Value,
        out: &mut Vec<NativeEvent>,
    ) {
        let name = tool_info
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        // `tool_info` carried no `id` in any measured capture [W1 P2/P3/P4], so
        // the step index is the identity. Kept as a lookup rather than
        // hardcoded so a build that starts emitting one is honoured.
        let id = tool_info
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| format!("step-{index}"));
        if self.requested_tools.insert(id.clone()) {
            out.push(NativeEvent {
                kind: KIND_TOOL_REQUESTED.to_string(),
                payload: json!({
                    "id": id,
                    "name": name,
                    // Verbatim: `\r\n` inside a parameter value is the actor's
                    // own data, never this module's to normalize.
                    "input": tool_info.get("parameters").cloned().unwrap_or(Value::Null),
                }),
            });
        }
        if !resolved {
            return;
        }
        self.tool_steps += 1;
        let error = tool_info.get("error").cloned();
        let denied = tool_denial_evidence(tool_info);
        if denied {
            self.denied_tools.push(name.clone());
        }
        let output = tool_info
            .get("output")
            .and_then(Value::as_str)
            .unwrap_or("");
        out.push(NativeEvent {
            kind: KIND_TOOL_COMPLETED.to_string(),
            payload: json!({
                "tool_use_id": id,
                "name": name,
                "state": state,
                "is_error": state == "ERROR" || error.is_some(),
                "error": error.unwrap_or(Value::Null),
                "denied": denied,
                // Measured: a tool auto-denied at 1.1.19 resolves DONE with
                // *no* output key at all, which is why this is reported
                // separately from an empty string.
                "has_output": tool_info.get("output").is_some(),
                "output_tail": truncate(&normalize_pty(output), TOOL_OUTPUT_TAIL).to_string(),
            }),
        });
    }

    fn ingest_result(&mut self, result: Option<&Value>, out: &mut Vec<NativeEvent>) {
        let Some(result) = result else {
            self.unknown_events
                .push("result (no result body)".to_string());
            return;
        };
        if self.conversation_id.is_none() {
            self.conversation_id = result
                .get("conversation_id")
                .and_then(Value::as_str)
                .filter(|id| !id.is_empty())
                .map(str::to_string);
        }
        let status = result
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let response = result
            .get("response")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let error = result
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        self.structured_output = result.get("structured_output").cloned();
        if !response.trim().is_empty() {
            self.last_response = Some(normalize_pty(&response));
        }
        if !error.is_empty() {
            self.last_error = Some(error.clone());
        }
        if let Some(usage) = result.get("usage") {
            self.terminal_usage = Some(usage.clone());
            out.push(NativeEvent {
                kind: KIND_USAGE_UPDATED.to_string(),
                payload: json!({
                    "conversation_id": self.conversation_id,
                    "scope": "turn",
                    "usage": usage,
                    "num_turns": result.get("num_turns").cloned().unwrap_or(Value::Null),
                    "duration_seconds": result
                        .get("duration_seconds")
                        .cloned()
                        .unwrap_or(Value::Null),
                }),
            });
        }
        if status == "ERROR" || status == "INVALID" {
            out.push(NativeEvent {
                kind: KIND_TURN_HARNESS_ERROR.to_string(),
                payload: json!({
                    "phase": "typed_terminal",
                    "status": status,
                    "error": error,
                }),
            });
        }
        self.terminal = Terminal::Status {
            status,
            response,
            error,
        };
    }

    /// The status string this turn's terminal carried, if any.
    fn status(&self) -> Option<&str> {
        match &self.terminal {
            Terminal::None => None,
            Terminal::Status { status, .. } => Some(status.as_str()),
        }
    }
}

/// Rule 1 of the denial detector: the packet's measured 1.1.17 signature — a
/// typed `tool_info.error {type: "TOOL_ERROR", message: "permission check
/// failed … user denied permission to run command"}` [packet 2].
///
/// **This rule does not fire at 1.1.19** [W1 P2]: an auto-denied tool there
/// resolves `DONE` with no `error` object at all. It is kept because a build
/// that emits the packet's shape must still be handled, and because keeping it
/// makes the pair of detectors — typed and stderr — the honest record of what
/// each version does.
///
/// **Deliberately narrow.** An ordinary nonzero command exit inside a
/// *permitted* tool call is normal agent work and must not trigger it; only a
/// permission denial does. A rule that fired on every failed command would
/// poison every honest turn.
fn tool_denial_evidence(tool_info: &Value) -> bool {
    let Some(error) = tool_info.get("error") else {
        return false;
    };
    let kind = error.get("type").and_then(Value::as_str).unwrap_or("");
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_lowercase();
    kind == "TOOL_ERROR" && message.contains("permission")
}

/// Rule 2 of the denial detector, and **the one that actually fires at
/// 1.1.19** [W1 P2 control and W1 P3 turn 1 — two live reproductions, and only
/// two; their stderr captures are byte-identical]: the drained stderr
/// carries the CLI's own auto-denial notice.
///
/// The substrings are taken from the binary's own format string —
/// `%s required the %s %s that headless mode cannot prompt for, so %s
/// auto-denied. Add an allow-rule under permissions.allow in settings.json
/// (e.g. %s)` — so they survive the parts of the sentence that are
/// tool-specific, and any one of them is enough.
fn denial_evidence_in_stderr(stderr: &str) -> bool {
    let lower = stderr.to_lowercase();
    lower.contains("auto-denied")
        || lower.contains("that headless mode cannot prompt for")
        || lower.contains("permissions.allow")
}

/// The measured stderr warning that a `--conversation <id>` was **not** found
/// and a fresh conversation was started instead [W1 P0.6] — the second,
/// independent detector of the silent-resume fork.
fn resume_fork_warning_in_stderr(stderr: &str) -> bool {
    let lower = stderr.to_lowercase();
    lower.contains("conversation") && lower.contains("not found")
}

/// agy's tool output is pty-captured: `"agy-w1-probe\r\n"` [packet 3, W1 P2].
/// Normalize CRLF to LF for every string this module puts into an event
/// payload — and for **nothing else**. The raw blob keeps the bytes the harness
/// wrote (§20): a normalized archive is an archive that has already been
/// interpreted. Not applied to `tool_info.parameters` either, which is
/// structured JSON whose `\r\n` is the actor's own data.
fn normalize_pty(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// The shape one finished turn resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TerminalOutcome {
    Completed,
    /// The harness said the turn failed. **Only** an explicit statement lands
    /// here.
    Failed {
        reason: String,
    },
    /// No conclusion about the stage, and the conversation stays resumable —
    /// measured [W1 P4]: a SIGKILLed turn left the conversation fully
    /// resumable with recall.
    InterruptedRunning,
    /// §25's ambiguity, failing closed.
    AmbiguousUnknown,
}

/// One settled outcome's stable, snake_case name — the string the journal
/// carries, kept out of `{:?}` so a payload consumer is not reading a derived
/// `Debug` rendering that changes shape whenever a field is added.
fn terminal_outcome_label(outcome: &TerminalOutcome) -> &'static str {
    match outcome {
        TerminalOutcome::Completed => "completed",
        TerminalOutcome::Failed { .. } => "failed",
        TerminalOutcome::InterruptedRunning => "interrupted_running",
        TerminalOutcome::AmbiguousUnknown => "ambiguous_unknown",
    }
}

/// Fold one turn's stream evidence, its process exit, the interrupt bit **and
/// its drained stderr** into an outcome.
///
/// The stderr argument is a declared departure from the W1 spec's
/// three-argument signature, forced by measurement: at 1.1.19 the *only*
/// evidence that a tool was auto-denied is a plain-text stderr notice
/// ([`denial_evidence_in_stderr`]), so a classifier that cannot see stderr
/// cannot implement the panel's own honesty rule.
///
/// **The order of the arms is the argument**: an explicit statement outranks
/// everything; an honesty rule outranks a status we would otherwise believe; a
/// kill we asked for outranks silence; ambiguity fails closed.
///
/// **A nonzero exit is never, by itself, a stage failure**, and — symmetrically
/// — **a `SUCCESS` status is never, by itself, a completion**. §15's
/// load-bearing invariant is that "a backend cannot complete a stage by
/// exiting, and cannot fail one by dying". The exit code travels into the
/// evidence of every arm where a human can act on it. Note the measured
/// asymmetries: the packet's ERROR shape carries exit 1 *with* a typed error,
/// while 1.1.19's auto-denial carries exit **0** with a `CANCELED` status and
/// nothing typed at all.
fn classify_terminal(
    acc: &TurnAccumulator,
    exit_code: Option<i32>,
    interrupted: bool,
    stderr: &str,
) -> TerminalOutcome {
    let denial = !acc.denied_tools.is_empty() || denial_evidence_in_stderr(stderr);
    let Terminal::Status {
        status,
        response,
        error,
    } = &acc.terminal
    else {
        // Arms 9 and 10: no `result` event at all.
        return if interrupted {
            TerminalOutcome::InterruptedRunning
        } else {
            TerminalOutcome::AmbiguousUnknown
        };
    };
    match status.as_str() {
        // Arm 1 — the harness's own explicit statement, the ONLY route to
        // Failed. [W1 P5] measured `--print-timeout` expiry landing here with
        // `error: "timeout waiting for response"`.
        "ERROR" | "INVALID" => TerminalOutcome::Failed {
            reason: if error.is_empty() {
                format!("agy reported status {status} with no error text")
            } else {
                error.clone()
            },
        },
        "SUCCESS" => {
            // Arm 2 — the SUCCESS-hiding-denied-tools rule. The harness said
            // the turn succeeded while a tool the actor asked for did not run;
            // a stage completed on that basis is a stage completed on work
            // that did not happen.
            if denial {
                return TerminalOutcome::AmbiguousUnknown;
            }
            // Arm 3 — the panel's empty-SUCCESS amendment. Fail-closed by
            // construction and version-independent; see the module doc for why
            // >= 1.1.18 is not an argument for removing it.
            if response.trim().is_empty() && acc.agent_response_steps == 0 {
                return TerminalOutcome::AmbiguousUnknown;
            }
            TerminalOutcome::Completed
        }
        // Arms 5 and 6. Treating an unrequested cancel as our own interrupt
        // would be the adapter claiming authorship of an event it did not
        // cause — and at 1.1.19 an unrequested CANCELED is precisely the
        // auto-denial shape [W1 P2].
        "CANCELED" | "CANCELLED" | "INTERRUPTED" => {
            if interrupted {
                TerminalOutcome::InterruptedRunning
            } else {
                TerminalOutcome::AmbiguousUnknown
            }
        }
        // Arms 7 and 8: a terminal event that says "still running" is a
        // contradiction, and an unknown status is §15's fail-closed case. Both
        // echo the literal status into the evidence. `exit_code` is folded in
        // by the caller's evidence string, never by this decision.
        _ => {
            let _ = exit_code;
            TerminalOutcome::AmbiguousUnknown
        }
    }
}

/// A private per-module copy — the sibling adapters' own precedent (each owner
/// keeps its own; `runtime/graph.rs` already has a third).
fn truncate(text: &str, max: usize) -> &str {
    match text.char_indices().nth(max) {
        Some((idx, _)) => &text[..idx],
        None => text,
    }
}

// ------------------------------------------------------ execution state

/// One finished turn's outcome, kept for OBSERVE.
#[derive(Debug, Clone)]
struct TurnOutcome {
    terminal: TerminalOutcome,
    /// Checked *before* the completion branch and fatal whatever else the turn
    /// produced: a turn served by a model the human did not ask for is not a
    /// completed stage, however well it went.
    pin_mismatch: Option<String>,
    pin: Value,
    /// Set when a turn that composed `--conversation <id>` got an `init` line
    /// echoing a **different** id (or none) — the silent-resume fork. Fatal for
    /// the same reason a pin mismatch is.
    resume_mismatch: Option<String>,
    steps: u32,
    agent_response_steps: u32,
    text_deltas: u32,
    tool_steps: u32,
    denied_tools: Vec<String>,
    unknown_events: Vec<String>,
    unparsed_lines: u32,
    saw_command_result: bool,
    summary: Option<String>,
    status: Option<String>,
    last_error: Option<String>,
    exit_code: Option<i32>,
    raw_blob: Option<String>,
    raw_error: Option<String>,
    stderr: String,
}

impl TurnOutcome {
    /// How the §20 archive turned out, rendered for evidence — a ref, a named
    /// failure, or an explicitly empty stream. No value here means "absent for
    /// reasons unknown".
    fn raw_evidence(&self) -> String {
        match (&self.raw_blob, &self.raw_error) {
            (Some(blob), _) => blob.clone(),
            (None, Some(error)) => format!("unarchived ({error})"),
            (None, None) => "unarchived (the turn streamed nothing)".to_string(),
        }
    }

    /// The sentence a denied tool owes the evidence, when there is one.
    fn denial_note(&self) -> Option<String> {
        if self.denied_tools.is_empty() && !denial_evidence_in_stderr(&self.stderr) {
            return None;
        }
        Some(format!(
            "a tool call was auto-denied on this turn (denied_tools={:?}; stderr evidence={}). At \
             1.1.19 the measured shape is a DONE tool step with no output, a CANCELED terminal and \
             exit 0, so the stderr notice is the only machine-readable signal (W1 P2). Add an \
             allow-rule under permissions.allow in the settings home this run reads",
            self.denied_tools,
            denial_evidence_in_stderr(&self.stderr)
        ))
    }
}

/// Turn lifecycle for one execution. Mirrors the sibling adapters' `TurnState`
/// exactly, including why `Unlaunched`/`Adopted` are their own variants rather
/// than a borrowed `Finished` placeholder: fabricating either would have
/// OBSERVE state a fact ("interrupted by request", "a turn is running") that
/// never happened.
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
    /// The last turn finished (or was killed) and left this outcome. Boxed to
    /// keep the enum from carrying `TurnOutcome`'s owned fields on every
    /// variant that is not `Finished`.
    Finished(Box<TurnOutcome>),
}

/// Adapter-side record of one execution (one durable agy conversation).
#[derive(Debug)]
struct AgyExecution {
    /// `None` only during the narrow in-process window between spawning turn 1
    /// and its `init` line arriving — never visible to a caller, since LAUNCH
    /// does not return a handle until this is `Some`.
    conversation_id: Option<String>,
    work_id: String,
    cwd: PathBuf,
    model: Option<String>,
    executable: PathBuf,
    env: BTreeMap<String, String>,
    settings_home: Option<PathBuf>,
    json_schema: Option<String>,
    /// Recorded once at LAUNCH and carried into every turn's evidence: this
    /// adapter composes no flag for out-of-surface bindings, so the fact has to
    /// be visible somewhere rather than assumed away.
    bindings_outside_cwd: Vec<PathBuf>,
    /// The launch-time permission posture, read off the `init` line.
    posture: Option<PermissionPosture>,
    turns: u32,
    turn: TurnState,
    /// The process group id of the most recent turn, recorded at **spawn**.
    /// `process_group(0)` makes the turn's direct child its own group leader,
    /// so this is that child's pid. Kept here rather than read back out of
    /// `TurnState::InFlight` at kill time, because the group can outlive the
    /// leader (opencode probe 11's lesson, carried without re-deriving it).
    turn_pgid: Option<u32>,
    stopped: bool,
    interrupt_requested: bool,
    reader: Option<std::thread::JoinHandle<()>>,
}

#[derive(Debug, Default)]
struct AdapterState {
    executions: BTreeMap<String, AgyExecution>,
}

/// One outcome of spawning turn 1, delivered from the reader thread back to the
/// LAUNCH call blocking on it.
enum FirstTurnSignal {
    /// The `init` line landed: identity, resolved model, permission mode.
    Initialized {
        conversation_id: String,
        model: Option<String>,
        permission_mode: Option<String>,
    },
    /// A terminal `result` arrived with **no preceding `init`** — the harness
    /// refused before minting identity (an invalid model, an empty prompt, a
    /// slash-command answer). Typed, and strictly better than a bare exit code:
    /// the invalid-model error enumerates the whole catalog [W1 P0.3 row A].
    RefusedBeforeIdentity {
        status: String,
        error: String,
        exit_code: Option<i32>,
    },
    /// The process died with neither an `init` line nor a terminal.
    ExitedWithoutInit {
        exit_code: Option<i32>,
        stderr: String,
        raw_blob: Option<String>,
    },
}

/// Everything one spawn needs, snapshotted out of adapter state under the lock.
/// A named struct rather than a tuple: nearly every field is a
/// `PathBuf`/`String`, so a positional tuple would let two same-typed slots
/// swap silently and still type-check.
struct SpawnPlan {
    executable: PathBuf,
    cwd: PathBuf,
    env: BTreeMap<String, String>,
    settings_home: Option<PathBuf>,
    json_schema: Option<String>,
    conversation_id: Option<String>,
    model: Option<String>,
    first_turn: bool,
    work_id: String,
    bindings_outside_cwd: Vec<PathBuf>,
    instruction_policy: Option<String>,
}

// -------------------------------------------------------------- the backend

/// The Antigravity backend.
pub struct AgyBackend {
    config: AgyConfig,
    probe_outcome: OnceLock<ProbeOutcome>,
    state: Arc<Mutex<AdapterState>>,
    sink: Mutex<Option<EventSink>>,
}

impl std::fmt::Debug for AgyBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgyBackend")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl AgyBackend {
    /// Build the adapter. Probing is lazy (first PROBE/PREPARE), so
    /// constructing one costs nothing on daemons that never route to it.
    pub fn new(config: AgyConfig) -> Self {
        Self {
            config,
            probe_outcome: OnceLock::new(),
            state: Arc::new(Mutex::new(AdapterState::default())),
            sink: Mutex::new(None),
        }
    }

    /// Install the event sink normalized events are pushed through (§27).
    pub fn set_event_sink(&self, sink: EventSink) {
        *self.sink.lock().expect("agy sink lock") = Some(sink);
    }

    /// Execution ids this adapter currently holds state for — the diagnostic
    /// answer to "did a refused LAUNCH leave a phantom execution behind?".
    pub fn tracked_executions(&self) -> Vec<String> {
        self.lock().executions.keys().cloned().collect()
    }

    /// Run the version/grammar/config probe once and cache the outcome. Every
    /// gate is offline or **zero-quota**: `--version`, `--help`, and one
    /// `-p "/config" --output-format json` which agy answers without starting a
    /// turn, spending quota or leaving a conversation behind
    /// [changelog 1.1.12, W1 P0.2].
    fn probe_outcome(&self) -> &ProbeOutcome {
        self.probe_outcome.get_or_init(|| self.run_probe())
    }

    /// One `--help`-shaped invocation's text, **stdout and stderr
    /// concatenated**. Measured [W1 P0.1]: at 1.1.19 `agy --help` writes to
    /// **stderr and only stderr** — which is the opposite of what the W1 spec
    /// recorded, and exactly why both streams are read. A build that moves the
    /// text back to stdout must not become a spurious refusal.
    fn help_text(&self, args: &[&str]) -> Result<String, String> {
        let exe = &self.config.executable;
        let mut command = Command::new(exe);
        command.args(args).stdin(Stdio::null());
        apply_env(
            &mut command,
            &self.config.env,
            self.config.settings_home.as_deref(),
        );
        let out = command.output().map_err(|e| {
            format!(
                "capability probe: cannot run {exe:?} {}: {e} (kind: {:?})",
                args.join(" "),
                e.kind()
            )
        })?;
        Ok(format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ))
    }

    /// The zero-quota `/config` read. Best-effort by construction: a CLI that
    /// cannot answer it is not refused, it is a CLI whose effective
    /// configuration this adapter cannot report. Bounded by
    /// [`CONFIG_PROBE_BUDGET`] — see its doc for why an unbounded wait here is
    /// not safe to run inside registration.
    fn read_config_probe(&self) -> ConfigProbe {
        let mut command = Command::new(&self.config.executable);
        command
            .args(["-p", "/config", "--output-format", "json"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        apply_env(
            &mut command,
            &self.config.env,
            self.config.settings_home.as_deref(),
        );
        let Ok(mut child) = command.spawn() else {
            return ConfigProbe::default();
        };
        let Some(mut stdout) = child.stdout.take() else {
            return ConfigProbe::default();
        };
        // Piped but deliberately unread here: a probe that never touches this
        // is not a probe whose stderr this adapter has ever needed. Drained on
        // its own thread purely so a child that writes to it does not block on
        // (or SIGPIPE from) a closed pipe while this call is still waiting on
        // stdout.
        if let Some(mut stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                let mut sink = Vec::new();
                let _ = stderr.read_to_end(&mut sink);
            });
        }
        let child = Arc::new(Mutex::new(child));
        let (tx, rx) = std::sync::mpsc::sync_channel::<Vec<u8>>(1);
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = stdout.read_to_end(&mut buf);
            let _ = tx.send(buf);
        });
        let stdout_bytes = match rx.recv_timeout(CONFIG_PROBE_BUDGET) {
            Ok(buf) => buf,
            Err(_) => {
                // Most likely cause, per the measurement in `CONFIG_PROBE_BUDGET`'s
                // doc: an unauthenticated `agy` blocked this call on an
                // interactive login prompt. Killed and treated as any other
                // probe failure — never propagated as a hang.
                let _ = child.lock().expect("agy config-probe child lock").kill();
                Vec::new()
            }
        };
        let _ = child.lock().expect("agy config-probe child lock").wait();
        serde_json::from_slice::<Value>(&stdout_bytes)
            .map(|value| decode_config_probe(&value))
            .unwrap_or_default()
    }

    fn run_probe(&self) -> ProbeOutcome {
        let exe = &self.config.executable;
        let mut version_command = Command::new(exe);
        version_command.arg("--version").stdin(Stdio::null());
        apply_env(
            &mut version_command,
            &self.config.env,
            self.config.settings_home.as_deref(),
        );
        let version_out = match version_command.output() {
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
                    config: ConfigProbe::default(),
                };
            }
        };
        // agy prints the version on stdout; stderr is concatenated for the same
        // reason `help_text` reads both — a build that moves it must not become
        // an unparseable-version refusal.
        let version_text = format!(
            "{}{}",
            String::from_utf8_lossy(&version_out.stdout),
            String::from_utf8_lossy(&version_out.stderr)
        )
        .trim()
        .to_string();
        let Some(triple) = parse_agy_version(&version_text) else {
            return ProbeOutcome {
                available: false,
                detail: format!(
                    "capability probe: cannot parse a version from {exe:?} --version output \
                     {version_text:?}; refusing an unmeasurable CLI"
                ),
                version: None,
                provenance: None,
                config: ConfigProbe::default(),
            };
        };
        let canonical = format!("{}.{}.{}", triple.0, triple.1, triple.2);
        let provenance = if triple >= MEASURED_FLOOR {
            VersionProvenance::Measured
        } else {
            VersionProvenance::BelowFloor
        };

        let help = match self.help_text(&["--help"]) {
            Ok(text) => text,
            Err(detail) => {
                return ProbeOutcome {
                    available: false,
                    detail,
                    version: Some(canonical),
                    provenance: Some(provenance),
                    config: ConfigProbe::default(),
                };
            }
        };
        let missing = missing_entries(&help, REQUIRED_FLAGS);
        if !missing.is_empty() {
            return ProbeOutcome {
                available: false,
                detail: format!(
                    "capability probe: {exe:?} --help (version {version_text}) is missing required \
                     flag(s) {}; this launch grammar was never measured against it",
                    missing.join(", ")
                ),
                version: Some(canonical),
                provenance: Some(provenance),
                config: ConfigProbe::default(),
            };
        }

        let config = self.read_config_probe();
        let version_clause = match provenance {
            VersionProvenance::Measured => format!("agy {canonical}"),
            VersionProvenance::BelowFloor => format!(
                "agy {canonical}; usable, but BELOW the measured floor {}.{}.{} — every \
                 behavioural claim in this adapter (the NDJSON grammar, resume, the permission \
                 posture, the terminal shapes) was measured at or above it and has not been \
                 re-measured here. Capabilities carry unmeasured provenance; upgrade to \
                 >= {}.{}.{} or expect surprises to be findings, not failures.",
                MEASURED_FLOOR.0,
                MEASURED_FLOOR.1,
                MEASURED_FLOOR.2,
                MEASURED_FLOOR.0,
                MEASURED_FLOOR.1,
                MEASURED_FLOOR.2,
            ),
        };
        let mut detail = format!(
            "{version_clause}; all {} required flags present; transport: {}",
            REQUIRED_FLAGS.len(),
            Transport::Print.as_str(),
        );
        // §11.3: the posture reaches `sgt doctor` (W2's reader) before any Work
        // runs, not only the turn that discovers it.
        if config.read {
            detail.push_str(&format!(
                "; effective toolPermission={:?}, {} allow-rule(s), {} trusted workspace(s), \
                 allowNonWorkspaceAccess={} (read zero-quota via `-p \"/config\"`)",
                config.tool_permission.as_deref().unwrap_or("<unset>"),
                config.allow_rules,
                config.trusted_workspaces.len(),
                config.allow_non_workspace_access,
            ));
        } else {
            detail.push_str(
                "; the zero-quota `/config` read did not parse, so this adapter cannot report the \
                 harness's effective permission configuration",
            );
        }
        match &self.config.settings_home {
            Some(home) => detail.push_str(&format!(
                "; permission policy is injected per launch via HOME={} (the measured channel, \
                 W1 P2) — note that overriding HOME also relocates the credential and \
                 conversation stores",
                home.display()
            )),
            None => detail.push_str(
                "; no settings home configured: this daemon runs on the operator's own agy \
                 configuration, and W1 P2 measured the default request-review mode auto-denying \
                 every `command` tool not covered by an allow-rule",
            ),
        }
        ProbeOutcome {
            available: true,
            detail,
            version: Some(canonical),
            provenance: Some(provenance),
            config,
        }
    }

    /// §14 applied to this CLI. `config_home` is **refused here, not ignored**:
    /// no agy config-home *variable* was measured, and honouring the field by
    /// guessing one would make the human's launch decision silently do nothing.
    fn launch_config(&self, profile: Option<&Profile>) -> Result<LaunchConfig, BackendError> {
        if let Some(profile) = profile {
            if profile.config_home.is_some() {
                return Err(self.err_failed(format!(
                    "profile {:?}: config_home is not supported by this adapter. No agy \
                     environment variable naming a config home has been measured — a `strings` \
                     scan of the binary names none (W1 P2) — and honouring this field by guessing \
                     one would make the human's launch decision silently do nothing. The measured \
                     channel is a settings HOME: set the backend's own `settings_home`, whose \
                     directory is composed as HOME for every turn, or use the profile's `env`.",
                    profile.name
                )));
            }
            if profile.options.contains_key("agy_agent") {
                return Err(self.err_failed(format!(
                    "profile {:?}: option agy_agent is not supported by this adapter. agy's \
                     `--agent` is not wired in this wave: W1 P6 defined a custom agent at the \
                     documented workspace path and `/agents` still answered {{\"agents\":[]}}, so \
                     the mechanism does not work as documented on this host; and even if it did, \
                     an agent applied to the first turn has to be re-applied on every \
                     `--conversation` turn for the conversation to stay under it, which is \
                     unmeasured (opencode's precedent).",
                    profile.name
                )));
            }
        }
        let executable = profile
            .and_then(|p| p.executable.clone())
            .unwrap_or_else(|| self.config.executable.clone());
        let mut env = self.config.env.clone();
        if let Some(profile) = profile {
            for (key, value) in &profile.env {
                env.insert(key.clone(), value.clone());
            }
        }
        Ok(LaunchConfig { executable, env })
    }

    fn err_failed(&self, detail: impl Into<String>) -> BackendError {
        BackendError::Failed {
            backend: AGY_BACKEND_NAME.to_string(),
            detail: detail.into(),
        }
    }

    fn err_unknown(&self, execution_id: &str) -> BackendError {
        BackendError::UnknownExecution {
            backend: AGY_BACKEND_NAME.to_string(),
            execution_id: execution_id.to_string(),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, AdapterState> {
        self.state.lock().expect("agy adapter state lock")
    }

    /// §25's identity rule: an execution is resolved by sergeant's id *and* the
    /// native (conversation) identity the handle carries.
    fn check_identity(
        &self,
        state: &AdapterState,
        handle: &ExecutionHandle,
    ) -> Result<(), BackendError> {
        let execution = state
            .executions
            .get(&handle.execution_id)
            .ok_or_else(|| self.err_unknown(&handle.execution_id))?;
        if handle.native_id.as_deref() != execution.conversation_id.as_deref() {
            return Err(self.err_unknown(&handle.execution_id));
        }
        Ok(())
    }

    fn emit(&self, execution_id: &str, work_id: &str, kind: &str, payload: Value) {
        let sink = self.sink.lock().expect("agy sink lock").clone();
        if let Some(sink) = sink {
            sink(EventDraft {
                source: EventSource::new("backend", AGY_BACKEND_NAME),
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

    fn spawn_plan(&self, execution_id: &str) -> Result<SpawnPlan, BackendError> {
        let state = self.lock();
        let execution = state
            .executions
            .get(execution_id)
            .ok_or_else(|| self.err_unknown(execution_id))?;
        Ok(SpawnPlan {
            executable: execution.executable.clone(),
            cwd: execution.cwd.clone(),
            env: execution.env.clone(),
            settings_home: execution.settings_home.clone(),
            json_schema: execution.json_schema.clone(),
            conversation_id: execution.conversation_id.clone(),
            model: execution.model.clone(),
            first_turn: execution.turns == 0,
            work_id: execution.work_id.clone(),
            bindings_outside_cwd: execution.bindings_outside_cwd.clone(),
            instruction_policy: None,
        })
    }

    /// Spawn one turn for an execution already registered in adapter state.
    /// `first_turn_signal`, when present, is used only for turn 1's
    /// synchronization with LAUNCH — SEND passes `None`.
    fn spawn_turn(
        &self,
        execution_id: &str,
        prompt: String,
        instruction_policy: Option<String>,
        first_turn_signal: Option<SyncSender<FirstTurnSignal>>,
    ) -> Result<(), BackendError> {
        let mut plan = self.spawn_plan(execution_id)?;
        plan.instruction_policy = instruction_policy;

        // The prompt is the VALUE of `-p` and rides argv only (W1 P0.3), so the
        // budget is checked before any process exists — PREPARE already refused
        // an over-long first turn, and this catches a SEND's own input.
        if prompt.len() > ARGV_PROMPT_CAP {
            return Err(self.err_failed(format!(
                "this turn's prompt is {} bytes, over the {ARGV_PROMPT_CAP}-byte argv cap this \
                 transport measured (E2BIG at 131072, W1 P0.4). Nothing is truncated.",
                prompt.len()
            )));
        }

        let mut command = Command::new(&plan.executable);
        let expected_conversation = if plan.first_turn {
            command.args(first_turn_argv(
                &prompt,
                plan.model.as_deref(),
                plan.json_schema.as_deref(),
            ));
            None
        } else {
            let conversation = plan.conversation_id.clone().ok_or_else(|| {
                self.err_failed("cannot send: no conversation id recorded for this execution")
            })?;
            command.args(resume_turn_argv(
                &prompt,
                plan.model.as_deref(),
                plan.json_schema.as_deref(),
                &conversation,
            ));
            Some(conversation)
        };
        command
            .current_dir(&plan.cwd)
            // Nothing is read from stdin on this transport (W1 P0.3 row B):
            // leaving it inherited risks a child that blocks on a terminal read.
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        apply_env(&mut command, &plan.env, plan.settings_home.as_deref());
        // Every turn's tool commands run under this process; a new process
        // group is what lets INTERRUPT kill the whole tree. Carried from
        // opencode probe 11 without re-deriving it (R2) — and W1 P4 measured
        // that whether agy keeps tool children in this group is *unmeasured*,
        // which is a reason to keep the group kill, not to drop it.
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }

        let mut child = command
            .spawn()
            .map_err(|e| self.err_failed(format!("cannot spawn {:?}: {e}", plan.executable)))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| self.err_failed("child stdout was not piped"))?;
        let stderr = child.stderr.take();

        // Stderr through a sync channel rather than a shared buffer (issue
        // #46): both pipes reach EOF at the same instant, and a reader that
        // snapshots a buffer the moment stdout closes is racing the thread
        // still filling it. Here that buffer holds the auto-denial notice
        // (W1 P2) and the resume-fork warning (W1 P0.6) — the only evidence of
        // either fact anywhere.
        let stderr_rx = stderr.map(|mut stderr| {
            let (tx, rx) = std::sync::mpsc::sync_channel::<String>(1);
            std::thread::spawn(move || {
                let text = read_bounded(&mut stderr, STREAM_MEMORY_CAP);
                let _ = tx.send(text);
            });
            rx
        });

        // Recorded here, at spawn, never derived from the child at kill time —
        // `process_group(0)` above made this child its own group leader, so its
        // pid *is* the group's id, and that id stays the right thing to signal
        // after the child itself has exited.
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
            &plan.work_id,
            KIND_CONVERSATION_USER,
            json!({
                "text": prompt,
                "conversation_id": plan.conversation_id,
                "bindings_outside_cwd": plan.bindings_outside_cwd,
                // Carried, not enforced: agy has no measured
                // `--setting-sources` analog, so a reader can see the policy
                // this Work pinned *and* that this transport composed nothing
                // for it, instead of assuming it was applied.
                "instruction_policy_unenforced": plan.instruction_policy,
            }),
        );

        let reader = TurnReader {
            backend_state: Arc::clone(&self.state),
            sink: self.sink.lock().expect("agy sink lock").clone(),
            data_dir: self.config.data_dir.clone(),
            execution_id: execution_id.to_string(),
            work_id: plan.work_id,
            model: plan.model,
            expected_conversation,
            bindings_outside_cwd: plan.bindings_outside_cwd,
            child: Arc::clone(&child),
            stderr_rx,
            first_turn_signal,
            settings_home: self.config.settings_home.clone(),
        };
        let reader_handle = std::thread::spawn(move || reader.run(stdout));
        if let Some(execution) = self.lock().executions.get_mut(execution_id) {
            execution.reader = Some(reader_handle);
        }
        Ok(())
    }

    /// LAUNCH's own spawn: fires turn 1 and blocks, bounded, for the `init`
    /// line. Does *not* remove adapter state on failure — the caller (`launch`)
    /// owns that, so there is one place a failed launch leaves no phantom.
    fn spawn_first_turn(
        &self,
        execution_id: &str,
        prompt: String,
        instruction_policy: Option<String>,
    ) -> Result<FirstTurnSignal, BackendError> {
        let (tx, rx) = std::sync::mpsc::sync_channel::<FirstTurnSignal>(1);
        self.spawn_turn(execution_id, prompt, instruction_policy, Some(tx))?;
        let budget = self.config.init_line_budget.unwrap_or(INIT_LINE_BUDGET);
        match rx.recv_timeout(budget) {
            Ok(signal) => Ok(signal),
            Err(_) => {
                self.kill_inflight_turn(execution_id);
                Err(self.err_failed(format!(
                    "agy emitted no `init` line within {budget:?}; the turn was killed. The \
                     conversation id is harness-minted and arrives on the init line (which is line \
                     1, before the model's first token), so until one lands there is no identity \
                     to hand back"
                )))
            }
        }
    }

    /// Kill this execution's in-flight turn process, group and all. The group
    /// id is taken whatever the turn state says: a turn that has already ended
    /// can still have left a background command running in its group.
    fn kill_inflight_turn(&self, execution_id: &str) {
        let (pgid, child) = {
            let state = self.lock();
            let Some(execution) = state.executions.get(execution_id) else {
                return;
            };
            let child = match &execution.turn {
                TurnState::InFlight(child) => Some(Arc::clone(child)),
                _ => None,
            };
            (execution.turn_pgid, child)
        };
        kill_turn(pgid, child.as_ref());
    }

    /// §11.3, and the [W1 P3] addition: everything LAUNCH has to say out loud
    /// once the `init` line has landed. Neither notice refuses the launch — a
    /// read-only stage runs fine under a denying mode, and a Work whose surface
    /// is untrusted may never write a file — but both are emitted, journaled
    /// and probe-visible, which is exactly what "reported honestly at launch"
    /// means.
    fn announce_launch_posture(
        &self,
        execution_id: &str,
        work_id: &str,
        cwd: &Path,
        posture: &PermissionPosture,
    ) {
        if posture.denies_tools {
            self.emit(
                execution_id,
                work_id,
                KIND_TURN_HARNESS_ERROR,
                json!({
                    "phase": "permission_mode_denies_tools",
                    "effective_mode": posture.effective_mode,
                    "injection": posture.injection,
                    "detail": "any tool call this turn attempts that is not covered by an \
                               allow-rule in the settings this run reads will be auto-denied, and \
                               the auto-denial CANCELS the whole turn and discards its response \
                               (measured, W1 P2 control; the packet's 1.1.17 hard-deny shape no \
                               longer reproduces). A read-only stage is unaffected. This is a \
                               warning, not a prediction: W1 P2 measured the same \
                               `request-review` mode echoed on both a denied turn and a permitted \
                               one, so the mode string alone decides nothing",
                }),
            );
        }
        let config = &self.probe_outcome().config;
        if config.read
            && !config.allow_non_workspace_access
            && !config.trusted_workspaces.is_empty()
            && !config
                .trusted_workspaces
                .iter()
                .any(|trusted| cwd.starts_with(trusted))
        {
            self.emit(
                execution_id,
                work_id,
                KIND_TURN_HARNESS_ERROR,
                json!({
                    "phase": "cwd_outside_trusted_workspaces",
                    "cwd": cwd,
                    "trusted_workspaces": config.trusted_workspaces,
                    "allow_non_workspace_access": config.allow_non_workspace_access,
                    "detail": "this Work's surface is not at or under any of agy's \
                               trustedWorkspaces and allowNonWorkspaceAccess is false. W1 P3 \
                               measured a write_to_file call under exactly these conditions land \
                               in the CLI's own scratch directory instead of the Work's cwd, with \
                               a SUCCESS terminal, an empty cwd and NOTHING on stderr or in the \
                               NDJSON saying so — the only trace was the absolute path inside \
                               tool_info.parameters. Whether the relocation is the CLI rewriting \
                               the path or the model being told the scratch dir is its writable \
                               surface is unmeasured; the outcome for sergeant is the same. Add \
                               this surface to trustedWorkspaces in the settings home this run \
                               reads",
                }),
            );
        }
    }
}

// ------------------------------------------------------------ turn reader

/// Send `SIGKILL` to a whole process group.
///
/// Nothing gates this on the leader being alive, and that is the point: the
/// group can outlive its leader (opencode probe 11), so the group id is
/// signalled unconditionally and `ESRCH` (an already-empty group) is success,
/// not an error to report.
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
            "no process-group signal mechanism on this platform; killing only the direct child \
             — any commands it spawned may still be running"
        );
    }
}

/// The group kill above plus `Child::kill()` on the direct child as a belt, for
/// the callers that still hold a live child handle. The group goes **first**:
/// the child's own death must never be what decides whether the group is
/// signalled.
fn kill_turn(pgid: Option<u32>, child: Option<&Arc<Mutex<Child>>>) {
    kill_process_group(pgid);
    if let Some(child) = child {
        let _ = child.lock().expect("agy turn child lock").kill();
    }
}

/// Read `reader` to EOF exactly as `Read::read_to_string` does, except the
/// returned `String` never grows past `cap` bytes. Every byte is still read
/// (never left sitting in the pipe — that would stall whatever is writing to
/// the other end); bytes past `cap` are simply not appended. Non-UTF-8 bytes
/// are replaced rather than failing the whole read. A trailing marker records
/// the loss when the cap was actually hit, so a capped capture reads as
/// "capped, N bytes missing" rather than silently looking complete.
fn read_bounded(reader: &mut impl std::io::Read, cap: usize) -> String {
    let mut buf: Vec<u8> = Vec::new();
    let mut total: usize = 0;
    let mut chunk = [0u8; 8192];
    loop {
        match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                total += n;
                if buf.len() < cap {
                    let take = (cap - buf.len()).min(n);
                    buf.extend_from_slice(&chunk[..take]);
                }
            }
            Err(_) => break,
        }
    }
    let mut text = String::from_utf8_lossy(&buf).into_owned();
    if total > buf.len() {
        text.push_str(&format!(
            "\n...<{}-byte in-memory cap hit; {} further bytes were read and discarded, not \
             archived>",
            cap,
            total - buf.len()
        ));
    }
    text
}

/// Everything the per-turn stdout reader thread needs. Owns ingestion end to
/// end: raw archive, normalization, the launch-time pin and resume-identity
/// checks, outcome recording — the sibling adapters' `TurnReader`, agy-shaped.
struct TurnReader {
    backend_state: Arc<Mutex<AdapterState>>,
    sink: Option<EventSink>,
    data_dir: PathBuf,
    execution_id: String,
    work_id: String,
    model: Option<String>,
    /// The conversation id this turn composed `--conversation` with, when it is
    /// a resume. `None` on turn 1. This is what the `init` line must echo back
    /// for the turn to be a resume at all (W1 P0.6's silent-fork hazard).
    expected_conversation: Option<String>,
    bindings_outside_cwd: Vec<PathBuf>,
    child: Arc<Mutex<Child>>,
    stderr_rx: Option<std::sync::mpsc::Receiver<String>>,
    /// Present only for turn 1: how this reader tells LAUNCH the identity (or
    /// that the harness refused before minting one).
    first_turn_signal: Option<SyncSender<FirstTurnSignal>>,
    /// [`AgyConfig::settings_home`], so this reader can compute the
    /// [`PermissionPosture`] itself at init-parse time. The reader is also the
    /// thread that composes `conversation.turn.ended`, so storing the posture
    /// here — not only from LAUNCH after the [`FirstTurnSignal`] round-trip —
    /// makes the turn-end read race-free by construction: a fast child cannot
    /// reach turn-end before the same thread has stored the posture.
    settings_home: Option<PathBuf>,
}

impl TurnReader {
    fn run(self, stdout: std::process::ChildStdout) {
        let mut raw = String::new();
        let mut raw_truncated = false;
        let mut acc = TurnAccumulator::new();
        let mut announced_init = false;

        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
            // Byte-exact, never normalize_pty'd: §20 fidelity means the blob is
            // what the harness wrote, and a normalized archive is an archive
            // that has already been interpreted. Every line is still parsed and
            // forwarded regardless of the cap — only the archive is bounded.
            if raw.len() < STREAM_MEMORY_CAP {
                raw.push_str(&line);
                raw.push('\n');
            } else {
                raw_truncated = true;
            }
            match serde_json::from_str::<Value>(&line) {
                Ok(value) => {
                    for event in acc.ingest_line(&value) {
                        self.emit(&event.kind, event.payload);
                    }
                    if !announced_init && let Some(id) = acc.conversation_id.clone() {
                        announced_init = true;
                        if let Some(execution) = self
                            .backend_state
                            .lock()
                            .expect("agy adapter state lock")
                            .executions
                            .get_mut(&self.execution_id)
                        {
                            // Only turn 1 mints identity. A later turn's `init`
                            // line is CHECKED against the id we asked for
                            // (`resume_mismatch`), never allowed to replace it:
                            // agy warns-and-continues on an unknown
                            // `--conversation` and starts a fresh conversation
                            // (W1 P0.6), so adopting whatever came back would be
                            // the adapter silently following the fork instead of
                            // reporting it — and would make every later handle
                            // an `UnknownExecution` besides.
                            if execution.conversation_id.is_none() {
                                execution.conversation_id = Some(id.clone());
                            }
                            // Same lock, same thread that later composes
                            // `conversation.turn.ended`: turn-end can never
                            // observe `posture: None` after an init line
                            // landed. LAUNCH stores it again after the signal
                            // round-trip — same value, belt not braces.
                            execution.posture = Some(PermissionPosture::from_init(
                                acc.init_permission_mode.as_deref(),
                                self.settings_home.as_deref(),
                            ));
                        }
                        if let Some(tx) = &self.first_turn_signal {
                            let _ = tx.send(FirstTurnSignal::Initialized {
                                conversation_id: id,
                                model: acc.init_model.clone(),
                                permission_mode: acc.init_permission_mode.clone(),
                            });
                        }
                    }
                }
                Err(_) => acc.unparsed_lines += 1,
            }
        }
        if raw_truncated {
            raw.push_str(&format!(
                "\n...<{STREAM_MEMORY_CAP}-byte in-memory cap hit; further stdout lines were still \
                 parsed and emitted above but were not archived here>\n"
            ));
        }

        // Stdout is closed; reap. The child lock is only taken after EOF so
        // INTERRUPT can always kill.
        let exit_code = self
            .child
            .lock()
            .expect("agy turn child lock")
            .wait()
            .ok()
            .and_then(|status| status.code());

        // §20: archived **before any conclusion is drawn from it**, and an
        // archive failure is reported rather than swallowed — the alternative
        // is a turn whose raw capture silently does not exist. This is the
        // module's single `BlobStore::put` call site (A4's ledger row pins it
        // to `impl TurnReader`).
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

        // The harness refused before minting identity, or died saying nothing.
        // Both are LAUNCH refusals; only turn 1 has anyone listening.
        if !announced_init && let Some(tx) = &self.first_turn_signal {
            let signal = match &acc.terminal {
                Terminal::Status { status, error, .. } => FirstTurnSignal::RefusedBeforeIdentity {
                    status: status.clone(),
                    error: error.clone(),
                    exit_code,
                },
                Terminal::None => FirstTurnSignal::ExitedWithoutInit {
                    exit_code,
                    stderr: stderr.clone(),
                    raw_blob: raw_blob.clone(),
                },
            };
            let _ = tx.send(signal);
        }

        let verdict = verify_pin_from_init(self.model.as_deref(), acc.init_model.as_deref());
        let pin_mismatch = verdict.mismatch(self.model.as_deref());
        let pin = verdict.as_json(self.model.as_deref());
        let resume_mismatch = self.resume_mismatch(&acc, &stderr);
        if let Some(mismatch) = &resume_mismatch {
            self.emit(
                KIND_TURN_HARNESS_ERROR,
                json!({
                    "phase": "resume_identity_mismatch",
                    "requested": self.expected_conversation,
                    "echoed": acc.conversation_id,
                    "stderr_warning": resume_fork_warning_in_stderr(&stderr),
                    "detail": mismatch,
                }),
            );
        }

        let mut state = self.backend_state.lock().expect("agy adapter state lock");
        let Some(execution) = state.executions.get_mut(&self.execution_id) else {
            return;
        };
        let interrupted = execution.interrupt_requested;
        let terminal = classify_terminal(&acc, exit_code, interrupted, &stderr);
        let conversation_for_event = execution.conversation_id.clone();
        let posture = execution.posture.clone();
        execution.turn = TurnState::Finished(Box::new(TurnOutcome {
            terminal: terminal.clone(),
            pin_mismatch,
            pin: pin.clone(),
            resume_mismatch,
            steps: acc.steps,
            agent_response_steps: acc.agent_response_steps,
            text_deltas: acc.text_deltas,
            tool_steps: acc.tool_steps,
            denied_tools: acc.denied_tools.clone(),
            unknown_events: acc.unknown_events.clone(),
            unparsed_lines: acc.unparsed_lines,
            saw_command_result: acc.saw_command_result,
            summary: acc.last_response.clone(),
            status: acc.status().map(str::to_string),
            last_error: acc.last_error.clone(),
            exit_code,
            raw_blob: raw_blob.clone(),
            raw_error: raw_error.clone(),
            stderr: stderr.clone(),
        }));
        drop(state);

        // Every turn ends with this event, however it ended — the only place
        // the §20 blob ref reaches the journal for a turn with no terminal.
        self.emit(
            KIND_CONVERSATION_TURN_ENDED,
            json!({
                "conversation_id": conversation_for_event,
                "interrupted": interrupted,
                "outcome": terminal_outcome_label(&terminal),
                "status": acc.status(),
                "init": {
                    "model": acc.init_model,
                    "permission_mode": acc.init_permission_mode,
                    "cwd": acc.init_cwd,
                    "tool_count": acc.init_tool_count,
                },
                "permission_posture": posture.map(|p| p.as_json()),
                "steps": acc.steps,
                "agent_response_steps": acc.agent_response_steps,
                "text_deltas": acc.text_deltas,
                "tool_steps": acc.tool_steps,
                "denied_tools": acc.denied_tools,
                "stderr_denial_notice": denial_evidence_in_stderr(&stderr),
                "unknown_events": acc.unknown_events,
                "unparsed_lines": acc.unparsed_lines,
                "saw_command_result": acc.saw_command_result,
                "bindings_outside_cwd": self.bindings_outside_cwd,
                // Both objects verbatim, never a synthetic sum: a reader can
                // see which is the final step's and which is the whole turn's.
                "usage_final_step": acc.last_step_usage,
                "usage_turn": acc.terminal_usage,
                "structured_output": acc.structured_output,
                "model_pin": pin,
                "exit_code": exit_code,
                "raw": raw_blob,
                "raw_error": raw_error,
                "stderr": truncate(normalize_pty(stderr.trim()).as_str(), 400).to_string(),
            }),
        );
    }

    /// The silent-resume-fork guard. A turn that composed `--conversation <id>`
    /// is only a resume if the `init` line echoes that exact id back: an
    /// unknown id does not refuse, it warns on stderr and starts a **fresh**
    /// conversation [W1 P0.6].
    fn resume_mismatch(&self, acc: &TurnAccumulator, stderr: &str) -> Option<String> {
        let requested = self.expected_conversation.as_deref()?;
        let echoed = acc.conversation_id.as_deref();
        if echoed == Some(requested) {
            return None;
        }
        Some(format!(
            "resume identity mismatch: this turn asked for conversation {requested} and agy's init \
             line echoed {}. agy warns-and-continues on an unknown conversation rather than \
             refusing (measured, W1 P0.6: a plain-text stderr `warning: conversation \"…\" not \
             found` and a fresh conversation), so a fresh conversation would otherwise have been \
             silently mistaken for a resume. stderr carried that warning: {}",
            echoed.unwrap_or("<no init line at all>"),
            resume_fork_warning_in_stderr(stderr),
        ))
    }

    fn emit(&self, kind: &str, payload: Value) {
        if let Some(sink) = &self.sink {
            sink(EventDraft {
                source: EventSource::new("backend", AGY_BACKEND_NAME),
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

// ------------------------------------------------------------- observation

impl Backend for AgyBackend {
    fn name(&self) -> &str {
        AGY_BACKEND_NAME
    }

    /// Capabilities as measured at 1.1.19. Every `true` names a contract test
    /// in `ADMISSION_ROWS` (L8, made structural by the module's own
    /// `tests::admission_rows_agree_with_capabilities`); every `false` names
    /// its reason in the same row.
    ///
    /// `history: false` is the row that differs from opencode's: agy has **no
    /// export verb**, and §15's rule is that unsupported means unsupported, not
    /// emulation — so [`Backend::history`] refuses rather than returning what
    /// this process happened to see.
    fn capabilities(&self) -> Capabilities {
        capabilities_for(Transport::Print)
    }

    /// §17: each turn is its own short-lived process; there is no backend-level
    /// service to start or attach to.
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
        ProbeReport {
            available: true,
            detail: Some(format!(
                "{}\nadmission rows:\n{}",
                outcome.detail,
                render_admission_rows()
            )),
        }
    }

    /// PREPARE: refuse an unavailable probe, an impossible pin, an impossible
    /// profile, or a prompt that cannot fit on argv; reserve **no** native id.
    ///
    /// `native_id: None` is exactly the honest answer
    /// [`PreparedExecution::native_id`]'s own contract blesses: the
    /// `conversation_id` is harness-minted and first appears on the `init`
    /// line, so there is nothing to reserve. The module doc names the resulting
    /// crash window rather than papering over it.
    ///
    /// No external effect (§22.6 forbids I/O under the core lock): no process,
    /// no network, no blocking wait. The probe is memoized and may run here on
    /// the very first call, exactly as the sibling adapters' do.
    fn prepare(&self, request: &StartRequest) -> Result<PreparedExecution, BackendError> {
        let probe = self.probe_outcome();
        if !probe.available {
            return Err(BackendError::Unavailable {
                backend: AGY_BACKEND_NAME.to_string(),
                detail: probe.detail.clone(),
            });
        }
        if let Some(model) = &request.model {
            preflight_model_pin(model).map_err(|reason| self.err_failed(reason))?;
        }
        // Validated without keeping the result: LAUNCH re-resolves it, so the
        // two phases can never disagree about it.
        self.launch_config(request.profile.as_ref())?;
        // Refused here rather than at LAUNCH so it costs nothing and is
        // journaled before any process exists.
        check_argv_prompt_budget(&compose_launch_prompt(request), request)
            .map_err(|reason| self.err_failed(reason))?;
        Ok(PreparedExecution {
            execution_id: request.execution_id.clone(),
            native_id: None,
            request: request.clone(),
        })
    }

    /// LAUNCH: register the execution, spawn turn 1, and wait bounded for the
    /// `init` line before returning a handle at all. A failed launch leaves no
    /// phantom: adapter state is removed on every error path, so a later
    /// OBSERVE of the reserved id is an honest `UnknownExecution` rather than a
    /// context nothing created.
    fn launch(&self, prepared: &PreparedExecution) -> Result<ExecutionHandle, BackendError> {
        let request = &prepared.request;
        let LaunchConfig { executable, env } = self.launch_config(request.profile.as_ref())?;
        {
            let mut state = self.lock();
            state.executions.insert(
                request.execution_id.clone(),
                AgyExecution {
                    conversation_id: None,
                    work_id: request.work_id.clone(),
                    cwd: request.cwd.clone(),
                    model: request.model.clone(),
                    executable,
                    env,
                    settings_home: self.config.settings_home.clone(),
                    json_schema: self.config.json_schema.clone(),
                    bindings_outside_cwd: bindings_outside_cwd(&request.cwd, &request.bindings),
                    posture: None,
                    turns: 0,
                    turn: TurnState::Unlaunched,
                    turn_pgid: None,
                    stopped: false,
                    interrupt_requested: false,
                    reader: None,
                },
            );
        }
        let policy = format!("{:?}", request.instruction_policy);
        let signal = match self.spawn_first_turn(
            &request.execution_id,
            compose_launch_prompt(request),
            Some(policy),
        ) {
            Ok(signal) => signal,
            Err(e) => {
                self.lock().executions.remove(&request.execution_id);
                return Err(e);
            }
        };
        match signal {
            FirstTurnSignal::Initialized {
                conversation_id,
                model,
                permission_mode,
            } => {
                // The R4 delta cashed in: `init` precedes any model output, so
                // this is the earliest possible moment and the fewest possible
                // tokens. A substituted model refuses the LAUNCH — the turn
                // that would have succeeded is not the turn the human asked
                // for.
                let verdict = verify_pin_from_init(request.model.as_deref(), model.as_deref());
                if let PinVerdict::Substituted(served) = &verdict {
                    self.kill_inflight_turn(&request.execution_id);
                    self.lock().executions.remove(&request.execution_id);
                    return Err(self.err_failed(format!(
                        "agy's init line names {served} as the model serving this conversation, \
                         but this execution requested {}. The init line precedes any model \
                         output, so the launch is refused here rather than reported after a turn \
                         the human did not ask for has already run.",
                        request.model.as_deref().unwrap_or("<none>")
                    )));
                }
                let posture = PermissionPosture::from_init(
                    permission_mode.as_deref(),
                    self.config.settings_home.as_deref(),
                );
                if let Some(execution) = self.lock().executions.get_mut(&request.execution_id) {
                    execution.posture = Some(posture.clone());
                }
                self.announce_launch_posture(
                    &request.execution_id,
                    &request.work_id,
                    &request.cwd,
                    &posture,
                );
                Ok(ExecutionHandle {
                    execution_id: request.execution_id.clone(),
                    native_id: Some(conversation_id),
                })
            }
            FirstTurnSignal::RefusedBeforeIdentity {
                status,
                error,
                exit_code,
            } => {
                self.lock().executions.remove(&request.execution_id);
                Err(self.err_failed(format!(
                    "agy refused this turn before minting a conversation (status={status}, \
                     exit_code={exit_code:?}), so no identity exists and nothing is resumable. \
                     agy's own error, verbatim: {error}"
                )))
            }
            FirstTurnSignal::ExitedWithoutInit {
                exit_code,
                stderr,
                raw_blob,
            } => {
                self.lock().executions.remove(&request.execution_id);
                Err(self.err_failed(format!(
                    "agy exited before any `init` line arrived and emitted no terminal either \
                     (exit_code={exit_code:?}), so no conversation was ever minted; stderr: {}; \
                     raw={}",
                    truncate(stderr.trim(), 400),
                    raw_blob
                        .unwrap_or_else(|| "unarchived (the turn streamed nothing)".to_string())
                )))
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
                    "execution {} already has a turn in flight; an agy conversation runs one turn \
                     at a time",
                    handle.execution_id
                )));
            }
            if execution.conversation_id.is_none() {
                return Err(self.err_failed(format!(
                    "execution {} has no conversation id, so there is nothing to compose \
                     --conversation with",
                    handle.execution_id
                )));
            }
        }
        self.spawn_turn(&handle.execution_id, input.to_string(), None, None)
    }

    fn observe(&self, handle: &ExecutionHandle) -> Result<Observation, BackendError> {
        let state = self.lock();
        self.check_identity(&state, handle)?;
        Ok(observe_in_memory(&state.executions[&handle.execution_id]))
    }

    /// INTERRUPT: stop the current turn without retiring the execution.
    /// Interrupting an execution with no turn in flight is a **no-op, not an
    /// error** — the goal state already holds.
    ///
    /// The `interrupt_requested` bit is set **before** the kill, so the reader
    /// thread's `classify_terminal` sees it: a race here is the difference
    /// between `InterruptedRunning` and `AmbiguousUnknown`. The completion is
    /// deferred on the reader's join so the killed turn's raw archive is
    /// durable before the interrupt's promise is true; the engine waits outside
    /// the core lock.
    fn interrupt(&self, handle: &ExecutionHandle) -> Result<Completion, BackendError> {
        let (pgid, child, reader) = {
            let mut state = self.lock();
            self.check_identity(&state, handle)?;
            let execution = state
                .executions
                .get_mut(&handle.execution_id)
                .expect("presence checked above");
            // The group id is taken whatever the turn state says: a turn that
            // has already ended can still have left a background command
            // running in its group. Only the `interrupt_requested` bit, a claim
            // about a *running* turn's outcome, is the in-flight turn's alone.
            let child = match &execution.turn {
                TurnState::InFlight(child) => {
                    execution.interrupt_requested = true;
                    Some(Arc::clone(child))
                }
                TurnState::Finished(_) | TurnState::Unlaunched | TurnState::Adopted => None,
            };
            let reader = child.is_some().then(|| execution.reader.take()).flatten();
            (execution.turn_pgid, child, reader)
        };
        kill_turn(pgid, child.as_ref());
        match reader {
            None => Ok(Completion::immediate()),
            Some(reader) => Ok(Completion::deferred(move || {
                let _ = reader.join();
            })),
        }
    }

    /// RESUME: re-adopt a durable conversation after a daemon restart.
    ///
    /// **No token-free re-adoption check was measured.** agy has no
    /// `export`-equivalent whose exit status opencode could lean on, and every
    /// read-only slash command answers about the *installation*, not about one
    /// conversation. So `Ok` from RESUME is a weaker claim here than there: it
    /// says this adapter now owns the id and will compose `--conversation` with
    /// it, and the durable-context check is deferred to the first subsequent
    /// SEND — where `TurnReader::resume_mismatch` fails the turn closed if
    /// the `init` line echoes a different id (W1 P0.6's silent fork). Said
    /// plainly rather than papered over.
    ///
    /// RESUME never starts a turn (§15: re-adoption costs no tokens and creates
    /// no second execution), and it invents nothing: model, profile,
    /// instruction policy and bindings all come from the request, and a pin not
    /// re-supplied is *not enforced* on later turns.
    fn resume(
        &self,
        handle: &ExecutionHandle,
        request: &ResumeRequest,
    ) -> Result<(), BackendError> {
        let conversation_id = handle
            .native_id
            .clone()
            .ok_or_else(|| self.err_unknown(&handle.execution_id))?;
        if let Some(model) = &request.model {
            preflight_model_pin(model).map_err(|reason| self.err_failed(reason))?;
        }
        let LaunchConfig { executable, env } = self.launch_config(request.profile.as_ref())?;
        let mut state = self.lock();
        if let Some(existing) = state.executions.get(&handle.execution_id) {
            if existing.conversation_id.as_deref() != Some(conversation_id.as_str()) {
                return Err(self.err_unknown(&handle.execution_id));
            }
            // Fail closed if a turn of this daemon's is already in flight: a
            // running turn is not a context to re-adopt underneath itself.
            if let TurnState::InFlight(_) = existing.turn {
                return Err(self.err_failed(format!(
                    "cannot re-adopt conversation {conversation_id}: a turn of it is already in \
                     flight under this daemon"
                )));
            }
            return Ok(());
        }
        state.executions.insert(
            handle.execution_id.clone(),
            AgyExecution {
                conversation_id: Some(conversation_id),
                work_id: request.work_id.clone(),
                cwd: request.cwd.clone(),
                model: request.model.clone(),
                executable,
                env,
                settings_home: self.config.settings_home.clone(),
                json_schema: self.config.json_schema.clone(),
                bindings_outside_cwd: bindings_outside_cwd(&request.cwd, &request.bindings),
                posture: None,
                turns: 1,
                turn: TurnState::Adopted,
                turn_pgid: None,
                stopped: false,
                interrupt_requested: false,
                reader: None,
            },
        );
        Ok(())
    }

    /// HISTORY: refused, and the refusal is the honest answer.
    ///
    /// agy has **no export verb**, so this adapter's only record of a
    /// conversation is what its own process happened to see — which after a
    /// restart is nothing. `Ok(vec![])` from a backend that simply cannot look
    /// is indistinguishable from "this conversation said nothing", which is
    /// exactly the confusion [`Capabilities::history`] exists to prevent, and
    /// `mod.rs`'s HISTORY contract pairs the refusal with the `false` flag.
    fn history(&self, handle: &ExecutionHandle) -> Result<Vec<NativeEvent>, BackendError> {
        {
            let state = self.lock();
            self.check_identity(&state, handle)?;
        }
        Err(BackendError::Unsupported {
            backend: AGY_BACKEND_NAME.to_string(),
            verb: "history".to_string(),
            detail: "agy exposes no export verb, so this adapter has no durable native history to \
                     retrieve: everything it knows is what its own turn readers observed, which a \
                     restart loses. Two leads exist and neither is promoted — \
                     ~/.gemini/antigravity-cli/conversations/<id>.db (one SQLite file per \
                     conversation, schema unmeasured, a private path of another product) and \
                     cache/last_conversations.json (cwd->id). Sergeant's own durable record of \
                     these events is the journal, fed by the event sink."
                .to_string(),
        })
    }

    /// STOP: kill any in-flight turn, refuse further input, hand back the
    /// reader's join as the completion's tail (issue #14/B3's rule — the engine
    /// never waits on it under the core lock), so the turn's archive is durable
    /// before STOP's promise is true.
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

/// Map an in-memory execution's turn state to an Observation.
fn observe_in_memory(execution: &AgyExecution) -> Observation {
    let conversation = execution.conversation_id.as_deref().unwrap_or("<unminted>");
    match &execution.turn {
        TurnState::Unlaunched => Observation {
            native: NativeState::Unknown,
            signal: BackendSignal::Running,
            evidence: Some(format!(
                "execution registered for conversation {conversation} but no turn was ever \
                 launched"
            )),
        },
        TurnState::Adopted => Observation {
            native: NativeState::Unknown,
            signal: BackendSignal::Running,
            evidence: Some(format!(
                "conversation {conversation} was re-adopted after a restart and no turn of this \
                 daemon's has run on it. agy exposes no token-free re-adoption check (no export \
                 verb), so this Ok is a weaker claim than opencode's: the durable-context check \
                 happens on the first subsequent SEND, whose init line must echo this id back"
            )),
        },
        TurnState::InFlight(_) => Observation {
            native: NativeState::Running,
            signal: BackendSignal::Running,
            evidence: Some(format!(
                "turn {} in flight on conversation {conversation}",
                execution.turns
            )),
        },
        TurnState::Finished(outcome) => {
            // Checked before the completion branch, whatever the turn otherwise
            // produced: a substituted model, or a conversation that forked
            // silently, outranks a successful turn — because the turn that
            // succeeded is not the turn the human asked for.
            if let Some(mismatch) = &outcome.resume_mismatch {
                return Observation {
                    native: NativeState::Exited,
                    signal: BackendSignal::Failed {
                        reason: mismatch.clone(),
                    },
                    evidence: Some(format!(
                        "conversation_id={conversation}; raw={}; stderr: {}",
                        outcome.raw_evidence(),
                        truncate(outcome.stderr.trim(), 400)
                    )),
                };
            }
            if let Some(mismatch) = &outcome.pin_mismatch {
                return Observation {
                    native: NativeState::Exited,
                    signal: BackendSignal::Failed {
                        reason: mismatch.clone(),
                    },
                    evidence: Some(format!(
                        "conversation_id={conversation}; model_pin={}; raw={}",
                        outcome.pin,
                        outcome.raw_evidence()
                    )),
                };
            }
            match &outcome.terminal {
                TerminalOutcome::Completed => Observation {
                    native: NativeState::Exited,
                    signal: BackendSignal::StageCompleted {
                        summary: outcome.summary.clone(),
                    },
                    evidence: Some(format!(
                        "conversation_id={conversation}; status={:?}; model_pin={}; raw={}; \
                         steps={}, agent_response_steps={}, text_deltas={}, tool_steps={}, \
                         unknown_events={:?}, unparsed_lines={}",
                        outcome.status,
                        outcome.pin,
                        outcome.raw_evidence(),
                        outcome.steps,
                        outcome.agent_response_steps,
                        outcome.text_deltas,
                        outcome.tool_steps,
                        outcome.unknown_events,
                        outcome.unparsed_lines,
                    )),
                },
                TerminalOutcome::Failed { reason } => Observation {
                    native: NativeState::Exited,
                    signal: BackendSignal::Failed {
                        reason: format!("turn failed: {}", truncate(reason, 400)),
                    },
                    evidence: Some(format!(
                        "conversation_id={conversation}; status={:?}; exit_code={:?}; raw={}; \
                         stderr: {}",
                        outcome.status,
                        outcome.exit_code,
                        outcome.raw_evidence(),
                        truncate(outcome.stderr.trim(), 400)
                    )),
                },
                TerminalOutcome::InterruptedRunning => Observation {
                    native: NativeState::Exited,
                    signal: BackendSignal::Running,
                    evidence: Some(format!(
                        "turn interrupted by request; conversation {conversation} remains \
                         resumable — W1 P4 measured a SIGKILLed turn leaving the conversation \
                         fully resumable, with the pre-kill content recalled on the next turn; \
                         raw={}",
                        outcome.raw_evidence()
                    )),
                },
                // §25's ambiguity, failing closed: `native: Unknown` blocks the
                // Work rather than letting a stage be completed or failed by a
                // process that merely stopped talking — or by a `SUCCESS` the
                // honesty rules say we may not believe.
                TerminalOutcome::AmbiguousUnknown => Observation {
                    native: NativeState::Unknown,
                    signal: BackendSignal::Running,
                    evidence: Some(format!(
                        "ambiguous turn on conversation {conversation}: status={:?}, \
                         agent_response_steps={}, denied_tools={:?}, saw_command_result={}, \
                         exit_code={:?}, last_error={:?}. {}This adapter never reads a nonzero \
                         exit as a failure, nor a SUCCESS as a completion: a SUCCESS with no \
                         response and no text-producing agent_response step is the \
                         dropped-stream class agy 1.1.18 fixed and this rule still guards; a \
                         SUCCESS or an unrequested CANCELED alongside denial evidence is a turn \
                         whose work did not happen. raw={}; stderr: {}",
                        outcome.status,
                        outcome.agent_response_steps,
                        outcome.denied_tools,
                        outcome.saw_command_result,
                        outcome.exit_code,
                        outcome.last_error,
                        outcome
                            .denial_note()
                            .map(|note| format!("{note}. "))
                            .unwrap_or_default(),
                        outcome.raw_evidence(),
                        truncate(normalize_pty(outcome.stderr.trim()).as_str(), 400),
                    )),
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Fixtures. Every file named `agy-1.1.19-*` is a REAL capture at agy
    // 1.1.19 except the three whose own first line labels them synthesized —
    // `dropped-stream-empty-success`, `soft-deny-success` and
    // `permission-denied-error-terminal`. That label line is itself an unknown
    // event kind, counted and never interpreted, which is why it can sit in an
    // NDJSON fixture without changing what the decoder does.
    const MINIMAL_TURN: &str = include_str!("../../tests/fixtures/agy-1.1.19-minimal-turn.jsonl");
    const TOOL_USE: &str = include_str!("../../tests/fixtures/agy-1.1.19-tool-use.jsonl");
    const DENIED_CANCELED: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-permission-denied-canceled.jsonl");
    const DENIED_ERROR_TERMINAL: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-permission-denied-error-terminal.jsonl");
    const SOFT_DENY_SUCCESS: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-soft-deny-success.jsonl");
    const EMPTY_SUCCESS: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-dropped-stream-empty-success.jsonl");
    const INVALID_MODEL: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-invalid-model-refusal.jsonl");
    const SLASH_COMMAND: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-slash-command-result.jsonl");
    const SIGKILL_TRUNCATED: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-sigkill-truncated.jsonl");
    const PRINT_TIMEOUT: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-print-timeout-error.jsonl");
    const JSON_SCHEMA: &str = include_str!("../../tests/fixtures/agy-1.1.19-json-schema.jsonl");
    const RESUME_TURN: &str = include_str!("../../tests/fixtures/agy-1.1.19-resume-turn.jsonl");

    /// The measured auto-denial notice — the captured bytes of W1 P2's control
    /// turn, byte-identical to P3 turn 1's. Shared with `tests/agy_backend.rs`
    /// through the file so the two copies cannot drift apart again (they had:
    /// the integration copy carried a hyphen for the em-dash and escaped quotes
    /// the capture never contained).
    const DENIAL_NOTICE: &str = include_str!("../../tests/fixtures/agy-1.1.19-denial-notice.txt");

    /// The measured resume-fork warning, verbatim from W1 P0.6.
    const RESUME_WARNING: &str =
        "warning: conversation \"00000000-0000-0000-0000-000000000000\" not found";

    fn replay(fixture: &str) -> (TurnAccumulator, Vec<NativeEvent>) {
        let mut acc = TurnAccumulator::new();
        let mut events = Vec::new();
        for line in fixture.lines() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<Value>(line) {
                Ok(value) => events.extend(acc.ingest_line(&value)),
                Err(_) => acc.unparsed_lines += 1,
            }
        }
        (acc, events)
    }

    fn kinds(events: &[NativeEvent]) -> Vec<&str> {
        events.iter().map(|event| event.kind.as_str()).collect()
    }

    // ------------------------------------------------------------- version

    #[test]
    fn parse_agy_version_pins_the_measured_shape() {
        // The bare triple `agy --version` actually printed (W1 P0).
        assert_eq!(parse_agy_version("1.1.19\n"), Some((1, 1, 19)));
        // A suffixed build still parses; the full string travels in `detail`.
        assert_eq!(parse_agy_version("1.1.19-rc.1"), Some((1, 1, 19)));
        assert_eq!(parse_agy_version("1.1.19 abc1234"), Some((1, 1, 19)));
        // A vendor prefix is a grammar change worth noticing, not a silent
        // parse of the second token (this is where agy differs from codex).
        assert_eq!(parse_agy_version("agy 1.1.19"), None);
        assert_eq!(parse_agy_version("nightly"), None);
        assert_eq!(parse_agy_version(""), None);
    }

    #[test]
    fn the_measured_floor_is_the_packets_version_not_the_installed_one() {
        // R1 and §1.1 in one assertion: the floor is the packet's 1.1.17
        // (provenance), while every fixture beside this module is a 1.1.19
        // capture. The two numbers differ on purpose.
        assert_eq!(MEASURED_FLOOR, (1, 1, 17));
        assert!(parse_agy_version("1.1.19").expect("parses") >= MEASURED_FLOOR);
        assert!(parse_agy_version("1.1.16").expect("parses") < MEASURED_FLOOR);
    }

    #[test]
    fn missing_entries_names_exactly_the_absent_ones() {
        let help = "--print --output-format --model";
        assert_eq!(
            missing_entries(help, REQUIRED_FLAGS),
            vec![
                "--conversation",
                "--disable-slash-commands",
                "--json-schema"
            ]
        );
        assert!(
            missing_entries(
                "--print --output-format --model --conversation \
                                 --disable-slash-commands --json-schema",
                REQUIRED_FLAGS
            )
            .is_empty()
        );
    }

    #[test]
    fn required_flags_are_exactly_what_the_launch_grammar_composes() {
        // `opencode.rs`'s rule about `--auto`, applied here: the probe must not
        // gate on a flag this adapter never composes, and must gate on every
        // one it does.
        let mut argv = resume_turn_argv("prompt", Some("m"), Some("{}"), "c");
        argv.extend(first_turn_argv("prompt", Some("m"), Some("{}")));
        for flag in REQUIRED_FLAGS {
            assert!(
                argv.iter().any(|arg| arg == flag) || *flag == "--print",
                "REQUIRED_FLAGS names {flag}, which no composed argv carries"
            );
        }
        // `-p` is the short form of `--print`; the help text renders both, and
        // the long form is what the probe scans for (stable against a
        // short-form change).
        assert!(argv.contains(&"-p".to_string()));
        assert!(
            !argv
                .iter()
                .any(|arg| arg == "--dangerously-skip-permissions")
        );
        assert!(!argv.iter().any(|arg| arg == "--print-timeout"));
        assert!(!argv.iter().any(|arg| arg == "--add-dir"));
        assert!(!argv.iter().any(|arg| arg == "--mode"));
        assert!(!argv.iter().any(|arg| arg == "--agent"));
        assert!(!argv.iter().any(|arg| arg == "--sandbox"));
        let _ = argv;
    }

    // ------------------------------------------------------ launch grammar

    #[test]
    fn first_turn_argv_carries_the_measured_shape() {
        let argv = first_turn_argv("say pong", Some("gemini-3.7-flash-low"), None);
        // The prompt is the VALUE of `-p`, immediately after it (W1 P0.3).
        assert_eq!(argv[0], "-p");
        assert_eq!(argv[1], "say pong");
        assert_eq!(argv[2], "--output-format");
        assert_eq!(argv[3], "stream-json");
        assert!(argv.contains(&"--disable-slash-commands".to_string()));
        let model_at = argv.iter().position(|a| a == "--model").expect("--model");
        assert_eq!(argv[model_at + 1], "gemini-3.7-flash-low");
        assert!(!argv.iter().any(|a| a == "--json-schema"));
        assert!(!argv.iter().any(|a| a == "--conversation"));
    }

    #[test]
    fn an_unpinned_turn_composes_no_model_flag() {
        let argv = first_turn_argv("say pong", None, None);
        assert!(!argv.iter().any(|a| a == "--model"));
    }

    #[test]
    fn resume_turn_argv_adds_the_conversation_and_keeps_the_pin() {
        let argv = resume_turn_argv("again", Some("gemini-3.7-flash-low"), Some("{}"), "conv-1");
        let at = argv
            .iter()
            .position(|a| a == "--conversation")
            .expect("--conversation");
        assert_eq!(argv[at + 1], "conv-1");
        // A pin the human asked for must not silently lapse after turn 1.
        let model_at = argv.iter().position(|a| a == "--model").expect("--model");
        assert_eq!(argv[model_at + 1], "gemini-3.7-flash-low");
        let schema_at = argv
            .iter()
            .position(|a| a == "--json-schema")
            .expect("--json-schema");
        assert_eq!(argv[schema_at + 1], "{}");
        assert!(argv.contains(&"--disable-slash-commands".to_string()));
    }

    #[test]
    fn the_environment_contract_matches_claudes_today() {
        // Copied, not imported, so a divergence is a decision rather than
        // drift nobody noticed.
        assert_eq!(
            ENVIRONMENT_CONTRACT,
            crate::backend::claude::ENVIRONMENT_CONTRACT
        );
    }

    #[test]
    fn the_execution_model_contract_states_agys_own_measured_model() {
        // The sentence that makes this constant agy's own rather than a copy:
        // a denied tool cancels the whole turn (W1 P2), which is not what
        // opencode's auto-reject does.
        assert!(EXECUTION_MODEL_CONTRACT.contains("agy --print"));
        assert!(EXECUTION_MODEL_CONTRACT.contains("cancelled"));
        assert_ne!(
            EXECUTION_MODEL_CONTRACT,
            crate::backend::opencode::EXECUTION_MODEL_CONTRACT
        );
    }

    fn prompt_request(bindings: Vec<BindingSummary>) -> StartRequest {
        StartRequest {
            work_id: "w".into(),
            execution_id: "e".into(),
            stage_id: "s".into(),
            attempt: 1,
            cwd: PathBuf::from("/tmp/surface"),
            intent: "THE-INTENT".into(),
            context: "THE-CONTEXT".into(),
            model: None,
            profile: None,
            execute: None,
            instruction_policy: Default::default(),
            bindings,
        }
    }

    #[test]
    fn compose_launch_prompt_orders_its_sections() {
        let request = prompt_request(vec![BindingSummary {
            repository: "repo".into(),
            worktree_path: PathBuf::from("/tmp/surface/repo"),
            work_branch: "work/x".into(),
            base_branch: Some("main".into()),
            base_sha: "abc123".into(),
        }]);
        let prompt = compose_launch_prompt(&request);
        let exec = prompt.find(EXECUTION_MODEL_CONTRACT).expect("exec model");
        let env = prompt.find(ENVIRONMENT_CONTRACT).expect("environment");
        let surface = prompt.find(MUTATION_SURFACE_HEADER).expect("surface");
        let intent = prompt.find("THE-INTENT").expect("intent");
        let context = prompt.find("THE-CONTEXT").expect("context");
        assert!(exec < env && env < surface && surface < intent && intent < context);
        assert!(prompt.contains("/tmp/surface/repo"));
        assert!(prompt.contains("cut from main at abc123"));
    }

    #[test]
    fn the_mutation_surface_section_is_omitted_when_there_are_no_bindings() {
        // "You may modify nothing" is a claim the section's silence does not
        // make, so the section is omitted entirely rather than emitted empty.
        let prompt = compose_launch_prompt(&prompt_request(Vec::new()));
        assert!(!prompt.contains(MUTATION_SURFACE_HEADER));
        assert!(prompt.contains("THE-INTENT") && prompt.contains("THE-CONTEXT"));
    }

    #[test]
    fn compose_launch_prompt_names_a_detached_admission() {
        let request = prompt_request(vec![BindingSummary {
            repository: "repo".into(),
            worktree_path: PathBuf::from("/tmp/surface/repo"),
            work_branch: "work/x".into(),
            base_branch: None,
            base_sha: "abc123".into(),
        }]);
        assert!(
            compose_launch_prompt(&request).contains("no named base branch (detached admission)")
        );
    }

    #[test]
    fn bindings_outside_cwd_reports_only_what_escapes_the_surface() {
        let inside = BindingSummary {
            repository: "in".into(),
            worktree_path: PathBuf::from("/tmp/surface/in"),
            work_branch: "b".into(),
            base_branch: None,
            base_sha: "s".into(),
        };
        let outside = BindingSummary {
            repository: "out".into(),
            worktree_path: PathBuf::from("/elsewhere/out"),
            work_branch: "b".into(),
            base_branch: None,
            base_sha: "s".into(),
        };
        assert_eq!(
            bindings_outside_cwd(Path::new("/tmp/surface"), &[inside, outside]),
            vec![PathBuf::from("/elsewhere/out")]
        );
    }

    #[test]
    fn preflight_refuses_only_an_empty_pin() {
        assert!(preflight_model_pin("").is_err());
        assert!(preflight_model_pin("   ").is_err());
        // An unrecognized model is NOT refused here: agy's own typed refusal
        // enumerates the whole catalog and is better evidence than a local
        // allowlist this adapter would have to maintain (R1).
        assert!(preflight_model_pin("not-a-real-model").is_ok());
        assert!(preflight_model_pin("gemini-3.7-flash-low").is_ok());
    }

    #[test]
    fn a_prompt_larger_than_the_argv_cap_is_refused_not_truncated() {
        let mut request = prompt_request(Vec::new());
        request.context = "x".repeat(ARGV_PROMPT_CAP + 1);
        let prompt = compose_launch_prompt(&request);
        let refusal = check_argv_prompt_budget(&prompt, &request).expect_err("must refuse");
        assert!(refusal.contains(&format!("{ARGV_PROMPT_CAP}-byte argv cap")));
        assert!(
            refusal.contains("131072"),
            "names the measured boundary: {refusal}"
        );
        assert!(
            refusal.contains("context (CONTEXT.md)"),
            "names the largest section"
        );
        assert!(
            refusal.contains("--input-format stream-json"),
            "names the measured channel"
        );
        assert!(refusal.contains("Nothing is truncated"));
        // And a prompt that fits is not refused.
        assert!(check_argv_prompt_budget("small", &prompt_request(Vec::new())).is_ok());
    }

    // ------------------------------------------------------ pin verification

    #[test]
    fn verify_pin_from_init_reads_the_resolved_model_off_line_one() {
        assert_eq!(verify_pin_from_init(None, Some("m")), PinVerdict::Unpinned);
        assert_eq!(
            verify_pin_from_init(Some("gemini-3.7-flash-low"), Some("gemini-3.7-flash-low")),
            PinVerdict::Honored("gemini-3.7-flash-low".into())
        );
        assert_eq!(
            verify_pin_from_init(Some("gemini-3.7-flash-low"), Some("gemini-3.1-pro-high")),
            PinVerdict::Substituted("gemini-3.1-pro-high".into())
        );
        // Exact string equality: agy's ids are flat, with no provider prefix,
        // so none of opencode's slash-splitting applies.
        assert_eq!(
            verify_pin_from_init(
                Some("gemini-3.7-flash-low"),
                Some("gemini/gemini-3.7-flash-low")
            ),
            PinVerdict::Substituted("gemini/gemini-3.7-flash-low".into())
        );
        assert!(matches!(
            verify_pin_from_init(Some("m"), None),
            PinVerdict::Attempted(_)
        ));
        assert!(matches!(
            verify_pin_from_init(Some("m"), Some("")),
            PinVerdict::Attempted(_)
        ));
    }

    #[test]
    fn only_a_substitution_is_a_stage_failure() {
        assert!(PinVerdict::Unpinned.mismatch(None).is_none());
        assert!(
            PinVerdict::Honored("m".into())
                .mismatch(Some("m"))
                .is_none()
        );
        // A pin that could not be checked is missing evidence, and failing a
        // stage on missing evidence would be this adapter deciding a Work's
        // fate on something it never saw.
        assert!(
            PinVerdict::Attempted("why".into())
                .mismatch(Some("m"))
                .is_none()
        );
        let mismatch = PinVerdict::Substituted("other".into())
            .mismatch(Some("m"))
            .expect("a substitution is a failure");
        assert!(mismatch.contains("requested m") && mismatch.contains("other"));
    }

    #[test]
    fn pin_verdicts_render_their_own_evidence() {
        assert_eq!(
            PinVerdict::Unpinned.as_json(None),
            json!({"verdict": "unpinned"})
        );
        assert_eq!(
            PinVerdict::Honored("m".into()).as_json(Some("m")),
            json!({"verdict": "honored", "requested": "m", "served": "m"})
        );
        assert_eq!(
            PinVerdict::Substituted("o".into()).as_json(Some("m")),
            json!({"verdict": "substituted", "requested": "m", "ran": "o"})
        );
        assert_eq!(
            PinVerdict::Attempted("why".into()).as_json(Some("m")),
            json!({"verdict": "attempted", "requested": "m", "detail": "why"})
        );
    }

    // --------------------------------------------------- permission posture

    #[test]
    fn an_unknown_permission_mode_is_treated_as_denying() {
        // Fail closed: the non-denying allowlist is empty because nothing has
        // been measured onto it, so every mode — including the measured
        // `request-review` and an entirely unknown string — denies.
        for mode in [
            Some("request-review"),
            Some("strict"),
            Some("what-is-this"),
            None,
        ] {
            let posture = PermissionPosture::from_init(mode, None);
            assert!(posture.denies_tools, "mode {mode:?} must fail closed");
        }
        assert!(NON_DENYING_MODES.is_empty());
    }

    #[test]
    fn the_posture_names_the_injection_channel_it_actually_used() {
        let none = PermissionPosture::from_init(Some("request-review"), None);
        assert!(none.injection.contains("operator config required"));
        let injected =
            PermissionPosture::from_init(Some("request-review"), Some(Path::new("/var/tmp/home")));
        assert!(injected.injection.contains("HOME=/var/tmp/home"));
        assert_eq!(
            injected.as_json()["effective_mode"],
            json!("request-review")
        );
        assert_eq!(injected.as_json()["denies_tools"], json!(true));
    }

    // -------------------------------------------------------------- decoding

    #[test]
    fn the_minimal_turn_fixture_decodes_to_text_and_usage() {
        let (acc, events) = replay(MINIMAL_TURN);
        assert_eq!(
            acc.conversation_id.as_deref(),
            Some("8bfcc611-f2b9-4eb1-b17d-22b4caec46df")
        );
        // The R4 delta: identity, resolved model and permission mode are all on
        // line 1, before any model output.
        assert_eq!(acc.init_model.as_deref(), Some("gemini-3.7-flash-low"));
        assert_eq!(acc.init_permission_mode.as_deref(), Some("request-review"));
        assert!(acc.init_tool_count > 50, "the roster is ~57 tools");
        assert_eq!(acc.agent_response_steps, 1);
        assert_eq!(acc.tool_steps, 0);
        assert_eq!(acc.unparsed_lines, 0);
        assert_eq!(acc.status(), Some("SUCCESS"));
        assert_eq!(acc.last_response.as_deref(), Some("pong\n"));
        assert_eq!(
            kinds(&events),
            vec![
                "usage.updated",                    // the step's own usage
                "conversation.assistant.completed", // the resolved step's text
                "usage.updated",                    // the terminal's usage
            ]
        );
        // Per-step usage is carried verbatim, never a synthetic sum.
        assert_eq!(events[0].payload["scope"], json!("step"));
        assert_eq!(events[0].payload["usage"]["total_tokens"], json!(13819));
        assert_eq!(events[2].payload["scope"], json!("turn"));
        assert_eq!(events[2].payload["usage"]["total_tokens"], json!(13819));
        assert_eq!(
            classify_terminal(&acc, Some(0), false, ""),
            TerminalOutcome::Completed
        );
    }

    #[test]
    fn per_step_and_terminal_usage_become_usage_events() {
        // The `usage` admission row's test. Two scopes, both verbatim, and the
        // per-step one arrives while the turn is still streaming.
        let (_, events) = replay(TOOL_USE);
        let usage: Vec<_> = events
            .iter()
            .filter(|e| e.kind == "usage.updated")
            .collect();
        assert!(usage.len() >= 2);
        assert_eq!(usage[0].payload["scope"], json!("step"));
        assert!(usage[0].payload["usage"]["input_tokens"].is_number());
        let turn = usage.last().expect("a terminal usage event");
        assert_eq!(turn.payload["scope"], json!("turn"));
        assert_eq!(turn.payload["usage"]["total_tokens"], json!(27997));
        assert_eq!(turn.payload["num_turns"], json!(1));
    }

    #[test]
    fn the_tool_use_fixture_produces_exactly_one_requested_completed_pair() {
        let (acc, events) = replay(TOOL_USE);
        assert_eq!(
            acc.tool_steps, 1,
            "the resolved line is the only completion"
        );
        let requested: Vec<_> = events
            .iter()
            .filter(|e| e.kind == "tool.requested")
            .collect();
        let completed: Vec<_> = events
            .iter()
            .filter(|e| e.kind == "tool.completed")
            .collect();
        // The harness emits an ACTIVE `tool_info` and then a resolved one for
        // the same call; `requested_tools` makes the pair idempotent.
        assert_eq!(requested.len(), 1);
        assert_eq!(completed.len(), 1);
        assert_eq!(requested[0].payload["name"], json!("run_command"));
        assert_eq!(
            requested[0].payload["input"]["CommandLine"],
            json!("echo agy-w1-probe")
        );
        assert_eq!(completed[0].payload["state"], json!("DONE"));
        assert_eq!(completed[0].payload["is_error"], json!(false));
        assert_eq!(completed[0].payload["denied"], json!(false));
        assert_eq!(completed[0].payload["has_output"], json!(true));
    }

    #[test]
    fn pty_carriage_returns_are_normalized_in_events_but_not_in_the_raw_blob() {
        // The measured tool output is `"agy-w1-probe\r\n"` (packet 3, W1 P2).
        assert!(
            TOOL_USE.contains("agy-w1-probe\\r\\n"),
            "the fixture must still carry the CRLF the harness wrote"
        );
        let (_, events) = replay(TOOL_USE);
        let completed = events
            .iter()
            .find(|e| e.kind == "tool.completed")
            .expect("tool.completed");
        let tail = completed.payload["output_tail"].as_str().expect("tail");
        assert_eq!(tail, "agy-w1-probe\n");
        assert!(!tail.contains('\r'));
        // And `normalize_pty` is applied to nothing else: parameters are
        // structured JSON whose `\r\n` is the actor's own data.
        assert_eq!(normalize_pty("a\r\nb"), "a\nb");
        assert_eq!(normalize_pty("a\rb"), "a\rb");
    }

    #[test]
    fn no_tool_event_is_ever_produced_from_narration() {
        // The narration rule as a structural assertion: a `text_delta` that
        // describes running a command produces zero `tool.*` events. The only
        // path to one is a `tool_info` object.
        let narration = r#"{"event":"init","conversation_id":"c","init":{"model":"m","permission_mode":"request-review","tools":[]}}
{"event":"step_update","step_update":{"conversation_id":"c","step_index":0,"state":"DONE","step_type":"agent_response","text_delta":"I ran `echo hi` with run_command and the tool_info said DONE; the output was agy-w1-probe.\n"}}
{"event":"result","result":{"conversation_id":"c","status":"SUCCESS","response":"I ran `echo hi`.\n","num_turns":1,"usage":{"total_tokens":1}}}"#;
        let (acc, events) = replay(narration);
        assert_eq!(acc.tool_steps, 0);
        assert!(acc.denied_tools.is_empty());
        assert!(
            !kinds(&events).iter().any(|kind| kind.starts_with("tool.")),
            "prose is never tool evidence: {:?}",
            kinds(&events)
        );
    }

    #[test]
    fn a_partial_delta_and_its_completion_become_one_assistant_event() {
        // Measured in the json-schema capture: step 2 emits ACTIVE with "pong"
        // and then DONE with "\n". Deltas are accumulated per step_index and
        // one event is emitted on the step's resolution, never one per frame.
        let (acc, events) = replay(JSON_SCHEMA);
        let assistant: Vec<_> = events
            .iter()
            .filter(|e| e.kind == "conversation.assistant.completed")
            .collect();
        assert_eq!(assistant.len(), 2, "two resolved agent_response steps");
        assert_eq!(assistant[0].payload["text"], json!("pong\n"));
        assert!(acc.text_deltas > assistant.len() as u32);
        assert!(
            acc.step_texts.is_empty(),
            "every resolved step's buffer is cleared"
        );
    }

    #[test]
    fn the_json_schema_fixture_carries_structured_output_beside_the_response() {
        let (acc, _) = replay(JSON_SCHEMA);
        assert_eq!(acc.status(), Some("SUCCESS"));
        assert_eq!(
            acc.structured_output,
            Some(json!({"word": "pong"})),
            "the validated object sits BESIDE the prose response, not instead of it"
        );
        let Terminal::Status { response, .. } = &acc.terminal else {
            panic!("a terminal status");
        };
        assert!(response.contains("pong"), "the prose response survives too");
    }

    #[test]
    fn a_step_type_this_decoder_does_not_know_is_counted_never_interpreted() {
        // `checkpoint`, `system_message` and `finish` are all measured step
        // types this vocabulary has no kind for.
        let (acc, _) = replay(RESUME_TURN);
        assert!(
            acc.unknown_events
                .iter()
                .any(|e| e == "step_type:system_message")
        );
        let (minimal, _) = replay(MINIMAL_TURN);
        assert!(
            minimal
                .unknown_events
                .iter()
                .any(|e| e == "step_type:checkpoint")
        );
        let (schema, _) = replay(JSON_SCHEMA);
        assert!(
            schema
                .unknown_events
                .iter()
                .any(|e| e == "step_type:finish")
        );
    }

    #[test]
    fn a_resumed_turns_step_indices_are_conversation_scoped() {
        // W1 P4: the resumed turn's first step_index is 5, not 0. Nothing here
        // may assume a turn starts its numbering over.
        let (acc, events) = replay(RESUME_TURN);
        assert_eq!(
            acc.conversation_id.as_deref(),
            Some("e32d8a74-ce73-4a51-86be-8d3d91724430")
        );
        let assistant = events
            .iter()
            .find(|e| e.kind == "conversation.assistant.completed")
            .expect("assistant event");
        assert_eq!(assistant.payload["step"], json!(7));
        assert_eq!(acc.last_response.as_deref(), Some("`sleep 120`\n"));
    }

    #[test]
    fn an_unrecognized_event_kind_is_counted_never_decoded() {
        let (acc, events) = replay(r#"{"event":"quantum_flux","payload":{"x":1}}"#);
        assert_eq!(acc.unknown_events, vec!["quantum_flux".to_string()]);
        assert!(events.is_empty());
        let (acc, _) = replay("{\"no_event_field\":true}");
        assert_eq!(acc.unknown_events, vec!["<no event field>".to_string()]);
        let (acc, _) = replay("not json at all");
        assert_eq!(acc.unparsed_lines, 1);
    }

    // -------------------------------------------------- terminal classification

    #[test]
    fn a_denied_tool_call_is_a_cancelled_turn_not_a_hang() {
        // The `non_blocking_run` admission row's test, on the REAL 1.1.19
        // capture: the tool step resolves DONE with no error and no output, the
        // terminal is CANCELED, and the process exited 0 in ~1.5 s.
        let (acc, events) = replay(DENIED_CANCELED);
        assert_eq!(acc.status(), Some("CANCELED"));
        assert_eq!(acc.tool_steps, 1);
        assert!(
            acc.denied_tools.is_empty(),
            "at 1.1.19 the typed detector does NOT fire — stderr is the only signal"
        );
        let completed = events
            .iter()
            .find(|e| e.kind == "tool.completed")
            .expect("tool.completed");
        assert_eq!(completed.payload["state"], json!("DONE"));
        assert_eq!(completed.payload["has_output"], json!(false));
        // Fails closed either way: an unrequested CANCELED is arm 6.
        assert_eq!(
            classify_terminal(&acc, Some(0), false, ""),
            TerminalOutcome::AmbiguousUnknown
        );
        assert_eq!(
            classify_terminal(&acc, Some(0), false, DENIAL_NOTICE),
            TerminalOutcome::AmbiguousUnknown
        );
        // And the stderr detector is what names the reason.
        assert!(denial_evidence_in_stderr(DENIAL_NOTICE));
        assert!(!denial_evidence_in_stderr("some ordinary warning"));
    }

    #[test]
    fn the_packets_hard_deny_shape_is_still_a_typed_failure() {
        // The 1.1.17 shape the packet measured. Kept because a build that emits
        // it must still be handled — and because rule 1 of the detector is
        // written against it.
        let (acc, events) = replay(DENIED_ERROR_TERMINAL);
        assert_eq!(acc.status(), Some("ERROR"));
        assert_eq!(acc.denied_tools, vec!["run_command".to_string()]);
        let completed = events
            .iter()
            .find(|e| e.kind == "tool.completed")
            .expect("tool.completed");
        assert_eq!(completed.payload["denied"], json!(true));
        assert_eq!(completed.payload["is_error"], json!(true));
        assert!(
            events
                .iter()
                .any(|e| e.kind == "conversation.turn.harness_error"
                    && e.payload["phase"] == json!("typed_terminal"))
        );
        let TerminalOutcome::Failed { reason } = classify_terminal(&acc, Some(1), false, "") else {
            panic!("an explicit ERROR is the only route to Failed");
        };
        assert!(reason.contains("permission check failed"));
    }

    #[test]
    fn a_success_terminal_hiding_a_denied_tool_is_ambiguous_not_completed() {
        // §9.3, both detectors. The fixture is SYNTHESIZED from the documented
        // soft-deny shape (its own first line says so) and exists to pin the
        // classifier, not to claim the shape was observed at 1.1.19.
        let (acc, _) = replay(SOFT_DENY_SUCCESS);
        assert_eq!(acc.status(), Some("SUCCESS"));
        assert_eq!(acc.agent_response_steps, 1, "this turn DID produce text");
        // Without denial evidence it would be a clean completion...
        assert_eq!(
            classify_terminal(&acc, Some(0), false, ""),
            TerminalOutcome::Completed
        );
        // ...and with it, the harness's SUCCESS is not believed: a stage
        // completed on that basis is a stage completed on work that did not
        // happen.
        assert_eq!(
            classify_terminal(&acc, Some(0), false, DENIAL_NOTICE),
            TerminalOutcome::AmbiguousUnknown
        );
    }

    #[test]
    fn an_ordinary_failed_command_does_not_trigger_the_denial_rule() {
        // Deliberately narrow: a nonzero exit inside a PERMITTED tool call is
        // normal agent work. A rule that fired on every failed command would
        // poison every honest turn.
        let value: Value = serde_json::from_str(
            r#"{"name":"run_command","parameters":{"CommandLine":"false"},"output":"","error":{"type":"TOOL_ERROR","message":"command exited with status 1"}}"#,
        )
        .expect("parse");
        assert!(!tool_denial_evidence(&value));
        assert!(!denial_evidence_in_stderr(
            "error: command exited with status 1"
        ));
    }

    #[test]
    fn an_empty_success_terminal_is_ambiguous_not_completed() {
        // The panel's amendment. Fail-closed by construction; the fixture is
        // synthesized from 1.1.18's own changelog description and labelled.
        let (acc, _) = replay(EMPTY_SUCCESS);
        assert_eq!(acc.status(), Some("SUCCESS"));
        assert_eq!(acc.agent_response_steps, 0);
        assert_eq!(
            classify_terminal(&acc, Some(0), false, ""),
            TerminalOutcome::AmbiguousUnknown,
            "a SUCCESS with no response and no text-producing step is never completed-clean"
        );
    }

    #[test]
    fn a_textless_agent_response_step_does_not_rescue_an_empty_success() {
        // The measured refinement (W1 P2 control had exactly such a step): a
        // textless `agent_response` is not agent output, so counting it would
        // let an empty-SUCCESS slip past arm 3.
        let stream = r#"{"event":"init","conversation_id":"c","init":{"model":"m","permission_mode":"request-review","tools":[]}}
{"event":"step_update","step_update":{"conversation_id":"c","step_index":0,"state":"DONE","step_type":"agent_response"}}
{"event":"result","result":{"conversation_id":"c","status":"SUCCESS","response":"","num_turns":1,"usage":{"total_tokens":1}}}"#;
        let (acc, _) = replay(stream);
        assert_eq!(acc.agent_response_steps, 0);
        assert_eq!(
            classify_terminal(&acc, Some(0), false, ""),
            TerminalOutcome::AmbiguousUnknown
        );
    }

    #[test]
    fn a_slash_command_result_never_reads_as_a_completed_turn() {
        // Defence in depth: `--disable-slash-commands` means this cannot occur,
        // and if it ever did the empty-SUCCESS rule catches it and
        // `saw_command_result` names why.
        let (acc, events) = replay(SLASH_COMMAND);
        assert!(acc.saw_command_result);
        assert_eq!(acc.status(), Some("SUCCESS"));
        assert!(
            acc.conversation_id.is_none(),
            "an empty id is not an identity"
        );
        assert_eq!(
            classify_terminal(&acc, Some(0), false, ""),
            TerminalOutcome::AmbiguousUnknown
        );
        assert!(!kinds(&events).contains(&"conversation.assistant.completed"));
    }

    #[test]
    fn the_invalid_model_refusal_mints_no_identity_and_carries_the_catalog() {
        let (acc, events) = replay(INVALID_MODEL);
        assert!(acc.conversation_id.is_none());
        assert_eq!(acc.status(), Some("ERROR"));
        let harness_error = events
            .iter()
            .find(|e| e.kind == "conversation.turn.harness_error")
            .expect("a typed terminal error");
        let text = harness_error.payload["error"].as_str().expect("error text");
        assert!(text.contains("not recognized as a known model"));
        assert!(
            text.contains("Gemini 3.7 Flash (Low)"),
            "the whole catalog travels"
        );
        let TerminalOutcome::Failed { reason } = classify_terminal(&acc, Some(1), false, "") else {
            panic!("Failed");
        };
        assert!(reason.contains("Available models"));
    }

    #[test]
    fn the_print_timeout_expiry_is_a_typed_failure_not_a_new_status() {
        // W1 P5: `--print-timeout` expiry lands on arm 1 with a typed error, so
        // arm 8 never has to guess. W3 can compose it as a native deadline.
        let (acc, _) = replay(PRINT_TIMEOUT);
        assert_eq!(acc.status(), Some("ERROR"));
        assert_eq!(
            classify_terminal(&acc, Some(1), false, ""),
            TerminalOutcome::Failed {
                reason: "timeout waiting for response".to_string()
            }
        );
    }

    #[test]
    fn a_sigkilled_turn_is_interrupted_when_we_asked_and_ambiguous_when_we_did_not() {
        // W1 P4: a group SIGKILL truncates the stream with NO terminal of any
        // kind. Arms 9 and 10.
        let (acc, _) = replay(SIGKILL_TRUNCATED);
        assert_eq!(acc.terminal, Terminal::None);
        assert_eq!(acc.status(), None);
        assert_eq!(
            classify_terminal(&acc, None, true, ""),
            TerminalOutcome::InterruptedRunning
        );
        assert_eq!(
            classify_terminal(&acc, None, false, ""),
            TerminalOutcome::AmbiguousUnknown,
            "§25: process death with no terminal and no requested kill fails closed"
        );
    }

    #[test]
    fn an_unrequested_cancel_is_ambiguous_not_an_interrupt() {
        // Arm 6. Treating an unrequested cancel as our own interrupt would be
        // the adapter claiming authorship of an event it did not cause.
        let (acc, _) = replay(DENIED_CANCELED);
        assert_eq!(
            classify_terminal(&acc, Some(0), true, ""),
            TerminalOutcome::InterruptedRunning
        );
        assert_eq!(
            classify_terminal(&acc, Some(0), false, ""),
            TerminalOutcome::AmbiguousUnknown
        );
    }

    #[test]
    fn an_unknown_terminal_status_is_ambiguous_with_the_status_echoed() {
        // Arms 7 and 8, and the whole point of arm 8: a new status string from
        // a future build is already handled.
        for status in ["WAITING", "RUNNING", "TOTALLY_NEW_THING"] {
            let stream = format!(
                r#"{{"event":"init","conversation_id":"c","init":{{"model":"m","tools":[]}}}}
{{"event":"result","result":{{"conversation_id":"c","status":"{status}","response":"words","num_turns":1}}}}"#
            );
            let (acc, _) = replay(&stream);
            assert_eq!(acc.status(), Some(status));
            assert_eq!(
                classify_terminal(&acc, Some(0), false, ""),
                TerminalOutcome::AmbiguousUnknown,
                "{status} must fail closed"
            );
        }
    }

    #[test]
    fn a_nonzero_exit_is_never_by_itself_a_stage_failure() {
        // §15's load-bearing invariant, in both directions: a clean stream with
        // a bad exit is still a completion (the exit code travels into the
        // evidence), and a SUCCESS status is not a completion when an honesty
        // rule says otherwise.
        let (clean, _) = replay(MINIMAL_TURN);
        assert_eq!(
            classify_terminal(&clean, Some(3), false, ""),
            TerminalOutcome::Completed
        );
        let (empty, _) = replay(EMPTY_SUCCESS);
        assert_eq!(
            classify_terminal(&empty, Some(0), false, ""),
            TerminalOutcome::AmbiguousUnknown
        );
    }

    #[test]
    fn terminal_outcome_labels_are_stable_snake_case() {
        assert_eq!(
            terminal_outcome_label(&TerminalOutcome::Completed),
            "completed"
        );
        assert_eq!(
            terminal_outcome_label(&TerminalOutcome::Failed {
                reason: String::new()
            }),
            "failed"
        );
        assert_eq!(
            terminal_outcome_label(&TerminalOutcome::InterruptedRunning),
            "interrupted_running"
        );
        assert_eq!(
            terminal_outcome_label(&TerminalOutcome::AmbiguousUnknown),
            "ambiguous_unknown"
        );
    }

    #[test]
    fn the_resume_fork_warning_is_recognized_on_stderr() {
        assert!(resume_fork_warning_in_stderr(RESUME_WARNING));
        assert!(!resume_fork_warning_in_stderr("everything is fine"));
    }

    // --------------------------------------------------------- config probe

    #[test]
    fn decode_config_probe_reads_the_measured_answer() {
        // The zero-quota `/config` shape, verbatim from W1 P0.2.
        let value: Value = serde_json::from_str(
            r#"{"conversation_id":"","status":"SUCCESS","response":"","num_turns":0,
                "usage":{"total_tokens":0},
                "command":{"name":"config","data":{"config":{
                  "toolPermission":"request-review","permissions":null,
                  "allowNonWorkspaceAccess":false,
                  "trustedWorkspaces":["/home/miztertea/sergeant-rs-workspace"]}}}}"#,
        )
        .expect("parse");
        let probe = decode_config_probe(&value);
        assert!(probe.read);
        assert_eq!(probe.tool_permission.as_deref(), Some("request-review"));
        assert_eq!(
            probe.trusted_workspaces,
            vec![PathBuf::from("/home/miztertea/sergeant-rs-workspace")]
        );
        assert!(!probe.allow_non_workspace_access);
        assert_eq!(probe.allow_rules, 0);
        // Anything that is not that shape is simply "not read" — never a
        // refusal, and never a claim about the configuration.
        assert!(!decode_config_probe(&json!({"status": "SUCCESS"})).read);
    }

    #[test]
    fn decode_config_probe_counts_allow_rules() {
        let value = json!({"command": {"data": {"config": {
            "toolPermission": "strict",
            "permissions": {"allow": ["command(echo)", "command(echo *)"]},
            "trustedWorkspaces": ["/var/tmp/ws"],
            "allowNonWorkspaceAccess": true
        }}}});
        let probe = decode_config_probe(&value);
        assert_eq!(probe.allow_rules, 2);
        assert!(probe.allow_non_workspace_access);
        assert_eq!(probe.tool_permission.as_deref(), Some("strict"));
    }

    // ------------------------------------------------------- admission rows

    /// The structural check that keeps the ledger honest.
    #[test]
    fn admission_rows_agree_with_capabilities() {
        let capabilities = capabilities_for(Transport::Print);
        let v1: &[(&str, bool)] = &[
            ("persistent_sessions", capabilities.persistent_sessions),
            ("native_background", capabilities.native_background),
            ("streaming", capabilities.streaming),
            ("history", capabilities.history),
            ("resume", capabilities.resume),
            ("interrupt", capabilities.interrupt),
            ("model_selection", capabilities.model_selection),
            ("profiles", capabilities.profiles),
            ("approval_flow", capabilities.approval_flow),
            ("human_attach", capabilities.human_attach),
            ("usage", capabilities.usage),
            ("native_subagents", capabilities.native_subagents),
            ("ask", capabilities.ask),
        ];
        for (name, claimed) in v1 {
            let rows: Vec<_> = ADMISSION_ROWS
                .iter()
                .filter(|row| row.capability == *name && row.transport == Transport::Print)
                .collect();
            assert_eq!(
                rows.len(),
                1,
                "{name}: exactly one row per v1 flag per transport"
            );
            assert_eq!(
                rows[0].claimed, *claimed,
                "{name}: the ledger and capabilities() must agree"
            );
        }

        let adapter_local: &[&str] = &[
            "config_injection",
            "permission_mode_reported_at_launch",
            "non_blocking_run",
            "structured_output",
        ];
        for name in adapter_local {
            assert_eq!(
                ADMISSION_ROWS
                    .iter()
                    .filter(|row| row.capability == *name)
                    .count(),
                1,
                "{name}: exactly one adapter-local row"
            );
        }

        for row in ADMISSION_ROWS {
            if row.claimed {
                assert!(
                    !row.admission_test.is_empty(),
                    "{}: a claimed capability with no admission test is a claim nothing checks (L8)",
                    row.capability
                );
            } else {
                assert!(
                    row.admission_test.is_empty(),
                    "{}: an unclaimed capability names no test; the reason lives in `note`",
                    row.capability
                );
            }
            assert!(
                !row.note.is_empty(),
                "{}: every row owes a note",
                row.capability
            );
            assert!(
                !row.tier.is_empty(),
                "{}: use \"-\" for no tier",
                row.capability
            );
        }

        assert_eq!(
            ADMISSION_ROWS.len(),
            v1.len() + adapter_local.len(),
            "every row is one of the thirteen v1 flags or one of the four adapter-local rows — \
             an unlisted row is a claim nothing checks"
        );
    }

    /// This module's own source, and the integration suite's — the only two
    /// places an `admission_test` name may resolve. Reading them as text is how
    /// [`every_admission_test_name_resolves_to_a_real_test`] turns "the ledger
    /// cites a real test" from a claim in a commit message into something the
    /// build enforces (the same trick `tests/coverage_stage_membership.rs` uses
    /// on suite wiring). `include_str!` of this very file is deliberate.
    const THIS_MODULE_SOURCE: &str = include_str!("agy.rs");
    const INTEGRATION_SUITE_SOURCE: &str = include_str!("../../tests/agy_backend.rs");

    /// The gap the template left open: [`admission_rows_agree_with_capabilities`]
    /// checks that a claimed row *names* a test, never that the name resolves,
    /// so a typo or a later rename would keep passing while the ledger cited a
    /// test that does not exist — the exact failure mode this wave's rules call
    /// disqualifying. This closes it mechanically.
    #[test]
    fn every_admission_test_name_resolves_to_a_real_test() {
        let mut checked = 0usize;
        for row in ADMISSION_ROWS {
            if row.admission_test.is_empty() {
                continue;
            }
            let definition = format!("fn {}(", row.admission_test);
            assert!(
                THIS_MODULE_SOURCE.contains(&definition)
                    || INTEGRATION_SUITE_SOURCE.contains(&definition),
                "{}: admission_test `{}` names no `{definition}` in src/backend/agy.rs or \
                 tests/agy_backend.rs — a ledger row citing a test that does not exist is the \
                 defect the last sprint's panel caught",
                row.capability,
                row.admission_test
            );
            checked += 1;
        }
        assert_eq!(
            checked,
            ADMISSION_ROWS.iter().filter(|row| row.claimed).count(),
            "every claimed row owes a resolvable test name, and only claimed rows carry one"
        );
    }

    /// The mechanical half of the test-honesty rule. The human half is the
    /// live tier's own gate and the transcript in the wave PR body.
    #[test]
    fn a_claimed_row_naming_a_live_test_is_labelled_live_measured() {
        for row in ADMISSION_ROWS {
            let names_live = row.admission_test.starts_with("live_agy_");
            let tagged_live = row.evidence == Evidence::LiveMeasured;
            assert_eq!(
                names_live, tagged_live,
                "{}: Evidence::LiveMeasured and a live_agy_* admission test must agree — tagging a \
                 row LiveMeasured whose test never runs live is the exact defect the last \
                 sprint's panel caught",
                row.capability
            );
        }
        // And the two rows that ARE live-measured are the two the live tier
        // actually drives (three turns total).
        let live: Vec<_> = ADMISSION_ROWS
            .iter()
            .filter(|row| row.evidence == Evidence::LiveMeasured)
            .map(|row| row.capability)
            .collect();
        assert_eq!(live, vec!["resume", "model_selection"]);
    }

    #[test]
    fn the_rendered_table_states_the_stability_fact_once() {
        let rendered = render_admission_rows();
        assert_eq!(rendered.matches("stability (all rows)").count(), 1);
        assert!(rendered.contains("MEASURED_FLOOR 1.1.17 is provenance, not a gate (R1)"));
        assert!(rendered.contains("capability | transport | claimed | tier | evidence"));
        assert!(rendered.contains("history | print-stream-json | false"));
        assert!(rendered.contains("config_injection | print-stream-json | true"));
        assert_eq!(
            rendered.lines().count(),
            ADMISSION_ROWS.len() + 2,
            "one header sentence, one column header, one line per row"
        );
    }

    #[test]
    fn the_profiles_divergence_is_declared_in_the_row_itself() {
        // §4.3: the panel adjudicates a declared decision rather than
        // discovering a silent one.
        let row = ADMISSION_ROWS
            .iter()
            .find(|row| row.capability == "profiles")
            .expect("a profiles row");
        assert!(row.claimed);
        assert!(
            row.note.contains("DECLARED DIVERGENCE"),
            "the divergence must be visible in the rendered ledger, not only in prose"
        );
    }

    #[test]
    fn the_backend_declares_its_name_and_scope() {
        let backend = AgyBackend::new(AgyConfig::new(Path::new("/var/tmp/agy-unit")));
        assert_eq!(backend.name(), AGY_BACKEND_NAME);
        assert_eq!(backend.name(), "agy");
        assert_eq!(backend.runtime_scope(), RuntimeScope::PerExecution);
        assert_eq!(backend.capabilities(), capabilities_for(Transport::Print));
    }

    #[test]
    fn the_config_debug_redacts_its_secrets() {
        let mut config = AgyConfig::new(Path::new("/var/tmp/agy-unit"));
        config
            .env
            .insert("GEMINI_API_KEY".into(), "super-secret-value".into());
        config.json_schema = Some(r#"{"type":"object"}"#.into());
        let rendered = format!("{config:?}");
        assert!(!rendered.contains("super-secret-value"), "{rendered}");
        assert!(rendered.contains("<1 vars, redacted>"), "{rendered}");
        assert!(rendered.contains("redacted, 17 bytes"), "{rendered}");
    }

    #[test]
    fn truncate_is_char_boundary_safe() {
        assert_eq!(truncate("héllo", 3), "hél");
        assert_eq!(truncate("héllo", 99), "héllo");
        assert_eq!(truncate("", 4), "");
    }

    /// K1, as a structural source scan: nothing in this module hardcodes a
    /// model id. The pin travels on `StartRequest::model` like any other.
    #[test]
    fn the_adapter_hardcodes_no_model() {
        let source = include_str!("agy.rs");
        let mut offenders = Vec::new();
        for (number, line) in source.lines().enumerate() {
            let trimmed = line.trim_start();
            // Doc comments, ordinary comments and this test's own fixtures are
            // where the measured ids are *documented*, which is the point.
            if trimmed.starts_with("//") {
                continue;
            }
            // What a hardcoded pin would actually look like: a BARE quoted
            // string literal that is nothing but a model id. A prose mention
            // inside a longer `note` or refusal sentence is documentation of
            // the measurement and is exactly what this module is supposed to
            // carry, so it must not trip the scan.
            for (start, _) in line.match_indices('"') {
                let rest = &line[start + 1..];
                let Some(end) = rest.find('"') else { continue };
                let literal = &rest[..end];
                let looks_like_a_model_id =
                    ["gemini-3.", "claude-sonnet-4", "claude-opus-4", "gpt-oss-"]
                        .iter()
                        .any(|prefix| literal.starts_with(prefix))
                        && literal.chars().all(|c| {
                            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.'
                        });
                if looks_like_a_model_id {
                    offenders.push(format!("{}: {}", number + 1, line.trim()));
                }
            }
        }
        // The `#[cfg(test)] mod tests` block below legitimately names ids in
        // assertions; every offender must be inside it.
        let test_mod_line = source
            .lines()
            .position(|line| line.trim() == "#[cfg(test)]")
            .expect("a test module")
            + 1;
        for offender in &offenders {
            let number: usize = offender
                .split(':')
                .next()
                .and_then(|n| n.parse().ok())
                .expect("a line number");
            assert!(
                number > test_mod_line,
                "a model id appears in non-test, non-comment code: {offender}"
            );
        }
    }
}
