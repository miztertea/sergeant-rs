# 30-verify: verify

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-recommend/output/README.md | L4 | upstream artifact produced by `20-recommend` |

## Purpose

The claim is reproduced or the PR diff is tested, reported as confirmed/failed/insufficient.

Trigger (workflow-level): An item is at the front of one of the three fixed attention buckets, oldest first.

## What must become true here (durable outcome)

The claim is reproduced or the PR diff is tested, reported as confirmed/failed/insufficient.

## Behavior contract

- **Before grilling, the actor verifies the claim empirically — reproducing a bug or checking out and testing a PR's diff — and reports one of confirmed, failed, or insufficient-detail, where confirmation strengthens the eventual agent brief.**
  (trigger: a recommendation has been given and direction received; outcome: the claim's validity is empirically established before further action)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Reproducing a bug or checking out and testing a PR's diff, and reporting confirmed/failed/insufficient-detail.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only when the claim has been empirically checked and reported as confirmed, failed, or insufficient-detail.

### Decision evidence
The confirmed/failed/insufficient-detail verdict is this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
