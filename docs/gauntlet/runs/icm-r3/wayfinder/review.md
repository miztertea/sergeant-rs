# Package adjudication review: wayfinder

Independent adversarial review of `docs/gauntlet/runs/icm-r3/wayfinder/
adjudication-draft.md`, per `reference/proposal-icm-r-procedure-authority.md`
§8.11 and `docs/icm/convention.md` §6.3 (this review qualifies as
independent: fresh execution, explicit inputs read directly from the live
package and the cited governing docs, review-only contract, no edit
authority over the draft or the package). Checklist applied: source
fidelity, rung order (PL and J), Captain/workflow boundary, stage/helper
boundary, authority grants and missing J0 cases, package identity/naming,
duplicated or drift-prone content, false pairing assumptions, unjustified
engine gaps.

Method: every citation in the draft was independently traced to
`reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` (read in
full) and to the live package under `.sergeant/workflows/wayfinder/` (read
in full, all four stage `CONTEXT.md` files, `CONTEXT.md`, `index.md`,
`workflow.toml`, all four `output/README.md`). `reference-corpus/{shared-
context-map,helper-map,provenance-map,synthesis,engine-pressure}.md`,
`docs/adr/0013-icm-r0-owner-rulings.md`, `docs/icm/{convention,record-
shapes}.md`, and `docs/icm/re-homing-record-2026-08-12.md` were read
independently rather than trusted from the draft's own paraphrase.

### BU-WAY-01 -- verdict: CONFIRMED

Re-derivation: `00-name-destination/CONTEXT.md`'s Delegation section
was independently checked against `skills/grilling/SKILL.md`'s current
content and `docs/icm/re-homing-record-2026-08-12.md:44-48`, both of which
confirm R-NS-6 ("execution ≠ dialogue," dogfood-measured 2/2 zero
`needs_input` runs) as the governing reason `grilling` runs live in-session
rather than as a dispatched Work. The PL-2 local narrowing for this one
sub-step, inside an otherwise PL-5 stage, is correct: the destination-naming
interview is Captain-shaped (live dialogue, no durable mid-turn hold), while
the stage as a whole still meets the PL-4/PL-5 workflow-stage tests (fresh
execution, explicit Inputs, a durable outcome recorded to `output/`). J5
citation to R-NS-6 is accurate and traced to a primary source, not a
paraphrase of the draft's own prior claim.

### BU-WAY-02 -- verdict: CONFIRMED

Re-derivation: `10-map-frontier/CONTEXT.md`'s first Behavior-contract bullet
and its citation (`BU-P4-095`, upstream L112) were independently checked
against the live upstream text — "If this surfaces no fog ... you don't need
a map. Stop and ask the user how they'd like to proceed" — an exact match.
The J0 carve-out (no fog found -> ask the user, do not silently create a
trivial map) is correctly identified as a genuine stop-and-ask condition
under J0's own test (§6.7 of the proposal: "the choice would change scope").
PL-5 is the right rung; this is a judgment call inside an already-admitted
stage, not new machinery.

### BU-WAY-03 -- verdict: CONFIRMED

Re-derivation: citation `BU-P4-091` traced to upstream "Out of scope," L97
("Fog only ever gathers toward the destination... work beyond it is out of
scope") — exact match. J2 (scope-vs-fog classification delegated to the
stage) is the correct rung; this is ordinary judgment inside a named
delegation, not a governing constraint or a settled record.

### BU-WAY-04 -- verdict: NEEDS-REVISION

