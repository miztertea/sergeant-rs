# Workflow-level helpers — notify-primary-session

Layer 3 (`_config/`), stable across every future run of this candidate, used by more than one of its stages (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`). Deterministic machinery `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` attached to `workflow=notify-primary-session`, `stage=null` — referenced by the workflow as a whole, not one specific stage.

- `BU-0680`: A worker update message is classified into one of three durable event kinds by a fixed prefix match on the message text: done/failed become completion, needs_input/blocked become escalation, anything else becomes update.
