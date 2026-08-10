# 30-background-watch: background watch

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-reconcile-terminal/output/README.md | L4 | upstream artifact produced by `20-reconcile-terminal` |

## Purpose

Idempotent start, failed-start detection, stale-unit cleanup, graceful on unsupported platforms.

Trigger (workflow-level): An operator or another workflow (dispatch's `80-monitor`) needs a live view of the fleet.

## What must become true here (durable outcome)

Idempotent start, failed-start detection, stale-unit cleanup, graceful on unsupported platforms.

## Behavior contract

- **`sgt-watch --background` must be idempotent (a duplicate start is detected, not double-started), must detect and report a failed background start, must recognize and clean up a stale systemd unit, and must handle platforms without systemd support gracefully, in addition to covering ordinary active/terminal transitions.**
  (trigger: an operator runs sgt-watch --background to persistently monitor a fleet task; outcome: background monitoring survives duplicate invocation, failed starts, stale leftover units, and TOCTOU races during cleanup, on platforms both with and without systemd)
  — `BU-P7-099`, `reference/sergeant-upstream/tests/sgt-watch-background-test.sh` (lines 1-4)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Additional note

Borderline per synthesis.md — closer to a deterministic helper for keeping the observation running than a procedural checkpoint.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
