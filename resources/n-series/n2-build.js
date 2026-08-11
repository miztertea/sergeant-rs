export const meta = {
  name: 'n2-build-repo-to-icm',
  description: 'Measure fake-backend semantics, author .sergeant/workflows/repo-to-icm per N2 contract, validate, blind-critique',
  phases: [
    { title: 'Measure', detail: 'fake-backend hold-open/advance semantics (U1)' },
    { title: 'Build', detail: '3 builders: stage groups + workflow-level files' },
    { title: 'Validate', detail: 'the §9.7 validator, run on its own tree' },
    { title: 'Critique', detail: '2 blind critics: convention compliance + executability' },
  ],
}
const REPO = '/home/user/sergeant-rs'
const G = `You work on sergeant-rs milestone N2. Read (in order): ${REPO}/docs/gauntlet/contracts/N2.md, ${REPO}/docs/icm/convention.md, ${REPO}/docs/icm/record-shapes.md, proposal §9 in ${REPO}/reference/proposal-next-iteration-icm-workflows.md, and GAUNTLET.md's register tables. Rules: never modify reference/ or reference-corpus/; do not commit; write only where your prompt assigns. The workflow you build will be executed by fresh per-stage agents who see ONLY their stage's pinned context + the worktree — write contexts a stranger can execute.`

phase('Measure')
const meas = await agent(`${G}
Measure N2 Unknown U1: how the fake backend holds a stage open and advances (start from scripts/perf/README.md's measured FIFO semantics and src/backend/fake.rs). Using target/debug/sgt with a scratch --data-dir under /tmp/claude-0/.../scratchpad (use dir 'n2-measure'), determine and DEMONSTRATE: (a) how SGT_FAKE_SCRIPT steps map to stage lifecycle (complete/fail/waiting/needs_input), (b) how to hold a stage open while external work happens and then advance it (input-send? respond endpoint?), (c) exact commands for a 10-stage run where every stage is held open and advanced externally. Write findings + a working demo transcript to ${REPO}/docs/gauntlet/notes/n2-fake-backend-semantics.md. Kill every daemon you start (pgrep -af "debug/sgt --data-dir" | grep -v "bash -c" must be empty when you finish). Do not modify src/.`,
  { label: 'measure:fake', phase: 'Measure', model: 'sonnet', schema: { type: 'object', required: ['hold_open_mechanism', 'advance_mechanism', 'demo_worked'], properties: { hold_open_mechanism: { type: 'string' }, advance_mechanism: { type: 'string' }, demo_worked: { type: 'boolean' } } } })
log(`measure: hold=${meas ? meas.hold_open_mechanism.slice(0, 80) : 'FAIL'} demo=${meas ? meas.demo_worked : '?'}`)

phase('Build')
const S = { type: 'object', required: ['files', 'notes'], properties: { files: { type: 'array', items: { type: 'string' } }, notes: { type: 'string' } } }
const built = (await parallel([
  () => agent(`${G}
You own ${REPO}/.sergeant/workflows/repo-to-icm/ workflow-level files: workflow.toml (10 stages per N2/§9.3, 00-contract … 90-reconcile), index.md (front matter per record-shapes §1, status: draft→no: this is an ADMITTED workflow in this repo, status: published, version: 1), CONTEXT.md (L1 orientation: what the workflow does, how stages hand off via output/ artifacts, blindness rule — actors NEVER read reference-corpus/), _config/ (evidence-policy.md: citation+quote+hash discipline per record-shapes §3 incl. the preimage convention; icm-ladder.md: the §6 ladder distilled with the §6.3 reimplementation test and §6.7 template), and scripts/finalize.py (the D9 disposition finalize helper: reads each stage's output/README.md dispositions, keeps promote, removes evidence-class+undeclared at tip) plus scripts/validate-structure.py (the §9.7 validator, adapted from reference-corpus/lint.py's package checks to validate BOTH this workflow's own tree AND a generated draft tree passed as argv[1]).`,
    { label: 'build:workflow-level', phase: 'Build', model: 'sonnet', schema: S }),
  () => agent(`${G}
You own stage dirs 00-contract, 10-inventory, 20-harvest, 30-normalize under ${REPO}/.sergeant/workflows/repo-to-icm/. Each: CONTEXT.md (Inputs table per record-shapes §1a; what must become true; how to do it distilled from proposal §9 — inventory dispositions, §9.4 unit shape with quote+hash discipline via @@-style reference to ../_config/evidence-policy.md; one behavior per unit), references/ where a method needs more than the context should carry, output/README.md declaring artifacts + dispositions (units NDJSON etc. = promote). Stage outputs land under the stage's output/ in the RUN worktree; downstream Inputs cite them.`,
    { label: 'build:stages-00-30', phase: 'Build', model: 'sonnet', schema: S }),
  () => agent(`${G}
You own stage dirs 40-classify, 50-synthesize, 60-draft, 70-lint, 80-adversarial-review, 90-reconcile under ${REPO}/.sergeant/workflows/repo-to-icm/. Same shape rules as the earlier stages. 40: ladder application citing ../_config/icm-ladder.md, §9.5 record incl. rationale+alternatives rules. 60: draft packages under the run worktree's .sergeant/drafts/workflows/ (draft boundary!), layered per convention, provenance per package. 70: run ../scripts/validate-structure.py on the draft tree, repair mechanical defects. 80: fresh-eyes adversarial review (boundary honesty + invention + engine-gap refutation; the reviewer never saw earlier stage conversations — context must say what to challenge). 90: reconcile findings, emit measurement package + grammar-pressure report (§6.7 template for every could-not-express), then run ../scripts/finalize.py.`,
    { label: 'build:stages-40-90', phase: 'Build', model: 'sonnet', schema: S }),
])).filter(Boolean)
log(`built: ${built.flatMap(b => b.files).length} files`)

