# 03-recover-from-interrupted-run

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the validation pipeline run outcome is failed or cancelled

**Outcome:** recovery follows exactly one of the three named branch_sync-driven paths

**Statement (the operative rule):** If the validation pipeline outcome is `failed` or `cancelled`, `branch_sync` state is inspected first and handled by exactly one of three named responses: `sync` runs the validation pipeline, `continue_active_run` keeps driving the reported run, `recover_custody` uses the validation pipeline.

## What must become true here (durable outcome)

Recovery follows exactly one of the three named branch_sync-driven paths — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0088`: A reset, stash, force-push, or branch replacement is never improvised around a blocked sync state.
- `BU-1241`: On `failed` or `cancelled`, the agent reads the output, fixes whatever it points at (a failing test, a lint error, a skipped finding), commits the fix on the same feature branch, and drives the pipeline again with a fresh pipeline-automation tool or the validation pipeline; this is correct only after a terminal outcome, never mid-run to circumvent a gate, and the agent must not leave the user at a `failed` outcome without either retrying or explaining what blocks it.
- `BU-1242`: Before any post-pipeline local commit or fresh run, the agent must read the structured `branch_sync` object returned by AXI home, status, or a drive result.
- `BU-1243`: Only when `branch_sync.next_action.code` is `sync` does the agent run the validation pipeline first.
- `BU-1244`: The guarded sync may be a strict fast-forward or a content-equivalent diverged advance that anchors the pre-sync head before moving the branch with reset semantics, but genuine divergence stays blocked.
- `BU-1245`: If `next_action.code` is `continue_active_run`, the pipeline still owns the branch: the agent runs the reported command, keeps driving the active run, and does not make local follow-up commits.
- `BU-1246`: When `next_action.code` is `recover_custody`, a terminal run left unpublished pipeline commits preserved in the local gate: the agent runs the validation pipeline to return custody and fast-forward to the preserved head, or the validation pipeline to resume validating it instead.
- `BU-1247`: A dirty or diverged worktree makes recovery refuse with explicit choices; `--keep-local` keeps the current head while the preserved commits stay anchored under `refs/no-mistakes/recover/<run>`.
- `BU-1248`: If synchronization is blocked, the agent processes that structured state instead of improvising reset, stash, merge, rebase, force, or branch replacement.
- `BU-1249`: After synchronization, the agent commits the follow-up on top and re-runs the validation pipeline with the original user intent, which preserves every prior gate-fix commit regardless of its configured subject.
- `BU-1251`: A PR that falls behind the default branch or hits a merge conflict after checks pass needs no command from the agent and must never be hand-rebased: when the CI monitor sees an actual conflict it rebases onto the base, resolves it, and re-pushes the branch itself; a PR that is merely behind but still clean needs nothing either, since the platform merges it.
- `BU-1252`: The one exception is when the CI monitor is no longer running (the PR was closed, the run was aborted or superseded, it idle-timed-out, or its auto-fix attempts were exhausted): the agent recovers with the validation pipeline, which cancels the stale monitor and re-runs the full pipeline including a deterministic rebase step; the agent must not reach for the validation pipeline to refresh a still-active PR, since after `checks-passed` it reattaches to the running monitor (HEAD unchanged) without rebasing.

