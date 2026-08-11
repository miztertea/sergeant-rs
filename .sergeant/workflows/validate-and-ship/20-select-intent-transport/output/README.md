# Output — `20-select-intent-transport`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — (coordinator-launched entry only) readiness verified, launch reservation acquired, isolated snapshot reserved and re-verified (folds the demoted `00-verify-readiness`, `10-acquire-launch-reservation`, `20-reserve-isolated-snapshot` checkpoints, N1 adjudication A4); then, for either entry, the intent transport probed against the installed build's real capability, decided once with explicit consent for the exposing option, and recorded twice for audit.

**Disposition:** `evidence`

This is Work-branch evidence of how the stage's outcome was reached (inputs consulted, decisions made, intermediate state); it does not by itself survive into the merge unless a later stage's disposition promotes it by name.
