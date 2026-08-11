export const meta = {
  name: 'north-star',
  description: 'Direction synthesis: four blind research seats → North Star draft → adversarial arbitration of competing paths. Owner rulings are hypotheses; evidence is the only fixed ground.',
  phases: [
    { title: 'Research', detail: '4 blind seats: product, core, ecosystem, history (Sonnet)' },
    { title: 'Synthesize', detail: 'North Star draft: destination, ownership layers, re-sequenced roadmap (Opus)', model: 'opus' },
    { title: 'Arbitrate', detail: '3 adversarial critics, each steelmanning a competing path (Opus)', model: 'opus' },
  ],
}

const REPO = '/home/miztertea/sergeant-rs'
const LICENSE = `EPISTEMIC LICENSE (owner-granted 2026-08-11, binding on every seat): every ruling in this repo — the owner's U-R1..R6, §2a, the ledger's rulings, the orchestrator's recommendations — is a HYPOTHESIS with provenance, not a constraint. "Everything in the repo is our thinking as of before right now." Argue positions on merit; the Ponytail minimality ladder applies to everyone's ideas including the owner's; measured evidence (dogfood runs, perf baselines, upstream incident history) outranks any ruling; the owner adjudicates OUTPUTS and expects to be argued with, not agreed with. A seat that merely ratifies the existing record has failed its brief.`

const PAPER = { type: 'object', required: ['position', 'evidence_cites', 'tensions_found', 'open_questions'], properties: { position: { type: 'string', description: 'the seat position paper, markdown, <=120 lines' }, evidence_cites: { type: 'array', items: { type: 'string' } }, tensions_found: { type: 'array', items: { type: 'string' }, description: 'places the existing record conflicts with this seat\'s evidence' }, open_questions: { type: 'array', items: { type: 'string' } } } }

phase('Research')
const seats = [
  { key: 'product', brief: `PRODUCT SEAT — what must day one feel like. Evidence: reference/sergeant-upstream/README.md + AGENTS.md (the boot contract's routing-table + standard-loop shape) + skills/ (the five operator skills); the owner's grill rulings in docs/gauntlet/notes/u-series-scope-draft-2026-08-11.md (as hypotheses); the dogfood friction evidence in docs/gauntlet/runs/dogfood-2026-08-11/run-manifest.md. The destination sentence to sharpen, owner verbatim: "You clone it down, cargo build or install or download the binary, get sgt on path -> open harness -> get to work... hop in your harness and say 'let's work on the api bug', it knows how to mount everything, drive sgt, knows how that repo works, what others it's associated with, its coding practices and flows." Define: the day-one loop, the first-five-minutes experience, what the human sees/types/says at each step, and what that DEMANDS from the OS layer. Include the mount-sgt-inside-sgt dogfood shape (the estate developing its own core).` },
  { key: 'core', brief: `CORE SEAT — what the engine is and must own. Evidence: src/ (cli.rs Command surface, lib.rs module tree, the §15 Backend trait in backend/mod.rs), CLAUDE.md's architecture invariants, docs/gauntlet/contracts/ (N0..N4 draft, BS2), docs/gauntlet/notes/upstream-core-function-map-2026-08-11.md (the 8-function delta + td answer). The identity hypothesis to test: "the core of sergeant is a durable workflow system that executes intents." Define: the crisp core boundary (what the binary owns forever vs what is OS/distro content vs what is nobody's), whether the 8 delta functions belong inside that boundary, and what the backlog/queued-intent concept does to the domain model (two-state capture->intended per the orchestrator's crack analysis — argue it).` },
  { key: 'ecosystem', brief: `ECOSYSTEM SEAT — surfaces and harnesses as claims, not commitments. Evidence: reference/proposal-tui-t-series.md (scope contract + IA), reference/proposal-harness-adapter-research-v2.md (the runtime-strategy family), docs/gauntlet/notes/h0-adjudication-packet-draft-2026-08-11.md. Both proposals are outside-in research the owner explicitly marked "for consideration not blind agreement." Define: which surface investments the destination actually requires in order (TUI? which harness transports? the E3 interactive hold?), which are premature, and what each proposal gets wrong or right against the day-one loop.` },
  { key: 'history', brief: `HISTORY SEAT — scar tissue and rejected paths. Evidence: docs/gauntlet/notes/upstream-lessons-mining-2026-08-11.md (12 lessons + the fail-closed-no-exit-door live risk), GAUNTLET.md (ledger + deviation register + backlog), LESSONS.md (all 18). Define: the constraints the destination must respect (what upstream proved by bleeding; what this program proved by measurement), the paths already rejected with reasons that still hold, and any current ruling whose rationale has quietly expired.` },
]
const papers = await parallel(seats.map(s => () => agent(
  `You are the ${s.key.toUpperCase()} research seat for sergeant-rs's North Star synthesis. Work read-only in ${REPO}; write no files. You are BLIND to the other three seats.
${LICENSE}
${s.brief}
Position papers are evidence-cited (file:line or doc section for every load-bearing claim), decisive, and honest about tensions with the existing record.`,
  { label: `seat:${s.key}`, phase: 'Research', model: 'sonnet', effort: 'high', schema: PAPER })))
