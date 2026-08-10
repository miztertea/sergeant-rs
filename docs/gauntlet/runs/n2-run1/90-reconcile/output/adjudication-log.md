# AMBIGUOUS — NOT RESOLVED

Relayed from the upstream artifacts named in this stage's own Inputs
table: `../00-contract/output/contract.md`, `../10-inventory/output/
inventory.md`, `../20-harvest/output/behavior-units.ndjson`,
`../30-normalize/output/behavior-units.normalized.ndjson`,
`../40-classify/output/classifications.ndjson`, `../50-synthesize/output/
candidates.md`, `../60-draft/output/draft-report.md`, and `../70-lint/
output/lint-report.md` — all eight open with this same heading. Per
`../_config/run-discipline.md` §2 and this stage's own `CONTEXT.md` step 0
("If any upstream artifact named in the Inputs table above opens with
`# AMBIGUOUS — NOT RESOLVED`, do not proceed with steps 1–4 below"), this
stage does not proceed with its ordinary adjudication work (assigning
accept/reject/park to each `80-adversarial-review` finding and applying
accepted repairs to the affected files) — that work is only meaningful
against a real corpus (behavior units, classification records, synthesis
candidates, draft packages) that `00-contract` was unable to establish a
subject and revision for, and that every intermediate stage through
`70-lint` correctly declined to fabricate in its place. This document is
the mechanical propagation of that unresolved state, not a re-diagnosis.

What is ambiguous, quoted from `00-contract/output/contract.md`'s "What is
ambiguous" line (relayed unchanged through every intermediate stage):

> What is ambiguous: the pinned revision of this run's subject cannot be
> established, and the subject identification itself rests on inference
> rather than an explicit statement in the Work's initiating task.

## The two findings this stage's Inputs table does carry

Unlike the eight artifacts above, `../80-adversarial-review/output/
findings.ndjson` and `../80-adversarial-review/output/review-summary.md`
are **not** themselves headed `# AMBIGUOUS — NOT RESOLVED` — `80-
adversarial-review` ran its own real review (per its own `CONTEXT.md`
step 0) and recorded two genuine findings:

- **AF-0001** (boundary-honesty, high) — observes exactly the propagation
  this document itself is also relaying: every Inputs-table artifact from
  `00-contract` through `70-lint` opens with the AMBIGUOUS heading, cleanly
  and without invention, and no candidate package was ever materialized.
- **AF-0002** (invention, medium) — a specific evidentiary claim in
  `00-contract/output/contract.md`'s "What was checked" list (that
  `git -C reference/sergeant-upstream rev-parse --is-inside-work-tree`
  fails) does not reproduce; the sibling check that actually controls the
  vendored-subtree classification (`ls reference/sergeant-upstream/.git`)
  does reproduce, so AF-0002 does not itself overturn the AMBIGUOUS
  determination, but the specific claim it flags is not accurate as
  written.

Neither finding is assigned an accept/reject/park disposition here, and no
repair is applied to `00-contract/output/contract.md` or any other file.
Disposing AF-0001 or AF-0002 — even AF-0002's narrow, seemingly
self-contained correction — is exactly the kind of "ordinary durable
outcome" `../_config/run-discipline.md` §2 forecloses once any
Inputs-table artifact carries the marker: this stage's own contract
(`CONTEXT.md` step 0) says "do not proceed with steps 1–4," step 1 being
adjudication itself, without carving out an exception for a finding whose
substance happens not to touch the subject/revision question directly.
Both findings remain genuine and unresolved, left for a future run of this
stage once `00-contract` can actually establish a subject and revision —
re-opening `../80-adversarial-review/output/findings.ndjson` at that time
will find them exactly as recorded, undisposed.

## Step 4 (closing the run) also not executed

Per the same step-0 instruction, `../scripts/finalize.py` was not run this
pass. See `output/measurement-package.md`'s own note on this for the full
explanation — recorded there once rather than repeated per file.

See `../00-contract/output/contract.md` in full for the complete diagnosis
("What was checked" list and the meta-level grammar-pressure note) that
every stage in this chain, including this one, relays rather than repeats.
