# Package adjudication review: triage

Independent adversarial review of `docs/gauntlet/runs/icm-r3/triage/
adjudication-draft.md`, per `reference/proposal-icm-r-procedure-authority.md`
§8.11 and `docs/adr/0013-icm-r0-owner-rulings.md` decision 7 (a later,
fresh, review-only execution in the same workflow qualifies as
independent — this review has no edit authority over the live package or
the producer's draft). Checklist applied: source fidelity, rung order (PL
and J), Captain/workflow boundary, stage/helper boundary, authority grants
and missing J0 cases, package identity/naming, duplicated or drift-prone
content, false pairing assumptions, unjustified engine gaps. Re-derivation
was against the live package content
(`.sergeant/workflows/triage/{workflow.toml,CONTEXT.md,index.md,10-gather-
context,20-verify,30-recommend,40-grill-if-underspecified,50-apply-outcome}`)
and the original source
(`reference/sergeant-upstream/.agents/skills/triage/SKILL.md`), not from
the producer's own citations.

## Headline finding not in the producer's draft

The producer's self-check verified that `workflow.toml`'s stage order
agrees with the on-disk directory listing (`docs/icm/convention.md` §1
rule 4) but never checked that stage order against the **source's own
step order** — which is exactly what an independent §8.11 pass is for.

Source (`reference/sergeant-upstream/.agents/skills/triage/SKILL.md`,
"Triage a specific issue or PR"): step 1 Gather context, **step 2
Recommend** ("Tell the maintainer your category and state recommendation
... Wait for direction," line 72), **step 3 Verify the claim** ("Before
any grilling, check that the claim holds up," line 74), step 4 Grill, step
5 Apply outcome. The provenance file itself preserves this: `BU-P3-066`
(recommend) cites source line 72, `BU-P3-067` (verify) cites source line
74 — recommend before verify, in the source's own line order.

Live package (`workflow.toml`): `stages = ["10-gather-context",
"20-verify", "30-recommend", "40-grill-if-underspecified",
"50-apply-outcome"]` — **verify is stage 20, recommend is stage 30**,
transposed from the source. This is not merely a labeling slip: `20-
verify/CONTEXT.md`'s own Behavior contract trigger clause reads "a
recommendation has been given and direction received" — i.e. the stage's
own authored text describes itself as running *after* recommend, while its
numeric position and `30-recommend/CONTEXT.md`'s Inputs table (which
declares `../20-verify/output/README.md` as input) both encode it running
*before* recommend. The package is internally self-contradictory, not just
source-drifted.

This matters for authority, not just sequencing: the source's step 2
explicitly gates verification effort (reproducing a bug, checking out and
testing a PR diff) behind the maintainer's direction — the whole point of
"wait for direction" before step 3 is to avoid spending investigative
effort on an item the maintainer may redirect or reject outright. With the
live stage order, `20-verify` executes *before* the maintainer has had any
chance to react to a recommendation, because no recommendation exists yet
at that point in the pipeline. That directly undercuts the very J5 claim
the producer's own table asserts for `BU-TRI-05` ("no state-changing
action proceeds before explicit maintainer direction") — verification
isn't state-changing on the tracker, but it is real effort the source
explicitly wanted gated, and the producer cited this constraint as already
correctly enforced when the stage graph as shipped does not enforce it.

## Behavior-unit dispositions

### BU-TRI-01 — verdict: CONFIRMED

Package-level PL-4 framing. Re-derived independently against PL-4's
checklist (§5.6): recognizable trigger after intent shaping (attention
bucket or named item), bounded outcome (terminal state + artifact),
completion condition, explicit I/O, durable checkpoints, coherent
authority envelope (modulo BU-TRI-09 below), result meaningful without the
originating conversation continuing. Holds. No PL-2 pull — see
BU-TRI-07's re-derivation below, which independently reaches the same
conclusion as the producer's Driver-and-admission-boundary section.

### BU-TRI-02 — verdict: CONFIRMED

`10-gather-context` re-read in full against `BU-P3-065`/`BU-P3-089` and
the live `CONTEXT.md`. PL-5 holds (fresh checkpoint: full item read plus
two evidence checks that gate everything downstream). J2 delegation
("evidence inspection and the redundancy/prior-rejection verdict") is
named in the stage's own Behavior contract, not asserted without bound —
satisfies §6.5's "the package must name the delegation."

