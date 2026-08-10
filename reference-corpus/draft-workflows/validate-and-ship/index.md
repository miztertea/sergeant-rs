---
kind: workflow
name: validate-and-ship
status: draft
version: 1
description: >-
  The single final shipping boundary: validate a committed change through the pipeline to a terminal outcome, routing every finding, without the validating actor ever editing the code.
tags:
  - shipping-gate
  - validation
  - no-mistakes
---

# Validate and Ship (no-mistakes)

Draft workflow candidate (N1 reference corpus, not admitted procedure —
see `docs/icm/convention.md` §2). Use when: Implementation, native tests, lint and independent review are complete and the coordinator has reached the approved shipping boundary.

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and `provenance.md` for the full behavior-unit citations.
