# Output — `00-contract`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory
is empty in the authored tree; a run of this stage writes its artifact here
in the materialized work surface, Git-tracked on the Work branch and
reviewable in the diff like any other change.

**Expected artifact:** `contract.md` — the subject repository and pinned
revision, in-scope/excluded paths with reasons, a restatement of the
per-stage output-path convention, and this run's success criteria, per
`../CONTEXT.md`'s "What must become true here."

**Disposition:** `promote`

`contract.md` is what makes every other promoted artifact in this run
interpretable after the merge — it is the record of what was and was not in
scope, and at what revision. A reviewer reading the merged output without it
cannot tell whether an absence downstream means "not present in the subject"
or "excluded by contract." Every stage from `10-inventory` on depends on it
by name (Inputs table row, L4); it does not stop mattering once those
stages have consumed it.
