# 01-list-fleet-status

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** the fleet-watch loop's listing view is invoked

**Outcome:** the listing reports an accurate per-status repo breakdown without overstating which tasks are currently active

**Statement (the operative rule):** No single record states this behavior directly; derived from this workflow's own workflow-level helpers (listed below) as a summary of what they collectively establish.

## What must become true here (durable outcome)

The listing reports an accurate per-status repo breakdown without overstating which tasks are currently active — per the Statement above, which is the operative rule this stage exists to enforce.

## Provenance note

This stage is a justified design inference — see `../provenance.md`. no `stage`-rung record carries `workflow=fleet-status-listing` — only `stage-context`/`helper` members do (a cluster whose checkpoint boundary was never itself classified `stage` rung, per `.sergeant/workflows/repo-to-icm/_config/icm-ladder.md` 6.3's own caution). A single stage is inferred so the runtime has a checkpoint to run at all; its content is the workflow-level helpers themselves, and its trigger/outcome are paraphrased from them since no record states a whole-workflow trigger/outcome directly.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0576`: The interactive fleet-watch loop --list explicitly avoids claiming its retained task records (including terminal ones) are currently active, directing a caller who needs an activity determination to --snapshot instead.
- `BU-0606`: The interactive fleet-watch loop --list reports, per task, a full breakdown of repo counts across every recognized status (done, in-progress, needs-input, blocked, waiting, drained, orphaned, failed), not merely a single aggregate count.

