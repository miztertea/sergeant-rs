export const meta = {
  name: 'n1-reference-decomposition',
  description: 'Decompose reference/sergeant-upstream into the adjudicated ICM reference corpus per docs/gauntlet/contracts/N1.md',
  phases: [
    { title: 'Convention', detail: 'ICM convention docs under docs/icm/' },
    { title: 'Inventory', detail: 'full source inventory with dispositions' },
    { title: 'Extract', detail: 'behavior units + ICM classification per partition' },
    { title: 'Synthesize', detail: 'cluster units into candidate workflows (Opus)' },
    { title: 'Draft', detail: 'materialize corpus documents and draft workflows' },
    { title: 'Lint', detail: 'structural validation' },
    { title: 'Refute', detail: 'two independent boundary/completeness reviewers (Opus)' },
  ],
}

const REPO = '/home/user/sergeant-rs'
const SRC = REPO + '/reference/sergeant-upstream'
const OUT = REPO + '/reference-corpus'
const PROPOSAL = REPO + '/reference/proposal-next-iteration-icm-workflows.md'
const CONTRACT = REPO + '/docs/gauntlet/contracts/N1.md'

const GROUNDING = `You are one worker inside the N1 milestone of sergeant-rs.
Read these first, in order (they are short where it matters):
1. ${CONTRACT} — the contract you are executing (binding).
2. ${PROPOSAL} — sections 5, 6 (the ICM decomposition ladder, apply it exactly), 8, 9.4-9.5 (record shapes).
3. ${REPO}/GAUNTLET.md — ONLY the deviation register + backlog tables at the top (settled rulings; do not re-litigate).
Rules that get work rejected: a behavior unit without source path + locator + a sha256 hash of the quoted text is invention; an engine-gap claim without named failed lower rungs is auto-rejected; "would be convenient" is not evidence. Old Sergeant's implementation mechanisms (tmux, sentinels, worker Bash) are NOT the behavior — extract intent, note mechanism separately. Tests and scripts may encode behavior more authoritatively than docs. Never modify anything under reference/ (it is read-only evidence) and never write outside your assigned output paths. Do not run cargo. Do not commit.`

const UNIT_SHAPE = `Each behavior-unit NDJSON line follows proposal §9.4: {"id":"BU-<partition>-<nnn>","statement":...,"source":{"path":...,"locator":...,"quote_hash":"sha256:..."},"scope":...,"trigger":...,"outcome":...,"authority":...,"confidence":"high|medium|low","notes":...}. Add per §9.5 classification fields on the same line: "representation" (one of: agents-invariant | workflow | stage | stage-context | helper | shared-context | shared-helper | obsolete-mechanism | engine-gap), "workflow" (candidate name or null), "stage" (candidate id or null), "rationale", "alternatives_considered" (array), "engine_gap" (null, or the §6.7 template object with behavior/evidence/lower_rungs_attempted/why_each_fails/minimum_capability/acceptance_test).`

const SUMMARY_SCHEMA = {
  type: 'object',
  required: ['units_written', 'output_files', 'uncertainties', 'engine_gap_claims', 'notes'],
  properties: {
    units_written: { type: 'number' },
    output_files: { type: 'array', items: { type: 'string' } },
    uncertainties: { type: 'array', items: { type: 'string' }, description: 'classifications you were genuinely unsure about, with unit ids' },
    engine_gap_claims: { type: 'array', items: { type: 'string' }, description: 'unit ids classified engine-gap' },
    notes: { type: 'string' },
  },
}

