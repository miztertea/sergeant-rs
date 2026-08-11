# 17-claim-action-lease

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the acceptance path is about to claim the action lease for a notification target

**Outcome:** at most one nonce ever holds the action lease for a given notification — a second target cannot silently steal or duplicate acceptance

**Statement (the operative rule):** When the notification delivery loop is about to accept a notification target, it re-checks whether an action lease already exists for a different nonce and refuses to overwrite it; only when no lease exists does it atomically claim the lease for the current nonce via mktemp+mv.

## What must become true here (durable outcome)

At most one nonce ever holds the action lease for a given notification — a second target cannot silently steal or duplicate acceptance — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0425`: Before relaunching, an outstanding notification action lease from a prior supervisor blocks relaunch unless the shared finalizer can prove the agent's own durable completion proof for that exact turn; an unprovable lease refuses the relaunch rather than fabricating completion.
- `BU-0426`: Before overwriting notification state for a relaunch, any existing pending notification target's pane identity is preserved as superseded evidence; if that evidence conflicts with a previously recorded record, the relaunch is refused.
- `BU-0493`: Before overwriting notification target state for the relaunch, any existing pending notification target's pane identity is preserved as superseded evidence; a conflict with previously recorded evidence refuses the recovery.
- `BU-0909`: Before the active-notification pointer advances to a new notification id, evidence that the outgoing notification was acknowledged and delivered (including which pane identity received it) is durably captured into per-notification proof files, written once and only once (guarded by the proof file's own existence).
- `BU-0910`: When the active notification id changes, any stale worktree acceptance marker left over from the prior notification is deleted, so it can never be misread as consent for the new notification.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0500`: The response-archive field reader accepts a record only when the requested key appears in it exactly once with a non-empty value; a missing, duplicated, or empty key causes the read to fail rather than returning a guessed or partial value.
- `BU-0502`: A response-archive entry only 'matches' a given response id and gate generation when every one of the entry's own recorded fields and its embedded proof's own fields agree with each other and with the caller's values; a field defaulting to empty is explicitly range-checked so it can never silently satisfy the comparison.
- `BU-0509`: An action lease is considered complete only when the worktree's own durable per-nonce completion file contains the exact literal '<notification_id>|<nonce>' token; nothing else, however plausible, satisfies completion.
- `BU-0510`: Finalizing an action lease never fabricates completion: a malformed lease nonce, a missing notification-target directory, or an agent proof that does not exactly match the expected token each fails closed and records the specific pending reason, rather than guessing at completion.
- `BU-0511`: Finalization is idempotent: a lease whose completion token is already recorded as matching the expected value is reported as finalized without attempting to write anything again.
- `BU-0512`: Every premise for publishing lease completion (the notification id, the lease nonce, and the agent's proof) is re-verified under the response lock immediately before the write, because a concurrent supersede could have replaced any of them since the caller's own earlier checks.
- `BU-0513`: If the response lock is already held by another context within this same process, finalization does not attempt to wait for it — since the liveness check would see its own live PID forever — and instead records the pending reason for a later exit-boundary call to settle.
- `BU-0514`: A lease-outcome record (pending or finalized) is written exactly once per record name and is never overwritten by a later finalization attempt, so the first, most-proximate reason for an outstanding lease survives every later retry.

