# notify-primary-session — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** the notify step is invoked
- **Outcome:** a durable, searchable activity record exists for every update regardless of the transport outcome
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `capture-wiki-activity`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-publish-notification` — the fleet watcher can discover the update from a durably persisted marker even if the requested transport later fails
2. `02-capture-wiki-activity` — a durable, searchable activity record exists for every update regardless of the transport outcome

## Cross-cutting mechanics

Deterministic machinery that applies throughout every stage below, not to one specific stage — see `_config/workflow-level-helpers.md`.

