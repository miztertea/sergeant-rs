# 01-plan-and-decompose

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** dispatch mode has been selected

**Outcome:** work is decomposed per-repository prior to dispatch

**Statement (the operative rule):** In dispatch mode, the coordinator loads context, plans, and decomposes the work by repository before dispatching.

## What must become true here (durable outcome)

Work is decomposed per-repository prior to dispatch — per the Statement above, which is the operative rule this stage exists to enforce.

