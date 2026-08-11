export const meta = {
  name: 'm4-round2-lean',
  description: 'M4 lean follow-up panel + seeded resume-wiring finding',
  phases: [
    { title: 'Critique', detail: 'Fresh critics, medium effort, committed M4 state' },
    { title: 'Verify', detail: 'One batched refuter per axis' },
    { title: 'Fix', detail: 'Opus fixer: seeded resume-wiring + confirmed findings' },
  ],
}

const REPO = '/home/user/sergeant-rs'

const SEEDED = [
  {
    axis: 'invariants',
    file: 'src/runtime/engine.rs',
    severity: 'warning',
    summary:
      'Backend::resume() has no production caller (checkpoint-gate finding resume-unwired, ask-user, ruled by the orchestrator): reconcile only observes and blocks, so resumable executions never resume.',
    evidence:
      'Gate run finding: engine.rs reconcile_work calls only backend.observe(); no CLI/API path invokes resume. ORCHESTRATOR RULING (binding): wire resume into reconcile per proposal section 25 (reattach/resume/classify) — when reconciliation evidence is unambiguously resumable, the engine resumes the execution instead of parking work in blocked; ambiguous evidence stays fail-closed to blocked; do NOT add a new public API/CLI verb (R1 — retry already covers explicit human re-entry for blocked work). Ship with pinning tests: fake-backend resumable script auto-resumes without duplicate execution; ambiguous still blocks; L6 audit on any new append sequence.',
    confirmed_because: 'Confirmed by the shipping-gate review and accepted by orchestrator ruling; not subject to refutation.',
  },
]

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

const HYGIENE = `Probe hygiene (binding): mutation probes ONLY in a disposable copy/worktree outside
the project tree; NEVER edit the main tree; report every probe. Exploration budget: the
M4 layer is \`git diff 245a10e..HEAD\` plus uncommitted changes — read that, the files it
touches, the contract, and directly referenced code; not a whole-repo re-audit.
Real-Claude tests: do NOT run them (budget); rely on the recorded run evidence and the
deterministic suite.`

const AXES = [
  { key: 'spec-fidelity', model: 'opus', focus: 'contract fidelity (docs/gauntlet/contracts/M4.md as amended by D6; proposal sections 15, 17, 20, 25, 27, 37). Did the round-1 fixes and the pipeline doc commit stay within contract? Six remaining acceptance tests real?' },
  { key: 'invariants', model: 'fable', focus: 'work state never from process state; session identity chosen pre-launch and verified before destructive ops; pin claims never stronger than evidence (is_error/modelUsage keyed, subtype distrusted); reconciliation evidence-based and fail-closed; capability flags honest; L6 multi-append windows in start/reconcile/resume paths; raw evidence durably archived and referenced' },
  { key: 'simplicity', model: 'opus', focus: 'Ponytail ladder: did round-1 fixes add machinery beyond their findings? Codex stub truly a stub? EventSink surface minimal? Dead code from the descope sweep?' },
  { key: 'test-honesty', model: 'opus', focus: 'L7: would reverting each round-1 fix leave the suite green? Regression catalog entries actually reproduce their Sergeant failures (check provenance comments against reference/sergeant-upstream)? Opt-in tests skip cleanly when unset? Fixture tests assert semantics, not parser mirrors?' },
]

phase('Critique')
const critiques = await parallel(
  AXES.map((axis) => () =>
    agent(
      `You are a fresh blind critic in a gauntlet loop, follow-up round after round-1 fixes
and a passed checkpoint gate. You did not write this code. Work in ${REPO}.

BEFORE filing: read GAUNTLET.md's deviation register (D1-D6), the pause marker, and the
M1-M3 ledger rulings, plus LESSONS.md. Re-litigating a registered deviation or ruling
requires arguing the ruling is wrong. Your job is what earlier rounds MISSED or what the
fixes regressed — not re-reporting fixed items. Note: resume-wiring is already ruled and
seeded for the fixer; do not re-file it, but DO file anything wrong with how it could
interact with your axis once wired (e.g. resume paths that would violate fail-closed).

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
let confirmed = [...SEEDED]
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
For EACH finding, try to REFUTE it via the actual files, docs/gauntlet/contracts/M4.md,
GAUNTLET.md's register/rulings, LESSONS.md, and running code or tests where useful.

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

phase('Fix')
const fix = await agent(
  `You are the M4 fixer, follow-up round. Work in ${REPO}. Contract:
docs/gauntlet/contracts/M4.md. LESSONS L6/L7 binding (audit multi-append windows; every
fix ships its pinning test — revert-probe your own fixes). The FIRST item below is a
seeded, orchestrator-ruled work item (resume wiring — its ruling text is binding);
the rest are adversarially confirmed findings:
${JSON.stringify(confirmed, null, 2)}
${gatesRed ? 'A critic also reported gates RED — reproduce and fix that first.' : ''}
Fix every item; preserve Non-goals and dependency budget; rung-tagged design decisions.
Gates green: cargo build && cargo fmt --check && cargo clippy --all-targets --
-D warnings && cargo test. Do NOT run the opt-in real-Claude tests. No git history
commands. Do not touch GAUNTLET.md, LESSONS.md, README.md, docs/, reference/.
Return gates_green, per-finding summary, rung-tagged design_decisions,
contract_ambiguities.`,
  { label: 'fix2', model: 'opus', effort: 'high', phase: 'Fix', schema: BUILD_SCHEMA },
)

return { findings: allFindings.length, confirmed, fix, gatesRed }
