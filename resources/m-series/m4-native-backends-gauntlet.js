export const meta = {
  name: 'm4-native-backends-gauntlet',
  description: 'Build M4 (Claude adapter, Codex protocol adapter, recovery, regression catalog) with round-1 gauntlet',
  phases: [
    { title: 'Build', detail: 'Fable builder — measured Claude adapter, drives gates green' },
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

const HYGIENE = `Probe hygiene (binding): mutation probes ONLY in a disposable copy or worktree
outside the project tree; NEVER edit the main tree you are verifying; report every probe.
Exploration budget: the M4 layer is the uncommitted diff — read it, the files it touches,
the contract, and directly referenced code; not a whole-repo re-audit.`

phase('Build')
const build = await agent(
  `You are the M4 builder for the sergeant-rs prototype. Work in ${REPO} (M0-M3 committed
and green — 96 tests; your work is the uncommitted M4 layer).

Your contract is docs/gauntlet/contracts/M4.md — read it fully. Then read, in order:
(1) LESSONS.md (all of it — L1, L2, L6, L7 are binding on you),
(2) GAUNTLET.md's deviation register (especially D2) and ledger rulings,
(3) reference/sergeant-upstream/docs/research/claude-background-harness-spike.md (the
    measured facts your adapter design rests on),
(4) the proposal sections the contract cites (15, 16, 17, 20, 25, 27, 37),
(5) the existing backend trait (src/backend/mod.rs), fake backend, engine, and recovery.

MEASURE FIRST (LESSONS L1): before writing adapter code, measure the installed claude
CLI (2.1.226) against the contract's Unknowns: session_id capture and --resume
continuity in print mode (including from a different cwd and across processes), what a
killed mid-turn process leaves resumable, the result envelope's model fields for pin
verification, and the flag surface for the capability probe. Budget discipline: smallest
usable model (haiku), one-line prompts, the fewest turns that answer the question —
these are measurements, not conversations. Record every measurement verbatim in your
returned "measurements" array; unmeasurable claims are recorded as documented-not-
measured and fail closed (the spike's own doctrine).

Then implement the contract: the Claude adapter (headless print-mode turns per D2,
--setting-sources user, three-layer model-pin verification, raw stream-json to blob
store + normalized events, interrupt = per-turn process kill with conversation
surviving), the Codex adapter (protocol layer against recorded fixtures, ALL live tests
#[ignore]d), the trait extension (only what evidence demands), restart reconciliation
against real session evidence, the version/capability gate, and the Sergeant regression
catalog with per-entry provenance — with ALL SEVEN acceptance tests from the contract
(count against the contract's own list). Real-Claude tests behind SERGEANT_CLAUDE_TESTS=1,
skipped cleanly otherwise; run them once yourself to prove they pass.

L6 is binding: audit every multi-append sequence you add (start, reconcile) for the
crash-window class. L7 is binding: every behavior you add ships with the test that
would catch its regression — revert-probe your own diff before returning.

Rules: design decisions carry Ponytail rungs (R7 names failed lower rungs). No new
dependencies without rung-logged justification. Gates green before returning: cargo
build && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
(all 96 M0-M3 tests stay green). No git history commands. Do not touch GAUNTLET.md,
LESSONS.md, README.md, docs/, reference/.

Return: gates_green, summary, measurements, design_decisions (rung-tagged),
contract_ambiguities.`,
  { label: 'build:m4-adapters', model: 'fable', effort: 'high', phase: 'Build', schema: BUILD_SCHEMA },
)

const AXES = [
  {
    key: 'spec-fidelity',
    model: 'opus',
    effort: 'medium',
    brief: `spec-fidelity: implementation vs docs/gauntlet/contracts/M4.md and proposal sections
15, 16, 17, 20, 25, 27 as the contract scopes them. D2's design honored (print-mode
turns, no --bg/attach/tmux)? Three-layer pin verification present with honest
"attempted vs honored" evidence semantics? Raw AND normalized capture (section 20's
both-streams rule)? Codex strictly fixture-tested with live tests #[ignore]d? Version
gate refuses missing capabilities? All seven acceptance tests present and real?`,
  },
  {
    key: 'invariants',
    model: 'fable',
    effort: 'high',
    brief: `invariants: the adapter is where work-state-is-not-process-state meets reality. Hunt
for: execution state inferred from per-turn process exit rather than recorded evidence;
interrupt paths that mutate work state directly; reconciliation that guesses instead of
failing closed to blocked-with-evidence; session identity trusted without verification
(id + evidence, the spike's identity doctrine); pin claims stronger than their evidence
(exit code or mission success treated as proof — the spike proved both insufficient);
multi-append crash windows in start/reconcile (LESSONS L6); raw evidence discarded
instead of archived. Capability flags must be honest — nothing advertised that is not
measured.`,
  },
  {
    key: 'simplicity',
    model: 'opus',
    effort: 'medium',
    brief: `simplicity: Ponytail ladder as rubric. Watch for: trait surface grown beyond what the
Claude adapter demonstrably needs; speculative Codex machinery beyond the fixture-tested
protocol layer; emulation of unsupported capabilities (section 15 forbids it);
subprocess-driving machinery duplicating what runtime/git.rs already established;
retry/backoff frameworks nothing measured a need for. Unjustified R7s and skipped rungs
are findings.`,
  },
  {
    key: 'test-honesty',
    model: 'opus',
    effort: 'medium',
    brief: `test-honesty: LESSONS L7 is your rubric — would reverting each behavior leave the
suite green? Are the opt-in real-Claude tests real (do they skip cleanly AND pass when
enabled — check the builder's claimed run evidence)? Do fixture tests for Codex and for
the substitution envelope assert the contracted semantics or mirror the parser? Does
the regression catalog actually reproduce each named Sergeant failure, with provenance
comments, or just gesture at it? Is the recovery test evidence-based (session existence)
rather than sleep-and-hope? Run cargo test yourself; set gates_verified.`,
  },
]

phase('Critique')
const critiques = await parallel(
  AXES.map((axis) => () =>
    agent(
      `You are a blind critic in a gauntlet loop, round 1. You did not write this code. Work
in ${REPO}. The work under review is the CURRENT UNCOMMITTED CHANGES against
docs/gauntlet/contracts/M4.md.

BEFORE filing: read GAUNTLET.md's deviation register (D1-D5) and the M1-M3 ledger
rulings, plus LESSONS.md. Re-litigating a registered deviation or ruling requires
arguing the ruling is wrong.

${HYGIENE}

Real-Claude usage: you may run the opt-in tests ONCE (SERGEANT_CLAUDE_TESTS=1) if your
axis needs it; smallest-model discipline applies. Do not run them repeatedly.

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
For EACH finding, try to REFUTE it via the actual files, docs/gauntlet/contracts/M4.md,
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
    `You are the M4 fixer, round 1. Work in ${REPO}. Contract: docs/gauntlet/contracts/M4.md.
LESSONS L6/L7 binding (audit multi-append windows; every fix ships its pinning test —
revert-probe your own fixes). Confirmed findings (adversarially verified):
${JSON.stringify(confirmed, null, 2)}
${gatesRed ? 'A critic also reported gates RED — reproduce and fix that first.' : ''}
Fix every confirmed finding; preserve Non-goals and dependency budget; rung-tagged
design decisions. Gates green: cargo build && cargo fmt --check && cargo clippy
--all-targets -- -D warnings && cargo test. No git history commands. Do not touch
GAUNTLET.md, LESSONS.md, README.md, docs/, reference/.
Return gates_green, per-finding summary, rung-tagged design_decisions,
contract_ambiguities.`,
    { label: 'fix:r1', model: 'opus', effort: 'high', phase: 'Fix', schema: BUILD_SCHEMA },
  )
}

return { build, findings: allFindings.length, confirmed, fix, gatesRed }
