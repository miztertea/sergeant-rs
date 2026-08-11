# 09-prepare-worker-brief

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the dispatch step is invoked with --td

**Outcome:** the worker's brief carries the full task tracker lifecycle instructions rather than a freeform mission with no task tracking

**Statement (the operative rule):** When dispatch is done from an existing task tracker task, the brief, branch name, and full task context are pulled from the task tracker automatically, and the worker's brief includes the task tracker's start, log, handoff, and review instructions so the task lifecycle is tracked end-to-end.

## What must become true here (durable outcome)

The worker's brief carries the full task tracker lifecycle instructions rather than a freeform mission with no task tracking — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0279`: A `--deps` ordering constraint causes the brief written into each dependent repo to include an instruction to wait for the prerequisite's `.sergeant-status` to read `done` before opening a PR; the workers themselves are responsible for honoring this, and the brief makes it explicit.

