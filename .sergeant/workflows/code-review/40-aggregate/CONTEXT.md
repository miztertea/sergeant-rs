# 40-aggregate: aggregate

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-30-parallel-review/output/README.md | L4 | upstream artifact produced by `20-30-parallel-review` |

## Purpose

The two axes are reported separately, never merged or reranked.

Trigger (workflow-level): A diff needs review before merge (invoked directly or delegated from `worker-mission`/`implement`).

## What must become true here (durable outcome)

The two axes are reported separately, never merged or reranked.

## Behavior contract

- **The two sub-agent reports are presented under separate `## Standards` and `## Spec` headings, verbatim or lightly cleaned, and must never be merged or reranked against each other since the two axes are deliberately kept separate.**
  (trigger: both sub-agent reports have returned; outcome: a combined report exists with the two axes still clearly distinguishable)
- **The aggregated report ends with a one-line summary of total findings per axis and the worst issue within each axis, without picking one overall winner across axes.**
  (trigger: the two-axis report has been assembled; outcome: a concise cross-cutting summary line closes the report, without collapsing the two axes into one ranking)
- **The two-axis design exists because a change can pass one axis and fail the other (standards-compliant but spec-wrong, or spec-correct but convention-breaking), and reporting them separately stops either axis from masking the other's failure.**
  (trigger: n/a (design rationale); outcome: the two-axis structure is preserved rather than collapsed)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Lightly cleaning the two sub-agent reports for the final aggregate without altering their substance.
- Writing the one-line closing summary — total findings per axis, worst issue within each axis.

### J1 — local choices allowed
- Exact Markdown formatting of the `## Standards` / `## Spec` headings and summary line.

### J0 — must become `needs_input`
- None identified for this stage — aggregation is bounded, mechanical composition of already-produced upstream reports. A downstream ambiguity (e.g. both reports empty and equally silent) is not a J0: report it truthfully rather than escalating.

### Completion boundary
This stage may complete only when both axes are presented under separate headings, unmerged and unreranked (J5 per the package's Authority envelope), and the closing summary line does not pick a single winner across axes.

### Decision evidence
The aggregated report itself, written to `output/README.md` with `promote` disposition, is the decision record.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
