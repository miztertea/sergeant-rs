export const meta = {
  name: 'mvp2-build',
  description: 'MVP-2 ADAPTERS per the adjudicated N4 contract + bucketing: Docker executor, Claude de-leak/provenance/usage, execute-stage content, fake fidelity',
  phases: [
    { title: 'Build', detail: 'lane D1 Docker executor (N4); lane D2 Claude adapter items; lane D3 execute content + fake fidelity' },
    { title: 'Panel', detail: 'Opus critics: invariants (§22.5/22.6 over Docker lifecycle) + test-honesty (L8 probe-gating)' },
    { title: 'Verify', detail: 'batched refuter + independent prober' },
    { title: 'Fix', detail: 'fixer over confirmed + survivors' },
  ],
}
// args: { base: '<git sha>' } — REQUIRED. The Panel phase diffs base..HEAD;
// launching without it throws before any agent spends a token.
const REPO = '/home/miztertea/sergeant-rs'
const G = `You are building sergeant-rs MVP-2 on Cerberus, branch cerberus/mvp-1. Binding, read in order: ${REPO}/docs/gauntlet/contracts/N4.md (incl. the 2026-08-12 ADJUDICATION section — Rule A is NOT in scope), ${REPO}/docs/gauntlet/notes/mvp-bucketing-2026-08-11.md MVP-2 table, ${REPO}/NORTH-STAR.md, ${REPO}/CLAUDE.md, ${REPO}/LESSONS.md (L1/L6/L7/L8/L16), ${REPO}/docs/environments/cerberus.md (Docker facts measured; cargo needs PATH="$HOME/.cargo/bin:$PATH"). Gates: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test. DataDir guard; L7 pins; L10 separable commits; guard_map per test (independent prober executes); §22.7 fixtures PROBE-GATE per the environments matrix (full matrix on this host; SKIPPED-ENV where a host can't express a shape). Commits end "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"; Fixes #NN only on fully-closing commits (#23 candidate in D1). Do NOT push. Leak checks: bracketed pgrep for sgt AND docker ps -a scoped to sergeant-owned names (§16.10) empty after suites.`
const BUILD = { type: 'object', required: ['commits', 'tests_added', 'gates_green', 'design_decisions', 'guard_map', 'notes'], properties: { commits: { type: 'array', items: { type: 'string' } }, tests_added: { type: 'number' }, gates_green: { type: 'boolean' }, design_decisions: { type: 'array', items: { type: 'string' } }, guard_map: { type: 'array', items: { type: 'object', required: ['test', 'guarded_behavior', 'mutation_that_should_kill_it'], properties: { test: { type: 'string' }, guarded_behavior: { type: 'string' }, mutation_that_should_kill_it: { type: 'string' } } } }, notes: { type: 'string' } } }
const FINDINGS = { type: 'object', required: ['findings', 'summary'], properties: { summary: { type: 'string' }, findings: { type: 'array', items: { type: 'object', required: ['id', 'severity', 'claim', 'evidence'], properties: { id: { type: 'string' }, severity: { type: 'string', enum: ['error', 'warning', 'info'] }, claim: { type: 'string' }, evidence: { type: 'string' } } } } } }

const BASE = args && args.base ? args.base : null
if (!BASE) throw new Error('mvp2-build requires args.base (the git diff base sha for the Panel phase) — refused up front, before the Build lanes spend anything')

