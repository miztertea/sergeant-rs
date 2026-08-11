export const meta = {
  name: 'n1-adjudication-fixer',
  description: 'Implement the N1 round-1 adjudication rulings (reference-corpus/adjudication-round1.md) across units, packages, and ledger',
  phases: [
    { title: 'Units', detail: 'per-partition: quote/hash re-derivation, rationale/alternatives backfill, normalization, new/split units' },
    { title: 'Packages', detail: 'structural rulings A3-A8 + BH-02 de-staging sweep, by disjoint workflow sets' },
    { title: 'Ledger', detail: 'X1-X20 adjudication statuses, engine-pressure G5/G9' },
    { title: 'Lint', detail: 'lint.py with quote verification, run green' },
    { title: 'Verify', detail: 'Opus residual check on the five error findings' },
  ],
}

const REPO = '/home/user/sergeant-rs'
const OUT = REPO + '/reference-corpus'
const SRC = REPO + '/reference/sergeant-upstream'
const RULINGS = OUT + '/adjudication-round1.md'

const GROUND = `You are a fixer in the N1 adjudication round of sergeant-rs.
Read first: ${RULINGS} (the rulings you implement — binding), then ${REPO}/docs/icm/record-shapes.md §3-§4 (just amended: representation vocabulary, required source.quote field, exact-contiguous-span hash convention, alternatives rules).
Hard rules: never modify anything under ${SRC} (read-only evidence). Never hand-compute a hash — every quote_hash you write or keep MUST be produced/verified by a script you run (python hashlib over the exact contiguous byte span found in the source file). Do not run cargo. Do not git commit. Write only under ${OUT}/ (plus nothing else). Deriving recorded reasoning from existing evidence is legitimate; inventing evidence is not — where evidence cannot support a fix, record the defect honestly instead (confidence: low, citation: disputed, or an explicit note).`

const UNIT_SCHEMA = {
  type: 'object', required: ['units_total', 'hashes_verified', 'citations_disputed', 'rationales_backfilled', 'alternatives_backfilled', 'statements_normalized', 'units_added_or_split', 'notes'],
  properties: {
    units_total: { type: 'number' }, hashes_verified: { type: 'number' }, citations_disputed: { type: 'number' },
    rationales_backfilled: { type: 'number' }, alternatives_backfilled: { type: 'number' },
    statements_normalized: { type: 'number' }, units_added_or_split: { type: 'number' }, notes: { type: 'string' },
  },
}

