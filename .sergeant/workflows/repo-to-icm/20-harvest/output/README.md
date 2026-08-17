# Output — `20-harvest`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory
is empty in the authored tree; a run of this stage writes its artifact here
in the materialized work surface, Git-tracked on the Work branch and
reviewable in the diff like any other change.

**Expected artifacts:**

| File | Disposition |
|---|---|
| behavior-units.ndjson | promote |
| partition-ledger.md | promote |
| consequence-class-sweep.md | promote |

`behavior-units.ndjson` — one behavior-unit record per line, in the shape
`../_config/evidence-policy.md` defines, covering every
`decompose`-dispositioned file/partition from `10-inventory` (or recording,
for any not reached, why not). `partition-ledger.md` — the per-partition
`done`/`pending` checkpoint record `references/partition-checkpoint-
protocol.md` defines; this is what makes an incomplete run's own coverage
gap durable and resumable instead of a fact that only lived in one attempt's
now-gone context window (this is the GP-5b fix applied at the source: an
artifact this workflow's own coverage-honesty claim depends on is declared
`promote` from the start, not left undeclared for `finalize.py` to silently
sweep away). `consequence-class-sweep.md` — the five-class hunt-list record
`references/consequence-class-checklist.md` defines, one row per
`decompose` file.

All three are `promote`. Extracted behavior units are this workflow's
central deliverable — the "normalized behavior units" and everything
downstream (classification, synthesis, draft workflows) traces back to this
file's records by `id`. The partition ledger and consequence-class sweep are
the run's own honesty record about coverage and about the specific
consequence-bearing behavior classes N2 run 2 silently missed
(`sergeant-rs-workspace/knowledge/evidence/gauntlet/runs/n2-run2/comparison-scorecard.md` §3) — all three must
survive the merge for any of that to be checkable after the run is over.
