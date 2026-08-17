# 10-implement-with-tdd: implement with tdd

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Implementation proceeds seam by seam.

Trigger (workflow-level): Explicitly invoked to implement a defined piece of work (never auto-loaded).

## What must become true here (durable outcome)

Implementation proceeds seam by seam.

## Behavior contract

No behavior units are cited directly against this stage; its content is wholly delegated (see Applies below) or is the workflow's own structural connective tissue. This is recorded explicitly rather than invented to fill the section.

## Applies

Apply `@@tdd` (test-driven development for one confirmed seam at a time)
and `@@test-quality` (what makes a test worth keeping) throughout this
stage. **Corrected 2026-08-16, ICM-R3**: `tdd` was previously a two-stage
workflow this stage delegated to via undeclared context composition; it
is now a shared technique context (`tdd`'s own ICM-R3 adjudication,
`sergeant-rs-workspace/knowledge/evidence/gauntlet/runs/icm-r3/tdd/adjudication-draft.md`, REHOME — an
independent reviewer's dispute over that disposition was resolved by
owner ruling the same day: `tdd` is a technique an actor applies inside
its own implementation turn, not a separate procedure with its own
intent to hand off to).

## Helper: verify (folded from demoted `20-verify`, N1 adjudication A4)

`20-verify` was classified at extraction as deterministic machinery (ladder §6.5) with no checkpoint argument beyond the boilerplate; per adjudication A4 it is demoted and its behavior folded here as a helper invoked while implementation is underway, subordinate to this stage's own judgment-bearing outcome:

- **During implementation, typechecking and single test files should be run regularly, with the full test suite run once at the end.**
  (trigger: implementation work is underway; outcome: fast, frequent local checks are interleaved with work, with one full-suite pass at the close)

## Bounded judgment

Apply `@@bounded-judgment`. This section carries the local specialization
this stage contributes; it inherits the workflow's own `## Authority
envelope` (`../CONTEXT.md`) unchanged except where narrowed below.

### J2 — delegated to this stage
- Which concrete seam, test-framework idiom, and minimal implementation
  shape to choose within one confirmed cycle, per `@@tdd`.
- Which files or modules the typecheck/single-test-file helper checks
  during implementation, and when to run them, so long as the full suite
  runs once at the close.

### J1 — local choices allowed
- Ordering of otherwise-equivalent confirmed seams.
- Local test file layout and naming, within the target repository's own
  existing test conventions.

### J0 — must become `needs_input`
- **Seam confirmation.** `@@tdd` requires: no test is written at an
  unconfirmed seam. Before the first test of a cycle, ask the user
  "What's the public interface, and which seams should we test?" and wait
  for confirmation rather than inferring seams from the ticket alone.
  This requirement is stated here explicitly, rather than left implicit
  in the delegated content, because an independent ICM-R3 review
  confirmed it was previously invisible except by reading `tdd`'s own
  stage in full — a hidden contract-bearing dependency
  (`docs/icm/record-shapes.md` §1a rule 4 / `docs/icm/convention.md`
  §1a rule 1).

### Completion boundary
This stage may complete only when every confirmed seam for the current
piece of work has a red-then-green cycle recorded, per `@@tdd`,
typecheck/single-test checks have been run regularly during the work, and
the full test suite has passed once at the close.

### Decision evidence
Write material decisions (seam confirmations and their answers, any
deviation from a vertical-slice/one-seam-at-a-time cycle) to this stage's
own output artifact per `@@bounded-judgment`'s recommended table shape.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
