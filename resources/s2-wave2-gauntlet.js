export const meta = {
  name: 's2-wave2-gauntlet',
  description: 'S2 wave 2 (fault injection): three Sonnet builders on disjoint surfaces, independent batched mutation-probe stage, invariants critic, fixer',
  phases: [
    { title: 'Build', detail: 'Three Sonnet builders: storage in-module, m2+m6 suites, daemon/surface+m3' },
    { title: 'Probe', detail: 'Independent Opus prober executes ALL mutation probes batched in one disposable worktree (L13 as design)' },
    { title: 'Verify', detail: 'Refuters over invariants/simplicity findings; probe survivors are direct evidence' },
    { title: 'Fix', detail: 'Fixer over probe survivors + confirmed findings' },
  ],
}

// args: { base: '<SHA before wave 2>' }
const REPO = '/home/user/sergeant-rs'
const BASE = args && args.base ? args.base : 'UNSET'

const BUILD_SCHEMA = {
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
    escalations: { type: 'array', items: { type: 'string' } },
    design_decisions: { type: 'array', items: { type: 'string' } },
  },
  required: ['gates_green', 'summary', 'tests_added', 'guard_map', 'flake_check_10x_green'],
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

const COMMON_BUILD = `You are an S2 wave-2 builder for sergeant-rs. Work in ${REPO} on the current
branch. Read first: docs/gauntlet/contracts/S2-STABILIZE.md (Hard rules binding), the issue text in
your task, LESSONS.md L5/L7/L12/L13, and the actual source under test (L12). Coverage evidence:
target/llvm-cov-reports/html.

WAVE-2 PROTOCOL CHANGES (learned from wave 1, binding):
- You do NOT run mutation probes. Instead you return guard_map: for every new test, the exact
  mutation (file, function, edit) that should make it fail. An independent prober executes the
  whole wave's map batched in one disposable worktree (L13 as design: non-author probes are
  stronger than self-probes, and one worktree respects the disk budget).
- NEVER touch any file outside your exclusive surface, even transiently. No unscoped
  \`cargo fmt\` (run \`cargo fmt --check\` only). Other builders share this checkout.
- tests/support/mod.rs is off-limits; a needed change there is an escalation, not an edit.
- df pre-check: if under 8 GB free, stop and escalate.

Still yours: 10x consecutive runs of each new test (a flake means not done); gates green before
returning (cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test — the
cargo lock serializes with your peers, waiting is normal); ONE commit per issue-cluster on your
surface, 'Fixes #NN' trailers ONLY for issues you fully close; test-only diffs — a behavior
unreachable without a production seam is an escalation, never a seam. Pre-ruled regions off
limits. Return the schema honestly — wave 1 caught a placeholder self-report; reports are
cross-checked against the diff.`

