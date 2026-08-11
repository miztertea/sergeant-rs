---
kind: workflow
name: code-review
status: published
version: 1
description: >-
  Review a diff on two parallel, non-contaminating axes — Standards and Spec — via isolated sub-reviews, reported side by side.
tags:
  - review
  - quality
  - spec-compliance
---

# Code Review

Five-stage actor-only workflow (N1 reference corpus,
`docs/gauntlet/contracts/N1.md`; `reference-corpus/synthesis.md` §1,
candidate **W24**) that reviews a diff on two parallel, non-contaminating
axes — Standards and Spec — via isolated sub-reviews, reported side by
side. Use when: a diff needs review before merge (invoked directly or
delegated from `worker-mission`/`implement`).

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and `docs/icm/promotion-spec-2026-08-11.md` plus the archived
`docs/gauntlet/promoted-provenance/code-review.md` for the full
behavior-unit citations and promotion record.
