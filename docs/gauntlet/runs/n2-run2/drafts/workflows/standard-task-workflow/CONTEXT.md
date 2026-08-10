# standard-task-workflow — workflow orientation

Layer 1. This file orients an actor entering this **draft** workflow for
the first time; it is not stage instruction. Only the first stage lists
this file in its Inputs table (`docs/icm/convention.md` §1a rule 5).

## What this workflow is for

The user brings a task. It advances through a fixed sequence of durable
checkpoints — load context, check the queue, reconcile in-flight state,
validate, reconcile and deliver — before being treated as delivered.
Completion condition: cleanup never runs ahead of verified terminal state
and preserved evidence.

## How the stages hand off

```text
10-load-context               → execution-mode selection happens only
                                 after context is loaded
20-check-queue                → an existing canonical task is reused
                                 rather than a duplicate created
30-reconcile-existing-state    → preserved in-flight work is resumed or
                                 taken over instead of duplicated
40-validate                    → validation runs exactly once as a
                                 dedicated boundary; post-readiness HEAD
                                 changes get rereview without
                                 retriggering the full cycle
50-reconcile-and-deliver       → cleanup never runs ahead of verified
                                 terminal state and preserved evidence
```

**Ordering note carried from synthesis.** Four of these five stages have
an explicit step number in their source evidence (1, 2, 4, 9); `40-validate`
does not. It is placed here between `30-reconcile-existing-state` (step 4)
and `50-reconcile-and-deliver` (step 9) on trigger/outcome-chaining grounds
alone — see `provenance.md` for the full reasoning `50-synthesize` recorded
and the exact step-number gap (5, 6, 7, or 8) it left open. A human
reviewer promoting this candidate should treat this ordering as a judgment
call open to challenge, not a fixed fact.

**Two steps of the source procedure are visible only as unattached
judgment content, not as stages of their own** — a step 5
("confirm-decisions") and a step 7 ("monitor-progress"). Neither earned a
`stage` classification in this corpus, so neither is a stage here; see
`provenance.md`'s "Unattached ground" section. A human reviewer should
decide whether either belongs inside an existing stage's own `CONTEXT.md`
as stage-context, or as a stage of its own, once the source material is
available for closer reading.

## Status

`status: draft`. This package lives under `.sergeant/drafts/workflows/`
and is not runnable procedure until a human promotes it into
`.sergeant/workflows/` (`docs/icm/convention.md` §2).
