# 18-reattach-after-attach-exit

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the blocking claude attach call exits

**Outcome:** a legitimate cooperative gate (needs_input/blocked/waiting) is never mistaken for an unexpected death and spuriously re-attached

**Statement (the operative rule):** Respawn/reattach after an attach exit is only ever considered while .sergeant-status is still the unchanged in_progress it started at; any other value (done, failed:*, drained, orphaned, or a cooperative needs_input/blocked/waiting gate the agent itself published) is treated as the agent or an existing mechanism having already decided the outcome, so attach exiting afterward is expected, not an unexpected death.

## What must become true here (durable outcome)

A legitimate cooperative gate (needs_input/blocked/waiting) is never mistaken for an unexpected death and spuriously re-attached — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0366`: When attach exits while status is still in_progress, the worker distinguishes a genuinely dead background session (state=stopped, requiring respawn before re-attach to restore the same session/conversation) from a session that never died (state=working/blocked, requiring only a direct re-attach with no respawn).

