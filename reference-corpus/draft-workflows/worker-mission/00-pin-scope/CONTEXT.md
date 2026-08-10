# 00-pin-scope: pin scope

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Refs fetched, a fixed base commit pinned, base SHA/commit list/diff scope recorded before implementation.

Trigger (workflow-level): A worker starts against a rendered brief.

## What must become true here (durable outcome)

Refs fetched, a fixed base commit pinned, base SHA/commit list/diff scope recorded before implementation.

## Behavior contract

- **A dispatched worker's mission brief pins a pre-implementation source of truth: fetch refs, pin a fixed base commit (normally the merge-base with origin/main), and record base SHA, commit list, and diff scope before implementation begins.**
  (trigger: a worker begins substantive implementation work; outcome: every later validation and review step operates against a recorded, reproducible diff scope rather than a moving target)
  — `BU-P7-005`, `reference/sergeant-upstream/templates/worker-brief.md` (section '### 1. Pin scope and source of truth')

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
