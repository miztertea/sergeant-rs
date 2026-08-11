# Output — `20-check-queue`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. Empty at
draft time — this describes artifact shape for **this candidate's own
future runs** once promoted, not an artifact of the `repo-to-icm` run that
materialized this draft package.

**Expected artifact:** the resolved canonical task identity — either the
existing task found, or the new one created — and which of the two
occurred.

**Disposition:** `promote`

Downstream stages need a single settled task identity to reconcile state
against and eventually deliver.
