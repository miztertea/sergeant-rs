# Sprint plan — Sergeant speaks Antigravity (2026-08-23)

Owner-commissioned 2026-08-23: build the Antigravity (`agy`) adapter,
Claude/Codex/OpenCode feature parity at minimum, exceed it where agy
measurably does better. This plan follows ADR 0020's own instruction —
its pattern is "the pattern any future adapter should copy rather than
re-derive" — and ADR 0021's worked second application of it. Where a
section matches those plans, that is the point, not an oversight.

**Spec sources (authority order):**

1. Owner kickoff rulings 2026-08-23 (this session, on the record):
   - **K1 — dev-test pin is `gemini-3.7-flash-low`** (free tier,
     generous daily limits — owner-authorized; calls stay small/short).
     The adapter takes model as a parameter and hardcodes nothing;
     every live test and proof Work pins flash-low.
   - **K2 — scope is the adapter; no core changes**, with one
     pre-ratified named exception: the `sgt agy` passthrough block in
     `cli.rs`/`harness.rs` (the exact mechanical mirror of the goose
     block, ADR 0006 D2) — origin-affinity routing has no origin to
     affine from without it. Anything else outside `src/backend/` goes
     on the K2 exception ledger for ratification at the head PR.
   - **K3 — version target 0.2.3** (patch bump, the 0.2.1/0.2.2
     precedent: "sergeant speaks antigravity").
   - **K4 — the name is `agy` everywhere**: registry name, `--backend
     agy`, `DaemonConfig.agy`, `sgt agy`, `src/backend/agy.rs` — the
     binary's own name, the verb-execs-binary precedent (goose).
2. Carried rulings from ADR 0020/0021, applying without re-litigation:
   **R1** (version floors are provenance, not gates), **R2**
   (harness/backend are separate user-composable axes), **R3** (= K2;
   contract v1 untouched, adapter-local admission ledger), **R4**
   (parity is the floor; a measured better capability is used and the
   record amended).
3. The probe evidence packet (`sergeant-rs-workspace`:
   `knowledge/evidence/agy-adapter-probes-2026-08-23.md`) — 6 live
   probes @ agy 1.1.17 plus a docs research pass; every agy behavior
   claim below is measured there unless marked **doc-claimed**.
