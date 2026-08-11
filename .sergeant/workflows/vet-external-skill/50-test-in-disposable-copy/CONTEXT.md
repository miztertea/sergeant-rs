# 50-test-in-disposable-copy: test in disposable copy

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-verify-no-conflict/output/README.md | L4 | upstream artifact produced by `30-verify-no-conflict` (this stage absorbed the demoted `40-pin-source` stage — N1 adjudication A4) |

## Purpose

The external skill's source is pinned or locked where the installer supports it; the skill is then tested in a disposable repository or worktree before broad installation.

Trigger (workflow-level): Before adopting an external skill, or when an adopted skill needs updating.

## What must become true here (durable outcome)

The external skill's source is pinned or locked where the installer supports it; the skill is tested in a disposable repository or worktree before broad installation.

## Behavior contract

- **Test the external skill in a disposable repository or worktree before broad installation.**
  (trigger: source pinned; outcome: the skill is proven in an isolated environment before being broadly installed)
  — `BU-P1-125`, `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L131, vet step 6)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Helper invocation: pin source

Demoted from a standalone stage (`40-pin-source`) at N1 adjudication A4: its only stage-level justification was the §6.5 deterministic-machinery boilerplate, with no additional checkpoint argument, so it folds into this stage as a helper invocation performed before testing (the source must be pinned before the pinned version is what gets tested). No `kind = "execute"` stage exists in the current engine, so the acting harness performs the pin/lock operation itself:

- **Pin or lock the external skill's source where the installer supports it.**
  (trigger: no conflict found; outcome: the installed skill version is pinned wherever the tooling allows)
  — `BU-P1-124`, `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L130, vet step 5)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
