# treehouse-init — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** treehouse initialization is run for a repo
- **Outcome:** initialization is idempotent: an already-initialized repo is reported as such rather than re-initialized
- **Completion condition:** the sole inferred stage `initialize-treehouse` has reached its outcome (design inference — see `provenance.md`).

## How the stages relate

This candidate has exactly one stage (see `provenance.md` for why it is a design inference, not a directly-evidenced checkpoint).

1. `01-initialize-treehouse` — initialization is idempotent: an already-initialized repo is reported as such rather than re-initialized

## Cross-cutting mechanics

This workflow's only stage carries these directly in its own `CONTEXT.md` (too few stages for `_config/` to mean "shared across more than one stage" per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`).

