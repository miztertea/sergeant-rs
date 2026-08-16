# ICM-R0 — enactability critic

Axis: can ICM-R0 through ICM-R4 actually be executed as dispatched work, or
does a section hide an undecided question behind confident prose? Graded
against `reference/proposal-icm-r-procedure-authority.md` in full (§1–§20
plus the Source-to-Decision Map and Owner Decisions), per
`docs/gauntlet/contracts/ICM-R0.md`'s axis 4 charge.

## Method

Read the proposal end to end. For §10's five workstreams (ICM-R0–ICM-R4),
checked each stated Outcome bullet against whether the proposal's own text
supplies everything a dispatched Work would need to satisfy it, without
inventing a missing answer. Sampled items from §15's 38-item Acceptance
Contract and the §8.13 package-adjudication record shape, tracing each
sampled item back to the proposal section that defines the terms it uses,
to check whether a dispatched actor could complete it from the proposal's
own text alone. Cross-checked every place in §7–§10 that states an
operative rule matching one of §19's twelve owner-decision recommended
defaults, to see whether that rule is presented as settled or is hedged as
pending the owner's ruling — per the contract's explicit instruction that
§19 decisions are not silently decided by the proposal itself. Did not
read anything under `docs/gauntlet/runs/icm-r0/` before writing this
report, and did not assume any other critic's findings.

Severity key: **error** = a dispatched Work would stall or produce an
unreviewable guess; **warning** = a Work could proceed but would have to
invent something the proposal should have supplied, or would silently
enact an undecided owner ruling; **info** = worth recording, doesn't block
dispatch.

## Findings

### F1 — severity: error — §10.1 (ICM-R0's own stated outcome)

**Claim.** §10.1 lists, as one of four parallel Outcome bullets for the
ICM-R0 workstream itself: "adjudicate the owner decisions in §19" —
alongside "challenge the two ladders," "amend this proposal," and "record
accepted definitions and hard boundaries," all presented as things the
ICM-R0 process performs.

**What I checked.** Whether the proposal supplies a procedure by which a
dispatched actor (or the orchestrating session standing in for one) could
actually "adjudicate" the twelve items in §19. §19's own opening line:
"This proposal recommends defaults but does not silently make these owner
rulings." Cross-checked against the proposal's own bounded-judgment
mechanics: §6.7 (J0) is the proposal's own named procedure for exactly
this situation — "no higher rung resolves the question... Do not guess.
Stop before the undecided effect and produce one precise question," with
a Captain-skill path that requires asking the question live and waiting
for the user's answer. §19's twelve items are structurally J0 material:
each is framed as a question with no J5–J3 settlement and an author
recommendation offered only as a non-binding default (matching J0's
"state the actor's recommended answer... end the turn with one direct
question" shape), not as something a J2 delegation authorizes an actor to
close out.

**What I found.** §10.1 never connects "adjudicate the owner decisions in
§19" to §6.7's J0/live-question mechanic, or to any other named procedure
for surfacing owner rulings. Read literally, a dispatched Work executing
§10.1 has two options the proposal doesn't distinguish between: (a) treat
its own recommended defaults as the adjudicated answer and record them as
settled — which is exactly the "silently make these owner rulings" §19
forbids — or (b) stop at twelve separate J0 boundaries and hand the
questions back, which is not "adjudicate" in any sense a reviewer could
call complete. This is also the exact tension `docs/gauntlet/contracts/
ICM-R0.md`'s own Non-goals section names from the outside ("Not ruling on
§19... This gauntlet does not make them either... never dispatched, never
decided by a panel") — the contract had to carve this out explicitly
because the proposal's own §10.1 text, taken at face value, assigns
"adjudicate" to the workstream without saying who performs it or how.

**Verdict.** Does not survive as written. The outcome bullet needs either
to name the mechanism (a live owner interview, structurally distinct from
the critic/refuter/adjudicator Works this gauntlet dispatches) or to be
reworded to something a dispatched Work can actually close — e.g. "surface
the owner decisions in §19 as named open questions with recommended
defaults," which is what the rest of the document (and the ICM-R0 contract
itself) actually treats §19 as being used for.

### F2 — severity: warning — §10.4 (ICM-R3 full library reconciliation)

**Claim.** §10.4's Subject list for full-corpus reconciliation includes
"every shared context, helper, and delegation they depend on" and "the
built-in software-change workflow as a separate embedded package,"
presented as in-scope without qualification.

**What I checked.** §19 decision 3: "Universal scope: Does 'every skill
and workflow must be validated' include embedded software-change, shared
contexts, and helpers as first-class review subjects? This proposal
recommends yes, with validation depth proportional to effect." — explicitly
listed as an owner decision, not yet ruled.

**What I found.** §10.4's Subject list states the "yes" answer as the
operative scope for ICM-R3 with no hedge, no cross-reference to decision 3,
and no fallback if the owner rules "no" or "partial." A Work dispatched
against §10.4 as written would reconcile the embedded software-change
workflow and every shared helper as full first-class subjects — the exact
scope decision 3 says is still open — without any signal in §10.4 itself
that this is contingent on an owner ruling rather than already settled.

**Verdict.** Does not survive as written for the embedded-workflow and
helper/shared-context portions specifically; the rest of §10.4 (the 23
workflows, four skills) is uncontested and enactable regardless of how
decision 3 is ruled. §10.4 needs an explicit "pending decision 3" flag on
the embedded-workflow and helper/shared-context lines, or those lines
should be moved to a contingent appendix rather than the primary Subject
list.

### F3 — severity: warning — §9.5 (independent review definition), used by §9.3 and §15 item 21

