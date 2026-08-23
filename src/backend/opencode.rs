//! OpenCode adapter: `opencode run --format json` non-interactive turns over
//! a durable, **server-minted** session (W1 of the *Sergeant speaks OpenCode*
//! sprint, `docs/proposals/opencode-adapter-2026-08-23.md`).
//!
//! Every claim below carries its provenance. **[measured]** means opencode
//! **1.18.19** on Cerberus, 2026-08-23 — the probe packet
//! (`sergeant-rs-workspace`:
//! `knowledge/evidence/opencode-adapter-probes-2026-08-23.md`, probes 1–10)
//! plus the six fixtures committed beside this module's own suite.
//! [`MEASURED_FLOOR`] records that version as **provenance, not a gate**
//! (ADR 0020's R1, now the repo-wide stance): a build below it is still
//! `available`, with an unmeasured-provenance detail, never refused. What
//! *is* refused is a CLI whose version cannot be parsed at all, or whose
//! `run --help` does not offer this adapter's launch grammar (the A2 split,
//! carried verbatim from the codex sprint).
//!
//! **Ponytail rungs this module cites.** R1 (build the smallest thing the
//! evidence supports) is the reason there is no `Stability` column in the
//! admission ledger and no permission-policy synthesis here; it is also why
//! this module shipped `interrupt` on a plain `Child::kill()` at first —
//! probe 10 measured no `opencode` process surviving it. Probe 11 (recorded
//! 2026-08-23, post-W1-implementation, on the same evidence page) closed
//! that measurement's gap: a bash-tool grandchild reparented to init and
//! **kept running** after the harness was killed, so `interrupt` now mirrors
//! `codex.rs`'s process-group termination (`kill_process_group`/`kill_turn`,
//! §5.5) — R1 built the smaller thing until the evidence stopped supporting
//! it, then this wave built the next rung up. R2 (reuse a shipped shape
//! rather than re-derive it) is why the ledger, the `TurnReader`, the
//! fail-closed terminal, the live-test gate, and now the process-group kill
//! itself are `codex.rs`'s shapes with opencode's own evidence in them; R3/K2
//! (adapter only) is why nothing outside `src/backend/` is touched — in
//! particular why this module declares **no new `KIND_*` constant**
//! (`tests/m6_surfaces.rs`'s `t6` would then require an `api::SSE_EVENT_KINDS`
//! edit, which is core) and instead reuses
//! [`KIND_TURN_HARNESS_ERROR`](super::codex::KIND_TURN_HARNESS_ERROR), which
//! already names exactly this fact and is already in that vocabulary.
//!
//! The measured facts this design rests on:
//!
//! - `opencode run --format json -m <provider/model>` writes NDJSON to
//!   stdout, one `{type, timestamp, sessionID, part}` envelope per line, and
//!   takes its prompt on **stdin** with no positional message [probe 8 —
//!   measured working, so a `CONTEXT.md` larger than any argv limit never has
//!   to ride argv]. `current_dir` is the Work's bound surface; `--dir` is
//!   never composed.
//! - The event grammar: `step_start` → (`tool_use` | `text`)* →
//!   `step_finish` {`reason`: `"stop"` | `"tool-calls"`, `tokens`:
//!   {total,input,output,reasoning,cache:{write,read}}, `cost`} [probes 1–2].
//! - **The session id is server-minted**, the opposite of claude's
//!   client-minted `--session-id`: it appears on *every* event and cannot be
//!   chosen. So, exactly like codex's harness-minted `thread_id`, PREPARE
//!   reserves no native id ([`PreparedExecution::native_id`]'s own contract
//!   blesses that as honest) and LAUNCH spawns turn 1 and waits, bounded, for
//!   the first event line ([`SESSION_ID_BUDGET`]) before returning a handle
//!   at all. Until that line lands there is **no session identity**: a
//!   process that dies first leaves nothing resumable, and LAUNCH says so
//!   rather than inventing one.
//! - Later turns are `opencode run --format json -m <model> -s <sessionID>`
//!   and continue the same conversation from a separate process, nonce
//!   continuity measured [probe 5].
//! - **The narration rule is enforceable here** [probe 2]: a `tool_use`
//!   part carries the tool name, `callID`, the full `input`, the `output`,
//!   `metadata.exit` and `metadata.truncated`. This module has exactly one
//!   code path that produces `tool.*` events — [`TurnAccumulator::
//!   ingest_tool_use`] — and no branch anywhere reads a `text` part as
//!   evidence that anything ran. `text` is transcript content, always.
//! - `opencode run` is **non-blocking by construction**: a permission rule
//!   resolving to `ask` auto-rejects in non-interactive mode (stderr notice,
//!   `state.status: "error"` on the tool part, exit 0) [probe 4] — it cannot
//!   hang a stage. This adapter therefore never passes `--auto`, which would
//!   auto-*approve* everything not explicitly denied.
//! - A permission/config block reaches a run through the
//!   `OPENCODE_CONFIG_CONTENT` environment variable, leaving the Work's own
//!   diff surface clean [probe 9 — and `deny` was measured to remove the tool
//!   from the model's toolset entirely rather than reject calls at use time].
//!   W1 wires the *mechanism* ([`OpencodeConfig::config_content`]) and
//!   synthesizes no policy: what belongs in that JSON is W3's mapping work.
//! - Typed terminal error: `{"type":"error", error:{name, data:{message,
//!   ref}}}`, exit 1 [probe 3].
//! - SIGKILL mid-turn truncates the stream with **no terminal event of any
//!   kind**, leaves **no surviving `opencode` process** (`pgrep -x` measured
//!   empty — the bun-compiled binary is a single process), and leaves the
//!   session fully resumable (the pre-kill nonce was recovered by a later
//!   `run -s`) [probe 10]. So INTERRUPT is a plain [`Child::kill`], the
//!   admission tier is `ProcessKill`, and a turn that ends with no terminal
//!   and no requested kill is `NativeState::Unknown` — never inferred
//!   `Exited`.
//! - **Token-free complete history** via `opencode export <sessionID>`
//!   [probe 6]: `{info, messages:[{info:{role, modelID, providerID, finish,
//!   tokens, cost}, parts:[…]}]}`, including `reasoning` parts the SDK's own
//!   documented Part union does not list. This is why
//!   [`Capabilities::history`] is **`true`** here and `false` on both claude
//!   and codex — R4's "parity is the floor, not the ceiling" cashed in.
//! - **Served-model evidence exists** [probe 7]: export names
//!   `info.modelID`/`info.providerID` per assistant message. The `run
//!   --format json` part events do **not** carry a model field, so the pin
//!   check is post-turn and reads export, not the stream ([`verify_model_pin`]).
//!   Note the shape mismatch it has to bridge: the request form is the
//!   slash-joined `"opencode/big-pickle"`, the served form is a split
//!   `providerID: "opencode"` + `modelID: "big-pickle"`.
//!
//! Measured while wiring this module (new facts, recorded here rather than
//! left as folklore):
//!
//! - `opencode --version` prints a **bare** `1.18.19\n` on stdout — no vendor
//!   token, unlike codex's `codex-cli 0.149.0`.
//! - `opencode run --help` and `opencode --help` write their help text to
//!   **stderr**, not stdout (yargs), and exit 0. The probe reads both streams
//!   so a build that moves the text does not become a spurious refusal.
//! - `opencode export <unknown-id>` exits 1 with an empty stdout and stderr
//!   `Error: Session not found: <id>`; a known id exits 0, writes the JSON to
//!   stdout and a one-line `Exporting session: <id>` progress note to stderr.
//!   That exit code is the durable-session evidence RESUME re-adopts on.
//!
//! **The crash window this adapter does not close (stated, not papered
//! over).** The session id cannot be pre-minted, so the journaled
//! `execution.reserved` carries `native_id: null`. A daemon that dies between
//! LAUNCH's spawn and the engine's `execution.started` commit leaves a live
//! `opencode run` whose session id is in no journal, plus a durable session in
//! opencode's own store, and nothing here reaps them. Restart reconciliation
//! fails the work closed (no native id to reconcile), which is the right
//! direction — but the orphan is real. It is the same window `codex.rs`
//! records, for the same structural reason, and it closes on a transport that
//! hands the identity back before any turn exists (W3's `opencode serve`
//! `POST /session`).
//!
//! **Two launch decisions this transport cannot carry (recorded, not
//! silently dropped).**
//!
//! - [`StartRequest::instruction_policy`] has no measured opencode analog:
//!   there is no `--setting-sources`-shaped flag on `run`, and opencode's own
//!   `AGENTS.md` discovery is not something this wave measured a switch for.
//!   Nothing is composed for it, and the resolved policy travels into the
//!   launch evidence (`conversation.user`'s payload) so a reader can see that
//!   it was *carried and not enforced* rather than assume it was applied.
//! - A profile's `config_home` is **refused**, not ignored
//!   ([`OpencodeBackend::launch_config`]): no opencode environment variable
//!   naming a config *home* was measured, and honoring it by guessing one
//!   would be the adapter inventing a launch decision. The measured channel
//!   is `OPENCODE_CONFIG_CONTENT` (content, not a home), which
//!   [`OpencodeConfig::config_content`] carries.
//!
//! Not abstracted from `codex.rs`/`claude.rs` (R2 rung log): the version
//! parser (opencode prints a bare triple — no vendor token to skip), the
//! terminal classifier (opencode's `step_finish{reason}` and typed `error`
//! event are its own vocabulary), `truncate` (5 lines; every owner keeps its
//! own — `runtime/graph.rs` already has a second copy), and the prompt
//! constants. [`ENVIRONMENT_CONTRACT`] and [`MUTATION_SURFACE_HEADER`] are
//! **copied, not imported**, so an edit to another adapter's prompt is never
//! an unreviewed edit to this one; a unit test pins that the environment text
//! matches claude's today.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Write};
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
use crate::platform::process::ProcessArgv;
use crate::runtime::blob::BlobStore;
use crate::runtime::graph::{
    KIND_CONVERSATION_ASSISTANT_COMPLETED, KIND_CONVERSATION_TURN_ENDED, KIND_CONVERSATION_USER,
    KIND_TOOL_COMPLETED, KIND_TOOL_REQUESTED, KIND_USAGE_UPDATED,
};

// ------------------------------------------------------------------ consts

/// Name this backend registers under. Registration itself is W2's gap (K2:
/// this wave is the adapter, nothing else) — this module compiles, is unit-
/// and contract-tested, and is reachable by nothing yet.
pub const OPENCODE_BACKEND_NAME: &str = "opencode";

/// Environment variable naming the `opencode` executable to use — the
/// `SGT_CLAUDE_BIN`/`SGT_CODEX_BIN` pattern, for the same reason: *which*
/// `opencode` is on the daemon's PATH is an operator fact, and `sgt doctor`
/// must be able to report on the same binary the daemon would run. Especially
/// load-bearing here: the packet measured `~/.opencode/bin` absent from a
/// non-interactive shell's PATH on Cerberus (W2 adds it to `harness.rs`'s
/// `toolchain_path_dirs`; until then this variable is the whole answer).
pub const OPENCODE_BIN_ENV: &str = "SGT_OPENCODE_BIN";

/// The opencode version every behavioural claim in this module was measured
/// against (probe packet, 2026-08-23, Cerberus). **Provenance, not a gate**
/// (R1): a build below this floor is `available: true` with an
/// unmeasured-provenance detail; a build at or above it is `available: true`
/// with measured provenance. Neither is refused.
pub const MEASURED_FLOOR: (u64, u64, u64) = (1, 18, 19);

/// Flags this adapter's launch grammar composes, which `opencode run --help`
/// must therefore offer. A build without one of them is a CLI whose grammar
/// this adapter has never measured, so it is refused — the A2 split's second
/// condition, and not a version question.
///
/// Long forms only: the help renders `-m, --model`, so `"--model"` matches
/// and is stable against a short-form change. `--auto` is deliberately absent
/// — this adapter never composes it (probe 4: `run` cannot hang without it,
/// and with it every non-denied permission is auto-approved).
pub const REQUIRED_RUN_FLAGS: &[&str] = &[
    "--format",  // the whole event transport
    "--model",   // the model pin; `-m` is its short form
    "--session", // the resume grammar; `-s` is its short form
];

/// Subcommands `opencode --help` must list. `run` is the transport; `export`
/// is what makes [`Capabilities::history`] `true` (probe 6) and what RESUME
/// re-adopts a durable session on, so a build without it is a build this
/// adapter's capability row would be lying about.
pub const REQUIRED_SUBCOMMANDS: &[&str] = &["run", "export"];

/// ADR 0007(a)'s execution-model half, opencode-worded. Reworded from the
/// sibling adapters' constants of the same purpose rather than imported: the
/// three execution models are not guaranteed to stay identical (W3's serve
/// transport has a real approval channel, which this sentence would then be
/// wrong about), so an edit to one must not silently change the others.
///
/// The permission sentence states probe 4's measurement rather than codex's:
/// opencode does not refuse the *action*, it auto-rejects the *permission*
/// and hands the model a tool error it can read and plan around.
pub const EXECUTION_MODEL_CONTRACT: &str = "\
Execution model: this is a single non-interactive turn (`opencode run`). You get one turn and no \
callbacks — nothing wakes you when a command you backgrounded finishes after you end your turn. \
There is no approval channel and no way to ask a human anything during this turn: any tool call \
that would need a human decision is auto-rejected by the harness and returned to you as a tool \
error, so plan around it rather than waiting for one. If a command might take a while, run it in \
the foreground with an adequate timeout and wait for it to finish before ending your turn.";

/// `claude.rs`'s `ENVIRONMENT_CONTRACT`, copied verbatim rather than imported
/// (the same rule `codex.rs` follows): its text already names `opencode` and
/// is transport-agnostic, but an `opencode.rs -> claude.rs` import would make
/// an edit to the Claude adapter's prompt an unreviewed edit to this one. A
/// unit test (`the_environment_contract_matches_claudes_today`) pins that the
/// two texts are equal *today*, so a divergence is a decision, not drift
/// nobody noticed.
pub const ENVIRONMENT_CONTRACT: &str = "\
Environment: if this session was reached through `sgt claude` (or `sgt codex`/`opencode`/\
`goose`), your PATH was deliberately composed before this turn was launched to include your \
toolchain (e.g. `~/.cargo/bin`, `~/.local/bin`), and you are bound to the estate that launch \
discovered — sergeant's daemon and every actor beneath it inherit that same environment. This \
does not hold for a daemon reached any other way: a terminal that never went through `sgt \
<harness>` inherits whatever environment it happened to have. If a tool you expect is missing, \
that is more likely an unenriched PATH than a permissions fault — run `sgt doctor` to check what \
this installation's environment actually guarantees before assuming otherwise.";

/// §10.1's section header, opencode-local copy of the sibling adapters'
/// private constant of the same text (same reasoning as
/// [`ENVIRONMENT_CONTRACT`]).
const MUTATION_SURFACE_HEADER: &str = "\
Mutation surface: this Work may modify exactly the worktree(s) listed below, and nothing else. \
The estate root, the `repos/` mounts those worktrees were cut from, unselected repositories, \
other Works' surfaces, and any other path on this machine are outside what this Work is \
authorized to change. Each worktree is already checked out on its own branch at its own base \
commit:";

/// How long LAUNCH waits for the first event line — the one that carries the
/// server-minted `sessionID` — before concluding the launch failed. Generous
/// by an order of magnitude: `step_start` was the first line of every
/// recorded stream, emitted before the model's first token, and even the
/// failing run (probe 3) emitted its typed `error` line promptly.
const SESSION_ID_BUDGET: Duration = Duration::from_secs(30);

/// How long the turn reader waits for stderr after the turn's process has
/// been reaped — the sibling adapters' identical fix for the same race (both
/// pipes reach EOF at the same instant; a reader that snapshots a shared
/// buffer the moment stdout closes is racing the thread still filling it).
/// Opencode needs it: the auto-rejection notice (probe 4) is *only* on
/// stderr, and it is the whole explanation of a tool part that says `error`.
const STDERR_DRAIN_BUDGET: Duration = Duration::from_secs(5);

