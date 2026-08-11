export const meta = {
  name: 'm1-event-core-gauntlet',
  description: 'Build the M1 event core (envelope, journal, blobs, projection) and run the full four-axis gauntlet to green',
  phases: [
    { title: 'Build', detail: 'Fable builder implements M1 contract, drives gates green' },
    { title: 'Critique', detail: 'Four blind critics: spec-fidelity, invariants, simplicity, test-honesty' },
    { title: 'Verify', detail: 'Adversarial refuters per finding' },
    { title: 'Fix', detail: 'Fable fixer resolves confirmed findings' },
  ],
}

const REPO = '/home/user/sergeant-rs'
const MAX_ROUNDS = 4

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
  `You are the M1 builder for the sergeant-rs prototype. Work in ${REPO} (M0 scaffold is
committed; your work is the uncommitted M1 layer).

Your contract is docs/gauntlet/contracts/M1.md — read it fully, then read the proposal
sections it cites (reference/proposal-depot-rust-execution-surface.md sections 18-21 and 24)
and the existing stubs in src/domain/event.rs, src/runtime/{journal,projection}.rs.

Implement the contract: the event envelope, the segmented append-only NDJSON journal with
crash-tail recovery, the BLAKE3 content-addressed blob store, and the reducer-based
projection with snapshot + replay — with ALL six acceptance tests from the contract
implemented as real cargo tests. The contract's Non-goals and Unknowns sections are
binding: plain std::fs synchronous I/O, no async, no daemon, no domain state machines.

You may add dev-dependencies needed for testing (e.g. tempfile). You may not add new
runtime dependencies beyond what M0 pinned (serde, serde_json, clap, tokio, ulid, blake3,
toml, tracing, thiserror) without recording why in your returned design_decisions.

Drive the gates green before returning: cargo build && cargo fmt --check &&
cargo clippy --all-targets -- -D warnings && cargo test. Do NOT run any git commands —
the orchestrator owns git. Do not touch GAUNTLET.md, LESSONS.md, README.md, docs/, or
reference/.

Return: gates_green, a short summary, design_decisions (choices the contract left open and
how you resolved them), contract_ambiguities (anything the contract got wrong or
underspecified — honest reporting, the orchestrator reads these).`,
  { label: 'build:event-core', model: 'fable', effort: 'high', phase: 'Build', schema: BUILD_SCHEMA },
)

const AXES = [
  {
    key: 'spec-fidelity',
    model: 'opus',
    effort: 'high',
    brief: `spec-fidelity: does the implementation match docs/gauntlet/contracts/M1.md and the
proposal sections it cites (18-21, 24)? Envelope fields per section 19 (adapted per
deviation D1 in GAUNTLET.md), journal semantics per section 21, snapshot-never-canonical
per section 24, unknown-field preservation per section 20. Are all six acceptance tests
from the contract present and does each actually assert the contracted behavior? Any
deviation not registered in GAUNTLET.md's deviation register is a finding.`,
  },
  {
    key: 'invariants',
    model: 'fable',
    effort: 'high',
    brief: `invariants: the proposal section-40 principles as they apply to this layer. Single
writer: is the journal structurally single-owner (one writer type, no shared mutable
access paths)? Durable trajectory: fsync before append returns, append-only, stored lines
never rewritten (crash recovery must not destroy complete events). Projections disposable:
snapshot is an optimization, unknown snapshot schema falls back to full replay (fail
closed). Fail-closed ambiguity: seq gaps/duplicates error rather than skip; blob hash
mismatch errors. Forward compatibility: unknown envelope fields survive read. Hunt for
places where the code infers state instead of failing on ambiguity.`,
  },
  {
    key: 'simplicity',
    model: 'opus',
    effort: 'medium',
    brief: `simplicity: minimum machinery (the contract names Ponytail as binding). Premature
abstraction, speculative generality (traits with one impl that nothing in M1 needs,
config knobs nothing sets), async creep (contract mandates plain std::fs), dependencies
beyond need, dead code. The bar is: could a reviewer delete this and still satisfy the
contract? Then it is a finding.`,
  },
  {
    key: 'test-honesty',
    model: 'opus',
    effort: 'high',
    brief: `test-honesty: do the tests verify the CONTRACT's claims, or do they mirror the
implementation? Would each test catch a real regression — e.g. does the crash-tail test
actually write a truncated line and reopen, does the determinism test compare full
projection state (not just counts), does the seq-gap test construct a real gap, can any
test pass vacuously? Run cargo test yourself to verify the suite is green and report
gates_verified accordingly. A test that cannot fail is a finding.`,
  },
]

function criticPrompt(axis, round) {
  return `You are a blind critic in a gauntlet loop, round ${round}. You did not write this
code and have not seen any earlier critique of it. Work in ${REPO}.

The work under review is the CURRENT UNCOMMITTED CHANGES (run git status --short and
git diff HEAD --stat, then read the changed/new files in full). The contract is
docs/gauntlet/contracts/M1.md.

Your single axis:
${axis.brief}

Rules: evidence-only — every finding cites a file and what you actually read or ran there.
Stay on your axis. Do not grade style preferences. An empty findings list is a valid and
welcome outcome. Also verify the deterministic gates yourself (cargo fmt --check, cargo
clippy --all-targets -- -D warnings, cargo test) and set gates_verified accordingly.`
}

let round = 0
let residual = []
const history = []

while (round < MAX_ROUNDS) {
  round += 1

  const critiques = await parallel(
    AXES.map((axis) => () =>
      agent(criticPrompt(axis, round), {
        label: `critic:${axis.key}:r${round}`,
        model: axis.model,
        effort: axis.effort,
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
  if (allFindings.length > 0) {
    const verdicts = await parallel(
      allFindings.map((f, i) => () =>
        agent(
          `Adversarially verify this gauntlet finding in ${REPO}, round ${round}. Try to REFUTE
it by reading the actual files, the contract docs/gauntlet/contracts/M1.md, and where
useful running the code or tests. Finding:
${JSON.stringify(f)}
Default to refuted=true if the evidence does not clearly support it. reason must be one
sentence grounded in what you read or ran.`,
          { label: `refute:r${round}:${i}`, model: 'opus', effort: 'medium', phase: 'Verify', schema: VERDICT_SCHEMA },
        ).then((v) => ({ finding: f, verdict: v })),
      ),
    )
    confirmed = verdicts.filter(Boolean).filter((v) => v.verdict && !v.verdict.refuted)
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

  if (round >= MAX_ROUNDS) break

  phase('Fix')
  await agent(
    `You are the M1 fixer in a gauntlet loop, round ${round}. Work in ${REPO}. The contract is
docs/gauntlet/contracts/M1.md. A blind critic panel confirmed these findings against the
current uncommitted M1 implementation (each was adversarially verified):

${JSON.stringify(confirmed.map((c) => ({ ...c.finding, confirmed_because: c.verdict.reason })), null, 2)}

Fix every confirmed finding. Preserve the contract's Non-goals (no async, no new runtime
deps, minimum machinery). Keep all gates green: cargo build && cargo fmt --check &&
cargo clippy --all-targets -- -D warnings && cargo test. No git commands. Do not touch
GAUNTLET.md, LESSONS.md, README.md, docs/, reference/.

Return: gates_green, summary of what changed per finding, design_decisions if a fix
required a judgment call, contract_ambiguities if a finding exposed a contract defect.`,
    { label: `fix:r${round}`, model: 'fable', effort: 'high', phase: 'Fix', schema: BUILD_SCHEMA },
  )
}

return { build, rounds: history, residual: residual.map((r) => ({ ...r.finding, confirmed_because: r.verdict.reason })) }