**Claim.** §9.5 states, as flat operative doctrine: "A later stage may
qualify [as independent review] when it receives only the artifact and
review rubric, does not inherit the producing conversation, and cannot
silently edit the subject it reviews." This definition is then relied on
by §9.3's workflow-validation checklist and by §15 item 21 ("Every
promotable artifact names its independent review and promotion path").

**What I checked.** §19 decision 7: "Review independence: May a later
stage in the same workflow qualify as independent when it has a fresh
execution, explicit inputs, a review-only contract, and no edit authority?
This proposal recommends yes." Listed as an owner decision, not a settled
rule.

**What I found.** §9.5's phrasing ("may qualify when...") reads as an
already-adopted rule, not a recommendation awaiting ruling — there is no
cross-reference to §19-7 or any hedge. A dispatched Work building a review
stage under §9.5, or a reviewer checking §15 item 21 against that stage,
has no way to tell from §9–§9.5 alone that same-workflow-stage
"independence" is exactly the question the owner has not yet ruled on. If
the owner rules the opposite way (same-workflow stages never count as
independent), every package built to satisfy §9.5's stated criterion
during ICM-R1–R3 would need rework, and nothing in §9 or §15 flags that
exposure.

**Verdict.** Does not survive as written. §9.5 should either cite §19-7
directly and mark the criterion as provisional, or the workstreams that
depend on it (ICM-R2, ICM-R3) should note the rework risk explicitly.

### F4 — severity: warning — §15 item 33 and the Executive Summary's "hard boundary," vs. §19 decision 10

**Claim.** The Executive Summary states as a hard boundary: "No changes to
src/, API routes, journal schema, Work state, backend traits, TUI
behavior, or workflow.toml grammar in the initial workstream." §15 item 33
restates this as an unconditional Acceptance Contract line: "No file under
src/, no API route, no Work state, no journal event, no backend trait, and
no TUI behavior changes during ICM-R0 through ICM-R4."

**What I checked.** §19 decision 10: "Runtime freeze: Is 'no Rust/runtime
changes through ICM-R4' a hard contract or a default that an urgent,
independently proven engine gap may interrupt? This proposal recommends a
hard contract for this workstream; urgent runtime defects remain separate
work." Listed as an owner decision.

**What I found.** Both the Executive Summary and §15 item 33 state the
freeze as flat, unconditional fact — matching the proposal's own
recommended default, but with no hedge that this is a recommendation
awaiting an owner ruling rather than an already-settled acceptance
criterion. A Work using §15 as its completion checklist (which is exactly
what §15's own framing — "The ICM-R workstream is complete when all of the
following are true" — instructs) would treat item 33 as binding with no
carve-out, even in the case §19-10 explicitly names: an urgent,
independently proven engine gap surfacing mid-campaign. Nothing in §15 or
the Executive Summary points back to §19-10's carve-out language.

**Verdict.** Does not survive as written for the "hard contract, no
exceptions" reading. Either §15 item 33 needs an explicit
cross-reference to §19-10 (so a dispatched Work knows the freeze is a
default pending ruling, not an immovable constraint), or — if the owner
has separately ruled this one already — the proposal text needs to say so
rather than leaving it implicit through consistency between the
Executive Summary and item 33 alone.

## What I checked and found nothing on

- **§10.2 (ICM-R1) and §10.3 (ICM-R2) Outcome bullets**, checked line by
  line against whether each supplies enough for a dispatched Work to
  execute it: "add the canonical bounded-judgment context," "amend
  docs/icm/convention.md, record-shapes.md, and the repo-to-icm
  classification method," "add package-adjudication and decision-record
  templates" all point to concrete files with concrete required content
  defined earlier in the document (§6 for the ladder text, §5.11/§8.13 for
  the templates). §10.3's pilot table is explicitly labeled "Recommended
  pilot," and its Outcome bullet ("No current package is moved merely
  because §12 predicts its likely outcome") correctly hedges against
  decision 8/9 rather than assuming them. Enactable as written.
- **§8.13's package-adjudication record shape**, sampled directly: every
  section header in the template (Original intention, Current trigger and
  outcome, Driver and admission boundary, Behavior-unit dispositions table,
  Surviving package design, Inputs and outputs, Review and promotion
  policy, Alternatives considered, Final disposition, Validation evidence)
  traces to a defined term or step earlier in the document — §8.2's
  Step 1 fields for "Original intention"/"trigger"/"outcome," §5.11's YAML
  record for the disposition table's columns, §5.10's modifier table for
  "Final disposition" values. A dispatched actor filling this template has
  a defined source for every field. Not a finding.
- **§15 sampled items 3, 5, 9, 16, 20, 27, 30, 35, 37** (canonical-source
  existence, ladder ordering, source-citation requirement, at-least-one
  needs_input/respond run, falsifiable completion boundaries, separated
  structural/semantic reporting, pilot-before-corpus gating, justified
  validation tooling, close-out hypothesis reconciliation): each is
  checkable against a definition supplied earlier in the same document
  (§5, §6, §8.2–8.4, §9.1, §11.3) and none requires inventing a fact the
  proposal doesn't supply. Not a finding.
- **§7.2–7.4's required instruction shapes** (workflow authority envelope,
  stage-level bounded judgment, skill-level bounded judgment): each
  template's placeholder sections map onto fields the proposal defines
  elsewhere (§6.2–6.7 for the J-rungs, §5 for placement rationale). A
  dispatched Work rewriting one package's CONTEXT.md or SKILL.md has
  enough structure to fill these sections without inventing new
  vocabulary. Not a finding, independent of the §19-4 stage-template
  question noted in passing under F3's neighborhood (§7.3 presents the
  full local-section form as the only template shown, which happens to
  match the proposal's own recommended default for decision 4 — this is
  consistent, not silently contrary, so it did not rise to a separate
  finding).
