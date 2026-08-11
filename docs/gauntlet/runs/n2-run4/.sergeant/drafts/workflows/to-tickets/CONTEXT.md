# to-tickets — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** the project name is not yet known
- **Outcome:** only tickets with no remaining blockers are reported as immediately dispatchable
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `report-dispatch-frontier`'s.

## How the stages relate

Stages run in the fixed order below; each consumes the previous stage's declared evidence artifact (see each stage's own `CONTEXT.md` Inputs table).

1. `01-load-ticket-context` — the project name is established before further context loading
2. `02-draft-tickets` — one ticket covers the full vertical slice rather than being split by layer, and is independently verifiable when done
3. `03-review-breakdown` — the user reviews the breakdown before any ticket is actually published
4. `04-publish-tickets` — epics exist with real IDs before any child ticket that references them is created
5. `05-validate-published-graph` — the dependency graph is checked to be free of cycles and fabricated cross-repo edges before being considered valid
6. `06-report-dispatch-frontier` — only tickets with no remaining blockers are reported as immediately dispatchable

