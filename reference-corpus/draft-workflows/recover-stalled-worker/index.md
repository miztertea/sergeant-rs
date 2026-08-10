---
kind: workflow
name: recover-stalled-worker
status: draft
version: 1
description: >-
  One bounded recovery attempt for a stalled worker: converge on a replacement or escalate — never guess.
tags:
  - worker
  - recovery
  - stall
---

# Recover Stalled Worker

Draft workflow candidate (N1 reference corpus, not admitted procedure —
see `docs/icm/convention.md` §2). Use when: A worker is `in_progress` with a stall classification recorded by the watcher.

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and `provenance.md` for the full behavior-unit citations.