### BU-TRI-03 — verdict: CONFIRMED

Independently re-checked the N1 adjudication A4 fold rationale against
§5.8 (PL-6 is evaluated after PL-5, so a checkpoint that is *also*
deterministic can still be folded once it fails PL-5 on its own). The
former `00-show-attention` stage has no independent retry/failure/cost
boundary of its own — it is pure query-and-display, consumed immediately
by the same execution that begins `10-gather-context`. The fold is
justified; J5 (fixed, non-actor-chosen bucket composition) is the correct
rung — the bucket definitions are lifted near-verbatim from the source
(`BU-P3-062/063/064`), leaving no actor choice to delegate at J2.

### BU-TRI-04 — verdict: NEEDS-REVISION

PL-5 rung itself is right (fresh checkpoint: an empirical verdict
downstream stages depend on), and `BU-P3-067`'s content citation is
accurate. What's wrong is **stage order**, not placement: per the
headline finding above, this stage's own trigger text ("a recommendation
has been given and direction received") contradicts its numeric position
(20, before `30-recommend`). Re-derived independently against the source
line order (recommend at line 72, verify at line 74) and against the
stage's own Inputs/trigger prose — both point the same direction, away
from where `workflow.toml` currently places it. This stage's number should
be `30` (after recommend), with `30-recommend` renumbered `20`, and each
stage's Inputs table corrected to match (`30-verify` would then declare
`../20-recommend/output/README.md`; `40-grill-if-underspecified` already
correctly points at whichever stage precedes it and needs no content
change beyond the directory it points to). Renumbering two stage
directories/citations is still in-place content amendment, not a
placement-rung change (PL-5 holds for both either way) — this does not
newly trigger REHOME/SPLIT/HARVEST.

### BU-TRI-05 — verdict: NEEDS-REVISION

PL-5 and the J2 delegation ("the recommendation itself") both re-derive
cleanly and are CONFIRMED on their own terms. The J5 claim attached here —
"no state-changing action proceeds before explicit maintainer
direction... the gate that authorizes everything `50-apply-outcome` later
does" — is accurate about the source's *intent* but is currently
**not enforced by the live stage graph**, because `20-verify` runs before
this stage under the current numbering (see BU-TRI-04). The disposition
itself should still be STAND for this behavior unit's placement, but the
J5 claim as worded overstates what the package currently guarantees; it
should read as correct-after-the-BU-TRI-04-fix, not as a description of
current behavior.

### BU-TRI-06 — verdict: CONFIRMED

Independently re-read `skills/grilling/SKILL.md` and
`docs/icm/agents-invariant-dispositions.md` row `BU-1064`. The corrected
row states plainly: no `domain-modeling` skill package exists; the two
live packages that named a paired "grilling/domain-modeling session"
(`triage/40-grill-if-underspecified`, `wayfinder/00-name-destination`)
were corrected to name `grilling` alone. `triage/40-grill-if-
underspecified/CONTEXT.md` matches this correction exactly. The fold is
real, not a producer assertion taken on faith — re-derived from the cited
row's own text, independent of the producer's paraphrase.

### BU-TRI-07 — verdict: CONFIRMED

