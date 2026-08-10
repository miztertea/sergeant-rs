# 60-draft: materialize draft workflow packages

## Inputs

| File | Layer | Why |
|---|---|---|
| references/draft-package-template.md | L3 | the exact four-layer package shape every materialized candidate must have |
| ../_config/icm-ladder.md | L3 | the representation vocabulary each candidate's classification records were assigned from |
| ../_config/run-discipline.md | L3 | the blindness rule and the `# AMBIGUOUS — NOT RESOLVED` propagation rule |
| ../50-synthesize/output/candidates.md | L4 | upstream artifact produced by `50-synthesize` — the named, clustered candidates this stage materializes |
| ../40-classify/output/classifications.ndjson | L4 | upstream artifact produced by `40-classify` — the classification records whose `behavior_id`s populate each package's `provenance.md` |

## Purpose

Materialize each workflow candidate from `../50-synthesize/output/candidates.md`
as a complete draft ICM workflow package under
`.sergeant/drafts/workflows/<candidate-name>/` in **this run's own
worktree** — never `.sergeant/workflows/`. This is the draft boundary
(`docs/icm/convention.md` §2): correctness of the generated content never
substitutes for the human review that promotion requires, so nothing this
stage writes lands in the runnable namespace.

## What must become true here (durable outcome)

Every workflow candidate from `50-synthesize` is materialized as a package
matching `references/draft-package-template.md` exactly: `index.md`
(`status: draft`), `workflow.toml`, `CONTEXT.md` (orientation only),
`provenance.md` (every stage and the workflow as a whole traced to
`behavior_id`s, or explicitly marked as a justified design inference), each
stage's `CONTEXT.md` with a correctly-layered `Inputs` table, and each
stage's `output/README.md` declaring that *candidate's own* future-run
artifact shape (never populated). Permanent-instruction, obsolete-mechanism,
and engine-pressure candidates from `50-synthesize` are **not** materialized
as packages (they are not workflows) — they are carried forward in this
stage's own `output/draft-report.md` instead, for `90-reconcile` to use.

## How to do it

0. If `../50-synthesize/output/candidates.md` opens with `#
   AMBIGUOUS — NOT RESOLVED`, do not proceed — follow
   `../_config/run-discipline.md` §2.

1. Before writing anything, re-check every candidate name against
   `.sergeant/workflows/`, `.sergeant/drafts/workflows/`, and every other
   candidate name from this same run — including `repo-to-icm` itself.
   `.sergeant/drafts/workflows/` (and even `.sergeant/drafts/`) may not
   exist yet in a fresh worktree — that is not an error, it means there is
   nothing under it to collide with yet; create it as an ordinary side
   effect of writing this run's first candidate package (`mkdir -p`
   semantics), nothing to flag or ask about. A collision must be resolved
   (rename the candidate, recording why) before materializing it.
2. For each workflow candidate, write the package per
   `references/draft-package-template.md`, section by section. Stage-context
   attachments from `50-synthesize` become guidance content inside their
   attached stage's own `CONTEXT.md` — not new stage directories.
3. Write `provenance.md` mapping every stage (and the workflow as a whole) to
   its member `behavior_id`(s). A stage with no direct source evidence is
   marked as a justified design inference with a one-line reason, never left
   silent and never given an invented citation.
4. Do not populate any candidate's own `NN-.../output/` beyond `README.md` —
   that directory describes shape for the candidate's future runs, not this
   run's artifacts.
5. Do not use an `@@name`-style shared-context reference unless it resolves
   to a real, already-existing `.sergeant/common/contexts/<name>.md`;
   otherwise write the content out in full.
6. After every candidate package is written, record what you produced in
   `output/draft-report.md`: a manifest of materialized package paths, plus
   the carried-through permanent-instruction, obsolete-mechanism, and
   engine-pressure candidate lists from `50-synthesize` (verbatim — this
   stage does not edit them).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition. Note: the *materialized packages themselves*
live under `.sergeant/drafts/workflows/`, outside this stage's own
`output/` directory — `output/draft-report.md` is this run's record of what
was produced and where, not the packages themselves.

**Meta-level grammar pressure, record for `90-reconcile`.** The materialized
packages are this run's principal deliverable, yet the D9 disposition/
finalize mechanism (`docs/icm/convention.md` §1a) only governs a stage's own
`output/` — it has no lower-rung way to give per-run content written
*elsewhere* in the worktree a disposition, or to bring it under
`../scripts/finalize.py`'s reach. This is a genuine could-not-express
moment, not something this stage is silently accepting as fine: state it
plainly in `output/draft-report.md` (a one-line note is enough — the six-
field engine-gap template is `90-reconcile`'s job to write, from this
recorded moment, per `../_config/run-discipline.md` and
`90-reconcile/references/reconciliation-method.md` §3).
