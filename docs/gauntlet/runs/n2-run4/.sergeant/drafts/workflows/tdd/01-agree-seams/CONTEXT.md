# 01-agree-seams

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** a TDD cycle is about to begin and seams have not yet been agreed

**Outcome:** testing effort is deliberately scoped to seams the user has confirmed, not left to improvisation

**Statement (the operative rule):** Before writing any test, the seams under test are written down and confirmed with the user; no test is written at a seam that hasn't been agreed, so testing effort lands on critical paths and complex logic rather than every edge case.

## What must become true here (durable outcome)

Testing effort is deliberately scoped to seams the user has confirmed, not left to improvisation — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1127`: When exploring the codebase for TDD work, CONTEXT.md (if it exists) is read so test names and interface vocabulary match the project's domain language, and ADRs in the touched area are respected.

