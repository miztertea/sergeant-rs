# 20-implement: implement

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-triage-and-route/output/README.md | L4 | upstream artifact produced by `10-triage-and-route` |

## Purpose

The discipline chosen at `10-triage-and-route` runs to its own completion.

Trigger (workflow-level): A worker starts against a rendered brief.

## What must become true here (durable outcome)

The discipline chosen at `10-triage-and-route` runs to its own completion.

## Behavior contract

No behavior units are cited directly against this stage; its content is wholly delegated (see Delegation below) or is the workflow's own structural connective tissue. This is recorded explicitly rather than invented to fill the section.

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Delegation

This stage's outcome is produced by running **diagnose-bug, prototype, tdd, implement, or deepen-module (whichever 10-triage-and-route selected)** to its own completion (context composition today — see `docs/icm/convention.md` §4 on `@@name` versus true nested-workflow invocation, which does not exist yet).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
