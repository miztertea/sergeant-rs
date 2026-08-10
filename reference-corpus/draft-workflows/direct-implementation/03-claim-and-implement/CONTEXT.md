# 03-claim-and-implement: claim and implement

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/standing-constraints.md | L3 | constraints binding every stage of this workflow |
| ../02-reconcile-existing-state/output/README.md | L4 | upstream artifact produced by `02-reconcile-existing-state` |

## Purpose

The task is claimed and the change is implemented.

Trigger (workflow-level): The user explicitly asks to work in this session, and one repository owns the complete outcome.

## What must become true here (durable outcome)

The task is claimed and the change is implemented.

## Behavior contract

- **In direct mode, claim or create the owning td task, then implement test-driven-first in the requested checkout or an isolated worktree.**
  (trigger: no conflicting work found; outcome: implementation proceeds under an owned, tracked task)
  — `BU-P1-010`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L28-29, direct-mode step 3)
- **In direct mode, never edit a default branch; create or reuse the owning feature branch before the first implementation change.**
  (trigger: implementation about to begin; outcome: all direct-mode changes land on a feature branch, never on default)
  — `BU-P1-011`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L30-31, direct-mode branch rule)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
