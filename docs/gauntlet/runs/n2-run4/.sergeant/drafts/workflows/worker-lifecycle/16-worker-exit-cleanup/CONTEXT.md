# 16-worker-exit-cleanup

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the worker process is exiting for any reason

**Outcome:** every background loop and the Claude background session are stopped on every exit path, including a clean completion no external script observes

**Statement (the operative rule):** On exit, the worker's _finish handler kills every background loop it started (notification delivery, progress watch, drain watch, termination watch) and stops any Claude background session as a ninth, in-process termination backstop independent of the eight external termination paths, so no background process leaks regardless of which exit path fired.

## What must become true here (durable outcome)

Every background loop and the Claude background session are stopped on every exit path, including a clean completion no external script observes — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0355`: At the worker's exit boundary, the accepted action lease is always settled — finalized from the agent's own completion proof, or explicitly recorded as pending with the exit reason — covering every terminal status (done, failed, drained, needs_input, blocked, waiting, orphaned) so no branch can exit leaving the lease silently outstanding.
- `BU-0356`: On exit, a status of done together with an empty .sergeant-result is downgraded to orphaned rather than accepted as a completed mission.
- `BU-0358`: When the agent itself publishes a completion proof for a notification, the worker settles the accepted action lease through the one shared finalizer (which re-verifies identity, lease, and proof under the response lock) rather than a second, separately-maintained inline implementation.

