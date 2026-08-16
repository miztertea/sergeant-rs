# Independent adversarial review: package `tdd` (ICM-R3)

Reviewer position per `docs/adr/0013-icm-r0-owner-rulings.md` decision 7 and
`reference/proposal-icm-r-procedure-authority.md` §8.11: fresh execution,
explicit inputs (this record, the producer's draft, the live package under
`.sergeant/workflows/tdd/`, its two current delegators, `docs/icm/
convention.md`, `docs/icm/record-shapes.md`), review-only contract, no edit
authority over `docs/gauntlet/runs/icm-r3/tdd/adjudication-draft.md` or the
live package. Classification below is independently re-derived against the
package's actual current content, not read off the producer's own citations.

## Summary of verdict

The producer's behavior-unit-level content extraction and citation trail
are sound and are CONFIRMED. The producer's core placement conclusion —
`tdd` is PL-3 and REHOMEs wholly into `.sergeant/common/contexts/` — rests
on an incomplete alternatives analysis: it never evaluates the PL-7
engine-gap alternative that `docs/icm/record-shapes.md` §5's own canonical
worked example describes, a scenario nearly identical to `tdd`'s actual
current delegation pattern. That gap, plus two weaker supporting arguments
in "Driver and admission boundary," are enough to move the package's
**Final disposition from REHOME to DISPUTED** pending a corrected
alternatives analysis — not a reversal to STAND, and not a confirmation of
REHOME as drafted.

## Behavior-unit dispositions

### BU-TDD-01 — verdict: NEEDS-REVISION

Source citation (`BU-P2-104`, `BU-P2-105` against `CONTEXT.md`/`index.md`'s
trigger and purpose) checks out against `docs/gauntlet/promoted-provenance/
tdd.md` and the live `CONTEXT.md`/`index.md`. The PL-3 rung and REHOME
destination inherit directly from the package-level "Driver and admission
boundary" argument disputed below (see Overall verdict). If that argument
is corrected and lands on a different placement, this unit's destination
changes with it — the unit itself is not independently wrong, but it is not
independently settled either.

### BU-TDD-02 — verdict: CONFIRMED

