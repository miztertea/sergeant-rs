# 11-acknowledge-response

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the response-acknowledgement step is run after a worker applies a response

**Outcome:** sensitive transport is only cleared after private archival succeeds, and a retry after partial failure converges idempotently without re-applying the decision

**Statement (the operative rule):** The response-acknowledgement step validates post-application proof, stages replay evidence in a private archive entry (`0700` directory, `0600` files), records acknowledgement, and only then clears active plaintext transport; if a later archive-marker or transport-cleanup step fails, rerunning the same command with the same response ID must converge existing archive, acknowledgement markers, and active transport without reapplying the decision.

## What must become true here (durable outcome)

Sensitive transport is only cleared after private archival succeeds, and a retry after partial failure converges idempotently without re-applying the decision — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0186`: `<repo> has pending or incomplete response acknowledgement` means a response was delivered but never acknowledged; the only path that completes the handshake is resuming the worker and acknowledging with the response-acknowledgement step from that worker's own pane.
- `BU-0436`: An acknowledgement is only accepted when it runs from the exact tmux pane recorded as owning the worker: both the calling process's own TMUX_PANE and its resolved pane identity must match the recorded dispatch pane.
- `BU-0437`: The response-acknowledgement step acquires the response lock around the entire acknowledgement, and its exit trap also removes any partially-written archive staging directory and ack staging files left by an interrupted run.
- `BU-0438`: Acknowledgement is refused unless the pending response id recorded in fleet state exactly equals the RESPONSE_ID argument.
- `BU-0439`: Acknowledgement is refused unless the pending response's recorded generation is a valid positive integer.
- `BU-0441`: If an archive entry already exists for this response id, acknowledgement requires it to be a complete, non-symlinked, non-retired directory recording all canonical fields whose recorded response id and generation match the pending response, and whose stored body is byte-identical to any still-present pending response transport.
- `BU-0442`: When no archive entry exists yet, acknowledgement requires the worktree's own post-application proof file to exist and its recorded response id, gate generation, and status to exactly match the worktree's current live state before anything is archived.
- `BU-0443`: A worker status of 'done' may only be acknowledged if the worktree also recorded a non-empty result; a done status with no result is refused.
- `BU-0444`: A worker status of 'failed' must carry a non-blank reason to be acknowledged; a blank reason is refused.
- `BU-0445`: A needs_input or blocked status may only be acknowledged once the worktree's own gate generation has advanced strictly past the generation the acknowledged response answered; the same generation is not sufficient.
- `BU-0446`: A post-application proof carrying any status other than in_progress, done, a non-blank failed reason, needs_input, or blocked is refused outright.
- `BU-0447`: The archive entry is assembled in a private staging directory (created via mkdir/chmod, contents chmod'd owner-only) and only renamed into its final location once every field has been written.
- `BU-0448`: Acknowledging a response atomically publishes the ack marker to both fleet state and the worktree, each via a temp-file-then-rename, before any consumed response files are removed.
- `BU-0449`: Once acknowledgement is published, the response-acknowledgement step removes the response body, its worktree mirror, and every worker-side response identity and applied-proof file.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0433`: The response-acknowledgement step requires exactly three positional arguments: task ID, repo, and response ID; any other count is rejected.
- `BU-0434`: Each of the task ID, repo, and response ID arguments to the response-acknowledgement step must match a restricted identifier pattern; any one failing is rejected.
- `BU-0435`: The response-acknowledgement step refuses to proceed if the worker's recorded worktree directory is unavailable.
- `BU-0440`: The response archive directory is created privately: restrictive umask, then explicitly chmod'd to owner-only.

