# Package review: prototype (independent adversarial review)

ICM-R3 reviewer pass, independent of the producer who wrote
`docs/gauntlet/runs/icm-r3/prototype/adjudication-draft.md`. Method per
`reference/proposal-icm-r-procedure-authority.md` §8.11 (challenge source
fidelity, rung order, Captain/workflow boundary, stage/helper boundary,
authority grants and missing J0 cases, package identity/naming, duplicated
or drift-prone content, false pairing assumptions, unjustified engine
gaps); rungs per §5/§6; record shape per `docs/icm/record-shapes.md` §6.
This reviewer re-derived every disposition against the live package
content (`.sergeant/workflows/prototype/`), the current upstream source
(`reference/sergeant-upstream/.agents/skills/prototype/{SKILL.md,LOGIC.md,
UI.md}`), and the N1 corpus's own extraction records
(`reference-corpus/behavior-units/P3.ndjson`,
`reference-corpus/shared-context-map.md`, `reference-corpus/synthesis.md`)
— not from the producer's own citations. No edit authority was exercised;
this record is review only.

## Headline finding

The producer's draft is well-sourced and its 27 non-BU-PROTO-28
dispositions hold up under independent re-derivation. **BU-PROTO-28 is
DISPUTED**: the draft characterizes all five upstream "Rules that apply to
both" as never extracted at N1 ("a harvest gap under proposal §8.4"). That
is true for only one of the five. The other four
(`reference-corpus/behavior-units/P3.ndjson` lines 15-18, BU-P3-015
through BU-P3-018) were extracted at N1 with an explicit recorded
representation of `shared-context`, `workflow: prototype`, and a rationale
that specifically warns against duplicating the rule text into each
branch's own stage doc ("avoids two near-duplicate copies drifting
apart" / "a single shared-context entry is the non-duplicating
representation"). This is a **placement/promotion gap** (units extracted
at N1, never carried into the published workflow or into
`docs/gauntlet/promoted-provenance/prototype.md`), not a harvest gap — and
the draft's own proposed remediation ("most naturally to both
`20L-build-logic` and `20U-build-variants`") would recreate exactly the
duplication N1's synthesis pass rejected. This does not change the
package's overall STAND disposition, but it does change the correct
remediation shape and is exactly the "duplicated or drift-prone content"
class of finding §8.11 asks a reviewer to catch.

## Behavior-unit dispositions

### BU-PROTO-01 -- verdict: CONFIRMED
Package-identity statement (throwaway workflow, branches by question
nature). Re-derived directly from `CONTEXT.md` "Purpose"/"Trigger" and
`index.md`; matches `BU-P3-010`/`BU-P3-011`
(`reference-corpus/behavior-units/P3.ndjson` lines 10-11) verbatim in
substance. PL-4/STAND agreed.

