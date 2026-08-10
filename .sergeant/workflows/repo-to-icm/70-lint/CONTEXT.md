# 70-lint: validate and mechanically repair the draft tree

## Inputs

| File | Layer | Why |
|---|---|---|
| references/mechanical-vs-substantive.md | L3 | the line between defects this stage may fix directly and defects it must leave for `80`/`90` |
| ../60-draft/output/README.md | L4 | upstream artifact produced by `60-draft` — the manifest naming every candidate package this stage validates |

## Purpose

Run `../scripts/validate-structure.py` against every candidate package
`60-draft` materialized; repair the defects that are mechanical per
`references/mechanical-vs-substantive.md`; leave substantive ones for
`80-adversarial-review` and `90-reconcile`. This stage's own tree
(`repo-to-icm` itself) is expected to already pass the same validator run
with no arguments — that is a property of how this workflow was authored,
not something this stage needs to re-verify at run time.

## What must become true here (durable outcome)

`output/lint-report.md` exists, covering every candidate package named in
`../60-draft/output/draft-report.md`: the validator's initial findings, each
classified mechanical-fixed or substantive-remaining per
`references/mechanical-vs-substantive.md`, and the final validator result
after mechanical repairs (pass, or fail with the substantive defects still
listed). No candidate is silently skipped.

## How to do it

For each candidate package path from `../60-draft/output/draft-report.md`:

1. Run `python3 ../scripts/validate-structure.py <candidate-path>`. Review
   its structured result — this is a helper, not something the engine
   interprets on its own; the judgment about what its output means is
   yours (`docs/icm/convention.md` §5).
2. Classify each reported defect using `references/mechanical-vs-substantive.md`'s
   test. When genuinely unsure, treat it as substantive.
3. Fix every mechanical defect directly in the candidate package.
4. Re-run the validator. Repeat 1–3 until no mechanical defects remain.
5. Record the final state — validator PASS/FAIL, defects fixed, defects
   remaining (with their `[Sn]` codes) — in `output/lint-report.md` under
   that candidate's own heading.

A candidate that still fails after mechanical repairs is not a failure of
this stage; it is real signal for `80-adversarial-review` and
`90-reconcile` to work from. Do not force a mechanical-looking fix onto a
substantive defect just to make the validator pass.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition.
