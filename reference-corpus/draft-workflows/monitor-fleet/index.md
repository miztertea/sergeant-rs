---
kind: workflow
name: monitor-fleet
status: draft
version: 2
description: >-
  Observe fleet state without mutating it: a bounded, versioned, strictly
  read-only snapshot plus liveness evaluation, interpreted by the workflow's
  single actor stage. Mutating reconciliation and background-watch lifecycle
  moved to reconcile-and-cleanup-fleet at N1 adjudication A7; the snapshot
  and liveness computations folded into one judgment-bearing stage at N1
  adjudication A4.
tags:
  - fleet
  - observability
  - read-only
---

# Monitor Fleet

Draft workflow candidate (N1 reference corpus, not admitted procedure —
see `docs/icm/convention.md` §2). Use when: An operator or another workflow (dispatch's `80-monitor`) needs a live view of the fleet.

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and `provenance.md` for the full behavior-unit citations.
