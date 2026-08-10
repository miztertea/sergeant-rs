# 30-retire-original: retire original

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-launch-replacement/output/README.md | L4 | upstream artifact produced by `20-launch-replacement` |

## Purpose

The original is retired only after the replacement is proven live.

Trigger (workflow-level): A worker is `in_progress` with a stall classification recorded by the watcher.

## What must become true here (durable outcome)

The original is retired only after the replacement is proven live.

## Behavior contract

- **sgt-recover performs exactly one bounded stall-recovery attempt per invocation for an in-progress worker: kill the stalled pane, relaunch a fresh worker, atomically update fleet metadata, and deliver a recovery notification — matching the source inventory's description of a single bounded operation, not an open-ended retry loop.**
  (trigger: an in-progress worker appears stalled (no observable progress); outcome: stall recovery is a single, boundable, observable action rather than an unbounded retry loop that could mask a genuinely broken worker)
  — `BU-P7-095`, `reference/sergeant-upstream/tests/sgt-recover-test.sh` (line 2 and source-inventory row)

> **Read `pane`/`tmux` above as this project's durable execution/session identity, not literally.** Old Sergeant's tmux pane is obsolete here (deviation register D2; `reference-corpus/synthesis.md` §4 clusters M1-M4) — `BU-P7-095` carry a durable identity/liveness/ownership policy that survives the pane; the pane itself does not.

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
