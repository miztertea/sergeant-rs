# Repo Release Verification
Draft workflow package — candidate **W19** `repo-release-verification` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

The source repository's own pre-push gate: the drain suite must pass before every push.

## Trigger

A push to the source repository is about to happen.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-release-verification` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | The drain suite must pass before every push; missing tooling fails closed rather than silently skipping. |

## Notes for reviewers

Survives §6.3 by name — it is the proposal's own worked example. Scoped as self-hosting behavior of the *source repository*, not a Sergeant-offered procedure other repositories would install.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
