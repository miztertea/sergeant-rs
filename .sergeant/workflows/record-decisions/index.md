---
kind: workflow
name: record-decisions
status: published
version: 1
edition: 0.1.1
description: >-
  Transcribe decisions already made in an in-session grilling into
  ADR/glossary material, with a missing rationale logged rather than
  invented, and fidelity to the brief as the review's top-weighted axis.
tags:
  - decisions
  - adr
  - documentation
---

# Record Decisions

Takes a brief carrying decisions already made — the record of an
in-session grilling (`skills/grilling`), not a request to make new
decisions — and turns it into ADR/glossary material: each decision
recorded with its alternatives and the reasons they were rejected. Filed
as issue #88 (candidate name `to-adr`).

Codifies two safeguards the issue proved by hand: a missing rationale is
logged as missing, never invented (`10-transcribe-decisions`) — inventing
one launders a guess into the permanent record; and fidelity to the brief
is the review's top-weighted axis (`20-fidelity-review`), reusing
`worker-mission/30-independent-review`'s brief-authoritative axis
mechanism — the brief's own axis list governs, never a generic review
skill's fewer axes.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order.
