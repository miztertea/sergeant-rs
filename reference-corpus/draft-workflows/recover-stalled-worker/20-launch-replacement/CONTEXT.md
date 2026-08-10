# 20-launch-replacement: launch replacement

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-preflight/output/README.md | L4 | upstream artifact produced by `10-preflight` |

## Purpose

The replacement is validated live before the original is retired.

Trigger (workflow-level): A worker is `in_progress` with a stall classification recorded by the watcher.

## What must become true here (durable outcome)

The replacement is validated live before the original is retired.

## Behavior contract

- **During recovery, the replacement worker is only launched, and its identity validated, before the original stalled worker instance is ever terminated, so that any failure in the relaunch sequence leaves the original stalled process intact for investigation rather than losing the supervisor entirely.**
  (trigger: a stall recovery attempt is relaunching a worker; outcome: a failed recovery attempt never leaves a Work with no supervisor at all)
  — `BU-P6-072`, `reference/sergeant-upstream/bin/sgt-recover` (L12-15)
- **Recovery must validate a replacement supervisor's liveness, published identity, and notification-target creation BEFORE killing the stalled original — the kill must be strictly ordered after the replacement is confirmed live, and every abort path must restore fleet state so the recorded worker identity still points at the surviving original.**
  (trigger: sgt-recover replaces a stalled worker supervisor with a fresh one; outcome: recovery can never end up with neither a working original nor a working replacement — the destructive step (killing the original) only happens once the replacement is proven viable)
  — `BU-P7-094`, `reference/sergeant-upstream/tests/sgt-recover-replacement-test.sh` (lines 1-11)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
