# Sprint plan — Sergeant speaks Codex (2026-08-21)

Owner-commissioned same-day: *"Claude works, now make codex work."* The core
boundary and §15 contract stay as they are — everything in this sprint is
the adapter and the thin surfaces that let users compose it.

**Spec sources (authority order):**
1. Owner rulings 2026-08-21 (this session, on the record):
   - **R1 — version floors are provenance, not gates.** claude.rs's
     minimum-version REFUSAL is struck ("strike it, why is this a
     decision"); measured floor = provenance marker; below it: report
     honestly, never block.
   - **R2 — harness and backend are separate, user-composable axes.** Any
     combination legal. Default backend at `sgt run` = the harness the
     session was launched with; explicit selection and estate config
     override.
   - **R3 — core stays; everything is the adapter.** No contract-v2 seam.
     Additive code against shipped v1, the way codex.rs's own doc-stub
     predicted.
   - **R4 — parity is the floor, not the ceiling.** Where codex works
     better than claude (native interrupt, sandbox enforcement, structured
     output), use it — and amend the record with a dated entry
     ([[repo-is-a-snapshot]]: the repo is what was known at commit time).
2. The H0 evidence packet
   (`h0-adjudication-evidence-2026-08-21.md` beside this plan) — every
   codex behavior claim this plan makes is measured there at codex-cli
   0.149.0 unless marked doc-claimed.
3. Shipped code: src/backend/mod.rs (the v1 contract: Capabilities'
   whole-or-refuse history rule, the ask flag's actor-authored rule, L8
   flag↔contract-test binding), claude.rs (the pattern to mirror),
   src/harness.rs + ADR 0006 (`sgt <harness>` compose-and-exec).

**Protocol** (unchanged): integration branch `integration/codex`, draft
head PR carrying this plan, wave branches `codex/w<N>-<slug>` in
`/var/tmp/codex-impl/` worktrees (warm base build first), per wave:
spec → implement (TDD, DataDir guards, R-S0-12) → 4-axis blind panel +
refuters (default refuted) → fixer on confirmed only → wave PR → CI by
SHA → merge to integration. Sonnet subagents, opus where earned (named
below), Fable never below the captain. Rung/ruling citations in every PR
body. **Dev-time codex invocations pin `-m gpt-5.6-luna`** (measured
slug); the adapter itself takes model as a parameter, hardcoding nothing.

## Measured facts the specs build on (packet citations)

- `codex exec --json` emits: `thread.started` (thread_id) → `turn.started`
  → `item.*` (`agent_message`, `command_execution` with
  command/aggregated_output/exit_code/status, `error`) → `turn.completed`
  {usage: input/cached/cache_write/output/reasoning tokens} |
  `turn.failed` {error}. Exit codes and refusal shapes measured.
- `codex exec` is **non-blocking by construction** — no approval flag
  exists on exec (a `-a` is a parse error); it cannot hang a stage.
- Durable thread resume measured twice with nonce continuity
  (`codex exec resume <thread-id>`); `--ephemeral` opts out of rollout
  persistence.
- **Narration caveat (Luna tier, measured 3×)**: `agent_message` can
  narrate command outcomes with NO corroborating `command_execution` item
  and no filesystem effect. Adapter trust rule: structured items are
  evidence; prose is never evidence.
- app-server: PID-managed daemon (`codex app-server daemon start`, control
  socket under `~/.codex/`), JSON-RPC-2.0-in-WebSocket-frames (even over
  unix sockets; stdio:// is unframed), 150+-method protocol dumpable
  offline (`generate-ts`/`generate-json-schema` — a mechanical protocol
  fingerprint), live-verified: thread/start {model}, full event stream,
  `turn/interrupt` → `turn/completed{interrupted}`, token-usage events.
  Self-declared `[experimental]`.
- Sandbox: `read-only | workspace-write | danger-full-access` +
  `--add-dir`; enforcement itself **unmeasurable on Cerberus** (nested
  bwrap) — capability rows carry that provenance honestly.
- Auth: ChatGPT-session tokens cannot self-refresh inside `codex exec`
  (binary strings) — unattended runs must surface auth expiry as an
  honest turn failure, never a hang. Unknown `-p` profile is silently
  ignored by codex (exit 0) — the adapter validates profile existence
  itself.
- Model catalog: `gpt-5.6-luna` / `-terra` / `-sol` (+ others) in
  `~/.codex/models_cache.json`; `thread/start`/`-m` verified with luna.

## Waves

### W1 — `codex/exec` adapter core (opus spec)
`src/backend/codex.rs` implements §15 on the exec transport, mirroring
claude.rs's shape:
- prepare/launch: `codex exec --json -m <model> -C <cwd>` composed from
  StartRequest + BindingSummary (prompt grammar states repo paths,
  branches, base SHAs, per §10.1); `--skip-git-repo-check`; config
  hygiene flags per spec (`--ignore-user-config` NOT used by default —
  users own their config; R1 spirit).
- observe: JSONL → sergeant events (`conversation.*` mapping, raw stream
  into the blob store like claude's raw_blob); `command_execution` items
  → tool events; **the narration rule**: agent_message text is transcript
  content, never tool evidence.
- terminals: turn.completed/turn.failed/error mapped; process death
  without terminal = fail-closed ambiguous terminal with raw evidence
  (§15 invariant).
- interrupt: process-tree termination (honest InterruptCapability tier);
  resume: `codex exec resume <thread>`; usage: per-turn native.
- **Version provenance, not gate** (R1): probe records codex-cli version;
  0.149.0 is the measured floor; below → capabilities carry unmeasured
  provenance + doctor detail, never refusal.
- Capability booleans set ONLY where the L8 contract test against the
  installed harness proves them (ask=false until measured
  actor-authored-question evidence exists; history honest per
  whole-or-refuse — rollout files are PrivateProtocol, so likely false).
- Contract tests: m10-style suite gated on installed codex (mirror
  claude's harness-gated tests), all pinned `-m gpt-5.6-luna`.

### W2 — claude.rs version-refusal strike + harness/selection surfaces
- **R1 applied to claude.rs**: refusal below the measured minimum becomes
  provenance reporting (probe detail + doctor); the m4-era contract test
  asserting refusal is rewritten to assert the honest report. CHANGELOG
  notes the usability fix.
- **`sgt codex`**: ADR 0006 compose-and-exec boundary grows the codex arm
  (mirror `sgt claude` — compose env/instructions, exec the harness).
- **Backend selection**: `sgt run --backend <name>` (verify current flag
  surface first — R2 recon step); default resolution = flag >
  session-launched harness (env set by `sgt <harness>`, e.g.
  SGT_HARNESS) > estate `default_backend` > engine default. Precedence
  journaled on the Work (auth/config explicitness in v1 terms).
- Preflight stays honest: submit-time capability preflight (already in
  engine) works unchanged against the codex capability rows.

### W3 — better-than-claude capabilities, each admitted by measurement
Capability-driven, not transport-driven; each lands only with its measured
admission row (R-H0-6 vocabulary: documented → implemented → measured →
admitted; provenance + stability tier recorded in the capability rows/
version-policy doc):
- app-server transport (adapter-internal daemon lifecycle under the
  existing `runtime_scope()` surface — no engine seam): native
  `turn/interrupt` (upgrade from process-kill), token-usage events,
  richer event stream. Ships only if its admission tests pass on 0.149.0;
  exec remains the fallback at every capability.
- Sandbox enforcement: `workspace-write` scoped to the declared surface
  (+`--add-dir` for multi-repo Works) — belt on top of sergeant's
  observation; **NORTH-STAR amendment 4 gets its dated amendment**
  ("non-goal for core; an adapter MAY use a harness's native
  enforcement"). Enforcement-unmeasurable-on-Cerberus is recorded as the
  provenance of that row, and the observation layer (integrity findings)
  stays the source of truth.
- `--output-schema` native structured output where a workflow stage wants
  it (StructuredOutputCapability::NativeSchema with its test).
- Ask/needs_input: measure whether an actor-authored question is
  distinguishable (app-server typed interactions); ask=true ONLY with the
  L8 test; else stays false — never prose-guessed.

### W4 — fake fidelity, doctrine, finalize
- Fake backend learns the measured shapes (R-H0-7, list from the packet):
  deferred finish; never-arriving terminal (SIGKILL → permanent
  inProgress) reconciled fail-closed; distinct interrupted terminal;
  multi-item turn with queued input; uncorroborated narration. Suite
  expresses each failure mode deterministically.
- Doctrine: adapter ADR (harness/backend axes, default-to-launching-
  harness, version-provenance posture — supersedes the refusal language);
  NORTH-STAR dated amendment (W3's); D6 closes (deviation resolved —
  #25 closes with the adapter); version-policy gains the R-H0-6
  provenance vocabulary; docs/DEVELOPMENT.md backend list updated.
- Finalize: version bump (proposed **0.2.0** — "sergeant speaks codex" is
  a minor-worthy milestone; owner ratifies at head PR), CHANGELOG,
  README (backend selection + sgt codex quickstart lines), retro,
  cleanup, head PR un-drafted. Proof: the m10-style codex contract suite
  green against the installed harness ON the PR, plus a live
  `sgt run --backend codex` Work against a scratch estate recorded in
  the PR body (bounded, luna).

## Ratify-at-review (owner, at head PR)
1. Version 0.2.0 vs 0.1.4.
2. The exact default-resolution precedence (flag > session harness >
   manifest > engine default) as specced by W2.
3. W3's NORTH-STAR amendment text.
4. Which W3 capabilities shipped vs fell back (each with its measured
   reason).

## Risks
- **Luna-tier flakiness in contract tests** (narration caveat): tests
  assert structured items and terminals, never prose content; retries
  bounded; any test that would pass on narration alone is a test-honesty
  panel finding.
- **app-server `[experimental]` drift**: protocol fingerprint recorded at
  admission (generate-json-schema hash); a fingerprint change surfaces in
  the probe as provenance-stale, not a crash.
- **Auth expiry mid-Work**: measured behavior (no self-refresh under
  exec) → adapter maps it to an honest failed terminal naming auth as
  the reason; never a hang (exec cannot hang — measured).
- **tmpfs**: builds in `/var/tmp/codex-impl/` only.
- Live-spend discipline: contract tests and the W4 proof Work run
  bounded, luna-pinned; no unbounded live loops in CI (codex tests are
  harness-gated like claude's, not required PR checks on runners without
  codex).

## Panel amendments (binding — 2026-08-21 panel, 5 confirmed / 1 refuted)

**A1 (MAJOR ×3, converged) — W2's selection/harness machinery already
ships; de-scope to registration + verification.** `sgt codex`
(cli.rs:1326), `sgt run --backend` (cli.rs:127), the four-tier precedence
(`router.rs` `Explicit > OriginAffinity > WorkspaceDefault >
GlobalDefault`, tested with "codex" as a fixture name), the
`SGT_ORIGIN_CLIENT` env var set by every `sgt <harness>` launch, and
`route_source` journaling ALL exist on main. The ONLY gap is
daemon.rs:742-744 refusing to register codex per D6. W2's bullets become:
(a) register the W1 adapter in the BackendRegistry under "codex";
(b) verify with tests, not new code, that `--backend codex` and
origin-affinity-from-`sgt codex` resolve through the existing chain.
**The invented `SGT_HARNESS` env var is struck** — `SGT_ORIGIN_CLIENT`
already carries exactly that signal; a second axis would be undefined
duplication. Ratify-at-review item 2 is REWORDED: the precedence is
shipped and test-ratified, not newly specced; the owner is asked only to
NOTICE that the shipped ordering puts session-harness affinity above the
estate default_backend — which matches the ruling ("default to the
harness you launched with") — not to ratify new design.

**A2 (MAJOR) — the R1 strike is surgical: version-comparison branch
ONLY.** The refusal site bundles three conditions
(claude.rs:1030-1086; test a7 in tests/m4_backends.rs asserts all three):
(1) version below MIN_TRUSTED_VERSION — STRUCK per R1, becomes
provenance report; (2) required launch flag missing from --help — STAYS
(the launch grammar was never verified against this CLI; launching anyway
is not a version-policy question, it is launching unmeasured grammar);
(3) unparseable version string — STAYS (an unmeasurable CLI). W2's spec
names the exact branch and the exact assertion to rewrite; the other two
`expect_err` assertions are untouched. A blind "rewrite the refusal test"
instruction is forbidden.

**A3 (MAJOR) — live-spend gating on contract tests.** W1's citation is
corrected: the precedent is tests/m4_backends.rs's live_gate /
claude_live_enabled opt-in pattern (NOT m10_harness.rs). Codex contract
tests require an explicit opt-in env (spec names it, e.g.
SERGEANT_CODEX_TESTS=1) AND a probe/auth precheck, on top of
binary-presence gating — otherwise every routine `cargo test` on a
codex-logged-in host (Cerberus itself) is unbounded live spend. All live
tests pin `-m gpt-5.6-luna`.
