# 50-escalate-undocumented: escalate undocumented

## Inputs

| File | Layer | Why |
|---|---|---|
| ../40-escalate-on-second-attempt/output/README.md | L4 | upstream artifact produced by `40-escalate-on-second-attempt` |

## Purpose

An undocumented/unrecognized stall class escalates rather than being guessed at.

Trigger (workflow-level): A worker is `in_progress` with a stall classification recorded by the watcher.

## What must become true here (durable outcome)

An undocumented/unrecognized stall class escalates rather than being guessed at.

## Behavior contract

- **When documentation does not cover an observed failure, the operator should use the sergeant-help skill to search existing docs first, then create a td task containing the exact reproduction, expected behavior, preserved state, and acceptance criteria.**
  (trigger: a failure is not covered by any existing troubleshooting entry; outcome: an undocumented failure always becomes a well-formed, reproducible, trackable task rather than being handled ad hoc and lost)
  — `BU-P8-109`, `reference/sergeant-upstream/docs/troubleshooting.md` (L242-244)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
