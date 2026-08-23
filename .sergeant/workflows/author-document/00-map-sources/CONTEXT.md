# 00-map-sources: map sources

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

The authoritative source material is inventoried by path, with what each
source is authoritative *for*.

## What must become true here (durable outcome)

Apply `@@identify-spec-source` where the document is meant to reflect a
specific spec or acceptance criteria. Every source the draft will draw
from is inventoried by path, with a statement of exactly what it is
authoritative for — not merely that it is "relevant."

## Behavior contract

- **Inventory sources by path, each with a one-line statement of what it
  is authoritative for.** A source that is merely background reading is
  named as such, distinct from a source a specific claim will cite.
  (trigger: this stage begins; outcome: `20-draft` can trace every claim
  back to a named source rather than writing from general impression)
- **Where the intent names a brief carrying decisions already made (the
  `record-decisions` profile), that brief is the top-priority source and
  is mapped as authoritative for what was decided, its alternatives, and
  its rejection reasons — not as one source among many.**
  (trigger: the intent names such a brief; outcome: `10` and `30` know
  which source governs fidelity)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
Judging what a given source is actually authoritative for, versus merely
background.

### J1 — local choices allowed
Exact wording of the source inventory.

### J0 — must become `needs_input`
A source the document needs to cite cannot be located and the intent
does not name where it should come from.

### Completion boundary
This stage may complete only once every source the draft will draw from
is inventoried by path with its authoritative scope stated.

### Decision evidence
`output/sources.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