Independently re-derived against the live `00-agree-seams/CONTEXT.md`
directly (not the producer's quotation of it): "no test is written at an
unconfirmed seam" and the literal elicitation question both appear verbatim.
Classifying this as a **caller-facing J0**, not a J2 the loading stage may
skip, is correct under either placement outcome (PL-3 shared context or a
hypothetical PL-4/PL-7 nested-workflow form) — the seam-confirmation stop
is a property of the technique itself, not of which surface hosts it. This
is the strongest single piece of authority-preservation work in the draft:
it is explicit that REHOME must not silently narrow this requirement
(proposal §4.6/§6.9), and the drafted `draft/.sergeant/common/contexts/
tdd.md` "What this context contributes when loaded inside a stage" section
does in fact restate it as a J0 the caller must honor. Confirmed as
correct regardless of how the Overall verdict below resolves.

### BU-TDD-03 — verdict: CONFIRMED

Independently re-derived against `10-red-green-cycle/CONTEXT.md`: the three
cited behaviors (vertical slicing, red-before-green, one-slice-at-a-time)
are quoted accurately and the J2/J1 split (concrete seam/idiom choice is
J2; ordering of equivalent confirmed seams is J1) is a reasonable read of
`bounded-judgment.md` §6.5/§6.6 applied to this content. Same caveat as
BU-TDD-01: the *destination* (shared context vs. stage) is downstream of
the disputed package-level placement call, not a defect in the unit's own
classification.

### BU-TDD-04 — verdict: CONFIRMED

Independently checked both delegating stages
(`.sergeant/workflows/implement/10-implement-with-tdd/CONTEXT.md`,
`.sergeant/workflows/worker-mission/20-implement/CONTEXT.md`): neither
restates the seam-confirmation J0 requirement in its own Bounded-judgment-
adjacent content ("Judgment required" section is boilerplate in both; the
"Delegation" section names only the outcome, not the authority it
inherits). This is a real hidden-dependency finding under `docs/icm/
record-shapes.md` §1a rule 4 / `convention.md` §1a rule 1, correctly scoped
out of this pass (fold notes only, no live edit to `implement`/
`worker-mission`) per the J0 guardrail this package's own dispatch
contract cites. Confirmed as a genuine finding independent of how the
package-level placement dispute resolves — the hidden-dependency problem
exists whether `tdd`'s content ends up in a shared context or stays a
workflow that gets a real invocation mechanism later.

### BU-TDD-05 — verdict: CONFIRMED

Independently grepped `.sergeant/common/` — only `bounded-judgment.md`
exists, confirming the producer's claim that no `@@test-quality` context
was ever materialized despite `tdd/CONTEXT.md`'s own reviewer note
promising it ("16 units land in the `test-quality` shared context, not in
this workflow"). Drafting the content while explicitly declining to wire
its other four named consumers (`diagnose-bug`, `prototype`, `implement`,
`deepen-module`) is correctly scoped — building those references would be
exactly the cross-package scope violation the producer's own "Alternatives
considered" section correctly identifies and rejects. Confirmed.

### BU-TDD-06 — verdict: CONFIRMED

Independently checked: every stage `CONTEXT.md` in the live package uses
"## Judgment required" boilerplate, not the "## Bounded judgment" heading
with named J2/J1/J0 subsections that `docs/icm/convention.md` §6.1 /
ADR 0013 decision 4 require. Treating this as moot under REHOME (a shared
context has no per-stage Bounded-judgment section of its own) is correct
*given* REHOME is the right disposition — see Overall verdict. If the
package-level placement instead resolves toward retaining `tdd` as a
workflow (in any form), this format gap would need to be fixed rather than
mooted, and this unit's disposition would need to change from "moot" to an
actual authoring-format remediation.

## Overall verdict on Final disposition: DISPUTED

The producer's "Driver and admission boundary" section makes four
arguments for PL-3. Re-derived independently against the same primary
sources:

**Argument 2 (two current consumers already delegate to it) is real, but
its conclusion is not the only one the evidence supports, and the producer
never tests the alternative.** `docs/icm/record-shapes.md` §5's own
canonical engine-gap worked example is:

> "Two workflows both need to invoke a shared 'run adversarial review'
> procedure with its own retry/measurement, not just shared text."

`tdd`'s actual current situation is structurally the same shape: `implement/
10-implement-with-tdd` and `worker-mission/20-implement` both need to
invoke `tdd`'s red-green-cycle discipline — a discipline that, *as
currently packaged*, has its own retry semantics (the cycle repeats,
per-seam, until the vertical slice is done) and its own fresh-execution
checkpoint boundaries (`00-agree-seams` and `10-red-green-cycle` are
distinct stage executions today). Both delegating stages' own text says
"context composition today — see `docs/icm/convention.md` §4 on `@@name`
versus true nested-workflow invocation, **which does not exist yet**" —
language already present in the live package, independently confirmed by
reading both files directly. `convention.md` §4 rule 1 is explicit that
using `@@name` to imply "and then run that other procedure as a
sub-workflow" is itself "a violation of scope" and that such intent "must
be recorded as an engine-gap claim (`record-shapes.md` §5), **not smuggled
through a context reference**."

The producer's REHOME disposition does exactly what §4 rule 1 warns
against: it takes an already-ambiguous "context composition today" note
and makes it literally true by converting `tdd` into a `@@tdd` shared
context — silently resolving the ambiguity toward "this was always meant
to be context composition" without ever writing the engine-gap record
`record-shapes.md` §5 supplies a template for, and without recording it in
"Alternatives considered" as a rejected option with a `why_each_fails`
rationale specific to a shared-context representation's actual mechanics
(the record-shapes.md worked example already states that rationale nearly
verbatim: "Pulls text into the current actor's turn; produces no
independent durable checkpoint, retry, or measurement — the parent's
single stage absorbs an unbounded sub-procedure"). That is precisely the
loss the REHOME disposition produces here: two currently-distinct fresh
executions (`00-agree-seams`, `10-red-green-cycle`) collapse into whichever
single stage of `implement` or `worker-mission` loads `@@tdd`, and per-seam
retry/measurement that exists today as separate stage attempts disappears
into one execution's private judgment.

This is not itself proof that `tdd` should file for PL-7 — an engine-gap
claim is a high bar (§6.7's six required fields, `record-shapes.md` §5's
lint-reject-on-missing-field rule) and this reviewer is not asserting one
should be filed. The finding is narrower: **the producer's "Alternatives
considered" section is required to weigh this alternative and did not.**
§5.9 states PL-7 "is evaluated after" the lower rungs, not skipped; the
record-shapes.md classification-record rule requires `alternatives_
considered` to be non-empty "for every unit carrying a workflow or stage
boundary" — `BU-TDD-01` through `BU-TDD-03` all carry exactly that
boundary (they currently live inside two real stages with real fresh
executions) and none of their entries, nor the package-level "Alternatives
considered" section, name or reject the engine-gap alternative.

**Argument 1 (the proposal's own §5.5 lists "a TDD technique" first) is
real but not dispositive on its own.** §5.5's example list is illustrative
prose describing PL-3 in the abstract ("a reusable reasoning or operating
technique that an actor applies inside a Captain interaction or a workflow
stage, without owning a complete durable Work lifecycle"). It is legitimate
evidence that a generic "TDD technique" belongs at PL-3, but the producer
treats it as settling *this specific package's* placement without
independently testing §5.5's own discriminator against §5.6 first. §5.6's
PL-4 question is: "Given an already-defined intent, repositories,
constraints, and expected outcome, can Sergeant execute this procedure
durably from admission to a terminal result whether or not the Captain
remains present?" A user-typed intent like "add retry backoff to the
worker, test-first" plausibly satisfies exactly that test: it absorbs a
specific intent (unlike a truly generic technique, which does not vary by
target), and the package's own two stages already behave like a durable,
checkpointed, admission-to-terminal-result procedure today. The producer's
citation of §5.5's example list is worth keeping in the record, but it is
weighed as if it closes the question rather than as one data point to be
tested against §5.6, and §5.6 was not actually applied to the package's
current stage content.

**Argument 3 ("does the same thing every time" fails the §2a test) is a
misapplication of that test as written.** §2a's actual test is "would a
human type `sgt run '<intent>' --workflow X`?" and its failure condition is
"if the package cannot absorb an intent — if it does the same thing every
time." "Does the same procedural discipline every time while absorbing a
different target intent" is not the failure condition §2a names — it is
the ordinary shape of *every* admitted workflow: `diagnose-bug` always runs
reproduce/isolate/prove/remediate regardless of which bug is being
diagnosed, which does not disqualify it from PL-4. `tdd` absorbs a
different concrete target (whatever feature or fix is being implemented)
on every invocation, exactly like any other workflow that fixes its
procedure but varies its content. This argument as written does not
distinguish `tdd` from packages the producer's own citation trail treats as
correctly PL-4, and should be dropped or replaced with a narrower argument
that actually discriminates.

**Argument 4 ("`shared` is the correct modifier, not `local`") conflates
the shared/local modifier axis with the placement-rung axis.** Proposal
§5.10 states plainly that "Shared/local is another modifier, not a rung."
Two independent consumers sharing an identical contract is genuine evidence
that *whatever rung `tdd` lands on*, it should be represented once and
reused rather than duplicated per-consumer — but it does not by itself
argue for PL-3 over PL-4/PL-7. A shared PL-4 workflow invoked by two
different callers (once true nested-workflow invocation exists) satisfies
"shared, not local" equally well. This argument is evidence for
deduplication, not for the specific rung chosen.

**The `promote`-disposition evidence in the package's own text does not
support the producer's reading.** The producer's point 1 quotes
`10-red-green-cycle/output/README.md`'s curation note ("the workflow has no
dedicated finalize step... Disposition here is applied by human review at
merge time, not mechanically") as evidence that the stage "produces no
terminal, independently meaningful Work outcome." Read directly, that file
states the opposite for the actual question at issue: the artifact's
**Disposition is `promote`** — "This is a workflow deliverable: it survives
into the merge under the finalize policy... a `promote` artifact is kept
explicitly." A `promote` disposition is evidence *for* a terminal,
merge-surviving outcome, not against one. What the curation note actually
flags is a narrower authoring gap — no dedicated finalize *stage* curates
that promotion mechanically, so a human does it at merge time instead. That
is a real gap worth recording (arguably itself a candidate PL-5/PL-6
finding: does `tdd` need a finalize step, the way other two-stage-plus
workflows have one?), but it is not evidence that no terminal outcome
exists, and citing it as such is a misreading that should be corrected
before this argument is reused.

## What is not in dispute

- Every behavior unit's source citation was independently re-verified
  against the live package and `docs/gauntlet/promoted-provenance/tdd.md`
  and is accurate. No fabricated or misquoted citation was found.
- The seam-confirmation J0 preservation work (BU-TDD-02) and the
  hidden-dependency finding on the two delegating stages (BU-TDD-04) are
  sound and valuable independent of how the placement dispute resolves.
- `test-quality.md`'s scope boundary (BU-TDD-05) is correctly drawn.
- No duplicated or drift-prone content, no false pairing assumption, and no
  package-identity/naming problem was found in the draft.
- Structural claims (no other package references `tdd` beyond the two
  known delegators; `@@tdd`/`@@test-quality` do not collide with an
  existing name) were independently re-checked by direct grep and hold.

## Recommendation

Do not promote REHOME as drafted. Before Captain's reconcile-and-publish
pass (§8.12), the producer (or a corrected revision) should add a
genuine PL-7 alternatives entry — either a real engine-gap record following
`record-shapes.md` §5's template (if the retry/measurement/fresh-execution
loss identified above is judged to matter enough to justify one) or an
explicit, mechanics-specific rejection of that alternative (if it is judged
not to matter, e.g. because the per-seam checkpoint granularity `tdd`
currently has is itself over-engineered for a 1-2 cycle technique and its
loss is acceptable) — and should drop or replace Argument 3, and correct
the `promote`-disposition misreading in Argument 1's supporting text. The
seam-confirmation authority-preservation work and the delegator
hidden-dependency finding should be carried forward into whatever revision
follows regardless of which way the placement question is ultimately
resolved.
