# 20-test-at-new-interface: test at new interface

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-design-it-twice/output/README.md | L4 | upstream artifact produced by `10-design-it-twice` |

## Purpose

Old shallow-module tests are deleted; new tests assert through the interface only.

Trigger (workflow-level): A module's interface needs redesign, or a port/adapter decision needs to be made deliberately rather than by default.

## What must become true here (durable outcome)

Old shallow-module tests are deleted; new tests assert through the interface only.

## Behavior contract

- **After deepening a module, delete the old unit tests that targeted the now-merged shallow modules rather than keeping them alongside new interface-level tests.**
  (trigger: a deepened module's new interface-level tests exist; outcome: the shallow modules' old unit tests are removed rather than retained)
  — `BU-P4-020`, `reference/sergeant-upstream/.agents/skills/codebase-design/DEEPENING.md` (Testing strategy, L34)
- **Tests written against a deepened module must assert on observable outcomes through the interface, not on internal state, so they survive internal refactors; a test that must change when the implementation changes is testing past the interface.**
  (trigger: writing tests for a deepened module; outcome: tests remain green across internal refactors that don't change observable behavior)
  — `BU-P4-021`, `reference/sergeant-upstream/.agents/skills/codebase-design/DEEPENING.md` (Testing strategy, L37)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