// ---- Phase 1: convention docs + inventory (parallel, independent) ----
phase('Convention')
const [convention, inventory] = await parallel([
  () => agent(`${GROUNDING}

Write the ICM filesystem convention documents under ${REPO}/docs/icm/ (create the directory):
- convention.md — the .sergeant/ catalog layout (proposal §7.1), the draft publication boundary (drafts/ is never runnable), the @@name shared-context convention (§7.5), helper rules (§7.6), and the AGENTS.md "small constitution" shape (§7.2).
- record-shapes.md — the OKF-compatible index.md front-matter shape (§7.3), authored-vs-observed metadata separation (§7.4), the behavior-unit record (§9.4), the classification record (§9.5), and the engine-gap claim template (§6.7).
These document SHAPES for authors and generators; no tooling, no Rust. Write them tight and normative — every rule states what violates it. Return a one-paragraph summary of what you wrote.`,
    { label: 'convention-docs', phase: 'Convention', model: 'sonnet' }),

  () => agent(`${GROUNDING}

Produce ${OUT}/source-inventory.md (create ${OUT}/). Enumerate EVERY file under ${SRC} (find gives ~179). For each: path, one-line what-it-is, and a disposition: decompose (behavior-bearing; goes to extraction), helper-evidence (mechanics that inform helper-map), obsolete-candidate (mechanism the Rust runtime structurally replaced), reference-only (research/PRDs/licenses/binary/pycache — excluded, with reason). Group by directory; end with a partition table assigning every "decompose" file to exactly one of these eight partitions: P1 root-instructions (AGENTS.md, README.md, docs/what-is-sergeant.md, docs/skills.md, docs/repo-scoped-skills.md and similar operating docs), P2 dev-skills-a (.agents/skills: no-mistakes, code-review, implement, tdd, diagnosing-bugs), P3 dev-skills-b (.agents/skills: prototype, research, triage, grilling, grill-with-docs, resolving-merge-conflicts), P4 design-skills (.agents/skills: codebase-design, domain-modeling, to-spec, to-tickets, wayfinder), P5 ops-skills (sergeant-setup + skills/: dispatch, load-project, cross-repo-work, sergeant-help, wiki), P6 bin-machinery (bin/, hooks, mise.toml, opencode.json), P7 tests-schema (tests/, schema/, templates/), P8 support-docs (remaining behavior-bearing docs/ files: callbacks.md, schema.md, docs/README.md etc; NOT prds/ or research/ which are reference-only). Return counts per disposition and per partition.`,
    { label: 'inventory', phase: 'Inventory', model: 'sonnet', schema: {
      type: 'object', required: ['total_files', 'decompose', 'reference_only', 'partition_counts'],
      properties: { total_files: { type: 'number' }, decompose: { type: 'number' }, reference_only: { type: 'number' }, partition_counts: { type: 'object' } },
    } }),
])
if (!inventory) throw new Error('inventory failed')
log(`inventory: ${inventory.total_files} files, ${inventory.decompose} to decompose`)

// ---- Phase 2: extract + normalize + classify, one Sonnet worker per partition ----
phase('Extract')
const PARTITIONS = ['P1', 'P2', 'P3', 'P4', 'P5', 'P6', 'P7', 'P8']
const extracts = (await parallel(PARTITIONS.map(p => () => agent(`${GROUNDING}

You own partition ${p}. Read ${OUT}/source-inventory.md to find exactly which files are assigned to ${p}; read every one of them completely. Extract atomic behavior units, normalize each into implementation-independent language, and classify each through the §6 ladder in order (§6.1 invariant → §6.2 workflow → §6.3 stage → §6.4 actor context → §6.5 helper → §6.6 shared → §6.7 engine-gap).

${UNIT_SHAPE}

Write ${OUT}/behavior-units/${p}.ndjson (create dirs). Compute quote hashes for real: sha256 over the exact quoted source text (use bash: printf '%s' "<quote>" | sha256sum). Prefer many small precise units over few vague ones. For skills: the SKILL.md prose, its referenced scripts, and its examples are one behavioral surface — cite each unit to the most authoritative source. For bin/ machinery (P6): most of it is obsolete-mechanism (the Rust daemon replaced dispatch/tmux/sentinel/callback plumbing structurally) — but the POLICY buried in the mechanism (e.g. drain semantics, response locking intent, validation ordering) is durable behavior; separate them ruthlessly. For tests (P7): a test pins behavior; extract what it proves, cite the test. Mark low confidence honestly — uncertainty preserved beats confidence invented.`,
  { label: `extract:${p}`, phase: 'Extract', model: 'sonnet', schema: SUMMARY_SCHEMA })
))).filter(Boolean)
const totalUnits = extracts.reduce((n, e) => n + e.units_written, 0)
log(`extracted ${totalUnits} behavior units across ${extracts.length}/8 partitions`)

