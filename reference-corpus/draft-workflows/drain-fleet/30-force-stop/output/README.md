# Output — `30-force-stop`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — the drain set, convergence awaited, each worker's cooperative-drain checkpoint (helper invocations, N1 adjudication A4), force-stop applied only if still refused-unless-active and explicitly confirmed, and the drain then lifted.

**Disposition:** `promote`

This is a workflow deliverable: it survives into the merge under the finalize policy (`docs/icm/convention.md` §1a open question 1 — "silence promotes nothing"). It inherits `promote` from the former `40-undrain` stage, whose idempotent lifting of the drain was this workflow's only deliverable-grade artifact; the folded evidence steps (set-drain, await-convergence, worker-side checkpoint, force-stop itself) remain part of this same record as intermediate decisions, not separately promoted.
