export const meta = {
  name: 'm6-clients-gauntlet',
  description: 'Build M6 (TUI, dashboard, doctor, demo) with round-1 gauntlet, economy shape',
  phases: [
    { title: 'Build', detail: 'Opus builder implements M6 contract, drives gates green' },
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

const HYGIENE = `Probe hygiene (binding): mutation probes ONLY in a disposable copy outside the
project tree (copy src/ + tests/ only, share CARGO_TARGET_DIR with the main checkout so
nothing rebuilds duckdb); NEVER edit the main tree; report every probe; delete copies.
Exploration budget: the M6 layer is the uncommitted diff — read it, the files it
touches, the contract, and directly referenced code; not a whole-repo re-audit. Do NOT
run the opt-in real-Claude tests.`

phase('Build')
const build = await agent(
  `You are the M6 builder for the sergeant-rs prototype — the LAST milestone of the P0.
Work in ${REPO} (M0-M5 committed and green — 178 tests; your work is the uncommitted M6
layer).

Your contract is docs/gauntlet/contracts/M6.md — read it fully. Then: LESSONS.md (all,
L6-L9 binding), GAUNTLET.md's register (D1-D7) and rulings, the proposal sections the
contract cites (29, 30, 31's doctor, 39, and 7's clients-are-equal rule), and the
existing api.rs client surface, cli.rs, and the stub tui.rs/web.rs.

Implement the contract: the ratatui TUI (fleet + work detail, SSE-live, API-only), the
embedded server-rendered HTML dashboard (include_str! assets, EventSource live, bearer
token via sgt web's tokenized URL), sgt doctor (each check names its remedy; --json
stable), and scripts/demo.sh — the section-39 walkthrough that must exit 0 with
resolvable evidence pointers — with ALL FIVE acceptance tests from the contract as real
cargo tests (count against the contract's own list; the demo script is additionally
exercised by acceptance test 4).

The invariants sentence of this milestone: "If the TUI needs a private shortcut, the
API is incomplete." tui.rs and web.rs reach state ONLY through the HTTP API and its
client types. If you find yourself importing runtime/daemon internals, stop and extend
the API instead — that extension is in-contract.

L6: audit any new multi-append sequences (unlikely — this milestone should append
nothing new). L7: revert-probe your own diff before returning. L8: every advertised
flag/command gets a test. Dependency budget: ratatui + crossterm only.

Gates green before returning: cargo build && cargo fmt --check && cargo clippy
--all-targets -- -D warnings && cargo test (all 178 M0-M5 tests stay green). No git
history commands. Do not touch GAUNTLET.md, LESSONS.md, README.md, docs/, reference/.
(README updates happen at close-out, by the orchestrator.)

Return: gates_green, summary, measurements (TUI test approach chosen per the contract's
Unknown; anything else measured), design_decisions (rung-tagged), contract_ambiguities.`,
  { label: 'build:m6-clients', model: 'opus', effort: 'high', phase: 'Build', schema: BUILD_SCHEMA },
)

const AXES = [
  {
    key: 'spec-fidelity',
    model: 'opus',
    effort: 'medium',
    brief: `spec-fidelity: implementation vs docs/gauntlet/contracts/M6.md and proposal sections
29, 30, 31, 39 as scoped. Dashboard embedded and self-contained (section 29: no Node,
no CDN, no filesystem reads at serve time)? TUI opens on bare \`sgt\` (section 30)?
doctor checks the contracted list with remedies? demo.sh walks the section-39 shape
with resolvable evidence pointers? All five acceptance tests present and real?`,
  },
  {
    key: 'invariants',
    model: 'fable',
    effort: 'high',
    brief: `invariants: CLIENTS ARE EQUAL is under test — section 30's own sentence: "If the TUI
needs a private shortcut, the API is incomplete." Hunt for: tui.rs/web.rs importing
anything from runtime/, daemon internals, journal, or storage (API client types only);
the dashboard reading state that isn't served over HTTP; doctor peeking past the API
where the API should answer (doctor MAY inspect the environment — git, claude CLI,
data-dir files — that is its job; the line is runtime STATE, which comes from the API);
token handling that leaks the bearer into logs or journal events; the TUI leaving the
terminal raw on panic; SSE reconnect gaps presented as live truth.`,
  },
  {
    key: 'simplicity',
    model: 'opus',
    effort: 'medium',
    brief: `simplicity: Ponytail ladder. Watch for: a template engine where format! suffices
(the contract names this R1); TUI framework ceremony beyond fleet+detail; speculative
dashboard screens; JS beyond the EventSource wiring; config knobs nothing sets; a
demo.sh that reimplements what sgt commands already print.`,
  },
  {
    key: 'test-honesty',
    model: 'opus',
    effort: 'medium',
    brief: `test-honesty: L7/L8. The clients-are-equal structural test must actually catch an
internals import (probe it). Dashboard tests must assert real seeded data in the HTML,
not just 200s. The empty-cwd probe must genuinely prove no serve-time filesystem reads.
Doctor tests must break the environment and see the named check fail with its remedy.
The demo test must run the actual script and resolve its printed pointers. Would each
test catch a real regression? Run cargo test yourself; set gates_verified.`,
  },
]

phase('Critique')
const critiques = await parallel(
  AXES.map((axis) => () =>
    agent(
      `You are a blind critic in a gauntlet loop, round 1. You did not write this code. Work
in ${REPO}. The work under review is the CURRENT UNCOMMITTED CHANGES against
docs/gauntlet/contracts/M6.md.

BEFORE filing: read GAUNTLET.md's deviation register (D1-D7) and the M1-M5 ledger
rulings, plus LESSONS.md. Re-litigating a registered deviation or ruling requires
arguing the ruling is wrong.

${HYGIENE}

Your single axis:
${axis.brief}

Rules: evidence-only — every finding cites a file and what you read or ran. Stay on
your axis. No style preferences. Empty findings list is valid and welcome. Verify the
deterministic gates yourself (cargo fmt --check, cargo clippy --all-targets --
-D warnings, cargo test — warm-cached) and set gates_verified.`,
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
For EACH finding, try to REFUTE it via the actual files, docs/gauntlet/contracts/M6.md,
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
    `You are the M6 fixer, round 1. Work in ${REPO}. Contract: docs/gauntlet/contracts/M6.md.
LESSONS L6-L9 binding. Confirmed findings (adversarially verified):
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
