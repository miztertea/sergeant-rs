# 10-dispatch-worker: spawn an isolated worker for a targeted repository

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

For each repository targeted by a dispatch invocation, bring a worker for
that repository from "not started" to "running" — an isolated checkout, a
written brief, and a spawned interactive agent session all in place — the
checkpoint `dispatch-mode`'s later steps (monitor, reconcile) depend on.

## What must become true here (durable outcome)

For every targeted repository: an isolated checkout exists, a brief has
been written for the worker, and an interactive agent session has been
spawned against that checkout. This is a per-repository checkpoint —
`dispatch-mode` fans out one of these per owning repository, not one for
the dispatch invocation as a whole.

## How to do it

*(Draft package — this candidate's own stage sequence has exactly one
member. Detailed method content is a promotion-time task for a human
reviewer, drawing on the source material `provenance.md` cites; this
`CONTEXT.md` is orientation-level per the draft package template, not a
finished, ready-to-run stage contract.)*

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition — describes shape for this candidate's own
future runs, not populated at draft time.
