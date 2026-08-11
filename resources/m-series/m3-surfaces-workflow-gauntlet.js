export const meta = {
  name: 'm3-surfaces-workflow-gauntlet',
  description: 'Build M3 (surfaces, workflow engine, routing, fake backend) with round-1 gauntlet, economy shape',
  phases: [
    { title: 'Build', detail: 'Opus builder implements M3 contract, drives gates green' },
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
    design_decisions: { type: 'array', items: { type: 'string' } },
    contract_ambiguities: { type: 'array', items: { type: 'string' } },
  },
  required: ['gates_green', 'summary'],
}

const HYGIENE = `Probe hygiene (binding): if you need a mutation probe (edit code to see a test fail),
do it ONLY in a disposable worktree (git worktree add /tmp/probe-<name> HEAD; work there;
git worktree remove --force when done). NEVER edit the main tree — you are verifying it,
not changing it. Report every probe you ran. Exploration budget: read the M3 diff
(git status --short, git diff HEAD --stat) and the files it touches plus the contract and
directly referenced code — do not re-audit the whole repository.`

phase('Build')
const build = await agent(
  `You are the M3 builder for the sergeant-rs prototype. Work in ${REPO} (M0-M2 committed
and green — 39 tests; your work is the uncommitted M3 layer).

Your contract is docs/gauntlet/contracts/M3.md — read it fully, then the proposal
sections it cites (reference/proposal-depot-rust-execution-surface.md sections 9, 11-15,
25, 37), GAUNTLET.md's deviation register (D1-D5) and the M1/M2 ledger rulings, and the
existing M2 layer (daemon.rs, api.rs, domain/work.rs) which you build on, not rework.

Implement the contract: workspace discovery + sergeant.toml, git-worktree work surfaces
with binding records, the versioned staged workflow engine (workflow.toml + CONTEXT.md,
section-12 verbs only, stage state orthogonal to Work state), the section-15 backend
trait (M3 subset: probe/start/send/observe/stop + capabilities), the deterministic
scripted fake backend, section-13 routing precedence + section-14 profiles, and the API/
CLI additions (input/respond, backends/profiles listing, stage+surface in work show) —
with ALL EIGHT acceptance tests from the contract as real cargo tests (count them
against the contract's own list; do not trust this brief's arithmetic).

Every design decision carries its Ponytail rung; an R7 names the lower rungs that
failed. No new dependencies without a rung-logged justification (git is driven via the
installed CLI, R4). Stage completion comes only from explicit backend/API signals —
never inferred from liveness.

Drive the gates green before returning: cargo build && cargo fmt --check && cargo
clippy --all-targets -- -D warnings && cargo test (all M1+M2 tests stay green). No git
commit/branch/push commands (worktree creation for surfaces is your feature — build and
test it; the orchestrator owns repository history). Do not touch GAUNTLET.md,
LESSONS.md, README.md, docs/, reference/.

Return: gates_green, summary, design_decisions (rung-tagged), contract_ambiguities.`,
  { label: 'build:m3-surfaces', model: 'opus', effort: 'high', phase: 'Build', schema: BUILD_SCHEMA },
)

