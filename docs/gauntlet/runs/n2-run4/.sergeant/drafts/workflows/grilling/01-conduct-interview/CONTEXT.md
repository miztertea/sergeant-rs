# 01-conduct-interview

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** a grilling interview is in progress

**Outcome:** the user is never presented with more than one open question at once

**Statement (the operative rule):** Questions during a grilling interview are asked one at a time, waiting for the user's feedback on each before continuing, because asking several at once is bewildering.

## What must become true here (durable outcome)

The user is never presented with more than one open question at once — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0972`: A question resolvable by exploring the environment (filesystem, tools, etc.) is looked up directly instead of being asked; only genuine decisions are put to the user, and the actor waits for the user's answer on those.
- `BU-0973`: The actor does not act on a grilling interview's conclusions until the user confirms a shared understanding has been reached.

