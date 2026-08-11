export const meta = {
  name: 'bs2-adapter-settle',
  description: 'Bug Sprint 2: turn-completion settle (#46) + permission-mode profile config (#47), full multi-axis loop round 1',
  phases: [
    { title: 'B1', detail: 'settle seam: turn end drives OBSERVE (contract Outcomes 1-3)', model: 'opus' },
    { title: 'B2', detail: 'permission-mode profile config + a5 probe-gate (Outcomes 4-5)' },
    { title: 'Critics', detail: 'blind panel: invariants + test-honesty, Opus' },
    { title: 'Refute', detail: 'batched adversarial verification, evidence-only' },
    { title: 'Probe', detail: 'independent prober executes the guard maps, one disposable worktree' },
    { title: 'Fix', detail: 'fixer over confirmed findings + probe survivors' },
  ],
}

// args: { base: '<SHA at launch>' }
const REPO = '/home/miztertea/sergeant-rs'
const BASE = args && args.base ? args.base : 'UNSET'

const G = `You are working on sergeant-rs Bug Sprint 2 on host Cerberus (non-root, uid 1001). Read IN ORDER before acting: ${REPO}/docs/gauntlet/contracts/BS2-ADAPTER-SETTLE.md (binding contract), ${REPO}/CLAUDE.md, ${REPO}/GAUNTLET.md deviation register + backlog tables, ${REPO}/LESSONS.md, ${REPO}/docs/gauntlet/runs/runB/run-manifest.md (the measured defect), ${REPO}/docs/environments/cerberus.md (this host's facts). Environment: cargo is NOT on non-interactive PATH — every shell command needs PATH="$HOME/.cargo/bin:$PATH". Repo rules bind: gates before done (cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test), daemon-spawning tests go through tests/support/mod.rs DataDir guard, every fix pinned by a test that fails when reverted (L7), adjacent-append windows analyzed (L6), fail closed on ambiguity, keep fix commits separable from build commits (L10). Commit messages end with "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>". Fixes #NN trailer ONLY on the commit that fully closes that issue. Do NOT push. Do NOT run scripts/gate.sh (the orchestrator runs it). Leak check before finishing: pgrep -f "debug/sgt [-]-data-dir" must find nothing (note the bracket — an unbracketed pattern matches your own shell).`

const BUILD_SCHEMA = {
  type: 'object',
  required: ['commits', 'tests_added', 'gates_green', 'design_decisions', 'guard_map', 'notes'],
  properties: {
    commits: { type: 'array', items: { type: 'string' } },
    tests_added: { type: 'number' },
    gates_green: { type: 'boolean' },
    design_decisions: { type: 'array', items: { type: 'string' }, description: 'each with its Ponytail rung, e.g. "R2: ..."' },
    guard_map: { type: 'array', items: { type: 'object', required: ['test', 'guarded_behavior', 'mutation_that_should_kill_it'], properties: { test: { type: 'string' }, guarded_behavior: { type: 'string' }, mutation_that_should_kill_it: { type: 'string' } } } },
    notes: { type: 'string' },
  },
}

const FINDINGS_SCHEMA = {
  type: 'object',
  required: ['findings', 'summary'],
  properties: {
    summary: { type: 'string' },
    findings: { type: 'array', items: { type: 'object', required: ['id', 'severity', 'claim', 'evidence'], properties: { id: { type: 'string' }, severity: { type: 'string', enum: ['error', 'warning', 'info'] }, claim: { type: 'string' }, evidence: { type: 'string', description: 'file:line + what was actually read/run' } } } },
  },
}

const VERDICTS_SCHEMA = {
  type: 'object',
  required: ['verdicts', 'tree_clean'],
  properties: {
    tree_clean: { type: 'boolean', description: 'git status --short of the main tree is empty of YOUR changes after verification' },
    verdicts: { type: 'array', items: { type: 'object', required: ['id', 'verdict', 'basis'], properties: { id: { type: 'string' }, verdict: { type: 'string', enum: ['CONFIRMED', 'REFUTED', 'PLAUSIBLE'] }, basis: { type: 'string' } } } },
  },
}

