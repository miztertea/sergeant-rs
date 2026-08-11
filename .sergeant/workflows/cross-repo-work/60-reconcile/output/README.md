# Output — `60-reconcile`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — PR URLs, heads, CI, review threads, merge and deployment order, terminal task/fleet state.

**Disposition:** `promote`

This is a workflow deliverable: it survives into the merge under the finalize policy (`docs/icm/convention.md` §1a open question 1 — "silence promotes nothing"; a `promote` artifact is kept explicitly).

**Curation note (added at promotion, `docs/icm/promotion-spec-2026-08-11.md` §1):** this is `cross-repo-work`'s true closing stage (per `workflow.toml`'s own stage order) and it declares a `promote` output, but the workflow has no dedicated finalize step (D9, convention §1a open questions — not a numbered Rule). Recorded here per the promotion spec's instruction to surface the gap rather than launder it silently; not a defect finding and not a change to this stage's adjudicated disposition above.
