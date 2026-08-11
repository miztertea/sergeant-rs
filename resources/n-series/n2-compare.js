export const meta = {
  name: 'n2-comparison',
  description: 'Blind comparison of run-2 generated output vs reference corpus v1, adjudication, grammar-pressure report',
  phases: [
    { title: 'Compare', detail: '3 comparers over disjoint §9.9 dimension groups' },
    { title: 'Adjudicate', detail: 'Opus: disagreement classification + grammar-pressure report' },
  ],
}
const REPO = '/home/user/sergeant-rs'
const GEN = REPO + '/docs/gauntlet/runs/n2-run2'
const REF = REPO + '/reference-corpus'
const G = `You compare sergeant-rs N2 run-2 generated output (${GEN} — 108 units in 20-harvest/behavior-units.ndjson normalized in 30-normalize, classifications in 40-classify, 3 draft packages, coverage-note, review/reconcile artifacts) against the frozen reference (${REF}, FROZEN.md v1: 979 units, 34 packages). Ground rules: the generated run covered only 18 of 136 decompose files by its own honest coverage-note — SEPARATE within-covered-scope quality from coverage everywhere; conflating them destroys the measurement. Cite unit ids/paths for every claim. Read docs/gauntlet/contracts/N2.md §9.8-§9.9 framing first (in reference/proposal-next-iteration-icm-workflows.md §9.9 for dimension definitions). Write your section to the file named in your prompt; return a summary.`
const S = { type: 'object', required: ['headline_numbers', 'notable'], properties: { headline_numbers: { type: 'string' }, notable: { type: 'array', items: { type: 'string' } } } }

phase('Compare')
const [c1, c2, c3] = await parallel([
  () => agent(`${G}
Dimensions: source coverage, behavioral recall, behavioral precision — WITHIN the 18 covered files (AGENTS.md, README.md, the 14-file bin fleet-dispatch partition, +2 root). Map each generated unit to reference units citing the same sources; count matched/missed/extra with id lists; judge precision (is each generated unit source-supported?) by spot-verifying ≥15 quotes. Also state raw coverage: 18/136 files, 108/979-scale. Write ${GEN}/comparison-c1-coverage-recall.md.`,
    { label: 'cmp:recall', phase: 'Compare', model: 'sonnet', schema: S }),
  () => agent(`${G}
Dimensions: workflow-boundary, stage-boundary, representation agreement. The 3 generated packages (dispatch-mode, standard-task-workflow, ship-with-no-mistakes) vs their reference counterparts (dispatch/worker-mission/task-intake-and-route family; validate-and-ship; direct-implementation/implement). Did the generator find similar reusable outcomes or mirror file shapes? Over/under-staging vs the reference's post-A4 standard? Representation distribution comparison (its shared-helper-heavy skew vs reference's) with the strongest examples either way. Write ${GEN}/comparison-c2-boundaries.md.`,
    { label: 'cmp:boundaries', phase: 'Compare', model: 'sonnet', schema: S }),
  () => agent(`${G}
Dimensions: provenance completeness, draft validity, engine-gap quality, review convergence. Provenance: can every generated artifact trace to units and sources (check the 3 packages' provenance files)? Draft validity: rerun ${REPO}/.sergeant/workflows/repo-to-icm/scripts/validate-structure.py on each generated package; assess the S12 finding. Engine-gap quality: any generated engine-pressure claims vs reference's 4 surviving (G-series). Review convergence: 80-adversarial-review returned ZERO findings — assess against what C1/C2-class defects plainly exist (a review that finds nothing on first-generation output is evidence about the review stage's context, grade it). Write ${GEN}/comparison-c3-validity.md.`,
    { label: 'cmp:validity', phase: 'Compare', model: 'sonnet', schema: S }),
])

phase('Adjudicate')
const adj = await agent(`${G.replace('You compare', 'You are the ADJUDICATOR over three comparers whose sections are in')}
Read ${GEN}/comparison-c1-coverage-recall.md, -c2-boundaries.md, -c3-validity.md plus the underlying artifacts where they disagree or hedge. Produce:
1. ${GEN}/comparison-scorecard.md — the §9.9 dimensions, no single accuracy number; every disagreement classified per §9.8 (generator miss / generator invention / gold miss / legitimate alternate / ambiguous source / genuine engine pressure), with the §22.2 success criterion answered explicitly: is any reference behavior with safety/identity/recovery/delivery/human-decision consequence silently absent WITHIN covered scope?
2. ${REPO}/docs/gauntlet/runs/n2-run2/grammar-pressure-report.md — the N2 deliverable licensing Program B. Compile with §6.7-template rigor, from the runs' own recorded evidence: (a) the turn-budget/volume wall (18/136 files — what lower rungs could fix it: per-partition stages? intra-stage iteration via retry? and where genuine fan-out/composition pressure begins per proposal §21.8's trigger); (b) no actor-initiated mid-run ask primitive (run 1's recorded observation); (c) fresh-context-per-stage held (run 2, zero cross-stage leakage observed) — a capability CONFIRMED, not pressure; (d) fake-execution identity not surviving daemon restart — engine failed closed correctly (run 2's blocked event), note as evidence the recovery invariant works, plus any residual; (e) generated-draft S12 gap (finalize steps) — authoring-guidance fix vs engine need; (f) the zero-findings review stage — what stage-context guidance the next workflow version needs. Rank claims by evidence strength; reject anything 'convenient'.
Return headline scorecard numbers + the report's claim list.`,
  { label: 'adjudicate', phase: 'Adjudicate', model: 'opus', effort: 'high', schema: {
    type: 'object', required: ['scorecard_headlines', 'safety_criterion_met', 'pressure_claims'],
    properties: { scorecard_headlines: { type: 'string' }, safety_criterion_met: { type: 'boolean' }, pressure_claims: { type: 'array', items: { type: 'string' } } },
  } })
return { c1, c2, c3, adj }
