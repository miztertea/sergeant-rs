# 40-validate: the single dedicated validation boundary

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-reconcile-existing-state/output/ | L4 | upstream artifact produced by this candidate's own `30-reconcile-existing-state` — this stage's trigger ("a worker reaches readiness") only fires meaningfully once reconciliation has settled |

## Purpose

Run validation exactly once as a dedicated boundary, triggered when a
worker reaches readiness or when remediation changes HEAD after readiness.
Post-readiness HEAD changes get rereview without retriggering the full
cycle.

## What must become true here (durable outcome)

Validation has run exactly once as a dedicated boundary for this task; any
HEAD change after readiness triggers rereview of the change alone, never a
full retrigger of the workflow from an earlier stage.

**Ordering note.** This stage's exact position in the source procedure's
numbered steps (5, 6, 7, or 8) is not pinned by direct evidence — see
`../provenance.md`'s `40-validate` entry and `../CONTEXT.md`'s "Ordering
note carried from synthesis." A human reviewer promoting this candidate
should treat the stage's *content* (what must become true) as evidenced,
and its *exact step number* as an open, challengeable placement.

## How to do it

*(Draft package — detailed method content is a promotion-time task for a
human reviewer, drawing on the source material `../provenance.md` cites;
this `CONTEXT.md` is orientation-level per the draft package template, not
a finished, ready-to-run stage contract.)*

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition — describes shape for this candidate's own
future runs, not populated at draft time.
