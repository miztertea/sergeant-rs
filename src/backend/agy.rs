//! Antigravity (`agy`) adapter: `agy -p <prompt> --output-format stream-json`
//! non-interactive turns over a **harness-minted, server-side durable
//! conversation** (W1 of the *Sergeant speaks Antigravity* sprint,
//! `sergeant-rs-workspace's knowledge/evidence/reference/agy-adapter-2026-08-23.md`, including its panel
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

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use super::codex::KIND_TURN_HARNESS_ERROR;
use super::{
    Backend, BackendError, BackendSignal, BindingSummary, Capabilities, Completion, EventSink,
    ExecutionHandle, NativeEvent, NativeState, Observation, PreparedExecution, ProbeReport,
    ResumeRequest, RuntimeScope, StartRequest,
};
use crate::backend::child;
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

/// The extra flag the **input-loop** transport composes, checked separately
/// from [`REQUIRED_FLAGS`] rather than added to it.
///
/// **Declared deviation from the W3 spec (§2.1 vs §2.8), and the reason.** §2.1
/// says `REQUIRED_FLAGS` gains `--input-format`; §2.8 says an `Auto` resolution
/// whose gate fails resolves to [`Transport::Print`] with a probe *detail*
/// naming the missing flag. Those two sentences cannot both hold: a flag in
/// `REQUIRED_FLAGS` makes the whole probe `available: false`, so a build with no
/// `--input-format` would be refused outright instead of being served on the
/// print transport it fully supports. §2.8's behaviour is the load-bearing one
/// (it is what keeps an older `agy` usable at all), so the membership rule is
/// preserved — `required_flags_are_exactly_what_the_launch_grammar_composes`
/// checks *both* lists against *both* argv builders, in both directions — while
/// the gate lives here. Recorded in the wave PR's K2/deviation ledger rather
/// than resolved silently.
pub const LOOP_GATE_FLAGS: &[&str] = &["--input-format"];

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
/// on an interactive login for up to 60s before giving up on its own.
/// `run_probe` runs inside the daemon's startup backend probe walk, which
/// since #293 happens *after* the runtime descriptor is published and
/// alongside a daemon that is already serving — so an unbounded wait here no
/// longer delays the descriptor the way it did when registration was
/// synchronous. It is still bounded, for two reasons the reordering did not
/// retire: the walk holds this backend's routing gate closed
/// (`backend::ProbeGate`) until its evidence lands, and a stopping daemon has
/// to wait out whatever probe child is in flight. An unbounded wait would put
/// an interactive login prompt on both, on any host that has `agy` installed
/// but is not logged in. Five seconds is
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

/// Composed-prompt bytes the **input-loop** transport can carry.
///
/// [`ARGV_PROMPT_CAP`] does **not** bind here: the prompt travels as the
/// `content` of one NDJSON line on stdin, not on argv [W3 P1], which is a real
/// capability delta — the loop can carry a `CONTEXT.md` that print mode refuses
/// at PREPARE. It is deliberately **not** claimed as unbounded. Nothing
/// measured a limit, and "no measured limit" is not "no limit"; 16 MiB is the
/// same order as [`STREAM_MEMORY_CAP`], and PREPARE must refuse on the
/// transport it will actually launch on rather than on the other one's number.
const LOOP_PROMPT_CAP: usize = 16 * 1024 * 1024;

/// How long a settled loop turn waits for stderr lines that belong to it before
/// attributing what it has (§2.6).
///
/// **Deliberately not [`STDERR_DRAIN_BUDGET`]**, and the difference is the
/// transport's: on print mode that five seconds is a one-shot wait at process
/// exit, while here it would be paid *by every turn* of a long-lived child and
/// would serialize straight into each turn's settle latency. The race it closes
/// is a same-instant one (the notice is written as the `result` is flushed), so
/// a short grace closes it; a line that arrives later is not dropped, it is
/// carried to the next turn and labelled `adjacent`.
const LOOP_STDERR_GRACE: Duration = Duration::from_millis(250);

/// How long STOP waits, after closing the loop child's stdin, for an in-flight
/// turn's `result` before group-killing. Closing stdin is the *graceful*
/// shutdown — [W3 P2] measured queued turns running to completion and the child
/// then exiting 0 with no further event — so this budget is the one place the
/// adapter chooses between "let the turn finish" and "stop now"; it expires
/// into the same group kill INTERRUPT uses.
const LOOP_STOP_DRAIN_BUDGET: Duration = Duration::from_secs(10);

/// How long a failed stdin write waits for the reader thread's death record
/// before falling back to reporting the raw I/O error. Comfortably over
/// [`LOOP_STDERR_GRACE`], which is the wait it is racing: the reader settles the
/// turn and drains stderr before recording the exit, so a SEND that arrives in
/// that window would otherwise report `Broken pipe` instead of the refusal that
/// names the still-resumable conversation.
const LOOP_DEATH_RECORD_GRACE: Duration = Duration::from_secs(3);

/// The terminal `error` string a `--print-timeout` expiry produces [W1 P5] —
/// **and, measured [W3 P4], the one a SIGINT we sent produces too**. `status`
/// can never disambiguate the two, which is why [`classify_terminal`] arm 1
/// consults `interrupt_requested` for exactly this string and why every such
/// terminal carries `terminal_ambiguity` into its evidence.
const LOOP_TIMEOUT_TERMINAL_ERROR: &str = "timeout waiting for response";

/// Prefix every typed stdin-message refusal shares [W3 P1 rows C–F, H]. The
/// adapter constructs only the accepted shape, so a terminal carrying one of
/// those refusals is an **adapter defect**, not a stage failure, and is reported
/// as one.
///
/// **The prefix alone is not the test, and measurement is why.** [W3 A7]
/// measured a SIGINT to an idle loop child producing
/// `stream input cancelled: context canceled` — the same prefix, and the
/// opposite meaning: nothing was malformed, we killed it. Classifying that as an
/// adapter defect would have blamed this file for an interrupt it performed
/// correctly, so [`TurnAccumulator::loop_input_rejection`] requires the prefix
/// **and** one of the measured refusal markers.
const LOOP_INPUT_REJECTION_PREFIX: &str = "stream input ";

/// The three sentence fragments every measured stdin-message refusal contains
/// [W3 P1 rows C–F, H], and which the cancellation terminal contains none of.
const LOOP_INPUT_REJECTION_MARKERS: &[&str] =
    &["is missing the", "has no content", "is not supported"];

/// The terminal a SIGINT produces when it lands **between** turns, while the
/// child is blocked reading stdin [W3 A7].
///
/// The wave's spec expected only [`LOOP_TIMEOUT_TERMINAL_ERROR`] here, on
/// [W3 P4]'s evidence. Re-measuring it found **two** shapes, split by where the
/// signal lands: mid-turn (awaiting the model) gives the timeout string, and
/// idle gives this one. Both are fatal to the child, both are `status: ERROR`,
/// and both are an interrupt when we asked for one — but only the first is
/// *ambiguous*, because only the first collides with a real deadline expiry.
const LOOP_CANCELLED_TERMINAL_ERROR: &str = "stream input cancelled: context canceled";

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
    /// Override for [`LOOP_STOP_DRAIN_BUDGET`], `None` in every production
    /// path. Same per-instance shape, and for the same reason, as
    /// [`AgyConfig::init_line_budget`]: STOP's bounded graceful shutdown has two
    /// outcomes — the in-flight turn settles, or the budget expires into the
    /// group kill — and the second is only testable deterministically if a test
    /// can shrink the budget below the turn it is racing.
    pub stop_drain_budget: Option<Duration>,
    /// Which transport to run on (W3). `Auto` — the default — resolves from the
    /// `--help` text the probe already read, spawning nothing extra.
    pub transport: TransportChoice,
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
            .field("stop_drain_budget", &self.stop_drain_budget)
            .field("transport", &self.transport)
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
            stop_drain_budget: None,
            transport: TransportChoice::default(),
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

/// Which transport a row's evidence was gathered on. W1 carried this column
/// with one variant precisely so that W3's second transport could not silently
/// re-attribute every existing row to itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Transport {
    /// `agy -p … --output-format stream-json`, W1's transport: one OS process
    /// per turn.
    Print,
    /// `agy --print= --input-format stream-json --output-format stream-json`,
    /// W3's: **one process for the whole execution**, one NDJSON `user` message
    /// per turn on its stdin [W3 P0/P1/P2].
    Loop,
}

impl Transport {
    fn as_str(self) -> &'static str {
        match self {
            Transport::Print => "print-stream-json",
            Transport::Loop => "input-loop-stream-json",
        }
    }
}

/// Which transport an operator asks this backend to run on.
///
/// `opencode.rs`'s `TransportChoice` verbatim in shape, and deliberately so
/// (R2). The one thing that differs is what `Auto` *costs*, and it is the whole
/// reason this enum is safe to resolve inside daemon registration — see
/// [`AgyBackend::transport_resolution`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransportChoice {
    /// Resolve from the memoized probe: the loop when the installed `--help`
    /// offers `--input-format`, print otherwise.
    #[default]
    Auto,
    /// Always [`Transport::Print`], whatever the installed build offers.
    PrintOnly,
    /// Always [`Transport::Loop`]; a build whose `--help` does not offer
    /// `--input-format` probes **`available: false`** naming the flag, rather
    /// than being quietly served on the other transport (codex §5.2 rule 2).
    LoopOnly,
}

