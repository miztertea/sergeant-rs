# Reconciliation method

Layer 3 (stable across runs), local to `90-reconcile`. Covers three things
this stage does: adjudicate `80-adversarial-review`'s findings, assemble the
measurement package, and consolidate the grammar-pressure report.

## 1. Adjudicating findings

For every finding in `../80-adversarial-review/output/findings.ndjson`,
record one disposition — `accept`, `reject`, or `park` — with a one-line
reason, in `output/adjudication-log.md`. This mirrors the reference corpus's
own adjudication method (`sergeant-rs-workspace/knowledge/evidence/gauntlet/contracts/N1.md`): evidence-only,
rulings recorded, disagreements preserved rather than erased.

- **accept**: the finding is correct; apply the repair directly to the
  affected file(s) in place (a draft package's content, a classification
  record, a citation's `confidence`/notes). Do not silently rewrite
  evidence — the adjudication-log entry states what changed and why, so the
  change is traceable even though the file itself now reads differently.
  This is the one point in the run where content already written by an
  earlier stage may be edited; do it deliberately, not incidentally.
- **reject**: the finding does not hold up (e.g. a claimed over-staging that
  actually does pass the reimplementation test on closer inspection). State
  why the challenge fails.
- **park**: real, but out of this run's scope to resolve now (e.g. a
  genuine ambiguity with no clearly correct resolution). State what would
  resolve it.

An `accept`ed engine-gap refutation (Axis 3 finding that found a working
lower rung) means that classification record's `representation` changes
away from `engine-gap` to the rung that was shown to work, with a new
`rationale` recording the refutation — it does **not** get carried into the
grammar-pressure report below.

## 2. The measurement package

`output/measurement-package.md` reports what this run can honestly measure
**without** the reference corpus — this run's actors never opened it, and
this stage does not either (see the blindness note in `../CONTEXT.md`).

Proposal §9.9 names ten measurement dimensions. This file states all ten by
name here (so recovering them never requires opening the proposal, which no
stage's Inputs table can name — `docs/icm/convention.md` §1a rule 1 governs
what this stage may depend on, and the proposal lives outside this
workflow's own tree). Five require comparing against `sergeant-rs-workspace/knowledge/evidence/reference-corpus/`
and are **out of reach for this run's blindness rule**; five are
**internally computable** from this run's own artifacts alone. Report both
groups explicitly — a dimension silently missing from either group is a
gap, not a shortcut:

```text
INTERNALLY COMPUTABLE (report a real value for each, from the stage named):

source coverage           file counts by disposition, partition count
                           (../10-inventory/output/inventory.md); whether
                           every partition reached `done`
                           (../20-harvest/output/partition-ledger.md) and
                           whether the consequence-class sweep covers every
                           `decompose` file with no blank cells
                           (../20-harvest/output/consequence-class-sweep.md)
behavioral precision       of the citations THIS RUN itself sampled and
                           reverified: fraction of ../80-adversarial-review's
                           Axis 2 citation-reverification sample that
                           verified cleanly (no invention finding) — record
                           numerator/denominator, e.g. "9/10 sampled
                           citations verified", not a bare percentage from a
                           small sample (../80-adversarial-review/output/
                           findings.ndjson, review-summary.md)
provenance completeness    fraction of materialized stages/candidates whose
                           provenance.md cites at least one real behavior_id
                           (../70-lint/output/lint-report.md's [S8] results
                           per candidate, cross-checked against any Axis-2
                           invention findings on fabricated citations)
draft validity             validator pass/fail per candidate, mechanical vs.
                           substantive defect counts (../70-lint)
review convergence         finding counts by axis/severity, accept/reject/
                           park counts (this stage's own adjudication-log.md)

NOT COVERED HERE (require sergeant-rs-workspace/knowledge/evidence/reference-corpus/ comparison — a separate, later
process performed by independent comparers not bound by this run's
blindness rule, sergeant-rs-workspace/knowledge/evidence/gauntlet/contracts/N2.md Outcome §3):

behavioral recall, workflow-boundary agreement, stage-boundary agreement,
representation agreement, engine-gap quality

SUPPORTING RUN STATISTICS (context for the above, not §9.9 dimensions in
their own right — report them too, but do not mistake this list for the
ten-dimension list):

run identity              subject repo + revision (../00-contract)
extraction coverage       unit count, any unreached partitions (../20-harvest)
normalization outcome     unit count after rewrite/split, confidence shifts
                          (../30-normalize)
representation mix        classification record counts by `representation`
                          (../40-classify)
candidate yield           counts of workflow/stage/helper/shared/invariant/
                          obsolete-mechanism/engine-pressure candidates
                          (../50-synthesize)
draft materialization     candidate packages produced, paths (../60-draft)
```

State plainly, once, which five §9.9 dimensions this package does **not**
and cannot cover from inside this run, using the exact names above.
Reporting this honestly is itself part of the "generator preserves
uncertainty instead of inventing confidence" gate item (proposal §22.1 item
9) — do not attempt to estimate those five dimensions from inside this run
to fill the gap.

