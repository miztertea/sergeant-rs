# 02-offer-adr

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |
| ../01-maintain-glossary-discipline/output/outcome.md | L4 | upstream evidence produced by `maintain-glossary-discipline` — this stage's own future runs expect that checkpoint to have already been reached |

## Purpose

**Trigger:** a decision has just been made during the session

**Outcome:** an ADR is offered only when the three-part test passes; otherwise it is not created

**Statement (the operative rule):** An ADR is only offered when all three of hard-to-reverse, surprising-without-context, and result-of-a-real-trade-off are true; if any one of the three is missing, the ADR is skipped.

## What must become true here (durable outcome)

An ADR is offered only when the three-part test passes; otherwise it is not created — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1068`: An ADR's required content is minimal: a short title plus 1-3 sentences covering the context, the decision, and the reason — the value is in recording that a decision was made and why, not in filling out sections.
- `BU-1069`: ADR optional sections (Status frontmatter, Considered Options, Consequences) are included only when they add genuine value — most ADRs need none of them.
- `BU-1071`: ADR-worthy decisions fall into specific named categories: architectural shape, integration patterns between contexts, lock-in technology choices, boundary/scope decisions (including explicit "no" boundaries), deliberate deviations from the obvious path, constraints not visible in the code, and non-obvious rejected alternatives.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-1066`: ADRs live in docs/adr/ and are named with sequential zero-padded numbering (0001-slug.md, 0002-slug.md, and so on).
- `BU-1067`: The docs/adr/ directory is created lazily, only when the first ADR is needed.
- `BU-1070`: A new ADR's number is chosen by scanning docs/adr/ for the highest existing number and incrementing by one.

