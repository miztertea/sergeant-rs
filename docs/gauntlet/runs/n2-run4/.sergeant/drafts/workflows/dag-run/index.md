---
kind: workflow
name: dag-run
status: draft
version: 1
description: >-
  Trigger: a DAG stage is defined.
  Outcome: the hook fails loudly rather than dispatching work it cannot later attribute to the DAG runner run.
  Completion: Every member stage below has reached its own outcome, ending in stage `run-dispatch-hook`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# dag-run

**Trigger:** a DAG stage is defined

**Outcome:** the hook fails loudly rather than dispatching work it cannot later attribute to the DAG runner run

**Completion condition:** every member stage below has reached its own outcome, ending in stage `run-dispatch-hook`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
