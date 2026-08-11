---
kind: workflow
name: wiki-digest
status: published
version: 2
description: >-
  Generate and publish a schema-driven wiki digest from configured sources, previewed before publication and never regressing an existing page.
tags:
  - wiki
  - digest
  - publishing
---

# Wiki Digest

Generates and publishes a schema-driven wiki digest from configured
sources, previewed before publication and never regressing an existing
page (N1 reference corpus, candidate **W35**,
`docs/gauntlet/contracts/N1.md`). Use when: a digest is due (scheduled) or
explicitly requested; or the schema/logic changed and needs a dry run
first.

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and `docs/icm/promotion-spec-2026-08-11.md` plus the archived
citation trail at `docs/gauntlet/promoted-provenance/wiki-digest.md` for
the full behavior-unit citations.
