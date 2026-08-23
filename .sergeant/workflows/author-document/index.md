---
kind: workflow
name: author-document
status: published
version: 1
edition: 0.2.1
description: >-
  Produce a document as the deliverable, with fidelity to the brief as
  the top-weighted review axis.
tags:
  - documentation
  - drafting
  - fidelity
---

# Author Document

Six-stage actor-only workflow that maps authoritative sources, establishes
an outline against a named audience and purpose, drafts from mapped
sources only, verifies fidelity-to-brief and facts, challenges the draft
adversarially, and finalizes with evidence of which sources and which
revision of each were used. Use when: a document is the deliverable
itself — including transcribing decisions an in-session grilling already
made, via this package's `record-decisions` profile section (see
`CONTEXT.md`'s Notes for reviewers).

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and
`sergeant-rs-workspace/knowledge/evidence/resources/distro-content-series/design-proposal-2026-08-22.md`
for this package's derivation and the owner rulings behind it.
