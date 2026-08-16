# Proposed fold: `.sergeant/workflows/implement/10-implement-with-tdd/CONTEXT.md`

Not a live edit (`docs/adr/0013-icm-r0-owner-rulings.md` decision 6) — the
`implement` package is not itself part of this ICM-R3 pass's assigned
scope (`tdd` only); this note is the concrete diff a later pass over
`implement` should apply once the `tdd` REHOME is accepted.

## Current `## Delegation` section

```markdown
## Delegation

This stage's outcome is produced by running **tdd** to its own completion
(context composition today — see `docs/icm/convention.md` §4 on `@@name`
versus true nested-workflow invocation, which does not exist yet).
```

## Proposed replacement

```markdown
## Applies

Apply `@@tdd` (test-driven development for one confirmed seam at a time)
and `@@test-quality` (what makes a test worth keeping) throughout this
stage.

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Which concrete seam, test-framework idiom, and minimal implementation
  shape to choose within one confirmed cycle, per `@@tdd`.

### J1 — local choices allowed
- Ordering of otherwise-equivalent confirmed seams; local test file
  layout.

### J0 — must become `needs_input`
- **Seam confirmation.** `@@tdd` requires: no test is written at an
  unconfirmed seam. Before the first test of a cycle, ask the user "What's
  the public interface, and which seams should we test?" and wait for
  confirmation rather than inferring seams from the ticket alone.

### Completion boundary
This stage may complete only when every confirmed seam for the current
piece of work has a red-then-green cycle recorded, per `@@tdd`.
```

This closes the finding in the `tdd` package adjudication
(`docs/gauntlet/runs/icm-r3/tdd/adjudication-draft.md`, unit `BU-TDD-04`):
the current delegation prose names "context composition" as the
mechanism but never actually states the seam-confirmation J0 requirement
inside this stage's own contract — it was only ever visible by reading
the delegated `tdd` workflow's own stage in full, which is exactly the
"hidden dependency" `docs/icm/record-shapes.md` §1a rule 4 and
`docs/icm/convention.md` §1a rule 1 warn against.
