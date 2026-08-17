# 30-verify-no-conflict: verify no conflict

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-check-actions/output/README.md | L4 | upstream artifact produced by `20-check-actions` |

## Purpose

The external skill does not conflict with repository AGENTS.md or safety policy.

Trigger (workflow-level): Before adopting an external skill, or when an adopted skill needs updating.

## What must become true here (durable outcome)

The external skill does not conflict with repository AGENTS.md or safety policy.

## Behavior contract

- **Verify the external skill does not conflict with repository AGENTS.md or safety policy.**
  (trigger: actions checked; outcome: no adopted skill contradicts the repository's own instruction or safety policy)

## Bounded judgment

Apply `@@bounded-judgment`.

### J5 — governing constraint
- No adopted skill may contradict repository instruction or safety policy (workflow-level constraint, restated here as a check at this stage).

### J2 — delegated to this stage
- Judging whether a given instruction or action in the skill is in fact a conflict.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- A conflict with repository `AGENTS.md` or safety policy is found: record the conflict and ask the user rather than proceeding to `50-test-in-disposable-copy` past a discovered conflict.

### Completion boundary
This stage may complete only once the skill is verified not to conflict with repository `AGENTS.md` or safety policy — or the stage has stopped at the J0 case above.

### Decision evidence
The conflict verdict is this stage's own durable output, recorded per `output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
