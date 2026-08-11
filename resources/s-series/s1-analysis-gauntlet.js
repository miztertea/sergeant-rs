export const meta = {
  name: 's1-analysis-gauntlet',
  description: 'S1 phases 2/3 support: Sonnet artifact analyzers per module group, batched Opus reproduce-or-refute over candidate findings',
  phases: [
    { title: 'Analyze', detail: 'Sonnet runners parse lcov/summary artifacts into candidate findings' },
    { title: 'Verify', detail: 'Batched Opus refuters: reproduce-or-refute each candidate against the tree' },
  ],
}

// args: { artifacts: '<dir with summary.txt + lcov.info + census logs>', sha: '<phase-1 SHA>' }
const REPO = '/home/user/sergeant-rs'
const ART = args && args.artifacts ? args.artifacts : 'docs/coverage/artifacts-2026-08-10'
const SHA = args && args.sha ? args.sha : 'UNSET'

const CANDIDATES_SCHEMA = {
  type: 'object',
  properties: {
    candidates: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          module: { type: 'string' },
          behavior: { type: 'string' },
          category: {
            type: 'string',
            enum: ['untested-error-path', 'dead-code-candidate', 'race-window', 'missing-fixture', 'artifact-of-instrument', 'flake'],
          },
          evidence: { type: 'string' },
          suggested_test_shape: { type: 'string' },
        },
        required: ['module', 'behavior', 'category', 'evidence'],
      },
    },
    notes: { type: 'array', items: { type: 'string' } },
  },
  required: ['candidates'],
}

const VERDICTS_SCHEMA = {
  type: 'object',
  properties: {
    verdicts: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          index: { type: 'integer' },
          refuted: { type: 'boolean' },
          reason: { type: 'string' },
        },
        required: ['index', 'refuted', 'reason'],
      },
    },
  },
  required: ['verdicts'],
}

const GROUPS = [
  { key: 'runtime-core', paths: 'src/runtime/{journal,projection,blob,fsutil,git}.rs' },
  { key: 'engine-recovery', paths: 'src/runtime/{engine,recovery,router,surface}.rs + src/domain/**' },
  { key: 'analytics-graph-telemetry', paths: 'src/runtime/{analytics,graph}.rs + src/telemetry.rs' },
  { key: 'daemon-api-cli', paths: 'src/{daemon,api,cli}.rs' },
  { key: 'clients-backends', paths: 'src/{tui,web}.rs + src/backend/**' },
]

const RULES = `Binding context: docs/gauntlet/contracts/S1-COVERAGE.md (findings discipline),
proposal sections 6.2 (pre-ruled regions: the B1 snapshot path and Analytics::table_rows are
NOT filable; claude.rs live-path-only regions are handed to N2, not filed) and 13 (non-goals).
A candidate names an unexercised BEHAVIOR with region evidence from the artifacts — never a
bare line range. Read-only: never modify the tree; the measured SHA is ${SHA}.`

phase('Analyze')
const analyses = await parallel(
  GROUPS.map((g) => () =>
    agent(
      `You are an S1 coverage analyst. Work read-only in ${REPO}. Artifacts from the
phase-2 measurement run are in ${ART} (summary table, lcov.info, census logs, harness
README with measured tool semantics). ${RULES}

Your module group: ${g.key} (${g.paths}). From the lcov/summary evidence, identify the
uncovered or weakly-covered REGIONS in your group, read the actual source to translate
regions into unexercised behaviors, classify each candidate, and propose the smallest
honest test shape (YAGNI-first: prefer an in-module cfg(test) poke or an existing
harness trick; a candidate needing new production seams gets flagged as such). Also
sweep the census logs for failures attributed to tests in your group. Known-loss sites
and pre-ruled regions are notes, not candidates. Empty candidate lists are valid.`,
      { label: `analyze:${g.key}`, model: 'sonnet', effort: 'medium', phase: 'Analyze', schema: CANDIDATES_SCHEMA },
    ).then((r) => ({ group: g.key, r })),
  ),
)

const candidates = analyses
  .filter(Boolean)
  .flatMap((a) => (a.r ? a.r.candidates.map((c) => ({ ...c, group: a.group })) : []))
log(`analysis: ${candidates.length} candidates across ${GROUPS.length} groups`)

phase('Verify')
let confirmed = []
if (candidates.length > 0) {
  const byGroup = {}
  candidates.forEach((c) => { (byGroup[c.group] = byGroup[c.group] || []).push(c) })
  const batches = await parallel(
    Object.entries(byGroup).map(([key, cs]) => () =>
      agent(
        `Adversarially verify S1 coverage candidates (group: ${key}) in ${REPO}. ${RULES}
For EACH candidate, try to REFUTE it: is the behavior actually exercised (search the
suites and unit tests for it — coverage regions lie when a behavior is tested through a
different module); is it a pre-ruled or known-loss region; is it an
artifact-of-instrument; is the claimed behavior real code or unreachable; does a claimed
flake reproduce (you may run a single targeted test up to 3 times; NEVER a full suite,
NEVER a mutation of the tree). Default refuted=true when evidence is unclear.
Candidates:
${JSON.stringify(cs.map((c, j) => ({ index: j, module: c.module, behavior: c.behavior, category: c.category, evidence: c.evidence })), null, 2)}
One grounded reason per verdict; a verdict for every index.`,
        { label: `refute:${key}`, model: 'opus', effort: 'medium', phase: 'Verify', schema: VERDICTS_SCHEMA },
      ).then((res) => ({ cs, res })),
    ),
  )
  for (const b of batches.filter(Boolean)) {
    if (!b.res) continue
    for (const v of b.res.verdicts) {
      const c = b.cs[v.index]
      if (c && !v.refuted) confirmed.push({ ...c, confirmed_because: v.reason })
    }
  }
}
log(`verify: ${confirmed.length}/${candidates.length} confirmed`)

return { candidates_total: candidates.length, confirmed }
