export const meta = {
  name: 'n2-measurement-run',
  description: 'Run repo-to-icm through the real sgt daemon against sergeant-upstream: setup, 10 held stages each worked by a fresh blind actor, teardown+collect',
  phases: [
    { title: 'Setup', detail: 'scratch subject repo + daemon + submit + hold stage 0' },
    { title: 'Stages', detail: '10 sequential fresh actors, one per held stage' },
    { title: 'Collect', detail: 'teardown, trajectory stats, outputs to docs/gauntlet/runs/' },
  ],
}
const REPO = '/home/user/sergeant-rs'
const SCRATCH = '/tmp/claude-0/-home-user-sergeant-rs/fff58c4e-2990-5de3-a53f-f5e2c669c45f/scratchpad/n2-run2'
const NOTE = REPO + '/docs/gauntlet/notes/n2-fake-backend-semantics.md'

phase('Setup')
const setup = await agent(`You are setting up sergeant-rs's N2 measurement run. Read ${NOTE} (the measured hold/advance mechanics) and ${REPO}/docs/gauntlet/contracts/N2.md §Outcome 2.
Do exactly:
1. mkdir -p ${SCRATCH}; create the SUBJECT repo at ${SCRATCH}/subject: a fresh git repo containing (a) the full CONTENT of ${REPO}/reference/sergeant-upstream copied to reference/sergeant-upstream/ (preserving that layout), (b) ${REPO}/reference/UPSTREAM.md copied to reference/UPSTREAM.md (the provenance document pinning the subtree — run 1 failed closed because it was missing), (c) ${REPO}/.sergeant/ (the repo-to-icm workflow + index.md) and ${REPO}/AGENTS.md. git add+commit all of it (identity: sgt-n2 <n2@localhost>). Record HEAD SHA.
2. Start the daemon per the note's method: SGT_FAKE_SCRIPT holding every stage open on needs_input (10 needs_input/complete pairs = 20 steps), data dir ${SCRATCH}/data, using ${REPO}/target/debug/sgt. From inside ${SCRATCH}/subject run: sgt run "Decompose the repository subtree reference/sergeant-upstream — pinned per reference/UPSTREAM.md at upstream SHA f430cfd4f90174a98adbd7abebbece6303817929 — into draft ICM workflows per .sergeant/workflows/repo-to-icm. Scope: the subtree only; exclude reference/UPSTREAM.md itself, .sergeant/, and AGENTS.md." --workflow repo-to-icm (check sgt run --help for exact flag names; use --json). Confirm work reaches needs_input on stage 00-contract and workflow.bound pinned 10 stages.
3. Report: work id, worktree path (from sgt work show --json), daemon data dir, subject HEAD SHA, and the exact sgt binary+env invocation later agents must use to respond. LEAVE THE DAEMON RUNNING.`,
  { label: 'setup', phase: 'Setup', model: 'sonnet', schema: {
    type: 'object', required: ['work_id', 'worktree', 'data_dir', 'subject_sha', 'respond_cmd'],
    properties: { work_id: { type: 'string' }, worktree: { type: 'string' }, data_dir: { type: 'string' }, subject_sha: { type: 'string' }, respond_cmd: { type: 'string', description: 'exact command template to send input, with WORK_ID and TEXT placeholders' } },
  } })
if (!setup) throw new Error('setup failed')
log(`work ${setup.work_id} held at stage 00; worktree ${setup.worktree}`)

phase('Stages')
const STAGES = ['00-contract', '10-inventory', '20-harvest', '30-normalize', '40-classify', '50-synthesize', '60-draft', '70-lint', '80-adversarial-review', '90-reconcile']
const stageReports = []
for (const st of STAGES) {
  const r = await agent(`You are the actor for exactly one stage of a running sergeant-rs Work. Your working directory: ${setup.worktree} (a git worktree; cd there first; all your work happens inside it).
Open .sergeant/workflows/repo-to-icm/${st}/CONTEXT.md and execute it faithfully: resolve its Inputs table, follow its method, produce its declared output/ artifacts (write per-run artifacts under .sergeant/workflows/repo-to-icm/${st}/output/ in THIS worktree), respect _config/run-discipline.md (it binds you). Stage your outputs as the context/finalize discipline requires (git add).
You know nothing about other stages beyond what your Inputs provide. Do not read any file the discipline forbids. Do not modify the daemon, ${REPO} itself, or anything outside the worktree.
When (and only when) your stage's contract is satisfied, advance the Work by sending input: ${setup.respond_cmd.replace('WORK_ID', setup.work_id).replace('TEXT', `"stage ${st} complete"`)} — then verify via the same CLI (--json) that the Work moved past your stage (next stage held on needs_input, or work completed if you are 90-reconcile). If your stage's fail-closed path triggers, write the AMBIGUOUS marker per run-discipline.md, still produce what you validly can, and advance with input text "stage ${st} complete (with recorded ambiguities)".
Return: artifacts produced (paths+sizes), key counts (units/files/findings as applicable to your stage), ambiguities recorded, and the Work state you verified after advancing.`,
    { label: `actor:${st}`, phase: 'Stages', model: 'sonnet', schema: {
      type: 'object', required: ['artifacts', 'counts', 'ambiguities', 'work_state_after'],
      properties: { artifacts: { type: 'array', items: { type: 'string' } }, counts: { type: 'string' }, ambiguities: { type: 'array', items: { type: 'string' } }, work_state_after: { type: 'string' } },
    } })
  stageReports.push({ stage: st, r })
  log(`${st}: ${r ? r.counts.slice(0, 100) : 'AGENT FAILED'} -> ${r ? r.work_state_after : '?'}`)
  if (!r) break
}

phase('Collect')
const collect = await agent(`Collect sergeant-rs N2 measurement-run evidence. The run's daemon data dir: ${setup.data_dir}; worktree: ${setup.worktree}; work id: ${setup.work_id}; sgt binary ${REPO}/target/debug/sgt.
1. Verify final Work state via CLI --json (expect completed; report whatever is true). Capture sgt work show + the work's event list.
2. Trajectory stats from the journal (${setup.data_dir}/journal or data/journal — find it): total events, events for this work, per-stage timings from stage.entered/stage.completed pairs, journal bytes.
3. Copy the run's deliverables OUT of the worktree into ${REPO}/docs/gauntlet/runs/n2-run2/: every .sergeant/workflows/repo-to-icm/*/output/ artifact (preserving stage structure), the generated draft packages from the worktree's .sergeant/drafts/workflows/ (if 60-draft placed them there), plus a run-manifest.md you write: subject SHA ${setup.subject_sha}, work id, final state, stage timings table, artifact inventory, ambiguity roll-up (read each stage's outputs for AMBIGUOUS markers).
4. Stop the daemon gracefully (sgt daemon stop or SIGTERM per docs), then: pgrep -af "debug/sgt --data-dir" | grep -v "bash -c" MUST be empty — report it. Do NOT delete ${SCRATCH} (evidence).
Return the manifest highlights.`,
  { label: 'collect', phase: 'Collect', model: 'sonnet', schema: {
    type: 'object', required: ['final_state', 'events_total', 'artifacts_copied', 'leak_check_clean', 'notes'],
    properties: { final_state: { type: 'string' }, events_total: { type: 'number' }, artifacts_copied: { type: 'number' }, leak_check_clean: { type: 'boolean' }, notes: { type: 'string' } },
  } })
return { setup, stageReports: stageReports.map(s => ({ stage: s.stage, counts: s.r ? s.r.counts : null, state: s.r ? s.r.work_state_after : null, ambiguities: s.r ? s.r.ambiguities : null })), collect }
