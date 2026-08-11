---
kind: workflow
name: wayfinder
status: draft
version: 1
description: >-
  Trigger: an effort's map Notes section states an override.
  Outcome: ticket selection is cheap and deterministic, and claimed before any work starts.
  Completion: Every member stage below has reached its own outcome, ending in stage `work-through-map-session`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# wayfinder

Established by `BU-0999` (`.agents/skills/wayfinder/SKILL.md (.agents/skills/wayfinder/SKILL.md L13-13)`): By default, wayfinder produces decisions rather than deliverables: each ticket resolves a decision, and the map is considered done once nothing is left to decide before someone goes and does the thing.

**Trigger:** an effort's map Notes section states an override

**Outcome:** ticket selection is cheap and deterministic, and claimed before any work starts

**Completion condition:** every member stage below has reached its own outcome, ending in stage `work-through-map-session`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
