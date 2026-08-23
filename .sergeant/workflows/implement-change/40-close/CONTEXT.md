# 40-close: close

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-orient/output/orientation.md | L4 | the pinned revision and acceptance source claims are tied to |
| ../15-validate/output/validation.md | L4 | the validation record cited in the evidence packet |
| ../25-refute/output/findings.md | L4 | the finding set's state after refutation |
| ../30-fix-confirmed/output/fixes.md | L4 | which confirmed findings were fixed or recorded unfixed |
| ../35-re-verify/output/re-verify.md | L4 | the re-verify result, including any new findings |

## Purpose

The evidence packet exists: what ran, against which revision, satisfying
which acceptance criterion; every declared `promote` artifact is named and
present.

## What must become true here (durable outcome)

A complete evidence packet exists per `@@close` and
`@@evidence-requirements`, and every declared `promote` artifact from this
package's earlier stages is named and confirmed present.

## Behavior contract

Apply `@@close` and `@@evidence-requirements`. This package's own
narrowing:

- **The packet states which tests ran, against which revision, satisfying
  which acceptance criterion** — drawing on `00-orient`'s pinned revision
  and located spec source, and `15-validate`'s real validation output.
  (trigger: all upstream stages have completed; outcome: a reader can
  verify every claim against a named command and a named criterion)
- **The finding set's final state is reported with every id accounted
  for**: raised, refuted, confirmed-and-fixed, confirmed-and-unfixed (with
  reason), and any new finding from `35-re-verify`.
  (trigger: assembling the packet; outcome: no finding silently vanishes
  between the panel and the close)
- **The panel's coverage is stated honestly** — four axes, or fewer with
  the missing axis named, per `20-panel`'s own record.
  (trigger: assembling the packet; outcome: reduced coverage is visible
  in the final record, not just in an intermediate stage's own output)
- **Every declared `promote` artifact from `15-validate`, `20-panel`,
  `25-refute`, `30-fix-confirmed`, and `35-re-verify` is named by path and
  confirmed to exist.** This is the disposition act itself, performed by
  this stage as a named act in its own contract — no deterministic
  finalize helper does this (§1.7 of the design record: the directory a
  finalize helper would live under is not one `sgt init` writes into an
  estate).
  (trigger: closing the run; outcome: the "silence promotes nothing"
  policy is applied explicitly rather than assumed)
- **Any recommended follow-up intent — from `30-fix-confirmed`'s
  unfixed-but-confirmed findings, or from anything a fixer or refuter
  noticed outside scope — is recorded as a recommendation Captain may
  act on, never as work this run already did.**
  (trigger: assembling the packet; outcome: scope stays exactly what
  `00-orient` bounded it to, even as further work is honestly surfaced)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
How to phrase the packet's narrative sections within the shape
`@@close`/`@@evidence-requirements` require.

### J1 — local choices allowed
Formatting and ordering of the packet.

### J0 — must become `needs_input`
A declared `promote` artifact from an earlier stage cannot be found, or
a finding's final disposition cannot be determined from the upstream
record — the packet says so rather than asserting completion it cannot
back.

### Completion boundary
This stage may complete only once every upstream `promote` artifact is
named and confirmed present, every finding id is accounted for, and the
evidence packet ties its claims to a named test, revision, and acceptance
criterion.

### Decision evidence
`output/close-packet.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
