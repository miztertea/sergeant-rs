---
kind: workflow
name: dispatch
status: draft
version: 2
description: >-
  Given a project, a brief or tracked task, and a repository set, produce one durable task with an isolated work surface, a rendered mission brief, and a running agent per repository — with every side effect validated and gated before the next repository's dispatch begins.
tags:
  - dispatch
  - fleet
  - multi-repository
---

# Dispatch

Draft workflow candidate (N1 reference corpus, not admitted procedure —
see `docs/icm/convention.md` §2). Use when: Work spans repositories, contains two or more independent repository-owned tasks, needs an isolated review worker, or the user asks for workers.

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and `provenance.md` for the full behavior-unit citations.
