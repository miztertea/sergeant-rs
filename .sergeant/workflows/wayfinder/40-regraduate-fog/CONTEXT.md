# 40-regraduate-fog: regraduate fog

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-resolve-one/output/README.md | L4 | upstream artifact produced by `30-resolve-one` |

## Purpose

Remaining fog is re-evaluated; the run loops back to `10-map-frontier` if fog remains.

Trigger (workflow-level): A destination is named that requires mapping fog before it can be reached.

## What must become true here (durable outcome)

Remaining fog is re-evaluated; the run loops back to `10-map-frontier` if fog remains.

## Behavior contract

- **Wayfinder defaults to planning only: each ticket resolves a decision and the map is done once nothing is left to decide, not once the underlying work is executed; an effort may explicitly override this default in its own Notes to carry execution into the map.**
  (trigger: a wayfinder map is being charted or worked; outcome: the map stays scoped to decisions unless the effort explicitly opts into carrying execution)
  — `BU-P4-076`, `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Plan don't do, L13)
- **After creating research-type tickets during charting, immediately fire a research subagent per ticket in parallel to resolve it, capturing findings on a throwaway branch with a context pointer back to the ticket.**
  (trigger: research-type tickets have just been created during charting; outcome: research tickets begin resolving immediately and in parallel, rather than waiting for a future session)
  — `BU-P4-097`, `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Invocation / Chart the map, L115)
- **Because unblocked tickets may be worked in parallel by other users, a session working through the map should expect other sessions to be editing the tracker concurrently.**
  (trigger: a session is working through the map; outcome: the actor's own edits account for the possibility of concurrent external changes)
  — `BU-P4-100`, `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (Invocation / Work through the map, L128)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Additional note

The loop from this stage back to `10-map-frontier` is exactly the shape engine-gap **G7** (dynamically-discovered checkpoint graph) claimed as requiring new runtime machinery. G7 was **rejected** (`reference-corpus/synthesis.md` §5): the claim's own "why it fails" for the external-tracker rung was an ownership preference, not a representational failure — Wayfinder is faithfully represented today at the shared-context/helper rung with the issue tracker as the durable store, including the claim primitive. This draft represents the loop as re-invocation: completing this stage with fog remaining resubmits the workflow (a fresh Work) rather than the engine looping a pinned stage sequence.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
