# Undocumented Failure Escalation

Layer 1 orientation only -- what this candidate workflow is for and how its stages relate. No stage instructions here (those live in each stage's own `CONTEXT.md`, Layer 2).

## What this is for

**Trigger.** A failure is not covered by existing documentation.

**Outcome.** Sergeant-help is used to search the docs, then the gap is escalated as a well-formed td task containing the exact reproduction, expected behavior, preserved state, and acceptance criteria -- rather than left unresolved or guessed at.

**Completion.** The td task exists with all four required fields.

## Zero materialized stages

No `representation: stage` record in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` carries this candidate's `workflow` value. Per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` and `../../../workflows/repo-to-icm/_config/icm-ladder.md` bucket 3, this is not resolved by inventing a stage: this package has no `NN-*/` directories and `workflow.toml` declares `stages = []`. See `provenance.md` for the evidence this candidate boundary rests on instead.

## Note

The corpus's most single-behavior workflow candidate of all eighteen -- one record, no supporting stage/context/helper material anywhere else in the classified ledger. Unlike `skill-adoption` and `sergeant-help-query`, no other record in the corpus carries a matching `workflow` value -- this candidate's name is minted fresh from BU-0192's own topic, kebab-cased (`../../../workflows/repo-to-icm/50-synthesize/output/candidates.md` #18).
