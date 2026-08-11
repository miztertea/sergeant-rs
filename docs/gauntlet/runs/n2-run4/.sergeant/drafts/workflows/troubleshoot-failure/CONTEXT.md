# troubleshoot-failure — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** a failure is not covered by existing documentation
- **Outcome:** the gap is escalated as a well-formed task tracker task rather than left unresolved or guessed at
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `escalate-undocumented-gap`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-escalate-undocumented-gap` — the gap is escalated as a well-formed task tracker task rather than left unresolved or guessed at

