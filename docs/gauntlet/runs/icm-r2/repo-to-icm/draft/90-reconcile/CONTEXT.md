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
| ../20-harvest/output/behavior-units.ndjson | L4 | upstream — extraction-coverage stats |
| ../20-harvest/output/partition-ledger.md | L4 | upstream — whether every partition reached `done`; any `pending` row is meta-level grammar pressure (this run needed more than one `20-harvest` attempt and was not, or not yet, retried) |
| ../20-harvest/output/consequence-class-sweep.md | L4 | upstream — whether the five-class sweep actually covers every `decompose` file with no blank cells; a gap here is itself worth naming in the measurement package, not just a `40-classify` concern |
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

## Bounded judgment

Apply `@@bounded-judgment`.

A governing constraint (J5, `docs/icm/convention.md` §6.2/§6.3): this is
the one point in the run where earlier stages' already-written content may
be edited — every other stage's output is otherwise immutable once
written. That authority is scoped to applying an *accepted* finding's
repair, not to reopening settled content on this stage's own initiative.

### J2 — delegated to this stage
- `accept`/`reject`/`park` on every finding in
  `../80-adversarial-review/output/findings.ndjson`, with a one-line
  reason, and applying the accepted repair directly to the affected file.
- Assembling `output/measurement-package.md`'s internally-computable
  dimensions from upstream stats, and stating plainly which five §9.9
  dimensions this run cannot cover from inside its own blindness boundary.
- Consolidating `output/grammar-pressure.ndjson` from genuinely recorded
  behavior-level and meta-level moments — never inventing one that was not
  actually recorded upstream, and never inflating "ran out of turn budget"
  into an engine-gap record.

### J1 — local choices allowed
- Ordering within `output/adjudication-log.md` beyond "one entry per
  finding" — the required content (disposition + reason) is fixed.

### J0 — must become `needs_input`
- Any Inputs-table artifact opens with `# AMBIGUOUS — NOT RESOLVED` — do
  not proceed with steps 1–4; follow `../_config/run-discipline.md` §2
  instead: record which upstream stage never got real facts to work with,
  and close the run honestly rather than adjudicating and measuring
  against artifacts known to be hollow.

### Completion boundary
This stage may complete only after, in order: every finding disposed with a
reason and accepted repairs applied (`output/adjudication-log.md`); the
measurement package assembled, naming the five dimensions not covered here
(`output/measurement-package.md`); the grammar-pressure report consolidated
(`output/grammar-pressure.ndjson`); and `../scripts/finalize.py` run for
real, with its result recorded.

### Decision evidence
`output/adjudication-log.md` (accept/reject/park + reason per finding) is
this stage's primary decision record; `output/measurement-package.md` and
`output/grammar-pressure.ndjson` record the measurement and grammar-
pressure judgments respectively.

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
   was never `git add`ed, and its own closing `git commit` takes no
   pathspec (it commits whatever is currently staged), so staging
   deliberately and in the right order is what makes the closing commit(s)
   both complete and free of anything unrelated:
   1. `git add` every stage's populated `output/` directory under this
      workflow's own tree (from the repository root:
      `git add .sergeant/workflows/repo-to-icm/*/output/`). If anything
      *else* happens to be staged in this worktree right now, unstage it
      first (`git restore --staged <path>`) — the closing commit(s) should
      contain only this run's own artifacts, not an unrelated staged
      change riding along.
   2. Preview the disposition plan: `python3
      .sergeant/workflows/repo-to-icm/scripts/finalize.py --dry-run`.
      Append its verbatim output to the end of
      `output/measurement-package.md`, under a `## Finalize` heading.
   3. `git add` `output/measurement-package.md` again now that it carries
      the preview — this is what gets the finalize record itself inside
      finalize.py's own commit(s), instead of a dirty file left over after
      it.
   4. Run `python3 .sergeant/workflows/repo-to-icm/scripts/finalize.py`
      (no `--dry-run`, no other argument) for real. **This now produces two
      commits, not one, on an ordinary run** (evidence-preservation guard,
      GP-5b): first a *capture* commit of everything staged in steps 1 and
      3 (making every evidence-class/undeclared file reachable in history
      before anything is removed), then the *removal* commit (its own
      `git rm` of evidence-class/undeclared files) — the "final commit"
      this run's own D9 disposition policy promises is now the second of
      the two, and both are finalize.py's own doing in this one invocation,
      not a separate step you run. This is a helper: review its result, do
      not assume the engine interpreted it for you
      (`docs/icm/convention.md` §5). If it exits `REFUSED` — either an
      ambiguous disposition, or a file slated for removal that turned out
      to be neither staged nor committed at all (the literal GP-5b shape:
      something under `output/` that step 1's `git add` genuinely never
      reached) — that is a real defect to record, not to route around by
      hand-editing or hand-committing files it should have finalized.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifacts and their disposition.
