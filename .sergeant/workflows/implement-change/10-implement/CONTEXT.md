# 10-implement: implement

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-orient/output/orientation.md | L4 | the pinned revision and boundary this implementation stays inside |
| ../05-baseline/output/baseline.md | L4 | the test command and pre-change state this implementation is built against |

## Purpose

The change exists as commits, built test-first per `@@test-first`.

## What must become true here (durable outcome)

The change is implemented as a series of commits, each following the
red-green-minimal loop at a confirmed seam; a merge conflict encountered
along the way is resolved per `@@resolve-conflicts`, never papered over.

## Behavior contract

- **Apply `@@test-first` for every seam this change touches**: confirm
  seams before writing tests against them, red before green, one seam at
  a time, vertical slices.
  (trigger: implementation begins; outcome: every behavior-bearing commit
  has a test that failed before it and passes after it)
- **A merge or rebase conflict met while implementing is resolved via
  `@@resolve-conflicts`** — research intent, never invent behavior, never
  abort.
  (trigger: a conflict arises mid-implementation; outcome: the conflict is
  resolved consistent with both sides' traced intent, or escalated per
  that context's own J0, never worked around by abandoning the merge)
- **The builder's own read of its diff is panel input, never a substitute
  for the panel.** Self-checking the change before `20-panel` runs is
  good practice; it is not itself the independent review this workflow
  requires (R-S0-12).
  (trigger: the implementation is complete; outcome: the actor does not
  treat its own confidence as equivalent to the panel's verdict)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Exact implementation shape within the confirmed seams and the change's
  stated boundary from `00-orient`.

### J1 — local choices allowed
- Commit granularity and ordering, provided each commit is a coherent
  red-green-minimal step.

### J0 — must become `needs_input`
- `@@test-first`'s own J0: no test may be written at an unconfirmed seam
  — stop and confirm the seam list with the user before the first test of
  a cycle.
- A conflict `@@resolve-conflicts` cannot resolve from traced intent
  alone.
- The only available implementation path would require changing the
  change's own boundary or acceptance from `00-orient` — that is a scope
  question, not an implementation choice.

### Completion boundary
This stage may complete only once every planned seam has a committed
red-green-minimal cycle and no unresolved conflict remains.

### Decision evidence
The commit history itself is the primary record; `output/implementation.md`
names the commits, the seams each one covers, and any conflict resolved
along the way.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
