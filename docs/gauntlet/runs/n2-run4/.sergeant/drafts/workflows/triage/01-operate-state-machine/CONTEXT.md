# 01-operate-state-machine

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** an unlabeled issue enters triage

**Outcome:** the issue is placed in the `needs-triage` state as its starting point

**Statement (the operative rule):** An unlabeled issue's default first state transition is to `needs-triage`.

## What must become true here (durable outcome)

The issue is placed in the `needs-triage` state as its starting point — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1147`: If an issue's or PR's state roles conflict, the conflict is flagged and the maintainer is asked before any further action is taken.
- `BU-1149`: An issue in `needs-info` automatically returns to `needs-triage` once the reporter replies.
- `BU-1150`: The maintainer can override any state transition at any time, but a transition that looks unusual is flagged and confirmed with the maintainer before it proceeds.

