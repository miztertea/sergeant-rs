# Test quality

Shared actor-skill context (Placement Ladder PL-3). Resolved as
`@@test-quality` from `.sergeant/common/contexts/test-quality.md` per
`docs/icm/convention.md` §4. Planned since `reference-corpus/
shared-context-map.md`'s N1 synthesis pass (16 units, anchored at the
`tdd` skill's intro line 10, `tests.md`, and `mocking.md`) but never
materialized under `.sergeant/common/contexts/` before this ICM-R3 pass —
see the `tdd` package adjudication's "Behavior-unit dispositions" table
for the gap finding this file closes. Named consumers per the
shared-context map: `diagnose-bug`, `prototype`, `tdd`, `implement`,
`deepen-module`. This ICM-R3 pass reconciles only the `tdd` package;
wiring the other four consumers' stage content to reference `@@test-quality`
is out of this pass's scope and is left as a follow-on finding, not done
silently here.

Provenance for this file's rules (which behavior unit justifies each rule,
and its upstream source) lives in `sergeant-rs-workspace`'s
`knowledge/evidence/provenance/common-contexts.md` — provenance markers
were stripped from the shipped content below; the record of why each rule
exists did not move with them.

What a good test is, where tests go, and how to design for mockability —
consulted before and during a red-green cycle (see `@@tdd`), not only at
the end.

## What a good test is

Tests verify behavior through public interfaces, not implementation
details. Code can change entirely; tests shouldn't. A good test reads
like a specification and survives refactors because it doesn't care about
internal structure.

## Good and bad tests

**Good — integration-style, tests through real interfaces:**

- Tests behavior users/callers care about
- Uses the public API only
- Survives internal refactors
- Describes WHAT, not HOW
- One logical assertion per test

**Bad — implementation-coupled.** The tell: the test breaks when you
refactor but behavior hasn't changed. Red flags:

- Mocking internal collaborators
- Testing private methods
- Asserting on call counts/order
- Test name describes HOW not WHAT
- Verifying through a side channel (e.g. querying the database directly)
  instead of the interface under test

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

## Source

The `tdd` skill's intro (line 10, "What a good test is" section),
`tests.md` (Good Tests / Bad Tests), `mocking.md` (When to Mock, Designing
for Mockability). Full behavior-unit citation trail for this content has
not been separately archived under `docs/gauntlet/promoted-provenance/` —
noted as a gap for the independent reviewer, not fabricated here (see the
`tdd` adjudication's Validation evidence section).
