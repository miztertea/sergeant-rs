---
kind: workflow
name: notify-primary-session
status: draft
version: 1
description: >-
  Trigger: the notify step is invoked.
  Outcome: a durable, searchable activity record exists for every update regardless of the transport outcome.
  Completion: Every member stage below has reached its own outcome, ending in stage `capture-wiki-activity`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# notify-primary-session

**Trigger:** the notify step is invoked

**Outcome:** a durable, searchable activity record exists for every update regardless of the transport outcome

**Completion condition:** every member stage below has reached its own outcome, ending in stage `capture-wiki-activity`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