const AXES = [
  {
    key: 'spec-fidelity',
    model: 'opus',
    effort: 'medium',
    brief: `spec-fidelity: implementation vs docs/gauntlet/contracts/M3.md and proposal sections
9, 11-15 (as scoped by the contract — the M3 trait subset and deferred verbs are IN the
contract, not findings). Workspace shape per section 9 (D1 naming), binding records per
section 11, section-12 workflow verbs with stage/Work orthogonality, section-13
precedence order exactly, section-14 profiles without credentials. All eight acceptance
tests present and asserting contracted behavior.`,
  },
  {
    key: 'invariants',
    model: 'fable',
    effort: 'high',
    brief: `invariants: this is the milestone the thesis lives in. WORK STATE IS NOT PROCESS
STATE: hunt for ANY place work/stage state is inferred from backend/process liveness
rather than explicit recorded signals (the contract's test 6 is the canary — check it
proves what it claims). One owner: surfaces/workflow state mutate only through the
daemon's journal path. Fail closed: ambiguous recovery goes to blocked-with-evidence,
never guessed; dirty-at-teardown recorded, not destroyed; no backend resolvable is an
error listing options, never a silent substitute (section 13's hard rule). Durable
trajectory: stage entry/completion/waits are journal events with correlation/causation.
Recovery per section 25: restart reconciles before any new execution spawns.`,
  },
  {
    key: 'simplicity',
    model: 'opus',
    effort: 'medium',
    brief: `simplicity: Ponytail ladder as rubric. Watch for: workflow engine growing DAG-shaped
machinery (section 4 forbids it), speculative trait surface beyond the M3 subset, config
knobs nothing sets, worktree-pooling or caching nothing needs (R1 — treehouse
explicitly deferred), duplicated state between workflow/stage/work records. Could a
reviewer delete it and still satisfy the contract? Finding.`,
  },
  {
    key: 'test-honesty',
    model: 'opus',
    effort: 'medium',
    brief: `test-honesty: do the eight acceptance tests verify the contract's claims? Test 3 must
prove journal ordering of stage events, not just final state. Test 6 must prove the
engine ignored a still-"running" fake process after cancel — not merely that cancel
works. Test 7 must prove precedence by giving multiple levels simultaneously and
watching the right one win, level by level. Test 8 must actually restart the daemon and
show reconciliation evidence. Can any test pass vacuously? Run cargo test yourself; set
gates_verified.`,
  },
]

phase('Critique')
const critiques = await parallel(
  AXES.map((axis) => () =>
    agent(
      `You are a blind critic in a gauntlet loop, round 1. You did not write this code. Work
in ${REPO}. The work under review is the CURRENT UNCOMMITTED CHANGES against
docs/gauntlet/contracts/M3.md.

BEFORE filing anything: read GAUNTLET.md's deviation register (D1-D5) and the M1/M2
ledger rulings. A finding re-litigating a registered deviation or recorded ruling must
argue why the ruling is WRONG, not that the deviation exists.

${HYGIENE}

Your single axis:
${axis.brief}

Rules: evidence-only — every finding cites a file and what you read or ran. Stay on
your axis. No style preferences. An empty findings list is valid and welcome. Verify
the gates yourself (cargo fmt --check, cargo clippy --all-targets -- -D warnings,
cargo test) and set gates_verified.`,
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
For EACH finding, try to REFUTE it by reading the actual files, the contract
docs/gauntlet/contracts/M3.md, GAUNTLET.md's register and rulings, and where useful
running code or tests.

${HYGIENE}

Findings:
${JSON.stringify(fs.map((f, j) => ({ index: j, axis: f.axis, file: f.file, severity: f.severity, summary: f.summary, evidence: f.evidence })), null, 2)}

Default to refuted=true when evidence does not clearly support a finding. One grounded
reason sentence per verdict. Return a verdict for every index.`,
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
    `You are the M3 fixer, round 1. Work in ${REPO}. Contract: docs/gauntlet/contracts/M3.md.
Confirmed findings (adversarially verified):
${JSON.stringify(confirmed, null, 2)}
${gatesRed ? 'A critic also reported the deterministic gates RED — reproduce and fix that first.' : ''}
Fix every confirmed finding; preserve the contract's Non-goals and dependency budget;
design decisions carry Ponytail rungs. Keep all gates green: cargo build && cargo fmt
--check && cargo clippy --all-targets -- -D warnings && cargo test. No git history
commands. Do not touch GAUNTLET.md, LESSONS.md, README.md, docs/, reference/.
Return gates_green, per-finding summary, rung-tagged design_decisions,
contract_ambiguities.`,
    { label: 'fix:r1', model: 'opus', effort: 'high', phase: 'Fix', schema: BUILD_SCHEMA },
  )
}

return { build, findings: allFindings.length, confirmed, fix, gatesRed }
