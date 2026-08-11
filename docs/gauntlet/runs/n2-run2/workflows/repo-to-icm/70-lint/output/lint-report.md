# 70-lint report

Produced by `70-lint` against every candidate package named in
`../60-draft/output/draft-report.md` (which did not open with `#
AMBIGUOUS — NOT RESOLVED`, so ordinary validation proceeded per
`../_config/run-discipline.md` §2), plus this workflow's own tree, per
`references/mechanical-vs-substantive.md`.

All runs used `.sergeant/workflows/repo-to-icm/scripts/validate-structure.py`,
invoked from the repository root.

## `dispatch-mode`

Path: `.sergeant/drafts/workflows/dispatch-mode/`

**Initial validator result:** FAIL — 1 defect.

- `[S12] dispatch-mode: outputs are declared but the closing stage
  10-dispatch-worker names no finalize step (docs/icm/convention.md §1a,
  D9)`

**Classification:** substantive. `10-dispatch-worker/CONTEXT.md`'s own "How
to do it" section states plainly that it is orientation-level only —
"Detailed method content is a promotion-time task for a human reviewer...
this `CONTEXT.md` is orientation-level per the draft package template, not
a finished, ready-to-run stage contract." Naming a finalize step means
deciding what that step actually does; per
`references/mechanical-vs-substantive.md`'s test this is new judgment
(inventing missing content), not aligning the file with a fact already
established elsewhere in this run's own artifacts. Not fixed.

**Mechanical repairs applied:** none.

**Final validator result:** FAIL — same 1 defect (`[S12]`), unchanged,
left for `80-adversarial-review` / `90-reconcile`.

## `standard-task-workflow`

Path: `.sergeant/drafts/workflows/standard-task-workflow/`

**Initial validator result:** FAIL — 1 defect.

- `[S12] standard-task-workflow: outputs are declared but the closing
  stage 50-reconcile-and-deliver names no finalize step
  (docs/icm/convention.md §1a, D9)`

**Classification:** substantive, for the same reason as `dispatch-mode`
above: `50-reconcile-and-deliver/CONTEXT.md`'s own "How to do it" section
states "detailed method content is a promotion-time task for a human
reviewer... this `CONTEXT.md` is orientation-level per the draft package
template, not a finished, ready-to-run stage contract." Naming a finalize
step here requires authoring new procedure content this stage was not
given the authority to invent. Not fixed.

**Mechanical repairs applied:** none.

**Final validator result:** FAIL — same 1 defect (`[S12]`), unchanged,
left for `80-adversarial-review` / `90-reconcile`. (Note: `40-validate`'s
step-number placement ambiguity flagged in `../60-draft/output/
draft-report.md`'s manifest notes is not a structural-lint concern — the
validator has no check for step-number placement within a stage's own
`CONTEXT.md` prose — so it does not surface here; it remains open for
`80`/`90` as `60-draft` recorded it.)

## `ship-with-no-mistakes`

Path: `.sergeant/drafts/workflows/ship-with-no-mistakes/`

**Initial validator result:** FAIL — 1 defect.

- `[S3] ship-with-no-mistakes: workflow.toml has no non-empty
  workflow.stages array`

**Classification:** substantive. This is not an accidental gap: `../60-
draft/output/draft-report.md`'s "Judgment call" section and this
package's own `workflow.toml` header comment and `CONTEXT.md` all state
plainly and deliberately that `stages = []` because this run's evidence
supports zero classified `stage` records for this candidate, and inventing
stage boundaries from the unattached `stage-context` records
(`BU-0028`–`BU-0034`) would be exactly the unsupported-invention defect
`provenance.md` itself flags as a risk. Per
`references/mechanical-vs-substantive.md`, deciding what stages this
workflow should have is a boundary/classification judgment, not a fact
already established elsewhere in this run to align the file to. Not fixed.

**Mechanical repairs applied:** none.

