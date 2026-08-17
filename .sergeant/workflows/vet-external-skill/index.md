---
kind: workflow
name: vet-external-skill
status: published
version: 2
edition: 0.1.0
description: >-
  Vet an external skill through a fixed sequence before adopting it, and
keep already-adopted skills updated through the same discipline.
tags:
  - skills
  - vetting
  - supply-chain
---

# Vet External Skill

Provenance for this template's rules (which behavior unit justifies each
rule, and its upstream source) lives in `sergeant-rs-workspace`'s
`knowledge/evidence/provenance/vet-external-skill.md` — this package's
provenance markers were stripped
from the shipped template content below; the record of why each rule
exists did not move with them.

Seven-stage actor-only workflow (N1 reference corpus,
`docs/gauntlet/contracts/N1.md`, candidate **W34**) that vets an external
skill through a fixed sequence before adopting it, and keeps
already-adopted skills updated through the same discipline. Use when:
before adopting an external skill, or when an adopted skill needs
updating.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Curated for promotion per
`docs/icm/promotion-spec-2026-08-11.md`; the full behavior-unit citations
live in the archived
`docs/gauntlet/promoted-provenance/vet-external-skill.md`.
