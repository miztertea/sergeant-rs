# Independent adversarial review: package `implement` (ICM-R3)

Reviewer position per `docs/adr/0013-icm-r0-owner-rulings.md` decision 7
and `reference/proposal-icm-r-procedure-authority.md` §8.11: fresh
execution, explicit inputs (this record, the producer's draft at
`docs/gauntlet/runs/icm-r3/implement/adjudication-draft.md` and its
`draft/` content, the live package under `.sergeant/workflows/implement/`,
both current delegates (`tdd`, `code-review`) read in full at their
current live state, `docs/gauntlet/runs/icm-r3/tdd/adjudication-draft.md`
and `review.md`, `docs/gauntlet/runs/icm-r2/code-review/
adjudication-draft.md`, `.sergeant/workflows/worker-mission/CONTEXT.md`
and `20-implement/CONTEXT.md`, `docs/gauntlet/promoted-provenance/
implement.md`, `reference/proposal-icm-r-procedure-authority.md` §5, §6,
§8.10-8.11, `docs/icm/record-shapes.md` §5-6, `docs/icm/convention.md`
§4-6, `docs/adr/0013-icm-r0-owner-rulings.md`), review-only contract, no
edit authority over the producer's draft or the live package.
Classification below is independently re-derived against the package's
actual live content (`.sergeant/workflows/implement/` read in full
directly, not assumed from the producer's own citations) and against the
draft's own generated content under `draft/`.

## Summary of verdict

The producer's draft is sound. Every citation independently re-checked
against `docs/gauntlet/promoted-provenance/implement.md`, the live
package, and both live delegates resolves and is not fabricated or
misquoted. The core structural finding — that both `## Delegation`
sections wrongly describe invoking a full checkpointed workflow as
"context composition," that this is settled and correctable in full for
`code-review` but only partially correctable for `tdd` pending its own
disputed ICM-R3 placement, and that a genuine engine-gap record was
missing and should be filed — holds up against independent re-derivation
from primary sources. Two narrow defects survive independent challenge:
a disposition-modifier misapplication (`FOLD` used for in-place prose
correction that does not fit §5.10's definition) and a rule-2 compliance
gap in the filed engine-gap claim's `lower_rungs_attempted` list. Neither
disturbs the package-level conclusion. **Final disposition: STAND,
CONFIRMED**, with two NEEDS-REVISION line items to correct before
promotion.

## Behavior-unit dispositions

### BU-IMPL-01 -- verdict: CONFIRMED

Independently re-derived: `.sergeant/workflows/implement/CONTEXT.md` and
`index.md` state the trigger and purpose exactly as cited, and
`docs/gauntlet/promoted-provenance/implement.md` confirms `BU-P2-050`,
`BU-P2-051`, `BU-P3-004` verbatim. The J5 classification for
explicit-invocation-only is correct against
`reference/proposal-icm-r-procedure-authority.md` §6.2: "workflow
prohibition" is named explicitly in J5's basis list, and "must never be
auto-loaded merely because the task looks like implementation" is exactly
that shape, not a J2 choice a loader could waive. PL-4 rung is addressed
under "Driver and admission boundary," not restated here — see the
overall verdict below for that independent re-derivation.

### BU-IMPL-02 -- verdict: CONFIRMED

Independently checked `docs/gauntlet/runs/icm-r3/tdd/adjudication-draft.md`
(producer: REHOME) and `review.md` (independent reviewer: DISPUTED, citing
a genuine PL-7 alternatives-analysis gap, not a reversal to STAND) in
full. The producer's characterization of that dispute is accurate — it
does not overstate the reviewer's finding as a rejection of REHOME, nor
understate it as settled. `BU-TDD-04` (the hidden seam-confirmation J0 at
this exact call site) is independently re-confirmed CONFIRMED by the
`tdd` reviewer on re-reading that record directly. Declining to adopt the
`tdd` producer's REHOME fold notes while still stating the
dispute-independent J0 explicitly is the correct scope boundary — the
`tdd`-side dispute is out of this package's assigned scope, and the draft
correctly does not silently resolve it either way.

### BU-IMPL-03 -- verdict: CONFIRMED

Independently re-read `.sergeant/workflows/implement/
10-implement-with-tdd/CONTEXT.md` (live) against
`docs/gauntlet/promoted-provenance/implement.md`'s "Adjudication A4"
section: the helper text, trigger, and `BU-P2-053` citation match exactly.
No defect found in the N1 A4 fold, and this pass correctly does not
re-litigate it.

### BU-IMPL-04 -- verdict: NEEDS-REVISION

The substantive content is CONFIRMED: independently checked
`.sergeant/workflows/code-review/CONTEXT.md`,
`workflow.toml`, and all four live stage directories directly —
`code-review` is a real, live, four-stage, `STAND`-settled workflow with
its own `## Authority envelope`, exactly as claimed. The prior
"context composition" wording in `implement/30-review/CONTEXT.md` is
independently confirmed wrong today against `docs/icm/convention.md` §4
rule 1 applied directly to `code-review`'s actual current structure
(pulling a four-stage, review-independence-bearing procedure into one
actor turn would collapse four fresh executions into one and destroy the
`20-30-parallel-review`/`40-aggregate` split's non-contamination
property). This finding does not depend on the disputed `tdd` outcome, as
claimed.

The disposition-modifier column, however, misapplies `FOLD`.
`reference/proposal-icm-r-procedure-authority.md` §5.10 defines `FOLD` as
"Unit becomes context or a helper inside an owning package" — a change of
representation. `BU-IMPL-04`'s actual disposition is a prose correction to
an existing `## Delegation` section that remains exactly what it already
was (a stage-level delegation reference); nothing here becomes a context
or a helper. The package's own driver/PL-4 rung analysis states plainly
that "Package identity and driver STAND" for `implement` itself, and the
same logic applies one level down: a corrected-in-place delegation
description is a `STAND`-with-revision, not a `FOLD`. Compare
`docs/gauntlet/runs/icm-r2/code-review/adjudication-draft.md`'s own
`BU-P2-009`, where `FOLD` is used correctly for content that actually
relocates (a stage-level constraint moving into the workflow-level
`Authority envelope` section) — that precedent does not cover
`BU-IMPL-04`'s case, where nothing relocates. Recommend re-labeling this
row `STAND` (revised in place) before promotion; the destination and
underlying correction are unaffected.

### BU-IMPL-05 -- verdict: CONFIRMED

Independently re-read `.sergeant/workflows/implement/30-review/
CONTEXT.md` (live) against `docs/gauntlet/promoted-provenance/
implement.md`: the helper text and `BU-P2-055` citation match exactly, no
defect in the N1 A4 fold. The J1/J2 split (commit itself mechanical,
message content a local J2 choice) is consistent with
`docs/icm/convention.md` §5 rule 5 (a helper is never itself a place to
hide judgment) — the message-content judgment is correctly surfaced to
the stage's own `## Bounded judgment` section in the draft rather than
left implicit in the helper.

### BU-IMPL-06 -- verdict: CONFIRMED

Independently re-read both live stage `CONTEXT.md` files: the "##
Judgment required" boilerplate is byte-identical across both, confirming
the claimed duplication. `docs/icm/convention.md` §6.1 / ADR 0013
decision 4 do require a named `## Bounded judgment` section on every
actor stage, "always present, even when it is only 'inherits workflow
envelope unchanged.'" The draft's replacement sections in
`draft/10-implement-with-tdd/CONTEXT.md` and `draft/30-review/
CONTEXT.md` independently checked against `@@bounded-judgment`'s J5-J0
definitions (`reference/proposal-icm-r-procedure-authority.md` §6.2-6.7):
the J2/J1/J0 splits drawn are defensible readings of that ladder applied
to this content (e.g. seam confirmation correctly lands at J0 per §6.7's
"authority is missing... do not guess" test, not J2). `FOLD (replace)`
here is a closer fit than in `BU-IMPL-04` — boilerplate prose is being
replaced by a structured context-ladder section, which is at least
adjacent to "becomes context" — but see the note under `BU-IMPL-04`: the
modifier vocabulary in §5.10 was not written with in-place authoring-format
corrections in mind, and a reviewer should not read `FOLD`'s use here as
precedent for using it on every prose edit.

### BU-IMPL-07 -- verdict: CONFIRMED

Independently confirmed by direct read: `.sergeant/workflows/implement/
CONTEXT.md` (live) has no `## Authority envelope` section anywhere in the
file. `docs/icm/convention.md` §6.1 requires one on every workflow
Layer-1 `CONTEXT.md`. The added section in `draft/CONTEXT.md`
independently checked against `code-review`'s own live `## Authority
envelope` for structural parity (Workflow may decide / may not decide /
Human or Captain gates / Decision record) — matches the required shape.
`FOLD (add)` is a reasonable use of the modifier here (new content
becoming part of a context section), consistent with the `BU-P2-009`
precedent noted under `BU-IMPL-04`.

### BU-IMPL-08 -- verdict: CONFIRMED

Independently confirmed by direct `ls .sergeant/workflows/implement/`:
only `10-implement-with-tdd/`, `30-review/`, `CONTEXT.md`, `index.md`,
`workflow.toml` exist — no `provenance.md`. The live `CONTEXT.md`'s "##
Provenance" section does say "See `provenance.md`," which is a genuine
broken self-reference, and the real file is confirmed to be
`docs/gauntlet/promoted-provenance/implement.md` (exists, its citations
independently checked and resolve). The claimed parallel to `code-review`
ICM-R2's own finding #3 ("Broken self-reference," `docs/gauntlet/runs/
icm-r2/code-review/adjudication-draft.md` line ~89) is independently
verified accurate — same defect shape, same fix. As with `BU-IMPL-04`,
`FOLD` is not a precise fit for a pointer-string correction (nothing
becomes context or a helper), but this is immaterial to the correction
itself and does not need a separate action beyond the `BU-IMPL-04` note
already flagging the systemic modifier-vocabulary looseness.

### BU-IMPL-09 -- verdict: NEEDS-REVISION

The core claim is CONFIRMED: independently grepped
`.sergeant/workflows/` for `code-review` and `tdd` references and
confirmed the producer's consumer-graph claims exactly — `code-review` is
named as a direct delegate only by `implement/30-review/CONTEXT.md`;
`worker-mission/20-implement/CONTEXT.md` reaches it only indirectly via
selecting `implement`, and does name `tdd` directly (one of five
selectable disciplines, byte-identical hedge wording to `implement`'s own
pre-revision text). This gives `tdd` two genuinely independent direct
parents and `code-review` one — the producer states this distinction
correctly rather than glossing over it. The filed
`draft/engine-gap-nested-workflow-invocation.md` independently checked
against `docs/icm/record-shapes.md` §5's six-field template: all six
required fields (`behavior`, `source_evidence`, `lower_rungs_attempted`,
`why_each_fails`, `minimum_runtime_capability_required`,
`observable_acceptance_test`) are present, well-evidenced, and not
generic boilerplate reused across entries (each `why_each_fails` value is
specific to that rung's own mechanics, satisfying record-shapes.md §5
rule 3's "identical reasons across rungs is rejection evidence" test).

One rule-2 compliance gap survives independent challenge:
`record-shapes.md` §5 rule 2 requires `lower_rungs_attempted` to "name
actual ladder rungs from §6 (invariant, workflow, stage, actor-stage,
helper, shared context/helper) — not restate the claimed gap in different
words." The filed claim's second entry, "context composition prose
without @@ syntax," is not a distinct ladder rung from its first entry,
"shared context (`@@tdd`...)" — both are the same underlying
representation (pulling reference text into the current actor's turn);
the second entry differs only in whether the pull is done through the
literal `@@name` token or informal prose describing the same intent
without ever mechanically pulling anything in. Read strictly, the second
entry is closer to "no representation was actually attempted, only a
hedge was written" than to a genuine lower-rung representation that was
tried and found wanting — which is arguably rule-2 non-compliant as a
distinct list entry, even though its underlying evidentiary point (the
hedge has stood unaddressed at three call sites since promotion) is real
and independently confirmed by direct grep. The claim's other three
entries ("shared context," "ad-hoc dispatch of a separate Work via `sgt
run`," "workflow-local duplication") are correctly distinct rungs and
directly parallel the canonical worked example in `record-shapes.md` §5
(which itself lists exactly three: shared context, shared helper,
duplicate workflow-local stage). Recommend merging the second entry's
evidentiary content into the first entry's `why_each_fails` value (noting
that the informal pre-revision hedge already demonstrates the same
representational failure without a formal `@@` token) rather than listing
it as a fourth-then-really-third distinct rung. This does not disturb the
claim's compliance with the six-required-fields bar, nor its underlying
conclusion — the claim should not be rejected at lint on this basis, but
should be tightened before Captain's reconcile-and-publish pass.

## What is not in dispute

- Every citation in the producer's draft (`BU-P2-050` through `BU-P2-055`,
  `BU-P3-004`) was independently re-verified against
  `docs/gauntlet/promoted-provenance/implement.md` and the live upstream
  `SKILL.md` reference and is accurate; no fabricated or misquoted
  citation was found anywhere in the draft or its `draft/` content.
- The package's own PL-4 driver/admission-boundary rung
  (`reference/proposal-icm-r-procedure-authority.md` §5.6) is correctly
  re-derived and not disputed by any source read for this review,
  including the `tdd` producer and reviewer records this pass also read.
- The `code-review` delegation defect and its correction are settled and
  independent of the `tdd` dispute's outcome, exactly as the producer
  argues.
- The decision to leave `10-implement-with-tdd`'s delegation prose only
  partially revised, tracking `tdd`'s own unresolved dispute rather than
  adopting either disputed side, is the correct scope boundary and is not
  disturbed by this review.
- The two demoted-and-folded helpers (`BU-IMPL-03`, `BU-IMPL-05`) were
  independently re-checked against the live package and found correctly
  unchanged; no defect in the N1 adjudication A4 fold survives independent
  re-derivation.
- No duplicated or drift-prone content, and no false-pairing assumption,
  was found beyond what the producer itself already surfaced and corrected
  (the identical "context composition today... does not exist yet" hedge
  at three call sites; the identical "## Judgment required" boilerplate
  across both stages).
- Structural claims (no `provenance.md` file; `code-review` and `tdd`
  consumer graphs; the engine-gap claim's six required fields) were
  independently re-checked by direct file read and grep and hold.

## Overall verdict on Final disposition: CONFIRMED (STAND)

The producer's Final disposition — `STAND` at the package level, with
`FOLD`-grain internal restructuring required before promotion — is
independently re-derived and holds. `implement`'s PL-4 rung and driver are
correctly unchallenged by any source this review read, including both
`tdd` records. The `code-review` delegation-mechanism defect is real,
settled, and correctly fixed in full in this draft. The `tdd`-side
partial revision is the correct scope boundary given that dispute's
unresolved state. Two narrow line items — the `FOLD` misapplication on
`BU-IMPL-04` (recommend `STAND`) and the rule-2 entry-merge needed in the
filed engine-gap claim (`BU-IMPL-09`) — should be corrected at Captain's
reconcile-and-publish pass but do not change the package-level verdict,
any destination path, or any substantive finding. Recommend: accept this
draft's `STAND` disposition and its `draft/` content for promotion, with
the two `NEEDS-REVISION` corrections above applied first.
