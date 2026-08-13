export const meta = {
  name: 'mvp3-review',
  description: 'MVP-3 review-only (build already committed): 2 opus critics + opus refuter + sonnet prober + sonnet fixer over the committed diff',
  phases: [
    { title: 'Panel', detail: 'Opus critics: invariants + test-honesty' },
    { title: 'Verify', detail: 'opus refuter + sonnet prober' },
    { title: 'Fix', detail: 'sonnet fixer over confirmed + survivors' },
  ],
}
const REPO = '/home/miztertea/sergeant-rs'
const BASE = '3ef5928'  // MVP-3 build lanes are 3ef5928..HEAD, committed by the (now-retired) driver
const FINDINGS = { type: 'object', required: ['findings', 'summary'], properties: { summary: { type: 'string' }, findings: { type: 'array', items: { type: 'object', required: ['id', 'severity', 'claim', 'evidence'], properties: { id: { type: 'string' }, severity: { type: 'string', enum: ['error', 'warning', 'info'] }, claim: { type: 'string' }, evidence: { type: 'string' } } } } } }

phase('Panel')
const axes = [
  { key: 'invariants', brief: `Axis INVARIANTS: R-NS-4 — every new CLI verb adds usability never functionality (no engine semantics smuggled through the CLI); m6 t5's ApiViews pin evolved deliberately or not at all; the estate manifest pens (domain::manifest) honor atomic-write/advisory-lock/fail-closed-parse per docs/gauntlet/notes/estate-manifest-design-2026-08-11.md; drain's event pair respects L6; data-dir discovery cannot escape $HOME or cross into a non-estate; envelope submission flags cannot bypass the R-MVP1-7 counting. THE CONTRACT DOCS are in scope (L9/L19).` },
  { key: 'test-honesty', brief: `Axis TEST-HONESTY: do m8_estate_cli tests drive the real CLI binary through DataDir rigs (not function calls); does the transcript test verify actual blob decode against a journaled turn; do init/repo/group tests exercise atomic-write + lock paths under contention; would each pin fail on revert (L7). Read-only; the prober executes mutations.` },
]
const crits = await parallel(axes.map(a => () => agent(
  `Blind critic, sergeant-rs MVP-3 (CLI). Read-only in ${REPO} (cargo test allowed, PATH="$HOME/.cargo/bin:$PATH"; NEVER edit). Read first: docs/gauntlet/notes/mvp-bucketing-2026-08-11.md MVP-3 section, docs/gauntlet/contracts/MVP-1.md estate rulings, docs/gauntlet/notes/estate-manifest-design-2026-08-11.md, GAUNTLET register (L3), LESSONS. Diff under review: git log --oneline ${BASE}..HEAD; git diff ${BASE}..HEAD. ${a.brief} Findings carry file:line evidence.`,
  { label: `critic:${a.key}`, phase: 'Panel', model: 'opus', effort: 'high', schema: FINDINGS })))
const all = crits.flatMap((c, i) => (c ? c.findings.map(f => ({ ...f, id: `${axes[i].key}:${f.id}` })) : []))
log(`panel: ${all.length} findings (${all.filter(f => f.severity === 'error').length} error)`)

phase('Verify')
let verdicts = []
let treeClean = true
if (all.length) {
  const r = await agent(`Batched adversarial refuter, MVP-3. ${REPO}, PATH="$HOME/.cargo/bin:$PATH". CONFIRM/REFUTE/PLAUSIBLE with evidence; mutations ONLY in one disposable worktree (own CARGO_TARGET_DIR), removed after, main tree verified clean; scratch discipline never ~/. FINDINGS:\n${JSON.stringify(all, null, 2)}`,
    { label: 'refuter', phase: 'Verify', model: 'opus', effort: 'high', schema: { type: 'object', required: ['verdicts', 'tree_clean'], properties: { tree_clean: { type: 'boolean' }, verdicts: { type: 'array', items: { type: 'object', required: ['id', 'verdict', 'basis'], properties: { id: { type: 'string' }, verdict: { type: 'string', enum: ['CONFIRMED', 'REFUTED', 'PLAUSIBLE'] }, basis: { type: 'string' } } } } } } })
  verdicts = r ? r.verdicts : []
  if (r && r.tree_clean === false) { treeClean = false; log('L5 HAZARD: refuter reports main tree dirty') }
}
const confirmed = verdicts.filter(v => v.verdict === 'CONFIRMED')
const plausible = verdicts.filter(v => v.verdict === 'PLAUSIBLE')
log(`verify: ${confirmed.length} confirmed / ${plausible.length} plausible / ${verdicts.filter(v => v.verdict === 'REFUTED').length} refuted`)

phase('Fix')
let fix = null
if (confirmed.length || plausible.length || !treeClean) {
  fix = await agent(`You are the MVP-3 fixer for sergeant-rs. Work in ${REPO} (branch cerberus/mvp-1), PATH="$HOME/.cargo/bin:$PATH". Gates before done: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test. L7 pins; L10 separable commits; scratch discipline never ~/; commits end "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"; do NOT push.
${treeClean ? '' : 'FIRST: refuter reported main tree dirty — verify git status, quarantine any probe residue (L5), report before other fixes. '}Close every confirmed finding; investigate plausibles (fix or refute with evidence). Ledger entry for the pass. Nothing silently dropped.
CONFIRMED: ${JSON.stringify(confirmed.map(v => ({ ...v, finding: all.find(f => f.id === v.id) })), null, 2)}
PLAUSIBLE: ${JSON.stringify(plausible.map(v => ({ ...v, finding: all.find(f => f.id === v.id) })), null, 2)}`,
    { label: 'fixer', phase: 'Fix', model: 'sonnet', effort: 'high', schema: { type: 'object', required: ['commits', 'gates_green', 'notes'], properties: { commits: { type: 'array', items: { type: 'string' } }, gates_green: { type: 'boolean' }, notes: { type: 'string' } } } })
}
return { findings: { total: all.length, confirmed: confirmed.length, plausible: plausible.length }, fix: fix ? { commits: fix.commits, gates: fix.gates_green } : 'no fixes needed' }
