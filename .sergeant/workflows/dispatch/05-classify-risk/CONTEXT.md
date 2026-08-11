# 05-classify-risk: classify risk

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-check-queue-and-plan/output/README.md | L4 | upstream artifact produced by `00-check-queue-and-plan` |

## Purpose

The objective is routed to the standard-isolated path or forced onto an explicit intent-file path by a fixed safety-sensitive keyword set.

Trigger (workflow-level): Work spans repositories, contains two or more independent repository-owned tasks, needs an isolated review worker, or the user asks for workers.

## What must become true here (durable outcome)

The objective is routed to the standard-isolated path or forced onto an explicit intent-file path by a fixed safety-sensitive keyword set.

## Behavior contract

- **An objective whose text matches a fixed set of safety-sensitive or stateful keywords (auth, security, secrets, payments, databases, migrations, production, destructive, persistent state, state transitions) cannot proceed on the standard-isolated intent path and must instead be given an explicit --intent-file.**
  (trigger: a work objective is being auto-converted into a minimal standard-isolated intent; outcome: risky-sounding work can never proceed on the lightweight auto-generated intent path; it must be given an explicit, fuller intent document)
  — `BU-P6-048`, `reference/sergeant-upstream/bin/_sgt-intent.sh` (L215-217)
- **The dispatch skill must document a `standard-isolated` execution path and name specific trigger keywords (auth, OAuth, security, secret, credential, payment, database, migration, stateful, production, destructive) that route work away from it, and must warn against mutation happening before validation, and must bound remediation to at most two cycles before escalating.**
  (trigger: a task is about to be dispatched to a worker; outcome: safety-sensitive work is routed through a different, more conservative path than routine implementation, and remediation loops are bounded rather than open-ended)
  — `BU-P7-016`, `reference/sergeant-upstream/tests/instruction-policy-test.sh` (lines 78-81)
- **--intent-file is mandatory whenever the objective names auth/OAuth, security, secrets or credentials, payments, databases or migrations, stateful/production work, destructive work, persistent state, or state transitions; the intent file must contain the eight required sections, and malformed, missing, path-traversing, symlinked, or oversized input fails before any dispatch mutation, while every other objective uses the lighter standard-isolated path.**
  (trigger: sgt-dispatch is about to launch a worker for a stated objective; outcome: high-risk objectives are structurally forced through a stricter, validated intent path before any state is created; low-risk objectives use a lighter path)
  — `BU-P8-069`, `reference/sergeant-upstream/docs/using-sergeant.md` (L112-117)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
