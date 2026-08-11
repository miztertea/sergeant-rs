# 03-commit-implementation

## Inputs

| File | Layer | Why |
|---|---|---|
| ../02-review-implementation/output/outcome.md | L4 | upstream evidence produced by `review-implementation` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** the work has been reviewed

**Outcome:** the work is durably recorded in version control

**Statement (the operative rule):** Completed, reviewed implementation work is committed to the current branch.

## What must become true here (durable outcome)

The work is durably recorded in version control — per the Statement above, which is the operative rule this stage exists to enforce.

