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

No behavior units are cited directly against this stage; its content is wholly delegated (see Delegation below) or is the workflow's own structural connective tissue. This is recorded explicitly rather than invented to fill the section.

## Delegation

This stage's outcome is produced by applying the `tdd` discipline
(`BU-P2-052`: "use the tdd workflow where possible, at seams pre-agreed
for testing"). **The current mechanism for doing so is unsettled**, not
merely a documentation gap: `tdd`'s own ICM-R3 package adjudication
(`docs/gauntlet/runs/icm-r3/tdd/adjudication-draft.md`) reached REHOME —
folding wholly into a `.sergeant/common/contexts/tdd.md` shared context,
loaded here via `@@tdd` once accepted — but its independent reviewer
(`docs/gauntlet/runs/icm-r3/tdd/review.md`) disputed that disposition,
finding the alternatives analysis never weighed the PL-7 engine-gap
representation `docs/icm/record-shapes.md` §5's own canonical worked
example describes for this exact shape (two workflows, `implement` and
`worker-mission`, each needing `tdd`'s own currently-distinct
`00-agree-seams`/`10-red-green-cycle` checkpoints with their own
retry/measurement). This stage's own `implement` package adjudication
(`docs/gauntlet/runs/icm-r3/implement/adjudication-draft.md`) does not
re-adjudicate that dispute — it is out of this pass's assigned scope —
and correspondingly does not commit this section to either outcome.
Until `tdd`'s dispute resolves, treat this as: apply the `tdd` package's
current content (`.sergeant/workflows/tdd/CONTEXT.md` and its two stage
`CONTEXT.md` files) as reference guidance for this stage's own judgment,
the same way it is applied today, while honoring the explicit J0 stated
below regardless of where that content ultimately lives.

## Helper: verify (folded from demoted `20-verify`, N1 adjudication A4)

`20-verify` was classified at extraction as deterministic machinery (ladder §6.5) with no checkpoint argument beyond the boilerplate; per adjudication A4 it is demoted and its behavior folded here as a helper invoked while implementation is underway, subordinate to this stage's own judgment-bearing outcome:

- **During implementation, typechecking and single test files should be run regularly, with the full test suite run once at the end.**
  (trigger: implementation work is underway; outcome: fast, frequent local checks are interleaved with work, with one full-suite pass at the close)
  — `BU-P2-053`, `reference/sergeant-upstream/.agents/skills/implement/SKILL.md` (body, lines 11-11)

## Bounded judgment

Apply `@@bounded-judgment`. This section carries the local specialization
this stage contributes; it inherits the workflow's own `## Authority
envelope` (`../CONTEXT.md`) unchanged except where narrowed below.

### J2 — delegated to this stage
- Which concrete seam, test-framework idiom, and minimal implementation
  shape to choose within one confirmed cycle.
- Which files or modules the typecheck/single-test-file helper checks
  during implementation, and when to run them, so long as the full suite
  runs once at the close (`BU-P2-053`).

### J1 — local choices allowed
- Ordering of otherwise-equivalent confirmed seams.
- Local test file layout and naming, within the target repository's own
  existing test conventions.

### J0 — must become `needs_input`
- **Seam confirmation.** No test is written at an unconfirmed seam. Before
  the first test of a cycle, ask the user "What's the public interface,
  and which seams should we test?" and wait for confirmation rather than
  inferring seams from the ticket alone. This requirement is stated here
  explicitly, rather than left implicit in the delegated `tdd` content,
  because an independent ICM-R3 review confirmed it was previously
  invisible except by reading `tdd`'s own stage in full — a hidden
  contract-bearing dependency (`docs/icm/record-shapes.md` §1a rule 4 /
  `docs/icm/convention.md` §1a rule 1; finding `BU-TDD-04`,
  `docs/gauntlet/runs/icm-r3/tdd/adjudication-draft.md` and
  `review.md`, CONFIRMED by the independent reviewer regardless of how
  `tdd`'s own placement dispute resolves).

### Completion boundary
This stage may complete only when every confirmed seam for the current
piece of work has a red-then-green cycle recorded, typecheck/single-test
checks have been run regularly during the work, and the full test suite
has passed once at the close.

### Decision evidence
Write material decisions (seam confirmations and their answers, any
deviation from a vertical-slice/one-seam-at-a-time cycle) to this stage's
own output artifact per `@@bounded-judgment`'s recommended table shape.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
