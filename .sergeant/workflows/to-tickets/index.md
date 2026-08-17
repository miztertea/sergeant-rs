---
kind: workflow
name: to-tickets
status: published
version: 2
description: >-
  Break a plan, spec, investigation, findings register, PR, or conversation into dependency-aware tracer-bullet work.
tags:
  - tickets
  - planning
  - decomposition
---

# To Tickets

Provenance for this template's rules (which behavior unit justifies each
rule, and its upstream source) lives in `sergeant-rs-workspace`'s
`knowledge/evidence/provenance/to-tickets.md` — this package's
`BU-####` citations and `reference/sergeant-upstream/` paths were stripped
from the shipped template content below; the record of why each rule
exists did not move with them.

Four-stage actor-only workflow (N1 reference corpus,
`docs/gauntlet/contracts/N1.md`, candidate **W32**) that breaks a plan,
spec, investigation, findings register, PR, or conversation into
dependency-aware tracer-bullet work. Use when: The user says "to tickets", "create issues", "create td tasks", "make epics", or asks to break something into work.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Curated for promotion per
`docs/icm/promotion-spec-2026-08-11.md`; the full behavior-unit citations
live in the archived `docs/gauntlet/promoted-provenance/to-tickets.md`.
