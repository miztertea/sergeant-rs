---
kind: workflow
name: repo-to-icm
status: published
version: 3
edition: 0.1.0
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

Eleven-stage workflow (`docs/gauntlet/contracts/N2.md`; proposal §9) —
ten actor stages plus one `kind = "execute"` stage, `65-self-check`
(N4, `docs/gauntlet/contracts/N4.md`) — that decomposes a target
repository's procedural knowledge into the ICM representation vocabulary
(`docs/icm/record-shapes.md` §4) and
materializes the result as **draft** workflow packages — never admitted
procedure. See `CONTEXT.md` for workflow orientation (what each stage
hands the next, the blindness rule for measurement runs, and v2's harvest
volume-handling decision), `workflow.toml` for the pinned stage order, and
`_config/` for the two policies every stage shares: `evidence-policy.md`
(citation discipline) and `icm-ladder.md` (the decomposition ladder,
distilled — v2 adds the required-before-`helper` §6.3 answer and the
over-promotion tell).

**v2** (`docs/gauntlet/runs/n2-run2/comparison-scorecard.md`,
`grammar-pressure-report.md`): `20-harvest` gained a per-partition
checkpoint/retry protocol and a mandatory five-class (safety, identity,
recovery, delivery, human-decision) consequence sweep with its own
artifact; `40-classify` now requires the §6.3 reimplementation-test answer
to be recorded before any `helper`/`shared-helper` classification, plus a
self-check for helper clusters that merely mirror source-file boundaries;
`80-adversarial-review` gained a fourth axis (structural self-consistency —
count cross-checks, hash-vs-stored-quote verification, representation-
distribution sanity); `scripts/finalize.py` now refuses to remove a file
that is not yet reachable in any committed tree (GP-5b / issue #29), proven
by `scripts/test-finalize-evidence-guard.py` and wired into
`scripts/validate-structure.py` as `[S15]`.

**v3** (`docs/gauntlet/contracts/N4.md`, MVP-2 lane D3): adds
`65-self-check`, a `kind = "execute"` stage between `60-draft` and
`70-lint` that runs `scripts/validate-structure.py` against this
workflow's own tree in a pinned, offline container — the mechanical
self-check `70-lint`'s own instructions used to ask an actor to run by
hand. `70-lint` now reads that stage's output artifact instead of
re-running the validator itself for that one check; every other stage's
contract is unchanged.

Use when: a repository's procedural knowledge needs to be surfaced as
reviewable ICM candidates — either to seed a first admitted decomposition
of a new repository, or to measure this workflow's own recall/precision
against an already-adjudicated reference corpus (as in the N2 measurement
run against `reference/sergeant-upstream`).

Output always lands under `.sergeant/drafts/workflows/<candidate>/` in the
run's own worktree (`docs/icm/convention.md` §2) — promotion to
`.sergeant/workflows/` is a distinct, human-reviewed act this workflow
never performs itself.
