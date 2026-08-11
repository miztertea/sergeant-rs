# list-projects — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** the project-listing step is invoked
- **Outcome:** only genuine project YAMLs are listed, and an empty result is reported explicitly rather than silently
- **Completion condition:** the sole inferred stage `list-projects` has reached its outcome (design inference — see `provenance.md`).

## How the stages relate

This candidate has exactly one stage (see `provenance.md` for why it is a design inference, not a directly-evidenced checkpoint).

1. `01-list-projects` — only genuine project YAMLs are listed, and an empty result is reported explicitly rather than silently

## Cross-cutting mechanics

This workflow's only stage carries these directly in its own `CONTEXT.md` (too few stages for `_config/` to mean "shared across more than one stage" per `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`).

