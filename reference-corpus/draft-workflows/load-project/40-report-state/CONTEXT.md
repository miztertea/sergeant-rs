# 40-report-state: report state

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-sync-repositories/output/README.md | L4 | upstream artifact produced by `30-sync-repositories` |

## Purpose

A read-only per-repo report of clone/branch/cleanliness/ahead-behind and open tracked work.

Trigger (workflow-level): A project is named, registered, edited, synced, or listed; or repository ownership is not already established.

## What must become true here (durable outcome)

A read-only per-repo report of clone/branch/cleanliness/ahead-behind and open tracked work.

## Behavior contract

- **Showing a project's status walks every configured repo and reports, per repo, whether it is cloned, its current branch, its working-tree cleanliness, and how far ahead/behind its upstream it is — never mutating anything.**
  (trigger: operator wants a read-only snapshot of every repo in a project; outcome: a per-repo clone/branch/dirty/ahead-behind summary with no side effects)
  — `BU-P6-012`, `reference/sergeant-upstream/bin/sgt-status` (L1-2)
- **Listing tracked work across a project defaults to showing only open tasks and can be narrowed by status, priority, or an explicit repo subset; every repo is queried independently and repos without an initialized task database are silently skipped rather than erroring the whole listing.**
  (trigger: operator wants a unified view of tracked work across a project's repos; outcome: a filtered, unified view of open (or otherwise selected) tracked work across every configured repo)
  — `BU-P6-035`, `reference/sergeant-upstream/bin/sgt-td-list` (L2-13)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Additional note

Borderline per synthesis.md (closer to a query than a checkpoint); kept as a stage because operators do care whether it succeeded before planning.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
