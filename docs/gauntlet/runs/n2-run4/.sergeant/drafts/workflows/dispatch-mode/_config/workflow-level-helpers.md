# Workflow-level helpers — dispatch-mode

Layer 3 (`_config/`), stable across every future run of this candidate, used by more than one of its stages (`.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md`). Deterministic machinery `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` attached to `workflow=dispatch-mode`, `stage=null` — referenced by the workflow as a whole, not one specific stage.

- `BU-0278`: When `treehouse.toml` exists in a repo, dispatch leases a pre-warmed worktree with the treehouse session manager and checks out the branch in it; when the treehouse session manager is not initialized in a repo, dispatch falls back to plain `git worktree add` at a sibling path.
