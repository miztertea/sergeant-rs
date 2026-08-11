export const meta = {
  name: 'mvp-plan-review',
  description: 'Owner-shaped review pipeline over the unreviewed planning batch: 4 Sonnet critics -> 1 Opus writer -> 1 Sonnet sanity check',
  phases: [
    { title: 'Review', detail: '4 Sonnet critics: dependency, design-reality, ponytail, honesty/vision' },
    { title: 'Write', detail: 'Opus writer applies confirmed findings, records disputes', model: 'opus' },
    { title: 'Sanity', detail: 'fresh Sonnet consistency pass over the rewritten documents' },
  ],
}

const REPO = '/home/miztertea/sergeant-rs'
const TARGETS = `The documents under review (the unreviewed batch — orchestrator+owner chat output, zero fresh eyes so far):
1. ${REPO}/docs/gauntlet/notes/mvp-bucketing-2026-08-11.md (the governing MVP plan)
2. ${REPO}/docs/gauntlet/notes/estate-manifest-design-2026-08-11.md (the manifest design capture)
3. ${REPO}/NORTH-STAR.md (specifically the 2026-08-11 REVISED waves amendment; the rest was panel-reviewed)`
const CONTEXT = `Grounding you may cite: NORTH-STAR.md (destination/layers/rulings), docs/gauntlet/notes/north-star-{draft,arbitration,dispositions}-2026-08-11.md, docs/gauntlet/notes/upstream-core-function-map-2026-08-11.md, docs/gauntlet/notes/upstream-lessons-mining-2026-08-11.md, docs/gauntlet/runs/dogfood-2026-08-11/run-manifest.md, docs/icm/retriage-2026-08-11.md, GAUNTLET.md (register/backlog), LESSONS.md, gh issue list (22 open), and src/ where a claim touches code. The epistemic license binds: every ruling including the owner's is a hypothesis; evidence outranks authority; ratifying the record is failing the brief.`
const FINDINGS = { type: 'object', required: ['findings', 'summary'], properties: { summary: { type: 'string' }, findings: { type: 'array', items: { type: 'object', required: ['id', 'severity', 'claim', 'evidence', 'proposed_fix'], properties: { id: { type: 'string' }, severity: { type: 'string', enum: ['error', 'warning', 'info'] }, claim: { type: 'string' }, evidence: { type: 'string' }, proposed_fix: { type: 'string' } } } } } }

phase('Review')
const axes = [
  { key: 'dependency', brief: `AXIS: DEPENDENCY & COMPLETENESS. Attack the bucketing's layer assignments and ordering: hidden cross-bucket dependencies (does MVP-3 sgt init need MVP-5 AGENTS.md content? does the MVP-2 de-leak order against the MVP-1 manifest parse? does MVP-4's soak need MVP-5's workflows to have something real to run?); items assigned to the wrong layer; and MISSING items — sweep gh issue list --state open, the E1-E7 list, the 8-function delta map, the 179-file upstream census categories (worker bundles, operator skills, invariant/helper units, templates/schema), and the retriage/sweep verdicts, and name anything with no bucket at all.` },
  { key: 'design-reality', brief: `AXIS: DESIGN vs SOURCE REALITY. Attack the manifest design against the code as it exists: what does Workspace::from_config actually parse from sergeant.toml today and does the [estate]/[[repo]]/[group] extension collide (read src/domain/ and the config path); does pin-at-bind fit how bindings/workflow.bound actually work in src/runtime/engine.rs; is per-operation manifest read compatible with how submission resolves workspaces today; do the CLI verb shapes conflict with the existing Command enum; is the --setting-sources de-leak actually one flag translation in src/backend/claude.rs or does it touch more (read the launch grammar); can the turn-count envelope enforce at PREPARE/LAUNCH with the types that exist (read the §15 trait). Every claim file:line grounded.` },
  { key: 'ponytail', brief: `AXIS: PONYTAIL, both directions. (1) Scope smuggled into MVP wearing a "cheap" costume — is the drain flag really small (check what admission touches in the daemon/api)? is the intent schema slot really a slot? is sgt group worth MVP when groups could be a manifest-only concept read by the harness? (2) Genuinely cheap enablers still deferred that will hurt later — anything in post-MVP whose absence forces rework of MVP code (e.g. does deferring the backlog TYPE while landing the schema slot actually avoid rework, or does promotion provenance need a hook now?). (3) Anything in MVP that exists because momentum, not because the happy path needs it — argue the strongest cut.` },
  { key: 'honesty-vision', brief: `AXIS: HONESTY & VISION. Grade the plan against the problem statement (NORTH-STAR.md destination + the owner's articulation: harness amnesia + session fragility = the returning-developer tangle). (1) Does every MVP bucket trace to a tangle it removes — and does the happy-path narrative actually become true if exactly these buckets land, nothing more? Walk it step by step and name the first step that breaks. (2) Honesty audit of claims: "E3 dissolved" (is it, for the payments loop?), "E6-as-corrected", "retention already works", "BS2 covers process-death", the cheap-now labels — any claim stated stronger than its evidence. (3) Sycophancy check: places where the plan encodes an owner suggestion the evidence contradicts, or an orchestrator recommendation that survived unexamined. (4) Vision drift: anything in the plan that quietly serves the old solution-first framing rather than the problem.` },
]
const reviews = await parallel(axes.map(a => () => agent(
  `You are a blind reviewer for sergeant-rs's MVP plan (fresh context; you built none of this). Work read-only in ${REPO}. ${TARGETS}
${CONTEXT}
${a.brief}
Findings per the schema: severity error/warning/info, evidence with file:line or doc-section cites, and a concrete proposed_fix per finding (the writer applies these — vague findings get dropped).`,
  { label: `review:${a.key}`, phase: 'Review', model: 'sonnet', effort: 'high', schema: FINDINGS })))
