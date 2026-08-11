# wiki-maintenance — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** a coordinator runs or regenerates a wiki daily digest
- **Outcome:** a scheduling task cannot be marked done on the basis of installation alone, only on verified successful execution
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `schedule-wiki-digest`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-operate-wiki-digest` — digest changes are always previewed and inspected before they take effect and are always followed by verification
2. `02-schedule-wiki-digest` — a scheduling task cannot be marked done on the basis of installation alone, only on verified successful execution

