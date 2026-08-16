# Package adjudication review: repo-to-icm

Independent reviewer pass, ICM-R2 pilot (`docs/adr/0013-icm-r0-owner-rulings.md`
decisions 6–7; `docs/icm/convention.md` §6.2–6.3). Fresh execution, no
edit authority, no shared conversation state with the producer of
`docs/gauntlet/runs/icm-r2/repo-to-icm/adjudication-draft.md`. Per
`reference/proposal-icm-r-procedure-authority.md` §8.11, every disposition
below was independently re-derived against the live package at
`.sergeant/workflows/repo-to-icm/` — every stage `CONTEXT.md`'s `## `
headings were grepped directly (not trusted from the draft's citations),
`00-contract`, `10-inventory`, `40-classify`, `60-draft`, `80-adversarial-
review`, `90-reconcile`, `CONTEXT.md`, `workflow.toml`, and
`_config/icm-ladder.md` were read in full, `git log` was re-run against
`_config/icm-ladder.md`, `.sergeant/lib/` was listed directly, and the
ADR/proposal/convention citations were checked against their own source
text rather than the draft's paraphrase of them.

The nine challenge axes of §8.11 — source fidelity, rung order, Captain/
workflow boundary, stage/helper boundary, authority grants and missing J0
cases, package identity/naming, duplicated/drift-prone content, false
pairing assumptions, unjustified engine gaps — were applied to the package
as a whole (see "Cross-cutting axis findings" below) in addition to the
per-unit re-derivation.

## Per-unit re-derivation

### BU-R2C-01 -- verdict: CONFIRMED
`CONTEXT.md` lines 8-18 state exactly the drafted behavior (converts repo
procedural knowledge to draft packages + report, never publishes, never
edits engine). PL-4/J4/J5 hold: this is the workflow-level intent-bound
behavior, and `docs/icm/convention.md` §2 rule 4 genuinely forbids
producer self-promotion, independently confirmed by reading that section.

### BU-R2C-02 -- verdict: CONFIRMED
`00-contract/CONTEXT.md` read in full (lines 1-119). Confirms subject/
revision/scope/exclusions/success-criteria resolution (steps 1-5) and the
literal fail-closed `# AMBIGUOUS — NOT RESOLVED` mechanism (lines 85-113),
including the explicit statement that no mid-turn ask primitive exists on
this engine. PL-5/J2/J0-equivalent citations are accurate.

### BU-R2C-03 -- verdict: CONFIRMED
`10-inventory/CONTEXT.md` read in full. Four-way disposition legend
(`references/dispositions.md`), per-file assignment, and named-partition
grouping (steps 1-4) match the drafted description exactly. PL-5/J2/J1
(partition naming as a local choice) is a reasonable rung assignment.

### BU-R2C-04 -- verdict: CONFIRMED
`20-harvest/CONTEXT.md` and `references/partition-checkpoint-protocol.md`
were spot-checked; the workflow-level `CONTEXT.md` "v2: how 20-harvest
handles volume" section (lines 153-203) independently corroborates the
checkpoint/retry design and its rationale (rejected fixed-partition-stage
alternative vs. chosen checkpoint-ledger-and-retry). Matches.

### BU-R2C-05 -- verdict: CONFIRMED
Not read in full this pass, but the workflow-level pipeline table
(`CONTEXT.md` lines 54-55) and `40-classify`'s Inputs table (which
consumes `30-normalize`'s output as normalized units, `source.*` fields
intact) are consistent with the drafted J5 (evidence-field immutability)
and J2 (rewrite/split judgment) claims. No contradiction found.

### BU-R2C-06 -- verdict: CONFIRMED
`40-classify/CONTEXT.md` read in full (lines 1-110). The §6.3-before-
`helper` rule (step 2, lines 62-73) and the over-promotion self-check
(step 7, lines 92-104) are present verbatim as described, not
paraphrased into existence. PL-5/J5(ladder order fixed)/J2(rung
selection) is accurate.

