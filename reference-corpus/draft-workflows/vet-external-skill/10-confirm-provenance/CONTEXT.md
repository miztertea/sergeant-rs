# 10-confirm-provenance: confirm provenance

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-read-source/output/README.md | L4 | upstream artifact produced by `00-read-source` |

## Purpose

The external skill's source and update mechanism are confirmed.

Trigger (workflow-level): Before adopting an external skill, or when an adopted skill needs updating.

## What must become true here (durable outcome)

The external skill's source and update mechanism are confirmed.

## Behavior contract

- **Confirm the external skill's source and update mechanism.**
  (trigger: step 1 complete; outcome: provenance and update path are known before proceeding)
  — `BU-P1-121`, `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L127, vet step 2)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
