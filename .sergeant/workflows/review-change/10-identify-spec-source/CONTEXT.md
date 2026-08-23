# 10-identify-spec-source: identify spec source

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-pin-fixed-point/output/README.md | L4 | upstream artifact produced by `00-pin-fixed-point` |

## Purpose

The spec/acceptance the diff is judged against is located by a fixed
priority order, or its absence is recorded — never invented.

## What must become true here (durable outcome)

The spec source is identified via a fixed priority order ending in asking
the user, or its absence is explicitly recorded.

## Behavior contract

Apply `@@identify-spec-source`. This package's own narrowing:

- **The spec source is located in a fixed priority order: issue
  references in commit messages, then a path the user passed as an
  argument, then a PRD/spec file under `docs/`, `specs/`, or `.scratch/`
  matching the branch or feature name, then — if nothing is found —
  asking the user; if the user says no spec exists, this is recorded and
  the panel's spec-fidelity axis is told explicitly that no spec source
  was available.**
  (trigger: identifying what the diff should be compared against;
  outcome: a spec source is found, or its absence is explicitly recorded)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
Judging whether a candidate PRD/spec file under `docs/`, `specs/`, or
`.scratch/` genuinely matches the branch or feature name.

### J1 — local choices allowed
None identified: the priority order is fully specified.

### J0 — must become `needs_input`
No spec source is found anywhere in the priority order: ask the user
whether one exists. If the user says no, the absence is recorded rather
than the stage inventing a source.

### Completion boundary
This stage may complete only once a spec source is identified, or its
absence is recorded.

### Decision evidence
Record the identified spec source — or its recorded absence — in
`output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
