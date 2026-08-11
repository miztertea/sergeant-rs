# 15-terminate-worker-process

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** _drain_terminate signals the process group

**Outcome:** termination never signals processes outside the worker's own ownership just because they share an ambient process group id

**Statement (the operative rule):** The termination handler only sends a signal to the worker's own process group when this worker process actually leads that group; if it does not, it falls back to terminating only the worker shell and lets that shell's own EXIT path clean up, because a group it does not lead may contain processes it does not own.

## What must become true here (durable outcome)

Termination never signals processes outside the worker's own ownership just because they share an ambient process group id — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0349`: The termination watcher's double guard mirrors _finish's own rule: it only stops the Claude background session on status=done together with a non-empty result; a done status with an empty result is treated as an orphaned mission still in progress, never as completion.
- `BU-0350`: The termination watcher does not stop the background session on a done status with an empty result; it explicitly lets _finish handle that empty-result (orphaned) case whenever attach exits by other means.
- `BU-0351`: Any status matching failed:<reason> is treated as terminal by the termination watcher regardless of the result field's state, and stops the background session.
- `BU-0491`: The stalled pane's current identity is re-verified to still match its recorded owner before it is allowed to be killed; a mismatch refuses recovery without killing anything.
- `BU-0564`: Before stopping a recorded Claude background session, if both a recorded and a live session id are available for that background id, a mismatch — meaning the recorded id has since been reused by an unrelated session — skips the stop call, so an unrelated live session is never accidentally stopped.
- `BU-0565`: The background-session id cross-check is deliberately best-effort, not fail-closed: any unresolvable verification (no session id yet persisted, jq unavailable, or the live-session query itself failing) still lets the stop call proceed, because preventing a genuinely leaked live session from running forever must not be defeated by an unrelated, transient verification failure — only a confirmed, positive id mismatch skips the call.
- `BU-0566`: Every termination path that kills a worker's pane or process must also call the background-Claude-session stop helper, because a background Claude session is not a child of the worker's process group and is invisible to process-tree or tmux kill-pane signals — omitting the call would silently leak the session.

