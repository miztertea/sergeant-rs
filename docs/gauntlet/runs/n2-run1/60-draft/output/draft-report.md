# AMBIGUOUS — NOT RESOLVED

Relayed from the upstream artifact named in this stage's own Inputs table,
`../50-synthesize/output/candidates.md`, which opens with this same
heading. Per `../_config/run-discipline.md` §2 and this stage's own
`CONTEXT.md` step 0 ("If `../50-synthesize/output/candidates.md` opens with
`# AMBIGUOUS — NOT RESOLVED`, do not proceed"), this stage does not proceed
with its ordinary work (materializing each workflow candidate from
`candidates.md` as a draft ICM workflow package under
`.sergeant/drafts/workflows/`) — none of that is possible without a
resolved candidates ledger to materialize, which `50-synthesize` was unable
to produce because `40-classify`, `30-normalize`, `20-harvest`,
`10-inventory`, and `00-contract`, upstream of it, were unable to establish
a pinned subject revision and scope. This document is the mechanical
propagation of that unresolved state, not a re-diagnosis.

What is ambiguous, quoted from `../50-synthesize/output/candidates.md`'s
"What is ambiguous" line (itself relaying
`../40-classify/output/classifications.ndjson`, itself relaying
`../30-normalize/output/behavior-units.normalized.ndjson`, itself relaying
`20-harvest/output/behavior-units.ndjson`, itself relaying
`10-inventory/output/inventory.md`, itself relaying
`00-contract/output/contract.md`):

> What is ambiguous: the pinned revision of this run's subject cannot be
> established, and the subject identification itself rests on inference
> rather than an explicit statement in the Work's initiating task.

No draft workflow packages are materialized under
`.sergeant/drafts/workflows/` in place of the above — per
`../_config/run-discipline.md` §2, this is the fail-closed marker itself,
not a partial manifest for `70-lint`, `80-adversarial-review`, or
`90-reconcile` to build on. No candidate package directories were created;
`.sergeant/drafts/workflows/` was not created by this run (there is nothing
to materialize, so the `mkdir -p` side effect described in this stage's own
`CONTEXT.md` step 1 never triggers). The permanent-instruction,
obsolete-mechanism, and engine-pressure candidate lists this stage would
otherwise carry through unchanged from `../50-synthesize/output/
candidates.md` are likewise absent upstream — there is nothing to carry.

See `../50-synthesize/output/candidates.md` in full for its own relay, and
`../00-contract/output/contract.md` for the complete diagnosis ("What was
checked" list and the meta-level grammar-pressure note), both of which this
stage relays rather than repeats.

## Meta-level grammar pressure (recorded for `90-reconcile`, per this
## stage's own `CONTEXT.md`)

Not applicable to this run of `60-draft`: no packages were materialized
outside `output/`, so the D9 disposition/finalize gap this stage's
`CONTEXT.md` asks it to record for `90-reconcile` — that the disposition/
finalize mechanism only governs a stage's own `output/`, with no lower-rung
way to give per-run content written elsewhere in the worktree a
disposition — never actually arose on this run. Recorded here only so
`90-reconcile` does not need to infer why the note is missing: it is
missing because the AMBIGUOUS short-circuit fired before this stage did
any of the work that would have surfaced it, not because the pressure was
overlooked.
