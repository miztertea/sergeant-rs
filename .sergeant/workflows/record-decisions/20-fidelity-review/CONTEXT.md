# 20-fidelity-review: fidelity review

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-transcribe-decisions/output/README.md | L4 | upstream artifact produced by `10-transcribe-decisions` |

## Purpose

Every axis named in the brief's authoritative list runs as a separate,
non-contaminating parallel review, with fidelity to the brief weighted
above every other axis; outputs unblended.

Trigger (workflow-level): An in-session grilling has produced decisions
that need to become durable ADR/glossary material, and the
transcription/write-up is delegated rather than done in the grilling
session itself.

## What must become true here (durable outcome)

Every axis named in the brief's authoritative list runs as a separate,
non-contaminating parallel review, with fidelity to the brief weighted
above every other axis; outputs unblended.

## Behavior contract

- **Independent review before completion runs every axis named in the brief's own authoritative axis list as separate parallel subagents whose contexts cannot contaminate each other, even if a loaded review skill names fewer axes — the same brief-authoritative axis mechanism `worker-mission/30-independent-review` uses, reused here rather than reinvented.**
  (trigger: the transcription reaches its pre-completion review gate; outcome: review coverage is driven by the dispatching brief, not silently narrowed by whatever generic skill text happens to be loaded)
- **Fidelity — does the transcribed material actually match what the brief said, with every gap logged rather than filled — is the top-weighted axis: a fidelity finding against the transcription outranks a finding on any other axis, and a transcription with an open fidelity finding is never considered complete regardless of how the other axes score.**
  (trigger: the review's axes report; outcome: a well-formatted document that quietly drifted from the brief is never mistaken for a good one — fidelity governs)
- **Review outputs stay in separate, unblended, unreranked per-axis sections, the same as `worker-mission/30-independent-review`'s own rule.**
  (trigger: the parallel reviews complete; outcome: a reader can see each axis's own finding, not a synthesized blend that could bury a fidelity finding under an average)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- None beyond ordinary tool mechanics of dispatching the parallel
  sub-reviews.

### J1 — local choices allowed
- Formatting/ordering of the unblended per-axis output sections,
  provided fidelity's findings are never buried below the fold.

### J0 — must become `needs_input`
- A fidelity finding cannot be resolved by returning to
  `10-transcribe-decisions` (e.g. the brief itself is genuinely ambiguous
  about what was decided).

### Completion boundary
This stage may complete only when every axis named in the brief's
authoritative list has run as a separate, non-contaminating parallel
review, with outputs unblended, and no open fidelity finding remains.

### Decision evidence
The per-axis review outputs, with the fidelity axis's findings named
explicitly, are this stage's own decision record.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
