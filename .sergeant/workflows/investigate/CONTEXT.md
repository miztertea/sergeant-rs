# Investigate

Layer 1 orientation only — never delivered as a stage's instructions;
each stage's own `CONTEXT.md` (Layer 2) is the actor's contract
(`.sergeant/common/contexts/icm-policy.md` §1a rule 5).

## Purpose

Answer a bounded question against evidence, with a stated stopping
condition, and leave a durable cited artifact.

## Trigger

A topic needs to be researched, or docs/API/codebase facts need
gathering, and the reading legwork is delegated.

## Stages

| Stage | Rung | Durable outcome |
|---|---|---|
| `00-frame` | actor-stage | The frontier, the bounded question(s), and the stopping condition are written down. |
| `10-fan-out-evidence` | actor-stage | N isolated evidence seats, spawned in one message, each with its own bounded sub-question and cap; each returns cited evidence. |
| `20-synthesize` | actor-stage | One structured document: numbered questions, each answer citing live-verified evidence, ending in a summary of recommendations — and an explicit coverage statement naming any seat that did not report. |
| `25-challenge` | actor-stage | A refuter pass over the conclusions, defaulting to refuted: a conclusion stands only if the challenge could not overturn it. |
| `30-record` | actor-stage | The durable repo artifact exists at a named path. |
| `40-close` | actor-stage | Answer, confidence, contradictory evidence found, remaining unknowns, and recommended next intents. |

## Relationships to other workflows

This workflow recommends, and never dispatches, whatever next intent its
close packet names — most often `implement-change` or `author-document`,
when the answer implies further work. There is no child-workflow dispatch
and no worker-side submission (`.sergeant/common/contexts/icm-policy.md` §7.5).

## Authority envelope

This workflow receives an already-admitted Work whose intent names a
question to investigate.

### May decide
- How many evidence seats to spawn and how to bound each sub-question
  (`10`), within what the framed question at `00` requires.
- How to synthesize seat reports into one structured document (`20`).
- Where the durable artifact is placed, per the repository's own
  note-keeping convention or an explicitly stated choice (`30`).

### May not decide
- Interview the user to establish the destination/question itself — that
  is Captain's live-dialogue act (R-NS-6), not this workflow's. This
  workflow frames what the intent already names; it never interviews.
- Merge or re-rank the evidence seats' reports against each other in a way
  that hides which seat said what (`20`).
- Treat a challenged conclusion as standing merely because it was not
  explicitly attacked (`25`) — silence never confirms.

### Human or Captain gates
- The intent does not itself name a bounded question or a stopping
  condition (`00`).
- A conclusion's challenge turns on a decision only a human can make
  (`25`).
- No convention exists for where to place the durable artifact and none
  is stated in the intent (`30`).

### Decision record
Material decisions cite J-rungs inline in each stage's own output
artifact per `.sergeant/common/contexts/bounded-judgment.md` §Decision
evidence; the recorded artifact from `30-record` and the close packet
from `40-close` are this workflow's central decision records.

## Robustness

**(a)** Six checkpoints; the fan-out's cost is banked at `20-synthesize` —
a stall after synthesis does not require re-running every evidence seat.

**(b)** `25-challenge` attacks the synthesis's conclusions.

**(c)** A seat that dies does not kill the run — `20-synthesize` reports
which seats completed and states its coverage honestly (the codex
sprint's 2-of-7 seat deaths are the measured precedent this design record
cites).

## Notes for reviewers

`wayfinder/00-name-destination` is deleted, not carried forward: naming a
destination is a live-interview act R-NS-6 places in Captain, before any
dispatch — a dispatched Work cannot receive it. `00-frame` absorbs only
the frontier-mapping *concept* (what's known, what's fog, what would stop
the search) as the framing this stage does against an intent that already
names its question; it does not recreate `wayfinder`'s issue-tracker
ticket graph, HITL/AFK ticket types, or map-and-fog loop-back mechanics —
those retire with the package, their behavior not represented as workflow
stages here.
