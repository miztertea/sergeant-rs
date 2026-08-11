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
| `30-force-stop` | actor-stage (§6.4, judgment) | Set drain, await convergence, and worker-side checkpoint run first (folded helpers); force-stop is then refused unless a drain is already active, requires explicit confirmation or dry-run, displays exact identity; undrain runs last (folded helper). |

## Adjudication note (A4)

N1 adjudication A4 (BH-02) applied the generic de-staging sweep: `00-set-drain`, `10-await-convergence`, `20-worker-side-checkpoint`, and `40-undrain` all carried no argument beyond the §6.5 "candidate execute-stage workload" boilerplate, and folded into the package's sole judgment-bearing stage, `30-force-stop`, as ordered helper invocations (set-drain/await-convergence/worker-side-checkpoint before the force-stop judgment; undrain after it). Stage count dropped from 5 to 1; no behavior unit was deleted — see `provenance.md`'s "Adjudication A4" section and `30-force-stop/CONTEXT.md`'s "Helper invocations" section.

## Notes for reviewers

Raises engine-gap **G4** (operator-declared, durable, scope-qualified admission block) — survives, ranked high-evidence/low-cost. See `reference-corpus/synthesis.md` §5.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
