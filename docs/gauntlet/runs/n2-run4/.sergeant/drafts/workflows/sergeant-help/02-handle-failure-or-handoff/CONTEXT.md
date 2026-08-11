# 02-handle-failure-or-handoff

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-research-and-answer/output/outcome.md | L4 | upstream evidence produced by `research-and-answer` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** one of the four named conditions occurs while answering a help question

**Outcome:** each condition triggers its own fixed required action rather than an ad hoc response

**Statement (the operative rule):** On a missing primary document, the skill reports the expected path and stops before guessing; on a command/docs mismatch it reports the mismatch and trusts tested/released behavior; on a question requiring project ownership it loads `load-project` and runs the project context-resolution step; on a question requiring implementation or fleet mutation it hands off to the owning procedural skill, since help remains read-only.

## What must become true here (durable outcome)

Each condition triggers its own fixed required action rather than an ad hoc response — per the Statement above, which is the operative rule this stage exists to enforce.

