# 80-adversarial-review: fresh-eyes challenge

## Inputs

| File | Layer | Why |
|---|---|---|
| references/challenge-checklist.md | L3 | the three challenge axes (boundary honesty, invention, engine-gap refutation) and how to apply each |
| ../_config/evidence-policy.md | L3 | the quote+hash recomputation procedure Axis 2's citation re-verification uses |
| ../_config/icm-ladder.md | L3 | the reimplementation test (Axis 2 over-staging) and the rungs Axis 3 must independently re-attempt |
| ../_config/run-discipline.md | L3 | the blindness rule this stage both follows itself and checks that earlier stages honored |
| ../00-contract/output/contract.md | L4 | upstream artifact produced by `00-contract` — this run's scope/exclusion record, needed to check the blindness boundary was actually honored |
| ../30-normalize/output/behavior-units.normalized.ndjson | L4 | upstream — the corpus Axis 2's citation-reverification sample is drawn from |
| ../40-classify/output/classifications.ndjson | L4 | upstream artifact produced by `40-classify` — the classification records (rationale, alternatives, engine-gap claims) this review interrogates directly |
| ../50-synthesize/output/candidates.md | L4 | upstream — what a materialized package's shape is checked against for Axis 2's hidden-translation check |
| ../60-draft/output/draft-report.md | L4 | upstream artifact produced by `60-draft` — the manifest naming every materialized candidate package this review challenges |
| ../70-lint/output/lint-report.md | L4 | upstream artifact produced by `70-lint` — the lint report, so already-fixed mechanical defects are not re-litigated as findings |
| ../10-inventory/output/inventory.md | L4 | upstream — the file/partition counts Axis 4's self-consistency check recomputes and cross-checks |
| ../20-harvest/output/behavior-units.ndjson | L4 | upstream — the raw (pre-normalize) corpus; Axis 4 recomputes its own distinct-`source.path` and unit counts rather than trusting any stage's stated headline number |
| ../20-harvest/output/partition-ledger.md | L4 | upstream — Axis 4 cross-checks this against `inventory.md`'s partition list and `behavior-units.ndjson`'s coverage |
| ../20-harvest/output/consequence-class-sweep.md | L4 | upstream — Axis 4 checks it names a row for every `decompose` file with no blank cells |

## You are a fresh execution

You have not seen any earlier stage's conversation or reasoning. Everything
you know about this run comes from this `CONTEXT.md`, the files named above,
and the worktree as it now stands. Do not assume good faith of earlier
stages — your job here is specifically to find what is wrong, not to
confirm what looks reasonable at a glance.

## The blindness rule still applies to you

This run's actors — including this stage — are blind to `sergeant-rs-workspace/knowledge/evidence/reference-corpus/`
for the entire run (`../00-contract`'s exclusion record is why it should
never have entered scope downstream). **Do not open, grep the contents of,
or read anything under `sergeant-rs-workspace/knowledge/evidence/reference-corpus/`.** Your job is to challenge this
run's own internal consistency and evidentiary rigor, not to check it
against a gold answer — that comparison is a separate process performed
later, by different comparers, outside this workflow
(`sergeant-rs-workspace/knowledge/evidence/gauntlet/contracts/N2.md` Outcome §3). Performing it here would both
contaminate this run's own record and short-circuit the actual measurement
this workflow exists to support. One of your checks (below) is specifically
to verify no earlier stage crossed this line — that check does not require
you to cross it yourself.

## Purpose

