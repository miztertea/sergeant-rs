---
kind: workflow
name: dispatch-mode
status: draft
version: 1
description: >-
  Trigger: dispatch mode has been selected.
  Outcome: a silent model substitution the account was never entitled to is durably surfaced even though the mission itself completed successfully.
  Completion: Every member stage below has reached its own outcome, ending in stage `detect-model-substitution`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# dispatch-mode

**Trigger:** dispatch mode has been selected

**Outcome:** a silent model substitution the account was never entitled to is durably surfaced even though the mission itself completed successfully

**Completion condition:** every member stage below has reached its own outcome, ending in stage `detect-model-substitution`'s.

**Ordering:** `dispatch-mode` is graph-shaped (independent event-triggered entry points, not a single pipeline) — source/behavior_id order is used as the defensible default per the run-wide note above, not a proven chain.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
