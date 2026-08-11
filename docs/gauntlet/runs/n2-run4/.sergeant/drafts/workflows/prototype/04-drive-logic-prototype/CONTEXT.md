# 04-drive-logic-prototype

## Inputs

| File | Layer | Why |
|---|---|---|
| ../03-build-logic-prototype/output/outcome.md | L4 | upstream evidence produced by `build-logic-prototype` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** the logic prototype has been handed over to the user

**Outcome:** user surprises are treated as the prototype's real output, and the prototype is extended on request rather than treated as frozen

**Statement (the operative rule):** Once handed over with its run command, the logic prototype is driven by the user; moments where the user says something shouldn't be possible or assumed something would be different are treated as the real bugs being sought, and requested new actions are added since prototypes evolve.

## What must become true here (durable outcome)

User surprises are treated as the prototype's real output, and the prototype is extended on request rather than treated as frozen — per the Statement above, which is the operative rule this stage exists to enforce.

