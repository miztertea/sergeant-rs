# Output — `50-escalate-undocumented`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory is empty in the authored tree; a run of this stage writes its artifact(s) here in the materialized work surface, Git-tracked on the Work branch and reviewable in the diff like any other change.

**Expected artifact:** a record of — An undocumented/unrecognized stall class escalates rather than being guessed at.

**Disposition:** `promote`

This is a workflow deliverable: it survives into the merge under the finalize policy (`docs/icm/convention.md` §1a open question 1 — "silence promotes nothing"; a `promote` artifact is kept explicitly).

Promotion note (`docs/icm/promotion-spec-2026-08-11.md` §1): this workflow declares a `promote` output and has no deterministic finalize step at its true closing stage — the D9 working rule under convention §1a's "Open questions (recorded, not resolved)" heading, not a numbered rule, so this is not a promotion blocker; recorded here rather than silently laundered.
