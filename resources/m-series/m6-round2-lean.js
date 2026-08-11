export const meta = {
  name: 'm6-round2-lean',
  description: 'M6 lean follow-up panel over round-1 fixes + checkpoint gate',
  phases: [
    { title: 'Critique', detail: 'Fresh critics, medium effort, committed M6 state' },
    { title: 'Verify', detail: 'One batched refuter per axis' },
    { title: 'Fix', detail: 'Opus fixer: confirmed findings + seeded daemon-leak hygiene' },
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

// Orchestrator-measured (L9: orchestrator rulings are findings too). Reproduced
// twice: ~89 leaked `sgt ... daemon` processes after repeated suite runs on
// 2026-08-09, and exactly one leaked daemon (pid 6116, --data-dir /tmp/.tmpvvnRBl)
// after a single fresh 195-test run today. Tests that exercise client auto-spawn
// leave the spawned daemon running; nothing reaps it.
const SEEDED = [
  {
    axis: 'test-honesty',
    file: 'tests/m6_surfaces.rs',
    severity: 'error',
    summary:
      'Tests that auto-spawn a daemon leak the process: at least one live `sgt --data-dir /tmp/... daemon` survives every full suite run (~89 accumulated over a working day). Any test that can spawn a daemon must reap it (kill by recorded pid / Drop guard), across all suites that hit the auto-spawn path, not just m6.',
    evidence:
      'Orchestrator measurement 2026-08-09: pgrep -fa "target/debug/sgt" after one clean cargo test run showed pid 6116 `sgt --data-dir /tmp/.tmpvvnRBl daemon` still alive; earlier the same day 89 accumulated instances were killed. Reproduce: cargo test && pgrep -f "debug/sgt ".',
    confirmed_because: 'orchestrator-measured, reproduced twice (see evidence); seeded per L9',
  },
]

const HYGIENE = `Probe hygiene (binding): mutation probes ONLY in a disposable copy outside the
project tree (copy src/ + tests/ + scripts/ only, share CARGO_TARGET_DIR with the main
checkout so nothing rebuilds duckdb); NEVER edit the main tree; report every probe;
delete copies. Exploration budget: the M6 layer is \`git diff 3a70148..HEAD\` — read that
diff, the files it touches, docs/gauntlet/contracts/M6.md, and directly referenced code;
not a whole-repo re-audit. Do NOT run the opt-in real-Claude tests. If you run cargo
test, afterwards reap any daemon the suite leaked: pkill -f "target/debug/sgt" is known
to also match your own shell — use pkill -f "debug/sgt --data-dir" instead.`

const AXES = [
  {
    key: 'spec-fidelity',
    model: 'opus',
    focus: `contract fidelity (docs/gauntlet/contracts/M6.md; proposal sections 29, 30, 31's doctor, 39). Did the round-1 fixes (13 confirmed, commit 69cb52e) and the checkpoint-gate doc commit e972e64 stay within contract? All five acceptance tests still real? demo.sh still walks the section-39 shape with resolvable evidence pointers? The contract's resolved Unknowns (TUI TestBackend approach, sgt web auto-spawn) reflected truthfully in the code?`,
  },
  {
    key: 'invariants',
    model: 'fable',
    focus: `CLIENTS ARE EQUAL after the fixes — section 30's sentence: "If the TUI needs a private shortcut, the API is incomplete." Did any round-1 fix reintroduce an internals import in tui.rs/web.rs, dashboard state not served over HTTP, bearer-token leakage into logs/journal/URLs beyond the registered token-in-URL tradeoff, terminal-left-raw panic paths, or SSE reconnect gaps presented as live truth? doctor may inspect the environment (that is its job) but runtime STATE must come from the API.`,
  },
  {
    key: 'simplicity',
    model: 'opus',
    focus: `Ponytail ladder: did the round-1 fixes add machinery beyond their findings? Template-engine creep past format!, speculative dashboard screens, JS beyond EventSource wiring, TUI ceremony beyond fleet+detail, demo.sh reimplementing what sgt commands already print, dead config knobs.`,
  },
  {
    key: 'test-honesty',
    model: 'opus',
    focus: `L7/L8 on the fixed state: would reverting each round-1 fix leave the suite green (revert-probe in a disposable copy)? Does the clients-are-equal structural test still catch a planted internals import? Dashboard tests assert real seeded data, not just 200s? Empty-cwd probe genuinely proves no serve-time filesystem reads? Doctor failure tests still break the environment and see the named check + remedy? Also: process hygiene — a seeded orchestrator finding says auto-spawn tests leak daemons; look for OTHER resource leaks the suites leave behind (temp dirs, worktrees, ports).`,
  },
]

phase('Critique')
const critiques = await parallel(
  AXES.map((axis) => () =>
    agent(
      `You are a fresh blind critic in a gauntlet loop, follow-up round after round-1 fixes
(13 confirmed findings fixed, commit 69cb52e) and a passed checkpoint gate (doc commit
e972e64 adopted). You did not write this code. Work in ${REPO}.

BEFORE filing: read GAUNTLET.md's deviation register (D1-D7) and the M1-M6 rulings,
plus LESSONS.md. Re-litigating a registered deviation or ruling requires arguing the
ruling is wrong. Your job is what round 1 MISSED or what the fixes REGRESSED — not
re-reporting fixed items.

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
let confirmed = [...SEEDED]
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
    `You are the M6 fixer, follow-up round. Work in ${REPO}. Contract:
docs/gauntlet/contracts/M6.md. LESSONS L6-L9 binding. Confirmed findings
(adversarially verified; the daemon-leak one is orchestrator-measured and seeded — fix
it across ALL test suites that hit the auto-spawn path, and add a pinning check so a
future leak fails a test rather than accumulating processes):
${JSON.stringify(confirmed, null, 2)}
${gatesRed ? 'A critic also reported gates RED — reproduce and fix that first.' : ''}
Fix every confirmed finding; preserve Non-goals and the ratatui+crossterm dependency
budget; rung-tagged design decisions. Gates green: cargo build && cargo fmt --check &&
cargo clippy --all-targets -- -D warnings && cargo test. After your final cargo test,
verify zero leaked daemons: pgrep -f "debug/sgt --data-dir" must find nothing (reap any
stragglers with pkill -f "debug/sgt --data-dir" and make sure your fix prevents new
ones). Do NOT run opt-in real-Claude tests. No git history commands. Do not touch
GAUNTLET.md, LESSONS.md, README.md, docs/, reference/.
Return gates_green, per-finding summary, rung-tagged design_decisions,
contract_ambiguities.`,
    { label: 'fix2', model: 'opus', effort: 'high', phase: 'Fix', schema: BUILD_SCHEMA },
  )
}

return { findings: allFindings.length, confirmed, fix, gatesRed }
