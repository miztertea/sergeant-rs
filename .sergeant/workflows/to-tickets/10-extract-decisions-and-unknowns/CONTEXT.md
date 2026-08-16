# 10-extract-decisions-and-unknowns: extract decisions and unknowns

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-load-project-context/output/README.md | L4 | upstream artifact produced by `00-load-project-context` |

## Purpose

An investigation ticket is created only for a genuinely blocking unknown, naming the exact artifact it must produce.

Trigger (workflow-level): The user says "to tickets", "create issues", "create td tasks", "make epics", or asks to break something into work.

## What must become true here (durable outcome)

An investigation ticket is created only for a genuinely blocking unknown, naming the exact artifact it must produce.

## Behavior contract

- **Create a short investigation ticket only when a genuinely blocking unknown cannot be answered from existing evidence, and that ticket must name the exact decision or artifact it is meant to produce.**
  (trigger: an unknown is identified while drafting a ticket breakdown; outcome: investigation tickets are created sparingly and each has a named deliverable)
  — `BU-P4-065`, `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` (Extract Decisions and Unknowns, L60-62)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Judging whether an unknown is genuinely blocking versus answerable from existing evidence (`BU-P4-065`).

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- The evidence for whether an unknown is genuinely blocking is itself ambiguous or contested (e.g. one reading of the source material treats it as blocking, another as safely deferrable): state what was checked and ask rather than resolving the disagreement unilaterally.

### Completion boundary
This stage may complete only once an investigation ticket exists for every genuinely blocking unknown, each naming its exact deliverable — or the stage has stopped at the J0 case above.

### Decision evidence
The extracted decisions and unknowns are this stage's own durable output, recorded per `output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
