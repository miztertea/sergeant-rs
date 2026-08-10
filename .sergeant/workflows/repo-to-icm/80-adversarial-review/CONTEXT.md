# 80-adversarial-review: fresh-eyes challenge

## Inputs

| File | Layer | Why |
|---|---|---|
| references/challenge-checklist.md | L3 | the three challenge axes (boundary honesty, invention, engine-gap refutation) and how to apply each |
| ../00-contract/output/README.md | L4 | upstream artifact produced by `00-contract` — this run's scope/exclusion record, needed to check the blindness boundary was actually honored |
| ../40-classify/output/README.md | L4 | upstream artifact produced by `40-classify` — the classification records (rationale, alternatives, engine-gap claims) this review interrogates directly |
| ../60-draft/output/README.md | L4 | upstream artifact produced by `60-draft` — the manifest naming every materialized candidate package this review challenges |
| ../70-lint/output/README.md | L4 | upstream artifact produced by `70-lint` — the lint report, so already-fixed mechanical defects are not re-litigated as findings |

## You are a fresh execution

You have not seen any earlier stage's conversation or reasoning. Everything
you know about this run comes from this `CONTEXT.md`, the files named above,
and the worktree as it now stands. Do not assume good faith of earlier
stages — your job here is specifically to find what is wrong, not to
confirm what looks reasonable at a glance.

## The blindness rule still applies to you

This run's actors — including this stage — are blind to `reference-corpus/`
for the entire run (`../00-contract`'s exclusion record is why it should
never have entered scope downstream). **Do not open, grep the contents of,
or read anything under `reference-corpus/`.** Your job is to challenge this
run's own internal consistency and evidentiary rigor, not to check it
against a gold answer — that comparison is a separate process performed
later, by different comparers, outside this workflow
(`docs/gauntlet/contracts/N2.md` Outcome §3). Performing it here would both
contaminate this run's own record and short-circuit the actual measurement
this workflow exists to support. One of your checks (below) is specifically
to verify no earlier stage crossed this line — that check does not require
you to cross it yourself.

## Purpose

Apply the three challenge axes in `references/challenge-checklist.md` —
boundary honesty, invention, engine-gap refutation — to everything this run
has produced so far. Produce findings; do not fix anything yourself
(reconciliation and repair are `90-reconcile`'s job).

## What must become true here (durable outcome)

`output/findings.ndjson` exists (possibly empty, if genuinely nothing was
found — but only after real effort under all three axes) and
`output/review-summary.md` states which candidate packages and which axes
were actually applied, with finding counts by axis and severity. A finding
you did not write down did not happen, as far as `90-reconcile` is
concerned.

## How to do it

Work through `references/challenge-checklist.md`'s three axes in order,
against every candidate package `../60-draft` materialized:

1. **Boundary honesty** — publication boundary, layer boundary, blindness
   boundary (grep for the literal string `reference-corpus` across every
   artifact this run produced), name-collision boundary.
2. **Invention** — re-verify a real sample of citations (recompute several
   `quote_hash` values yourself), re-verify provenance citations resolve to
   real `behavior_id`s, check rationale discrimination, re-apply the
   reimplementation test to every `stage`-rung record, compare each
   materialized package's actual shape against what was classified and
   synthesized to catch hidden translation.
3. **Engine-gap refutation** — independently re-attempt every lower rung for
   every surviving `engine-gap` record; check `why_each_fails` for
   rung-specific reasoning and `observable_acceptance_test` for a checkable
   scenario; disqualify anything reading as "would be convenient."

Write every finding to `output/findings.ndjson` per this record shape:

```json
{"id": "AF-0001", "axis": "invention", "target": "<candidate>/provenance.md", "description": "...", "evidence": "...", "severity": "high"}
```

`axis` is one of `boundary-honesty`, `invention`, `engine-gap-refutation`.
`severity` is `high` (a violation as described in this workflow's governing
documents), `medium` (a real weakness that does not rise to a documented
violation), or `low` (worth recording, unlikely to change an outcome). This
stage does not assign accept/reject dispositions — that is `90-reconcile`'s
job.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifacts and their disposition.
