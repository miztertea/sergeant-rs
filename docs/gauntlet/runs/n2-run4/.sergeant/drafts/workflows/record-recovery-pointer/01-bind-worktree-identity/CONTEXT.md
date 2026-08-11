# 01-bind-worktree-identity

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** the task-tracker memory step is invoked with a worktree path

**Outcome:** git identity is never captured against a wrong or unrelated checkout that merely happens to satisfy a looser "is a git worktree" test

**Statement (the operative rule):** The task-tracker memory step binds every task tracker call to the worktree its own fleet state record actually owns: a missing owned-worktree record, or a WORKTREE argument that resolves (via realpath) to a different path than the recorded one, fails closed with a diagnostic rather than trusting the caller's argument.

## What must become true here (durable outcome)

Git identity is never captured against a wrong or unrelated checkout that merely happens to satisfy a looser "is a git worktree" test — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0337`: For a handoff action, the worktree must resolve as the ROOT of a real git worktree (via git rev-parse --show-toplevel, realpath-compared), not merely a subdirectory of one or any unrelated repository; a response action has no such requirement since it records no git identity.
- `BU-0338`: A handoff whose git HEAD cannot be resolved fails closed (diagnostic + exit 1) rather than recording a handoff with missing or fabricated git identity.
- `BU-0340`: The task tracker handoff records a checkpoint summary (status/branch/head), a pointer to fleet state (message/diagnostic/worker.log) to reconcile before resuming, and an explicit decision note that raw escalation and response text stay out of the task tracker, delivered instead through the atomic .sergeant-response transport.
- `BU-0341`: A response action's response ID is validated as exactly a 32-character lowercase hex string before any task tracker write; an invalid ID is diagnosed and the action fails.
- `BU-0342`: The task tracker decision log for a delivered response intentionally excludes the response's exact text, recording only that a human response was received via Sergeant (by response-id) and directing the reader to the atomic .sergeant-response transport and updated fleet/git state.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0339`: A detached HEAD is recorded as the literal branch value "detached" in the handoff summary rather than left blank, since git reports an empty current-branch value for a real, valid detached state — not a capture failure.

## Cross-cutting mechanics (workflow-level helpers)

Deterministic machinery attached to this workflow as a whole (`workflow=record-recovery-pointer`, `stage=null`) — folded in here rather than `_config/` because this candidate has only this one stage:

- `BU-0334`: Recording a worker recovery pointer in the task tracker is skipped entirely (exit 0, no-op) when the repo state records no td_task for this repo.
- `BU-0335`: If the task tracker CLI is unavailable, the task-tracker memory step records a diagnostic naming the action and task and exits nonzero, rather than silently skipping the recovery-pointer write.