4. Shipped code: `src/backend/mod.rs` (§15 v1), `opencode.rs`/
   `opencode_serve.rs` (the nearest template: AdmissionRow ledger,
   one-decoder-two-transports, live-test gating), `claude.rs` (the
   print/stream-json shape agy's CLI mirrors), ADR 0020/0021.

**Protocol** (unchanged from codex/opencode): integration branch
`integration/agy`, draft head PR carrying this plan, wave branches
`agy/w<N>-<slug>` in `/var/tmp/agy-impl/` worktrees (warm base build
first; tmpfs `/tmp` never — #70). Per wave, ONE workflow: spec →
implement (TDD, DataDir guards) → 4-axis blind panel + per-finding
refuters (default refuted) → fixer on confirmed only → gate rerun.
Wave PR → CI by SHA → captain merges to integration; the owner merges
the head PR (ADR 0015). Sonnet subagents, opus where earned, Fable
never below the captain. Rung/ruling citations in every PR body.
Coverage floor 90% (Gate D): every new suite lands with its
`c2-suites.sh` wiring line in the same commit — the
`coverage_stage_membership` guard now enforces it — and total margin
is measured before each wave PR. Panel prompts demand the run
transcript wherever a row claims LiveMeasured (opencode retro: six
cited tests didn't exist until the panel caught it). **Dev-time agy
invocations pin `--model gemini-3.7-flash-low`** (K1).

## Measured facts this plan builds on (packet citations)

- Claude-shaped CLI: `-p` + `--output-format stream-json` emits typed
  NDJSON — `init` → `step_update`* → `result{status}`; status enum
  SUCCESS/ERROR/CANCELED/INTERRUPTED/INVALID/WAITING/RUNNING (probes
  1–4; enum doc-claimed beyond SUCCESS/ERROR).
- **Identity and the resolved model echo on line 1**: `init` carries
  `conversation_id`, the effective `model`, the full tool roster, and
  the effective permission mode before any model output (probe 1) —
  native launch-time pin verification, better than claude's post-hoc
  modelUsage, opencode's post-hoc export/response verification
  (substitution detectable, after the turn — its own rows' words), and
  codex's genuinely substitution-undetectable posture. Per-step usage {input/output/thinking/cache_read/total}
  rides every step (probe 1). Both are R4 delta candidates.
- **Default print mode auto-DENIES tool permissions and the whole turn
  errors** (typed TOOL_ERROR, terminal `status:"ERROR"`, exit 1, ~4.5s,
  no hang — probe 2). The docs claim the opposite (soft-deny: run
  continues, exit 0, stderr notice). W1 resolves this discrepancy with
  a recorded probe; if soft-deny manifests anywhere, SUCCESS hiding
  skipped tools is an adapter honesty hazard and the terminal
  classifier must not trust `status` alone on tool-bearing turns.
- Granular permissions exist (doc-claimed):
  `~/.gemini/antigravity-cli/settings.json` `permissions.allow/deny/
  ask` with pattern namespaces `command(...)`, `read_file/
  write_file(...)`, `read_url(...)`, `mcp(...)`. The adapter never
  defaults `--dangerously-skip-permissions` (claude #47). `--mode`
  governs only file-edit approval; `--permission-mode` is not a real
  flag.
- Cross-process resume: `--conversation <id>` recalled a prior turn's
  state, SUCCESS (probe 5). Conversation state is server-side;
  headless is "stateless by default" (doc-claimed); `--continue`
  resolves via a cwd→id cache file.
- Native structured output: `--json-schema` → validated
  `structured_output` field beside the prose response (probe 6).
- Typed invalid-model refusal enumerating the whole catalog, with an
  empty `conversation_id` — refused before identity was minted
  (probe 4).
- `--input-format stream-json`: persistent stdin turn loop, one NDJSON
  message per line, one turn each (doc-claimed; arrived 1.1.15) — the
  per-execution second-transport candidate. Plain stdio: zero new
  crates, no ports, no auth. Driver must serialize turns (wait for
  `result` before the next line); closing stdin exits 0.
- Tool roster leads (probe 1): `ask_question` + `ask_permission` (ask
  and approval_flow candidates), `define_subagent`/`invoke_subagent`/
  `manage_subagents`/`browser_subagent` (**native_subagents — no
  registry adapter claims it; would be the first true anywhere**),
  browser automation suite, `generate_image`, `schedule`, `search_web`.
- `--sandbox` is OS-native (doc-claimed): nsjail on Linux — NORTH-STAR
  amendment 4 territory (an adapter MAY use native enforcement); scope
  vs `--add-dir` undocumented; nothing is claimed unmeasured.
- `--print-timeout` (default 5m): a native turn deadline; expiry shape
  unmeasured.
- **MEASURED_FLOOR = 1.1.17** (installed; upstream 1.1.19). Changelog:
  1.1.16 fixed print mode exiting SUCCESS with an empty response when
  the state stream dropped mid-run (the empty-success adapter-killer
  class); the input loop arrived in 1.1.15. Provenance only (R1).
- Binary: `~/.local/bin/agy`, **already on `harness.rs`'s
  `toolchain_path_dirs`** — no PATH line this sprint, unlike opencode.
- `agy --version` → bare `1.1.17`. `agy models` lists the catalog
  including non-Google models (claude-sonnet-4-6, gpt-oss-120b) —
  model is genuinely per-execution routing, not a vendor pin.

## Waves

### W1 — print-mode stream-json adapter core (opus spec + implementer)

`src/backend/agy.rs` implements §15 on the process-per-turn
stream-json transport, mirroring opencode.rs's shape (adapter-local
`Evidence`/`AdmissionRow`/`ADMISSION_ROWS` + the structural agreement
test, transport-tagged rows from day one):

- prepare/launch: `agy -p <prompt> --output-format stream-json --model
  <model>` with `current_dir` = the bound worktree; prompt on argv or
  stdin per W1's own probe. First turn learns `conversation_id` from
  the `init` line — line 1, before any model output, so the adoption
  window is a single bounded read, cheaper than opencode's
  first-event wait but with the same fail-closed posture: no init
  line, no handle. Later turns `--conversation <id>`. Prompt
  composition reuses the claude/codex/opencode grammar.
- observe: `step_update` → sergeant events. `tool_info` {name, exact
  parameters, output} is structured tool evidence (probe 3) — the
  narration rule is structural; normalize the measured pty `\r\n`.
  Per-step + terminal usage → usage events. Raw stream archived to the
  blob store (the A4 PUT-site/recovery row in
  `tests/a4_blob_ref_pinning.rs` is gate-forced — a known K2 ledger
  item, ADR 0021's precedent).
- terminals: `result{status}` mapped across the full enum — SUCCESS →
  completed; ERROR → failed with the typed error named; CANCELED/
  INTERRUPTED → the interrupt disposition; unknown statuses and
  process death with no terminal → fail-closed ambiguous with raw
  evidence (§15). The 1.1.16 changelog class (empty-success on a
  dropped stream) argues for a completeness check on SUCCESS
  terminals: a SUCCESS with no response and no steps is suspicious.
  Consequence specified (panel amendment, 2026-08-23): such a terminal
  classifies **fail-closed ambiguous with raw evidence — never
  completed-clean**, pinned by a StubAgy fixture replaying the
  dropped-stream shape.
- interrupt: process-group termination (the probe-11 grandchild
  lesson, carried); W1's probe answers the SIGKILL-mid-turn class
  (terminal event? conversation resumable? orphaned children?).
- permission posture: W1 probes the settings.json granular path — how
  a permission config reaches the run without dirtying the Work's own
  diff (the file is user-global; workspace/env override unmeasured) —
  and resolves the soft-deny discrepancy, both with recorded answers.
  **Shipping on agy's defaults is NOT a viable fallback for
  tool-bearing stages** (panel amendment, 2026-08-23): probe 2
  measured default `request-review` auto-denying the tool AND erroring
  the whole turn — an adapter on defaults cannot execute any tool. The
  honest ladder: (a) a measured clean injection channel (env var,
  workspace-level settings, flag) → use it, mapping the Work's
  declared mutation surface onto the `permissions` namespaces; (b) no
  clean channel measured → the adapter documents the operator-config
  requirement, and PREPARE/LAUNCH reads the **effective
  `permission_mode` off the init line** (measured, probe 1) so a
  tool-bearing intent launching under a denying mode is reported
  honestly at launch, not discovered as a mid-run turn error. Either
  outcome rides the head-PR ratify list; the blanket dangerous flag is
  never a default.
- version provenance, not gate (R1): probe records `agy --version`
  (bare token); 1.1.17 is the measured floor; below → available with
  unmeasured-provenance detail. Unparseable version or missing
  required launch grammar in `--help` → refused (the A2 split,
  verbatim).
- Capability booleans set ONLY where an L8 contract test against the
  installed harness proves them. Honest W1 expectations: streaming/
  resume/persistent_sessions/model_selection/usage/interrupt true with
  their tests; **model_selection's row records verified-at-launch via
  the init echo** (R4 delta — every prior adapter records
  substitution-undetectable or post-hoc); **history false pending
  measurement** — headless is stateless by default and no export verb
  is documented; unlike opencode there is no cheap true here, and §15
  forbids emulation; ask/approval_flow/native_subagents/
  native_background/human_attach false pending W3; **`profiles`**
  (panel amendment, 2026-08-23 — all 13 contract booleans now
  accounted): candidate via `--agent` + the custom-agent definition
  mechanism (doc-claimed; `agy agents` printed an empty list on this
  host — probe), true only if a W1 admission test measures a defined
  agent altering the launch (opencode's precedent), else false with
  the probe recorded. `--print-timeout` expiry shape is a W1 probe
  (adapter-local detail, no v1 boolean).
- Contract tests: `tests/agy_backend.rs` — StubAgy shell-script
  stand-in + fixtures recorded at 1.1.17 for the deterministic tier;
  live tier `#[ignore]`d + `SERGEANT_AGY_TESTS=1` + probe/auth
  precheck (the A3 pattern), all pinned flash-low, small turns.
  Wired into `c2-suites.sh` in the same commit.

### W2 — registration + routing + the `sgt agy` passthrough

- Register the W1 adapter under `"agy"`: `DaemonConfig.agy`, the
  daemon.rs construction/registration/event-sink block — the
  mechanical mirror of the opencode block. **Registration does no
  transport resolution and constructs no blocking HTTP client** (the
  0.2.2 daemon-panic lesson, c46152a2); a regression test boots the
  daemon with an agy stub on PATH.
- `tests/agy_routing.rs` on the codex/opencode routing template:
  explicit `--backend agy`, origin affinity from `sgt agy`
  (`SGT_ORIGIN_CLIENT`), estate `default_backend = "agy"` — each
  proven to reach the real registered adapter.
- **The K2 pre-ratified exception**: an `Agy` passthrough subcommand
  in `cli.rs` + `harness.rs` (exec `agy`, compose the estate
  environment, set the origin client) — the exact mechanical mirror of
  the goose block (ADR 0006 D2). `goose` stays the canonical
  unregistered fixture name; no fixture-name swap is forced this
  sprint, only the mechanical registered-backend count/list widening
  in m2/m3/m4 (same-commit fallout of registration, ADR 0021's
  precedent).
- No PATH line: `~/.local/bin` is already in `toolchain_path_dirs`
  (measured — agy resolves from a non-interactive shell).
- Daemon-level registration tests (the codex/opencode W2 pair):
  daemon start registers agy + journals its probe; daemon start with
  no agy installed still starts and says why.

### W3 — input-loop transport upgrades, each admitted by measurement
(opus spec)

A persistent `agy --input-format stream-json --output-format
stream-json` child **per execution** (`RuntimeScope::PerExecution`;
`mod.rs`'s ENSURE-RUNTIME seam untouched — K2), driven over plain
stdio: zero new crates, no ports, no auth posture to carry — the
cheapest second transport any adapter has had. Print mode remains the
fallback at every capability; a loop child that fails to spawn fails
LAUNCH honestly (codex §5.3 / ADR 0021 verbatim). Turn serialization
per the documented driver contract (wait for `result` before the next
NDJSON line), measured before relied on.

Capability upgrades, each landing only with its live admission row:

- **`ask` candidate** — the typed `ask_question` tool (probe-1 roster)
  surfacing as a schema-distinguishable event mid-turn, answered by a
  stdin message, turn resuming: opencode's question-tool precedent.
  Actor authorship must be typed, never guessed from prose
  (`Capabilities::ask`'s own contract).
- **`approval_flow` candidate** — `ask_permission` + a stdin reply
  relaying approve/deny; would be the registry's second true. How the
  reply is encoded is unmeasured; measured or stays false.
- **`native_subagents` candidate — the headline.** agy ships
  `define_subagent`/`invoke_subagent`/`manage_subagents`/
  `browser_subagent` natively; packet open question 7 is how subagent
  activity surfaces in the stream. A typed record admits the registry's
  first true on this flag anywhere; anything less stays false with the
  probe transcript recorded.
- **native interrupt tier** — SIGINT → `status:"INTERRUPTED"` typed
  terminal (doc-claimed enum) as an upgrade from process-group kill;
  conversation-resumable-after-interrupt measured. Any failure falls
  back to the group kill and journals the downgrade (codex §7.3).
- **structured_output** — native `--json-schema` (measured, probe 6) →
  the adapter-local row, mirroring codex/opencode's posture (R3: no v1
  boolean invented). The cheapest first in the fleet: the channel is a
  CLI flag, not a protocol negotiation.
- **Sandbox** — measure `--sandbox`'s nsjail semantics and `--add-dir`
  interaction. Only a measured mechanism earns a NORTH-STAR amendment
  4 claim, at codex's "enforcement-claimed, not locally proven" tier
  at most; nothing measured → honest silence (ADR 0021's posture).
  Sergeant's observation layer stays the source of truth either way.

### W4 — fake fidelity, doctrine, finalize

- Fake backend learns the measured agy shapes the suite needs
  deterministically: auto-deny turn-error (typed TOOL_ERROR + ERROR
  terminal), typed invalid-model refusal with empty conversation_id,
  init-first identity, death-without-terminal, pty `\r\n` output.
- Coverage lift to the 90 floor if any module measures thin (the
  0.2.2 precedent: measure per wave, lift before finalize).
- Doctrine: ADR 0022 (the agy adapter — transports, the soft-deny
  discrepancy's resolution, the sandbox stance, R4 deltas, the
  complete K2 exception ledger); **the NORTH-STAR amendment-4 outcome
  is W4's assigned writing task** (panel amendment): if W3 measured a
  real nsjail enforcement mechanism, W4 appends the dated amendment-4
  claim at codex's "enforcement-claimed, not locally proven" tier; if
  not, ADR 0022 records honest silence — the decision is written
  either way, not left implicit. `docs/DEVELOPMENT.md` backend list;
  CHANGELOG 0.2.3; README quickstart line.
- Finalize: version 0.2.3 (K3), retro, head PR un-drafted. Proof: the
  agy contract suite green against the installed harness ON the PR;
  the daemon-boot e2e with agy on PATH; a live `sgt run --backend agy`
  Work against a scratch estate recorded in the PR body (bounded,
  flash-low).

## Ratify-at-review (owner, at head PR)

1. Which W3 capabilities shipped vs fell back, each with its measured
   reason (`native_subagents` first-true is the headline candidate).
2. The `cli.rs`/`harness.rs` passthrough block (K2's pre-ratified
   exception, confirmed against its actual diff).
3. The sandbox stance (amendment-4 claim or honest silence).
4. The soft-deny discrepancy's resolution and any terminal-classifier
   consequence.
5. The permission-injection channel shipped — or, if no clean channel
   measured, the documented operator-config requirement plus the
   launch-time permission-mode honesty check (panel amendment).
6. ADR 0022's text.

## Risks

- **Free-tier quota shape unmeasured** (packet open question 5): 429?
  hang? typed error? Live tests are opt-in-gated, bounded, small,
  never required CI checks; any test that would pass on narration
  alone is a test-honesty panel finding.
- **Upstream churn**: installed 1.1.17, upstream 1.1.19, no
  documented stability policy. Fixtures pinned at 1.1.17; R1
  provenance posture is the mitigation.
- **Docs contradict measurement already** (soft-deny; BYO-key
  self-contradiction) — every W3 claim is measured before specced,
  the 0.2.2 lesson that caught the v1/v2 permission fork.
- **Conversation scoping unmeasured**: `--project` identity effects,
  `--continue` cwd-cache semantics — the resume row states exactly
  what probe 5 proved (resume-by-id, same host, same user) and no
  more.
- **tmpfs**: builds in `/var/tmp/agy-impl/` only (#70).
- **The input-loop transport is doc-claimed end to end** — if W3's
  measurement finds it unfit (deadlocks, no mid-turn events on
  stdin), every W3 candidate falls back to print mode and the ADR
  records why; the sprint still ships parity on W1's transport.

## Plan-review record (2026-08-23, pre-W1)

Owner-directed panel review before any wave ran: blind 4-axis panel
(evidence grounding, doctrine/template fidelity, technical soundness,
completeness/risk) + one adversarial refuter per finding, default
refuted — all sonnet. 12 raw findings, 5 confirmed, 7 refuted with
recorded reasons. The five confirmed amendments are applied in place
above, each marked "(panel amendment, 2026-08-23)": the opencode
model_selection mischaracterization corrected; the permission-fallback
"ship with defaults" defect (blocking — probe 2 shows defaults error
every tool-bearing turn) replaced with the honest two-rung ladder and
a launch-time permission-mode check; the empty-SUCCESS completeness
check given its fail-closed classification consequence and fixture;
`profiles` added so all 13 contract booleans are accounted; the
NORTH-STAR amendment-4 outcome assigned to W4 as a writing task.

## As-landed postscript (2026-08-23, appended by W4)

This plan's own open questions and candidate framings did not all
land the way they were written above; recorded here rather than left
for a reader to reconcile against `docs/adr/0022-agy-adapter.md`
silently.

**The soft-deny discrepancy resolved inverted from the packet on
print, and split by transport in a way this plan did not
anticipate.** §W1 framed the discrepancy as "resolves with a recorded
probe" and hedged only one direction: "if soft-deny manifests
anywhere… the terminal classifier must not trust `status` alone." It
manifested — print mode at the installed 1.1.19 is soft-deny, exactly
inverting the packet's own hard-deny hypothesis, `CANCELED`/exit
0/stderr-only, classified fail-closed ambiguous. What this plan's own
§W1 did not predict is that §W3's loop transport would measure the
*opposite* shape on the identical stimulus: a typed `TOOL_ERROR`, a
failed terminal, and the **child process itself exiting** — the
wave's own "most operationally important measurement," per the
admission row's own words, and a genuine argument (recorded, not
acted on) for `PrintOnly` as an operator default until a per-Work
allow-rule policy exists.

**Both W3 refutation candidates this plan hedged resolved negative,
the mirror image of opencode's own postscript.** §W3 named `ask` and
`approval_flow` as candidates "measured or stays false" with no
stated expectation, and named a SIGINT-based native interrupt tier as
an upgrade candidate. All three refuted, each with a live transcript:
sixteen candidate loop reply-event names tried and rejected/ignored
for `ask`/`approval_flow` (W3 P1), and a SIGINT terminal
byte-identical to a plain `--print-timeout` expiry for the interrupt
tier (W3 P4). OpenCode's own postscript records both of *its*
analogous candidates landing `true`; agy's land `false` — two
adjacent adapters, two different measured answers to structurally
similar questions, neither forced to match the other.

**`native_subagents` landed exactly as this plan's own §W3 framed
it** — "the headline candidate," admitted only "on a typed record" —
and closed the good way: `true` on the loop transport, the first
`true` for this flag anywhere in the registry, on all three pieces of
evidence the spec demanded (a typed `step_type "subagent"` step, a
typed `subagent_info` payload naming a distinct child
`conversation_id`, and that step reaching `DONE`).

**`profiles` landed as a declared divergence, not the plan's own
either/or framing.** §W1's panel amendment asked for `true` only if a
W1 admission test measured a defined agent altering the launch, else
`false` with the probe recorded. What actually shipped is a third
outcome the plan did not name: `true`, but on generic sergeant axes
only (executable + env reaching every turn), with agy's own `--agent`
mechanism itself measured unreachable on this host (`agy agents`
printed an empty list; the documented workspace mechanism for
defining one did not work either, W1 P6) — a distinction recorded in
`agy.rs`'s own admission row as a DECLARED DIVERGENCE, and carried
into ADR 0022's own open-questions section as a hand-off rather than
silently resolved.

**Sandbox landed as honest silence, as this plan's own §W3 already
said it would if nothing measured** — but not *nothing* was measured:
W3 S1 found `proceed-in-sandbox` genuinely lifts the permission gate
as a real second channel, on a host where the sandbox server itself
does not run (a connection-reset failure at the tool layer). The plan
asked for "codex's own tier at most, or honest silence" and got
honest silence with a corroborating negative (`nsjail` absent from
the binary's own strings) plus one adjacent positive fact recorded
for a future wave rather than promoted into a claim this host cannot
back. No NORTH-STAR amendment 4 entry was appended, identically to
ADR 0021's own reasoning for opencode.

**One version-attribution correction, made because a build-version
argument is exactly what R1's fail-closed rules exist to resist**:
this plan and the probe packet both attributed the empty-SUCCESS
stream-drop fix to agy 1.1.16; the CLI's own bundled changelog names
it **1.1.18**. This changes nothing about the classifier — the panel's
empty-SUCCESS rule is fail-closed by construction and
version-independent — but is recorded rather than silently corrected
without a trace, per this plan's own §1.1 framing of the two version
numbers.

**Two fixes this plan's own W1/W2 sections did not anticipate,
folded in as they were found**: the zero-quota `/config` probe is not
always zero-interaction (an unauthenticated `agy` blocks on an
interactive OAuth prompt for up to 60s; `CONFIG_PROBE_BUDGET` now
bounds it, found during W2's registration wave, not W1's own probe
pass), and a turn-end posture race on the loop transport (the reader
thread now computes `PermissionPosture` itself at init-parse time
rather than depending on LAUNCH's own round trip, closed during W1's
own captain pass — commit `38932727`).

**The K2 "no core changes" ledger is real but not empty, exactly as
K2's own text anticipated.** Six items across W1–W3 touched something
outside `src/backend/agy.rs`: the `sgt agy` passthrough itself
(`src/cli.rs`, `src/harness.rs`, K2's own pre-ratified exception); the
`"agy"` `BackendRegistry` registration (`src/daemon.rs`); the
A4-required blob-ref PUT-site/recovery row
(`tests/a4_blob_ref_pinning.rs`, gate-forced); three pre-existing
fixtures' registered-backend count/list widened by the registration
commit itself (mechanical fallout, not a separate decision); the two
new suites' `c2-suites.sh` wiring (gate-forced by
`tests/coverage_stage_membership.rs`, built during the opencode
sprint); and the adapter's own `tests/agy_backend.rs`/
`tests/agy_routing.rs` files, listed for the ledger's completeness
rather than because either is a scope question. All six are listed
with their individual reasons in ADR 0022's own K2 exception ledger
for owner ratification at the head PR, exactly as this plan's own
"Ratify-at-review" list promised. Unlike opencode's own sprint, agy
needed **no** new `Cargo.toml` dependency or feature flag at all —
the input loop is plain stdio, and W3's own §"input-loop transport
upgrades" framing ("zero new crates, no ports, no auth posture to
carry") landed exactly as written.

**Coverage held at or above the floor without a dedicated lift wave.**
W4's own gate rerun found `src/backend/agy.rs` and `src/backend/
fake.rs` both clear of the 90% floor already; see the gate results
recorded in this task's own delivery record rather than restated here.
