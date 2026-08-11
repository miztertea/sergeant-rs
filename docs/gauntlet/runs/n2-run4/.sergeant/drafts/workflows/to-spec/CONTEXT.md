# to-spec — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** the current state of the codebase has not already been explored
- **Outcome:** the spec is immediately actionable in the tracker without a further triage pass
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `publish-spec`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-prepare-spec-inputs` — the spec is grounded in the actual codebase and its existing vocabulary/decisions
2. `02-sketch-test-seams` — the spec settles on the minimum number of new, high-leverage test seams
3. `03-publish-spec` — the spec is immediately actionable in the tracker without a further triage pass

