# 01-list-tasks

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** the task-tracker listing step is invoked

**Outcome:** only initialized, in-scope repos matching the requested status/priority filters are listed

**Statement (the operative rule):** No single record states this behavior directly; derived from this workflow's own workflow-level helpers (listed below) as a summary of what they collectively establish.

## What must become true here (durable outcome)

Only initialized, in-scope repos matching the requested status/priority filters are listed — per the Statement above, which is the operative rule this stage exists to enforce.

## Provenance note

This stage is a justified design inference — see `../provenance.md`. no `stage`-rung record carries `workflow=list-tasks` — only `stage-context`/`helper` members do (a cluster whose checkpoint boundary was never itself classified `stage` rung, per `.sergeant/workflows/repo-to-icm/_config/icm-ladder.md` 6.3's own caution). A single stage is inferred so the runtime has a checkpoint to run at all; its content is the workflow-level helpers themselves, and its trigger/outcome are paraphrased from them since no record states a whole-workflow trigger/outcome directly.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0243`: By default the task-tracker listing step filters to `status=open`; `--all` removes the status filter, and `--priority` ANDs an additional priority filter onto whatever status filter is active.
- `BU-0244`: The task-tracker listing step silently skips a target repo whose resolved path is not an initialized git repository, rather than erroring the whole listing.

