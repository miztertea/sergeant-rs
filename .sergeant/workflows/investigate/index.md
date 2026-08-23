---
kind: workflow
name: investigate
status: published
version: 1
edition: 0.2.0
description: >-
  Answer a bounded question against evidence, with a stated stopping
  condition, and leave a durable cited artifact.
tags:
  - research
  - evidence
  - fan-out
---

# Investigate

Six-stage actor-only workflow that frames a bounded question and its
stopping condition, fans out isolated evidence seats against it,
synthesizes their findings into one cited document, challenges the
conclusions, records the durable artifact, and closes with an honest
answer — or an honest "not yet known." Use when: a topic needs research,
or docs/API/codebase facts need gathering, and the reading legwork is
delegated rather than done in the current conversation.

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and
`sergeant-rs-workspace/knowledge/evidence/resources/distro-content-series/design-proposal-2026-08-22.md`
for this package's derivation and the owner rulings behind it.
