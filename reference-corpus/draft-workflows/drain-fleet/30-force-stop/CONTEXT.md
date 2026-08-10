# 30-force-stop: force stop

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-worker-side-checkpoint/output/README.md | L4 | upstream artifact produced by `20-worker-side-checkpoint` |

## Purpose

Force-stop is refused unless a drain is already active; requires explicit confirmation or dry-run; displays exact identity.

Trigger (workflow-level): An operator needs to freeze new stage/turn admission — globally or for one project — before a disruptive operation.

## What must become true here (durable outcome)

Force-stop is refused unless a drain is already active; requires explicit confirmation or dry-run; displays exact identity.

## Behavior contract

- **Force-stopping workers is refused unless a cooperative drain is already active for the targeted scope, and it always requires explicit confirmation (--yes) or is limited to a --dry-run preview; it never runs automatically as a side effect of anything else.**
  (trigger: cooperative drain has failed to stop some workers within a bounded wait; outcome: a destructive force-stop only ever happens as an explicit, confirmed, drain-scoped operator action with full identity disclosed first)
  — `BU-P6-039`, `reference/sergeant-upstream/bin/sgt-drain-force` (L1-4, L45-46, L58-62)
- **sgt-drain-force must require an active drain and an explicit `--yes` (or offer `--dry-run`) before force-stopping any drain-eligible worker, and it must display the exact worker identity before stopping it, and it invokes a harness-specific backstop (e.g. a Claude background-session stop call) as part of the force-stop loop.**
  (trigger: cooperative drain fails to stop a worker and an operator must force-stop it; outcome: a destructive force-stop is never accidental: it requires both an active drain state and explicit operator confirmation, with the exact target identity shown first)
  — `BU-P7-083`, `reference/sergeant-upstream/tests/sgt-drain-force-test.sh` (line 2 and source-inventory description)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
