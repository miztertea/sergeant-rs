---
kind: workflow
name: deliver-external-callback
status: draft
version: 2
description: >-
  Durable at-least-once notification to a registered external consumer.
tags:
  - notification
  - callback
  - delivery
---

# Deliver External Callback

Draft workflow candidate (N1 reference corpus, not admitted procedure —
see `docs/icm/convention.md` §2). Use when: A Work reaches a needs-input/blocked/failed/done transition and a registered consumer exists.

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and `provenance.md` for the full behavior-unit citations.
