---
kind: workflow
name: worker-mission
status: published
version: 2
description: >-
  From a rendered brief, produce a merged-ready change with evidence — the contract a dispatched worker delivers against.
tags:
  - worker
  - software-change
  - implementation
---

# Worker Mission (software-change)

Provenance for this template's rules (which behavior unit justifies each
rule, and its upstream source) lives in `sergeant-rs-workspace`'s
`knowledge/evidence/provenance/worker-mission.md` — this package's
`BU-####` citations and `reference/sergeant-upstream/` paths were stripped
from the shipped template content below; the record of why each rule
exists did not move with them.

Four-stage admitted workflow (N1 reference corpus,
`docs/gauntlet/contracts/N1.md`; `reference-corpus/synthesis.md` §1,
candidate **W9** `worker-mission`) that, from a rendered brief, produces a
merged-ready change with evidence — the contract a dispatched worker
delivers against. Use when: A worker starts against a rendered brief.

`20-implement` delegates to whichever of **diagnose-bug, prototype, tdd,
implement, or deepen-module** was selected at `10-triage-and-route`
(context composition today, not true nested-workflow invocation —
`docs/icm/convention.md` §4). All five targets are published in this
library.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Full behavior-unit citations and the N1 adjudication
record (A4, folding six extracted stages into four) live in the archived
provenance copy, `docs/gauntlet/promoted-provenance/worker-mission.md`,
per `docs/icm/promotion-spec-2026-08-11.md`.
