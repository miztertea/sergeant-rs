# 04-validate: validate

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/standing-constraints.md | L3 | constraints binding every stage of this workflow |
| ../03-claim-and-implement/output/README.md | L4 | upstream artifact produced by `03-claim-and-implement` |

## Purpose

The change is validated against native project checks.

Trigger (workflow-level): The user explicitly asks to work in this session, and one repository owns the complete outcome.

## What must become true here (durable outcome)

The change is validated against native project checks.

## Behavior contract

- **In direct mode, run repository-native validation, independent reviews, and the final shipping gate exactly as a dispatched worker would.**
  (trigger: implementation complete; outcome: direct-mode changes pass the same gates a dispatched worker's changes would)
  — `BU-P1-012`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L32-33, direct-mode validation step)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
