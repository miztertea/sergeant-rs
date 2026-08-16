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

## Bounded judgment

Reclassified from `stage (§6.3, deterministic-machinery candidate)` to actor-stage at N1 adjudication A4: the checkpoint here is not the installer rerun (external, deterministic machinery outside Sergeant's control) but the decision that follows it — inspecting the diff and updated lock file and deciding whether to accept the update. That decision survives any reimplementation of how the installer itself is invoked (§6.3's test), so it is genuine judgment (PL-5).

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Accept or reject the update after inspecting its diff and lock-file change.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- Inspection finds something that should not be silently accepted (a scope-expanding permission, an unexpected new network call, a maintainer change): record the finding and ask the user rather than accepting or rejecting unilaterally.

### Completion boundary
This stage may complete only once the diff and updated lock file are inspected and the update is accepted or rejected — or the stage has stopped at the J0 case above.

### Decision evidence
The inspection findings and accept/reject decision are this stage's own durable output, recorded per `output/README.md`.

## Additional note

Alternate entry: only reached when updating an already-adopted, skills.sh-managed skill, not during the initial `00`-`50` vetting sequence. Mutually exclusive with `60-update-owned`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
