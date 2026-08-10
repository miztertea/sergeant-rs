# Vet External Skill
Draft workflow package — candidate **W34** `vet-external-skill` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Vet an external skill through a fixed sequence before adopting it, and keep already-adopted skills updated through the same discipline.

## Trigger

Before adopting an external skill, or when an adopted skill needs updating.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-read-source` | actor-stage (§6.4, judgment) | The external skill's complete SKILL.md and referenced scripts are read before adopting it. |
| `10-confirm-provenance` | actor-stage (§6.4, judgment) | The external skill's source and update mechanism are confirmed. |
| `20-check-actions` | actor-stage (§6.4, judgment) | The external skill's filesystem, shell, network, Git, and credential actions are checked. |
| `30-verify-no-conflict` | actor-stage (§6.4, judgment) | The external skill does not conflict with repository AGENTS.md or safety policy. |
| `40-pin-source` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | The external skill's source is pinned or locked where the installer supports it. |
| `50-test-in-disposable-copy` | actor-stage (§6.4, judgment) | The external skill is tested in a disposable repository or worktree before broad installation. |
| `60-update-managed` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | For skills.sh-managed skills: rerun the official installer and inspect the diff and updated lock file before accepting changes. |
| `60-update-owned` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | For Sergeant-owned skills: update this repository through a reviewed PR and run the instruction-policy test plus the full test suite. |

## Notes for reviewers

Six ordered checkpoints (`00`-`50`) plus two mutually exclusive update variants (`60-update-managed`/`60-update-owned`) reached only when refreshing an already-adopted skill. Each step's outcome ("the source was read", "the actions were checked", "it was tested in a disposable copy") survives any reimplementation of *how* the checking is done — a strong candidate for the smallest complete reference workflow in the corpus.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
