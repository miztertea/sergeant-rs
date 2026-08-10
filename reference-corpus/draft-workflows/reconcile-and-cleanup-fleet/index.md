---
kind: workflow
name: reconcile-and-cleanup-fleet
status: draft
version: 1
description: >-
  Retire a completed task's surfaces and state.
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
