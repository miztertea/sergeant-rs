# Package adjudication: resolving-merge-conflicts

ICM-R3 full-reconciliation pass, `docs/adr/0013-icm-r0-owner-rulings.md`;
method per `reference/proposal-icm-r-procedure-authority.md` §8; record
shape per `docs/icm/record-shapes.md` §6. Producer pass only — independent
review is a separate step (§8.11 of the proposal; §6.2/6.3 of
`docs/icm/convention.md`) and has not run yet. This record is itself draft
and does not self-promote (ADR 0013 decision 6, decision 7).

Proposal §12.7 names this package among "Review and investigation
workflows" with a hypothesis of "likely STAND after authority and handoff
rewrites, not guaranteed unchanged." This adjudication verifies that
hypothesis directly against the package's current content rather than
assuming it; it is not treated as a starting conclusion (proposal §2.5:
"No current package is reclassified merely because this proposal names it
as suspicious" applies symmetrically to a predicted STAND).

## Original intention

Resolve an in-progress git merge/rebase conflict to completion without
inventing behavior or aborting (`.sergeant/workflows/resolving-merge-
conflicts/CONTEXT.md` "Purpose"; `index.md` description). Promoted into
the N1 reference corpus as candidate **W26**
(`docs/gauntlet/contracts/N1.md`, `docs/icm/promotion-spec-2026-08-11.md`),
with a full behavior-unit citation trail archived at
`docs/gauntlet/promoted-provenance/resolving-merge-conflicts.md`. That
trail traces every current behavior unit to a five-step upstream skill,
`reference/sergeant-upstream/.agents/skills/resolving-merge-conflicts/
SKILL.md` (verified directly against the current file content in this
pass, not assumed from the archived trail): see the current state, find
primary sources for each conflict's intent, resolve each hunk preserving
or trading off intent, run the project's automated checks, finish the
merge/rebase.

This ICM-R3 pass does not re-run the N1 extraction. It applies the
Placement and Bounded-Judgment ladders on top of the already-cited N1
content and checks the package's compliance with ADR 0013's rulings,
following the same method the ICM-R2 pilot already applied to
`validate-and-ship` (`docs/gauntlet/runs/icm-r2/validate-and-ship/
adjudication-draft.md`), which is the closest sibling precedent: another
"review and investigation"-shaped workflow (proposal §12.7) found to
STAND with in-place content amendment, not restructuring.

## Current trigger and outcome

One linear stage list (`workflow.toml`: `10-research-intent`,
`20-resolve-hunks`), one entry point, no branching:

- **Trigger:** a git merge or rebase is in a conflicted state
  (`CONTEXT.md` "Trigger"; both stage `CONTEXT.md` files restate the same
  workflow-level trigger, correctly, since neither introduces a
  different one).
- **Outcome:** the intent behind each conflicting side is researched
  (`10-research-intent`); every hunk is resolved consistent with that
  recorded intent, with no invented behavior; the project's own automated
  checks (typecheck, tests, format) pass; and the merge or rebase is
  carried to completion and committed — never aborted (`20-resolve-hunks`).

`00-assess-state`, `30-validate`, and `40-finish` were demoted at N1
adjudication A4 (`docs/gauntlet/promoted-provenance/resolving-merge-
conflicts.md` "Adjudication A4"): each was classified at extraction as
deterministic machinery (ladder §6.5) with no checkpoint argument beyond
the boilerplate, and folded into the adjacent judgment-bearing stage as a
helper invocation. This ICM-R3 pass independently re-checked that fold
against the Placement Ladder's PL-6 test (§5.8: "repeatable machinery
whose output follows mechanically from declared inputs and whose
invocation does not itself require substantive judgment") and confirms it
holds for all three — see Behavior-unit dispositions below.

## Driver and admission boundary

Driver: **stage actor**, both stages. Admission boundary: **in-work** —
the workflow receives an already-conflicted git state as its trigger; no
Captain-shaped intent beyond "resolve this conflict" is required before
admission, and the workflow's own first stage (`10-research-intent`)
performs the investigative work needed to act, rather than requiring that
investigation to happen in a pre-admission conversation. This passes the
execution-surface test (`docs/icm/convention.md` §2a): "would a human type
`sgt run '<intent>' --workflow resolving-merge-conflicts`?" — yes; a
conflicted merge is itself a sufficient, already-bounded intent, and
nothing about tracing commit/PR/issue history or resolving hunks requires
live dialogue to decide what Work should exist (contrast the PL-2
discriminator, proposal §5.4: "If the procedure's job is to decide what
Work should exist, it cannot itself require an already-existing Work
merely to make that decision" — this package's job is the opposite: it
acts *on* an already-existing conflict, it does not decide whether one
should be resolved).

No PL-0 absorption candidate was found: nothing in the current product
already owns "trace per-side intent, then resolve conflicting hunks
consistent with it." `sgt`'s own git-facing surfaces (worktree/branch
management) do not perform hunk-level conflict resolution.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| BU-P3-045 | `CONTEXT.md`/`index.md` (workflow-level) — resolves an in-progress git merge/rebase conflict to completion, never aborting | PL-4 | J5 (contract-level prohibition, restated at every stage: never invent behavior; never abort) | STAND | `resolving-merge-conflicts` (workflow) |
| BU-P3-046 | `10-research-intent/CONTEXT.md` Helper section (folded from demoted `00-assess-state`, N1 adjudication A4) — establish current merge/rebase state via git history and conflicting files | PL-6 | J5 (governing: an accurate state picture precedes any resolution judgment; mechanical inspection, no substantive choice) | STAND — fold already correctly executed | `10-research-intent` (helper) |
| BU-P3-047 | `10-research-intent/CONTEXT.md` — trace each side's original intent via commit messages, PRs, and issues/tickets before attempting resolution | PL-5 | J2 (delegated: which primary sources to inspect and how to trace intent, by direct analogy to the research workflow's own `BU-P3-042`/`BU-P3-043` J2 grant for source selection) — but **no J0 clause exists** for the case no primary source can be found for one side's intent | STAND, in-place amendment required (add the missing J0 clause) | `10-research-intent` |
| BU-P3-048 | `20-resolve-hunks/CONTEXT.md` — resolve each hunk preserving both intents where possible, or picking the side matching the merge's stated goal with the trade-off recorded when incompatible; never invent behavior; never abort | PL-5 | J5 (governing: never invent new behavior, never `--abort`) + J2 (delegated: preserve-both vs. pick-a-side, and which side matches the stated goal) — but **no J0 clause exists** for the case the two sides are genuinely irreconcilable with no discoverable "stated goal" to break the tie | STAND, in-place amendment required (add the missing J0 clause) | `20-resolve-hunks` |
| BU-P3-049 | `20-resolve-hunks/CONTEXT.md` Helper section (folded from demoted `30-validate`, N1 adjudication A4) — discover and run typecheck, tests, format in order; fix anything the merge broke | PL-6 | J2 (delegated, narrowly: what counts as "anything the merge broke" and how to fix it) bounded by J5 (never invent unrelated behavior, per BU-P3-048's governing constraint, which this helper inherits since it executes inside the same checkpoint) | STAND — fold already correctly executed; the J2 sliver is bounded enough to remain subordinate helper judgment rather than requiring its own checkpoint (convention.md §5 rule 5 test: the decision is "does this specific breakage relate to the merge," not an open design choice) | `20-resolve-hunks` (helper) |
| BU-P3-050 | `20-resolve-hunks/CONTEXT.md` Helper section (folded from demoted `40-finish`, N1 adjudication A4) — stage and commit everything; if rebasing, continue until every commit is rebased | PL-6 | J5 (governing: the operation must be carried to completion, never aborted — BU-P3-045/048's constraint applied at the concluding mechanical step) | STAND — fold already correctly executed | `20-resolve-hunks` (helper) |
| BU-RMC-01 | `CONTEXT.md` (L1) — no `## Authority envelope` section exists | N/A (authoring-format compliance) | J5 (`docs/icm/convention.md` §6.1: every workflow Layer-1 `CONTEXT.md` carries an `## Authority envelope` section) | STAND, in-place amendment required | `CONTEXT.md` |
| BU-RMC-02 | Both stage `CONTEXT.md` files — uniform `## Judgment required` boilerplate paragraph; no stage names J2 decision classes, J1 local choices, or J0 escalation triggers in the required shape | N/A (authoring-format compliance) | J5 (ADR 0013 decision 4 + `docs/icm/convention.md` §6.1: every actor stage's `CONTEXT.md` carries a `## Bounded judgment` section "always present, even when it is only 'inherits workflow envelope unchanged' ... omission is never ambiguous" — governing requirement this package predates and does not yet satisfy) | STAND (package identity correct; in-place content amendment required) | both stage `CONTEXT.md` files |

## The two missing J0 clauses (BU-P3-047, BU-P3-048), full record

Unlike `validate-and-ship`'s push/pr/ci gap (`docs/gauntlet/runs/icm-r2/
validate-and-ship/adjudication-draft.md`, BU-VAS-15), which turned on a
live, unresolved *policy* question only the owner can rule on, both gaps
found here are ordinary missing escalation clauses with no owner-level
policy content to invent — the same shape as the research workflow's own
already-drafted `## Bounded judgment` section (pasted into this Work's own
brief as a worked example), which names "No primary source can be found
for a claim the requester needs answered" as a stated J0 trigger for a
structurally identical research step. This producer therefore states the
recommended clause content directly rather than leaving a bare
placeholder, while still not editing the live package (per this task's
own instruction: STAND packages are documented here, not rewritten
in-place, since that is a distinct, independently-reviewed content
edit — see Review and promotion policy below).

**Gap 1 — `10-research-intent`, BU-P3-047.** Rungs checked: J5 no
governing constraint speaks to this case; J4 no explicit user/Work
decision addresses it; J3 no settled record exists; J2 the stage delegates
*which* sources to inspect, not what to do when none exist. **Conclusion:
J0.** Recommended clause: when no commit message, PR, or issue/ticket can
be found for one side's change, the actor states what evidence was
checked and asks the user for the missing context (or explicit permission
to proceed on the visible diff alone) rather than guessing at unstated
intent — the same shape as `BU-P3-042`'s sibling J0 example in the
research workflow.

**Gap 2 — `20-resolve-hunks`, BU-P3-048.** Rungs checked: J5 governs
"never invent, never abort" but does not resolve *which side to pick* when
both are equally plausible; J4 no explicit user/Work decision names a
tie-breaker; J3 no settled record exists; J2 the stage delegates picking
the side matching "the merge's stated goal," which presupposes a
discoverable goal — it does not delegate authority to invent one when
none exists. **Conclusion: J0.** Recommended clause: when the two sides
are genuinely irreconcilable and no stated goal (from the Work intent, a
commit message, or `10-research-intent`'s own findings) resolves which one
governs, the actor records both intents, states the trade-off, and asks
the user to choose rather than resolving the tie unilaterally — consistent
with BU-P3-048's own already-stated principle that resolution "must never
invent new behavior."

Both recommendations are offered as evidence for the amendment, per
`bounded-judgment.md`'s J0 procedure ("state the actor's recommended
answer when one can be responsibly offered"); they are not authored into
the live package by this producer, since — unlike a bare content gap —
actually landing them is the in-place amendment this record defers to
review (see Surviving package design).

## Surviving package design

No stage moves, merges, splits, or renames. The two-stage sequence, the
already-executed A4 folds, and every already-cited N1 behavior unit remain
correctly placed at PL-4 (package) / PL-5 (each judgment-bearing stage) /
PL-6 (each folded helper). The package requires **in-place content
amendment**, not restructuring:

1. Add a `## Bounded judgment` section (per `docs/icm/convention.md` §6.1
   / `@@bounded-judgment`) to both stage `CONTEXT.md` files, replacing the
   current `## Judgment required` boilerplate with named J2 delegations,
   J1 local choices, and the J0 escalation triggers identified above
   (BU-RMC-02, and the two gap records).
2. Add a `## Authority envelope` section to the workflow-level
   `CONTEXT.md` (per `convention.md` §7.2 / `record-shapes.md` §6),
   stating plainly what the workflow may decide (which side to preserve,
   how to trace intent, what counts as merge-induced breakage), what it
   may not decide (inventing behavior, aborting, resolving a genuine tie
   without asking), and that there are no Captain/human gates beyond the
   two J0 triggers above.
3. No dangling references, no orphaned helper folds, and no undeclared
   Inputs-table dependency were found (see Inputs and outputs below) — 
   unlike `validate-and-ship`, this package has no third remediation item
   of that kind.

Neither amendment changes which package owns the behavior, so neither
triggers ADR 0013's REHOME/SPLIT/HARVEST draft-and-rehome step (decision
6; task brief). They are recorded here as the concrete remediation this
adjudication found, for the owner/reviewer to schedule — the same
disposition validate-and-ship's ICM-R2 pass reached for an analogous gap.

## Inputs and outputs

Inputs: both stages already comply with `record-shapes.md` §1a.
`10-research-intent` inputs only `../CONTEXT.md` (L1, first stage).
`20-resolve-hunks` inputs `../10-research-intent/output/README.md` (L4,
the upstream artifact it consumes) — correctly an L4 input from an
earlier stage, not a later one (`convention.md` §1a rule 3). No
contract-bearing dependency was found undeclared, and no `@@`-style
reference exists in this package to check for dangling targets (unlike
`validate-and-ship`'s `route-review-findings` reference).

Outputs: `10-research-intent/output/README.md` declares an `evidence`
artifact — Work-branch record only, correctly reflecting that it feeds
the next stage rather than being a workflow deliverable itself.
`20-resolve-hunks/output/README.md` declares a `promote` artifact,
correctly reflecting that it is the workflow's terminal stage since
`30-validate` and `40-finish` were folded in (its own note already
records this reasoning). No violation found in either Layer 4 declaration.

## Review and promotion policy

This package's own content is already `status: published` under
`.sergeant/workflows/` (not a draft) — its structural and provenance
identity does not change. The remediation items above are ordinary
content edits to an admitted workflow and should go through this
repository's normal review path for workflow content changes, not a new
draft-and-promote cycle (`docs/icm/convention.md` §2 governs *new or
substantially rewritten* content; adding required sections to an
already-admitted stage's `CONTEXT.md` is neither). Per ADR 0013 decision
6, only the promotable form of this change (once actually made) needs
independent review before it lands — this adjudication record itself,
being ICM-R3 producer output, needs its own independent-review step
(`reference/proposal-icm-r-procedure-authority.md` §8.11) before its
findings are treated as settled.

## Alternatives considered

- **REHOME or SPLIT the package**, on the theory that "trace intent from
  commit messages, PRs, and issues" resembles the `research` workflow's
  investigative work closely enough to fold into it. Rejected: the two
  packages share a reusable *technique* (primary-source tracing) but not
  a durable outcome — `research` produces a cited findings document for a
  requester; `resolving-merge-conflicts` produces a resolved, committed
  merge. Harvesting the shared technique into a `.sergeant/common/
  contexts/` shared method is plausible future work if a third consumer
  appears (`convention.md` §5 rule 3's reuse-not-convenience test), but
  one additional structural similarity does not by itself justify
  merging two packages with different bounded outcomes and different
  trigger conditions (proposal §8.8: clustering is by behavioral contract
  and durable outcome, not surface resemblance).
- **Treat the two J0 gaps as engine-gap (PL-7) claims.** Rejected: neither
  requires the runtime to own a new durable fact; both require the
  package's own content to state an escalation boundary it currently
  omits. Lower rungs (a stage-local J-clause) had not been attempted
  before this pass, so PL-7 is unreached per the ladder's own
  first-honest-rung rule (proposal §4.8).
- **Silently author the two J-clauses directly into the live
  `CONTEXT.md` files on this producer's own authority.** Rejected: per
  this task's own instruction, a STAND disposition's remediation is
  recorded for independent review and scheduling, not landed by the
  producer — the same boundary `validate-and-ship`'s ICM-R2 pass drew
  for its own remediation items, and consistent with proposal §4.9 ("a
  producer does not independently promote its own output").
- **Treat `00-assess-state`/`30-validate`/`40-finish`'s N1-era folds as
  needing re-litigation from scratch.** Rejected: this pass independently
  re-derived the PL-6 classification for all three against the current
  ladder text (§5.8) rather than merely trusting the archived N1
  adjudication, and reached the same conclusion — re-opening a fold that
  still holds under re-derivation would violate §6.3/§6.6 of
  `bounded-judgment.md`'s own J3 principle ("do not reopen settled intent
  merely because another choice is possible").

## Final disposition
STAND

## Validation evidence

- Source-valid: every current behavior-unit citation in this package's two
  stage `CONTEXT.md` files was read in full and independently re-traced to
  the current content of `reference/sergeant-upstream/.agents/skills/
  resolving-merge-conflicts/SKILL.md` (read directly in this pass, not
  assumed from the archived provenance file) — the archived
  `docs/gauntlet/promoted-provenance/resolving-merge-conflicts.md` trail
  matches the current upstream source exactly; no drift found.
- Placement-valid: every stage's PL-5 rung and every helper's PL-6 rung
  was independently re-derived from the Placement Ladder in this pass
  (§5.7's reimplementation test for the two judgment-bearing stages;
  §5.8's mechanical-machinery test for the three folded helpers), not
  merely copied from the package's own stage table.
- Authority-valid: **not yet** — this is precisely what BU-RMC-01/02 and
  the two J0 gap records found missing. The package cannot be called
  authority-valid (`reference/proposal-icm-r-procedure-authority.md` §9.1
  claim 3) until the amendments under "Surviving package design" land.
- Structurally valid: both stage directories, their `output/README.md`
  declarations, and `workflow.toml`'s stage order agree
  (`docs/icm/convention.md` §1 rule 4) — verified directly.
- Execution-valid: **out of scope for this producer pass** — this
  adjudication is a content/citation review, not a re-run of the
  package; `reference/proposal-icm-r-procedure-authority.md` §9.3's
  execution-validation claims (needs_input on a real/scripted J0 case,
  operation without Captain present) remain to be measured separately,
  once the two J0 clauses above actually exist in the package content to
  exercise.
- This record itself is a draft producer output, not yet independently
  reviewed (`docs/adr/0013-icm-r0-owner-rulings.md` decisions 6-7); it
  does not self-promote.