/// Cap on the **in-memory** accumulation of one turn's raw NDJSON stdout
/// before it is archived to the blob store (§20), and of its stderr. Neither
/// pipe had a cap before this: a turn with very large tool output (a `cat` of
/// a big file) or a process that writes pathologically to stderr could grow
/// `TurnReader::run`'s `raw` `String` and the stderr-drain thread's buffer
/// without bound, a per-turn, per-daemon memory-exhaustion vector (`codex.rs`
/// carries the identical unbounded pattern; this bounds only this adapter's
/// copy of it, since `codex.rs` is out of this wave's scope). Every line is
/// still parsed and forwarded to the normal event pipeline regardless of
/// this cap — nothing downstream of a single line is affected — and both
/// pipes are still drained to EOF past the cap, so a capped turn's process
/// never blocks on a full pipe. What is lost past the cap is only the raw
/// archive's completeness beyond it; [`TurnReader::run`] and
/// [`read_bounded`] mark that loss in the archived text itself rather than
/// silently truncating it. 16 MiB is generous against every fixture this
/// suite carries (the largest is a few KiB) while still being a real,
/// finite bound rather than none at all.
const STREAM_MEMORY_CAP: usize = 16 * 1024 * 1024;

/// Bounded tail of a `tool_use` part's `output` kept inline in
/// `tool.completed`'s payload. The full bytes are never dropped up to
/// [`STREAM_MEMORY_CAP`] — they are in the turn's raw blob by construction
/// (§20), unless the turn's raw stream itself hit that cap — this only
/// bounds what an unbounded command output costs the journal a second time.
/// `docker.rs`'s
/// `TAIL_BYTES` bounded-tail-plus-blob pattern is the precedent.
const TOOL_OUTPUT_TAIL: usize = 1024;

/// The environment variable that delivers a whole opencode config (JSON) to
/// one run without writing `opencode.json` into the Work's diff surface
/// [probe 9 — found in the bundled SDK server source and measured honored].
pub const OPENCODE_CONFIG_CONTENT_ENV: &str = "OPENCODE_CONFIG_CONTENT";

// ----------------------------------------------------------------- config

/// Launch configuration for the adapter, resolved once at construction from
/// the daemon's own environment.
///
/// `Debug` is hand-written, not derived (below): `env` and `config_content`
/// can plausibly carry provider API keys (the latter is documented above as
/// "a whole opencode config document", per opencode's own config schema),
/// and a derived `Debug` would print both in full into any future `{:?}`
/// format of this struct -- a diagnostic log line, a panic message, a
/// `dbg!()` left in during later wiring. No call site formats this struct
/// today, but the hazard is latent, not hypothetical, so it is closed here
/// rather than left for the first log line that does.
#[derive(Clone)]
pub struct OpencodeConfig {
    /// The CLI executable (a profile may override it per execution).
    pub executable: PathBuf,
    /// Sergeant's data dir; raw per-turn stdout is archived to its blob store
    /// (§20).
    pub data_dir: PathBuf,
    /// Extra environment for every spawned turn (and for the token-free
    /// `export` calls HISTORY, RESUME and the pin check make).
    pub env: BTreeMap<String, String>,
    /// A whole opencode config document (JSON text), delivered per launch
    /// through [`OPENCODE_CONFIG_CONTENT_ENV`] [probe 9]. `None` — the
    /// default — means the operator's own opencode configuration decides,
    /// which is exactly the posture probe 4 measured as non-blocking.
    ///
    /// W1 wires the channel and synthesizes nothing: mapping sergeant's
    /// declared mutation surface onto opencode's per-tool glob permission
    /// vocabulary is W3's spec'd work, and a policy this wave invented would
    /// be a security decision made without a measurement behind it.
    pub config_content: Option<String>,
    /// Override for [`SESSION_ID_BUDGET`], `None` in every production path.
    /// A per-instance field rather than an environment variable, for the
    /// reason `CodexConfig::thread_id_budget` documents: each test builds its
    /// own config, so a shrunk budget can never leak into another test's
    /// `launch()` — no process-global mutable state, no `--test-threads`
    /// ordering hazard, no `unsafe { std::env::set_var }` to serialize.
    pub session_id_budget: Option<Duration>,
}

impl std::fmt::Debug for OpencodeConfig {
    /// Redacts `env` (values may be provider API keys) and `config_content`
    /// (may embed provider credentials per opencode's own config schema,
    /// probe 9) to a count and a length rather than their contents -- see
    /// the struct's own doc comment for why this is hand-written.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpencodeConfig")
            .field("executable", &self.executable)
            .field("data_dir", &self.data_dir)
            .field("env", &format!("<{} vars, redacted>", self.env.len()))
            .field(
                "config_content",
                &self
                    .config_content
                    .as_ref()
                    .map(|c| format!("<redacted, {} bytes>", c.len())),
            )
            .field("session_id_budget", &self.session_id_budget)
            .finish()
    }
}

impl OpencodeConfig {
    /// Config for a daemon owning `data_dir`, with the system `opencode`.
    pub fn new(data_dir: &Path) -> Self {
        Self {
            executable: std::env::var_os(OPENCODE_BIN_ENV)
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("opencode")),
            data_dir: data_dir.to_path_buf(),
            env: BTreeMap::new(),
            config_content: None,
            session_id_budget: None,
        }
    }
}

// ---------------------------------------------------------- admission rows

/// Which transport a row's evidence was gathered on. One variant today, and
/// the column exists anyway: W3 adds `Serve` (the `opencode serve` HTTP+SSE
/// child), and the codex sprint's own experience is that retrofitting the
/// column *after* a second transport arrives silently re-attributes every
/// existing row to it. Transport-tagged from day one, as the plan asks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Transport {
    /// `opencode run --format json`, this wave's only transport.
    Run,
}

impl Transport {
    fn as_str(self) -> &'static str {
        match self {
            Transport::Run => "run-json",
        }
    }
}

/// How a capability's `true`/`false` was established. Deliberately the codex
/// ledger's four tiers, with the third renamed: opencode publishes no
/// generated JSON schema to cite, so the doc-only tier is named for what it
/// actually is — a claim in `opencode.ai/docs` that proves the product
/// *names* a thing, never that it fires here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Evidence {
    /// Driven against the real, installed harness (an `#[ignore]`d live test,
    /// gated behind `SERGEANT_OPENCODE_TESTS=1`).
    LiveMeasured,
    /// Proven deterministically (a committed fixture, the shell stub) without
    /// a live run — still a real assertion, just not against the installed
    /// binary today.
    LocallyMeasured,
    /// Named by opencode's own documentation; never promoted to
    /// `claimed: true` on this evidence alone.
    DocClaimed,
    /// Looked for and not found — a probe ran, no assertion could be made.
    Unmeasured,
}

/// One row of the wave's capability ledger: the v1 boolean `capabilities()`
/// returns is the contract; this is the *evidence* behind it, adapter-local
/// until a contract revision gives it a home. Rendered into
/// [`ProbeReport::detail`] and the wave PR body.
///
/// **No `stability` column, deliberately (R1).** Codex's ledger carries one
/// because its two transports genuinely differ (`stable` exec vs. an
/// `[experimental]`-labelled app-server). Every row here would carry the same
/// value — opencode publishes no breaking-change policy for any surface, and
/// the repo moved `sst` → `anomalyco` mid-flight — so the fact is stated once,
/// on [`render_admission_rows`]'s own header, instead of repeated fourteen
/// times. The column arrives when a second value does.
#[derive(Debug, Clone, Copy)]
struct AdmissionRow {
    /// The v1 flag name, or a name v1 has no row for at all
    /// (`config_injection`, `non_blocking_run`).
    capability: &'static str,
    transport: Transport,
    /// What `capabilities()` claims for this transport.
    claimed: bool,
    /// The typed tier this row's evidence actually supports (`ProcessKill`,
    /// `ExportSnapshot`, …), or `"-"` when the flag is a plain boolean with
    /// no tier of its own.
    tier: &'static str,
    evidence: Evidence,
    /// The exact test name backing a `claimed: true`, or `""` when `claimed`
    /// is `false` (the structural reason lives in `note`).
    admission_test: &'static str,
    note: &'static str,
}

/// The wave's own ledger. [`tests::admission_rows_agree_with_capabilities`]
/// is the structural check that keeps it honest: a `claimed: true` with no
/// `admission_test` fails the build, and so does a row whose `Evidence` tier
/// disagrees with whether its named test is a `live_*` one.
const ADMISSION_ROWS: &[AdmissionRow] = &[
    AdmissionRow {
        capability: "persistent_sessions",
        transport: Transport::Run,
        claimed: true,
        tier: "-",
        evidence: Evidence::LocallyMeasured,
        // `launch_binds_the_server_minted_session_id` only proves the id is
        // bound and cited within one turn -- it never issues a second turn,
        // so it cannot back "sessions outlive a single turn" (mod.rs's own
        // definition). `a_resume_turn_names_the_session_and_keeps_the_pin`
        // is the test that actually does: it drives SEND after LAUNCH's turn
        // has settled, and asserts the *same* session id is composed
        // (`-s <id>`) on that second, separately-spawned `opencode run`
        // process -- the session surviving past the first turn's process,
        // which is exactly this capability's claim.
        admission_test: "a_resume_turn_names_the_session_and_keeps_the_pin",
        note: "the session is minted by the harness on turn 1 and reused, unprompted, as the \
               pin on turn 2's separately-spawned process -- opencode's own store keeps it \
               (also measured resumable even after an uncleanly killed turn, probe 10, and \
               across a daemon restart under the distinct `resume` capability below)",
    },
    AdmissionRow {
        capability: "native_background",
        transport: Transport::Run,
        claimed: false,
        tier: "-",
        evidence: Evidence::Unmeasured,
        admission_test: "",
        note: "no measured mechanism for a `run` turn to survive its own process; the turn IS the \
               process",
    },
    AdmissionRow {
        capability: "streaming",
        transport: Transport::Run,
        claimed: true,
        tier: "-",
        evidence: Evidence::LocallyMeasured,
        admission_test: "events_are_delivered_before_the_turn_process_exits",
        note: "NDJSON, one event per line, read and normalized as it arrives -- the stub stalls \
               mid-stream and the assertion is that events already landed",
    },
    AdmissionRow {
        capability: "history",
        transport: Transport::Run,
        claimed: true,
        tier: "ExportSnapshot",
        evidence: Evidence::LiveMeasured,
        admission_test: "live_opencode_history_exports_the_whole_session",
        note: "`opencode export <sessionID>`: token-free, complete, role-attributed, and it \
               includes `reasoning` parts the documented Part union does not list (probe 6). \
               Neither claude.rs nor codex.rs claims this flag -- R4's 'parity is the floor' \
               cashed in. The deterministic decoder proof against the committed export fixture \
               is `history_decodes_every_message_and_part_of_the_export_fixture`",
    },
    AdmissionRow {
        capability: "resume",
        transport: Transport::Run,
        claimed: true,
        tier: "-",
        evidence: Evidence::LiveMeasured,
        admission_test: "live_opencode_resume_recalls_a_nonce_across_processes",
        note: "`run -s <sessionID>` from a separate OS process (probe 5); RESUME's own \
               re-adoption evidence is `opencode export`'s exit status, measured 0 for a known \
               id and 1 with `Session not found` for an unknown one",
    },
    AdmissionRow {
        capability: "interrupt",
        transport: Transport::Run,
        claimed: true,
        tier: "ProcessTreeTermination",
        // `opencode_interrupt_kills_the_process_group` is StubOpencode-driven
        // and deterministic, mirroring `codex.rs`'s own split between this
        // row and a live resumability test -- tagging this row
        // `LiveMeasured` would credit the wrong test with a live run it
        // never performs.
        evidence: Evidence::LocallyMeasured,
        admission_test: "opencode_interrupt_kills_the_process_group",
        note: "probe 10 measured `pgrep -x opencode` empty after a plain Child::kill() (the \
               bun-compiled binary is a single process), but probe 11 (2026-08-23, \
               post-W1-implementation) measured the gap that scan missed: a bash-tool \
               grandchild reparented to init and kept running after the harness was killed. \
               interrupt now mirrors codex.rs's process-group termination -- process_group(0) \
               at spawn, a negated-pgid SIGKILL at interrupt -- so the whole turn tree dies, not \
               just the opencode leader",
    },
    AdmissionRow {
        capability: "model_selection",
        transport: Transport::Run,
        claimed: true,
        tier: "ExportVerifiedPin",
        evidence: Evidence::LocallyMeasured,
        admission_test: "a_substituted_model_is_a_failed_observation",
        note: "-m on every turn including resumes; substitution is DETECTABLE here (unlike \
               codex-exec, whose row says substitution-undetectable): export names \
               info.providerID/info.modelID per assistant message (probe 7), so the post-turn \
               verdict is positive evidence, not 'attempted'. One layer, not claude's three: \
               there is no measured pre-flight refusal shape to build layer 1 on",
    },
    AdmissionRow {
        capability: "profiles",
        transport: Transport::Run,
        claimed: true,
        tier: "-",
        evidence: Evidence::LocallyMeasured,
        admission_test: "a_profile_executable_and_env_reach_every_turn",
        note: "the generic sergeant axes only: executable + env. `config_home` is refused, not \
               ignored (no measured opencode config-home variable), and opencode's own `--agent` \
               is NOT wired in this wave -- an agent applied to turn 1 must be re-applied on \
               every resume, and that re-application is unmeasured",
    },
    AdmissionRow {
        capability: "approval_flow",
        transport: Transport::Run,
        claimed: false,
        tier: "-",
        evidence: Evidence::DocClaimed,
        admission_test: "",
        note: "structural on this transport: a permission that resolves to `ask` auto-rejects \
               (probe 4) -- there is nobody to approve to. The interactive loop \
               (`permission.asked` + POST /session/:id/permissions/:permissionID) is doc-claimed \
               on the serve transport and is W3's first-true candidate",
    },
    AdmissionRow {
        capability: "human_attach",
        transport: Transport::Run,
        claimed: false,
        tier: "-",
        evidence: Evidence::Unmeasured,
        admission_test: "",
        note: "no attach mechanism on a non-interactive `run` turn (`run --attach <url>` attaches \
               THIS client to a server, not a human to this turn)",
    },
    AdmissionRow {
        capability: "usage",
        transport: Transport::Run,
        claimed: true,
        tier: "-",
        evidence: Evidence::LocallyMeasured,
        admission_test: "step_finish_tokens_and_cost_become_usage_events",
        note: "one usage.updated per step_finish, carrying that step's native \
               {total,input,output,reasoning,cache:{write,read}} and cost -- known DURING the \
               turn, not only at its end (codex-exec learns usage only at turn.completed). \
               Export carries per-message tokens/cost as a second, token-free source",
    },
    AdmissionRow {
        capability: "native_subagents",
        transport: Transport::Run,
        claimed: false,
        tier: "-",
        evidence: Evidence::DocClaimed,
        admission_test: "",
        note: "opencode has agents (`opencode agent`, `run --agent`) [doc-claimed]; how or \
               whether a subagent surfaces in the run-json event grammar was never measured, and \
               documented is not supported (§15)",
    },
    AdmissionRow {
        capability: "ask",
        transport: Transport::Run,
        claimed: false,
        tier: "-",
        evidence: Evidence::Unmeasured,
        admission_test: "",
        note: "no measured actor-authored-question record in this event grammar. Export's session \
               info carries a `question` permission entry (defaulted to `deny`), which says a \
               question CATEGORY exists -- not that an actor's question is schema-distinguishable \
               end-of-turn here. Guessing one from a text part is precisely the heuristic \
               Capabilities::ask forbids",
    },
    // Two rows with no v1 boolean at all, the same adapter-local-evidence
    // posture codex's `structured_output`/`sandbox_enforcement` rows take.
    AdmissionRow {
        capability: "config_injection",
        transport: Transport::Run,
        claimed: true,
        tier: "EnvConfigContent",
        evidence: Evidence::LocallyMeasured,
        admission_test: "config_content_reaches_every_child_process",
        note: "OPENCODE_CONFIG_CONTENT delivers a whole config per launch, leaving the Work's own \
               diff surface clean (probe 9). W1 wires the mechanism and synthesizes no policy; \
               `deny` was measured to remove the tool from the toolset entirely rather than \
               reject at use time, which is a mapping subtlety W3 owns",
    },
    AdmissionRow {
        capability: "non_blocking_run",
        transport: Transport::Run,
        claimed: true,
        tier: "AutoRejectOnAsk",
        evidence: Evidence::LocallyMeasured,
        admission_test: "an_auto_rejected_tool_call_is_a_failed_tool_event_not_a_hang",
        note: "probe 4: an `ask`-resolving permission auto-rejects in non-interactive mode \
               (stderr notice, state.status:\"error\", exit 0) -- the same non-hang guarantee \
               `codex exec` has, and the reason this adapter never passes `--auto`",
    },
];

