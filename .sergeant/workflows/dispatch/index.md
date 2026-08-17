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

Provenance for this template's rules (which behavior unit justifies each
rule, and its upstream source) lives in `sergeant-rs-workspace`'s
`knowledge/evidence/provenance/dispatch.md` — this package's `BU-####`
citations and `reference/sergeant-upstream/` paths were stripped from the
shipped template content below; the record of why each rule exists did
not move with them.

Six-stage admitted workflow (N1 reference corpus, `docs/gauntlet/contracts/N1.md`,
candidate **W8** `dispatch`) that, given a project, a brief or tracked
task, and a repository set, produces one durable task with an isolated
work surface, a rendered mission brief, and a running agent per
repository — with every side effect validated and gated before the next
repository's dispatch begins. Use when: Work spans repositories, contains
two or more independent repository-owned tasks, needs an isolated review
worker, or the user asks for workers.

**Corrected 2026-08-16, ICM-R3:** `15-check-admission` holds and releases
the fleet-wide admission lock itself; `80-monitor` delivers escalation
responses via the shipped `sgt respond` command. Neither `drain-fleet`
nor `respond-to-worker` is published in this library — both name open,
unbuilt engine gaps, not live delegation targets.

**No `sgt dispatch` verb (skew-check-2026-08-17 finding 3):** every stage's
prose speaks of "sgt-dispatch" as provenance from the upstream bash tool
this package decomposes — `sgt` has no `dispatch` subcommand. This
workflow is invoked as `sgt run --workflow dispatch`; see `CONTEXT.md`'s
"No `sgt dispatch` verb" section for the full disclaimer.

See `CONTEXT.md` for workflow orientation and `workflow.toml` for the
pinned stage order. Full behavior-unit citations and the N1 adjudication
record (A4, folding twelve extracted stages into six) live in the archived
provenance copy, `docs/gauntlet/promoted-provenance/dispatch.md`, per
`docs/icm/promotion-spec-2026-08-11.md`.