const all = reviews.filter(Boolean).flatMap((r, i) => r.findings.map(f => ({ ...f, id: `${axes[i].key}:${f.id}` })))
log(`review: ${all.length} findings (${all.filter(f => f.severity === 'error').length} error) across ${reviews.filter(Boolean).length}/4 axes`)

phase('Write')
const written = await agent(
  `You are the Opus writer for sergeant-rs's MVP plan revision. Work in ${REPO} (branch cerberus/day2-adjudications). ${TARGETS}
${CONTEXT}
Your job: produce the corrected documents by applying the review findings below. Rules:
- Apply every finding you verify (check its evidence yourself where load-bearing — file:line cites especially). A finding you REJECT gets recorded, not silently dropped: add a "## Review dispositions (2026-08-11 pipeline)" section at the bottom of the bucketing doc listing every finding id -> applied / rejected(reason) / escalated(needs owner or orchestrator).
- Rewrite in place: docs/gauntlet/notes/mvp-bucketing-2026-08-11.md and docs/gauntlet/notes/estate-manifest-design-2026-08-11.md (and NORTH-STAR.md's REVISED paragraph ONLY if a finding demands it). Preserve each document's voice and length discipline; you are correcting, not expanding — the bucketing stays under ~120 lines + dispositions, the manifest capture under ~90.
- Escalate genuine fork-decisions (where two fixes are defensible and the choice is the owner's) rather than picking silently.
- Commit your rewrite as ONE commit: "MVP plan v2: review pipeline findings applied" + "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>". Do NOT push.
FINDINGS (${all.length}):
${JSON.stringify(all, null, 2)}`,
  { label: 'writer', phase: 'Write', model: 'opus', effort: 'high', schema: { type: 'object', required: ['applied', 'rejected', 'escalated', 'commit', 'notes'], properties: { applied: { type: 'number' }, rejected: { type: 'number' }, escalated: { type: 'array', items: { type: 'string' } }, commit: { type: 'string' }, notes: { type: 'string' } } } })
if (!written) throw new Error('writer failed')
log(`writer: ${written.applied} applied, ${written.rejected} rejected, ${written.escalated.length} escalated — ${written.commit}`)

phase('Sanity')
const sanity = await agent(
  `Fresh Sonnet sanity check for sergeant-rs. Read ONLY the final documents as they now stand in ${REPO}: docs/gauntlet/notes/mvp-bucketing-2026-08-11.md, docs/gauntlet/notes/estate-manifest-design-2026-08-11.md, NORTH-STAR.md. Check: internal consistency (no bucket references a thing another section deleted; the dispositions section accounts for the findings it names), no contradictions between the three documents, every doc/file reference resolves (spot-check paths exist), the happy-path narrative matches the buckets as revised, and nothing reads as obviously garbled. Do NOT re-litigate design. Return pass/fail per document with any defects listed (file + line-ish + what).`,
  { label: 'sanity', phase: 'Sanity', model: 'sonnet', effort: 'medium', schema: { type: 'object', required: ['pass', 'defects'], properties: { pass: { type: 'boolean' }, defects: { type: 'array', items: { type: 'string' } } } } })
return { findings: all.length, errors: all.filter(f => f.severity === 'error').length, writer: written, sanity }
