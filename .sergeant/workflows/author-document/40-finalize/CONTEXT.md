# 40-finalize: finalize

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-map-sources/output/sources.md | L4 | the sources cited in the closing evidence |
| ../20-draft/output/draft.md | L4 | the draft this stage edits into its final form |
| ../30-verify-fidelity-and-facts/output/verification.md | L4 | the fidelity/fact findings this stage must have resolved |
| ../35-challenge/output/challenge.md | L4 | any standing challenge this stage must address or explicitly carry forward |

## Purpose

Edited for audience; closed with evidence (which sources, which revision
of each).

## What must become true here (durable outcome)

The document is edited into its final, audience-appropriate form, every
`30`/`35` finding is resolved or explicitly carried forward as a known
gap, and the close cites which sources were used and which revision of
each.

## Behavior contract

Apply `@@close` and `@@evidence-requirements`. This package's own
narrowing:

- **Every fidelity, fact, and consistency finding from `30` is resolved
  in the final document, or explicitly recorded as a known, unresolved
  gap** — never silently dropped.
  (trigger: finalizing; outcome: no finding from `30` disappears between
  verification and publication)
- **Every standing challenge from `35` is either addressed in the edit or
  explicitly carried forward as a stated limitation of the document.**
  (trigger: finalizing; outcome: the audience-facing gaps `35` found are
  visible to the reader, not quietly smoothed over)
- **The close cites which sources were used and which revision of each** —
  a path and, where the source is itself versioned (a commit, a spec
  revision), that revision.
  (trigger: the document is otherwise final; outcome: a future reader can
  tell exactly what this document was built from)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
Editing for audience within the bounds of what `30`/`35` already found.

### J1 — local choices allowed
Final formatting.

### J0 — must become `needs_input`
A finding or standing challenge cannot be resolved without a decision
only the human can make (e.g. whether an open challenge should change the
document's scope).

### Completion boundary
This stage may complete only once every `30`/`35` finding is resolved or
explicitly carried forward, and the close cites its sources and their
revisions.

### Decision evidence
`output/final.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
