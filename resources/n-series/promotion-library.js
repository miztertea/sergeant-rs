export const meta = {
  name: 'promotion-library',
  description: 'Promotion step (migration plan addendum 2026-08-11): curate the 34 adjudicated reference packages into .sergeant/workflows/ as a runnable, engine-tested library',
  phases: [
    { title: 'Survey', detail: 'curation spec from the convention + exemplar + drafts' },
    { title: 'Curate', detail: 'pipeline: one curator per package, engine-acceptance gated' },
    { title: 'Review', detail: 'library-level Opus critic + fixer' },
  ],
}

const WT = '/home/miztertea/sergeant-promotion'      // worktree, branch cerberus/promotion
const SGT = '/home/miztertea/sergeant-runb/target/debug/sgt'  // prebuilt at the same HEAD; read-only use
const SCRATCH = '/tmp/claude-1001/-home-miztertea-sergeant-rs/6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/promotion'

const G = `You are working on the sergeant-rs promotion step in the dedicated worktree ${WT} (branch cerberus/promotion). NEVER touch /home/miztertea/sergeant-rs, /home/miztertea/sergeant-harvest, or /home/miztertea/sergeant-runb except reading the prebuilt binary ${SGT}. Do not run cargo anywhere. Commits in ${WT} end "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>". Do NOT push. Every daemon you start uses its own data dir under ${SCRATCH}/ and MUST be stopped and verified gone before you finish (pgrep the exact data-dir path).`

phase('Survey')
const spec = await agent(`${G}
Produce the CURATION SPEC the per-package curators will follow. Read: ${WT}/docs/icm/convention.md (the four-layer model, binding), ${WT}/.sergeant/workflows/repo-to-icm/ (the exemplar RUNNABLE package — its workflow.toml, stage CONTEXT.md shapes with Inputs tables, _config/, references/, output/ scaffolds), ${WT}/.sergeant/workflows/software-change/ (the sample), the migration-plan addendum (${WT}/docs/gauntlet/notes/v2-measurement-and-migration-plan.md, read the addendum section — note it may only exist on the PR #49 branch; if absent locally, the runs README quote in your prompt suffices: curate reference-corpus/draft-workflows/ — the 34 adjudicated packages, source of record — into a runnable library; structure-validate each; run each through the engine on the fake backend as its acceptance gate), and THREE draft packages of different sizes from ${WT}/reference-corpus/draft-workflows/ (pick small/medium/large by stage count).
Write ${WT}/docs/icm/promotion-spec-2026-08-11.md: (1) the exact structural delta between a draft package and a runnable package (what must be added/renamed/rewritten — workflow.toml stage declarations, CONTEXT.md Inputs tables resolving to real files, _config presence, output/ scaffolds with README disposition lines, references/ layer placement); (2) what is FORBIDDEN to change (the adjudicated behavioral content — stage boundaries, behavior-unit citations, judgment checkpoints; curation is packaging, not re-authoring); (3) the engine-acceptance procedure verbatim: private daemon on a fresh data dir (binary ${SGT}, fake backend, NO SGT_FAKE_SCRIPT set so every stage completes), scratch subject repo containing the package under .sergeant/workflows/, submit with a plausible intent naming the workflow, assert work completes with all stages entered+completed in order and workflow.bound pinned the full stage list, daemon stopped + verified; (4) the naming rule for the library (draft dir name -> .sergeant/workflows/<name>); (5) a classification of all 34 packages: STRAIGHTFORWARD vs NEEDS-JUDGMENT (with one line why each judgment case is hard). Commit the spec. Return the classification as the schema requires.`,
  { label: 'survey', phase: 'Survey', model: 'sonnet', effort: 'high', schema: {
    type: 'object', required: ['spec_path', 'straightforward', 'needs_judgment', 'notes'],
    properties: { spec_path: { type: 'string' }, straightforward: { type: 'array', items: { type: 'string' } }, needs_judgment: { type: 'array', items: { type: 'object', required: ['package', 'why'], properties: { package: { type: 'string' }, why: { type: 'string' } } } }, notes: { type: 'string' } },
  } })
if (!spec) throw new Error('survey failed')
log(`spec at ${spec.spec_path}: ${spec.straightforward.length} straightforward, ${spec.needs_judgment.length} needs-judgment`)

