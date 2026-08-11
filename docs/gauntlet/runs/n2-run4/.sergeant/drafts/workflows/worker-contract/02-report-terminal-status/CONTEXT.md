# 02-report-terminal-status

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-route-to-phase-skill/output/outcome.md | L4 | upstream evidence produced by `route-to-phase-skill` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a worker reaches a terminal outcome

**Outcome:** done is only ever reported once every gate has genuinely passed, and failed carries an exact, specific reason rather than a generic one

**Statement (the operative rule):** A worker writes `.sergeant-result` and sets `.sergeant-status=done` only after every gate passes; `failed: <exact reason>` is reserved for an unrecoverable terminal failure.

## What must become true here (durable outcome)

Done is only ever reported once every gate has genuinely passed, and failed carries an exact, specific reason rather than a generic one — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0286`: To recover a waiting or orphaned worker, response-delivery step is used and the worker is never marked done manually; to retry a failed repo, the underlying issue is fixed first, and `.sergeant-result` plus `.sergeant-status=done` are written only after every completion gate passes.

