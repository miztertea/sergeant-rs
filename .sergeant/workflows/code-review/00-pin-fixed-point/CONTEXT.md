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

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
