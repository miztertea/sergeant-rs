# Output — `00-inspect-preview`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — Secrets, duplicate entities, wrong outcomes, unresolved errors are checked; a secret stops the run and only the source *class* is recorded; and (per N1 adjudication A4, folded from the now-removed `40-publish-and-index`) the published `~/wiki/sessions/YYYY-MM-DD.md` page and its `~/wiki/index.md` link.

**Disposition:** `promote`

This is a workflow deliverable: it survives into the merge under the finalize policy (`docs/icm/convention.md` §1a open question 1 — "silence promotes nothing"; a `promote` artifact is kept explicitly). This stage's disposition changed from `evidence` to `promote` under A4: it absorbed `40-publish-and-index`, which was this workflow's original `promote`-disposition terminal stage.
