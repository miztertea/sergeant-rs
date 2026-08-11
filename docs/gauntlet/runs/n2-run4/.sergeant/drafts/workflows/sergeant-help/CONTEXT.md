# sergeant-help — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** sergeant-help is answering a question
- **Outcome:** each condition triggers its own fixed required action rather than an ad hoc response
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `handle-failure-or-handoff`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-research-and-answer` — the answer follows this fixed research/answer sequence rather than free-form search
2. `02-handle-failure-or-handoff` — each condition triggers its own fixed required action rather than an ad hoc response

