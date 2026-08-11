# 05-stalled-worker-recovery

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** recovery is being considered for a stalled worker

**Outcome:** recovery is gated on having already reconciled identity/worktree/handoff/notification evidence, and only applies to the one named diagnostic

**Statement (the operative rule):** The stalled-worker recovery step is used only after reconciling the exact pane identity, worktree, the task tracker handoff, and response/notification state, and only for the exact `live worker stalled` case.

## What must become true here (durable outcome)

Recovery is gated on having already reconciled identity/worktree/handoff/notification evidence, and only applies to the one named diagnostic — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0159`: The stalled-worker recovery step is one-shot per repo attempt: Sergeant records `stall_recovery_attempted`, relaunches only after replacement metadata is validated, and escalates to `needs_input` instead of retrying when the prior notification delivery still holds an unfinished action lease, the recorded pane identity no longer matches, or any later relaunch step fails.
- `BU-0175`: If Sergeant refuses recovery because pane identity or unfinished notification delivery evidence no longer matches, the preserved state is kept and the resulting `needs_input` handoff is followed instead of forcing another retry.
- `BU-0483`: Recovery is only available to a worker whose current status is exactly in_progress; any other status is refused.
- `BU-0484`: Recovery requires the fleet diagnostic to begin with the literal prefix 'live worker stalled:', written specifically by the interactive fleet-watch loop's stall classification; a worker lacking that exact proof is refused recovery.
- `BU-0485`: The response lock is acquired before any recovery mutation, and both the status and stall-diagnostic checks are re-verified again once the lock is held, not trusted from the pre-lock read.
- `BU-0486`: Recovery is strictly one-shot: if a stall_recovery_attempted marker already exists for this worker, a second invocation is refused and the worker is instead escalated to needs_input.
- `BU-0487`: A notification-lease owner is only ever treated as provably dead when its recorded tmux pane no longer resolves at all AND its recorded process is not running; a resolvable pane (matching or reused identity), a still-live recorded pid, or a missing/malformed identity record all fail closed instead.
- `BU-0488`: An in-flight notification action lease from a prior supervisor blocks recovery unless the shared finalizer proves completion from the agent's own durable proof, or the lease owner is adjudicated provably dead; otherwise recovery is refused and the worker is escalated to needs_input rather than proceeding over unresolved delivery evidence.
- `BU-0490`: Before any mutation, relaunch metadata (tmux availability, session, window name, agent) must be fully present, or recovery is refused and the worker is escalated.
- `BU-0492`: The one-shot recovery marker is stamped only after every pre-flight check has passed, guaranteeing at most one recovery attempt is ever made even if a later step in this same invocation fails.
- `BU-0499`: Stall evidence (the diagnostic and message files) is cleared only once recovery has fully succeeded; every failure path preserves it so the eventual escalation reports an accurate reason instead of an empty one.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0480`: The stalled-worker recovery step requires exactly two positional arguments, a task ID and a repo; any other count is rejected.
- `BU-0481`: Each of the task ID and repo arguments to the stalled-worker recovery step must match a restricted identifier pattern; either failing is rejected.
- `BU-0482`: The stalled-worker recovery step refuses to proceed if the named task or repo does not exist in fleet state, or if the recorded worktree is unavailable.

