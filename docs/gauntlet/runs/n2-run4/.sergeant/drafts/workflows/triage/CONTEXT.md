# triage — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** an unlabeled issue enters triage
- **Outcome:** the named state is applied directly, bypassing the ordinary multi-step triage procedure
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `quick-override`'s.

## How the stages relate

`triage` is graph-shaped (independent event-triggered entry points, not a single pipeline) — source/behavior_id order is used as the defensible default per the run-wide note above, not a proven chain. The list below is the corpus's own defensible-default ordering (source/behavior_id order), not a claim that each stage's input is the previous stage's output.

1. `01-operate-state-machine` — the issue is placed in the `needs-triage` state as its starting point
2. `02-surface-attention-queue` — three ordered buckets of items are presented, oldest first within each
3. `03-gather-context` — a redundancy check against the existing codebase is performed and its search scope is reported
4. `04-recommend-and-wait` — no further triage action is taken until the maintainer responds
5. `05-verify-claim` — the underlying claim (bug report or PR's stated effect) is actually exercised, not just read
6. `06-grill-if-needed` — the request is progressively sharpened and decisions are recorded inline rather than left implicit
7. `07-apply-outcome` — an agent brief comment is posted
8. `08-quick-override` — the named state is applied directly, bypassing the ordinary multi-step triage procedure

## Cross-cutting mechanics

Deterministic machinery that applies throughout every stage below, not to one specific stage — see `_config/workflow-level-helpers.md`.

