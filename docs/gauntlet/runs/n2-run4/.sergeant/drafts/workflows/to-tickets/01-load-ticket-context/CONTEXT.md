# 01-load-ticket-context

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** the project name is not yet known

**Outcome:** the project name is established before further context loading

**Statement (the operative rule):** Run the fleet-listing step if the project name is not already established, as the first step of loading project context.

## What must become true here (durable outcome)

The project name is established before further context loading — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1307`: Run the task-tracker listing step to deduplicate against every status before drafting tickets.
- `BU-1308`: For architecture or codebase questions, use the existing graph-generation tool graph before reading files individually.
- `BU-1309`: Read any referenced issue, PR, specification, ADR, or findings register in full before drafting tickets.
- `BU-1310`: If an owning repository has no task tracker database, it is initialized with the task tracker only after confirming it is a real project repository.

