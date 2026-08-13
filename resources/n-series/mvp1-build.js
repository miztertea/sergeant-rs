export const meta = {
  name: 'mvp1-build',
  description: 'MVP-1 CORE build per the adjudicated contract: instrument-first, three Sonnet build lanes, Opus panel + refuter + independent prober, fixer',
  phases: [
    { title: 'Instrument', detail: 'fake-backend deferred-finish knob first (R-MVP1-8) + the --repo measurement task (R-MVP1-5)' },
    { title: 'Build', detail: 'lane A: manifest+schema+discovery; lane B: envelope+ceiling+preflight; lane C: pointer+eviction+exit-doors' },
    { title: 'Panel', detail: 'Opus critics: invariants + test-honesty, contract itself in scope (L9/L19)' },
    { title: 'Verify', detail: 'batched refuter + independent prober (guard maps)' },
    { title: 'Fix', detail: 'fixer over confirmed findings + survivors' },
  ],
}
const REPO = '/home/miztertea/sergeant-rs'
const G = `You are building sergeant-rs MVP-1 on host Cerberus, branch cerberus/mvp-1. Read IN ORDER, binding: ${REPO}/docs/gauntlet/contracts/MVP-1.md (THE contract — your ruling numbers are cited below), ${REPO}/NORTH-STAR.md, ${REPO}/CLAUDE.md, ${REPO}/GAUNTLET.md register, ${REPO}/LESSONS.md, ${REPO}/docs/environments/cerberus.md (cargo needs PATH="$HOME/.cargo/bin:$PATH" in every shell). Gates before done: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test. DataDir guard for daemon tests; every fix pinned (L7); separable commits (L10); guard_map per new test (an independent prober executes them — never self-probe as verification, L13); fail closed on ambiguity. Commits end "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>". Do NOT push. Leak check: pgrep -af "sergeant-rs/target/debug/sgt [-]-data-dir" empty.`
const BUILD = { type: 'object', required: ['commits', 'tests_added', 'gates_green', 'design_decisions', 'guard_map', 'notes'], properties: { commits: { type: 'array', items: { type: 'string' } }, tests_added: { type: 'number' }, gates_green: { type: 'boolean' }, design_decisions: { type: 'array', items: { type: 'string' } }, guard_map: { type: 'array', items: { type: 'object', required: ['test', 'guarded_behavior', 'mutation_that_should_kill_it'], properties: { test: { type: 'string' }, guarded_behavior: { type: 'string' }, mutation_that_should_kill_it: { type: 'string' } } } }, notes: { type: 'string' } } }
const FINDINGS = { type: 'object', required: ['findings', 'summary'], properties: { summary: { type: 'string' }, findings: { type: 'array', items: { type: 'object', required: ['id', 'severity', 'claim', 'evidence'], properties: { id: { type: 'string' }, severity: { type: 'string', enum: ['error', 'warning', 'info'] }, claim: { type: 'string' }, evidence: { type: 'string' } } } } } }

phase('Instrument')
const inst = await agent(`${G}
Scope — the contract's two preconditions, in one lane:
1. R-MVP1-8: the fake backend deferred-finish knob (settle-delay, k=0 default so zero existing tests change) — the contract's Instrument section is exact. The fake currently has NO in-flight turn (fake.rs:669-717); build the knob so turn-boundary tests can exist at all.
2. R-MVP1-5's measurement task: measure what multi-repo --repo binding DOES today (real daemon, two scratch repos, one work, --repo twice): surface layout, execution cwd, workflow.bound content, per-repo policy reachability. Write the measured facts as docs/gauntlet/notes/multirepo-measurement-2026-08-11.md (this gates lane A's group-expansion pin and MVP-2's projection work).`,
  { label: 'instrument', phase: 'Instrument', model: 'sonnet', effort: 'high', schema: BUILD })
if (!inst || !inst.gates_green) throw new Error('instrument lane not green — stop')
log(`instrument: ${inst.commits.length} commits`)

