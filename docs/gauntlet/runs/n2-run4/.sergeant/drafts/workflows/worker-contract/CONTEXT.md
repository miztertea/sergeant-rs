# worker-contract — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** a worker begins a phase of implementation work
- **Outcome:** done is only ever reported once every gate has genuinely passed, and failed carries an exact, specific reason rather than a generic one
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `report-terminal-status`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-route-to-phase-skill` — the worker's approach is selected by a fixed routing table matching work shape to skill, rather than improvised per worker
2. `02-report-terminal-status` — done is only ever reported once every gate has genuinely passed, and failed carries an exact, specific reason rather than a generic one

