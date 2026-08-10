# 40-commit: commit

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-review/output/README.md | L4 | upstream artifact produced by `30-review` |

## Purpose

The verified change is committed.

Trigger (workflow-level): Explicitly invoked to implement a defined piece of work (never auto-loaded).

## What must become true here (durable outcome)

The verified change is committed.

## Behavior contract

- **The final step of implement is to commit the work to the current branch.**
  (trigger: the work has been implemented, verified, and reviewed; outcome: the change is committed to the current branch)
  — `BU-P2-055`, `reference/sergeant-upstream/.agents/skills/implement/SKILL.md` (body, lines 15-15)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