/// How [`TransportChoice`] resolved against the installed build. Computed once,
/// from a value the probe already had.
#[derive(Debug, Clone)]
struct TransportResolution {
    transport: Transport,
    /// `false` only for `LoopOnly` against a build with no `--input-format`:
    /// the one resolution that makes the whole backend unavailable.
    available: bool,
    detail: String,
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
    // ------------------------------------------------------------------
    // Transport::Loop — W3's rows. Every one of these was measured on the
    // input-loop transport specifically; nothing is inherited from the print
    // column, because a claim carried across transports is a claim nobody made.
    // ------------------------------------------------------------------
    AdmissionRow {
        capability: "persistent_sessions",
        transport: Transport::Loop,
        claimed: true,
        tier: "OneChildOneConversation",
        evidence: Evidence::LocallyMeasured,
        admission_test: "a_loop_turn_boundary_resets_the_accumulator_but_not_the_conversation",
        note: "stronger than print's on the same flag: one OS process carries EVERY turn of the \
               execution over one conversation, minted on the child's single `init` line and \
               never re-minted (W3 P2: no second init, every step_update and every result carries \
               the same conversation_id). --conversation still re-adopts it from a fresh child \
               (W3 P3), so the print column's claim also holds here",
    },
    AdmissionRow {
        capability: "native_background",
        transport: Transport::Loop,
        claimed: false,
        tier: "-",
        evidence: Evidence::DocClaimed,
        admission_test: "",
        note: "unchanged from print and for a related reason: the `schedule` tool is in the roster \
               (W3 A1 measured the model actually CALLING it, to wait for a subagent) but nothing \
               measured a mechanism for work to outlive the child, and closing stdin lets queued \
               turns finish and then exits (W3 P2). Documented is not supported",
    },
    AdmissionRow {
        capability: "streaming",
        transport: Transport::Loop,
        claimed: true,
        tier: "-",
        evidence: Evidence::LocallyMeasured,
        admission_test: "a_loop_child_streams_each_turns_events_before_the_next_is_written",
        note: "W3 P2's timeline shows text_delta steps arriving mid-turn, 2 s before the terminal, \
               on a child that then ran a second turn. Same decoder as print (ADR 0020/0021's \
               seam): the loop is a driver, not a second decoder",
    },
    AdmissionRow {
        capability: "history",
        transport: Transport::Loop,
        claimed: false,
        tier: "-",
        evidence: Evidence::Unmeasured,
        admission_test: "",
        note: "no export verb exists on either transport, so §15's rule stands unchanged: \
               Backend::history refuses rather than emulating. The loop adds ONE lead and does not \
               promote it — W3 A1's subagent_info carries a log_uri pointing at a real \
               transcript.jsonl under the settings home, which is a CHILD trajectory's log and not \
               this conversation's history",
    },
    AdmissionRow {
        capability: "resume",
        transport: Transport::Loop,
        claimed: true,
        tier: "InitEchoAtChildStart",
        evidence: Evidence::LiveMeasured,
        admission_test: "live_agy_loop_resume_echoes_the_conversation_before_any_turn",
        note: "the same check as print's ConversationIdEchoOnNextTurn, moved BEFORE the first \
               turn: --conversation on the loop grammar makes the child echo the requested id on \
               its init line at child start, so the silent-fork guard (W1 P0.6) costs ZERO quota \
               and runs once per child instead of once per turn. Measured live both ways in W3: \
               the real id echoed back exactly, and an unknown id echoed a DIFFERENT fresh id with \
               `warning: conversation \"…\" not found` on stderr — both for no turns at all",
    },
    AdmissionRow {
        capability: "interrupt",
        transport: Transport::Loop,
        claimed: true,
        tier: "ProcessTreeTermination",
        evidence: Evidence::LocallyMeasured,
        admission_test: "loop_interrupt_group_kills_the_child_and_its_grandchild",
        note: "the UPGRADE CANDIDATE IS REFUTED and the group kill stands. W3 P4: SIGINT to a loop \
               child yields no INTERRUPTED status anywhere, kills the child within ~100 ms (there \
               is no cancel-the-turn-keep-the-session gesture), and emits status ERROR with error \
               \"timeout waiting for response\" — BYTE-IDENTICAL to a --print-timeout expiry \
               (W1 P5). So a SIGINT-first ladder would trade a measured guarantee for a \
               mislabelled terminal. native_interrupt_refuted; the downgrade is journaled rather \
               than implicit (codex §7.3), and classify_terminal arm 1a now reads that terminal as \
               InterruptedRunning when we asked and Failed when we did not, carrying \
               terminal_ambiguity=timeout_or_interrupt in both readings",
    },
    AdmissionRow {
        capability: "model_selection",
        transport: Transport::Loop,
        claimed: true,
        tier: "InitEchoVerifiedPin",
        evidence: Evidence::LiveMeasured,
        admission_test: "live_agy_loop_resume_echoes_the_conversation_before_any_turn",
        note: "same tier as print, and a cost no adapter in the registry can match: init.model \
               echoes the resolved pin at CHILD START, before any message is consumed (W3 P1 row \
               I proved init arrives even when stdin is closed with nothing written), so a \
               Substituted verdict refuses the LAUNCH having spent ZERO quota where print mode \
               must burn turn 1 to find out. One live test backs two rows, which is why they share \
               a name",
    },
    AdmissionRow {
        capability: "profiles",
        transport: Transport::Loop,
        claimed: true,
        tier: "-",
        evidence: Evidence::LocallyMeasured,
        admission_test: "a_profile_executable_and_env_reach_a_loop_child",
        note: "W1's DECLARED DIVERGENCE carries over verbatim and no live turn was spent on it: \
               generic sergeant axes only (executable + env), config_home REFUSED rather than \
               ignored, and agy's own --agent still unwired (W1 P6: the documented workspace \
               mechanism does not work on this host, and `agy agents` is an interactive TUI that \
               hangs headless). The loop changes nothing about this row except that env now \
               reaches ONE child rather than one per turn, which is strictly easier to guarantee",
    },
    AdmissionRow {
        capability: "approval_flow",
        transport: Transport::Loop,
        claimed: false,
        tier: "-",
        evidence: Evidence::LocallyMeasured,
        admission_test: "",
        note: "MEASURED false, not merely unmeasured — the plan's headline hope, refuted. \
               ask_permission/ask_custom_permission are in the 57-tool roster, but W3 P1 wrote \
               SIXTEEN candidate reply-event names into a live child and every one but \
               control_request was skipped with `warning: ignoring unsupported stream input \
               message event`; control_request itself is refused as \"not supported yet\" (rc=2, \
               upstream's own word, quoted). There is no message the driver may send to approve or \
               deny. The working permission channel is the one W1 shipped — the settings home via \
               HOME — and it is a LAUNCH-time policy, not an interactive flow; see config_injection",
    },
    AdmissionRow {
        capability: "human_attach",
        transport: Transport::Loop,
        claimed: false,
        tier: "-",
        evidence: Evidence::Unmeasured,
        admission_test: "",
        note: "unchanged: the loop child is still non-interactive (--print=), and \
               --prompt-interactive/-i is a different execution mode this adapter composes nowhere",
    },
    AdmissionRow {
        capability: "usage",
        transport: Transport::Loop,
        claimed: true,
        tier: "-",
        evidence: Evidence::LocallyMeasured,
        admission_test: "a_conversation_scoped_counter_is_never_assumed_to_start_at_zero",
        note: "per-step and terminal usage both present, same as print. The loop adds one hazard \
               this row owns: result.num_turns, step_index and duration_seconds are all \
               CONVERSATION-scoped, not child-scoped (W3 P2 saw 0,1,2 then 3,4 and num_turns 1 \
               then 2; W3 P3's resumed child opened at step_index 5 with num_turns 3 and \
               duration_seconds 133.16). Nothing keys on any of them starting at zero, and \
               duration_seconds is carried verbatim and NEVER read as a turn duration",
    },
    AdmissionRow {
        capability: "native_subagents",
        transport: Transport::Loop,
        claimed: true,
        tier: "TypedSubagentInfoRecord",
        evidence: Evidence::LiveMeasured,
        admission_test: "live_agy_loop_invokes_a_subagent_and_records_its_typed_conversation_id",
        note: "THE WAVE'S HEADLINE, and the first `true` for this flag anywhere in the registry. \
               Admitted on all three pieces of evidence the spec demanded and nothing less: (1) a \
               step_update with step_type \"subagent\" and tool_name invoke_subagent, (2) a TYPED \
               subagent_info payload on it carrying conversation_id 18a52ef3-… — DISTINCT from the \
               parent's 24c4ff64-… — plus a log_uri to the child's own transcript.jsonl, and (3) \
               that step reaching DONE. The measured shape is a LIST \
               (subagent_info.subagents[{type_name, role, initial_prompt, conversation_id, \
               log_uri}]) and not the flat object the changelog's prose implied, and the child's \
               identity appears ONLY on the resolved step — the ACTIVE one carries the first three \
               fields. Explicitly NOT evidence and not accepted: assistant text saying it \
               delegated, a tool step distinguished only by its name, or a subagent_info with no \
               child conversation_id. The child id is carried verbatim into tool.completed so a \
               human can resume that trajectory by hand; sergeant does NOT adopt it as an \
               execution, which would be a second execution nothing prepared",
    },
    AdmissionRow {
        capability: "ask",
        transport: Transport::Loop,
        claimed: false,
        tier: "-",
        evidence: Evidence::LocallyMeasured,
        admission_test: "",
        note: "MEASURED false, and this is the stronger statement of the two refutations. It is \
               not that a question is unrecognisable here — CORTEX_STEP_TYPE_ASK_QUESTION exists \
               and ask_question is in the roster, so a question may well SURFACE — it is that \
               W3 P1 measured there is NO channel to answer one on (sixteen candidate reply events \
               skipped, control_request refused \"not supported yet\", and printmode's complete \
               123-name symbol table contains no answer, reply or permission handler). A question \
               that surfaces with nobody able to answer it is a stage sergeant would park forever. \
               Capabilities::ask forbids guessing a question from prose; this row says even a \
               TYPED one would be unanswerable",
    },
    AdmissionRow {
        capability: "config_injection",
        transport: Transport::Loop,
        claimed: true,
        tier: "SettingsHomeViaHome",
        evidence: Evidence::LocallyMeasured,
        admission_test: "a_settings_home_reaches_a_loop_child",
        note: "W1's measured channel, re-exercised on this transport for every paid W3 probe: the \
               subagent admission, the denied-tool admission and the sandbox probe each ran under \
               their own HOME=<dir> with their own settings.json and got three different measured \
               behaviours out of it, which is the channel working. W3 also read the \
               AUTHORITATIVE permission-rule namespace list out of the binary — the regex \
               ^(command|read_file|write_file|read_url|mcp|execute_url|unsandboxed)\\s*\\(.*\\)$ — \
               two namespaces (execute_url, unsandboxed) more than W1's docs-derived list. W3 \
               still synthesizes NO policy: mapping a Work's mutation surface onto those \
               namespaces remains unbuilt, because a policy invented here is a security decision \
               with no measurement behind it",
    },
    AdmissionRow {
        capability: "permission_mode_reported_at_launch",
        transport: Transport::Loop,
        claimed: true,
        tier: "InitEchoPermissionModeBeforeAnyTurn",
        evidence: Evidence::LocallyMeasured,
        admission_test: "a_loop_launch_learns_identity_and_posture_before_any_message_is_written",
        note: "the rung-(b) honesty check, and on this transport it is read BEFORE turn 1 rather \
               than during it: init lands at child start, so the posture notice and the \
               cwd_outside_trusted_workspaces notice are both emitted while zero quota has been \
               spent. Still deliberately over-warns for W1's reason (the mode string predicts \
               nothing), and W3 measured a third value's behaviour for the first time — see the \
               sandbox row for what proceed-in-sandbox actually did",
    },
    AdmissionRow {
        capability: "non_blocking_run",
        transport: Transport::Loop,
        claimed: true,
        tier: "DeniedToolKillsTheChild",
        evidence: Evidence::LocallyMeasured,
        admission_test: "a_denied_tool_on_the_loop_kills_the_child_and_the_next_send_is_refused",
        note: "the non-hang guarantee holds — it resolved in 4.2 s — but the SHAPE INVERTS W1 and \
               this is the wave's most operationally important measurement. W3 A2, one live turn \
               with no allow-rule: the tool step resolved ACTIVE->ERROR carrying the PACKET'S OWN \
               1.1.17 typed shape (tool_info.error {type TOOL_ERROR, message \"permission check \
               failed … user denied permission to run command\"}), the terminal was ERROR with \
               that same string, stderr was EMPTY — no auto-denied notice at all — and the CHILD \
               EXITED 1, so the second message queued behind it never ran. On print mode 1.1.19 \
               the same stimulus gives DONE/CANCELED/exit 0/stderr-only. Two consequences: W1's \
               tool_denial_evidence detector (kept 'in case a build emits it') is the one that \
               fires here, and §2.5's dead-transport path is ROUTINE on this transport rather than \
               exceptional — which is a genuine argument for PrintOnly as an operator's default \
               until a per-Work allow-rule policy exists. The conversation survives: a fresh child \
               re-adopted it at zero quota immediately afterwards",
    },
    AdmissionRow {
        capability: "structured_output",
        transport: Transport::Loop,
        claimed: true,
        tier: "NativeSchemaFlag",
        evidence: Evidence::LocallyMeasured,
        admission_test: "the_loop_schema_fixture_carries_structured_output_on_every_turn",
        note: "the open question is CLOSED and it closed the good way. --help says --json-schema \
               is \"for stream-json, only applicable to the final result\", which on a multi-turn \
               child is ambiguous between the final result of each TURN and of the CHILD. W3 A3, \
               two live turns through one child with a two-field schema: BOTH results carried a \
               validated structured_output ({word:alpha,n:1} then {word:bravo,n:2}) plus a \
               json_schema echo. So the tier is unchanged from print and a None on an intermediate \
               turn would be an anomaly, not an expectation. The channel is still \
               AgyConfig::json_schema and this wave synthesizes no schema (K2)",
    },
    AdmissionRow {
        capability: "turn_serialization",
        transport: Transport::Loop,
        claimed: true,
        tier: "HarnessQueuesAndSerializes",
        evidence: Evidence::LiveMeasured,
        admission_test: "live_agy_loop_invokes_a_subagent_and_records_its_typed_conversation_id",
        note: "W3 P2 wrote two messages back-to-back at t=0.001 s with no wait and they ran \
               STRICTLY sequentially — turn 2's first step landed 205 ms after turn 1's result, \
               never interleaved — so the documented \"wait for result before the next line\" rule \
               is a courtesy, not a liveness requirement. The adapter keeps its own \
               one-turn-in-flight rule anyway, for two reasons said out loud: nothing measured a \
               BOUND on that queue, and sergeant's SEND contract is per-turn regardless of what \
               the harness would tolerate. The live test drives two turns through one child, one \
               after the other, which is the rule being exercised rather than merely asserted",
    },
    AdmissionRow {
        capability: "identity_before_first_turn",
        transport: Transport::Loop,
        claimed: true,
        tier: "InitAtChildStart",
        evidence: Evidence::LiveMeasured,
        admission_test: "live_agy_loop_resume_echoes_the_conversation_before_any_turn",
        note: "the transport's real prize, and a registry first. init is emitted at CHILD START, \
               before any message is consumed — proven by W3 P1 row I, an empty-stdin child that \
               emitted init and exited 0 having consumed nothing, and re-run by this wave as its \
               own zero-turn transport probe. Consequences, all free: the conversation id, the \
               resolved model and the effective permission_mode are known before quota is spent, \
               so verify_pin_from_init's Substituted verdict refuses the LAUNCH for ZERO turns, \
               PermissionPosture::from_init and the trusted-workspace notice are emitted for zero \
               turns, and on a resume the silent-fork check runs once, free, at child start. If \
               init does NOT arrive within INIT_LINE_BUDGET the launch fails closed and the child \
               is group-killed — W1's rule verbatim",
    },
    AdmissionRow {
        capability: "prompt_channel",
        transport: Transport::Loop,
        claimed: true,
        tier: "StdinNdjsonNoArgvCap",
        evidence: Evidence::LocallyMeasured,
        admission_test: "a_prompt_over_the_loop_cap_is_refused_at_prepare_not_truncated",
        note: "a real capability delta over print: the prompt travels as the content of one NDJSON \
               line on stdin, so ARGV_PROMPT_CAP's measured 131072-byte E2BIG wall (W1 P0.4) does \
               NOT bind and the loop can carry a CONTEXT.md that print mode refuses at PREPARE. It \
               is NOT claimed as unbounded: nothing measured a limit, \"no measured limit\" is not \
               \"no limit\", and LOOP_PROMPT_CAP (16 MiB) refuses at PREPARE on the transport this \
               execution will actually launch on. The line is serialized from a typed struct, \
               never string-formatted — a hand-built line carrying arbitrary stage text is a \
               JSON-injection defect waiting for a newline, and W3 P1 measured that a malformed \
               line is fatal to the WHOLE CHILD",
    },
    AdmissionRow {
        capability: "sandbox",
        transport: Transport::Loop,
        claimed: false,
        tier: "-",
        evidence: Evidence::LocallyMeasured,
        admission_test: "",
        note: "HONEST SILENCE, and it is now an argued decision rather than an absence — W4 owes \
               ADR 0022 either way. Free reconnaissance first: nsjail appears NOWHERE in the \
               installed binary, nor does sandbox-exec, so the packet's OS-native-mechanism claim \
               is website documentation with no corroboration in the shipped artifact. Grammar: \
               --sandbox and --add-dir are accepted on the loop grammar and change NOTHING \
               observable in init (permission_mode still request-review, no sandbox field), so \
               sandbox state is not launch-observable and this adapter must not pretend to report \
               it. Then one paid turn (W3 S1): toolPermission proceed-in-sandbox with NO \
               permissions.allow at all, launched --sandbox, asking for run_command. The \
               permission gate DID lift — there was no auto-deny and no \"user denied permission\" \
               anywhere — and the tool then failed at the MECHANISM: tool_info.error {TOOL_ERROR, \
               \"connecting to sandbox server: read unix @->@: recvmsg: connection reset by \
               peer\"}, a retry that resolved DONE with no output, and a terminal ERROR carrying \
               the same string. So proceed-in-sandbox is evidenced as a real SECOND PERMISSION \
               CHANNEL (one needing no per-Work allow-rule synthesis) on a host where the sandbox \
               itself does not run — which is exactly why nothing is claimed, and why the adapter \
               composes neither --sandbox nor --add-dir by default on either transport: an \
               uninvited sandbox here is not merely an invented launch decision, it is a broken \
               one. S2 and S3 were CUT deliberately, not for budget: with no working sandbox \
               server on this host, a write-escape probe would have measured the same connect \
               failure again",
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
        // **The divergence between the two columns is deliberately small, and
        // that is the honest result.** The loop's wins are in *cost and timing*
        // — zero-quota identity, a pre-turn pin refusal, a zero-quota
        // resume-fork check, no argv cap — and those live in tiers, notes and
        // adapter-local rows, not in v1 booleans. A wave that flipped booleans
        // to look productive would be the defect this ledger exists to prevent.
        //
        // Exactly one boolean moves, and only on the typed record [W3 A1]
        // demanded of it: `native_subagents`, the first `true` for that flag
        // anywhere in the registry. `ask` and `approval_flow` do NOT move and
        // are now **measured** false rather than merely unmeasured false
        // (§3.1/§3.2) — a refutation is a result, not a gap.
        Transport::Loop => Capabilities {
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
            native_subagents: true,
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
    /// Whether the `--help` text this probe already read offers every entry of
    /// [`LOOP_GATE_FLAGS`]. **The whole of `TransportChoice::Auto`'s
    /// resolution**, and the reason it costs nothing: it is a substring test on
    /// a string the probe had in hand, so `capabilities()` called straight from
    /// `daemon::start_with` spawns no process, opens no port and builds no HTTP
    /// client. That is the 0.2.2 daemon-panic lesson (c46152a2) applied by
    /// construction rather than by isolation-thread patchwork.
    loop_gate: bool,
    /// The `--help` entries [`LOOP_GATE_FLAGS`] wanted and did not find, for the
    /// resolution detail to name.
    loop_gate_missing: Vec<&'static str>,
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

/// The **input-loop** child's argv, after `<executable>` [W3 P0].
///
/// A sibling of [`first_turn_argv`]/[`resume_turn_argv`] rather than a boolean
/// parameter on them: the two grammars share no positional structure at all —
/// the loop carries **no prompt on argv** — and a shared builder with a mode
/// flag would be one `if` away from composing a print turn with no prompt.
///
/// Three measured rules, each load-bearing:
///
/// - **`--print=` with the `=` and an empty value is mandatory.** A bare `-p`
///   consumes the next flag as its prompt and fails **rc=2 with plain-text
///   stderr and no NDJSON at all** [W3 P0] — a shape the stream decoder can
///   never see, so it must never be composed.
/// - **`--input-format stream-json` requires `--output-format stream-json`**;
///   the binary refuses otherwise (`Error: --input-format %s requires
///   --output-format %s`). They are composed together or not at all.
/// - `--disable-slash-commands` on **every** child (W1's rule).
///
/// `--sandbox`/`--add-dir` are composed by neither transport: [W3 S1] measured
/// `--sandbox` on this host failing every `run_command` at
/// `connecting to sandbox server`, so a sandbox the operator did not ask for is
/// not merely an uninvited launch decision — it is a broken one.
fn loop_argv(
    model: Option<&str>,
    json_schema: Option<&str>,
    conversation: Option<&str>,
) -> Vec<String> {
    let mut argv = vec![
        "--print=".to_string(),
        "--input-format".to_string(),
        "stream-json".to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
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
    if let Some(conversation) = conversation {
        argv.push("--conversation".to_string());
        argv.push(conversation.to_string());
    }
    argv
}

/// One stdin line's `message` body.
#[derive(Debug, serde::Serialize)]
struct LoopUserBody<'a> {
    role: &'static str,
    content: &'a str,
}

/// The **only** stdin shape this adapter composes [W3 P1 row A]:
/// `{"event":"user","message":{"role":"user","content":"<prompt>"}}`.
///
/// A typed struct serialized with `serde_json`, never a `format!` — the prompt
/// carries arbitrary stage text, and a hand-built line is a JSON-injection
/// defect waiting for a newline. The block-list form (row B) is equally
/// accepted by the harness and deliberately unused: one wire shape, one
/// fixture, and `"text"` is the only supported block type anyway.
#[derive(Debug, serde::Serialize)]
struct LoopUserMessage<'a> {
    event: &'static str,
    message: LoopUserBody<'a>,
}

/// Serialize one turn's stdin line, **without** its trailing newline.
fn compose_loop_message(prompt: &str) -> String {
    serde_json::to_string(&LoopUserMessage {
        event: "user",
        message: LoopUserBody {
            role: "user",
            content: prompt,
        },
    })
    .expect("a struct of two &'static strs and one &str always serializes")
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
fn check_prompt_budget(
    transport: Transport,
    prompt: &str,
    request: &StartRequest,
) -> Result<(), String> {
    match transport {
        Transport::Print => check_argv_prompt_budget(prompt, request),
        Transport::Loop => check_loop_prompt_budget(prompt, request),
    }
}

/// The loop transport's own bound. See [`LOOP_PROMPT_CAP`] for why there is one
/// at all when nothing measured a limit.
fn check_loop_prompt_budget(prompt: &str, request: &StartRequest) -> Result<(), String> {
    let len = prompt.len();
    if len <= LOOP_PROMPT_CAP {
        return Ok(());
    }
    let intent = request.intent.len();
    let context = request.context.len();
    Err(format!(
        "the composed prompt is {len} bytes, over the input-loop transport's \
         {LOOP_PROMPT_CAP}-byte cap (intent {intent} B, context {context} B). This transport \
         carries the prompt as the `content` of one NDJSON line on stdin, so the {ARGV_PROMPT_CAP}\
         -byte argv cap does NOT bind here — but no upper bound was ever measured either, and \
         \"no measured limit\" is not \"no limit\". Nothing is truncated: dropping part of a \
         stage's CONTEXT.md would be the adapter silently discarding procedure data."
    ))
}

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
    /// ([`tool_denial_evidence`] — the packet's 1.1.17 shape, which [W3 A2]
    /// measured **does** fire on the input-loop transport).
    denied_tools: Vec<String>,
    /// Child conversation ids recovered from a **typed** `subagent_info`
    /// payload [W3 A1]. This is the `native_subagents` admission's own
    /// evidence: a name in a tool roster is not a subagent, a child
    /// `conversation_id` on the wire is.
    subagent_conversations: Vec<String>,
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
        // The subagent path, routed on the same rule and for the same reason.
        // Measured [W3 A1]: an `invoke_subagent` step carries `subagent_info`
        // *instead of* `tool_info`, so a decoder that only looked at
        // `tool_info` would have shown a delegated child trajectory as nothing
        // at all.
        if let Some(subagent_info) = step.get("subagent_info") {
            self.ingest_subagent_info(index, &state, &step_type, resolved, subagent_info, out);
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

    /// Decode a **typed** `subagent_info` payload into the existing tool
    /// vocabulary (R3/K2: no new `KIND_*` constant, which would force a core
    /// `api::SSE_EVENT_KINDS` edit).
    ///
    /// **The measured shape** [W3 A1, the `native_subagents` admission
    /// transcript], which is a list and not the flat object the changelog's
    /// prose implied:
    ///
    /// ```json
    /// "subagent_info": {"subagents": [{
    ///    "type_name": "echoer", "role": "Word Echoer", "initial_prompt": "delta",
    ///    "conversation_id": "18a52ef3-…", "log_uri": "file:///…/transcript.jsonl"}]}
    /// ```
    ///
    /// The `ACTIVE` step carries the first three fields and the `DONE` step
    /// adds `conversation_id` and `log_uri` — so the child's identity exists
    /// only on the **resolved** step, which is exactly why the admission demands
    /// a settled record rather than an in-flight one.
    ///
    /// The child `conversation_id` is carried **verbatim** so a human can resume
    /// that trajectory by hand. Sergeant deliberately does **not** adopt it as an
    /// execution: that would be a second execution nothing prepared, with no
    /// Work, no surface and no journal row of its own.
    fn ingest_subagent_info(
        &mut self,
        index: i64,
        state: &str,
        step_type: &str,
        resolved: bool,
        subagent_info: &Value,
        out: &mut Vec<NativeEvent>,
    ) {
        let name = format!("subagent:{step_type}");
        let id = format!("step-{index}");
        // The same `parameters`-verbatim posture `ingest_tool_info` takes: the
        // whole typed record travels, and nothing here interprets it.
        let subagents = subagent_info
            .get("subagents")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if self.requested_tools.insert(id.clone()) {
            out.push(NativeEvent {
                kind: KIND_TOOL_REQUESTED.to_string(),
                payload: json!({
                    "id": id,
                    "name": name,
                    "input": subagent_info.clone(),
                }),
            });
        }
        if !resolved {
            return;
        }
        self.tool_steps += 1;
        let children: Vec<Value> = subagents
            .iter()
            .map(|child| {
                let conversation = child.get("conversation_id").and_then(Value::as_str);
                if let Some(conversation) = conversation.filter(|id| !id.is_empty()) {
                    self.subagent_conversations.push(conversation.to_string());
                }
                json!({
                    "name": child.get("type_name").cloned().unwrap_or(Value::Null),
                    "role": child.get("role").cloned().unwrap_or(Value::Null),
                    "conversation_id": conversation,
                    "log_uri": child.get("log_uri").cloned().unwrap_or(Value::Null),
                })
            })
            .collect();
        out.push(NativeEvent {
            kind: KIND_TOOL_COMPLETED.to_string(),
            payload: json!({
                "tool_use_id": id,
                "name": name,
                "state": state,
                "is_error": state == "ERROR",
                "error": Value::Null,
                "denied": false,
                "has_output": !children.is_empty(),
                "output_tail": "",
                // The admission's own evidence, in the payload rather than only
                // in a note: a reader can see the child's identity and its log.
                "subagent": children,
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

    /// The typed refusal text when this turn's terminal says the *adapter* sent
    /// a malformed stdin message [W3 P1] — never a stage failure, always an
    /// adapter defect, and fatal to the child either way.
    ///
    /// The adapter constructs only the accepted shape
    /// ([`compose_loop_message`]), so this can only fire if that construction
    /// broke; reporting it as a stage failure would blame a Work for a bug in
    /// this file.
    fn loop_input_rejection(&self) -> Option<&str> {
        match &self.terminal {
            Terminal::Status { error, .. }
                if error.starts_with(LOOP_INPUT_REJECTION_PREFIX)
                    && LOOP_INPUT_REJECTION_MARKERS
                        .iter()
                        .any(|marker| error.contains(marker)) =>
            {
                Some(error.as_str())
            }
            _ => None,
        }
    }

    /// Whether this turn's terminal is one a signal we sent could have caused
    /// [W3 P4, W3 A7] — the timeout string (mid-turn) or the cancellation
    /// string (idle). Only the first is *ambiguous* with a real deadline
    /// expiry; both are an interrupt when `interrupt_requested` is set.
    fn terminal_is_signal_shaped(&self) -> bool {
        matches!(
            &self.terminal,
            Terminal::Status { status, error, .. }
                if status == "ERROR"
                    && (error == LOOP_TIMEOUT_TERMINAL_ERROR
                        || error == LOOP_CANCELLED_TERMINAL_ERROR)
        )
    }

    /// Whether this turn's terminal is the [W3 P4] ambiguity: an `ERROR` whose
    /// text is [`LOOP_TIMEOUT_TERMINAL_ERROR`], which a deadline expiry and an
    /// interrupt we sent produce **identically**.
    fn terminal_is_timeout_ambiguous(&self) -> bool {
        matches!(
            &self.terminal,
            Terminal::Status { status, error, .. }
                if status == "ERROR" && error == LOOP_TIMEOUT_TERMINAL_ERROR
        )
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
        //
        // **Arm 1a, the W3 amendment, and it is a correctness fix rather than a
        // nicety.** [W3 P4] measured a SIGINT to a loop child producing that
        // *same* `ERROR` + `timeout waiting for response` terminal — the status
        // can never disambiguate "the deadline expired" from "we killed it",
        // and only this adapter's own `interrupt_requested` bit can. A stage we
        // interrupted is not a stage that failed, so when we asked for the kill
        // the outcome is `InterruptedRunning`; the ambiguity travels into the
        // evidence in **both** readings as `terminal_ambiguity`, so a reader can
        // see the classifier leant on our bit rather than on the wire.
        "ERROR" | "INVALID" => {
            if interrupted && acc.terminal_is_signal_shaped() {
                return TerminalOutcome::InterruptedRunning;
            }
            TerminalOutcome::Failed {
                reason: if error.is_empty() {
                    format!("agy reported status {status} with no error text")
                } else {
                    error.clone()
                },
            }
        }
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

// ------------------------------------------------------- the loop transport

/// The shared, bounded, **line-stamped** stderr of one loop child (§2.6).
///
/// On print mode stderr is per-turn because the process is per-turn. On the
/// loop it is **one stream for the whole child**, and it carries the only
/// machine-readable evidence of a resume fork [W1 P0.6] — and, on builds that
/// soft-deny, of an auto-denied tool [W1 P2]. So it is read line by line on its
/// own thread and each line is stamped, rather than read to EOF once at the end
/// (which on this transport is the end of the *execution*, far too late for the
/// turn that earned it).
#[derive(Debug, Default)]
struct StderrLog {
    lines: VecDeque<(Instant, String)>,
    bytes: usize,
    /// Lines evicted by the cap. Counted rather than silently forgotten: a
    /// dropped auto-denial notice is the exact hazard this structure exists for.
    dropped: usize,
}

impl StderrLog {
    fn push(&mut self, mut line: String) {
        // A single line longer than the whole cap is truncated rather than
        // allowed to evict everything including itself — the loss is marked, so
        // a capped line reads as "capped" and not as complete.
        if line.len() > STREAM_MEMORY_CAP {
            line = format!(
                "{}...<{STREAM_MEMORY_CAP}-byte stderr line cap hit; the rest was discarded>",
                truncate(&line, STREAM_MEMORY_CAP)
            );
            self.dropped += 1;
        }
        self.bytes += line.len() + 1;
        self.lines.push_back((Instant::now(), line));
        while self.bytes > STREAM_MEMORY_CAP && self.lines.len() > 1 {
            match self.lines.pop_front() {
                Some((_, gone)) => {
                    self.bytes -= gone.len() + 1;
                    self.dropped += 1;
                }
                None => break,
            }
        }
    }

    fn take_all(&mut self) -> Vec<(Instant, String)> {
        self.lines.drain(..).collect::<Vec<_>>()
    }
}

/// One turn's share of the shared stderr stream, and how sure the classifier is
/// that it belongs to that turn.
#[derive(Debug, Clone)]
struct StderrSlice {
    text: String,
    /// `"exact"` when every line landed between this turn's first event and its
    /// `result`; `"adjacent"` when at least one line could only be placed *next
    /// to* a settled turn rather than inside one.
    ///
    /// **Attribution fails closed toward noticing, never toward silence.** A
    /// line that cannot be placed is attached to the turn in flight (or, failing
    /// that, carried to the next one) and labelled — because dropping it would
    /// make an auto-denied tool invisible, which is the exact hazard W1 built
    /// [`denial_evidence_in_stderr`] for.
    attribution: &'static str,
    dropped: usize,
}

impl Default for StderrSlice {
    fn default() -> Self {
        Self {
            text: String::new(),
            attribution: "exact",
            dropped: 0,
        }
    }
}

/// A loop child that has exited, and everything the next SEND owes its caller.
#[derive(Debug, Clone)]
struct LoopDeath {
    exit_code: Option<i32>,
    stderr_tail: String,
    /// **The actionable part.** [W3 P3] measured a conversation resuming
    /// perfectly from a *fresh* child, and [W3 A2] measured it resuming after a
    /// denied tool had killed the child — so a dead transport is not a lost
    /// conversation, and the refusal names the id to resume.
    conversation_id: Option<String>,
}

/// The persistent child one loop-transport execution owns.
///
/// Spawned in LAUNCH (or, for a re-adopted execution, on its first SEND), held
/// by [`AgyExecution`], and execution-scoped: [`RuntimeScope::PerExecution`] is
/// unchanged and `mod.rs`'s ENSURE-RUNTIME seam is untouched.
#[derive(Debug)]
struct LoopChild {
    child: Arc<Mutex<Child>>,
    /// `None` once STOP has closed it. Closing stdin is the graceful shutdown:
    /// [W3 P2] measured queued turns running to completion and the child then
    /// exiting 0 with no further event.
    stdin: Arc<Mutex<Option<std::process::ChildStdin>>>,
    /// Recorded at **spawn** (`process_group(0)` made the child its own group
    /// leader), never derived at kill time — the group can outlive the leader
    /// (opencode probe 11, carried without re-deriving it).
    pgid: Option<u32>,
    reader: Option<std::thread::JoinHandle<()>>,
    /// Set by the reader thread when the child exits. **The one field SEND
    /// consults before writing**: a turn written to a dead pipe would surface as
    /// an I/O error with none of the context that makes it actionable.
    ///
    /// The shared [`StderrLog`] is deliberately *not* held here — the reader
    /// thread owns the only `Arc` that consumes it, and it folds the tail into
    /// this record at death. A second handle here would be a second reader of a
    /// drain-on-read structure, which is how a turn's stderr goes missing.
    death: Arc<Mutex<Option<LoopDeath>>>,
}

/// The identity one loop child learned from its single `init` line, carried
/// across every turn of that child.
///
/// **This is why a per-turn accumulator does not lose the conversation.**
/// [W3 P1 row I / P2] measured `init` arriving once, at child start, before any
/// message is consumed; a turn-2 accumulator that started empty would emit
/// `conversation_id: null` and would make [`verify_pin_from_init`] answer
/// `Attempted` for a pin that was verified — at zero quota — before turn 1 ever
/// ran.
#[derive(Debug, Clone, Default)]
struct LoopIdentity {
    conversation_id: Option<String>,
    model: Option<String>,
    permission_mode: Option<String>,
    cwd: Option<String>,
    tool_count: usize,
}

impl LoopIdentity {
    fn from_accumulator(acc: &TurnAccumulator) -> Self {
        Self {
            conversation_id: acc.conversation_id.clone(),
            model: acc.init_model.clone(),
            permission_mode: acc.init_permission_mode.clone(),
            cwd: acc.init_cwd.clone(),
            tool_count: acc.init_tool_count,
        }
    }

    /// Seed a fresh per-turn accumulator with the child's identity. Everything
    /// else — step counts, texts, tool records, the terminal — starts empty,
    /// which is the whole point of cutting the accumulator at each `result`.
    fn reseed(&self, acc: &mut TurnAccumulator) {
        acc.conversation_id = self.conversation_id.clone();
        acc.init_model = self.model.clone();
        acc.init_permission_mode = self.permission_mode.clone();
        acc.init_cwd = self.cwd.clone();
        acc.init_tool_count = self.tool_count;
    }
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
    /// Which transport this execution runs on, snapshotted at LAUNCH/RESUME.
    /// Per-execution rather than read back off the backend so a resolution that
    /// somehow changed under a running execution could never re-route its
    /// turns halfway through.
    transport: Transport,
    /// The persistent child, on [`Transport::Loop`] only. `None` on the print
    /// transport, and on a re-adopted loop execution until its first SEND
    /// spawns one.
    loop_child: Option<LoopChild>,
}

#[derive(Debug, Default)]
struct AdapterState {
    executions: BTreeMap<String, AgyExecution>,
}

/// One outcome of spawning turn 1 (print) or the loop child (loop), delivered
/// from the reader thread back to the call blocking on it.
#[derive(Debug)]
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
    /// Memoized [`TransportChoice`] resolution. A `OnceLock` beside the probe's
    /// own, not a field of it: the resolution is a pure function of the probe
    /// **and** of the operator's choice, and folding it into `ProbeOutcome`
    /// would make the probe's cached value depend on config it does not carry.
    transport: OnceLock<TransportResolution>,
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
            transport: OnceLock::new(),
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

    /// Resolve [`AgyConfig::transport`] against the installed build, once.
    ///
    /// **This spawns nothing.** The gate is one substring test over a `--help`
    /// text the probe already read (from **stderr**, where 1.1.19 actually
    /// writes it — [W1 P0.1]) plus the parsed version, so `capabilities()`
    /// called straight out of `daemon::start_with` performs no I/O it was not
    /// already doing. opencode's `Auto` had to spawn a serve child and build a
    /// blocking HTTP client to answer the same question, which is the shape of
    /// the 0.2.2 registration panic; agy's needs no process, no port and no
    /// client. `resolving_capabilities_spawns_no_extra_process` pins it.
    ///
    /// **There is no per-execution downgrade here or anywhere.** This runs once,
    /// at probe time, before any execution exists. A loop child that later fails
    /// to spawn, or that never emits `init`, fails LAUNCH honestly (codex §5.3 /
    /// ADR 0021): the print transport remaining available as a *registration*
    /// choice at every capability is what the ruling's word "fallback" means,
    /// never that some execution quietly changed transports mid-flight.
    fn transport_resolution(&self) -> &TransportResolution {
        self.transport.get_or_init(|| {
            let probe = self.probe_outcome();
            match self.config.transport {
                TransportChoice::PrintOnly => TransportResolution {
                    transport: Transport::Print,
                    available: true,
                    detail: format!(
                        "transport: {} (PrintOnly: the operator pinned it)",
                        Transport::Print.as_str()
                    ),
                },
                TransportChoice::LoopOnly if probe.loop_gate => TransportResolution {
                    transport: Transport::Loop,
                    available: true,
                    detail: format!(
                        "transport: {} (LoopOnly: the operator pinned it and --help offers {})",
                        Transport::Loop.as_str(),
                        LOOP_GATE_FLAGS.join(", ")
                    ),
                },
                TransportChoice::LoopOnly => TransportResolution {
                    transport: Transport::Loop,
                    available: false,
                    detail: format!(
                        "capability probe: transport LoopOnly was pinned, but this build's --help                          offers no {}; the persistent stdin turn loop cannot be composed against                          it. Refused rather than served on the print transport, which is a                          different set of measured claims than the one that was asked for.",
                        probe.loop_gate_missing.join(", ")
                    ),
                },
                TransportChoice::Auto if probe.loop_gate => TransportResolution {
                    transport: Transport::Loop,
                    available: true,
                    detail: format!(
                        "transport: {} (Auto: --help offers {})",
                        Transport::Loop.as_str(),
                        LOOP_GATE_FLAGS.join(", ")
                    ),
                },
                TransportChoice::Auto => TransportResolution {
                    transport: Transport::Print,
                    available: true,
                    detail: format!(
                        "transport: {} (Auto: {} absent from --help). This is a RESOLUTION, not a downgrade: it happens once, at probe time, before any execution exists",
                        Transport::Print.as_str(),
                        probe.loop_gate_missing.join(", ")
                    ),
                },
            }
        })
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
        // #310: `output()` waits, so this child cannot outlive *this call* —
        // but it can outlive a daemon SIGKILLed during the call, and a
        // `--help` that never returns is exactly how one gets stuck there.
        child::harden_probe_child(&mut command);
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
        // #310: this is a *full* `agy` invocation, not a `--help`, so it can
        // spawn children of its own and it can block indefinitely on an
        // interactive login prompt. Hardened and group-led so the kill below
        // reaches whatever it started, and so a daemon SIGKILLed while this
        // is in flight takes it with it instead of reparenting it to init.
        child::harden_probe_child(&mut command);
        let Ok(mut spawned) = command.spawn() else {
            return ConfigProbe::default();
        };
        let stdout = spawned.stdout.take();
        let stderr = spawned.stderr.take();
        // Owns the child from here on. Every exit from this function below —
        // the early return, the budget expiry, the ordinary answer, a panic
        // in the parse — goes through this guard's `Drop`, which is the only
        // cleanup an early return cannot skip. Before #310 the `stdout.take()`
        // arm returned with the child still running and nothing at all
        // holding it.
        let probe_child = ConfigProbeChild::adopt(spawned);
        let Some(mut stdout) = stdout else {
            return ConfigProbe::default();
        };
        // Piped but deliberately unread here: a probe that never touches this
        // is not a probe whose stderr this adapter has ever needed. Drained on
        // its own thread purely so a child that writes to it does not block on
        // (or SIGPIPE from) a closed pipe while this call is still waiting on
        // stdout.
        if let Some(mut stderr) = stderr {
            std::thread::spawn(move || {
                let mut sink = Vec::new();
                let _ = stderr.read_to_end(&mut sink);
            });
        }
        let (tx, rx) = std::sync::mpsc::sync_channel::<Vec<u8>>(1);
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = stdout.read_to_end(&mut buf);
            let _ = tx.send(buf);
        });
        // An expired budget yields no bytes at all. Most likely cause, per the
        // measurement in `CONFIG_PROBE_BUDGET`'s doc: an unauthenticated `agy`
        // blocked this call on an interactive login prompt. Killed by the
        // guard below and treated as any other probe failure — never
        // propagated as a hang.
        let stdout_bytes = rx.recv_timeout(CONFIG_PROBE_BUDGET).unwrap_or_default();
        // Explicit, at the completion point, rather than left to scope end:
        // the measurement is finished, so the child has no further reason to
        // exist and the parse below must not run while it does.
        drop(probe_child);
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
        child::harden_probe_child(&mut version_command);
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
                    loop_gate: false,
                    loop_gate_missing: LOOP_GATE_FLAGS.to_vec(),
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
                loop_gate: false,
                loop_gate_missing: LOOP_GATE_FLAGS.to_vec(),
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
                    loop_gate: false,
                    loop_gate_missing: LOOP_GATE_FLAGS.to_vec(),
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
                loop_gate: false,
                loop_gate_missing: LOOP_GATE_FLAGS.to_vec(),
            };
        }
        // Computed here, from the text already read, and never again: this is
        // the entirety of `TransportChoice::Auto`'s resolution work.
        let loop_gate_missing = missing_entries(&help, LOOP_GATE_FLAGS);
        let loop_gate = loop_gate_missing.is_empty();

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
            "{version_clause}; all {} required flags present",
            REQUIRED_FLAGS.len(),
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
            loop_gate,
            loop_gate_missing,
        }
    }

    /// §14 applied to this CLI. `config_home` is **refused here, not ignored**:
    /// no agy config-home *variable* was measured, and honouring the field by
    /// guessing one would make the human's launch decision silently do nothing.
    ///
    /// `causation` is S2 E6's triple ([`crate::backend::causation_env`]),
    /// merged **after** the profile so a workflow-authored `Profile.env` key
    /// cannot shadow what sergeant itself intended to send. It is folded into
    /// the resolved `env` map here rather than added as a third parameter to
    /// [`Self::apply_env`], which is deliberately generic over "whatever env
    /// map was resolved" so a probe call and a turn can never read two
    /// different configurations — a probe reaches `apply_env` with the map
    /// this function never touched. RESUME rebuilds the triple via
    /// [`crate::backend::resume_causation_env`] from
    /// `ResumeRequest::estate_root` (re-supplied, S2 E6) and the execution id
    /// on the `handle` every `resume()` already receives — the resolved env
    /// this produces is what every later turn on the execution reuses, so an
    /// empty map here silently dropped causation for the rest of the
    /// execution's life, not only for the reconciliation snapshot.
    fn launch_config(
        &self,
        profile: Option<&Profile>,
        causation: &BTreeMap<String, String>,
    ) -> Result<LaunchConfig, BackendError> {
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
        for (key, value) in causation {
            env.insert(key.clone(), value.clone());
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

    /// Spawn this execution's persistent loop child and block, bounded, for its
    /// `init` line.
    ///
    /// **No turn is spent here.** [W3 P1 row I] measured `init` arriving at
    /// child start even when nothing is ever written to stdin, so identity, the
    /// resolved model and the effective permission mode are all free — which is
    /// what lets a `Substituted` pin refuse the LAUNCH having spent **zero**
    /// quota, where print mode must burn turn 1 to find out.
    ///
    /// **If `init` does not arrive inside the budget, this fails closed and the
    /// child is group-killed** — W1's rule, verbatim. A loop child that fails to
    /// spawn, or that never speaks, fails LAUNCH honestly; it is never quietly
    /// re-routed onto the print transport.
    fn spawn_loop_child(
        &self,
        execution_id: &str,
        conversation: Option<String>,
    ) -> Result<FirstTurnSignal, BackendError> {
        let plan = self.spawn_plan(execution_id)?;
        let mut command = Command::new(&plan.executable);
        command
            .args(loop_argv(
                plan.model.as_deref(),
                plan.json_schema.as_deref(),
                conversation.as_deref(),
            ))
            .current_dir(&plan.cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        apply_env(&mut command, &plan.env, plan.settings_home.as_deref());
        // Every turn of this execution runs under this one process, so the group
        // is the execution's, not the turn's. Carried from opencode probe 11
        // without re-deriving it (R2).
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }

        let mut child = command.spawn().map_err(|e| {
            self.err_failed(format!(
                "cannot spawn the loop child {:?}: {e}. This LAUNCH fails rather than falling back \
                 to the print transport: a per-execution downgrade would silently change which \
                 measured capability set this execution is running under",
                plan.executable
            ))
        })?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| self.err_failed("loop child stdout was not piped"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| self.err_failed("loop child stdin was not piped"))?;
        let pgid = child.id();
        let stderr_log = Arc::new(Mutex::new(StderrLog::default()));
        if let Some(stderr) = child.stderr.take() {
            let sink = Arc::clone(&stderr_log);
            std::thread::spawn(move || {
                for line in BufReader::new(stderr).lines() {
                    let Ok(line) = line else { break };
                    sink.lock().expect("agy loop stderr lock").push(line);
                }
            });
        }
        let child = Arc::new(Mutex::new(child));
        let death = Arc::new(Mutex::new(None));
        let (tx, rx) = std::sync::mpsc::sync_channel::<FirstTurnSignal>(1);
        let reader = LoopReader {
            backend_state: Arc::clone(&self.state),
            sink: self.sink.lock().expect("agy sink lock").clone(),
            data_dir: self.config.data_dir.clone(),
            execution_id: execution_id.to_string(),
            work_id: plan.work_id.clone(),
            model: plan.model.clone(),
            expected_conversation: conversation.clone(),
            bindings_outside_cwd: plan.bindings_outside_cwd.clone(),
            child: Arc::clone(&child),
            stderr: Arc::clone(&stderr_log),
            death: Arc::clone(&death),
            init_signal: Some(tx),
            settings_home: self.config.settings_home.clone(),
        };
        let reader_handle = std::thread::spawn(move || reader.run(stdout));
        {
            let mut state = self.lock();
            let execution = state
                .executions
                .get_mut(execution_id)
                .ok_or_else(|| self.err_unknown(execution_id))?;
            execution.turn_pgid = Some(pgid);
            execution.loop_child = Some(LoopChild {
                child: Arc::clone(&child),
                stdin: Arc::new(Mutex::new(Some(stdin))),
                pgid: Some(pgid),
                reader: Some(reader_handle),
                death,
            });
        }
        let budget = self.config.init_line_budget.unwrap_or(INIT_LINE_BUDGET);
        match rx.recv_timeout(budget) {
            Ok(signal) => Ok(signal),
            Err(_) => {
                self.kill_loop_child(execution_id);
                Err(self.err_failed(format!(
                    "the agy loop child emitted no `init` line within {budget:?}; it was \
                     group-killed. On this transport `init` precedes any message being consumed \
                     (W3 P1), so a child that has not spoken by now has no identity to hand back \
                     and no turn has been spent"
                )))
            }
        }
    }

    /// Group-kill this execution's loop child, whatever state it is in.
    fn kill_loop_child(&self, execution_id: &str) {
        let (pgid, child) = {
            let state = self.lock();
            let Some(execution) = state.executions.get(execution_id) else {
                return;
            };
            match &execution.loop_child {
                Some(loop_child) => (loop_child.pgid, Some(Arc::clone(&loop_child.child))),
                None => (execution.turn_pgid, None),
            }
        };
        kill_turn(pgid, child.as_ref());
    }

    /// Whether this execution's loop child has already exited, and with what.
    fn loop_death(&self, execution_id: &str) -> Option<LoopDeath> {
        self.lock()
            .executions
            .get(execution_id)?
            .loop_child
            .as_ref()?
            .death
            .lock()
            .expect("agy loop death lock")
            .clone()
    }

    /// The refusal a SEND owes its caller when the transport is gone (§2.5).
    ///
    /// **Not an auto-respawn**, deliberately: [W3 P3] proves a fresh child
    /// resumes a conversation, but respawning silently is how a stage's turn
    /// count starts lying, and inventing a recovery policy is not this wave's
    /// work. So the refusal is made *actionable* instead — it names the exit
    /// code, the stderr tail and the conversation id that is still fully
    /// resumable.
    fn err_loop_transport_dead(&self, execution_id: &str, death: &LoopDeath) -> BackendError {
        self.err_failed(format!(
            "execution {execution_id}'s agy loop child has exited (exit_code={:?}), so this \
             transport is gone and no further turn can be written to it. The conversation is NOT \
             lost: {} is fully resumable from a fresh child (measured W3 P3, and again in W3 A2 \
             after a denied tool killed a child mid-execution). This adapter does not respawn one \
             by itself — a recovery policy invented here is how a stage's turn count starts lying. \
             stderr tail: {}",
            death.exit_code,
            death
                .conversation_id
                .as_deref()
                .unwrap_or("<no conversation was ever minted>"),
            if death.stderr_tail.is_empty() {
                "<empty>"
            } else {
                &death.stderr_tail
            },
        ))
    }

    /// Write exactly one NDJSON `user` message — one turn — to the loop child's
    /// stdin.
    ///
    /// The adapter keeps its own **one-turn-in-flight** rule even though
    /// [W3 P2] measured agy serialising its own queue (two messages written at
    /// t≈0 ran strictly sequentially). Two reasons, said out loud: nothing
    /// measured a *bound* on that queue, and sergeant's SEND contract is
    /// per-turn regardless of what the harness would tolerate.
    fn write_loop_turn(
        &self,
        execution_id: &str,
        prompt: &str,
        instruction_policy: Option<String>,
    ) -> Result<(), BackendError> {
        if prompt.len() > LOOP_PROMPT_CAP {
            return Err(self.err_failed(format!(
                "this turn's prompt is {} bytes, over the input-loop transport's \
                 {LOOP_PROMPT_CAP}-byte cap. Nothing is truncated.",
                prompt.len()
            )));
        }
        if let Some(death) = self.loop_death(execution_id) {
            return Err(self.err_loop_transport_dead(execution_id, &death));
        }
        let line = compose_loop_message(prompt);
        // The handles are lifted out from under the adapter lock and the write
        // happens without it: a blocking write to a child's pipe is I/O, and
        // §22.6's rule about not doing I/O under a lock does not stop being
        // true because the lock is this adapter's own.
        let (child, stdin) = {
            let state = self.lock();
            let execution = state
                .executions
                .get(execution_id)
                .ok_or_else(|| self.err_unknown(execution_id))?;
            let loop_child = execution.loop_child.as_ref().ok_or_else(|| {
                self.err_failed(format!(
                    "execution {execution_id} has no loop child to write a turn to"
                ))
            })?;
            (Arc::clone(&loop_child.child), Arc::clone(&loop_child.stdin))
        };
        {
            let mut handle = stdin.lock().expect("agy loop stdin lock");
            let handle = handle.as_mut().ok_or_else(|| {
                self.err_failed(format!(
                    "execution {execution_id}'s loop child has had its stdin closed (STOP's \
                     graceful shutdown), so no further turn can be written to it"
                ))
            })?;
            handle
                .write_all(line.as_bytes())
                .and_then(|()| handle.write_all(b"\n"))
                .and_then(|()| handle.flush())
                .map_err(|e| self.write_failure(execution_id, e))?;
        }
        let (work_id, conversation, bindings) = {
            let mut state = self.lock();
            let execution = state
                .executions
                .get_mut(execution_id)
                .ok_or_else(|| self.err_unknown(execution_id))?;
            execution.turn = TurnState::InFlight(child);
            execution.turns += 1;
            execution.interrupt_requested = false;
            (
                execution.work_id.clone(),
                execution.conversation_id.clone(),
                execution.bindings_outside_cwd.clone(),
            )
        };
        self.emit(
            execution_id,
            &work_id,
            KIND_CONVERSATION_USER,
            json!({
                "text": prompt,
                "transport": Transport::Loop.as_str(),
                "conversation_id": conversation,
                "bindings_outside_cwd": bindings,
                "instruction_policy_unenforced": instruction_policy,
            }),
        );
        Ok(())
    }

    /// Turn a failed stdin write into the most useful refusal available.
    ///
    /// A child that died a moment ago has not necessarily finished being
    /// *recorded* dead: the reader settles the turn and drains stderr before it
    /// writes the death record. A SEND racing that window would otherwise
    /// surface a bare `Broken pipe` — technically true and operationally
    /// useless — instead of the refusal that names the still-resumable
    /// conversation. So the failure path waits, bounded, for the better answer.
    /// It only ever runs on a write that has already failed.
    fn write_failure(&self, execution_id: &str, e: std::io::Error) -> BackendError {
        let deadline = Instant::now() + LOOP_DEATH_RECORD_GRACE;
        loop {
            if let Some(death) = self.loop_death(execution_id) {
                return self.err_loop_transport_dead(execution_id, &death);
            }
            if Instant::now() >= deadline {
                return self.err_failed(format!(
                    "cannot write this turn to the agy loop child's stdin: {e}, and no exit was \
                     recorded for it within {LOOP_DEATH_RECORD_GRACE:?}. The child is most likely \
                     gone; a malformed message would have been answered with a typed refusal \
                     instead (W3 P1)"
                ));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// LAUNCH on the loop transport: spawn the child, cash in the free identity
    /// window, then write turn 1.
    fn launch_loop(&self, request: &StartRequest) -> Result<ExecutionHandle, BackendError> {
        let signal = self.spawn_loop_child(&request.execution_id, None)?;
        let (conversation_id, model, permission_mode) = match signal {
            FirstTurnSignal::Initialized {
                conversation_id,
                model,
                permission_mode,
            } => (conversation_id, model, permission_mode),
            FirstTurnSignal::RefusedBeforeIdentity {
                status,
                error,
                exit_code,
            } => {
                return Err(self.err_failed(format!(
                    "agy refused this loop child before minting a conversation (status={status}, \
                     exit_code={exit_code:?}), so no identity exists and nothing is resumable — \
                     and no turn was spent finding out. agy's own error, verbatim: {error}"
                )));
            }
            FirstTurnSignal::ExitedWithoutInit {
                exit_code,
                stderr,
                raw_blob,
            } => {
                return Err(self.err_failed(format!(
                    "the agy loop child exited before any `init` line arrived and emitted no \
                     terminal either (exit_code={exit_code:?}), so no conversation was ever \
                     minted; stderr: {}; raw={}",
                    truncate(stderr.trim(), 400),
                    raw_blob
                        .unwrap_or_else(|| "unarchived (the child streamed nothing)".to_string())
                )));
            }
        };
        // The R4 delta cashed in at the earliest possible moment and for the
        // fewest possible tokens — which on this transport is **none at all**.
        let verdict = verify_pin_from_init(request.model.as_deref(), model.as_deref());
        if let PinVerdict::Substituted(served) = &verdict {
            self.kill_loop_child(&request.execution_id);
            return Err(self.err_failed(format!(
                "agy's init line names {served} as the model serving this conversation, but this \
                 execution requested {}. On the input-loop transport the init line arrives before \
                 any message is consumed, so this launch is refused having spent ZERO quota — no \
                 adapter in the registry can otherwise say that.",
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
        self.write_loop_turn(
            &request.execution_id,
            &compose_launch_prompt(request),
            Some(format!("{:?}", request.instruction_policy)),
        )?;
        Ok(ExecutionHandle {
            execution_id: request.execution_id.clone(),
            native_id: Some(conversation_id),
        })
    }

    /// A re-adopted loop execution has no child of this daemon's yet. Spawning
    /// one on its first SEND is **not** the auto-respawn §2.5 refuses: that is
    /// about a child this daemon watched die mid-execution, and this is a
    /// conversation this daemon has never had a child for at all. The
    /// distinction is the whole difference between a recovery policy nobody
    /// specified and RESUME's own contract.
    ///
    /// The silent-resume-fork check runs here, at child start, for zero quota.
    fn adopt_loop_child(&self, execution_id: &str) -> Result<(), BackendError> {
        let conversation = {
            let state = self.lock();
            let execution = state
                .executions
                .get(execution_id)
                .ok_or_else(|| self.err_unknown(execution_id))?;
            if execution.loop_child.is_some() {
                return Ok(());
            }
            execution.conversation_id.clone()
        };
        let conversation = conversation.ok_or_else(|| {
            self.err_failed(format!(
                "execution {execution_id} has no conversation id, so there is nothing to compose \
                 --conversation with"
            ))
        })?;
        let signal = self.spawn_loop_child(execution_id, Some(conversation.clone()))?;
        match signal {
            FirstTurnSignal::Initialized {
                conversation_id, ..
            } if conversation_id == conversation => Ok(()),
            FirstTurnSignal::Initialized {
                conversation_id, ..
            } => {
                self.kill_loop_child(execution_id);
                Err(self.err_failed(format!(
                    "re-adopting conversation {conversation} spawned a loop child whose init line \
                     echoed {conversation_id} instead. agy warns-and-continues on an unknown \
                     conversation and starts a FRESH one (W1 P0.6, re-measured on this transport), \
                     so this is a silent fork caught before a single turn was spent — not a resume."
                )))
            }
            other => {
                self.kill_loop_child(execution_id);
                Err(self.err_failed(format!(
                    "re-adopting conversation {conversation} could not start a loop child: \
                     {other:?}"
                )))
            }
        }
    }

    /// Close the loop child's stdin — the graceful shutdown [W3 P2] — and say
    /// whether there was one to close. `false` on the print transport, which
    /// has nothing to close and whose STOP therefore behaves exactly as W1
    /// shipped it.
    fn close_loop_stdin(&self, execution_id: &str) -> bool {
        let state = self.lock();
        let Some(execution) = state.executions.get(execution_id) else {
            return false;
        };
        let Some(loop_child) = &execution.loop_child else {
            return false;
        };
        // Dropped, not shut down: dropping the handle closes the pipe, which is
        // exactly the stimulus measured.
        loop_child
            .stdin
            .lock()
            .expect("agy loop stdin lock")
            .take()
            .is_some()
    }

    /// Wait, bounded, for a loop execution's in-flight turn to settle after its
    /// stdin was closed. Expiry is not an error — it falls through to the group
    /// kill, which is what a bounded graceful shutdown means.
    fn await_loop_settle(&self, execution_id: &str, budget: Duration) {
        let deadline = Instant::now() + budget;
        loop {
            let in_flight = matches!(
                self.lock()
                    .executions
                    .get(execution_id)
                    .map(|execution| &execution.turn),
                Some(TurnState::InFlight(_))
            );
            if !in_flight || Instant::now() >= deadline {
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
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

/// One `agy -p /config` probe child, killed group-first and reaped when this
/// guard drops (#310 requirement 2: a probe's child dies when the probe
/// completes, with a `Drop` backstop covering every path that is not the
/// ordinary one — an early return, an expired budget, a panic in the parse).
///
/// The group goes first for the reason [`kill_process_group`] documents: this
/// is a whole agent invocation, not a `--help`, so it may have started
/// commands of its own, and a group routinely outlives its leader.
struct ConfigProbeChild {
    child: Child,
    pgid: u32,
    /// Deregisters this pgid from the owning probe walk's live set. Held,
    /// never read.
    _registration: child::ProbeChildRegistration,
}

impl ConfigProbeChild {
    fn adopt(spawned: Child) -> Self {
        let pgid = spawned.id();
        Self {
            child: spawned,
            pgid,
            _registration: child::register_probe_child(pgid),
        }
    }
}

impl Drop for ConfigProbeChild {
    fn drop(&mut self) {
        kill_process_group(Some(self.pgid));
        let _ = self.child.kill();
        // Reaped, not merely signalled: an un-`wait`ed child becomes a zombie
        // the instant it exits, and a zombie still answers `kill(pid, 0)` as
        // alive — which is exactly what an orphan check would then report.
        let _ = self.child.wait();
    }
}

/// Kill a turn's whole process group, by the pgid recorded at spawn.
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
        let (raw_blob, raw_error) = Self::archive(&self.data_dir, &raw);

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

    /// §20's archive, and **this module's single `BlobStore::put` call site**
    /// for both transports.
    ///
    /// An associated function of `TurnReader` on purpose: `tests/
    /// a4_blob_ref_pinning.rs`'s ledger row pins agy's put site to
    /// `impl TurnReader`, and W3 adds a second reader. Routing the loop
    /// transport's archive through here keeps that row true **without a K2 edit
    /// to a core test** — and it is the R2 answer anyway, since one archive path
    /// for both transports is exactly what "the one decoder serves both" means
    /// applied to the blob store.
    ///
    /// Archived **before any conclusion is drawn from it**, and an archive
    /// failure is reported rather than swallowed: the alternative is a turn
    /// whose raw capture silently does not exist.
    fn archive(data_dir: &Path, raw: &str) -> (Option<String>, Option<String>) {
        if raw.is_empty() {
            return (None, None);
        }
        match BlobStore::open(data_dir).and_then(|store| store.put(raw.as_bytes())) {
            Ok(blob_ref) => (Some(blob_ref.to_string()), None),
            Err(e) => (None, Some(e.to_string())),
        }
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

// -------------------------------------------------------- the loop reader

/// Everything the **input-loop** child's stdout reader thread needs.
///
/// One thread for the child's whole life, not one per turn — which is the
/// transport's defining difference and the source of its one genuinely new
/// hazard (§2.6's stderr attribution). It reuses [`TurnAccumulator`],
/// [`classify_terminal`], [`verify_pin_from_init`], [`PermissionPosture`] and
/// [`TurnReader::archive`] **unchanged in behaviour**: this is a driver, not a
/// second decoder (ADR 0020/0021's seam, R2).
struct LoopReader {
    backend_state: Arc<Mutex<AdapterState>>,
    sink: Option<EventSink>,
    data_dir: PathBuf,
    execution_id: String,
    work_id: String,
    model: Option<String>,
    /// The conversation id this child composed `--conversation` with, when it is
    /// a resume. `None` for a fresh child.
    expected_conversation: Option<String>,
    bindings_outside_cwd: Vec<PathBuf>,
    child: Arc<Mutex<Child>>,
    stderr: Arc<Mutex<StderrLog>>,
    death: Arc<Mutex<Option<LoopDeath>>>,
    /// Present until the `init` line lands: how this reader tells the caller
    /// blocking on LAUNCH (or on the first SEND of a re-adopted execution) that
    /// identity arrived — or that the harness refused before minting one.
    init_signal: Option<SyncSender<FirstTurnSignal>>,
    settings_home: Option<PathBuf>,
}

impl LoopReader {
    fn run(self, stdout: std::process::ChildStdout) {
        let mut identity = LoopIdentity::default();
        let mut acc = TurnAccumulator::new();
        let mut raw = String::new();
        let mut raw_truncated = false;
        let mut announced_init = false;
        // Where a turn's stderr slice starts: the first stream event seen since
        // the previous `result`. `None` between turns, which is precisely the
        // window whose stderr can only be attributed `adjacent`.
        let mut first_event_at: Option<Instant> = None;
        // Turn 1's blob, referenced by every later turn's evidence so the `init`
        // line is archived exactly once and every turn still traces back to it.
        let mut init_blob: Option<String> = None;
        let mut turns_settled: u32 = 0;

        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
            // Byte-exact, never normalize_pty'd: §20 fidelity means the blob is
            // what the harness wrote. Every line is still parsed and forwarded
            // regardless of the cap; only the archive is bounded.
            if raw.len() < STREAM_MEMORY_CAP {
                raw.push_str(&line);
                raw.push('\n');
            } else {
                raw_truncated = true;
            }
            if first_event_at.is_none() {
                first_event_at = Some(Instant::now());
            }
            let Ok(value) = serde_json::from_str::<Value>(&line) else {
                acc.unparsed_lines += 1;
                continue;
            };
            let event = value
                .get("event")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            for native in acc.ingest_line(&value) {
                self.emit(&native.kind, native.payload);
            }
            if event == "init" && !announced_init {
                announced_init = true;
                identity = LoopIdentity::from_accumulator(&acc);
                self.announce_identity(&identity);
            }
            // A `result` that arrives **before** any `init` is not a turn: no
            // line has been written to this child's stdin yet, so settling one
            // here would fabricate a `conversation.turn.ended` (and a
            // `TurnState::Finished`) for a turn nobody sent, and would reset the
            // accumulator that carries agy's own refusal text. Leaving it in
            // `acc` is what lets the `!announced_init` classifier below report
            // `RefusedBeforeIdentity` with that text verbatim instead of the
            // said-nothing-at-all `ExitedWithoutInit`. `init` precedes any
            // message being consumed on this transport [W3 P1], so this window
            // only ever holds a harness refusal.
            if event == "result" && announced_init {
                turns_settled += 1;
                let settled = self.settle_turn(
                    &mut acc,
                    &identity,
                    std::mem::take(&mut raw),
                    raw_truncated,
                    first_event_at,
                    init_blob.clone(),
                    None,
                );
                if turns_settled == 1 {
                    init_blob = settled;
                }
                raw_truncated = false;
                first_event_at = None;
                acc = TurnAccumulator::new();
                identity.reseed(&mut acc);
            }
        }

        // Stdout is closed; reap. The child lock is only taken after EOF so
        // INTERRUPT can always kill.
        let exit_code = self
            .child
            .lock()
            .expect("agy loop child lock")
            .wait()
            .ok()
            .and_then(|status| status.code());

        // The harness refused before minting identity, or died saying nothing.
        // Only a caller blocking on the init line has anyone listening.
        if !announced_init && let Some(tx) = &self.init_signal {
            let stderr = self.take_stderr(None).text;
            let (raw_blob, _) = TurnReader::archive(&self.data_dir, &raw);
            let signal = match &acc.terminal {
                Terminal::Status { status, error, .. } => FirstTurnSignal::RefusedBeforeIdentity {
                    status: status.clone(),
                    error: error.clone(),
                    exit_code,
                },
                Terminal::None => FirstTurnSignal::ExitedWithoutInit {
                    exit_code,
                    stderr,
                    raw_blob,
                },
            };
            let _ = tx.send(signal);
            self.record_death(exit_code, &identity);
            return;
        }

        // A turn that was in flight when the child died settles here, through
        // the *same* classifier: arms 9/10 read `InterruptedRunning` if we asked
        // and `AmbiguousUnknown` otherwise. Unchanged code, new call site.
        //
        // [W3 A2] made this a routine path rather than an exceptional one on
        // this transport: a denied tool exits the whole child.
        let in_flight = matches!(
            self.backend_state
                .lock()
                .expect("agy adapter state lock")
                .executions
                .get(&self.execution_id)
                .map(|execution| &execution.turn),
            Some(TurnState::InFlight(_))
        );
        if in_flight {
            self.settle_turn(
                &mut acc,
                &identity,
                std::mem::take(&mut raw),
                raw_truncated,
                first_event_at,
                init_blob.clone(),
                Some(exit_code),
            );
        }
        self.record_death(exit_code, &identity);
    }

    /// Store the child's identity and posture, and release the caller blocked
    /// on the `init` line.
    ///
    /// **The transport's real prize, and this is the moment it is cashed in.**
    /// [W3 P1 row I] proved `init` arrives even when nothing is ever written to
    /// stdin, so everything below — the conversation id, the resolved model, the
    /// effective permission mode, and therefore the pin check, the posture
    /// notice and the silent-resume-fork check — is known for **zero turns and
    /// zero quota**. Print mode can only learn any of it by spending turn 1.
    fn announce_identity(&self, identity: &LoopIdentity) {
        if let Some(execution) = self
            .backend_state
            .lock()
            .expect("agy adapter state lock")
            .executions
            .get_mut(&self.execution_id)
        {
            // Only a fresh child mints identity. A resumed child's `init` is
            // CHECKED against the id we asked for, never allowed to replace it:
            // agy warns-and-continues on an unknown `--conversation` and starts
            // a fresh conversation [W1 P0.6], so adopting whatever came back
            // would be the adapter silently following the fork.
            if execution.conversation_id.is_none() {
                execution.conversation_id = identity.conversation_id.clone();
            }
            execution.posture = Some(PermissionPosture::from_init(
                identity.permission_mode.as_deref(),
                self.settings_home.as_deref(),
            ));
        }
        if let (Some(tx), Some(id)) = (&self.init_signal, identity.conversation_id.clone()) {
            let _ = tx.send(FirstTurnSignal::Initialized {
                conversation_id: id,
                model: identity.model.clone(),
                permission_mode: identity.permission_mode.clone(),
            });
        }
    }

    /// Cut this turn at its `result` (or at the child's death), archive its raw
    /// bytes, classify it, record it and journal `conversation.turn.ended`.
    /// Returns the blob ref, so turn 1 can hand its `init`-bearing archive to
    /// every later turn's evidence.
    #[allow(clippy::too_many_arguments)]
    fn settle_turn(
        &self,
        acc: &mut TurnAccumulator,
        identity: &LoopIdentity,
        mut raw: String,
        raw_truncated: bool,
        first_event_at: Option<Instant>,
        init_blob: Option<String>,
        died_with: Option<Option<i32>>,
    ) -> Option<String> {
        if raw_truncated {
            raw.push_str(&format!(
                "\n...<{STREAM_MEMORY_CAP}-byte in-memory cap hit; further stdout lines were still \
                 parsed and emitted above but were not archived here>\n"
            ));
        }
        let (raw_blob, raw_error) = TurnReader::archive(&self.data_dir, &raw);
        let stderr = self.take_stderr(first_event_at);

        let verdict = verify_pin_from_init(self.model.as_deref(), identity.model.as_deref());
        let pin_mismatch = verdict.mismatch(self.model.as_deref());
        let pin = verdict.as_json(self.model.as_deref());
        let resume_mismatch = self.resume_mismatch(identity, &stderr.text);
        if let Some(mismatch) = &resume_mismatch {
            self.emit(
                KIND_TURN_HARNESS_ERROR,
                json!({
                    "phase": "resume_identity_mismatch",
                    "requested": self.expected_conversation,
                    "echoed": identity.conversation_id,
                    "stderr_warning": resume_fork_warning_in_stderr(&stderr.text),
                    "detail": mismatch,
                }),
            );
        }
        // The adapter composes only the accepted stdin shape
        // ([`compose_loop_message`]), so this is never a stage's fault — and it
        // is fatal to the child besides [W3 P1]. Named as an adapter defect
        // rather than reported as a failed turn, which would blame a Work for a
        // bug in this file.
        if let Some(rejection) = acc.loop_input_rejection() {
            self.emit(
                KIND_TURN_HARNESS_ERROR,
                json!({
                    "phase": "loop_input_rejected",
                    "error": rejection,
                    "detail": "agy refused a line this adapter wrote to the loop child's stdin. \
                               The adapter composes exactly one message shape and this is not a \
                               stage failure but an adapter defect; the refusal is fatal to the \
                               whole child (W3 P1), so the transport is now dead and the \
                               conversation must be resumed from a fresh one",
                }),
            );
        }

        let mut state = self.backend_state.lock().expect("agy adapter state lock");
        let Some(execution) = state.executions.get_mut(&self.execution_id) else {
            return raw_blob;
        };
        let interrupted = execution.interrupt_requested;
        let exit_code = died_with.flatten();
        let terminal = classify_terminal(acc, exit_code, interrupted, &stderr.text);
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
            stderr: stderr.text.clone(),
        }));
        drop(state);

        self.emit(
            KIND_CONVERSATION_TURN_ENDED,
            json!({
                "conversation_id": conversation_for_event,
                "transport": Transport::Loop.as_str(),
                "interrupted": interrupted,
                "outcome": terminal_outcome_label(&terminal),
                "status": acc.status(),
                "init": {
                    "model": identity.model,
                    "permission_mode": identity.permission_mode,
                    "cwd": identity.cwd,
                    "tool_count": identity.tool_count,
                },
                // The `init` line is archived exactly once, in turn 1's blob;
                // every later turn points at it rather than re-archiving it.
                "init_blob": init_blob,
                "permission_posture": posture.map(|p| p.as_json()),
                "steps": acc.steps,
                "agent_response_steps": acc.agent_response_steps,
                "text_deltas": acc.text_deltas,
                "tool_steps": acc.tool_steps,
                "denied_tools": acc.denied_tools,
                // The admission's own evidence, journaled: child conversation
                // ids recovered from a TYPED subagent_info record [W3 A1].
                "subagent_conversations": acc.subagent_conversations,
                "stderr_denial_notice": denial_evidence_in_stderr(&stderr.text),
                "stderr_attribution": stderr.attribution,
                "stderr_lines_dropped": stderr.dropped,
                // [W3 P4]: an ERROR + "timeout waiting for response" terminal is
                // ambiguous between a deadline expiry and an interrupt we sent,
                // and `status` can never disambiguate them. Carried in BOTH
                // readings so a consumer can see the classifier leant on this
                // adapter's own bit rather than on the wire.
                "terminal_ambiguity": acc
                    .terminal_is_timeout_ambiguous()
                    .then_some("timeout_or_interrupt"),
                "loop_input_rejected": acc.loop_input_rejection(),
                "unknown_events": acc.unknown_events,
                "unparsed_lines": acc.unparsed_lines,
                "saw_command_result": acc.saw_command_result,
                "bindings_outside_cwd": self.bindings_outside_cwd,
                "usage_final_step": acc.last_step_usage,
                "usage_turn": acc.terminal_usage,
                "structured_output": acc.structured_output,
                "model_pin": pin,
                "exit_code": exit_code,
                "raw": raw_blob,
                "raw_error": raw_error,
                "stderr": truncate(normalize_pty(stderr.text.trim()).as_str(), 400).to_string(),
            }),
        );
        raw_blob
    }

    /// This turn's slice of the shared stderr stream (§2.6).
    ///
    /// Every pending line is taken — **none is ever dropped**. Lines stamped at
    /// or after this turn's first stream event are inside it (`exact`); anything
    /// older arrived between turns and can only be placed *next to* one, so it
    /// is attached here and the whole slice is labelled `adjacent` so a reader
    /// can see the classifier was not certain.
    ///
    /// The short [`LOOP_STDERR_GRACE`] closes the same-instant race W1 already
    /// fixes on the other transport: the auto-denial notice is written as the
    /// `result` is flushed, and a reader that snapshots the moment the result
    /// parses is racing the thread still filling the log.
    fn take_stderr(&self, first_event_at: Option<Instant>) -> StderrSlice {
        std::thread::sleep(LOOP_STDERR_GRACE);
        let mut log = self.stderr.lock().expect("agy loop stderr lock");
        let dropped = log.dropped;
        let lines = log.take_all();
        drop(log);
        let mut adjacent = false;
        let mut text = String::new();
        for (stamped, line) in lines {
            if first_event_at.is_none_or(|start| stamped < start) {
                adjacent = true;
            }
            text.push_str(&line);
            text.push('\n');
        }
        StderrSlice {
            text,
            attribution: if adjacent { "adjacent" } else { "exact" },
            dropped,
        }
    }

    /// The silent-resume-fork guard, run against the child's **single** `init`
    /// line — so on this transport it costs zero quota and happens once, at
    /// child start, rather than being checked after a turn has already run
    /// [W3 P3].
    fn resume_mismatch(&self, identity: &LoopIdentity, stderr: &str) -> Option<String> {
        let requested = self.expected_conversation.as_deref()?;
        let echoed = identity.conversation_id.as_deref();
        if echoed == Some(requested) {
            return None;
        }
        Some(format!(
            "resume identity mismatch: this loop child asked for conversation {requested} and \
             agy's init line echoed {}. agy warns-and-continues on an unknown conversation rather \
             than refusing (measured, W1 P0.6 and re-measured on this transport in W3: a \
             plain-text stderr `warning: conversation \"…\" not found` and a fresh conversation), \
             so a fresh conversation would otherwise have been silently mistaken for a resume. \
             stderr carried that warning: {}",
            echoed.unwrap_or("<no init line at all>"),
            resume_fork_warning_in_stderr(stderr),
        ))
    }

    /// Record that the transport is gone, with everything the next SEND owes
    /// its caller — including the conversation id, which is fully resumable
    /// from a fresh child [W3 P3, W3 A2].
    fn record_death(&self, exit_code: Option<i32>, identity: &LoopIdentity) {
        let stderr_tail = self.take_stderr(None).text;
        let conversation_id = identity.conversation_id.clone().or_else(|| {
            self.backend_state
                .lock()
                .expect("agy adapter state lock")
                .executions
                .get(&self.execution_id)
                .and_then(|execution| execution.conversation_id.clone())
        });
        *self.death.lock().expect("agy loop death lock") = Some(LoopDeath {
            exit_code,
            stderr_tail: truncate(stderr_tail.trim(), 400).to_string(),
            conversation_id,
        });
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
        capabilities_for(self.transport_resolution().transport)
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
        // The only resolution that can make an otherwise-usable build
        // unavailable is `LoopOnly` against a build with no `--input-format`
        // (codex §5.2 rule 2): serving it on the other transport would be
        // serving a different set of measured claims than the one asked for.
        let resolution = self.transport_resolution();
        if !resolution.available {
            return ProbeReport {
                available: false,
                detail: Some(format!("{}; {}", outcome.detail, resolution.detail)),
            };
        }
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
        let resolution = self.transport_resolution();
        if !resolution.available {
            return Err(BackendError::Unavailable {
                backend: AGY_BACKEND_NAME.to_string(),
                detail: resolution.detail.clone(),
            });
        }
        if let Some(model) = &request.model {
            preflight_model_pin(model).map_err(|reason| self.err_failed(reason))?;
        }
        // Validated without keeping the result: LAUNCH re-resolves it, so the
        // two phases can never disagree about it.
        self.launch_config(
            request.profile.as_ref(),
            &crate::backend::causation_env(request),
        )?;
        // Refused here rather than at LAUNCH so it costs nothing and is
        // journaled before any process exists — and refused against the
        // transport this execution will ACTUALLY launch on, since the loop
        // carries its prompt on stdin and the argv cap does not bind there.
        check_prompt_budget(
            resolution.transport,
            &compose_launch_prompt(request),
            request,
        )
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
        let LaunchConfig { executable, env } = self.launch_config(
            request.profile.as_ref(),
            &crate::backend::causation_env(request),
        )?;
        let transport = self.transport_resolution().transport;
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
                    transport,
                    loop_child: None,
                },
            );
        }
        // Both transports share this function's one guarantee: a failed launch
        // leaves NO phantom execution behind, whichever way it failed.
        if transport == Transport::Loop {
            return match self.launch_loop(request) {
                Ok(handle) => Ok(handle),
                Err(e) => {
                    self.kill_loop_child(&request.execution_id);
                    self.lock().executions.remove(&request.execution_id);
                    Err(e)
                }
            };
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
        let transport = {
            let state = self.lock();
            self.check_identity(&state, handle)?;
            let execution = &state.executions[&handle.execution_id];
            if execution.stopped {
                return Err(self.err_failed(format!(
                    "execution {} is stopped; not accepting input",
                    handle.execution_id
                )));
            }
            // Kept on BOTH transports. W3 P2 measured agy serialising its own
            // queue, so this is not a liveness requirement on the loop — but
            // nothing measured a bound on that queue, and sergeant's SEND
            // contract is per-turn regardless of what the harness tolerates.
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
            execution.transport
        };
        match transport {
            Transport::Print => {
                self.spawn_turn(&handle.execution_id, input.to_string(), None, None)
            }
            Transport::Loop => {
                self.adopt_loop_child(&handle.execution_id)?;
                self.write_loop_turn(&handle.execution_id, input, None)
            }
        }
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
            // The reader lives on the execution for the print transport and on
            // the loop child for the loop one; either way INTERRUPT hands it
            // back as the completion's tail so the killed turn's archive is
            // durable before the interrupt's promise is true.
            let reader = child
                .is_some()
                .then(|| {
                    execution.reader.take().or_else(|| {
                        execution
                            .loop_child
                            .as_mut()
                            .and_then(|loop_child| loop_child.reader.take())
                    })
                })
                .flatten();
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
        let LaunchConfig { executable, env } = self.launch_config(
            request.profile.as_ref(),
            &crate::backend::resume_causation_env(request, &handle.execution_id),
        )?;
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
                transport: self.transport_resolution().transport,
                loop_child: None,
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
        // On the loop transport, closing stdin is the GRACEFUL shutdown: W3 P2
        // measured queued turns running to completion and the child then
        // exiting 0 with no further event. So STOP closes stdin, waits a bounded
        // while for the in-flight turn to settle, and only then falls through to
        // the group kill — where print mode has nothing to close and goes
        // straight there.
        if self.close_loop_stdin(&handle.execution_id) {
            let budget = self
                .config
                .stop_drain_budget
                .unwrap_or(LOOP_STOP_DRAIN_BUDGET);
            self.await_loop_settle(&handle.execution_id, budget);
        }
        self.interrupt(handle)?.wait();
        let reader = {
            let mut state = self.lock();
            self.check_identity(&state, handle)?;
            let execution = state
                .executions
                .get_mut(&handle.execution_id)
                .expect("presence checked above");
            execution.stopped = true;
            execution.reader.take().or_else(|| {
                execution
                    .loop_child
                    .as_mut()
                    .and_then(|loop_child| loop_child.reader.take())
            })
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
            estate_root: None,
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

    // ------------------------------------------------- W3: the loop transport

    const LOOP_TWO_TURNS: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-loop-two-turns.jsonl");
    const LOOP_INIT_ONLY: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-loop-init-only-empty-stdin.jsonl");
    const LOOP_RESUME_INIT_ECHO: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-loop-resume-init-echo.jsonl");
    const LOOP_CONTROL_REQUEST: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-loop-control-request-refusal.jsonl");
    const LOOP_MISSING_EVENT: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-loop-missing-event-field.jsonl");
    const LOOP_BAD_BLOCK: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-loop-unsupported-block-type.jsonl");
    const LOOP_UNKNOWN_EVENT_WARNING: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-loop-unknown-event-warning.txt");
    const LOOP_SUBAGENT: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-loop-subagent-info.jsonl");
    const LOOP_SCHEMA: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-loop-schema-two-turns.jsonl");
    const LOOP_DENIED_TOOL: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-loop-denied-tool-kills-child.jsonl");
    const LOOP_SIGINT_CANCELLED: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-loop-sigint-cancelled-terminal.jsonl");
    const LOOP_SANDBOX_UNAVAILABLE: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-loop-sandbox-unavailable.jsonl");
    const LOOP_RESUME_FORK_WARNING: &str =
        include_str!("../../tests/fixtures/agy-1.1.19-loop-resume-fork-warning.txt");

    /// Replay one loop child's whole stream the way [`LoopReader`] does — a
    /// fresh [`TurnAccumulator`] cut at every `result`, reseeded from the
    /// child's single `init` — and return one entry per settled turn.
    ///
    /// This is the reader's turn loop with the I/O taken out, which is what
    /// makes every loop behaviour drivable with no `agy` binary and no quota.
    fn replay_loop(fixture: &str) -> Vec<(TurnAccumulator, Vec<NativeEvent>)> {
        let mut identity = LoopIdentity::default();
        let mut acc = TurnAccumulator::new();
        let mut events = Vec::new();
        let mut turns = Vec::new();
        for line in fixture.lines().filter(|line| !line.trim().is_empty()) {
            let value: Value = serde_json::from_str(line).expect("fixture line parses");
            let event = value.get("event").and_then(Value::as_str).unwrap_or("");
            events.extend(acc.ingest_line(&value));
            if event == "init" {
                identity = LoopIdentity::from_accumulator(&acc);
            }
            if event == "result" {
                turns.push((std::mem::take(&mut acc), std::mem::take(&mut events)));
                identity.reseed(&mut acc);
            }
        }
        // Anything left over is an unsettled turn — a child that died mid-turn.
        if acc.steps > 0 || !matches!(acc.terminal, Terminal::None) {
            turns.push((acc, events));
        }
        turns
    }

    #[test]
    fn loop_argv_carries_the_measured_launch_grammar() {
        let argv = loop_argv(Some("gemini-3.7-flash-low"), None, None);
        // `--print=` with the `=` and an empty value: a bare `-p` swallows the
        // next flag and fails rc=2 with plain-text stderr and NO NDJSON at all
        // (W3 P0) — a shape the stream decoder can never see.
        assert_eq!(argv[0], "--print=");
        assert!(!argv.iter().any(|arg| arg == "-p"));
        // `--input-format stream-json` REQUIRES `--output-format stream-json`;
        // composed together or not at all.
        let input_at = argv
            .iter()
            .position(|a| a == "--input-format")
            .expect("flag");
        assert_eq!(argv[input_at + 1], "stream-json");
        let output_at = argv
            .iter()
            .position(|a| a == "--output-format")
            .expect("flag");
        assert_eq!(argv[output_at + 1], "stream-json");
        assert!(argv.contains(&"--disable-slash-commands".to_string()));
        // No prompt anywhere on argv: that is the transport's whole point.
        assert!(!argv.iter().any(|arg| arg.contains("do the")));
        // Never composed by default on either transport (W3 S1 measured
        // --sandbox breaking every run_command on this host).
        assert!(!argv.iter().any(|arg| arg == "--sandbox"));
        assert!(!argv.iter().any(|arg| arg == "--add-dir"));
        assert!(!argv.iter().any(|arg| arg == "--agent"));
        assert!(
            !argv
                .iter()
                .any(|arg| arg == "--dangerously-skip-permissions")
        );
        assert!(!argv.iter().any(|a| a == "--json-schema"));
        assert!(!argv.iter().any(|a| a == "--conversation"));

        let resumed = loop_argv(None, Some("{}"), Some("conv-1"));
        assert!(!resumed.iter().any(|a| a == "--model"));
        let at = resumed
            .iter()
            .position(|a| a == "--conversation")
            .expect("--conversation");
        assert_eq!(resumed[at + 1], "conv-1");
        let schema_at = resumed
            .iter()
            .position(|a| a == "--json-schema")
            .expect("--json-schema");
        assert_eq!(resumed[schema_at + 1], "{}");
    }

    #[test]
    fn the_loop_gate_flag_is_what_the_loop_grammar_composes() {
        // The membership rule `required_flags_are_exactly_what_the_launch_grammar
        // _composes` enforces for print, enforced for the loop's own gate — in
        // both directions, which is what makes LOOP_GATE_FLAGS a gate and not a
        // wish. See LOOP_GATE_FLAGS' own doc for why it is not folded into
        // REQUIRED_FLAGS.
        let argv = loop_argv(Some("m"), Some("{}"), Some("c"));
        for flag in LOOP_GATE_FLAGS {
            assert!(
                argv.iter().any(|arg| arg == flag),
                "LOOP_GATE_FLAGS names {flag}, which the loop grammar never composes"
            );
        }
        // And every flag the loop composes is gated by one list or the other.
        for arg in &argv {
            if !arg.starts_with("--") {
                continue;
            }
            let long = arg.split('=').next().unwrap_or(arg);
            assert!(
                REQUIRED_FLAGS.contains(&long) || LOOP_GATE_FLAGS.contains(&long),
                "the loop grammar composes {long}, which neither gate list requires — a flag this \
                 adapter composes but never checks for is a grammar it never measured"
            );
        }
    }

    #[test]
    fn the_adapter_composes_exactly_one_accepted_stdin_shape() {
        // W3 P1: five of the six typed refusals are shape refusals, and every
        // one of them is FATAL TO THE WHOLE CHILD. So the composed line is
        // round-tripped against each refusal's own condition, rather than
        // trusted to be right because it was written carefully.
        let line = compose_loop_message("say pong");
        assert!(!line.contains('\n'), "one NDJSON object, one line");
        let value: Value = serde_json::from_str(&line).expect("composed line parses");
        // Row C: a line with no `event` key.
        assert_eq!(value.get("event").and_then(Value::as_str), Some("user"));
        // Row D: a `user` message with no `message` field.
        let message = value.get("message").expect("a message field");
        // Row E: a `message` with no content.
        let content = message.get("content").expect("a content field");
        // Row F: a non-text block. The string form carries no blocks at all,
        // which is why it is the shape composed.
        assert!(content.is_string(), "the string form, not the block list");
        assert_eq!(content.as_str(), Some("say pong"));
        assert_eq!(message.get("role").and_then(Value::as_str), Some("user"));
        // Row H: `control_request` is the only non-`user` event the decoder
        // recognises at all, and it is refused.
        assert_ne!(
            value.get("event").and_then(Value::as_str),
            Some("control_request")
        );

        // The injection hazard the typed struct exists for: a prompt carrying a
        // newline and a quote must not become two lines or escape the string.
        let nasty = compose_loop_message("line one\n{\"event\":\"control_request\"}\n\" end");
        assert!(
            !nasty.contains('\n'),
            "a newline in the prompt stays escaped"
        );
        let parsed: Value = serde_json::from_str(&nasty).expect("still one object");
        assert_eq!(parsed["event"], json!("user"));
        assert!(
            parsed["message"]["content"]
                .as_str()
                .expect("content")
                .contains("control_request"),
            "the injected text is DATA, carried verbatim inside content"
        );
    }

    #[test]
    fn a_control_request_message_is_refused_as_unsupported() {
        // Backs the loop's `ask` and `approval_flow` rows: this is the ONLY
        // non-`user` event the decoder recognises, and upstream refuses it in
        // its own words — including the "yet".
        let turns = replay_loop(LOOP_CONTROL_REQUEST);
        assert_eq!(turns.len(), 1);
        let (acc, _) = &turns[0];
        let Terminal::Status { status, error, .. } = &acc.terminal else {
            panic!("a typed terminal")
        };
        assert_eq!(status, "ERROR");
        assert_eq!(
            error,
            "stream input message event \"control_request\" is not supported yet"
        );
        // Zero-quota: the refusal costs nothing, which is why `ask` could be
        // refuted without spending a turn on it.
        assert_eq!(
            acc.terminal_usage.as_ref().expect("usage")["total_tokens"],
            json!(0)
        );
        // And it is an ADAPTER-shape refusal, so it is reported as an adapter
        // defect rather than as a stage failure.
        assert_eq!(acc.loop_input_rejection(), Some(error.as_str()));
    }

    #[test]
    fn a_malformed_input_line_is_reported_as_an_adapter_defect_not_a_stage_failure() {
        for (fixture, expected) in [
            (
                LOOP_MISSING_EVENT,
                "stream input message is missing the \"event\" field",
            ),
            (
                LOOP_BAD_BLOCK,
                "stream input content block type \"image\" is not supported (only \"text\")",
            ),
        ] {
            let turns = replay_loop(fixture);
            let (acc, _) = &turns[0];
            assert_eq!(acc.loop_input_rejection(), Some(expected));
            // The classifier still says Failed — the turn did not happen — but
            // the `loop_input_rejected` phase is what tells a reader the fault
            // is in this file and not in the stage.
            assert_eq!(
                classify_terminal(acc, Some(1), false, ""),
                TerminalOutcome::Failed {
                    reason: expected.to_string()
                }
            );
        }
        // An ordinary harness failure must NOT be mistaken for one.
        let turns = replay_loop(LOOP_DENIED_TOOL);
        assert_eq!(turns[0].0.loop_input_rejection(), None);
    }

    #[test]
    fn an_unknown_stream_input_event_is_a_warning_the_child_survives() {
        // W3 P1 row G, the one refusal shape that is NOT fatal: the line is
        // skipped, the child survives, and there is no `result` at all — which
        // is why the sixteen candidate reply-event names taught us nothing
        // except that none of them exists.
        assert!(
            LOOP_UNKNOWN_EVENT_WARNING.contains("ignoring unsupported stream input message event"),
            "the captured warning names the skipped event"
        );
        let turns = replay_loop(LOOP_INIT_ONLY);
        assert!(
            turns.is_empty(),
            "an init-only child settles no turn at all"
        );
    }

    #[test]
    fn init_arrives_at_child_start_before_any_message_is_consumed() {
        // W3 P1 row I, the transport's whole prize: stdin was closed with
        // NOTHING written and the child still emitted a full identity line.
        let value: Value =
            serde_json::from_str(LOOP_INIT_ONLY.lines().next().expect("a line")).expect("parses");
        let mut acc = TurnAccumulator::new();
        assert!(acc.ingest_line(&value).is_empty(), "init emits no event");
        let identity = LoopIdentity::from_accumulator(&acc);
        assert!(identity.conversation_id.is_some());
        assert_eq!(identity.model.as_deref(), Some("gemini-3.7-flash-low"));
        assert_eq!(identity.permission_mode.as_deref(), Some("request-review"));
        assert_eq!(identity.tool_count, 57);
        // So the pin verdict — and therefore the LAUNCH refusal — is decidable
        // here, with zero quota spent.
        assert_eq!(
            verify_pin_from_init(Some("gemini-3.7-pro"), identity.model.as_deref()),
            PinVerdict::Substituted("gemini-3.7-flash-low".to_string())
        );
    }

    #[test]
    fn a_loop_turn_boundary_resets_the_accumulator_but_not_the_conversation() {
        let turns = replay_loop(LOOP_TWO_TURNS);
        assert_eq!(turns.len(), 2, "one accumulator per turn, cut at `result`");
        let (first, _) = &turns[0];
        let (second, _) = &turns[1];
        // The conversation survives the cut — one child, one identity, minted
        // on the single `init` line and never re-minted.
        assert!(first.conversation_id.is_some());
        assert_eq!(first.conversation_id, second.conversation_id);
        assert_eq!(first.init_model, second.init_model);
        assert_eq!(first.init_permission_mode, second.init_permission_mode);
        // The per-turn evidence does NOT survive it: turn 2's summary is its
        // own, and its step counts start from this turn rather than the child.
        assert_eq!(first.last_response.as_deref(), Some("alpha\n"));
        assert_eq!(second.last_response.as_deref(), Some("bravo\n"));
        assert!(second.steps < first.steps || second.steps > 0);
        assert_eq!(second.tool_steps, 0);
        // Both are ordinary completions.
        for (acc, _) in &turns {
            assert_eq!(
                classify_terminal(acc, Some(0), false, ""),
                TerminalOutcome::Completed
            );
        }
    }

    #[test]
    fn a_conversation_scoped_counter_is_never_assumed_to_start_at_zero() {
        // W1 delta 8, now also across turn boundaries inside one child: W3 P2
        // saw step_index 0,1,2 then 3,4, and W3 P3's RESUMED child opened at
        // step_index 5 with num_turns 3 and a cumulative duration_seconds of
        // 133.16. Nothing in this decoder keys on any of them starting at zero.
        let turns = replay_loop(LOOP_TWO_TURNS);
        let first_indices = step_indices(LOOP_TWO_TURNS);
        assert!(
            first_indices.windows(2).all(|w| w[1] >= w[0]),
            "step_index is monotone across the whole child, not per turn: {first_indices:?}"
        );
        assert!(
            first_indices.iter().any(|index| *index > 2),
            "turn 2's steps continue the child's numbering: {first_indices:?}"
        );
        // num_turns is a conversation counter, carried verbatim into evidence
        // and never read as an index of this child's turns.
        let terminal_turns: Vec<u64> = turns
            .iter()
            .filter_map(|(acc, _)| {
                acc.terminal_usage.as_ref()?;
                None
            })
            .collect();
        assert!(terminal_turns.is_empty(), "nothing derives a turn index");
        // The resumed capture proves the hazard is real rather than theoretical.
        let resumed = step_indices(LOOP_RESUME_INIT_ECHO);
        assert!(
            resumed.is_empty(),
            "the resume-echo capture is init only — zero turns spent to check identity"
        );
    }

    /// Every `step_index` in a capture, in wire order.
    fn step_indices(fixture: &str) -> Vec<i64> {
        fixture
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter_map(|value| {
                value
                    .pointer("/step_update/step_index")
                    .and_then(Value::as_i64)
            })
            .collect()
    }

    #[test]
    fn a_subagent_info_payload_becomes_a_typed_tool_event() {
        // **The `native_subagents` admission, deterministically.** The live test
        // earns the row; this keeps it honest in CI with no quota, over the
        // fixture cut from that very run.
        let turns = replay_loop(LOOP_SUBAGENT);
        assert_eq!(turns.len(), 2, "define on turn 1, invoke on turn 2");
        let parent = turns[0].0.conversation_id.clone().expect("a parent id");
        let (invoke, events) = &turns[1];

        // (1) the step exists and is a subagent step, not merely a tool whose
        // NAME looks like one.
        let completed: Vec<_> = events
            .iter()
            .filter(|event| event.kind == KIND_TOOL_COMPLETED)
            .filter(|event| event.payload["name"] == json!("subagent:subagent"))
            .collect();
        assert_eq!(completed.len(), 1, "exactly one settled subagent record");
        let children = completed[0].payload["subagent"]
            .as_array()
            .expect("the typed child list");
        assert_eq!(children.len(), 1);

        // (2) a TYPED child conversation_id, distinct from the parent's.
        let child_id = children[0]["conversation_id"]
            .as_str()
            .expect("a child conversation_id — the whole admission");
        assert!(!child_id.is_empty());
        assert_ne!(
            child_id, parent,
            "a subagent that shares the parent's conversation is not a subagent"
        );
        assert_eq!(children[0]["name"], json!("echoer"));
        assert!(
            children[0]["log_uri"]
                .as_str()
                .expect("a log_uri")
                .ends_with("transcript.jsonl"),
            "the child's own trajectory log, carried verbatim so a human can read it"
        );
        // (3) a settled outcome for it.
        assert_eq!(completed[0].payload["state"], json!("DONE"));
        assert_eq!(completed[0].payload["is_error"], json!(false));
        assert_eq!(invoke.subagent_conversations, vec![child_id.to_string()]);

        // The ACTIVE step carries no child id at all — which is exactly why the
        // admission demands a *settled* record and not an in-flight one.
        let requested: Vec<_> = events
            .iter()
            .filter(|event| event.kind == KIND_TOOL_REQUESTED)
            .filter(|event| event.payload["name"] == json!("subagent:subagent"))
            .collect();
        assert_eq!(requested.len(), 1, "one request per subagent step, not two");

        // And the negative that keeps the row honest: turn 1 DEFINED a subagent
        // through an ordinary tool step, and that is not an invocation.
        assert!(
            turns[0].0.subagent_conversations.is_empty(),
            "define_subagent mints no child conversation; only invoking one does"
        );
    }

    #[test]
    fn narration_about_delegating_is_never_a_subagent_record() {
        // The explicit non-evidence list, made structural: assistant text
        // saying it delegated must produce no subagent record at all.
        let mut acc = TurnAccumulator::new();
        let events = acc.ingest_line(&json!({
            "event": "step_update",
            "step_update": {
                "step_index": 1,
                "state": "DONE",
                "step_type": "agent_response",
                "text_delta": "I have invoked the `echoer` subagent with conversation_id abc-123.",
            }
        }));
        assert!(acc.subagent_conversations.is_empty());
        assert!(
            events
                .iter()
                .all(|event| event.kind != KIND_TOOL_COMPLETED
                    && event.kind != KIND_TOOL_REQUESTED),
            "prose is never tool evidence"
        );
        // And a subagent_info with no child conversation_id is not one either.
        let mut acc = TurnAccumulator::new();
        acc.ingest_line(&json!({
            "event": "step_update",
            "step_update": {
                "step_index": 7,
                "state": "DONE",
                "step_type": "subagent",
                "tool_name": "invoke_subagent",
                "subagent_info": {"subagents": [{"type_name": "echoer", "role": "Word Echoer"}]},
            }
        }));
        assert!(
            acc.subagent_conversations.is_empty(),
            "a subagent_info with no child conversation_id admits nothing"
        );
    }

    #[test]
    fn the_loop_schema_fixture_carries_structured_output_on_every_turn() {
        // W3 A3 closed the open question the good way: `--help`'s "only
        // applicable to the final result" means the final result of each TURN,
        // not only of the child. So a None on an intermediate turn would be an
        // anomaly, not an expectation, and the tier stays NativeSchemaFlag.
        let turns = replay_loop(LOOP_SCHEMA);
        assert_eq!(turns.len(), 2);
        let first = turns[0]
            .0
            .structured_output
            .clone()
            .expect("turn 1's schema output");
        let second = turns[1]
            .0
            .structured_output
            .clone()
            .expect("turn 2's schema output");
        assert_eq!(first, json!({"word": "alpha", "n": 1}));
        assert_eq!(second, json!({"word": "bravo", "n": 2}));
        // Beside the prose response, never instead of it.
        assert!(
            turns[0]
                .0
                .last_response
                .as_deref()
                .expect("a response")
                .contains("alpha")
        );
    }

    #[test]
    fn a_denied_tool_on_the_loop_is_a_typed_failure_and_the_last_turn_of_its_child() {
        // **The measurement that inverts W1's, and the reason W1 was right to
        // keep the packet's typed detector.** On the loop at 1.1.19 a denied
        // `command` tool resolves ACTIVE->ERROR with the packet's own 1.1.17
        // typed shape, the terminal is ERROR with the same string, stderr is
        // EMPTY, and the child exits 1 — where print mode gives
        // DONE/CANCELED/exit 0/stderr-only.
        let turns = replay_loop(LOOP_DENIED_TOOL);
        assert_eq!(turns.len(), 1);
        let (acc, _) = &turns[0];
        assert_eq!(acc.denied_tools, vec!["run_command".to_string()]);
        assert!(
            !denial_evidence_in_stderr(""),
            "no stderr notice at all on this transport — the typed detector is the one that fires"
        );
        let TerminalOutcome::Failed { reason } = classify_terminal(acc, Some(1), false, "") else {
            panic!("an explicit, typed failure")
        };
        assert!(
            reason.contains("user denied permission to run command"),
            "{reason}"
        );
        // Not the empty-SUCCESS class and not an ambiguity: the harness said
        // exactly what went wrong, which is the ONLY route to Failed.
        assert_eq!(acc.status(), Some("ERROR"));
    }

    #[test]
    fn a_timeout_terminal_is_an_interrupt_when_we_asked_and_a_failure_when_we_did_not() {
        // **Both readings of the IDENTICAL bytes.** [W3 P4] measured a SIGINT
        // landing mid-turn producing `status: ERROR` with
        // `error: "timeout waiting for response"` — the very string a
        // `--print-timeout` expiry produces [W1 P5], which is why the print
        // transport's own captured timeout fixture is the right bytes to drive
        // this with: the whole finding is that the two are indistinguishable on
        // the wire. `status` can never tell them apart, so only this adapter's
        // own bit can — which is precisely why `classify_terminal` takes it.
        let (acc, _) = replay(PRINT_TIMEOUT);
        assert_eq!(acc.status(), Some("ERROR"));
        assert!(acc.terminal_is_timeout_ambiguous());
        assert!(acc.terminal_is_signal_shaped());
        assert_eq!(
            classify_terminal(&acc, Some(1), true, ""),
            TerminalOutcome::InterruptedRunning,
            "we asked for the kill: a stage we interrupted is not a stage that failed, and W3 A7 \
             measured the conversation staying fully resumable afterwards"
        );
        let TerminalOutcome::Failed { reason } = classify_terminal(&acc, Some(1), false, "") else {
            panic!("nobody asked: the harness's own statement stands")
        };
        assert_eq!(reason, "timeout waiting for response");

        // The amendment is narrow on purpose: a DIFFERENT ERROR string is still
        // a failure even when we did ask, because this adapter may not claim
        // authorship of an error it did not cause.
        let denied = replay_loop(LOOP_DENIED_TOOL);
        assert!(!denied[0].0.terminal_is_signal_shaped());
        assert!(matches!(
            classify_terminal(&denied[0].0, Some(1), true, ""),
            TerminalOutcome::Failed { .. }
        ));
    }

    #[test]
    fn an_idle_child_sigint_is_an_interrupt_and_never_an_adapter_defect() {
        // **[W3 A7] corrects the spec's own expectation, and the correction was
        // load-bearing.** The spec (on W3 P4) expected one SIGINT terminal;
        // re-measuring found two, split by where the signal lands. An idle
        // child — blocked reading stdin — answers
        // `stream input cancelled: context canceled`, which shares its prefix
        // with the five typed *malformed-message* refusals and means the
        // opposite. A prefix-only classifier reported a correct interrupt as an
        // adapter defect; this test is why it does not.
        let turns = replay_loop(LOOP_SIGINT_CANCELLED);
        let (acc, _) = turns.last().expect("the cancelled terminal");
        assert_eq!(acc.status(), Some("ERROR"));
        assert_eq!(
            acc.loop_input_rejection(),
            None,
            "a cancellation is not a malformed message: nothing this adapter composed was wrong"
        );
        assert!(acc.terminal_is_signal_shaped());
        assert!(
            !acc.terminal_is_timeout_ambiguous(),
            "this shape, unlike the timeout one, collides with no deadline expiry"
        );
        assert_eq!(
            classify_terminal(acc, Some(1), true, ""),
            TerminalOutcome::InterruptedRunning
        );
        // And the five real refusals are still caught.
        for fixture in [LOOP_MISSING_EVENT, LOOP_BAD_BLOCK, LOOP_CONTROL_REQUEST] {
            assert!(replay_loop(fixture)[0].0.loop_input_rejection().is_some());
        }
    }

    #[test]
    fn the_sandbox_probe_terminal_is_a_mechanism_failure_not_a_permission_denial() {
        // W3 S1, and the distinction is the whole finding: `proceed-in-sandbox`
        // with NO allow-rule did not auto-deny — the permission gate lifted —
        // and the tool then failed at the sandbox MECHANISM. A classifier that
        // read this as a denial would have reported a working second permission
        // channel as a broken one.
        let turns = replay_loop(LOOP_SANDBOX_UNAVAILABLE);
        let (acc, _) = &turns[0];
        assert!(
            acc.denied_tools.is_empty(),
            "no permission denial anywhere: the gate lifted"
        );
        let TerminalOutcome::Failed { reason } = classify_terminal(acc, Some(1), false, "") else {
            panic!("a typed failure")
        };
        assert!(reason.contains("connecting to sandbox server"), "{reason}");
        // And the retry that resolved DONE with no output must not be mistaken
        // for a completion: the terminal outranks it.
        assert!(acc.tool_steps >= 2);
    }

    #[test]
    fn the_resume_fork_is_detectable_at_child_start_for_zero_quota() {
        // Both independent detectors, on the loop, before a turn is spent.
        let echoed: Value =
            serde_json::from_str(LOOP_RESUME_INIT_ECHO.lines().next().expect("a line"))
                .expect("parses");
        let mut acc = TurnAccumulator::new();
        acc.ingest_line(&echoed);
        let identity = LoopIdentity::from_accumulator(&acc);
        assert_eq!(
            identity.conversation_id.as_deref(),
            Some("b3be71a6-fd10-4525-875a-e7789a9811c3"),
            "the requested id echoed back at child start"
        );
        // Detector two, independent of the echo.
        assert!(resume_fork_warning_in_stderr(LOOP_RESUME_FORK_WARNING));
        assert!(!resume_fork_warning_in_stderr(""));
    }

    #[test]
    fn a_prompt_over_the_loop_cap_is_refused_not_truncated() {
        // The capability delta, and its honest bound: a prompt print mode
        // refuses at PREPARE rides the loop fine, and one over the loop's own
        // cap is still refused rather than silently trimmed.
        let request = prompt_request(Vec::new());
        let big = "x".repeat(ARGV_PROMPT_CAP + 1);
        assert!(
            check_prompt_budget(Transport::Print, &big, &request).is_err(),
            "print refuses it"
        );
        assert!(
            check_prompt_budget(Transport::Loop, &big, &request).is_ok(),
            "the loop carries it: the prompt leaves argv entirely"
        );
        let enormous = "x".repeat(LOOP_PROMPT_CAP + 1);
        let error = check_prompt_budget(Transport::Loop, &enormous, &request)
            .expect_err("over the loop cap");
        assert!(error.contains("Nothing is truncated"), "{error}");
        assert!(
            error.contains("no measured limit"),
            "the refusal says why a cap exists at all when nothing measured one: {error}"
        );
    }

    #[test]
    fn the_stderr_log_is_bounded_and_says_what_it_dropped() {
        let mut log = StderrLog::default();
        log.push("first".to_string());
        assert_eq!(log.dropped, 0);
        log.push("x".repeat(STREAM_MEMORY_CAP));
        // The older line is evicted and COUNTED — a dropped auto-denial notice
        // that nobody counted is the hazard this whole structure exists for —
        // while the newest line survives rather than evicting itself.
        assert_eq!(log.dropped, 1);
        let taken = log.take_all();
        assert_eq!(taken.len(), 1);
        assert!(taken[0].1.starts_with('x'));
        assert!(log.take_all().is_empty(), "draining is a move, not a copy");
        // A single over-cap line is truncated with its loss marked, never
        // dropped outright.
        let mut log = StderrLog::default();
        log.push("y".repeat(STREAM_MEMORY_CAP + 10));
        let taken = log.take_all();
        assert_eq!(taken.len(), 1);
        assert!(taken[0].1.contains("stderr line cap hit"));
    }

    // ------------------------------------------------------- admission rows

    /// The structural check that keeps the ledger honest.
    #[test]
    fn admission_rows_agree_with_capabilities() {
        // Now driven over EVERY transport, so a row added to one and forgotten
        // on the other fails the build rather than a review.
        for transport in [Transport::Print, Transport::Loop] {
            let capabilities = capabilities_for(transport);
            for (name, claimed) in v1_flags(&capabilities) {
                let rows: Vec<_> = ADMISSION_ROWS
                    .iter()
                    .filter(|row| row.capability == name && row.transport == transport)
                    .collect();
                assert_eq!(
                    rows.len(),
                    1,
                    "{name} on {}: exactly one row per v1 flag per transport",
                    transport.as_str()
                );
                assert_eq!(
                    rows[0].claimed,
                    claimed,
                    "{name} on {}: the ledger and capabilities_for() must agree",
                    transport.as_str()
                );
            }
        }

        // Adapter-local rows, transport-tagged. The four W1 rows are duplicated
        // for the loop (they still hold, with their own measured notes) and W3
        // adds four of its own.
        let adapter_local: &[(&str, Transport)] = &[
            ("config_injection", Transport::Print),
            ("permission_mode_reported_at_launch", Transport::Print),
            ("non_blocking_run", Transport::Print),
            ("structured_output", Transport::Print),
            ("config_injection", Transport::Loop),
            ("permission_mode_reported_at_launch", Transport::Loop),
            ("non_blocking_run", Transport::Loop),
            ("structured_output", Transport::Loop),
            ("turn_serialization", Transport::Loop),
            ("identity_before_first_turn", Transport::Loop),
            ("prompt_channel", Transport::Loop),
            ("sandbox", Transport::Loop),
        ];
        for (name, transport) in adapter_local {
            assert_eq!(
                ADMISSION_ROWS
                    .iter()
                    .filter(|row| row.capability == *name && row.transport == *transport)
                    .count(),
                1,
                "{name} on {}: exactly one adapter-local row",
                transport.as_str()
            );
        }

        for row in ADMISSION_ROWS {
            if row.claimed {
                assert!(
                    !row.admission_test.is_empty(),
                    "{} on {}: a claimed capability with no admission test is a claim nothing \
                     checks (L8)",
                    row.capability,
                    row.transport.as_str()
                );
            } else {
                assert!(
                    row.admission_test.is_empty(),
                    "{} on {}: an unclaimed capability names no test; the reason lives in `note`",
                    row.capability,
                    row.transport.as_str()
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

        let v1_len = v1_flags(&capabilities_for(Transport::Print)).len();
        assert_eq!(
            ADMISSION_ROWS.len(),
            2 * v1_len + adapter_local.len(),
            "every row is one of the thirteen v1 flags on one of the two transports, or one of \
             the transport-tagged adapter-local rows — an unlisted row is a claim nothing checks"
        );
    }

    /// The v1 contract's flags, paired with what a capability set claims for
    /// each. One list, used by every structural check, so a flag added to
    /// `Capabilities` cannot be checked in one test and forgotten in another.
    fn v1_flags(capabilities: &Capabilities) -> Vec<(&'static str, bool)> {
        vec![
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
        ]
    }

    /// **New in W3.** `capabilities_for` is exhaustively matched over
    /// `Transport` and every flag is set explicitly in each arm — no
    /// `..Default::default()`, no struct-update from the other transport — so a
    /// future transport cannot inherit a claim it never measured. Checked as
    /// source text because that is the only place the *absence* of a spread is
    /// visible.
    #[test]
    fn both_transports_answer_every_v1_flag() {
        let body_start = THIS_MODULE_SOURCE
            .find("fn capabilities_for(transport: Transport) -> Capabilities {")
            .expect("capabilities_for");
        let body_end = body_start
            + THIS_MODULE_SOURCE[body_start..]
                .find("\n}\n")
                .expect("its closing brace");
        let body = &THIS_MODULE_SOURCE[body_start..body_end];
        assert!(
            !body.contains("..Default::default()") && !body.contains("..capabilities"),
            "capabilities_for must set every flag explicitly in every arm: a spread lets a new \
             transport inherit a claim nobody measured for it"
        );
        for transport in [Transport::Print, Transport::Loop] {
            let capabilities = capabilities_for(transport);
            assert_eq!(
                v1_flags(&capabilities).len(),
                13,
                "{}: every v1 flag is answered",
                transport.as_str()
            );
        }
        // And the two columns are genuinely different, on exactly the one
        // boolean W3's evidence moved.
        let print = capabilities_for(Transport::Print);
        let loop_ = capabilities_for(Transport::Loop);
        let moved: Vec<_> = v1_flags(&print)
            .into_iter()
            .zip(v1_flags(&loop_))
            .filter(|((_, p), (_, l))| p != l)
            .map(|((name, _), _)| name)
            .collect();
        assert_eq!(
            moved,
            vec!["native_subagents"],
            "exactly one v1 boolean may differ between the transports, and only on W3 A1's typed \
             subagent record — a wave that flipped booleans to look productive is the defect this \
             ledger exists to prevent"
        );
    }

    /// This module's own source, and the integration suite's — the only two
    /// places an `admission_test` name may resolve. Reading them as text is how
    /// [`every_admission_test_name_resolves_to_a_real_test`] turns "the ledger
    /// cites a real test" from a claim in a commit message into something the
    /// build enforces (the same trick `tests/c2_light/coverage_stage_membership.rs`
    /// uses on suite wiring). `include_str!` of this very file is deliberate.
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
        // And the seven rows that ARE live-measured are backed by the four
        // `live_agy_*` tests the live tier actually drives — SEVEN turns total
        // (1+2+2+2, per this wave's `live-tier-run.txt`): two rows on print, and
        // on the loop one test covering resume/model_selection/
        // identity_before_first_turn and one covering native_subagents/
        // turn_serialization. This comment states a count the assertion below
        // enforces; the two move in the same commit.
        let live: Vec<_> = ADMISSION_ROWS
            .iter()
            .filter(|row| row.evidence == Evidence::LiveMeasured)
            .map(|row| row.capability)
            .collect();
        assert_eq!(
            live,
            vec![
                // print
                "resume",
                "model_selection",
                // loop — five rows, two live tests, and the sharing is
                // deliberate: one turn that resumes a conversation also proves
                // the pin echo and the pre-turn identity window.
                "resume",
                "model_selection",
                "native_subagents",
                "turn_serialization",
                "identity_before_first_turn",
            ],
            "the LiveMeasured set must be exactly the rows this wave's live tier actually drove; \
             it is updated in the same commit as any row that gains or loses a live test"
        );
    }

    #[test]
    fn the_rendered_table_states_the_stability_fact_once() {
        let rendered = render_admission_rows();
        assert_eq!(rendered.matches("stability (all rows)").count(), 1);
        assert!(rendered.contains("MEASURED_FLOOR 1.1.17 is provenance, not a gate (R1)"));
        assert!(rendered.contains("capability | transport | claimed | tier | evidence"));
        assert!(rendered.contains("history | print-stream-json | false"));
        assert!(rendered.contains("config_injection | print-stream-json | true"));
        // A transport that silently stopped being rendered fails here rather
        // than quietly shrinking the ledger a reader is shown.
        assert!(
            rendered
                .lines()
                .filter(|line| line.contains("| input-loop-stream-json |"))
                .count()
                >= 13,
            "every v1 flag owes the loop transport a rendered row"
        );
        assert!(rendered.contains("native_subagents | input-loop-stream-json | true"));
        assert!(rendered.contains("ask | input-loop-stream-json | false"));
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
        // Deliberately compared against the RESOLVED transport rather than
        // hardcoded to print: on a host whose installed agy offers
        // --input-format, `Auto` resolves to the loop and this backend honestly
        // claims the loop's set. Hardcoding print here would have made the test
        // pass by asserting the adapter lies about itself.
        assert_eq!(
            backend.capabilities(),
            capabilities_for(backend.transport_resolution().transport)
        );
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
