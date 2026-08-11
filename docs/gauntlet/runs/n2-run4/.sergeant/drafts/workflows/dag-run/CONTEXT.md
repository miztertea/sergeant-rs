# dag-run — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** a DAG stage is defined
- **Outcome:** the hook fails loudly rather than dispatching work it cannot later attribute to the DAG runner run
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `run-dispatch-hook`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-resolve-stage-brief` — the stage's brief source is one of the two named alternatives, resolved by whether the task tracker is set
2. `02-advance-on-dependency-completion` — the stage only becomes ready to dispatch once its named predecessor stages have completed, advanced automatically by the interactive fleet-watch loop
3. `03-verify-dag-prerequisites` — the run fails closed with actionable install guidance rather than failing deep inside a later DAG-runner call
4. `04-run-dispatch-hook` — the hook fails loudly rather than dispatching work it cannot later attribute to the DAG runner run

## Cross-cutting mechanics

Deterministic machinery that applies throughout every stage below, not to one specific stage — see `_config/workflow-level-helpers.md`.

