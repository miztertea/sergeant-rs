# 17-capture-background-session-identity

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** claude --bg returns a background ID

**Outcome:** a malformed background ID can never be persisted into fleet state where every termination backstop assumes the well-formed shape

**Statement (the operative rule):** The Claude background session ID returned by claude --bg is validated at capture time against the exact same charset regex every one of the nine termination-path backstops uses to resolve it; a malformed ID fails the launch immediately rather than being persisted and silently defeating every one of those backstops later.

## What must become true here (durable outcome)

A malformed background ID can never be persisted into fleet state where every termination backstop assumes the well-formed shape — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0363`: The Claude background ID is persisted to fleet state immediately after capture, before the (potentially multi-second) post-launch liveness check loop runs, so a worker process death during that window cannot leave a genuinely live background session with no recorded identity anywhere.
- `BU-0364`: During the bounded post-launch liveness poll, if the Claude background session's reported state reaches "failed", the launch is treated as a terminal failure (status/sentinel written, process exits) rather than continuing to poll or attaching anyway.

