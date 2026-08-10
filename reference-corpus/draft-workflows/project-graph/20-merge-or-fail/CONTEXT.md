# 20-merge-or-fail: merge or fail

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-extract-per-repo/output/README.md | L4 | upstream artifact produced by `10-extract-per-repo` |

## Purpose

All-or-nothing: any repo's extraction failure fails the run before merge.

Trigger (workflow-level): Architecture work needs whole-project structure, or the operator asks for a graph/refresh.

## What must become true here (durable outcome)

All-or-nothing: any repo's extraction failure fails the run before merge.

## Behavior contract

- **Building a project graph across many repos is all-or-nothing at the merge step: if extraction fails for any included repo, the whole run fails before attempting to merge or publish anything, rather than publishing a graph silently missing some repos.**
  (trigger: one or more repos fail extraction during a multi-repo graph build; outcome: a published project graph is always complete over every repo it claims to cover, never silently partial)
  — `BU-P6-091`, `reference/sergeant-upstream/bin/sgt-graphify` (L430-433)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Additional note

"We never publish a partial graph" outlives any particular merger implementation.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