/// Render [`ADMISSION_ROWS`] into the plain-text table the wave PR body
/// pastes verbatim and the probe's own journaled detail carries. The
/// stability fact every row would otherwise repeat is stated once, in the
/// header — see [`AdmissionRow`]'s own doc for why it is not a column.
fn render_admission_rows() -> String {
    let mut out = String::from(
        "stability (all rows): opencode publishes no API/CLI breaking-change policy; \
         MEASURED_FLOOR 1.18.19 is provenance, not a gate (R1)\n\
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

/// Outcome of the registration-time version/grammar probe.
///
/// `version` and `provenance` are carried for a future `sgt doctor` /
/// provenance reader (W2's hand-off) even though nothing in this wave reads
/// them back out — `detail` already carries their rendering for today's one
/// reader ([`ProbeReport`]). Allowed dead code rather than dropped: W1 must
/// not invent W2's reader, but must not throw away the fields it will need
/// either.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ProbeOutcome {
    available: bool,
    detail: String,
    version: Option<String>,
    provenance: Option<VersionProvenance>,
}

/// Parse `opencode --version`'s output into a comparable triple.
///
/// Unlike codex, opencode prints a **bare** `1.18.19\n` — no vendor token —
/// so this takes the *first* whitespace-separated token rather than the last:
/// a build that later prefixes a vendor name would be a grammar change worth
/// noticing, and a build that suffixes a git hash (`1.18.19 abc1234`) should
/// still parse. The patch segment is read up to its first non-digit, so a
/// pre-release suffix (`1.18.19-rc.1`) still yields a comparable triple; the
/// full string always travels in the probe's `detail`, never silently
/// dropped.
fn parse_opencode_version(text: &str) -> Option<(u64, u64, u64)> {
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

// --------------------------------------------------------- launch grammar

/// One execution's resolved launch configuration (§14 applied to this CLI).
/// One function ([`OpencodeBackend::launch_config`]) produces it for both
/// LAUNCH and RESUME, so a re-adopted execution cannot launch under different
/// rules than the one it re-adopts.
#[derive(Debug, Clone)]
struct LaunchConfig {
    executable: PathBuf,
    env: BTreeMap<String, String>,
}

/// Turn 1's argv, after `<executable>`: `run --format json [-m <model>]`.
/// The prompt travels on stdin with **no positional message** (probe 8), and
/// the working directory is set with `Command::current_dir`, never `--dir`.
fn first_turn_argv(model: Option<&str>) -> Vec<String> {
    let mut argv = vec![
        "run".to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];
    if let Some(model) = model {
        argv.push("-m".to_string());
        argv.push(model.to_string());
    }
    argv
}

/// Turn N >= 2's argv: turn 1's, plus `-s <sessionID>`.
///
/// The model pin is composed here too, exactly as on turn 1: a pin the human
/// asked for that silently lapses after the first turn is the adapter
/// dropping a launch decision. The session id sits immediately after `-s`,
/// which is what makes [`argv_names_session`]'s liveness rule an adjacency
/// check rather than a substring search.
fn resume_turn_argv(model: Option<&str>, session_id: &str) -> Vec<String> {
    let mut argv = first_turn_argv(model);
    argv.push("-s".to_string());
    argv.push(session_id.to_string());
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
/// may modify nothing" is a claim its silence does not make), the intent,
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

/// Every binding path this request carries that does not lie at or under
/// `cwd`. Opencode composes no `--add-dir`-shaped flag at all, so nothing
/// here changes a launch — this exists so the assumption that sergeant's own
/// binding shape keeps every worktree under the surface root is *checked*
/// rather than trusted, and a future shape that breaks it is a fact in the
/// launch evidence instead of a silent gap.
fn bindings_outside_cwd(cwd: &Path, bindings: &[BindingSummary]) -> Vec<PathBuf> {
    bindings
        .iter()
        .filter(|binding| !binding.worktree_path.starts_with(cwd))
        .map(|binding| binding.worktree_path.clone())
        .collect()
}

/// Pre-flight pin check: refuse only an empty or whitespace-only pin.
///
/// Claude's provider-qualification refusal is deliberately **inverted** here
/// rather than copied: opencode's `-m` help says "model to use in the format
/// of `provider/model`", so a slash is the *expected* shape, not a refusal
/// trigger. What a bare, unqualified pin does on this CLI was never measured,
/// so it is not refused either — [`verify_model_pin`] records honestly that
/// such a pin can only be checked against `modelID`, never against the
/// provider.
fn preflight_model_pin(model: &str) -> Result<(), String> {
    if model.trim().is_empty() {
        return Err("model pin is empty".to_string());
    }
    Ok(())
}

// ------------------------------------------------------- model pin verdict

/// The verdict of this transport's one-layer pin verification (probe 7).
///
/// Kept simpler than `claude.rs`'s three layers because the evidence supports
/// less: there is no measured pre-flight refusal shape (layer 1) and the run
/// stream carries no model field (claude's layer 2 lives in `system:init`).
/// What *does* exist is positive post-turn evidence — export's per-assistant-
/// message `providerID`/`modelID` — which is more than `codex.rs` has, and it
/// is why `Honored` and `Substituted` are reachable arms here rather than the
/// structurally-dead ones codex documents.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PinVerdict {
    /// No pin was requested; nothing to verify.
    Unpinned,
    /// Positive evidence: export's served `providerID`/`modelID` matches the
    /// requested pin. Carries the served, slash-joined identity.
    Honored(String),
    /// Export names a model that does not match the pin. Carries what ran.
    Substituted(String),
    /// No usable model evidence could be gathered (export failed, or named no
    /// assistant message). The pin is recorded as attempted, never as
    /// honored — and, unlike a substitution, this is not by itself a stage
    /// failure.
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
    /// other verdict. Only a substitution is a stage failure: a pin that
    /// could not be checked is missing evidence, and failing a stage on
    /// missing evidence would be this adapter deciding a Work's fate on
    /// something it never saw.
    fn mismatch(&self, requested: Option<&str>) -> Option<String> {
        match self {
            PinVerdict::Substituted(served) => Some(format!(
                "model pin not honored: requested {}, opencode export names {served} as the model \
                 that served this turn",
                requested.unwrap_or("<none>")
            )),
            PinVerdict::Unpinned | PinVerdict::Honored(_) | PinVerdict::Attempted(_) => None,
        }
    }
}

/// Compare a requested pin against the model export says actually served the
/// session's **last** assistant message (probe 7).
///
/// The last one, not the first: a multi-turn session's earlier messages were
/// served by whatever pin *those* turns carried, and this verdict is about
/// the turn that just ended. The shape mismatch this bridges is the whole
/// subtlety — the request form is slash-joined (`"opencode/big-pickle"`) and
/// the served form is split (`providerID: "opencode"`, `modelID:
/// "big-pickle"`) — so a pin carrying no slash can only be checked against
/// `modelID`, and [`PinVerdict::Attempted`]'s detail says so rather than
/// pretending the provider was verified.
fn verify_model_pin(requested: Option<&str>, export: &Value) -> PinVerdict {
    let Some(requested) = requested else {
        return PinVerdict::Unpinned;
    };
    let served = export
        .get("messages")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|message| {
            message.pointer("/info/role").and_then(Value::as_str) == Some("assistant")
        })
        .filter_map(|message| {
            let model = message.pointer("/info/modelID").and_then(Value::as_str)?;
            let provider = message
                .pointer("/info/providerID")
                .and_then(Value::as_str)
                .unwrap_or("");
            Some((provider.to_string(), model.to_string()))
        })
        .next_back();
    let Some((provider, model)) = served else {
        return PinVerdict::Attempted(
            "opencode export named no assistant message carrying info.modelID, so nothing here \
             evidences which model served this turn"
                .to_string(),
        );
    };
    let joined = if provider.is_empty() {
        model.clone()
    } else {
        format!("{provider}/{model}")
    };
    match requested.split_once('/') {
        Some((requested_provider, requested_model)) => {
            if requested_provider == provider && requested_model == model {
                PinVerdict::Honored(joined)
            } else {
                PinVerdict::Substituted(joined)
            }
        }
        // An unqualified pin: opencode's own `-m` help asks for
        // `provider/model`, so this shape was never measured. It is compared
        // against `modelID` alone and the verdict says which half went
        // unchecked, rather than silently treating a provider match as
        // proven.
        None => {
            if requested == model {
                PinVerdict::Honored(joined)
            } else {
                PinVerdict::Substituted(joined)
            }
        }
    }
}

// ---------------------------------------------------------------- decoding

/// One finished turn's terminal shape, before any process-exit evidence is
/// folded in (that happens in [`classify_terminal`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum Terminal {
    /// Neither a `step_finish{reason:"stop"}` nor an `error` line was seen.
    #[default]
    None,
    /// A `step_finish` carrying `reason: "stop"` was seen.
    Stopped,
    /// A typed `{"type":"error"}` line was seen (probe 3).
    Error {
        name: String,
        message: String,
        reference: String,
    },
}

/// The whole decoder: folds one turn's NDJSON stream into normalized events
/// plus the honest counts the narration rule needs. Pure — no I/O, no process
/// — which is what makes the fixture-driven suite possible with no `opencode`
/// binary anywhere in the loop.
#[derive(Debug, Default)]
struct TurnAccumulator {
    /// The server-minted session id, learned from the first event that
    /// carries one (every event does).
    session_id: Option<String>,
    /// `step_start` lines seen.
    steps: u32,
    /// `text` parts seen — transcript content, never tool evidence.
    text_parts: u32,
    /// `tool_use` parts that reached a resolved state.
    tool_parts: u32,
    /// `callID`s a `tool.requested` has already been emitted for, so a
    /// harness that ever emits a pending `tool_use` followed by a resolved
    /// one cannot produce two requests for one call.
    requested_calls: BTreeSet<String>,
    /// Envelope `type` strings this decoder does not know, counted and named
    /// but never decoded (they are in the raw blob by construction).
    unknown_events: Vec<String>,
    /// Lines that did not parse as JSON at all.
    unparsed_lines: u32,
    /// `text` parts of the step currently in flight, cleared at each
    /// `step_start`.
    current_step_texts: Vec<String>,
    /// The text of the most recently finished step — the completion summary
    /// rule ("the concatenated text of the final step").
    last_step_summary: Option<String>,
    /// The final `step_finish`'s `tokens` object, **verbatim**. Never a
    /// synthetic sum: `input` already includes cache reads, and adding them
    /// across steps would invent a number nobody measured.
    last_tokens: Option<Value>,
    /// Summed `cost` across every step of this turn. Cost *is* additive by
    /// construction, unlike the token object.
    cost_total: f64,
    /// Every `step_finish`'s `reason`, in order.
    reasons: Vec<String>,
    /// This turn's terminal, if any.
    terminal: Terminal,
    /// The typed error's rendered message, kept for the ambiguous case's
    /// evidence — it is often the only thing that says *why* a turn never
    /// reached a terminal.
    last_error: Option<String>,
}

impl TurnAccumulator {
    fn new() -> Self {
        Self::default()
    }

