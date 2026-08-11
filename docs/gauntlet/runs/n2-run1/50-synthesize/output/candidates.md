# AMBIGUOUS — NOT RESOLVED

Relayed from the upstream artifact named in this stage's own Inputs table,
`../40-classify/output/classifications.ndjson`, which opens with this same
heading. Per `../_config/run-discipline.md` §2 and this stage's own
`CONTEXT.md` step 0 ("If `../40-classify/output/classifications.ndjson`
opens with `# AMBIGUOUS — NOT RESOLVED`, do not proceed"), this stage does
not proceed with its ordinary work (clustering classified units into the
seven `references/synthesis-method.md` buckets) — none of that is possible
without a resolved classification ledger to cluster, which `40-classify`
was unable to produce because `30-normalize`, `20-harvest`, `10-inventory`,
and `00-contract`, upstream of it, were unable to establish a pinned
subject revision and scope. This document is the mechanical propagation of
that unresolved state, not a re-diagnosis.

What is ambiguous, quoted from
`../40-classify/output/classifications.ndjson`'s "What is ambiguous" line
(itself relaying `../30-normalize/output/behavior-units.normalized.ndjson`,
itself relaying `20-harvest/output/behavior-units.ndjson`, itself relaying
`10-inventory/output/inventory.md`, itself relaying
`00-contract/output/contract.md`):

> What is ambiguous: the pinned revision of this run's subject cannot be
> established, and the subject identification itself rests on inference
> rather than an explicit statement in the Work's initiating task.

No candidate buckets (workflow, stage, stage-context, permanent-instruction,
shared helper/context, obsolete-mechanism, or engine-pressure) are produced
in place of the above — per `../_config/run-discipline.md` §2, this is the
fail-closed marker itself, not a partial candidates ledger for `60-draft`
to build on.

See `../40-classify/output/classifications.ndjson` in full for its own
relay, and `../00-contract/output/contract.md` for the complete diagnosis
("What was checked" list and the meta-level grammar-pressure note), both
of which this stage relays rather than repeats.
