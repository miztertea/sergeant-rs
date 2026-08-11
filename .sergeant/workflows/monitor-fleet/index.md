---
kind: workflow
name: monitor-fleet
status: published
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

One-stage actor-only workflow (N1 reference corpus, candidate **W13**
`monitor-fleet`, `docs/gauntlet/contracts/N1.md`) that observes fleet state
without mutating it: a bounded, versioned, strictly read-only snapshot plus
a per-worker liveness evaluation, interpreted by this workflow's single
actor stage. Use when: an operator or another workflow (dispatch's
`80-monitor`) needs a live view of the fleet.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Full behavior-unit citations are archived at
`docs/gauntlet/promoted-provenance/monitor-fleet.md`; the curation act
itself is recorded at `docs/icm/promotion-spec-2026-08-11.md`.
