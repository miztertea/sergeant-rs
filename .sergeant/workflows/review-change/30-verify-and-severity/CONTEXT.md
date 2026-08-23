# 30-verify-and-severity: verify and severity

## Inputs

| File | Layer | Why |
|---|---|---|
| ../25-refute/output/findings.md | L4 | the refuted/confirmed finding set this stage independently re-checks |

## Purpose

Each surviving finding is independently verified against current state (a
finding may already be stale) and assigned `blocker`/`major`/`minor`.

## What must become true here (durable outcome)

Every finding that is not `refuted` (i.e. every `confirmed` finding) is
independently checked against the diff's current state — since time may
have passed since `20-panel` first read it — and assigned a severity.

## Behavior contract

- **Re-check each `confirmed` finding against the diff's current state.**
  A colleague's PR reviewed here may itself have moved since the panel
  ran; a finding that no longer applies is marked stale with evidence,
  not silently carried forward as if still live.
  (trigger: `25-refute` has produced its final finding set; outcome: only
  findings that still hold against current state reach the report)
- **Assign each surviving finding a severity — `blocker`, `major`, or
  `minor`** — independent of whatever informal severity language a seat
  may have used in its own prose.
  (trigger: a finding is confirmed and current; outcome: the finding
  carries a severity a reader can act on without re-reading the seat's
  free text)
- **This stage does not fix anything (this workflow has no fixer stage)
  and does not re-open a `refuted` finding.** Verification here means
  checking staleness and assigning severity to what already survived
  `25-refute`, never re-litigating what `25-refute` already decided.
  (trigger: reviewing the refuted set; outcome: this stage's own scope
  stays exactly what it is named for)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
Judging whether a confirmed finding still applies against current state,
and its severity.

### J1 — local choices allowed
Exact wording of the staleness/severity record.

### J0 — must become `needs_input`
A finding's staleness or severity turns on a judgment only the requester
can make (e.g. whether a since-changed acceptance criterion still governs
it).

### Completion boundary
This stage may complete only once every confirmed finding has a
staleness check and a severity.

### Decision evidence
`output/findings.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
