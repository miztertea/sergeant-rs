---
kind: workflow
name: route-review-findings
status: published
version: 2
description: >-
  Turn independent review output into tracked work and a gate.
tags:
  - review
  - findings
  - routing
---

# Route Review Findings

Single-stage admitted workflow (`docs/gauntlet/contracts/N1.md`, candidate
**W16**) that routes independent review findings to tracked work and a
blocking gate. Use when: A review pass (worker-mission's
`30-independent-review`, or code-review) has produced findings.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Full behavior-unit citations and the N1 adjudication
record (A4, folding the original four stages into this workflow's sole
stage) live in the archived provenance copy,
`docs/gauntlet/promoted-provenance/route-review-findings.md`, per
`docs/icm/promotion-spec-2026-08-11.md`.
