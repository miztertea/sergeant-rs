# Provenance -- callback-delivery

Maps every stage materialized in this package, and the workflow as a whole, to the `behavior_id`(s) in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`.

## Workflow as a whole

Justified design inference -- no `representation: workflow` record exists for `callback-delivery`. The workflow boundary is inferred from 7 `stage`/`stage-context`/`helper` records sharing the `workflow` value `"callback-delivery"` in the classification corpus, all naming a coherent trigger/outcome/completion pattern (reason: a shared `workflow` value across independently-classified records is itself the evidence of one procedure, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`'s provenance method).

## Stages

None -- no `representation: stage` record carries this candidate's `workflow` value. See `CONTEXT.md` "Zero materialized stages".

## Workflow-local helper evidence (not separately packaged)

`representation: helper` records supporting this workflow's stages (deterministic machinery, per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5 -- not a checkpoint in its own right, so not given a stage directory):

- `BU-0216` -- a supplied correlation ID must be opaque, not platform-identifier-shaped
- `BU-0218` -- a repeat origin registration succeeds harmlessly; a changed one is rejected
- `BU-0219` -- sgt-callback sync is idempotent across reruns
- `BU-0220` -- source identity validated, never stored in plaintext, idempotent re-use
- `BU-0224` -- a consumer executable is bounded in time and output size
- `BU-0225` -- a consumer's return maps to a closed outcome set, defaulting to pending on anything malformed
- `BU-0228` -- a requeue preserves the original idempotency key
