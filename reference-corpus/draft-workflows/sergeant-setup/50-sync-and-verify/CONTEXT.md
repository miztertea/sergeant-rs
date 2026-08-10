# 50-sync-and-verify: sync and verify

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/standing-constraints.md | L3 | constraints binding every stage of this workflow |
| ../40-repair-existing/output/README.md | L4 | upstream artifact produced by `40-repair-existing` |

## Purpose

The four verification commands run in fixed order, stopping at the first failure.

Trigger (workflow-level): First install, a new project/repository to register, a broken or incomplete installation, or a verification request.

## What must become true here (durable outcome)

The four verification commands run in fixed order, stopping at the first failure.

## Behavior contract

- **After the project YAML is written, sergeant-setup runs sgt-list, sgt-context <project>, sgt-status <project>, and sgt-sync <project> in that fixed order, stopping at the first failure with its full output and never advancing to the next command until the previous one succeeds.**
  (trigger: the project YAML has been written or repaired; outcome: the new or repaired configuration is proven to actually work end-to-end before later phases run)
  — `BU-P5-031`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 220-231)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
