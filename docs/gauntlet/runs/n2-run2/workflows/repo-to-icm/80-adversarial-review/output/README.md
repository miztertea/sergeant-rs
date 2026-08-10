# Output — `80-adversarial-review`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory
is empty in the authored tree; a run of this stage writes its artifacts here
in the materialized work surface, Git-tracked on the Work branch and
reviewable in the diff like any other change.

**Expected artifacts:**

| File | Disposition |
|---|---|
| findings.ndjson | promote |
| review-summary.md | promote |

`findings.ndjson` — one finding per line, per `../CONTEXT.md`'s record
shape, tagged by axis (`boundary-honesty` / `invention` /
`engine-gap-refutation`) and severity. `review-summary.md` — which candidate
packages and axes were actually applied, plus finding counts.

Both are the raw evidence for the "Review convergence" measurement
dimension (§9.9) and the direct input to `90-reconcile`'s adjudication —
they must survive the merge for either purpose, independent of whether any
individual finding is later accepted or rejected.
