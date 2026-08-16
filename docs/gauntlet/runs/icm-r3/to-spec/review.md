# Independent adversarial review: to-spec

Proposal §8.11 / `docs/icm/convention.md` §6.2-6.3 independent review step.
Fresh execution, explicit inputs, review-only contract, no edit authority
over `adjudication-draft.md` or `draft/skills/to-spec/SKILL.md` (ADR 0013
decision 7; `convention.md` §6.3's independence test — this review is a
separate turn with its own re-reads of the package content, not a
continuation of the producer's own reasoning). Method: re-derive each
disposition against `.sergeant/workflows/to-spec/`'s actual files first,
then compare to the producer's citations and conclusions, per the §8.11
checklist (source fidelity; rung order; Captain/workflow boundary;
stage/helper boundary; authority grants and missing J0 cases; package
identity/naming; duplicated or drift-prone content; false pairing
assumptions; unjustified engine gaps).

## Source fidelity check (all units)

Every `BU-P4-0{50..54}` quote/locator was checked against
`reference/sergeant-upstream/.agents/skills/to-spec/SKILL.md` directly
(not against the producer's own restatement) and against the live
`.sergeant/workflows/to-spec/{00-gather-context,10-sketch-seams}/
CONTEXT.md`. All five resolve to the cited lines with no invented content.
The "byte-identical" claim for `.agents/skills/to-spec/SKILL.md` vs.
`.claude/skills/to-spec/SKILL.md` was independently re-run with `diff`
(both live under `reference/sergeant-upstream/`) — confirmed identical.
The `matt-pocock` grep claim was independently re-run (excluding
`reference/sergeant-upstream/` and `docs/gauntlet/runs/n2-run4/`) — no
hits, confirmed. The `BU-P3-069` quote against `triage/50-apply-outcome/
CONTEXT.md` was independently re-read — matches verbatim, so the
"ready-for-agent conflict" is a real conflict between two admitted
packages, not a producer invention.

### BU-P4-050 -- verdict: CONFIRMED

Re-derived independently, not from the producer's own citation trail: the
package's admission boundary today is `sgt run --workflow to-spec`
(`workflow.toml`, `index.md status: published`). Applying
`convention.md` §2a's execution-surface test directly ("would a human type
`sgt run '<intent>' --workflow to-spec`?") against `00-gather-context`'s
own Behavior contract — triggered by "the user asks to turn the *current
conversation*" and instructed to "write the spec from what has already
been discussed" — fails structurally: there is no way to type that intent
without the user first distilling the whole prior discussion themselves,
which is Captain's PL-2 job per proposal §5.4, not this package's. The
`00-gather-context/CONTEXT.md` Inputs table independently confirms this:
its only row is `../CONTEXT.md` (L1, workflow orientation) — no file or
Work field carries "the current conversation" in. This is the same
execution-surface failure `convention.md` §2a itself names ("if the
package cannot absorb an intent... it is not a workflow"), reached here
independently of the producer's own §7.7/§4.7 citations, which were
separately checked and both resolve to real proposal sections with the
quoted text present verbatim. PL-2 over PL-3 also holds: this behavior
"turns user conversation into a bounded submission" (proposal §5.4's own
PL-2 example), and does own a complete lifecycle (conversation to
published ticket), so PL-3's "does not independently own one admitted
intent from start to terminal outcome" discriminator does not apply.

One completeness gap, not a correctness one: "Alternatives considered"
does not record and reject the alternative of keeping `to-spec` at PL-4/
PL-5 by redefining its Inputs to accept an already-written design-doc
file instead of "the current conversation." That alternative was
independently checked here and rejected — the upstream source's own
behavior (`BU-P4-050`) is explicitly conversation-synthesis, not
file-based drafting, so accepting a design-doc file would invent new
behavior not licensed by the source (the same over-generalization the
ladder's own normalization rule forbids). The rejection is sound, but the
producer's record should show it was considered rather than omit it
silently — this is a documentation gap, not a disposition error.

### BU-P4-051 -- verdict: CONFIRMED

PL-3-flavored/single-consumer reasoning re-checked against §5.10's rule
("shared/local... becomes shared only when two or more consumers use the
same contract"). No other package in the live tree performs "explore repo
if not already done, use domain glossary, respect ADRs in the touched
area" as a named, citable contract — this is generic due-diligence
language distinct from the seam-technique overlap found below, and I did
not find a second consumer for it. REHOME as local content inside
`skills/to-spec/SKILL.md` holds.

### BU-P4-052 -- verdict: NEEDS-REVISION

The producer's classification ("PL-3-flavored reusable technique, but
single consumer today") is not supported once the rest of this same
ICM-R3 pass is checked: `.sergeant/workflows/tdd/00-agree-seams/
CONTEXT.md` (`BU-P2-109`/`BU-P2-110`) states a directly adjacent
contract — "before writing any test the actor writes down the seams under
test and confirms them with the user... Ask: 'What's the public
interface, and which seams should we test?'" — and this repository's own
ICM-R3 `tdd` package (`docs/gauntlet/runs/icm-r3/tdd/adjudication-draft.md`,
row `BU-TDD-02`) has already REHOMEd that exact behavior to a shared
context, `.sergeant/common/contexts/tdd.md` (`@@tdd`, "Seams — where
tests go" section), specifically because it is a PL-3 reusable technique
(proposal §5.5's own worked-example list names "a TDD technique" first).
`to-spec`'s `BU-P4-052` ("sketch out the seams... prefer existing,
highest possible, fewest new") and `BU-P4-053` ("confirm the seam plan
with the user") are the same family of judgment — choose/confirm test
seams before code exists — applied one lifecycle stage earlier (spec-time
vs. red/green-time). The producer's `Inputs and outputs` section never
lists `.sergeant/workflows/tdd/` or the sibling ICM-R3 `tdd` package among
what was read, so this adjacency was not checked before concluding
"single consumer."

This is not a confirmed merge order — the trigger and admission boundary
genuinely differ (spec-drafting, before implementation exists, vs.
red/green, when a test file is about to be written), and to-spec's own
extra guidance ("prefer existing over new," "highest possible seam,"
"ideally exactly one") is not present in `tdd`'s current `@@tdd` content.
Forcing them together on speculation would itself be the false-pairing
failure the checklist warns about. But the producer's own record should
name this adjacent package and either (a) explain concretely why the two
contracts are distinct enough not to share representation, or (b) propose
extending `@@tdd`'s "Seams" section with the preference-ordering guidance
and having `skills/to-spec/SKILL.md` reference `@@tdd` rather than
duplicate seam-sketching prose. As drafted, the `draft/skills/to-spec/
SKILL.md` process step 2 restates the seam guidance in full local prose
with no cross-reference to `@@tdd` — a real drift risk (two admitted
surfaces independently telling actors how to choose test seams, able to
diverge silently) that this producer's own "duplicated or drift-prone
content" self-check (proposal §8.10) should have caught, especially since
the sibling `tdd` reconciliation ran in the same ICM-R3 pass.

### BU-P4-053 -- verdict: NEEDS-REVISION

Same underlying gap as `BU-P4-052`: "confirm the proposed seams with the
user" is the same act `tdd`'s `BU-P2-109` already states ("confirms them
with the user... no test is written at an unconfirmed seam") and that
ICM-R3's own `tdd` reconciliation classified as a **J0 the caller must
honor** inside `@@tdd` ("seam confirmation is not a J2 judgment call the
loaded stage may skip or infer"). The producer's table instead states
this unit is "Not a J-rung question — this is Captain's 'must ask the
user' bucket (§7.4 skill shape), not a delegated decision." That framing
is defensible on its own terms (§6.7's J0 procedure does direct a Captain
skill to "ask the question live and wait," so treating a designed
confirmation step as outside the delegated-decision ladder rather than as
an escalation is a reasonable, non-generic reading) and is not itself
wrong. But it was reached without checking `@@tdd`'s parallel treatment of
the identical act as an explicit **J0-the-caller-must-honor** citation,
which is a stronger, ladder-grounded way to state the same constraint and
would keep `to-spec`'s framing consistent with its sibling package rather
than inventing a second, uncited vocabulary ("Captain's 'must ask the
user' bucket") for the same situation. Revise to either cite the ladder
explicitly (as `@@tdd` does) or state why `to-spec`'s confirmation moment
does not warrant the same citation `tdd`'s does.

### BU-P4-054 -- verdict: CONFIRMED

Independently re-checked: PL-6 classification for the mechanical
write/publish/label step is sound under the reimplementation test
(proposal §5.7) — replacing the tracker-call mechanism tomorrow would not
change what an operator cares about at a boundary here. The J0 finding on
the `ready-for-agent` conflict is confirmed as genuine, not manufactured:
`triage/50-apply-outcome/CONTEXT.md`'s `BU-P3-069` ("Reaching the
ready-for-agent outcome requires posting a structured agent brief
comment") and the upstream `to-spec` source's own text ("Apply the
`ready-for-agent` triage label - no need for additional triage") were both
read directly and do state incompatible requirements for the same
outcome label inside the same admitted catalog. The rungs-checked table
(J5 through J1) is correctly applied and the conflict rule
(`bounded-judgment.md` "Conflict rule") is correctly invoked rather than
silently resolved in the producer's own favor. The `draft/skills/to-spec/
SKILL.md`'s "Before step 3, resolve..." section carries the same
unresolved gap forward as an explicit ask-the-user instruction rather than
guessing — consistent with the J0 procedure.

One unrelated observation, not a defect in this producer's own reasoning:
the live (pre-reconciliation) `10-sketch-seams/CONTEXT.md` line 34 still
states "No `kind = "execute"` stage exists in the current engine" as part
of the historical N1 adjudication A4 fold rationale. That claim is false
as of this branch — `repo-to-icm/workflow.toml`'s `65-self-check` is a
live `kind = "execute"` stage, the same correction ICM-R2's pilot review
already made for `research`'s `00-investigate/CONTEXT.md`. The producer's
own PL-6 reasoning for `BU-P4-054` does not repeat or rely on this stale
claim (it cites proposal §5.8's "ordinary tool invocation" instead), so
this does not affect the `BU-P4-054` verdict — it is moot once
`.sergeant/workflows/to-spec/` is deleted on promotion, per the package's
own REHOME disposition. Flagged here only so it is not silently
re-asserted as current if this record or its citations are ever reused.

## Additional checklist items

**Package identity/naming.** `to-spec` is retained unchanged for the new
skill, consistent with the `grilling` precedent (workflow name carried
directly to skill name). No issue.

**Captain/workflow boundary.** Independently re-derived above under
`BU-P4-050` using `convention.md` §2a's execution-surface test directly,
not merely the producer's own proposal §5.4/§5.6 citations — the same
conclusion is reached from a second, independently-sourced test. Confirms
the PL-2 placement is not an artifact of selective citation.

**Missing J0 case: headless/no-live-user invocation.** The draft
`skills/to-spec/SKILL.md` has no equivalent of `skills/grilling/
SKILL.md`'s "Failure behavior" section (what happens if this skill is
invoked with no live human to confirm seams or hold the current
conversation). Given `to-spec`'s entire PL-2 argument rests on live
conversational content and a live confirmation step (`BU-P4-053`), and its
own sibling `grilling` explicitly addresses this failure mode after an
ICM-R2 correction, its absence here is a gap the producer's self-check
(proposal §8.10, "authority envelopes are complete") should have caught.
This does not change the PL-2/REHOME classification, but the draft skill
is incomplete without it.

**Duplicated or drift-prone content.** The spec `<spec-template>` block
was checked against the rest of the live and draft tree
(`grep -rl "## Problem Statement"`) — no other copy exists, so no drift
risk there. The seam-sketching prose (`BU-P4-052`/`BU-P4-053`) is the one
genuine duplication/drift risk found, detailed above.

**Unjustified engine gaps.** None claimed by the producer (PL-7
unreached); independently confirmed no unit here requires new runtime
capability — the whole package resolves at PL-2/PL-6 with one J0 gap
flagged, not invented.

**Rung order.** PL-0 through PL-3 were checked in order before landing at
PL-2 for `BU-P4-050`; independently re-run here and PL-0/PL-1 clearly do
not apply (no absorbing surface exists; this is not a broad, rarely-
changing invariant). No skipped rung.

## Overall verdict

**Final disposition: REHOME — CONFIRMED**, with **NEEDS-REVISION** on the
draft skill content before promotion.

The core placement argument (PL-2, not PL-4/PL-5) is independently
re-derivable from the package's own content and from a second, separately
sourced test (`convention.md` §2a) that the producer did not itself cite
for this purpose — this is a stronger result than the producer's own
record shows, not a weaker one. The `ready-for-agent` J0 finding is
genuine and correctly handled. REHOME to `skills/to-spec/SKILL.md`, with
`.sergeant/workflows/to-spec/` retiring, should stand as the final
disposition.

Before Captain's reconcile-and-publish pass accepts this record, two
concrete revisions are owed:

1. Reconcile `BU-P4-052`/`BU-P4-053`'s seam-sketching and seam-confirming
   content against ICM-R3's own sibling `tdd` reconciliation
   (`@@tdd`'s "Seams" section) — either justify keeping them separate in
   the record (not just in practice) or fold the shared portion into
   `@@tdd` and have `skills/to-spec/SKILL.md` reference it.
2. Add a "Failure behavior" section to `draft/skills/to-spec/SKILL.md`
   addressing headless/no-live-user invocation, matching the precedent
   already corrected in `skills/grilling/SKILL.md`.

Neither revision is a rung-order or source-fidelity error large enough to
overturn REHOME as the final disposition; both are promotion-readiness
gaps this producer's own self-check step should close before Captain's
reconcile-and-publish pass treats this record as settled.
