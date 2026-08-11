---
kind: workflow
name: implement
status: draft
version: 1
description: >-
  Trigger: implementation work has pre-agreed seams.
  Outcome: the work is durably recorded in version control.
  Completion: Every member stage below has reached its own outcome, ending in stage `commit-implementation`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# implement

**Trigger:** implementation work has pre-agreed seams

**Outcome:** the work is durably recorded in version control

**Completion condition:** every member stage below has reached its own outcome, ending in stage `commit-implementation`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
