---
kind: workflow
name: worker-lifecycle
status: draft
version: 1
description: >-
  Trigger: a worker is resumed or recovered.
  Outcome: the call refuses to stop anything and reports the inconsistency instead of guessing.
  Completion: Every member stage below has reached its own outcome, ending in stage `stop-background-monitor`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# worker-lifecycle

**Trigger:** a worker is resumed or recovered

**Outcome:** the call refuses to stop anything and reports the inconsistency instead of guessing

**Completion condition:** every member stage below has reached its own outcome, ending in stage `stop-background-monitor`'s.

**Ordering:** `worker-lifecycle` is graph-shaped (independent event-triggered entry points, not a single pipeline) — source/behavior_id order is used as the defensible default per the run-wide note above, not a proven chain.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
