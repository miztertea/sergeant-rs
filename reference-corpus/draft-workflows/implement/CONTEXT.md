# Implement
Draft workflow package — candidate **W23** `implement` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Implement a piece of work from a spec or ticket set, explicit-invocation-only.

## Trigger

Explicitly invoked to implement a defined piece of work (never auto-loaded).

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `10-implement-with-tdd` | actor-stage (§6.4, judgment) | Implementation proceeds seam by seam; folds the demoted `20-verify` checkpoint as a helper (N1 adjudication A4). |
| `30-review` | actor-stage (§6.4, judgment) | The change is reviewed; folds the demoted `40-commit` checkpoint as a helper (N1 adjudication A4). |

`20-verify` and `40-commit` were demoted per N1 adjudication A4 (finding N1-BH-02): both were classified at extraction as deterministic machinery (ladder §6.5) with no checkpoint argument beyond the boilerplate. Their behavior units survive, folded into the adjacent judgment-bearing stage as helper invocations — see each stage's own `CONTEXT.md` and `provenance.md`'s "Adjudication A4" section.

## Relationships to other workflows

- `10-implement-with-tdd` delegates to **tdd**.
- `30-review` delegates to **code-review**.

## Notes for reviewers

Explicit-invocation-only (BU-P2-051) — this workflow must never be auto-loaded merely because the task looks like implementation; its cross-harness mirror is BU-P3-004.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