phase('Curate')
const CURATE_SCHEMA = {
  type: 'object', required: ['package', 'promoted', 'engine_gate_passed', 'stages', 'deviations', 'commit'],
  properties: { package: { type: 'string' }, promoted: { type: 'boolean' }, engine_gate_passed: { type: 'boolean' }, stages: { type: 'number' }, deviations: { type: 'array', items: { type: 'string' }, description: 'anything the spec did not anticipate, or a decline-with-reason' }, commit: { type: 'string' }, daemon_clean: { type: 'boolean' } },
}
const hardSet = new Set(spec.needs_judgment.map(j => j.package))
const all = [...spec.straightforward, ...spec.needs_judgment.map(j => j.package)]
const curated = await pipeline(all, (pkg, _orig, i) => agent(`${G}
Curate ONE package: reference-corpus/draft-workflows/${pkg} -> ${WT}/.sergeant/workflows/. The law: ${WT}/${spec.spec_path.replace(WT + '/', '')} (read it first, follow it exactly — packaging, never re-authoring; the adjudicated content is frozen behavior). ${hardSet.has(pkg) ? 'This package is classified NEEDS-JUDGMENT: ' + (spec.needs_judgment.find(j => j.package === pkg) || {}).why + ' — resolve it the minimal-packaging way, record the resolution in deviations, and if it genuinely requires re-authoring adjudicated content, do NOT promote: return promoted=false with the reason.' : ''}
Then the engine-acceptance gate per the spec's procedure verbatim (your OWN data dir: ${SCRATCH}/gate-${i}/). A package that fails the gate is fixed at the packaging layer and re-gated, or returned promoted=false with the failure verbatim. ONE commit per package ("promote(${pkg}): ..."). git pull --rebase --autostash before committing if the branch moved (other curators commit concurrently; your package dir is exclusively yours). Verify your daemon is gone before returning.`,
  { label: `curate:${pkg}`, phase: 'Curate', model: 'sonnet', schema: CURATE_SCHEMA }))
const done = curated.filter(Boolean)
const promoted = done.filter(c => c.promoted && c.engine_gate_passed)
const declined = done.filter(c => !c.promoted)
const dirtyDaemons = done.filter(c => c.daemon_clean === false)
log(`curated: ${promoted.length}/${all.length} promoted+gated, ${declined.length} declined-with-reason, ${curated.length - done.length} agent failures, ${dirtyDaemons.length} dirty-daemon reports`)

phase('Review')
const review = await agent(`You are the blind library-level critic for the sergeant-rs promotion step (fresh eyes; you curated nothing). Work read-only in ${WT} (git log/diff on branch cerberus/promotion; you may run engine-acceptance probes using ${SGT} with your own data dirs under ${SCRATCH}/review/, stopped+verified after). Read first: docs/icm/convention.md, the promotion spec (docs/icm/promotion-spec-2026-08-11.md), GAUNTLET.md's register (L3).
Axes, all yours: (1) LIBRARY COHERENCE — the full .sergeant/workflows/ listing: naming collisions, packages that shadow or duplicate each other, index/discovery coherence (if an index file convention exists, is it updated?); (2) PACKAGING FIDELITY — pick 6 packages (2 you choose randomly, 2 from the needs-judgment list, 2 largest): diff promoted content against its reference-corpus/draft-workflows/ source — any behavioral re-authoring (changed stage boundaries, altered checkpoints, dropped citations) is an ERROR (the source of record is adjudicated); (3) GATE HONESTY — re-run the engine-acceptance gate yourself on 3 of the 6: does it actually pass, and does workflow.bound pin the declared stages? (4) DECLINED packages — is each decline reason real (spot-check one)?
Findings with evidence. DECLINED: ${JSON.stringify(declined.map(d => ({ package: d.package, deviations: d.deviations })), null, 2)}`,
  { label: 'critic:library', phase: 'Review', model: 'opus', effort: 'high', schema: {
    type: 'object', required: ['findings', 'summary'],
    properties: { summary: { type: 'string' }, findings: { type: 'array', items: { type: 'object', required: ['id', 'severity', 'claim', 'evidence'], properties: { id: { type: 'string' }, severity: { type: 'string', enum: ['error', 'warning', 'info'] }, claim: { type: 'string' }, evidence: { type: 'string' } } } } },
  } })
const errs = review ? review.findings.filter(f => f.severity !== 'info') : []
let fix = null
if (errs.length > 0) {
  fix = await agent(`${G}
Promotion fixer. Close every error/warning finding below (fix at the packaging layer; if a finding proves a package needs re-authoring of adjudicated content, DEMOTE it — remove from .sergeant/workflows/, record why). Re-run the engine gate on every package you touch. Separable commits.
FINDINGS: ${JSON.stringify(errs, null, 2)}`,
    { label: 'fixer:library', phase: 'Review', model: 'sonnet', effort: 'high', schema: {
      type: 'object', required: ['commits', 'notes'], properties: { commits: { type: 'array', items: { type: 'string' } }, notes: { type: 'string' } },
    } })
}

return {
  promoted: promoted.length,
  declined: declined.map(d => ({ pkg: d.package, why: d.deviations })),
  agent_failures: curated.length - done.length,
  review: review ? { findings: review.findings.length, errors: errs.length } : null,
  fix: fix ? fix.commits : null,
}
