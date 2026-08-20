# Output — `10-transcribe-decisions`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This
directory is empty in the authored tree; a run of this stage writes its
artifact(s) here in the materialized work surface, Git-tracked on the
Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — the transcribed ADR/glossary
material: every decision in the brief, with its alternatives and
rejection reasons, and every rationale/alternative/rejection-reason gap
the brief did not carry logged explicitly rather than invented.

**Disposition:** `promote`

This is a workflow deliverable: it survives into the merge under the
finalize policy (`docs/icm/convention.md` §1a open question 1 — "silence
promotes nothing"; a `promote` artifact is kept explicitly).
