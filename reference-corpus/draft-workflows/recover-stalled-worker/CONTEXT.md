# Recover Stalled Worker
Draft workflow package — candidate **W11** `recover-stalled-worker` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

One bounded recovery attempt for a stalled worker: converge on a replacement or escalate — never guess.

## Trigger

A worker is `in_progress` with a stall classification recorded by the watcher.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-collect-signals` | actor-stage (§6.4, judgment) | Four signals are collected together before any kill/relaunch decision. |
| `40-escalate-on-second-attempt` | actor-stage (§6.4, judgment) | Preflight, launch-replacement, and retire-original run first (folded helpers); exactly one bounded recovery attempt is made; a second stall escalates to needs-input. |
| `50-escalate-undocumented` | actor-stage (§6.4, judgment) | An undocumented/unrecognized stall class escalates rather than being guessed at. |

## Adjudication note (A4)

N1 adjudication A4 (BH-02) applied the generic de-staging sweep:
`10-preflight`, `20-launch-replacement`, and `30-retire-original` carried
no argument beyond the §6.5 "candidate execute-stage workload"
boilerplate and folded into `40-escalate-on-second-attempt` as ordered
helper invocations. Stage count dropped from 6 to 3; no behavior unit was
deleted — see `provenance.md`'s "Adjudication A4" section and
`40-escalate-on-second-attempt/CONTEXT.md`'s "Helper invocations" section.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
