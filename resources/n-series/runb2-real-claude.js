export const meta = {
  name: 'runb2-real-claude',
  description: 'Run B2: bounded real-Claude re-run (#19) on the post-BS2 binary — watch the settle driver cascade live, ask-withdrawal live, budget-guarded',
  phases: [
    { title: 'Setup', detail: 'subject + daemon (real claude backend, bypassPermissions via profile opt-in) + submit' },
    { title: 'Watch', detail: 'observe cascade, enforce $2.50 budget guard, no client cranks' },
    { title: 'Collect', detail: 'evidence -> docs/gauntlet/runs/runB2/ on cerberus/runb' },
  ],
}
const WT = '/home/miztertea/sergeant-runb'
const SGT = WT + '/target/debug/sgt'
const SCRATCH = '/tmp/claude-1001/-home-miztertea-sergeant-rs/6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/runB2'
const MANIFEST = WT + '/docs/gauntlet/runs/runB/run-manifest.md'

phase('Setup')
const setup = await agent(`You are setting up sergeant-rs Run B2 (the #19 re-run) on Cerberus, non-root uid 1001, against the REAL claude backend on the post-BS2 binary ${SGT}. Read first: ${MANIFEST} (the original run's exact recipe + intent wording), ${WT}/docs/environments/cerberus.md (this host's facts: no IS_SANDBOX needed; ask grammar absent — GP-2 cannot fire, the withdrawal path is the watch item), and the #47 permission-mode plumbing (git -C ${WT} show 06fb6e8 --stat, then read src/domain/profile.rs enough to configure a profile).
Do exactly:
1. mkdir -p ${SCRATCH}; build the SUBJECT repo at ${SCRATCH}/subject exactly per the manifest's recipe (sergeant-upstream content + UPSTREAM.md + .sergeant/ + AGENTS.md from ${WT}, committed).
2. Configure the work's profile with permission_mode = "bypassPermissions" — the EXPLICIT opt-in #47 built (find the mechanism: sergeant.toml or profile config; this is itself a live test of #47's plumbing — record exactly what you had to write where). No IS_SANDBOX anywhere (non-root host; measured unnecessary).
3. Start the daemon (data dir ${SCRATCH}/data, binary ${SGT}, REAL claude backend available — verify with ${SGT} --data-dir ${SCRATCH}/data doctor first: claude probe ok, permission_mode surfaced per profile).
4. From ${SCRATCH}/subject submit with the manifest's VERBATIM intent (the two-partition bounded scope) via sgt run ... --workflow repo-to-icm --backend claude --json.
5. Report work id, data dir, worktree path, submit timestamp, and the doctor output's claude + permission rows. LEAVE the daemon and work RUNNING — do NOT poll into it beyond one initial work show.`,
  { label: 'setup', phase: 'Setup', model: 'sonnet', schema: {
    type: 'object', required: ['work_id', 'data_dir', 'worktree', 'profile_mechanism', 'doctor_rows', 'notes'],
    properties: { work_id: { type: 'string' }, data_dir: { type: 'string' }, worktree: { type: 'string' }, profile_mechanism: { type: 'string' }, doctor_rows: { type: 'string' }, notes: { type: 'string' } },
  } })
if (!setup) throw new Error('runB2 setup failed')
log(`runB2 work ${setup.work_id} submitted; profile via: ${setup.profile_mechanism.slice(0, 120)}`)