phase('Build')
const SURFACES = [
  {
    key: 'storage-inmodule',
    files: 'src/runtime/{journal,blob,fsutil,analytics}.rs — #[cfg(test)] modules ONLY',
    task: `Issue #30's remaining pair (in-module: poison a Journal by swapping its private segment_file
for a read-only handle so write_all fails — assert Err + poisoned latch + subsequent
Err(Poisoned); and the torn-append rollback evidence per the issue text) and issue #31 (blob
get non-NotFound io error via dir-as-file; fsutil non-WouldBlock lock error and ancestor-walk
break arm; analytics remove_if_present non-NotFound via read-only parent dir). Trailer 'Fixes #30'
only if the fixture items already landed in wave 1 plus your pair covers every behavior in the
issue; 'Fixes #31' likewise.`,
  },
  {
    key: 'suites-m2-m6',
    files: 'tests/m2_daemon_api.rs + tests/m6_surfaces.rs ONLY',
    task: `Issue #36 (input/retry idempotency triple: malformed body, invalid command_id, duplicate
command_id byte-identical replay — mirror the existing submit tests; engine_error_status Core→500
and NoRun→404 through real calls; post-accept start() failure via a fake-backend script whose
first start fails — if the fake grammar cannot express it, escalate, do not extend the grammar),
issue #37's SSE race pair (history replay Err mid-tail; subscriber disconnect during history
replay — the m2 SSE helpers own this machinery), issue #38 (SGT_DATA_DIR/XDG/HOME fallbacks via
env-controlled spawns through the DataDir helpers; sgt work list non-JSON both branches; retry
success print; BROWSER=/bin/false error branch; doctor per-check failure arms in m6 via
dir-as-file tricks).`,
  },
  {
    key: 'daemon-surface-m3',
    files: 'src/daemon.rs + src/runtime/surface.rs #[cfg(test)] modules + tests/m3_execution.rs ONLY',
    task: `Issue #35 (committer-thread failure paths: sink send after committer gone, Weak upgrade
failure mid-shutdown; export_events Lagged via a tiny-capacity broadcast channel and Closed
force-flush — in-module or through start_with shutdown-ordering tests) and issue #40
(rematerialize's deleted-branch fallback to base_sha — delete the branch between bind and retry
in a real-git m3 fixture; teardown's worktree-remove failure via an unremovable dir, asserting
RetainedError disposition and journaled evidence).`,
  },
]

const builds = await parallel(
  SURFACES.map((s) => () =>
    agent(
      `${COMMON_BUILD}

YOUR SURFACE (exclusive):
${s.files}

YOUR TASK:
${s.task}`,
      { label: `build:${s.key}`, model: 'sonnet', effort: 'high', phase: 'Build', schema: BUILD_SCHEMA },
    ).then((r) => ({ surface: s.key, r })),
  ),
)
log(`wave 2 builds: ${builds.filter(Boolean).map((b) => `${b.surface}:${b.r && b.r.gates_green ? 'green' : 'RED'}`).join(' ')}`)

const guardMaps = builds
  .filter(Boolean)
  .flatMap((b) => (b.r && b.r.guard_map ? b.r.guard_map.map((g) => ({ ...g, surface: b.surface })) : []))

phase('Probe')
const probeResult = await agent(
  `You are the independent mutation prober for S2 wave 2 (the wave's test-honesty stage runs
through you — L13: you did not write these tests). Work from ${REPO}. Read
docs/gauntlet/contracts/S2-STABILIZE.md and the wave-2 diff (git diff ${BASE}..HEAD, tests/ and
cfg(test) modules only).

PROTOCOL: df pre-check (abort under 8 GB free, report). ONE disposable git worktree
(git worktree add /tmp/probe-s2w2 HEAD, its OWN target dir via cd — never CARGO_TARGET_DIR, never
the main tree), build once, then execute EVERY entry in the guard map below as a batched
mutation-probe campaign: apply the mutation, run the named test (targeted: cargo test --lib X or
--test mX Y), record kill or SURVIVAL, revert the mutation (verify with git diff --stat clean
between probes). Where a builder's stated mutation looks wrong or too gentle, design a better one
— your judgment outranks their map. Also probe 2-3 mutations of YOUR choosing per surface that
the maps did NOT list (the wave-1 lesson: unlisted compositions are where pins fail). Remove the
worktree when done (git worktree remove --force; report before/after df). Main tree: read-only —
verify at the end (git -C ${REPO} status --short must show nothing) and report it.

GUARD MAP (${guardMaps.length} entries):
${JSON.stringify(guardMaps, null, 2)}

Return: worktree_hygiene (the df/status/removal evidence), probes_executed, kills (test names),
survivors (each with why — these become confirmed findings verbatim), notes.`,
  { label: 'probe:batched', model: 'opus', effort: 'high', phase: 'Probe', schema: PROBE_SCHEMA },
)

