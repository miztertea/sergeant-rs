---
kind: workflow
name: drain-fleet
status: published
version: 2
description: >-
  A cooperative, bounded, non-destructive admission block: refuse new work without terminating anything already running.
tags:
  - fleet
  - admission-control
  - drain
---

# Drain Fleet

Single-stage workflow (N1 reference corpus, `docs/gauntlet/contracts/N1.md`;
candidate **W12** `drain-fleet`, `reference-corpus/synthesis.md` §1) that
sets a cooperative, scope-qualified admission drain, awaits
bounded convergence of in-scope workers, force-stops what cooperative
draining left unresolved (refused unless a drain is already active,
requiring explicit confirmation or dry-run), and lifts the drain. Use
when: An operator needs to freeze new stage/turn admission — globally or
for one project — before a disruptive operation.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Behavior-unit citations and the N1 adjudication
record live in `docs/icm/promotion-spec-2026-08-11.md` and the archived
`docs/gauntlet/promoted-provenance/drain-fleet.md`.