Re-derivation of the disposition itself (PL-6, helper folded into
`10-map-frontier`, STAND) is CONFIRMED: `BU-P4-096` traces exactly to
upstream L114 ("Create the tickets you can specify now... then wire
blocking edges in a second pass... issues need ids before they can
reference each other"), and the two-pass creation order is genuinely
deterministic machinery with no additional checkpoint argument, matching
N1 adjudication A4's own default-demotion rule.

But the draft's own validation evidence claims "every existing behavior-unit
citation... was read in full" (Validation evidence, "Source-valid") and
separately claims this pass performed a completeness check "beyond the five
§9.1 claims" by "read[ing] the entire package." That check missed a live
factual error inside the very Helper-invocation section this unit
dispositions. `10-map-frontier/CONTEXT.md` line 40 reads: "No `kind =
"execute"` stage exists in the current engine, so the acting harness
performs the ticket-creation operation itself." This is false as of this
branch: `.sergeant/workflows/repo-to-icm/workflow.toml` (line 44) declares
`kind = "execute"` for its `65-self-check` stage — a live, currently
admitted execute stage. Independently confirmed: `grep kind
.sergeant/workflows/repo-to-icm/workflow.toml` shows the directive, and
`65-self-check/` exists as a stage directory. This is not a hypothetical —
the identical false claim, in the identical wording, was already found and
corrected once in this same reconciliation effort (the `research` workflow's
`00-investigate/CONTEXT.md` "Helper invocation: write findings" section
carries an explicit correction note dated to the ICM-R2 pilot review
recording that exact sentence as false and parking the resulting "should
`10-map-frontier`'s two-pass ticket creation instead be a
`kind = "execute"` stage riding after the actor's own turn?" question as an
open follow-on, not silently re-asserted). `wayfinder`'s own helper text
still carries the uncorrected claim verbatim, and this producer draft did
not catch it despite explicitly claiming a full-package read plus a
completeness check beyond the five §9.1 claims.

This does not change BU-WAY-04's disposition (helper, not a stage, STAND)
— replacing the false "no execute stage exists" premise with the correct
"an execute-stage pattern exists elsewhere, but this fold has no additional
checkpoint argument (§6.4) beyond deterministic sequencing" reaches the same
PL-6 rung by the right reasoning instead of a false one. But it is an
in-place content defect of the same shape and severity as BU-WAY-11's
dangling `provenance.md` reference — a stale/false citation inside package
prose — and should be added to the "Surviving package design" remediation
list (alongside BU-WAY-11, BU-WAY-12, BU-WAY-13) rather than left
unaddressed. Recommend folding this correction into the same in-place
amendment pass the draft already scopes for BU-WAY-11/12/13, using the
already-corrected `00-investigate/CONTEXT.md` wording as the direct
precedent for how to phrase it (park, don't resolve, the open
execute-stage question).

### BU-WAY-05 -- verdict: CONFIRMED

Re-derivation: `BU-P4-098` traces exactly to upstream "Work through the
map," L122-123 ("Load the map... Choose the ticket... Claim it: assign it
to yourself before any work"). J2 (which ticket to choose when unnamed) +
J5 (claim before work, always, no exception offered anywhere upstream) is
correctly split — claiming is a hard governing sequencing rule (skipping it
breaks the whole claim-as-concurrency-signal mechanism the map depends on),
while ticket selection when none is named is a bounded judgment call.

### BU-WAY-06 -- verdict: CONFIRMED, with a sharper J-boundary note

Re-derivation: `BU-P4-085` traces exactly to upstream "Ticket Types," L75.
The J5 citation ("HITL ticket resolution requires a live human exchange —
the agent may not self-answer") is correct as a governing constraint. One
addition worth recording for the eventual `## Bounded judgment` rewrite
(BU-WAY-12): the HITL/AFK *classification* itself — is a given ticket HITL
or AFK — is not named as its own delegation anywhere in this unit or
elsewhere in the package. Upstream ties it to ticket type (Prototype and
Grilling are HITL, Research is AFK, Task is either), so for the two types
this package's stage prose actually names (research, task) the
classification is J3 (settled by the ticket's own type label) rather than
open judgment — worth stating explicitly rather than leaving implicit, but
this is a J1-level clarity improvement to the future `## Bounded judgment`
section, not a defect in this unit's PL/J rungs as currently recorded.
Disposition and rungs as drafted: CONFIRMED.

### BU-WAY-07 -- verdict: CONFIRMED

Re-derivation: `BU-P4-086` (upstream "Ticket Types," L77, Research) and
`BU-P4-087` (L80, Task) both trace exactly. J2 (research/task/decision
classification delegated) is correct — the package names the types but
requires the actor to classify an incoming ticket against them, which is
exactly J2's shape ("the active skill or stage explicitly delegates this
class of decision within named bounds").

### BU-WAY-08 -- verdict: CONFIRMED

Re-derivation: all three citations (`BU-P4-093`, `BU-P4-099`, `BU-P4-092`)
trace exactly to upstream L105, L125, and L101 respectively. The J5/J2 split
(one-ticket cap and the fixed three-part resolution sequence are governing;
in-scope-vs-out-of-scope reclassification during resolution is delegated)
matches the ladder's own worked-example shape: the cap and sequence are
stated as unconditional rules upstream ("never resolve more than one...";
"post... close... append"), while scope reclassification requires the actor
to judge against the destination, which is J2.

### BU-WAY-09 -- verdict: CONFIRMED

Re-derivation: citations `BU-P4-076`, `BU-P4-097`, `BU-P4-100` all trace
exactly to upstream L13 ("Plan, don't do"), L115 (fire research subagents),
and L128 (concurrent-editing awareness). The G7 rejection was independently
re-checked against `reference-corpus/synthesis.md` §5 and
`reference-corpus/engine-pressure.md` (line ~1044, ~979: "G7 dynamic ticket
graph | BU-P4-090 | rejected — absorbed at §6.5/§6.6 (shared-context/
helper)") — the draft's characterization ("ownership preference, not a
representational failure") is a fair paraphrase of that already-settled
verdict, not a new or unjustified engine-gap claim. J4 (an effort's own
Notes override the plan-don't-do default) and J5 (G7 closed) are correctly
distinguished. Representing the `40-regraduate-fog` -> `10-map-frontier`
loop as fresh re-invocation rather than engine-level looping is consistent
with `workflow.toml`'s own stage list (four stages, no loop primitive
declared) and does not conflict with any admitted engine capability.

### BU-WAY-10 -- verdict: CONFIRMED, disposition gap independently
re-derived and found accurate but incomplete in scope

Re-derivation of the core claim: independently confirmed by reading
`reference-corpus/shared-context-map.md` (Part 3, line 348: `BU-P4-077,
BU-P4-080` assigned to `W33 wayfinder`, "Refer by name") and
`reference-corpus/helper-map.md` (line 337: `BU-P4-078, BU-P4-079,
BU-P4-081, BU-P4-082, BU-P4-083, BU-P4-084` assigned to `W33 wayfinder`,
"The Map"), cross-checked against `reference-corpus/provenance-map.md`
(line 146: all 26 units `BU-P4-075`-`BU-P4-100` sourced from upstream
`SKILL.md`, landing distribution "draft-workflows/wayfinder ×17;
helper-map.md ×6; shared-context-map.md ×2" — 17+6+2=25, plus the one
rejected engine-gap unit `BU-P4-090` = 26, which is internally consistent
and matches the draft's own count). `grep -rn "@@" .sergeant/workflows/
wayfinder/` independently confirms none of these eight units, nor any
`@@name` reference at all, appear anywhere in the live package. The gap is
real and the draft's classification of where it belongs (workflow-local
helper -> `10-map-frontier`; workflow-local context -> `CONTEXT.md` or
`00-name-destination`) is correct per the reference corpus's own already-
settled assignment — this producer correctly did not re-litigate that
placement, consistent with proposal §8.11's own scope (challenge boundary
and completeness, not re-run extraction).

One completeness gap in the draft's own gap-finding: `reference-corpus/
shared-context-map.md` Part 2 ("already-conventionalized `@@name`
contexts") lists two more shared contexts as "Shared by" `W33 wayfinder` —
`@@ticket-shaping` (line 289-290) and `@@triage-state-machine` (line
312) — that the draft's "missing-content gap" section does not mention at
all. Independently checked whether this is a live omission or already
covered: `grep -rln "ticket-shaping\|triage-state-machine"
.sergeant/workflows/ .sergeant/common/` finds no materialized
`.sergeant/common/contexts/{ticket-shaping,triage-state-machine}.md` file
anywhere in the repo, and no *other* workflow that shares these same two
contexts (`to-tickets`, `triage`, `cross-repo-work`) references them via
`@@name` either — this is a cross-package materialization gap in the
`@@name` shared-context catalog generally, not specific to `wayfinder`, and
matches the exact precedent this package's own `00-name-destination/
CONTEXT.md` already documents for `@@domain-modeling` ("no `domain-modeling`
skill package exists in this repo yet ... only frozen upstream evidence").
Given that precedent is already in-package and explicit, treating
`@@ticket-shaping`/`@@triage-state-machine` the same way is a reasonable J2
call, not an omission requiring a disposition change — but the draft's
"missing-content gap" section should have said so explicitly (one sentence,
following its own `@@domain-modeling` precedent) rather than staying silent
on two units the reference corpus itself names as belonging to this
package's Shared-by list. This is a documentation completeness note, not a
disposition dispute: verdict CONFIRMED, with the recommendation that the
in-place remediation pass's item 1 (land `BU-P4-078/079/081-084` and
`BU-P4-077/080`) add one sentence covering `@@ticket-shaping`/
`@@triage-state-machine` the same way `00-name-destination/CONTEXT.md`
already covers `@@domain-modeling`, so a future reader does not have to
re-derive this from the reference corpus independently.

### BU-WAY-11 -- verdict: CONFIRMED

Re-derivation: `find .sergeant/workflows/wayfinder -iname "provenance.md"`
independently confirms no such file exists in the live package tree, while
`CONTEXT.md`'s "Provenance" section (line 37) still names it. The actual
file, `docs/gauntlet/promoted-provenance/wayfinder.md`, exists and does
carry the stage-to-behavior-unit mapping (confirmed by reading it as part
of this review's own citation-tracing pass). FOLD (in-place reference
correction, no placement change) is the right disposition — this is a
dangling pointer, not a placement or authority question.

### BU-WAY-12 -- verdict: CONFIRMED, with an additional defect folded in

Re-derivation: all four stage `CONTEXT.md` files were independently read in
full and confirmed to carry byte-identical "## Judgment required"
boilerplate, none naming J2/J1/J0 in the shape `docs/icm/convention.md`
§6.1 requires ("its J2 delegations by name, its J1 local choices, what must
become `needs_input` at J0, its completion boundary, where decisions are
recorded"). Cross-checked against the four already-updated ICM-R2 pilot
packages (`code-review`, `repo-to-icm`, `research`, `validate-and-ship`,
confirmed via `grep -l "## Bounded judgment" .sergeant/workflows/*/*/
CONTEXT.md`), which do carry the required section — `wayfinder` was not
part of that nine-package pilot (ADR 0013 decision 8-9) and has not yet
received the same update, consistent with the draft's framing that this is
an ICM-R3 finding, not a regression.

Additional defect independently found and folded into this unit rather than
raised separately, since it lives in the same boilerplate paragraph
BU-WAY-12 already flags for replacement: the boilerplate's citation "This is
an actor stage (ladder §6.4)" does not resolve against any current governing
document. Neither `reference/proposal-icm-r-procedure-authority.md` (whose
own §6.4 is J3, "Settled authoritative record," not a stage-classification
rung) nor `.sergeant/common/contexts/bounded-judgment.md` (which has no
numbered `§6.4`) defines a "ladder §6.4" that means "this is an actor
stage." This is stale numbering from a numbering scheme that predates the
current PL/J ladders and was never updated when this package's boilerplate
was authored. Since BU-WAY-12 already schedules wholesale replacement of
this exact paragraph with a proper `## Bounded judgment` section, no
separate remediation item is needed — but the remediation description
should note the stale citation is being *replaced*, not merely
*reformatted*, so a future reader does not assume "ladder §6.4" was ever a
valid cross-reference worth preserving in spirit.

### BU-WAY-13 -- verdict: CONFIRMED

Re-derivation: `grep "Authority envelope" .sergeant/workflows/wayfinder/
CONTEXT.md` independently confirms no such section exists, and
`docs/icm/convention.md` §6.1 (line 422) requires it on every workflow's
Layer-1 `CONTEXT.md`. Cross-checked against the same four ICM-R2 pilot
packages (`grep -l "## Authority envelope" .sergeant/workflows/*/
CONTEXT.md`), which do carry it. Same status as BU-WAY-12: a real,
already-established requirement this not-yet-piloted package has not yet
received.

## Overall verdict on Final disposition

**CONFIRMED: STAND**, with the remediation list amended.

The draft's package-level re-derivation holds under independent
adversarial challenge: the four-stage sequence, the single delegation to
`grilling`, the PL-4/PL-5 workflow-and-stage rungs, the one PL-6 helper, and
the local PL-2 narrowing at `00-name-destination` are all correctly placed
and none of this review's findings requires REHOME, SPLIT, HARVEST, or
RETIRE. The rejected-alternatives section (Captain-skill rehome; reopening
G7; authoring the eight missing units on this producer's own authority) was
independently re-checked and each rejection holds for the reasons the draft
gives.

The "Surviving package design" remediation list should grow from five items
to six, adding the BU-WAY-04 execute-stage-claim correction found above
between the existing items 1 and 2 (it is the same kind of in-place content
fix, discovered by the same full-package read this pass already performed,
and should land in the same amendment pass rather than as a separate
follow-on):

1. Add the eight missing behavior units (`BU-P4-077`-`BU-P4-084`), per
   BU-WAY-10 — and extend this item's own prose to note
   `@@ticket-shaping`/`@@triage-state-machine` are treated the same as
   `@@domain-modeling` (not yet materialized in this repo), rather than
   leaving that silent.
2. Correct `10-map-frontier/CONTEXT.md`'s "Helper invocation: create
   tickets" section: replace "No `kind = "execute"` stage exists in the
   current engine" (false — `repo-to-icm/65-self-check` is a live
   `kind = "execute"` stage) with the corrected framing already used at
   `research/00-investigate/CONTEXT.md`'s equivalent helper section — park,
   don't resolve, the open question of whether this fold should ride on an
   execute stage instead.
3. Add `## Bounded judgment` sections to each of the four stage
   `CONTEXT.md` files, replacing the current boilerplate (including its
   stale "ladder §6.4" citation) per BU-WAY-12.
4. Add an `## Authority envelope` section to the workflow-level
   `CONTEXT.md` per BU-WAY-13.
5. Correct the dangling `provenance.md` reference per BU-WAY-11.
6. (Lower priority, unchanged from the draft) Consider whether Prototype-
   and Grilling-type tickets deserve their own behavior units in a future
   N1-extraction pass.

None of these six items changes which package owns the behavior, so none
triggers ADR 0013 decision 6's draft-and-rehome step — the draft's
reasoning on this point is independently confirmed. This review record
itself, per `docs/icm/convention.md` §6.2/6.3 and ADR 0013 decisions 6-7,
completes the independent-reviewer step for this package; the six-item
remediation list above is now ready for the reconciliation's own
reconcile-and-publish step (proposal §8.12).
