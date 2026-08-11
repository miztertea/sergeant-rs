export const meta = {
  name: 'bs2-round2-and-44',
  description: '#44 group-commit build, then ONE shared lean round-2 panel over BS2 round-1 + #44 (batched per the 2026-08-11 economy ruling)',
  phases: [
    { title: 'Build44', detail: 'journal group commit: one fsync per lock hold (Opus)', model: 'opus' },
    { title: 'Panel', detail: 'lean round 2: fresh critics on the confirmed-finding axes + #44, batched refuter' },
    { title: 'Probe', detail: 'independent prober: #44 guard map + BS2 round-1 fix pins' },
    { title: 'Fix', detail: 'fixer over confirmed findings + survivors' },
  ],
}

// args: { base: '<BS2 round-1 BASE>', mid: '<SHA where BS2 round 1 ended / #44 starts>' }
const REPO = '/home/miztertea/sergeant-rs'
const BASE = args && args.base ? args.base : 'UNSET'
const MID = args && args.mid ? args.mid : 'UNSET'

const G = `You are working on sergeant-rs on host Cerberus (non-root). Read IN ORDER before acting: ${REPO}/CLAUDE.md, ${REPO}/GAUNTLET.md deviation register + backlog, ${REPO}/LESSONS.md, ${REPO}/docs/environments/cerberus.md. cargo needs PATH="$HOME/.cargo/bin:$PATH" in every shell. Gates before done: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test. Daemon tests via tests/support DataDir guard. Every fix pinned (L7), separable commits (L10), fail closed on ambiguity. Commits end "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>". Do NOT push. Leak check: pgrep -af "sergeant-rs/target/debug/sgt [-]-data-dir" must be empty (the sergeant-harvest worktree runs a LEGITIMATE daemon — never kill anything outside sergeant-rs/target paths).`

const BUILD_SCHEMA = {
  type: 'object',
  required: ['commits', 'tests_added', 'gates_green', 'design_decisions', 'guard_map', 'perf_evidence', 'notes'],
  properties: {
    commits: { type: 'array', items: { type: 'string' } },
    tests_added: { type: 'number' },
    gates_green: { type: 'boolean' },
    design_decisions: { type: 'array', items: { type: 'string' } },
    guard_map: { type: 'array', items: { type: 'object', required: ['test', 'guarded_behavior', 'mutation_that_should_kill_it'], properties: { test: { type: 'string' }, guarded_behavior: { type: 'string' }, mutation_that_should_kill_it: { type: 'string' } } } },
    perf_evidence: { type: 'string' },
    notes: { type: 'string' },
  },
}
const FINDINGS_SCHEMA = { type: 'object', required: ['findings', 'summary'], properties: { summary: { type: 'string' }, findings: { type: 'array', items: { type: 'object', required: ['id', 'severity', 'claim', 'evidence'], properties: { id: { type: 'string' }, severity: { type: 'string', enum: ['error', 'warning', 'info'] }, claim: { type: 'string' }, evidence: { type: 'string' } } } } } }
const VERDICTS_SCHEMA = { type: 'object', required: ['verdicts', 'tree_clean'], properties: { tree_clean: { type: 'boolean' }, verdicts: { type: 'array', items: { type: 'object', required: ['id', 'verdict', 'basis'], properties: { id: { type: 'string' }, verdict: { type: 'string', enum: ['CONFIRMED', 'REFUTED', 'PLAUSIBLE'] }, basis: { type: 'string' } } } } } }
const PROBE_SCHEMA = { type: 'object', required: ['worktree_hygiene', 'probes_executed', 'kills', 'survivors'], properties: { worktree_hygiene: { type: 'string' }, probes_executed: { type: 'integer' }, kills: { type: 'array', items: { type: 'string' } }, survivors: { type: 'array', items: { type: 'object', required: ['test', 'mutation', 'why_it_survived'], properties: { test: { type: 'string' }, mutation: { type: 'string' }, why_it_survived: { type: 'string' } } } }, notes: { type: 'array', items: { type: 'string' } } } }

