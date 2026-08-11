---
kind: workflow
name: research
status: draft
version: 1
description: >-
  Trigger: a topic needs primary-source research.
  Outcome: research proceeds in parallel with the invoking actor's other work instead of blocking it.
  Completion: Every member stage below has reached its own outcome, ending in stage `conduct-research`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# research

**Trigger:** a topic needs primary-source research

**Outcome:** research proceeds in parallel with the invoking actor's other work instead of blocking it

**Completion condition:** every member stage below has reached its own outcome, ending in stage `conduct-research`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