const PROBE_SCHEMA = {
  type: 'object',
  required: ['worktree_hygiene', 'probes_executed', 'kills', 'survivors'],
  properties: {
    worktree_hygiene: { type: 'string' },
    probes_executed: { type: 'integer' },
    kills: { type: 'array', items: { type: 'string' } },
    survivors: { type: 'array', items: { type: 'object', required: ['test', 'mutation', 'why_it_survived'], properties: { test: { type: 'string' }, mutation: { type: 'string' }, why_it_survived: { type: 'string' } } } },
    notes: { type: 'array', items: { type: 'string' } },
  },
}

phase('B1')
const b1 = await agent(`${G}
Your scope — contract Outcomes 1, 2, 3 (the settle seam). Design before editing; log every design decision with its rung.
1. Outcome 1: a real-backend turn that finishes after launch-settle gets its observation taken and settled by the daemon with no client call. The observation is taken OFF the core lock and settled through the existing performer/crank machinery (src/api.rs crank + src/runtime/engine.rs drive are the homes; read engine.rs's drive() doc comment on why observe never happens under the guard — m6 t11 and the §22.6 instrumentation enforce it structurally). Settle must be currency-checked per §14.5: late/duplicate observation of a superseded execution/attempt is subordinate to durable state (no double-complete, no resurrecting canceled work, no racing SEND/interrupt settles). The contract names the candidate lower rung (daemon-side completion driver polling active executions' in-memory turn state — ClaudeBackend::observe is an in-memory read) — if you choose higher machinery, name why the poll failed (R7 discipline).
2. Outcome 2: contract tests driving the REAL turn-end path through a spawned daemon (tests/support DataDir guard): a fake-CLI stub binary/script (the S-series pattern — look at how existing m4 tests stub the claude CLI; grep tests/ for existing stub fixtures) whose process OUTLIVES launch-settle, then (a) emits a complete stream-json turn with clean result envelope and exits -> stage completes and cascades with no mutating client call; (b) exits pre-envelope with stderr -> stage lands blocked with the stderr in evidence. Assert from the journal + read-only API views. Polling a read view for the transition is fine; causing the transition with a mutating call is not.
3. Outcome 3: crash-window pin — daemon killed after conversation.turn.ended is journaled but before settle; restart must re-derive the outcome via recovery (never silently active forever). Use the existing crash-injection style from the §22.5 matrix tests.
Primary files: src/api.rs, src/runtime/engine.rs, src/daemon.rs, src/backend/claude.rs (observe path only — B2 owns the launch grammar), tests/m4_backends.rs or tests/m2_daemon_api.rs as fits the suite layout. Return a guard_map entry per new test. Commit "Fixes #46" on exactly the commit that closes the live-settle defect.`,
  { label: 'B1:settle-seam', phase: 'B1', model: 'opus', effort: 'high', schema: BUILD_SCHEMA })
log(`B1: ${b1 ? b1.commits.length + ' commits, ' + b1.tests_added + ' tests, gates=' + b1.gates_green : 'FAILED'}`)
if (!b1 || !b1.gates_green) throw new Error('B1 not green — stop sprint')

phase('B2')
const b2 = await agent(`${G}
B1 just landed the settle seam — read its commits first (git log ${BASE}..HEAD) and do not undo its choices.
Your scope — contract Outcomes 4 and 5.
1. Outcome 4 (#47): permission_mode becomes profile configuration passed through as the CLI's own vocabulary (default | acceptEdits | bypassPermissions | dontAsk | plan). Unspecified -> NO permission flag on the CLI invocation at all (the CLI's own default) — the hardcoded --dangerously-skip-permissions goes away; bypassPermissions requires explicit profile opt-in. Unknown mode strings fail closed at profile load with a structured error. Doctor/capabilities surface the effective mode per profile. Find where profiles are defined (grep src/ for profile config parsing) and follow the existing config plumbing. Tests: flag-construction per mode (deterministic, stub level); plus ONE bounded opt-in real-CLI test (SERGEANT_CLAUDE_TESTS=1, --ignored) pinning what a default-mode headless turn can and cannot do — one haiku turn maximum, measured facts only (L1). Commit "Fixes #47" on the closing commit.
2. Outcome 5: a5 (tests/m4_backends.rs, the opt-in ask test) probe-gates per docs/gauntlet/notes/cerberus-ask-grammar-remeasurement-2026-08-11.md disposition 2: transcript carries post_turn_summary -> assert the NeedsInput mapping; carries none -> assert the withdrawal fired (Capabilities::ask lowered + conversation.turn.grammar_unmeasured journaled) and skip the mapping assertion loudly printing SKIPPED-ENV with the reason. Both arms are measured claims; neither guesses.
Primary files: src/backend/claude.rs (launch grammar, capabilities), profile/config domain files, src/daemon.rs or doctor path, tests/m4_backends.rs. Guard_map per new test. Opt-in tests bill tokens: run the opt-in suite ONCE to verify, report its result in notes.`,
  { label: 'B2:permission-mode', phase: 'B2', model: 'sonnet', effort: 'high', schema: BUILD_SCHEMA })