phase('Validate')
const val = await agent(`${G}
Run ${REPO}/.sergeant/workflows/repo-to-icm/scripts/validate-structure.py on the workflow's own tree; fix mechanical defects (broken Inputs paths, order mismatches, missing dispositions); re-run to exit 0. Also sanity-run scripts/finalize.py --dry-run semantics if it supports it (add a --dry-run flag if missing). Then verify the workflow LOADS in the real engine: use target/debug/sgt with a scratch data dir + a scratch git repo containing a copy of .sergeant/ (per docs/gauntlet/notes/n2-fake-backend-semantics.md), submit a work with this workflow, confirm workflow.bound pins all 10 stages with executor metadata, then cancel and kill the daemon (leak check). Report what the engine accepted/rejected.`,
  { label: 'validate', phase: 'Validate', model: 'sonnet', schema: { type: 'object', required: ['validator_clean', 'engine_loaded', 'notes'], properties: { validator_clean: { type: 'boolean' }, engine_loaded: { type: 'boolean' }, notes: { type: 'string' } } } })
log(`validate: lint=${val ? val.validator_clean : '?'} engine=${val ? val.engine_loaded : '?'}`)

phase('Critique')
const F = { type: 'object', required: ['findings'], properties: { findings: { type: 'array', items: { type: 'object', required: ['id', 'severity', 'target', 'claim', 'evidence'], properties: { id: { type: 'string' }, severity: { enum: ['error', 'major', 'minor'] }, target: { type: 'string' }, claim: { type: 'string' }, evidence: { type: 'string' } } } } } }
const crits = (await parallel([
  () => agent(`${G}
BLIND CRITIC (you built nothing). LENS: convention compliance. Grade ${REPO}/.sergeant/workflows/repo-to-icm/ against docs/icm/convention.md + record-shapes.md literally: layer discipline (L1 not a super-stage; Inputs tables complete and resolving; L3 vs L4 lifetime split), disposition+finalize rules, draft boundary in 60's instructions, helper rules (§5 — no machinery-as-stage), @@/reference resolution. Also: does 70-lint's validator invocation match what validate-structure.py actually checks? Findings only, no edits.`,
    { label: 'crit:convention', phase: 'Critique', model: 'opus', effort: 'medium', schema: F }),
  () => agent(`${G}
BLIND CRITIC (you built nothing). LENS: executability by a stranger. For each of the 10 stage contexts in ${REPO}/.sergeant/workflows/repo-to-icm/: could a fresh agent with ONLY this context + the worktree produce the declared outputs correctly? Hunt for: undefined terms, references to conversation state that won't exist, missing output-shape specs, ambiguous success criteria, blindness rule gaps (could an actor accidentally be led to reference-corpus/?), harvest/classify contexts that would produce units failing the evidence-policy (quote+hash) mechanically. Findings only, no edits.`,
    { label: 'crit:executability', phase: 'Critique', model: 'opus', effort: 'medium', schema: F }),
])).filter(Boolean)
const findings = crits.flatMap(c => c.findings)
log(`critique: ${findings.length} findings (${findings.filter(f => f.severity === 'error').length} errors)`)
return { meas, built, val, findings }
