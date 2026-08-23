---
kind: workflow
name: implement-change
status: published
version: 1
edition: 0.2.1
description: >-
  Take an intent for a change to code, produce the change with its tests,
  attack it on four axes, refute the attack, fix only what survives,
  re-attack the fixes, and close with evidence tied to acceptance.
tags:
  - implementation
  - review
  - panel
---

# Implement Change

Nine-stage actor-only workflow that pins the change's revision and
boundary, records a baseline, implements test-first, validates, panels
the result on four non-contaminating axes, refutes what the panel raised,
fixes only what survives refutation, re-attacks the fixer's own commits,
and closes with an evidence packet tied to acceptance. Use when: a change
to a registered repository is specified well enough to build, and the
Work should end with reviewed, evidence-backed commits.

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and
`sergeant-rs-workspace/knowledge/evidence/resources/distro-content-series/design-proposal-2026-08-22.md`
for this package's derivation and the owner rulings behind it.