    /// Ingest one already-parsed line, returning the normalized events it
    /// produced. Malformed-line counting happens in the caller, which only
    /// reaches this function on a line that parsed.
    fn ingest_line(&mut self, value: &Value) -> Vec<NativeEvent> {
        if self.session_id.is_none()
            && let Some(id) = value.get("sessionID").and_then(Value::as_str)
            && !id.is_empty()
        {
            self.session_id = Some(id.to_string());
        }
        let part = value.get("part");
        let mut out = Vec::new();
        match value.get("type").and_then(Value::as_str) {
            Some("step_start") => {
                self.steps += 1;
                self.current_step_texts.clear();
            }
            // The narration rule, stated as code: a text part becomes
            // transcript content and nothing else. No branch in this decoder
            // reads it as evidence that a command ran.
            Some("text") => {
                let text = part
                    .and_then(|p| p.get("text"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                self.text_parts += 1;
                self.current_step_texts.push(text.clone());
                out.push(NativeEvent {
                    kind: KIND_CONVERSATION_ASSISTANT_COMPLETED.to_string(),
                    payload: json!({
                        "session_id": self.session_id,
                        "text": text,
                        "part_id": part.and_then(|p| p.get("id")).cloned().unwrap_or(Value::Null),
                    }),
                });
            }
            Some("tool_use") => self.ingest_tool_use(part, &mut out),
            Some("step_finish") => self.ingest_step_finish(part, &mut out),
            Some("error") => self.ingest_error(value, &mut out),
            Some(other) => self.unknown_events.push(other.to_string()),
            None => self.unknown_events.push("<no type field>".to_string()),
        }
        out
    }

    /// The **only** code path in this module that produces `tool.*` events.
    ///
    /// One measured `tool_use` line carries both facts a `tool.requested` /
    /// `tool.completed` pair reports — the input the actor asked for, and the
    /// state the harness resolved it to — so both events come from that one
    /// line. Nothing is inferred: each event's payload is read out of the
    /// part, and `requested_calls` makes the pair idempotent for a build that
    /// ever emits an unresolved `tool_use` first.
    fn ingest_tool_use(&mut self, part: Option<&Value>, out: &mut Vec<NativeEvent>) {
        let Some(part) = part else {
            self.unknown_events.push("tool_use (no part)".to_string());
            return;
        };
        let call_id = part
            .get("callID")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let tool = part
            .get("tool")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let state = part.get("state");
        let status = state
            .and_then(|s| s.get("status"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if self.requested_calls.insert(call_id.clone()) {
            out.push(NativeEvent {
                kind: KIND_TOOL_REQUESTED.to_string(),
                payload: json!({
                    "id": call_id,
                    "name": tool,
                    "input": state.and_then(|s| s.get("input")).cloned().unwrap_or(Value::Null),
                }),
            });
        }
        // Unresolved states (a build that emits a pending `tool_use`) have no
        // outcome to report yet; the resolved line that follows carries it.
        if status != "completed" && status != "error" {
            return;
        }
        self.tool_parts += 1;
        let exit_code = state
            .and_then(|s| s.pointer("/metadata/exit"))
            .cloned()
            .unwrap_or(Value::Null);
        let output = state
            .and_then(|s| s.get("output"))
            .and_then(Value::as_str)
            .unwrap_or("");
        // Two independent ways a call can have failed, and both are read:
        // the harness's own `status` (probe 4's auto-rejection sets it to
        // `error` with no metadata at all) and the command's exit code
        // (probe 2's `metadata.exit`).
        let is_error = status != "completed" || exit_code.as_i64().is_some_and(|code| code != 0);
        out.push(NativeEvent {
            kind: KIND_TOOL_COMPLETED.to_string(),
            payload: json!({
                "tool_use_id": call_id,
                "name": tool,
                "is_error": is_error,
                "status": status,
                "exit_code": exit_code,
                "truncated": state
                    .and_then(|s| s.pointer("/metadata/truncated"))
                    .cloned()
                    .unwrap_or(Value::Null),
                "error": state.and_then(|s| s.get("error")).cloned().unwrap_or(Value::Null),
                "output_tail": truncate(output, TOOL_OUTPUT_TAIL),
            }),
        });
    }

    /// `step_finish` is both the usage record and — when its `reason` is
    /// `"stop"` — the turn's terminal.
    fn ingest_step_finish(&mut self, part: Option<&Value>, out: &mut Vec<NativeEvent>) {
        let reason = part
            .and_then(|p| p.get("reason"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let tokens = part
            .and_then(|p| p.get("tokens"))
            .cloned()
            .unwrap_or(Value::Null);
        let cost = part
            .and_then(|p| p.get("cost"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        self.cost_total += cost;
        self.last_tokens = Some(tokens.clone());
        let summary = self.current_step_texts.join("\n");
        self.last_step_summary = (!summary.is_empty()).then_some(summary);
        if reason == "stop" {
            self.terminal = Terminal::Stopped;
        }
        out.push(NativeEvent {
            kind: KIND_USAGE_UPDATED.to_string(),
            payload: json!({
                "session_id": self.session_id,
                "step": self.steps,
                "reason": reason,
                "tokens": tokens,
                "cost": cost,
            }),
        });
        self.reasons.push(reason);
    }

    /// The typed terminal error (probe 3). Unlike codex's bare `error` line —
    /// which that harness emits as a *warning* on turns that continue — this
    /// one was measured as the turn's whole terminal, with exit 1 and an
    /// otherwise empty stream, so it sets [`Terminal::Error`] rather than
    /// merely being remembered.
    fn ingest_error(&mut self, value: &Value, out: &mut Vec<NativeEvent>) {
        let name = value
            .pointer("/error/name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let message = value
            .pointer("/error/data/message")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let reference = value
            .pointer("/error/data/ref")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        out.push(NativeEvent {
            kind: KIND_TURN_HARNESS_ERROR.to_string(),
            payload: json!({
                "phase": "typed_error",
                "name": name,
                "message": message,
                "ref": reference,
            }),
        });
        self.last_error = Some(render_typed_error(&name, &message, &reference));
        self.terminal = Terminal::Error {
            name,
            message,
            reference,
        };
    }
}

/// One typed error, rendered for a human — the same string in the accumulator
/// and in the failing observation, so a reader never has to reconcile two
/// spellings of one fact.
fn render_typed_error(name: &str, message: &str, reference: &str) -> String {
    let mut rendered = format!("{name}: {message}");
    if !reference.is_empty() {
        rendered.push_str(&format!(" (ref {reference})"));
    }
    rendered
}

/// The shape one finished turn resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TerminalOutcome {
    /// A `step_finish{reason:"stop"}` and a clean process exit.
    Completed,
    /// The harness said the turn failed — a typed `error` event. **Only** an
    /// explicit statement lands here.
    Failed { reason: String },
    /// No terminal arrived, but sergeant requested the kill: no conclusion
    /// about the stage, the conversation stays resumable (probe 10 measured
    /// exactly this — a truncated stream and a still-resumable session).
    InterruptedRunning,
    /// No terminal arrived and nobody asked for that: §25's ambiguity, fails
    /// closed.
    AmbiguousUnknown,
}

/// Fold one turn's stream evidence, its process exit and the interrupt bit
/// into an outcome.
///
/// **A nonzero exit is deliberately not, by itself, a stage failure.** §15's
/// load-bearing invariant is that "a backend cannot complete a stage by
/// exiting, and cannot fail one by dying" — so the only route to
/// [`TerminalOutcome::Failed`] is the harness's own typed `error` event, which
/// is precisely what the one measured nonzero-exit shape carries (probe 3:
/// exit 1 *with* the typed error). A process that dies with a nonzero status
/// and says nothing is ambiguous, and the exit code travels into the
/// ambiguous evidence where a human can act on it. The measured SIGKILL case
/// (probe 10) is the reason this matters: it produces exactly that shape.
///
/// The order of the arms is the argument: an explicit statement outranks
/// everything; a kill we asked for outranks silence; a `stop` plus a clean
/// exit is the only completion; a `stop` contradicted by a bad exit is
/// ambiguous, not a completion.
fn classify_terminal(
    acc: &TurnAccumulator,
    exit_code: Option<i32>,
    interrupted: bool,
) -> TerminalOutcome {
    if let Terminal::Error {
        name,
        message,
        reference,
    } = &acc.terminal
    {
        return TerminalOutcome::Failed {
            reason: render_typed_error(name, message, reference),
        };
    }
    if interrupted && acc.terminal != Terminal::Stopped {
        return TerminalOutcome::InterruptedRunning;
    }
    if acc.terminal == Terminal::Stopped && exit_code == Some(0) {
        return TerminalOutcome::Completed;
    }
    TerminalOutcome::AmbiguousUnknown
}

/// One settled outcome's stable, snake_case name — the string the journal
/// carries, kept out of `{:?}` so a payload consumer is not reading a derived
/// Debug rendering that changes shape whenever a field is added.
fn terminal_outcome_label(outcome: &TerminalOutcome) -> &'static str {
    match outcome {
        TerminalOutcome::Completed => "completed",
        TerminalOutcome::Failed { .. } => "failed",
        TerminalOutcome::InterruptedRunning => "interrupted_running",
        TerminalOutcome::AmbiguousUnknown => "ambiguous_unknown",
    }
}

// ------------------------------------------------------- history decoding

/// Decode one `opencode export` document into normalized events, in order
/// (§27) — the whole session, never a prefix (§15's HISTORY contract).
///
/// One `conversation.turn.ended` closes each **assistant** message, carrying
/// that message's own terminal facts (`finish`, model, provider, tokens,
/// cost). It is also where opencode's `reasoning` parts land, verbatim, in a
/// `reasoning` array: sergeant's §27 vocabulary has no kind for reasoning,
/// and minting one here would mean editing `api::SSE_EVENT_KINDS` (a core
/// file K2 puts out of scope this wave) to satisfy `tests/m6_surfaces.rs`'s
/// `t6`. Carrying the text in an existing event's payload loses nothing and
/// invents nothing; `undecoded_parts` names any other part type by its own
/// wire string, counted and never interpreted.
fn decode_export(export: &Value) -> Vec<NativeEvent> {
    let session_id = export.pointer("/info/id").cloned().unwrap_or(Value::Null);
    let mut out = Vec::new();
    let Some(messages) = export.get("messages").and_then(Value::as_array) else {
        return out;
    };
    for message in messages {
        let info = message.get("info");
        let message_id = info
            .and_then(|i| i.get("id"))
            .cloned()
            .unwrap_or(Value::Null);
        let role = info
            .and_then(|i| i.get("role"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let parts = message
            .get("parts")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        if role == "user" {
            let text = parts
                .iter()
                .filter(|part| part.get("type").and_then(Value::as_str) == Some("text"))
                .filter_map(|part| part.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("\n");
            out.push(NativeEvent {
                kind: KIND_CONVERSATION_USER.to_string(),
                payload: json!({
                    "session_id": session_id,
                    "message_id": message_id,
                    "text": text,
                }),
            });
            continue;
        }
        let mut reasoning = Vec::new();
        let mut undecoded = Vec::new();
        for part in parts {
            match part.get("type").and_then(Value::as_str).unwrap_or("") {
                "text" => out.push(NativeEvent {
                    kind: KIND_CONVERSATION_ASSISTANT_COMPLETED.to_string(),
                    payload: json!({
                        "session_id": session_id,
                        "message_id": message_id,
                        "part_id": part.get("id").cloned().unwrap_or(Value::Null),
                        "text": part.get("text").and_then(Value::as_str).unwrap_or(""),
                    }),
                }),
                "tool" => decode_export_tool(part, &mut out),
                "step-finish" => out.push(NativeEvent {
                    kind: KIND_USAGE_UPDATED.to_string(),
                    payload: json!({
                        "session_id": session_id,
                        "message_id": message_id,
                        "reason": part.get("reason").cloned().unwrap_or(Value::Null),
                        "tokens": part.get("tokens").cloned().unwrap_or(Value::Null),
                        "cost": part.get("cost").cloned().unwrap_or(Value::Null),
                    }),
                }),
                "reasoning" => {
                    reasoning.push(part.get("text").and_then(Value::as_str).unwrap_or(""));
                }
                // `step-start` carries nothing this vocabulary names; every
                // other type is counted by its own wire string.
                "step-start" => {}
                other => undecoded.push(other.to_string()),
            }
        }
        out.push(NativeEvent {
            kind: KIND_CONVERSATION_TURN_ENDED.to_string(),
            payload: json!({
                "session_id": session_id,
                "message_id": message_id,
                "source": "export",
                "finish": info.and_then(|i| i.get("finish")).cloned().unwrap_or(Value::Null),
                "model": info.and_then(|i| i.get("modelID")).cloned().unwrap_or(Value::Null),
                "provider": info.and_then(|i| i.get("providerID")).cloned().unwrap_or(Value::Null),
                "tokens": info.and_then(|i| i.get("tokens")).cloned().unwrap_or(Value::Null),
                "cost": info.and_then(|i| i.get("cost")).cloned().unwrap_or(Value::Null),
                "reasoning": reasoning,
                "undecoded_parts": undecoded,
            }),
        });
    }
    out
}

/// One export `tool` part → the same `tool.requested`/`tool.completed` pair
/// the live stream produces, so a history replay and a live turn describe the
/// same call the same way -- **if** an export `tool` part is shaped the way
/// this function assumes.
///
/// That assumption was never measured. Every field this function reads
/// (`callID`, `tool`, `state.status`, `state.input`, `state.output`,
/// `state.metadata.exit`) is copied verbatim from the *run-json stream's*
/// `tool_use` part shape (probes 2/4) on the unverified guess that `export`
/// serializes a tool call the same way. No probe in the evidence packet ever
/// exercised a tool call and then exported that session -- probe 6, the only
/// export probe, used a tool-free nonce prompt -- and the committed
/// `tests/fixtures/opencode-1.18.19-export-session.json` fixture (the only
/// real export capture in this repo) contains no `"type":"tool"` part at
/// all. `decode_export_turns_a_tool_part_into_the_same_pair_the_stream_does`
/// proves only that *this function* round-trips a hand-authored literal built
/// from the stream's shape -- it is a decoder unit test, not evidence the
/// real CLI ever emits that literal from `export`. Until a live probe closes
/// this gap, treat this branch's shape as guessed, not measured -- the same
/// honesty this module gives `config_home` and the unqualified-pin case.
fn decode_export_tool(part: &Value, out: &mut Vec<NativeEvent>) {
    let call_id = part.get("callID").cloned().unwrap_or(Value::Null);
    let tool = part.get("tool").cloned().unwrap_or(Value::Null);
    let state = part.get("state");
    let status = state
        .and_then(|s| s.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("");
    out.push(NativeEvent {
        kind: KIND_TOOL_REQUESTED.to_string(),
        payload: json!({
            "id": call_id,
            "name": tool,
            "input": state.and_then(|s| s.get("input")).cloned().unwrap_or(Value::Null),
        }),
    });
    let exit_code = state
        .and_then(|s| s.pointer("/metadata/exit"))
        .cloned()
        .unwrap_or(Value::Null);
    let output = state
        .and_then(|s| s.get("output"))
        .and_then(Value::as_str)
        .unwrap_or("");
    out.push(NativeEvent {
        kind: KIND_TOOL_COMPLETED.to_string(),
        payload: json!({
            "tool_use_id": call_id,
            "name": tool,
            "is_error": status != "completed" || exit_code.as_i64().is_some_and(|code| code != 0),
            "status": status,
            "exit_code": exit_code,
            "output_tail": truncate(output, TOOL_OUTPUT_TAIL),
        }),
    });
}

// ----------------------------------------------------------------- export

/// Run `opencode export <session_id>` and parse its stdout.
///
/// Token-free (probe 6) and therefore safe to call from RESUME, HISTORY and
/// the post-turn pin check alike. Measured shapes: a known id exits 0 with
/// the JSON on stdout and a `Exporting session: <id>` progress line on
/// stderr; an unknown id exits 1 with an empty stdout and `Error: Session not
/// found: <id>` on stderr. Both stderr shapes travel into the `Err` string —
/// the "not found" one is the durable-session evidence RESUME fails closed
/// on, so it must not be flattened to "export failed".
fn run_export(
    executable: &Path,
    cwd: &Path,
    env: &BTreeMap<String, String>,
    config_content: Option<&str>,
    session_id: &str,
) -> Result<Value, String> {
    let mut command = Command::new(executable);
    command
        .arg("export")
        .arg(session_id)
        .current_dir(cwd)
        .stdin(Stdio::null());
    apply_env(&mut command, env, config_content);
    let output = command
        .output()
        .map_err(|e| format!("cannot run {executable:?} export {session_id}: {e}"))?;
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !output.status.success() {
        return Err(format!(
            "opencode export {session_id} exited {:?}: {}",
            output.status.code(),
            truncate(stderr.trim(), 400)
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|e| {
        format!("opencode export {session_id} exited 0 but its stdout is not JSON: {e}")
    })
}

/// Apply an execution's environment plus, when configured, probe 9's config
/// channel. One function so a turn and its own `export` calls can never run
/// under two different configurations.
fn apply_env(command: &mut Command, env: &BTreeMap<String, String>, config_content: Option<&str>) {
    for (key, value) in env {
        command.env(key, value);
    }
    if let Some(content) = config_content {
        command.env(OPENCODE_CONFIG_CONTENT_ENV, content);
    }
}

// ---------------------------------------------------------------- liveness

/// Does this process's argv belong to a turn of `session_id`?
///
/// Deliberately narrow, and for the reason `claude.rs`'s
/// `cmdline_names_session` spells out at length: some argv element must be
/// exactly `-s` or `--session` and the *next* one exactly this id, and some
/// element must be exactly `run`. Never a substring match on a joined command
/// line — this project's own harness wraps commands as `bash -c '<text>'`,
/// which puts any quoted id into some process's argv without a turn running.
///
/// A **first turn carries no session id at all** (it does not exist yet), and
/// this adapter passes no `--dir`, so a first turn leaves no argv evidence
/// whatsoever. That is why there is no `SurfaceAmbiguous` attribution here
/// the way `codex.rs` has one: an over-approximating rule with nothing to
/// approximate from would be a guess, not a weaker fact.
fn argv_names_session(argv: &[String], session_id: &str) -> bool {
    let names_session = argv
        .windows(2)
        .any(|pair| (pair[0] == "-s" || pair[0] == "--session") && pair[1] == session_id);
    names_session && argv.iter().any(|arg| arg == "run")
}

/// What a process scan can say about a session's per-turn process.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Liveness {
    /// A running process's argv names this session.
    Alive(u32),
    /// No running process names this session.
    Dead,
    /// Liveness cannot be evidenced here. §25: the caller fails closed.
    Unknowable(String),
}

/// [`OpencodeBackend::session_liveness`]'s decision logic, taking the process
/// scan as a parameter (ADR 0002 D3's testability rule, the sibling adapters'
/// precedent).
fn session_liveness_among(
    session_id: &str,
    skip_pid: u32,
    processes: Option<Vec<ProcessArgv>>,
) -> Liveness {
    let Some(processes) = processes else {
        return Liveness::Unknowable(
            "process liveness needs a process-listing mechanism; none is available here"
                .to_string(),
        );
    };
    for process in processes {
        if process.pid == skip_pid {
            continue;
        }
        if argv_names_session(&process.argv, session_id) {
            return Liveness::Alive(process.pid);
        }
    }
    Liveness::Dead
}

// ------------------------------------------------------------ adapter state

/// One finished turn's outcome, kept for OBSERVE.
#[derive(Debug, Clone)]
struct TurnOutcome {
    terminal: TerminalOutcome,
    /// Checked *before* the completion branch and fatal whatever else the
    /// turn produced: a turn served by a model the human did not ask for is
    /// not a completed stage, however well it went.
    pin_mismatch: Option<String>,
    /// The pin verdict's own JSON, for the evidence string.
    pin: Value,
    steps: u32,
    text_parts: u32,
    tool_parts: u32,
    unknown_events: Vec<String>,
    unparsed_lines: u32,
    summary: Option<String>,
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

/// Adapter-side record of one execution (one durable opencode session).
#[derive(Debug)]
struct OpencodeExecution {
    /// `None` only during the narrow in-process window between spawning turn
    /// 1 and its first event arriving — never visible to a caller, since
    /// LAUNCH does not return a handle until this is `Some`.
    session_id: Option<String>,
    work_id: String,
    cwd: PathBuf,
    model: Option<String>,
    executable: PathBuf,
    env: BTreeMap<String, String>,
    config_content: Option<String>,
    /// Recorded once at LAUNCH and carried into every turn's evidence: this
    /// adapter composes no flag for out-of-surface bindings, so the fact has
    /// to be visible somewhere rather than assumed away.
    bindings_outside_cwd: Vec<PathBuf>,
    turns: u32,
    turn: TurnState,
    /// The process group id of the most recent turn, recorded at **spawn**
    /// (mirrors `codex.rs::CodexExecution::turn_pgid`, §5.5): `process_group
    /// (0)` makes the turn's direct child its own group leader, so this is
    /// that child's pid. Kept here rather than read back out of
    /// `TurnState::InFlight` at kill time, because the group outlives the
    /// leader — a bash-tool command the turn started keeps running in this
    /// group after the opencode process itself has exited and been reaped
    /// (probe 11), and that is exactly the case INTERRUPT exists to clean
    /// up. Stays valid to signal for as long as it is worth signalling:
    /// Linux keeps a pid number allocated while any process still uses it as
    /// its process-group id.
    turn_pgid: Option<u32>,
    stopped: bool,
    interrupt_requested: bool,
    reader: Option<std::thread::JoinHandle<()>>,
}

#[derive(Debug, Default)]
struct AdapterState {
    executions: BTreeMap<String, OpencodeExecution>,
}

/// Whether this daemon has re-adopted the session being classified — changes
/// nothing about the evidence, only what the human-readable reason says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Adoption {
    Unowned,
    Adopted,
}

/// One outcome of spawning turn 1, delivered from the reader thread back to
/// the LAUNCH call blocking on it.
enum FirstTurnSignal {
    /// The first event line arrived; this is the server-minted session id.
    SessionMinted(String),
    /// The process exited (or was reaped) before any event line arrived — so
    /// **no session identity exists**, nothing is resumable, and LAUNCH must
    /// say so rather than hand back a handle naming nothing.
    ExitedWithoutSession {
        exit_code: Option<i32>,
        stderr: String,
        raw_blob: Option<String>,
    },
}

/// Everything one spawn needs, snapshotted out of adapter state under the
/// lock. A named struct rather than a tuple: every field but two is a
/// `PathBuf`/`String`, so a positional tuple would let two same-typed slots
/// swap silently and still type-check.
struct SpawnPlan {
    executable: PathBuf,
    cwd: PathBuf,
    env: BTreeMap<String, String>,
    config_content: Option<String>,
    session_id: Option<String>,
    model: Option<String>,
    first_turn: bool,
    work_id: String,
    bindings_outside_cwd: Vec<PathBuf>,
    instruction_policy: Option<String>,
}

// ------------------------------------------------------------- the backend

/// The OpenCode backend.
pub struct OpencodeBackend {
    config: OpencodeConfig,
    probe_outcome: OnceLock<ProbeOutcome>,
    state: Arc<Mutex<AdapterState>>,
    sink: Mutex<Option<EventSink>>,
}

impl std::fmt::Debug for OpencodeBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpencodeBackend")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl OpencodeBackend {
    /// Build the adapter. Probing is lazy (first PROBE/PREPARE), so
    /// constructing one costs nothing on daemons that never route to it.
    pub fn new(config: OpencodeConfig) -> Self {
        Self {
            config,
            probe_outcome: OnceLock::new(),
            state: Arc::new(Mutex::new(AdapterState::default())),
            sink: Mutex::new(None),
        }
    }

    /// Install the event sink normalized events are pushed through (§27).
    pub fn set_event_sink(&self, sink: EventSink) {
        *self.sink.lock().expect("opencode sink lock") = Some(sink);
    }

    /// Execution ids this adapter currently holds state for — the diagnostic
    /// answer to "did a refused LAUNCH leave a phantom execution behind?".
    pub fn tracked_executions(&self) -> Vec<String> {
        self.lock().executions.keys().cloned().collect()
    }

    /// Run the version/grammar probe once and cache the outcome. Both gates
    /// are offline and token-free (`--version`, two `--help`s).
    fn probe_outcome(&self) -> &ProbeOutcome {
        self.probe_outcome.get_or_init(|| self.run_probe())
    }

    /// One `--help`-shaped invocation's text, **stdout and stderr
    /// concatenated**. Measured: opencode's yargs help goes to stderr and
    /// exits 0, the opposite of codex; reading both means a build that moves
    /// the text back to stdout does not become a spurious refusal.
    fn help_text(&self, args: &[&str]) -> Result<String, String> {
        let exe = &self.config.executable;
        let mut command = Command::new(exe);
        command.args(args).stdin(Stdio::null());
        apply_env(&mut command, &self.config.env, None);
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

    fn run_probe(&self) -> ProbeOutcome {
        let exe = &self.config.executable;
        let mut version_command = Command::new(exe);
        version_command.arg("--version").stdin(Stdio::null());
        apply_env(&mut version_command, &self.config.env, None);
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
                };
            }
        };
        let version_text = String::from_utf8_lossy(&version_out.stdout)
            .trim()
            .to_string();
        let Some(triple) = parse_opencode_version(&version_text) else {
            return ProbeOutcome {
                available: false,
                detail: format!(
                    "capability probe: cannot parse a version from {exe:?} --version output \
                     {version_text:?}; refusing an unmeasurable CLI"
                ),
                version: None,
                provenance: None,
            };
        };
        let canonical = format!("{}.{}.{}", triple.0, triple.1, triple.2);
        let provenance = if triple >= MEASURED_FLOOR {
            VersionProvenance::Measured
        } else {
            VersionProvenance::BelowFloor
        };

        let run_help = match self.help_text(&["run", "--help"]) {
            Ok(text) => text,
            Err(detail) => {
                return ProbeOutcome {
                    available: false,
                    detail,
                    version: Some(canonical),
                    provenance: Some(provenance),
                };
            }
        };
        let top_help = match self.help_text(&["--help"]) {
            Ok(text) => text,
            Err(detail) => {
                return ProbeOutcome {
                    available: false,
                    detail,
                    version: Some(canonical),
                    provenance: Some(provenance),
                };
            }
        };

        let mut missing_clauses: Vec<String> = Vec::new();
        let missing_flags = missing_entries(&run_help, REQUIRED_RUN_FLAGS);
        if !missing_flags.is_empty() {
            missing_clauses.push(format!("required flag(s) {}", missing_flags.join(", ")));
        }
        let missing_subcommands = missing_entries(&top_help, REQUIRED_SUBCOMMANDS);
        if !missing_subcommands.is_empty() {
            missing_clauses.push(format!(
                "and `opencode --help` is missing required subcommand(s) {}",
                missing_subcommands.join(", ")
            ));
        }
        if !missing_clauses.is_empty() {
            return ProbeOutcome {
                available: false,
                detail: format!(
                    "capability probe: {exe:?} run --help (version {version_text}) is missing {}; \
                     this launch grammar was never measured against it",
                    missing_clauses.join("; ")
                ),
                version: Some(canonical),
                provenance: Some(provenance),
            };
        }

        let version_clause = match provenance {
            VersionProvenance::Measured => format!("opencode {canonical}"),
            VersionProvenance::BelowFloor => format!(
                "opencode {canonical}; usable, but BELOW the measured floor {}.{}.{} — every \
                 behavioural claim in this adapter (event grammar, resume, export, terminal \
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
        let mut detail = format!(
            "{version_clause}; all {} required run flags and {} required subcommands present; \
             transport: {}",
            REQUIRED_RUN_FLAGS.len(),
            REQUIRED_SUBCOMMANDS.len(),
            Transport::Run.as_str(),
        );
        if self.config.config_content.is_some() {
            detail.push_str(
                "; a config document is injected per launch via OPENCODE_CONFIG_CONTENT (probe 9)",
            );
        }
        ProbeOutcome {
            available: true,
            detail,
            version: Some(canonical),
            provenance: Some(provenance),
        }
    }

    /// Resolve one execution's launch configuration from adapter config plus
    /// the profile (§14). One function, used by PREPARE, LAUNCH and RESUME
    /// alike — the rule `claude.rs::launch_config` and `codex.rs`'s namesake
    /// both follow — so a re-adopted execution cannot launch under different
    /// rules than the one it re-adopts.
    ///
    /// Two profile axes are **refused** rather than silently dropped, each
    /// naming what to use instead:
    ///
    /// - `config_home`: no opencode environment variable naming a config
    ///   *home* was measured. The measured channel is
    ///   `OPENCODE_CONFIG_CONTENT` (probe 9), which carries content, not a
    ///   directory — so honoring `config_home` would mean guessing a variable
    ///   name, and guessing wrong means the human's launch decision quietly
    ///   does nothing.
    /// - `opencode_agent`: opencode's own `--agent` is not wired in W1. An
    ///   agent applied to turn 1 must be re-applied on every resume for the
    ///   conversation to stay under it, and whether `run -s … --agent …`
    ///   does that was never measured — the same failure `codex.rs` refuses
    ///   `codex_profile` over.
    fn launch_config(&self, profile: Option<&Profile>) -> Result<LaunchConfig, BackendError> {
        if let Some(profile) = profile {
            if profile.config_home.is_some() {
                return Err(self.err_failed(format!(
                    "profile {:?}: config_home is not supported by this adapter. No opencode \
                     environment variable naming a config home has been measured, and honoring \
                     this field by guessing one would make the human's launch decision silently \
                     do nothing. The measured channel is OPENCODE_CONFIG_CONTENT, which carries \
                     a whole config document rather than a directory — set it through the \
                     backend's own `config_content`, or through the profile's `env`.",
                    profile.name
                )));
            }
            if profile.options.contains_key("opencode_agent") {
                return Err(self.err_failed(format!(
                    "profile {:?}: option opencode_agent is not supported by this adapter. \
                     opencode's `--agent` is not wired in this wave: an agent applied to the \
                     first turn has to be re-applied on every `run -s` turn for the conversation \
                     to stay under it, and that re-application is unmeasured — an agent that \
                     silently lapses on turn 2 is the adapter dropping a launch decision the \
                     human made.",
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
            backend: OPENCODE_BACKEND_NAME.to_string(),
            detail: detail.into(),
        }
    }

    fn err_unknown(&self, execution_id: &str) -> BackendError {
        BackendError::UnknownExecution {
            backend: OPENCODE_BACKEND_NAME.to_string(),
            execution_id: execution_id.to_string(),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, AdapterState> {
        self.state.lock().expect("opencode adapter state lock")
    }

    /// §25's identity rule: an execution is resolved by sergeant's id *and*
    /// the native (session) identity the handle carries.
    fn check_identity(
        &self,
        state: &AdapterState,
        handle: &ExecutionHandle,
    ) -> Result<(), BackendError> {
        let execution = state
            .executions
            .get(&handle.execution_id)
            .ok_or_else(|| self.err_unknown(&handle.execution_id))?;
        if handle.native_id.as_deref() != execution.session_id.as_deref() {
            return Err(self.err_unknown(&handle.execution_id));
        }
        Ok(())
    }

    fn emit(&self, execution_id: &str, work_id: &str, kind: &str, payload: Value) {
        let sink = self.sink.lock().expect("opencode sink lock").clone();
        if let Some(sink) = sink {
            sink(EventDraft {
                source: EventSource::new("backend", OPENCODE_BACKEND_NAME),
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

    fn session_liveness(&self, session_id: &str) -> Liveness {
        session_liveness_among(
            session_id,
            std::process::id(),
            crate::platform::process::running_processes(),
        )
    }

    /// Classify a session this daemon did not run a turn on, from liveness
    /// plus export-existence. `None` means there is no durable session to
    /// classify — the caller turns that into `UnknownExecution`.
    ///
    /// `executable`/`env` are passed in rather than read off `self.config`
    /// because a re-adopted execution may have been launched under a
    /// profile's own executable, and asking a *different* binary whether this
    /// session exists would be evidence about the wrong harness.
    fn classify_restart(
        &self,
        executable: &Path,
        env: &BTreeMap<String, String>,
        session_id: &str,
        cwd: &Path,
        adoption: Adoption,
    ) -> Option<Observation> {
        let liveness = self.session_liveness(session_id);
        let durable = run_export(
            executable,
            cwd,
            env,
            self.config.config_content.as_deref(),
            session_id,
        );
        match (liveness, durable) {
            (Liveness::Alive(pid), _) => Some(Observation {
                native: NativeState::Running,
                signal: BackendSignal::Blocked {
                    reason: format!(
                        "daemon restarted while a turn of session {session_id} was still running \
                         (pid {pid}); that turn is unowned — its output is going nowhere and \
                         sergeant did not adopt it"
                    ),
                },
                evidence: Some(format!(
                    "live turn: pid {pid} runs `opencode run … -s {session_id}` in its argv"
                )),
            }),
            (Liveness::Dead, Ok(_)) => Some(Observation {
                native: NativeState::Exited,
                signal: BackendSignal::Blocked {
                    reason: match adoption {
                        Adoption::Unowned => format!(
                            "daemon restarted mid-execution; session {session_id} is resumable \
                             (opencode export answers for it) but the in-flight turn's outcome is \
                             unknown"
                        ),
                        Adoption::Adopted => format!(
                            "session {session_id} was re-adopted after a daemon restart and is \
                             resumable (opencode export answers for it, no turn running), but the \
                             turn that was in flight when the daemon died left no outcome this \
                             daemon can read — the stage's result is unknown, not absent"
                        ),
                    },
                },
                evidence: Some(format!(
                    "no live process names session {session_id}; opencode export exited 0 for it; \
                     adopted={}",
                    adoption == Adoption::Adopted
                )),
            }),
            (Liveness::Unknowable(why), Ok(_)) => Some(Observation {
                native: NativeState::Unknown,
                signal: BackendSignal::Blocked {
                    reason: format!(
                        "daemon restarted mid-execution; session {session_id} is resumable \
                         (opencode export answers for it) but whether its turn process is still \
                         running cannot be evidenced here"
                    ),
                },
                evidence: Some(format!("process liveness unknowable: {why}")),
            }),
            (_, Err(_)) => None,
        }
    }

    /// Snapshot everything one spawn needs, under the state lock.
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
            config_content: execution.config_content.clone(),
            session_id: execution.session_id.clone(),
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

        let mut command = Command::new(&plan.executable);
        if plan.first_turn {
            command.args(first_turn_argv(plan.model.as_deref()));
        } else {
            let session_id = plan.session_id.clone().ok_or_else(|| {
                self.err_failed("cannot send: no session id recorded for this execution")
            })?;
            command.args(resume_turn_argv(plan.model.as_deref(), &session_id));
        }
        command
            .current_dir(&plan.cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        apply_env(&mut command, &plan.env, plan.config_content.as_deref());
        // §5.5 (codex.rs): every turn's tool commands run as children of this
        // process; a new process group is what lets INTERRUPT kill the whole
        // tree (probe 11 measured a bash-tool grandchild survive a plain
        // kill of the leader).
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }

        let mut child = command
            .spawn()
            .map_err(|e| self.err_failed(format!("cannot spawn {:?}: {e}", plan.executable)))?;

        let stdin = child.stdin.take();
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| self.err_failed("child stdout was not piped"))?;
        let stderr = child.stderr.take();

        // Prompt on its own thread: a CONTEXT.md larger than the pipe buffer
        // must not deadlock the spawn path (claude.rs's own deadlock fix, and
        // the reason probe 8 measured stdin delivery before this was built).
        let stdin_prompt = prompt.clone();
        std::thread::spawn(move || {
            if let Some(mut stdin) = stdin {
                let _ = stdin.write_all(stdin_prompt.as_bytes());
            }
        });
        // Stderr through a sync channel rather than a shared buffer (issue
        // #46): both pipes reach EOF at the same instant, and a reader that
        // snapshots a buffer the moment stdout closes is racing the thread
        // still filling it. Here that buffer holds probe 4's auto-rejection
        // notice, which is the entire explanation of an `error` tool part.
        let stderr_rx = stderr.map(|mut stderr| {
            let (tx, rx) = std::sync::mpsc::sync_channel::<String>(1);
            std::thread::spawn(move || {
                let text = read_bounded(&mut stderr, STREAM_MEMORY_CAP);
                let _ = tx.send(text);
            });
            rx
        });

        // Recorded here, at spawn, never derived from the child at kill
        // time -- `process_group(0)` above made this child its own group
        // leader, so its pid *is* the group's id, and that id stays the
        // right thing to signal after the child itself has exited.
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
                "session_id": plan.session_id,
                "bindings_outside_cwd": plan.bindings_outside_cwd,
                // Carried, not enforced: opencode has no measured
                // `--setting-sources` analog, so a reader can see the policy
                // this Work pinned *and* that this transport composed nothing
                // for it, instead of assuming it was applied.
                "instruction_policy_unenforced": plan.instruction_policy,
            }),
        );

        let reader = TurnReader {
            backend_state: Arc::clone(&self.state),
            sink: self.sink.lock().expect("opencode sink lock").clone(),
            data_dir: self.config.data_dir.clone(),
            execution_id: execution_id.to_string(),
            work_id: plan.work_id,
            executable: plan.executable,
            cwd: plan.cwd,
            env: plan.env,
            config_content: plan.config_content,
            model: plan.model,
            bindings_outside_cwd: plan.bindings_outside_cwd,
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

    /// LAUNCH's own spawn: fires turn 1 and blocks, bounded, for the first
    /// event line's server-minted session id. Does *not* remove adapter state
    /// on failure — the caller (`launch`) owns that, so there is one place a
    /// failed launch leaves no phantom.
    fn spawn_first_turn(
        &self,
        execution_id: &str,
        prompt: String,
        instruction_policy: Option<String>,
    ) -> Result<String, BackendError> {
        let (tx, rx) = std::sync::mpsc::sync_channel::<FirstTurnSignal>(1);
        self.spawn_turn(execution_id, prompt, instruction_policy, Some(tx))?;
        let budget = self.config.session_id_budget.unwrap_or(SESSION_ID_BUDGET);
        match rx.recv_timeout(budget) {
            Ok(FirstTurnSignal::SessionMinted(session_id)) => Ok(session_id),
            Ok(FirstTurnSignal::ExitedWithoutSession {
                exit_code,
                stderr,
                raw_blob,
            }) => Err(self.err_failed(format!(
                "opencode run exited before any event line arrived (exit_code={exit_code:?}), so \
                 no session was ever minted and nothing is resumable; stderr: {}; raw={}",
                truncate(stderr.trim(), 400),
                raw_blob.unwrap_or_else(|| "unarchived (the turn streamed nothing)".to_string()),
            ))),
            Err(_) => {
                self.kill_inflight_turn(execution_id);
                Err(self.err_failed(format!(
                    "opencode run emitted no event line within {budget:?}; the turn was killed. \
                     The session id is server-minted and arrives on the first event, so until one \
                     lands there is no identity to hand back"
                )))
            }
        }
    }

    /// Kill this execution's in-flight turn process, group and all — probe
    /// 11 measured a bash-tool grandchild survive a plain `Child::kill()` of
    /// the leader (see [`kill_turn`]). The group id is taken whatever the
    /// turn state says, the same reasoning `codex.rs::interrupt` uses: a turn
    /// that has already ended can still have left a background command
    /// running in its group.
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
}

/// Kill a turn's whole process group (mirrors `codex.rs::kill_process_group`,
/// §5.5): `SIGKILL` to the negated group id recorded at spawn, through a
/// shell rather than a `libc`/`nix` dependency for one signal (R5). Through
/// `/bin/sh -c` specifically, not by spawning `kill` as a program: `kill` is
/// a shell builtin every POSIX shell has, while `kill(1)` as an executable on
/// `PATH` is a package a host need not install, and `Command::new("kill")`
/// fails with `ENOENT` on such a host — a silent no-op if the caller drops
/// the result.
///
/// Nothing gates this on the leader being alive, and that is the whole
/// point: the group routinely outlives its leader (a command the turn
/// started in the background survives the opencode process once it has
/// exited and been reaped — exactly probe 11's finding), so the group id is
/// signalled unconditionally and `ESRCH` (an already-empty group) is
/// success, not an error to report.
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
/// for the callers that still hold a live child handle. The group goes
/// first: the child's own death must never be what decides whether the group
/// is signalled.
fn kill_turn(pgid: Option<u32>, child: Option<&Arc<Mutex<Child>>>) {
    kill_process_group(pgid);
    if let Some(child) = child {
        let _ = child.lock().expect("opencode turn child lock").kill();
    }
}

/// Everything the per-turn stdout reader thread needs. Owns ingestion end to
/// end: raw archive, normalization, the post-turn pin check, outcome
/// recording — the sibling adapters' `TurnReader`, opencode-shaped.
struct TurnReader {
    backend_state: Arc<Mutex<AdapterState>>,
    sink: Option<EventSink>,
    data_dir: PathBuf,
    execution_id: String,
    work_id: String,
    /// The four fields the post-turn `opencode export` pin check needs — the
    /// same executable, cwd, env and config document the turn itself ran
    /// under, so the check cannot read a different configuration's answer.
    executable: PathBuf,
    cwd: PathBuf,
    env: BTreeMap<String, String>,
    config_content: Option<String>,
    model: Option<String>,
    bindings_outside_cwd: Vec<PathBuf>,
    child: Arc<Mutex<Child>>,
    stderr_rx: Option<std::sync::mpsc::Receiver<String>>,
    /// Present only for turn 1: how this reader tells LAUNCH the session id
    /// (or that the process died before one existed).
    first_turn_signal: Option<SyncSender<FirstTurnSignal>>,
}

/// Read `reader` to EOF exactly as `Read::read_to_string` does, except the
/// returned `String` never grows past `cap` bytes. Every byte is still read
/// (never left sitting in the pipe — that would stall whatever is writing to
/// the other end, which is worse than losing them); bytes past `cap` are
/// simply not appended. Non-UTF-8 bytes are replaced (`str::from_utf8_lossy`)
/// rather than failing the whole read the way `read_to_string` would, which
/// only makes the capture *more* complete than before on a stream that
/// writes invalid UTF-8, not less.
///
/// A trailing marker records the loss when the cap was actually hit, so a
/// capped capture reads as "capped, N bytes missing" rather than silently
/// looking complete.
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

impl TurnReader {
    fn run(self, stdout: std::process::ChildStdout) {
        let mut raw = String::new();
        let mut raw_truncated = false;
        let mut acc = TurnAccumulator::new();
        let mut announced_session = false;

        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
            // Every line is parsed and forwarded to the event pipeline below
            // regardless of the cap — only the raw archive's completeness is
            // bounded (`STREAM_MEMORY_CAP`).
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
                    if !announced_session && let Some(id) = acc.session_id.clone() {
                        announced_session = true;
                        if let Some(execution) = self
                            .backend_state
                            .lock()
                            .expect("opencode adapter state lock")
                            .executions
                            .get_mut(&self.execution_id)
                        {
                            execution.session_id = Some(id.clone());
                        }
                        if let Some(tx) = &self.first_turn_signal {
                            let _ = tx.send(FirstTurnSignal::SessionMinted(id));
                        }
                    }
                }
                Err(_) => acc.unparsed_lines += 1,
            }
        }
        if raw_truncated {
            raw.push_str(&format!(
                "\n...<{STREAM_MEMORY_CAP}-byte in-memory cap hit; further stdout lines were \
                 still parsed and emitted above but were not archived here>\n"
            ));
        }

        // Stdout is closed; reap. The child lock is only taken after EOF so
        // INTERRUPT can always kill.
        let exit_code = self
            .child
            .lock()
            .expect("opencode turn child lock")
            .wait()
            .ok()
            .and_then(|status| status.code());

        // §20: archived before any conclusion is drawn from it, and an
        // archive failure is reported rather than swallowed — the alternative
        // is a turn whose raw capture silently does not exist.
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

        if !announced_session && let Some(tx) = &self.first_turn_signal {
            let _ = tx.send(FirstTurnSignal::ExitedWithoutSession {
                exit_code,
                stderr: stderr.clone(),
                raw_blob: raw_blob.clone(),
            });
        }

        // The post-turn pin check (probe 7): token-free, and only run when a
        // pin was actually requested — an unpinned turn has nothing to
        // verify and should not pay for a process spawn to learn that.
        let verdict = self.pin_verdict(acc.session_id.as_deref());
        let pin_mismatch = verdict.mismatch(self.model.as_deref());
        let pin = verdict.as_json(self.model.as_deref());

        let mut state = self
            .backend_state
            .lock()
            .expect("opencode adapter state lock");
        let Some(execution) = state.executions.get_mut(&self.execution_id) else {
            return;
        };
        let interrupted = execution.interrupt_requested;
        let terminal = classify_terminal(&acc, exit_code, interrupted);
        let session_id_for_event = execution.session_id.clone();
        execution.turn = TurnState::Finished(Box::new(TurnOutcome {
            terminal: terminal.clone(),
            pin_mismatch,
            pin: pin.clone(),
            steps: acc.steps,
            text_parts: acc.text_parts,
            tool_parts: acc.tool_parts,
            unknown_events: acc.unknown_events.clone(),
            unparsed_lines: acc.unparsed_lines,
            summary: acc.last_step_summary.clone(),
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
                "session_id": session_id_for_event,
                "interrupted": interrupted,
                "outcome": terminal_outcome_label(&terminal),
                "steps": acc.steps,
                "reasons": acc.reasons,
                "text_parts": acc.text_parts,
                "tool_parts": acc.tool_parts,
                "unknown_events": acc.unknown_events,
                "unparsed_lines": acc.unparsed_lines,
                "bindings_outside_cwd": self.bindings_outside_cwd,
                // The final step's token object verbatim plus the summed
                // cost — never a synthetic token sum (see
                // `TurnAccumulator::last_tokens`).
                "tokens_final_step": acc.last_tokens,
                "cost_total": acc.cost_total,
                "model_pin": pin,
                "exit_code": exit_code,
                "raw": raw_blob,
                "raw_error": raw_error,
                "stderr": truncate(stderr.trim(), 400),
            }),
        );
    }

    /// Verify the model pin against `opencode export`, when there is a pin
    /// and a session to check it against.
    fn pin_verdict(&self, session_id: Option<&str>) -> PinVerdict {
        let Some(requested) = self.model.as_deref() else {
            return PinVerdict::Unpinned;
        };
        let Some(session_id) = session_id else {
            return PinVerdict::Attempted(
                "no session id was minted for this turn, so there is nothing to export and \
                 nothing evidences which model served it"
                    .to_string(),
            );
        };
        match run_export(
            &self.executable,
            &self.cwd,
            &self.env,
            self.config_content.as_deref(),
            session_id,
        ) {
            Ok(export) => verify_model_pin(Some(requested), &export),
            Err(why) => PinVerdict::Attempted(format!(
                "opencode export could not be read for the pin check: {}",
                truncate(&why, 300)
            )),
        }
    }

    fn emit(&self, kind: &str, payload: Value) {
        if let Some(sink) = &self.sink {
            sink(EventDraft {
                source: EventSource::new("backend", OPENCODE_BACKEND_NAME),
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

impl Backend for OpencodeBackend {
    fn name(&self) -> &str {
        OPENCODE_BACKEND_NAME
    }

    /// Capabilities as measured on 1.18.19. Every `true` names a contract
    /// test in [`ADMISSION_ROWS`] (L8, made structural by
    /// [`tests::admission_rows_agree_with_capabilities`]); every `false`
    /// names its reason in the same row.
    ///
    /// `history: true` is the row that differs from every other adapter in
    /// this registry — `opencode export` is a complete, token-free, durable
    /// record, which is exactly what [`Capabilities::history`] asks for and
    /// what neither claude's per-turn stream nor codex's unmeasured rollout
    /// format could honestly supply (R4).
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            persistent_sessions: true,
            native_background: false,
            streaming: true,
            history: true,
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
    /// backend-level service to start or attach to. (W3's `opencode serve`
    /// child is *per execution* and adapter-owned, so it does not change this
    /// declaration — which is why `mod.rs`'s anticipated ENSURE-RUNTIME seam
    /// stays untouched.)
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

    /// PREPARE: refuse an unavailable probe or an impossible pin; resolve and
    /// validate the launch configuration so an impossible profile never
    /// reaches a spawn; reserve **no** native id — the session id is minted
    /// server-side on the first event, and `PreparedExecution::native_id:
    /// None` is exactly the honest answer its own contract blesses.
    fn prepare(&self, request: &StartRequest) -> Result<PreparedExecution, BackendError> {
        let probe = self.probe_outcome();
        if !probe.available {
            return Err(BackendError::Unavailable {
                backend: OPENCODE_BACKEND_NAME.to_string(),
                detail: probe.detail.clone(),
            });
        }
        if let Some(model) = &request.model {
            preflight_model_pin(model).map_err(|reason| self.err_failed(reason))?;
        }
        // Validated without keeping the result: LAUNCH re-resolves it, so the
        // two phases can never disagree about it.
        self.launch_config(request.profile.as_ref())?;
        Ok(PreparedExecution {
            execution_id: request.execution_id.clone(),
            native_id: None,
            request: request.clone(),
        })
    }

    /// LAUNCH: register the execution, spawn turn 1, and wait bounded for the
    /// first event line's session id before returning a handle at all. A
    /// failed launch leaves no phantom: adapter state is removed on every
    /// error path.
    fn launch(&self, prepared: &PreparedExecution) -> Result<ExecutionHandle, BackendError> {
        let request = &prepared.request;
        let LaunchConfig { executable, env } = self.launch_config(request.profile.as_ref())?;
        {
            let mut state = self.lock();
            state.executions.insert(
                request.execution_id.clone(),
                OpencodeExecution {
                    session_id: None,
                    work_id: request.work_id.clone(),
                    cwd: request.cwd.clone(),
                    model: request.model.clone(),
                    executable,
                    env,
                    config_content: self.config.config_content.clone(),
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
        let policy = format!("{:?}", request.instruction_policy);
        match self.spawn_first_turn(
            &request.execution_id,
            compose_launch_prompt(request),
            Some(policy),
        ) {
            Ok(session_id) => Ok(ExecutionHandle {
                execution_id: request.execution_id.clone(),
                native_id: Some(session_id),
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
                    "execution {} already has a turn in flight; an opencode session runs one turn \
                     at a time",
                    handle.execution_id
                )));
            }
        }
        self.spawn_turn(&handle.execution_id, input.to_string(), None, None)
    }

    fn observe(&self, handle: &ExecutionHandle) -> Result<Observation, BackendError> {
        let state = self.lock();
        if state.executions.contains_key(&handle.execution_id) {
            self.check_identity(&state, handle)?;
            let execution = &state.executions[&handle.execution_id];
            if matches!(execution.turn, TurnState::Adopted) {
                let session_id = execution.session_id.clone().unwrap_or_default();
                let cwd = execution.cwd.clone();
                let executable = execution.executable.clone();
                let env = execution.env.clone();
                drop(state);
                return self
                    .classify_restart(&executable, &env, &session_id, &cwd, Adoption::Adopted)
                    .ok_or_else(|| self.err_unknown(&handle.execution_id));
            }
            return Ok(observe_in_memory(execution));
        }
        drop(state);
        let session_id = handle
            .native_id
            .as_deref()
            .ok_or_else(|| self.err_unknown(&handle.execution_id))?;
        // No cwd survives a restart for an execution this adapter never
        // registered, so the export probe runs from the daemon's own data
        // dir. Opencode's session store is not cwd-scoped for lookup
        // (measured: `export` answered for a session created elsewhere), and
        // a failure here fails closed to `UnknownExecution` either way.
        let data_dir = self.config.data_dir.clone();
        self.classify_restart(
            &self.config.executable.clone(),
            &self.config.env.clone(),
            session_id,
            &data_dir,
            Adoption::Unowned,
        )
        .ok_or_else(|| self.err_unknown(&handle.execution_id))
    }

    /// INTERRUPT: stop the current turn without retiring the execution.
    /// Interrupting an execution with no turn in flight is a no-op, not an
    /// error — the goal state already holds.
    fn interrupt(&self, handle: &ExecutionHandle) -> Result<Completion, BackendError> {
        let (pgid, child) = {
            let mut state = self.lock();
            self.check_identity(&state, handle)?;
            let execution = state
                .executions
                .get_mut(&handle.execution_id)
                .expect("presence checked above");
            // The group id is taken whatever the turn state says (mirrors
            // `codex.rs::interrupt`, §5.5): a turn that has already ended can
            // still have left a background command running in its group —
            // exactly what probe 11 measured surviving a plain kill of the
            // leader — so the group kill is never gated on the direct child
            // being alive. Only the `interrupt_requested` bit, a claim about
            // a *running* turn's outcome, is still the in-flight turn's
            // alone.
            let child = match &execution.turn {
                TurnState::InFlight(child) => {
                    execution.interrupt_requested = true;
                    Some(Arc::clone(child))
                }
                TurnState::Finished(_) | TurnState::Unlaunched | TurnState::Adopted => None,
            };
            (execution.turn_pgid, child)
        };
        kill_turn(pgid, child.as_ref());
        Ok(Completion::immediate())
    }

    /// RESUME: re-adopt a durable session after a daemon restart.
    ///
    /// The evidence is `opencode export`'s own exit status (measured: 0 for a
    /// known id, 1 with `Session not found` for an unknown one) plus a
    /// process scan. Fails closed on anything else: a turn still running
    /// under a previous daemon is not a session this one may adopt, and
    /// liveness that cannot be read at all is not a licence to assume it is
    /// dead.
    fn resume(
        &self,
        handle: &ExecutionHandle,
        request: &ResumeRequest,
    ) -> Result<(), BackendError> {
        let session_id = handle
            .native_id
            .clone()
            .ok_or_else(|| self.err_unknown(&handle.execution_id))?;
        if let Some(model) = &request.model {
            preflight_model_pin(model).map_err(|reason| self.err_failed(reason))?;
        }
        {
            let state = self.lock();
            if let Some(existing) = state.executions.get(&handle.execution_id) {
                if existing.session_id.as_deref() != Some(session_id.as_str()) {
                    return Err(self.err_unknown(&handle.execution_id));
                }
                return Ok(());
            }
        }
        let LaunchConfig { executable, env } = self.launch_config(request.profile.as_ref())?;
        if run_export(
            &executable,
            &request.cwd,
            &env,
            self.config.config_content.as_deref(),
            &session_id,
        )
        .is_err()
        {
            return Err(self.err_unknown(&handle.execution_id));
        }
        match self.session_liveness(&session_id) {
            Liveness::Dead => {}
            Liveness::Alive(pid) => {
                return Err(self.err_failed(format!(
                    "cannot re-adopt session {session_id}: a turn of it is still running (pid \
                     {pid}) and this adapter does not own that process"
                )));
            }
            Liveness::Unknowable(why) => {
                return Err(self.err_failed(format!(
                    "cannot re-adopt session {session_id}: whether a turn of it is still running \
                     cannot be evidenced here ({why})"
                )));
            }
        }
        let mut state = self.lock();
        if let Some(existing) = state.executions.get(&handle.execution_id) {
            if existing.session_id.as_deref() != Some(session_id.as_str()) {
                return Err(self.err_unknown(&handle.execution_id));
            }
            return Ok(());
        }
        state.executions.insert(
            handle.execution_id.clone(),
            OpencodeExecution {
                session_id: Some(session_id),
                work_id: request.work_id.clone(),
                cwd: request.cwd.clone(),
                model: request.model.clone(),
                executable,
                env,
                config_content: self.config.config_content.clone(),
                bindings_outside_cwd: bindings_outside_cwd(&request.cwd, &request.bindings),
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

    /// HISTORY: the whole session, decoded from `opencode export` (§27).
    ///
    /// The one adapter in this registry that can answer this honestly. The
    /// answer is complete or it is a refusal — never a prefix — and
    /// completeness is not this process's memory but the harness's own
    /// durable store, so it survives a restart intact. An export that cannot
    /// be read is [`BackendError::Failed`], not `Ok(vec![])`: an empty vector
    /// from a backend that simply could not look is indistinguishable from
    /// "this conversation said nothing", which is exactly the confusion
    /// [`Capabilities::history`] exists to prevent.
    fn history(&self, handle: &ExecutionHandle) -> Result<Vec<NativeEvent>, BackendError> {
        let (executable, cwd, env, config_content, session_id) = {
            let state = self.lock();
            self.check_identity(&state, handle)?;
            let execution = &state.executions[&handle.execution_id];
            let session_id = execution
                .session_id
                .clone()
                .ok_or_else(|| self.err_unknown(&handle.execution_id))?;
            (
                execution.executable.clone(),
                execution.cwd.clone(),
                execution.env.clone(),
                execution.config_content.clone(),
                session_id,
            )
        };
        let export = run_export(
            &executable,
            &cwd,
            &env,
            config_content.as_deref(),
            &session_id,
        )
        .map_err(|why| {
            self.err_failed(format!(
                "cannot retrieve native history for session {session_id}: {why}. This adapter \
                 advertises history: true on the strength of `opencode export`, so a failure to \
                 read it is reported rather than answered with an empty list"
            ))
        })?;
        Ok(decode_export(&export))
    }

    /// STOP: kill any in-flight turn, refuse further input, hand back the
    /// reader's join as the completion's tail (issue #14/B3's rule — the
    /// engine never waits on it under the core lock).
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
fn observe_in_memory(execution: &OpencodeExecution) -> Observation {
    let session_ref = execution.session_id.as_deref().unwrap_or("<unminted>");
    match &execution.turn {
        TurnState::Unlaunched => Observation {
            native: NativeState::Unknown,
            signal: BackendSignal::Running,
            evidence: Some(format!(
                "execution registered for session {session_ref} but no turn was ever launched"
            )),
        },
        TurnState::Adopted => Observation {
            native: NativeState::Unknown,
            signal: BackendSignal::Running,
            evidence: Some(format!(
                "session {session_ref} was re-adopted after a restart; no turn of this daemon's \
                 has run on it"
            )),
        },
        TurnState::InFlight(_) => Observation {
            native: NativeState::Running,
            signal: BackendSignal::Running,
            evidence: Some(format!(
                "turn {} in flight on session {session_ref}",
                execution.turns
            )),
        },
        TurnState::Finished(outcome) => {
            // Checked before the completion branch, whatever the turn
            // otherwise produced: a substituted model outranks a successful
            // turn, because the turn that succeeded is not the turn the human
            // asked for.
            if let Some(mismatch) = &outcome.pin_mismatch {
                return Observation {
                    native: NativeState::Exited,
                    signal: BackendSignal::Failed {
                        reason: mismatch.clone(),
                    },
                    evidence: Some(format!(
                        "session_id={session_ref}; model_pin={}; raw={}",
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
                        "session_id={session_ref}; model_pin={}; raw={}; steps={}, \
                         text_parts={}, tool_parts={}, unknown_events={:?}, unparsed_lines={}",
                        outcome.pin,
                        outcome.raw_evidence(),
                        outcome.steps,
                        outcome.text_parts,
                        outcome.tool_parts,
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
                        "session_id={session_ref}; exit_code={:?}; raw={}; stderr: {}",
                        outcome.exit_code,
                        outcome.raw_evidence(),
                        truncate(outcome.stderr.trim(), 400)
                    )),
                },
                TerminalOutcome::InterruptedRunning => Observation {
                    native: NativeState::Exited,
                    signal: BackendSignal::Running,
                    evidence: Some(format!(
                        "turn interrupted by request; session {session_ref} remains resumable \
                         (probe 10: an uncleanly killed turn leaves the session store intact); \
                         raw={}",
                        outcome.raw_evidence()
                    )),
                },
                // §25's ambiguity, failing closed: `native: Unknown` blocks
                // the Work rather than letting a stage be completed or failed
                // by a process that merely stopped talking.
                TerminalOutcome::AmbiguousUnknown => Observation {
                    native: NativeState::Unknown,
                    signal: BackendSignal::Running,
                    evidence: Some(format!(
                        "turn process ended with no step_finish reason \"stop\" and no typed \
                         error (session {session_ref}); exit_code={:?}; last_error={:?}; raw={}; \
                         stderr: {}",
                        outcome.exit_code,
                        outcome.last_error,
                        outcome.raw_evidence(),
                        truncate(outcome.stderr.trim(), 400)
                    )),
                },
            }
        }
    }
}

/// A private per-module copy — the sibling adapters' own precedent (each
/// owner keeps its own; `runtime/graph.rs` already has a third).
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

    const MINIMAL_TURN: &str =
        include_str!("../../tests/fixtures/opencode-1.18.19-minimal-turn.jsonl");
    const TOOL_USE_TURN: &str =
        include_str!("../../tests/fixtures/opencode-1.18.19-tool-use.jsonl");
    const AUTOREJECT_TURN: &str =
        include_str!("../../tests/fixtures/opencode-1.18.19-permission-autoreject.jsonl");
    const SIGKILL_TRUNCATED: &str =
        include_str!("../../tests/fixtures/opencode-1.18.19-sigkill-truncated.jsonl");
    const UNKNOWN_MODEL_ERROR: &str =
        include_str!("../../tests/fixtures/opencode-1.18.19-error-unknown-model.jsonl");
    const EXPORT_SESSION: &str =
        include_str!("../../tests/fixtures/opencode-1.18.19-export-session.json");

    /// Replay a fixture through the decoder, exactly as `TurnReader::run`
    /// does, and hand back the accumulator plus every event it produced.
    fn replay(fixture: &str) -> (TurnAccumulator, Vec<NativeEvent>) {
        let mut acc = TurnAccumulator::new();
        let mut events = Vec::new();
        for line in fixture.lines().filter(|line| !line.trim().is_empty()) {
            match serde_json::from_str::<Value>(line) {
                Ok(value) => events.extend(acc.ingest_line(&value)),
                Err(_) => acc.unparsed_lines += 1,
            }
        }
        (acc, events)
    }

    fn kinds(events: &[NativeEvent]) -> Vec<&str> {
        events.iter().map(|e| e.kind.as_str()).collect()
    }

    // -------------------------------------------------------- version, R1

    #[test]
    fn parse_opencode_version_pins_the_measured_shape() {
        assert_eq!(parse_opencode_version("1.18.19\n"), Some((1, 18, 19)));
        assert_eq!(parse_opencode_version("1.18.21"), Some((1, 18, 21)));
        assert_eq!(
            parse_opencode_version("1.19.0-rc.1"),
            Some((1, 19, 0)),
            "patch parsed up to the first non-digit"
        );
        assert_eq!(
            parse_opencode_version("1.18.19 abc1234"),
            Some((1, 18, 19)),
            "a suffixed build token does not break the parse"
        );
        assert_eq!(parse_opencode_version("nightly"), None);
        assert_eq!(
            parse_opencode_version("1.18"),
            None,
            "two segments are not a version"
        );
        assert_eq!(parse_opencode_version(""), None);
    }

    #[test]
    fn the_measured_floor_is_the_probed_version() {
        assert_eq!(MEASURED_FLOOR, (1, 18, 19));
        assert!(parse_opencode_version("1.18.18").unwrap() < MEASURED_FLOOR);
        assert!(parse_opencode_version("1.18.19").unwrap() >= MEASURED_FLOOR);
    }

    #[test]
    fn missing_entries_names_exactly_the_absent_ones() {
        assert!(missing_entries("--format --model --session", REQUIRED_RUN_FLAGS).is_empty());
        assert_eq!(
            missing_entries("--format --model", REQUIRED_RUN_FLAGS),
            vec!["--session"]
        );
        assert_eq!(
            missing_entries("opencode run", REQUIRED_SUBCOMMANDS),
            vec!["export"]
        );
    }

    // ------------------------------------------------------- launch grammar

    #[test]
    fn first_turn_argv_carries_the_measured_shape() {
        assert_eq!(first_turn_argv(None), vec!["run", "--format", "json"]);
        assert_eq!(
            first_turn_argv(Some("opencode/big-pickle")),
            vec!["run", "--format", "json", "-m", "opencode/big-pickle"]
        );
        // `--auto` would auto-approve every non-denied permission; probe 4
        // measured that `run` cannot hang without it, so it is never composed.
        for absent in ["--auto", "--dir", "--agent", "--share", "-i", "--attach"] {
            assert!(
                !first_turn_argv(Some("opencode/big-pickle")).contains(&absent.to_string()),
                "{absent} must never appear on turn 1"
            );
        }
    }

    #[test]
    fn resume_turn_argv_adds_the_session_and_keeps_the_pin() {
        let argv = resume_turn_argv(Some("opencode/big-pickle"), "ses_abc");
        assert_eq!(
            argv,
            vec![
                "run",
                "--format",
                "json",
                "-m",
                "opencode/big-pickle",
                "-s",
                "ses_abc"
            ]
        );
        let s_index = argv.iter().position(|arg| arg == "-s").unwrap();
        assert_eq!(argv[s_index + 1], "ses_abc");
        assert!(
            resume_turn_argv(None, "ses_abc").contains(&"-s".to_string()),
            "an unpinned resume still names its session"
        );
    }

    #[test]
    fn the_environment_contract_matches_claudes_today() {
        assert_eq!(
            ENVIRONMENT_CONTRACT, CLAUDE_ENVIRONMENT_CONTRACT,
            "the two texts are copied, not shared; a divergence must be a decision"
        );
    }

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
    fn compose_launch_prompt_orders_its_sections() {
        let prompt = compose_launch_prompt(&prompt_request(vec![]));
        let sections: Vec<&str> = prompt.split("\n\n").collect();
        assert_eq!(
            sections.len(),
            4,
            "no bindings -> no mutation-surface claim"
        );
        assert_eq!(sections[0], EXECUTION_MODEL_CONTRACT);
        assert_eq!(sections[1], ENVIRONMENT_CONTRACT);
        assert_eq!(sections[2], "do the thing");
        assert_eq!(sections[3], "context body");

        let with_bindings = compose_launch_prompt(&prompt_request(vec![BindingSummary {
            repository: "solo".to_string(),
            worktree_path: PathBuf::from("/work/solo"),
            work_branch: "sergeant/w1".to_string(),
            base_branch: Some("main".to_string()),
            base_sha: "a".repeat(40),
        }]));
        let sections: Vec<&str> = with_bindings.split("\n\n").collect();
        assert_eq!(sections.len(), 5);
        assert!(sections[2].starts_with("Mutation surface:"));
        assert!(sections[2].contains("solo: /work/solo (branch sergeant/w1, cut from main at"));
    }

    #[test]
    fn compose_launch_prompt_names_a_detached_admission() {
        let prompt = compose_launch_prompt(&prompt_request(vec![BindingSummary {
            repository: "solo".to_string(),
            worktree_path: PathBuf::from("/work/solo"),
            work_branch: "sergeant/w1".to_string(),
            base_branch: None,
            base_sha: "b".repeat(40),
        }]));
        assert!(prompt.contains("no named base branch (detached admission)"));
    }

    #[test]
    fn bindings_outside_cwd_reports_only_what_escapes_the_surface() {
        let inside = BindingSummary {
            repository: "a".to_string(),
            worktree_path: PathBuf::from("/work/a"),
            work_branch: "b".to_string(),
            base_branch: None,
            base_sha: "c".repeat(40),
        };
        let outside = BindingSummary {
            worktree_path: PathBuf::from("/elsewhere/b"),
            ..inside.clone()
        };
        assert!(bindings_outside_cwd(Path::new("/work"), std::slice::from_ref(&inside)).is_empty());
        assert_eq!(
            bindings_outside_cwd(Path::new("/work"), &[inside, outside]),
            vec![PathBuf::from("/elsewhere/b")]
        );
    }

    #[test]
    fn preflight_refuses_only_an_empty_pin() {
        assert!(preflight_model_pin("opencode/big-pickle").is_ok());
        // Inverted from claude's rule on purpose: a slash is opencode's own
        // documented `-m` grammar, not a refusal trigger.
        assert!(preflight_model_pin("big-pickle").is_ok());
        assert!(preflight_model_pin("   ").is_err());
    }

    // ------------------------------------------------------------- decoding

    #[test]
    fn the_minimal_turn_fixture_decodes_to_text_and_usage() {
        let (acc, events) = replay(MINIMAL_TURN);
        assert_eq!(
            acc.session_id.as_deref(),
            Some("ses_fd1bbfff3ffek3hxLwVl6yk5FW"),
            "the session id is learned from the first event"
        );
        assert_eq!(
            kinds(&events),
            vec!["conversation.assistant.completed", "usage.updated"]
        );
        assert_eq!(acc.steps, 1);
        assert_eq!(acc.text_parts, 1);
        assert_eq!(acc.tool_parts, 0);
        assert_eq!(acc.terminal, Terminal::Stopped);
        assert_eq!(acc.last_step_summary.as_deref(), Some("pong"));
        assert_eq!(events[0].payload["text"], "pong");
        assert_eq!(events[1].payload["tokens"]["total"], 7762);
        assert_eq!(events[1].payload["reason"], "stop");
        assert!(acc.unknown_events.is_empty());
        assert_eq!(acc.unparsed_lines, 0);
    }

    #[test]
    fn the_tool_use_fixture_produces_exactly_one_requested_completed_pair() {
        let (acc, events) = replay(TOOL_USE_TURN);
        assert_eq!(
            kinds(&events),
            vec![
                "tool.requested",
                "tool.completed",
                "usage.updated",
                "conversation.assistant.completed",
                "usage.updated",
            ]
        );
        assert_eq!(acc.steps, 2);
        assert_eq!(acc.tool_parts, 1);
        assert_eq!(acc.text_parts, 1);
        assert_eq!(acc.reasons, vec!["tool-calls", "stop"]);
        assert_eq!(acc.terminal, Terminal::Stopped);
        let requested = &events[0].payload;
        assert_eq!(requested["name"], "bash");
        assert_eq!(requested["id"], "call_3d212511f9314d519bcac1fa");
        assert_eq!(requested["input"]["command"], "echo sergeant-probe-42");
        let completed = &events[1].payload;
        assert_eq!(completed["is_error"], false);
        assert_eq!(completed["exit_code"], 0);
        assert_eq!(completed["status"], "completed");
        assert_eq!(completed["truncated"], false);
        assert_eq!(completed["output_tail"], "sergeant-probe-42\n");
        assert_eq!(
            acc.last_step_summary.as_deref(),
            Some("sergeant-probe-42"),
            "the summary is the final step's text, not the narration before the tool call"
        );
    }

    /// The narration rule, as an assertion: prose can never mint a tool
    /// event, because the only branch that produces one reads a `tool_use`
    /// part. The truncated SIGKILL fixture is the natural control — it
    /// carries a text part that *describes* running a command, and no
    /// `tool_use` line at all.
    #[test]
    fn a_text_part_that_narrates_a_command_produces_no_tool_event() {
        let (acc, events) = replay(SIGKILL_TRUNCATED);
        assert!(
            events[0].payload["text"]
                .as_str()
                .unwrap()
                .contains("running the sleep command"),
            "the fixture must actually narrate a command for this to be a control"
        );
        assert!(
            !kinds(&events).iter().any(|kind| kind.starts_with("tool.")),
            "narration is transcript content, never tool evidence"
        );
        assert_eq!(acc.tool_parts, 0);
        assert_eq!(acc.terminal, Terminal::None);
    }

    #[test]
    fn an_auto_rejected_tool_call_decodes_as_an_errored_tool_completion() {
        let (acc, events) = replay(AUTOREJECT_TURN);
        assert_eq!(
            kinds(&events),
            vec!["tool.requested", "tool.completed", "usage.updated"]
        );
        let completed = &events[1].payload;
        assert_eq!(completed["is_error"], true);
        assert_eq!(completed["status"], "error");
        assert_eq!(
            completed["exit_code"],
            Value::Null,
            "an auto-rejected call never ran, so it has no exit code to report"
        );
        assert!(
            completed["error"]
                .as_str()
                .unwrap()
                .contains("rejected permission")
        );
        assert_eq!(acc.reasons, vec!["tool-calls"]);
        assert_eq!(
            acc.terminal,
            Terminal::None,
            "probe 4's run ends with reason tool-calls, never stop"
        );
    }

    #[test]
    fn the_typed_error_fixture_is_a_terminal_failure() {
        let (acc, events) = replay(UNKNOWN_MODEL_ERROR);
        assert_eq!(kinds(&events), vec!["conversation.turn.harness_error"]);
        assert_eq!(events[0].payload["phase"], "typed_error");
        assert_eq!(events[0].payload["name"], "UnknownError");
        assert_eq!(events[0].payload["ref"], "err_a2c9f0ac");
        assert!(matches!(acc.terminal, Terminal::Error { .. }));
        assert_eq!(
            acc.session_id.as_deref(),
            Some("ses_fd1bb2e8cffeYeYyK0tBKW39Ff"),
            "a session is minted even for a failing run (probe 3)"
        );
    }

    #[test]
    fn an_unrecognized_envelope_type_is_counted_never_decoded() {
        let (acc, events) = replay(
            "{\"type\":\"snapshot\",\"sessionID\":\"ses_x\",\"part\":{}}\nnot json at all\n",
        );
        assert!(events.is_empty());
        assert_eq!(acc.unknown_events, vec!["snapshot"]);
        assert_eq!(acc.unparsed_lines, 1);
        assert_eq!(acc.session_id.as_deref(), Some("ses_x"));
    }

    // ------------------------------------------------------------ terminals

    #[test]
    fn classify_terminal_maps_every_measured_shape() {
        let (stopped, _) = replay(MINIMAL_TURN);
        assert_eq!(
            classify_terminal(&stopped, Some(0), false),
            TerminalOutcome::Completed
        );

        let (errored, _) = replay(UNKNOWN_MODEL_ERROR);
        match classify_terminal(&errored, Some(1), false) {
            TerminalOutcome::Failed { reason } => {
                assert!(reason.contains("UnknownError"));
                assert!(reason.contains("err_a2c9f0ac"));
            }
            other => panic!("expected Failed, got {other:?}"),
        }

        let (truncated, _) = replay(SIGKILL_TRUNCATED);
        assert_eq!(
            classify_terminal(&truncated, None, true),
            TerminalOutcome::InterruptedRunning,
            "a kill we asked for is not a stage verdict; the session stays resumable"
        );
        assert_eq!(
            classify_terminal(&truncated, None, false),
            TerminalOutcome::AmbiguousUnknown,
            "§15: a backend cannot fail a stage by dying"
        );
    }

    /// The invariant that keeps §15 honest: a nonzero exit with nothing said
    /// is ambiguity, never a failure — otherwise a harness could fail a stage
    /// merely by dying badly.
    #[test]
    fn a_nonzero_exit_without_a_typed_error_is_ambiguous_not_failed() {
        let (silent, _) = replay(SIGKILL_TRUNCATED);
        assert_eq!(
            classify_terminal(&silent, Some(137), false),
            TerminalOutcome::AmbiguousUnknown
        );
        // And the converse: a `stop` the process then contradicted with a bad
        // exit is not a completion either.
        let (stopped, _) = replay(MINIMAL_TURN);
        assert_eq!(
            classify_terminal(&stopped, Some(1), false),
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

    // ----------------------------------------------------------- model pin

    fn export_fixture() -> Value {
        serde_json::from_str(EXPORT_SESSION).expect("export fixture is valid JSON")
    }

    #[test]
    fn verify_model_pin_reads_the_served_model_out_of_export() {
        let export = export_fixture();
        assert_eq!(verify_model_pin(None, &export), PinVerdict::Unpinned);
        assert_eq!(
            verify_model_pin(Some("opencode/big-pickle"), &export),
            PinVerdict::Honored("opencode/big-pickle".to_string()),
            "the slash-joined request form is compared against the split served form (probe 7)"
        );
        assert_eq!(
            verify_model_pin(Some("big-pickle"), &export),
            PinVerdict::Honored("opencode/big-pickle".to_string()),
            "an unqualified pin is checked against modelID alone"
        );
        assert_eq!(
            verify_model_pin(Some("opencode/hy3-free"), &export),
            PinVerdict::Substituted("opencode/big-pickle".to_string())
        );
        assert_eq!(
            verify_model_pin(Some("other/big-pickle"), &export),
            PinVerdict::Substituted("opencode/big-pickle".to_string()),
            "a provider mismatch is a substitution too"
        );
        match verify_model_pin(Some("x"), &json!({"messages": []})) {
            PinVerdict::Attempted(detail) => assert!(detail.contains("no assistant message")),
            other => panic!("expected Attempted, got {other:?}"),
        }
    }

    #[test]
    fn only_a_substitution_is_a_stage_failure() {
        let requested = Some("opencode/big-pickle");
        assert!(PinVerdict::Unpinned.mismatch(None).is_none());
        assert!(
            PinVerdict::Honored("opencode/big-pickle".to_string())
                .mismatch(requested)
                .is_none()
        );
        assert!(
            PinVerdict::Attempted("no export".to_string())
                .mismatch(requested)
                .is_none(),
            "missing evidence is not a verdict about the Work"
        );
        let mismatch = PinVerdict::Substituted("opencode/hy3-free".to_string())
            .mismatch(requested)
            .expect("a substitution is fatal");
        assert!(mismatch.contains("opencode/big-pickle"));
        assert!(mismatch.contains("opencode/hy3-free"));
    }

    #[test]
    fn pin_verdicts_render_their_own_evidence() {
        let requested = Some("opencode/big-pickle");
        assert_eq!(
            PinVerdict::Honored("opencode/big-pickle".to_string()).as_json(requested)["verdict"],
            "honored"
        );
        assert_eq!(
            PinVerdict::Substituted("x/y".to_string()).as_json(requested)["ran"],
            "x/y"
        );
        assert_eq!(PinVerdict::Unpinned.as_json(None)["verdict"], "unpinned");
    }

    // ------------------------------------------------------------- history

    #[test]
    fn decode_export_covers_every_message_and_part_of_the_fixture() {
        let events = decode_export(&export_fixture());
        assert_eq!(
            kinds(&events),
            vec![
                "conversation.user",
                "conversation.assistant.completed",
                "usage.updated",
                "conversation.turn.ended",
                "conversation.user",
                "conversation.assistant.completed",
                "usage.updated",
                "conversation.turn.ended",
            ],
            "four messages, in order, with each assistant message closed by a turn.ended"
        );
        assert!(
            events[0].payload["text"]
                .as_str()
                .unwrap()
                .contains("zebra-7134")
        );
        assert_eq!(events[1].payload["text"], "stored.");
        let first_end = &events[3].payload;
        assert_eq!(first_end["finish"], "stop");
        assert_eq!(first_end["model"], "big-pickle");
        assert_eq!(first_end["provider"], "opencode");
        assert_eq!(first_end["tokens"]["total"], 7772);
        assert_eq!(
            first_end["reasoning"].as_array().unwrap().len(),
            1,
            "opencode's reasoning parts have no §27 kind, so they are preserved verbatim here \
             rather than dropped"
        );
        assert!(
            first_end["undecoded_parts"].as_array().unwrap().is_empty(),
            "every part type in this fixture is accounted for"
        );
        assert_eq!(events[5].payload["text"], "zebra-7134");
        assert_eq!(
            events[0].payload["session_id"],
            "ses_fd1baa02dffeRqK8HTSxGEhQDD"
        );
    }

    #[test]
    fn decode_export_of_an_empty_document_is_empty_not_a_panic() {
        assert!(decode_export(&json!({})).is_empty());
        assert!(decode_export(&json!({"messages": []})).is_empty());
    }

    #[test]
    fn decode_export_names_a_part_type_it_does_not_know() {
        let events = decode_export(&json!({
            "info": {"id": "ses_x"},
            "messages": [{
                "info": {"role": "assistant", "id": "msg_x", "finish": "stop"},
                "parts": [{"type": "patch", "id": "prt_x"}],
            }],
        }));
        assert_eq!(kinds(&events), vec!["conversation.turn.ended"]);
        assert_eq!(events[0].payload["undecoded_parts"][0], "patch");
    }

    #[test]
    fn decode_export_turns_a_tool_part_into_the_same_pair_the_stream_does() {
        let events = decode_export(&json!({
            "info": {"id": "ses_x"},
            "messages": [{
                "info": {"role": "assistant", "id": "msg_x"},
                "parts": [{
                    "type": "tool",
                    "tool": "bash",
                    "callID": "call_1",
                    "state": {
                        "status": "completed",
                        "input": {"command": "true"},
                        "output": "ok\n",
                        "metadata": {"exit": 0},
                    },
                }],
            }],
        }));
        assert_eq!(
            kinds(&events),
            vec![
                "tool.requested",
                "tool.completed",
                "conversation.turn.ended"
            ]
        );
        assert_eq!(events[1].payload["is_error"], false);
        assert_eq!(events[1].payload["output_tail"], "ok\n");
    }

    // ------------------------------------------------------------ liveness

    #[test]
    fn argv_names_session_matches_only_the_launch_grammar() {
        let live = vec![
            "opencode".to_string(),
            "run".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "-s".to_string(),
            "ses_abc".to_string(),
        ];
        assert!(argv_names_session(&live, "ses_abc"));
        assert!(!argv_names_session(&live, "ses_other"));
        // A wrapper that merely quotes the id in one argv element is not a
        // running turn (claude.rs's own false-positive lesson).
        let wrapper = vec![
            "bash".to_string(),
            "-c".to_string(),
            "opencode run -s ses_abc".to_string(),
        ];
        assert!(!argv_names_session(&wrapper, "ses_abc"));
        // `-s ses_abc` without the `run` subcommand is some other verb.
        let other_verb = vec![
            "opencode".to_string(),
            "export".to_string(),
            "-s".to_string(),
            "ses_abc".to_string(),
        ];
        assert!(!argv_names_session(&other_verb, "ses_abc"));
    }

    #[test]
    fn session_liveness_among_is_fail_closed_without_a_process_list() {
        match session_liveness_among("ses_abc", 1, None) {
            Liveness::Unknowable(why) => assert!(why.contains("process-listing")),
            other => panic!("expected Unknowable, got {other:?}"),
        }
        assert_eq!(
            session_liveness_among("ses_abc", 1, Some(Vec::new())),
            Liveness::Dead
        );
        let processes = vec![
            ProcessArgv {
                pid: 1,
                argv: vec!["run".to_string(), "-s".to_string(), "ses_abc".to_string()],
            },
            ProcessArgv {
                pid: 2,
                argv: vec!["run".to_string(), "-s".to_string(), "ses_abc".to_string()],
            },
        ];
        assert_eq!(
            session_liveness_among("ses_abc", 1, Some(processes)),
            Liveness::Alive(2),
            "our own pid is skipped"
        );
    }

    // ------------------------------------------------------ admission rows

    /// L8, made structural: every `true` in `capabilities()` has a row, every
    /// row agrees with the contract, and every claimed row names a real test.
    #[test]
    fn admission_rows_agree_with_capabilities() {
        let backend = OpencodeBackend::new(OpencodeConfig::new(Path::new("/nonexistent")));
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
            let row = ADMISSION_ROWS
                .iter()
                .find(|row| row.capability == flag && row.transport == Transport::Run)
                .unwrap_or_else(|| panic!("no ADMISSION_ROWS entry for {flag}"));
            assert_eq!(
                row.claimed, claimed_by_contract,
                "{flag}: capabilities() says {claimed_by_contract}, the row says {}",
                row.claimed
            );
            if row.claimed {
                assert!(
                    !row.admission_test.is_empty(),
                    "{flag}: claimed true with no admission_test named"
                );
            }
        }
        // The two rows with no v1 boolean hold the same invariant.
        for capability in ["config_injection", "non_blocking_run"] {
            let row = ADMISSION_ROWS
                .iter()
                .find(|row| row.capability == capability)
                .unwrap_or_else(|| panic!("no ADMISSION_ROWS entry for {capability}"));
            assert!(row.claimed);
            assert!(!row.admission_test.is_empty());
        }
        assert_eq!(
            ADMISSION_ROWS.len(),
            flags.len() + 2,
            "every row is one of the thirteen v1 flags or one of the two adapter-local rows — an \
             unlisted row is a claim nothing checks"
        );
    }

    /// `Evidence`'s own definitions are the contract: `LiveMeasured` means
    /// "driven against the real, installed harness". This suite's naming
    /// convention makes that checkable — every live test in
    /// `tests/opencode_backend.rs` is named `live_*` and nothing else is — so
    /// a `claimed: true` row is only internally consistent if the two agree.
    #[test]
    fn a_claimed_row_naming_a_live_test_is_labelled_live_measured() {
        for row in ADMISSION_ROWS.iter().filter(|row| row.claimed) {
            assert_eq!(
                row.admission_test.starts_with("live_"),
                row.evidence == Evidence::LiveMeasured,
                "{}: evidence {:?} disagrees with its admission test {:?}",
                row.capability,
                row.evidence,
                row.admission_test
            );
        }
    }

    #[test]
    fn the_rendered_table_states_the_stability_fact_once() {
        let table = render_admission_rows();
        assert!(table.starts_with("stability (all rows): opencode publishes no"));
        assert!(table.contains("provenance, not a gate (R1)"));
        for row in ADMISSION_ROWS {
            assert!(
                table.contains(row.capability),
                "{} is missing from the rendered table",
                row.capability
            );
        }
    }

    // --------------------------------------------------------------- misc

    #[test]
    fn the_backend_declares_its_name_and_scope() {
        let backend = OpencodeBackend::new(OpencodeConfig::new(Path::new("/nonexistent")));
        assert_eq!(backend.name(), OPENCODE_BACKEND_NAME);
        assert_eq!(backend.runtime_scope(), RuntimeScope::PerExecution);
        assert!(backend.tracked_executions().is_empty());
    }

    #[test]
    fn truncate_is_char_boundary_safe() {
        assert_eq!(truncate("abcdef", 3), "abc");
        assert_eq!(truncate("abc", 10), "abc");
        assert_eq!(truncate("héllo", 2), "hé");
    }
}
