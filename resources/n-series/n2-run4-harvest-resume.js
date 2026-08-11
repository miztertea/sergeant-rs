export const meta = {
  name: 'n2-run4-harvest-resume',
  description: 'Run 4: resume repo-to-icm harvest at P7 per the committed partition ledger, drive retries to all-done, re-run 30..90 downstream, collect evidence',
  phases: [
    { title: 'Setup', detail: 'scratch subject + daemon (harvest worktree binary) + submit + hold stage 0' },
    { title: 'Pre-harvest', detail: '00-contract, 10-inventory actors' },
    { title: 'Harvest-loop', detail: 'seed run-3 ledger, fresh actor per retry until every partition done' },
    { title: 'Downstream', detail: '30-normalize .. 90-reconcile actors over the combined corpus' },
    { title: 'Collect', detail: 'teardown, stats, evidence into docs/gauntlet/runs/n2-run4/ on cerberus/harvest' },
  ],
}
const REPO = '/home/miztertea/sergeant-harvest'          // dedicated worktree, branch cerberus/harvest
const MAIN = '/home/miztertea/sergeant-rs'               // read-only source of run-3 evidence; NEVER write here
const SCRATCH = '/tmp/claude-1001/-home-miztertea-sergeant-rs/6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/n2-run4'
const NOTE = REPO + '/docs/gauntlet/notes/n2-fake-backend-semantics.md'
const SEED = MAIN + '/docs/gauntlet/runs/n2-run3/.sergeant/workflows/repo-to-icm/20-harvest/output'

phase('Setup')
const setup = await agent(`You are setting up sergeant-rs's N2 run 4 (harvest resume) on host Cerberus. Read ${NOTE} (measured hold/advance mechanics), ${REPO}/docs/gauntlet/contracts/N2.md §Outcome 2, and ${MAIN}/docs/gauntlet/runs/n2-run3/run-manifest.md's setup section (run 4 mirrors run 3's setup exactly except where stated).
Do exactly:
1. mkdir -p ${SCRATCH}; create the SUBJECT repo at ${SCRATCH}/subject exactly as run 3 did: (a) full CONTENT of ${REPO}/reference/sergeant-upstream under reference/sergeant-upstream/, (b) ${REPO}/reference/UPSTREAM.md as reference/UPSTREAM.md, (c) ${REPO}/.sergeant/ and ${REPO}/AGENTS.md. git add+commit (identity: sgt-n2 <n2@localhost>). Record HEAD SHA.
2. Start the daemon per the note's method with SGT_FAKE_SCRIPT holding every stage open on needs_input — use 80 needs_input/complete pairs (160 steps): run 4's harvest loop retries the stage once per fresh actor and 15 partitions remain; headroom is deliberate. Data dir ${SCRATCH}/data. Binary: ${REPO}/target/debug/sgt with CARGO-free env (the binary is prebuilt; do NOT run cargo). All sgt commands need nothing from cargo.
3. From inside ${SCRATCH}/subject: submit with the SAME intent wording run 3 used (read it from ${MAIN}/docs/gauntlet/runs/n2-run3/run-manifest.md verbatim — full 82-file scope, NOT a partial scope) via sgt run ... --workflow repo-to-icm --json. Confirm the work holds needs_input on 00-contract and workflow.bound pinned 10 stages.
4. Report: work id, worktree path (sgt work show --json), data dir, subject SHA, and the exact respond command template (WORK_ID and TEXT placeholders) plus the exact retry command template later agents/driver must use.`,
  { label: 'setup', phase: 'Setup', model: 'sonnet', schema: {
    type: 'object', required: ['work_id', 'worktree', 'data_dir', 'subject_sha', 'respond_cmd', 'retry_cmd'],
    properties: { work_id: { type: 'string' }, worktree: { type: 'string' }, data_dir: { type: 'string' }, subject_sha: { type: 'string' }, respond_cmd: { type: 'string' }, retry_cmd: { type: 'string' } },
  } })
