# review-findings-routing — workflow orientation

Layer 1 orientation only (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`, distilling `docs/icm/convention.md` §1a) — no stage instructions live here; each stage's own contract is in its `NN-.../CONTEXT.md`.

## What this workflow does

- **Trigger:** a dispatched worker produces a review finding artifact
- **Outcome:** nothing observing the status file can ever see status=blocked while the message/generation describing why are still missing or stale
- **Completion condition:** every member stage below has reached its own outcome, ending in stage `publish-blocked-gate`'s.

## How the stages relate

`review-findings-routing` is graph-shaped (independent event-triggered entry points, not a single pipeline) — source/behavior_id order is used as the defensible default per the run-wide note above, not a proven chain. The list below is the corpus's own defensible-default ordering (source/behavior_id order), not a claim that each stage's input is the previous stage's output.

1. `01-route-finding` — actionable findings become owning-repo task tracker tasks with durably published blocking guidance
2. `02-preserve-retry-evidence-on-failure` — the parsed, sanitized findings are durably retained with an exact retry command surfaced
3. `03-publish-blocked-gate` — nothing observing the status file can ever see status=blocked while the message/generation describing why are still missing or stale

