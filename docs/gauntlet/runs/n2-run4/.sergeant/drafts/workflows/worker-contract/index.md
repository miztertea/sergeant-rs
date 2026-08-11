---
kind: workflow
name: worker-contract
status: draft
version: 1
description: >-
  Trigger: a worker begins a phase of implementation work.
  Outcome: done is only ever reported once every gate has genuinely passed, and failed carries an exact, specific reason rather than a generic one.
  Completion: Every member stage below has reached its own outcome, ending in stage `report-terminal-status`'s.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# worker-contract

**Trigger:** a worker begins a phase of implementation work

**Outcome:** done is only ever reported once every gate has genuinely passed, and failed carries an exact, specific reason rather than a generic one

**Completion condition:** every member stage below has reached its own outcome, ending in stage `report-terminal-status`'s.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
