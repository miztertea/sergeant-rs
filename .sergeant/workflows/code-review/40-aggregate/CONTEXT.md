# 40-aggregate: aggregate

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-parallel-review-spec/output/README.md | L4 | upstream artifact produced by `30-parallel-review-spec` |

## Purpose

The two axes are reported separately, never merged or reranked.

Trigger (workflow-level): A diff needs review before merge (invoked directly or delegated from `worker-mission`/`implement`).

## What must become true here (durable outcome)

The two axes are reported separately, never merged or reranked.

## Behavior contract

- **The two sub-agent reports are presented under separate `## Standards` and `## Spec` headings, verbatim or lightly cleaned, and must never be merged or reranked against each other since the two axes are deliberately kept separate.**
  (trigger: both sub-agent reports have returned; outcome: a combined report exists with the two axes still clearly distinguishable)
  — `BU-P2-016`, `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 5: Aggregate, lines 78-78)
- **The aggregated report ends with a one-line summary of total findings per axis and the worst issue within each axis, without picking one overall winner across axes.**
  (trigger: the two-axis report has been assembled; outcome: a concise cross-cutting summary line closes the report, without collapsing the two axes into one ranking)
  — `BU-P2-017`, `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Step 5: Aggregate, lines 80-80)
- **The two-axis design exists because a change can pass one axis and fail the other (standards-compliant but spec-wrong, or spec-correct but convention-breaking), and reporting them separately stops either axis from masking the other's failure.**
  (trigger: n/a (design rationale); outcome: the two-axis structure is preserved rather than collapsed)
  — `BU-P2-018`, `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md` (Why two axes, lines 84-87)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
