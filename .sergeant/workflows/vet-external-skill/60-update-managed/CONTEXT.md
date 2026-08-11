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

## Judgment required

Reclassified from `stage (§6.3, deterministic-machinery candidate)` to actor-stage at N1 adjudication A4: the checkpoint here is not the installer rerun (external, deterministic machinery outside Sergeant's control) but the decision that follows it — inspecting the diff and updated lock file and deciding whether to accept the update. That decision survives any reimplementation of how the installer itself is invoked (§6.3's test), so it is genuine judgment: the acting harness must inspect evidence and decide whether to accept the update, or explain why not — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Additional note

Alternate entry: only reached when updating an already-adopted, skills.sh-managed skill, not during the initial `00`-`50` vetting sequence. Mutually exclusive with `60-update-owned`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
