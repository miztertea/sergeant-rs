---
kind: workflow
name: repo-to-icm
status: published
version: 1
description: >-
  Convert a repository's distributed procedural knowledge — skills, agent
  instructions, scripts, docs, tests — into draft ICM workflow packages
  under the draft namespace, plus an evidence-backed measurement report
  comparing them against a frozen reference. Never publishes workflows,
  never changes the engine.
tags:
  - icm
  - generator
  - measurement
  - decomposition
---

# repo-to-icm

Ten-stage actor-only workflow (`docs/gauntlet/contracts/N2.md`; proposal
§9) that decomposes a target repository's procedural knowledge into the
ICM representation vocabulary (`docs/icm/record-shapes.md` §4) and
materializes the result as **draft** workflow packages — never admitted
procedure. See `CONTEXT.md` for workflow orientation (what each stage
hands the next, the blindness rule for measurement runs), `workflow.toml`
for the pinned stage order, and `_config/` for the two policies every
stage shares: `evidence-policy.md` (citation discipline) and
`icm-ladder.md` (the decomposition ladder, distilled).

Use when: a repository's procedural knowledge needs to be surfaced as
reviewable ICM candidates — either to seed a first admitted decomposition
of a new repository, or to measure this workflow's own recall/precision
against an already-adjudicated reference corpus (as in the N2 measurement
run against `reference/sergeant-upstream`).

Output always lands under `.sergeant/drafts/workflows/<candidate>/` in the
run's own worktree (`docs/icm/convention.md` §2) — promotion to
`.sergeant/workflows/` is a distinct, human-reviewed act this workflow
never performs itself.
