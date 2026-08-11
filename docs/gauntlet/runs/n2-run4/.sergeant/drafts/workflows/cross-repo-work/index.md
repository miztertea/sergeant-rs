---
kind: workflow
name: cross-repo-work
status: draft
version: 1
description: >-
  Trigger: a requested outcome is being decomposed across repositories.
  Outcome: completion claims require every owning repo to individually be terminal or explicitly blocked, not merely a subset.
  Completion: Every member stage below has reached its own outcome, ending in stage `reconcile-cross-repo-outcome`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# cross-repo-work

**Trigger:** a requested outcome is being decomposed across repositories

**Outcome:** completion claims require every owning repo to individually be terminal or explicitly blocked, not merely a subset

**Completion condition:** every member stage below has reached its own outcome, ending in stage `reconcile-cross-repo-outcome`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
