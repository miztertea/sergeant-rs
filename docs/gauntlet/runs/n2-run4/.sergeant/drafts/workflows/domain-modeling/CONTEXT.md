# domain-modeling — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** the user uses a term that conflicts with CONTEXT.md's existing definition
- **Outcome:** an ADR is offered only when the three-part test passes; otherwise it is not created
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `offer-adr`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-maintain-glossary-discipline` — the conflict is surfaced to the user immediately instead of silently accepted
2. `02-offer-adr` — an ADR is offered only when the three-part test passes; otherwise it is not created

## Cross-cutting mechanics

Deterministic machinery that applies throughout every stage below, not to one specific stage — see `_config/workflow-level-helpers.md`.

