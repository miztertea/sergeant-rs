---
kind: workflow
name: respond-to-worker
status: published
version: 2
description: >-
  A blocked/needs-input/waiting/orphaned worker is durably given exactly one decision, applies it exactly once, and returns to forward progress.
tags:
  - worker
  - escalation
  - recovery
---

# Respond to Worker

Two-stage actor-only workflow (N1 reference corpus, candidate **W10**
`respond-to-worker`, `docs/gauntlet/contracts/N1.md`) that durably gives a
blocked/needs-input/waiting/orphaned worker exactly one decision, applies
it exactly once, and returns it to forward progress. Use when: A worker
has published an escalation and a human decision exists.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Full behavior-unit citations are archived at
`docs/gauntlet/promoted-provenance/respond-to-worker.md`; the curation act
itself is recorded at `docs/icm/promotion-spec-2026-08-11.md`.
