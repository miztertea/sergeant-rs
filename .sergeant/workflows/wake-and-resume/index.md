---
kind: workflow
name: wake-and-resume
status: published
version: 2
description: >-
  Resume a waiting worker when its durable condition is met.
tags:
  - worker
  - scheduling
  - waiting
---

# Wake and Resume

Resumes a waiting worker when its durable wake condition is met (N1
reference corpus, candidate **W14**, `docs/gauntlet/contracts/N1.md`). Use
when: a worker is in the `waiting` state with a recorded wake condition.

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and `docs/icm/promotion-spec-2026-08-11.md` plus the archived
citation trail at `docs/gauntlet/promoted-provenance/wake-and-resume.md`
for the full behavior-unit citations.
