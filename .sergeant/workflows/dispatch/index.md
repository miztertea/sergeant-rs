---
kind: workflow
name: dispatch
status: published
version: 2
description: >-
  Given a project, a brief or tracked task, and a repository set, produce one durable task with an isolated work surface, a rendered mission brief, and a running agent per repository — with every side effect validated and gated before the next repository's dispatch begins.
tags:
  - dispatch
  - fleet
  - multi-repository
---

# Dispatch

Six-stage admitted workflow (N1 reference corpus, `docs/gauntlet/contracts/N1.md`,
candidate **W8** `dispatch`) that, given a project, a brief or tracked
task, and a repository set, produces one durable task with an isolated
work surface, a rendered mission brief, and a running agent per
repository — with every side effect validated and gated before the next
repository's dispatch begins. Use when: Work spans repositories, contains
two or more independent repository-owned tasks, needs an isolated review
worker, or the user asks for workers.

`15-check-admission` delegates to **drain-fleet**; `80-monitor` delegates
to **respond-to-worker** (context composition today, not true
nested-workflow invocation — `docs/icm/convention.md` §4). Both targets
are published in this library.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Full behavior-unit citations and the N1 adjudication
record (A4, folding twelve extracted stages into six) live in the archived
provenance copy, `docs/gauntlet/promoted-provenance/dispatch.md`, per
`docs/icm/promotion-spec-2026-08-11.md`.
