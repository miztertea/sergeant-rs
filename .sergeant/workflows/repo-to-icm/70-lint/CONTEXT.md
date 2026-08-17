# 70-lint: validate and mechanically repair the draft tree

## Inputs

| File | Layer | Why |
|---|---|---|
| references/mechanical-vs-substantive.md | L3 | the line between defects this stage may fix directly and defects it must leave for `80`/`90` |
| ../_config/run-discipline.md | L3 | the blindness rule and the `# AMBIGUOUS — NOT RESOLVED` propagation rule |
| ../scripts/validate-structure.py | L3 | the helper this stage runs against each candidate package (see "How to do it"); this workflow's own tree is checked by `65-self-check`'s pinned container instead — see that stage's own row below |
| ../60-draft/output/draft-report.md | L4 | upstream artifact produced by `60-draft` — the manifest naming every candidate package this stage validates |
| ../40-classify/output/classifications.ndjson | L4 | upstream — `references/mechanical-vs-substantive.md`'s substantive test for a missing `engine_gap` field requires checking whether the correct content is already recoverable verbatim from here |
| ../65-self-check/output/self-check-result.txt | L4 | upstream artifact produced by `65-self-check` (`kind = "execute"`, N4) — this workflow's own tree's validator result, already run mechanically before this stage started; folded in verbatim rather than re-run |

## Working directory

Every command below is written **from the repository root** — the same
directory `.sergeant/` lives directly under, which is this run's actual
working directory (the materialized work surface's single bound worktree;
`sergeant-rs-workspace/knowledge/evidence/gauntlet/notes/n2-fake-backend-semantics.md`'s cwd is a fresh actor
turn's cwd, not any one stage's own directory). Do not run these commands
from inside `70-lint/` itself, and do not assume `../scripts/...` resolves
— it only would from inside this stage's own directory, which is not where
this turn starts.

## Purpose

Run `.sergeant/workflows/repo-to-icm/scripts/validate-structure.py` against
every candidate package `60-draft` materialized; repair the defects that
are mechanical per `references/mechanical-vs-substantive.md`; leave
substantive ones for `80-adversarial-review` and `90-reconcile`. This
stage's `output/lint-report.md` **also** covers this workflow's own tree —
the *authored* tree passes cleanly by construction, but this is a *run*
worktree: `40-classify` has just written new `classifications.ndjson` into
it, and that is exactly the kind of NDJSON the validator's S9 check (engine-
gap record completeness) exists to scan. That check itself already ran,
mechanically, in `65-self-check` (`kind = "execute"`, N4) before this stage
started; this stage folds its result in rather than re-running the
validator against this workflow's own tree a second time.

## Bounded judgment

Apply `@@bounded-judgment`.

A governing constraint (J5, `references/mechanical-vs-substantive.md`'s own
test): a defect this stage cannot repair without deciding something it was
not given the authority to decide is never force-fixed to make the
validator pass — it is logged as a finding for `80`/`90`.

### J2 — delegated to this stage
- Classifying each validator-reported defect as mechanical or substantive
  per `references/mechanical-vs-substantive.md`'s test, and fixing every
  mechanical one directly (re-running the validator until none remain).
- Attributing a repository-wide `[S7]` finding correctly (recorded once,
  under its own heading, never repeated under a candidate it happens to
  surface under).

### J1 — local choices allowed
- The order candidates are processed in, so long as every candidate named
  in `../60-draft/output/draft-report.md` is covered and this workflow's
  own tree is covered via `65-self-check`'s result (not re-run).

### J0 — must become `needs_input`
- `../60-draft/output/draft-report.md` opens with `#
  AMBIGUOUS — NOT RESOLVED` — do not proceed; follow `../_config/
  run-discipline.md` §2.

When genuinely unsure whether a defect is mechanical, this stage's own
contract already resolves the tie: treat it as substantive and log it for
`80`/`90` — this is not itself a needs_input escalation (the run continues;
the finding travels forward as recorded signal), but it is the operative
rule for every case this stage cannot cleanly place in J2.

### Completion boundary
This stage may complete only when `output/lint-report.md` covers every
candidate named in `../60-draft/output/draft-report.md` **plus** this
workflow's own tree (from `65-self-check`'s result), each candidate's
defects classified mechanical-fixed or substantive-remaining, with no
candidate silently skipped and this workflow's own tree not silently
assumed clean.

### Decision evidence
`output/lint-report.md` — per-candidate mechanical-fixed vs. substantive-
remaining defect lists — is this stage's decision record.

## What must become true here (durable outcome)

`output/lint-report.md` exists, covering every candidate package named in
`../60-draft/output/draft-report.md`, **plus** this workflow's own tree: the
validator's initial findings for each, each classified mechanical-fixed or
substantive-remaining per `references/mechanical-vs-substantive.md`, and
the final validator result after mechanical repairs (pass, or fail with the
substantive defects still listed). No candidate is silently skipped, and
this workflow's own tree is not silently assumed clean.

## How to do it

0. If `../60-draft/output/draft-report.md` opens with `#
   AMBIGUOUS — NOT RESOLVED`, do not proceed — follow
   `../_config/run-discipline.md` §2.

For each candidate package path from `../60-draft/output/draft-report.md`
(paths there are relative to the repository root, e.g.
`.sergeant/drafts/workflows/<candidate-name>`):

1. Run `python3 .sergeant/workflows/repo-to-icm/scripts/validate-structure.py
   <candidate-path>`. Review its structured result — this is a helper, not
   something the engine interprets on its own; the judgment about what its
   output means is yours (`docs/icm/convention.md` §5).
2. Classify each reported defect using `references/mechanical-vs-substantive.md`'s
   test. When genuinely unsure, treat it as substantive.
3. Fix every mechanical defect directly in the candidate package.
4. Re-run the validator. Repeat 1–3 until no mechanical defects remain.
5. Record the final state — validator PASS/FAIL, defects fixed, defects
   remaining (with their `[Sn]` codes) — in `output/lint-report.md` under
   that candidate's own heading. **Exception:** an `[S7]` finding whose
   package name is not the candidate you just validated is a repository-
   wide check result (it compares `.sergeant/workflows/` against
   `.sergeant/drafts/workflows/` as a whole, independent of which single
   tree you pointed the validator at) — it is not attributable to this
   candidate. Record it at most once, under a `## Repository-wide (not
   attributable to any one candidate)` heading, not repeated under every
   candidate's own heading it happens to surface under.

After every candidate has been processed:

6. Read `../65-self-check/output/self-check-result.txt` — the validator's
   result against this workflow's own tree, already run mechanically by
   the `65-self-check` execute stage before this stage started (nothing to
   invoke here; that stage's container ran it once, unconditionally, for
   every run). Record it under a `## This workflow's own tree` heading in
   `output/lint-report.md`, the same way as a candidate: PASS/FAIL
   verbatim from that file, and any `[S9]` engine-gap defects found in
   `40-classify/output/classifications.ndjson` are substantive findings
   for `80`/`90`, not something this stage force-fixes. Do not re-run the
   validator against this workflow's own tree yourself — `65-self-check`
   already did, and re-running it here would only reproduce the same
   mechanical result a second time for no reason.

A candidate (or this workflow's own tree) that still fails after mechanical
repairs is not a failure of this stage; it is real signal for
`80-adversarial-review` and `90-reconcile` to work from. Do not force a
mechanical-looking fix onto a substantive defect just to make the validator
pass.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected
artifact and its disposition.
