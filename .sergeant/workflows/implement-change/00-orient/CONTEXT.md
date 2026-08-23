# 00-orient: orient

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

The revision is pinned, the spec/acceptance source is located, and the
change's boundary is stated in the actor's own words, before any code is
touched.

## What must become true here (durable outcome)

A fixed revision the whole run is judged against, a located spec or
acceptance source (or an honest "none exists"), and a stated boundary —
what is in scope for this change, what is explicitly out — all exist
before implementation begins.

## Behavior contract

- **Apply `@@pin-fixed-point`: pin the revision this run is judged
  against and confirm it resolves.**
  (trigger: this stage begins; outcome: every later stage shares one
  fixed comparison point, never re-derived)
- **Apply `@@identify-spec-source`: locate the intent/spec/acceptance
  criteria by the fixed priority order, ending in asking.**
  (trigger: the revision is pinned; outcome: a spec source is found, or
  its absence is explicitly recorded rather than invented)
- **State the change's boundary in the actor's own words: what this
  change is for, and what it explicitly does not attempt.**
  (trigger: the spec source is located, or its absence recorded; outcome:
  a boundary statement exists that `20-panel`'s spec-fidelity axis and
  `40-close`'s evidence packet can both be checked against)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Phrasing the boundary statement, and judging which parts of a
  discovered spec source are actually in scope for this specific change
  versus adjacent work the intent does not ask for.

### J1 — local choices allowed
- Exact wording of the pinned-revision and spec-source records.

### J0 — must become `needs_input`
- No fixed point can be resolved (`@@pin-fixed-point`'s own J0).
- No spec/acceptance source can be located anywhere in the priority order
  *and* the intent does not itself state what "done" means
  (`@@identify-spec-source`'s own J0, restated here as this stage's own
  completion condition).

### Completion boundary
This stage may complete only once the revision is pinned and confirmed,
the spec/acceptance source is located or its absence is recorded, and the
change's boundary is stated.

### Decision evidence
The pinned revision, the located (or absent) spec source, and the
boundary statement are recorded in `output/orientation.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
