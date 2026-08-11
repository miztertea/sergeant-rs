# 02-claim-ticket

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |
| ../01-select-wayfinder-mode/output/outcome.md | L4 | upstream evidence produced by `select-wayfinder-mode` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a session is about to start work on a ticket

**Outcome:** two concurrent sessions do not duplicate work on the same ticket

**Statement (the operative rule):** A session claims a ticket by assigning it to the dev driving the map before doing any work on it, so concurrent sessions skip an already-claimed ticket; the assignment itself is the claim, so an open, unassigned ticket is unclaimed.

## What must become true here (durable outcome)

Two concurrent sessions do not duplicate work on the same ticket — per the Statement above, which is the operative rule this stage exists to enforce.

