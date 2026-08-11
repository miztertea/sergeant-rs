# 08-rollback-on-launch-failure

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a validation launch fails before commit

**Outcome:** rollback is scoped strictly to provably-owned artifacts of this invocation, never touching state it cannot prove it created

**Statement (the operative rule):** If launch fails before the validation child commits the release, Sergeant rolls back only the checkout, pane, temp files, and fleet-state markers that the current invocation both created and can still prove it owns, preserving preexisting state, reused panes, dangling paths, and concurrent replacements; after the recorded pane and process group have fully exited, rerunning the project-validation step safely resets only identity-matched finished state and retries.

## What must become true here (durable outcome)

Rollback is scoped strictly to provably-owned artifacts of this invocation, never touching state it cannot prove it created — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0385`: Rollback of a failed or aborted validation launch removes an owned path (or restores a backed-up prior window/stage state) only if that path's captured identity (device+inode+birthtime, plus content checksum for files) still matches what was recorded when this launch owned it, so a path that was replaced by something else in the meantime is left untouched rather than deleted or overwritten.
- `BU-0386`: Every durable state write during validation launch (_validation_write_owned) follows the same pattern: create a private temp candidate file, verify its identity is unchanged immediately after writing content, hard-link it into its final path, and record ownership of each intermediate path at each step, so the launch can distinguish and reliably roll back exactly what it created.

