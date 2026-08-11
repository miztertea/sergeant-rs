# 08-deliver-and-cleanup

## Inputs

| File | Layer | Why |
|---|---|---|
| ../07-resolve-blocking-gate/output/outcome.md | L4 | upstream evidence produced by `resolve-blocking-gate` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** work has reached a terminal or deliverable state

**Outcome:** cleanup runs only after terminal state and evidence preservation are verified

**Statement (the operative rule):** Step 9 of the standard workflow: surface PRs and merge order, complete approved merges/deployments, and run the fleet cleanup step only after terminal state and preserved evidence are verified.

## What must become true here (durable outcome)

Cleanup runs only after terminal state and evidence preservation are verified — per the Statement above, which is the operative rule this stage exists to enforce.

