# Sprint plan — Sergeant speaks OpenCode (2026-08-23)

Owner-commissioned 2026-08-23: build the OpenCode adapter, Claude and
Codex feature parity at minimum, exceed it where opencode measurably
does better. This plan follows ADR 0020's own instruction that its
pattern is "the pattern any future adapter (opencode, goose, …) should
copy rather than re-derive" — where a section below matches the codex
plan, that is the point, not an oversight.

**Spec sources (authority order):**

1. Owner kickoff rulings 2026-08-23 (this session, on the record):
   - **K1 — dev-test pin is `opencode/big-pickle`** (free Zen tier,
     zero cost, no stored credential measured needed). The adapter
     takes model as a parameter and hardcodes nothing; every live test
     and proof Work pins big-pickle.
   - **K2 — scope is the adapter; no core changes.** The §15 contract,
     the router, the engine are untouched (ADR 0020 R3 re-affirmed).
     The owner offered a carve-out for cheap dependencies if the serve
     transport needed them; **the carve-out goes unused** — opencode's
     serve transport is HTTP + SSE, not WebSocket, and `reqwest`
     0.13 (rustls, json) is already a direct dependency that
     `ApiClient` already uses for streaming SSE reads
     (`src/api.rs:4300-4630`). W3 is an R5 (installed-dependency) rung
     citation, zero new crates, no `Cargo.toml` change.
   - **K3 — version target 0.2.2** ("sergeant speaks opencode",
     patch-to-minor sizing per the 0.2.0 precedent; owner named 0.2.2
     explicitly).
2. Carried rulings from ADR 0020, applying without re-litigation:
   **R1** (version floors are provenance, not gates — now repo-wide
   stance, ADR 0020's own words), **R2** (harness/backend are shipped,
   user-composable axes), **R3** (= K2), **R4** (parity is the floor,
   not the ceiling; a measured better-than-claude capability is used
   and the record amended with a dated entry, `[[repo-is-a-snapshot]]`).
3. The probe evidence packet
   (`sergeant-rs-workspace`:
   `knowledge/evidence/opencode-adapter-probes-2026-08-23.md`) — every
   opencode behavior claim below is measured there at opencode 1.18.19
   unless marked **doc-claimed**.
4. Shipped code: `src/backend/mod.rs` (the §15 v1 contract),
   `codex.rs`/`codex_appserver.rs` (the AdmissionRow ledger, the
   one-decoder-two-transports seam, the live-test gating), `claude.rs`
   (the ask/pin-verification patterns), ADR 0020.

**Protocol** (unchanged from codex): integration branch
`integration/opencode`, draft head PR carrying this plan, wave branches
`opencode/w<N>-<slug>` in `/var/tmp/opencode-impl/` worktrees (warm
base build first; tmpfs `/tmp` never — #70). Per wave: spec → implement
(TDD, DataDir guards, R-S0-12) → 4-axis blind panel + refuters (default
refuted) → fixer on confirmed findings only → wave PR → CI by SHA →
merge to integration. Sonnet subagents, opus where earned, Fable never
below the captain. Rung/ruling citations in every PR body. **Dev-time
opencode invocations pin `-m opencode/big-pickle`** (K1).

## Measured facts this plan builds on (packet citations)

- `opencode run --format json` emits NDJSON, one
  `{type, timestamp, sessionID, part}` per line: `step_start` →
  (`tool_use` | `text`)* → `step_finish` {reason: "stop"|"tool-calls",
  tokens: {total/input/output/reasoning/cache}, cost}. Probe 1–2.
- `tool_use.part.state` carries tool name, callID, full input, output,
  `metadata.exit`, `metadata.truncated`, timing — structured items are
  evidence; **the narration rule is enforceable** (probe 2).
- **`run` is non-blocking by construction**: a permission rule
  resolving to `ask` auto-rejects in non-interactive mode (stderr
  notice + `state.status:"error"` on the tool part, exit 0) — it
  cannot hang a stage (probe 4). `--auto` exists but is not this
  adapter's default posture.
- Typed terminal error: `{"type":"error", error:{name, data:{message,
  ref}}}`, exit 1 (probe 3, invalid model).
- Durable resume: `opencode run -s <sessionID>` continues a session
  from a separate process, nonce continuity verified (probe 5).
  sessionID is **server-minted**, learned from the first event —
  opposite of claude's client-minted `--session-id`; the adapter's
  restart-reconciliation argv evidence must key on `-s <id>` presence
  (first turns are adoptable only after the first event line lands).
- **Token-free complete history**: `opencode export <sessionID>`
  returns `{info, messages:[{info:{role,…}, parts:[…]}]}` including
  `reasoning` parts (probe 6).
- No native OS sandbox exists (docs exhaustively silent; permission
  config and tool disables are the only controls). Sergeant's
  observation layer stays the source of truth; unlike codex there is
  no enforcement belt to claim, so no NORTH-STAR amendment is needed —
  amendment 4's "an adapter MAY use its harness's native enforcement"
  simply has nothing to bind here.
- Upstream surface is **not contractually stable** (no documented
  breaking-change policy; repo moved sst→anomalyco; Part vocabulary
  churning). MEASURED_FLOOR = **1.18.19**, provenance only (R1).
- Binary location on Cerberus: `~/.opencode/bin/opencode`, absent from
  a non-interactive shell's PATH and from `harness.rs`'s
  `toolchain_path_dirs` (measured `command not found`).

## Waves

### W1 — `opencode/run` adapter core (opus spec)

`src/backend/opencode.rs` implements §15 on the run-json transport,
mirroring codex.rs's shape (adapter-local `Evidence`/`Stability`/
`AdmissionRow`/`ADMISSION_ROWS` + the agreement test, transport-tagged
rows from day one even while only one transport exists):

