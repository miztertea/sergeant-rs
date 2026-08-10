# 10-load-context: load context and select an execution mode

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Load the task's owning repository or repositories, inherited instructions,
paths, and cross-repo dependencies, and only then select an execution
mode — the boundary every task must cross before mode selection happens.

## What must become true here (durable outcome)

Context is loaded (owning repository/repositories, inherited instructions,
configured paths, cross-repo dependencies) and an execution mode has been
selected on the basis of that loaded context, not before it.

## How to do it

*(Draft package — detailed method content is a promotion-time task for a
human reviewer, drawing on the source material `../provenance.md` cites;
this `CONTEXT.md` is orientation-level per the draft package template, not
a finished, ready-to-run stage contract.)*

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition — describes shape for this candidate's own
future runs, not populated at draft time.
