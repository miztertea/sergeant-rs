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

## Bounded judgment

Apply `@@bounded-judgment`.

### J4 — explicit user or bound Work decision
- An effort's own Notes may explicitly override the plan-don't-do default to carry execution into the map (`BU-P4-076`); the stage honors this without reconfirming it.

### J5 — governing constraint
- G7 (dynamic ticket graph, dynamically-discovered checkpoint graph) is closed — the loop back to `10-map-frontier` is represented as fresh re-invocation, not engine-level looping (`reference-corpus/synthesis.md` §5).

### J2 — delegated to this stage
- None beyond the plan-don't-do default itself, which is J4/J5 as above.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only once remaining fog is re-evaluated; if fog remains, the workflow loops back to `10-map-frontier` via fresh re-invocation rather than an engine-level loop primitive.

### Decision evidence
The map's Not yet specified section is this stage's own durable record of what fog remains.

## Additional note

The loop from this stage back to `10-map-frontier` is exactly the shape engine-gap **G7** (dynamically-discovered checkpoint graph) claimed as requiring new runtime machinery. G7 was **rejected** (`reference-corpus/synthesis.md` §5): the claim's own "why it fails" for the external-tracker rung was an ownership preference, not a representational failure — Wayfinder is faithfully represented today at the shared-context/helper rung with the issue tracker as the durable store, including the claim primitive. This draft represents the loop as re-invocation: completing this stage with fog remaining resubmits the workflow (a fresh Work) rather than the engine looping a pinned stage sequence.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
