# 04-deliver

## Inputs

| File | Layer | Why |
|---|---|---|
| ../03-validate-and-review/output/outcome.md | L4 | upstream evidence produced by `validate-and-review` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a direct-mode implementation is ready for delivery

**Outcome:** delivery is only declared complete once PR, CI, review, and merge authorization are all satisfied

**Statement (the operative rule):** In direct mode, a PR is opened for every implementation, and required CI, review threads, and merge authorization must be satisfied before delivery is called complete.

## What must become true here (durable outcome)

Delivery is only declared complete once PR, CI, review, and merge authorization are all satisfied — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0016`: In direct mode, handoff, PR, merge, deployment, and cleanup outcomes are recorded.

