# Output — `10-map-frontier`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — Breadth-first mapping; stop and do not create a map if no fog exists; specifiable decisions become child issues first, blocking edges wired in a second pass (folded in from the demoted `20-create-tickets` stage, N1 adjudication A4).

**Disposition:** `evidence`

This is Work-branch evidence of how the stage's outcome was reached (inputs consulted, decisions made, intermediate state); it does not by itself survive into the merge unless a later stage's disposition promotes it by name.
