# N1 Adjudication — Round 1 (2026-08-10)

Orchestrator rulings on the refute phase's 21 findings (two independent
reviewers: boundary-honesty N1-BH-*, completeness/invention N1-R3-*) and the
structural lint's two substantive defects. Per L9, these rulings are
themselves reviewable findings; the freeze does not erase them.

## Spec-level rulings (the corpus was right, the spec doc was wrong)

**A1 — representation vocabulary (lint defect 1, part of BH-03 context).**
The extraction instructions defined the vocabulary the 966-unit corpus uses
consistently: `agents-invariant | workflow | stage | stage-context | helper |
shared-helper | shared-context | obsolete-mechanism | engine-gap`.
`record-shapes.md` §4 — written in parallel by a different worker — invented
a conflicting enum while its own worked example used `stage-context`. Ruling:
**amend `record-shapes.md` §4 to the corpus vocabulary.** `obsolete-mechanism`
is a legal terminal category: the contract's §8.2 stress test and proposal
§8.1's `obsolete-mechanisms.md` require the disposition to be expressible
per-unit. The corpus is not reclassified. (Fixed directly by the
orchestrator, this commit series.)

**A2 — quote hashes need their preimage convention and their preimage
(R3-01 error, R3-02, BH-11).** A hash whose hashed-span convention is
unspecified and whose preimage is unrecorded verifies nothing — 106 units
(57 in P8) currently fail reproduction, indistinguishably from invention.
Ruling: `record-shapes.md` §3 gains (a) the convention — sha256 over the
exact contiguous byte span quoted from the cited file, no normalization —
and (b) a required `quote` field carrying the quoted text verbatim (≤500
chars; longer spans quote their first 500 bytes and hash the full span with
a recorded `span_bytes`). Fixer re-derives `quote` + `quote_hash` for all
966 units mechanically; units whose statement cannot be re-anchored to a
real contiguous span are marked `confidence: low` with a `citation: disputed`
note rather than deleted — absence of a reproducible quote is a fact the
corpus records, not one it hides.

## Structural rulings on the draft workflows

**A3 — BH-01 (error): dispatch ordering.** Confirmed; fix as found: fleet
reconciliation must precede tracked-work creation. Reorder
`40-reconcile-before-launch` ahead of `30-create-tracked-work` (renumber
accordingly); its contract already says "before new work is created."

**A4 — BH-02 (error): 81 machinery stages.** Confirmed — the drafters used
a future engine feature as a stage justification, which convention §5 rule 1
forbids and which would poison N2's measurement (the generator would be
graded against over-staged gold). Ruling: **demote by default.** Every stage
whose only argument is the §6.5 boilerplate folds into its adjacent
judgment-bearing stage as a helper invocation (helper listed in the stage
context, script reference preserved). The 10 stages carrying a real
"Additional note" checkpoint argument are judged case-by-case by the fixer
against §6.3's reimplementation test, with the argument kept or the stage
demoted and the decision recorded in the package's provenance.md. Expected
outcome: stage counts drop substantially; that is correct, not lossy — the
behavior units survive as helper/stage-context material.

**A5 — BH-04 (error): validate-and-ship.** Confirmed on both halves: restore
the two extracted checkpoints (`10-check-scope`, `20-do-the-work` — id
collisions are renamed, never dissolved), and re-rung `40-start-run` as an
actor stage (its contract carries judgment). Redesign that package's stage
list accordingly.

**A6 — BH-06: repo-release-verification.** Confirmed — file-shape
mirroring; §6.2 was never argued. Demote the package: the behavior becomes a
stage/helper candidate inside `validate-and-ship` (per its own citation
trail), and the standalone package is removed with a provenance note.

**A7 — BH-07: monitor-fleet.** Confirmed. The two mutating stages
(`20-reconcile-terminal`, `30-background-...`) move to
`reconcile-and-cleanup-fleet` (already a candidate); `monitor-fleet` keeps
its read-only outcome with its remaining stages. Both packages' purposes and
provenance updated.

