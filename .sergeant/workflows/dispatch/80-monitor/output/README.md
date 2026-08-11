# Output — `80-monitor`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — fleet reconciliation, tracked-work creation, surface acquisition, brief rendering, and launch evidence (helper invocations, N1 adjudication A4), followed by escalations read in full, human decisions obtained without inference, delivered to the exact task/repo pair.

**Disposition:** `evidence`

Each of the five folded stages' own output declared `evidence` disposition before A4; the merged record keeps that disposition.

This is Work-branch evidence of how the stage's outcome was reached (inputs consulted, decisions made, intermediate state); it does not by itself survive into the merge unless a later stage's disposition promotes it by name.