// ---- Phase 3: synthesis (needs ALL units — genuine barrier), Opus ----
phase('Synthesize')
const synthesis = await agent(`${GROUNDING}

All behavior units are in ${OUT}/behavior-units/P1.ndjson … P8.ndjson. Read every line of every file. Your job is the cross-partition synthesis (§8.3 step 5):

Write ${OUT}/synthesis.md containing:
1. Candidate workflow list — cluster units into named candidate workflows (expect shapes like diagnose-bug, implement-change, no-mistakes-validation, prototype, load-project, cross-repo-work, sergeant-setup, code-review, plus whatever the units actually support). For each: purpose, triggering situation, the unit ids it absorbs, proposed stage list with each stage's durable outcome (apply §6.3's reimplementation test — a stage must survive its implementation being replaced), and which stages are judgment (actor) vs deterministic-machinery candidates.
2. Permanent-instruction set — units classified agents-invariant, deduplicated into a coherent small constitution outline.
3. Shared context/helper candidates — reused guidance and reused mechanics, with the workflows that share them.
4. Obsolete-mechanism roll-up — mechanism clusters the Rust runtime structurally replaced, each with the durable policy (if any) that must survive it.
5. Engine-pressure roll-up — collect ALL engine_gap claims, merge duplicates, and for each: does it survive §6.7 (are the lower-rung failures real)? Rank by evidence strength. Be adversarial with weak claims — most first-pass engine gaps are actually representable at a lower rung.
6. Cross-partition conflicts — units that contradict each other across partitions, listed explicitly with both citations.
Do not drop any unit silently: every unit id appears either in a cluster, the invariant set, a shared/helper map, the obsolete roll-up, or an explicit unassigned list with a reason.`,
  { label: 'synthesize', phase: 'Synthesize', model: 'opus', effort: 'high', schema: {
    type: 'object', required: ['workflow_candidates', 'engine_gaps_surviving', 'engine_gaps_rejected', 'conflicts', 'unassigned_units'],
    properties: { workflow_candidates: { type: 'array', items: { type: 'string' } }, engine_gaps_surviving: { type: 'number' }, engine_gaps_rejected: { type: 'number' }, conflicts: { type: 'number' }, unassigned_units: { type: 'number' } },
  } })
if (!synthesis) throw new Error('synthesis failed')
log(`synthesis: ${synthesis.workflow_candidates.length} workflow candidates, ${synthesis.engine_gaps_surviving} surviving engine gaps`)

// ---- Phase 4: draft the corpus documents (parallel Sonnet, disjoint outputs) ----
phase('Draft')
const drafters = [
  {
    label: 'draft:workflows',
    prompt: `From ${OUT}/synthesis.md section 1 (and the underlying ${OUT}/behavior-units/*.ndjson for detail), materialize EVERY candidate workflow as a draft package under ${OUT}/draft-workflows/<name>/ following docs/icm/convention.md and record-shapes.md: index.md (OKF front matter, status: draft), workflow.toml (ordered stage ids), per-stage <nn>-<id>/CONTEXT.md for actor stages (real usable context distilled from the source behavior, not placeholders), and provenance.md mapping every stage to its unit ids. Draft quality bar: a competent agent could execute the stage from its CONTEXT.md alone.`,
  },
  {
    label: 'draft:instructions-maps',
    prompt: `From ${OUT}/synthesis.md sections 2-4 and the unit files, write: ${OUT}/permanent-instructions.md (the small-constitution candidate with per-line unit citations), ${OUT}/helper-map.md (each helper: contract, source evidence, which workflows use it, workflow-local vs shared verdict), ${OUT}/shared-context-map.md (each shared context: the guidance, sharing workflows, @@name it would get), ${OUT}/obsolete-mechanisms.md (each mechanism cluster: what it was, why the Rust runtime replaces it structurally, the surviving policy and where that policy now lives in the corpus).`,
  },
  {
    label: 'draft:ledger-pressure',
    prompt: `From ${OUT}/synthesis.md sections 5-6 and the unit files, write: ${OUT}/engine-pressure.md (each surviving engine-gap claim in the full §6.7 template; each rejected claim listed with the lower rung that absorbs it), ${OUT}/classification-ledger.md (the adjudication surface: per-partition unit counts, confidence distribution, every cross-partition conflict with both citations and a proposed resolution marked PROPOSED, every low-confidence unit listed, synthesis unassigned units), and ${OUT}/provenance-map.md (source file → unit ids → destination artifact, so every path in source-inventory.md marked "decompose" is traceable forward; list any decompose-file with zero units as a gap).`,
  },
]
const drafted = (await parallel(drafters.map(d => () => agent(`${GROUNDING}\n\n${d.prompt}\n\nReturn the files you wrote and any gaps you could not fill.`,
  { label: d.label, phase: 'Draft', model: 'sonnet', schema: { type: 'object', required: ['files', 'gaps'], properties: { files: { type: 'array', items: { type: 'string' } }, gaps: { type: 'array', items: { type: 'string' } } } } })
))).filter(Boolean)
log(`drafted: ${drafted.flatMap(d => d.files).length} files, ${drafted.flatMap(d => d.gaps).length} declared gaps`)

