# 03-select-execution-mode

## Inputs

| File | Layer | Why |
|---|---|---|
| ../02-load-or-create-task/output/outcome.md | L4 | upstream evidence produced by `load-or-create-task` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** the task queue has been checked

**Outcome:** an execution mode is chosen according to this rule

**Statement (the operative rule):** Step 3 of the standard workflow: choose direct mode for explicit single-repo work in this session, dispatch mode for cross-repo, parallel, or explicitly delegated work.

## What must become true here (durable outcome)

An execution mode is chosen according to this rule — per the Statement above, which is the operative rule this stage exists to enforce.

