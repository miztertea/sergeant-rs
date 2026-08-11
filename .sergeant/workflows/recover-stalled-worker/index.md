---
kind: workflow
name: recover-stalled-worker
status: published
version: 2
description: >-
  One bounded recovery attempt for a stalled worker: converge on a replacement or escalate — never guess.
tags:
  - worker
  - recovery
  - stall
---

# Recover Stalled Worker

Three-stage actor-only workflow (N1 reference corpus,
`docs/gauntlet/contracts/N1.md`, candidate **W11** `recover-stalled-worker`,
`reference-corpus/synthesis.md` §1) for one bounded recovery attempt on a
stalled worker: converge on a replacement or escalate — never guess. Use
when: A worker is `in_progress` with a stall classification recorded by the
watcher.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Behavior-unit citations and the N1 adjudication record
live in `docs/icm/promotion-spec-2026-08-11.md` and the archived
`docs/gauntlet/promoted-provenance/recover-stalled-worker.md`.
