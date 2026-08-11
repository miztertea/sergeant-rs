# Output — `60-draft`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory
is empty in the authored tree; a run of this stage writes its artifact here
in the materialized work surface, Git-tracked on the Work branch and
reviewable in the diff like any other change.

**Expected artifact:** `draft-report.md` — a manifest of every draft
workflow package materialized under `.sergeant/drafts/workflows/` by this
run (candidate name and path), plus the permanent-instruction,
obsolete-mechanism, and engine-pressure candidate lists carried through
unchanged from `../50-synthesize/output/candidates.md` (§9.1's `OUTPUT`
list items that are not themselves workflow packages).

**Disposition:** `promote`

The materialized packages themselves live under `.sergeant/drafts/
workflows/`, outside this stage's own `output/` — this file is the pointer
and carry-through record `70-lint`, `80-adversarial-review`, and
`90-reconcile` all depend on by name. Note: the draft packages' own
(empty, templated) `NN-.../output/` directories describe artifact shape for
each *candidate's own future runs* — they are not this run's artifacts and
carry no disposition of their own here; `../scripts/finalize.py` does not
touch them (see its own module docstring).