if (!setup) throw new Error('setup failed')
log(`work ${setup.work_id} held at 00-contract; worktree ${setup.worktree}`)

const ACTOR_RULES = (st) => `You are the actor for exactly one stage of a running sergeant-rs Work. Working directory: ${setup.worktree} (cd there first; all work happens inside it). Open .sergeant/workflows/repo-to-icm/${st}/CONTEXT.md and execute it faithfully: resolve its Inputs table, follow its method, produce its declared output/ artifacts under .sergeant/workflows/repo-to-icm/${st}/output/, respect _config/run-discipline.md. Stage outputs (git add). You know nothing about other stages beyond your Inputs. Do not modify the daemon, ${REPO}, ${MAIN}, or anything outside the worktree.`

phase('Pre-harvest')
for (const st of ['00-contract', '10-inventory']) {
  const r = await agent(`${ACTOR_RULES(st)}
When your stage's contract is satisfied, advance: ${setup.respond_cmd.replace('WORK_ID', setup.work_id).replace('TEXT', `"stage ${st} complete"`)} — then verify via --json that the Work moved past your stage. Fail-closed path: write the AMBIGUOUS marker per run-discipline.md and advance with "stage ${st} complete (with recorded ambiguities)".
Return artifacts, counts, ambiguities, verified work state.`,
    { label: `actor:${st}`, phase: 'Pre-harvest', model: 'sonnet', schema: {
      type: 'object', required: ['artifacts', 'counts', 'ambiguities', 'work_state_after'],
      properties: { artifacts: { type: 'array', items: { type: 'string' } }, counts: { type: 'string' }, ambiguities: { type: 'array', items: { type: 'string' } }, work_state_after: { type: 'string' } },
    } })
  log(`${st}: ${r ? r.counts.slice(0, 100) : 'AGENT FAILED'} -> ${r ? r.work_state_after : '?'}`)
  if (!r) throw new Error(`${st} actor failed`)
}

