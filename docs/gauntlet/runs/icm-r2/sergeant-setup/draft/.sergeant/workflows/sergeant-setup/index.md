---
kind: workflow
name: sergeant-setup
status: retired
version: 4
description: >-
  Retired at ICM-R2. Behavior absorbed into AGENTS.md guardrails and
  skills/estate-navigation/SKILL.md; two fragments parked pending a
  sergeant.toml schema decision. See the adjudication record.
tags:
  - installation
  - setup
  - retired
---

# Sergeant Setup (retired)

Retired by the ICM-R2 pilot reconciliation
(`docs/gauntlet/runs/icm-r2/sergeant-setup/adjudication-draft.md`, SPLIT
disposition). This package's two surviving stages (`05-file-capability-gaps`,
`30-project-interview`) — already narrowed from eight at the prior MVP-5 F2
execution-surface re-triage (`docs/icm/re-homing-record-2026-08-12.md`) — do
not survive as a workflow:

- The capability-gap-to-tracked-work judgment (`05-file-capability-gaps`,
  `BU-P5-012`) moves to `skills/estate-navigation/SKILL.md` as a live,
  Captain-session procedure rather than an admitted background Work.
- The project-registration interview (`30-project-interview`) targeted an
  upstream `~/.config/sergeant/<project>.yaml` multi-project registry that
  `skills/estate-navigation/SKILL.md` already documents as not existing in
  sergeant-rs. The transplantable fragment (ask for repo name, clone origin,
  group membership) moves to `skills/estate-navigation/SKILL.md` alongside
  the item above; the rest (per-repo role, free-text `agent_instructions`,
  project-level GitHub identity, Graphify output path) has no current
  `sergeant.toml` field to attach to and is parked, not translated.
- The write-scope guardrails (`_config/standing-constraints.md`) were
  already present near-verbatim in `AGENTS.md`'s Guardrails section
  (`BU-1263`/`BU-1264`/`BU-1295`); one orphaned bullet
  (`BU-P5-006`, consent-gating optional-tool auto-init) is added there.

`AGENTS.md`'s own trigger routing table already sent both of this package's
triggers elsewhere before this retirement — see the adjudication record's
"Current trigger and outcome" section.

This is draft content for independent review (ADR 0013 decision 6); it does
not replace the live, published `.sergeant/workflows/sergeant-setup/` tree
until reconciled and promoted.

See `docs/gauntlet/runs/icm-r2/sergeant-setup/adjudication-draft.md` for the
full behavior-unit disposition table and rationale.
