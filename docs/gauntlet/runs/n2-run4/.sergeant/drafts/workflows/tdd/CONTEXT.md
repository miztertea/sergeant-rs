# tdd — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** a TDD cycle is about to begin and seams have not yet been agreed
- **Outcome:** work proceeds one test-then-implementation slice at a time rather than as separate bulk test and implementation phases
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `run-red-green-loop`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-agree-seams` — testing effort is deliberately scoped to seams the user has confirmed, not left to improvisation
2. `02-run-red-green-loop` — work proceeds one test-then-implementation slice at a time rather than as separate bulk test and implementation phases

