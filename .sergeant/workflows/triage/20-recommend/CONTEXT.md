# 20-recommend: recommend

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-gather-context/output/README.md | L4 | upstream artifact produced by `10-gather-context` |

## Purpose

A category/state proposal is made, then the run waits for direction.

Trigger (workflow-level): An item is at the front of one of the three fixed attention buckets, oldest first.

## What must become true here (durable outcome)

A category/state proposal is made, then the run waits for direction.

## Behavior contract

- **The actor proposes a category/state recommendation with reasoning and a relevant codebase summary, then waits for the maintainer's direction before proceeding.**
  (trigger: context has been gathered; outcome: the maintainer has a recommendation to react to before any state-changing action occurs)
  — `BU-P3-066`, `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` (line 72)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Proposing a category/state recommendation with reasoning and a relevant codebase summary (`BU-P3-066`).

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- **No state-changing action proceeds before explicit maintainer direction.** This stage always ends by waiting for direction — the recommendation is a proposal, never a decision the stage makes for itself.

### Completion boundary
This stage may complete only when a recommendation with reasoning has been proposed and the run is waiting for maintainer direction.

### Decision evidence
The proposed recommendation is this stage's own durable output.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
