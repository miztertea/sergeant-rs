# 60-update-managed: update managed

## Inputs

| File | Layer | Why |
|---|---|---|
| — | — | no contract-bearing upstream dependency beyond this workflow's ordering |

## Purpose

For skills.sh-managed skills: rerun the official installer and inspect the diff and updated lock file before accepting changes.

Trigger (workflow-level): Before adopting an external skill, or when an adopted skill needs updating.

## What must become true here (durable outcome)

For skills.sh-managed skills: rerun the official installer and inspect the diff and updated lock file before accepting changes.

## Behavior contract

- **For skills.sh-managed skills, rerun the official installer and inspect the diff and updated lock file before accepting changes.**
  (trigger: updating a skills.sh-managed skill; outcome: no update is accepted without inspecting its diff and lock-file change first)
  — `BU-P1-126`, `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L138-139, skills.sh update path)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Additional note

Alternate entry: only reached when updating an already-adopted, skills.sh-managed skill, not during the initial `00`-`50` vetting sequence. Mutually exclusive with `60-update-owned`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
