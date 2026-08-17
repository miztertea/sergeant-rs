# Recover Stalled Worker
Draft workflow package — candidate **W11** `recover-stalled-worker` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`).
This is Layer 1 orientation only — it is never delivered as a stage's
instructions; each stage's own `CONTEXT.md` (Layer 2) is the actor's
contract (`docs/icm/convention.md` §1a rule 5).

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
deleted — see `docs/gauntlet/promoted-provenance/recover-stalled-worker.md`'s
"Adjudication A4" section and
`40-escalate-on-second-attempt/CONTEXT.md`'s "Helper invocations" section.

## Authority envelope

This workflow receives an already-admitted Work intent that names the
specific stalled worker (task-id and repo) at admission. This mirrors the
upstream `sgt-recover <task-id> <repo>` CLI's own required positional
arguments: the destructive action this workflow performs (kill the
stalled process, launch a replacement) is never an automatic reaction to
the watcher's own stall diagnostic — `sgt-watch` never invokes
`sgt-recover` itself — it is always a separately-issued, explicitly-
targeted human or Captain-delegated action.

### Workflow may decide
- How to reconcile a nonterminal stall diagnostic through the documented
  progress rules, and how to investigate a repeated notification's
  specific cause (`00-collect-signals`).
- Whether a pre-flight validation, a lease owner's liveness, and a
  replacement's viability are established before committing to the one
  bounded recovery attempt (`40-escalate-on-second-attempt`).
- How to search existing docs and compose a `td` task for an
  undocumented stall class (`50-escalate-undocumented`).

### Workflow may not decide
- Kill or relaunch a worker on partial evidence (fewer than all four
  collected signals).
- Make more than one bounded recovery attempt per worker — a second
  stamped attempt always escalates rather than retries.
- Terminate the original worker before the replacement is launched and
  proven live.
- Guess at an undocumented or unrecognized stall classification.

### Human or Captain gates
- A second recovery attempt on an already-stamped worker.
- An undocumented/unrecognized stall class.
- A first, correctly-blocked preflight failure (active drain, unprovable
  lease owner) — see `40-escalate-on-second-attempt`'s own `## Bounded
  judgment` section for how this is surfaced.

### Decision record
Material decisions (collected signals, preflight/stamp state, replacement
viability, any `J0` stop) are recorded in each stage's own turn and
surfaced through `needs_input` where applicable; this workflow declares no
separate decision-log file.

## Provenance

See `docs/gauntlet/promoted-provenance/recover-stalled-worker.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
