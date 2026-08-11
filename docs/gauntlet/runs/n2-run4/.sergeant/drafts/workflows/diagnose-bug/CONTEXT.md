# diagnose-bug — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** diagnosing any hard bug
- **Outcome:** the bug is not declared done until all five completion conditions hold
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `declare-bug-fixed`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-build-feedback-loop` — effort concentrates on constructing the loop before anything else is attempted
2. `02-reproduce-and-minimize` — minimisation only begins once the loop is confirmed to be catching the right bug, reliably, with the symptom on record
3. `03-hypothesize-and-test` — testing starts from a ranked set of candidates rather than the first idea
4. `04-apply-fix` — a test-first fix is preferred, conditioned on a correct seam being available
5. `05-declare-bug-fixed` — the bug is not declared done until all five completion conditions hold

