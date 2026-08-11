# Output — `00-observe-and-interpret`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — the bounded, versioned activity snapshot (`busy:true`/`busy:null`, helper invocation 1), each in-progress worker's liveness evaluation against the identity-plus-progress fallback chain (helper invocation 2), and this stage's own interpretation: a healthy/stalled determination per worker, with any stalled worker's non-terminal diagnostic recorded rather than acted on.

**Disposition:** `promote`

This is a workflow deliverable: it survives into the merge under the finalize policy (`docs/icm/convention.md` §1a open question 1 — "silence promotes nothing"). Judgment call recorded here (no prior stage's disposition to inherit — see `provenance.md`'s "Adjudication A4" section): before the A4 fold, both `00-snapshot` and `10-evaluate-liveness` were `evidence`-disposition, because each fed a later stage's own decision. After the fold this is the workflow's only stage, and its interpreted output is exactly what the package's trigger promises a caller ("a live view of the fleet") — an `evidence`-only disposition here would mean the workflow silently promotes nothing, contradicting its own purpose. `promote` is therefore the correct disposition for the merged record, not an inheritance from either folded stage.
