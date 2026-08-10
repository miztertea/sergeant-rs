# 90-reconcile: adjudicate, measure, close the run

## Inputs

| File | Layer | Why |
|---|---|---|
| references/reconciliation-method.md | L3 | the adjudication method, measurement-package shape, and grammar-pressure consolidation rule |
| ../00-contract/output/README.md | L4 | upstream — run identity (subject repo + revision) for the measurement package |
| ../10-inventory/output/README.md | L4 | upstream — source-coverage stats |
| ../20-harvest/output/README.md | L4 | upstream — extraction-coverage stats, including any recorded unreached partitions (meta-level grammar pressure) |
| ../30-normalize/output/README.md | L4 | upstream — normalization outcome stats |
| ../40-classify/output/README.md | L4 | upstream — representation-mix stats and the classification records this stage's adjudication may amend |
| ../50-synthesize/output/README.md | L4 | upstream — candidate-yield stats |
| ../60-draft/output/README.md | L4 | upstream — draft-materialization manifest and the permanent-instruction/obsolete-mechanism/engine-pressure candidates carried through it |
| ../70-lint/output/README.md | L4 | upstream — draft-validity stats (mechanical vs. substantive defect counts) |
| ../80-adversarial-review/output/README.md | L4 | upstream — the findings this stage adjudicates, and review-convergence stats |

## The blindness rule still applies to you, and to what you produce

Do not open `reference-corpus/`. The measurement package this stage emits
reports only what this run can honestly compute **without** it — the five
comparison-dependent dimensions (behavioral recall, workflow-boundary
agreement, stage-boundary agreement, representation agreement, engine-gap
quality) are explicitly named as *not covered here* per
`references/reconciliation-method.md` §2, not estimated or guessed at. That
comparison happens later, separately, performed by comparers this run's
blindness rule does not bind (`docs/gauntlet/contracts/N2.md` Outcome §3).
Reporting the boundary of what you can and cannot measure honestly is part
of this stage's job, not a gap to paper over.

## Purpose

Close the run: adjudicate every finding from `80-adversarial-review`, apply
accepted repairs, assemble the measurement package, consolidate the
grammar-pressure report, and finalize this workflow's own per-run
`output/` dispositions.

## What must become true here (durable outcome)

`output/adjudication-log.md` disposes every finding in
`../80-adversarial-review/output/findings.ndjson` (accept/reject/park, each
with a reason); accepted findings are applied to the affected files in
place. `output/measurement-package.md` reports the internally-computable
`§9.9` dimensions and names the five it explicitly does not cover.
`output/grammar-pressure.ndjson` carries every surviving engine-gap claim
(behavior-level and meta-level) as a full six-field record. `../scripts/finalize.py`
has been run and its result is recorded.

## How to do it

1. Work through `../80-adversarial-review/output/findings.ndjson` per
   `references/reconciliation-method.md` §1: accept, reject, or park each,
   with a one-line reason, into `output/adjudication-log.md`. Apply accepted
   repairs directly to the affected files (a draft package's content, a
   classification record's `representation`/`rationale`, a citation's
   `confidence`) — state what changed in the log entry.
2. Assemble `output/measurement-package.md` per
   `references/reconciliation-method.md` §2, pulling stats from every
   upstream stage's output named in the Inputs table above.
3. Consolidate `output/grammar-pressure.ndjson` per
   `references/reconciliation-method.md` §3: every surviving (not
   adjudicated away) `engine-gap` classification record, tagged
   `source: behavior`; every genuinely recorded meta-level could-not-express
   moment from any upstream stage's own output, tagged `source: meta`. Do
   not invent a moment that was not actually recorded upstream, and do not
   include anything that only amounts to "ran out of turn budget" or "would
   have been more convenient."
4. Run `python3 ../scripts/finalize.py` (no argument — it defaults to this
   workflow's own root) only after 1–3 are complete and committed to their
   output files. Record its full output at the end of
   `output/measurement-package.md`. This is a helper: review its result, do
   not assume the engine interpreted it for you (`docs/icm/convention.md`
   §5).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifacts and their disposition.
