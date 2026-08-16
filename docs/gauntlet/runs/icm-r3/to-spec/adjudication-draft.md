# Package adjudication: to-spec

ICM-R3 full-reconciliation package, `docs/adr/0013-icm-r0-owner-rulings.md`;
method per `reference/proposal-icm-r-procedure-authority.md` §8 (`§10.4`
scope); record shape per `docs/icm/record-shapes.md` §6. Producer pass
only — independent review is a separate step (§8.11 of the proposal;
§6.2/6.3 of `docs/icm/convention.md`) and has not run yet. This record is
itself draft — it does not self-promote (ADR 0013 decision 6, decision 7).
No prior ICM-R hypothesis existed for this package (it was not in the
ICM-R2 pilot corpus); this is a fresh investigation.

## Original intention

Turn a plan/design already discussed with the user into a single published
spec ticket — synthesized, not interviewed — with a minimal test-seam plan
confirmed by the user, written on a fixed template, and published to the
project issue tracker carrying the `ready-for-agent` triage label with no
further triage needed (`.sergeant/workflows/to-spec/CONTEXT.md` "Purpose";
`index.md` description). Promoted into the N1 reference corpus as
candidate **W31** (`docs/gauntlet/contracts/N1.md`,
`docs/icm/promotion-spec-2026-08-11.md`), decomposed from
`reference/sergeant-upstream/.agents/skills/to-spec/SKILL.md` per
`reference-corpus/synthesis.md` §1, with the full citation trail archived
at `docs/gauntlet/promoted-provenance/to-spec.md`. This ICM-R3 pass does
not re-run that N1 extraction; it applies the Placement and
Bounded-Judgment ladders on top of the already-cited N1 content, checks
compliance with ADR 0013's rulings, and re-verifies every citation against
the **current** content of both this package's own tree and everything it
delegates to (the upstream `SKILL.md` and the sibling `repo-to-icm`
workflow it makes a factual claim about).

## Current trigger and outcome

Trigger (workflow-level, both stages): "A design needs to be turned into a
spec-shaped ticket before implementation" (`CONTEXT.md` Trigger;
`00-gather-context/CONTEXT.md`, `10-sketch-seams/CONTEXT.md`).

Two ordinary actor stages, pinned in `workflow.toml`
(`stages = ["00-gather-context", "10-sketch-seams"]`, matching the
directory listing lexically per `docs/icm/convention.md` §1 rule 4):

- `00-gather-context` — context is synthesized from the existing
  conversation and codebase exploration, never gathered by a separate
  interview pass.
- `10-sketch-seams` — the fewest new seams at the highest possible seam
  are sketched and confirmed with the user; once confirmed, the spec is
  written on a fixed template, published to the tracker, and labeled
  `ready-for-agent` with no further triage required. This stage also
  performs the write-and-publish work folded in from a demoted terminal
  stage (N1 adjudication A4, see below).

Outcome: a published, `ready-for-agent`-labeled spec issue exists, whose
content was synthesized (not interview-gathered) and whose implementation
shape commits to a user-confirmed, minimal seam plan before publication.

## Driver and admission boundary