phase('Build')
const lanes = [
  { key: 'D1-docker', brief: `Lane D1 — the Docker executor, N4's contract verbatim: kind="execute" schema activation (§11.2/§12.3), local Engine client + capability probe (§16.1-16.3), image resolution/digest pinning + immutable identity (§16.4), worktree mount contract (§16.6), default no-network/no-privilege (§16.7), streaming bounded capture (§16.9, §22.8's 64MiB RSS budget + Rule B disk measurement), exit-to-stage mapping (§15), exact cancel/retry/recovery/cleanup (§16.10-16.12), lifecycle events + projections (§19.4), doctor rows incl. the #23 disk-pressure check (Fixes #23 on that commit). §22.5 crash windows over create/start/exit/cancel; §22.6 no Engine call under the core lock (the settle driver + performer pattern is the home). The mixed actor→execute→actor proof workflow runs on the fake actor + real Docker.`, effort: 'high' },
  { key: 'D2-claude', brief: `Lane D2 — Claude adapter items: (1) --setting-sources de-leak: manifest instructions policy (pinned at bind by MVP-1's R-MVP1-4) translates at launch — "suppress" keeps today's flag, "local" MEASURED FIRST (L1: one real bounded haiku turn in a scratch repo with project memory present; record what the CLI actually loads with/without the flag; then implement translation + un-refuse "local" at submit); mixed-policy refusal already lands at MVP-1's submit path — contract-test the launch side. (2) Capability provenance (cv2 item 7): measured capability facts (the 2.1.227 ask withdrawal) persist durably (journaled per adapter+version) and feed the R-MVP1-11 preflight so a withdrawn capability refuses at submit on the NEXT work. (3) Partial-usage on interrupt: measure what the CLI emits on kill (one bounded turn, interrupt it); document the fact in docs/environments/ + adapter docs; no promises beyond measurement. Small real-token budget: ≤$2 total, log spend in notes.`, effort: 'high' },
  { key: 'D3-content', brief: `Lane D3 — execute-stage content + fake fidelity: (1) give ONE workflow a real execute stage per N4's adjudicated criterion (cheap/fast/constant — leading: a validate-structure.py run; measure its container runtime, pick with evidence, wire the stage per the ICM convention with declared inputs/outputs); (2) R-H0-7 full fake-backend fidelity review: beyond MVP-1's settle-delay knob — scriptable timing shapes, deferred terminal signals, interrupt behavior matching the §15 trait's real semantics; document what the fake can now express and what it still cannot (honest register in the module docs).`, effort: 'high' },
]
const built = []
for (const l of lanes) {
  const r = await agent(`${G}\n${l.brief}\nRead prior lanes' commits first (git log). Exclusive surfaces; shared files only in named seams.`,
    { label: `build:${l.key}`, phase: 'Build', model: 'sonnet', effort: l.effort, schema: BUILD })
  built.push({ lane: l.key, r })
  log(`${l.key}: ${r ? r.commits.length + ' commits, green=' + r.gates_green : 'FAILED'}`)
  if (!r || !r.gates_green) throw new Error(`${l.key} not green — stop for orchestrator`)
}

phase('Panel')
const axes = [
  { key: 'invariants', brief: `Axis INVARIANTS: §22.5 crash windows actually injected over the Docker lifecycle; §22.6 lock discipline (no Engine/socket call under the guard — check the performer wiring); container ownership exactness (§16.10-16.12: no leaked containers after clean AND crash runs — verify the harness sweep); image identity immutability; the de-leak translation defines nothing (core policy pinned at bind, adapter translates); provenance events respect journal-only-truth; L6 on every new append pair. THE CONTRACT (N4 + its adjudication) is in scope (L9/L19).` },
  { key: 'test-honesty', brief: `Axis TEST-HONESTY: L8 — every advertised Docker capability has a contract test against the real engine on THIS host, probe-gated for hosts without; do §22.8's budget tests actually capture 1GiB (not a token stream); does the de-leak's "local" test measure the real CLI behavior or assume it; do fake-fidelity tests exercise the new shapes or mirror the implementation; would each pin fail on revert; are the guard-map mutations right. Read-only.` },
]
const crits = await parallel(axes.map(a => () => agent(
  `Blind critic, sergeant-rs MVP-2 round 1. Read-only in ${REPO} (cargo test + docker allowed; NEVER edit). Read first: contracts/N4.md + adjudication, GAUNTLET register (L3), LESSONS. Diff: git diff ${BASE}..HEAD. ${a.brief} Findings carry file:line evidence.`,
  { label: `critic:${a.key}`, phase: 'Panel', model: 'opus', effort: 'high', schema: FINDINGS })))
// Label by original index BEFORE dropping nulls: filter(Boolean).flatMap((c, i) => ...)
// compacts a dead critic out and hands the survivor the wrong axes[i].
const all = crits.flatMap((c, i) => (c ? c.findings.map(f => ({ ...f, id: `${axes[i].key}:${f.id}` })) : []))
log(`panel: ${all.length} findings (${all.filter(f => f.severity === 'error').length} error)`)

