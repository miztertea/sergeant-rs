# 13-retire-response-handshake

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** cleanup considers retiring a handshake because the worker appears gone

**Outcome:** retirement requires two independently re-verified conditions (closed owning task, provably dead worker) on every attempt, not a one-time check

**Statement (the operative rule):** When the worker is gone for good, cleanup can retire the handshake instead, but only when, re-checked on every attempt, both hold: the owning task tracker task is closed, and the recorded worker is provably dead (pane gone/dead with matching identity, recorded PID not running, no process in its recorded process group, and worker_pid/worker_process_start/worker_process_group all recorded).

## What must become true here (durable outcome)

Retirement requires two independently re-verified conditions (closed owning task, provably dead worker) on every attempt, not a one-time check — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0188`: The refusal names which condition failed (e.g. process still alive, PID reused, pane identity mismatch, owning task tracker task not closed); a live, PID-reused, or identity-mismatched owner is always refused, since it is never correct to retire a handshake underneath a worker that might still finish it.
- `BU-0189`: Retirement records the exact partial state under `~/.local/share/sergeant/fleet/<task>/<repo>/response-retirement/` before mutating anything (verbatim copies of both sides, owner death evidence, and provable response-archive fields), never writes an acknowledgement, and marks a `retired` directory so the archive can never be read as one; the archive shares the fleet task's lifetime so a retried cleanup converges.
- `BU-0190`: Cleanup refuses a retirement archive that no longer describes the state it preserved — a changed response, a tampered or symlinked copy, or a drifted recorded owner — rather than trusting stale evidence.
- `BU-0624`: The fleet cleanup step only retires an unfinished response handshake without an acknowledgement when the owning task tracker task is closed and the worker that owned the handshake is provably dead by every check available (recorded pane identity, process liveness, process group); this is the single path in the file allowed to bypass the acknowledgement requirement.
- `BU-0625`: Every refusal to retire a response handshake is fail-closed, including cases that merely cannot be proven: incomplete worker process provenance, a still-live or non-matching recorded pane, a live or PID-reused recorded process, or live processes remaining in the recorded worker's process group.
- `BU-0626`: Before publishing a response-handshake retirement archive, the fleet cleanup step re-computes the digest of both the fleet-side and worktree-side response state and refuses to publish (rolling back instead) if either digest no longer matches what was captured, so a state change mid-archive can never be published as the preserved evidence.
- `BU-0627`: On a retry, the fleet cleanup step treats an existing response-retirement archive as valid evidence only if re-deriving its fleet-state digest, worktree-state digest, archive-fields digest, full manifest, and owner record from the files on disk right now reproduces exactly what the archive recorded; any divergence is refused rather than trusted.
- `BU-0628`: The fleet cleanup step writes an explicit 'retired, not an acknowledgement' marker file as part of every response-retirement archive, covers it by the same digest as the archive's real fields, and applies the same 0600 permission discipline the response-acknowledgement step uses, so a partial handshake that happens to carry every field a real acknowledgement has can still never be read as one.
- `BU-0629`: If rolling back a failed retirement-archive transaction itself fails, the fleet cleanup step reports a distinct CRITICAL exit path (return code 3) naming the preserved artifact to inspect, rather than folding the failure into the ordinary 'handshake not retired' refusal message.
- `BU-0630`: The fleet cleanup step only allows cleanup of a repo carrying any response-handshake artifact (fleet or worktree side) to proceed when that handshake is either fully acknowledged (matching response_id, generation, and both ack markers, and a validated archive entry) or has gone through explicit retirement; every other shape is refused.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0501`: A response-archive entry is only considered complete when it is an unsymlinked directory containing every one of the four canonical fields (body, gate_generation, applied_status, proof) as unsymlinked regular files, and carries no retirement marker.

