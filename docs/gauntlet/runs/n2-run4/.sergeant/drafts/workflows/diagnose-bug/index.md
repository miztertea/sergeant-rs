---
kind: workflow
name: diagnose-bug
status: draft
version: 1
description: >-
  Trigger: diagnosing any hard bug.
  Outcome: the bug is not declared done until all five completion conditions hold.
  Completion: Every member stage below has reached its own outcome, ending in stage `declare-bug-fixed`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# diagnose-bug

**Trigger:** diagnosing any hard bug

**Outcome:** the bug is not declared done until all five completion conditions hold

**Completion condition:** every member stage below has reached its own outcome, ending in stage `declare-bug-fixed`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
