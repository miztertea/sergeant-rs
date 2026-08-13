---
kind: workflow
name: load-project
status: published
version: 3
description: >-
  Establish, before any mutation, which repositories own the requested outcome, where they are, what instructions govern them, and what state they are in.
tags:
  - project
  - context-resolution
  - installation
---

# Load Project

Three-stage actor-only workflow (N1 reference corpus,
`docs/gauntlet/contracts/N1.md`, candidate **W1**) that establishes, before
any mutation, which repositories own the requested outcome, where they are,
what instructions govern them, and what state they are in. Use when: A
project is named, registered, edited, synced, or listed; or repository
ownership is not already established.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Curated for promotion per
`docs/icm/promotion-spec-2026-08-11.md`; the full behavior-unit citations
live in the archived `docs/gauntlet/promoted-provenance/load-project.md`.
