# 20-create-tickets: create tickets

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-map-frontier/output/README.md | L4 | upstream artifact produced by `10-map-frontier` |

## Purpose

Specifiable decisions become child issues first; blocking edges are wired in a second pass.

Trigger (workflow-level): A destination is named that requires mapping fog before it can be reached.

## What must become true here (durable outcome)

Specifiable decisions become child issues first; blocking edges are wired in a second pass.

## Behavior contract

- **When creating a wayfinder map, create the tickets that can already be specified as child issues first, then wire their blocking edges in a second pass, because issues need ids before they can reference each other.**
  (trigger: specifiable decisions have been identified during charting; outcome: the resulting ticket set has correct blocking edges despite the two-pass creation order)
  — `BU-P4-096`, `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Invocation / Chart the map, L114)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
