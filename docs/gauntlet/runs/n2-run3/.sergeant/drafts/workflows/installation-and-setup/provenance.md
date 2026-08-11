# Provenance -- installation-and-setup

Maps every stage materialized in this package, and the workflow as a whole, to the `behavior_id`(s) in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`.

## Workflow as a whole

Justified design inference -- no `representation: workflow` record exists for `installation-and-setup`. The workflow boundary is inferred from 9 `stage`/`stage-context`/`helper` records sharing the `workflow` value `"installation-and-setup"` in the classification corpus, all naming a coherent trigger/outcome/completion pattern (reason: a shared `workflow` value across independently-classified records is itself the evidence of one procedure, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`'s provenance method).

## Stages

### 01-dependency-check

`BU-0129` (`representation: stage`). Trigger: the dependency check runs during installation. Outcome: installation does not proceed until both the td-implementation check and the agent-availability check pass.

## Workflow-local helper evidence (not separately packaged)

`representation: helper` records supporting this workflow's stages (deterministic machinery, per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5 -- not a checkpoint in its own right, so not given a stage directory):

- `BU-0204` -- install links every current and future matching script generically
- `BU-0205` -- a stale legacy symlink is removed automatically, scoped to symlinks only
- `BU-0206` -- hook install skips cleanly when a directory is absent
- `BU-0207` -- uninstall removes only hooks this repo actually installed
- `BU-0208` -- uninstall removes only symlinks provably pointing into this repo's bin/
- `BU-0210` -- an agent harness check passes given any one of three, in priority order
- `BU-0211` -- a failed dependency check fails closed with an actionable message
- `BU-0212` -- Bash-compatibility proof required under both ambient and minimum-supported Bash

## Shared-helper/shared-context evidence, external to this package

- **5d td-capability-surface-check** (`BU-0132, BU-0209`) -- dependency-check stage's td verification.
