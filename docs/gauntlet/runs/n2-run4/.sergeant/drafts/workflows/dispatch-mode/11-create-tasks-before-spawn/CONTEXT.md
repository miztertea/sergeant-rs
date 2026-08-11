# 11-create-tasks-before-spawn

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** dispatch is invoked from a freeform brief across multiple repos

**Outcome:** task creation is all-or-nothing across the selected repos, with rollback on partial failure, before any worker is spawned

**Statement (the operative rule):** When dispatching from a freeform brief, the dispatch step creates exactly one task tracker task in each target repo before spawning any worker; if the task tracker is unavailable, task creation fails, generated metadata cannot be injected, or any selected repo does not get a generated task, dispatch aborts before spawning and rolls back the generated cards.

## What must become true here (durable outcome)

Task creation is all-or-nothing across the selected repos, with rollback on partial failure, before any worker is spawned — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0285`: `--td` dispatch keeps using the existing task instead of generating replacement task tracker tasks.
- `BU-0290`: Generated task tracker task results are strictly validated before use — every selected repo must have exactly one task, no repo or task id may repeat, and any malformed or unexpected result triggers a full rollback (deleting every task tracker task already created) with a failure message naming the exact validation error.
- `BU-0297`: Every target repo is validated (cloned, the task tracker initialized) before any task is created, specifically so that a prerequisite failure never requires rolling back an already-created task.
- `BU-0298`: If task creation fails, or the created task's JSON result cannot be parsed into a valid task id, for any target repo, every task tracker task already created in this run is rolled back (deleted) before the command exits with an error.

