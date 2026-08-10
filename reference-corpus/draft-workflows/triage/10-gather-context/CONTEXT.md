# 10-gather-context: gather context

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-show-attention/output/README.md | L4 | upstream artifact produced by `00-show-attention` |

## Purpose

The item and prior notes are read; an already-implemented check and out-of-scope-KB concept match are run.

Trigger (workflow-level): An item is at the front of one of the three fixed attention buckets, oldest first.

## What must become true here (durable outcome)

The item and prior notes are read; an already-implemented check and out-of-scope-KB concept match are run.

## Behavior contract

- **Triaging a specific item begins by fully reading the item and prior triage notes, exploring the codebase via its domain glossary and ADRs, and running two checks: whether the behavior is already implemented (by domain concept, not literal wording) and whether the request resembles a prior recorded out-of-scope rejection.**
  (trigger: a specific issue or PR is being triaged; outcome: the actor has full context plus a redundancy verdict and a prior-rejection match (if any) before recommending)
  — `BU-P3-065`, `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 70)
- **Matching a new issue against the out-of-scope KB is done by concept similarity rather than literal keyword overlap.**
  (trigger: gather-context's prior-rejection check runs; outcome: a conceptually similar but differently-worded request is still recognized as a match)
  — `BU-P3-089`, `reference/sergeant-upstream/.agents/skills/triage/OUT-OF-SCOPE.md` (line 75)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
