# 03-sync-and-produce-events

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the callback-delivery step sync runs repeatedly against the same underlying state

**Outcome:** re-running sync is idempotent — it never fabricates a new event generation for state it has already classified

**Statement (the operative rule):** Waiting-event identity includes the repository, class, and `.sergeant-gate-generation`; terminal identity includes the repository and terminal class; repeated synchronization of the same source creates no new generation.

## What must become true here (durable outcome)

Re-running sync is idempotent — it never fabricates a new event generation for state it has already classified — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0229`: Automatic callback producers make one bounded delivery attempt and return without waiting indefinitely; events survive callback/process restarts.
- `BU-0604`: The interactive fleet-watch loop triggers the callback-delivery step sync for a task only when that task's fleet state carries a .callbacks/origin.json marker (as a file or symlink); tasks with no such marker never invoke the callback path during reconciliation.
- `BU-0681`: When a task has a registered durable callback origin, the notify step triggers a sync of that task's durable callback events as part of handling the update.
- `BU-0682`: A failed durable-callback sync during a notify call does not abort the notify call itself, but is recorded so the command's own exit status still reflects the failure.
- `BU-0783`: When determining a repo's authoritative status for callback purposes, a recorded worktree pointer's own status file is consulted in preference to the fleet-level status file, and the worktree pointer itself must be an absolute path to a real, user-owned directory or the lookup fails.
- `BU-0785`: A repo in needs_input or blocked status produces a callback event of the matching type whose deduplication source id is derived from a hash of the repo name, the status, and the gate generation together — so repeated syncs of the same unresolved gate do not enqueue duplicate events, but the gate generation advancing produces a fresh one.
- `BU-0786`: A repo whose authoritative status is failed:<reason> produces a 'failed' callback event whose payload is the reason text following the prefix, with a source id scoped as a one-time terminal marker for that repo.
- `BU-0787`: A repo whose authoritative status is 'done' produces a 'done' callback event carrying the task's result content, with a source id scoped as a one-time terminal marker for that repo.
- `BU-0788`: A failure syncing one repo's callback status within a multi-repo task does not prevent the other repos in that task from being synced — per-repo failures are collected across all repos and reported together once every repo has been attempted.
- `BU-0789`: Syncing a task's callback events always attempts a bounded (one-event) drain at the end of the call, whether or not new events were enqueued during that same sync.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0784`: Syncing a task's callback events is a no-op if the task has no registered callback origin.

