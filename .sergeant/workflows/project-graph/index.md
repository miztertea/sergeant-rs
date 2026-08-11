---
kind: workflow
name: project-graph
status: published
version: 2
description: >-
  Produce exactly one merged, published graph per project, outside every source repository, usable for architecture questions.
tags:
  - project
  - graph
  - architecture
---

# Project Graph

Two-stage actor workflow (N1 reference corpus, `docs/gauntlet/contracts/
N1.md`, candidate **W2**) that produces exactly one merged, published
graph per project, outside every source repository, usable for
architecture questions. Use when: Architecture work needs whole-project
structure, or the operator asks for a graph/refresh.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Curated for promotion per
`docs/icm/promotion-spec-2026-08-11.md`; the full behavior-unit citations
live in the archived `docs/gauntlet/promoted-provenance/project-graph.md`.