log(`B2: ${b2 ? b2.commits.length + ' commits, ' + b2.tests_added + ' tests, gates=' + b2.gates_green : 'FAILED'}`)
if (!b2 || !b2.gates_green) throw new Error('B2 not green — stop sprint')

phase('Critics')
const axes = [
  { key: 'invariants', brief: `Axis: INVARIANTS (proposal §40 + N-series two-phase discipline). Grade the diff against: one owner; work state != process state; durable trajectory / journal is the only truth; disposable projections; idempotency; fail-closed ambiguity; §22.6 lock discipline (no external I/O, process wait, or thread join under the core lock — including anything the new settle driver does); §14.5 currency (late/duplicate observations subordinate to durable state — hunt for double-settle, canceled-work resurrection, races between the driver and SEND/interrupt/cancel settles); L6 adjacent-append crash windows (any new causally-linked append pair must tolerate the second missing). Also: does the settle driver change daemon shutdown ordering in a way that touches the #45 flake class (contract Unknowns)?` },
  { key: 'test-honesty', brief: `Axis: TEST-HONESTY. Do the new tests verify the contract's claims or mirror the implementation? Specifically: do the Outcome-2 tests actually drive the real turn-end path through a spawned daemon (a stub process that OUTLIVES launch-settle), or do they sneak the old fake-backend shortcut (turn finishing inside launch) back in? Does any test cause the transition it asserts via a mutating client call? Is the crash-window pin (Outcome 3) actually killing the daemon in the claimed window? Are the a5 probe-gate arms both falsifiable? Would each new test fail if its fix were reverted (L7)? Check the guard maps' mutations are the RIGHT mutations. Do NOT execute mutations in the main tree — reason from reading; the independent prober executes.` },
]
const critiques = await parallel(axes.map(a => () => agent(
  `You are a blind critic for sergeant-rs Bug Sprint 2 — you did not build this; grade the actual artifacts, never a summary. Work in ${REPO} (read-only; you may run cargo test with PATH="$HOME/.cargo/bin:$PATH" but NEVER edit files — any mutation probe belongs to the independent prober, not you). Read first: ${REPO}/docs/gauntlet/contracts/BS2-ADAPTER-SETTLE.md, ${REPO}/GAUNTLET.md deviation register + ledger rulings (L3: a finding that re-litigates a registered deviation must argue the ruling wrong, not that the deviation exists), ${REPO}/LESSONS.md. The diff under review: git log --oneline ${BASE}..HEAD and git diff ${BASE}..HEAD.
${a.brief}
Every finding carries evidence: file:line actually read, test output actually produced. A finding you cannot ground is not filed. Severity: error (contract violated / defect), warning (risk or gap), info (observation).`,
  { label: `critic:${a.key}`, phase: 'Critics', model: 'opus', effort: 'high', schema: FINDINGS_SCHEMA })))
const allFindings = critiques.filter(Boolean).flatMap((c, i) => c.findings.map(f => ({ ...f, id: `${axes[i].key}:${f.id}` })))
log(`Critics: ${allFindings.length} findings (${allFindings.filter(f => f.severity === 'error').length} error)`)

