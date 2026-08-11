# Output — `10-inventory`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory
is empty in the authored tree; a run of this stage writes its artifact here
in the materialized work surface, Git-tracked on the Work branch and
reviewable in the diff like any other change.

**Expected artifact:** `inventory.md` — every in-scope file, its
disposition (`decompose` / `helper-evidence` / `obsolete-candidate` /
`reference-only`), a one-line description, a reason, and — for `decompose`
rows — its named partition, per `../CONTEXT.md` and
`references/dispositions.md`. Include a disposition-count and
partition-count summary; the counts must sum to the total files enumerated.

**Disposition:** `promote`

The source inventory is one of this workflow's own declared outputs (§9.1's
`OUTPUT` list) and the evidentiary basis for `20-harvest`'s coverage — a
reviewer checking source coverage after the merge needs this file present,
not discarded as scratch.
