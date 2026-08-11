---
kind: workflow
name: prototype
status: draft
version: 1
description: >-
  Trigger: the user wants a throwaway prototype to answer a design question.
  Outcome: the user can independently explore variants, and cross-variant preferences are captured as signal rather than treated as noise.
  Completion: Every member stage below has reached its own outcome, ending in stage `drive-ui-prototype`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# prototype

**Trigger:** the user wants a throwaway prototype to answer a design question

**Outcome:** the user can independently explore variants, and cross-variant preferences are captured as signal rather than treated as noise

**Completion condition:** every member stage below has reached its own outcome, ending in stage `drive-ui-prototype`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
