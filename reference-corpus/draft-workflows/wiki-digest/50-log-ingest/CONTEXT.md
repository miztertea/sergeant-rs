# 50-log-ingest: log ingest

## Inputs

| File | Layer | Why |
|---|---|---|
| ../40-publish-and-index/output/README.md | L4 | upstream artifact produced by `40-publish-and-index` |

## Purpose

The ingest is logged.

Trigger (workflow-level): A digest is due (scheduled) or explicitly requested; or the schema/logic changed and needs a dry run first.

## What must become true here (durable outcome)

The ingest is logged.

## Behavior contract

- **The schema-required ingest log entry is appended or verified after every digest run.**
  (trigger: a real digest run has completed; outcome: the ingest log stays complete and accurate for every run)
  — `BU-P5-142`, `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 55)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
