---
kind: workflow
name: wake-and-resume
status: draft
version: 1
description: >-
  Resume a waiting worker when its durable condition is met.
tags:
  - worker
  - scheduling
  - waiting
---

# Wake and Resume

Draft workflow candidate (N1 reference corpus, not admitted procedure —
see `docs/icm/convention.md` §2). Use when: A worker is in the `waiting` state with a recorded wake condition.

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and `provenance.md` for the full behavior-unit citations.
