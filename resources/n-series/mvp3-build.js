export const meta = {
  name: 'mvp3-build',
  description: 'MVP-3 CLI per the bucketing + checkpoint friction: init, repo/group verbs, transcript, pointer surfacing, doctor estate, data-dir flip, envelope controls',
  phases: [
    { title: 'Build', detail: 'lane E1: init+verbs+data-dir; lane E2: transcript+pointer+doctor+E5; lane E3: cheap-now (daemon stop, drain, #13) + envelope controls' },
    { title: 'Panel', detail: 'Opus critics: invariants + test-honesty (CLI surfaces are clients — m6 t5 discipline)' },
    { title: 'Verify', detail: 'batched refuter + independent prober' },
    { title: 'Fix', detail: 'fixer over confirmed + survivors' },
  ],
}
// base hardcoded at write time; the args path failed silently once (wf_ea4191d0).
const REPO = '/home/miztertea/sergeant-rs'
const BASE = '3ef5928ab7ad7f733fa0c629c3e10a2b91720245'
const G = `You are building sergeant-rs MVP-3 (CLI) on Cerberus, branch cerberus/mvp-1. Binding, read in order: ${REPO}/docs/gauntlet/notes/mvp-bucketing-2026-08-11.md MVP-3 section, ${REPO}/docs/gauntlet/contracts/MVP-1.md (R-MVP1-1/3/4/5/12 define the estate surfaces your verbs expose; the group-expansion ruling + docs/gauntlet/notes/multirepo-measurement-2026-08-11.md gate the group verbs), ${REPO}/docs/gauntlet/notes/estate-manifest-design-2026-08-11.md (three pens, atomic writes, advisory lock, mkdir-p group semantics), ${REPO}/NORTH-STAR.md (R-NS-4: surfaces add usability never functionality — every verb is API/file plumbing, no new engine semantics), ${REPO}/CLAUDE.md, ${REPO}/LESSONS.md, ${REPO}/docs/environments/cerberus.md (PATH="$HOME/.cargo/bin:$PATH" everywhere). SCRATCH DISCIPLINE: write only inside the repo, the session scratchpad, or /tmp — NEVER ~/; exit check: ls ~ | grep -v -E '^(sergeant-rs|inbox)$' shows no new entries. Gates: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test. m6 t5 pins the ApiViews method set — if a verb needs API surface, extend the API properly and update t5's pin deliberately, never sneak internals. DataDir guard; L7 pins; L10 separable commits; guard_map per test. Commits end "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>". Fixes #NN on fully-closing commits (#13 candidate). Do NOT push. Leak check: bracketed pgrep empty.`
const BUILD = { type: 'object', required: ['commits', 'tests_added', 'gates_green', 'design_decisions', 'guard_map', 'notes'], properties: { commits: { type: 'array', items: { type: 'string' } }, tests_added: { type: 'number' }, gates_green: { type: 'boolean' }, design_decisions: { type: 'array', items: { type: 'string' } }, guard_map: { type: 'array', items: { type: 'object', required: ['test', 'guarded_behavior', 'mutation_that_should_kill_it'], properties: { test: { type: 'string' }, guarded_behavior: { type: 'string' }, mutation_that_should_kill_it: { type: 'string' } } } }, notes: { type: 'string' } } }
const FINDINGS = { type: 'object', required: ['findings', 'summary'], properties: { summary: { type: 'string' }, findings: { type: 'array', items: { type: 'object', required: ['id', 'severity', 'claim', 'evidence'], properties: { id: { type: 'string' }, severity: { type: 'string', enum: ['error', 'warning', 'info'] }, claim: { type: 'string' }, evidence: { type: 'string' } } } } } }

