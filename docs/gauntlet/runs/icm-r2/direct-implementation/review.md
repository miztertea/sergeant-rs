# Package adjudication review: direct-implementation

Independent adversarial review, ICM-R2 pilot (`docs/adr/0013-icm-r0-owner-rulings.md`
decision 7). Reviewer is a separate actor position from the producer,
fresh execution, explicit inputs (`reference/proposal-icm-r-procedure-authority.md`
§8.11, `docs/adr/0013`, `docs/icm/record-shapes.md` §6, the producer's
draft at `docs/gauntlet/runs/icm-r2/direct-implementation/adjudication-draft.md`
and `draft/`, and the live package under
`.sergeant/workflows/direct-implementation/`), review-only contract, no
edit authority over the producer's draft or the live package. Every
citation below was independently re-derived by reading the live package,
`AGENTS.md`, `validate-and-ship`'s directly-invoked-entry stages, and the
frozen upstream sources — not accepted from the producer's own citations.

## BU-P1-007 -- verdict: CONFIRMED

Re-derived: `reference/sergeant-upstream/AGENTS.md` L22-23 states the
direct-mode trigger. Current `AGENTS.md` "When NOT to use `sgt`"
(L169-173) states the same trigger — "the user explicitly asks to
work in-session ... and one repository owns the complete outcome" —
in different words but the same substance. PL-1/J5, ABSORBED, holds.

## BU-P1-016 -- verdict: NEEDS-REVISION

Re-derived: upstream source (`AGENTS.md` L39-41) is two distinct
prohibitions — (a) never use direct mode across several repositories in
one checkout, and (b) never use direct mode to bypass repository
instructions, task ownership, review independence, or shipping gates.
Current `AGENTS.md` covers (a) only by inference (the single-repository
requirement in "When NOT to use `sgt`" L171, and the inverse framing of
dispatch being for cross-repo work, L166-168) and covers the
shipping-gate half of (b) explicitly (Guardrails, L202-205: "Standing
authorization ... never extends to skipping the shipping gate"). But
"task ownership" and "review independence" specifically are not stated
anywhere in current `AGENTS.md` outside the sergeant-rs-scoped L187
("no mode waives tests, review, or the shipping gate") that the producer
itself already flags as too narrowly scoped for BU-P1-107. The producer's
ABSORBED disposition treats the whole unit as covered by a citation that
in fact only covers half of it, and covers the other half by inference
rather than an explicit statement. This does not change the underlying
conclusion — the surviving fragment ("task ownership, review
independence never waived by direct mode") is the same general-restatement
gap already identified for BU-P1-107 — but it should be dispositioned as
**FOLD** into the same `AGENTS-md-fold.md` addition proposed for
BU-P1-107, not ABSORBED outright, so the fold text actually names task
ownership and review independence rather than leaving that half of the
upstream behavior uncited anywhere in the destination.

## BU-P1-107 -- verdict: CONFIRMED

Re-derived: `reference/sergeant-upstream/docs/what-is-sergeant.md` L62-66
states direct mode "still requires a task, TDD, repository-native checks,
independent review, shipping validation, and handoff" — a general claim.
Current `AGENTS.md` states the equivalent claim only under "Working on
sergeant-rs itself" (L179-187), scoped to this repo's own code. The
producer's FOLD-not-ABSORBED distinction is correct and the proposed
`AGENTS-md-fold.md` text is a faithful, general restatement. Holds.

## BU-P8-055 -- verdict: NEEDS-REVISION (rung, not disposition)

Re-derived: this is the eight-step enumeration from
`docs/using-sergeant.md` L21-28. RETIRE is the right disposition — every
constituent step is separately dispositioned elsewhere in the table, and
nothing needs the ordered-list structure to survive. But the cited rung,
**PL-6 ("mechanism enumeration")**, misapplies §5.8: PL-6 answers "is
this repeatable machinery whose output follows mechanically from declared
inputs, with no substantive judgment in its invocation" (CLI verbs,
execute stages, helpers). An ordered prose checklist of judgment-bearing
steps is not machinery in that sense — it is documentation-level
packaging of already-classified behavior, which is squarely §5.2 PL-0's
own language: "the source mechanism is a historical implementation whose
policy has been superseded... [r]ehome any surviving policy to its actual
owner." The disposition (RETIRE) is right; the rung citation should read
PL-0, not PL-6.

## BU-P1-008 -- verdict: CONFIRMED

Independently grepped `AGENTS.md` and `docs/DEVELOPMENT.md` for
`sgt-context`, `td context`: zero hits. Grepped `reference/`: both terms
exist only in the frozen upstream corpus
(`reference/sergeant-upstream/{AGENTS.md,docs/using-sergeant.md,bin/sgt-context,...}`).
The cited mechanism is genuinely obsolete in the current product. PL-0
rung is correct here (and is the producer's only row that correctly uses
PL-0 for an obsolete-mechanism case — see the BU-P1-010/011/012/P8-058/013/014
finding below for the inconsistency this creates). "Load context before
mutating" is genuinely already owned by `AGENTS.md` Standard workflow
loop step 1 and `validate-and-ship`'s `00-check-scope`. ABSORBED holds.

## BU-P1-009 -- verdict: CONFIRMED

Re-derived: `AGENTS.md` Standard workflow loop step 2 (L76-83) currently
reconciles running work only for the dispatch path ("reuse or resume a
matching Work item"), not for a Captain about to choose direct mode. The
producer's FOLD (extend step 2 to state it applies equally before direct,
in-session implementation) is the right shape and the draft
`AGENTS-md-fold.md` text does this. PL-2/J2 (Captain judgment, checked
once before committing to a mode) is a defensible placement; PL-3's
"reusable reconciliation method" example is a plausible alternative
reading but the producer's PL-2 framing (a one-time Captain-level check,
not a method re-invoked by multiple stages) is the better fit and I do
not dispute it.

