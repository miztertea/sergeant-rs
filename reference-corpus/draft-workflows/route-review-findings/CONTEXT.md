# Route Review Findings
Draft workflow package — candidate **W16** `route-review-findings` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Turn independent review output into tracked work and a gate.

## Trigger

A review pass (worker-mission's `30-independent-review`, or code-review) has produced findings.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-publish-or-clear-gate` | actor-stage (§6.4, judgment) | The gate is published or cleared only after every finding reached tracked work. |

## Notes for reviewers

**N1 adjudication A4 (finding N1-BH-02).** This package originally decomposed the routing procedure into four stages (`00-parse-and-sanitize`, `10-retain-artifact`, `20-route-each`, `30-publish-or-clear-gate`). The first three carried no argument beyond the §6.5 deterministic-machinery boilerplate, so all three demote by default and fold into `30-publish-or-clear-gate` (renamed `00-publish-or-clear-gate`, now the workflow's sole stage, since it was already the package's only genuine judgment-bearing checkpoint) as helper invocations. The behavior units survive — see `00-publish-or-clear-gate/CONTEXT.md`'s "Helpers (folded per N1 adjudication A4)" section and `provenance.md`.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
