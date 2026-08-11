export const meta = {
  name: 's2-wave1-gauntlet',
  description: 'S2 wave 1: four Sonnet builders on disjoint surfaces (fixtures + in-module pokes), lean two-critic panel, refuters, fixer',
  phases: [
    { title: 'Build', detail: 'Four Sonnet builders, disjoint file surfaces, issues #30/#32/#33/#34/#37/#39/#41' },
    { title: 'Critique', detail: 'Test-honesty (Opus-high) + invariants/simplicity (Opus-medium) over the wave diff' },
    { title: 'Verify', detail: 'Batched Opus refuters' },
    { title: 'Fix', detail: 'Fixer if findings confirm' },
  ],
}

// args: { base: '<SHA before wave 1>' }
const REPO = '/home/user/sergeant-rs'
const BASE = args && args.base ? args.base : 'UNSET'

const BUILD_SCHEMA = {
  type: 'object',
  properties: {
    gates_green: { type: 'boolean' },
    summary: { type: 'string' },
    tests_added: { type: 'array', items: { type: 'string' } },
    probes: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          behavior: { type: 'string' },
          probe: { type: 'string' },
          new_test_failed_as_expected: { type: 'boolean' },
        },
        required: ['behavior', 'probe', 'new_test_failed_as_expected'],
      },
    },
    flake_check_10x_green: { type: 'boolean' },
    escalations: { type: 'array', items: { type: 'string' } },
    design_decisions: { type: 'array', items: { type: 'string' } },
  },
  required: ['gates_green', 'summary', 'tests_added', 'probes', 'flake_check_10x_green'],
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

const COMMON_BUILD = `You are an S2 wave-1 builder for sergeant-rs. Work in ${REPO} on the current
branch. Read first: docs/gauntlet/contracts/S2-STABILIZE.md (your contract — the Hard
rules section is binding), docs/coverage/baseline-2026-08-10.md, the GitHub issue text
quoted in your task below, LESSONS.md L5/L7/L12/L13, and the actual source you are
testing (L12: read the artifact, never work from the summary). Coverage evidence:
target/llvm-cov-reports/html (per-line) and the lcov at target/llvm-cov-reports/lcov.info.

RULES, all binding: TEST-ONLY diffs — tests/, #[cfg(test)] modules, fixtures. Zero
production-behavior change; if a behavior is unreachable without a production seam,
STOP on that item and record it in escalations instead of adding the seam. Do not
touch Cargo.toml, GAUNTLET.md, LESSONS.md, docs/, reference/, resources/,
scripts/coverage/ or any file outside your assigned surface. Never run the opt-in
real-Claude tests. Pre-ruled regions are off limits (B1 snapshot path,
Analytics::table_rows).

For EVERY new test: (1) mutation-probe it in a disposable git worktree with its own
target dir (L5) — break the guarded behavior (delete the guard / reject branch /
error mapping) and confirm YOUR test fails; restore; record in probes. (2) run it
10x consecutively (cargo test <name> in a loop) — a flake means not done. Commit
your work as ONE commit on your surface with 'Fixes #NN' trailers for issues you
fully close (only claim Fixes when every behavior in the issue on YOUR surface is
covered; otherwise reference without closing). Gates green before returning:
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test.
Concurrent builders share the cargo lock — waiting on it is normal, never create
alternate target dirs. Return the schema fields honestly.`

