---
kind: workflow
name: fix-defect
status: published
version: 1
edition: 0.2.0
description: >-
  Reproduce a defect before touching it, prove the cause, fix it with a
  regression test, and put the fix through the same review chain a
  feature change gets.
tags:
  - debugging
  - defect
  - review
---

# Fix Defect

Eight-stage actor-only workflow that builds a red-capable feedback loop,
reproduces and minimizes the defect as a hard gate before any edit, forms
and instruments falsifiable hypotheses, fixes with a regression test,
panels the fix on four axes, refutes what the panel raised, and re-verifies
the fix commits with a root-cause postmortem. Use when: a defect needs
diagnosis and a fix, and the fix should get the same review chain a
feature change gets (a fix is code).

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and
`sergeant-rs-workspace/knowledge/evidence/resources/distro-content-series/design-proposal-2026-08-22.md`
for this package's derivation and the owner rulings behind it.
