# Output — `10-load-context`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. Empty at
draft time — this describes artifact shape for **this candidate's own
future runs** once promoted, not an artifact of the `repo-to-icm` run that
materialized this draft package.

**Expected artifact:** a record of the loaded context (owning
repository/repositories, inherited instructions, resolved paths, cross-repo
dependencies) and the execution mode selected on its basis.

**Disposition:** `promote`

Downstream stages (`20-check-queue` onward) need the selected execution
mode and resolved repository/paths to act on without redoing this stage's
own resolution work.
