# Provenance -- project-registration

Maps every stage materialized in this package, and the workflow as a whole, to the `behavior_id`(s) in `../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson` that justify it, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`.

## Workflow as a whole

Justified design inference -- no `representation: workflow` record exists for `project-registration`. The workflow boundary is inferred from 19 `stage`/`stage-context`/`helper` records sharing the `workflow` value `"project-registration"` in the classification corpus, all naming a coherent trigger/outcome/completion pattern (reason: a shared `workflow` value across independently-classified records is itself the evidence of one procedure, per `../../../workflows/repo-to-icm/60-draft/references/draft-package-template.md`'s provenance method).

## Stages

### 01-confirm-context-loaded

`BU-0258` (`representation: stage`). Trigger: project context loading is claimed complete. Outcome: completeness is defined by an observable evidence artifact, not merely having run the command.

Stage-context folded into this stage's own `CONTEXT.md`:
- `BU-0256` -- a raw-YAML read is only a fallback for a field sgt-context output doesn't surface
- `BU-0266` -- a discrepancy between sgt-context output and the raw YAML blocks progress and preserves evidence rather than silently picking a source

## Unattached stage-context evidence (not materialized)

Named a `workflow`+`stage` pair with no matching `representation: stage` record; not resolved by inventing a stage (`../../../workflows/repo-to-icm/_config/icm-ladder.md` bucket 3):

- `resolve-instructions` -- `BU-0053`
- `confirm-project-name` -- `BU-0255`
- `sync-required-repo` -- `BU-0257`
- `validate-project-edit` -- `BU-0260, BU-0261`

## Workflow-local helper evidence (not separately packaged)

`representation: helper` records supporting this workflow's stages (deterministic machinery, per `../../../workflows/repo-to-icm/_config/icm-ladder.md` §6.5 -- not a checkpoint in its own right, so not given a stage directory):

- `BU-0131` -- project YAML six-field shape check
- `BU-0200` -- layered-instruction conflict resolution by position/specificity
- `BU-0235` -- config.yaml excluded from sgt-list
- `BU-0236` -- empty project directory reported as an actionable error
- `BU-0237` -- sgt-context always distinguishes three clone states
- `BU-0238` -- sgt-context always reports graph existence/build path
- `BU-0239` -- sgt-status reports the exact reason for a missing/non-git repo path
- `BU-0240` -- upstream divergence surfaced explicitly
- `BU-0242` -- clone only under the exact defined precondition
- `BU-0243` -- sgt-td-list filter built deterministically from flags
- `BU-0244` -- a repo missing .git is omitted, not fatal

## Shared-helper/shared-context evidence, external to this package

- **5b dev-root-relative-path-resolution** (`BU-0051, BU-0193`) -- resolves repo paths from project YAML.
- **5c project-name-identity** (`BU-0052, BU-0195`) -- derives addressable project name.
