---
kind: workflow
name: reconcile-and-cleanup-fleet
status: published
version: 2
description: >-
  Require every targeted repo safely terminal and its task verifiably
  closed, then reconcile fleet status, verify ownership and handshake
  acknowledgement, remove each repo's surface, and retire whole-task state
  once every repo is done. One actor stage (N1 adjudication A4 folded five
  deterministic-machinery stages into it, including the two mutating
  stages moved in from monitor-fleet under A7).
tags:
  - fleet
  - cleanup
  - lifecycle
---

# Reconcile and Cleanup Fleet

One-stage actor-only workflow (N1 reference corpus,
`docs/gauntlet/contracts/N1.md`, candidate **W15**) that requires every
targeted repo safely terminal and its task verifiably closed, then
reconciles fleet status, re-verifies ownership, confirms and seals
handshake acknowledgement, removes each repo's surface, and retires
whole-task state once every repo is done. Use when: A task's repos are
believed terminal and the operator (or an automated sweep) requests
cleanup.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Curated for promotion per
`docs/icm/promotion-spec-2026-08-11.md`; the full behavior-unit citations
live in the archived
`docs/gauntlet/promoted-provenance/reconcile-and-cleanup-fleet.md`.
