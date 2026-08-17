# TDD technique

Shared actor-skill context (`docs/icm/convention.md` §6, Placement Ladder
PL-3 — `reference/proposal-icm-r-procedure-authority.md` §5.5, whose own
worked example list names "a TDD technique" first). Resolved as `@@tdd`
from `.sergeant/common/contexts/tdd.md` per `docs/icm/convention.md` §4.
Rehomed at ICM-R3 from the two-stage `tdd` workflow
(`.sergeant/workflows/tdd/`, N1 candidate **W22**); full behavior-unit
citation trail archived at `docs/gauntlet/promoted-provenance/tdd.md`. This
file does not re-derive those citations — it re-places the same cited
content at the rung the current corpus actually earns.

Provenance for this file's rules (which behavior unit justifies each rule,
and its upstream source) also lives in `sergeant-rs-workspace`'s
`knowledge/evidence/provenance/common-contexts.md` — provenance markers
were stripped from the shipped content below; the record of why each rule
exists did not move with them.

Test-driven development for one confirmed seam at a time: red, green, one
minimal implementation. Consult this whenever implementation proceeds
test-first — before and during the loop, not after. For what makes a test
worth keeping (as opposed to how the loop runs), see `@@test-quality`.

## Seams — where tests go

A seam is the public boundary you test at: the interface where you observe
behavior without reaching inside. Tests live at seams, never against
internals.

**Test only at pre-agreed seams.** Before writing any test, write down the
seams under test and confirm them with the user. No test is written at an
unconfirmed seam. Ask: "What's the public interface, and which seams
should we test?"

## Rules of the loop

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
- **Refactoring is not part of this loop.** It belongs to the review
  stage (`code-review`/`deepen-module` discipline), not the red-green
  cycle.

## What this context contributes when loaded inside a stage

Per `docs/icm/convention.md` §7.4: an actor skill does not claim authority
independently of its caller; it states the decision classes it
contributes.

- **J0 the caller must honor:** seam confirmation is not a J2 judgment
  call the loaded stage may skip or infer — no test is written at an
  unconfirmed seam. A stage applying this technique must
  stop and ask the user the seam question above before its first test of
  a cycle, and its own Bounded-judgment section should name this
  explicitly rather than leave it implicit in a delegated reference.
- **J2 the caller retains:** which concrete seam, test framework idiom,
  and minimal-implementation shape to choose within one confirmed cycle.
- **J1 the caller retains:** ordering of otherwise-equivalent confirmed
  seams; local test file layout.

## Workflow-level notes carried from the retired `tdd` workflow

- The workflow-level trigger and purpose statement are subsumed by this
  file's own opening paragraph; they were never independent of the
  technique content itself.
- Refactoring was already explicitly out of scope for the loop before
  this rehome — that boundary is unchanged, only its representation is.
