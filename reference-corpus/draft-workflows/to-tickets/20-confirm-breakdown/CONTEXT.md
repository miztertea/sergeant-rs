# 20-confirm-breakdown: confirm breakdown

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-extract-decisions-and-unknowns/output/README.md | L4 | upstream artifact produced by `10-extract-decisions-and-unknowns` |

## Purpose

Granularity, ownership and blocking edges are confirmed unless immediate publication was requested.

Trigger (workflow-level): The user says "to tickets", "create issues", "create td tasks", "make epics", or asks to break something into work.

## What must become true here (durable outcome)

Granularity, ownership and blocking edges are confirmed unless immediate publication was requested.

## Behavior contract

- **Unless the user explicitly asked to publish immediately, present the proposed ticket breakdown first and ask only whether granularity, ownership, and blocking edges are correct -- do not re-ask about decisions already made.**
  (trigger: a candidate ticket breakdown has been drafted; outcome: the user confirms structural correctness before tickets are published)
  — `BU-P4-068`, `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Confirm the Breakdown, L100-109)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
