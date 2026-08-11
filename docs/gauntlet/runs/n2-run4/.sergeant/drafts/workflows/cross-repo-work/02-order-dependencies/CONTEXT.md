# 02-order-dependencies

## Inputs

| File | Layer | Why |
|---|---|---|
| ../01-assign-ownership/output/outcome.md | L4 | upstream evidence produced by `assign-ownership` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a dependency graph among repositories contains a cycle

**Outcome:** dispatch does not proceed with a cyclic dependency graph; the cycle is broken by design instead

**Statement (the operative rule):** Dependency cycles are rejected before dispatch; if a cycle reflects a genuinely coupled contract, a contract artifact or compatibility phase is defined to break the cycle instead of dispatching a cyclic dependency graph.

## What must become true here (durable outcome)

Dispatch does not proceed with a cyclic dependency graph; the cycle is broken by design instead — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0270`: Repository state is never stashed, reset, switched, or cleaned during cross-repo planning; an existing canonical branch/worktree is routed to the worker brief instead, or the procedure stops for a decision when state conflicts with the requested outcome.