## BU-P8-056 -- verdict: CONFIRMED

Re-derived: `docs/using-sergeant.md` L23 states the identical
worktree/worker-reconciliation step as `AGENTS.md` L26-27 (BU-P1-009) —
genuinely the same behavior from Conflict X16's two overlapping upstream
sources, not two distinct behaviors. FOLD to the same destination as
BU-P1-009, not a separate row's destination, is correct.

## BU-P1-010 -- verdict: NEEDS-REVISION

Re-derived: `AGENTS.md` L28-29 upstream text bundles two things — (a)
"claim or create the owning td task," and (b) "implement test-driven-first
in the requested checkout or an isolated worktree." Read
`validate-and-ship/10-do-the-work/CONTEXT.md` in full: its behavior
contract (`BU-P2-060`, `BU-P2-061`) covers isolating and committing only
the task's own changes on a feature branch — it does **not** state or
imply TDD-first sequencing, and it does not claim/create any task record
(there is no `td`-equivalent "claim a task" step anywhere in
`validate-and-ship`'s directly-invoked entry). The producer's own
rationale correctly routes the TDD half to the separately-admitted `tdd`
workflow (confirmed: `.sergeant/workflows/tdd/` exists,
`AGENTS.md` L219 names it). But the "claim or create the owning task"
half is left unaddressed — it cites `10-do-the-work` for a claim that
stage does not make, and it is not obviously ABSORBED anywhere else
either, since `td` itself is the same obsolete mechanism already
retired in BU-P1-008's disposition (current product has no equivalent
"claim a task" step inside direct/task-first execution — a Work item is
the closest analog, and direct mode by definition has no separate Work
item). This fragment is closer to **PL-0 (obsolete mechanism, no
surviving policy to rehome — direct mode has nothing analogous to
"claim a td task")** than to a clean ABSORBED-into-`10-do-the-work`.
Also see the rung finding below: even setting the citation-accuracy point
aside, the row's PL-4 tag for an ABSORBED unit is itself inconsistent with
how BU-P1-008 was ruled.

## BU-P1-011 -- verdict: CONFIRMED

Re-derived: `validate-and-ship/10-do-the-work/CONTEXT.md`'s `BU-P2-061`
states, near-verbatim, "if the user is on the repository's default
branch, a feature branch must be created first" — matching upstream
`AGENTS.md` L30-31's "never edit a default branch ... create or reuse the
owning feature branch." Direct, accurate citation. ABSORBED holds on the
merits (see rung finding below for the PL-4 vs PL-0 citation issue, which
applies to this row too).

## BU-P1-012 -- verdict: CONFIRMED

