# 20-check-queue: reuse an existing canonical task instead of duplicating

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-load-context/output/ | L4 | upstream artifact produced by this candidate's own `10-load-context` — the loaded context and selected execution mode this stage checks a queue within |

## Purpose

Check whether a canonical task already exists for this work before a new
one is created — task creation genuinely branches on this outcome.

## What must become true here (durable outcome)

Either an existing canonical task is found and reused, or its absence is
confirmed and a new one is created — never both, and never skipped.

## How to do it

*(Draft package — detailed method content is a promotion-time task for a
human reviewer, drawing on the source material `../provenance.md` cites;
this `CONTEXT.md` is orientation-level per the draft package template, not
a finished, ready-to-run stage contract.)*

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition — describes shape for this candidate's own
future runs, not populated at draft time.
