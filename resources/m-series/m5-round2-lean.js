export const meta = {
  name: 'm5-round2-lean',
  description: 'M5 lean follow-up panel over round-1 + checkpoint-gate fixes',
  phases: [
    { title: 'Critique', detail: 'Fresh critics, medium effort, committed M5 state' },
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

const HYGIENE = `Probe hygiene (binding): mutation probes ONLY in a disposable copy/worktree outside
the project tree; NEVER edit the main tree; report every probe. Exploration budget: the
M5 layer is \`git diff bd41653..HEAD\` — read that diff, the files it touches, the
contract, and directly referenced code; not a whole-repo re-audit. Do NOT run the opt-in
real-Claude tests. DISK NOTE: the container is space-tight; if you copy the tree for a
probe, copy src/ and tests/ only and point CARGO_TARGET_DIR at the main checkout's
target/ so nothing rebuilds duckdb from scratch; delete the copy when done.`

const AXES = [
  { key: 'spec-fidelity', model: 'opus', focus: 'contract fidelity (docs/gauntlet/contracts/M5.md; proposal sections 21-23, 28; register D7 covers the OTel dependency shape). Did round-1 fixes and the two checkpoint-gate commits (token-scan test rewrite 2423b44, B1 doc bc0c918) stay within contract? All six acceptance tests still real?' },
  { key: 'invariants', model: 'fable', focus: 'projections disposable and never authoritative; one-owner on the duckdb file; rebuild determinism truly identical (ordering/timestamps); OTel off-by-default with zero hot-path coupling; projection failure answers 503 without touching work state; no state inferred from projection contents anywhere' },
  { key: 'simplicity', model: 'opus', focus: 'Ponytail ladder: did the fixes add machinery beyond their findings? row! macro and path() accessor cleanups landed? Any speculative analytics surface left?' },
  { key: 'test-honesty', model: 'opus', focus: 'L7/L8: the strengthened t2 (token scan) and t5 (zero-bytes-to-collector) — do they now catch what the old grep tests missed? Would reverting each round-1 fix leave the suite green? Rebuild-determinism still row-for-row? Graph provenance still resolves source_seq payloads?' },
]

phase('Critique')
const critiques = await parallel(
  AXES.map((axis) => () =>
    agent(
      `You are a fresh blind critic in a gauntlet loop, follow-up round after round-1 fixes
and a passed checkpoint gate (two pipeline commits adopted). You did not write this
code. Work in ${REPO}.

BEFORE filing: read GAUNTLET.md's deviation register (D1-D7) and the M1-M5 rulings
(including the B1 backlog row's M5 revisit), plus LESSONS.md. Re-litigating a
registered deviation or ruling requires arguing the ruling is wrong. Your job is what
earlier rounds MISSED or what the fixes regressed — not re-reporting fixed items.

${HYGIENE}

Your single axis: ${axis.focus}

Rules: evidence-only; cite files and what you read or ran. Stay on your axis. Empty
findings list is valid and welcome. Verify gates yourself (cargo fmt --check, cargo
clippy --all-targets -- -D warnings, cargo test — all warm-cached in the main target)
and set gates_verified.`,
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
For EACH finding, try to REFUTE it via the actual files, docs/gauntlet/contracts/M5.md,
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

let fix = null
if (confirmed.length > 0 || gatesRed) {
  phase('Fix')
  fix = await agent(
    `You are the M5 fixer, follow-up round. Work in ${REPO}. Contract:
docs/gauntlet/contracts/M5.md. LESSONS L6/L7/L8 binding. Confirmed findings
(adversarially verified):
${JSON.stringify(confirmed, null, 2)}
${gatesRed ? 'A critic also reported gates RED — reproduce and fix that first.' : ''}
Fix every confirmed finding; rung-tagged design decisions. Gates green: cargo build &&
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test. Do NOT run
opt-in real-Claude tests. No git history commands. Do not touch GAUNTLET.md, LESSONS.md,
README.md, docs/, reference/.
Return gates_green, per-finding summary, rung-tagged design_decisions,
contract_ambiguities.`,
    { label: 'fix2', model: 'opus', effort: 'high', phase: 'Fix', schema: BUILD_SCHEMA },
  )
}

return { findings: allFindings.length, confirmed, fix, gatesRed }
