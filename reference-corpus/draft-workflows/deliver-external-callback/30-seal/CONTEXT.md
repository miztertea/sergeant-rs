# 30-seal: seal

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-validate-acknowledgement/output/README.md | L4 | upstream artifact produced by `20-validate-acknowledgement` |

## Purpose

No cleanup while any event is unacknowledged; sealed history is retired, not deleted.

Trigger (workflow-level): A Work reaches a needs-input/blocked/failed/done transition and a registered consumer exists.

## What must become true here (durable outcome)

No cleanup while any event is unacknowledged; sealed history is retired, not deleted.

## Behavior contract

- **A fleet task's callback events cannot be cleaned up (sealed) while any event for it remains unacknowledged, and once sealed a callback task's event history is marked retired rather than deleted, so no cleanup can silently discard an external system's undelivered or unacknowledged notification.**
  (trigger: fleet state cleanup wants to retire a task's callback history; outcome: cleanup can never destroy evidence of an external notification that has not been proven to have been received)
  — `BU-P6-122`, `reference/sergeant-upstream/bin/sgt-callback` (L861-881)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
