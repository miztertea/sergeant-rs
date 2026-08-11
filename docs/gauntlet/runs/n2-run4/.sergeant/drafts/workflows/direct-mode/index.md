---
kind: workflow
name: direct-mode
status: draft
version: 1
description: >-
  Trigger: direct mode is active and an edit is about to be made.
  Outcome: delivery is only declared complete once PR, CI, review, and merge authorization are all satisfied.
  Completion: Every member stage below has reached its own outcome, ending in stage `deliver`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# direct-mode

**Trigger:** direct mode is active and an edit is about to be made

**Outcome:** delivery is only declared complete once PR, CI, review, and merge authorization are all satisfied

**Completion condition:** every member stage below has reached its own outcome, ending in stage `deliver`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
