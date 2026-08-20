# 10-transcribe-decisions: transcribe decisions

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Every decision in the brief is transcribed into ADR/glossary material
with its alternatives and rejection reasons; a rationale the brief does
not carry is logged as a gap, never invented.

Trigger (workflow-level): An in-session grilling has produced decisions
that need to become durable ADR/glossary material, and the
transcription/write-up is delegated rather than done in the grilling
session itself.

## What must become true here (durable outcome)

Every decision in the brief is transcribed with its alternatives and
rejection reasons into ADR/glossary material; a decision whose
rationale, alternatives, or rejection reason the brief does not carry
has that gap logged explicitly rather than filled in.

## Behavior contract

- **Every decision named in the brief is transcribed into the ADR/glossary material in substance: what was decided, the alternatives considered, and why each alternative was rejected, exactly as the brief states them.**
  (trigger: the brief names a decision; outcome: the durable record matches the brief's own account of that decision, not a paraphrase that drifts from it)
- **A missing rationale, alternative, or rejection reason is recorded as missing — never invented.** Inventing one launders a guess into the permanent record: a fabricated rationale reads exactly like a real one to every future reader, and nothing about the document format distinguishes "the brief said this" from "the transcriber guessed this."
  (trigger: the brief names a decision but not its rationale, alternatives, or rejection reasons; outcome: the gap is visible in the record as a gap, so a future reader knows to go back to the source rather than trusting an invented account)
- **This stage transcribes decisions already made; it never makes a new decision, resolves an open question, or picks between alternatives the brief itself left unresolved.**
  (trigger: the brief is ambiguous or silent about something this stage is tempted to resolve on its own; outcome: the transcription stays faithful to what was actually decided, never quietly extending it)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Phrasing each transcribed decision, alternative, and rejection reason
  without changing what the brief actually says.
- Choosing the ADR/glossary document's own structure and placement,
  where the brief and the repository's own convention leave it open.

### J1 — local choices allowed
- Mechanical formatting (heading structure, ADR numbering) consistent
  with the repository's existing convention.

### J0 — must become `needs_input`
- The brief does not identify what was actually decided at all (as
  opposed to merely missing a rationale, which is logged as a gap, not
  escalated).
- The repository's ADR/glossary convention cannot be determined and no
  explicit placement is stated in the brief.

### Completion boundary
This stage may complete only when every decision named in the brief has
been transcribed, with every gap in rationale/alternatives/rejection-
reasons logged explicitly rather than invented, and no new decision has
been made on the brief's behalf.

### Decision evidence
The transcribed ADR/glossary material, including every logged gap, is
this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