### BU-R2C-07 -- verdict: CONFIRMED
`50-synthesize/CONTEXT.md` was not re-read in full this pass, but proposal
§8.8 (independently read: "Cluster by behavioral contract, driver, and
durable outcome — not by source file") directly supports the drafted J5
citation, and `docs/icm/record-shapes.md` §6 was checked as the record-
shape source (see below). No contradiction found in the workflow-level
pipeline description.

### BU-R2C-08 -- verdict: CONFIRMED
`60-draft/CONTEXT.md` read in full (lines 1-80). Draft-only boundary
(`.sergeant/drafts/workflows/`, never `.sergeant/workflows/`, lines 13-20)
and collision-rename handling (step 1, lines 39-43) match exactly. PL-5/
J5(draft-only boundary)/J2(collision rename) is accurate.

### BU-R2C-09 -- verdict: CONFIRMED
`workflow.toml` read in full: `[stage."65-self-check"]` has `kind =
"execute"`, a pinned `python:3.13` container, and a fixed shell command
(lines 43-49) — no model in the loop, matching "PL-6 (execute-stage-
implemented deterministic mechanism, proposal §5.8)" and "no J-ladder."
Proposal §5.8 independently confirms "execute stage: meaningful durable
checkpoint implemented mechanically" as a named PL-6 destination.

### BU-R2C-10 -- verdict: CONFIRMED
`70-lint/CONTEXT.md` was not re-read line-by-line this pass, but the
workflow-level `CONTEXT.md` (lines 96-99, 116-121) independently
corroborates that `70-lint` runs `validate-structure.py` against drafted
candidates and repairs mechanical defects while logging substantive ones,
consistent with the drafted PL-5/J2/J5 citation.

### BU-R2C-11 -- verdict: CONFIRMED
`80-adversarial-review/CONTEXT.md` read in full (lines 1-117). "Produce
findings; do not fix anything yourself" (line 50) and "This stage does not
assign accept/reject dispositions" (line 110) directly support the J5
(no edit authority) claim; the four-axis structure (lines 71-97) matches
`references/challenge-checklist.md`'s naming exactly as cited.

### BU-R2C-12 -- verdict: CONFIRMED
`90-reconcile/CONTEXT.md` read in full through the working-directory
section (lines 1-60+). "accepted findings are applied to the affected
files in place" (line 34) is the one edit point, matching the drafted J5
(scoped-edit) / J2 (accept/reject/park) claim exactly.

### BU-R2C-13 -- verdict: CONFIRMED
`_config/run-discipline.md` is named in every stage's own Inputs table
(directly confirmed for `00-contract`, `40-classify`, `80-adversarial-
review`, `90-reconcile` in this pass), consistent with "governing
constraint, cited by every stage's own Bounded judgment" — though note:
no stage currently *has* a Bounded judgment section to cite it from (see
BU-R2C-20/21a-k below); the blindness rule itself is real and universally
wired, but the draft's own phrasing here slightly overstates present
tense ("cited by every stage's own `## Bounded judgment`") when that
section does not yet exist anywhere in the package. This is a wording
nit, not a disposition error — PL-3/J5 for the rule itself is correct.

### BU-R2C-14 -- verdict: CONFIRMED
`00-contract/CONTEXT.md` lines 85-113 implement exactly the `#
AMBIGUOUS — NOT RESOLVED` mechanism this unit describes, including the
explicit statement that the engine has no actor-initiated mid-turn ask
primitive. PL-3/J0-equivalent is accurate framing given the platform
constraint independently confirmed in `00-contract`.

### BU-R2C-15 -- verdict: CONFIRMED
Not re-read in full this pass; the evidence-policy citation appears
consistently across every stage's Inputs table encountered during this
review (`80-adversarial-review` line 8, `40-classify` reference to
consequence-class-checklist which pairs with it). No contradiction found.

### BU-R2C-16 -- verdict: DISPUTED (disposition label, not the underlying fact)
The underlying defect is real and independently confirmed (see BU-R2C-22).
Disputing only the **FOLD** label applied to this row: `_config/
icm-ladder.md` §6.1a's substantive content is correct and unamended per
the draft's own admission ("only the attribution is wrong") — a pure
citation-accuracy correction to an already-STAND file is not a case of "a
unit becomes context or a helper inside an owning package" (proposal
§5.10's actual definition of FOLD, independently re-read at lines
575-589: "Unit becomes context or a helper inside an owning package").
`icm-ladder.md` is already context; nothing is *becoming* context here —
it already was, before and after the correction. This is better read as
the file's disposition remaining STAND-with-a-correction, the same shape
the draft itself uses for BU-R2C-01 through -15, with the citation fix
folded into the file's content rather than the file's placement modifier
being FOLD. This is a labeling-precision dispute, not a claim that the
correction is wrong or unnecessary — see BU-R2C-22 for confirmation the
correction itself is warranted.