phase('Build')
const lanes = [
  { key: 'A-manifest', brief: `Lane A — estate vocabulary. Rulings R-MVP1-3 (schema rename-with-refusal: estate/repo/profile; legacy keys raise the NAMED migration refusal; every fixture migrates in the same commit, grep gate), R-MVP1-12 (estate discovery: upward walk past inner .git bounded at $HOME, [estate]-keyed; #22 fixtures), R-MVP1-4 (instruction projection: manifest declares, core resolves+pins at bind — widen the workflow.bound workspace pin to resolved specs additive-only; one-process-one-policy, multi-repo disagreement fails closed at submit; "local" parses+pins but is REFUSED at submit until MVP-2 measures, L1), R-MVP1-1 (surfaces_root split, default <data_dir>/surfaces so nothing moves). Files: src/domain/workspace.rs, src/runtime/{surface,engine}.rs bind path, src/cli.rs, tests/m1/m3. Exclusive surface: domain/config + bind path.` },
  { key: 'B-envelope', brief: `Lane B — bounds. Rulings R-MVP1-7 (turn envelope: gates launch AND send, counts SPAWNED TURNS from the journal not verb calls — send-retry double-count is the named trap, engine.rs:258-266; blocks at N+1 with a structured refusal; per-turn wall-clock ceiling riding the 200ms completion driver; L16 arithmetic in the docs), R-MVP1-11 (submit-time capability preflight: new requires_ask stage declaration + refusal when the resolved backend's ask capability is low; honest-bound doc per the contract), R-MVP1-6 (intent schema: five optional fields, free text primary, additive-only, journaled+pinned; NO dedup reservation). Files: src/domain/{work,workflow}.rs, src/runtime/engine.rs launch/send seams, src/api.rs submit path, tests/m2/m4. Exclusive surface: envelope + submit validation.` },
  { key: 'C-legibility', brief: `Lane C — legibility + lifecycle. Rulings R-MVP1-2 as held (output POINTER is core: terminal work surfaces branch + retained paths in work show/API — read them from the teardown records that already exist, surface.rs; finalize EXECUTION stays workflow content — build the shared deterministic finalize helper as .sergeant/ content invoked by closing stages, engine untouched), R-MVP1-9 (Rule A eviction: terminal works evict projection state, re-derive on access; s2-churn slope flat + byte-identical evicted views), R-MVP1-10 (blocked exit-door invariant: for EVERY fault-landing state, inject the fault and prove the operator exit; SKIPPED-ENV where the host can't express the fault; L6 review clause). Files: src/runtime/{projection,recovery,surface}.rs, src/api.rs views, tests/m2/m5/m6. Exclusive surface: projections + teardown reporting.` },
]
const built = []
for (const l of lanes) {
  const r = await agent(`${G}\n${l.brief}\nRead the instrument lane's commits first (git log). Respect other lanes' exclusive surfaces — shared files (engine.rs) only in your named seams; rebase-friendly small commits.`,
    { label: `build:${l.key}`, phase: 'Build', model: 'sonnet', effort: 'high', schema: BUILD })
  built.push({ lane: l.key, r })
  log(`${l.key}: ${r ? r.commits.length + ' commits, green=' + r.gates_green : 'FAILED'}`)
  if (!r || !r.gates_green) throw new Error(`${l.key} not green — stop for orchestrator`)
}

phase('Panel')
const BASE = 'c955422'
const axes = [
  { key: 'invariants', brief: `Axis INVARIANTS: one owner, journal-only-truth, fail-closed, §22.6 lock discipline, L6 adjacent-append windows across the new bind-pin/envelope/eviction seams; R-MVP1-10's exit-door claims actually hold; the envelope cannot be raced via send-retry; eviction cannot resurrect canceled work; THE CONTRACT ITSELF is in scope (L9/L19) — a ruling the source contradicts is a finding.` },
  { key: 'test-honesty', brief: `Axis TEST-HONESTY: do the new tests use the deferred-finish knob to stand INSIDE turn boundaries, or race sleeps? Are evicted-view comparisons byte-identical or spot-checks? Does the schema-migration grep gate actually gate? Would each pin fail on revert (L7)? Guard maps: are the mutations the RIGHT mutations? Read-only; the prober executes.` },
]
const crits = await parallel(axes.map(a => () => agent(
  `Blind critic, sergeant-rs MVP-1 round 1. Read-only in ${REPO} (cargo test allowed, PATH="$HOME/.cargo/bin:$PATH"; NEVER edit). Read first: docs/gauntlet/contracts/MVP-1.md, GAUNTLET register (L3), LESSONS. Diff: git log --oneline ${BASE}..HEAD; git diff ${BASE}..HEAD. ${a.brief} Findings carry file:line evidence.`,
  { label: `critic:${a.key}`, phase: 'Panel', model: 'opus', effort: 'high', schema: FINDINGS })))
