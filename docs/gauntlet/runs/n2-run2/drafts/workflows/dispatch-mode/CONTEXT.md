# dispatch-mode — workflow orientation

Layer 1. This file orients an actor entering this **draft** workflow for
the first time; it is not stage instruction — no stage substitutes reading
this for its own `CONTEXT.md` contract. Only the first stage lists this
file in its Inputs table, mirroring the convention this candidate was
itself materialized from (`docs/icm/convention.md` §1a rule 5).

## What this workflow is for

Dispatch mode fires when work spans repositories, has independent
repo-owned sub-tasks, needs isolated review, or the user explicitly
requests workers. Its outcome is one worker dispatched per owning
repository; its completion condition is progress monitored through to
reconciliation of merge order and cross-repo implications.

## How the stage hands off

This candidate has exactly one member stage in this corpus,
`10-dispatch-worker`: the checkpoint that exists once a worker's isolated
checkout, written brief, and spawned interactive agent session are all in
place for a given repository. Monitoring and reconciliation of the
resulting workers (merge order, cross-repo implications) are named in this
candidate's trigger/outcome text but have no member `stage` record of
their own in this corpus — see `provenance.md`. A human reviewer promoting
this candidate should treat that as open ground to fill in, not as
settled.

## Status

`status: draft`. This package lives under `.sergeant/drafts/workflows/`
and is not runnable procedure until a human promotes it into
`.sergeant/workflows/` (`docs/icm/convention.md` §2).