phase('Build44')
const b44 = await agent(`${G}
Your scope — issue #44 (read it: gh issue view 44), the journal group commit. Governing evidence to read first: docs/perf/n3-two-phase-boundary-2026-08-10.md (the regression + A-N3-1's rejected compound-event alternative — that rejection is SETTLED, the §22.5 crash windows stay), docs/perf/baseline-cerberus-2026-08-11.md (this host: burst-50 42.0 works/s, 18 events/work), src/runtime/journal.rs and the Core::commit path in src/runtime/engine.rs (find where events append + fsync per commit).
Outcome: events appended within ONE authoritative core-lock hold share a single fsync; crash semantics BETWEEN holds unchanged (an event whose commit returned is durable exactly as today). No compound events; no event-count change; no crash-window deletion.
Mandatory analysis, recorded in the landing commit message or a doc comment: L6 adjacent-append — what happens when the daemon dies BETWEEN grouped appends and AFTER the group's appends but BEFORE its single fsync (the answer must be: the existing torn-tail/segment recovery already covers it, PROVEN by a crash-injection test in the §22.5 style, or the design changes until it does).
Perf evidence (perf_evidence field): scripts/perf/s1-burst.sh against a scratch outdir, 3 reps before your change (or cite the committed Cerberus baseline as the before) and 3 after; report burst-50 throughput and note ambient load honestly (a harvest workflow's Sonnet actors run on this host — mostly API-bound; state what pgrep/loadavg showed). The R-N0-4 budget is ≥28 works/s; the interesting number is how much of the N3 regression the group commit recovers.
Fixes #44 trailer on exactly the closing commit. Guard map per new test.`,
  { label: 'build:#44', phase: 'Build44', model: 'opus', effort: 'high', schema: BUILD_SCHEMA })
log(`#44: ${b44 ? b44.commits.length + ' commits, gates=' + b44.gates_green + ', perf: ' + b44.perf_evidence.slice(0, 120) : 'FAILED'}`)
if (!b44 || !b44.gates_green) throw new Error('#44 build not green — stop')

phase('Panel')
const axes = [
  { key: 'invariants-r2', brief: `Axis: INVARIANTS, lean round 2. Two diffs under one review: (A) BS2 round-1 fixes (git diff ${BASE}..${MID} — the settle driver as a fourth performer, §14.5 currency pins, shutdown ordering, stderr channel, permission-mode profile config); round 1 confirmed INV-1 (respond-path settle) and INV-3 (daemon.stopped ordering) — verify their fixes hold and hunt what round 1 missed, especially: driver-vs-cancel races, the window before the driver's first tick after restart (recovery must own it), §22.6 lock discipline of the new performer, whether removing --dangerously-skip-permissions from REQUIRED_FLAGS leaves any test/doctor path still assuming it. (B) the NEW #44 group-commit diff (git diff ${MID}..HEAD): single-fsync-per-hold — check durability-boundary honesty (nothing observable claims durability before the hold's fsync), the L6 torn-group analysis actually proves what it claims, and no §22.5-pinned window silently widened.` },
  { key: 'test-honesty-r2', brief: `Axis: TEST-HONESTY, lean round 2. Same two diffs (git diff ${BASE}..${MID}, git diff ${MID}..HEAD). Round 1 confirmed TH-2/3/4/9 — verify the fixes pin what they claim. Then: do the new settle-seam tests actually stand inside the turn-end window (DaemonConfig.completion_poll hour-cadence trick) or do any race a sleep? Does the #44 crash-injection test actually kill inside the claimed window (grep for the injection seam it uses)? Does any new test mirror implementation instead of contract? Are the a5 probe-gate's two arms each falsifiable on the environment that runs them? Do NOT execute mutations in the main tree; reason from reading — the prober executes.` },
]
const critiques = await parallel(axes.map(a => () => agent(
  `You are a fresh blind critic for sergeant-rs (lean round 2 — you did not build and did not see round 1 beyond what is written). Work read-only in ${REPO} (cargo test allowed with PATH="$HOME/.cargo/bin:$PATH"; NEVER edit files). Read first: ${REPO}/docs/gauntlet/contracts/BS2-ADAPTER-SETTLE.md, ${REPO}/GAUNTLET.md register + rulings (L3: re-litigating a settled ruling requires arguing the ruling wrong), ${REPO}/LESSONS.md.
${a.brief}
Findings carry evidence (file:line read, test output produced). Severity error/warning/info.`,
  { label: `critic:${a.key}`, phase: 'Panel', model: 'opus', effort: 'high', schema: FINDINGS_SCHEMA })))
