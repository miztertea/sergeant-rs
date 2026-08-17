---
kind: workflow
name: cross-repo-work
status: published
version: 2
description: >-
  Decompose a requested outcome across repositories and define delivery order: produce a plan in which every required behavior has exactly one owning repository, an acyclic dependency position, a brief, and acceptance evidence — before any dispatch happens.
tags:
  - multi-repository
  - planning
---

# Cross-Repo Work

Provenance for this package's rules (the BU-#### citations and
`reference/sergeant-upstream/` sources previously inline here) is recorded
in `sergeant-rs-workspace`'s `knowledge/evidence/provenance/cross-repo-work.md`.

Five-stage actor-only workflow (N1 reference corpus, candidate **W7**
`cross-repo-work`, `docs/gauntlet/contracts/N1.md`) that decomposes a
requested outcome across the repositories that own it and defines
delivery order: every required behavior gets exactly one owning
repository, an acyclic dependency position, a brief, and acceptance
evidence, before any dispatch happens. Use when: Resolved project context
shows more than one repository owns the requested outcome (not merely
that the project has several repos).

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Full behavior-unit citations are archived at
`docs/gauntlet/promoted-provenance/cross-repo-work.md`; the curation act
itself is recorded at `docs/icm/promotion-spec-2026-08-11.md`.