// ---- Phase 5: structural lint ----
phase('Lint')
const lint = await agent(`${GROUNDING}

Structurally validate the corpus at ${OUT} against docs/icm/record-shapes.md and the N1 contract gate. Check mechanically (write and run a throwaway python script in ${REPO}/../n1-lint-scratch or /tmp — NOT inside the repo):
1. every NDJSON line parses and has all required §9.4/§9.5 fields;
2. unit ids unique; every source path cited actually exists under ${SRC};
3. every engine-gap unit carries the full §6.7 template;
4. every draft workflow has index.md + workflow.toml + provenance.md, stage dirs match workflow.toml order, actor stages have non-empty CONTEXT.md;
5. every provenance.md unit id exists in the NDJSON;
6. every source-inventory "decompose" file appears in provenance-map.md (zero-unit files listed as gaps count as appearing);
7. no file written outside ${OUT}, docs/icm/, and no modification under ${SRC} (git -C ${REPO} status --short must show only reference-corpus/, docs/icm/ additions -- plus pre-existing staged/untracked entries you must list but not touch).
FIX mechanical defects you find (malformed lines, missing fields with recoverable intent, broken paths) directly in the corpus files; report substantive defects (misclassification, invention) without fixing. Return pass/fail per check with counts.`,
  { label: 'lint', phase: 'Lint', model: 'sonnet', schema: {
    type: 'object', required: ['checks_passed', 'checks_failed', 'fixed', 'substantive_defects'],
    properties: { checks_passed: { type: 'number' }, checks_failed: { type: 'number' }, fixed: { type: 'array', items: { type: 'string' } }, substantive_defects: { type: 'array', items: { type: 'string' } } },
  } })
log(`lint: ${lint ? lint.checks_passed + ' passed, ' + lint.checks_failed + ' failed' : 'FAILED TO RUN'}`)

// ---- Phase 6: two independent Opus reviewers, distinct lenses (contract requires >=2) ----
phase('Refute')
const FINDING_SCHEMA = {
  type: 'object', required: ['findings'],
  properties: { findings: { type: 'array', items: {
    type: 'object', required: ['id', 'severity', 'target', 'claim', 'evidence'],
    properties: { id: { type: 'string' }, severity: { enum: ['error', 'major', 'minor'] }, target: { type: 'string' }, claim: { type: 'string' }, evidence: { type: 'string' } },
  } } },
}
const reviewers = [
  {
    label: 'refute:boundaries',
    lens: `LENS: boundary honesty. Challenge workflow and stage boundaries: does each candidate workflow have a real trigger/outcome/completion (§6.2) or does it mirror a source file's shape? Does each stage survive §6.3's reimplementation test, or is it a command promoted to a checkpoint (over-staging)? Are independently measurable checkpoints collapsed into prose (under-staging)? Is anything labeled a helper actually a durable checkpoint, or vice versa? Sample deeply: read at least no-mistakes, dispatch-derived, diagnose-bug, and sergeant-setup candidates end-to-end against their source files.`,
  },
  {
    label: 'refute:completeness',
    lens: `LENS: completeness and invention. Independently spot-check source files against the corpus: pick at least 12 behavior-bearing source files across different partitions (must include AGENTS.md, no-mistakes SKILL.md, two bin/ scripts, one test file), re-derive their behaviors yourself, and diff against the units + provenance-map. Report: behaviors present in source but missing from the corpus (misses), units not actually supported by their citation (invention — verify quote hashes for at least 10 units by recomputing), engine-gap claims whose lower-rung failure argument is wrong (a lower rung actually works), and obsolete-mechanism entries that silently discarded durable policy.`,
  },
]
const reviews = (await parallel(reviewers.map(r => () => agent(`${GROUNDING}

You are an INDEPENDENT reviewer. You did not build this corpus. Grade the actual artifacts under ${OUT} and docs/icm/ against the actual sources under ${SRC} — never a builder summary. Read the deviation register first (L3): re-litigating a settled ruling requires arguing the ruling is wrong.

${r.lens}

Also flag: classification-ledger conflicts whose PROPOSED resolutions are wrong, and uncertainty that was papered over (a unit marked high confidence that should not be). Do not edit any file. Return structured findings; an empty findings list is a legitimate result ONLY if you name what you sampled and found sound.`,
  { label: r.label, phase: 'Refute', model: 'opus', effort: 'high', schema: FINDING_SCHEMA })
))).filter(Boolean)
const allFindings = reviews.flatMap(r => r.findings)
log(`refute: ${allFindings.length} findings (${allFindings.filter(f => f.severity === 'error').length} errors, ${allFindings.filter(f => f.severity === 'major').length} major)`)

return {
  inventory,
  totalUnits,
  synthesis,
  lint,
  findings: allFindings,
  extract_uncertainties: extracts.flatMap(e => e.uncertainties),
  engine_gap_claims: extracts.flatMap(e => e.engine_gap_claims),
  draft_gaps: drafted.flatMap(d => d.gaps),
}
