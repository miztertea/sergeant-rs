# Output — `30-reconcile-existing-state`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. Empty at
draft time — this describes artifact shape for **this candidate's own
future runs** once promoted, not an artifact of the `repo-to-icm` run that
materialized this draft package.

**Expected artifact:** a reconciliation record — what in-flight state
(workers, branches, worktrees, retained gates, handoffs) was found, and
whether it was resumed, taken over, or confirmed absent.

**Disposition:** `promote`

`40-validate` needs to know whether it is waiting on reconciled in-flight
work or on freshly dispatched work.
