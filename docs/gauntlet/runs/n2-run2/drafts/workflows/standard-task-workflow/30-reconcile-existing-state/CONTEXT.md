# 30-reconcile-existing-state: reconcile in-flight state before new work

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-check-queue/output/ | L4 | upstream artifact produced by this candidate's own `20-check-queue` — the settled canonical task identity this stage reconciles state for |

## Purpose

Reconcile active workers, branches, worktrees, retained gates, and
handoffs before starting new work — a boundary operators would want
measured: did reconciliation find something to resume, or not.

## What must become true here (durable outcome)

Preserved in-flight work is either resumed/taken over, or its absence is
confirmed — never silently duplicated by starting new work over
unreconciled state.

## How to do it

*(Draft package — detailed method content is a promotion-time task for a
human reviewer, drawing on the source material `../provenance.md` cites;
this `CONTEXT.md` is orientation-level per the draft package template, not
a finished, ready-to-run stage contract.)*

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition — describes shape for this candidate's own
future runs, not populated at draft time.
