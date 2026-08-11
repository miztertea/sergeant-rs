---
kind: workflow
name: sergeant-setup
status: draft
version: 1
description: >-
  Trigger: a checklist step is reached.
  Outcome: a successful Graphify run is verified by the presence of both named output files.
  Completion: Every member stage below has reached its own outcome, ending in stage `phase9-graphify-init`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# sergeant-setup

**Trigger:** a checklist step is reached

**Outcome:** a successful Graphify run is verified by the presence of both named output files

**Completion condition:** every member stage below has reached its own outcome, ending in stage `phase9-graphify-init`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
