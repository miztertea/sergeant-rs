export const meta = {
  name: 'dogfood-gauntlet',
  description: 'Product-fitness dogfood: three promoted workflows run end-to-end on real tasks with the real backend, graded for human value ($25 owner-authorized cap, 2026-08-11)',
  phases: [
    { title: 'Run-diagnose', detail: 'diagnose-bug (6 stages) on real issue #45, cap $12' },
    { title: 'Run-research', detail: 'research (1 stage, ladder suspect) on the Rule C compression question, cap $4' },
    { title: 'Run-grilling', detail: 'grilling (2 stages) on the R-H0-2 decision — expected finding: ask pathway cannot park on this host, cap $4' },
    { title: 'Grade', detail: 'fresh product-fitness critic over all three trajectories (all-sonnet run per owner direction — this is an end-to-end pass, not a judgment showcase)' },
  ],
}

const REPO = '/home/miztertea/sergeant-rs'
const SGT = REPO + '/target/debug/sgt'
const SCRATCH = '/tmp/claude-1001/-home-miztertea-sergeant-rs/6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/dogfood'

const RUN_SCHEMA = {
  type: 'object',
  required: ['work_id', 'final_state', 'stages_completed', 'stages_total', 'cost_usd', 'canceled_by_guard', 'human_deliverable', 'stage_timeline', 'friction_log', 'notes'],
  properties: {
    work_id: { type: 'string' }, final_state: { type: 'string' }, stages_completed: { type: 'number' }, stages_total: { type: 'number' },
    cost_usd: { type: 'number' }, canceled_by_guard: { type: 'boolean' },
    human_deliverable: { type: 'string', description: 'what a human actually got, with the artifact path — or "nothing", honestly' },
    stage_timeline: { type: 'string' },
    friction_log: { type: 'array', items: { type: 'string' }, description: 'every point where a human operator would have been confused, blocked, or better off without sergeant — the product-fitness raw data' },
    notes: { type: 'string' },
  },
}

const COMMON = (name, cap, subjectSetup, intent, expect) => `You are the operator for ONE dogfood run of the promoted sergeant workflow "${name}" — you act as a first-time HUMAN USER would, and your friction_log is the product: record every confusion, missing doc, surprising state, and moment you needed knowledge a real user would not have. Rules: real claude backend, budget cap $${cap} for THIS run (L16: usage lands per turn — check cumulative cost from the journal's usage.updated events between turns; if over cap, sgt cancel once, record it); never edit ${REPO}'s tracked files; daemon on its own data dir under ${SCRATCH}/${name}/data with binary ${SGT}; profile permission_mode=bypassPermissions (copy the mechanism from docs/gauntlet/runs/runB2/run-manifest.md's setup section); stop the daemon at the end and verify with pgrep -f "${name}/data" style bracketed patterns; do NOT delete ${SCRATCH}.
SETUP: ${subjectSetup}
SUBMIT (from inside the subject): ${SGT} --data-dir ${SCRATCH}/${name}/data run "${intent}" --workflow ${name} --backend claude --json
Then OBSERVE as a user: poll read-only (sgt work show --json, ~60-90s cadence) while the daemon's settle driver advances stages on its own. Respond via sgt respond ONLY if the work genuinely parks in needs_input. ${expect}
Timebox 50 minutes wall; at the timebox, cancel and record where it stood. Copy the work worktree's output artifacts into ${SCRATCH}/${name}/deliverables/ before daemon stop. Report per the schema — friction_log is the most valuable field; do not sand it down.`

phase('Run-diagnose')
const r1 = await agent(COMMON(
  'diagnose-bug', 12,
  `mkdir -p ${SCRATCH}/diagnose-bug; git clone --quiet ${REPO} ${SCRATCH}/diagnose-bug/subject (a full local clone at main; the subject IS sergeant-rs — true dogfood). cd into it. Fetch the target evidence: gh issue view 45 --json title,body -q '.title + \"\\n\\n\" + .body' > ISSUE-45.md and git add+commit it (identity dogfood <dogfood@localhost>).`,
  `Diagnose issue #45 (see ISSUE-45.md at repo root): the m6 dropped-daemon composition test flake under heavy load — daemon dead with runtime.json still on disk. Produce the diagnosis per this workflow's stages; do not implement a fix.`,
  `Expectation: 6 stages should cascade autonomously post-BS2; the deliverable is a diagnosis artifact a maintainer could act on. Judge it as a maintainer: is the diagnosis right, grounded, actionable?`
), { label: 'dogfood:diagnose-bug', phase: 'Run-diagnose', model: 'sonnet', effort: 'high', schema: RUN_SCHEMA })
log(`diagnose-bug: ${r1 ? r1.stages_completed + '/' + r1.stages_total + ' stages, $' + r1.cost_usd + ', deliverable: ' + r1.human_deliverable.slice(0, 80) : 'FAILED'}`)

