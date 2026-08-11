# 10-extract-decisions-and-unknowns: extract decisions and unknowns

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-load-project-context/output/README.md | L4 | upstream artifact produced by `00-load-project-context` |

## Purpose

An investigation ticket is created only for a genuinely blocking unknown, naming the exact artifact it must produce.

Trigger (workflow-level): The user says "to tickets", "create issues", "create td tasks", "make epics", or asks to break something into work.

## What must become true here (durable outcome)

An investigation ticket is created only for a genuinely blocking unknown, naming the exact artifact it must produce.

## Behavior contract

- **Create a short investigation ticket only when a genuinely blocking unknown cannot be answered from existing evidence, and that ticket must name the exact decision or artifact it is meant to produce.**
  (trigger: an unknown is identified while drafting a ticket breakdown; outcome: investigation tickets are created sparingly and each has a named deliverable)
  — `BU-P4-065`, `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Extract Decisions and Unknowns, L60-62)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