phase('Build')
const lanes = [
  { key: 'E1-estate-cli', brief: `Lane E1 — estate CLI: sgt init (scaffold [estate] sections in sergeant.toml, repos/ dir, .gitignore entries for .sergeant/data + repos/, run doctor estate checks; idempotent re-run); sgt repo add/remove/list (validate at the pen per the manifest design: atomic temp+rename writes, advisory lock for sgt's own read-modify-write, origin clone-or-verify on add); sgt group add/remove/list (members must be declared repos, fail closed with remedy; mkdir-p create semantics) + sgt run --group expansion PER the MVP-1 group-expansion ruling (read it first — if it ruled no-new-engine-surface, --group expands client-side into the existing --repo path); estate-resolved data-dir default (client-side upward discovery in cli.rs mirroring R-MVP1-12's rules, [estate]-keyed, $HOME-bounded, flag/env override precedence unchanged).` },
  { key: 'E2-legibility-cli', brief: `Lane E2 — legibility surfaces: sgt work transcript <id> (decode the work's conversation blobs from the journal refs in order — plain text render, --json for raw; read-only, no daemon mutation); output-pointer surfacing (work show + terminal CLI output state branch + retained paths from the R-MVP1-2 pointer the core now serves; make the happy-path answer to "where did my deliverable land" one command); doctor estate checks with named remedies (manifest parse errors with line/key, missing repos with the origin remedy, undeclared dirs in repos/); E5 discoverability (config errors name file+key+expected everywhere the CLI parses; sgt profile visibility via doctor rows).` },
  { key: 'E3-controls', brief: `Lane E3 — controls + cheap-now: sgt daemon stop (graceful, journaled daemon.stopped, refuses nothing); admission drain scoped exactly (one journaled event pair admission.paused/resumed + submit refusal while paused; daemon stop drains first); the checkpoint friction items — turn-cap/ceiling settable at submission (sgt run --turns N --ceiling-secs S flowing into the R-MVP1-7 envelope config for that work) and envelope visibility (turns_spawned/cap/ceiling in sgt work show + status); #13 perf-harness gaps (s6 resource sampling, kill_delay comment drift, DuckDB size timing — Fixes #13). NOTE: #50's fix already landed via the self-hosting checkpoint.` },
]
const built = []
for (const l of lanes) {
  const r = await agent(`${G}\n${l.brief}\nRead prior lanes' commits (git log ${BASE}..HEAD). Exclusive surfaces: E1 owns cli.rs estate/verb code + init; E2 owns transcript/show/doctor render paths; E3 owns daemon lifecycle + envelope plumbing; shared cli.rs arg-parsing merges rebase-friendly.`,
    { label: `build:${l.key}`, phase: 'Build', model: 'sonnet', effort: 'high', schema: BUILD })
  built.push({ lane: l.key, r })
  log(`${l.key}: ${r ? r.commits.length + ' commits, green=' + r.gates_green : 'FAILED'}`)
  if (!r || !r.gates_green) throw new Error(`${l.key} not green — stop for orchestrator`)
}

phase('Panel')
const axes = [
  { key: 'invariants', brief: `Axis INVARIANTS: R-NS-4 — every new verb adds usability never functionality (no engine semantics smuggled through the CLI); m6 t5's ApiViews pin evolved deliberately or not at all; the manifest pens honor atomic-write/advisory-lock/fail-closed-parse as designed; drain's event pair respects L6; data-dir discovery cannot escape $HOME or cross into a non-estate; envelope submission flags cannot bypass the R-MVP1-7 counting. THE CONTRACT DOCS are in scope (L9/L19).` },
  { key: 'test-honesty', brief: `Axis TEST-HONESTY: do verb tests drive the real CLI binary through DataDir rigs (not function calls); does the transcript test verify actual blob decode against a journaled turn; do init/repo/group tests exercise the atomic-write and lock paths under contention; would each pin fail on revert (L7); are guard-map mutations right. Read-only; prober executes.` },
]
const crits = await parallel(axes.map(a => () => agent(
  `Blind critic, sergeant-rs MVP-3 round 1. Read-only in ${REPO} (cargo test allowed; NEVER edit). Read first: the MVP-3 bucketing section, MVP-1.md's estate rulings, GAUNTLET register (L3), LESSONS. Diff: git diff ${BASE}..HEAD. ${a.brief} Findings carry file:line evidence.`,
  { label: `critic:${a.key}`, phase: 'Panel', model: 'opus', effort: 'high', schema: FINDINGS })))
const all = crits.flatMap((c, i) => (c ? c.findings.map(f => ({ ...f, id: `${axes[i].key}:${f.id}` })) : []))
log(`panel: ${all.length} findings (${all.filter(f => f.severity === 'error').length} error)`)

