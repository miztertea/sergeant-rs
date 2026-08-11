---
kind: workflow
name: standard-workflow
status: draft
version: 1
description: >-
  Trigger: a task is brought to the session.
  Outcome: cleanup runs only after terminal state and evidence preservation are verified.
  Completion: Every member stage below has reached its own outcome, ending in stage `deliver-and-cleanup`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# standard-workflow

**Trigger:** a task is brought to the session

**Outcome:** cleanup runs only after terminal state and evidence preservation are verified

**Completion condition:** every member stage below has reached its own outcome, ending in stage `deliver-and-cleanup`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
