# 00-pin-fixed-point: pin fixed point

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |
| ../scripts/capture-diff.sh | L3 | deterministic diff/commit-list capture helper this stage runs |

## Purpose

The fixed point resolves and the diff is non-empty, or this fails here rather than inside a sub-review.

Trigger (workflow-level): A diff needs review before merge (invoked directly or delegated from `worker-mission`/`implement`).

## What must become true here (durable outcome)

The fixed point resolves and the diff is non-empty, or this fails here rather than inside a sub-review.

## Behavior contract

- **The review's fixed comparison point is whatever the user specified (commit SHA, branch, tag, HEAD~N, etc); if the user did not specify one, the actor must ask for it before proceeding.**
  (trigger: user requests a review without naming a comparison point; outcome: the actor asks the user for the fixed point rather than guessing)
  — `BU-P2-004`, `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 1: Pin the fixed point, lines 19-19)
- **Before spawning the two parallel review sub-agents, the actor must confirm the fixed point resolves (`git rev-parse`) and the diff is non-empty; a bad ref or empty diff must fail at this point, not inside the sub-agents.**
  (trigger: the fixed point and diff command are captured; outcome: invalid input is caught before expensive parallel work starts)
  — `BU-P2-006`, `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 1: Pin the fixed point, lines 23-23)
- **The comparison diff is captured once as the three-dot form `git diff <fixed-point>...HEAD` (against the merge-base), alongside the commit list via `git log <fixed-point>..HEAD --oneline`, via `../scripts/capture-diff.sh`.**
  (trigger: the fixed point has been established; outcome: a stable diff and commit list exist for both later sub-agents to consume)
  — `BU-P2-005`, `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 1: Pin the fixed point, lines 21-21)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Confirm the fixed point resolves (`git rev-parse`) and the diff is non-empty before proceeding (`BU-P2-006`).

### J1 — local choices allowed
- Exact wording used to report a bad ref or empty diff failure.

### J0 — must become `needs_input`
- The user did not specify a fixed comparison point (`BU-P2-004`): stop and ask rather than guessing `HEAD~1`, `main`, or any other default.

### Completion boundary
This stage may complete only when the fixed point resolves and the diff (captured via `../scripts/capture-diff.sh`) is confirmed non-empty.

### Decision evidence
Write the resolved fixed point, the diff command, the commit list, and (if asked) the user's answer to `output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
