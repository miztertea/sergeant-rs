# Output — `10-dispatch-worker`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. Empty at
draft time — this describes artifact shape for **this candidate's own
future runs** once promoted, not an artifact of the `repo-to-icm` run that
materialized this draft package.

**Expected artifact:** a per-repository dispatch record — which repository
was targeted, the checkout path created for it, the brief written, and the
spawned agent session's identity.

**Disposition:** `promote`

The dispatch record is what `dispatch-mode`'s later monitoring and
reconciliation steps would need to find the worker they are tracking.
