# 03-monitor-and-reconcile

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** workers have been dispatched

**Outcome:** merge order, PR state, and cross-repo implications are reconciled by the coordinator

**Statement (the operative rule):** In dispatch mode, the coordinator monitors worker progress and reconciles merge order, PRs, and cross-repo implications.

## What must become true here (durable outcome)

Merge order, PR state, and cross-repo implications are reconciled by the coordinator — per the Statement above, which is the operative rule this stage exists to enforce.