Re-derived: `validate-and-ship/CONTEXT.md`'s own Purpose line states it
is "the single final shipping boundary" and this package's own
`05-shipping-gate` delegates its outcome to it. The behavior ("no mode
waives validation/review/gate") is genuinely already owned by the
delegation target's identity, not merely by analogy. ABSORBED holds on
the merits.

## BU-P8-058 -- verdict: CONFIRMED

Re-derived: `validate-and-ship/CONTEXT.md`'s Trigger line states
"Implementation, native tests, lint and independent review are complete
and the coordinator has reached the approved shipping boundary" —
matching `docs/using-sergeant.md` L26's "run the final shipping gate only
at the approved shipping boundary." ABSORBED holds on the merits.

## BU-P1-013 -- verdict: CONFIRMED

Re-derived: `validate-and-ship/60-close-out/CONTEXT.md` (`BU-P2-086`
through `BU-P2-097`) fully covers PR-open, CI, review-thread, and
merge-authorization handling through to a terminal outcome — a closer and
more detailed match than the upstream unit itself. ABSORBED holds on the
merits.

## BU-P1-014 -- verdict: NEEDS-REVISION

Re-derived: upstream `AGENTS.md` L36 is "record handoff, PR, merge,
**deployment**, and **cleanup** outcomes." `60-close-out`'s folded helper
(`BU-P8-089`, `BU-P7-104`) durably logs ownership-transfer/handover
events, and the stage's main contract (`BU-P2-086`-`BU-P2-097`) covers
PR/merge/CI state. Grepped both `50-reconcile-custody/CONTEXT.md` and
`60-close-out/CONTEXT.md` for "deploy" and "cleanup": zero hits in
either. The "deployment and cleanup outcomes" half of this unit is not
actually stated in the cited destination. This is very likely still a
correct ABSORBED/RETIRE outcome in substance — the current product has no
deployment concept anywhere else either, so "deployment outcomes" is
plausibly obsolete framing rather than a real gap — but the producer's
citation overstates what `60-close-out` actually owns, and should either
narrow the Destination text to "handoff/PR/merge outcomes" (with
deployment/cleanup separately noted as obsolete, PL-0) or show a citation
that actually covers them. Also affected by the rung finding below.

## Cross-cutting finding: PL rung mislabeled on the ABSORBED rows (BU-P1-010, BU-P1-011, BU-P1-012, BU-P8-058, BU-P1-013, BU-P1-014)

`reference/proposal-icm-r-procedure-authority.md` §5.2 (PL-0) gives, as
its own literal fourth example, "duplicate shipping instructions already
owned by validate-and-ship" — the exact shape of every one of these six
rows. The table instead cites `PL-4` ("already-admitted workflow") for
all six ABSORBED rows, while correctly citing `PL-0` for BU-P1-008 (also
ABSORBED, also a duplicate-of-an-existing-surface case). That is an
internal inconsistency in the producer's own table, not just a citation
style choice: two rows adjudicated the identical situation ("this
behavior already lives in an admitted surface") to two different rungs.
Per the ladder's own worked example, all six should read PL-0, matching
BU-P1-008. This does not change any row's disposition (ABSORBED is still
correct on the merits for all six, per the CONFIRMED/NEEDS-REVISION notes
above) or the package's Final disposition, but it is a real rung-order
defect the independent reviewer is specifically charged with catching
(§8.11 "rung order").

## Cross-cutting finding: unrecorded engine-gap assumption inside the live package's own `05-shipping-gate` (confirms, does not weaken, the HARVEST case)

`05-shipping-gate/CONTEXT.md`'s own "Delegation" section states its
outcome "is produced by running **validate-and-ship** to its own
completion," citing `docs/icm/convention.md` §4 on `@@name` versus true
nested-workflow invocation. Read `convention.md` §4 rule 1 in full: it
states plainly that `@@name` is context composition only, that using a
context reference to imply "run that other procedure as a sub-workflow"
is a scope violation, and that true nested workflows "do not exist yet"
(§7.7) — any such intent must be recorded as a formal engine-gap claim
(`record-shapes.md` §5), not assumed. The live stage's own text describes
exactly the not-yet-existing capability (fully running one workflow to
completion from inside another's stage) without either an `@@name`
reference or a recorded engine-gap claim. This is independent
confirmation — beyond the PL-4 self-contradiction the producer already
found in the package's own trigger — that the live package as packaged
rests on a capability the engine does not currently provide. It does not
change the Final disposition; it strengthens it. Worth naming explicitly
in the record rather than left implicit, since the producer's Alternatives
Considered section makes the identical point only about the rejected
SPLIT alternative, not about the live package's actual current content.

## Self-check items independently re-verified

- **Completeness.** Independently enumerated every behavior unit cited
  across the package's own files (`_config/standing-constraints.md` and
  all five stage `CONTEXT.md` files): BU-P1-007, 016, 107, P8-055, P1-008,
  009, P8-056, 010, 011, 012, P8-058, 013, 014 — thirteen units, matching
  the table exactly. No unit is missing from the disposition table.
- **`## Bounded judgment` heading gap.** Independently confirmed: all five
  stage `CONTEXT.md` files use `## Judgment required`, not the
  `## Bounded judgment` heading with named J2/J1/J0 subsections that
  `docs/icm/convention.md` §6 (L425) and `docs/adr/0013` decision 4
  require of every actor stage. The producer's own validation evidence
  already flags this as immaterial to the HARVEST verdict but worth a
  corpus-wide check — confirmed accurate and appropriately scoped.
- **Missing `provenance.md`.** Independently confirmed: `find
  .sergeant/workflows/direct-implementation -type f` lists exactly eight
  files (`CONTEXT.md`, `index.md`, `workflow.toml`,
  `_config/standing-constraints.md`, five stage `CONTEXT.md`/`output/README.md`
  pairs), none named `provenance.md`, despite both `CONTEXT.md` and
  `workflow.toml` referencing one. Confirmed as a real gap, correctly
  flagged as noted-not-fabricated.
- **Directory/`workflow.toml` stage-order agreement.** Independently
  confirmed: `ls -d .sergeant/workflows/direct-implementation/*/`
  (excluding `_config/`) lists `01-load-task-context`,
  `03-claim-and-implement`, `04-validate`, `05-shipping-gate`,
  `06-pr-and-merge` in that lexical order, matching `workflow.toml`'s
  `stages` array exactly. No violation of `docs/icm/convention.md` §1
  rule 4.
- **PL-4 self-contradiction argument (Driver and admission boundary
  section).** Re-derived independently from the package's own
  `CONTEXT.md` alone, without relying on the pilot dispatch instruction's
  framing. PL-4 (§5.6) asks whether Sergeant can execute the procedure
  "durably from admission to a terminal result whether or not the Captain
  remains present." The package's own Trigger is "the user explicitly
  asks to work in this session" — i.e., its entire premise is that the
  Captain/session *does* remain present and *is* the executing party.
  The only currently-existing invocation path (`sgt run --workflow
  direct-implementation`) requires Work admission and dispatch, the
  opposite condition. The argument holds under independent
  re-derivation; it is not merely restating the pilot's own hint. The
  cross-cutting `05-shipping-gate` finding above independently
  corroborates it from a different angle (an unjustified engine-gap
  assumption inside the same package).
- **REHOME-to-new-skill alternative.** Independently checked: after each
  behavior unit's actual destination is traced (`AGENTS.md`'s routing
  section plus `validate-and-ship`'s `00-check-scope`/`10-do-the-work`/
  `60-close-out`), no unit requires a dedicated new skill body. The
  rejection is sound.
