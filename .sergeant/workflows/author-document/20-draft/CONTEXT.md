# 20-draft: draft

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-map-sources/output/sources.md | L4 | the sources this draft is written from |
| ../10-establish-structure/output/structure.md | L4 | the outline this draft fills in |

## Purpose

The draft, written only from mapped sources.

## What must become true here (durable outcome)

A full draft exists, filling in `10-establish-structure`'s outline,
written only from the sources `00-map-sources` inventoried.

## Behavior contract

- **Write only from mapped sources.** A claim that cannot be traced to a
  source from `00-map-sources` is not written into the draft — it is
  either sourced properly first, or the gap is left explicit for
  `30-verify-fidelity-and-facts` to catch.
  (trigger: drafting a section; outcome: every claim in the draft has a
  traceable origin)
- **Follow the established outline's structure**, filling gaps rather
  than reshaping the document's own organization mid-draft — a
  structural change belongs back at `10`, not silently improvised here.
  (trigger: drafting begins; outcome: the draft and its outline stay in
  agreement)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
Wording and level of detail within each outlined section, provided every
claim traces to a mapped source.

### J1 — local choices allowed
Sentence-level phrasing.

### J0 — must become `needs_input`
A section the outline calls for has no mapped source to write it from,
and none can be found — this is a structural gap, not something to paper
over with an unsourced claim.

### Completion boundary
This stage may complete only once every outlined section is drafted from
a traceable source, or its gap is left explicit.

### Decision evidence
`output/draft.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
