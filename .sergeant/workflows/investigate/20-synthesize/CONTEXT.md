# 20-synthesize: synthesize

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-frame/output/frame.md | L4 | the numbered questions this document is organized around |
| ../10-fan-out-evidence/output/evidence.md | L4 | the cited evidence this stage synthesizes |

## Purpose

One structured document: numbered questions, each answer citing
live-verified evidence, ending in a summary of recommendations — and an
explicit coverage statement naming any seat that did not report.

## What must become true here (durable outcome)

A single structured document exists: each framed question numbered, each
answer citing the evidence seat(s) it draws from, a closing summary of
recommendations, and an explicit statement of which seats reported and
which did not.

## Behavior contract

- **Organize the document by the numbered questions from `00-frame`, not
  by which seat produced which fact.** A reader should be able to find
  the answer to question 3 without knowing how many seats it took.
  (trigger: evidence has been gathered; outcome: the document answers
  the framed questions directly)
- **Every answer cites the specific evidence it rests on**, re-verified
  live where practical rather than trusted from the seat's report alone
  — this is the estate's own recon shape (numbered questions, cited
  evidence, coverage statement).
  (trigger: writing an answer; outcome: a reader can check the citation
  themselves, not just trust the synthesis)
- **State coverage honestly**: which seats reported, which did not, and
  what that gap means for confidence in the affected answers.
  (trigger: assembling the document; outcome: a review of the document
  can distinguish "thoroughly answered" from "answered on partial
  coverage")
- **End with a summary of recommendations** — what the evidence suggests
  should happen next, stated as a recommendation, not as work this
  workflow has already decided to do.
  (trigger: the document is otherwise complete; outcome: `40-close` has a
  recommendations section to draw its own recommended-next-intents from)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
How to phrase each answer and how to weigh evidence when seats disagree,
short of an irreconcilable conflict.

### J1 — local choices allowed
Document structure and formatting, provided every numbered question is
answered or explicitly left open.

### J0 — must become `needs_input`
Evidence from different seats materially conflicts on a fact central to
an answer, and no higher rung resolves which governs.

### Completion boundary
This stage may complete only once every framed question has a cited
answer (or an explicit "not yet known"), coverage is stated honestly, and
a recommendations summary closes the document.

### Decision evidence
`output/synthesis.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
