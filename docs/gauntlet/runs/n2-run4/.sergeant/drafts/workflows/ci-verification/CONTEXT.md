# ci-verification — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** the toolchain/task runner run test:docker:drain runs
- **Outcome:** compatibility is proven under both the host's ambient Bash and the minimum supported Bash 3.2, not just one
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `verify-bash-compat-both-passes`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-verify-bash-compat-both-passes` — compatibility is proven under both the host's ambient Bash and the minimum supported Bash 3.2, not just one

