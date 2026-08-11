# 00-name-destination: name destination

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

The destination is named via a grilling/domain-modeling session; scope is settled first.

Trigger (workflow-level): A destination is named that requires mapping fog before it can be reached.

## What must become true here (durable outcome)

The destination is named via a grilling/domain-modeling session; scope is settled first.

## Behavior contract

- **Charting a wayfinder map first names the destination via a grilling/domain-modeling session (settling scope first), then maps the frontier breadth-first across the whole space rather than deep on one thread.**
  (trigger: a loose, oversized idea is presented to be charted; outcome: the destination is fixed before any tickets are drafted)
  — `BU-P4-094`, `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Invocation / Chart the map, L111)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Delegation

This stage's outcome is produced by running **grilling** to its own completion (context composition today — see `docs/icm/convention.md` §4 on `@@name` versus true nested-workflow invocation, which does not exist yet).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
