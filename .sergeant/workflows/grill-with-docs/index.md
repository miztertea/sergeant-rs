---
kind: workflow
name: grill-with-docs
status: published
version: 2
description: >-
  Runs the `grilling` interview while using the domain-modeling discipline to capture ADRs/glossary entries as decisions land.
tags:
  - interview
  - domain-modeling
  - composition
---

# Grill with Docs

Two-stage actor-only workflow (N1 candidate **W29**, `docs/gauntlet/contracts/N1.md`) that runs the `grilling` interview to sharpen a plan or design, then gates on an explicit user confirmation before capturing the decisions landed during the interview as durable ADRs/glossary entries per domain-modeling conventions. Use when: a plan or design needs interview-style stress-testing that should also produce durable domain artifacts.

See `CONTEXT.md` for workflow orientation (including the `## Delegation` to `grilling` at both stages and the reviewer note on why this composition needs no engine change) and `workflow.toml` for the pinned stage order. The full behavior-unit citation trail is archived at `docs/gauntlet/promoted-provenance/grill-with-docs.md`, per the promotion procedure in `docs/icm/promotion-spec-2026-08-11.md`.
