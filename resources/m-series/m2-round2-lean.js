export const meta = {
  name: 'm2-round2-lean',
  description: 'M2 lean follow-up panel: four medium-effort critics, batched refuters, one fixer if needed',
  phases: [
    { title: 'Critique', detail: 'Fresh critics, medium effort, committed M2 state' },
    { title: 'Verify', detail: 'One batched refuter per axis' },
    { title: 'Fix', detail: 'Opus fixer if findings confirmed' },
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
    design_decisions: { type: 'array', items: { type: 'string' } },
    contract_ambiguities: { type: 'array', items: { type: 'string' } },
  },
  required: ['gates_green', 'summary'],
}

const AXES = [
  { key: 'spec-fidelity', focus: 'contract and proposal fidelity (docs/gauntlet/contracts/M2.md; proposal sections 6, 7, 8, 10, 26, 31)' },
  { key: 'invariants', focus: 'one owner, no second path to state, journal-derived idempotency incl. across restart, fail-closed transitions/auth/second-daemon, no state inferred from process liveness' },
  { key: 'simplicity', focus: 'Ponytail ladder as rubric: unjustified R7s, dead code, duplicated state, deps beyond need' },
  { key: 'test-honesty', focus: 'do tests verify the contract claims or mirror the implementation; can any pass vacuously; do auth tests cover POST routes; does the idempotency test inspect the journal' },
]

phase('Critique')
const critiques = await parallel(
  AXES.map((axis) => () =>
    agent(
      `You are a fresh blind critic in a gauntlet loop, follow-up round after round-1 fixes
and an independent pipeline pass. You did not write this code and have not seen earlier
critiques. Work in ${REPO}.

The work under review is the M2 layer: run \`git log --oneline 4e749bc..HEAD\` and
\`git diff 4e749bc..HEAD --stat\`, then read the changed/new files in full (src/domain/work.rs,
src/daemon.rs, src/api.rs, src/cli.rs, tests/m2_daemon_api.rs, and anything else in the
diff). The contract is docs/gauntlet/contracts/M2.md.

BEFORE filing anything: read GAUNTLET.md's deviation register and the M1 ledger entry's
rulings. Round-1 fixes are already applied and a no-mistakes pipeline pass already ran —
your job is what they MISSED or what the fixes regressed, not re-reporting fixed items. A
finding re-litigating a registered deviation must argue why the ruling is wrong.

Your single axis: ${axis.focus}

Rules: evidence-only; cite files and what you read or ran. Stay on your axis. An empty
findings list is valid and welcome. Verify the gates yourself (cargo fmt --check, cargo
clippy --all-targets -- -D warnings, cargo test) and set gates_verified.`,
      { label: `critic2:${axis.key}`, model: axis.key === 'invariants' ? 'fable' : 'opus', effort: 'medium', phase: 'Critique', schema: FINDINGS_SCHEMA },
    ).then((r) => ({ axis: axis.key, result: r })),
  ),
)

const gatesRed = critiques.filter(Boolean).some((c) => c.result && c.result.gates_verified === false)
const allFindings = critiques
  .filter(Boolean)
  .flatMap((c) => (c.result ? c.result.findings.map((f) => ({ ...f, axis: f.axis || c.axis })) : []))
log(`follow-up round: ${allFindings.length} findings${gatesRed ? ' (GATES RED per critic)' : ''}`)

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
For EACH finding, try to REFUTE it by reading the actual files, the contract
docs/gauntlet/contracts/M2.md, GAUNTLET.md's register and rulings, and where useful
running code or tests. Findings:
${JSON.stringify(fs.map((f, j) => ({ index: j, axis: f.axis, file: f.file, severity: f.severity, summary: f.summary, evidence: f.evidence })), null, 2)}
Default to refuted=true when evidence does not clearly support a finding. One grounded
reason sentence per verdict. Return a verdict for every index.`,
        { label: `refute2:${axisKey}`, model: 'opus', effort: 'medium', phase: 'Verify', schema: BATCH_VERDICT_SCHEMA },
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
    `You are the M2 fixer, follow-up round. Work in ${REPO}. Contract:
docs/gauntlet/contracts/M2.md. Confirmed findings (adversarially verified):
${JSON.stringify(confirmed, null, 2)}
${gatesRed ? 'A critic also reported the deterministic gates RED — reproduce and fix that first.' : ''}
Fix every confirmed finding; design decisions carry Ponytail rungs. Keep gates green:
cargo build && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo
test. No git commands. Do not touch GAUNTLET.md, LESSONS.md, README.md, docs/, reference/.
Return gates_green, per-finding summary, rung-tagged design_decisions, contract_ambiguities.`,
    { label: 'fix2', model: 'opus', effort: 'high', phase: 'Fix', schema: BUILD_SCHEMA },
  )
}

return { findings: allFindings.length, confirmed, fix, gatesRed }
