---
kind: workflow
name: task-intake-and-route
status: published
version: 2
description: >-
  The standing entry procedure every task passes through before any implementation workflow starts: it turns a user request into a chosen, scoped execution mode.
tags:
  - intake
  - routing
  - orchestration
---

# Task Intake and Route

Six-stage actor-only workflow (N1 reference corpus, `docs/gauntlet/contracts/N1.md`,
candidate **W5**) that is the standing entry procedure every task passes
through before any implementation workflow starts: it turns a user request
into a chosen, scoped execution mode. Use when: Any task the user brings.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Full behavior-unit citations and the N1 adjudication
record (A4, folding `02-check-queue`/`04-reconcile-state`/`07-monitor`
into the judgment stages they precede) live in the archived provenance
copy, `docs/gauntlet/promoted-provenance/task-intake-and-route.md`, per
`docs/icm/promotion-spec-2026-08-11.md`.