phase('Build')
const SURFACES = [
  {
    key: 'domain-fake',
    files: 'src/domain/{workflow,workspace,event,work}.rs + src/backend/fake.rs (in-module #[cfg(test)] only)',
    task: `Issues #32 (domain validation reject branches: WorkflowError NameMismatch/NoStages/DuplicateStage;
WorkspaceError NoRepositories/DuplicateProfile; unix_millis structural rejects; WorkState Display for
Pending/Waiting) and #39 (fake backend: parse_script grammar table — each verb's parse + dispatch,
reject cases; the perf harness's measured FIFO semantics in scripts/perf/common.sh are the behavior
spec, read them).`,
  },
  {
    key: 'projections-telemetry',
    files: 'src/runtime/analytics.rs + src/runtime/graph.rs + src/telemetry.rs (in-module #[cfg(test)] only)',
    task: `Issues #33 (analytics malformed-event guards for the six kinds; the KIND_EXECUTION_RECONCILED
fold; graph derive guards; truncate() shortening; decide-and-document the Analytics Debug impl) and
#34 (telemetry: enabled from_config reaching Self::otlp — the M5 OTLP smoke pattern already binds a
collector, reuse it; timestamp guard; malformed-event guards; stage supersession; tool is_error
status; needs_input counter; wait-duration; backend_failure counter; terminal-state span cleanup —
use the opentelemetry_sdk testing feature already in dev-deps; if it cannot express an assertion,
escalate rather than add a dep).`,
  },
  {
    key: 'clients-decoder',
    files: 'src/tui.rs + src/web.rs + src/api.rs (in-module #[cfg(test)] only)',
    task: `Issue #41 (tui respond-flow abandonment statuses via the existing key-event test pattern;
web render_work second fixture shape) and #37's pure-function part ONLY (the EventStream frame
decoder: multi-line data: coalescing, comment/keep-alive frames; stage_label's (Some,None)/(None,_)
arms). Do NOT touch #37's SSE race items — those are wave 2.`,
  },
  {
    key: 'journal-fixtures',
    files: 'tests/m1_event_core.rs only',
    task: `Issue #30's fixture-shaped items ONLY: (a) empty trailing rotation segment — small
segment_max_bytes, rotate, do not append, replay_after skips it; (b) malformed FIRST line of a
second segment surfacing first_seq's own Malformed{line:1} via the replay_after skip-scan (mirror
the existing malformed-line tests at ~:494 and ~:676); (c) Replay::next io arms — segment deleted
between listing and next(), and an invalid-UTF-8 (raw 0xFF) segment, both asserting the error
surfaces and the iterator stays failed. Do NOT touch the poison/rollback pair — wave 2's in-module
work.`,
  },
]

const builds = await parallel(
  SURFACES.map((s) => () =>
    agent(
      `${COMMON_BUILD}

YOUR SURFACE (exclusive — no other builder touches these files, do not leave it):
${s.files}

YOUR TASK:
${s.task}`,
      { label: `build:${s.key}`, model: 'sonnet', effort: 'high', phase: 'Build', schema: BUILD_SCHEMA },
    ).then((r) => ({ surface: s.key, r })),
  ),
)
log(`wave 1 builds: ${builds.filter(Boolean).map((b) => `${b.surface}:${b.r && b.r.gates_green ? 'green' : 'RED'}`).join(' ')}`)

const HYGIENE = `Probe hygiene (L5): mutation probes ONLY in a disposable git worktree with its own
target dir; never the main tree; report every probe. Exploration budget: the wave-1 layer is
\`git diff ${BASE}..HEAD\` restricted to tests/ and #[cfg(test)] modules — plus each builder's
returned probes list. Never run opt-in real-Claude tests; never run a full coverage build.`

