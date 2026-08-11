# 30-recommend: recommend

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-verify/output/README.md | L4 | upstream artifact produced by `20-verify` |

## Purpose

A category/state proposal is made, then the run waits for direction.

Trigger (workflow-level): An item is at the front of one of the three fixed attention buckets, oldest first.

## What must become true here (durable outcome)

A category/state proposal is made, then the run waits for direction.

## Behavior contract

- **The actor proposes a category/state recommendation with reasoning and a relevant codebase summary, then waits for the maintainer's direction before proceeding.**
  (trigger: context has been gathered; outcome: the maintainer has a recommendation to react to before any state-changing action occurs)
  — `BU-P3-066`, `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 72)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
