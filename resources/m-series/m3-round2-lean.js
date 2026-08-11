export const meta = {
  name: 'm3-round2-lean',
  description: 'M3 lean follow-up panel over round-1 + checkpoint-gate fixes',
  phases: [
    { title: 'Critique', detail: 'Fresh critics, medium effort, committed M3 state' },
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

const HYGIENE = `Probe hygiene (binding): mutation probes ONLY in a disposable worktree
(git worktree add /tmp/probe-<name> HEAD; git worktree remove --force after), NEVER the
main tree; report every probe. Exploration budget: the M3 layer is
\`git diff 30494be..HEAD\` — read that diff, the files it touches, the contract, and
directly referenced code; not a whole-repo re-audit.`

const AXES = [
  { key: 'spec-fidelity', model: 'opus', focus: 'contract/proposal fidelity (docs/gauntlet/contracts/M3.md; proposal sections 9, 11-15, 25). Includes: did the checkpoint-gate fix commit (5a60f49) stay within contracted behavior?' },
  { key: 'invariants', model: 'fable', focus: 'work state never inferred from process state; one owner; fail-closed recovery (per-work isolation per the F8 ruling: one failing work blocks itself with evidence, never the daemon); traversal guards on workflow and repository names actually fail closed (F2/F4 fixes); duplicate-repository submits rejected before any worktree exists (F1); stale worktree registrations pruned on rematerialize (F3)' },
  { key: 'simplicity', model: 'opus', focus: 'Ponytail ladder: did round-1 and gate fixes add machinery beyond their findings? Dead code, duplicated guards, speculative surface' },
  { key: 'test-honesty', model: 'opus', focus: 'are the 11 gate-fix behaviors pinned by tests, or fixed-but-unverifiable? Especially: traversal rejection, per-work reconcile isolation, duplicate-repo rejection, Blocked-stage cancel evidence. A fix without a test that would catch its regression is a finding' },
]

phase('Critique')
const critiques = await parallel(
  AXES.map((axis) => () =>
    agent(
      `You are a fresh blind critic in a gauntlet loop, follow-up round after round-1 fixes
AND an 11-finding checkpoint-gate fix commit (5a60f49). You did not write any of it.
Work in ${REPO}.

BEFORE filing: read GAUNTLET.md's deviation register (D1-D5) and M1/M2 ledger rulings.
Re-litigating a registered deviation or ruling requires arguing the ruling is wrong.
Your job is what earlier rounds MISSED or what the fixes regressed or left untested —
not re-reporting fixed items.

${HYGIENE}

Your single axis: ${axis.focus}

Rules: evidence-only; cite files and what you read or ran. Stay on your axis. Empty
findings list is valid and welcome. Verify gates yourself (cargo fmt --check, cargo
clippy --all-targets -- -D warnings, cargo test); set gates_verified.`,
      { label: `critic2:${axis.key}`, model: axis.model, effort: 'medium', phase: 'Critique', schema: FINDINGS_SCHEMA },
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
For EACH finding, try to REFUTE it via the actual files, docs/gauntlet/contracts/M3.md,
GAUNTLET.md's register/rulings, and running code or tests where useful.

${HYGIENE}

Findings:
${JSON.stringify(fs.map((f, j) => ({ index: j, axis: f.axis, file: f.file, severity: f.severity, summary: f.summary, evidence: f.evidence })), null, 2)}

Default to refuted=true when evidence does not clearly support a finding. One grounded
reason per verdict. Return a verdict for every index.`,
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
    `You are the M3 fixer, follow-up round. Work in ${REPO}. Contract:
docs/gauntlet/contracts/M3.md. Confirmed findings (adversarially verified):
${JSON.stringify(confirmed, null, 2)}
${gatesRed ? 'A critic also reported gates RED — reproduce and fix that first.' : ''}
Fix every confirmed finding; design decisions carry Ponytail rungs. Keep gates green:
cargo build && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo
test. No git history commands. Do not touch GAUNTLET.md, LESSONS.md, README.md, docs/,
reference/.
Return gates_green, per-finding summary, rung-tagged design_decisions,
contract_ambiguities.`,
    { label: 'fix2', model: 'opus', effort: 'high', phase: 'Fix', schema: BUILD_SCHEMA },
  )
}

return { findings: allFindings.length, confirmed, fix, gatesRed }
