# graphify — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** the graph-generation step is run for a project
- **Outcome:** a failure leaves the previous graph output intact and cleans up its own temporary artifacts rather than leaving a half-swapped or missing output
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `recover-from-failed-publish`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-run-graph-generation` — the run is only considered successful once both named output artifacts exist
2. `02-recover-from-failed-publish` — a failure leaves the previous graph output intact and cleans up its own temporary artifacts rather than leaving a half-swapped or missing output

