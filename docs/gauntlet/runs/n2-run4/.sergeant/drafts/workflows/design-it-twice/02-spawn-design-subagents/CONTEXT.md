# 02-spawn-design-subagents

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-frame-problem-space/output/outcome.md | L4 | upstream evidence produced by `frame-problem-space` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** alternative interface designs are being explored for a deepening candidate

**Outcome:** at least three meaningfully distinct interface designs are produced instead of one

**Statement (the operative rule):** At least 3 sub-agents are spawned in parallel to design the deepened module's interface, each required to produce a radically different interface.

## What must become true here (durable outcome)

At least three meaningfully distinct interface designs are produced instead of one — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1051`: Each design sub-agent is prompted with its own separate technical brief (file paths, coupling details, dependency category, what sits behind the seam), kept independent of the user-facing problem-space explanation shown in step 1.
- `BU-1052`: Each spawned design sub-agent is assigned a different design constraint from a fixed contrasting set (minimize the interface, maximize flexibility, optimize for the most common caller, or design around ports & adapters) so their outputs diverge meaningfully.
- `BU-1053`: Each design sub-agent's brief includes both the codebase-design skill's own vocabulary and the project's CONTEXT.md vocabulary, so sub-agents name things consistently with both the architecture language and the project's domain language.
- `BU-1054`: Each design sub-agent's output must cover five specific elements: the interface itself (types, methods, params, invariants, ordering, error modes), a usage example, what the implementation hides behind the seam, the dependency/adapter strategy, and the trade-offs.