- prepare/launch: `opencode run --format json -m <model> [--agent
  <agent>]` with `current_dir` = the bound worktree; prompt on argv or
  stdin per W1's own probe (argv length limits vs stdin support —
  measure, don't guess); first turn learns the server-minted sessionID
  from the first event; later turns `opencode run --format json -s
  <sessionID> …`. Prompt composition reuses the claude/codex grammar:
  execution-model contract + environment contract + mutation-surface
  section + intent + context.
- observe: NDJSON → sergeant events (tool_use → tool events with the
  structured input/output/exit evidence; text → assistant-completed;
  step_finish tokens/cost → usage events; raw stream archived to the
  blob store like claude's raw_blob, with reported-never-swallowed
  archive failure). **The narration rule**: prose text is transcript
  content, never tool evidence.
- terminals: final step_finish + process exit 0 → completed;
  `{"type":"error"}` / exit ≠ 0 → failed with the typed error named;
  process death with no terminal → fail-closed ambiguous with raw
  evidence (§15 invariant).
- interrupt: process-tree termination (honest tier), with W1's probe
  answering the packet's open question 1 (is the session resumable
  after SIGKILL mid-turn? what, if anything, is emitted?).
- permission posture: the adapter relies on measured auto-reject for
  non-interactive safety and does not pass `--auto`. How a permission
  config reaches the run without dirtying the Work's own diff (packet
  open question 3: env var? global config merge? `--dir` semantics?)
  is a W1 probe with its own recorded answer; if no clean channel
  exists, W1 ships with opencode's defaults and the finding is
  recorded honestly.
- version provenance, not gate (R1): probe records `opencode
  --version`; 1.18.19 is the measured floor; below → available with
  unmeasured-provenance detail, never refusal. Unparseable version or
  missing required flag in `--help` → refused (the A2 split, verbatim).
- Capability booleans set ONLY where an L8 contract test against the
  installed harness proves them. Honest W1 expectations:
  streaming/resume/interrupt/model_selection/profiles/usage/
  persistent_sessions true with their tests; **history true via
  `export`** (R4 — exceeds both claude and codex) with a completeness
  proof against a multi-turn session; ask/approval_flow/
  native_background/human_attach/native_subagents false pending
  measurement. Model-pin verdict: W1 probes whether any event/export
  field names the served model (packet open question 2); absent
  positive evidence, model_selection's row records
  substitution-undetectable — codex-exec's own posture, stated
  honestly.
- Contract tests: `tests/opencode_backend.rs` — StubOpencode
  shell-script stand-in + fixtures recorded at 1.18.19 for the
  deterministic tier; live tier `#[ignore]`d + `SERGEANT_OPENCODE_TESTS=1`
  + probe/auth precheck (the A3 pattern), all pinned big-pickle.

### W2 — registration + routing verification + the PATH line

- Register the W1 adapter under `"opencode"`: `DaemonConfig.opencode`,
  the daemon.rs construction/registration block, event-sink wiring —
  the mechanical mirror of daemon.rs's codex block (ADR 0020 already
  proved the selection machinery ships; registration is verification,
  not invention).
- `tests/opencode_routing.rs` on the codex_routing.rs template:
  explicit `--backend opencode`, origin-affinity from `sgt opencode`
  (`SGT_ORIGIN_CLIENT`), estate `default_backend = "opencode"` — each
  proven to reach the real registered adapter (nonexistent-executable
  config, host-independent).
