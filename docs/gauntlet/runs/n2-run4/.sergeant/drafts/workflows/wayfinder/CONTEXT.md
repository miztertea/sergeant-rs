# wayfinder — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** an effort's map Notes section states an override
- **Outcome:** ticket selection is cheap and deterministic, and claimed before any work starts
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `work-through-map-session`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-select-wayfinder-mode` — the map can drive execution rather than only decisions, when explicitly opted in
2. `02-claim-ticket` — two concurrent sessions do not duplicate work on the same ticket
3. `03-chart-the-map` — every subsequent charting step is shaped by an already-fixed destination
4. `04-work-through-map-session` — ticket selection is cheap and deterministic, and claimed before any work starts

## Cross-cutting mechanics

Deterministic machinery that applies throughout every stage below, not to one specific stage — see `_config/workflow-level-helpers.md`.

