# 02-dispatch-one-worker-per-repo

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** dispatch mode has been selected and work has been decomposed by repository

**Outcome:** each owning repository receives one dispatched worker via the dispatch step

**Statement (the operative rule):** In dispatch mode, the coordinator dispatches exactly one worker per owning repository, using the dispatch step.

## What must become true here (durable outcome)

Each owning repository receives one dispatched worker via the dispatch step — per the Statement above, which is the operative rule this stage exists to enforce.

