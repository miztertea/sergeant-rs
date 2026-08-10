# Mechanical versus substantive defects

Layer 3 (stable across runs), local to `70-lint`. `scripts/validate-structure.py`
finds defects; it does not distinguish which ones this stage may fix on its
own authority versus which ones must be left for `80-adversarial-review` and
`90-reconcile`. This file draws that line.

## The test

A defect is **mechanical** iff repairing it requires no new judgment about
what the package should mean — only making the file agree with a fact
already established elsewhere in this run's own artifacts. A defect is
**substantive** iff repairing it would require deciding something
(inventing missing evidence, choosing a different classification, judging
whether a boundary is correctly drawn) that this stage was not given the
authority to decide.

## Mechanical (fix immediately, then re-run the validator)

- `index.md` `name:` disagreeing with the actual directory name — align the
  field to the directory (the directory is the identity per
  `docs/icm/record-shapes.md` §1).
- `workflow.toml` `stages` list disagreeing with actual directory names only
  because of ordering/typo drift, where the correct order is unambiguous
  from the directory listing itself.
- An `Inputs` table row with the wrong `Layer` tag (e.g. an L4 row for a
  same-package earlier-stage `output/` path mislabeled `L3`) — correct the
  tag to match the rule in `docs/icm/record-shapes.md` §1a rule 2.
- A relative path in an `Inputs` row with an obvious typo (wrong number of
  `../`, a missing stage-directory segment) where exactly one real file in
  the tree matches the evident intent.
- A stray executable bit on a file that is not meant to be run (no
  `CONTEXT.md`/`_config` file in the package names it as a helper).
- A missing `**Disposition:**` line in a candidate's own stage
  `output/README.md` where the artifact description already makes the
  intended disposition unambiguous from context written elsewhere in the
  same file — otherwise leave it (see below).
- Whitespace/Markdown-table formatting that breaks the validator's parser
  (e.g. a missing blank line after an `## Inputs` table) without changing
  any row's content.

## Substantive (log as a finding for `80`/`90`, do not silently fix)

- A missing or empty `provenance.md`, or one that cites no real
  `behavior_id` — filling this in requires reconstructing evidence, which
  is exactly the invention risk `80-adversarial-review` exists to catch.
- An `engine_gap` object missing a required field, where the correct
  content of that field is not already recoverable verbatim from
  `../40-classify/output/classifications.ndjson` — do not draft a
  `why_each_fails` sentence yourself; that is authoring new judgment.
- A stage whose `Inputs` table is missing a row entirely (as opposed to a
  present row with a wrong tag or typo) — adding a new row asserts a new
  dependency claim this stage cannot verify on its own.
- Any disagreement about whether a boundary (workflow, stage,
  representation) is correctly drawn. Structure lint has no opinion on
  that; only judgment does.
- A missing `**Disposition:**` line where the artifact's own purpose is NOT
  obvious from the surrounding text — guessing `promote` vs `evidence` here
  is a content decision, not a mechanical repair.

## Process

1. Run `python3 .sergeant/workflows/repo-to-icm/scripts/validate-structure.py
   <candidate-path>` (from the repository root — see `../CONTEXT.md`
   "Working directory") for every candidate `60-draft` materialized.
2. Classify each reported defect using the test above.
3. Fix every mechanical defect directly in the candidate package; re-run the
   validator.
4. Repeat 1–3 until no more mechanical defects remain (substantive ones may
   remain — that is expected and is not a failure of this stage).
5. Record everything in `output/lint-report.md` per `../CONTEXT.md`,
   including the no-argument run against this workflow's own tree.

When genuinely unsure whether a defect is mechanical, treat it as
substantive — a wrongly "fixed" substantive defect hides real signal from
`80-adversarial-review`; a substantive defect merely logged instead of
force-fixed loses nothing, since the next stage exists precisely to look at
it with fresh eyes.
