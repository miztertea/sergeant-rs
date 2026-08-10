# 40-report-frontier: report frontier

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-confirm-breakdown/output/README.md | L4 | upstream artifact produced by `20-confirm-breakdown` (absorbed the demoted `30-publish` stage — N1 adjudication A4) |

## Purpose

One worker per owning repo is the default; reporting is not authorization to dispatch.

Trigger (workflow-level): The user says "to tickets", "create issues", "create td tasks", "make epics", or asks to break something into work.

## What must become true here (durable outcome)

One worker per owning repo is the default; reporting is not authorization to dispatch.

## Behavior contract

- **When reporting the dispatch frontier, recommend one worker per owning repository as the default concurrency, unless the project explicitly supports more.**
  (trigger: the dispatch frontier is being reported after publication; outcome: a sensible default concurrency is recommended alongside the frontier)
  — `BU-P4-072`, `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Report the Dispatch Frontier, L181-182)
- **Do not actually dispatch any ticket unless the user asked to begin implementation; reporting the frontier is not itself authorization to start work.**
  (trigger: the dispatch frontier and next commands have been reported; outcome: publication and reporting never silently trigger execution)
  — `BU-P4-073`, `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Report the Dispatch Frontier, L189)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