phase('Refute')
let verdicts = []
if (allFindings.length > 0) {
  const r = await agent(
    `You are the batched adversarial refuter for sergeant-rs Bug Sprint 2. You did not build the code and did not write these findings. Work from ${REPO}; diff is git diff ${BASE}..HEAD; PATH="$HOME/.cargo/bin:$PATH" for cargo. For each finding: try to REFUTE it with evidence (read the code, run the tests read-only). Verdict CONFIRMED only when you reproduced/verified the claim yourself; REFUTED with the disproof; PLAUSIBLE when neither (never drop). Probe hygiene is absolute: if a verdict needs a mutation probe, do it in ONE disposable git worktree (git worktree add /tmp/bs2-refute HEAD, own CARGO_TARGET_DIR=/tmp/bs2-refute-target, targeted builds), remove both after, and verify git -C ${REPO} status --short shows no change of yours. FINDINGS:
${JSON.stringify(allFindings, null, 2)}`,
    { label: 'refuter:batch', phase: 'Refute', model: 'opus', effort: 'high', schema: VERDICTS_SCHEMA })
  verdicts = r ? r.verdicts : []
  if (r && r.tree_clean === false) log('WARNING: refuter reports main tree not clean — orchestrator must inspect before anything else lands')
}
const confirmed = verdicts.filter(v => v.verdict === 'CONFIRMED')
const plausible = verdicts.filter(v => v.verdict === 'PLAUSIBLE')
log(`Refute: ${confirmed.length} confirmed, ${plausible.length} plausible, ${verdicts.filter(v => v.verdict === 'REFUTED').length} refuted`)

phase('Probe')
const guardMap = [...(b1.guard_map || []), ...(b2.guard_map || [])]
let probe = null
if (guardMap.length > 0) {
  probe = await agent(
    `You are the independent mutation prober for sergeant-rs Bug Sprint 2 (L13: builders never probe their own pins; you did not write these tests). Work from ${REPO}; PATH="$HOME/.cargo/bin:$PATH". df pre-check: abort if the target filesystem has under 15 GB free. ONE disposable worktree: git worktree add /tmp/bs2-probe HEAD with its OWN target dir (export CARGO_TARGET_DIR=/tmp/bs2-probe-target — NEVER the main checkout's target/, see CLAUDE.md's shared-cache hazard), targeted test builds only. Execute every guard-map entry: apply the mutation, run the named test, record kill or survival, revert clean between probes (git -C /tmp/bs2-probe checkout .). Add 2-3 unlisted composition probes of your own devising (parts-vs-composition is this repo's recurring miss). Afterwards: remove the worktree (git worktree remove --force /tmp/bs2-probe), delete /tmp/bs2-probe-target, verify git -C ${REPO} status --short is empty of your changes. GUARD MAP (${guardMap.length} entries):
${JSON.stringify(guardMap, null, 2)}`,
    { label: 'prober', phase: 'Probe', model: 'opus', effort: 'high', schema: PROBE_SCHEMA })
  log(`Probe: ${probe ? probe.probes_executed + ' executed, ' + probe.survivors.length + ' survivors' : 'FAILED'}`)
}

phase('Fix')
const survivors = probe && probe.survivors ? probe.survivors : []
let fix = null
if (confirmed.length > 0 || survivors.length > 0) {
  fix = await agent(`${G}
You are the Bug Sprint 2 fixer. Inputs are executed evidence and confirmed findings — close every one or record why not (with evidence) in notes; nothing is silently dropped. PLAUSIBLE findings: investigate, then fix or refute with evidence.
CONFIRMED FINDINGS:
${JSON.stringify(confirmed.map(v => ({ ...v, finding: allFindings.find(f => f.id === v.id) })), null, 2)}
PLAUSIBLE:
${JSON.stringify(plausible.map(v => ({ ...v, finding: allFindings.find(f => f.id === v.id) })), null, 2)}
PROBE SURVIVORS (strengthen each test so its mutation kills it; re-execute the mutation in a disposable worktree with its own target dir, removed after):
${JSON.stringify(survivors, null, 2)}
Separable fix commits (L10), each pinned (L7), gates green before returning.`,
    { label: 'fixer', phase: 'Fix', model: 'sonnet', effort: 'high', schema: BUILD_SCHEMA })
  log(`Fix: ${fix ? fix.commits.length + ' commits, gates=' + fix.gates_green : 'FAILED'}`)
}

return {
  b1: { commits: b1.commits, tests: b1.tests_added, decisions: b1.design_decisions },
  b2: { commits: b2.commits, tests: b2.tests_added, decisions: b2.design_decisions },
  findings: { total: allFindings.length, confirmed: confirmed.length, plausible: plausible.length, refuted: verdicts.filter(v => v.verdict === 'REFUTED').length },
  probe: probe ? { executed: probe.probes_executed, kills: probe.kills.length, survivors: survivors.length } : null,
  fix: fix ? { commits: fix.commits, gates: fix.gates_green } : null,
}
