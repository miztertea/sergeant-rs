# 10-confirm-provenance: confirm provenance

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-read-source/output/README.md | L4 | upstream artifact produced by `00-read-source` |

## Purpose

The external skill's source and update mechanism are confirmed.

Trigger (workflow-level): Before adopting an external skill, or when an adopted skill needs updating.

## What must become true here (durable outcome)

The external skill's source and update mechanism are confirmed.

## Behavior contract

- **Confirm the external skill's source and update mechanism.**
  (trigger: step 1 complete; outcome: provenance and update path are known before proceeding)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Judging whether a claimed source and update mechanism are actually confirmable from available evidence.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- Provenance cannot be confirmed (an unsigned tarball, an anonymous fork, a source that does not state how it is updated): state what evidence was checked and ask the user rather than proceeding to `20-check-actions` on an unconfirmed source.

### Completion boundary
This stage may complete only once the skill's source and update mechanism are confirmed — or the stage has stopped at the J0 case above.

### Decision evidence
The confirmed provenance is this stage's own durable output, recorded per `output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
