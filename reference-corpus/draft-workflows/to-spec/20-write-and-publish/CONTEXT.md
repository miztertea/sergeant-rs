# 20-write-and-publish: write and publish

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-sketch-seams/output/README.md | L4 | upstream artifact produced by `10-sketch-seams` |

## Purpose

A fixed template is published to the tracker with the ready label.

Trigger (workflow-level): A design needs to be turned into a spec-shaped ticket before implementation.

## What must become true here (durable outcome)

A fixed template is published to the tracker with the ready label.

## Behavior contract

- **Write the spec using the fixed spec template, publish it to the project issue tracker, and apply the ready-for-agent triage label without requiring additional triage.**
  (trigger: the spec content is finalized; outcome: a published, triage-labeled spec issue exists)
  — `BU-P4-054`, `reference/sergeant-upstream/.agents/skills/to-spec/SKILL.md` (Process step 3, L19)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