phase('Critique')
const AXES = [
  {
    key: 'test-honesty',
    model: 'opus',
    effort: 'high',
    brief: `test-honesty over the wave-1 diff (L7/L13 — the builders probed their own tests; you
re-probe independently). Sample EVERY builder's surface: for each cluster of new tests, mutation-probe
at least the highest-value behavior yourself in a disposable worktree (delete the reject branch /
guard / mapping; the new test must fail). Hunt specifically for: tests that assert the fixture
rather than the behavior (a NameMismatch test that never checks WHICH error); tests green against
a broken guard; composition gaps (the part pinned, the caller's path not — Bug Sprint 1 / round-2
shape); mock-shaped tests that mirror implementation. Also verify no new test is flaky: pick the
three most timing-adjacent new tests and run each 10x.`,
  },
  {
    key: 'invariants-simplicity',
    model: 'opus',
    effort: 'medium',
    brief: `invariants + simplicity over the wave-1 diff. Invariants: the diff must be genuinely
test-only — any production line changed, any #[cfg(test)] leaking helpers into production
compilation, any pre-ruled region tested (B1 snapshot path, table_rows), any real-Claude path
touched is an error. Simplicity: Ponytail — fixture machinery beyond need, new test frameworks,
helper abstractions where a plain fixture would do, duplicated fixtures the suite already owns.`,
  },
]
const critiques = await parallel(
  AXES.map((axis) => () =>
    agent(
      `You are a fresh blind critic, S2 wave-1 panel. You did not write the diff. Work in ${REPO}.
Read first: docs/gauntlet/contracts/S2-STABILIZE.md, docs/gauntlet/contracts/S0.md rulings,
LESSONS.md. ${HYGIENE}

Builders' self-reports (their claims are inputs, not evidence):
${JSON.stringify(builds.filter(Boolean).map((b) => ({ surface: b.surface, summary: b.r ? b.r.summary : null, tests_added: b.r ? b.r.tests_added : null, probes: b.r ? b.r.probes : null, escalations: b.r ? b.r.escalations : null })), null, 2)}

Your single axis: ${axis.brief}

Evidence-only; cite files and what you ran. Empty findings valid. Verify gates yourself
(cargo fmt --check, cargo clippy --all-targets -- -D warnings, cargo test); set gates_verified.`,
      { label: `critic:${axis.key}`, model: axis.model, effort: axis.effort, phase: 'Critique', schema: FINDINGS_SCHEMA },
    ).then((r) => ({ axis: axis.key, result: r })),
  ),
)

const gatesRed = critiques.filter(Boolean).some((c) => c.result && c.result.gates_verified === false)
const allFindings = critiques
  .filter(Boolean)
  .flatMap((c) => (c.result ? c.result.findings.map((f) => ({ ...f, axis: f.axis || c.axis })) : []))
log(`wave-1 panel: ${allFindings.length} findings${gatesRed ? ' (GATES RED)' : ''}`)

phase('Verify')
let confirmed = []
if (allFindings.length > 0) {
  const byAxis = {}
  allFindings.forEach((f) => { const k = f.axis || 'unknown'; (byAxis[k] = byAxis[k] || []).push(f) })
  const batches = await parallel(
    Object.entries(byAxis).map(([axisKey, fs]) => () =>
      agent(
        `Adversarially verify these S2 wave-1 findings (axis: ${axisKey}) in ${REPO} against the
contract (docs/gauntlet/contracts/S2-STABILIZE.md), the diff ${BASE}..HEAD, and the code. Try to
REFUTE each. ${HYGIENE}
Findings:
${JSON.stringify(fs.map((f, j) => ({ index: j, axis: f.axis, file: f.file, severity: f.severity, summary: f.summary, evidence: f.evidence })), null, 2)}
Default refuted=true on unclear evidence. A verdict for every index.`,
        { label: `refute:${axisKey}`, model: 'opus', effort: 'medium', phase: 'Verify', schema: BATCH_VERDICT_SCHEMA },
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
log(`wave-1 verify: ${confirmed.length}/${allFindings.length} confirmed`)

let fix = null
if (confirmed.length > 0 || gatesRed) {
  phase('Fix')
  const architectural = confirmed.some((f) => f.severity === 'error')
  fix = await agent(
    `You are the S2 wave-1 fixer. Work in ${REPO}. Contract: docs/gauntlet/contracts/S2-STABILIZE.md
(test-only, Hard rules binding). Confirmed findings:
${JSON.stringify(confirmed, null, 2)}
${gatesRed ? 'A critic reported gates RED — reproduce and fix first.' : ''}
Every fix keeps L7 (mutation-probe in a disposable worktree) and the 10x flake rule. Separable
commits. Gates green before returning. Return the schema honestly.`,
    { label: 'fix', model: architectural ? 'opus' : 'sonnet', effort: 'high', phase: 'Fix', schema: BUILD_SCHEMA },
  )
}

return { builds: builds.filter(Boolean).map((b) => ({ surface: b.surface, gates: b.r && b.r.gates_green, tests: b.r ? b.r.tests_added.length : 0, escalations: b.r ? b.r.escalations : null })), findings: allFindings.length, confirmed, fix, gatesRed }
