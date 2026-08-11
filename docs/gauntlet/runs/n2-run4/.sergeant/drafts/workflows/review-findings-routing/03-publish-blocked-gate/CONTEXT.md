# 03-publish-blocked-gate

## Inputs

| File | Layer | Why |
|---|---|---|

## Purpose

**Trigger:** _publish_blocked runs

**Outcome:** nothing observing the status file can ever see status=blocked while the message/generation describing why are still missing or stale

**Statement (the operative rule):** When publishing a blocked review-gate state, the message and generation files are durably written and published before the status file itself is flipped to 'blocked' — the status transition to blocked is the last write, and only happens after the notify attempt.

## What must become true here (durable outcome)

Nothing observing the status file can ever see status=blocked while the message/generation describing why are still missing or stale — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0715`: A failure notifying the coordinator (the notify step) while publishing a blocked review state is reported as an error and its exit status is propagated from _publish_blocked, rather than being silently absorbed.
- `BU-0721`: The review-findings router takes a snapshot, under lock, of the current global gate generation and (if one exists) this axis's own gate-file generation at the very start of a routing attempt, before doing any of the attempt's real work.
- `BU-0757`: Any actionable finding whose severity is in the blocking class causes the run to publish a blocked review-remediation state naming every routed task-tracker task for the axis (whether newly created or deduplicated) and to exit 2 — this happens regardless of whether each finding's task tracker side effect was a create, an update, or came from a prior run.
- `BU-0758`: Clearing this axis's own published gate file, and re-deriving the aggregate blocked message from what remains, only happens if the axis's gate-file generation is still exactly what was observed when this routing attempt started — a concurrent invocation that already advanced or cleared that same gate is not clobbered by this attempt's own clear.
- `BU-0759`: When there is no per-axis gate file involved (a gate-less recovery from a prior routing failure), the review-findings router only clears the overall blocked status if no OTHER review axis currently has an open gate file — a clean retry for one axis must never unblock a worker that a different axis is still legitimately blocking.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0712`: Publishing a blocked review-gate state advances a single worktree-wide gate generation counter every time it is called.
- `BU-0713`: Each review axis's blocked-state message is stored in its own gate file, and the worktree's aggregate blocked message is the concatenation of every currently active axis's gate message — one axis publishing a block does not erase another axis's still-open block message.