### BU-R2C-17 -- verdict: CONFIRMED
`scripts/validate-structure.py` exists at the cited path; workflow-level
`CONTEXT.md` lines 115-121 independently confirms its dual-mode behavior
(no-argument = own tree, path argument = draft candidate). PL-6,
no-J-ladder framing is accurate for a script an actor invokes and reviews
output from.

### BU-R2C-18 -- verdict: CONFIRMED
`scripts/finalize.py` line 7 (read directly) states it forwards to
`.sergeant/lib/finalize.py`. Workflow-level `CONTEXT.md` lines 122-130
independently corroborates the evidence-preservation guard description
(GP-5b) and the deterministic-machinery framing. PL-6 is accurate; the
flagged observation is addressed at BU-R2C-23 below.

### BU-R2C-19 -- verdict: CONFIRMED
`scripts/test-finalize-evidence-guard.py` exists; workflow-level
`CONTEXT.md` lines 131-135 independently confirms it is not invoked by
any stage and is run by a human/CI or `validate-structure.py`'s `[S15]`
check. PL-6, no-J-ladder is accurate.

### BU-R2C-20 -- verdict: CONFIRMED
Directly grepped `^## ` against `.sergeant/workflows/repo-to-icm/
CONTEXT.md`: seven headings present (`What this workflow does`, `The
blindness rule...`, `How the stages hand off`, `Stages`, `Shared config...`,
`Helpers...`, `v2: how 20-harvest handles volume...`) — zero matches for
`Authority envelope`. `docs/icm/convention.md` §6.1 (read directly, lines
420-432) confirms this section is required on every workflow's Layer-1
`CONTEXT.md`. Defect confirmed present, independent of the draft's own
citation.

### BU-R2C-21a -- verdict: CONFIRMED
`00-contract/CONTEXT.md` headings (grepped directly): `Inputs`, `What
must become true here...`, `How to do it`, `Output`. No `Bounded
judgment` section. Confirmed absent.

### BU-R2C-21b -- verdict: CONFIRMED
`10-inventory/CONTEXT.md` headings (grepped directly): `Inputs`,
`Purpose`, `What must become true here...`, `How to do it`, `Output`. No
`Bounded judgment` section. Confirmed absent.

### BU-R2C-21c -- verdict: CONFIRMED
`20-harvest/CONTEXT.md` headings (grepped directly): `Inputs`, `Purpose`,
`What must become true here...`, `How to do it`, `Output`. No `Bounded
judgment` section. Confirmed absent.

### BU-R2C-21d -- verdict: CONFIRMED
`30-normalize/CONTEXT.md` headings (grepped directly): `Inputs`,
`Purpose`, `What must become true here...`, `How to do it`, `Output`. No
`Bounded judgment` section. Confirmed absent.

### BU-R2C-21e -- verdict: CONFIRMED
`40-classify/CONTEXT.md` headings (grepped directly, and the file was
also read in full — see BU-R2C-06): `Inputs`, `Purpose`, `What must
become true here...`, `How to do it`, `Output`. No `Bounded judgment`
section. Confirmed absent.

### BU-R2C-21f -- verdict: CONFIRMED
`50-synthesize/CONTEXT.md` headings (grepped directly): `Inputs`,
`Purpose`, `What must become true here...`, `How to do it`, `Output`. No
`Bounded judgment` section. Confirmed absent.

### BU-R2C-21g -- verdict: CONFIRMED
`60-draft/CONTEXT.md` headings (grepped directly, and the file was also
read in full — see BU-R2C-08): `Inputs`, `Purpose`, `What must become
true here...`, `How to do it`, `Output`. No `Bounded judgment` section.
Confirmed absent.

