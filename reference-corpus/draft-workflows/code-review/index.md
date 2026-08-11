---
kind: workflow
name: code-review
status: draft
version: 1
description: >-
  Review a diff on two parallel, non-contaminating axes — Standards and Spec — via isolated sub-reviews, reported side by side.
tags:
  - review
  - quality
  - spec-compliance
---

# Code Review

Draft workflow candidate (N1 reference corpus, not admitted procedure —
see `docs/icm/convention.md` §2). Use when: A diff needs review before merge (invoked directly or delegated from `worker-mission`/`implement`).

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and `provenance.md` for the full behavior-unit citations.
