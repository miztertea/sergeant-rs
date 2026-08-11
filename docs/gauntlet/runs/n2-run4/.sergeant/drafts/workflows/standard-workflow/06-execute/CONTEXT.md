# 06-execute

## Inputs

| File | Layer | Why |
|---|---|---|
| ../05-confirm-with-user/output/outcome.md | L4 | upstream evidence produced by `confirm-with-user` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** decisions have been confirmed

**Outcome:** execution proceeds via the mode-appropriate path

**Statement (the operative rule):** Step 6 of the standard workflow (execute): in direct mode, start the task tracker task and implement through tests, review, and delivery; in dispatch mode, use the dispatch step.

## What must become true here (durable outcome)

Execution proceeds via the mode-appropriate path — per the Statement above, which is the operative rule this stage exists to enforce.

