---
kind: workflow
name: deliver-external-callback
status: published
version: 2
description: >-
  Durable at-least-once notification to a registered external consumer.
tags:
  - notification
  - callback
  - delivery
---

# Deliver External Callback

Single-stage actor-only workflow (N1 reference corpus,
`docs/gauntlet/contracts/N1.md`, candidate **W17**) providing durable
at-least-once notification to a registered external consumer. Use when: A
Work reaches a needs-input/blocked/failed/done transition and a registered
consumer exists.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Curated for promotion per
`docs/icm/promotion-spec-2026-08-11.md`; the full behavior-unit citations
live in the archived `docs/gauntlet/promoted-provenance/deliver-external-callback.md`.
