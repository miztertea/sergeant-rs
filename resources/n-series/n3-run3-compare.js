export const meta = {
  name: 'n3-run3-comparison',
  description: 'Blind comparison of run-3 (generator v2) output vs reference corpus v1 + the v1-to-v2 delta verdict',
  phases: [
    { title: 'Compare', detail: '3 comparers over disjoint dimension groups' },
    { title: 'Adjudicate', detail: 'Opus: scorecard + v2 delta verdict' },
  ],
}
const REPO = '/home/user/sergeant-rs'
const GEN = REPO + '/docs/gauntlet/runs/n2-run3'
const V1 = REPO + '/docs/gauntlet/runs/n2-run2'
const REF = REPO + '/reference-corpus'
const G = `You compare sergeant-rs run-3 output (generator v2) against the frozen reference and against run 2 (generator v1). Run 3 (${GEN}): 312 units (20-harvest, normalized in 30-normalize, classified in 40-classify), 18 draft packages, per-file consequence-class-sweep.md, partition-ledger.md — which honestly records 6/21 partitions done (28/82 decompose files, per its own ledger; the other 15 are pending-and-resumable, a capability v1 lacked). Reference (${REF}, FROZEN.md v1): 979 units, 34 packages. Run 2 v1 baseline (${V1}): 108 units, 16 files, in-scope recall 47.3%, 11 silent consequence-class misses, precision clean — see its comparison-scorecard.md. Ground rules: SEPARATE within-covered-scope quality from raw coverage everywhere; the covered scope is exactly the files partition-ledger.md marks done. Cite unit ids/paths for every claim. Dimension definitions: proposal §9.9 (reference/proposal-next-iteration-icm-workflows.md). Write your section to the file named in your prompt; return a summary.`
const S = { type: 'object', required: ['headline_numbers', 'notable'], properties: { headline_numbers: { type: 'string' }, notable: { type: 'array', items: { type: 'string' } } } }

phase('Compare')
const [c1, c2, c3] = await parallel([
  () => agent(`${G}
Dimensions: source coverage, behavioral recall, behavioral precision — WITHIN run 3's covered files (read partition-ledger.md for the exact list). Map generated units to reference units citing the same sources; matched/missed/extra with id lists. THE question: the 11 consequence-class behaviors run 2 silently missed (listed in ${V1}/comparison-scorecard.md) — for each that lives in run 3's covered scope, is it now captured (cite the BU id) or still missing? Verify the consequence-class-sweep.md's "swept, none found" cells against reference units for those files (a false "none found" is a sweep failure — name it). Precision: spot-verify ≥15 quotes by recomputing hashes. Write ${GEN}/comparison-c1-coverage-recall.md.`,
    { label: 'cmp:recall', phase: 'Compare', model: 'sonnet', schema: S }),
  () => agent(`${G}
Dimensions: workflow-boundary, stage-boundary, representation agreement. Run 3 produced 18 packages (11 with stages, 7 zero-stage) — map each to its reference counterpart(s) and judge: reusable outcomes or file-shape mirroring? Over/under-staging vs the reference's post-A4 standard? Representation distribution (run 3: agents-invariant 106 / helper 87 / stage-context 73 / stage 31 / shared-*) vs the reference's within the same scope AND vs run 2's 9x shared-helper skew — did v2's §6.3-before-helper classify discipline actually move the distribution? Strongest examples both ways with ids. Write ${GEN}/comparison-c2-boundaries.md.`,
    { label: 'cmp:boundaries', phase: 'Compare', model: 'sonnet', schema: S }),
  () => agent(`${G}
Dimensions: provenance completeness, draft validity, review convergence, protocol mechanics. Provenance: sample ≥5 of the 18 packages' provenance trails to units+sources. Draft validity: rerun ${REPO}/.sergeant/workflows/repo-to-icm/scripts/validate-structure.py on ≥6 packages incl. zero-stage ones; did v2's finalize-aware drafting close run 2's S12/S3 failures? Review convergence: run 3's 80-adversarial-review found 5 findings (v1 found zero) and 90-reconcile adjudicated them — grade the axis addition: were the 5 real, and what C1/C2-class defects did it STILL miss? Protocol mechanics: assess partition-ledger.md + the checkpoint protocol as run — resumable state real and correctly formed? consequence-sweep rows complete for done partitions? Write ${GEN}/comparison-c3-validity.md.`,
    { label: 'cmp:validity', phase: 'Compare', model: 'sonnet', schema: S }),
])

phase('Adjudicate')
const adj = await agent(`${G.replace('You compare', 'You are the ADJUDICATOR over three comparers whose sections are in')}
Read ${GEN}/comparison-c1-coverage-recall.md, -c2-boundaries.md, -c3-validity.md, plus underlying artifacts where they disagree or hedge. Produce:
1. ${GEN}/comparison-scorecard.md — §9.9 dimensions, no single accuracy number; disagreements classified per §9.8; the §22.2 criterion answered explicitly for run 3's covered scope: is any consequence-class reference behavior silently absent? (An honestly-pending partition is not "silent" — the ledger names it; only misses inside done partitions count against the criterion.)
2. ${GEN}/v2-delta-report.md — the verdict on generator v2 vs v1, dimension by dimension: per-turn coverage (16→28 files), recall within comparable scope, the 11-miss list disposition, precision held?, representation skew moved?, review found real findings?, checkpoint/retry mechanics proven?, and the remaining gaps for a v3 (with the §6.7 lower-rung discipline for anything smelling like an engine ask — note GP-1's remaining lower rung is an orchestration retry loop in the CALLER, which run 3's own harvest report demonstrates; that is not engine pressure).
Return headline numbers + the delta verdict + safety criterion.`,
  { label: 'adjudicate', phase: 'Adjudicate', model: 'opus', effort: 'high', schema: {
    type: 'object', required: ['scorecard_headlines', 'safety_criterion_met', 'delta_verdict'],
    properties: { scorecard_headlines: { type: 'string' }, safety_criterion_met: { type: 'boolean' }, delta_verdict: { type: 'string' } },
  } })
return { c1, c2, c3, adj }
