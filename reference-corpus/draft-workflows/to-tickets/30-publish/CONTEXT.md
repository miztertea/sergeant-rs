# 30-publish: publish

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-confirm-breakdown/output/README.md | L4 | upstream artifact produced by `20-confirm-breakdown` |

## Purpose

New tickets stay open; cross-repo blockers recorded as counterpart ids plus merge order.

Trigger (workflow-level): The user says "to tickets", "create issues", "create td tasks", "make epics", or asks to break something into work.

## What must become true here (durable outcome)

New tickets stay open; cross-repo blockers recorded as counterpart ids plus merge order.

## Behavior contract

- **Do not mark newly published tasks in_progress; that transition belongs to dispatch or the worker that later starts the work. New tickets remain open until execution actually begins.**
  (trigger: tickets/epics are being published to td; outcome: ticket status accurately reflects that planning, not execution, has occurred)
  — `BU-P4-070`, `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Publish to td, L155)
- **td dependencies are repository-local; for cross-repository blockers, record the counterpart repo/ticket id and exact merge order in both descriptions/logs rather than inventing a native dependency edge td cannot enforce across separate databases.**
  (trigger: a ticket is blocked by a ticket in a different repository's td database; outcome: cross-repo blocking is tracked honestly as a documented convention, not a fabricated native edge)
  — `BU-P4-071`, `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Publish to td, L149-153)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
