# 00-enqueue: enqueue

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Identity is hashed from (type, source); a repeat enqueue returns the existing event rather than duplicating it.

Trigger (workflow-level): A Work reaches a needs-input/blocked/failed/done transition and a registered consumer exists.

## What must become true here (durable outcome)

Identity is hashed from (type, source); a repeat enqueue returns the existing event rather than duplicating it.

## Behavior contract

- **A callback event's identity is derived by hashing its event type together with a caller-supplied source ID, and enqueuing the same (type, source) pair twice is idempotent — it returns the already-recorded event rather than creating a duplicate — so a caller can safely retry enqueuing without risking a duplicate delivery.**
  (trigger: a caller enqueues a callback event that may have already been enqueued; outcome: the same logical event is never recorded, or delivered, more than once, regardless of how many times the enqueuing caller retries)
  — `BU-P6-120`, `reference/sergeant-upstream/bin/sgt-callback` (L425-441)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
