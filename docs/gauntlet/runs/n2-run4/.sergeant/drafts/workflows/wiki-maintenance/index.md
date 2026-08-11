---
kind: workflow
name: wiki-maintenance
status: draft
version: 1
description: >-
  Trigger: a coordinator runs or regenerates a wiki daily digest.
  Outcome: a scheduling task cannot be marked done on the basis of installation alone, only on verified successful execution.
  Completion: Every member stage below has reached its own outcome, ending in stage `schedule-wiki-digest`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# wiki-maintenance

Established by `BU-0816` (`skills/wiki/SKILL.md (skills/wiki/SKILL.md L7-9)`): The wiki skill is loaded only for explicit wiki-maintenance requests (ingest, backfill, regenerate, inspect, or change wiki output), never for routine dispatch, notification, or cleanup, which write automatic captures without coordinator action.

**Trigger:** a coordinator runs or regenerates a wiki daily digest

**Outcome:** a scheduling task cannot be marked done on the basis of installation alone, only on verified successful execution

**Completion condition:** every member stage below has reached its own outcome, ending in stage `schedule-wiki-digest`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
