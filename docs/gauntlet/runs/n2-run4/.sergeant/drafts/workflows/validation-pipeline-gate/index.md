---
kind: workflow
name: validation-pipeline-gate
status: draft
version: 1
description: >-
  Trigger: a dispatched worker reaches readiness.
  Outcome: the run only advances through an explicit pipeline-automation tool, never spontaneously.
  Completion: Every member stage below has reached its own outcome, ending in stage `monitor-active-run`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# validation-pipeline-gate

**Trigger:** a dispatched worker reaches readiness

**Outcome:** the run only advances through an explicit pipeline-automation tool, never spontaneously

**Completion condition:** every member stage below has reached its own outcome, ending in stage `monitor-active-run`'s.

**Ordering:** `validation-pipeline-gate` is graph-shaped (independent event-triggered entry points, not a single pipeline) — source/behavior_id order is used as the defensible default per the run-wide note above, not a proven chain.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
