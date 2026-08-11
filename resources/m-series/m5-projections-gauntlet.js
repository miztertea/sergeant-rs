export const meta = {
  name: 'm5-projections-gauntlet',
  description: 'Build M5 (DuckDB projection, graph, OTel) with round-1 gauntlet, economy shape',
  phases: [
    { title: 'Build', detail: 'Opus builder implements M5 contract, drives gates green' },
    { title: 'Critique', detail: 'Four blind critics (invariants on Fable), exploration-budgeted' },
    { title: 'Verify', detail: 'One batched refuter per axis, probe hygiene enforced' },
    { title: 'Fix', detail: 'Opus fixer resolves confirmed findings' },
  ],
}

const REPO = '/home/user/sergeant-rs'

const FINDINGS_SCHEMA = {
  type: 'object',
  properties: {
    gates_verified: { type: 'boolean' },
    findings: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          axis: { type: 'string' },
          file: { type: 'string' },
          summary: { type: 'string' },
          evidence: { type: 'string' },
          severity: { type: 'string', enum: ['error', 'warning', 'info'] },
        },
        required: ['axis', 'summary', 'evidence', 'severity'],
      },
    },
  },
  required: ['gates_verified', 'findings'],
}

const BATCH_VERDICT_SCHEMA = {
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

const BUILD_SCHEMA = {
  type: 'object',
  properties: {
    gates_green: { type: 'boolean' },
    summary: { type: 'string' },
    measurements: { type: 'array', items: { type: 'string' } },
    design_decisions: { type: 'array', items: { type: 'string' } },
    contract_ambiguities: { type: 'array', items: { type: 'string' } },
  },
  required: ['gates_green', 'summary'],
}

const HYGIENE = `Probe hygiene (binding): mutation probes ONLY in a disposable copy/worktree outside
the project tree; NEVER edit the main tree; report every probe. Exploration budget: the
M5 layer is the uncommitted diff — read it, the files it touches, the contract, and
directly referenced code; not a whole-repo re-audit. Do NOT run the opt-in real-Claude
tests.`

phase('Build')
const build = await agent(
  `You are the M5 builder for the sergeant-rs prototype. Work in ${REPO} (M0-M4 committed
and green — 150 tests; your work is the uncommitted M5 layer).

Your contract is docs/gauntlet/contracts/M5.md — read it fully. Then: LESSONS.md (all,
L6/L7/L8 binding), GAUNTLET.md's register (D1-D6) and rulings (note B1's trigger
condition), the proposal sections the contract cites (21, 22, 23, 28, and 8's graph
endpoints), and the existing runtime (journal, projection.rs, engine, daemon, api).

MEASURE FIRST where the contract says so: the duckdb crate's bundled build cost in this
container (time it; if prohibitive, STOP and return with the measurement and a
recommendation rather than silently swapping — the fallback is an orchestrator
adjudication). Rebuild latency on a realistic journal gets a number in your report.

Then implement the contract: the DuckDB analytical projection (daemon-owned, rebuilt
from the journal, §22 tables as far as real data exists), the §23 graph projection with
per-edge source_seq provenance and the /v1/graph/work/{id} endpoint, and §28 OTel export
off-by-default with in-memory/stdout exporter verification — with ALL SIX acceptance
tests from the contract as real cargo tests (count against the contract's own list).

L6: audit any new multi-append sequences. L7: revert-probe your own diff before
returning. L8: any capability/feature you advertise (config flags included) gets a test.
Projections are disposable — that is the §40 principle this milestone exists to prove;
if any code path treats DuckDB or graph state as authoritative, you have failed the
contract.

Rules: rung-tagged design decisions; dependency budget is the contract's list. Gates
green before returning: cargo build && cargo fmt --check && cargo clippy --all-targets
-- -D warnings && cargo test (all 150 M0-M4 tests stay green). No git history commands.
Do not touch GAUNTLET.md, LESSONS.md, README.md, docs/, reference/.

Return: gates_green, summary, measurements, design_decisions (rung-tagged),
contract_ambiguities.`,
  { label: 'build:m5-projections', model: 'opus', effort: 'high', phase: 'Build', schema: BUILD_SCHEMA },
)

const AXES = [
  {
    key: 'spec-fidelity',
    model: 'opus',
    effort: 'medium',
    brief: `spec-fidelity: implementation vs docs/gauntlet/contracts/M5.md and proposal sections
21-23, 28 as the contract scopes them. DuckDB file at the contracted path, §22 tables
justified by real data (empty speculative tables are R1 findings the CONTRACT itself
declares), graph edges per §23 with source_seq, OTel off by default, §8 graph endpoint
shape. All six acceptance tests present and real.`,
  },
  {
    key: 'invariants',
    model: 'fable',
    effort: 'high',
    brief: `invariants: PROJECTIONS ARE DISPOSABLE is under test. Hunt for: any read path that
treats DuckDB/graph state as authoritative over the journal; any write to runtime state
derived from a projection; rebuild paths that are almost-deterministic (ordering,
timestamps, floating point) rather than identical; the daemon failing closed if DuckDB
is corrupt vs wedging; one-owner — any module outside daemon/runtime opening the
projection file; OTel export mutating or blocking the hot path (export must be
fire-and-forget, off by default); L6 multi-append windows in projection-population
paths.`,
  },
  {
    key: 'simplicity',
    model: 'opus',
    effort: 'medium',
    brief: `simplicity: Ponytail ladder. Watch for: speculative §22 tables with no data source;
a query-API surface beyond the contract's minimal neighborhood + canned analytics; ORM-
ish abstraction over duckdb; incremental-population machinery without the rebuild-
latency measurement that justifies it; OTel metric plumbing for §28 metrics whose
inputs don't exist yet; config knobs nothing sets.`,
  },
  {
    key: 'test-honesty',
    model: 'opus',
    effort: 'medium',
    brief: `test-honesty: L7/L8. Rebuild-determinism test must compare row-for-row results, not
row counts. Graph-provenance test must resolve source_seq to the actual journal event
and check it justifies the edge, not just that the field exists. Disposability test
must delete the file and prove full restoration. OTel-disabled test must prove zero
exporter machinery, not just no output. Would each test catch a real regression? Run
cargo test yourself; set gates_verified.`,
  },
]

phase('Critique')
const critiques = await parallel(
  AXES.map((axis) => () =>
    agent(
      `You are a blind critic in a gauntlet loop, round 1. You did not write this code. Work
in ${REPO}. The work under review is the CURRENT UNCOMMITTED CHANGES against
docs/gauntlet/contracts/M5.md.

BEFORE filing: read GAUNTLET.md's deviation register (D1-D6) and the M1-M4 ledger
rulings, plus LESSONS.md. Re-litigating a registered deviation or ruling requires
arguing the ruling is wrong.

${HYGIENE}

Your single axis:
${axis.brief}

Rules: evidence-only — every finding cites a file and what you read or ran. Stay on
your axis. No style preferences. Empty findings list is valid and welcome. Verify the
deterministic gates yourself (cargo fmt --check, cargo clippy --all-targets --
-D warnings, cargo test) and set gates_verified.`,
      { label: `critic:${axis.key}:r1`, model: axis.model, effort: axis.effort, phase: 'Critique', schema: FINDINGS_SCHEMA },
    ).then((r) => ({ axis: axis.key, result: r })),
  ),
)

const gatesRed = critiques.filter(Boolean).some((c) => c.result && c.result.gates_verified === false)
const allFindings = critiques
  .filter(Boolean)
  .flatMap((c) => (c.result ? c.result.findings.map((f) => ({ ...f, axis: f.axis || c.axis })) : []))
log(`round 1: ${allFindings.length} findings${gatesRed ? ' (GATES RED per critic)' : ''}`)

phase('Verify')
let confirmed = []
if (allFindings.length > 0) {
  const byAxis = {}
  allFindings.forEach((f) => {
    const k = f.axis || 'unknown'
    ;(byAxis[k] = byAxis[k] || []).push(f)
  })
  const batches = await parallel(
    Object.entries(byAxis).map(([axisKey, fs]) => () =>
      agent(
        `Adversarially verify this batch of gauntlet findings (axis: ${axisKey}) in ${REPO}.
For EACH finding, try to REFUTE it via the actual files, docs/gauntlet/contracts/M5.md,
GAUNTLET.md's register/rulings, LESSONS.md, and running code or tests where useful.

${HYGIENE}

Findings:
${JSON.stringify(fs.map((f, j) => ({ index: j, axis: f.axis, file: f.file, severity: f.severity, summary: f.summary, evidence: f.evidence })), null, 2)}

Default to refuted=true when evidence does not clearly support a finding. One grounded
reason per verdict. Return a verdict for every index.`,
        { label: `refute:${axisKey}:r1`, model: 'opus', effort: 'medium', phase: 'Verify', schema: BATCH_VERDICT_SCHEMA },
      ).then((res) => ({ fs, res })),
    ),
  )
  for (const b of batches.filter(Boolean)) {
    if (!b.res) continue
    for (const v of b.res.verdicts) {
      const f = b.fs[v.index]
      if (f && !v.refuted) confirmed.push({ ...f, confirmed_because: v.reason })
    }
  }
}

let fix = null
if (confirmed.length > 0 || gatesRed) {
  phase('Fix')
  fix = await agent(
    `You are the M5 fixer, round 1. Work in ${REPO}. Contract: docs/gauntlet/contracts/M5.md.
LESSONS L6/L7/L8 binding. Confirmed findings (adversarially verified):
${JSON.stringify(confirmed, null, 2)}
${gatesRed ? 'A critic also reported gates RED — reproduce and fix that first.' : ''}
Fix every confirmed finding; preserve Non-goals and dependency budget; rung-tagged
design decisions. Gates green: cargo build && cargo fmt --check && cargo clippy
--all-targets -- -D warnings && cargo test. Do NOT run opt-in real-Claude tests. No git
history commands. Do not touch GAUNTLET.md, LESSONS.md, README.md, docs/, reference/.
Return gates_green, per-finding summary, rung-tagged design_decisions,
contract_ambiguities.`,
    { label: 'fix:r1', model: 'opus', effort: 'high', phase: 'Fix', schema: BUILD_SCHEMA },
  )
}

return { build, findings: allFindings.length, confirmed, fix, gatesRed }
