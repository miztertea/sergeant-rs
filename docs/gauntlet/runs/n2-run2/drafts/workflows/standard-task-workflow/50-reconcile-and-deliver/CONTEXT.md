# 50-reconcile-and-deliver: close out only after verified terminal state

## Inputs

| File | Layer | Why |
|---|---|---|
| ../40-validate/output/ | L4 | upstream artifact produced by this candidate's own `40-validate` — this stage's own precondition ("verified terminal state") is that stage's outcome |

## Purpose

Run cleanup only after terminal state and preserved evidence are
verified, as the final step of `standard-task-workflow`. Cleanup running
early is exactly the kind of failure worth tracking independently.

## What must become true here (durable outcome)

Terminal state and preserved evidence are verified before any cleanup
runs; cleanup never runs ahead of that verification.

## How to do it

*(Draft package — detailed method content is a promotion-time task for a
human reviewer, drawing on the source material `../provenance.md` cites;
this `CONTEXT.md` is orientation-level per the draft package template, not
a finished, ready-to-run stage contract.)*

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition — describes shape for this candidate's own
future runs, not populated at draft time.
