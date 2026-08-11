# 04-reconcile-existing-state

## Inputs

| File | Layer | Why |
|---|---|---|
| ../03-select-execution-mode/output/outcome.md | L4 | upstream evidence produced by `select-execution-mode` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** an execution mode has been chosen, before starting work

**Outcome:** existing state is reconciled and reused rather than duplicated

**Statement (the operative rule):** Step 4 of the standard workflow: run the interactive fleet-watch loop, then inspect active workers, branches, worktrees, retained gates, and handoffs before starting; resume or take over preserved work rather than creating duplicates.

## What must become true here (durable outcome)

Existing state is reconciled and reused rather than duplicated — per the Statement above, which is the operative rule this stage exists to enforce.

