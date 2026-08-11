# 05-acquire-launch-reservation

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the project-validation step is about to clone a checkout or publish launch state

**Outcome:** exactly one validation launch proceeds per task/repository pair at a time, with concurrent attempts failing closed

**Statement (the operative rule):** Before cloning the validation checkout or publishing launch state, the coordinator acquires an identity-checked validation-launch reservation for that task/repository pair; concurrent launches fail closed until the recorded owner exits or stale-ownership recovery proves the reservation is abandoned.

## What must become true here (durable outcome)

Exactly one validation launch proceeds per task/repository pair at a time, with concurrent attempts failing closed — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0377`: The validation launch lock can be recovered from a stale prior owner only if that owner's PID is a genuine number, its recorded coordinator and purpose exactly match the current claimant's, and — critically — if that PID is still alive, its process start time must differ from the one recorded at lock time (proving PID reuse, not the same still-running holder) before the lock is treated as abandoned.
- `BU-0387`: Acquiring the validation launch lock uses an atomic hard-link (ln) creation, which can only succeed for one caller; on failure, exactly one stale-lock recovery attempt is made before giving up, so two competing launches for the same task/repo can never both believe they hold the lock.

