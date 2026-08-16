# ICM-R3 independent adversarial review: diagnose-bug

Reviewer pass per `reference/proposal-icm-r-procedure-authority.md` §8.11
(Independent adversarial review) and §8.10 (self-check, re-derived
independently rather than trusted). This review has no edit authority over
`.sergeant/workflows/diagnose-bug/` or over the producer's draft at
`docs/gauntlet/runs/icm-r3/diagnose-bug/adjudication-draft.md`; it is a
fresh, separate execution with a review-only contract (ADR 0013 decision 7).

Method: every classification below was independently re-derived against
the package's actual current content
(`.sergeant/workflows/diagnose-bug/{CONTEXT.md,index.md,workflow.toml,*/CONTEXT.md,*/output/README.md}`)
and, where the producer cites an N1 behavior unit, against the underlying
upstream source
(`reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md`) —
not accepted from the producer's own citations or table values.

## Behavior-unit dispositions

### BU-DB-01 -- verdict: NEEDS-REVISION

Re-derivation: the workflow-level trigger text in `CONTEXT.md` ("Trigger")
matches `BU-P2-019`'s trigger clause and the PL-4 rung is correct (checked
independently against the driver/admission-boundary test, proposal §5.6 —
confirmed below under "Overall verdict"). But the J5 citation the producer
builds is weaker than presented. `BU-P2-019`'s source locator in
`docs/gauntlet/promoted-provenance/diagnose-bug.md` reads "front matter
description, lines 3-3." Line 3 of the upstream `SKILL.md` is only the
`description:` front-matter field (the trigger clause). The clause the
producer actually leans on for the J5 claim — "phases may be skipped only
when explicitly justified" — is not on line 3 at all; it is a separate
sentence on line 8, under the `# Diagnosing Bugs` heading, outside front
matter entirely ("A discipline for hard bugs. Skip phases only when
explicitly justified."). `BU-P2-019`'s `statement` field silently splices
two non-contiguous spans (front matter line 3, and body line 8) into one
behavior unit under a single locator that only covers one of them. That is
an unmarked conjunction of two independently-triggerable statements (a
trigger condition, and a phase-skipping discipline) in one record —
exactly what `docs/icm/record-shapes.md` §3 rule 1 forbids ("A record
whose `statement` contains an unmarked conjunction of independently-
triggerable behaviors ... is a violation — split it into separate units").
This is a pre-existing N1 provenance defect, not something introduced by
this ICM-R3 pass, and out of this package's own scope to fix — but §8.11
explicitly requires challenging source fidelity, and the producer's table
cites `BU-P2-019` as if it cleanly supported a J5 characterization without
noting the locator mismatch or the bundling problem. The disposition
(STAND) is still correct on independent re-derivation, but the J5 label
should not be asserted from this citation as-is; either re-derive the J5
claim from the clean, correctly-located text ("Skip phases only when
explicitly justified," SKILL.md line 8) directly, or flag `BU-P2-019` itself
for a corpus-wide provenance fix before citing it as settled support.

### BU-DB-02 -- verdict: CONFIRMED

Re-derived independently against `10-build-feedback-loop/CONTEXT.md`: the
Behavior contract's nine bullets track `BU-P2-021` through `BU-P2-030`
verbatim-in-substance against the upstream Phase 1 text, PL-5 holds (fresh
execution boundary, distinct artifact, independently retryable), and the
J2 delegation (construction-strategy choice, tightening) plus the J0
carve-out (`BU-P2-028`, "if no loop can genuinely be built ... stop ...
and ask") are both actually present in the stage's own Behavior contract,
not invented by the producer. Confirmed.

### BU-DB-03 -- verdict: CONFIRMED

Re-derived against `20-reproduce-and-minimize/CONTEXT.md`. The J5
completion gate the producer cites ("must not proceed past Phase 2 until
... both reproduced and minimized," `BU-P2-036`) is a real governing
prohibition in the stage's own text, correctly distinguished from the J2
delegation over cut-ordering (`BU-P2-033`). PL-5 holds under the same
reimplementation test as BU-DB-02. Confirmed.

### BU-DB-04 -- verdict: CONFIRMED

Re-derived against `30-hypothesize/CONTEXT.md`. The producer's split of
`BU-P2-039` into "delegated ranking judgment" plus "an explicitly named
non-blocking exception" (proceed without the user's re-ranking if they are
away) is a correct, non-generic reading of the stage's actual text ("this
is a cheap checkpoint that should not block progress if the user is
away"). This is also the correct basis for rejecting a Captain-shaped
split for this stage (checked independently below under "Overall
verdict"). Confirmed.

### BU-DB-05 -- verdict: CONFIRMED

Re-derived against `40-instrument/CONTEXT.md`. J2 delegation (tool choice,
tagging) matches the stage's own text; no J5/J0 case is missing here on
independent check — the performance-branch rule (`BU-P2-043`) is itself a
J2-delegated choice of method, not a governing prohibition, and the
producer does not mischaracterize it as one. Confirmed.

### BU-DB-06 -- verdict: CONFIRMED

Re-derived against `50-fix-with-regression-test/CONTEXT.md`. The named
fallback disposition for "no correct seam exists" (`BU-P2-046`, record the
absence as a finding) is present in the stage's own text and correctly
distinguished from a discretionary J2 choice. PL-5/J2 holds. Confirmed.

### BU-DB-07 -- verdict: CONFIRMED

Re-derived against `60-cleanup-and-postmortem/CONTEXT.md`. The closing
checklist (`BU-P2-048`) and the architecture-handoff judgment (`BU-P2-049`)
are both present and correctly classified J2. The "one in-place
correction" the producer defers to BU-DB-10 is the right place for it —
see below. Confirmed.

### BU-DB-08 -- verdict: CONFIRMED

Independently grepped: all six stage `CONTEXT.md` files carry the
identical `## Judgment required` paragraph verbatim and none carries a
`## Bounded judgment` section, a named J2/J1/J0 breakdown, or a completion
boundary in the shape ADR 0013 decision 4 and the proposal §7.3 require.
The producer's remediation (replace boilerplate with named sections,
sourced from the Behavior contract prose each stage already carries) is
the correct minimal fix — an in-place amendment, not a placement change.
Confirmed.

### BU-DB-09 -- verdict: CONFIRMED

Independently read: the workflow-level `CONTEXT.md` has no `## Authority
envelope` section (only Purpose / Trigger / Stages / Notes for reviewers /
Provenance). This is a real gap against proposal §7.2 and ADR 0013 decision
4's requirement. Confirmed. (See also the new finding below, in the same
file, that the producer's table did not surface.)

### BU-DB-10 -- verdict: CONFIRMED

Independently grepped repository-wide (excluding `.git` and
`reference/sergeant-upstream`) for `improve-codebase-architecture`: it
appears only in `60-cleanup-and-postmortem/CONTEXT.md`, the archived
provenance file, and a stale prior draft snapshot under
`docs/gauntlet/runs/n2-run4/`. `skills/` contains only `estate-navigation`,
`grill-with-docs`, `grilling`, `sergeant-help` — none is an architecture
skill. The producer's FOLD disposition (correct the text in place to
describe the actual required behavior — record the finding in the stage's
own `promote` output — rather than naming a nonexistent downstream skill)
is the right call under the ladder's first-honest-rung rule; this is not a
J0 (nothing material is blocked on the missing target) and not an
engine-gap (no new runtime fact is needed). Confirmed.

### BU-DB-11 -- verdict: CONFIRMED

Independently checked: `60-cleanup-and-postmortem/output/README.md`
declares `promote` but names no deterministic finalize step, and
`docs/icm/convention.md`'s own open-questions framing (cited by the
producer) already treats this as a cross-cutting, corpus-wide gap (~30
packages), not specific to this package's placement or authority. Parking
it rather than resolving it package-by-package correctly avoids the
file-shape-mirroring failure `docs/icm/record-shapes.md` §6 rule 4 warns
against. Confirmed.

## Additional finding (not in the producer's table)

### BU-DB-12 (reviewer-added) -- verdict: NEEDS-REVISION

`diagnose-bug/CONTEXT.md`'s own "Provenance" section reads: "See
`provenance.md` for the complete stage-to-behavior-unit mapping and
workflow-level citations." Independently checked:
`find .sergeant/workflows/diagnose-bug -iname "provenance*"` returns
nothing — no `provenance.md` file exists anywhere in this package's tree.
The actual provenance record lives at
`docs/gauntlet/promoted-provenance/diagnose-bug.md`, an entirely different
path that the workflow's own Layer 1 orientation file never names.

This is the identical class of defect the producer itself already caught
and disposed at BU-DB-10 — a reference to a named artifact that does not
exist where the text says it does — but applied to the workflow's own
Layer-1 file rather than a cited upstream skill name. The producer's
"Notes for reviewers" section of `adjudication-draft.md` even names the
precedent for this exact failure class (ICM-R2's `validate-and-ship`
`route-review-findings`, BU-VAS-10) without applying it to this second,
in-package instance. Rungs checked, same reasoning as BU-DB-10: not J0
(nothing material blocks on it — a reader who wants provenance can still
find it via `docs/gauntlet/promoted-provenance/diagnose-bug.md`, which is
independently discoverable and already cited elsewhere in the package's
own `index.md`), not an engine-gap (no new runtime fact required).
Disposition: FOLD — correct the "Provenance" section in `CONTEXT.md` to
point at the real path
(`docs/gauntlet/promoted-provenance/diagnose-bug.md`) or remove the
dangling filename and rely on `index.md`'s existing pointer. This should
be added as a fifth in-place remediation item alongside the producer's
four in "Surviving package design," not treated as a reason to change the
package's placement or Final disposition.

## Overall verdict on Final disposition

**STAND — confirmed**, with two corrections folded into the remediation
list before the package is treated as authority-valid.

Independent re-derivation of the package-level classification, done from
the actual content rather than the producer's own framing:

- **Driver/admission boundary (PL-4 check):** every one of the six stage
  `CONTEXT.md` Behavior contracts is written as instructions to an actor
  operating on an already-known defect (build a loop, minimize, generate
  hypotheses, instrument, fix, close out) — none asks whether a defect
  investigation should exist, none requires live conversational
  continuity (each stage's Inputs table names only L1 orientation on
  first entry or the immediately preceding stage's L4 output, verified by
  direct read of all six Inputs tables), and the two user-facing
  checkpoints (`10-build-feedback-loop`'s J0 escalation, `30-hypothesize`'s
  advisory re-rank) are both bounded questions raised during an
  already-admitted execution, not the package's primary product. This
  independently satisfies PL-4 over PL-2/PL-3 under proposal §5.4-5.6's
  discriminators — not merely re-asserting the producer's own conclusion.
- **PL-5 (stage boundary) for all six stages:** independently applying the
  §5.7 reimplementation test to each stage — distinct artifact,
  independent retry unit, a plausible different actor/authority envelope
  at each boundary — none of the six looks like a heading, script, or
  helper masquerading as a stage. No stage/helper-boundary dispute.
- **No missing J0 case found** beyond the one the producer already
  surfaced (`10-build-feedback-loop`'s "no loop can be built"): the
  seam-absence fallback (Phase 5) and the architecture-handoff judgment
  (Phase 6) are both genuinely J2-delegable, not risk-changing enough to
  require escalation, and the package does not itself perform any
  destructive, irreversible, or promotion-bearing action beyond writing
  Work-branch evidence and one `promote`-disposition output.
- **No false pairing assumption and no unjustified engine-gap claim**
  found anywhere in the package — none is made.
- **Package identity:** `name: diagnose-bug` agrees across `index.md`
  front matter, the containing directory, and `workflow.toml`; no
  collision or mismatch found.

The producer's four in-place-amendment items (Bounded-judgment sections,
Authority envelope, the `/improve-codebase-architecture` FOLD) are
correct and necessary. This review adds a fifth: the dangling
`provenance.md` self-reference in `CONTEXT.md` (BU-DB-12 above). None of
the five changes the package's placement, its PL-4/PL-5 rungs, or its
STAND disposition — they are all in-place content corrections, so ADR
0013 decision 6's REHOME/SPLIT/HARVEST draft-and-rehome step is correctly
not triggered, matching the producer's own conclusion.

The producer's "Authority-valid: not yet" self-assessment is correct and,
on this review, should explicitly include the newly found BU-DB-12 item
before the package can be called authority-valid under proposal §9.1
claim 3.
