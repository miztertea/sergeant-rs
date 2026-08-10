# Output — `50-synthesize`

Layer 4 (per-run artifact), per `docs/icm/convention.md` §1a. This directory
is empty in the authored tree; a run of this stage writes its artifact here
in the materialized work surface, Git-tracked on the Work branch and
reviewable in the diff like any other change.

**Expected artifact:** `candidates.md` — the seven clustered buckets from
`references/synthesis-method.md` (workflow/stage candidates,
stage-context attachments, permanent-instruction candidates, shared
helper/context candidates, obsolete-mechanism findings, engine-pressure
candidates), with every classification record from `../40-classify/output/
classifications.ndjson` accounted for in exactly one bucket appearance.

**Disposition:** `promote`

This is the bridge between the flat classification ledger and the
materialized draft packages (`60-draft`) — it is also where
permanent-instruction, obsolete-mechanism, and engine-pressure candidates
first take named shape, and `90-reconcile`'s grammar-pressure report traces
back to it. It must survive the merge for either purpose.
