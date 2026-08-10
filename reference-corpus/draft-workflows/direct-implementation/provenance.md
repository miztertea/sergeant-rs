# Provenance — Direct Implementation

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W6** `direct-implementation`.

## Stages

### `01-load-task-context`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-008` | In direct mode, run sgt-context for the project and td context for the owning task before making any edit. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L24-25, direct-mode step 1) |

### `03-claim-and-implement`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-010` | In direct mode, claim or create the owning td task, then implement test-driven-first in the requested checkout or an isolated worktree. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L28-29, direct-mode step 3) |
| `BU-P1-011` | In direct mode, never edit a default branch; create or reuse the owning feature branch before the first implementation change. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L30-31, direct-mode branch rule) |

Folds the demoted `02-reconcile-existing-state` checkpoint as a helper (see Adjudication A4 below):

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-009` | In direct mode, reconcile existing workers and preserved worktrees before editing; never duplicate or race work already in progress. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L26-27, direct-mode step 2) |
| `BU-P8-056` | Before editing anything in direct mode, existing worktrees and workers for the same owning repository/task must be reconciled. | `reference/sergeant-upstream/docs/using-sergeant.md` (L23 (Direct mode, step 2)) |

### `04-validate`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-012` | In direct mode, run repository-native validation, independent reviews, and the final shipping gate exactly as a dispatched worker would. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L32-33, direct-mode validation step) |

### `05-shipping-gate`

| Unit | Statement | Source |
|---|---|---|
| `BU-P8-058` | The final shipping gate in direct mode is run only at the approved shipping boundary, not automatically at the end of implementation. | `reference/sergeant-upstream/docs/using-sergeant.md` (L26 (Direct mode, step 6)) |

### `06-pr-and-merge`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-013` | In direct mode, open a PR for every implementation and satisfy required CI, review threads, and merge authorization before calling delivery complete. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L34-35, direct-mode PR step) |

Folds the demoted `07-record-outcomes` checkpoint as a helper (see Adjudication A4 below):

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-014` | In direct mode, record handoff, PR, merge, deployment, and cleanup outcomes. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L36, direct-mode handoff step) |

## Adjudication A4

N1 adjudication A4 (finding N1-BH-02, `reference-corpus/adjudication-round1.md`): every stage whose `CONTEXT.md` justification was only the §6.5 deterministic-machinery boilerplate is demoted by default, folded into the adjacent judgment-bearing stage as a helper invocation.

- **`02-reconcile-existing-state` — DEMOTED.** Classified at extraction as deterministic machinery (ladder §6.5) with no "Additional note" checkpoint argument; fails the reimplementation test as an independent checkpoint (reconciling existing worktrees/workers is a repeatable precondition check subordinate to the claim-and-implement checkpoint, not itself a durable state anyone inspects). Folded into `03-claim-and-implement` (its only neighbor, and the stage whose claim it protects) as a helper. `BU-P1-009`, `BU-P8-056` survive, re-homed.
- **`07-record-outcomes` — DEMOTED.** Same boilerplate-only classification, no surviving checkpoint argument. Folded into `06-pr-and-merge` (its only neighbor; recording outcomes is the mechanical conclusion of a merged PR, not an independently observable checkpoint). `BU-P1-014` survives, re-homed. `06-pr-and-merge`'s output now carries the `promote` disposition `07-record-outcomes`'s output previously carried, since `06-pr-and-merge` is now the workflow's last stage.

## Notes

**Demoted/merged candidates:** Conflict X16 (synthesis.md §6): `AGENTS.md`'s six-stage `direct-mode` and `docs/using-sergeant.md`'s eight-step `direct-implementation` describe the same procedure with different stage boundaries (the docs variant splits reconciliation and the shipping gate into their own steps). This workflow follows the docs' finer boundary.

**Standing constraints** (`BU-P1-016`, `BU-P1-007`, `BU-P1-107`, `BU-P8-055`): Direct-mode work always uses a feature branch and always opens a PR — never a direct push to the default branch. Direct mode requires an explicit user request and a single owning repository; it is not a way to avoid dispatch when the outcome genuinely spans repositories.