phase('Watch')
const watch = await agent(`You are the Run B2 watcher. Work id ${setup.work_id}, data dir ${setup.data_dir}, binary ${SGT}. Rules that outrank everything: (a) you NEVER issue a mutating command except the one budget/timebox cancel described below — no respond, no retry; the entire point is watching whether the daemon's own settle driver advances stages with zero client cranks (the #46 defect this binary claims to fix); read-only work show/journal reads only. (b) Every "still running" claim needs process-table evidence (pgrep the claude CLI process; a dead turn and a long turn are indistinguishable from the projection alone).
Loop (bash sleep loops are fine, poll every ~90s, HARD CAP 100 minutes total):
- Track: current stage index/status, cumulative cost (sum every usage.updated event's total_cost_usd in the journal — the authoritative spend record), live claude process count, and the two watch items: (1) any stage.completed WITHOUT a client command between turn end and settle (grep the journal ordering — conversation.turn.ended followed by stage.completed with no command.accepted between: that is the settle driver firing live, the headline evidence); (2) conversation.turn.grammar_unmeasured (the ask-withdrawal firing live on this host's grammar-less CLI).
- BUDGET GUARD: if cumulative cost exceeds $2.50, or wall clock exceeds 100 minutes, issue exactly one sgt cancel (--json, record output) and stop watching. The ~$1 precedent is the owner's pre-authorization; $2.50 is the orchestrator's recorded stretch bound for a run that is actually progressing — anything more waits for the owner.
- If the work reaches a terminal state (completed/failed/blocked/canceled) or stops progressing for 3 consecutive polls with ZERO live claude processes (dead-turn evidence per rule b), stop watching and report exactly what the journal shows — do not diagnose beyond the evidence.
Return the timeline (per-stage: entered, turn ended, settled, gap-seconds, cost), watch-item verdicts with journal line evidence, final state, total cost.`,
  { label: 'watch', phase: 'Watch', model: 'sonnet', effort: 'high', schema: {
    type: 'object', required: ['final_state', 'stages_completed', 'settle_driver_fired_live', 'grammar_withdrawal_fired', 'total_cost_usd', 'canceled_by_guard', 'timeline', 'notes'],
    properties: { final_state: { type: 'string' }, stages_completed: { type: 'number' }, settle_driver_fired_live: { type: 'boolean' }, grammar_withdrawal_fired: { type: 'boolean' }, total_cost_usd: { type: 'number' }, canceled_by_guard: { type: 'boolean' }, timeline: { type: 'string' }, notes: { type: 'string' } },
  } })
if (!watch) throw new Error('runB2 watcher failed — daemon may still be running; collect phase must handle it')
log(`runB2: ${watch.stages_completed} stages, settle-live=${watch.settle_driver_fired_live}, withdrawal=${watch.grammar_withdrawal_fired}, $${watch.total_cost_usd}, final=${watch.final_state}`)

phase('Collect')
const collect = await agent(`Collect Run B2 evidence. Data dir ${setup.data_dir}; work ${setup.work_id}; binary ${SGT}; worktree ${setup.worktree}.
1. Copy into ${WT}/docs/gauntlet/runs/runB2/: the full journal (journal segments as journal_full.ndjson), submit output, any cancel output, every turn's raw stream-json transcript decoded from the blob store (the blob refs are in conversation.turn.ended events), and the work worktree's produced artifacts (preserve stage structure).
2. Write run-manifest.md in that dir, runB's manifest as the template (${MANIFEST}): setup (this host, non-root, NO IS_SANDBOX, profile permission_mode=bypassPermissions via ${JSON.stringify(setup.profile_mechanism).slice(0, 200)}), per-stage timeline, the TWO WATCH VERDICTS with journal-line evidence (settle driver firing live = the #46 fix's first real-world measurement; grammar_unmeasured = the withdrawal path live), usage table per turn (verbatim fields, nothing estimated), total spend, what did and did not close #19 (R-N0-6: #19 closes only with the evidence attached — state plainly whether this run's scope constitutes the soak or a bounded step toward it).
3. git add + commit ONLY docs/gauntlet/runs/runB2/ on branch cerberus/runb in ${WT}, message "Run B2: real-Claude re-run on the post-BS2 settle driver" + Fixes trailer ONLY if the manifest's own #19 verdict says the soak evidence is complete (be conservative; when unsure, no trailer) + "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>". Do NOT push.
4. Stop the daemon gracefully; verify: pgrep -f "runb/target/debug/[s]gt" empty AND no stray claude -p processes from this run. Do NOT delete ${SCRATCH}.
WATCHER REPORT: ${JSON.stringify(watch)}`,
  { label: 'collect', phase: 'Collect', model: 'sonnet', schema: {
    type: 'object', required: ['final_state', 'total_cost_usd', 'committed', 'leak_clean', 'manifest_19_verdict', 'notes'],
    properties: { final_state: { type: 'string' }, total_cost_usd: { type: 'number' }, committed: { type: 'boolean' }, leak_clean: { type: 'boolean' }, manifest_19_verdict: { type: 'string' }, notes: { type: 'string' } },
  } })
return { setup: { work_id: setup.work_id, profile: setup.profile_mechanism }, watch, collect }