Independently re-applied the PL-2 discriminator (§5.4: "If the
procedure's job is to decide what Work should exist, it cannot itself
require an already-existing Work merely to make that decision") against
`BU-P3-061`/`BU-P3-073`. Both the discovery trigger and `quick-override`
operate only inside an already-admitted `triage` execution, selecting
among a closed set of state-machine actions on an already-identified item;
neither decides whether triage-the-item should be direct-vs-durable Work,
nor picks a workflow. PL-2 does not fire. The `validate-and-ship/00-check-
scope` analogy the producer draws is apt and independently checked against
that record — same shape (bounded interpretation of a free-form request
into one of a fixed set of already-defined actions), same resolution.

### BU-TRI-08 — verdict: CONFIRMED

Re-read all nine cited behavior units (`BU-P3-069` through `BU-P3-096`)
against `50-apply-outcome/CONTEXT.md` verbatim — every KB write/no-write
rule, the already-implemented/rejected-bug/rejected-enhancement branching,
and the reconsideration-doesn't-reopen-old-closures rule are present and
accurately paraphrased. J5 (fixed KB rules) + J4 (maintainer direction
already obtained upstream authorizes the terminal action) both re-derive
correctly. One caveat, not a defect: this stage's Inputs table points to
`../40-grill-if-underspecified/output/README.md`, which is unaffected by
the BU-TRI-04 renumbering (grill stays after both verify and recommend
either way) — flagging only to confirm the fix scoped in BU-TRI-04 does
not cascade here.

### BU-TRI-09 — verdict: CONFIRMED

Independently grepped all six `CONTEXT.md` files (five stages + workflow
level) for `## Bounded judgment` and `## Authority envelope` — neither
heading exists anywhere in the package; every stage instead carries the
generic "## Judgment required" boilerplate paragraph. Matches ADR 0013
decision 4 ("every actor stage carries an explicit local `## Bounded
judgment` section always... omission is never ambiguous") and
`docs/icm/convention.md` §6.1/§7.2's authority-envelope requirement.
Confirmed as a real, present gap, not a stale citation.

### BU-TRI-10 — verdict: CONFIRMED

Independently confirmed: `CONTEXT.md:36` and `CONTEXT.md:40` both cite a
local `provenance.md`; `ls .sergeant/workflows/triage/*.md` shows only
`CONTEXT.md` and `index.md` — no `provenance.md` anywhere in the live
tree. `index.md` already correctly points to
`docs/gauntlet/promoted-provenance/triage.md`. Dangling reference
confirmed exactly as described; mechanical fix, no placement
implication.

### BU-TRI-11 — verdict: CONFIRMED

Re-ran the citation-accuracy check independently against
`skills/grilling/SKILL.md`'s current content (its "Failure behavior" and
"Bounded judgment" sections, and its "This skill must not do" clause
forbidding `sgt run`/Work dispatch). All three of `triage`'s claims about
`grilling` (execution location, retirement event/rationale, E3-dependency
resolution) hold against the current text; `triage`'s delegation text does
not reference the specific passage the ICM-R2 pilot corrected
(`docs/environments/cerberus.md`), so no drift was introduced. Independent
re-check reaches the same conclusion as the producer's own record.

## Additional checklist items not separately tabled by the producer

- **Package identity/naming:** no issue. `triage` names its behavior
  accurately; no collision found.
- **Duplicated or drift-prone content:** the producer already flagged the
  `provenance.md`-dangling-reference pattern as corpus-recurring
  (BU-TRI-10) and correctly declined to fix it corpus-wide here — agreed,
  that is genuinely out of this pass's scope. One further drift risk not
  named by the producer: once BU-TRI-04's renumbering lands, `index.md`
  and the workflow-level `CONTEXT.md`'s stage table (`## Stages`) both
  restate stage-to-behavior-unit mappings that will need the same
  renumbering — not a new defect, but worth listing as part of the same
  remediation pass so it isn't half-applied.
- **False pairing assumptions:** none found beyond the already-resolved
  grilling/domain-modeling pairing (BU-TRI-06), which the producer
  correctly did not re-litigate — checked and it is settled, not
  reopened.
- **Unjustified engine gaps:** none claimed by this package; the producer
  correctly upheld the source extractor's original rejection of an
  engine-gap claim for `BU-P3-060`'s non-linear transition graph
  (re-checked: each transition is a fresh stage invocation, not a
  control-flow primitive the runtime must own — holds).

## Overall verdict on Final disposition

**STAND, with the producer's Final disposition (STAND) confirmed but its
scope of required work understated.** The producer characterized the
package as needing only in-place content amendment (Bounded-
judgment/Authority-envelope sections, one dangling reference). That is
correct as far as it goes, but incomplete: the `20-verify`/`30-recommend`
stage-order transposition found here is a fourth required in-place
amendment (renumber the two stage directories and update the three Inputs
tables and two stage-table rows that reference them), not a placement or
authority defect that would change the package's disposition modifier.
Renumbering two already-PL-5 stages relative to each other does not
trigger REHOME/SPLIT/HARVEST — the producer's underlying architectural
conclusion (STAND, no stage moves/merges/splits, no draft-and-rehome step)
still holds. The package is **not yet authority-valid** (agreed with the
producer) **and now also not yet source-fidelity-valid** until the stage
order matches the source's documented sequence and the internal
contradiction between `20-verify`'s trigger prose and its numeric position
is resolved.
