# 30-verify-no-conflict: verify no conflict

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-check-actions/output/README.md | L4 | upstream artifact produced by `20-check-actions` |

## Purpose

The external skill does not conflict with repository AGENTS.md or safety policy.

Trigger (workflow-level): Before adopting an external skill, or when an adopted skill needs updating.

## What must become true here (durable outcome)

The external skill does not conflict with repository AGENTS.md or safety policy.

## Behavior contract

- **Verify the external skill does not conflict with repository AGENTS.md or safety policy.**
  (trigger: actions checked; outcome: no adopted skill contradicts the repository's own instruction or safety policy)
  — `BU-P1-123`, `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L129, vet step 4)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