// ---- Phase 1: per-partition unit fixers (disjoint files, parallel) ----
phase('Units')
const EXTRAS = {
  P1: 'A12: additionally extract the AGENTS.md "Procedural skills" trigger→skill→owns routing table (finding N1-R3-08 cites L111-118, six rows) as new units continuing the BU-P1- id range.',
  P5: 'A12: additionally extract dispatch worker-contract items 6, 16, 17, 18 (finding N1-R3-09: skills/dispatch/SKILL.md lines 175, 185-187) as new units continuing the BU-P5- id range.',
  P6: 'A10: split BU-P6-129 into units its actual quotes support, each confidence-rated on its real evidence; the original four-requirement statement must not survive with confidence: high on an 83-char comment quote. Update BU-P6-082 similarly if its citation is equally weak (finding N1-BH-08).',
  P8: 'A12: split BU-P8-077 into one unit per drain-fleet stage that cites it (00-set-drain and 10-await-convergence aspects), retiring the original id with a note pointing to its successors.',
}
const unitResults = (await parallel(['P1','P2','P3','P4','P5','P6','P7','P8'].map(p => () => agent(`${GROUND}

You own ${OUT}/behavior-units/${p}.ndjson exclusively. Apply, in this order:

1. **A2 (quote + hash, every unit):** write a python script that, for each unit, searches the cited source file for the contiguous span the statement was derived from (start from the locator; the old hash may match some span — try common spans first: the full line(s) at the locator, the paragraph, the sentence). Record "quote" (verbatim, ≤500 chars; longer spans: first 500 chars + "span_bytes") and recompute "quote_hash" as sha256 over the exact span bytes. Script-verify every final hash. Units where no supporting contiguous span exists: keep the best-anchored quote for what IS in the source, set confidence to "low" and add "citation": "disputed" plus a note saying what could not be anchored (per A2 — never delete).
2. **A9:** backfill "rationale" for every unit where it is null (derive from statement + source: why this rung and not the adjacent ones). Backfill non-empty "alternatives_considered" for every unit that carries a workflow or stage boundary (representation workflow|stage), every engine-gap unit, and every unit named in ${OUT}/classification-ledger.md's conflicts. Other stage-context/helper units may keep [] only where no adjacent rung was facially plausible.
3. **A11:** detect statements still naming old-Sergeant mechanisms (tmux, pane, sentinel, specific sgt-* script filenames, callback files, worker windows) where the mechanism is not itself the behavior; rewrite implementation-independently (mechanism nouns → role nouns), preserving id and citation. The mechanism belongs in "notes".
4. ${EXTRAS[p] || 'No partition-specific extras.'}

Keep NDJSON valid (one object per line); keep ids stable except where a ruling splits/retires them. Return counts.`,
  { label: `units:${p}`, phase: 'Units', model: 'sonnet', schema: UNIT_SCHEMA })
))).filter(Boolean)
log(`units: ${unitResults.reduce((n,r)=>n+r.hashes_verified,0)} hashes verified, ${unitResults.reduce((n,r)=>n+r.citations_disputed,0)} disputed, ${unitResults.reduce((n,r)=>n+r.rationales_backfilled,0)} rationales backfilled`)

