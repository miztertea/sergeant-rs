---
kind: workflow
name: research
status: published
version: 2
edition: 0.1.0
description: >-
  Investigate a question against high-trust primary sources and capture
the findings as a Markdown file in the repo.
tags:
  - research
  - investigation
  - documentation
---

# Research

Provenance for this template's rules (which behavior unit justifies each
rule, and its upstream source) lives in `sergeant-rs-workspace`'s
`knowledge/evidence/provenance/research.md` — this package's provenance
markers were stripped from the
shipped template content below; the record of why each rule exists did not
move with them.

Investigates a question against high-trust primary sources and captures
the findings as a single cited Markdown file in the repository (N1
reference corpus, candidate **W27**, `docs/gauntlet/contracts/N1.md`). Use
when: a topic needs to be researched, or docs/API facts need gathering,
and reading legwork is delegated.

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and `docs/icm/promotion-spec-2026-08-11.md` plus the archived
citation trail at `docs/gauntlet/promoted-provenance/research.md` for the
full behavior-unit citations.