### BU-PROTO-02 -- verdict: CONFIRMED
`00-select-branch/CONTEXT.md` behavior contract matches `BU-P3-012`
(SKILL.md line 12, "Identify which question is being answered — from the
user's prompt, the surrounding code, or by asking if the user is
around"). J2 (classify from available evidence) is the correct rung: the
package delegates this class of decision by name in the stage's own
Behavior contract prose. STAND/PL-5 agreed.

### BU-PROTO-03 -- verdict: CONFIRMED
Matches `BU-P3-013` and SKILL.md's UI-branch routing sentence. J1
(mechanical routing once classified) is right — once the type is decided,
which branch runs is not itself a further judgment call.

### BU-PROTO-04 -- verdict: NEEDS-REVISION (low severity)
Core disposition (STAND, PL-5, J4-when-reachable with a named J2
heuristic fallback) is independently confirmed against SKILL.md's "If the
question is genuinely ambiguous and the user isn't reachable, default to
whichever branch better matches the surrounding code... and state the
assumption." One imprecision: the draft calls this "structurally
identical in kind to `40-drive-gates`'s auto-fix/ask-user split." Checked
directly (`.sergeant/workflows/validate-and-ship/40-drive-gates/
CONTEXT.md` lines 102-116): that stage's actual split is J2
(auto-fix/no-op, delegated) versus **J0** (ask-user, full escalation) —
not a J4-primary/J2-fallback pattern. The analogy is loose (both are
"delegate unless a higher authority must be consulted" shapes) but as
worded overstates structural identity with a stage whose fallback rung is
J0, not J2. Recommend rewording to "the same general shape of an ordered
fallback when higher authority is unavailable" without naming
`40-drive-gates` as structurally identical.

### BU-PROTO-05 -- verdict: CONFIRMED
Matches `BU-P3-021` / LOGIC.md step 1 ("write down what state model and
what question you're prototyping... before writing code"). J2, PL-5,
STAND agreed.

### BU-PROTO-06 -- verdict: CONFIRMED
Matches `BU-P3-020` / LOGIC.md header. J2, PL-5, STAND agreed.

### BU-PROTO-07 -- verdict: CONFIRMED
Matches `BU-P3-022` / LOGIC.md step 3 intro. The "bounded by J5 below"
framing correctly anticipates BU-PROTO-08's purity constraint governing
this same interface. STAND agreed.

### BU-PROTO-08 -- verdict: CONFIRMED
Matches `BU-P3-023` / LOGIC.md step 3 ("Keep it pure: no I/O, no terminal
code, no `console.log` for control flow... nothing flows the other
direction"). J5 is correct: this is phrased as a hard constraint on the
module's shape, not a delegated choice.

### BU-PROTO-09 -- verdict: CONFIRMED
Matches `BU-P3-024`/`BU-P3-025` / LOGIC.md step 4 (full-frame re-render,
stated once as the step's opening instruction and once in the numbered
behaviour list). Merging into one BU is defensible — the source itself
states the same rule twice within one step, so folding avoids inflating
the unit count for a single rule. J5 (source says "should always see one
stable view," a must-level constraint) agreed.

### BU-PROTO-10 -- verdict: CONFIRMED
Matches `BU-P3-026` / LOGIC.md step 5. J2 (choose the invocation name)
agreed — the source only requires "one command," not a specific name.

### BU-PROTO-11 -- verdict: CONFIRMED
Matches `BU-P3-029` / UI.md header. J2, PL-5, STAND agreed.

### BU-PROTO-12 -- verdict: CONFIRMED
Matches `BU-P3-030` / UI.md "Two sub-shapes — strongly prefer sub-shape
A." J2 with a named default preference is the correct characterization —
the source states a default ("strongly prefer... this is the default")
while still allowing sub-shape B when justified.

### BU-PROTO-13 -- verdict: CONFIRMED
Matches `BU-P3-031` / UI.md sub-shape B section ("don't invent a new
top-level structure... Name it so it's obviously a prototype"). J5
(governing) agreed — both clauses are phrased as musts.

### BU-PROTO-14 -- verdict: CONFIRMED
Matches `BU-P3-032` / UI.md process step 1 ("Default to 3 variants...cap
there"). J5 (the cap) + J2 (choosing within the bound) is the right split.

### BU-PROTO-15 -- verdict: CONFIRMED
Matches `BU-P3-033` / UI.md process step 2. J5 (structural divergence
required) + J2 (judging whether divergence is real) agreed.

### BU-PROTO-16 -- verdict: CONFIRMED
Matches `BU-P3-034` / UI.md process step 4 ("Hidden in production
builds... so a stray prototype merge can't ship the bar to users"). J5,
safety-relevant, agreed.

### BU-PROTO-17 -- verdict: CONFIRMED
Matches `BU-P3-038` / UI.md Anti-patterns ("Wiring variants to real
mutations... point it at a stub"). J5 agreed.

### BU-PROTO-18 -- verdict: CONFIRMED
Matches `BU-P3-035` / UI.md process step 5. J2 agreed.

### BU-PROTO-19 -- verdict: CONFIRMED
Matches `BU-P3-019` / SKILL.md item 6. Independently re-checked the J4-gap
analysis against all six rungs (§6.2-6.7): no governing constraint,
explicit user grant, settled record, or named delegation in this
package's own content answers "who decides the question is answered" at
the workflow/logic-branch level, only at the UI sub-shape level
(`BU-P3-036`/`037`). The decision **not** to escalate to J0 is also
independently confirmed: `40-capture`'s output lands on the Work branch,
not `main`, and passes through this repository's normal review path
before merge (`proposal §9.7`; confirmed no push/PR/merge action exists
anywhere in `40-capture/CONTEXT.md`'s behavior contract). This correctly
distinguishes the case from `validate-and-ship`'s BU-VAS-15, where the
gap sat directly in front of an autonomous, externally-visible action.
"STAND, required in-place amendment" agreed.

### BU-PROTO-20 -- verdict: CONFIRMED
Matches `BU-P3-027` / LOGIC.md step 7. Same J4-gap analysis as
BU-PROTO-19 applies correctly. Note: the draft uses "in-place amendment
recommended" here versus "required" for BU-PROTO-19 — both describe the
same remediation item (item 4 under "Surviving package design"); this is
a wording inconsistency, not a substantive disagreement, and does not
change the verdict.

### BU-PROTO-21 -- verdict: CONFIRMED
Matches `BU-P3-028` / LOGIC.md Anti-patterns ("Don't ship the TUI shell
into production"). J5 agreed.

### BU-PROTO-22 -- verdict: CONFIRMED
Matches `BU-P3-036` / UI.md process step 6, sub-shape A. J4 trigger
("the user has picked a winning variant") is explicit in the source
("Once a variant has won") — correctly identified as the one branch where
the capture trigger is already unambiguous. J5 (non-winners must not
survive on `main`) agreed.

### BU-PROTO-23 -- verdict: CONFIRMED
Matches `BU-P3-037` / UI.md process step 6, sub-shape B. Same shape as
BU-PROTO-22, confirmed.

### BU-PROTO-24 -- verdict: CONFIRMED
Matches `BU-P3-039` / UI.md Anti-patterns. J5 (quality gate) agreed.

### BU-PROTO-25 -- verdict: CONFIRMED
Independently re-verified by reading all six stage `CONTEXT.md` files in
full: every one carries only the generic "## Judgment required" boilerplate
paragraph ("This is an actor stage (ladder §6.4)...") with no named J2
delegations, J1 local choices, or J0 escalation triggers in the
`convention.md` §6.1 required shape. `convention.md` §6.1 and ADR 0013
decision 4 confirmed directly (read in full) to require this section
"always present... omission is never ambiguous." STAND, in-place
amendment required, agreed.

### BU-PROTO-26 -- verdict: CONFIRMED
Independently re-verified: `.sergeant/workflows/prototype/CONTEXT.md`
contains no `## Authority envelope` heading anywhere in the file.
`convention.md` §6.1 requirement confirmed directly. STAND, in-place
amendment required, agreed.

### BU-PROTO-27 -- verdict: NEEDS-REVISION (terminology only)
The underlying defect is independently confirmed: `CONTEXT.md`'s
"Provenance" section reads "See `provenance.md` for the complete
stage-to-behavior-unit mapping," and no `provenance.md` file exists
anywhere under `.sergeant/workflows/prototype/` (verified by directory
listing); the actual citation trail is at
`docs/gauntlet/promoted-provenance/prototype.md`. However, the draft
tags this **FOLD** as its disposition modifier. Per
`reference/proposal-icm-r-procedure-authority.md` §5.10, `FOLD` means "a
unit becomes context or a helper inside an owning package" — a placement
outcome for a behavior unit, not a category for correcting a dangling
file-path string. This is not a placement question at all (the draft's
own prose says as much: "not a placement change"), so no §5.10 modifier
actually applies; the draft's disposition column should read something
like "N/A (editorial correction)" rather than reuse a placement modifier
for a non-placement fix. Low severity — does not change what needs to
happen, only how it is labeled.

### BU-PROTO-28 -- verdict: DISPUTED
Independently re-derived from `reference-corpus/behavior-units/P3.ndjson`
(the N1 harvest record itself, not the promoted-provenance file the
draft checked against) and confirmed the draft's classification of the
gap is materially wrong for four of the five items it names.

The draft claims all five of SKILL.md's "Rules that apply to both" items
1-5 were "never extracted as behavior units for this package at all... a
harvest gap under proposal §8.4." Checked directly against
`reference-corpus/behavior-units/P3.ndjson`:

- Item 1 (throwaway naming/location, routing convention) **was**
  extracted as `BU-P3-015`, `representation: "shared-context"`,
  `workflow: "prototype"`.
- Item 2 (one command to run) **was** extracted as `BU-P3-016`, same
  representation — in addition to the branch-specific `BU-P3-026`
  (LOGIC.md step 5) the draft already cites. N1 extracted this rule
  twice, at two granularities; the draft's framing ("never extracted")
  misses the general-purpose extraction entirely.
- Item 3 (no persistence by default) **was** extracted as `BU-P3-017`,
  same representation.
- Item 4 (skip the polish — no tests, minimal error handling, no
  abstractions) has **no** corresponding `BU-P3-0xx` unit anywhere in
  `P3.ndjson` (confirmed by grep across the file for "polish"/"no
  tests"/"no abstractions" — the only hit is `BU-P3-039`, a different,
  narrower rule about rewriting winning code at capture time, not the
  general "skip polish while building" rule). This one item is a genuine
  harvest gap; the draft is correct only for this item.
- Item 5 (surface the state) **was** extracted as `BU-P3-018`, same
  representation.

`reference-corpus/shared-context-map.md` line 346 and
`reference-corpus/synthesis.md` line 758 both record these four units
(`BU-P3-015`-`018`) under "Workflow-local contexts (kept local per
§6.6)" — N1's synthesis pass already decided these four belong inside
the `prototype` workflow as **one shared, non-duplicated context entry**,
with an explicit rationale recorded in the ndjson itself: "a single
shared-context entry is the non-duplicating representation" (BU-P3-016),
"avoids two near-duplicate copies drifting apart" (BU-P3-015), "kept as
one shared-context entry at the workflow level instead" (BU-P3-018).

This changes the finding's category. It is not, for four of the five
items, a **harvest** gap (proposal §8.4's Harvest step — a unit never
extracted at all). It is a **placement/promotion** gap: units that were
already extracted and already assigned a representation
(`shared-context`, workflow-local) at N1, but that representation was
never actually built — no shared-context file exists anywhere under
`.sergeant/workflows/prototype/`, and `docs/gauntlet/promoted-provenance/
prototype.md` (the document the draft did check) never carries them
forward at all, silently dropping four already-adjudicated units between
N1 and promotion.

This also means the draft's own proposed remediation is the wrong shape
for four of the five items. "Surviving package design" item 5 recommends
adding the harvested rules "most naturally to both `20L-build-logic` and
`20U-build-variants`" — i.e., duplicating the same rule text into two
separate stage `CONTEXT.md` files. That is precisely the drift risk
N1's own synthesis pass considered and rejected for these four units
("avoids two near-duplicate copies drifting apart"). Per §8.11's own
"duplicated or drift-prone content" challenge, an adversarial reviewer
should not let this recommendation stand as written: the correct
remediation for items 1, 2, 3, and 5 is a single workflow-local shared
context (referenced by both `20L-build-logic` and `20U-build-variants`,
consistent with how `@@name` shared contexts are used elsewhere in this
repository per `convention.md` §4), carrying forward `BU-P3-015`,
`016`, `017`, `018` with their already-recorded N1 rationale — not two
independent copies. Only item 4 ("skip the polish") is a genuine harvest
gap requiring fresh extraction and placement, as the draft describes.

Disposition: **STAND at package level is still correct** (this remains a
content-amendment finding, not a rehome/split trigger), but the
draft's *sub*-disposition — "harvest gap under proposal §8.4" applied
uniformly to five items — should be revised to distinguish the
already-extracted-but-unplaced four (a promotion/citation-trail defect
in `docs/gauntlet/promoted-provenance/prototype.md`, which should have
cited `BU-P3-015`-`018` alongside its 26 other citations) from the one
genuine harvest gap (item 4). The remediation in "Surviving package
design" item 5 should be rewritten to specify one shared-context
placement, not per-branch duplication.

## Other §8.11 checks (no additional findings)

- **Captain/workflow boundary:** independently re-checked the "Rejected
  alternative reading" for `00-select-branch`/`10-record-question`
  against the execution-surface test (`convention.md` §2a, read in full).
  Both stages have declared Inputs/outputs and a bounded per-run outcome;
  neither negotiates whether Work should exist. Confirmed correctly
  rejected as Captain-shaped.
- **Stage/helper boundary:** no deterministic helper or `kind = "execute"`
  stage exists anywhere in this package's `workflow.toml`; nothing here
  was mischaracterized as a stage that should have been a helper, or vice
  versa.
- **Package identity/naming:** `prototype` has no naming collision;
  confirmed by directory search (`find . -iname "*prototype*"`) — the
  only other hits are the N1 provenance record, the upstream source, and
  an unrelated `n2-run4` draft directory tree, none of which contend for
  the same live package identity.
- **False pairing assumptions:** the "Known consumer" claim
  (`worker-mission/20-implement/CONTEXT.md` line 29 naming `prototype` as
  one of five disciplines) was independently re-read and confirmed
  verbatim, correctly characterized as context composition rather than
  true nested-workflow invocation per `convention.md` §4.
- **Unjustified engine gaps:** the rejection of a PL-7 reading for the
  `20L`/`20U` branch is independently confirmed — the current linear
  `workflow.toml` already represents both stage directories with one a
  documented no-op, so no new runtime capability is actually required
  today.
- **Retriage citation:** `docs/icm/retriage-2026-08-11.md` line 26 was
  read directly and matches the draft's quote verbatim.
- **`docs/icm/agents-invariant-dispositions.md` citation:** `BU-1080`
  through `BU-1084` were read directly and confirmed to record exactly
  the "not-adopted... belongs to skill: prototype" disposition the draft
  describes for the harvested-but-unplaced rules — and, notably,
  `BU-1084`'s statement text is close to word-for-word the same as
  `BU-P3-018` (surface-the-state), independent evidence that this rule
  was adjudicated as prototype-owned at least twice and still never
  landed in the live package.

## Overall verdict on Final disposition

**STAND — confirmed**, with one correction to the record. The package's
structure (six stages, PL-4/PL-5, no rehome/split/harvest trigger at the
package level) and all remediation items except item 5 are independently
re-derived and hold. Item 5 of "Surviving package design" needs revision
before this record is treated as settled: the disposition category
for `BU-PROTO-28` should split into (a) a promotion/citation-trail defect
for `BU-P3-015`/`016`/`017`/`018` (already extracted at N1, never carried
into the live package or `promoted-provenance/prototype.md`, correct
remediation is one shared, non-duplicated workflow-local context) and (b)
a genuine harvest gap for item 4 only ("skip the polish," never extracted
at N1 at all). This is a content-accuracy correction to the adjudication
record itself, not a reason to change the package's STAND verdict.
