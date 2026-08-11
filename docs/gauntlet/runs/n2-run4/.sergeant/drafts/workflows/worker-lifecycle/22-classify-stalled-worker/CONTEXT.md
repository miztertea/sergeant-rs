# 22-classify-stalled-worker

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the interactive fleet-watch loop classifies an in_progress worker as stalled or active

**Outcome:** a worker that is actually producing tool-call or streamed output is never misclassified as stalled merely because progress_ts happens to be older

**Statement (the operative rule):** Stall detection's authoritative liveness signal is live tmux pane-activity output (any terminal output from the agent, including tool calls), taking precedence over the progress_ts file, because a process-tree-based check would incorrectly count the interactive worker's own delivery loop as meaningful activity.

## What must become true here (durable outcome)

A worker that is actually producing tool-call or streamed output is never misclassified as stalled merely because progress_ts happens to be older — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0590`: Clearing a stall diagnostic only removes it when the current diagnostic text is exactly an owned 'live worker stalled:' marker; any other diagnostic (orphan, dispatch-failure, etc.) is left untouched.
- `BU-0592`: Stall classification never changes a repo's status field — it only ever rewrites the diagnostic for an in_progress repo whose pane has already passed live identity verification, so a stalled worker remains resumable rather than being forced into a terminal state.
- `BU-0593`: The interactive fleet-watch loop only rewrites the 'live worker stalled' diagnostic when the stall is newly detected or the elapsed time crosses into a new bucket (default 60s, SERGEANT_STALL_DIAG_BUCKET), rather than on every reconciliation pass, to avoid a full watch redraw on every sync-interval tick.

