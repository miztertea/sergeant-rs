# Output — `00-check-scope`

Layer 4 (per-run artifact), per `.sergeant/common/contexts/icm-policy.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — the invocation mode determined (validate-only or task-first), any user request translated into concrete pipeline flags, and the declared delivery state (or the assumed `validated-working-tree` floor, stated as assumed).

**Disposition:** `evidence`

This is Work-branch evidence of how the stage's outcome was reached (inputs consulted, decisions made, intermediate state); it does not by itself survive into the merge unless a later stage's disposition promotes it by name.
