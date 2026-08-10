# Reconcile and Cleanup Fleet
Draft workflow package — candidate **W15** `reconcile-and-cleanup-fleet` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Retire a completed task's surfaces and state.

## Trigger

A task's repos are believed terminal and the operator (or an automated sweep) requests cleanup.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-require-terminal` | actor-stage (§6.4, judgment) | Every targeted repo is safely terminal and the owning task is verifiably closed; "not closed" is distinguished from "could not be looked up". |
| `10-verify-ownership` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Repo identity, not path, is verified; retry-owner spoofing vectors are rejected. |
| `20-verify-handshakes` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Acknowledgement is verified, re-verified under lock immediately before deletion, and a terminal seal is written. |
| `30-remove-surface` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | A resumable cleanup-phase record is published before and after; no process runs with its cwd inside the surface being removed. |
| `40-retire-state` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Whole-task state is retired only when every repo is cleaned together. |

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
