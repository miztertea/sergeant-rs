# 30-review: review

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-implement-with-tdd/output/README.md | L4 | upstream artifact produced by `10-implement-with-tdd` (folds the demoted `20-verify` checkpoint, N1 adjudication A4) |

## Purpose

The change is reviewed.

Trigger (workflow-level): Explicitly invoked to implement a defined piece of work (never auto-loaded).

## What must become true here (durable outcome)

The change is reviewed.

## Behavior contract

No behavior units are cited directly against this stage; its content is wholly delegated (see Delegation below) or is the workflow's own structural connective tissue. This is recorded explicitly rather than invented to fill the section.

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Delegation

This stage's outcome is produced by running **code-review** to its own completion (context composition today — see `docs/icm/convention.md` §4 on `@@name` versus true nested-workflow invocation, which does not exist yet).

## Helper: commit (folded from demoted `40-commit`, N1 adjudication A4)

`40-commit` was classified at extraction as deterministic machinery (ladder §6.5) with no checkpoint argument beyond the boilerplate; per adjudication A4 it is demoted and its behavior folded here as the concluding helper invocation of this checkpoint, subordinate to this stage's own judgment-bearing outcome:

- **The final step of implement is to commit the work to the current branch.**
  (trigger: the work has been implemented, verified, and reviewed; outcome: the change is committed to the current branch)
  — `BU-P2-055`, `reference/sergeant-upstream/.agents/skills/implement/SKILL.md` (body, lines 15-15)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
