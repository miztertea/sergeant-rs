---
kind: workflow
name: load-project
status: draft
version: 1
description: >-
  Trigger: the project name for a task is not already known exactly.
  Outcome: the edit is validated against resolved context output, not just YAML syntax validity.
  Completion: Every member stage below has reached its own outcome, ending in stage `edit-and-validate-project`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# load-project

**Trigger:** the project name for a task is not already known exactly

**Outcome:** the edit is validated against resolved context output, not just YAML syntax validity

**Completion condition:** every member stage below has reached its own outcome, ending in stage `edit-and-validate-project`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
