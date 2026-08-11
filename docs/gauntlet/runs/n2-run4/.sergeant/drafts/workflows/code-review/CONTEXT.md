# code-review — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** both axes are ready to be evaluated
- **Outcome:** the two axes stay visibly separate in the final report
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `aggregate-review-report`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-run-parallel-axis-reviews` — each axis's findings are reasoned about in an isolated context before being combined
2. `02-prepare-review-inputs` — a bad ref or an empty diff fails the review at this checkpoint instead of surfacing confusingly inside two parallel sub-agents
3. `03-aggregate-review-report` — the two axes stay visibly separate in the final report