phase('Verify')
let verdicts = []
if (all.length) {
  const r = await agent(`Batched adversarial refuter, MVP-2. ${REPO}, PATH="$HOME/.cargo/bin:$PATH". CONFIRM/REFUTE/PLAUSIBLE with evidence; mutations ONLY in one disposable worktree (own CARGO_TARGET_DIR), removed after, main tree verified clean. FINDINGS:\n${JSON.stringify(all, null, 2)}`,
    { label: 'refuter', phase: 'Verify', model: 'opus', effort: 'high', schema: { type: 'object', required: ['verdicts', 'tree_clean'], properties: { tree_clean: { type: 'boolean' }, verdicts: { type: 'array', items: { type: 'object', required: ['id', 'verdict', 'basis'], properties: { id: { type: 'string' }, verdict: { type: 'string', enum: ['CONFIRMED', 'REFUTED', 'PLAUSIBLE'] }, basis: { type: 'string' } } } } } } })
  if (r && r.tree_clean === false) throw new Error('refuter reported tree_clean=false — mutation-probe residue in the main tree (L5); stop for orchestrator before Fix')
  verdicts = r ? r.verdicts : []
}
const guardMap = built.map(b => b.r).flatMap(b => b.guard_map || [])
const probe = guardMap.length ? await agent(`Independent mutation prober (L13), MVP-2. ${REPO}. df pre-check (abort under 15GB). ONE disposable worktree, own CARGO_TARGET_DIR, targeted builds; execute every entry + 3 composition probes on the Docker lifecycle seams; docker hygiene: any container you create is sergeant-test-named and removed. Worktree+target removed after; main tree clean. GUARD MAP (${guardMap.length}):\n${JSON.stringify(guardMap, null, 2)}`,
  { label: 'prober', phase: 'Verify', model: 'opus', effort: 'high', schema: { type: 'object', required: ['probes_executed', 'kills', 'survivors'], properties: { probes_executed: { type: 'integer' }, kills: { type: 'array', items: { type: 'string' } }, survivors: { type: 'array', items: { type: 'object', required: ['test', 'mutation', 'why_it_survived'], properties: { test: { type: 'string' }, mutation: { type: 'string' }, why_it_survived: { type: 'string' } } } } } } }) : null
const confirmed = verdicts.filter(v => v.verdict === 'CONFIRMED')
const plausible = verdicts.filter(v => v.verdict === 'PLAUSIBLE')
const survivors = probe && probe.survivors ? probe.survivors : []
log(`verify: ${confirmed.length}/${plausible.length}/${survivors.length} confirmed/plausible/survivors`)

phase('Fix')
let fix = null
if (confirmed.length || plausible.length || survivors.length) {
  fix = await agent(`${G}\nMVP-2 fixer. Close every confirmed; investigate plausibles; strengthen survivors (re-execute mutations in a disposable worktree). Nothing silently dropped; ledger entry for the pass.\nCONFIRMED: ${JSON.stringify(confirmed.map(v => ({ ...v, finding: all.find(f => f.id === v.id) })), null, 2)}\nPLAUSIBLE: ${JSON.stringify(plausible.map(v => ({ ...v, finding: all.find(f => f.id === v.id) })), null, 2)}\nSURVIVORS: ${JSON.stringify(survivors, null, 2)}`,
    { label: 'fixer', phase: 'Fix', model: 'sonnet', effort: 'high', schema: { type: 'object', required: ['commits', 'gates_green', 'notes'], properties: { commits: { type: 'array', items: { type: 'string' } }, gates_green: { type: 'boolean' }, notes: { type: 'string' } } } })
}
return { lanes: built.map(b => ({ lane: b.lane, commits: b.r.commits, tests: b.r.tests_added })), findings: { total: all.length, confirmed: confirmed.length }, probe: probe ? { executed: probe.probes_executed, survivors: survivors.length } : null, fix: fix ? { commits: fix.commits, gates: fix.gates_green } : null }
