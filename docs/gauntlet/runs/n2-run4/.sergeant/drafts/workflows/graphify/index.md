---
kind: workflow
name: graphify
status: draft
version: 1
description: >-
  Trigger: the graph-generation step is run for a project.
  Outcome: a failure leaves the previous graph output intact and cleans up its own temporary artifacts rather than leaving a half-swapped or missing output.
  Completion: Every member stage below has reached its own outcome, ending in stage `recover-from-failed-publish`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# graphify

**Trigger:** the graph-generation step is run for a project

**Outcome:** a failure leaves the previous graph output intact and cleans up its own temporary artifacts rather than leaving a half-swapped or missing output

**Completion condition:** every member stage below has reached its own outcome, ending in stage `recover-from-failed-publish`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
