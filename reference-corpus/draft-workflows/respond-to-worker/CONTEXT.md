# Respond to Worker
Draft workflow package — candidate **W10** `respond-to-worker` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

A blocked/needs-input/waiting/orphaned worker is durably given exactly one decision, applies it exactly once, and returns to forward progress.

## Trigger

A worker has published an escalation and a human decision exists.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-precondition-check` | actor-stage (§6.4, judgment) | Exact question read, only genuinely missing decisions asked, decision recorded in tracked work, no unconsumed generation already pending. |
| `40-apply-and-acknowledge` | actor-stage (§6.4, judgment) | Validate target, publish response, and deliver-and-accept run first (folded helpers); decision applied once, truthful status restored, applied id/generation/status recorded, acknowledged; archive evidence, notify coordinator, and relaunch-if-needed run after (further folded helpers). |

## Adjudication note (A4)

N1 adjudication A4 (BH-02) applied the generic de-staging sweep: the six
extracted stages between and around this package's two judgment-bearing
stages — `10-validate-target`, `20-publish-response`,
`30-deliver-and-accept`, `50-archive-evidence`, `60-notify-coordinator`,
`70-relaunch-if-needed` — carried no argument beyond the §6.5 "candidate
execute-stage workload" boilerplate. `10`/`20`/`30` folded forward into
`40-apply-and-acknowledge` as preceding helper invocations; `50`/`60`/`70`
folded backward into it as following helper invocations (there is no
judgment-bearing stage after `70` to fold forward into). Stage count
dropped from 8 to 2; no behavior unit was deleted — see `provenance.md`'s
"Adjudication A4" section and `40-apply-and-acknowledge/CONTEXT.md`'s
"Helper invocations" section.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
