# 05-select-ui-subshape

## Inputs

| File | Layer | Why |
|---|---|---|
| ../04-drive-logic-prototype/output/outcome.md | L4 | upstream evidence produced by `drive-logic-prototype` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a UI prototype is being started and a sub-shape must be chosen

**Outcome:** sub-shape A is chosen unless there's a specific reason it can't host the variants

**Statement (the operative rule):** Between the two UI-prototype sub-shapes, sub-shape A (adjustment to an existing page) is the default whenever a plausible existing page can host the variants; sub-shape B (a new page) is reached for only when the prototype genuinely has no nearby home.

## What must become true here (durable outcome)

Sub-shape A is chosen unless there's a specific reason it can't host the variants — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1104`: In sub-shape A, variants are rendered on the existing route itself, gated by a ?variant= URL search param, while the route's existing data fetching, params, and auth are left unchanged — only the rendering swaps.
- `BU-1105`: A feature that doesn't yet have its own page but would naturally live inside an existing one (a new dashboard section, a new settings card, a new flow step) is still treated as sub-shape A, with the variants mounted inside that host page.
- `BU-1106`: Sub-shape B (a throwaway new route) is used only when the prototyped thing genuinely has no existing page to live inside; the route follows the project's existing routing convention, is named to be obviously a prototype, and uses the same ?variant= pattern as sub-shape A.
- `BU-1107`: Before committing to sub-shape B, a sanity check is made for whether the prototype could really not be embedded in an existing page, because an empty route hides design problems that a populated one would expose.

