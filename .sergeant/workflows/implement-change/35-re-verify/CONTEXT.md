# 35-re-verify: re-verify

## Inputs

| File | Layer | Why |
|---|---|---|
| ../30-fix-confirmed/output/fixes.md | L4 | the fix commits this stage attacks — its only subject |

## Purpose

The fixer's commits have been re-attacked and their tests audited; new
findings are recorded and severity-ranked.

## What must become true here (durable outcome)

Both the re-attack pass and the test-honesty audit have run over every
fix commit listed in `30-fix-confirmed/output/fixes.md`, and any new
finding is recorded in the same typed set, severity-ranked.

## Behavior contract

Apply `@@re-verify`. This package's own narrowing:

- **The subject of this stage is the fix commits listed in
  `../30-fix-confirmed/output/fixes.md` — not the whole change and not the
  original feature diff.**
  (trigger: this stage begins; outcome: the re-attack lands on the code
  most likely to carry a fresh defect, which is the measured point of the
  stage)
- **Two passes run over those commits: a re-attack for defects the fixes
  themselves introduced, and a test-honesty audit of every test the fixer
  added or changed.**
  (trigger: the fix commits are identified; outcome: both classes the
  sprints measured — the fixer-introduced blocker and the test that
  proves nothing — are looked for by name)
- **The test-honesty audit checks, for each new or changed test, that it
  fails against the pre-fix code — or records explicitly that this was
  not demonstrated and why.**
  (trigger: auditing a test; outcome: "tests passed" is never accepted as
  evidence on its own)
- **New findings are recorded in the same typed shape, with ids
  continuing the panel's series, and are severity-ranked.**
  (trigger: the re-attack finds something; outcome: one finding set spans
  the whole run and the close packet can account for all of it)
- **A new `blocker` found here becomes a `needs_input` escalation carrying
  the finding, its evidence, and the decision required. This stage does
  not start a second fix round and this workflow has no loop.**
  (trigger: a blocker survives into the fixes; outcome: the human decides
  whether to extend this Work or open another, rather than the workflow
  improvising a round the engine cannot express)
- **A clean re-verify is recorded as a positive result — what was
  attacked, how, and what was found not to be wrong — never as an empty
  file.**
  (trigger: no new findings; outcome: the close packet can show the stage
  ran rather than assuming it)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
How to design the re-attack and the test-honesty audit for the specific
fix commits in front of it.

### J1 — local choices allowed
None beyond ordinary tool mechanics — both passes are required, in full,
over every listed commit.

### J0 — must become `needs_input`
A new `blocker` survives into the fix commits.

### Completion boundary
Both passes have run over every listed fix commit, and every new finding
is in the set with a severity.

### Decision evidence
`output/re-verify.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its merge disposition.
