# 06-execute: execute

## Inputs

| File | Layer | Why |
|---|---|---|
| ../05-confirm-decisions/output/README.md | L4 | upstream artifact produced by `05-confirm-decisions` |

## Purpose

Control passes to `direct-implementation` or `dispatch`.

Trigger (workflow-level): Any task the user brings.

## What must become true here (durable outcome)

Control passes to `direct-implementation` or `dispatch`.

## Behavior contract

- **Execute: in direct mode, start the td task and implement through tests, review, and delivery; in dispatch mode, run sgt-dispatch with a repository list or a td id.**
  (trigger: decisions confirmed; outcome: the chosen mode's procedure begins)
  — `BU-P1-031`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L141-143, step 6)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Delegation

This stage's outcome is produced by running **direct-implementation or dispatch (chosen at 03-choose-mode)** to its own completion (context composition today — see `docs/icm/convention.md` §4 on `@@name` versus true nested-workflow invocation, which does not exist yet).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
