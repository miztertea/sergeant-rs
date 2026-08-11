# Monitor Fleet
Draft workflow package — candidate **W13** `monitor-fleet` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Observe fleet state without mutating it.

## Trigger

An operator or another workflow (dispatch's `80-monitor`) needs a live view of the fleet.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-observe-and-interpret` | actor-stage (§6.4, judgment — see stage CONTEXT.md; folds `00-snapshot`/`10-evaluate-liveness` as helpers per N1 adjudication A4) | A bounded, constant-size, versioned, strictly read-only snapshot (`busy:true` only with a verified witness, otherwise `busy:null`) plus a per-worker healthy/stalled interpretation; a stalled live worker records a non-terminal diagnostic, never an automatic kill. |

## Adjudication note (A7)

At N1 adjudication A7 (BH-07), this package's two mutating stages —
`20-reconcile-terminal` and `30-background-watch` — moved to
**reconcile-and-cleanup-fleet**, which already owned fleet mutation and
cleanup. `monitor-fleet` keeps only its strictly read-only outcome: a
bounded snapshot and a liveness evaluation, neither of which writes fleet
state. See `reconcile-and-cleanup-fleet/provenance.md` for where the moved
units landed and `reconcile-and-cleanup-fleet/CONTEXT.md` for the
receiving stage.

## Adjudication note (A4)

Applying the generic N1 adjudication A4 sweep (finding N1-BH-02) to the two
stages A7 left behind: both `00-snapshot` and `10-evaluate-liveness` were
extracted as §6.5 "deterministic-machinery candidate" stages, and neither
carried an "Additional note" checkpoint argument surviving §6.3's
reimplementation test — swapping the snapshot's or the liveness
fallback-chain's implementation leaves each checkpoint's outcome unchanged.
Per A4's own default rule this package's two surviving stages were
therefore boilerplate-only machinery with no adjacent judgment-bearing
stage to fold into (unlike every other A4 sweep in this corpus, where at
least one extracted stage was already actor-stage). Judgment call: rather
than leave the package with zero judgment-bearing stages — which A4 does
not license, since a workflow's outcome must ultimately be an actor's
judgment, not raw machinery output — both stages collapse into one new
actor stage, `00-observe-and-interpret`: the snapshot and liveness
computations fold in as ordered helper invocations, and the judgment this
stage performs is interpreting what they jointly mean (a verified busy/idle
answer; a correctly distinguished healthy/stalled worker) and reporting it,
never acting on it. See `provenance.md`'s "Adjudication A4" section and
`00-observe-and-interpret/CONTEXT.md` for the full disposition. Stage count
dropped from 2 to 1; no behavior unit was deleted.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
