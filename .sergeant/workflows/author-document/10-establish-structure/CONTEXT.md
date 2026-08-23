# 10-establish-structure: establish structure

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-map-sources/output/sources.md | L4 | the mapped sources this outline is built to organize |

## Purpose

An outline against a named audience and purpose.

## What must become true here (durable outcome)

An outline exists, structured against a named audience and purpose, that
`20-draft` can fill in from the mapped sources.

## Behavior contract

- **Name the audience and purpose explicitly before outlining.** A
  document without a stated audience drifts toward whichever tone the
  drafting stage happens to default to.
  (trigger: this stage begins; outcome: the outline is built against a
  concrete "who reads this, and why" rather than an implicit one)
- **The outline's sections correspond to what the mapped sources can
  actually support** — it is not padded with sections no source informs.
  (trigger: sources are mapped; outcome: `20-draft` has no section it
  cannot fill from real material)

## The `record-decisions` profile section

Where the intent names a brief carrying decisions an in-session grilling
already made:

- **This stage transcribes decisions already made; it never makes a new
  decision, resolves an open question, or picks between alternatives the
  brief itself left unresolved.**
  (trigger: the brief is ambiguous or silent about something this stage
  is tempted to resolve on its own; outcome: the outline stays faithful
  to what was actually decided, never quietly extending it)
- **A missing rationale, alternative, or rejection reason is recorded as
  missing — never invented.** A fabricated rationale reads exactly like a
  real one to every future reader; the gap must be visible in the
  outline as a gap.
  (trigger: the brief names a decision but not its rationale,
  alternatives, or rejection reasons; outcome: a future reader knows to
  go back to the source rather than trusting an invented account)
- **The outline's structure and placement (ADR numbering, glossary
  format) follow the repository's own convention where one exists, or an
  explicit placement stated in the brief.**

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Establishing the outline's own structure against the named audience and
  purpose.
- Where the `record-decisions` profile applies: phrasing each transcribed
  decision, alternative, and rejection reason without changing what the
  brief actually says; choosing the document's own structure and
  placement where the brief and repository convention leave it open.

### J1 — local choices allowed
Mechanical formatting (heading structure, numbering) consistent with the
repository's existing convention.

### J0 — must become `needs_input`
- No audience or purpose can be established from the intent.
- Where the `record-decisions` profile applies: the brief does not
  identify what was actually decided at all (as opposed to merely
  missing a rationale, which is logged, not escalated); or the
  repository's convention cannot be determined and no explicit placement
  is stated in the brief.

### Completion boundary
This stage may complete only once the outline exists against a named
audience and purpose and, where the `record-decisions` profile applies,
every decision named in the brief is represented with its gaps logged
rather than filled in.

### Decision evidence
`output/structure.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
