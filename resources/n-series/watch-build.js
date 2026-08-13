export const meta = {
  name: 'watch-build',
  description: 'WATCH implementation gauntlet: build sgt watch per the adjudicated contract, then blind critics, adversarial verify, fix',
  phases: [
    { title: 'Build', detail: 'Sonnet builder: watch module, CLI verb, EventStream tightening, W-tests, docs' },
    { title: 'Panel', detail: '2 blind Sonnet critics: contract-fidelity, test-honesty' },
    { title: 'Verify', detail: 'Opus batched refuter, disposable-worktree probes' },
    { title: 'Fix', detail: 'Sonnet fixer over confirmed + plausible' },
  ],
}
const REPO = '/home/miztertea/sergeant-rs'
const BASE = '6780321'
const G = `You are working on sergeant-rs branch cerberus/watch on Cerberus. BINDING, read in order before any edit: ${REPO}/docs/gauntlet/contracts/WATCH.md (the adjudicated contract — every R-WATCH ruling is law), ${REPO}/reference/proposal-sgt-watch-v1.md (governs where the contract is silent; the contract wins on conflict), ${REPO}/docs/DEVELOPMENT.md, ${REPO}/CLAUDE.md, ${REPO}/LESSONS.md (L7 pins, L8 capability=claim, L10 separable commits). PATH="$HOME/.cargo/bin:$PATH" everywhere. SCRATCH DISCIPLINE: write only in repo checkout / /tmp scratch, NEVER ~/; no untracked files left behind. Gates before done: cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test. NO real-Claude spend — all tests use the fake backend / DataDir rigs. Commits end "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>". Do NOT push.`
const BUILD = { type: 'object', required: ['commits', 'tests_added', 'gates_green', 'notes'], properties: { commits: { type: 'array', items: { type: 'string' } }, tests_added: { type: 'number' }, gates_green: { type: 'boolean' }, notes: { type: 'string' } } }

phase('Build')
const build = await agent(`${G}
Implement sgt watch per the contract. The shape (proposal §13, as amended):
1. src/watch.rs — WatchOptions/WatchNotice/WatchReason/WatchTrigger, the watch loop, six-state classification (R-WATCH-1), fingerprint with stage.detail hash (R-WATCH-2 names the exact field and sources), human one-line rendering, JSONL serialization (sergeant.watch/v1, §9). Import boundary per §16.3: ApiClient/Event/EventStream/serde only — no journal/projection/engine/backend/daemon internals.
2. src/cli.rs — Watch { id: Option<String>, --follow }; R-WATCH-3's three detection branches, NO auto-spawn (the contract cites the daemon_stop/doctor precedent lines); global --json means JSONL for this command (WATCH-06).
3. EventStream tightening per R-WATCH-7: distinguish keep-alive/comment vs valid event vs malformed data frame vs transport end; REVISE the existing pinned test decode_frame_coalesces_data_lines_with_a_real_newline in the same commit, cite R-WATCH-7 in the commit body; audit src/tui.rs:1072's consumer for deliberate handling of the new variant.
4. R-WATCH-6's SGT_WATCH_TEST_HOLD seam with the 60s dead-man, exactly as ruled.
5. Terminal-lag honesty per R-WATCH-9: no waiting for the teardown cascade; snapshot forwarded verbatim (all eleven top-level keys).
6. Tests: W1-W8 (§16.2 as amended — W2/W3/W4 via the hold seam where ruled), R-WATCH-1's waiting tests, R-WATCH-2's two-questions test, R-WATCH-3's no-spawn refusal test (process-table check, inverse of m2 t7), R-WATCH-10's signal test + DataDir rigs + §16.1 unit bullets as amended, §16.3 structural import checks. Every fix/feature pin must fail on revert (L7). Repeated-run: run the new suite 5x to shake flakes.
7. Docs: AGENTS.md step-6 amendment per R-WATCH-5 (the BU-tagged journal-not-liveness sentence SURVIVES — fold, don't drop; --follow-for-long-waits guidance; R-WATCH-4's attach-then-reconcile order stated), README day-to-day section (§15.2 plus the R-WATCH-8 abandoned-failure row and no-auto-spawn), skills/sergeant-help/SKILL.md routing (§15.3).
L10: separable commits (module, CLI verb, EventStream revision, tests may bundle per area, docs). Report commits, tests_added, gates_green honestly — if a gate fails, say so, do not paper over.`,
  { label: 'builder', phase: 'Build', model: 'sonnet', effort: 'high', schema: BUILD })
log(`build: ${build ? build.commits.length + ' commits, ' + build.tests_added + ' tests, green=' + build.gates_green : 'FAILED'}`)
if (!build || !build.gates_green) throw new Error('build not green')

