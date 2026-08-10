---
kind: workflow
name: respond-to-worker
status: draft
version: 1
description: >-
  A blocked/needs-input/waiting/orphaned worker is durably given exactly one decision, applies it exactly once, and returns to forward progress.
tags:
  - worker
  - escalation
  - recovery
---

# Respond to Worker

Draft workflow candidate (N1 reference corpus, not admitted procedure —
see `docs/icm/convention.md` §2). Use when: A worker has published an escalation and a human decision exists.

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and `provenance.md` for the full behavior-unit citations.
