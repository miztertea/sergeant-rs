---
kind: workflow
name: sergeant-setup
status: published
version: 2
description: >-
  Bring an installation from any partial state to a verified-complete state without ever silently reconfiguring anything the operator did not consent to.
tags:
  - installation
  - setup
  - consent-gated
---

# Sergeant Setup

Eight-stage actor-only workflow (N1 candidate **W3**,
`docs/gauntlet/contracts/N1.md`) that brings a Sergeant installation from
any partial state to a verified-complete state without ever silently
reconfiguring anything the operator did not consent to. Use when: first
install, a new project/repository to register, a broken or incomplete
installation, or a verification request.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. This package was promoted per
`docs/icm/promotion-spec-2026-08-11.md`; the full behavior-unit citation
trail lives in the archived copy at
`docs/gauntlet/promoted-provenance/sergeant-setup.md`.
