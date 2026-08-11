# 05-shipping-gate: shipping gate

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/standing-constraints.md | L3 | constraints binding every stage of this workflow |
| ../04-validate/output/README.md | L4 | upstream artifact produced by `04-validate` |

## Purpose

The shipping gate runs at the approved boundary only.

Trigger (workflow-level): The user explicitly asks to work in this session, and one repository owns the complete outcome.

## What must become true here (durable outcome)

The shipping gate runs at the approved boundary only.

## Behavior contract

- **The final shipping gate in direct mode is run only at the approved shipping boundary, not automatically at the end of implementation.**
  (trigger: native validation and independent review have completed; outcome: the shipping gate never runs before the actor has confirmed the work has actually reached its approved boundary)
  — `BU-P8-058`, `reference/sergeant-upstream/docs/using-sergeant.md` (L26 (Direct mode, step 6))

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Delegation

This stage's outcome is produced by running **validate-and-ship** to its own completion (context composition today — see `docs/icm/convention.md` §4 on `@@name` versus true nested-workflow invocation, which does not exist yet).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
