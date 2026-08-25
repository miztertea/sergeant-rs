---
kind: workflow
name: review-change
status: published
version: 1
edition: 0.2.1
description: >-
  Review a diff that arrives from outside a Work — a colleague's PR, a
  merge candidate, a change someone else built — on four axes, and emit
  verified findings, not fixes.
tags:
  - review
  - panel
  - read-only
---

# Review Change

Six-stage, read-only actor workflow that pins the diff's fixed point,
locates the spec it should be judged against, panels it on four
non-contaminating axes, refutes what the panel raised, independently
verifies and severity-ranks every surviving finding, and reports the
typed finding set. Use when: a diff needs review before merge, arriving
from outside this Work (a colleague's PR, a merge candidate). The
reviewing actor never edits the code.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. This package's derivation and the owner rulings
behind it are dev-corpus provenance, kept in this project's private
development record, not shipped here.
