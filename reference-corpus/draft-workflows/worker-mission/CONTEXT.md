# Worker Mission (software-change)
Draft workflow package — candidate **W9** `worker-mission` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

From a rendered brief, produce a merged-ready change with evidence — the contract a dispatched worker delivers against.

## Trigger

A worker starts against a rendered brief.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `10-triage-and-route` | actor-stage (§6.4, judgment) | Pin scope runs first (folded helper); full originating context read, redundant work checked, and the work classified into one of five categories, each loading a different canonical procedure. |
| `20-implement` | actor-stage (§6.4, judgment) | The discipline chosen at `10-triage-and-route` runs to its own completion. |
| `30-independent-review` | actor-stage (§6.4, judgment) | Every axis named in the brief's authoritative list runs as a separate, non-contaminating parallel review; outputs unblended. |
| `40-escalate-or-continue` | actor-stage (§6.4, judgment) | A new gate is published only when a monotonic generation actually advanced; the handshake is acknowledged, accepted, acted on once, and marked complete. Publish result runs after, on the concluding path (folded helper). |

## Relationships to other workflows

- `20-implement` delegates to **diagnose-bug, prototype, tdd, implement, or deepen-module (whichever 10-triage-and-route selected)**.

## Adjudication note (A4)

N1 adjudication A4 (BH-02) applied the generic de-staging sweep:
`00-pin-scope` and `50-publish-result` were the package's only stages
extracted at ladder §6.5 with no "Additional note" arguing otherwise.
`00-pin-scope` folded forward into `10-triage-and-route` (the next stage);
`50-publish-result` folded backward into `40-escalate-or-continue` (the
last stage, with nothing after it to fold forward into). Stage count
dropped from 6 to 4; no behavior unit was deleted — see `provenance.md`'s
"Adjudication A4" section and the two receiving stages' "Helper
invocations" sections.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
