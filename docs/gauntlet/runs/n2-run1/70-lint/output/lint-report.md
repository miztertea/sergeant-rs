# AMBIGUOUS — NOT RESOLVED

Relayed from the upstream artifact named in this stage's own Inputs table,
`../60-draft/output/draft-report.md`, which opens with this same heading.
Per `../_config/run-discipline.md` §2 and this stage's own `CONTEXT.md`
step 0 ("If `../60-draft/output/draft-report.md` opens with
`# AMBIGUOUS — NOT RESOLVED`, do not proceed"), this stage does not proceed
with its ordinary work (running `../scripts/validate-structure.py` against
every candidate package named in `draft-report.md`, classifying and
mechanically repairing defects) — none of that is possible without a
resolved manifest of candidate packages, which `60-draft` was unable to
produce because `50-synthesize`, `40-classify`, `30-normalize`,
`20-harvest`, `10-inventory`, and `00-contract`, upstream of it, were
unable to establish a pinned subject revision and scope. This document is
the mechanical propagation of that unresolved state, not a re-diagnosis.

What is ambiguous, quoted from `../60-draft/output/draft-report.md`'s "What
is ambiguous" line (itself relaying `../50-synthesize/output/candidates.md`,
itself relaying `../40-classify/output/classifications.ndjson`, itself
relaying `../30-normalize/output/behavior-units.normalized.ndjson`, itself
relaying `20-harvest/output/behavior-units.ndjson`, itself relaying
`10-inventory/output/inventory.md`, itself relaying
`00-contract/output/contract.md`):

> What is ambiguous: the pinned revision of this run's subject cannot be
> established, and the subject identification itself rests on inference
> rather than an explicit statement in the Work's initiating task.

No candidate packages were validated or linted in place of the above — per
`../_config/run-discipline.md` §2, this is the fail-closed marker itself,
not a partial `lint-report.md` for `80-adversarial-review` or
`90-reconcile` to build on. `.sergeant/drafts/workflows/` does not exist in
this worktree (confirmed by directory listing before writing this report),
consistent with `60-draft`'s own relay that nothing was materialized.

## This workflow's own tree

Step 6 of this stage's `CONTEXT.md` (running
`python3 .sergeant/workflows/repo-to-icm/scripts/validate-structure.py`
with no argument, against this workflow's own run tree) was **not**
executed. Step 0's fail-closed instruction ("do not proceed") takes
precedence over step 6, since step 6 is part of the ordinary work this
stage was told not to perform once the upstream marker was found — running
the no-argument validation pass and reasoning about its `[S9]` findings
against `../40-classify/output/classifications.ndjson` would be exactly
the kind of substantive judgment this stage has no resolved basis to
exercise on this run. This is recorded here, rather than left silent, so
`90-reconcile` does not have to infer why the section is missing: it is
missing because the AMBIGUOUS short-circuit fired before this stage did
any of its ordinary work, not because the check was overlooked.

## Repository-wide (not attributable to any one candidate)

Not applicable — no validator run means no `[S7]` repository-wide finding
was produced on this run.

See `../60-draft/output/draft-report.md` in full for its own relay, and
`../00-contract/output/contract.md` for the complete diagnosis ("What was
checked" list and the meta-level grammar-pressure note), both of which this
stage relays rather than repeats.
