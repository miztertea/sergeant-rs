# 60-update-owned: update owned

## Inputs

| File | Layer | Why |
|---|---|---|
| — | — | no contract-bearing upstream dependency beyond this workflow's ordering |

## Purpose

For Sergeant-owned skills: update this repository through a reviewed PR and run the instruction-policy test plus the full test suite.

Trigger (workflow-level): Before adopting an external skill, or when an adopted skill needs updating.

## What must become true here (durable outcome)

For Sergeant-owned skills: update this repository through a reviewed PR and run the instruction-policy test plus the full test suite.

## Behavior contract

- **For Sergeant-owned skills, update this repository through a reviewed PR and run tests/instruction-policy-test.sh plus the full Sergeant test suite.**
  (trigger: updating a Sergeant-owned skill; outcome: no Sergeant-owned skill changes ship without passing review and the full test suite, including the instruction-policy test)
  — `BU-P1-127`, `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L142-144, Sergeant-owned update path)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Additional note

Alternate entry: only reached when updating an already-adopted, Sergeant-owned skill, not during the initial `00`-`50` vetting sequence. Mutually exclusive with `60-update-managed`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