- Swap the three `tests/m3_execution.rs` fixtures that use
  `"opencode"` as the canonical *unregistered* backend name
  (m3_execution.rs:1793, 2045, 4689) to a name that stays unregistered
  (`"goose"`), with the same explanatory comments codex's registration
  left behind.
- **`harness.rs` PATH line**: add `~/.opencode/bin` to
  `toolchain_path_dirs` with the measured evidence cited (the packet's
  `command not found` row) — the same failure class and the same
  one-line remedy that put `~/.cargo/bin` and `~/.local/bin` there.
  Named here as this sprint's one touch outside `src/backend/`,
  ratified by this plan's owner review rather than smuggled (K2
  scoped to "no core changes"; `harness.rs` is harness composition
  surface, and the list is designed to grow by measured entries —
  its own module doc says so).
- Daemon-level registration tests (the codex W2 pair): daemon start
  registers opencode + journals its probe; daemon start with no
  opencode installed still starts and says why.

### W3 — serve-transport capability upgrades, each admitted by
measurement (opus spec)

An adapter-owned `opencode serve` child **per execution** on
`127.0.0.1` with an ephemeral port and `OPENCODE_SERVER_PASSWORD` set
by the adapter (doc-claimed: no auth otherwise) — the same
adapter-owned-child shape ADR 0020 chose over a shared daemon, for the
same blast-radius reasons; `RuntimeScope` stays `PerExecution`, so
`mod.rs`'s anticipated ENSURE-RUNTIME seam stays untouched (K2).
Driven via the already-installed `reqwest` (R5, zero new crates).
run-json remains the fallback at every capability; a serve child that
fails to spawn fails LAUNCH honestly, never silently downgrades
mid-registration (codex W3 §5.3 verbatim).

Capability upgrades, each landing only with its measured admission row:

- **`approval_flow: true` candidate** — `permission.asked` SSE event +
  `POST /session/:id/permissions/:permissionID` reply (doc-claimed;
  W3 measures the event shape, the reply body schema the docs leave
  unstated, and the deny path). Would exceed both claude and codex —
  the first true on this flag in the registry. Sergeant's declared
  mutation surface maps onto opencode's per-tool glob permission
  config; W3 specs that mapping.
- **`ask`** — opencode has a `question` permission category and
  typed permission events; whether an actor-authored question is
  schema-distinguishable end-of-turn is measured, not prose-guessed;
  ask stays false without its L8 test (codex's open-admission
  posture).
- **native interrupt** — `session.abort()` (doc-claimed) upgrades the
  tier from ProcessTreeTermination; measured or stays process-kill.
- **history** — serve's `messages()` vs W1's `export`: whichever
  carries the completeness proof more cheaply wins; the other is
  recorded as the alternative.
- **structured output** — `format: {type:"json_schema", schema}` on
  `session.prompt` (SDK-documented) → the adapter-local
  structured_output row, mirroring codex's.
- The serve OpenAPI document (`GET /doc`) is fingerprinted at
  admission like codex's schema dump — drift surfaces in the probe as
  provenance-stale, not a crash.

### W4 — fake fidelity, doctrine, finalize

- Fake backend learns the measured opencode failure shapes the suite
  needs deterministically (auto-rejected tool call; typed error
  terminal; death-without-terminal; server-minted-id first turn).
- Doctrine: ADR 0021 (the opencode adapter — transports, the unused
  WebSocket carve-out, the no-native-sandbox posture, R4 deltas);
  `docs/DEVELOPMENT.md` backend list gains opencode; CHANGELOG 0.2.2;
  README quickstart line. No NORTH-STAR amendment (nothing to claim —
  see measured facts).
- Finalize: version 0.2.2 (K3), retro, head PR un-drafted. Proof: the
  opencode contract suite green against the installed harness ON the
  PR, plus a live `sgt run --backend opencode` Work against a scratch
  estate recorded in the PR body (bounded, big-pickle).

## Ratify-at-review (owner, at head PR)

1. Which W3 capabilities shipped vs fell back, each with its measured
   reason (the approval_flow first-true is the headline candidate).
