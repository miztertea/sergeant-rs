# Provenance -- worker-response-and-recovery

Maps every stage materialized in this package, and the workflow as a whole, to the `behavior_id`(s) in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`.

## Workflow as a whole

Justified design inference -- no `representation: workflow` record exists for `worker-response-and-recovery`. The workflow boundary is inferred from 19 `stage`/`stage-context`/`helper` records sharing the `workflow` value `"worker-response-and-recovery"` in the classification corpus, all naming a coherent trigger/outcome/completion pattern (reason: a shared `workflow` value across independently-classified records is itself the evidence of one procedure, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`'s provenance method).

## Stages

### 01-evaluate-wake-condition

`BU-0151` (`representation: stage`). Trigger: a wake condition becomes permanently unsatisfiable (four named cases). Outcome: the worker escalates to needs_input with a stated remedy rather than retrying indefinitely.

### 02-respond-to-worker

`BU-0155, BU-0275` (`representation: stage`). Trigger (BU-0155): sgt-respond is about to be used. Trigger (BU-0275): a worker escalates with needs_input/blocked. Outcome: the five-step precondition/delivery sequence runs, and the human decision is genuinely obtained (not inferred) before a response is sent.

Stage-context folded into this stage's own `CONTEXT.md`:
- `BU-0157` -- a delivered response is applied exactly once, matching ID/generation/status
- `BU-0177` -- a pending response is never clobbered; the correct convergence path is used instead of recover

## Unattached stage-context evidence (not materialized)

Named a `workflow`+`stage` pair with no matching `representation: stage` record; not resolved by inventing a stage (`../../../workflows/repo-to-icm/_config/icm-ladder.md` bucket 3):

- `select-resume-path` -- `BU-0037`
- `recover-worker` -- `BU-0039, BU-0146, BU-0159, BU-0174, BU-0286`
- `drain-wait` -- `BU-0154`
- `acknowledge-response` -- `BU-0158`
- `diagnose-repeated-notification` -- `BU-0179`

## Workflow-local helper evidence (not separately packaged)

`representation: helper` records supporting this workflow's stages (deterministic machinery, per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5 -- not a checkpoint in its own right, so not given a stage directory):

- `BU-0148` -- durable wake-condition representation, no live sleep loop
- `BU-0149` -- sgt-wake resumes only the exact matching, generation-tagged worker
- `BU-0150` -- github_check wake gated strictly on a success conclusion
- `BU-0156` -- token-matched acceptance round-trip before a nudge is acted on
- `BU-0178` -- pane-not-found classification depends on durable handoff state
