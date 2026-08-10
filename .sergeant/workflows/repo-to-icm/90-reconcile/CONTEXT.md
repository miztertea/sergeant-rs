# 90-reconcile: adjudicate, measure, close the run

## Inputs

| File | Layer | Why |
|---|---|---|
| references/reconciliation-method.md | L3 | the adjudication method, measurement-package shape, and grammar-pressure consolidation rule |
| ../_config/icm-ladder.md | L3 | the six-field engine-gap template §3 (grammar-pressure consolidation) requires, verbatim field names |
| ../scripts/finalize.py | L3 | the helper this stage runs as its closing act (step 4) |
| ../_config/run-discipline.md | L3 | the blindness rule and the `# AMBIGUOUS — NOT RESOLVED` propagation rule |
| ../00-contract/output/contract.md | L4 | upstream — run identity (subject repo + revision) for the measurement package |
| ../10-inventory/output/inventory.md | L4 | upstream — source-coverage stats, and any recorded unreached paths (meta-level grammar pressure) |
| ../20-harvest/output/behavior-units.ndjson | L4 | upstream — extraction-coverage stats, including any recorded unreached partitions (meta-level grammar pressure) |
| ../30-normalize/output/behavior-units.normalized.ndjson | L4 | upstream — normalization outcome stats |
| ../40-classify/output/classifications.ndjson | L4 | upstream — representation-mix stats and the classification records this stage's adjudication may amend |
| ../50-synthesize/output/candidates.md | L4 | upstream — candidate-yield stats |
| ../60-draft/output/draft-report.md | L4 | upstream — draft-materialization manifest, the permanent-instruction/obsolete-mechanism/engine-pressure candidates carried through it, and any meta-level grammar-pressure note it recorded |
| ../70-lint/output/lint-report.md | L4 | upstream — draft-validity stats (mechanical vs. substantive defect counts) |
| ../80-adversarial-review/output/findings.ndjson | L4 | upstream — the findings this stage adjudicates |
| ../80-adversarial-review/output/review-summary.md | L4 | upstream — review-convergence stats (which candidates/axes were actually applied) |

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

## Working directory

Commands below are written **from the repository root** (the directory
`.sergeant/` lives directly under) — this run's actual working directory,
not `90-reconcile/` itself (`docs/gauntlet/notes/n2-fake-backend-semantics.md`).

## How to do it

0. If any upstream artifact named in the Inputs table above opens with `#
   AMBIGUOUS — NOT RESOLVED`, do not proceed with steps 1–4 below — follow
   `../_config/run-discipline.md` §2 instead: record which upstream stage
   never got real facts to work with, and close the run honestly rather
   than adjudicating and measuring against artifacts you know are hollow.

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
4. Only after 1–3 are complete and written to their output files, close the
   run in this exact order — `finalize.py`'s `git rm` fails on a file that
   was never `git add`ed, and its `git commit` takes no pathspec (it
   commits whatever is currently staged), so staging deliberately and in
   the right order is what makes the closing commit both complete and free
   of anything unrelated:
   1. `git add` every stage's populated `output/` directory under this
      workflow's own tree (from the repository root:
      `git add .sergeant/workflows/repo-to-icm/*/output/`). If anything
      *else* happens to be staged in this worktree right now, unstage it
      first (`git restore --staged <path>`) — the closing commit should
      contain only this run's own artifacts, not an unrelated staged
      change riding along.
   2. Preview the disposition plan: `python3
      .sergeant/workflows/repo-to-icm/scripts/finalize.py --dry-run`.
      Append its verbatim output to the end of
      `output/measurement-package.md`, under a `## Finalize` heading.
   3. `git add` `output/measurement-package.md` again now that it carries
      the preview — this is what gets the finalize record itself inside
      the same closing commit finalize.py is about to make, instead of a
      dirty file left over after it.
   4. Run `python3 .sergeant/workflows/repo-to-icm/scripts/finalize.py`
      (no `--dry-run`, no other argument) for real. Its single `git commit`
      (no pathspec) now captures everything staged in steps 1 and 3 plus
      its own `git rm` of evidence-class/undeclared files, in one commit —
      the "final commit" this run's own D9 disposition policy promises.
      This is a helper: review its result, do not assume the engine
      interpreted it for you (`docs/icm/convention.md` §5). If it exits
      `REFUSED` (ambiguous disposition), that is a real defect to record,
      not to route around by hand-editing files it should have finalized.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifacts and their disposition.
