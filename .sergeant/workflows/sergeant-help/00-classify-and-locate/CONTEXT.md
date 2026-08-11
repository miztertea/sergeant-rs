# 00-classify-and-locate: classify and locate

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

The question is bound to one primary document, which is read before any broad search; a missing primary document stops the run with its expected path.

Trigger (workflow-level): The user asks what Sergeant is, how to install/configure/use it, where skills come from, or how to diagnose a Sergeant error.

## What must become true here (durable outcome)

The question is bound to one primary document, which is read before any broad search; a missing primary document stops the run with its expected path.

## Behavior contract

- **Answering a question first classifies it against the documentation map, then reads the primary document before searching broadly.**
  (trigger: a question arrives; outcome: the most authoritative source is consulted before any broad search)
  — `BU-P5-117`, `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (lines 30-31)
- **If the primary document for a question is missing, sergeant-help reports its expected path and stops before guessing.**
  (trigger: the mapped primary document does not exist; outcome: a missing authoritative source is reported, never silently worked around)
  — `BU-P5-126`, `reference/sergeant-upstream/skills/sergeant-help/SKILL.md` (line 71)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
