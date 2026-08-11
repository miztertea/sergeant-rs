# 04-run-dispatch-hook

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |
| ../03-verify-dag-prerequisites/output/outcome.md | L4 | upstream evidence produced by `verify-dag-prerequisites` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** the DAG runner calls the hook without DAGR_RUN_ID set

**Outcome:** the hook fails loudly rather than dispatching work it cannot later attribute to the DAG runner run

**Statement (the operative rule):** The DAG dispatch hook refuses to proceed (exits 1) if DAGR_RUN_ID is not set in its environment.

## What must become true here (durable outcome)

The hook fails loudly rather than dispatching work it cannot later attribute to the DAG runner run — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0872`: The DAG dispatch hook refuses to proceed (exits 1) if DAGR_STAGE_ID is not set in its environment.
- `BU-0873`: If the fleet task ID cannot be parsed out of the dispatch step's output, the hook logs a warning and exits 0 (success) rather than failing the DAG runner stage — dispatch has already happened by this point and is not undone.
- `BU-0874`: Once a fleet task ID is known, the hook writes the DAG runner run ID and stage ID into every one of that task's dispatched-repo directories, so the interactive fleet-watch loop can later read them back to auto-advance the DAG when the task completes.
- `BU-0875`: Dagr tracking files are only written if the fleet task's state directory actually exists on disk; if it does not, the hook silently skips writing tracking files (and still exits successfully).
- `BU-0876`: The hook's final stdout output is exactly the fleet task ID, which the DAG runner records as the hook's dispatch_id for the stage.