log(`research: ${papers.filter(Boolean).length}/4 seats returned`)
if (papers.filter(Boolean).length < 4) throw new Error('a research seat failed — stop before synthesis')

phase('Synthesize')
const draft = await agent(
  `You are the North Star synthesist for sergeant-rs. Work read-only in ${REPO}; your returned draft is the artifact.
${LICENSE}
Inputs: the four seat papers below. Also read (for form, not authority): docs/gauntlet/notes/u-series-scope-draft-2026-08-11.md.
Produce the NORTH STAR draft, markdown, <=250 lines, sections exactly:
1. DESTINATION (one page max): what sergeant IS when finished — the AgentOS distro, the clone-to-working-in-five-minutes loop, the estate model, the core identity sentence — written so a stranger could build toward it.
2. OWNERSHIP LAYERS: core (binary) / OS (AGENTS.md, skills, conventions) / estate (repos, working set, per-repo instructions) / surfaces (CLI, TUI, harnesses) — what each owns, with the boundary rules that keep them from bleeding.
3. THE ROADMAP RE-SEQUENCED: every live proposal and queue item (U-series scope, T-series TUI, H-series adapters, N4 Docker, E1-E7, E3, the 8-function delta, backlog concept, library re-homing) placed in dependency order AGAINST THE DESTINATION — with explicit "not yet" and "not ever" bins and one-line reasons.
4. GAPS UNCOVERED: what the destination requires that nothing in the record currently provides.
5. TENSIONS RULED ON: every tension the seats found, with your call and one-line argument (these become adjudication items for the owner).
Where seats conflict, argue it out in the text — do not average. Where a ruling (owner's or orchestrator's) loses to evidence, say so plainly with the citation.
SEAT PAPERS:
${JSON.stringify(papers.map((p, i) => ({ seat: seats[i].key, paper: p })), null, 2)}`,
  { label: 'synthesize', phase: 'Synthesize', model: 'opus', effort: 'high', schema: { type: 'object', required: ['draft'], properties: { draft: { type: 'string' } } } })
if (!draft) throw new Error('synthesis failed')
log(`draft: ${draft.draft.length} chars`)

phase('Arbitrate')
const paths = [
  { key: 'core-first', stance: 'The destination is best served by hardening and finishing the CORE before any OS/surface investment: finish the engine deltas (backlog, callbacks, stall detection, E-fixes), keep the distro layer minimal until the core is unimpeachable. Argue this path over the draft\'s sequencing wherever the draft disagrees.' },
  { key: 'experience-first', stance: 'The destination is best served by making the day-one EXPERIENCE real immediately even on an imperfect core: AGENTS.md + skills + estate + sgt init land first, dogfooding drives the core fixes by lived friction, engine work happens strictly on demand. Argue this path over the draft\'s sequencing wherever the draft disagrees.' },
  { key: 'narrow-and-cut', stance: 'The destination is being over-scoped: argue the strongest CUT list — which proposals, delta functions, and concepts (TUI scope, harness families, backlog machinery, N4 Docker) should be dropped or radically narrowed per Ponytail, including any owner rulings that add machinery the destination does not require. The best version of this seat saves months.' },
]
const challenges = await parallel(paths.map(p => () => agent(
  `You are an adversarial arbitration critic for sergeant-rs's North Star. Work read-only in ${REPO}.
${LICENSE}
Steelman this competing path against the draft below: ${p.stance}
Rules: argue from the same evidence base (cite files/measurements); attack the draft's specific sequencing/ownership/scope claims, not vibes; concede points the draft genuinely wins; end with your path's strongest 5-line case and the single decision where choosing wrong costs the most.
THE DRAFT:
${'```'}
${draft.draft}
${'```'}`,
  { label: `arbitrate:${p.key}`, phase: 'Arbitrate', model: 'opus', effort: 'high', schema: { type: 'object', required: ['attack', 'concessions', 'strongest_case', 'costliest_decision'], properties: { attack: { type: 'string' }, concessions: { type: 'string' }, strongest_case: { type: 'string' }, costliest_decision: { type: 'string' } } } })))
log(`arbitration: ${challenges.filter(Boolean).length}/3 challengers returned`)

return { papers: papers.map((p, i) => ({ seat: seats[i].key, tensions: p.tensions_found, questions: p.open_questions })), draft: draft.draft, challenges: challenges.map((c, i) => c ? { path: paths[i].key, ...c } : null) }
