# 00-interview-loop: interview loop

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

One question at a time, waiting for each answer.

Trigger (workflow-level): A plan or design needs interview-style stress-testing that should also produce durable domain artifacts.

## What must become true here (durable outcome)

One question at a time, waiting for each answer.

## Behavior contract

No behavior units are cited directly against this stage; its content is wholly delegated (see Delegation below) or is the workflow's own structural connective tissue. This is recorded explicitly rather than invented to fill the section.

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Delegation

This stage's outcome is produced by running **grilling** to its own completion (context composition today — see `docs/icm/convention.md` §4 on `@@name` versus true nested-workflow invocation, which does not exist yet).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
