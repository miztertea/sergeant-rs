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

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- None beyond ordinary tool mechanics of writing the interface-level tests.

### J1 — local choices allowed
- Test file organization and naming, within the target repository's own conventions.

### J5 — governing constraints
- **Delete the old shallow-module unit tests rather than keeping them alongside the new ones** — an unconditional discipline, not a case-by-case judgment call (`BU-P4-020`).
- **Tests assert observable outcomes through the interface, never internal state** (`BU-P4-021`).

### Completion boundary
This stage may complete only when old shallow-module tests are deleted and new tests exist that assert only through the new interface.

### Decision evidence
The test suite change is this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
