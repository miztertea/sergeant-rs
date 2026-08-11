# Output — `10-sketch-seams`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — The fewest new seams at the highest possible seam, confirmed with the user; the workflow deliverable — a fixed template published to the tracker with the ready label (folded in from the demoted `20-write-and-publish` stage, N1 adjudication A4).

**Disposition:** `promote`

This is a workflow deliverable: it survives into the merge under the finalize policy (`docs/icm/convention.md` §1a open question 1 — "silence promotes nothing"; a `promote` artifact is kept explicitly). This stage absorbed the former terminal stage `20-write-and-publish`'s `promote` disposition when that stage was demoted (N1 adjudication A4).
