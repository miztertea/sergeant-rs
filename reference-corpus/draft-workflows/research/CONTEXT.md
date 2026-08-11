# Research
Draft workflow package — candidate **W27** `research` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Investigate a question against high-trust primary sources and capture the findings as a Markdown file in the repo.

## Trigger

A topic needs to be researched, or docs/API facts need gathering, and reading legwork is delegated.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-investigate` | actor-stage (§6.4, judgment) | Primary sources only, every claim traced; one Markdown file, every claim cited, placed per the repo's convention or an explicitly stated choice. |

## Notes for reviewers

Delegated to a background execution context (BU-P3-041) — that delegation is a *scheduling* property of how research is invoked, not a stage of the procedure itself.

**N1 adjudication A4:** the former `10-write-findings` stage carried only the §6.5 deterministic-machinery boilerplate as its stage-level justification, with no additional checkpoint argument; it is demoted and folded into `00-investigate` as a helper invocation. See `provenance.md`'s "Adjudication A4" section.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
