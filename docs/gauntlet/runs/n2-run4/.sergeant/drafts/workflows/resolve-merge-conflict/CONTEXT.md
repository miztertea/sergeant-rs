# resolve-merge-conflict — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** the resolving-merge-conflicts skill is invoked
- **Outcome:** the merge/rebase always reaches a resolved state rather than being abandoned mid-way
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `complete-merge`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-establish-conflict-state` — resolution proceeds from an understood starting state rather than a blind one
2. `02-resolve-hunk` — the resolved hunk reflects one of the two original intents (or both), never a fabricated third behaviour
3. `03-complete-merge` — the merge/rebase always reaches a resolved state rather than being abandoned mid-way

