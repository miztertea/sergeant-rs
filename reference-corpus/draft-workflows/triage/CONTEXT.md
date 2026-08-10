# Triage
Draft workflow package — candidate **W30** `triage` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Work through the attention queue: gather context, verify claims, recommend a disposition, and apply the terminal outcome with its required artifact.

## Trigger

An item is at the front of one of the three fixed attention buckets, oldest first.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-show-attention` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Three fixed buckets, oldest first. |
| `10-gather-context` | actor-stage (§6.4, judgment) | The item and prior notes are read; an already-implemented check and out-of-scope-KB concept match are run. |
| `20-verify` | actor-stage (§6.4, judgment) | The claim is reproduced or the PR diff is tested, reported as confirmed/failed/insufficient. |
| `30-recommend` | actor-stage (§6.4, judgment) | A category/state proposal is made, then the run waits for direction. |
| `40-grill-if-underspecified` | actor-stage (§6.4, judgment) | Underspecified items are escalated to an interview. |
| `50-apply-outcome` | actor-stage (§6.4, judgment) | The terminal disposition is applied with its required artifact. |

## Relationships to other workflows

- `40-grill-if-underspecified` delegates to **grilling**.

## Notes for reviewers

`resume` and `quick-override` (BU-P3-075, BU-P3-073) are documented re-entry variants of this same stage sequence, not separate stage directories. BU-P3-060's transition graph is explicitly non-linear (loops, maintainer override at any point) — the source extractor considered and rejected an engine-gap claim for it, and that rejection is upheld here: each transition is a fresh invocation of a stage, not a control-flow construct the runtime must own.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
