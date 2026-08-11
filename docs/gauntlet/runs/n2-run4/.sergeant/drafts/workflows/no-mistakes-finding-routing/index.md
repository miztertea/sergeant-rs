---
kind: workflow
name: no-mistakes-finding-routing
status: draft
version: 1
description: >-
  Trigger: the validation pipeline surfaces an actionable finding.
  Outcome: remediation converges to one worker per root cause, is rechecked before merge, and escalates to a human after two unsuccessful cycles rather than looping indefinitely.
  Completion: Every member stage below has reached its own outcome, ending in stage `remediate-grouped-findings`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# no-mistakes-finding-routing

**Trigger:** the validation pipeline surfaces an actionable finding

**Outcome:** remediation converges to one worker per root cause, is rechecked before merge, and escalates to a human after two unsuccessful cycles rather than looping indefinitely

**Completion condition:** every member stage below has reached its own outcome, ending in stage `remediate-grouped-findings`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
