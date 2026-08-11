# 01-resolve-stage-brief

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a DAG stage is defined

**Outcome:** the stage's brief source is one of the two named alternatives, resolved by whether the task tracker is set

**Statement (the operative rule):** Each DAG stage names the repos it dispatches to and pulls its brief from the task tracker task via `td:`, or from an explicit inline `brief:` when `td:` is not set.

## What must become true here (durable outcome)

The stage's brief source is one of the two named alternatives, resolved by whether the task tracker is set — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0867`: A stage's explicit `brief` is only passed through to dispatch when the stage has no task tracker task reference; when both the task tracker and `brief` are given for a stage, the task tracker takes precedence and the explicit brief is dropped.
- `BU-0868`: Every DAG stage is registered with the DAG dispatch hook (not the dispatch step directly) as its DAG runner hook, so stage readiness always routes through the hook that also writes the DAG runner tracking files.

