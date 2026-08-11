export const meta = {
  name: 's1-instrument-round2-lean',
  description: 'S1 phase-1 lean round 2: fresh blind critics over the instrument commits, batched refuters, fixer if confirmed',
  phases: [
    { title: 'Critique', detail: 'Fresh critics (test-honesty on Fable, invariants+simplicity on Opus) over the phase-1 diff' },
    { title: 'Verify', detail: 'One batched refuter per axis, probe hygiene enforced' },
    { title: 'Fix', detail: 'Opus fixer if findings confirmed' },
  ],
}

// args: { base: '<SHA before phase 1>' }
const REPO = '/home/user/sergeant-rs'
const BASE = args && args.base ? args.base : 'f20df90'

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
  },
  required: ['gates_green', 'summary'],
}

const HYGIENE = `Probe hygiene (binding, L5): mutation probes ONLY in a disposable git worktree
(git worktree add /tmp/probe-<name> HEAD; git worktree remove --force after; the worktree
gets its OWN target dir — never set CARGO_TARGET_DIR), NEVER the main tree; report every
probe. Exploration budget: the phase-1 layer is \`git diff ${BASE}..HEAD\` RESTRICTED to
the builder's surfaces — tests/, scripts/, .github/, .gitattributes, .gitignore,
docs/coverage/ — read that part of the diff, the files it touches, the S1 contract
(docs/gauntlet/contracts/S1-COVERAGE.md), the S0 rulings (S0.md), and directly referenced
code; not a whole-repo re-audit. Orchestrator record commits in the same range
(docs/gauntlet/, reference/notes/, CLAUDE.md, resources/) are OUT of scope — the
adjudication surface, not the reviewed code. Never run the opt-in real-Claude tests. Never run a full instrumented
coverage build (a probe of one stage script's guards is fine; the 15+ min build is not).`

const AXES = [
  {
    key: 'spec-fidelity',
    model: 'sonnet',
    effort: 'high',
    focus: `spec-fidelity: the phase-1 diff vs docs/gauntlet/contracts/S1-COVERAGE.md's
"Phase 1" list (items 1-7) and the S0 rulings it cites. Every listed deliverable present
and nothing beyond the list (the contract's own scope line: "EXACTLY the contract's
phase-1 list, nothing more"). The R-S0-3 command lines must appear VERBATIM in the
committed stage scripts — a paraphrased convention is the L11 failure the ruling exists
to prevent. Design decisions rung-logged; commits separable per L10 (one deliverable
class per commit, no fix folded into an unrelated build).`,
  },
  {
    key: 'test-honesty',
    model: 'opus',
    effort: 'high',
    focus: `test-honesty over the instrument repairs — this is the axis this round exists for
(L7; the builder probed its own pins, which the gauntlet pattern forbids trusting).
For EACH of the three repairs verify the pin yourself by reverting the repair in a
disposable worktree and confirming its pinning test FAILS: (1) SpawnedDaemon SIGTERM
teardown — does the pin assert the daemon exited via SIGTERM (not SIGKILL), and does a
reverted rig make it fail? Does it pin the COMPOSITION (Bug Sprint 1's lesson: parts
pinned, composition not)? (2) reaper TERM-vs-KILL reporting — does the extended test
distinguish the two, or would any reap pass it? (3) demo.sh verify-gone — is the
fail-closed check reachable and does the script actually exit nonzero, or is it
decorative? Also: do the harness stage scripts' own guards (disk pre-flight, profraw
accounting, hygiene sweep) FAIL when their condition trips, or only print? A guard that
cannot fail is a finding. And: does scripts/coverage/README.md's claimed tool-semantics
list match what the scripts actually do?`,
  },
  {
    key: 'invariants-simplicity',
    model: 'opus',
    effort: 'medium',
    focus: `invariants + simplicity combined (lean round). Invariants: the repairs must not
weaken the guard rails they touch — the reaper must still SIGKILL a daemon that ignores
SIGTERM (the 89-orphan lesson stands); demo.sh must still delete its dir on the happy
path; the SIGTERM-first drop must not let a wedged daemon outlive a test run; ci.yml must
still gate PRs identically (only the duplicate push trigger removed); .gitattributes must
not hide real source (scripts/ and tests/fixtures stay counted). Simplicity: Ponytail
ladder over the diff — machinery beyond the contract's phase-1 list, speculative harness
features, duplicated guard logic that support/mod.rs already owns, config knobs nothing
sets. Rulings R-S0-1..11 are settled; re-litigating one requires arguing it wrong (L3).`,
  },
]

phase('Critique')
const critiques = await parallel(
  AXES.map((axis) => () =>
    agent(
      `You are a fresh blind critic in a gauntlet loop, lean round 2 over the S1 phase-1
instrument commits (${BASE}..HEAD). You did not write any of it. Work in ${REPO}.

BEFORE filing: read docs/gauntlet/contracts/S0.md and S1-COVERAGE.md, GAUNTLET.md's
deviation register and Bug Sprint 1 entry, LESSONS.md (L5/L7/L10 especially).

${HYGIENE}

Your single axis:
${axis.focus}

Rules: evidence-only — every finding cites a file and what you read or ran. Stay on your
axis. Empty findings list is valid and welcome. Verify the deterministic gates yourself
(cargo fmt --check, cargo clippy --all-targets -- -D warnings, cargo test) and set
gates_verified. After any worktree build, remember the main checkout's target may need a
rebuild before timing-sensitive measurement — do not measure timings.`,
      { label: `critic2:${axis.key}`, model: axis.model, effort: axis.effort, phase: 'Critique', schema: FINDINGS_SCHEMA },
    ).then((r) => ({ axis: axis.key, result: r })),
  ),
)

const gatesRed = critiques.filter(Boolean).some((c) => c.result && c.result.gates_verified === false)
const allFindings = critiques
  .filter(Boolean)
  .flatMap((c) => (c.result ? c.result.findings.map((f) => ({ ...f, axis: f.axis || c.axis })) : []))
log(`lean round 2: ${allFindings.length} findings${gatesRed ? ' (GATES RED per critic)' : ''}`)

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
        `Adversarially verify this batch of gauntlet findings (axis: ${axisKey}) in ${REPO},
against the S1 phase-1 diff (${BASE}..HEAD), docs/gauntlet/contracts/S0.md and
S1-COVERAGE.md, and LESSONS.md. Try to REFUTE each.

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
log(`verify: ${confirmed.length}/${allFindings.length} confirmed`)

let fix = null
if (confirmed.length > 0 || gatesRed) {
  phase('Fix')
  fix = await agent(
    `You are the S1 phase-1 fixer, lean round 2. Work in ${REPO}. Contract:
docs/gauntlet/contracts/S1-COVERAGE.md (phase 1 scope ONLY); rulings in S0.md binding.
Confirmed findings (adversarially verified):
${JSON.stringify(confirmed, null, 2)}
${gatesRed ? 'A critic also reported gates RED — reproduce and fix that first.' : ''}
Fix every confirmed finding; every fix ships its pinning test or strengthens an existing
one (L7 — and revert-probe it in a disposable worktree, L5); separable commits (L10);
rung-tagged design decisions. Do NOT touch src/, Cargo.toml, GAUNTLET.md, LESSONS.md,
reference/, docs/gauntlet/ (contracts are the orchestrator's). Gates green before
returning: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test.
Return gates_green, per-finding summary, rung-tagged design_decisions.`,
    { label: 'fix2', model: 'opus', effort: 'high', phase: 'Fix', schema: BUILD_SCHEMA },
  )
}

return { findings: allFindings.length, confirmed, fix, gatesRed }
