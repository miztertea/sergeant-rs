---
kind: workflow
name: reconcile-and-cleanup-fleet
status: draft
version: 1
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

Draft workflow candidate (N1 reference corpus, not admitted procedure —
see `docs/icm/convention.md` §2). Use when: A task's repos are believed terminal and the operator (or an automated sweep) requests cleanup.

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and `provenance.md` for the full behavior-unit citations.
