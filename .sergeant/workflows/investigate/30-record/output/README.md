# Output — `30-record`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This
directory is empty in the authored tree; a run of this stage writes its
artifact(s) here in the materialized work surface, Git-tracked on the Work
branch and reviewable in the diff like any other change.

**Expected artifact:** a pointer record naming the durable artifact's
actual repo path — the artifact itself is written outside `output/`, at
that named path, per this stage's own contract.

**Disposition:** `promote`
