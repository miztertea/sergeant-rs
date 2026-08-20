# Worker Mission (software-change)
Draft workflow package — candidate **W9** `worker-mission` from the N1
manual reference-corpus decomposition (`sergeant-rs-workspace/knowledge/evidence/gauntlet/contracts/N1.md`).
This is Layer 1 orientation only — it is never delivered as a stage's
instructions; each stage's own `CONTEXT.md` (Layer 2) is the actor's
contract (`docs/icm/convention.md` §1a rule 5).

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

## Authority envelope

This workflow receives an already-admitted Work intent (a rendered mission brief from a dispatching supervisor).

### Workflow may decide
- Classifying pinned work into exactly one of the five named triage categories (`10-triage-and-route`).
- Which concrete Work-scoping details to hand the selected discipline (`20-implement`).

### Workflow may not decide
- Whether a straddling work item belongs to a single triage category when none is clearly dominant — J0 (`10-triage-and-route`).
- Whether to narrow independent-review coverage below the brief's own authoritative axis list (`30-independent-review`).
- Whether a repeated blocker counts as a new gate absent a monotonic generation advance (`40-escalate-or-continue`).

### Human or Captain gates
- A straddling triage classification.
- Every escalation the handshake actually reaches.

### Decision record
Material decisions are recorded per-stage in each stage's own output artifact.

## Relationships to other workflows

- `20-implement` delegates to **diagnose-bug, prototype, implement, or deepen-module** (whichever `10-triage-and-route` selected), each dispatched as its own separately-admitted Work — or, when the TDD discipline is selected directly, applies **`@@tdd`**/**`@@test-quality`** in place (`tdd`'s own ICM-R3 REHOME, confirmed). Under the estate-root contract that dispatch is Captain's own submission from the estate root; `20-implement`'s own worker cannot invoke `sgt run` from inside its Work surface (see that stage's `CONTEXT.md` for the engine-gap note).

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

See `sergeant-rs-workspace/knowledge/evidence/gauntlet/promoted-provenance/worker-mission.md` for the complete stage-to-behavior-unit mapping and workflow-level citations. (ICM-R3 correction: the prior text pointed at a workflow-local `provenance.md` that does not exist under `.sergeant/workflows/worker-mission/`.)