const invCritic = await agent(
  `You are a fresh blind critic, S2 wave-2 panel, axis invariants+simplicity. You did not write
the diff. Work in ${REPO}, read-only mutation-wise (no probes — the prober owns those). Read:
docs/gauntlet/contracts/S2-STABILIZE.md, S0.md rulings, LESSONS.md. Scope: git diff
${BASE}..HEAD. Invariants: genuinely test-only (no production line changed, no cfg(test) leakage,
no pre-ruled region, no real-Claude path, Cargo.toml and tests/support/mod.rs untouched);
fault-injection fixtures must clean up after themselves (a read-only dir left behind breaks the
next run — check Drop/cleanup paths); env-var tests must not leak env into parallel peers
(std::env::set_var in a parallel harness is a race — flag any). Simplicity: Ponytail over the
diff — machinery beyond need, duplicated fixtures. Evidence-only; empty findings valid. Verify
gates yourself; set gates_verified.`,
  { label: 'critic:inv-simplicity', model: 'opus', effort: 'medium', phase: 'Probe', schema: FINDINGS_SCHEMA },
)

const survivors = (probeResult && probeResult.survivors ? probeResult.survivors : []).map((s) => ({
  axis: 'test-honesty',
  file: s.test,
  summary: `Probe survivor: ${s.test} stays green under mutation "${s.mutation}"`,
  evidence: s.why_it_survived,
  severity: 'error',
}))
log(`probe: ${probeResult ? probeResult.probes_executed : 0} executed, ${survivors.length} survivors; inv critic: ${invCritic ? invCritic.findings.length : 0} findings`)

phase('Verify')
let confirmed = [...survivors]
const invFindings = invCritic ? invCritic.findings : []
if (invFindings.length > 0) {
  const res = await agent(
    `Adversarially verify these S2 wave-2 invariants/simplicity findings in ${REPO} against the
contract and the diff ${BASE}..HEAD. Try to REFUTE each; default refuted=true on unclear
evidence. No tree mutations. Findings:
${JSON.stringify(invFindings.map((f, j) => ({ index: j, file: f.file, severity: f.severity, summary: f.summary, evidence: f.evidence })), null, 2)}
A verdict for every index.`,
    { label: 'refute:inv-simplicity', model: 'opus', effort: 'medium', phase: 'Verify', schema: BATCH_VERDICT_SCHEMA },
  )
  if (res) {
    for (const v of res.verdicts) {
      const f = invFindings[v.index]
      if (f && !v.refuted) confirmed.push({ ...f, axis: 'invariants-simplicity', confirmed_because: v.reason })
    }
  }
}
const gatesRed = invCritic ? invCritic.gates_verified === false : false
log(`wave-2 confirmed: ${confirmed.length} (${survivors.length} probe survivors + ${confirmed.length - survivors.length} panel)`)

let fix = null
if (confirmed.length > 0 || gatesRed) {
  phase('Fix')
  fix = await agent(
    `You are the S2 wave-2 fixer. Work in ${REPO}. Contract: docs/gauntlet/contracts/S2-STABILIZE.md
(test-only, Hard rules binding). Confirmed findings (probe survivors are executed evidence, not
claims):
${JSON.stringify(confirmed, null, 2)}
${gatesRed ? 'The critic reported gates RED — reproduce and fix first.' : ''}
Every strengthened test gets its mutation re-executed in a disposable worktree (one at a time,
removed after — the wave-1 fixer's batch pattern). 10x flake rule holds. Separable commits;
gates green before returning.`,
    { label: 'fix', model: 'opus', effort: 'high', phase: 'Fix', schema: BUILD_SCHEMA },
  )
}

return {
  builds: builds.filter(Boolean).map((b) => ({ surface: b.surface, gates: b.r && b.r.gates_green, tests: b.r ? b.r.tests_added.length : 0, escalations: b.r ? b.r.escalations : null })),
  probes: probeResult ? { executed: probeResult.probes_executed, kills: probeResult.kills.length, survivors: survivors.length, hygiene: probeResult.worktree_hygiene } : null,
  confirmed,
  fix,
  gatesRed,
}
