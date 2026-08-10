# Drain Fleet
Draft workflow package — candidate **W12** `drain-fleet` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

A cooperative, bounded, non-destructive admission block: refuse new work without terminating anything already running.

## Trigger

An operator needs to freeze new stage/turn admission — globally or for one project — before a disruptive operation.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-set-drain` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Admission is refused the instant the drain is set, scope global or per-project, race closed by an explicit lock. |
| `10-await-convergence` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | A bounded wait; a worker counts as drained only when its exit is provable; timeout leaves the drain active, exits non-zero, and names the unresolved. |
| `20-worker-side-checkpoint` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Idempotent drain detection; publish handoff and settle the lease before terminating anything. |
| `30-force-stop` | actor-stage (§6.4, judgment) | Force-stop is refused unless a drain is already active; requires explicit confirmation or dry-run; displays exact identity. |
| `40-undrain` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Undrain is idempotent, with mutually exclusive scopes. |

## Notes for reviewers

Raises engine-gap **G4** (operator-declared, durable, scope-qualified admission block) — survives, ranked high-evidence/low-cost. See `reference-corpus/synthesis.md` §5.

**Reading `pane`/`tmux` in cited statements.** The following citations in this package's behavior contracts describe identity, liveness, or ownership checks in terms of old Sergeant's tmux pane: `BU-P6-058`, `BU-P7-084`. Per obsolete-mechanism clusters M1-M4 (`reference-corpus/synthesis.md` §4) and deviation register D2, this project structurally replaced the pane with headless per-turn processes owned by the daemon and a durable session/execution identity in the journal — there is no tmux pane in this architecture. Read every 'pane identity' / 'pane liveness' / 'pane recycling' phrase in those citations as **the durable execution or session identity this project already journals**, not as an instruction to introduce tmux. The policy (verify identity before acting, never infer liveness from a UI artifact, settle a lease before terminating) is durable; the pane is not.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
