# 01-maintain-glossary-discipline

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the user uses a term that conflicts with CONTEXT.md's existing definition

**Outcome:** the conflict is surfaced to the user immediately instead of silently accepted

**Statement (the operative rule):** When the user uses a term that conflicts with the existing language already recorded in CONTEXT.md, that conflict is called out immediately rather than let pass.

## What must become true here (durable outcome)

The conflict is surfaced to the user immediately instead of silently accepted — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1060`: When the user uses a vague or overloaded term, a precise canonical term is proposed to replace it.
- `BU-1061`: When domain relationships are being discussed, they are stress-tested with invented edge-case scenarios that force precision about the boundaries between concepts.
- `BU-1062`: When the user states how something works, the code is checked against that statement, and any contradiction found is surfaced to the user rather than left unresolved.
- `BU-1063`: When a domain term is resolved, CONTEXT.md is updated inline at that moment rather than the update being batched up for later.
- `BU-1072`: When multiple words exist for the same domain concept, one is opinionatedly picked as canonical in CONTEXT.md and the others are listed under that term's _Avoid_ line.
- `BU-1073`: CONTEXT.md term definitions are kept to one or two sentences at most, and define what the term IS rather than what it does.
- `BU-1074`: Only terms specific to the project's own context are included in CONTEXT.md; general programming concepts (timeouts, error types, utility patterns) are excluded even if heavily used, judged by whether the concept is unique to this context or general-purpose.
- `BU-1075`: CONTEXT.md terms are grouped under subheadings when natural clusters emerge, and left as a flat list when all terms belong to a single cohesive area.
- `BU-1077`: When multiple contexts exist, which context the current topic relates to is inferred; if that is unclear, the user is asked rather than guessed.

