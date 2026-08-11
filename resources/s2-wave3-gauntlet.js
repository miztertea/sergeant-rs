export const meta = {
  name: 's2-wave3-gauntlet',
  description: 'S2 wave 3: blind auditor over the fresh residual map, independent batched prober, fixer — the close-out sweep before census and CI lane',
  phases: [
    { title: 'Audit', detail: 'Opus auditor: cheap tricks the waves missed, from the re-measured lcov' },
    { title: 'Probe', detail: 'Independent batched mutation probes, one disposable worktree' },
    { title: 'Fix', detail: 'Fixer over probe survivors' },
  ],
}

// args: { base: '<SHA before wave 3>' }
const REPO = '/home/user/sergeant-rs'
const BASE = args && args.base ? args.base : 'UNSET'

const AUDIT_SCHEMA = {
  type: 'object',
  properties: {
    gates_green: { type: 'boolean' },
    summary: { type: 'string' },
    tests_added: { type: 'array', items: { type: 'string' } },
    guard_map: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          test: { type: 'string' },
          guarded_behavior: { type: 'string' },
          mutation_that_should_kill_it: { type: 'string' },
        },
        required: ['test', 'guarded_behavior', 'mutation_that_should_kill_it'],
      },
    },
    flake_check_10x_green: { type: 'boolean' },
    declined_with_reason: { type: 'array', items: { type: 'string' } },
    escalations: { type: 'array', items: { type: 'string' } },
  },
  required: ['gates_green', 'summary', 'tests_added', 'guard_map', 'flake_check_10x_green', 'declined_with_reason'],
}

const PROBE_SCHEMA = {
  type: 'object',
  properties: {
    worktree_hygiene: { type: 'string' },
    probes_executed: { type: 'integer' },
    kills: { type: 'array', items: { type: 'string' } },
    survivors: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          test: { type: 'string' },
          mutation: { type: 'string' },
          why_it_survived: { type: 'string' },
        },
        required: ['test', 'mutation', 'why_it_survived'],
      },
    },
    notes: { type: 'array', items: { type: 'string' } },
  },
  required: ['worktree_hygiene', 'probes_executed', 'kills', 'survivors'],
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

phase('Audit')
const audit = await agent(
  `You are the S2 wave-3 blind auditor for sergeant-rs — the nim-proxy round-2 pattern: a fresh
skeptic sweeping what two waves of targeted work left uncovered, looking for branches marked
hard that are actually cheap. Work in ${REPO}. Read first: docs/gauntlet/contracts/S2-STABILIZE.md
(Hard rules binding — test-only, no seams, no Cargo.toml, no tests/support/mod.rs, pre-ruled
regions off limits: B1 snapshot path, Analytics::table_rows, real-Claude live paths),
docs/coverage/baseline-2026-08-10.md (the Residuals section especially), and the FRESH re-measured
evidence: docs/coverage/artifacts-2026-08-10/c4-summary.txt (93.58% lines at the current tip) and
target/llvm-cov-reports/html per-line views.

Environment facts from the waves (binding): the suite runs as ROOT — permission-bit tricks
silently succeed; use dir-as-file, wrong-file-type, fd-open-mode, and immutable-bit (chattr +i,
always cleaned up in Drop/defer) tricks instead. EAGAIN==EWOULDBLOCK on Linux flock.

Your sweep, in priority order: (1) engine.rs (was 76.8% functions at baseline — re-check what
remains after the waves; the narrowed residual 'no profile/backend mismatch on a tier-4 route' is
known intake); (2) any module still under 90% lines in the fresh summary; (3) the waves' own
declined-as-hard items — re-derive whether an existing harness trick reaches them; (4) anything
the fresh lcov shows red that a five-line test closes. For each region: add the smallest honest
test, or record it in declined_with_reason (a behavior needing a seam, a genuine
infrastructure-only fault, an instrument artifact — one line each, these go into the ledger).
Chasing the number is not the job; closing real behaviors cheaply is.

Rules: guard_map entry per new test (an independent prober executes them — you do NOT
self-probe); 10x each new test; ONE commit ('test(s2-w3): ...', no Fixes trailers — the twelve
issues are closed; reference #NN only if directly relevant); gates green before returning
(fmt --check, clippy -D warnings, cargo test).`,
  { label: 'audit:w3', model: 'opus', effort: 'high', phase: 'Audit', schema: AUDIT_SCHEMA },
)
log(`w3 audit: ${audit ? audit.tests_added.length : 0} tests, ${audit ? audit.declined_with_reason.length : 0} declined-with-reason`)

let probeResult = null
let fix = null
if (audit && audit.guard_map && audit.guard_map.length > 0) {
  phase('Probe')
  probeResult = await agent(
    `You are the independent mutation prober for S2 wave 3 (L13: you did not write these tests).
Work from ${REPO}; the wave-3 diff is git diff ${BASE}..HEAD. df pre-check (abort under 8 GB).
ONE disposable worktree (git worktree add /tmp/probe-s2w3 HEAD, own target dir, targeted builds
only), execute every guard-map entry plus 2-3 unlisted mutations of your own devising, revert and
verify clean between probes, remove the worktree, verify the main tree untouched
(git -C ${REPO} status --short empty). Report per the schema; survivors verbatim with why.

GUARD MAP (${audit.guard_map.length} entries):
${JSON.stringify(audit.guard_map, null, 2)}`,
    { label: 'probe:w3', model: 'opus', effort: 'high', phase: 'Probe', schema: PROBE_SCHEMA },
  )
  const survivors = probeResult && probeResult.survivors ? probeResult.survivors : []
  log(`w3 probe: ${probeResult ? probeResult.probes_executed : 0} executed, ${survivors.length} survivors`)

  if (survivors.length > 0 || (audit && audit.gates_green === false)) {
    phase('Fix')
    fix = await agent(
      `You are the S2 wave-3 fixer. Work in ${REPO}. Contract: docs/gauntlet/contracts/
S2-STABILIZE.md (test-only, Hard rules binding). Probe survivors (executed evidence):
${JSON.stringify(survivors, null, 2)}
${audit && audit.gates_green === false ? 'The auditor also reported gates RED — reproduce and fix first.' : ''}
Strengthen each surviving test so its mutation kills it; re-execute the mutation in one
disposable worktree (removed after); 10x flake rule; separable commit; gates green.`,
      { label: 'fix:w3', model: 'opus', effort: 'high', phase: 'Fix', schema: BUILD_SCHEMA },
    )
  }
}

return {
  audit: audit ? { tests: audit.tests_added.length, declined: audit.declined_with_reason, escalations: audit.escalations || [], gates: audit.gates_green } : null,
  probes: probeResult ? { executed: probeResult.probes_executed, kills: probeResult.kills.length, survivors: probeResult.survivors.length } : null,
  fix,
}
