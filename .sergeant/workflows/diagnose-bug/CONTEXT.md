# Diagnose Bug
Draft workflow package — candidate **W20** `diagnose-bug` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Reproduce, isolate, prove, remediate and verify a defect.

## Trigger

"Diagnose"/"debug this", or something reported broken, throwing, failing, slow.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `10-build-feedback-loop` | actor-stage (§6.4, judgment) | A named, already-run, red-capable, deterministic, fast, agent-runnable command exists, or the run stops and asks for access/artifacts. |
| `20-reproduce-and-minimize` | actor-stage (§6.4, judgment) | The loop goes red on the user's exact symptom and every remaining element is load-bearing. |
| `30-hypothesize` | actor-stage (§6.4, judgment) | 3-5 ranked falsifiable hypotheses are shown to the user. |
| `40-instrument` | actor-stage (§6.4, judgment) | One probe per prediction, one variable at a time, tagged logs. |
| `50-fix-with-regression-test` | actor-stage (§6.4, judgment) | A test exists at a correct seam before the fix, or the seam's absence is recorded as the finding. |
| `60-cleanup-and-postmortem` | actor-stage (§6.4, judgment) | Repro gone, test passing, instrumentation removed, hypothesis recorded, architectural hand-off if warranted. |

## Notes for reviewers

Proposal §8.2's "strong low-ambiguity reference workflow" assessment holds — all six stages survive the §6.3 reimplementation test.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
