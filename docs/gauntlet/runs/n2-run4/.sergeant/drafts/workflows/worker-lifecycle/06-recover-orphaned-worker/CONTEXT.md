# 06-recover-orphaned-worker

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a worker is observed in the orphaned state

**Outcome:** orphaned is treated as requiring full reconciliation before any recovery action, not a quick retry

**Statement (the operative rule):** `orphaned` means the expected supervisor identity disappeared without a durable waiting state, and the required operator action is to reconcile process, pane, worktree, branch, task, and handoff before recovery.

## What must become true here (durable outcome)

Orphaned is treated as requiring full reconciliation before any recovery action, not a quick retry — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0176`: An expected dependency-blocked exit must remain blocked; it is not an orphan merely because the process ended, so supported response/recovery is used only after reconciling the record, and its worktree is never cleaned.
- `BU-0178`: A missing pane plus durable blocked/handoff state is classified as waiting work; a missing pane from `in_progress` without a handoff is orphan evidence.
- `BU-0306`: If an agent exits before reaching a terminal or waiting state, the supervisor records `orphaned` with durable diagnostics and the task tracker recovery pointers; orphaned work is resumed only through the worker response-delivery step, and its recovery state is never overwritten or discarded.
- `BU-0357`: When a worker exits orphaned, _finish inspects git log above the dispatch base (or, absent that record, commits not on the upstream tracking branch) and records in the diagnostic whether real committed work exists, so the coordinator can distinguish a clean orphan from an interrupted worker with real work needing reconciliation before re-dispatch.
- `BU-0596`: When a repo's recorded worktree is missing or absent and its fleet status is in_progress, the interactive fleet-watch loop reclassifies it to orphaned with a diagnostic directing reconciliation of the preserved branch and handoff, rather than leaving the stale in_progress status in place.
- `BU-0597`: The interactive fleet-watch loop treats a worktree reporting terminal status done with no substantiating .sergeant-result file as an ambiguous terminal condition: it reclassifies both the worktree and fleet status to orphaned with a diagnostic, and hands off via the task-tracker memory step, rather than accepting done at face value.
- `BU-0599`: An in_progress repo with no recorded worker pane is reclassified to orphaned with a diagnostic instructing resume via the worker response-delivery step, rather than continuing to poll a repo that was never given a pane to check.
- `BU-0600`: An in_progress repo whose recorded pane fails supervisor-identity verification (dead, or no longer the expected worker) is reclassified to orphaned with a diagnostic instructing resume via the worker response-delivery step, rather than continuing to treat a foreign or dead pane as the live worker.