2. The `harness.rs` PATH line (W2's named exception to "no core").
3. ADR 0021's text.

## Risks

- **Free-tier availability/limits**: big-pickle is "for a limited
  time" per opencode's own docs; rate-limit and auth-expiry shapes are
  unmeasured (packet open question 6). Contract tests assert
  structured shapes and terminals, never prose; bounded retries; any
  test that would pass on narration alone is a test-honesty panel
  finding.
- **Upstream churn**: no stability policy; the fingerprint + R1
  provenance posture is the mitigation, plus fixtures pinned at
  1.18.19.
- **Server-minted sessionID**: the first-turn adoption window (process
  alive, no event line yet) needs the same fail-closed care as
  claude's ambiguous-terminal handling; W1 spec addresses it
  explicitly.
- **tmpfs**: builds in `/var/tmp/opencode-impl/` only (#70).
- **Live-spend discipline**: big-pickle costs $0 but consumes
  free-tier quota — live tests stay opt-in-gated exactly as if they
  billed (A3 pattern), no unbounded loops, not required PR checks.

## As-landed postscript (2026-08-23, appended by W4)

This plan's own open questions and candidate framings did not all land
the way they were written above; recorded here rather than left for a
reader to reconcile against `docs/adr/0021-opencode-adapter.md`
silently.

**Both W3 open questions this plan hedged resolved positive, not
negative.** §W3 called `approval_flow` "a candidate" and framed `ask`
as needing measurement "not prose-guessed" with no stated expectation
either way. Both are `true` on the serve transport as shipped, each
with a live admission test run against the installed 1.18.19 binary,
`-m opencode/big-pickle`: `permission.asked`/the deprecated-but-live v1
reply endpoint for `approval_flow`, and the distinct typed `question`
tool/`question.asked` event for `ask` — the one place this plan
predicted codex's own protocol *might* have exceeded Claude's and
codex's open admission never closed it (ADR 0020's own "open
admission test, not a claimed negative" section). opencode's did. Two
corrections to this plan's own written spec surfaced only by running
those live tests, both recorded in `opencode.rs`'s admission rows and
ADR 0021 rather than silently absorbed: the serve abort's terminal
signature also appears on the *synchronous* `POST` response, not only
an SSE frame as assumed; and `structured_output` lands at
`info.structured` with `info.finish == "tool-calls"`, not the guessed
`structured_output` field with `"stop"`.

**The K2 "no core changes" ledger is real but not empty, exactly as
K2's own text anticipated ("the adapter is scope; no core changes" —
not "zero files outside `src/backend/`").** Four items across W1–W3
touched something outside `src/backend/`: the A4-required blob-ref
PUT-site/recovery row (`tests/a4_blob_ref_pinning.rs`, W1, gate-forced
by the suite's own admission rule); the `~/.opencode/bin`
`toolchain_path_dirs` line (`src/harness.rs`, W2, pre-ratified by this
plan's own W2 section as "this sprint's one touch outside
`src/backend/`"); three pre-existing fixtures' registered-backend
count/list widened by W2's registration commit itself
(`tests/m3_execution.rs`, `tests/m2_daemon_api.rs`,
`tests/m4_backends.rs`, mechanical fallout, not a separate decision);
and `reqwest`'s own `"blocking"` feature flag made explicit
(`Cargo.toml`, W3, `Cargo.lock` byte-identical before and after). All
four are listed with their individual reasons in ADR 0021's own K2
exception ledger for owner ratification at the head PR, exactly as
this plan's own "Ratify-at-review" list promised.

**The WebSocket carve-out the owner offered at kickoff went
unused**, and this plan's own K2 bullet already predicted why: W3
measured opencode's serve transport as HTTP + SSE, not WebSocket, so
the already-installed `reqwest` (R5, zero new crates) carried the
whole transport. The carve-out's existence was not wasted motion —
it is the reason W3 did not have to stop and ask when the transport
turned out not to need it.

**Coverage-lift was added to W4's scope by owner ruling, not written
into this plan's own W4 section above.** The W3 branch measured
green at Gate D's 90% floor but thin on two files
(`backend/opencode.rs` 82.91% lines, `backend/opencode_serve.rs`
79.69% lines / 67.74% functions) — both driven by every existing
serve-transport test pinning a transport explicitly, leaving the
*default* `Auto` path, its gate-failure fallback, and several
terminal-classification and ask/permission reply-relay arms
unreached. W4 lifted both to 91%+ lines before finalize, per the
owner's own ruling recorded in this task's brief rather than a change
this document's own text anticipated.

**No NORTH-STAR amendment landed, as this plan's own W4 section
already said it would not.** opencode has no native sandbox
mechanism to claim (permission config and per-tool disables only,
policy the model's tool layer honors, not a kernel-level write
barrier) — ADR 0020's amendment 4 permits an adapter to use native
enforcement where the harness has one; this adapter has nothing that
qualifies, so nothing was appended.
