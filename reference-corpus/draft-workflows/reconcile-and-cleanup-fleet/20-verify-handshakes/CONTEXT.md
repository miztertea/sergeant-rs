# 20-verify-handshakes: verify handshakes

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-verify-ownership/output/README.md | L4 | upstream artifact produced by `10-verify-ownership` |

## Purpose

Acknowledgement is verified, re-verified under lock immediately before deletion, and a terminal seal is written.

Trigger (workflow-level): A task's repos are believed terminal and the operator (or an automated sweep) requests cleanup.

## What must become true here (durable outcome)

Acknowledgement is verified, re-verified under lock immediately before deletion, and a terminal seal is written.

## Behavior contract

- **Fleet cleanup for a task is blocked until sgt-callback check-acked succeeds for it, and immediately before the actual deletion, cleanup re-verifies the same condition under a callback lock and writes a terminal seal that rejects any new event generation, closing the race between the acknowledgement check and the deletion.**
  (trigger: sgt-cleanup is about to delete a task's fleet state; outcome: a task can never be deleted while any callback event is unacknowledged, and no new event can appear in the window between the check and the deletion)
  — `BU-P8-026`, `reference/sergeant-upstream/docs/callbacks.md` (L167-179)
- **Rejected callback events are intentionally left unacknowledged and therefore also block cleanup until an operator repairs the consumer and reruns the retry command; and if cleanup fails after the terminal seal is written and the fleet must resume, only that specific seal (not any other state) may be removed, using an explicit unseal command.**
  (trigger: a callback event is in the reject state when cleanup is attempted, or a seal was written but cleanup then failed; outcome: a permanently-failed delivery cannot silently vanish through cleanup, and a stuck seal has one narrow, explicit recovery path)
  — `BU-P8-027`, `reference/sergeant-upstream/docs/callbacks.md` (L174-184)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
