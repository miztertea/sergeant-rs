# 20-force-stop-worker

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the forced-drain step is invoked

**Outcome:** the command dies with an error if no matching drain is active

**Statement (the operative rule):** The forced-drain step refuses to run unless a matching drain (global, or the named project) is already active; force never operates without an antecedent cooperative drain having been set.

## What must become true here (durable outcome)

The command dies with an error if no matching drain is active — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0531`: Force-stopping requires either --dry-run (preview only, no termination) or an explicit --yes; invoking it with neither is refused.
- `BU-0532`: Workers whose recorded status is already done, failed:*, drained, force-stopped, or orphaned are excluded from force-stop eligibility entirely.
- `BU-0533`: --dry-run prints the exact set of eligible force-stop targets (task, repo, worktree, pid) without terminating anything.
- `BU-0534`: Before signalling a worker's recorded PID, the forced-drain step always stops any recorded Claude background session first, idempotently, because a background Claude session is not a child of the worker's process group and is invisible to process-group signalling.
- `BU-0535`: Before sending any kill signal to a worker's recorded PID, the forced-drain step re-verifies that PID's recorded process-start time against its actual current start time, to prevent killing an unrelated process that has since reused the same PID.
- `BU-0536`: When a PID-reuse mismatch is detected, the original worker is marked force-stopped without any signal being sent to the now-unrelated process holding that PID.
- `BU-0537`: Force-stop escalates from SIGTERM to SIGKILL only if the process is still alive after waiting up to five seconds (fifty 0.1s polls); SIGKILL is never sent immediately.
- `BU-0538`: A SIGTERM send failure for one worker is recorded and that worker is marked failed, but force-stop continues attempting the remaining eligible workers rather than aborting the whole batch; the run still exits nonzero overall so the operator knows manual recovery is needed.