phase('Panel')
const FINDINGS = { type: 'object', required: ['findings', 'summary'], properties: { summary: { type: 'string' }, findings: { type: 'array', items: { type: 'object', required: ['id', 'severity', 'claim', 'evidence'], properties: { id: { type: 'string' }, severity: { type: 'string', enum: ['error', 'warning', 'info'] }, claim: { type: 'string' }, evidence: { type: 'string' } } } } } }
const axes = [
  { key: 'contract-fidelity', brief: `Axis CONTRACT-FIDELITY: grade the diff against every R-WATCH ruling in docs/gauntlet/contracts/WATCH.md, one by one — six-state set incl. waiting continuation; fingerprint hashes stage.detail exactly; the three R-WATCH-3 branches with correct fail-closed wording and provably no spawn; R-WATCH-4/5 doc text (BU tags survive in AGENTS.md, attach-then-reconcile stated, abandoned-failure row present); the hold's 60s dead-man; decode_frame's four-way distinction with the old pin revised in the same commit and the TUI consumer handled deliberately; snapshot verbatim (all eleven keys, no reduced parallel schema); terminal-lag not waited on; §16.3 import boundary (watch.rs imports nothing from journal/projection/engine/backend/daemon); R-NS-4 (no new engine semantics anywhere in the diff — check the daemon side is untouched except EventStream client code). Also: does any CLI/help/README text promise something the code doesn't do (L8)?` },
  { key: 'test-honesty', brief: `Axis TEST-HONESTY: do the W-tests drive the real sgt binary through DataDir rigs as live processes (not function calls)? Does W3/W4 actually prove the hold engaged (the .ready handshake asserted — a hold that never engages makes the test vacuous, TH-05's class)? Would each pin fail on revert (L7 — spot-check by reverting 2-3 key fixes in a disposable worktree: the fingerprint detail hash, the no-spawn branch, the decode_frame distinction)? Does the no-spawn test actually check the process table? Does the two-questions test genuinely produce two different questions in one stage attempt through the fake backend? Is the signal test asserting no journal write? Any wall-clock sleeps standing in for synchronization? Run the new suites yourself and re-run 3x for flakes.` },
]
const crits = await parallel(axes.map(a => () => agent(
  `Blind critic, sergeant-rs WATCH build. Read-only in ${REPO} (cargo test allowed, PATH="$HOME/.cargo/bin:$PATH"; NEVER edit the main tree; disposable-worktree probes with own CARGO_TARGET_DIR allowed for revert checks, removed after). Read first: docs/gauntlet/contracts/WATCH.md, reference/proposal-sgt-watch-v1.md, GAUNTLET.md register (L3 — the contract's rulings are ADJUDICATED; re-litigating one requires new evidence it is wrong), LESSONS.md. Diff under review: git log --oneline ${BASE}..HEAD; git diff ${BASE}..HEAD. ${a.brief} Findings carry file:line evidence.`,
  { label: `critic:${a.key}`, phase: 'Panel', model: 'sonnet', effort: 'high', schema: FINDINGS })))
const all = crits.flatMap((c, i) => (c ? c.findings.map(f => ({ ...f, id: `${axes[i].key}:${f.id}` })) : []))
log(`panel: ${all.length} findings (${all.filter(f => f.severity === 'error').length} error)`)

phase('Verify')
let verdicts = [], treeClean = true
if (all.length) {
  const r = await agent(`Batched adversarial refuter, WATCH build. ${REPO}, PATH="$HOME/.cargo/bin:$PATH". CONFIRM/REFUTE/PLAUSIBLE each finding with evidence read or executed this session; mutations ONLY in one disposable worktree (own CARGO_TARGET_DIR), removed after, main tree verified clean at the end; scratch never ~/. Deduplicate: same defect twice → confirm one, refute the other as 'duplicate of <id>'. FINDINGS:\n${JSON.stringify(all, null, 2)}`,
    { label: 'refuter', phase: 'Verify', model: 'opus', effort: 'high', schema: { type: 'object', required: ['verdicts', 'tree_clean'], properties: { tree_clean: { type: 'boolean' }, verdicts: { type: 'array', items: { type: 'object', required: ['id', 'verdict', 'basis'], properties: { id: { type: 'string' }, verdict: { type: 'string', enum: ['CONFIRMED', 'REFUTED', 'PLAUSIBLE'] }, basis: { type: 'string' } } } } } } })
  verdicts = r ? r.verdicts : []
  if (r && r.tree_clean === false) { treeClean = false; log('L5 HAZARD: refuter reports main tree dirty') }
}
const confirmed = verdicts.filter(v => v.verdict === 'CONFIRMED')
const plausible = verdicts.filter(v => v.verdict === 'PLAUSIBLE')
log(`verify: ${confirmed.length} confirmed / ${plausible.length} plausible / ${verdicts.filter(v => v.verdict === 'REFUTED').length} refuted`)

phase('Fix')
let fix = null
if (confirmed.length || plausible.length || !treeClean) {
  fix = await agent(`${G}
You are the WATCH fixer. ${treeClean ? '' : 'FIRST: refuter reported main tree dirty — git status, quarantine probe residue (L5), report before other fixes. '}Close every confirmed finding; investigate plausibles (fix, or refute with evidence — a rejection gets a recorded reason). Every fix commit separable from the build (L10) and carries its pinning test (L7). Gates green before done.
CONFIRMED: ${JSON.stringify(confirmed.map(v => ({ ...v, finding: all.find(f => f.id === v.id) })), null, 2)}
PLAUSIBLE: ${JSON.stringify(plausible.map(v => ({ ...v, finding: all.find(f => f.id === v.id) })), null, 2)}`,
    { label: 'fixer', phase: 'Fix', model: 'sonnet', effort: 'high', schema: BUILD })
  if (!fix || !fix.gates_green) throw new Error('fixer not green')
}
return {
  build: { commits: build.commits, tests: build.tests_added },
  panel: { findings: all.length, confirmed: confirmed.length, plausible: plausible.length, refuted: verdicts.filter(v => v.verdict === 'REFUTED').length },
  fix: fix ? { commits: fix.commits, notes: fix.notes } : 'no fixes needed',
  tree_clean: treeClean,
}