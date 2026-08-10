---
kind: workflow
name: monitor-fleet
status: draft
version: 1
description: >-
  Observe fleet state without mutating it: a bounded, versioned, strictly
  read-only snapshot plus liveness evaluation. Mutating reconciliation and
  background-watch lifecycle moved to reconcile-and-cleanup-fleet at N1
  adjudication A7.
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
