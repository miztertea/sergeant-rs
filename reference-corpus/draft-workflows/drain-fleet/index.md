---
kind: workflow
name: drain-fleet
status: draft
version: 1
description: >-
  A cooperative, bounded, non-destructive admission block: refuse new work without terminating anything already running.
tags:
  - fleet
  - admission-control
  - drain
---

# Drain Fleet

Draft workflow candidate (N1 reference corpus, not admitted procedure —
see `docs/icm/convention.md` §2). Use when: An operator needs to freeze new stage/turn admission — globally or for one project — before a disruptive operation.

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and `provenance.md` for the full behavior-unit citations.
