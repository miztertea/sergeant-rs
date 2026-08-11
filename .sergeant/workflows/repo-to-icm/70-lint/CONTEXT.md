# 70-lint: validate and mechanically repair the draft tree

## Inputs

| File | Layer | Why |
|---|---|---|
| references/mechanical-vs-substantive.md | L3 | the line between defects this stage may fix directly and defects it must leave for `80`/`90` |
| ../_config/run-discipline.md | L3 | the blindness rule and the `# AMBIGUOUS — NOT RESOLVED` propagation rule |
| ../scripts/validate-structure.py | L3 | the helper this stage runs, both against each candidate package and against this workflow's own run worktree (see "How to do it") |
| ../60-draft/output/draft-report.md | L4 | upstream artifact produced by `60-draft` — the manifest naming every candidate package this stage validates |
| ../40-classify/output/classifications.ndjson | L4 | upstream — `references/mechanical-vs-substantive.md`'s substantive test for a missing `engine_gap` field requires checking whether the correct content is already recoverable verbatim from here |

## Working directory

Every command below is written **from the repository root** — the same
directory `.sergeant/` lives directly under, which is this run's actual
working directory (the materialized work surface's single bound worktree;
`docs/gauntlet/notes/n2-fake-backend-semantics.md`'s cwd is a fresh actor
turn's cwd, not any one stage's own directory). Do not run these commands
from inside `70-lint/` itself, and do not assume `../scripts/...` resolves
— it only would from inside this stage's own directory, which is not where
this turn starts.

## Purpose

Run `.sergeant/workflows/repo-to-icm/scripts/validate-structure.py` against
every candidate package `60-draft` materialized; repair the defects that
are mechanical per `references/mechanical-vs-substantive.md`; leave
substantive ones for `80-adversarial-review` and `90-reconcile`. This stage
**also** re-validates this workflow's own tree (no path argument) — the
*authored* tree passes cleanly by construction, but this is a *run*
worktree: `40-classify` has just written new `classifications.ndjson` into
it, and that is exactly the kind of NDJSON the validator's S9 check (engine-
gap record completeness) exists to scan. Skipping the no-argument run would
mean this run's own engine-gap records are never checked by anything.

## What must become true here (durable outcome)

`output/lint-report.md` exists, covering every candidate package named in
`../60-draft/output/draft-report.md`, **plus** this workflow's own tree: the
validator's initial findings for each, each classified mechanical-fixed or
substantive-remaining per `references/mechanical-vs-substantive.md`, and
the final validator result after mechanical repairs (pass, or fail with the
substantive defects still listed). No candidate is silently skipped, and
this workflow's own tree is not silently assumed clean.

## How to do it

0. If `../60-draft/output/draft-report.md` opens with `#
   AMBIGUOUS — NOT RESOLVED`, do not proceed — follow
   `../_config/run-discipline.md` §2.

For each candidate package path from `../60-draft/output/draft-report.md`
(paths there are relative to the repository root, e.g.
`.sergeant/drafts/workflows/<candidate-name>`):

1. Run `python3 .sergeant/workflows/repo-to-icm/scripts/validate-structure.py
   <candidate-path>`. Review its structured result — this is a helper, not
   something the engine interprets on its own; the judgment about what its
   output means is yours (`docs/icm/convention.md` §5).
2. Classify each reported defect using `references/mechanical-vs-substantive.md`'s
   test. When genuinely unsure, treat it as substantive.
3. Fix every mechanical defect directly in the candidate package.
4. Re-run the validator. Repeat 1–3 until no mechanical defects remain.
5. Record the final state — validator PASS/FAIL, defects fixed, defects
   remaining (with their `[Sn]` codes) — in `output/lint-report.md` under
   that candidate's own heading. **Exception:** an `[S7]` finding whose
   package name is not the candidate you just validated is a repository-
   wide check result (it compares `.sergeant/workflows/` against
   `.sergeant/drafts/workflows/` as a whole, independent of which single
   tree you pointed the validator at) — it is not attributable to this
   candidate. Record it at most once, under a `## Repository-wide (not
   attributable to any one candidate)` heading, not repeated under every
   candidate's own heading it happens to surface under.

After every candidate has been processed:

6. Run `python3 .sergeant/workflows/repo-to-icm/scripts/validate-structure.py`
   (no argument — validates this workflow's own tree as admitted). Record
   its result under a `## This workflow's own tree` heading in
   `output/lint-report.md`, the same way as a candidate: PASS/FAIL, and any
   `[S9]` engine-gap defects found in `40-classify/output/classifications.ndjson`
   are substantive findings for `80`/`90`, not something this stage
   force-fixes.

A candidate (or this workflow's own tree) that still fails after mechanical
repairs is not a failure of this stage; it is real signal for
`80-adversarial-review` and `90-reconcile` to work from. Do not force a
mechanical-looking fix onto a substantive defect just to make the validator
pass.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition.