// ---- Phase 2: structural package fixers (disjoint workflow sets) + ledger (disjoint files) ----
phase('Packages')
const DESTAGE = `**A4 de-staging sweep (finding N1-BH-02), applies to every package in your set:** every stage whose CONTEXT.md justification is only the §6.5 deterministic-machinery boilerplate ("candidate execute-stage workload") is DEMOTED by default: fold it into the adjacent judgment-bearing stage as a helper invocation (the script/operation moves into that stage's context and Inputs; the stage directory is removed; workflow.toml and index.md updated; renumber only when required for order sanity). A stage carrying a real "Additional note" checkpoint argument is judged against §6.3's reimplementation test — keep it (recording the surviving argument in provenance.md) or demote it (recording why the argument failed). Record every keep/demote decision in the package's provenance.md under a "## Adjudication A4" heading. Behavior units are NOT deleted by demotion — their representation stays as extracted; the package simply stops materializing them as stages. Also remove the copy-pasted "read pane as..." reader-note blocks (A11 makes them redundant).`
const PKG_SCHEMA = {
  type: 'object', required: ['packages_touched', 'stages_before', 'stages_after', 'demoted', 'kept_with_argument', 'notes'],
  properties: { packages_touched: { type: 'array', items: { type: 'string' } }, stages_before: { type: 'number' }, stages_after: { type: 'number' }, demoted: { type: 'number' }, kept_with_argument: { type: 'number' }, notes: { type: 'string' } },
}
const pkgSets = [
  {
    label: 'pkg:fleet-core',
    extra: `Specific rulings first: **A3** — in dispatch, move reconciliation before tracked-work creation (reorder/renumber; contexts already say "before new work is created"). **A7** — move monitor-fleet's mutating stages (20-reconcile-terminal, 30-background-*) into reconcile-and-cleanup-fleet; monitor-fleet keeps the read-only outcome; update both purposes + provenance. **A8** — dispatch owns fleet reconciliation; narrow cross-repo-work's 60-reconcile to repo-set-specific completion facts, name dispatch's reconciliation as adjacent owned procedure (no pretend-invocation), and append the composition-pressure evidence to ${OUT}/engine-pressure.md under the existing composition trigger discussion (do not create a new claim).`,
    dirs: ['dispatch', 'cross-repo-work', 'monitor-fleet', 'reconcile-and-cleanup-fleet', 'drain-fleet', 'wake-and-resume', 'recover-stalled-worker', 'respond-to-worker', 'worker-mission'],
  },
  {
    label: 'pkg:ops-intake',
    extra: 'No package-specific rulings beyond the sweep.',
    dirs: ['deliver-external-callback', 'route-review-findings', 'task-intake-and-route', 'load-project', 'project-graph', 'sergeant-setup', 'sergeant-help', 'wiki-digest'],
  },
  {
    label: 'pkg:dev-ship',
    extra: `Specific rulings first: **A5** — validate-and-ship: restore 10-check-scope and 20-do-the-work as stages (rename on id collision, never dissolve), re-rung 40-start-run as an actor stage (its contract carries judgment); redesign the stage list coherently. **A6** — repo-release-verification is demoted from standalone workflow: its behavior becomes a stage/helper candidate inside validate-and-ship per its citation trail; remove the standalone package directory, leaving the re-homing recorded in validate-and-ship/provenance.md and a tombstone note in ${OUT}/provenance-map.md.`,
    dirs: ['validate-and-ship', 'repo-release-verification', 'direct-implementation', 'implement', 'tdd', 'code-review', 'diagnose-bug', 'resolving-merge-conflicts', 'prototype'],
  },
  {
    label: 'pkg:design-support',
    extra: 'No package-specific rulings beyond the sweep.',
    dirs: ['deepen-module', 'research', 'grilling', 'grill-with-docs', 'triage', 'to-spec', 'to-tickets', 'wayfinder', 'vet-external-skill'],
  },
]
const [pkgResults, ledgerResult] = await parallel([
  () => parallel(pkgSets.map(s => () => agent(`${GROUND}

You own exactly these draft-workflow package directories under ${OUT}/draft-workflows/ (touch no others): ${s.dirs.join(', ')}.

${s.extra}

${DESTAGE}

Keep each package structurally valid per ${REPO}/docs/icm/convention.md (workflow.toml order = directory order; actor stages have CONTEXT.md with an Inputs table; index front matter intact; provenance complete). Return counts.`,
    { label: s.label, phase: 'Packages', model: 'sonnet', schema: PKG_SCHEMA }))),
  () => agent(`${GROUND}

You own ${OUT}/classification-ledger.md and ${OUT}/engine-pressure.md exclusively (plus read-only everything else). Implement **A13 and A14** exactly as ruled: X1 and X8 overturned as specified; X3's citation corrected (BU-P1-072's source is README.md — finding N1-BH-12); every X-entry's status moves PROPOSED → ADJUDICATED with a one-line ruling citation to adjudication-round1.md (X2-X7, X9-X20 adopt their proposed resolution; note that two independent reviewers challenged the set and objected only where recorded). In engine-pressure.md: G5 → rejected with A13's reason (its lower rung exists in the shipped engine — verify the claim against ${REPO}/src/domain/work.rs and ${REPO}/src/api.rs needs_input handling and cite what you find); G9 → re-derived on its own evidence with X1's circular ground severed — record the re-derivation and its outcome honestly, whichever way it lands. Also append a "## Round-1 adjudication" section to classification-ledger.md summarizing: 21 findings, dispositions per adjudication-round1.md, residuals owned by the Verify pass.`,
    { label: 'ledger', phase: 'Ledger', model: 'sonnet', schema: { type: 'object', required: ['x_adjudicated', 'g5_outcome', 'g9_outcome', 'notes'], properties: { x_adjudicated: { type: 'number' }, g5_outcome: { type: 'string' }, g9_outcome: { type: 'string' }, notes: { type: 'string' } } } }),
])
const pr = (pkgResults || []).filter(Boolean)
log(`packages: ${pr.reduce((n,r)=>n+r.stages_before,0)} -> ${pr.reduce((n,r)=>n+r.stages_after,0)} stages (${pr.reduce((n,r)=>n+r.demoted,0)} demoted, ${pr.reduce((n,r)=>n+r.kept_with_argument,0)} kept with argument)`)

