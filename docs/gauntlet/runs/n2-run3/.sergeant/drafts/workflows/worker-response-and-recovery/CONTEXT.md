# Worker Response And Recovery

Layer 1 orientation only -- what this candidate workflow is for and how its stages relate. No stage instructions here (those live in each stage's own `CONTEXT.md`, Layer 2).

## What this is for

**Trigger.** A worker signals a nonterminal state (waiting/needs_input/blocked), or a wake condition becomes permanently unsatisfiable.

**Outcome.** The worker is either resumed through a verified wake/response round-trip, or escalated to a human decision -- never guessed at or force-recovered.

**Completion.** The response/resume action is durably recorded and the worker's own consumption of it completes the round-trip.

## How its stages relate

Ordered, trigger-to-outcome:

1. **evaluate-wake-condition** (`01-evaluate-wake-condition/`) -- Trigger: a wake condition becomes permanently unsatisfiable (four named cases). Outcome: the worker escalates to needs_input with a stated remedy rather than retrying indefinitely.
2. **respond-to-worker** (`02-respond-to-worker/`) -- Trigger (BU-0155): sgt-respond is about to be used. Trigger (BU-0275): a worker escalates with needs_input/blocked. Outcome: the five-step precondition/delivery sequence runs, and the human decision is genuinely obtained (not inferred) before a response is sent.

## Unattached stage-context evidence, not materialized

9 `stage-context` behavior_id(s), across 5 named checkpoint(s), name a `workflow`+`stage` pair in the classification corpus with no matching `representation: stage` record. Per bucket 3 these are not resolved by inventing a stage directory to hang them on; see `provenance.md` for the list and `../../../workflows/repo-to-icm/60-draft/output/draft-report.md` for the run-level carry-through.

## Workflow-local helper machinery (not separately packaged)

5 `helper` records support this workflow's stages (deterministic machinery, not checkpoints in their own right per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5). No `scripts/` directory is created here: this run's Inputs give behavior_id and a one-line functional description, not an actual script name to point at, and inventing one would be unsupported invention. See `provenance.md` for the full list.

## Note

**`recover-worker` never became a stage -- the sharpest unattached-record finding in this run.** Five stage-context records (BU-0039, BU-0146, BU-0159, BU-0174, BU-0286) all name `stage: recover-worker` -- a checkpoint clearly operator-visible in a workflow whose own name is "worker-response-**and-recovery**" -- yet no `representation: stage` record for it exists anywhere in the 312-record corpus. Per `../../../workflows/repo-to-icm/_config/icm-ladder.md` bucket 3 and `../../../workflows/repo-to-icm/50-synthesize/references/synthesis-method.md`, this is not resolved here by inventing a `recover-worker` stage directory; it is recorded in `provenance.md` and carried to `../../../workflows/repo-to-icm/60-draft/output/draft-report.md` for `90-reconcile`, exactly as `50-synthesize` flagged it (`../../../workflows/repo-to-icm/50-synthesize/output/candidates.md` #3).
