# 10-dry-run: dry run

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-read-schema/output/README.md | L4 | upstream artifact produced by `00-read-schema` |

## Purpose

A dry run always runs first when regenerating or changing logic.

Trigger (workflow-level): A digest is due (scheduled) or explicitly requested; or the schema/logic changed and needs a dry run first.

## What must become true here (durable outcome)

A dry run always runs first when regenerating or changing logic.

## Behavior contract

- **--dry-run is run first whenever regenerating an existing day or changing digest logic, before any non-dry run.**
  (trigger: an existing day is being regenerated or digest logic changed; outcome: a preview always precedes a real write in these higher-risk cases)
  — `BU-P5-138`, `reference/sergeant-upstream/skills/wiki/SKILL.md` (lines 49-50)
- **A day's digest, once written, is never silently regenerated on a later run; the operator must explicitly delete the existing page to force resynthesis, unless running in dry-run mode.**
  (trigger: a digest is requested for a date that already has a written page; outcome: a digest is idempotent by default: re-running the job never clobbers an existing day's synthesized page)
  — `BU-P6-093`, `reference/sergeant-upstream/bin/wiki-daily-digest` (L411-414)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