**A8 — BH-10: duplicated reconcile checkpoint.** Ruling: `dispatch` owns
fleet reconciliation (its source is the authority for the automatic
pre-launch sweep). `cross-repo-work/60-reconcile` narrows to the
repo-set-specific completion facts (PRs/CI/merge order for the repos in this
Work) and its context *names* dispatch's reconciliation as adjacent owned
procedure without pretending to invoke it — the wish to invoke it is
recorded in `engine-pressure.md` as evidence for the existing composition
trigger (proposal §21.8), not as a new claim.

## Unit-record rulings

**A9 — BH-03/R3-05: missing rationale (226) and alternatives (430).**
Split ruling. `rationale: null` is a genuine defect: fixer backfills all 226
from the unit's own statement + source evidence (deriving the recorded
reasoning is legitimate; inventing evidence would not be). For
`alternatives_considered: []`, amend `record-shapes.md` §4: an empty list is
legal **only** where no adjacent rung was facially plausible; fixer
backfills alternatives for every unit that carries a workflow or stage
boundary, every engine-gap unit, and every unit named in a conflict —
boundary-bearing classifications must be refutable; leaf stage-context
units with an obvious single rung may stay empty.

**A10 — BH-08: BU-P6-129.** Confirmed. Demote to `confidence: low`, split
the four-requirement statement into units its actual quotes support, and let
validate-and-ship's boundary rest on the stronger citations already in its
provenance.

**A11 — BH-09: 146 un-normalized statements.** Confirmed. Fixer rewrites the
146 statements implementation-independently (mechanism nouns → role nouns),
preserving ids and citations; the copy-pasted reader-note blocks in draft
packages are then redundant and removed.

**A12 — R3-08/R3-09 + lint residual: missing coverage.** Extract the
AGENTS.md routing table (6 rows) and the 4 dispatch worker-contract items as
new units (P1/P5 id ranges continue); split BU-P8-077 into one unit per
stage. Route the new units into the affected packages' provenance.

## Ledger and engine-pressure rulings

**A13 — BH-05 + R3-04 + R3-03: X1, X8, G5, G9.**
- X1: overturned as found — the test-backed unit wins (binding rule: tests
  may outrank documentation); the circular G9 citation is severed.
- G9: re-derived on its merits after X1 flips; if its only rejection ground
  was X1's discarded reading, it re-enters as a candidate claim and must
  survive §6.7 on its own evidence.
- G5: rejected — its "never attempted" lower rung (needs_input round-trips)
  exists in the shipped engine; re-file only if a measured N2 run shows the
  narrower residual gap the finding describes.
- X8: overturned — BU-P4-053's representable-today note stands; the
  distinction X8 leaned on does not exist.

**A14 — X2–X7, X9–X20 (no reviewer objection).** Adjudicated as proposed:
two independent reviewers challenged the conflict set and objected only to
X1/X8 (plus X3's citation, corrected per BH-12: BU-P1-072's source is
README.md). Each entry's status moves PROPOSED → ADJUDICATED citing this
document; X3's misattribution is fixed in place.

**A15 — R3-06/R3-07: missing gate artifacts.** Confirmed and scheduled: the
structural lint script joins the corpus as `reference-corpus/lint.py`
(re-run green required before freeze), `FROZEN.md` is written at freeze with
corpus version, source SHA `f430cfd4f90174a98adbd7abebbece6303817929`, and
the adjudication date. R3-07's "not adjudicated" was true when written; this
document is the adjudication it called for.

## Lesson candidate (for LESSONS.md at close-out)

A hash without its preimage convention is not evidence: rules of the form
"hash must verify" bind only when the hashed span's derivation is specified
and the preimage is recorded. 106 units' citations were unverifiable not
because they were invented but because the rule was unenforceable by
construction (R3-02). Specify the convention before demanding the proof.
