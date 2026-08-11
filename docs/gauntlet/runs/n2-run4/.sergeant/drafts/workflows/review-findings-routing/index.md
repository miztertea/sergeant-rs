---
kind: workflow
name: review-findings-routing
status: draft
version: 1
description: >-
  Trigger: a dispatched worker produces a review finding artifact.
  Outcome: nothing observing the status file can ever see status=blocked while the message/generation describing why are still missing or stale.
  Completion: Every member stage below has reached its own outcome, ending in stage `publish-blocked-gate`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# review-findings-routing

**Trigger:** a dispatched worker produces a review finding artifact

**Outcome:** nothing observing the status file can ever see status=blocked while the message/generation describing why are still missing or stale

**Completion condition:** every member stage below has reached its own outcome, ending in stage `publish-blocked-gate`'s.

**Ordering:** `review-findings-routing` is graph-shaped (independent event-triggered entry points, not a single pipeline) — source/behavior_id order is used as the defensible default per the run-wide note above, not a proven chain.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
