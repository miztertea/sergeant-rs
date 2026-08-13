export const meta = {
  name: 'mvp4-stabilize',
  description: 'MVP-4 STABILIZE (token-free half): perf re-baseline, coverage, #45 flake, #22 workspace edges — S-series pattern on the assembled product. Real #19 soak is owner-gated, NOT here.',
  phases: [
    { title: 'Measure', detail: 'perf re-baseline + coverage on the assembled binary (fake backend, no tokens)' },
    { title: 'Harden', detail: '#45 flake fix + #22 workspace-edge tests' },
    { title: 'Panel', detail: 'Opus test-honesty critic over the new tests + a sonnet fixer' },
  ],
}
const REPO = '/home/miztertea/sergeant-rs'
const BASE = '563ed23'
const G = `You are stabilizing sergeant-rs MVP-4 on Cerberus, branch cerberus/mvp-1. Binding: ${REPO}/docs/gauntlet/notes/mvp-bucketing-2026-08-11.md MVP-4 section, ${REPO}/docs/gauntlet/contracts/P1-PERF.md (perf method), ${REPO}/docs/perf/baseline-cerberus-2026-08-11.md, ${REPO}/CLAUDE.md, ${REPO}/LESSONS.md, ${REPO}/docs/environments/cerberus.md. PATH="$HOME/.cargo/bin:$PATH" everywhere. SCRATCH DISCIPLINE: write only in repo / session scratchpad / /tmp, NEVER ~/; exit-check ls ~ shows no new entries. Gates: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test. L7 pins; L10 separable commits; DataDir guard; commits end "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"; Fixes #NN on fully-closing commits. Do NOT push. NO real-Claude spend — everything here is fake backend or pure code; the real soak is a separate owner-gated step, do not run it.`
const BUILD = { type: 'object', required: ['commits', 'tests_added', 'gates_green', 'findings', 'notes'], properties: { commits: { type: 'array', items: { type: 'string' } }, tests_added: { type: 'number' }, gates_green: { type: 'boolean' }, findings: { type: 'string', description: 'measured numbers / what was found' }, notes: { type: 'string' } } }

phase('Measure')
const measure = await agent(`${G}
Re-baseline the assembled product (all of MVP-1..3 landed): run scripts/perf/run-all.sh against target/release/sgt (cargo build --release first) into a scratch outdir, compare to docs/perf/baseline-cerberus-2026-08-11.md's R-N0-4 budgets (burst-50 >=28 works/s, rebuild >=15k ev/s, single-submit p50 <=50ms, journal O(1)/stage) — the new manifest/envelope/Docker code must not have regressed them. Write docs/perf/baseline-mvp-2026-08-12.md (same format) with the fresh numbers + a budget-compliance table. Also measure coverage if the harness (scripts/coverage/) runs cleanly; record the line number honestly, no chasing. Commit the baseline doc. Report the budget table in findings. This phase spends ZERO tokens (fake backend).`,
  { label: 'measure', phase: 'Measure', model: 'sonnet', effort: 'high', schema: BUILD })
log(`measure: ${measure ? measure.findings.slice(0, 120) : 'FAILED'}`)
if (!measure || !measure.gates_green) throw new Error('measure phase not green')

phase('Harden')
const harden = await agent(`${G}
Two hardening items:
1. #45 (gh issue view 45): the m6 dropped-daemon composition flake — daemon dead with runtime.json still on disk under heavy load. Root-cause from the issue + the #26 startup-window class it may belong to; fix the race or, if it is genuinely environment-only, probe-gate it loudly (SKIPPED-ENV) with the measured evidence. Pin whatever you conclude. Fixes #45 if closed.
2. #22 (gh issue view 22): workspace discovery edge cases untested — nested repos, submodules, paths with spaces, repo-inside-a-worktree. Now especially load-bearing given MVP-1's estate-discovery-past-inner-.git (R-MVP1-12). Add the fixture tests; fix any real bug they surface. Fixes #22 if fully covered.
Repeated-run gate: run the affected suites 10x to confirm no residual flake (L7 corollary).`,
  { label: 'harden', phase: 'Harden', model: 'sonnet', effort: 'high', schema: BUILD })
log(`harden: ${harden ? harden.commits.length + ' commits, ' + harden.tests_added + ' tests' : 'FAILED'}`)
if (!harden || !harden.gates_green) throw new Error('harden phase not green')

phase('Panel')
const crit = await agent(`Blind test-honesty critic, sergeant-rs MVP-4 stabilization. Read-only in ${REPO} (cargo test allowed, PATH="$HOME/.cargo/bin:$PATH"; NEVER edit). Read the MVP-4 bucketing section, GAUNTLET register (L3), LESSONS. Diff: git diff ${BASE}..HEAD. Grade: does the perf doc's numbers trace to a real run (not copied from the old baseline)? Do #45/#22's new tests actually exercise the race/edges or mirror the fix? Would each pin fail on revert (L7)? Is any probe-gate honest (real SKIPPED-ENV, not a silent skip)? Is the coverage number measured or asserted? Findings with file:line evidence.`,
  { label: 'critic:test-honesty', phase: 'Panel', model: 'opus', effort: 'high', schema: { type: 'object', required: ['findings', 'summary'], properties: { summary: { type: 'string' }, findings: { type: 'array', items: { type: 'object', required: ['id', 'severity', 'claim', 'evidence'], properties: { id: { type: 'string' }, severity: { type: 'string', enum: ['error', 'warning', 'info'] }, claim: { type: 'string' }, evidence: { type: 'string' } } } } } } })
const errs = crit ? crit.findings.filter(f => f.severity !== 'info') : []
log(`panel: ${crit ? crit.findings.length : 0} findings, ${errs.length} actionable`)
let fix = null
if (errs.length) {
  fix = await agent(`${G}\nMVP-4 fixer. Close every error/warning finding below; a finding you reject gets a recorded reason. Ledger entry for the pass.\nFINDINGS: ${JSON.stringify(errs, null, 2)}`,
    { label: 'fixer', phase: 'Panel', model: 'sonnet', effort: 'high', schema: { type: 'object', required: ['commits', 'gates_green', 'notes'], properties: { commits: { type: 'array', items: { type: 'string' } }, gates_green: { type: 'boolean' }, notes: { type: 'string' } } } })
}
return { measure: measure.findings, harden: { commits: harden.commits, tests: harden.tests_added }, panel: { findings: crit ? crit.findings.length : 0, actionable: errs.length }, fix: fix ? fix.commits : 'none', soak_note: 'Real #19 soak NOT run — owner-gated spend decision; token-free stabilization only.' }
