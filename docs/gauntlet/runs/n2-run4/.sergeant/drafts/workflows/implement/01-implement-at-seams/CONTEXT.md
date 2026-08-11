# 01-implement-at-seams

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** implementation work has pre-agreed seams

**Outcome:** implementation at those seams follows the TDD discipline rather than an ad hoc approach

**Statement (the operative rule):** /tdd is used where possible, at pre-agreed seams, when implementing work from a spec or tickets.

## What must become true here (durable outcome)

Implementation at those seams follows the TDD discipline rather than an ad hoc approach — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0975`: Typechecking and single test files are run regularly during implementation, and the full test suite is run once at the end.