phase('Verify')
let verdicts = []
let treeCleanFlag = true
if (all.length) {
  const r = await agent(`Batched adversarial refuter, MVP-3. ${REPO}, PATH="$HOME/.cargo/bin:$PATH". CONFIRM/REFUTE/PLAUSIBLE with evidence; mutations ONLY in one disposable worktree (own CARGO_TARGET_DIR), removed after, main tree verified clean. Scratch discipline: never ~/. FINDINGS:\n${JSON.stringify(all, null, 2)}`,
    { label: 'refuter', phase: 'Verify', model: 'opus', effort: 'high', schema: { type: 'object', required: ['verdicts', 'tree_clean'], properties: { tree_clean: { type: 'boolean' }, verdicts: { type: 'array', items: { type: 'object', required: ['id', 'verdict', 'basis'], properties: { id: { type: 'string' }, verdict: { type: 'string', enum: ['CONFIRMED', 'REFUTED', 'PLAUSIBLE'] }, basis: { type: 'string' } } } } } } })
  verdicts = r ? r.verdicts : []
  if (r && r.tree_clean === false) { treeCleanFlag = false; log('HALT-WORTHY: refuter reports main tree dirty (L5) — fixer must verify tree state FIRST') }
}
const guardMap = built.map(b => b.r).flatMap(b => b.guard_map || [])
const probe = guardMap.length ? await agent(`Independent mutation prober (L13), MVP-3. ${REPO}. df pre-check (abort under 15GB). ONE disposable worktree, own CARGO_TARGET_DIR, targeted builds; execute every entry + 3 composition probes on the pen/lock/discovery seams; scratch discipline: never ~/. Worktree+target removed after; main tree clean. GUARD MAP (${guardMap.length}):\n${JSON.stringify(guardMap, null, 2)}`,
  { label: 'prober', phase: 'Verify', model: 'sonnet', effort: 'high', schema: { type: 'object', required: ['probes_executed', 'kills', 'survivors'], properties: { probes_executed: { type: 'integer' }, kills: { type: 'array', items: { type: 'string' } }, survivors: { type: 'array', items: { type: 'object', required: ['test', 'mutation', 'why_it_survived'], properties: { test: { type: 'string' }, mutation: { type: 'string' }, why_it_survived: { type: 'string' } } } } } } }) : null
const confirmed = verdicts.filter(v => v.verdict === 'CONFIRMED')
const plausible = verdicts.filter(v => v.verdict === 'PLAUSIBLE')
const survivors = probe && probe.survivors ? probe.survivors : []
log(`verify: ${confirmed.length}/${plausible.length}/${survivors.length} confirmed/plausible/survivors`)

phase('Fix')
let fix = null
if (confirmed.length || plausible.length || survivors.length || !treeCleanFlag) {
  fix = await agent(`${G}\nMVP-3 fixer. ${treeCleanFlag ? '' : 'FIRST: the refuter reported the main tree dirty — verify git status, quarantine any probe residue per L5, report what you found before fixing anything else. '}Close every confirmed; investigate plausibles; strengthen survivors (re-execute mutations in a disposable worktree). Ledger entry for the pass. Nothing silently dropped.\nCONFIRMED: ${JSON.stringify(confirmed.map(v => ({ ...v, finding: all.find(f => f.id === v.id) })), null, 2)}\nPLAUSIBLE: ${JSON.stringify(plausible.map(v => ({ ...v, finding: all.find(f => f.id === v.id) })), null, 2)}\nSURVIVORS: ${JSON.stringify(survivors, null, 2)}`,
    { label: 'fixer', phase: 'Fix', model: 'sonnet', effort: 'high', schema: { type: 'object', required: ['commits', 'gates_green', 'notes'], properties: { commits: { type: 'array', items: { type: 'string' } }, gates_green: { type: 'boolean' }, notes: { type: 'string' } } } })
}
return { lanes: built.map(b => ({ lane: b.lane, commits: b.r.commits, tests: b.r.tests_added })), findings: { total: all.length, confirmed: confirmed.length }, probe: probe ? { executed: probe.probes_executed, survivors: survivors.length } : null, fix: fix ? { commits: fix.commits, gates: fix.gates_green } : null }
