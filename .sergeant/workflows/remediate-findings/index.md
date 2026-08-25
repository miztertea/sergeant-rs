---
kind: workflow
name: remediate-findings
status: published
version: 1
edition: 0.2.1
description: >-
  Consume an approved typed finding set and account for every finding —
  accepted, rejected, superseded, or unverifiable — with the fixes
  themselves re-attacked.
tags:
  - remediation
  - findings
  - review
---

# Remediate Findings

Six-stage actor-only workflow that ingests a typed finding set a human has
authorized acting on, verifies each finding against current state,
disposes every one of them (accepted/rejected/superseded/unverifiable),
implements fixes for accepted findings only, re-attacks those fix commits,
and closes with a disposition matrix that accounts for every ingested id.
Use when: a review (most often `review-change`) has produced a finding set
and a human has authorized acting on it.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. This package's derivation and the owner rulings
behind it are dev-corpus provenance, kept in this project's private
development record, not shipped here.
