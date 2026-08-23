# Output — `20-panel`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This
directory is empty in the authored tree; a run of this stage writes its
artifact(s) here in the materialized work surface, Git-tracked on the Work
branch and reviewable in the diff like any other change.

**Expected artifact:** `findings.md` — the typed finding table (§2.7 of
the design record: `id`, `axis`, `claim`, `evidence`, `severity`,
`status`, `refutation`), every row at `status: raised`, plus the four
seats' verbatim reports beneath it, tagged by axis; any missing axis named
explicitly.

**Disposition:** `promote`