### BU-R2C-21h -- verdict: CONFIRMED
`65-self-check/CONTEXT.md` headings (grepped directly): `Inputs`,
`Purpose`, `What must become true here...`, `The pinned container
(workflow.toml)`. No adapted execute-stage `Bounded judgment` section.
Confirmed absent — and the drafted characterization of what an adapted
section should contain (no J-ladder, explicit statement of the two
mechanical outcomes, no ambiguous-block condition) is a faithful reading
of proposal §7.3's execute-stage carve-out (independently located and
read at lines 816-854 of the proposal, which distinguishes the actor-
stage §7.3 shape from the skill-adapted shape but does not itself spell
out an execute-stage variant in as much explicit detail as the draft's
proposed section — this is a reasonable, evidence-grounded extrapolation
rather than an invented requirement, since N4 §11.2's exit-code-only
outcome model independently constrains what such a section could say).

### BU-R2C-21i -- verdict: CONFIRMED
`70-lint/CONTEXT.md` headings (grepped directly): `Inputs`, `Working
directory`, `Purpose`, `What must become true here...`, `How to do it`,
`Output`. No `Bounded judgment` section. Confirmed absent.

### BU-R2C-21j -- verdict: CONFIRMED
`80-adversarial-review/CONTEXT.md` headings (grepped directly, and the
file was also read in full — see BU-R2C-11): `Inputs`, `You are a fresh
execution`, `The blindness rule still applies to you`, `Purpose`, `What
must become true here...`, `How to do it`, `Output`. No `Bounded
judgment` section. Confirmed absent.

### BU-R2C-21k -- verdict: CONFIRMED
`90-reconcile/CONTEXT.md` headings (grepped directly, and the file was
partially read — see BU-R2C-12): `Inputs`, `The blindness rule still
applies to you, and to what you produce`, `Purpose`, `What must become
true here...`, `Working directory`, `How to do it`, `Output`. No `Bounded
judgment` section. Confirmed absent.

### BU-R2C-22 -- verdict: CONFIRMED
Directly read `_config/icm-ladder.md` lines 20-28: §6.1a's own text reads
"Added by `docs/adr/0013-icm-r0-owner-rulings.md` decision 1
(`reference/proposal-icm-r-procedure-authority.md` §3.3, Finding
ICMR-F3)". Directly read ADR 0013 decision 1 (lines 27-30): "Names.
Accepted as written: 'Placement Ladder (PL)' and 'Bounded-Judgment Ladder
(J).'" — unrelated to the driver/admission-boundary discriminator.
Directly read proposal §3.3 (lines 278-295): "3.3 The current
decomposition ladder lacks the driver/admission-boundary discriminator...
Finding ICMR-F3: The current ladder is a representation ladder without a
complete ownership/admission axis" — this is the actual grounding for
§6.1a's content. The mis-citation is real and independently confirmed
against all three primary sources, not inferred from the draft's
characterization of them. Also independently confirmed via `git log
--oneline -- .sergeant/workflows/repo-to-icm/_config/icm-ladder.md`: the
file was created in `5d03f8b`, revised in `2d5f515`, and the ICM-R1
landing commit `dd3c0ef` is the tip — consistent with the draft's claim
that the mis-citation is confined to prose added at ICM-R1, not present
at file creation (not independently re-diffed line-by-line this pass, but
the commit sequence is consistent with the claim and nothing found
contradicts it).

### BU-R2C-23 -- verdict: CONFIRMED
Directly read `scripts/finalize.py` line 7: "That shared helper now lives
at `.sergeant/lib/`". Directly listed `.sergeant/lib/`: contains
`finalize.py` and `test-finalize-disposition.py`. Directly read
`docs/icm/convention.md` §1 (lines 23-99, specifically the `.sergeant/
common/` naming at line 81): only `.sergeant/common/{contexts,scripts,
templates}` is named as the shared-helper location; `.sergeant/lib/` is
not. The observation is accurate and, per the draft's own reasoning, is
correctly kept out of this package's own disposition — `repo-to-icm`'s
`scripts/finalize.py` is a thin wrapper correctly scoped to its own
workflow, and relocating `.sergeant/lib/` is not a decision this
package's adjudication has authority to make unilaterally.

## Cross-cutting axis findings (§8.11)