Apply the four challenge axes in `references/challenge-checklist.md` —
boundary honesty, invention, engine-gap refutation, structural
self-consistency — to everything this run has produced so far. Produce
findings; do not fix anything yourself (reconciliation and repair are
`90-reconcile`'s job).

## Bounded judgment

Apply `@@bounded-judgment`.

A governing constraint (J5, this stage's own contract and `docs/icm/
convention.md` §4.9's producer-does-not-self-promote rule): this stage may
not edit the implementation under review, and may not assign
accept/reject/park to its own findings — both are `90-reconcile`'s
authority, kept in a separate, later execution deliberately so the review
stays independent (`docs/icm/convention.md` §6.3).

### J2 — delegated to this stage
- Judging each finding's `severity` (`high`/`medium`/`low`) per the
  definitions in this stage's own record shape below.
- Classifying an ambiguous blindness-boundary hit (a `reference-corpus`
  string this stage cannot place as citation or as policy-quoting prose
  from context alone) as `medium` rather than guessing `high` or `low`.
- Deciding which citation sample and which engine-gap records genuinely
  received "tried harder than the classifying stage did" re-attempts, and
  stating so in `output/review-summary.md`.

### J1 — local choices allowed
- The order candidates and axes are worked through, so long as
  `output/review-summary.md` states which candidates and axes were
  actually applied.

### J0 — must become `needs_input`
- Any Inputs-table artifact opens with `# AMBIGUOUS — NOT RESOLVED` — that
  propagation failing to reach this stage's own inputs cleanly is itself a
  Boundary Honesty finding to record (name the artifact, quote its "What is
  ambiguous" line); do not silently review artifacts resting on an
  unresolved contract as if they were ordinary output.

### Completion boundary
This stage may complete only when `output/findings.ndjson` exists
(possibly empty, only after real effort under all four axes) and
`output/review-summary.md` states which candidates and axes were actually
applied, with finding counts by axis and severity.

### Decision evidence
`output/findings.ndjson` and `output/review-summary.md` are this stage's
decision record; a finding not written down did not happen, as far as
`90-reconcile` is concerned.

## What must become true here (durable outcome)

`output/findings.ndjson` exists (possibly empty, if genuinely nothing was
found — but only after real effort under all four axes) and
`output/review-summary.md` states which candidate packages and which axes
were actually applied, with finding counts by axis and severity. A finding
you did not write down did not happen, as far as `90-reconcile` is
concerned.

## How to do it

0. If `../00-contract/output/contract.md` (or any other Inputs-table
   artifact) opens with `# AMBIGUOUS — NOT RESOLVED`, that propagation
   failing to reach this stage's own inputs cleanly *is itself* a Boundary
   Honesty finding to record (name which artifact, quote its "What is
   ambiguous" line) — do not silently review artifacts you know rest on an
   unresolved contract as if they were ordinary output.

Work through `references/challenge-checklist.md`'s four axes in order,
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
4. **Structural self-consistency** — three checks this stage can run
   entirely blind, no `sergeant-rs-workspace/knowledge/evidence/reference-corpus/` needed: recompute every
   cross-artifact count rather than trusting a stated headline number;
   recompute `quote_hash` directly over each sampled record's *stored*
   `quote` field, not only over a re-located source span; sanity-check the
   `representation` distribution against the ladder's own shape. N2 run 2's
   own review pass missed all three while otherwise being genuinely
   effortful (`sergeant-rs-workspace/knowledge/evidence/gauntlet/runs/n2-run2/comparison-c1-coverage-recall.md`
   §4.1, `comparison-scorecard.md` D-1/D-6,
   `grammar-pressure-report.md` GP-6) — see
   `references/challenge-checklist.md` Axis 4 for the exact method.

Write every finding to `output/findings.ndjson` per this record shape:

```json
{"id": "AF-0001", "axis": "invention", "target": "<candidate>/provenance.md", "description": "...", "evidence": "...", "severity": "high"}
```

`axis` is one of `boundary-honesty`, `invention`, `engine-gap-refutation`,
`structural-self-consistency`.
`severity` is `high` (a violation as described in this workflow's governing
documents), `medium` (a real weakness that does not rise to a documented
violation), or `low` (worth recording, unlikely to change an outcome). This
stage does not assign accept/reject dispositions — that is `90-reconcile`'s
job.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifacts and their disposition.
