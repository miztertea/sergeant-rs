# 00-pin-fixed-point: pin fixed point

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

The fixed point resolves and the diff is non-empty, or this fails here rather than inside a sub-review.

Trigger (workflow-level): A diff needs review before merge (invoked directly or delegated from `worker-mission`/`implement`).

## What must become true here (durable outcome)

The fixed point resolves and the diff is non-empty, or this fails here rather than inside a sub-review.

## Behavior contract

- **The review's fixed comparison point is whatever the user specified (commit SHA, branch, tag, HEAD~N, etc); if the user did not specify one, the actor must ask for it before proceeding.**
  (trigger: user requests a review without naming a comparison point; outcome: the actor asks the user for the fixed point rather than guessing)
- **Before spawning the two parallel review sub-agents, the actor must confirm the fixed point resolves (`git rev-parse`) and the diff is non-empty; a bad ref or empty diff must fail at this point, not inside the sub-agents.**
  (trigger: the fixed point and diff command are captured; outcome: invalid input is caught before expensive parallel work starts)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- None identified: resolving the given ref (`git rev-parse`) and checking the diff is non-empty are deterministic checks, not judgment calls.

### J1 — local choices allowed
- Exact wording of the failure message when the ref doesn't resolve or the diff is empty.

### J0 — must become `needs_input`
- The user did not specify a fixed comparison point: ask for one before proceeding rather than guessing (e.g. defaulting to `HEAD~1` or `main`).
- The given fixed point does not resolve, or the diff against it is empty: stop and report rather than passing a broken input into the parallel sub-reviews.

### Completion boundary
This stage may complete only once the fixed point resolves and the diff is confirmed non-empty.

### Decision evidence
Record the resolved fixed point and diff command in `output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
