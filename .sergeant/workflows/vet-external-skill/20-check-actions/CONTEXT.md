# 20-check-actions: check actions

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-confirm-provenance/output/README.md | L4 | upstream artifact produced by `10-confirm-provenance` |

## Purpose

The external skill's filesystem, shell, network, Git, and credential actions are checked.

Trigger (workflow-level): Before adopting an external skill, or when an adopted skill needs updating.

## What must become true here (durable outcome)

The external skill's filesystem, shell, network, Git, and credential actions are checked.

## Behavior contract

- **Check the external skill's filesystem, shell, network, Git, and credential actions.**
  (trigger: source confirmed; outcome: the skill's side-effect surface across five named categories is known)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Assessing the actual side-effect surface across the five named categories from source inspection.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- A checked action is severe enough (e.g. credential exfiltration, arbitrary network egress, a destructive filesystem/Git action) that continuing would be irresponsible without a stop: record the finding and ask the user rather than silently proceeding to `30-verify-no-conflict`.

### Completion boundary
This stage may complete only once the skill's filesystem, shell, network, Git, and credential actions are checked across all five categories — or the stage has stopped at the J0 case above.

### Decision evidence
The checked action surface is this stage's own durable output, recorded per `output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