phase('Harvest-loop')
const HARVEST_SCHEMA = {
  type: 'object', required: ['seeded', 'partitions_done_this_turn', 'ledger_all_done', 'advanced', 'unit_count_now', 'notes'],
  properties: { seeded: { type: 'boolean' }, partitions_done_this_turn: { type: 'array', items: { type: 'string' } }, ledger_all_done: { type: 'boolean' }, advanced: { type: 'boolean', description: 'true only if you sent the respond that completed the stage' }, unit_count_now: { type: 'number' }, notes: { type: 'string' } },
}
let allDone = false
let firstAttempt = true
let zeroProgressStreak = 0
for (let attempt = 1; attempt <= 20 && !allDone; attempt++) {
  const h = await agent(`${ACTOR_RULES('20-harvest')}
This is harvest attempt ${attempt} of a RESUMED run. The stage's references/partition-checkpoint-protocol.md is your law — follow its "On stage entry" rules exactly.
ORCHESTRATOR RULING (2026-08-11, recorded per L9, binding on every attempt): the census delta found at seeding — exactly one file, .agents/skills/diagnosing-bugs/scripts/hitl-loop.template.sh, decompose in run 4's inventory vs helper-evidence in run 3's — is ruled in run 3's favor because run 3 matches the FROZEN reference corpus (reference-corpus/source-inventory.md line 64: helper-evidence, ladder §6.5; its behavior is BU-P2-024, representation "helper", N1 adjudication A11). The file is NOT harvested under this run's scheme. If the ledger's census-mismatch subsection does not yet record this ruling, append it there (cite this ruling verbatim, note run 4's own inventory disposition stands recorded as a generator-vs-reference disposition disagreement for the comparison record) and then proceed — a census delta consisting ONLY of that one ruled file is resolved, not blocking. Any OTHER file-level delta remains fail-closed.
If output/ already contains the seeded run-3 artifacts and the ledger's scheme-provenance section (a prior attempt stopped at the census gate), do NOT re-seed and do NOT treat the non-empty output/ as a violation — verify the seed is intact (behavior-units.ndjson ends at BU-0312, ledger P1–P6 done), apply the ruling above, and continue harvesting.
${firstAttempt ? `SEEDING (first attempt only, run-4 specific, every step recorded in your notes). Run 3 and this run partitioned the same file census differently (measured: 21 P-partitions vs 19 A–S partitions — partitioning is actor judgment, not deterministic), so the resume follows RUN 3's scheme, reconciled at the FILE level: (1) verify output/ holds no prior run-4 artifacts (the scaffold README.md alone is fine) and reference/UPSTREAM.md pins f430cfd4f90174a98adbd7abebbece6303817929; (2) copy run 3's committed harvest outputs from ${SEED}/ (README.md, behavior-units.ndjson, consequence-class-sweep.md, partition-ledger.md) into output/, and ALSO copy run 3's inventory — ${MAIN}/docs/gauntlet/runs/n2-run3/.sergeant/workflows/repo-to-icm/10-inventory/output/inventory.md — to output/run3-inventory.md (scheme provenance; later attempts use it without leaving the worktree); (3) FILE CENSUS CHECK: the union of decompose files across run3-inventory.md's 21 partitions must equal this run's ../10-inventory/output/inventory.md decompose-file census exactly — if any file is in one set and not the other, STOP before harvesting, list the exact differing files in notes (that is the remaining fail-closed condition); (4) append a "## Scheme provenance (run 4)" section to output/partition-ledger.md stating: this resumed run harvests by run 3's partition scheme per output/run3-inventory.md's file lists; this run's own inventory.md (A–S) stands as its stage's record but is not the harvest bookkeeping; seeded P1–P6 units BU-0001–BU-0312 originate from run 3 (source: committed evidence at ${SEED}). Then harvest pending partitions P7 onward using run3-inventory.md's per-partition file lists.` : 'The ledger already exists in output/ from prior attempts — the protocol\'s retry rules apply: trust done rows, work the next pending partitions in order per the ledger\'s "Scheme provenance (run 4)" section (file lists in output/run3-inventory.md).'}
Work pending partitions IN ORDER per the protocol: read every file, extract units (append to behavior-units.ndjson per partition, ids continuing the existing BU sequence), sweep per consequence-class-checklist.md (append rows), mark done with unit ranges. Fit as many partitions as you can do HONESTLY in this turn — protocol rule: stop at a partition boundary, never shortcut a sweep to squeeze one more in.
THEN: if EVERY ledger row is done, advance the stage: ${setup.respond_cmd.replace('WORK_ID', setup.work_id).replace('TEXT', '"stage 20-harvest complete"')} and set advanced=true. If pending rows remain, do NOT respond, do NOT retry the work yourself — just end, reporting honestly (the driver issues the retry).`,
    { label: `harvest:attempt${attempt}`, phase: 'Harvest-loop', model: 'sonnet', schema: HARVEST_SCHEMA })
  if (!h) throw new Error(`harvest attempt ${attempt} agent failed`)
  firstAttempt = false
  allDone = h.ledger_all_done
  log(`harvest ${attempt}: +[${h.partitions_done_this_turn.join(', ')}] units=${h.unit_count_now} allDone=${allDone}`)
  zeroProgressStreak = (h.partitions_done_this_turn.length === 0 && !h.ledger_all_done) ? zeroProgressStreak + 1 : 0
  if (zeroProgressStreak >= 2) throw new Error('two consecutive harvest attempts made no progress — orchestrator attention required: ' + h.notes.slice(0, 800))
  if (!allDone) {
    const retried = await agent(`Issue exactly one stage retry for the running sergeant-rs work and verify it re-held. Run: ${setup.retry_cmd.replace('WORK_ID', setup.work_id)} — then sgt work show --json (same binary/data-dir) and confirm stage 20-harvest is active/held on needs_input again with a fresh execution. Report the state you saw. Touch nothing else.`,
      { label: `retry:${attempt}`, phase: 'Harvest-loop', model: 'sonnet', effort: 'low', schema: { type: 'object', required: ['reheld', 'state'], properties: { reheld: { type: 'boolean' }, state: { type: 'string' } } } })
    if (!retried || !retried.reheld) throw new Error(`retry after attempt ${attempt} did not re-hold — driver stops for orchestrator`)
  }
}
if (!allDone) throw new Error('20 attempts exhausted with pending partitions — orchestrator attention')

