# 04-recommend-and-wait

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the triage skill has presented its recommendation to the maintainer

**Outcome:** no further triage action is taken until the maintainer responds

**Statement (the operative rule):** After presenting the category/state recommendation and a codebase summary (including whether the request is already implemented), the triage skill waits for the maintainer's direction before proceeding further.

## What must become true here (durable outcome)

No further triage action is taken until the maintainer responds — per the Statement above, which is the operative rule this stage exists to enforce.

