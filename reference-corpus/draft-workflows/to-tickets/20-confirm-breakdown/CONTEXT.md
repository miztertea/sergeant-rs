# 20-confirm-breakdown: confirm breakdown

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-extract-decisions-and-unknowns/output/README.md | L4 | upstream artifact produced by `10-extract-decisions-and-unknowns` |

## Purpose

Granularity, ownership and blocking edges are confirmed unless immediate publication was requested; new tickets are then published, staying open, with cross-repo blockers recorded as counterpart ids plus merge order.

Trigger (workflow-level): The user says "to tickets", "create issues", "create td tasks", "make epics", or asks to break something into work.

## What must become true here (durable outcome)

Granularity, ownership and blocking edges are confirmed unless immediate publication was requested; new tickets stay open, with cross-repo blockers recorded as counterpart ids plus merge order.

## Behavior contract

- **Unless the user explicitly asked to publish immediately, present the proposed ticket breakdown first and ask only whether granularity, ownership, and blocking edges are correct -- do not re-ask about decisions already made.**
  (trigger: a candidate ticket breakdown has been drafted; outcome: the user confirms structural correctness before tickets are published)
  — `BU-P4-068`, `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Confirm the Breakdown, L100-109)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Helper invocation: publish

Demoted from a standalone stage (`30-publish`) at N1 adjudication A4: its only stage-level justification was the §6.5 deterministic-machinery boilerplate, with no additional checkpoint argument, so it folds into this stage as a helper invocation performed once the breakdown is confirmed. No `kind = "execute"` stage exists in the current engine, so the acting harness performs the publish operation itself:

- **Do not mark newly published tasks in_progress; that transition belongs to dispatch or the worker that later starts the work. New tickets remain open until execution actually begins.**
  (trigger: tickets/epics are being published to td; outcome: ticket status accurately reflects that planning, not execution, has occurred)
  — `BU-P4-070`, `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Publish to td, L155)
- **td dependencies are repository-local; for cross-repository blockers, record the counterpart repo/ticket id and exact merge order in both descriptions/logs rather than inventing a native dependency edge td cannot enforce across separate databases.**
  (trigger: a ticket is blocked by a ticket in a different repository's td database; outcome: cross-repo blocking is tracked honestly as a documented convention, not a fabricated native edge)
  — `BU-P4-071`, `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Publish to td, L149-153)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
