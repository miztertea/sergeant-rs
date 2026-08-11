export const meta = {
  name: 'n1-closure',
  description: 'Close the 13 verifier residuals (V1-V13) from the N1 fixer round, then re-lint',
  phases: [
    { title: 'Close', detail: 'units (2 agents), packages (1), corpus docs (1)' },
    { title: 'Relint', detail: 'remove carve-out, run lint.py green' },
  ],
}

const REPO = '/home/user/sergeant-rs'
const OUT = REPO + '/reference-corpus'
const GROUND = `You are closing enumerated verifier residuals in sergeant-rs's N1 reference corpus. The verifier's findings are precise — implement them, do not re-litigate them. Read ${OUT}/adjudication-round1.md for the underlying rulings when a finding cites one. Hard rules: never modify reference/sergeant-upstream; never hand-compute a hash (script only); do not run cargo; do not git commit; write only inside ${OUT}/. When a finding's fix requires a judgment call, make it, and record it where the finding says the record belongs.`
const S = { type: 'object', required: ['closed', 'notes'], properties: { closed: { type: 'array', items: { type: 'string' } }, notes: { type: 'string' } } }

phase('Close')
const [uA, uB, pk, docs] = await parallel([
  () => agent(`${GROUND}
You own ${OUT}/behavior-units/P1.ndjson-P4.ndjson exclusively. Close, in these files only:
- V11 (partial): of the 104 boundary-bearing units whose alternatives_considered entries are bare rung names, fix every one living in P1-P4: each listed alternative gains a short specific rejection reason ("stage: no independent durable checkpoint — outcome folds into X" style), grounded in that unit's content. Find them mechanically (alternatives entries that are bare rung tokens with no explanatory text).
Return the ids you touched.`, { label: 'close:units-P1-4', phase: 'Close', model: 'sonnet', schema: S }),

  () => agent(`${GROUND}
You own ${OUT}/behavior-units/P5.ndjson-P8.ndjson exclusively. Close, in these files only:
- V2: apply ruling A13/X8 to BU-P5-025 itself — reclassify off the engine-gap rung to the representation X8's adjudicated resolution implies (BU-P4-053's "representable today via needs_input" reading won; so stage-context of its owning workflow), engine_gap -> null, with a note citing adjudication-round1.md A13.
- V4 (unit side): BU-P8-077 — confidence drops from high (its own notes admit the split), representation reflects retirement (add "retired": true style note is already there; ensure successors BU-P8-110/111 are the operative units and P8-077's note names them as such).
- V5: BU-P8-110/111 declare stages that the A4 sweep deleted (drain-fleet now has one stage). Re-anchor them to drain-fleet's current structure: representation stage-context of the surviving stage (the demoted checkpoints' behavior lives in its context now), with notes recording the original intended checkpoints.
- V8: the verifier found re-anchored citations whose span verifies but does not support the statement's claim (named: BU-P8-057, BU-P8-009). Audit those two plus every P5-P8 unit whose quote was re-anchored to a different span than the original hash implied; where the span does not carry the statement's claim, mark confidence: low + "citation": "disputed" with a note — per A2, honesty over laundering.
- V11 (partial): same bare-alternatives fix as described for P1-P4, applied to P5-P8's share of the 104.
Return the ids you touched per finding.`, { label: 'close:units-P5-8', phase: 'Close', model: 'sonnet', schema: S }),

  () => agent(`${GROUND}
You own ${OUT}/draft-workflows/ exclusively. Close:
- V1: apply the A4 sweep to monitor-fleet — its two surviving stages are boilerplate-only machinery. Judgment (make it): a read-only observation procedure whose steps are all mechanical collapses to ONE actor stage ("observe and interpret fleet state") whose context invokes the snapshot/watch operations as helpers and whose judgment is the interpretation/escalation decision; record the disposition under "## Adjudication A4" in its provenance.md, bump version.
- V3 (package side): route the 13 new A12 units (BU-P1-132..137, BU-P5-150..153, BU-P8-110..111, and the P6 split results) into the provenance.md of the packages their content belongs to (routing-table units -> the workflows they route to or task-intake-and-route; dispatch worker-contract items -> dispatch/worker-mission; drain units -> drain-fleet).
- V4 (package side): drain-fleet/provenance.md cites the successors, not retired BU-P8-077.
- V6: bump version on the 9 restructured packages that did not bump (cross-repo-work, dispatch, drain-fleet, monitor-fleet, reconcile-and-cleanup-fleet, +the others the verifier lists — detect: stage sequence changed in the A4/A5/A7 sweep but version still 1).
- V7: prune validate-and-ship/30-start-run's contract of BU-P2-059/060/061 (they belong to the restored checkpoints); one unit = binding contract in exactly one stage of a workflow.
- V12: fix the self-contradicting sentence in dispatch's A4 adjudication note.
Return per-finding closures.`, { label: 'close:packages', phase: 'Close', model: 'sonnet', schema: S }),

  () => agent(`${GROUND}
You own the corpus-level documents exclusively: provenance-map.md, synthesis.md, classification-ledger.md, helper-map.md, shared-context-map.md, engine-pressure.md (read-only: everything else). Close:
- V3 (map side): provenance-map.md's four affected source rows gain the 13 new A12 unit ids and updated counts.
- V9: correct every stale 966 total to 979 (and any derived census figures), with a one-line "round-1 adjudication added/split units" note where the number appears as a verification claim.
- V10: add a superseding header note to synthesis.md and to the stale helper-map.md/shared-context-map.md entries: stage vocabulary as of synthesis is historical; each package's provenance.md + workflow.toml are authoritative post-adjudication; name adjudication-round1.md as the bridge. Do not rewrite synthesis content (it is a phase artifact).
Return per-finding closures.`, { label: 'close:docs', phase: 'Close', model: 'sonnet', schema: S }),
])

phase('Relint')
const lint = await agent(`${GROUND}
Run ${OUT}/lint.py. First REMOVE the carve-out the previous round added for BU-P8-077 (V13/V4 — the accommodation is now unnecessary; if it is not, that is a defect to report, not re-accommodate). Fix any mechanical fallout from the closure round (stale provenance ids, broken Inputs paths, version/front-matter inconsistencies) and re-run to exit 0, or report substantive residuals without fixing. Then verify the three error findings are closed with concrete evidence: (V1) monitor-fleet has no boilerplate-only stage and carries an A4 disposition; (V2) BU-P5-025 is no longer representation engine-gap; (V3) all 13 new unit ids each appear in >=1 package provenance.md and in provenance-map.md. Return the evidence.`,
  { label: 'relint', phase: 'Relint', model: 'sonnet', schema: {
    type: 'object', required: ['exit_clean', 'carveout_removed', 'v1_closed', 'v2_closed', 'v3_closed', 'notes'],
    properties: { exit_clean: { type: 'boolean' }, carveout_removed: { type: 'boolean' }, v1_closed: { type: 'boolean' }, v2_closed: { type: 'boolean' }, v3_closed: { type: 'boolean' }, notes: { type: 'string' } },
  } })

return { uA, uB, pk, docs, lint }
