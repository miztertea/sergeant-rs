# 10-implement-with-tdd: implement with tdd

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Implementation proceeds seam by seam.

Trigger (workflow-level): Explicitly invoked to implement a defined piece of work (never auto-loaded).

## What must become true here (durable outcome)

Implementation proceeds seam by seam.

## Behavior contract

No behavior units are cited directly against this stage; its content is wholly delegated (see Delegation below) or is the workflow's own structural connective tissue. This is recorded explicitly rather than invented to fill the section.

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Delegation

This stage's outcome is produced by running **tdd** to its own completion (context composition today — see `docs/icm/convention.md` §4 on `@@name` versus true nested-workflow invocation, which does not exist yet).

## Helper: verify (folded from demoted `20-verify`, N1 adjudication A4)

`20-verify` was classified at extraction as deterministic machinery (ladder §6.5) with no checkpoint argument beyond the boilerplate; per adjudication A4 it is demoted and its behavior folded here as a helper invoked while implementation is underway, subordinate to this stage's own judgment-bearing outcome:

- **During implementation, typechecking and single test files should be run regularly, with the full test suite run once at the end.**
  (trigger: implementation work is underway; outcome: fast, frequent local checks are interleaved with work, with one full-suite pass at the close)
  — `BU-P2-053`, `reference/sergeant-upstream/.agents/skills/implement/SKILL.md` (body, lines 11-11)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
