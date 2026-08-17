---
kind: workflow
name: deepen-module
status: published
version: 1
description: >-
  Turn a shallow module into a deep one at a deliberately chosen seam.
tags:
  - module-design
  - architecture
  - seams
---

# Deepen Module

Provenance for this package's rules (the BU-#### citations and
`reference/sergeant-upstream/` sources previously inline here) is recorded
in `sergeant-rs-workspace`'s `knowledge/evidence/provenance/deepen-module.md`.

Three-stage actor-only workflow (`docs/gauntlet/contracts/N1.md`, candidate
**W25**) that turns a shallow module into a deep one at a deliberately
chosen seam: classify a dependency cluster's coupling, generate and compare
at least three independently designed interfaces, then replace the old
shallow-module tests with tests at the new interface.

Use when: a module's interface needs redesign, or a port/adapter decision
needs to be made deliberately rather than by default.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Promoted per `docs/icm/promotion-spec-2026-08-11.md`;
the full behavior-unit citation trail is archived at
`docs/gauntlet/promoted-provenance/deepen-module.md`.
