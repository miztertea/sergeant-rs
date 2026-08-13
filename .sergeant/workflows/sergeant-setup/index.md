---
kind: workflow
name: sergeant-setup
status: published
version: 3
description: >-
  Capture a complete project definition through interview and track discovered capability gaps as approved work, without ever silently reconfiguring anything the operator did not consent to.
tags:
  - installation
  - setup
  - consent-gated
---

# Sergeant Setup

Two-stage actor-only workflow (N1 candidate **W3**,
`docs/gauntlet/contracts/N1.md`; narrowed from eight stages at the MVP-5 F2
execution-surface re-triage, 2026-08-12 — see `CONTEXT.md`'s "Retired" note
and `docs/icm/re-homing-record-2026-08-12.md`) that captures a complete
project definition through interview and tracks capability gaps discovered
along the way, without ever silently reconfiguring anything the operator
did not consent to. Use when: a new project/repository needs registering
through interview, or a capability gap needs tracking as approved work.
Bootstrap/repair (fresh install, PATH commands, global config, existing-def
repair, task-tracking init, optional capabilities) is `sgt init`/`sgt
doctor`'s job now, not this workflow's.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Originally promoted per
`docs/icm/promotion-spec-2026-08-11.md`; the full behavior-unit citation
trail for all eight original stages lives in the archived copy at
`docs/gauntlet/promoted-provenance/sergeant-setup.md`.