## 3. The grammar-pressure report

`output/grammar-pressure.ndjson` collects every genuine could-not-express
moment this run surfaced, each as a full six-field engine-gap-claim record
(`docs/icm/record-shapes.md` §5 / `../_config/icm-ladder.md` §6.7 —
verbatim field names, all six required). Two distinct sources feed it, and
both use the same template:

- **Behavior-level.** Every `representation: engine-gap` classification
  record that survived adjudication above (i.e. was not downgraded by an
  accepted refutation finding). Carry its `engine_gap` object through
  unchanged, tagged `"source": "behavior"` with the originating
  `behavior_id`.
- **Meta-level.** Pressure this workflow's *own* stages hit while executing
  — e.g. `10-inventory` or `20-harvest` recording paths/partitions they
  could not reach within one turn because no fan-out exists;
  `60-draft` recording that its materialized packages sit outside the D9
  disposition/finalize mechanism entirely (no stage `output/` governs
  per-run content written elsewhere in the worktree); a candidate needing a
  shared sub-procedure with its own retry/measurement that
  `docs/icm/convention.md` §4 rule 1 rules out expressing through `@@name`;
  `00-contract` recording that this turn had no way to pause and ask a
  human when the subject/revision/scope was ambiguous
  (`../_config/run-discipline.md` §2); `70-lint` finding a substantive
  defect no lower-rung repair mechanism in this grammar can fix without
  inventing judgment machinery it was not given. Scan every upstream
  stage's own output for this kind of explicitly-recorded gap (do not
  invent one that was not actually recorded — this is consolidation, not
  new discovery) and write each as a full six-field record, tagged
  `"source": "meta"` and naming which stage surfaced it.

### Record shape

Each line is a wrapper around the six-field `engine_gap` template, nested
exactly as a classification record nests it (never flattened to top
level), and **never carrying a `representation` field** — this is not a
classification record, and `scripts/validate-structure.py`'s `[S9]` check
only inspects records whose `representation` is literally `engine-gap`;
omitting that field here keeps this consolidation file correctly out of
`[S9]`'s scope (which governs classification ledgers, not this file).

Behavior-level (`source: behavior`) — `behavior_id` identifies the
originating unit:

```json
{"source": "behavior", "behavior_id": "EX-0117", "engine_gap": {"behavior": "...", "source_evidence": ["EX-0117"], "lower_rungs_attempted": ["..."], "why_each_fails": {"...": "..."}, "minimum_runtime_capability_required": "...", "observable_acceptance_test": "..."}}
```

Meta-level (`source: meta`) — there is no behavior unit behind a meta-level
gap, so `stage` (which stage's own output surfaced the moment) replaces
`behavior_id`, and the nested template's own `source_evidence` field names
*where the gap was recorded* instead of a `behavior_id` list — a pointer
precise enough for a reader to re-open the actual recorded note (e.g.
`"20-harvest/output/behavior-units.ndjson: coverage note, partitions P4/P7
not reached"`), never a bare restatement of the gap itself:

```json
{"source": "meta", "stage": "20-harvest", "engine_gap": {"behavior": "...", "source_evidence": ["20-harvest/output/behavior-units.ndjson: coverage note, partitions P4/P7 not reached"], "lower_rungs_attempted": ["..."], "why_each_fails": {"...": "..."}, "minimum_runtime_capability_required": "...", "observable_acceptance_test": "..."}}
```

Both variants carry all six `engine_gap` fields, verbatim field names, all
populated — the same completeness bar `40-classify` applies (§6.7: missing
`lower_rungs_attempted` or `why_each_fails` is auto-rejected, not merely
flagged).

A moment that amounts to "we ran out of turn budget" or "this would have
been more convenient with branching" without a rung-specific mechanical
reason is **not** grammar pressure — exclude it, or note it separately as
an operational limitation, never inflate it into an engine-gap record
(record-shapes.md §5 rule 4 applies here exactly as it does at `40-classify`).

## 4. Closing the run

Only after 1–3 are written: run `../scripts/finalize.py` (no path argument,
so it targets this workflow's own root) and record its output verbatim at
the end of `output/measurement-package.md`. It applies this workflow's own
`output/` disposition policy across every stage of *this run* — actor and
`65-self-check`'s execute stage alike — it does not touch any candidate
package's own (empty) `output/` directories.
