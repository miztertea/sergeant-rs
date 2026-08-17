---
kind: workflow
name: validate-and-ship
status: published
version: 2
description: >-
  The single final shipping boundary: validate a committed change through
the pipeline to a terminal outcome, routing every finding, without the
validating actor ever editing the code.
tags:
  - shipping-gate
  - validation
  - no-mistakes
---

# Validate and Ship (no-mistakes)

Provenance for this template's rules (which behavior unit justifies each
rule, and its upstream source) lives in `sergeant-rs-workspace`'s
`knowledge/evidence/provenance/validate-and-ship.md` — this package's
provenance markers were stripped
from the shipped template content below; the record of why each rule
exists did not move with them.

Seven-stage actor-only workflow (N1 reference corpus, candidate **W18**
`validate-and-ship`, `docs/gauntlet/contracts/N1.md`) that is the single
final shipping boundary: validate a committed change through the pipeline
to a terminal outcome, routing every finding, without the validating actor
ever editing the code. Use when: Implementation, native tests, lint and
independent review are complete and the coordinator has reached the
approved shipping boundary.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Full behavior-unit citations are archived at
`docs/gauntlet/promoted-provenance/validate-and-ship.md`; the curation act
itself is recorded at `docs/icm/promotion-spec-2026-08-11.md`.
