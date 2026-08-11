# standard-workflow — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** a task is brought to the session
- **Outcome:** cleanup runs only after terminal state and evidence preservation are verified
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `deliver-and-cleanup`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-load-context` — context is fully loaded before an execution mode is chosen
2. `02-load-or-create-task` — an existing canonical task tracker task is reused when one exists; a new one is created only otherwise
3. `03-select-execution-mode` — an execution mode is chosen according to this rule
4. `04-reconcile-existing-state` — existing state is reconciled and reused rather than duplicated
5. `05-confirm-with-user` — the user is asked only for genuinely unresolved, scope/risk-changing decisions
6. `06-execute` — execution proceeds via the mode-appropriate path
7. `07-resolve-blocking-gate` — only genuinely missing decisions are solicited, recorded in the task tracker, and remediation continues without redundant re-asks
8. `08-deliver-and-cleanup` — cleanup runs only after terminal state and evidence preservation are verified