// Label by original index BEFORE dropping nulls: filter(Boolean).flatMap((c, i) => ...)
// compacts a dead critic out and hands the survivor the wrong axes[i].
const all = crits.flatMap((c, i) => (c ? c.findings.map(f => ({ ...f, id: `${axes[i].key}:${f.id}` })) : []))
log(`panel: ${all.length} findings (${all.filter(f => f.severity === 'error').length} error)`)

phase('Verify')
let verdicts = []
if (all.length) {
  const r = await agent(`Batched adversarial refuter, MVP-1. You built nothing. ${REPO}, PATH="$HOME/.cargo/bin:$PATH". CONFIRM/REFUTE/PLAUSIBLE each with evidence; mutation probes ONLY in one disposable worktree (git worktree add /tmp/mvp1-refute HEAD, own CARGO_TARGET_DIR), removed after, main tree verified clean. FINDINGS:\n${JSON.stringify(all, null, 2)}`,
    { label: 'refuter', phase: 'Verify', model: 'opus', effort: 'high', schema: { type: 'object', required: ['verdicts', 'tree_clean'], properties: { tree_clean: { type: 'boolean' }, verdicts: { type: 'array', items: { type: 'object', required: ['id', 'verdict', 'basis'], properties: { id: { type: 'string' }, verdict: { type: 'string', enum: ['CONFIRMED', 'REFUTED', 'PLAUSIBLE'] }, basis: { type: 'string' } } } } } } })
  if (r && r.tree_clean === false) throw new Error('refuter reported tree_clean=false — mutation-probe residue in the main tree (L5); stop for orchestrator before Fix')
  verdicts = r ? r.verdicts : []
}
const guardMap = [inst, ...built.map(b => b.r)].filter(Boolean).flatMap(b => b.guard_map || [])
const probe = guardMap.length ? await agent(`Independent mutation prober (L13), MVP-1. ${REPO}, PATH="$HOME/.cargo/bin:$PATH". df pre-check (abort under 15GB). ONE disposable worktree (git worktree add /tmp/mvp1-probe HEAD, own CARGO_TARGET_DIR=/tmp/mvp1-probe-target, targeted builds). Execute every entry + 3 composition probes of your own (envelope/eviction/bind-pin seams). Remove worktree+target after; main tree clean. GUARD MAP (${guardMap.length}):\n${JSON.stringify(guardMap, null, 2)}`,
  { label: 'prober', phase: 'Verify', model: 'opus', effort: 'high', schema: { type: 'object', required: ['probes_executed', 'kills', 'survivors'], properties: { probes_executed: { type: 'integer' }, kills: { type: 'array', items: { type: 'string' } }, survivors: { type: 'array', items: { type: 'object', required: ['test', 'mutation', 'why_it_survived'], properties: { test: { type: 'string' }, mutation: { type: 'string' }, why_it_survived: { type: 'string' } } } } } } }) : null
const confirmed = verdicts.filter(v => v.verdict === 'CONFIRMED')
const plausible = verdicts.filter(v => v.verdict === 'PLAUSIBLE')
const survivors = probe && probe.survivors ? probe.survivors : []
log(`verify: ${confirmed.length} confirmed / ${plausible.length} plausible; probes ${probe ? probe.probes_executed : 0}, survivors ${survivors.length}`)

phase('Fix')
let fix = null
if (confirmed.length || plausible.length || survivors.length) {
  fix = await agent(`${G}\nMVP-1 fixer. Close every confirmed finding; investigate plausibles (fix or refute with evidence); strengthen each probe survivor until its mutation kills (re-execute in a disposable worktree, own target dir, removed after). Nothing silently dropped.\nCONFIRMED: ${JSON.stringify(confirmed.map(v => ({ ...v, finding: all.find(f => f.id === v.id) })), null, 2)}\nPLAUSIBLE: ${JSON.stringify(plausible.map(v => ({ ...v, finding: all.find(f => f.id === v.id) })), null, 2)}\nSURVIVORS: ${JSON.stringify(survivors, null, 2)}`,
    { label: 'fixer', phase: 'Fix', model: 'sonnet', effort: 'high', schema: { type: 'object', required: ['commits', 'gates_green', 'notes'], properties: { commits: { type: 'array', items: { type: 'string' } }, gates_green: { type: 'boolean' }, notes: { type: 'string' } } } })
}
return { instrument: inst.commits, lanes: built.map(b => ({ lane: b.lane, commits: b.r.commits, tests: b.r.tests_added })), findings: { total: all.length, confirmed: confirmed.length }, probe: probe ? { executed: probe.probes_executed, survivors: survivors.length } : null, fix: fix ? { commits: fix.commits, gates: fix.gates_green } : null }
