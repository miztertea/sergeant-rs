# 02-load-or-create-task

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-load-context/output/outcome.md | L4 | upstream evidence produced by `load-context` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** context has been loaded

**Outcome:** an existing canonical task tracker task is reused when one exists; a new one is created only otherwise

**Statement (the operative rule):** Step 2 of the standard workflow: run the task-tracker listing step and reuse a matching task in direct or dispatch mode; create a task only when no canonical task exists.

## What must become true here (durable outcome)

An existing canonical task tracker task is reused when one exists; a new one is created only otherwise — per the Statement above, which is the operative rule this stage exists to enforce.

