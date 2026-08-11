---
kind: workflow
name: to-spec
status: draft
version: 1
description: >-
  Trigger: the current state of the codebase has not already been explored.
  Outcome: the spec is immediately actionable in the tracker without a further triage pass.
  Completion: Every member stage below has reached its own outcome, ending in stage `publish-spec`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# to-spec

**Trigger:** the current state of the codebase has not already been explored

**Outcome:** the spec is immediately actionable in the tracker without a further triage pass

**Completion condition:** every member stage below has reached its own outcome, ending in stage `publish-spec`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
