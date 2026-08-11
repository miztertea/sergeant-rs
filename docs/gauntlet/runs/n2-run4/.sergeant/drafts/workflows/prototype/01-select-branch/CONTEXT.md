# 01-select-branch

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** the user wants a throwaway prototype to answer a design question

**Outcome:** the question is classified as logic or UI and routed to the matching branch's process

**Statement (the operative rule):** Which prototype branch to use is identified from the user's prompt, the surrounding code, or by asking the user if they are around: a logic/state-model question routes to the LOGIC.md branch, a what-should-this-look-like question routes to the UI.md branch.

## What must become true here (durable outcome)

The question is classified as logic or UI and routed to the matching branch's process — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1079`: If the branch (logic vs UI) is genuinely ambiguous and the user is not reachable, the branch is chosen by default to match the surrounding code (a backend module implies logic, a page or component implies UI), and the assumption is stated at the top of the prototype.