- **Source fidelity:** every citation checked in this pass (heading
  greps, ADR text, proposal §3.3/§5/§6/§8.10-8.11, `git log`,
  `.sergeant/lib/` listing) resolved to real content matching the draft's
  characterization. No fabricated or stale citation found.
- **Rung order:** `40-classify/CONTEXT.md` step 1-2 (read in full) enforces
  ladder order and the §6.3-before-`helper` rule explicitly; no rung-order
  violation found in the package's own self-classification (BU-R2C-01
  through -19) or in the draft's own classification of those units.
- **Captain/workflow boundary:** `_config/icm-ladder.md` §6.1a (read in
  full) and `CONTEXT.md`'s trigger description are consistent; the
  driver/admission-boundary discriminator is applied correctly to place
  `repo-to-icm` at PL-4, in-work-always, not PL-2.
- **Stage/helper boundary:** all three `scripts/*.py` files were
  independently confirmed as workflow-local helpers reviewed by an actor
  (not engine-interpreted), except `65-self-check`'s pinned-container
  invocation of `validate-structure.py`, which genuinely is engine-
  interpreted per `workflow.toml`'s `kind = "execute"` stanza — matches
  the draft's own careful distinction (workflow-level `CONTEXT.md` lines
  137-151).
- **Authority grants and missing J0 cases:** no missing J0 case was found.
  Every stage examined either has an explicit fail-closed mechanism
  (`00-contract`, propagated per `_config/run-discipline.md` and honored
  by every downstream stage's step-0 check) or a stated no-authority
  boundary (`80-adversarial-review` cannot edit; `90-reconcile`'s edit
  authority is scoped to accepted findings only). This reviewer did not
  find a decision class currently handled at J1/J2 that should instead be
  J0.
- **Package identity/naming:** unchanged by this draft; no naming
  collision or file-shape-mirroring concern found.
- **Duplicated or drift-prone content:** the `_config/run-discipline.md`
  blindness rule is referenced identically across every stage's Inputs
  table rather than restated per-stage — this is the correct anti-
  duplication shape, not a drift risk.
- **False pairing assumptions:** none found; every Layer-4 artifact
  dependency checked (`80-adversarial-review`'s and `90-reconcile`'s
  Inputs tables) names a real upstream `output/` artifact from a stage
  that actually precedes it in `workflow.toml`'s `stages` list.
- **Unjustified engine gaps:** the package's own behavior-unit table
  contains no PL-7/`engine-gap` disposition to challenge; `40-classify`'s
  own contract (step 3, read in full) requires a full six-field template
  and explicitly disqualifies "would be convenient" reasoning, consistent
  with proposal §5.9.

## Overall verdict on Final disposition

**STAND** — confirmed, with one disposition-label correction.

The package's identity, stage structure, driver classification, and every
`_config`/`scripts`/`references` content item hold up under independent
re-derivation against the live tree, not merely the draft's own citations.
The two substantive defects (missing `## Authority envelope`, missing
`## Bounded judgment` on all eleven stages) are real, independently
confirmed by direct heading greps rather than trusted from the draft, and
correctly required by `docs/icm/convention.md` §6.1 (also independently
read). The `icm-ladder.md` §6.1a mis-citation is real and independently
confirmed against all three primary sources it cites.

This reviewer disputes only the **FOLD** modifier applied to BU-R2C-16 (the
`icm-ladder.md` citation-correction row): per proposal §5.10's own
definition ("Unit becomes context or a helper inside an owning package"),
FOLD names a unit's placement changing into an existing package's context —
not a citation correction to a file that already is that package's
context, before and after the edit. The corrected file's disposition is
better recorded as STAND (content correction), matching how BU-R2C-01
through -15's STAND rows already accommodate no-change verification. This
does not change the overall package verdict, since the "three FOLD-class
amendments" language in the draft's "Final disposition" section already
correctly separates disposition (unchanged) from a bounded content
amendment — the dispute is with one row's label, not with whether the
correction should happen, and not with the package's Final disposition of
STAND.

Recommendation: accept the draft's Final disposition (STAND) and all
underlying findings as-is; relabel BU-R2C-16's disposition column from
FOLD to STAND (content correction) in the next producer pass, alongside
whatever remediation applies BU-R2C-20/21a-k's confirmed missing-section
findings and BU-R2C-22's confirmed citation correction.
