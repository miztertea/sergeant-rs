---
kind: workflow
name: validate-and-ship
status: published
version: 3
edition: 0.2.1
description: >-
  The single final shipping boundary: validate a committed change through
the pipeline to a terminal outcome, routing every finding, without the
validating actor ever editing the code.
tags:
  - shipping-gate
  - validation
  - no-mistakes
---

# Validate and Ship (no-mistakes)

Provenance for this template's rules (which behavior unit justifies each
rule, and its upstream source) is kept in this project's private
development record — this package's provenance markers were stripped
from the shipped template content below; the record of why each rule
exists did not move with them.

Seven-stage actor-only workflow (N1 reference corpus, candidate **W18**
`validate-and-ship`) that is the single final shipping boundary: validate
a committed change through the pipeline to a terminal outcome, routing
every finding, without the validating actor ever editing the code. Use
when: Implementation, native tests, lint and independent review are
complete and the coordinator has reached the approved shipping boundary.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Full behavior-unit citations and the promotion record
are dev-corpus provenance, kept in this project's private development
record, not shipped here.
