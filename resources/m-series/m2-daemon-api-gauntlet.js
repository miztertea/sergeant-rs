export const meta = {
  name: 'm2-daemon-api-gauntlet',
  description: 'Build the M2 daemon/API/CLI layer and run the full four-axis gauntlet to green',
  phases: [
    { title: 'Build', detail: 'Fable builder implements M2 contract, drives gates green' },
    { title: 'Critique', detail: 'Four blind critics: spec-fidelity, invariants, simplicity, test-honesty' },
    { title: 'Verify', detail: 'Adversarial refuters per finding' },
    { title: 'Fix', detail: 'Fable fixer resolves confirmed findings' },
  ],
}

const REPO = '/home/user/sergeant-rs'
// Economy restructure 2026-08-08: this workflow runs ONLY build -> round-1
// panel -> round-1 refuters -> round-1 fix, then returns. The checkpoint
// no-mistakes gate and one lean adaptive follow-up round run under direct
// orchestrator control afterward.
const MAX_ROUNDS = 1

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

const VERDICT_SCHEMA = {
  type: 'object',
  properties: { refuted: { type: 'boolean' }, reason: { type: 'string' } },
  required: ['refuted', 'reason'],
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

phase('Build')
const build = await agent(
  `You are the M2 builder for the sergeant-rs prototype. Work in ${REPO} (M0+M1 are
committed and green — 16 tests; your work is the uncommitted M2 layer on top).

Your contract is docs/gauntlet/contracts/M2.md — read it fully, then the proposal
sections it cites (reference/proposal-depot-rust-execution-surface.md sections 6, 7, 8,
10, 24, 26, 31) and GAUNTLET.md's deviation register (D1: naming — sergeant-rs/sgt, not
depot). Build on the M1 layer as it exists (journal lock, projection catch_up, etc.) —
reuse it, do not rework it.

Implement the contract: Work domain with the section-10 state machine (transitions only
via journal events), the daemon owning journal+projections with runtime descriptor +
bearer token, the v1 HTTP/JSON API with SSE event stream and command-id idempotency, and
the clap CLI with daemon auto-spawn — with ALL seven acceptance tests from the contract
as real cargo tests. The contract's Non-goals and Unknowns are binding.

Dependency budget (contract): axum, tower, tokio (runtime features), reqwest, and
tracing-subscriber may be added; anything else needs a rung-logged justification in
design_decisions. Every design decision you report must carry its Ponytail rung (ladder
in reference/notes/ideaos-agent-contract.md), e.g. "R3: ...", and an R7 names which lower
rungs failed.

Drive the gates green before returning: cargo build && cargo fmt --check &&
cargo clippy --all-targets -- -D warnings && cargo test (M1's 16 tests must stay green).
No git commands. Do not touch GAUNTLET.md, LESSONS.md, README.md, docs/, reference/.

Return: gates_green, a short summary, design_decisions (rung-tagged), and
contract_ambiguities (anything the contract got wrong or underspecified — honest
reporting, the orchestrator reads these).`,
  { label: 'build:daemon-api', model: 'fable', effort: 'high', phase: 'Build', schema: BUILD_SCHEMA },
)

const AXES = [
  {
    key: 'spec-fidelity',
    model: 'opus',
    effort: 'high',
    brief: `spec-fidelity: does the implementation match docs/gauntlet/contracts/M2.md and the
proposal sections it cites (6, 7, 8, 10, 24, 26, 31)? Daemon-is-the-application per
section 6 (auto-spawn, one process mutating state); loopback-only binding with runtime
descriptor + bearer token per section 7; API surface per section 8 as scoped by the
contract (respond/resume/retry deliberately deferred — that deferral is IN the contract,
not a finding); Work states exactly section 10 (workflow stage is orthogonal, must NOT be
folded into Work state); command-id idempotency per section 26; CLI per section 31 subset
with human + --json output. All seven acceptance tests present and asserting the
contracted behavior.`,
  },
  {
    key: 'invariants',
    model: 'fable',
    effort: 'high',
    brief: `invariants: proposal section-40 principles as they apply here. ONE OWNER: only the
daemon process mutates runtime state; the CLI must have no path to the journal or data
dir except the HTTP API (auto-spawn excepted: it may only start the daemon binary, never
touch state itself). NO SECOND PATH: every mutation appends journal events and state is
only ever a fold of them — hunt for any direct state write that bypasses the journal.
IDEMPOTENCY: duplicate command_id returns the original result without re-execution, and
this must hold across a daemon restart (derived from the journal, not only from memory).
FAIL CLOSED: illegal Work transitions rejected without appending; unauthenticated
requests rejected; a second daemon on the same data dir must fail to start, not corrupt.
Descriptor hygiene: owner-only perms, token never logged. Hunt for state inferred from
process liveness rather than recorded evidence.`,
  },
  {
    key: 'simplicity',
    model: 'opus',
    effort: 'medium',
    brief: `simplicity: the Ponytail Minimality Ladder (reference/notes/ideaos-agent-contract.md)
is the grading rubric — every addition should sit on its lowest viable rung; unjustified
R7s and skipped rungs are findings. Premature abstraction (traits with one impl, config
knobs nothing sets, API surface the contract deferred), duplicated state, dead code,
dependencies beyond the contract's budget. The bar: could a reviewer delete this and
still satisfy the contract? Then it is a finding.`,
  },
  {
    key: 'test-honesty',
    model: 'opus',
    effort: 'high',
    brief: `test-honesty: do the tests verify the CONTRACT's seven acceptance claims, or mirror
the implementation? Idempotency test must prove one journal event sequence, not just
equal HTTP responses. Restart test must actually kill and restart the daemon (or its
in-process equivalent) and rebuild from the journal. SSE test must receive real events
over the stream, gap-free with from_seq catch-up. Auth test must prove 401 without token
AND that /healthz is open. Illegal-transition test must prove no event was appended.
Would each test catch a real regression? Can any pass vacuously? Run cargo test yourself
and set gates_verified accordingly. A test that cannot fail is a finding.`,
  },
]

function criticPrompt(axis, round) {
  return `You are a blind critic in a gauntlet loop, round ${round}. You did not write this
code and have not seen any earlier critique of it. Work in ${REPO}.

The work under review is the CURRENT UNCOMMITTED CHANGES (run git status --short and
git diff HEAD --stat, then read the changed/new files in full). The contract is
docs/gauntlet/contracts/M2.md.

BEFORE filing anything: read GAUNTLET.md's deviation register (D1-D4) and the M1 ledger
entry's adjudication rulings. A finding that re-litigates a registered deviation or a
recorded ruling must argue why the ruling is WRONG, not merely that the deviation exists
— otherwise do not file it.

Your single axis:
${axis.brief}

Rules: evidence-only — every finding cites a file and what you actually read or ran
there. Stay on your axis. Do not grade style preferences. An empty findings list is a
valid and welcome outcome. Also verify the deterministic gates yourself (cargo fmt
--check, cargo clippy --all-targets -- -D warnings, cargo test) and set gates_verified
accordingly.`
}

let round = 0
let residual = []
const history = []

let activeAxes = AXES

while (round < MAX_ROUNDS) {
  round += 1

  const critiques = await parallel(
    activeAxes.map((axis) => () =>
      agent(criticPrompt(axis, round), {
        label: `critic:${axis.key}:r${round}`,
        model: axis.model,
        effort: round === 1 ? axis.effort : 'medium',
        phase: 'Critique',
        schema: FINDINGS_SCHEMA,
      }).then((r) => ({ axis: axis.key, result: r })),
    ),
  )

  const gatesRed = critiques.filter(Boolean).some((c) => c.result && c.result.gates_verified === false)
  const allFindings = critiques
    .filter(Boolean)
    .flatMap((c) => (c.result ? c.result.findings.map((f) => ({ ...f, axis: f.axis || c.axis })) : []))

  log(`round ${round}: ${allFindings.length} findings from ${critiques.filter(Boolean).length} critics${gatesRed ? ' (GATES RED per critic)' : ''}`)

  let confirmed = []
  if (allFindings.length > 0 && round === 1) {
    const verdicts = await parallel(
      allFindings.map((f, i) => () =>
        agent(
          `Adversarially verify this gauntlet finding in ${REPO}, round ${round}. Try to REFUTE
it by reading the actual files, the contract docs/gauntlet/contracts/M2.md, GAUNTLET.md's
deviation register and M1 rulings, and where useful running the code or tests. Finding:
${JSON.stringify(f)}
Default to refuted=true if the evidence does not clearly support it. reason must be one
sentence grounded in what you read or ran.`,
          { label: `refute:r${round}:${i}`, model: 'opus', effort: 'medium', phase: 'Verify', schema: VERDICT_SCHEMA },
        ).then((v) => ({ finding: f, verdict: v })),
      ),
    )
    confirmed = verdicts.filter(Boolean).filter((v) => v.verdict && !v.verdict.refuted)
  } else if (allFindings.length > 0) {
    const byAxis = {}
    allFindings.forEach((f, i) => {
      const k = f.axis || 'unknown'
      ;(byAxis[k] = byAxis[k] || []).push({ ...f, __i: i })
    })
    const batches = await parallel(
      Object.entries(byAxis).map(([axisKey, fs]) => () =>
        agent(
          `Adversarially verify this batch of gauntlet findings (axis: ${axisKey}) in ${REPO},
round ${round}. For EACH finding, try to REFUTE it by reading the actual files, the
contract docs/gauntlet/contracts/M2.md, GAUNTLET.md's deviation register and M1 rulings,
and where useful running the code or tests. Findings (index refers to the "index" field
you must echo back):
${JSON.stringify(fs.map((f, j) => ({ index: j, axis: f.axis, file: f.file, severity: f.severity, summary: f.summary, evidence: f.evidence })), null, 2)}
Default to refuted=true when evidence does not clearly support a finding. One reason
sentence per verdict, grounded in what you read or ran. Return a verdict for every index.`,
          { label: `refute-batch:${axisKey}:r${round}`, model: 'opus', effort: 'medium', phase: 'Verify', schema: BATCH_VERDICT_SCHEMA },
        ).then((res) => ({ fs, res })),
      ),
    )
    for (const b of batches.filter(Boolean)) {
      if (!b.res) continue
      for (const v of b.res.verdicts) {
        const f = b.fs[v.index]
        if (f && !v.refuted) confirmed.push({ finding: f, verdict: { refuted: false, reason: v.reason } })
      }
    }
  }

  history.push({
    round,
    findings: allFindings.length,
    confirmed: confirmed.map((c) => ({ axis: c.finding.axis, severity: c.finding.severity, summary: c.finding.summary })),
  })

  if (confirmed.length === 0 && !gatesRed) {
    residual = []
    break
  }

  residual = confirmed

  const confirmedAxes = new Set(confirmed.map((c) => c.finding.axis))
  activeAxes = AXES.filter((a) => confirmedAxes.has(a.key))
  if (activeAxes.length === 0) activeAxes = AXES

  if (round >= MAX_ROUNDS) break

  phase('Fix')
  await agent(
    `You are the M2 fixer in a gauntlet loop, round ${round}. Work in ${REPO}. The contract is
docs/gauntlet/contracts/M2.md. A blind critic panel confirmed these findings against the
current uncommitted M2 implementation (each was adversarially verified):

${JSON.stringify(confirmed.map((c) => ({ ...c.finding, confirmed_because: c.verdict.reason })), null, 2)}

Fix every confirmed finding. Preserve the contract's Non-goals and dependency budget;
design decisions in your report carry Ponytail rungs. Keep all gates green: cargo build
&& cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test (M1's
tests included). No git commands. Do not touch GAUNTLET.md, LESSONS.md, README.md,
docs/, reference/.

Return: gates_green, summary of what changed per finding, design_decisions (rung-tagged)
if a fix required a judgment call, contract_ambiguities if a finding exposed a contract
defect.`,
    { label: `fix:r${round}`, model: 'fable', effort: 'high', phase: 'Fix', schema: BUILD_SCHEMA },
  )
}

let fix = null
if (residual.length > 0) {
  phase('Fix')
  fix = await agent(
    `You are the M2 fixer in a gauntlet loop, round 1. Work in ${REPO}. The contract is
docs/gauntlet/contracts/M2.md. A blind critic panel confirmed these findings against the
current uncommitted M2 implementation (each was adversarially verified):

${JSON.stringify(residual.map((c) => ({ ...c.finding, confirmed_because: c.verdict.reason })), null, 2)}

Fix every confirmed finding. Preserve the contract's Non-goals and dependency budget;
design decisions in your report carry Ponytail rungs. Keep all gates green: cargo build
&& cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test (M1's
tests included). No git commands. Do not touch GAUNTLET.md, LESSONS.md, README.md,
docs/, reference/.

Return: gates_green, summary of what changed per finding, design_decisions (rung-tagged)
if a fix required a judgment call, contract_ambiguities if a finding exposed a contract
defect.`,
    { label: 'fix:r1', model: 'opus', effort: 'high', phase: 'Fix', schema: BUILD_SCHEMA },
  )
}

return { build, rounds: history, confirmed_r1: residual.map((r) => ({ ...r.finding, confirmed_because: r.verdict.reason })), fix }
