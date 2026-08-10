---
kind: workflow
name: repo-release-verification
status: draft
version: 1
description: >-
  The source repository's own pre-push gate: the drain suite must pass before every push.
tags:
  - self-hosting
  - pre-push
  - verification
---

# Repo Release Verification

Draft workflow candidate (N1 reference corpus, not admitted procedure —
see `docs/icm/convention.md` §2). Use when: A push to the source repository is about to happen.

See `CONTEXT.md` for workflow orientation, `workflow.toml` for the pinned
stage order, and `provenance.md` for the full behavior-unit citations.
