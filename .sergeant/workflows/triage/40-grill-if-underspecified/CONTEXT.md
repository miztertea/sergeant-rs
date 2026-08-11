# 40-grill-if-underspecified: grill if underspecified

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-recommend/output/README.md | L4 | upstream artifact produced by `30-recommend` |

## Purpose

Underspecified items are escalated to an interview.

Trigger (workflow-level): An item is at the front of one of the three fixed attention buckets, oldest first.

## What must become true here (durable outcome)

Underspecified items are escalated to an interview.

## Behavior contract

- **If the item is underspecified after verification, the actor invokes the grilling and domain-modeling procedures together to sharpen it into shape.**
  (trigger: verification shows the request needs fleshing out; outcome: the item's specification and domain terms are sharpened, with decisions captured inline)
  — `BU-P3-068`, `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 76)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Delegation

This stage's outcome is produced by running **grilling** to its own completion (context composition today — see `docs/icm/convention.md` §4 on `@@name` versus true nested-workflow invocation, which does not exist yet).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
