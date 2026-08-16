# 30-hand-off: hand off

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20L-build-logic/output/README.md | L4 | upstream artifact produced by `20L-build-logic` |
| ../20U-build-variants/output/README.md | L4 | upstream artifact produced by `20U-build-variants` |

## Purpose

The prototype and its answer are handed off.

Trigger (workflow-level): The user wants to sanity-check whether a state model or logic feels right, or explore what a UI should look like.

## What must become true here (durable outcome)

The prototype and its answer are handed off.

## Behavior contract

- **After building the variants, the actor hands the user the URL and variant keys; the most useful feedback typically recombines pieces across variants rather than picking one outright.**
  (trigger: the variant switcher is built and ready; outcome: the user has a shareable URL to explore variants and can express cross-variant preferences)
  — `BU-P3-035`, `reference/sergeant-upstream/.agents/skills/prototype/UI.md` (process step5, line 96)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- How to present the handoff (URL, variant keys, or the logic prototype's own invocation instructions).

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only when the user has a shareable, runnable handoff and can express a preference (including cross-variant recombination, for the UI branch — `BU-P3-035`).

### Decision evidence
The handoff artifact is this stage's own durable output.

## Additional note

Depends on whichever of `20L-build-logic` / `20U-build-variants` actually executed for this run — the two are mutually exclusive, not both-required.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
