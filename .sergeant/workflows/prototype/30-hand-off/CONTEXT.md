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

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Additional note

Depends on whichever of `20L-build-logic` / `20U-build-variants` actually executed for this run — the two are mutually exclusive, not both-required.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
