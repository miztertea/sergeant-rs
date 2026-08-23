# 40-disposition-matrix: disposition matrix

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-ingest/output/ingest.md | L4 | the original ingested id set this matrix must fully account for |
| ../20-dispose/output/dispositions.md | L4 | each finding's disposition and reason |
| ../30-implement-accepted/output/fixes.md | L4 | which accepted findings were fixed, and by which commit |
| ../35-re-verify/output/re-verify.md | L4 | the re-verify result, including any new finding |

## Purpose

Every ingested id appears exactly once with its disposition, reason, and
(where accepted) its commit.

## What must become true here (durable outcome)

A completeness matrix exists: every id ingested at `00-ingest` appears
exactly once, with its disposition, reason, and — where `accepted` — its
fix commit or its recorded unfixed reason.

## Behavior contract

Apply `@@close` and `@@evidence-requirements`. This package's own
narrowing:

- **Every id from `00-ingest` appears exactly once.** Not fewer (nothing
  silently dropped), not more (nothing invented).
  (trigger: assembling the matrix; outcome: the completeness proof this
  workflow exists to provide is checkable by counting rows)
- **Each row states the disposition, the reason, and — for `accepted` —
  the fix commit(s) or the recorded unfixed reason.**
  (trigger: assembling the matrix; outcome: a reader can verify every
  claim without re-reading every upstream stage)
- **Any new finding from `35-re-verify` is added to the matrix as its own
  row**, not folded silently into an existing id's disposition.
  (trigger: re-verify produced a new finding; outcome: the matrix
  accounts for it explicitly rather than conflating it with the original
  set)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
How to phrase the matrix's narrative sections.

### J1 — local choices allowed
Formatting and ordering of the matrix.

### J0 — must become `needs_input`
An ingested id cannot be traced to a disposition from the upstream
record — the matrix says so rather than asserting completeness it
cannot back.

### Completion boundary
This stage may complete only once every ingested id appears exactly once
with its disposition, reason, and (where accepted) commit.

### Decision evidence
`output/disposition-matrix.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
