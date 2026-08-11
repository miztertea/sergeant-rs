# Callback Delivery

Layer 1 orientation only -- what this candidate workflow is for and how its stages relate. No stage instructions here (those live in each stage's own `CONTEXT.md`, Layer 2).

## What this is for

**Trigger.** A callback event is registered, enqueued, or delivered to an external consumer.

**Outcome.** Origin identity, idempotency, bounded consumer execution, a closed outcome set, and requeue behavior are all deterministic.

**Completion.** No stage checkpoint was classified in this corpus for this workflow -- see provenance.md.

## Zero materialized stages

No `representation: stage` record in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` carries this candidate's `workflow` value. Per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md` and `../../../workflows/repo-to-icm/_config/icm-ladder.md` bucket 3, this is not resolved by inventing a stage: this package has no `NN-*/` directories and `workflow.toml` declares `stages = []`. See `provenance.md` for the evidence this candidate boundary rests on instead.

## Workflow-local helper machinery (not separately packaged)

7 `helper` records support this workflow's stages (deterministic machinery, not checkpoints in their own right per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5). No `scripts/` directory is created here: this run's Inputs give behavior_id and a one-line functional description, not an actual script name to point at, and inventing one would be unsupported invention. See `provenance.md` for the full list.

## Note

**Cross-reference to Bucket 7.** The actual durable-delivery-guarantee gap this workflow's behavior implies surfaces separately as `engine-gap` candidate BU-0227 in `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md` Bucket 7 -- not materialized here (`60-draft` does not package engine-gap findings; they are carried through in `../../../workflows/repo-to-icm/60-draft/output/draft-report.md`). BU-0227 explicitly attempted and rejected `stage` for the same reason this workflow has none: no independently operator-visible checkpoint boundary, just crash-safe claim/lease machinery spanning many attempts.