Driver: **stage actor**, both stages. Admission boundary: **in-Work** —
the workflow receives an already-defined intent ("turn this
conversation/design into a spec") and executes durably from admission to
a terminal published-artifact outcome; it is not live Captain dialogue
about what Work should exist (`docs/icm/convention.md` §2a execution-
surface test: "would a human type `sgt run '<intent>' --workflow
to-spec`?" — yes, once a design exists to synthesize from). Both stages
are already labeled in the package's own table as "actor-stage (§6.4,
judgment)," matching this pass's independently re-derived rung (see
Behavior-unit dispositions below).

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| BU-TS-01 | `CONTEXT.md` (Purpose) — turn a plan/design into a published spec ticket: gathered context, sketched seams, confirmed with the user, published on template | PL-4 | J5 (contract-level: synthesis-only, never interview — see BU-TS-02) | STAND | `to-spec` (workflow) |
| BU-TS-02 | `00-gather-context/CONTEXT.md` — synthesis-only: do not interview the user, write the spec from what has already been discussed and codebase exploration — `BU-P4-050`, `reference/sergeant-upstream/.agents/skills/to-spec/SKILL.md` (L3, L7) | PL-5 | J2 (delegated: how to synthesize context from conversation + exploration without a separate interview) | STAND | `00-gather-context` |
| BU-TS-03 | `00-gather-context/CONTEXT.md` — explore the repo before drafting (if not already done); use the project's glossary vocabulary; respect ADRs in the touched area — `BU-P4-051`, same source (Process step 1, L13) | PL-5 | J2 (delegated: which exploration is sufficient; which ADRs are in scope) + J3 (accepted ADRs, once identified, are settled records to respect, not reopen) | STAND | `00-gather-context` |
| BU-TS-04 | `10-sketch-seams/CONTEXT.md` — sketch fewest new seams at highest possible seam before writing the implementation section — `BU-P4-052`, same source (Process step 2, L15) | PL-5 | J2 (delegated: seam-count/seam-height tradeoff judgment) | STAND | `10-sketch-seams` |
| BU-TS-05 | `10-sketch-seams/CONTEXT.md` — confirm sketched seams with the user before finalizing — `BU-P4-053`, same source (Process step 2, L17) | PL-5 | J2 (delegated: when/how to present the seam plan for confirmation) narrowing to J4 for the confirmation's outcome itself (the user's answer is the actual decision the stage then applies without reconfirming) | STAND | `10-sketch-seams` |
| BU-TS-06 | `10-sketch-seams/CONTEXT.md` Helper section — write the spec on the fixed template, publish to the tracker, apply `ready-for-agent` without requiring additional triage — `BU-P4-054`, same source (Process step 3, L19) | PL-6 (folded helper, not a standalone checkpoint — see N1 A4 below) | J3 (the template shape itself, once cited, is a settled record to reuse verbatim, not redesign) + J2 (delegated: filling the template's sections from the synthesized/confirmed content) | STAND, **in-place amendment required** — see "The missing template gap" below | `10-sketch-seams` (helper) |
| BU-TS-07 (gap) | `reference/sergeant-upstream/.agents/skills/to-spec/SKILL.md` line 9 — "The issue tracker and triage label vocabulary should have been provided to you — run `/setup-matt-pocock-skills` if not." Present in the current upstream source; **never normalized into any behavior unit** in either stage's `CONTEXT.md` | N/A (harvest omission, not yet a placement question) | **J0 — not delegated, conflicting, or risk-changing** (see "The tracker/label-vocabulary gap" below) | STAND at package-identity level; the missing content is **not drafted by this producer** | `00-gather-context/CONTEXT.md` (owner TBD) |
| BU-TS-08 (stale claim) | `10-sketch-seams/CONTEXT.md` line 34 — "No `kind = \"execute\"` stage exists in the current engine, so the acting harness performs the write-and-publish operation itself" | N/A (authoring-accuracy defect, not a placement question) | J5 (`docs/icm/convention.md` and the N1/N4 record require normalized statements to be factually accurate; a false premise inside a folding rationale is a defect regardless of whether the fold's conclusion still holds) | STAND, **in-place correction required** — see "The stale execute-stage claim" below | `10-sketch-seams/CONTEXT.md` |
| BU-TS-09 (format gap) | `CONTEXT.md` (no `## Authority envelope` section); `00-gather-context/CONTEXT.md` and `10-sketch-seams/CONTEXT.md` (both carry the generic `## Judgment required` boilerplate paragraph, not a `## Bounded judgment` section with named J2/J1/J0 classes) | N/A (authoring-format compliance) | J5 (`docs/icm/convention.md` §6.1: every workflow `CONTEXT.md` carries an `## Authority envelope` section; every actor stage's `CONTEXT.md` carries a `## Bounded judgment` section, "always present... omission is never ambiguous") | STAND, **in-place amendment required** | `CONTEXT.md` and both stage `CONTEXT.md` files |

## The missing template gap (BU-TS-06, full record)

The Behavior contract in `10-sketch-seams/CONTEXT.md` twice refers to "the
fixed spec template" and "the fixed template," and `output/README.md`
repeats "a fixed template published to the tracker." **The template's
actual content does not exist anywhere in this package's own tree** — no
`references/` or `_config/` directory, no Layer 3 file at all
(`find .sergeant/workflows/to-spec -type f` returns only the six files
already inventoried above). The template body (`Problem Statement`,
`Solution`, `User Stories`, `Implementation Decisions`, `Testing
Decisions`, `Out of Scope`, `Further Notes`, each with authoring guidance)
exists only in `reference/sergeant-upstream/.agents/skills/to-spec/
SKILL.md` lines 21-75, inside `<spec-template>` tags.

This is a real structural gap, not a stylistic preference. `docs/icm/
convention.md` §1a rule 2 defines Layer 3 as "reference material STABLE
ACROSS RUNS... edited only to change every future run" — exactly what a
fixed template is — and other admitted packages already follow this
pattern for the same kind of dependency (e.g.
`.sergeant/workflows/repo-to-icm/60-draft/references/
draft-package-template.md`). A stage actor executing `10-sketch-seams`
today has no in-package way to know the template's actual section names
or authoring guidance; it would have to either recall the upstream
`reference/sergeant-upstream` file from outside its declared Inputs table
(a Layer-3-reference-as-Inputs-table violation, `convention.md` §1a rule
1) or improvise a template shape from the one-line paraphrase, which is
exactly the "mechanism vs. behavioral intent" loss `record-shapes.md` §3
rule 1's normalization discipline exists to prevent.

**Recommended remediation** (not performed by this producer, since no
placement/authority verdict changes and in-place content edits are the
correct next step per the validate-and-ship ICM-R2 precedent): add
`10-sketch-seams/references/spec-template.md` containing the template
verbatim from the cited upstream source, and add it to
`10-sketch-seams/CONTEXT.md`'s Inputs table as a Layer 3 dependency.

## The tracker/label-vocabulary gap (BU-TS-07, full record)

`SKILL.md` line 9 states a precondition this package's Behavior contracts
never restate: the issue tracker identity and the triage label vocabulary
(of which `ready-for-agent` is one member) are assumed already provided,
with `/setup-matt-pocock-skills` as the upstream's own fallback to
establish them. Neither `00-gather-context/CONTEXT.md` nor
`10-sketch-seams/CONTEXT.md` names where a `to-spec` run in this
repository/estate learns which tracker to publish to or what its label
vocabulary is. `/setup-matt-pocock-skills` does not exist in this
codebase (no `.claude/commands`, `skills/`, or `.sergeant/` match for that
name) — the upstream's own escape hatch is not available here.

This is confirmed by comparison, not merely hypothesized: the sibling
`triage` workflow independently treats reaching the same `ready-for-agent`
disposition as requiring "posting a structured agent brief comment"
(`.sergeant/workflows/triage/50-apply-outcome/CONTEXT.md`, `BU-P3-069`,
citing `triage`'s own separate upstream `SKILL.md` line 79) — a
requirement `to-spec`'s own upstream source does not carry ("Apply the
`ready-for-agent` triage label - no need for additional triage," SKILL.md
line 19). The two packages are not necessarily in conflict — a
fully-written spec plausibly *is* self-sufficient in a way a
freshly-triaged issue is not — but nothing in either package's own content
states that reconciliation explicitly; it is this producer's inference,
not a settled record.

**Rungs checked (bounded-judgment.md order), for this producer's own
classification of the gap's current state — not a resolution of the
underlying policy:**

- **J5** — No governing constraint inside this package's own content
  states how tracker identity or label vocabulary is established for a
  given project/estate.
- **J4** — No explicit user or bound-Work decision is visible to either
  stage that would supply this; a Work's brief could in principle name the
  tracker, but no stage names "tracker identity" or "label vocabulary" as
  a decision class it consumes.
- **J3** — No settled record (ADR, prior stage output, pinned spec) fixes
  which tracker or label vocabulary this repository/estate uses for
  `to-spec` runs.
- **J2** — Neither stage's Behavior contract names "which tracker/which
  labels" as a delegated decision class.
- **J1** — Does not apply: publishing to the wrong tracker, or applying a
  triage label the target tracker does not recognize, is exactly the kind
  of externally-visible, not-easily-reversible effect `bounded-judgment.md`'s
  J1 definition excludes.

**Conclusion: J0**, as the package stands today — a description of the
gap, not a design recommendation. This producer does not draft the
missing content, per the same reasoning the validate-and-ship ICM-R2
precedent used for its own push/pr/ci gap: the underlying question (how
does this repository/estate establish tracker/label vocabulary for a
`to-spec` run, and should the package name a fallback when it is
unestablished) is a live, separate owner decision, not one this producer
is entitled to author on the owner's behalf.

## The stale execute-stage claim (BU-TS-08, full record)

`10-sketch-seams/CONTEXT.md` line 34 currently reads: "No `kind =
\"execute\"` stage exists in the current engine, so the acting harness
performs the write-and-publish operation itself." This is false as of
this branch. `.sergeant/workflows/repo-to-icm/workflow.toml` defines
`[stage."65-self-check"]` with `kind = "execute"`, a live execute stage
added at MVP-2 lane D3 (`repo-to-icm/workflow.toml` header comment,
2026-08-12) that runs `scripts/validate-structure.py` in a pinned
container. This is the identical stale claim already caught and corrected
in this same ICM-R3 wave's `research/00-investigate` stage contract (see
that stage's own "Rung-rationale correction (ICM-R2 pilot review,
2026-08-16)" note) — `to-spec`'s copy of the claim was written at the same
N1 pass and was never independently re-checked against the engine's
current state.

Correcting the sentence does not reopen the underlying N1 adjudication A4
fold: that decision rested on "its only stage-level justification was the
§6.5 deterministic-machinery boilerplate, with no additional checkpoint
argument" — a reason independent of whether an execute-stage *kind*
happened to exist in the engine at extraction time. Whether the
write-and-publish operation *should* now become a `kind = "execute"`
stage riding after `10-sketch-seams` (mirroring `65-self-check`'s
pattern) is a real open question this producer does not resolve here —
building it would be new `workflow.toml`/execute-stage content beyond
this reconciliation pass's own scope (mirroring the same parking decision
the `research` stage's own correction already made for its analogous
case). It is parked as a follow-on finding, not silently re-asserted as
settled either way.

**Recommended remediation** (in-place text correction only): replace the
sentence with the corrected framing already used in `research/
00-investigate/CONTEXT.md` — state plainly that a `kind = "execute"`
stage now exists precedent (`repo-to-icm`'s `65-self-check`) and that
whether the write-and-publish helper should become one is an open,
unresolved question, not a settled absence.

## Surviving package design

No stage moves, merges, splits, or renames. The two-stage sequence and
both already-cited N1 behavior units remain correctly placed at PL-4
(package) / PL-5 (each stage) / PL-6 (the one identified helper). The
package requires **in-place content amendment**, not restructuring:

1. Add a `## Bounded judgment` section (per `convention.md` §7.3 /
   `bounded-judgment.md`) to both stage `CONTEXT.md` files, replacing the
   current `## Judgment required` boilerplate with named J2 delegations,
   J1 local choices, and J0 escalation triggers specific to that stage —
   largely a direct restatement of the J-boundary column derived above
   from this package's own existing Behavior contract prose, plus an
   explicit J0 clause naming the tracker/label-vocabulary gap (BU-TS-07)
   for `00-gather-context` or `10-sketch-seams`, whichever the owner
   assigns it to.
2. Add an `## Authority envelope` section to the workflow-level
   `CONTEXT.md` (per `convention.md` §7.2).
3. Add `10-sketch-seams/references/spec-template.md` (the template body,
   copied verbatim from the cited upstream source) and declare it in
   `10-sketch-seams/CONTEXT.md`'s Inputs table (BU-TS-06 remediation).
4. Correct the stale execute-stage claim in `10-sketch-seams/CONTEXT.md`
   line 34 (BU-TS-08 remediation).
5. Leave a citable placeholder at the tracker/label-vocabulary gap
   (BU-TS-07) for the owner's eventual ruling; do not invent its content.

None of these five amendments changes which package owns the behavior, so
none triggers ADR 0013's REHOME/SPLIT/HARVEST draft-and-rehome step
(decision 6; task brief) — no `draft/` directory is produced alongside
this record, matching the validate-and-ship ICM-R2 precedent for the same
reason.

## Inputs and outputs

Inputs: `00-gather-context/CONTEXT.md` correctly declares only
`../CONTEXT.md` (L1, first-stage orientation). `10-sketch-seams/
CONTEXT.md` correctly declares only `../00-gather-context/output/
README.md` (L4, upstream artifact). Both comply with `record-shapes.md`
§1a. The one contract-bearing dependency this pass found **undeclared** is
the fixed template's own content (BU-TS-06/"missing template gap" above)
— once `references/spec-template.md` exists, `10-sketch-seams/CONTEXT.md`
must add it to its Inputs table (Layer 3) per §1a rule 1.

Outputs: `output/README.md` in each stage declares its expected artifact
and disposition. `00-gather-context`'s is `evidence` (Work-branch record
only); `10-sketch-seams`'s is `promote` (workflow deliverable), correctly
reflecting that it is the terminal stage since the demoted
`20-write-and-publish` stage's `promote` disposition was absorbed into it
at N1 adjudication A4. No violation found in the Layer 4 declarations
themselves.

## Review and promotion policy

This package's own content is already `status: published` under
`.sergeant/workflows/` (not a draft) — its structural and provenance
identity does not change. The five remediation items above are ordinary
content edits to an admitted workflow and should go through this
repository's normal review path for workflow content changes, not a new
draft-and-promote cycle, per `docs/icm/convention.md` §2 (the
draft/admitted split governs *new or substantially rewritten* content;
adding required sections and a missing reference file to an
already-admitted stage's `CONTEXT.md` is neither). Per ADR 0013 decision
6, only the promotable form of this change (once actually made) needs
independent review before it lands — this adjudication record itself,
being ICM-R3 evidence, needs its own reviewer step
(`reference/proposal-icm-r-procedure-authority.md` §8.11) before its
findings are treated as settled.

## Alternatives considered

- **Treat the missing spec template (BU-TS-06) as an engine-gap (PL-7)
  claim.** Rejected: nothing about a static template body requires the
  runtime to own a new durable fact; it requires an ordinary Layer 3
  reference file this package's own tree omits. The lower rung (add the
  file) has not been attempted yet, so PL-7 is unreached per the ladder's
  own first-honest-rung rule (proposal §4.8).
- **Silently normalize the tracker/label-vocabulary gap (BU-TS-07) to
  name `td` or GitHub Issues on this producer's own authority.** Rejected:
  `to-spec`'s own upstream source names neither explicitly (unlike
  `to-tickets`'s upstream, which explicitly names `td` — a different
  upstream skill with a different assumed tool), so inventing a specific
  tracker here would not be normalization, it would be new policy content
  authored without evidence. Per `bounded-judgment.md`'s own J0 procedure,
  a producer at J0 states the gap; inventing the answer would be exactly
  the "guess instead of ask" failure the ladder exists to prevent.
- **Leave the stale execute-stage claim (BU-TS-08) uncorrected on the
  theory that the underlying fold decision is unaffected by it.**
  Rejected: a false factual premise inside an admitted package's own
  contract is a defect independent of whether the conclusion it was
  offered to support happens to still hold — the same standard already
  applied to the identical claim in `research/00-investigate` earlier in
  this same wave.
- **Treat `to-spec` and `triage` as requiring reconciliation with each
  other over the `ready-for-agent` disposition in this same pass.**
  Rejected as out of this package's own scope: no source content in
  either package states they are the same procedure or that one supersedes
  the other's requirements at that shared terminal label; recorded as an
  observation for the owner, not adjudicated here as a finding against
  either package.

## Final disposition
STAND

## Validation evidence

- Source-valid: every behavior-unit citation in this package's two stage
  `CONTEXT.md` files was re-read against the **current** content of
  `reference/sergeant-upstream/.agents/skills/to-spec/SKILL.md` (not
  merely the archived provenance) for this pass; all five citations
  (`BU-P4-050` through `BU-P4-054`) still match the live source text and
  line numbers. One source sentence (SKILL.md line 9) was found never
  captured by any behavior unit (BU-TS-07).
- Placement-valid: both stages' already-recorded PL-5 rung ("actor-stage
  (§6.4, judgment)") was independently re-derived from the Placement
  Ladder in this pass and confirmed. The one helper (write-and-publish)
  was independently re-derived as PL-6, matching its already-recorded
  fold at N1 adjudication A4.
- Authority-valid: **not yet** — this is precisely what BU-TS-09 (missing
  `## Authority envelope`/`## Bounded judgment` sections) and BU-TS-07
  (unresolved tracker/label-vocabulary J0) found missing. The package
  cannot be called authority-valid
  (`reference/proposal-icm-r-procedure-authority.md` §9.1 claim 3) until
  those sections are added and BU-TS-07 is either ruled on or explicitly
  deferred by the owner with a citable record.
- Structurally valid: both stage directories, their `output/README.md`
  declarations, and `workflow.toml`'s stage order agree
  (`docs/icm/convention.md` §1 rule 4) — verified directly. One
  structural gap found: the fixed template has no Layer 3 file anywhere
  in the package's own tree (BU-TS-06).
- Execution-valid: **out of scope for this producer pass** — this
  adjudication is a content/citation review, not a re-run of the package;
  `reference/proposal-icm-r-procedure-authority.md` §9.3's
  execution-validation claims (needs_input on a real/scripted J0 case,
  operation without Captain present) remain to be measured separately.
- This record itself is a draft producer output, not yet independently
  reviewed (`docs/adr/0013-icm-r0-owner-rulings.md` decisions 6-7); it
  does not self-promote.
