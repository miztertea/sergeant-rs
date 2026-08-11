---
kind: workflow
name: implement
status: published
version: 2
description: >-
  Implement a piece of work from a spec or ticket set, explicit-invocation-only.
tags:
  - implementation
  - explicit-invocation
---

# Implement

Two-stage actor-only workflow (N1 reference corpus,
`docs/gauntlet/contracts/N1.md`; `reference-corpus/synthesis.md` §1,
candidate **W23** `implement`) that implements a piece of work from a spec
or ticket set. Explicit-invocation-only (BU-P2-051) — this workflow must
never be auto-loaded merely because the task looks like implementation.
Use when: Explicitly invoked to implement a defined piece of work (never
auto-loaded).

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Curated for promotion per
`docs/icm/promotion-spec-2026-08-11.md`; the full behavior-unit citations
live in the archived `docs/gauntlet/promoted-provenance/implement.md`.
