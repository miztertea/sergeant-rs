# 60-re-verify-and-postmortem: re-verify and postmortem

## Inputs

| File | Layer | Why |
|---|---|---|
| ../55-refute/output/findings.md | L4 | the confirmed/refuted finding set from the panel over the fix |
| ../50-panel/output/findings.md | L4 | the panel's own coverage record, cited honestly in the close packet |
| ../40-fix-with-regression-test/output/fix.md | L4 | the original defect fix this stage re-verifies and, if the panel confirmed further findings, extends |

## Purpose

The fix commits have been re-attacked and their tests audited; the
root-cause record exists: what was actually wrong, how it got in, what
would have caught it.

## What must become true here (durable outcome)

Every `confirmed` finding from `55-refute` is fixed in a named commit or
recorded as unfixed with a reason (`@@fix-confirmed`); the resulting fix
commits — the original defect fix from `40`, plus any commit this stage
adds for a confirmed finding — are re-attacked and their tests audited
(`@@re-verify`); a root-cause postmortem is written; and a complete
evidence packet per `@@close` and `@@evidence-requirements` closes the
workflow, with every declared `promote` artifact from this package's
earlier stages — `40-fix-with-regression-test/fix.md`,
`50-panel/findings.md`, `55-refute/findings.md`, and this stage's own
`re-verify-and-postmortem.md` — named and confirmed present.

## Behavior contract

Apply `@@fix-confirmed` for any `confirmed` finding from `55-refute`, then
`@@re-verify` over the resulting fix commits, then `@@close` and
`@@evidence-requirements` to assemble and confirm the closing packet.
This package's own narrowing:

- **Only `confirmed` findings from `55-refute` are fixed here; no
  opportunistic change.** A `refuted` finding is not touched.
  (trigger: this stage begins; outcome: any additional fix this stage
  makes is exactly the confirmed panel findings, nothing more)
- **The subject of the re-verify pass is every fix commit that exists by
  this point** — the original fix from `40-fix-with-regression-test`, and
  any commit this stage added for a confirmed finding.
  (trigger: fixes are settled; outcome: the re-attack covers the whole
  fix, not just whatever this stage itself just added)
- **Two passes run: a re-attack for defects the fixes introduced, and a
  test-honesty audit of every test added or changed across those
  commits.**
  (trigger: fix commits are identified; outcome: both measured failure
  classes — a fixer-introduced blocker and a test that proves nothing —
  are looked for by name)
- **A new blocker is a `needs_input` escalation, not a second fix round.**
  There is no loop primitive in this engine.
  (trigger: the re-attack finds a blocker; outcome: the human decides
  whether to extend this Work, rather than the workflow improvising
  another round)
- **Before the postmortem, confirm the closing checklist**: the original
  repro from `10-reproduce-and-minimize` no longer reproduces, the
  regression test passes (or its seam-absence is documented), all tagged
  debug instrumentation from `30-instrument` is removed via a prefix
  grep, and throwaway prototypes are deleted or clearly marked.
  (trigger: re-verify is clean; outcome: the diagnosis is confirmed
  actually closed, not just believed closed)
- **The postmortem — what was actually wrong, how it got in, what would
  have caught it — is written after the fix and re-verify, when the most
  is known, and states the correct hypothesis (from `20`/`30`) for the
  next debugger.**
  (trigger: the closing checklist passes; outcome: a durable root-cause
  record exists, timed deliberately after the fix rather than guessed
  before it)
- **If the answer to "what would have prevented this" involves an
  architectural change (no good test seam, tangled callers, hidden
  coupling), that finding and a recommendation are recorded — never
  dispatched or delegated, only recommended for Captain and the human to
  act on.**
  (trigger: the postmortem implicates architecture; outcome: the
  recommendation is durably recorded without this workflow broadening its
  own scope to include the architectural work)
- **A clean re-verify is recorded as a positive result — what was
  attacked, how, and what was found not to be wrong — never as an empty
  file.**
- **Every declared `promote` artifact from `40-fix-with-regression-test`,
  `50-panel`, and `55-refute` is named by path and confirmed to exist,
  alongside this stage's own `re-verify-and-postmortem.md`.** This is the
  disposition act itself, performed by this stage as a named act in its
  own contract — no deterministic finalize helper does this (§1.7 of the
  design record: the directory a finalize helper would live under is not
  one `sgt init` writes into an estate). The panel's coverage from `50` is
  stated honestly here too — four axes, or fewer with the missing axis
  named.
  (trigger: closing the workflow; outcome: the "silence promotes nothing"
  policy is applied explicitly, and this package's terminal stage carries
  the same disposition duty every other closing stage in the corpus
  carries)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- How to fix a confirmed finding, within `@@fix-confirmed`'s bounds.
- How to design the re-attack and test-honesty audit for the specific
  commits in front of it.
- Judging whether the fix implicates an architectural change worth
  flagging.

### J1 — local choices allowed
None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- A confirmed finding cannot be fixed within the intent's scope.
- A new blocker survives into the fix commits.
- A declared `promote` artifact from an earlier stage cannot be found, or
  a finding's final disposition cannot be determined from the upstream
  record — the packet says so rather than asserting completion it cannot
  back.

### Completion boundary
This stage may complete only when every confirmed finding is fixed or
recorded unfixed with a reason, both re-verify passes have run over every
fix commit, the closing checklist is verified, the postmortem is written,
and every upstream `promote` artifact is named and confirmed present.

### Decision evidence
`output/re-verify-and-postmortem.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
