# no-mistakes-finding-routing — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** the validation pipeline surfaces an actionable finding
- **Outcome:** remediation converges to one worker per root cause, is rechecked before merge, and escalates to a human after two unsuccessful cycles rather than looping indefinitely
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `remediate-grouped-findings`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-route-finding` — the finding becomes owning-repo task tracker work rather than being fixed inline by the run
2. `02-remediate-grouped-findings` — remediation converges to one worker per root cause, is rechecked before merge, and escalates to a human after two unsuccessful cycles rather than looping indefinitely

