# Output — `10-triage-and-route`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — scope pinned (helper invocation, N1 adjudication A4), then full originating context read, redundant work checked, and the work classified into one of five categories, each loading a different canonical procedure.

**Disposition:** `evidence`

This is Work-branch evidence of how the stage's outcome was reached (inputs consulted, decisions made, intermediate state); it does not by itself survive into the merge unless a later stage's disposition promotes it by name.
