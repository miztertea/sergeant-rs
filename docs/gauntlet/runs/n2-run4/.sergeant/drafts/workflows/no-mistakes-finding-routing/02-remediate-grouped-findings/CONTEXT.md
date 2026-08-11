# 02-remediate-grouped-findings

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-route-finding/output/outcome.md | L4 | upstream evidence produced by `route-finding` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** multiple findings share the same root cause

**Outcome:** remediation converges to one worker per root cause, is rechecked before merge, and escalates to a human after two unsuccessful cycles rather than looping indefinitely

**Statement (the operative rule):** Findings sharing the same originating run, head, owning module, and root cause share one serialized remediation worker/branch rather than one worker per finding; before merging the group, native tests and independent rereviews (verifying mutation before validation, partial publication or rollback, and identity/provenance) are rerun; after two remediation cycles, fix dispatch stops and an architectural/root-cause review plus a human decision is required.

## What must become true here (durable outcome)

Remediation converges to one worker per root cause, is rechecked before merge, and escalates to a human after two unsuccessful cycles rather than looping indefinitely — per the Statement above, which is the operative rule this stage exists to enforce.

