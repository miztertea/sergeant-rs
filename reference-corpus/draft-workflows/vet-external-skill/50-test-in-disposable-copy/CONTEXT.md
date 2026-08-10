# 50-test-in-disposable-copy: test in disposable copy

## Inputs

| File | Layer | Why |
|---|---|---|
| ../40-pin-source/output/README.md | L4 | upstream artifact produced by `40-pin-source` |

## Purpose

The external skill is tested in a disposable repository or worktree before broad installation.

Trigger (workflow-level): Before adopting an external skill, or when an adopted skill needs updating.

## What must become true here (durable outcome)

The external skill is tested in a disposable repository or worktree before broad installation.

## Behavior contract

- **Test the external skill in a disposable repository or worktree before broad installation.**
  (trigger: source pinned; outcome: the skill is proven in an isolated environment before being broadly installed)
  — `BU-P1-125`, `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L131, vet step 6)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
