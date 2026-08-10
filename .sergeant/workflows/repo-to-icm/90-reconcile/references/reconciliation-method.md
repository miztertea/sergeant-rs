# Reconciliation method

Layer 3 (stable across runs), local to `90-reconcile`. Covers three things
this stage does: adjudicate `80-adversarial-review`'s findings, assemble the
measurement package, and consolidate the grammar-pressure report.

## 1. Adjudicating findings

For every finding in `../80-adversarial-review/output/findings.ndjson`,
record one disposition — `accept`, `reject`, or `park` — with a one-line
reason, in `output/adjudication-log.md`. This mirrors the reference corpus's
own adjudication method (`docs/gauntlet/contracts/N1.md`): evidence-only,
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
Pull from every upstream stage's own declared output:

```text
run identity           subject repo + revision (../00-contract)
source coverage         file counts by disposition, partition count (../10-inventory)
extraction coverage      unit count, any unreached partitions (../20-harvest)
normalization outcome    unit count after rewrite/split, confidence shifts (../30-normalize)
representation mix       classification record counts by `representation` (../40-classify)
candidate yield          counts of workflow/stage/helper/shared/invariant/
                         obsolete-mechanism/engine-pressure candidates (../50-synthesize)
draft materialization    candidate packages produced, paths (../60-draft)
draft validity           validator pass/fail per candidate, mechanical vs.
                         substantive defect counts (../70-lint)
review convergence       finding counts by axis/severity, accept/reject/park
                         counts (this stage's own adjudication-log.md)
```

State plainly, once, which of proposal §9.9's ten measurement dimensions
this package does **not** and cannot cover from inside this run —
behavioral recall, workflow-boundary agreement, stage-boundary agreement,
representation agreement, and engine-gap quality all require comparing
against `reference-corpus/`, which is a separate, later process performed
by independent comparers who are not bound by this run's blindness rule
(`docs/gauntlet/contracts/N2.md` Outcome §3). Reporting this honestly is
itself part of the "generator preserves uncertainty instead of inventing
confidence" gate item (proposal §22.1 item 9) — do not attempt to estimate
those five dimensions from inside this run to fill the gap.

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
  — e.g. `20-harvest` recording partitions it could not reach within one
  turn because no fan-out exists; a candidate needing a shared sub-procedure
  with its own retry/measurement that `docs/icm/convention.md` §4 rule 1
  rules out expressing through `@@name`; `70-lint` finding a substantive
  defect no lower-rung repair mechanism in this grammar can fix without
  inventing judgment machinery it was not given. Scan every upstream
  stage's own output for this kind of explicitly-recorded gap (do not
  invent one that was not actually recorded — this is consolidation, not
  new discovery) and write each as a full six-field record, tagged
  `"source": "meta"` and naming which stage surfaced it.

A moment that amounts to "we ran out of turn budget" or "this would have
been more convenient with branching" without a rung-specific mechanical
reason is **not** grammar pressure — exclude it, or note it separately as
an operational limitation, never inflate it into an engine-gap record
(record-shapes.md §5 rule 4 applies here exactly as it does at `40-classify`).

## 4. Closing the run

Only after 1–3 are written: run `../scripts/finalize.py` (no path argument,
so it targets this workflow's own root) and record its output verbatim at
the end of `output/measurement-package.md`. It applies this workflow's own
`output/` disposition policy across all ten stages of *this run* — it does
not touch any candidate package's own (empty) `output/` directories.
