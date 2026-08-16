# 00-read-source: read source

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

The external skill's complete SKILL.md and referenced scripts are read before adopting it.

Trigger (workflow-level): Before adopting an external skill, or when an adopted skill needs updating.

## What must become true here (durable outcome)

The external skill's complete SKILL.md and referenced scripts are read before adopting it.

## Behavior contract

- **Read the external skill's complete SKILL.md and referenced scripts before adopting it.**
  (trigger: vet-external-skill workflow entered; outcome: the skill's full instructions and scripts are read, not sampled)
  — `BU-P1-120`, `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L126, vet step 1)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- What counts as a "referenced script" — following the skill's own reference graph to completion rather than reading only the top-level file (`BU-P1-120`).

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- None specific to this stage beyond `@@bounded-judgment`'s general triggers.

### Completion boundary
This stage may complete only once the skill's complete `SKILL.md` and every referenced script have been read in full, not sampled.

### Decision evidence
The read evidence is this stage's own durable output, recorded per `output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