phase('Downstream')
const reports = []
for (const st of ['30-normalize', '40-classify', '50-synthesize', '60-draft', '70-lint', '80-adversarial-review', '90-reconcile']) {
  const r = await agent(`${ACTOR_RULES(st)}
The corpus this run is larger than any prior run (all 21 partitions harvested — run 3 stopped at 6). Your CONTEXT.md's method is unchanged; take the volume seriously and do not truncate silently — if you must bound something, bound it the way your CONTEXT.md/run-discipline.md licenses and record it.
When satisfied, advance: ${setup.respond_cmd.replace('WORK_ID', setup.work_id).replace('TEXT', `"stage ${st} complete"`)} — verify the Work moved on (--json). Fail-closed: AMBIGUOUS marker + advance with "(with recorded ambiguities)".
Return artifacts, counts, ambiguities, verified work state.`,
    { label: `actor:${st}`, phase: 'Downstream', model: 'sonnet', schema: {
      type: 'object', required: ['artifacts', 'counts', 'ambiguities', 'work_state_after'],
      properties: { artifacts: { type: 'array', items: { type: 'string' } }, counts: { type: 'string' }, ambiguities: { type: 'array', items: { type: 'string' } }, work_state_after: { type: 'string' } },
    } })
  reports.push({ stage: st, r })
  log(`${st}: ${r ? r.counts.slice(0, 100) : 'AGENT FAILED'} -> ${r ? r.work_state_after : '?'}`)
  if (!r) break
}

phase('Collect')
const collect = await agent(`Collect sergeant-rs N2 run-4 evidence. Data dir: ${setup.data_dir}; worktree: ${setup.worktree}; work id: ${setup.work_id}; binary ${REPO}/target/debug/sgt.
1. Final Work state via --json (expect completed; report what is true). 2. Trajectory stats from the journal: total events, per-stage timings, retry count on 20-harvest (count execution records), journal bytes. 3. Copy deliverables into ${REPO}/docs/gauntlet/runs/n2-run4/ (stage outputs preserving structure, generated draft packages, and a run-manifest.md you write: subject SHA ${setup.subject_sha}, work id, the SEEDING provenance — run 3's P1–P6 outputs copied in at harvest attempt 1, source path ${SEED} — stage timings, harvest attempt count, artifact inventory, ambiguity roll-up, final unit/package counts vs run 3's 312 units/18 packages). 4. git add + commit everything under docs/gauntlet/runs/n2-run4/ in ${REPO} on branch cerberus/harvest, message "N2 run 4: harvest resumed P7–P21 per the committed ledger; full-corpus downstream re-run" + "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>". Do NOT push. 5. Stop the daemon gracefully; pgrep -f "debug/sgt [-]-data-dir" must be empty — report. Do NOT delete ${SCRATCH}.`,
  { label: 'collect', phase: 'Collect', model: 'sonnet', schema: {
    type: 'object', required: ['final_state', 'events_total', 'harvest_attempts', 'unit_total', 'packages_total', 'leak_check_clean', 'committed', 'notes'],
    properties: { final_state: { type: 'string' }, events_total: { type: 'number' }, harvest_attempts: { type: 'number' }, unit_total: { type: 'number' }, packages_total: { type: 'number' }, leak_check_clean: { type: 'boolean' }, committed: { type: 'boolean' }, notes: { type: 'string' } },
  } })
return { setup: { work_id: setup.work_id, subject_sha: setup.subject_sha }, downstream: reports.map(s => ({ stage: s.stage, counts: s.r ? s.r.counts : null })), collect }
