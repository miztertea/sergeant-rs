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
  modelUsage and than codex/opencode's substitution-undetectable
  posture. Per-step usage {input/output/thinking/cache_read/total}
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
  terminals: a SUCCESS with no response and no steps is suspicious,
  recorded honestly.
- interrupt: process-group termination (the probe-11 grandchild
  lesson, carried); W1's probe answers the SIGKILL-mid-turn class
  (terminal event? conversation resumable? orphaned children?).
- permission posture: W1 probes the settings.json granular path — how
  a permission config reaches the run without dirtying the Work's own
  diff (the file is user-global; workspace/env override unmeasured) —
  and resolves the soft-deny discrepancy, both with recorded answers.
  If no clean channel exists, W1 ships with agy's defaults (auto-deny,
  measured non-blocking) and records the finding; the blanket
  dangerous flag is never a default.
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
  native_background/human_attach false pending W3. `--print-timeout`
  expiry shape is a W1 probe (adapter-local detail, no v1 boolean).
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
  complete K2 exception ledger); `docs/DEVELOPMENT.md` backend list;
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
5. ADR 0022's text.

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