// ---- Phase 3: lint (A15) ----
phase('Lint')
const lint = await agent(`${GROUND}

Implement **A15's lint artifact**: write ${OUT}/lint.py (python3, stdlib only, exit 0 iff clean) checking: (1) NDJSON validity + all record-shapes §3/§4 required fields including source.quote, with the amended representation vocabulary; (2) unique unit ids; (3) every quote appears contiguously in its cited file AND sha256 of that exact span equals quote_hash (for span_bytes units, verify the 500-char prefix appears and hash the located full span) — citation:disputed units are exempt from the verify but counted; (4) engine-gap units carry the full §6.7 template with canonical field names; (5) package structure: workflow.toml order = stage-directory lexical order, actor stages have CONTEXT.md with an Inputs table whose paths resolve, index.md front matter parses with required fields, no package outside draft-workflows/, provenance.md cites only existing unit ids; (6) every source-inventory "decompose" file appears in provenance-map.md; (7) rationale non-null everywhere; alternatives_considered non-empty for workflow/stage/engine-gap/conflict-named units. Run it; fix mechanical defects it finds (malformed records, broken paths, stale provenance references from the package restructuring — coordinate nothing, everything upstream is done); re-run until exit 0 or only substantive defects remain, which you report without fixing. Return the final tallies.`,
  { label: 'lint', phase: 'Lint', model: 'sonnet', schema: {
    type: 'object', required: ['exit_clean', 'hash_verified', 'hash_disputed', 'mechanical_fixes', 'substantive_defects'],
    properties: { exit_clean: { type: 'boolean' }, hash_verified: { type: 'number' }, hash_disputed: { type: 'number' }, mechanical_fixes: { type: 'array', items: { type: 'string' } }, substantive_defects: { type: 'array', items: { type: 'string' } } },
  } })
log(`lint: clean=${lint ? lint.exit_clean : 'FAILED'} verified=${lint ? lint.hash_verified : '?'} disputed=${lint ? lint.hash_disputed : '?'}`)

// ---- Phase 4: one Opus verifier over the five error findings ----
phase('Verify')
const verify = await agent(`${GROUND.replace('You are a fixer', 'You are an INDEPENDENT verifier — you fixed nothing and grade the actual artifacts')}

The round-1 refute phase raised five ERROR findings; the fixer round claims to have closed them per the rulings. Verify each against the corpus as it now stands (read the actual files; sample deeply; run ${OUT}/lint.py yourself):
1. N1-BH-01 (dispatch ordering) — is reconciliation now genuinely before tracked-work creation, contexts consistent?
2. N1-BH-02 (81 machinery stages) — sample ≥15 formerly-boilerplate stages across ≥6 packages: demoted with the helper preserved, or kept with a real §6.3 argument recorded? Any package where the sweep was skipped?
3. N1-BH-03/R3-05 (rationale/alternatives) — spot-check ≥20 units incl. boundary-bearing ones: are backfilled rationales discriminating (would fail if copy-pasted onto another rung) or generic filler?
4. N1-BH-04 (validate-and-ship) — checkpoints restored, 40-start-run re-rung, package coherent?
5. N1-R3-01/A2 (quotes) — re-verify ≥30 random quote hashes with your own script across ≥5 partitions; confirm disputed units are honestly marked, not laundered.
Also: X1/X8/G5/G9 dispositions match the rulings; no fixer touched ${SRC}; report anything the fixers broke (stale references, orphaned dirs, inconsistent index front matter). Do not edit files. Empty findings are legitimate only with named samples.`,
  { label: 'verify:errors', phase: 'Verify', model: 'opus', effort: 'high', schema: {
    type: 'object', required: ['findings'],
    properties: { findings: { type: 'array', items: { type: 'object', required: ['id', 'severity', 'target', 'claim', 'evidence'], properties: { id: { type: 'string' }, severity: { enum: ['error', 'major', 'minor'] }, target: { type: 'string' }, claim: { type: 'string' }, evidence: { type: 'string' } } } } },
  } })

return { unitResults, pkgResults: pr, ledgerResult, lint, verifyFindings: verify ? verify.findings : null }
