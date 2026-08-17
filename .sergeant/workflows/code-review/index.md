---
kind: workflow
name: code-review
status: published
version: 2
edition: 0.1.0
description: >-
  Review a diff on two parallel, non-contaminating axes — Standards and
Spec — via isolated sub-reviews, reported side by side.
tags:
  - review
  - quality
  - spec-compliance
---

Provenance for this template's rules (which behavior unit justifies each
rule, and its upstream source) lives in `sergeant-rs-workspace`'s
`knowledge/evidence/provenance/code-review.md` — provenance markers were
stripped from the shipped template content
below; the record of why each rule exists did not move with them.

# Code Review

Four-stage actor-only workflow (N1 reference corpus,
`docs/gauntlet/contracts/N1.md`; `reference-corpus/synthesis.md` §1,
candidate **W24**; revised at ICM-R2) that reviews a diff on two parallel,
non-contaminating axes — Standards and Spec — via isolated sub-reviews,
reported side by side. Use when: a diff needs review before merge (invoked
directly or delegated from `worker-mission`/`implement`).

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, `docs/icm/promotion-spec-2026-08-11.md` plus the archived
`docs/gauntlet/promoted-provenance/code-review.md` for the prior revision's
full behavior-unit citations and promotion record, and
`docs/gauntlet/runs/icm-r2/code-review/adjudication-draft.md` for this
revision's package-adjudication record.
