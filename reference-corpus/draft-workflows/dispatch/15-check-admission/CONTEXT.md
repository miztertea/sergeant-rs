# 15-check-admission: check admission

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-preflight-capabilities/output/README.md | L4 | upstream artifact produced by `10-preflight-capabilities` |

## Purpose

The fleet-wide admission lock is held only across the first side effect, then released.

Trigger (workflow-level): Work spans repositories, contains two or more independent repository-owned tasks, needs an isolated review worker, or the user asks for workers.

## What must become true here (durable outcome)

The fleet-wide admission lock is held only across the first side effect, then released.

## Behavior contract

- **A tracked-work task per targeted repo is created before dispatch commits to any worker launch when no explicit task reference was supplied, but the admission (drain) lock is held only through that first side effect and released immediately afterward, so dispatch does not hold a fleet-wide lock across the much longer per-repo worktree/launch sequence.**
  (trigger: a dispatch is committing its first durable side effect; outcome: a concurrent drain can never race a dispatch's admission decision, while dispatch still does not hold a shared lock for its entire (much longer) execution)
  — `BU-P6-128`, `reference/sergeant-upstream/bin/sgt-dispatch` (L473-486, L524-529)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Additional note

Blocks on `drain-fleet`'s admission-block state.

## Delegation

This stage's outcome is produced by running **drain-fleet** to its own completion (context composition today — see `docs/icm/convention.md` §4 on `@@name` versus true nested-workflow invocation, which does not exist yet).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
