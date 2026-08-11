---
kind: workflow
name: fleet-status-listing
status: draft
version: 1
description: >-
  Trigger: the fleet-watch loop's listing view is invoked.
  Outcome: the listing reports an accurate per-status repo breakdown without overstating which tasks are currently active.
  Completion: The sole inferred stage `list-fleet-status` has reached its outcome (design inference — see `provenance.md`).
tags:
  - draft
  - repo-to-icm-n2-run4
---

# fleet-status-listing

No `stage`-rung record in this run's corpus carries `workflow=fleet-status-listing` directly; this candidate's evidence is its workflow-level helpers alone (see `provenance.md`).

**Trigger:** the fleet-watch loop's listing view is invoked

**Outcome:** the listing reports an accurate per-status repo breakdown without overstating which tasks are currently active

**Completion condition:** the sole inferred stage `list-fleet-status` has reached its outcome (design inference — see `provenance.md`).

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
