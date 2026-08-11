# 20-check-actions: check actions

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-confirm-provenance/output/README.md | L4 | upstream artifact produced by `10-confirm-provenance` |

## Purpose

The external skill's filesystem, shell, network, Git, and credential actions are checked.

Trigger (workflow-level): Before adopting an external skill, or when an adopted skill needs updating.

## What must become true here (durable outcome)

The external skill's filesystem, shell, network, Git, and credential actions are checked.

## Behavior contract

- **Check the external skill's filesystem, shell, network, Git, and credential actions.**
  (trigger: source confirmed; outcome: the skill's side-effect surface across five named categories is known)
  — `BU-P1-122`, `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L128, vet step 3)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
