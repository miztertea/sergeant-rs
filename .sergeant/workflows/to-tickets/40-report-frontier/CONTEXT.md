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

## Bounded judgment

Apply `@@bounded-judgment`.

### J5 — governing constraint
- Never dispatch any ticket unless the user asked to begin implementation — reporting the frontier is never itself authorization (`BU-P4-073`).

### J2 — delegated to this stage
- What counts as "the project explicitly supports more" concurrency (`BU-P4-072`).

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only once the dispatch frontier is reported with a default concurrency recommendation, without dispatching anything.

### Decision evidence
The reported frontier is this stage's own durable output, recorded per `output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
