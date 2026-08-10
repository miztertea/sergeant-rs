# 40-escalate-on-second-attempt: escalate on second attempt

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-retire-original/output/README.md | L4 | upstream artifact produced by `30-retire-original` |

## Purpose

Exactly one bounded recovery attempt is made; a second stall escalates to needs-input.

Trigger (workflow-level): A worker is `in_progress` with a stall classification recorded by the watcher.

## What must become true here (durable outcome)

Exactly one bounded recovery attempt is made; a second stall escalates to needs-input.

## Behavior contract

- **A stall recovery attempt is gated on concrete stall proof — status must be in_progress and the fleet diagnostic must begin with a stall-classification marker written by the watcher — and every invocation is stamped so a second attempt always escalates to needs_input instead of retrying, guaranteeing exactly one bounded relaunch.**
  (trigger: a worker's status is in_progress with a recorded stall diagnostic; outcome: a stalled worker gets exactly one automatic relaunch attempt, ever, before requiring human input)
  — `BU-P6-071`, `reference/sergeant-upstream/bin/sgt-recover` (L6-10)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
