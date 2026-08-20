---
kind: workflow
name: validate-intent
status: published
version: 1
edition: 0.1.1
description: >-
  Review an intent document against AGENTS.md's eight-dimension Captain
  intent discipline, reporting each dimension covered, gapped, or
  not-applicable — never rewriting the intent or inventing content for a
  gap.
tags:
  - intent
  - review
  - captain-tooling
---

# Validate Intent

Reviews an intent document — the Work's own intent text — against the
eight dimensions named in AGENTS.md's `### INTENT — Captain's intent
discipline` — the authoritative list — reporting each dimension
`covered`, `gap` (naming what's missing), or `not-applicable` (with a
reason). Never rewrites the intent and never invents content to fill a
gap — it reports, nothing more.

Filed as issue #201. Optional tooling: a Captain may run this before an
expensive or dangerous dispatch to check an intent's own coverage before
spending a worker on it; it is never a mandatory gate on `sgt run` or on
any workflow's own admission (owner ruling) — nothing in the engine
requires it, and no other doctrine should point at it as though it were
required.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order.