- **SPLIT alternative.** Independently checked against
  `docs/icm/convention.md` §4/§7.7: true nested-workflow invocation does
  not exist in the engine today; the rejection (inventing one here would
  be an unjustified PL-7 claim) is correct and squarely on point per
  §8.11's "unjustified engine gaps" checklist item.

## Overall verdict on Final disposition

**HARVEST — CONFIRMED**, with corrections. Every behavior unit
independently re-traces to an existing surface (`AGENTS.md`'s routing
section, `AGENTS.md` Standard workflow loop step 2, or
`validate-and-ship`'s directly-invoked entry) or to genuinely obsolete
upstream mechanism (`sgt-context`/`td`), exactly as the producer found.
The package's own PL-4 self-contradiction (its trigger names the
condition under which the only path that runs it — dispatch — is not
used) independently re-derives and is further corroborated by an
unrecorded engine-gap assumption inside the live `05-shipping-gate`
stage itself. No unit earns a new Captain skill, actor skill, workflow,
or stage.

Before Captain's reconcile-and-publish pass accepts this record, the
following corrections from this review should be merged in:

1. BU-P1-016 and BU-P1-014's destination citations should be narrowed or
   supplemented so they actually name the parts of the upstream behavior
   ("task ownership, review independence" for BU-P1-016;
   "deployment, cleanup" for BU-P1-014) that are not literally present in
   the cited destination text, rather than leaving them implicitly
   covered.
2. BU-P1-010's "claim or create the owning task" fragment should be
   re-classified PL-0 (obsolete `td` mechanism, no current analog inside
   direct-mode execution) rather than folded wholesale into
   `10-do-the-work`, which does not state it.
3. The rung column on BU-P1-010, BU-P1-011, BU-P1-012, BU-P8-058,
   BU-P1-013, and BU-P1-014 should read `PL-0`, not `PL-4`, per §5.2's
   own "duplicate shipping instructions already owned by
   validate-and-ship" example and for consistency with BU-P1-008's
   correct PL-0 citation in the same table.
4. BU-P8-055's rung should read `PL-0`, not `PL-6`.

None of these four corrections changes any row's disposition or the
package's Final disposition. They are rung-citation and destination-text
precision fixes, not substantive disagreements with the producer's
classification work.