phase('Run-research')
const r2 = await agent(COMMON(
  'research', 4,
  `mkdir -p ${SCRATCH}/research; git clone --quiet ${REPO} ${SCRATCH}/research/subject; cd into it.`,
  `Research journal cold-segment compression options for the retention ruling's amended Rule C ladder (docs/gauntlet/notes/retention-design-ruling-draft-2026-08-11.md): zstd vs alternatives for append-only NDJSON segments that must remain replayable, Rust crate landscape, decompress-on-rebuild cost estimates against the measured 54.6k events/s rebuild rate. Produce the research artifact this workflow declares.`,
  `Expectation: single-stage workflow — this is one of the eight §6-ladder suspects. The grading question it exists to answer: did wrapping this in a sergeant Work add ANYTHING over just asking an agent directly? Record that comparison explicitly in your friction_log.`
), { label: 'dogfood:research', phase: 'Run-research', model: 'sonnet', effort: 'high', schema: RUN_SCHEMA })
log(`research: ${r2 ? r2.stages_completed + '/' + r2.stages_total + ' stages, $' + r2.cost_usd : 'FAILED'}`)

phase('Run-grilling')
const r3 = await agent(COMMON(
  'grilling', 4,
  `mkdir -p ${SCRATCH}/grilling; git clone --quiet ${REPO} ${SCRATCH}/grilling/subject; cd into it. The decision under grill: copy docs/gauntlet/notes/h0-adjudication-packet-draft-2026-08-11.md to DECISION.md at the subject root, git add+commit.`,
  `Grill me about ruling R-H0-2 in DECISION.md (the sequencing decision: N4-first vs contract-v2-first vs B-lite). One question at a time per this workflow's discipline.`,
  `CRITICAL EXPECTATION (measured, docs/gauntlet/notes/cerberus-ask-grammar-remeasurement-2026-08-11.md): this host's CLI carries no post_turn_summary, so the actor CANNOT park the stage in needs_input to await an answer — the predicted failure shape is the turn completing with its first question as the completion summary, GP-2's original defect resurfacing as a product limitation. Your job is to measure exactly what happens: does the stage park (respond and continue the interview if it genuinely does), or complete without waiting (record the journal evidence)? Either way the run answers "can a human use interactive workflows on this host today" — that verdict IS the deliverable.`
), { label: 'dogfood:grilling', phase: 'Run-grilling', model: 'sonnet', effort: 'high', schema: RUN_SCHEMA })
log(`grilling: ${r3 ? r3.final_state + ', parked=' + (r3.notes.includes('parked') ? 'see notes' : 'see notes') + ', $' + r3.cost_usd : 'FAILED'}`)

phase('Grade')
const runs = [{ name: 'diagnose-bug', r: r1 }, { name: 'research', r: r2 }, { name: 'grilling', r: r3 }].filter(x => x.r)
const grade = await agent(`You are the product-fitness critic for sergeant-rs's first dogfood gauntlet — fresh eyes, you operated none of these runs. The question you grade is the owner's, verbatim: "we still don't really have anything we can use as a human... I'm worried we spent 10+ mil tokens building a workflow library that didn't understand the assignment."
Read first: ${REPO}/reference/proposal-next-iteration-icm-workflows.md §6.1-6.7 (the publication ladder — what SHOULD be a workflow vs an invariant vs a helper), ${REPO}/docs/icm/convention.md, and each run's full evidence under ${SCRATCH}/<name>/ (journal, deliverables, the operator reports below).
For each run, verdict on three axes with evidence: (1) CHECKPOINT HONESTY — did each stage constitute a real §6.3 durable checkpoint, or was it ceremony around one instruction? (2) HUMAN VALUE — would a competent human with plain claude-in-terminal have gotten the same artifact faster? Grade the actual deliverable quality yourself (read the diagnosis — is it RIGHT about #45? read the research — is it load-bearing for Rule C?). (3) PRODUCT FRICTION — from the friction logs + journals: what blocked, confused, or added ceremony?
Then the library-level verdicts: (a) which of the 8 single-stage packages (research, monitor-fleet, drain-fleet, wiki-digest, wake-and-resume, route-review-findings, reconcile-and-cleanup-fleet, deliver-external-callback) survive the §6 ladder as workflows vs belong as AGENTS.md invariants or helpers — argue each from its package content, not vibes; (b) the evidence-grounded engine-needs list ordered by what these runs actually hit (not what outside-in research predicts); (c) a one-paragraph answer to the owner's worry, honest in both directions.
OPERATOR REPORTS: ${JSON.stringify(runs.map(x => ({ name: x.name, report: x.r })), null, 2)}`,
  { label: 'critic:fitness', phase: 'Grade', model: 'sonnet', effort: 'high', schema: {
    type: 'object',
    required: ['per_run', 'ladder_verdicts', 'engine_needs_ranked', 'owner_answer'],
    properties: {
      per_run: { type: 'array', items: { type: 'object', required: ['name', 'checkpoint_honesty', 'human_value', 'friction'], properties: { name: { type: 'string' }, checkpoint_honesty: { type: 'string' }, human_value: { type: 'string' }, friction: { type: 'string' } } } },
      ladder_verdicts: { type: 'array', items: { type: 'object', required: ['package', 'verdict', 'argument'], properties: { package: { type: 'string' }, verdict: { type: 'string', enum: ['workflow', 'agents-invariant', 'helper', 'needs-owner-call'] }, argument: { type: 'string' } } } },
      engine_needs_ranked: { type: 'array', items: { type: 'string' } },
      owner_answer: { type: 'string' },
    },
  } })
return {
  spend: (r1 ? r1.cost_usd : 0) + (r2 ? r2.cost_usd : 0) + (r3 ? r3.cost_usd : 0),
  runs: runs.map(x => ({ name: x.name, state: x.r.final_state, stages: x.r.stages_completed + '/' + x.r.stages_total, cost: x.r.cost_usd })),
  grade,
}
