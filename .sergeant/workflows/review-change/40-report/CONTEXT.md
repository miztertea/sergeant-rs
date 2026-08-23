# 40-report: report

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-pin-fixed-point/output/README.md | L4 | the pinned revision cited in the report |
| ../30-verify-and-severity/output/findings.md | L4 | the final finding set this report is built from |

## Purpose

The typed finding set is complete, with the panel's coverage stated.

## What must become true here (durable outcome)

A complete, typed finding report exists per `@@close`, naming the pinned
revision, the panel's coverage (four axes or fewer, named), and every
finding with its final status and severity.

## Behavior contract

Apply `@@close` and `@@evidence-requirements`. This package's own
narrowing:

- **The report states the pinned revision and diff command the panel
  ran against**, so a reader can reproduce exactly what was reviewed.
  (trigger: all upstream stages complete; outcome: the report is
  reproducible, not just assertable)
- **The panel's coverage is stated honestly** — four axes, or fewer with
  the missing axis named.
  (trigger: assembling the report; outcome: reduced coverage from
  `20-panel` is visible in the final artifact)
- **Every finding id from `20-panel` is accounted for**: raised, refuted,
  or confirmed-with-severity, including any marked stale at
  `30-verify-and-severity`.
  (trigger: assembling the report; outcome: no finding silently vanishes
  between the panel and the report)
- **This workflow recommends, and never dispatches, `remediate-findings`**
  for acting on the report — recorded as a recommendation, not a
  delegation.
  (trigger: the report is complete; outcome: the next act, if any, is
  Captain's and the human's to authorize)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
How to phrase the report's narrative sections within the shape
`@@close`/`@@evidence-requirements` require.

### J1 — local choices allowed
Formatting and ordering of the report.

### J0 — must become `needs_input`
A finding's final disposition cannot be determined from the upstream
record — the report says so rather than asserting completeness it cannot
back.

### Completion boundary
This stage may complete only once every finding id is accounted for and
the panel's coverage is stated.

### Decision evidence
`output/report.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
