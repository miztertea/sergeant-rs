---
kind: workflow
name: design-it-twice
status: draft
version: 1
description: >-
  Trigger: the user wants to explore alternative interfaces for a chosen deepening candidate.
  Outcome: the user receives a structured, sequential presentation and a comparison along three named axes.
  Completion: Every member stage below has reached its own outcome, ending in stage `compare-and-recommend`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# design-it-twice

**Trigger:** the user wants to explore alternative interfaces for a chosen deepening candidate

**Outcome:** the user receives a structured, sequential presentation and a comparison along three named axes

**Completion condition:** every member stage below has reached its own outcome, ending in stage `compare-and-recommend`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