**Final validator result:** FAIL — same 1 defect (`[S3]`), unchanged. This
is real signal, already anticipated by `60-draft` and by this package's own
`CONTEXT.md` ("not promotable as-is... a human reviewer needs to either (a)
... define real stage candidates... or (b) reconsider whether
`ship-with-no-mistakes` is better represented some other way entirely"),
carried forward as-is for `80-adversarial-review` / `90-reconcile`.

## This workflow's own tree

Path: `.sergeant/workflows/repo-to-icm/` (validated with no path argument,
admitted mode).

**Initial validator result:** FAIL — 1 defect.

- `[S10] repo-to-icm: 20-harvest/quote.sh is executable but is not named
  by any CONTEXT.md/_config file in this package (unclassified machinery,
  convention.md §5 rule 1)`

`engine-gap records checked: 0` — zero `representation: engine-gap`
records exist anywhere under this tree's NDJSON files, including the
freshly-written `40-classify/output/classifications.ndjson` (6 lines
mention the string `engine-gap`, all inside `alternatives_considered`
arrays with `representation: shared-helper` and `engine_gap: null` — none
is an actual `engine-gap` record). This matches `../60-draft/output/
draft-report.md`'s bucket 7 finding ("Zero `representation: engine-gap`
records exist in this corpus"). No `[S9]` defects, so nothing here for
this stage to decline to force-fix on that specific ground.

**Classification of the `[S10]` finding:** substantive, not fixed, for two
independent reasons:

1. **Genuine ambiguity about the correct repair.** `references/
   mechanical-vs-substantive.md` lists "a stray executable bit on a file
   that is not meant to be run" as mechanical (fix: strip the bit). But
   `20-harvest/quote.sh`'s content (`sed -n START,ENDp file | sha256sum`,
   plus a JSON-escaping step) closely mirrors the exact capture-once-
   reuse-twice recipe `_config/evidence-policy.md` itself documents inline
   — strong evidence it *is* meant to be run, not stray. If so, the
   correct mechanical-looking fix would run the other direction (name it
   in `20-harvest/CONTEXT.md` as a helper) rather than strip its
   executable bit — but deciding which of those two repairs is correct
   requires judging authorial intent for a script this stage did not
   write, which `references/mechanical-vs-substantive.md`'s own guidance
   ("when genuinely unsure, treat it as substantive") covers directly.
2. **This is the admitted tree, not a draft candidate.** `references/
   mechanical-vs-substantive.md`'s process and the mechanical-fix examples
   are framed around candidate packages `60-draft` materializes; this
   stage's own `CONTEXT.md` says the authored tree "passes cleanly by
   construction" and frames the no-argument re-check as existing
   specifically to catch `[S9]` engine-gap issues in the freshly-written
   `classifications.ndjson` — not as a general license to edit
   `.sergeant/workflows/repo-to-icm` itself. `[S10]` here is unrelated to
   `40-classify`'s output and predates this run; editing the admitted
   workflow's own source material is outside what this stage's contract
   authorizes it to decide on its own.

**Mechanical repairs applied:** none.

**Final validator result:** FAIL — same 1 defect (`[S10]`), unchanged,
recorded here as a finding for `80-adversarial-review` / `90-reconcile`
(or a human maintainer) to resolve: either document `20-harvest/quote.sh`
as this package's own citation-hashing helper, or remove its executable
bit if it is in fact stray.

## Repository-wide (not attributable to any one candidate)

None. No `[S7]` finding appeared in any of the four validator runs above
(no draft was found misplaced under `.sergeant/workflows/`, and no
admitted package was found misplaced under `.sergeant/drafts/`).

## Summary

| Tree | Initial | Mechanical fixes | Final | Substantive findings remaining |
|---|---|---|---|---|
| `dispatch-mode` | FAIL (1) | 0 | FAIL (1) | `[S12]` no finalize step named |
| `standard-task-workflow` | FAIL (1) | 0 | FAIL (1) | `[S12]` no finalize step named |
| `ship-with-no-mistakes` | FAIL (1) | 0 | FAIL (1) | `[S3]` deliberate empty stages array |
| This workflow's own tree | FAIL (1) | 0 | FAIL (1) | `[S10]` `20-harvest/quote.sh` unclassified |

No mechanical defects were found in any of the four validator runs — every
reported defect required judgment this stage was not given the authority
to exercise on its own, so every candidate (and this workflow's own tree)
is passed through to `80-adversarial-review` / `90-reconcile` with its
initial defect(s) still present and explicitly classified above, per
`references/mechanical-vs-substantive.md`'s closing guidance that a
substantive defect merely logged "loses nothing, since the next stage
exists precisely to look at it with fresh eyes."
