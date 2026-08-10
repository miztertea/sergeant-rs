# 07-record-outcomes: record outcomes

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/standing-constraints.md | L3 | constraints binding every stage of this workflow |
| ../06-pr-and-merge/output/README.md | L4 | upstream artifact produced by `06-pr-and-merge` |

## Purpose

Outcomes are recorded against the owning tracked task.

Trigger (workflow-level): The user explicitly asks to work in this session, and one repository owns the complete outcome.

## What must become true here (durable outcome)

Outcomes are recorded against the owning tracked task.

## Behavior contract

- **In direct mode, record handoff, PR, merge, deployment, and cleanup outcomes.**
  (trigger: delivery complete; outcome: delivery outcomes are durably recorded)
  — `BU-P1-014`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L36, direct-mode handoff step)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
