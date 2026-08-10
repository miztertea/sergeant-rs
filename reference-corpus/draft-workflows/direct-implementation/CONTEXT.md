# Direct Implementation
Draft workflow package — candidate **W6** `direct-implementation` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Implement in the current session, against one owning repository, under the same delivery contract as a dispatched worker.

## Trigger

The user explicitly asks to work in this session, and one repository owns the complete outcome.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `01-load-task-context` | actor-stage (§6.4, judgment) | The task's originating context is loaded and understood. |
| `02-reconcile-existing-state` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Existing branch/worktree/handoff state is inspected and resumed rather than duplicated. |
| `03-claim-and-implement` | actor-stage (§6.4, judgment) | The task is claimed and the change is implemented. |
| `04-validate` | actor-stage (§6.4, judgment) | The change is validated against native project checks. |
| `05-shipping-gate` | actor-stage (§6.4, judgment) | The shipping gate runs at the approved boundary only. |
| `06-pr-and-merge` | actor-stage (§6.4, judgment) | A PR is opened and merged per repository convention. |
| `07-record-outcomes` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Outcomes are recorded against the owning tracked task. |

## Relationships to other workflows

- `05-shipping-gate` delegates to **validate-and-ship**.

## Standing constraints (Layer 3, `_config/standing-constraints.md`)

Direct-mode work always uses a feature branch and always opens a PR — never a direct push to the default branch. Direct mode requires an explicit user request and a single owning repository; it is not a way to avoid dispatch when the outcome genuinely spans repositories.

## Notes for reviewers

Conflict X16 (synthesis.md §6): `AGENTS.md`'s six-stage `direct-mode` and `docs/using-sergeant.md`'s eight-step `direct-implementation` describe the same procedure with different stage boundaries (the docs variant splits reconciliation and the shipping gate into their own steps). This workflow follows the docs' finer boundary.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
