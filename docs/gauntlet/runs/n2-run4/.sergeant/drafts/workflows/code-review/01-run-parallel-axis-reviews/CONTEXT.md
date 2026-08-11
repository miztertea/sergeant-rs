# 01-run-parallel-axis-reviews

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** both axes are ready to be evaluated

**Outcome:** each axis's findings are reasoned about in an isolated context before being combined

**Statement (the operative rule):** The Standards and Spec reviews each run as a separate parallel sub-agent so that neither review's context-gathering pollutes the other's, and this skill aggregates their findings afterward.

## What must become true here (durable outcome)

Each axis's findings are reasoned about in an isolated context before being combined — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0933`: The Standards axis always carries a fixed baseline of Fowler code smells in addition to whatever the repo's own documented standards say, applying even when the repo documents nothing.
- `BU-0934`: A documented repo standard always overrides the smell baseline: where the repo endorses something the baseline would flag, the smell is suppressed.
- `BU-0935`: Every baseline smell is reported as a judgement-call heuristic, never a hard violation, and is skipped if tooling already enforces it.
- `BU-0936`: The Standards and Spec sub-agents are spawned together in a single message (two `Agent` tool calls), both using the `general-purpose` subagent type.
- `BU-0937`: The Standards sub-agent's brief requires it to report documented-standard violations (citing the file and rule) separately from baseline smells (named and quoted), explicitly distinguish hard violations from judgement calls, skip anything tooling enforces, and stay under 400 words.
- `BU-0938`: The Spec sub-agent's brief requires it to report requirements missing or partial, behaviour not asked for (scope creep), and requirements that look implemented but wrong, quoting the spec line for each finding, and stay under 400 words.

