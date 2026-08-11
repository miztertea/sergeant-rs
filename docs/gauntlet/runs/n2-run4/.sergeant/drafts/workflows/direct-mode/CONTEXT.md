# direct-mode — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** direct mode is active and an edit is about to be made
- **Outcome:** delivery is only declared complete once PR, CI, review, and merge authorization are all satisfied
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `deliver`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-pre-edit-context-load` — context and the task tracker task state are loaded before any edit
2. `02-implement` — the owning task tracker task is claimed/created and implementation proceeds test-first
3. `03-validate-and-review` — direct-mode work passes through the same validation/review/gate steps as dispatched work
4. `04-deliver` — delivery is only declared complete once PR, CI, review, and merge authorization are all satisfied

