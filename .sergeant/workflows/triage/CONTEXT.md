# Triage
Draft workflow package — candidate **W30** `triage` from the N1
manual reference-corpus decomposition (`sergeant-rs-workspace/knowledge/evidence/gauntlet/contracts/N1.md`).
This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Work through the attention queue: gather context, verify claims, recommend a disposition, and apply the terminal outcome with its required artifact.

## Trigger

An item is at the front of one of the three fixed attention buckets, oldest first.

## Authority envelope

This workflow reasons about issues/PRs and drafts artifacts (recommendations,
briefs, triage notes, closing comments, KB records) but never takes an
externally-visible action (posting, closing, writing to the KB) without the
maintainer direction gated at `20-recommend`'s J0 clause. See each stage's own
`## Bounded judgment` section for its specific J5/J2/J1/J0 breakdown.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `10-gather-context` | actor-stage (§6.4, judgment) | Three fixed buckets, oldest first; the item and prior notes are read; an already-implemented check and out-of-scope-KB concept match are run. |
| `20-recommend` | actor-stage (§6.4, judgment) | A category/state proposal is made, then the run waits for direction. |
| `30-verify` | actor-stage (§6.4, judgment) | The claim is reproduced or the PR diff is tested, reported as confirmed/failed/insufficient. |
| `40-grill-if-underspecified` | actor-stage (§6.4, judgment) | Underspecified items are escalated to an interview. |
| `50-apply-outcome` | actor-stage (§6.4, judgment) | The terminal disposition is applied with its required artifact. |

**Revised at ICM-R3** (`sergeant-rs-workspace/knowledge/evidence/gauntlet/runs/icm-r3/triage/adjudication-draft.md`):
`recommend` and `verify` were renumbered `20`/`30` (previously
`30`/`20`) — verify's own trigger text ("a recommendation has been given and
direction received") and the upstream source's own line order both place
verification after recommendation, not before. No behavior unit's content or
placement rung changed.

## Relationships to other workflows

- `40-grill-if-underspecified` delegates to **grilling**.

## Notes for reviewers

`resume` and `quick-override` are documented re-entry variants of this same stage sequence, not separate stage directories. The state machine's transition graph is explicitly non-linear (loops, maintainer override at any point) — the source extractor considered and rejected an engine-gap claim for it, and that rejection is upheld here: each transition is a fresh invocation of a stage, not a control-flow construct the runtime must own.

**N1 adjudication A4:** the former `00-show-attention` stage carried only the §6.5 deterministic-machinery boilerplate as its stage-level justification, with no additional checkpoint argument; it is demoted and folded into `10-gather-context` as a helper invocation. Stage ordinals are unchanged (`10`-`50` are already correctly ordered without `00`) — see `sergeant-rs-workspace/knowledge/evidence/gauntlet/promoted-provenance/triage.md`'s "Adjudication A4" section.

## Provenance

See `sergeant-rs-workspace/knowledge/evidence/gauntlet/promoted-provenance/triage.md` for the complete
stage-to-behavior-unit mapping and workflow-level citations.