const allFindings = critiques.filter(Boolean).flatMap((c, i) => c.findings.map(f => ({ ...f, id: `${axes[i].key}:${f.id}` })))
log(`Panel r2: ${allFindings.length} findings (${allFindings.filter(f => f.severity === 'error').length} error)`)

phase('Refute')
let verdicts = []
if (allFindings.length > 0) {
  const r = await agent(
    `Batched adversarial refuter, sergeant-rs lean round 2. You built nothing and wrote none of these findings. Work from ${REPO}; PATH="$HOME/.cargo/bin:$PATH". Refute or confirm each with evidence; PLAUSIBLE only when neither side lands. Mutation probes ONLY in one disposable worktree (git worktree add /tmp/r2-refute HEAD, own CARGO_TARGET_DIR=/tmp/r2-refute-target), removed after, main tree verified untouched. FINDINGS:
${JSON.stringify(allFindings, null, 2)}`,
    { label: 'refuter', phase: 'Panel', model: 'opus', effort: 'high', schema: VERDICTS_SCHEMA })
  verdicts = r ? r.verdicts : []
  if (r && r.tree_clean === false) log('WARNING: refuter reports main tree dirty — orchestrator must inspect')
}
const confirmed = verdicts.filter(v => v.verdict === 'CONFIRMED')
const plausible = verdicts.filter(v => v.verdict === 'PLAUSIBLE')
log(`Refute: ${confirmed.length} confirmed / ${plausible.length} plausible / ${verdicts.filter(v => v.verdict === 'REFUTED').length} refuted`)

phase('Probe')
const guardMap = b44.guard_map || []
let probe = null
if (guardMap.length > 0) {
  probe = await agent(
    `Independent mutation prober (L13). Work from ${REPO}; PATH="$HOME/.cargo/bin:$PATH". df pre-check (abort under 15 GB free). ONE disposable worktree (git worktree add /tmp/r2-probe HEAD, own CARGO_TARGET_DIR=/tmp/r2-probe-target, targeted builds). Execute every guard-map entry (mutation -> named test -> kill/survive), revert clean between probes. Add 2-3 composition probes of your own for the #44 seam (e.g. drop the group's fsync entirely; fsync only the first event of a group; reorder append vs fsync) — the torn-group recovery claim must die when its guard is broken. Remove worktree + target after; verify main tree untouched. GUARD MAP:
${JSON.stringify(guardMap, null, 2)}`,
    { label: 'prober', phase: 'Probe', model: 'opus', effort: 'high', schema: PROBE_SCHEMA })
  log(`Probe: ${probe ? probe.probes_executed + ' executed, ' + (probe.survivors || []).length + ' survivors' : 'FAILED'}`)
}

phase('Fix')
const survivors = probe && probe.survivors ? probe.survivors : []
let fix = null
if (confirmed.length > 0 || plausible.length > 0 || survivors.length > 0) {
  fix = await agent(`${G}
Round-2 fixer. Close every confirmed finding; investigate each plausible (fix or refute with evidence); strengthen every probe survivor's test until its mutation kills it (re-execute in a disposable worktree with its own target dir, removed after). Nothing silently dropped — declined items get reasons in notes.
CONFIRMED: ${JSON.stringify(confirmed.map(v => ({ ...v, finding: allFindings.find(f => f.id === v.id) })), null, 2)}
PLAUSIBLE: ${JSON.stringify(plausible.map(v => ({ ...v, finding: allFindings.find(f => f.id === v.id) })), null, 2)}
SURVIVORS: ${JSON.stringify(survivors, null, 2)}
Separable commits, gates green before returning.`,
    { label: 'fixer', phase: 'Fix', model: 'sonnet', effort: 'high', schema: { type: 'object', required: ['commits', 'gates_green', 'notes'], properties: { commits: { type: 'array', items: { type: 'string' } }, gates_green: { type: 'boolean' }, notes: { type: 'string' } } } })
  log(`Fix: ${fix ? fix.commits.length + ' commits, gates=' + fix.gates_green : 'none/FAILED'}`)
}

return {
  b44: { commits: b44.commits, perf: b44.perf_evidence, decisions: b44.design_decisions },
  panel: { findings: allFindings.length, confirmed: confirmed.length, plausible: plausible.length },
  probe: probe ? { executed: probe.probes_executed, survivors: survivors.length } : null,
  fix: fix ? { commits: fix.commits, gates: fix.gates_green } : null,
}
