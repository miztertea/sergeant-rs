---
kind: workflow
name: diagnose-bug
status: published
version: 1
edition: 0.1.0
description: >-
  Reproduce, isolate, prove, remediate and verify a defect.
tags:
  - debugging
  - defect
  - investigation
---

Provenance for this template's rules (which behavior unit justifies each
rule, and its upstream source) lives in `sergeant-rs-workspace`'s
`knowledge/evidence/provenance/diagnose-bug.md` — provenance markers were
stripped from the shipped template content
below; the record of why each rule exists did not move with them.

# Diagnose Bug

Six-stage actor-only workflow (N1 reference corpus,
`sergeant-rs-workspace/knowledge/evidence/gauntlet/contracts/N1.md`, candidate **W20**) that reproduces,
isolates, proves, remediates and verifies a defect. Use when: "Diagnose"/
"debug this", or something reported broken, throwing, failing, slow.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Curated for promotion per
`docs/icm/promotion-spec-2026-08-11.md`; the full behavior-unit citations
live in the archived `sergeant-rs-workspace/knowledge/evidence/gauntlet/promoted-provenance/diagnose-bug.md`.
