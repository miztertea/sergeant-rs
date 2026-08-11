# 01-prepare-spec-inputs

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** the current state of the codebase has not already been explored

**Outcome:** the spec is grounded in the actual codebase and its existing vocabulary/decisions

**Statement (the operative rule):** If the current state of the codebase has not already been explored, it is explored before writing the spec; the project's domain glossary vocabulary is used throughout, and ADRs in the touched area are respected.

## What must become true here (durable outcome)

The spec is grounded in the actual codebase and its existing vocabulary/decisions — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0988`: A spec produced by to-spec synthesizes what has already been discussed and understood about the codebase; the user is not interviewed.
- `BU-0989`: If the issue tracker and triage label vocabulary have not been provided, /setup-matt-pocock-skills is run to establish them.

