# Test-first

Resolved as `@@test-first` from `.sergeant/common/contexts/test-first.md`
per `docs/icm/convention.md` §4. Policy context (Placement Ladder PL-3).
**Consolidates the retired `tdd.md` and `test-quality.md` into one file;
both are deleted.** Consumers: `implement-change/10-implement`,
`fix-defect/40-fix-with-regression-test`, and any package's own stage
that builds a change test-first.

Test-driven development for one confirmed seam at a time — red, green,
one minimal implementation — and what makes the resulting test worth
keeping, in one place: how the loop runs, and what a good test is, were
never really two different questions.

## Seams — where tests go

A seam is the public boundary you test at: the interface where you
observe behavior without reaching inside. Tests live at seams, never
against internals.

**Test only at pre-agreed seams.** Before writing any test, write down
the seams under test and confirm them with the user. No test is written
at an unconfirmed seam. Ask: "What's the public interface, and which
seams should we test?"

## The loop

- **Red before green.** Write the failing test first, then only enough
  code to pass it. Don't anticipate future tests or add speculative
  features.
- **One slice at a time.** One seam, one test, one minimal implementation
  per cycle.
- **Vertical, not horizontal.** Horizontal slicing (writing all tests
  first, then all implementation) verifies imagined behavior rather than
  user-facing behavior, tests the shape of things rather than real
  behavior, goes insensitive to real changes, and commits to test
  structure before the implementation is understood. Work in vertical
  slices instead — one test, one implementation, repeat — each test a
  tracer bullet responding to what the last cycle taught.
- **Refactoring is not part of this loop.** It belongs to review, not the
  red-green cycle.

## What a good test is

Tests verify behavior through public interfaces, not implementation
details. Code can change entirely; tests shouldn't. A good test reads
like a specification and survives refactors because it doesn't care
about internal structure.

**Good — integration-style, tests through real interfaces:**

- Tests behavior users/callers care about
- Uses the public API only
- Survives internal refactors
- Describes WHAT, not HOW
- One logical assertion per test

**Bad — implementation-coupled.** The tell: the test breaks when you
refactor but behavior hasn't changed. Red flags: mocking internal
collaborators, testing private methods, asserting on call counts/order, a
test name describing HOW not WHAT, verifying through a side channel
instead of the interface under test.

**Bad — tautological.** The assertion recomputes the expected value the
way the code does, so it passes by construction and can never disagree
with the code. Expected values must come from an independent source of
truth — a known-good literal, a worked example, the spec — never a value
derived the same way the implementation derives it.

## Mocking

Mock at system boundaries only: external APIs, databases (prefer a test
database when practical), time/randomness, the file system. Don't mock
your own classes, internal collaborators, or anything you control.

**Designing for mockability at the boundary:**

- Prefer dependency injection: pass external dependencies in rather than
  constructing them internally, so a test can substitute them without
  reaching into module internals.
- Prefer SDK-style interfaces (one specific function per external
  operation) over one generic fetcher with conditional logic — each mock
  then returns one specific shape, with no conditional logic in test
  setup and clear visibility into which endpoints a test exercises.

## The policy sentence this estate keeps restating

**A builder's own read of its diff is panel input, never a substitute for
independent review** (`docs/DEVELOPMENT.md`:50, R-S0-12: "a builder's
self-probe is panel input, never a substitute"). Passing your own tests is
evidence for the panel to weigh, not a completion claim on its own.

## What this context contributes when loaded inside a stage

- **J0 the caller must honor:** seam confirmation is not a J2 judgment
  call the loaded stage may skip or infer — no test is written at an
  unconfirmed seam. A stage applying this policy stops and asks the seam
  question above before its first test of a cycle, and states this
  explicitly in its own `## Bounded judgment` rather than leaving it
  implicit in a delegated reference.
- **J2 the caller retains:** which concrete seam, test framework idiom,
  and minimal-implementation shape to choose within one confirmed cycle.
- **J1 the caller retains:** ordering of otherwise-equivalent confirmed
  seams; local test file layout.

There is no stage library in this engine. This file is shared text pulled
into a stage's own `CONTEXT.md` by `@@` reference. A change here must be
hand-propagated to every narrowing consumer — drift by construction,
named rather than hidden.
