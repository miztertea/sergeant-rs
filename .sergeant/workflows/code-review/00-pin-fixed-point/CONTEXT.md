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
- None of substance: confirming the fixed point resolves (`git rev-parse`) and the diff against it is non-empty are both deterministic checks, not judgment calls.

### J1 — local choices allowed
- Exact wording of the request when asking the user for a fixed point.

### J0 — must become `needs_input`
- The user did not specify a comparison point (commit SHA, branch, tag, `HEAD~N`, etc.): ask for it before proceeding, rather than guessing one.
- The fixed point does not resolve (`git rev-parse` fails) or the diff against it is empty: stop here and report the failure, rather than letting a bad ref or empty diff surface inside a sub-review.

### Completion boundary
This stage may complete only once the fixed point has been confirmed to resolve and the diff against it confirmed non-empty.

### Decision evidence
Which fixed point was used (user-specified, or the resolved default) is recorded in this stage's own `output/README.md`-declared artifact.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
